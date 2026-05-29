// Arnaud Legoux Moving Average (ALMA) helpers.
//
// Backend body: { closes: number[], period: usize, offset: f64, sigma: f64 }
// Returns: (number | null)[]  — parallel to closes.
//
// Gaussian-weighted FIR filter with offset-controlled lag. Lower lag than
// EMA, sharper response than centered SMA.

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 9;
export const DEFAULT_OFFSET = 0.85;
export const DEFAULT_SIGMA  = 6.0;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    period: DEFAULT_PERIOD,
    offset: DEFAULT_OFFSET,
    sigma:  DEFAULT_SIGMA,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                      return t('view.alma.validate.closes_array');
    if (!Number.isInteger(input.period))                   return t('view.alma.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                            return t('view.alma.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isFinite(input.offset) || input.offset < 0 || input.offset > 1)
                                                            return t('view.alma.validate.offset');
    if (!Number.isFinite(input.sigma) || input.sigma <= 0) return t('view.alma.validate.sigma');
    if (input.closes.length < input.period)                return t('view.alma.validate.closes_min', { period: input.period });
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))             return t('view.alma.validate.close_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        closes: input.closes,
        period: input.period,
        offset: input.offset,
        sigma:  input.sigma,
    };
}

// Pure-JS mirror of crates/traderview-core/src/alma_legoux.rs::compute.
export function localCompute(closes, period, offset, sigma) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (period < 2 || n < period
        || !Number.isFinite(offset) || !Number.isFinite(sigma)
        || sigma <= 0 || offset < 0 || offset > 1) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    const m = Math.floor(offset * (period - 1));
    const s = period / sigma;
    const denom_inv = 1 / (2 * s * s);
    const w = new Array(period);
    let w_sum = 0;
    for (let i = 0; i < period; i++) {
        const d = i - m;
        w[i] = Math.exp(-d * d * denom_inv);
        w_sum += w[i];
    }
    if (w_sum <= 0) return out;
    for (let i = period - 1; i < n; i++) {
        let acc = 0;
        for (let k = 0; k < period; k++) {
            acc += w[k] * closes[i + 1 - period + k];
        }
        out[i] = acc / w_sum;
    }
    return out;
}

// Convert (number|null)[] → masked array suitable for uPlot (null → null,
// finite → number).
export function toPlotLine(arr) {
    if (!Array.isArray(arr)) return [];
    return arr.map(v => (v == null || !Number.isFinite(v) ? null : v));
}

// Parse whitespace/comma-separated closes; comments + blanks ignored.
export function parseClosesBlob(blob) {
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
        const raw = tokens[i];
        let tok = raw.replace(/[\$,]/g, '');
        const v = Number(tok);
        if (!Number.isFinite(v) || v <= 0) {
            out.errors.push({ line_no: i + 1, message: `token "${raw}" not a positive finite price` });
            continue;
        }
        out.closes.push(v);
    }
    return out;
}

export function closesToBlob(closes) {
    return closes.join('\n');
}

// Trend verdict over the most-recent `lookback` ALMA values (default 5).
export function trendBadge(alma, lookback = 5) {
    if (!Array.isArray(alma) || alma.length < lookback) {
        return { key: 'view.alma.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = alma.length - 1; i >= 0 && tail.length < lookback; i--) {
        const v = alma[i];
        if (v != null && Number.isFinite(v)) tail.unshift(v);
    }
    if (tail.length < 2) return { key: 'view.alma.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.alma.trend.flat',    cls: '' };
    if (slope > range * 0.6)       return { key: 'view.alma.trend.up_strong', cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.alma.trend.up',      cls: 'pos' };
    if (slope < -range * 0.6)      return { key: 'view.alma.trend.down_strong', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.alma.trend.down',    cls: 'neg' };
    return { key: 'view.alma.trend.flat', cls: '' };
}

// Position verdict: close vs last ALMA (above/below).
export function positionBadge(close, alma_last) {
    if (close == null || alma_last == null
        || !Number.isFinite(close) || !Number.isFinite(alma_last)) {
        return { key: 'view.alma.position.unknown', cls: '' };
    }
    if (alma_last === 0) return { key: 'view.alma.position.unknown', cls: '' };
    const rel = (close - alma_last) / Math.abs(alma_last);
    if (rel > 0.02)  return { key: 'view.alma.position.well_above', cls: 'pos' };
    if (rel > 0)     return { key: 'view.alma.position.above',      cls: 'pos' };
    if (rel < -0.02) return { key: 'view.alma.position.well_below', cls: 'neg' };
    if (rel < 0)     return { key: 'view.alma.position.below',      cls: 'neg' };
    return { key: 'view.alma.position.at', cls: '' };
}

// Series stats describing the input (count, min/max/last) for the panel.
export function summarizeCloses(closes) {
    if (!Array.isArray(closes) || closes.length === 0) {
        return { count: 0, last: NaN, min: NaN, max: NaN, mean: NaN };
    }
    let sum = 0, mx = -Infinity, mn = Infinity;
    for (const v of closes) {
        sum += v;
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    return {
        count: closes.length,
        last: closes[closes.length - 1],
        min: Number.isFinite(mn) ? mn : NaN,
        max: Number.isFinite(mx) ? mx : NaN,
        mean: sum / closes.length,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend': {
            const rand = lcg(42n);
            return { closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.8 + (rand() - 0.5) * 1.5),
                     period: 9, offset: 0.85, sigma: 6 };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return { closes: Array.from({ length: 80 }, (_, i) => 180 - i * 0.8 + (rand() - 0.5) * 1.5),
                     period: 9, offset: 0.85, sigma: 6 };
        }
        case 'sideways': {
            const rand = lcg(11n);
            return { closes: Array.from({ length: 80 }, () => 100 + (rand() - 0.5) * 4),
                     period: 9, offset: 0.85, sigma: 6 };
        }
        case 'step-up': {
            const closes = [];
            for (let i = 0; i < 20; i++) closes.push(100);
            for (let i = 0; i < 20; i++) closes.push(120);
            return { closes, period: 9, offset: 0.85, sigma: 6 };
        }
        case 'high-offset': {
            // offset → 1 reacts fastest (close to EMA-limit behavior).
            const rand = lcg(99n);
            return { closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.5 + (rand() - 0.5) * 2),
                     period: 9, offset: 0.95, sigma: 6 };
        }
        case 'low-offset': {
            // offset → 0 puts kernel peak at oldest bar (just smoothing).
            const rand = lcg(13n);
            return { closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.5 + (rand() - 0.5) * 2),
                     period: 9, offset: 0.10, sigma: 6 };
        }
        case 'sharp-kernel': {
            // High sigma → sharper kernel = less smoothing.
            const rand = lcg(21n);
            return { closes: Array.from({ length: 80 }, (_, i) => 100 + Math.sin(i * 0.3) * 5 + (rand() - 0.5) * 0.5),
                     period: 21, offset: 0.85, sigma: 12 };
        }
        case 'soft-kernel': {
            // Low sigma → wider kernel = stronger smoothing.
            const rand = lcg(33n);
            return { closes: Array.from({ length: 80 }, (_, i) => 100 + Math.sin(i * 0.3) * 5 + (rand() - 0.5) * 0.5),
                     period: 21, offset: 0.85, sigma: 2 };
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
