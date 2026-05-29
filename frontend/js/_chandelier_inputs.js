// Chandelier Exit (Chuck LeBeau) helpers — ATR trailing stop "hanging" from HH/LL.
//
// Backend body: { bars: Bar[], period: usize, multiplier: f64 }
//   where Bar = { high, low, close }
// Returns: {
//   stop: (number|null)[], direction: ('long'|'short'|null)[],
//   long_stop: (number|null)[], short_stop: (number|null)[],
//   period, multiplier,
// }

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 22;
export const DEFAULT_MULTIPLIER = 3.0;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
    multiplier: DEFAULT_MULTIPLIER,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                       return t('view.chandelier.validate.bars_array');
    if (!Number.isInteger(input.period) || input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                           return t('view.chandelier.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isFinite(input.multiplier) || input.multiplier <= 0)
                                                           return t('view.chandelier.validate.multiplier');
    if (input.bars.length < input.period + 1)             return t('view.chandelier.validate.bars_min', { n: input.period + 1 });
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return t('view.chandelier.validate.bar_missing', { i });
        if (typeof b.high !== 'number' || typeof b.low !== 'number' || typeof b.close !== 'number')
                                                            return t('view.chandelier.validate.hlc_numbers', { i });
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close))
                                                            return t('view.chandelier.validate.hlc_finite', { i });
        if (b.high < b.low)                                return t('view.chandelier.validate.high_lt_low', { i });
        if (b.close < b.low || b.close > b.high)           return t('view.chandelier.validate.close_outside', { i });
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

// Pure-JS mirror of crates/traderview-core/src/chandelier_exit.rs::compute.
export function localCompute(bars, period, multiplier) {
    const n = bars.length;
    const report = {
        stop:       new Array(n).fill(null),
        direction:  new Array(n).fill(null),
        long_stop:  new Array(n).fill(null),
        short_stop: new Array(n).fill(null),
        period, multiplier,
    };
    if (period < 2 || !Number.isFinite(multiplier) || multiplier <= 0 || n < period + 1) return report;
    for (const b of bars) {
        if (!Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close)) return report;
    }
    // Wilder ATR.
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
    let seedSum = 0;
    for (let i = 1; i <= period; i++) seedSum += tr[i];
    const seed = seedSum / p_f;
    const atr = new Array(n).fill(null);
    atr[period] = seed;
    let cur = seed;
    for (let i = period + 1; i < n; i++) {
        cur = (cur * (p_f - 1) + tr[i]) / p_f;
        atr[i] = cur;
    }
    // Raw long/short stops.
    for (let i = period - 1; i < n; i++) {
        let hh = -Infinity, ll = Infinity;
        for (let j = i + 1 - period; j <= i; j++) {
            if (bars[j].high > hh) hh = bars[j].high;
            if (bars[j].low  < ll) ll = bars[j].low;
        }
        const a = atr[i];
        if (a != null) {
            report.long_stop[i]  = hh - multiplier * a;
            report.short_stop[i] = ll + multiplier * a;
        }
    }
    // Ratcheted stop + direction.
    let dir = 'long';
    let cur_stop = null;
    for (let i = 0; i < n; i++) {
        const raw_long  = report.long_stop[i];
        const raw_short = report.short_stop[i];
        if (raw_long == null || raw_short == null) continue;
        const close = bars[i].close;
        if (dir === 'long') {
            const ref = cur_stop != null ? cur_stop : raw_long;
            if (close < ref) {
                dir = 'short';
                cur_stop = raw_short;
            } else {
                cur_stop = Math.max(raw_long, ref);
            }
        } else {
            const ref = cur_stop != null ? cur_stop : raw_short;
            if (close > ref) {
                dir = 'long';
                cur_stop = raw_long;
            } else {
                cur_stop = Math.min(raw_short, ref);
            }
        }
        report.stop[i] = cur_stop;
        report.direction[i] = dir;
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

// Current direction badge.
export function dirBadge(direction_last) {
    if (direction_last == null) return { key: 'view.chx.dir.unknown', cls: '' };
    if (direction_last === 'long')  return { key: 'view.chx.dir.long',  cls: 'pos' };
    if (direction_last === 'short') return { key: 'view.chx.dir.short', cls: 'neg' };
    return { key: 'view.chx.dir.unknown', cls: '' };
}

// Last direction flip detector (with bars-ago).
export function flipBadge(direction) {
    if (!Array.isArray(direction)) return { key: 'view.chx.flip.unknown', cls: '' };
    let prev = null;
    let last_flip = null;
    let last_flip_idx = -1;
    for (let i = 0; i < direction.length; i++) {
        const v = direction[i];
        if (v == null) continue;
        if (prev != null && prev !== v) { last_flip = v; last_flip_idx = i; }
        prev = v;
    }
    if (last_flip == null) return { key: 'view.chx.flip.none', cls: '' };
    const barsAgo = direction.length - 1 - last_flip_idx;
    if (last_flip === 'long')  return { key: 'view.chx.flip.to_long',  cls: 'pos', barsAgo };
    return { key: 'view.chx.flip.to_short', cls: 'neg', barsAgo };
}

// Stop distance from current price (last close).
export function distanceBadge(stop_last, close_last) {
    if (stop_last == null || close_last == null
        || !Number.isFinite(stop_last) || !Number.isFinite(close_last)) {
        return { key: 'view.chx.dist.unknown', cls: '', distance: NaN, distance_pct: NaN };
    }
    if (close_last === 0) return { key: 'view.chx.dist.unknown', cls: '', distance: NaN, distance_pct: NaN };
    const d = close_last - stop_last;
    const pct = Math.abs(d) / Math.abs(close_last);
    let key, cls;
    if (pct < 0.005)      { key = 'view.chx.dist.at_stop';   cls = 'neg'; }
    else if (pct < 0.02)  { key = 'view.chx.dist.near_stop'; cls = 'neg'; }
    else if (pct < 0.05)  { key = 'view.chx.dist.normal';    cls = ''; }
    else if (pct < 0.10)  { key = 'view.chx.dist.safe';      cls = 'pos'; }
    else                  { key = 'view.chx.dist.very_safe'; cls = 'pos'; }
    return { key, cls, distance: d, distance_pct: pct };
}

// Flip-count diagnostic over the entire series.
export function flipStats(direction) {
    let flips = 0;
    let long_bars = 0;
    let short_bars = 0;
    let prev = null;
    for (const d of direction) {
        if (d == null) continue;
        if (d === 'long')       long_bars++;
        else if (d === 'short') short_bars++;
        if (prev != null && prev !== d) flips++;
        prev = d;
    }
    return { flips, long_bars, short_bars };
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

export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend': {
            const rand = lcg(42n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i, 1, rand)),
                period: 22, multiplier: 3.0,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(160 - i, 1, rand)),
                period: 22, multiplier: 3.0,
            };
        }
        case 'flat': {
            return {
                bars: Array.from({ length: 50 }, () => ({ high: 101, low: 99, close: 100 })),
                period: 22, multiplier: 3.0,
            };
        }
        case 'reversal-up': {
            const rand = lcg(11n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(130 - i * 0.8, 1, rand));
            for (let i = 0; i < 30; i++) bars.push(mkBar(106 + i * 0.8, 1, rand));
            return { bars, period: 22, multiplier: 3.0 };
        }
        case 'reversal-down': {
            const rand = lcg(13n);
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push(mkBar(100 + i * 0.8, 1, rand));
            for (let i = 0; i < 30; i++) bars.push(mkBar(124 - i * 0.8, 1, rand));
            return { bars, period: 22, multiplier: 3.0 };
        }
        case 'whipsaw': {
            // Many small fakeouts → frequent direction flips.
            const rand = lcg(21n);
            return {
                bars: Array.from({ length: 80 }, (_, i) => mkBar(100 + Math.sin(i * 0.5) * 3, 2, rand)),
                period: 22, multiplier: 3.0,
            };
        }
        case 'tight-mult': {
            // Multiplier=1 → tight stops, more flips.
            const rand = lcg(33n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.5 + Math.sin(i * 0.5) * 1.5, 1, rand)),
                period: 22, multiplier: 1.0,
            };
        }
        case 'wide-mult': {
            // Multiplier=5 → very wide stops, rare flips.
            const rand = lcg(57n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.5 + Math.sin(i * 0.5) * 1.5, 1, rand)),
                period: 22, multiplier: 5.0,
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
