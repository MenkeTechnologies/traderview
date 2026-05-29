// Chande-Kroll Stop helpers — two-pass volatility trailing stop.
//
// Backend body: { bars: Bar[], p: usize, x: f64, q: usize }
//   where Bar = { high, low, close }
// Returns: {
//   long_stop: (number|null)[], short_stop: (number|null)[],
//   p, x, q
// }

export const DEFAULT_P = 10;
export const DEFAULT_X = 1.0;
export const DEFAULT_Q = 9;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    p: DEFAULT_P,
    x: DEFAULT_X,
    q: DEFAULT_Q,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                       return 'bars must be an array';
    if (!Number.isInteger(input.p) || input.p < MIN_PERIOD || input.p > MAX_PERIOD)
                                                           return `p must be integer in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (!Number.isInteger(input.q) || input.q < MIN_PERIOD || input.q > MAX_PERIOD)
                                                           return `q must be integer in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (!Number.isFinite(input.x) || input.x <= 0)        return 'x must be positive finite';
    if (input.bars.length < input.p + input.q)            return `need at least p + q = ${input.p + input.q} bars`;
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return `bars[${i}] missing`;
        if (typeof b.high !== 'number' || typeof b.low !== 'number' || typeof b.close !== 'number')
                                                            return `bars[${i}] HLC must be numbers`;
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close))
                                                            return `bars[${i}] HLC must be finite`;
        if (b.high < b.low)                                return `bars[${i}] high < low`;
        if (b.close < b.low || b.close > b.high)           return `bars[${i}] close outside [low, high]`;
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low, close: b.close })),
        p: input.p, x: input.x, q: input.q,
    };
}

// Pure-JS mirror of crates/traderview-core/src/chande_kroll_stop.rs::compute.
export function localCompute(bars, p, x, q) {
    const n = bars.length;
    const report = {
        long_stop:  new Array(n).fill(null),
        short_stop: new Array(n).fill(null),
        p, x, q,
    };
    if (p < 2 || q < 2 || !Number.isFinite(x) || x <= 0 || n < p + q) return report;
    for (const b of bars) {
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close)) return report;
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
    const p_f = p;
    let seedSum = 0;
    for (let i = 1; i <= p; i++) seedSum += tr[i];
    const seed = seedSum / p_f;
    const atr = new Array(n).fill(null);
    atr[p] = seed;
    let cur = seed;
    for (let i = p + 1; i < n; i++) {
        cur = (cur * (p_f - 1) + tr[i]) / p_f;
        atr[i] = cur;
    }
    const raw_long  = new Array(n).fill(null);
    const raw_short = new Array(n).fill(null);
    for (let i = p - 1; i < n; i++) {
        let hh = -Infinity, ll = Infinity;
        for (let j = i + 1 - p; j <= i; j++) {
            if (bars[j].high > hh) hh = bars[j].high;
            if (bars[j].low  < ll) ll = bars[j].low;
        }
        const a = atr[i];
        if (a != null) {
            raw_long[i]  = hh - x * a;
            raw_short[i] = ll + x * a;
        }
    }
    for (let i = p + q - 1; i < n; i++) {
        let mx = -Infinity, mn = Infinity, allLong = true, allShort = true;
        for (let j = i + 1 - q; j <= i; j++) {
            if (raw_long[j]  == null) allLong = false;
            else if (raw_long[j]  > mx) mx = raw_long[j];
            if (raw_short[j] == null) allShort = false;
            else if (raw_short[j] < mn) mn = raw_short[j];
        }
        if (allLong)  report.long_stop[i]  = mx;
        if (allShort) report.short_stop[i] = mn;
    }
    return report;
}

// Parse "high low close" 3-token-per-line blob.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
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

// Regime detector — long_stop crossing BELOW short_stop = long bias; ABOVE = short bias.
export function regimeBadge(long_last, short_last, close_last) {
    if (long_last == null || short_last == null
        || !Number.isFinite(long_last) || !Number.isFinite(short_last)) {
        return { key: 'view.cks.regime.unknown', cls: '' };
    }
    if (Number.isFinite(close_last)) {
        if (close_last < long_last && close_last < short_last) return { key: 'view.cks.regime.short_active', cls: 'neg' };
        if (close_last > short_last && close_last > long_last) return { key: 'view.cks.regime.long_active',  cls: 'pos' };
    }
    if (long_last < short_last) return { key: 'view.cks.regime.long_bias',  cls: 'pos' };
    if (long_last > short_last) return { key: 'view.cks.regime.short_bias', cls: 'neg' };
    return { key: 'view.cks.regime.neutral', cls: '' };
}

// Stop-band width verdict (signed: short_stop − long_stop).
export function widthBadge(long_last, short_last) {
    if (long_last == null || short_last == null
        || !Number.isFinite(long_last) || !Number.isFinite(short_last)) {
        return { key: 'view.cks.width.unknown', cls: '' };
    }
    const d = short_last - long_last;
    const mid = (long_last + short_last) / 2;
    if (mid === 0) return { key: 'view.cks.width.unknown', cls: '' };
    const pct = d / Math.abs(mid);
    if (d < 0)           return { key: 'view.cks.width.inverted',   cls: 'neg' };
    if (Math.abs(pct) < 0.005) return { key: 'view.cks.width.tight', cls: '' };
    if (Math.abs(pct) < 0.02)  return { key: 'view.cks.width.normal', cls: '' };
    if (Math.abs(pct) < 0.05)  return { key: 'view.cks.width.wide',   cls: '' };
    return { key: 'view.cks.width.very_wide', cls: '' };
}

// Trend over the long_stop series.
export function longTrendBadge(long_series, lookback = 10) {
    return tailTrend(long_series, lookback, 'long');
}

// Trend over the short_stop series.
export function shortTrendBadge(short_series, lookback = 10) {
    return tailTrend(short_series, lookback, 'short');
}

function tailTrend(series, lookback, which) {
    const baseKey = which === 'long' ? 'view.cks.long_trend' : 'view.cks.short_trend';
    if (!Array.isArray(series) || series.length === 0) {
        return { key: `${baseKey}.unknown`, cls: '' };
    }
    const tail = [];
    for (let i = series.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (series[i] != null && Number.isFinite(series[i])) tail.unshift(series[i]);
    }
    if (tail.length < 2) return { key: `${baseKey}.unknown`, cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: `${baseKey}.flat`,    cls: '' };
    if (slope > range * 0.1)       return { key: `${baseKey}.rising`, cls: which === 'long' ? 'pos' : 'neg' };
    if (slope < -range * 0.1)      return { key: `${baseKey}.falling`, cls: which === 'long' ? 'neg' : 'pos' };
    return { key: `${baseKey}.flat`, cls: '' };
}

export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, min_low: NaN, max_high: NaN, mean_close: NaN };
    }
    let sumC = 0, mxH = -Infinity, mnL = Infinity;
    for (const b of bars) {
        sumC += b.close;
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        mean_close: sumC / bars.length,
        min_low:  Number.isFinite(mnL) ? mnL : NaN,
        max_high: Number.isFinite(mxH) ? mxH : NaN,
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

export function makeDemoInput(kind = 'flat') {
    switch (kind) {
        case 'flat': {
            return {
                bars: Array.from({ length: 40 }, () => ({ high: 101, low: 99, close: 100 })),
                p: 10, x: 1.0, q: 9,
            };
        }
        case 'uptrend': {
            const rand = lcg(42n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i, 1.0, rand)),
                p: 10, x: 1.0, q: 9,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(160 - i, 1.0, rand)),
                p: 10, x: 1.0, q: 9,
            };
        }
        case 'reversal-up': {
            const rand = lcg(11n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(130 - i * 0.5, 1, rand));
            for (let i = 0; i < 30; i++) bars.push(mkBar(115 + i * 0.5, 1, rand));
            return { bars, p: 10, x: 1.0, q: 9 };
        }
        case 'reversal-down': {
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(100 + i * 0.5, 1, rand));
            for (let i = 0; i < 30; i++) bars.push(mkBar(115 - i * 0.5, 1, rand));
            return { bars, p: 10, x: 1.0, q: 9 };
        }
        case 'high-x': {
            // Wide stops (x=3) → further from price.
            const rand = lcg(21n);
            return {
                bars: Array.from({ length: 50 }, (_, i) => mkBar(100 + Math.sin(i * 0.3) * 3, 1, rand)),
                p: 10, x: 3.0, q: 9,
            };
        }
        case 'short-bars': {
            // p=5, q=4 — faster responsive stops.
            const rand = lcg(33n);
            return {
                bars: Array.from({ length: 30 }, (_, i) => mkBar(100 + i * 0.5, 1, rand)),
                p: 5, x: 1.0, q: 4,
            };
        }
        case 'volatile': {
            const rand = lcg(57n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + Math.sin(i * 0.2) * 10, 3, rand)),
                p: 10, x: 1.5, q: 9,
            };
        }
        default: return makeDemoInput('flat');
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
