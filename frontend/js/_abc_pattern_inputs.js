// ABC correction pattern detector helpers.
//
// Backend body: { swings: SwingPoint[], config: AbcConfig }
//   SwingPoint = { index: number, price: number, kind: 'high'|'low' }
//   AbcConfig  = { min_b_retrace: f64, max_b_retrace: f64, min_c_extension: f64 }
// Returns:      { events: AbcEvent[] }

import { t } from './i18n.js';

export const DEFAULT_MIN_B = 0.382;
export const DEFAULT_MAX_B = 0.618;
export const DEFAULT_MIN_C_EXT = 1.0;
export const MIN_SWINGS = 3;
export const MAX_SWINGS = 1000;

export const DEFAULT_INPUTS = {
    swings: [
        { index: 0,  price: 150, kind: 'high' },
        { index: 10, price: 130, kind: 'low'  },
        { index: 20, price: 155, kind: 'high' },
    ],
    min_b_retrace: DEFAULT_MIN_B,
    max_b_retrace: DEFAULT_MAX_B,
    min_c_extension: DEFAULT_MIN_C_EXT,
};

export function validateInputs(input) {
    if (!Array.isArray(input.swings))                       return t('view.abc_pattern.validate.swings_array');
    if (input.swings.length < MIN_SWINGS)                   return t('view.abc_pattern.validate.swings_min', { n: MIN_SWINGS });
    if (input.swings.length > MAX_SWINGS)                   return t('view.abc_pattern.validate.swings_max', { n: MAX_SWINGS });
    for (let i = 0; i < input.swings.length; i++) {
        const s = input.swings[i];
        if (!s || typeof s !== 'object')                    return t('view.abc_pattern.validate.swing_object', { i });
        if (!Number.isInteger(s.index) || s.index < 0)      return t('view.abc_pattern.validate.swing_index', { i });
        if (typeof s.price !== 'number' || !Number.isFinite(s.price))
                                                              return t('view.abc_pattern.validate.swing_price', { i });
        if (s.kind !== 'high' && s.kind !== 'low')          return t('view.abc_pattern.validate.swing_kind', { i });
    }
    if (!Number.isFinite(input.min_b_retrace) || input.min_b_retrace < 0 || input.min_b_retrace > 1)
                                                              return t('view.abc_pattern.validate.min_b');
    if (!Number.isFinite(input.max_b_retrace) || input.max_b_retrace < 0 || input.max_b_retrace > 1)
                                                              return t('view.abc_pattern.validate.max_b');
    if (input.min_b_retrace > input.max_b_retrace)          return t('view.abc_pattern.validate.min_gt_max');
    if (!Number.isFinite(input.min_c_extension) || input.min_c_extension <= 0)
                                                              return t('view.abc_pattern.validate.min_c');
    return null;
}

export function buildBody(input) {
    return {
        swings: input.swings.map(s => ({ index: s.index, price: s.price, kind: s.kind })),
        config: {
            min_b_retrace:    input.min_b_retrace,
            max_b_retrace:    input.max_b_retrace,
            min_c_extension:  input.min_c_extension,
        },
    };
}

// Pure-JS mirror of crates/traderview-core/src/abc_pattern.rs::detect.
export function localDetect(swings, cfg) {
    const out = { events: [] };
    if (!Array.isArray(swings) || swings.length < 3) return out;
    if (!(cfg.min_b_retrace >= 0 && cfg.min_b_retrace <= 1)) return out;
    if (!(cfg.max_b_retrace >= 0 && cfg.max_b_retrace <= 1)) return out;
    if (cfg.min_b_retrace > cfg.max_b_retrace) return out;
    if (!(cfg.min_c_extension > 0)) return out;
    for (let i = 0; i + 2 < swings.length; i++) {
        const a = swings[i], b = swings[i + 1], c = swings[i + 2];
        let bias = null;
        if (a.kind === 'high' && b.kind === 'low' && c.kind === 'high')      bias = 'bearish';
        else if (a.kind === 'low' && b.kind === 'high' && c.kind === 'low') bias = 'bullish';
        else continue;
        const ab = Math.abs(b.price - a.price);
        const bc = Math.abs(c.price - b.price);
        if (!(ab > 0 && bc > 0)) continue;
        const b_retrace = ab / Math.max(ab, bc);
        const c_ext = bc / ab;
        if (c_ext < cfg.min_c_extension) continue;
        const b_proxy = ab / (ab + bc);
        if (b_proxy < cfg.min_b_retrace || b_proxy > cfg.max_b_retrace) continue;
        out.events.push({
            a_idx: a.index, b_idx: b.index, c_idx: c.index, bias,
            ab_length: ab, bc_length: bc,
            b_retrace_pct: b_retrace, c_extension_ratio: c_ext,
        });
    }
    return out;
}

export function statusBadge(report) {
    if (!report || !Array.isArray(report.events))                 return { key: 'view.abc.status.unknown',    cls: '' };
    if (report.events.length === 0)                                return { key: 'view.abc.status.none',       cls: '' };
    const last = report.events[report.events.length - 1];
    return last.bias === 'bullish'
        ? { key: 'view.abc.status.bullish', cls: 'pos' }
        : { key: 'view.abc.status.bearish', cls: 'neg' };
}

export function biasMixBadge(report) {
    if (!report || !report.events.length) return { key: 'view.abc.mix.unknown', cls: '' };
    let bull = 0, bear = 0;
    for (const ev of report.events) (ev.bias === 'bullish' ? ++bull : ++bear);
    if (bull > 0 && bear === 0) return { key: 'view.abc.mix.all_bull',   cls: 'pos' };
    if (bear > 0 && bull === 0) return { key: 'view.abc.mix.all_bear',   cls: 'neg' };
    if (bull > bear)            return { key: 'view.abc.mix.bull_lean',  cls: 'pos' };
    if (bear > bull)            return { key: 'view.abc.mix.bear_lean',  cls: 'neg' };
    return { key: 'view.abc.mix.balanced', cls: '' };
}

export function strengthBadge(ev) {
    if (!ev) return { key: 'view.abc.strength.unknown', cls: '' };
    const r = ev.c_extension_ratio;
    if (r >= 2.0)  return { key: 'view.abc.strength.very_strong', cls: 'pos' };
    if (r >= 1.5)  return { key: 'view.abc.strength.strong',      cls: 'pos' };
    if (r >= 1.0)  return { key: 'view.abc.strength.standard',    cls: '' };
    return { key: 'view.abc.strength.weak', cls: 'neg' };
}

// Parse blob: one swing per line, "index price kind" (kind = high|low|h|l).
export function parseSwingsBlob(blob) {
    const out = { swings: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length !== 3) {
            out.errors.push({ line_no: i + 1, message: 'expected 3 tokens: index price kind' });
            continue;
        }
        const idx = Number(parts[0]);
        const price = Number(parts[1]);
        const kRaw = parts[2].toLowerCase();
        const kind = (kRaw === 'h' || kRaw === 'high') ? 'high'
                   : (kRaw === 'l' || kRaw === 'low')  ? 'low'  : null;
        if (!Number.isInteger(idx) || idx < 0) {
            out.errors.push({ line_no: i + 1, message: 'index must be non-negative integer' });
            continue;
        }
        if (!Number.isFinite(price)) {
            out.errors.push({ line_no: i + 1, message: 'price not finite' });
            continue;
        }
        if (!kind) {
            out.errors.push({ line_no: i + 1, message: 'kind must be high or low' });
            continue;
        }
        out.swings.push({ index: idx, price, kind });
    }
    return out;
}

export function swingsToBlob(swings) {
    return swings.map(s => `${s.index} ${s.price} ${s.kind}`).join('\n');
}

export function summarizeSwings(swings) {
    if (!Array.isArray(swings) || swings.length === 0) {
        return { count: 0, highs: 0, lows: 0, min_price: NaN, max_price: NaN, span: NaN };
    }
    let highs = 0, lows = 0, mn = Infinity, mx = -Infinity;
    for (const s of swings) {
        if (s.kind === 'high') highs++; else lows++;
        if (s.price < mn) mn = s.price;
        if (s.price > mx) mx = s.price;
    }
    return { count: swings.length, highs, lows, min_price: mn, max_price: mx, span: mx - mn };
}

export function makeDemoInput(kind = 'bearish-classic') {
    const base = { min_b_retrace: DEFAULT_MIN_B, max_b_retrace: DEFAULT_MAX_B, min_c_extension: DEFAULT_MIN_C_EXT };
    switch (kind) {
        case 'bearish-classic':
            return { ...base, swings: [
                { index: 0,  price: 150, kind: 'high' },
                { index: 10, price: 130, kind: 'low'  },
                { index: 20, price: 155, kind: 'high' },
            ] };
        case 'bullish-classic':
            return { ...base, swings: [
                { index: 0,  price: 100, kind: 'low'  },
                { index: 10, price: 120, kind: 'high' },
                { index: 20, price:  95, kind: 'low'  },
            ] };
        case 'weak-c':
            return { ...base, swings: [
                { index: 0,  price: 150, kind: 'high' },
                { index: 10, price: 130, kind: 'low'  },
                { index: 20, price: 135, kind: 'high' },
            ] };
        case 'non-alternating':
            return { ...base, swings: [
                { index: 0,  price: 100, kind: 'high' },
                { index: 10, price: 120, kind: 'high' },
                { index: 20, price:  95, kind: 'high' },
            ] };
        case 'multi-events': {
            // Two ABCs back to back (5 swings → 3 windows; first/last may qualify, middle is mixed).
            return { ...base, swings: [
                { index: 0,  price: 150, kind: 'high' },
                { index: 10, price: 130, kind: 'low'  },
                { index: 20, price: 155, kind: 'high' },
                { index: 30, price: 125, kind: 'low'  },
                { index: 40, price: 152, kind: 'high' },
            ] };
        }
        case 'very-strong': {
            // c_extension > 2.0 — needs a wider b band to pass b_proxy filter.
            return {
                min_b_retrace: 0.25, max_b_retrace: 0.75, min_c_extension: DEFAULT_MIN_C_EXT,
                swings: [
                    { index: 0,  price: 200, kind: 'high' },
                    { index: 10, price: 180, kind: 'low'  },
                    { index: 20, price: 230, kind: 'high' },
                ],
            };
        }
        case 'zero-leg':
            return { ...base, swings: [
                { index: 0,  price: 100, kind: 'high' },
                { index: 10, price: 100, kind: 'low'  },
                { index: 20, price: 110, kind: 'high' },
            ] };
        case 'tight-config':
            return { ...base,
                min_b_retrace: 0.45, max_b_retrace: 0.55,
                swings: [
                    { index: 0,  price: 150, kind: 'high' },
                    { index: 10, price: 130, kind: 'low'  },
                    { index: 20, price: 152, kind: 'high' },   // ab=20, bc=22, b_proxy=20/42≈0.476 ∈ [0.45,0.55]
                ] };
        default: return makeDemoInput('bearish-classic');
    }
}

// ── formatters ──
export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
export function fmtRatio(v, d = 3) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
export function fmtPct(v, d = 1) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}
export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
