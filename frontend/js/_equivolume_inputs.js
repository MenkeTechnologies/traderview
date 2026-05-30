// Equivolume bars (Richard Arms) helpers.
//
// Backend body: { bars: [{high, low, volume}, ...], total_width }
// Returns: { widths: number[], kinds: ('normal'|'narrow'|'wide'|'power')[],
//   avg_volume, avg_range, total_width }
//
// width_i = volume_i / Σ volume × total_width  (chart-width normalization)
// kind:
//   Narrow:  vol ≤ 0.5 × avg
//   Wide:    vol > 1.5 × avg
//   Power:   vol > 1.5 × avg AND range > 1.5 × avg range
//   Normal:  otherwise

import { t } from './i18n.js';

export const KINDS = ['normal', 'narrow', 'wide', 'power'];

export const DEFAULT_INPUTS = {
    bars: [],
    total_width: 1000,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                              return t('view.equivolume.validate.bars_array');
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || typeof b !== 'object')                         return t('view.equivolume.validate.bar_object', { i });
        if (!Number.isFinite(b.high))                            return t('view.equivolume.validate.high_finite', { i });
        if (!Number.isFinite(b.low))                             return t('view.equivolume.validate.low_finite', { i });
        if (!Number.isFinite(b.volume))                          return t('view.equivolume.validate.vol_finite', { i });
        if (b.volume < 0)                                        return t('view.equivolume.validate.vol_negative', { i });
        if (b.high < b.low)                                      return t('view.equivolume.validate.high_ge_low', { i });
    }
    if (!Number.isFinite(input.total_width))                     return t('view.equivolume.validate.width_finite');
    if (input.total_width <= 0)                                  return t('view.equivolume.validate.width_positive');
    return null;
}

export function buildBody(input) {
    return {
        bars:        input.bars.map(b => ({ high: b.high, low: b.low, volume: b.volume })),
        total_width: input.total_width,
    };
}

// Pure-JS mirror of crates/traderview-core/src/equivolume_bars.rs::compute.
// Returns same shape on failure: all-zero widths + 'normal' kinds + zero stats.
export function localCompute(bars, total_width) {
    const n = Array.isArray(bars) ? bars.length : 0;
    const empty = {
        widths: new Array(n).fill(0),
        kinds: new Array(n).fill('normal'),
        avg_volume: 0,
        avg_range: 0,
        total_width: 0,
    };
    if (n === 0) return empty;
    if (!Number.isFinite(total_width) || total_width <= 0) return empty;
    for (const b of bars) {
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.volume)
            || b.volume < 0 || b.high < b.low) return empty;
    }
    let total_vol = 0;
    let total_range = 0;
    for (const b of bars) { total_vol += b.volume; total_range += (b.high - b.low); }
    if (total_vol <= 0) return empty;
    const widths = bars.map(b => b.volume / total_vol * total_width);
    const avg_vol = total_vol / n;
    const avg_range = total_range / n;
    const kinds = bars.map(b => {
        const range = b.high - b.low;
        const big_vol = b.volume > avg_vol * 1.5;
        const big_range = avg_range > 0 && range > avg_range * 1.5;
        if (big_vol && big_range) return 'power';
        if (big_vol)              return 'wide';
        if (b.volume <= avg_vol * 0.5) return 'narrow';
        return 'normal';
    });
    return {
        widths,
        kinds,
        avg_volume: avg_vol,
        avg_range,
        total_width,
    };
}

// Parse "high low volume" per line (same shape as volume-at-price).
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 3) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.expected_high_low_volume') });
            continue;
        }
        const [high, low, volume] = toks.map(Number);
        if (![high, low, volume].every(Number.isFinite)) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.non_finite_token') });
            continue;
        }
        if (high < low) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.high_lt_low') });
            continue;
        }
        if (volume < 0) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.volume_must_be_non_negative') });
            continue;
        }
        out.bars.push({ high, low, volume });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.volume}`).join('\n');
}

// Tally + verdicts.
export function summarize(report) {
    if (!report || !Array.isArray(report.kinds) || report.kinds.length === 0) {
        return { count: 0, narrow: 0, normal: 0, wide: 0, power: 0,
                 max_width: NaN, min_width: NaN };
    }
    let narrow = 0, normal = 0, wide = 0, power = 0;
    for (const k of report.kinds) {
        if (k === 'narrow')      narrow++;
        else if (k === 'wide')   wide++;
        else if (k === 'power')  power++;
        else                     normal++;
    }
    let mxW = -Infinity, mnW = Infinity;
    for (const w of report.widths) {
        if (Number.isFinite(w)) {
            if (w > mxW) mxW = w;
            if (w < mnW) mnW = w;
        }
    }
    return {
        count: report.kinds.length,
        narrow, normal, wide, power,
        max_width: Number.isFinite(mxW) ? mxW : NaN,
        min_width: Number.isFinite(mnW) ? mnW : NaN,
    };
}

// Activity verdict — fraction of bars showing strong conviction.
export function convictionBadge(s) {
    if (!s || !Number.isFinite(s.count) || s.count === 0)
        return { key: 'view.equivol.badge.unknown', cls: '' };
    const pf = (s.power + s.wide) / s.count;
    if (s.power >= 1 && (s.power / s.count) >= 0.10)  return { key: 'view.equivol.badge.power_run', cls: 'pos' };
    if (pf >= 0.30) return { key: 'view.equivol.badge.heavy',   cls: 'neg' };
    if (pf >= 0.10) return { key: 'view.equivol.badge.normal',  cls: '' };
    return { key: 'view.equivol.badge.quiet', cls: '' };
}

// Trend verdict from last bar's classification.
export function lastBadge(kind) {
    if (kind === 'power')  return { key: 'view.equivol.last.power',  cls: 'neg' };
    if (kind === 'wide')   return { key: 'view.equivol.last.wide',   cls: 'neg' };
    if (kind === 'narrow') return { key: 'view.equivol.last.narrow', cls: '' };
    if (kind === 'normal') return { key: 'view.equivol.last.normal', cls: '' };
    return { key: 'view.equivol.last.unknown', cls: '' };
}

// Demos.
export function makeDemoInput(kind = 'normal-mix') {
    switch (kind) {
        case 'normal-mix': {
            const bars = [];
            for (let i = 0; i < 20; i++) bars.push(b(101 + Math.sin(i * 0.3), 99 + Math.sin(i * 0.3), 1000 + (i * 17) % 300));
            return { bars, total_width: 1000 };
        }
        case 'power-spike': {
            const bars = [];
            for (let i = 0; i < 9; i++) bars.push(b(101, 99, 1000));
            bars.push(b(115, 95, 5000));    // 5× avg vol, ~10× avg range
            return { bars, total_width: 1000 };
        }
        case 'wide-only': {
            const bars = [];
            for (let i = 0; i < 9; i++) bars.push(b(101, 99, 1000));
            bars.push(b(101, 99, 5000));    // 5× vol but same range
            return { bars, total_width: 1000 };
        }
        case 'narrow-spike': {
            const bars = [];
            for (let i = 0; i < 9; i++) bars.push(b(101, 99, 2000));
            bars.push(b(101, 99, 100));     // 5% of avg
            return { bars, total_width: 1000 };
        }
        case 'flat-volume': {
            const bars = [];
            for (let i = 0; i < 10; i++) bars.push(b(101, 99, 1000));
            return { bars, total_width: 1000 };
        }
        case 'climax-day': {
            // Quiet baseline + power finale.
            const bars = [];
            for (let i = 0; i < 18; i++) bars.push(b(101, 100.5, 800));
            bars.push(b(105, 100, 6000));
            bars.push(b(108, 102, 7500));
            return { bars, total_width: 1000 };
        }
        case 'mixed-kinds': {
            // Designed to have all 4 kinds present.
            return { bars: [
                b(101, 99, 1000),     // normal baseline
                b(101, 99, 1000),
                b(101, 99, 1000),
                b(101, 99, 1000),
                b(101, 99, 1000),
                b(101, 99, 1000),
                b(101, 99, 1000),
                b(101, 99, 1000),
                b(101, 99, 100),      // narrow
                b(101, 99, 4000),     // wide
                b(115, 95, 5000),     // power
            ], total_width: 1000 };
        }
        case 'noisy-walk': {
            const bars = [];
            let state = BigInt(7919);
            for (let i = 0; i < 50; i++) {
                state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
                const u = Number(state >> 32n) / 0xFFFFFFFF;
                const high = 101 + u * 2;
                const low  = 99  - u * 2;
                const vol  = 800 + u * 4000;
                bars.push(b(round(high), round(low), round(vol)));
            }
            return { bars, total_width: 1000 };
        }
        default: return makeDemoInput('normal-mix');
    }
}

function b(high, low, volume) { return { high, low, volume }; }
function round(v) { return Math.round(v * 10000) / 10000; }

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtNum(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtVol(v) {
    if (!Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e6) return (v / 1e6).toFixed(2) + 'M';
    if (Math.abs(v) >= 1e3) return (v / 1e3).toFixed(2) + 'k';
    return v.toFixed(0);
}

export function kindLabelKey(kind) {
    if (kind === 'power')  return 'view.equivol.kind.power';
    if (kind === 'wide')   return 'view.equivol.kind.wide';
    if (kind === 'narrow') return 'view.equivol.kind.narrow';
    if (kind === 'normal') return 'view.equivol.kind.normal';
    return 'view.equivol.kind.unknown';
}

export function kindCls(kind) {
    if (kind === 'power' || kind === 'wide') return 'neg';
    if (kind === 'narrow') return '';
    return '';
}
