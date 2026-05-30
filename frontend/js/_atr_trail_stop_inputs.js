// ATR Trailing Stop helpers (raw multi-of-ATR with ratchet).
//
// Backend body: { bars: Bar[], period: usize, multiplier: f64 }
//   where Bar = { high, low, close }
// Returns: {
//   long_stop: (number|null)[], short_stop: (number|null)[],
//   period, multiplier
// }

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 14;
export const DEFAULT_MULTIPLIER = 3.0;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
    multiplier: DEFAULT_MULTIPLIER,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                  return t('view.atr_trail_stop.validate.bars_array');
    if (!Number.isInteger(input.period))             return t('view.atr_trail_stop.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                      return t('view.atr_trail_stop.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isFinite(input.multiplier) || input.multiplier <= 0)
                                                      return t('view.atr_trail_stop.validate.multiplier');
    if (input.bars.length < input.period + 1)         return t('view.atr_trail_stop.validate.bars_min', { n: input.period + 1 });
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || !Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close))
                                                      return t('view.atr_trail_stop.validate.bar_finite', { i });
        if (b.high < b.low)                           return t('view.atr_trail_stop.validate.high_lt_low', { i });
        if (b.close < b.low || b.close > b.high)      return t('view.atr_trail_stop.validate.close_outside', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low, close: b.close })),
        period: input.period,
        multiplier: input.multiplier,
    };
}

// Pure-JS mirror of crates/traderview-core/src/atr_trailing_stop.rs::compute.
export function localCompute(bars, period, multiplier) {
    const n = bars.length;
    const report = {
        long_stop:  new Array(n).fill(null),
        short_stop: new Array(n).fill(null),
        period, multiplier,
    };
    if (period < 2 || !Number.isFinite(multiplier) || multiplier <= 0 || n < period + 1) return report;
    for (const b of bars) {
        if (!b || !Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close)) return report;
    }
    const tr = new Array(n);
    tr[0] = bars[0].high - bars[0].low;
    for (let i = 1; i < n; i++) {
        const pc = bars[i - 1].close;
        tr[i] = Math.max(
            bars[i].high - bars[i].low,
            Math.abs(bars[i].high - pc),
            Math.abs(bars[i].low - pc),
        );
    }
    const p_f = period;
    let sumSeed = 0;
    for (let i = 1; i <= period; i++) sumSeed += tr[i];
    const seed = sumSeed / p_f;
    const atr = new Array(n).fill(null);
    atr[period] = seed;
    let cur = seed;
    for (let i = period + 1; i < n; i++) {
        cur = (cur * (p_f - 1) + tr[i]) / p_f;
        atr[i] = cur;
    }
    let last_long = null, last_short = null;
    for (let i = 0; i < n; i++) {
        const a = atr[i];
        if (a == null) continue;
        const raw_long  = bars[i].close - multiplier * a;
        const raw_short = bars[i].close + multiplier * a;
        const new_long  = last_long  == null ? raw_long  : Math.max(raw_long,  last_long);
        const new_short = last_short == null ? raw_short : Math.min(raw_short, last_short);
        report.long_stop[i]  = new_long;
        report.short_stop[i] = new_short;
        last_long  = new_long;
        last_short = new_short;
    }
    return report;
}

// Parse "high low close" 3-token-per-line blob.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length !== 3) {
            out.errors.push({ line_no: i + 1, message: `expected 3 tokens (high low close), got ${parts.length}` });
            continue;
        }
        const h = Number(parts[0].replace(/\$/g, ''));
        const l = Number(parts[1].replace(/\$/g, ''));
        const c = Number(parts[2].replace(/\$/g, ''));
        if (!Number.isFinite(h) || !Number.isFinite(l) || !Number.isFinite(c) || h <= 0 || l <= 0 || c <= 0) {
            out.errors.push({ line_no: i + 1, message: `HLC must be positive finite` });
            continue;
        }
        if (l > h) {
            out.errors.push({ line_no: i + 1, message: `low > high` });
            continue;
        }
        if (c < l || c > h) {
            out.errors.push({ line_no: i + 1, message: `close outside [low, high]` });
            continue;
        }
        out.bars.push({ high: h, low: l, close: c });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low} ${b.close}`).join('\n');
}

// Long-stop verdict: close vs last long stop.
export function longBadge(close, long_stop) {
    if (close == null || long_stop == null
        || !Number.isFinite(close) || !Number.isFinite(long_stop)) {
        return { key: 'view.atrts.long.unknown', cls: '' };
    }
    if (close <= long_stop) return { key: 'view.atrts.long.triggered', cls: 'neg' };
    const margin = (close - long_stop) / long_stop;
    if (margin > 0.05) return { key: 'view.atrts.long.safe',    cls: 'pos' };
    if (margin > 0.02) return { key: 'view.atrts.long.holding', cls: 'pos' };
    return { key: 'view.atrts.long.tight', cls: '' };
}

// Short-stop verdict: close vs last short stop.
export function shortBadge(close, short_stop) {
    if (close == null || short_stop == null
        || !Number.isFinite(close) || !Number.isFinite(short_stop)) {
        return { key: 'view.atrts.short.unknown', cls: '' };
    }
    if (close >= short_stop) return { key: 'view.atrts.short.triggered', cls: 'neg' };
    const margin = (short_stop - close) / short_stop;
    if (margin > 0.05) return { key: 'view.atrts.short.safe',    cls: 'pos' };
    if (margin > 0.02) return { key: 'view.atrts.short.holding', cls: 'pos' };
    return { key: 'view.atrts.short.tight', cls: '' };
}

// Combined verdict: overall trade-side bias from both stops.
export function regimeBadge(close, long_stop, short_stop) {
    if (close == null || long_stop == null || short_stop == null
        || !Number.isFinite(close) || !Number.isFinite(long_stop) || !Number.isFinite(short_stop)) {
        return { key: 'view.atrts.regime.unknown', cls: '' };
    }
    const longHolds  = close > long_stop;
    const shortHolds = close < short_stop;
    if (longHolds && shortHolds) {
        const longMargin  = (close - long_stop)  / long_stop;
        const shortMargin = (short_stop - close) / short_stop;
        if (longMargin > shortMargin * 1.5)  return { key: 'view.atrts.regime.long_bias',  cls: 'pos' };
        if (shortMargin > longMargin * 1.5)  return { key: 'view.atrts.regime.short_bias', cls: 'neg' };
        return { key: 'view.atrts.regime.balanced', cls: '' };
    }
    if (!longHolds && !shortHolds) return { key: 'view.atrts.regime.both_triggered', cls: 'neg' };
    if (longHolds)                  return { key: 'view.atrts.regime.long_only',      cls: 'pos' };
    return { key: 'view.atrts.regime.short_only',     cls: 'neg' };
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, min_low: NaN, max_high: NaN, mean_close: NaN };
    }
    let sum = 0, mxH = -Infinity, mnL = Infinity;
    for (const b of bars) {
        sum += b.close;
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        min_low: Number.isFinite(mnL) ? mnL : NaN,
        max_high: Number.isFinite(mxH) ? mxH : NaN,
        mean_close: sum / bars.length,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function mkBar(close, range, rand) {
    const r = rand ? rand() : 0.5;
    return { high: close + range * r, low: close - range * (1 - r), close };
}

export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend': {
            const rand = lcg(42n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.6, 0.8, rand)),
                period: 14, multiplier: 3.0,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(160 - i * 0.6, 0.8, rand)),
                period: 14, multiplier: 3.0,
            };
        }
        case 'sideways': {
            const rand = lcg(11n);
            return {
                bars: Array.from({ length: 60 }, () => mkBar(100 + (rand() - 0.5) * 2, 0.5, rand)),
                period: 14, multiplier: 3.0,
            };
        }
        case 'long-trigger': {
            // Trend up then sharp drop → long stop gets triggered.
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(100 + i, 0.5, rand));
            for (let i = 0; i < 20; i++) bars.push(mkBar(130 - i * 2, 1.5, rand));
            return { bars, period: 14, multiplier: 3.0 };
        }
        case 'short-trigger': {
            // Trend down then sharp rise → short stop gets triggered.
            const rand = lcg(21n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(130 - i, 0.5, rand));
            for (let i = 0; i < 20; i++) bars.push(mkBar(100 + i * 2, 1.5, rand));
            return { bars, period: 14, multiplier: 3.0 };
        }
        case 'tight-mult': {
            const rand = lcg(33n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.3, 0.5, rand)),
                period: 14, multiplier: 1.0,
            };
        }
        case 'wide-mult': {
            const rand = lcg(57n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.3, 0.5, rand)),
                period: 14, multiplier: 5.0,
            };
        }
        case 'flat': {
            return {
                bars: Array.from({ length: 60 }, () => ({ high: 101, low: 99, close: 100 })),
                period: 14, multiplier: 3.0,
            };
        }
        default: return makeDemoInput('uptrend');
    }
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPriceSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
