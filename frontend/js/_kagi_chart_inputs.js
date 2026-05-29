// Kagi-chart helpers.
//
// Backend body: { closes: number[], reversal: number, kind: 'absolute'|'pct' }
// Returns: KagiLine[] = { direction: 'Up'|'Down', anchor_price, end_price, source_index }
//
// Algorithm: trend-following line drawn until price reverses by `reversal`
// against the running extreme. Reversal is either an absolute price or
// a percentage of the running extreme.

import { t } from './i18n.js';

export const KINDS = ['absolute', 'pct'];

export const DEFAULT_INPUTS = {
    closes: [],
    reversal: 2.0,
    kind: 'absolute',
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                return t('view.kagi_chart.validate.closes_array');
    if (input.closes.some(v => !Number.isFinite(v))) return t('view.kagi_chart.validate.closes_finite');
    if (input.closes.some(v => v <= 0))              return t('view.kagi_chart.validate.closes_positive');
    if (!Number.isFinite(input.reversal))            return t('view.kagi_chart.validate.reversal_finite');
    if (input.reversal <= 0)                         return t('view.kagi_chart.validate.reversal_positive');
    if (!KINDS.includes(input.kind))                 return t('view.kagi_chart.validate.kind', { list: KINDS.join(', ') });
    return null;
}

export function buildBody(input) {
    return {
        closes:   input.closes,
        reversal: input.reversal,
        kind:     input.kind,
    };
}

// Pure-JS mirror of crates/traderview-core/src/kagi_chart.rs::compute.
// Returns lines with the SAME enum strings as Rust ('Up'/'Down').
export function localCompute(closes, reversal, kind) {
    const out = [];
    if (!Array.isArray(closes) || closes.length === 0) return out;
    if (!Number.isFinite(reversal) || reversal <= 0) return out;
    for (const v of closes) {
        if (!Number.isFinite(v) || v <= 0) return out;
    }
    const threshold = (anchor) => kind === 'pct' ? anchor * reversal / 100 : reversal;
    let direction = null;
    let anchor = closes[0];
    let extreme = closes[0];
    let start_idx = 0;
    for (let i = 1; i < closes.length; i++) {
        const px = closes[i];
        if (direction === null) {
            const delta = px - anchor;
            if (Math.abs(delta) >= threshold(anchor)) {
                direction = delta > 0 ? 'Up' : 'Down';
                extreme = px;
            }
        } else if (direction === 'Up') {
            if (px > extreme) {
                extreme = px;
            } else if (extreme - px >= threshold(extreme)) {
                out.push({ direction: 'Up', anchor_price: anchor, end_price: extreme, source_index: start_idx });
                anchor = extreme;
                extreme = px;
                start_idx = i;
                direction = 'Down';
            }
        } else {
            // 'Down'
            if (px < extreme) {
                extreme = px;
            } else if (px - extreme >= threshold(extreme)) {
                out.push({ direction: 'Down', anchor_price: anchor, end_price: extreme, source_index: start_idx });
                anchor = extreme;
                extreme = px;
                start_idx = i;
                direction = 'Up';
            }
        }
    }
    if (direction !== null) {
        out.push({ direction, anchor_price: anchor, end_price: extreme, source_index: start_idx });
    }
    return out;
}

// Parse comma/whitespace-separated price series; ignores # comments + blanks.
export function parseCloses(blob) {
    const out = { closes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const v = Number(tokens[i]);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        if (v <= 0) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" must be > 0` });
            continue;
        }
        out.closes.push(v);
    }
    return out;
}

// Trend verdict from line directions — biased toward latest movement.
export function trendBadge(lines) {
    if (!Array.isArray(lines) || lines.length === 0) return { key: 'view.kagi.badge.flat', cls: '' };
    const last = lines[lines.length - 1];
    if (last.direction === 'Up')   return { key: 'view.kagi.badge.uptrend',   cls: 'pos' };
    if (last.direction === 'Down') return { key: 'view.kagi.badge.downtrend', cls: 'neg' };
    return { key: 'view.kagi.badge.flat', cls: '' };
}

// Yang/yin classification — Up line crossing prior peak = yang (thick),
// Down line crossing prior trough = yin (thin).
export function classifyYangYin(lines) {
    if (!Array.isArray(lines) || lines.length === 0) return [];
    let priorPeak = -Infinity;
    let priorTrough = Infinity;
    const out = [];
    for (const l of lines) {
        let kind = 'neutral';
        if (l.direction === 'Up') {
            if (l.end_price > priorPeak) kind = 'yang';
            priorPeak = Math.max(priorPeak, l.end_price);
        } else if (l.direction === 'Down') {
            if (l.end_price < priorTrough) kind = 'yin';
            priorTrough = Math.min(priorTrough, l.end_price);
        }
        out.push(kind);
    }
    return out;
}

// Aggregate stats about the run.
export function summarize(lines) {
    if (!Array.isArray(lines) || lines.length === 0) {
        return { count: 0, ups: 0, downs: 0, avg_up: NaN, avg_down: NaN, last_dir: null };
    }
    let ups = 0, downs = 0, sumUp = 0, sumDown = 0;
    for (const l of lines) {
        const move = Math.abs(l.end_price - l.anchor_price);
        if (l.direction === 'Up')   { ups++;   sumUp += move; }
        if (l.direction === 'Down') { downs++; sumDown += move; }
    }
    return {
        count: lines.length,
        ups, downs,
        avg_up: ups > 0 ? sumUp / ups : NaN,
        avg_down: downs > 0 ? sumDown / downs : NaN,
        last_dir: lines[lines.length - 1].direction,
    };
}

// Synthetic demos.
export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend':       return { closes: range(100, 120, 1),                reversal: 1, kind: 'absolute' };
        case 'downtrend':     return { closes: range(120, 100, -1),               reversal: 1, kind: 'absolute' };
        case 'choppy':        return { closes: chop(100, 30, 3),                  reversal: 2, kind: 'absolute' };
        case 'breakout':      return { closes: [...flat(100, 20, 0.2), ...range(100, 130, 1.5)],
                                       reversal: 1.5, kind: 'absolute' };
        case 'flat':          return { closes: flat(100, 50, 0.05),               reversal: 1, kind: 'absolute' };
        case 'pct-reversal':  return { closes: range(100, 130, 0.8),              reversal: 1, kind: 'pct' };
        case 'reversal-storm':return { closes: zigzag(100, 30, 5),                reversal: 3, kind: 'absolute' };
        case 'gentle-bull':   return { closes: range(50, 60, 0.25),               reversal: 0.5, kind: 'absolute' };
        default:              return makeDemoInput('uptrend');
    }
}

function range(start, end, step) {
    const out = [];
    if (step > 0) for (let v = start; v <= end + 1e-9; v += step) out.push(round(v));
    else          for (let v = start; v >= end - 1e-9; v += step) out.push(round(v));
    return out;
}

function flat(price, n, jitter) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(round(price + Math.sin(i * 0.4) * jitter));
    return out;
}

function chop(center, n, amp) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(round(center + Math.sin(i * 0.7) * amp + Math.cos(i * 1.3) * amp * 0.6));
    return out;
}

function zigzag(center, n, amp) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(round(center + (i % 2 === 0 ? amp : -amp)));
    return out;
}

function round(v) { return Math.round(v * 10000) / 10000; }

// Convert line into [x, y] polyline points (anchor → end at same x for vertical leg).
// Returns flat-x stepped polyline: anchor → (same x, end) → next anchor.
export function linesToPolyline(lines) {
    if (!Array.isArray(lines) || lines.length === 0) return { xs: [], ys: [] };
    const xs = [];
    const ys = [];
    let cursor = 0;
    for (const l of lines) {
        xs.push(cursor); ys.push(l.anchor_price);
        xs.push(cursor); ys.push(l.end_price);
        cursor++;
    }
    return { xs, ys };
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtMove(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function dirLabelKey(dir) {
    if (dir === 'Up')   return 'view.kagi.dir.up';
    if (dir === 'Down') return 'view.kagi.dir.down';
    return 'view.kagi.dir.unknown';
}

export function yangYinLabelKey(yy) {
    if (yy === 'yang') return 'view.kagi.yy.yang';
    if (yy === 'yin')  return 'view.kagi.yy.yin';
    return 'view.kagi.yy.neutral';
}
