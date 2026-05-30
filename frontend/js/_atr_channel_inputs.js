// ATR Channel helpers (EMA/SMA midline + Wilder-ATR bands).
//
// Backend body: { bars: Bar[], period: usize, multiplier: f64, use_ema: bool }
//   where Bar = { high, low, close }
// Returns: {
//   middle: (number|null)[], upper: (number|null)[], lower: (number|null)[],
//   period, multiplier, use_ema
// }

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 20;
export const DEFAULT_MULTIPLIER = 2.0;
export const DEFAULT_USE_EMA = true;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
    multiplier: DEFAULT_MULTIPLIER,
    use_ema: DEFAULT_USE_EMA,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                  return t('view.atr_channel.validate.bars_array');
    if (!Number.isInteger(input.period))             return t('view.atr_channel.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                      return t('view.atr_channel.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isFinite(input.multiplier) || input.multiplier <= 0)
                                                      return t('view.atr_channel.validate.multiplier');
    if (typeof input.use_ema !== 'boolean')           return t('view.atr_channel.validate.use_ema');
    if (input.bars.length < input.period + 1)         return t('view.atr_channel.validate.bars_min', { n: input.period + 1 });
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || !Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close))
                                                      return t('view.atr_channel.validate.bar_finite', { i });
        if (b.high < b.low)                           return t('view.atr_channel.validate.high_lt_low', { i });
        if (b.close < b.low || b.close > b.high)      return t('view.atr_channel.validate.close_outside', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        bars: input.bars.map(b => ({ high: b.high, low: b.low, close: b.close })),
        period: input.period,
        multiplier: input.multiplier,
        use_ema: input.use_ema,
    };
}

// Pure-JS mirror of crates/traderview-core/src/atr_channel.rs::compute.
export function localCompute(bars, period, multiplier, use_ema) {
    const n = bars.length;
    const report = {
        middle: new Array(n).fill(null),
        upper:  new Array(n).fill(null),
        lower:  new Array(n).fill(null),
        period, multiplier, use_ema,
    };
    if (period < 2 || !Number.isFinite(multiplier) || multiplier <= 0 || n < period + 1) return report;
    for (const b of bars) {
        if (!b || !Number.isFinite(b.high) || !Number.isFinite(b.low) || !Number.isFinite(b.close)) return report;
    }
    const closes = bars.map(b => b.close);
    report.middle = use_ema ? ema(closes, period) : sma(closes, period);
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
    for (let i = 0; i < n; i++) {
        const m = report.middle[i];
        const a = atr[i];
        if (m != null && a != null) {
            report.upper[i] = m + multiplier * a;
            report.lower[i] = m - multiplier * a;
        }
    }
    return report;
}

export function sma(series, period) {
    const n = series.length;
    const out = new Array(n).fill(null);
    if (period === 0 || n < period) return out;
    const p_f = period;
    let sum = 0;
    for (let i = 0; i < period; i++) sum += series[i];
    out[period - 1] = sum / p_f;
    for (let i = period; i < n; i++) {
        sum += series[i] - series[i - period];
        out[i] = sum / p_f;
    }
    return out;
}

export function ema(series, period) {
    const n = series.length;
    const out = new Array(n).fill(null);
    if (period === 0 || n < period) return out;
    const p_f = period;
    const k = 2 / (p_f + 1);
    let sum = 0;
    for (let i = 0; i < period; i++) sum += series[i];
    const seed = sum / p_f;
    out[period - 1] = seed;
    let cur = seed;
    for (let i = period; i < n; i++) {
        cur = series[i] * k + cur * (1 - k);
        out[i] = cur;
    }
    return out;
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

// Position verdict: close vs upper/lower/middle.
export function positionBadge(close, upper, lower, middle) {
    if (close == null || upper == null || lower == null || middle == null
        || !Number.isFinite(close) || !Number.isFinite(upper)
        || !Number.isFinite(lower) || !Number.isFinite(middle)) {
        return { key: 'view.atrc.pos.unknown', cls: '' };
    }
    if (close >= upper) return { key: 'view.atrc.pos.above_upper',    cls: 'pos' };
    if (close <= lower) return { key: 'view.atrc.pos.below_lower',    cls: 'neg' };
    if (close >  middle) return { key: 'view.atrc.pos.upper_half',    cls: 'pos' };
    if (close <  middle) return { key: 'view.atrc.pos.lower_half',    cls: 'neg' };
    return { key: 'view.atrc.pos.at_mid', cls: '' };
}

// Trend verdict over the last `lookback` midline values.
export function trendBadge(middle, lookback = 5) {
    if (!Array.isArray(middle) || middle.length < lookback) {
        return { key: 'view.atrc.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = middle.length - 1; i >= 0 && tail.length < lookback; i--) {
        const v = middle[i];
        if (v != null && Number.isFinite(v)) tail.unshift(v);
    }
    if (tail.length < 2) return { key: 'view.atrc.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.atrc.trend.flat',       cls: '' };
    if (slope > range * 0.6)       return { key: 'view.atrc.trend.up_strong', cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.atrc.trend.up',        cls: 'pos' };
    if (slope < -range * 0.6)      return { key: 'view.atrc.trend.down_strong', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.atrc.trend.down',      cls: 'neg' };
    return { key: 'view.atrc.trend.flat', cls: '' };
}

// Width verdict: (upper - lower) / middle as % volatility envelope.
export function widthBadge(upper, lower, middle) {
    if (upper == null || lower == null || middle == null
        || !Number.isFinite(upper) || !Number.isFinite(lower) || !Number.isFinite(middle)
        || middle === 0) {
        return { key: 'view.atrc.width.unknown', cls: '' };
    }
    const pct = (upper - lower) / Math.abs(middle);
    if (pct >= 0.20) return { key: 'view.atrc.width.very_wide', cls: 'neg' };
    if (pct >= 0.10) return { key: 'view.atrc.width.wide',      cls: '' };
    if (pct >= 0.04) return { key: 'view.atrc.width.normal',    cls: '' };
    if (pct >= 0.01) return { key: 'view.atrc.width.narrow',    cls: 'pos' };
    return { key: 'view.atrc.width.very_narrow', cls: 'pos' };
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
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.6, 1.0, rand)),
                period: 20, multiplier: 2.0, use_ema: true,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(160 - i * 0.6, 1.0, rand)),
                period: 20, multiplier: 2.0, use_ema: true,
            };
        }
        case 'volatile-side': {
            const rand = lcg(11n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + Math.sin(i * 0.3) * 4, 2.0, rand)),
                period: 20, multiplier: 2.0, use_ema: true,
            };
        }
        case 'tight-side': {
            const rand = lcg(13n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + Math.sin(i * 0.3) * 1.5, 0.5, rand)),
                period: 20, multiplier: 2.0, use_ema: true,
            };
        }
        case 'breakout': {
            const rand = lcg(21n);
            const bars = [];
            for (let i = 0; i < 40; i++) bars.push(mkBar(100, 0.5, rand));
            for (let i = 0; i < 20; i++) bars.push(mkBar(100 + i * 2, 1.0, rand));
            return { bars, period: 20, multiplier: 2.0, use_ema: true };
        }
        case 'breakdown': {
            const rand = lcg(33n);
            const bars = [];
            for (let i = 0; i < 40; i++) bars.push(mkBar(100, 0.5, rand));
            for (let i = 0; i < 20; i++) bars.push(mkBar(100 - i * 2, 1.0, rand));
            return { bars, period: 20, multiplier: 2.0, use_ema: true };
        }
        case 'sma': {
            // Same series as uptrend but with SMA midline for compare.
            const rand = lcg(42n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.6, 1.0, rand)),
                period: 20, multiplier: 2.0, use_ema: false,
            };
        }
        case 'wide-bands': {
            // Higher multiplier → wider envelope.
            const rand = lcg(57n);
            return {
                bars: Array.from({ length: 60 }, (_, i) => mkBar(100 + i * 0.3, 1.5, rand)),
                period: 20, multiplier: 3.5, use_ema: true,
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
