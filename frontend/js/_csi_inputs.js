// Candle Strength Index (CSI) helpers.
//
// Backend body: { bars: Bar[], period: usize }
//   where Bar = { open, high, low, close }
// Returns: (number|null)[]  — EMA of per-bar (close − open) / (high − low) ∈ [-1, +1].

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 14;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                       return t('view.csi.validate.bars_array');
    if (!Number.isInteger(input.period))                  return t('view.csi.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                           return t('view.csi.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (input.bars.length < input.period)                 return t('view.csi.validate.bars_min', { period: input.period });
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b)                                            return t('view.csi.validate.bar_missing', { i });
        if (typeof b.open !== 'number' || typeof b.high !== 'number'
            || typeof b.low !== 'number' || typeof b.close !== 'number')
                                                            return t('view.csi.validate.ohlc_numbers', { i });
        if (!Number.isFinite(b.open) || !Number.isFinite(b.high)
            || !Number.isFinite(b.low)  || !Number.isFinite(b.close))
                                                            return t('view.csi.validate.ohlc_finite', { i });
        if (b.high < b.low)                                return t('view.csi.validate.high_lt_low', { i });
        if (b.close < b.low || b.close > b.high)           return t('view.csi.validate.close_outside', { i });
        if (b.open  < b.low || b.open  > b.high)           return t('view.csi.validate.open_outside', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ open: b.open, high: b.high, low: b.low, close: b.close })),
        period: input.period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/candle_strength_index.rs::compute.
export function localCompute(bars, period) {
    const n = bars.length;
    const out = new Array(n).fill(null);
    if (period < 2 || n < period) return out;
    for (const b of bars) {
        if (!Number.isFinite(b.open) || !Number.isFinite(b.high)
            || !Number.isFinite(b.low)  || !Number.isFinite(b.close)) return out;
    }
    const raw = bars.map(b => {
        const r = b.high - b.low;
        return r > 0 ? (b.close - b.open) / r : 0;
    });
    const p_f = period;
    const k = 2 / (p_f + 1);
    let sum = 0;
    for (let i = 0; i < period; i++) sum += raw[i];
    const seed = sum / p_f;
    out[period - 1] = seed;
    let cur = seed;
    for (let i = period; i < n; i++) {
        cur = raw[i] * k + cur * (1 - k);
        out[i] = cur;
    }
    return out;
}

// Parse "open high low close" 4-token-per-line blob.
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
        if (parts.length !== 4) {
            out.errors.push({ line_no: i + 1, message: `expected 4 tokens (open high low close), got ${parts.length}` });
            continue;
        }
        const o = Number(parts[0].replace(/\$/g, ''));
        const h = Number(parts[1].replace(/\$/g, ''));
        const l = Number(parts[2].replace(/\$/g, ''));
        const c = Number(parts[3].replace(/\$/g, ''));
        if (!Number.isFinite(o) || !Number.isFinite(h) || !Number.isFinite(l) || !Number.isFinite(c)
            || o <= 0 || h <= 0 || l <= 0 || c <= 0) {
            out.errors.push({ line_no: i + 1, message: `OHLC must be positive finite` });
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
        if (o < l || o > h) {
            out.errors.push({ line_no: i + 1, message: `open outside [low, high]` });
            continue;
        }
        out.bars.push({ open: o, high: h, low: l, close: c });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.open} ${b.high} ${b.low} ${b.close}`).join('\n');
}

// 7-tier strength badge on most recent CSI.
export function strengthBadge(csi_last) {
    if (csi_last == null || !Number.isFinite(csi_last)) {
        return { key: 'view.csi.strength.unknown', cls: '' };
    }
    if (csi_last >= 0.75)  return { key: 'view.csi.strength.marubozu_green', cls: 'pos' };
    if (csi_last >= 0.40)  return { key: 'view.csi.strength.strong_buy',     cls: 'pos' };
    if (csi_last >= 0.15)  return { key: 'view.csi.strength.buy_lean',       cls: 'pos' };
    if (csi_last > -0.15)  return { key: 'view.csi.strength.indecision',     cls: '' };
    if (csi_last > -0.40)  return { key: 'view.csi.strength.sell_lean',      cls: 'neg' };
    if (csi_last > -0.75)  return { key: 'view.csi.strength.strong_sell',    cls: 'neg' };
    return { key: 'view.csi.strength.marubozu_red', cls: 'neg' };
}

// Trend over last N populated CSI values.
export function trendBadge(csi, lookback = 10) {
    if (!Array.isArray(csi) || csi.length === 0) {
        return { key: 'view.csi.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = csi.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (csi[i] != null && Number.isFinite(csi[i])) tail.unshift(csi[i]);
    }
    if (tail.length < 2) return { key: 'view.csi.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.csi.trend.flat',         cls: '' };
    if (slope > range * 0.5)       return { key: 'view.csi.trend.rising_fast', cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.csi.trend.rising',      cls: 'pos' };
    if (slope < -range * 0.5)      return { key: 'view.csi.trend.falling_fast', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.csi.trend.falling',     cls: 'neg' };
    return { key: 'view.csi.trend.flat', cls: '' };
}

// Recent zero-cross detector.
export function crossBadge(csi) {
    if (!Array.isArray(csi)) return { key: 'view.csi.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < csi.length; i++) {
        const v = csi[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 0 && v > 0) { last_cross = 'up';   last_cross_idx = i; }
            else if (prev >= 0 && v < 0) { last_cross = 'down'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.csi.cross.none', cls: '' };
    const barsAgo = csi.length - 1 - last_cross_idx;
    if (last_cross === 'up') return { key: 'view.csi.cross.up_recent', cls: 'pos', barsAgo };
    return { key: 'view.csi.cross.down_recent', cls: 'neg', barsAgo };
}

// Per-bar OHLC stats for the lower panel.
export function summarizeBars(bars) {
    if (!Array.isArray(bars) || bars.length === 0) {
        return { count: 0, last_close: NaN, min_low: NaN, max_high: NaN,
                 up_bars: 0, down_bars: 0, doji_bars: 0 };
    }
    let mxH = -Infinity, mnL = Infinity;
    let up = 0, down = 0, doji = 0;
    for (const b of bars) {
        if (b.high > mxH) mxH = b.high;
        if (b.low  < mnL) mnL = b.low;
        if (b.close > b.open)       up++;
        else if (b.close < b.open)  down++;
        else                        doji++;
    }
    return {
        count: bars.length,
        last_close: bars[bars.length - 1].close,
        min_low:  Number.isFinite(mnL) ? mnL : NaN,
        max_high: Number.isFinite(mxH) ? mxH : NaN,
        up_bars: up, down_bars: down, doji_bars: doji,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function mkBar(open, range, close_offset, rand) {
    const close = open + close_offset;
    const h_extra = rand ? rand() * 0.1 : 0.05;
    const l_extra = rand ? rand() * 0.1 : 0.05;
    const high = Math.max(open, close) + range * h_extra;
    const low  = Math.min(open, close) - range * l_extra;
    return { open, high, low, close };
}

export function makeDemoInput(kind = 'mixed') {
    switch (kind) {
        case 'mixed': {
            const rand = lcg(42n);
            const bars = [];
            let price = 100;
            for (let i = 0; i < 40; i++) {
                const range = 1 + rand();
                const dir = rand() - 0.5;
                bars.push(mkBar(price, range, dir, rand));
                price = bars[bars.length - 1].close;
            }
            return { bars, period: 14 };
        }
        case 'green-marubozu': {
            const bars = [];
            let p = 100;
            for (let i = 0; i < 40; i++) {
                bars.push({ open: p, high: p + 1, low: p, close: p + 1 });
                p += 1;
            }
            return { bars, period: 14 };
        }
        case 'red-marubozu': {
            const bars = [];
            let p = 140;
            for (let i = 0; i < 40; i++) {
                bars.push({ open: p, high: p, low: p - 1, close: p - 1 });
                p -= 1;
            }
            return { bars, period: 14 };
        }
        case 'doji-cluster': {
            const bars = [];
            for (let i = 0; i < 40; i++) {
                bars.push({ open: 100, high: 101, low: 99, close: 100 });
            }
            return { bars, period: 14 };
        }
        case 'alternating': {
            // ±1 alternates each bar → EMA wobbles near 0.
            const bars = [];
            for (let i = 0; i < 40; i++) {
                if (i % 2 === 0) bars.push({ open: 100, high: 110, low: 100, close: 110 });
                else              bars.push({ open: 110, high: 110, low: 100, close: 100 });
            }
            return { bars, period: 14 };
        }
        case 'shifting-bullish': {
            // First half mixed, second half mostly green marubozu.
            const rand = lcg(13n);
            const bars = [];
            let p = 100;
            for (let i = 0; i < 20; i++) {
                bars.push(mkBar(p, 1, (rand() - 0.5) * 0.5, rand));
                p = bars[bars.length - 1].close;
            }
            for (let i = 0; i < 30; i++) {
                bars.push({ open: p, high: p + 1, low: p, close: p + 1 });
                p += 1;
            }
            return { bars, period: 14 };
        }
        case 'long-period': {
            // Period=30 → smoother.
            const rand = lcg(21n);
            const bars = [];
            let p = 100;
            for (let i = 0; i < 80; i++) {
                bars.push(mkBar(p, 1.5, (rand() - 0.5) * 1.2, rand));
                p = bars[bars.length - 1].close;
            }
            return { bars, period: 30 };
        }
        case 'breakout-up': {
            // Tight chop then green-marubozu burst → CSI swings positive.
            const rand = lcg(33n);
            const bars = [];
            for (let i = 0; i < 25; i++) bars.push(mkBar(100, 0.4, (rand() - 0.5) * 0.1, rand));
            let p = 100;
            for (let i = 0; i < 15; i++) {
                bars.push({ open: p, high: p + 1, low: p, close: p + 1 });
                p += 1;
            }
            return { bars, period: 14 };
        }
        default: return makeDemoInput('mixed');
    }
}

export function fmtRatio(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
