// Bollinger Oscillators (combined %B + Bandwidth) helpers.
//
// Backend body: { closes: number[], period: usize, k: f64 }
// Returns: {
//   percent_b: (number|null)[],
//   bandwidth: (number|null)[],
//   middle:    (number|null)[],
//   upper:     (number|null)[],
//   lower:     (number|null)[],
// }

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 20;
export const DEFAULT_K = 2.0;
export const MIN_PERIOD = 1;       // Rust impl allows period=1 (returns all-null on n<period)
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    period: DEFAULT_PERIOD,
    k: DEFAULT_K,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return t('view.bbosc.validate.closes_array');
    if (!Number.isInteger(input.period) || input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                             return t('view.bbosc.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isFinite(input.k) || input.k < 0)           return t('view.bbosc.validate.k_non_negative');
    if (input.closes.length < input.period)                 return t('view.bbosc.validate.closes_min_period', { period: input.period });
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return t('view.bbosc.validate.close_not_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return { closes: input.closes.slice(), period: input.period, k: input.k };
}

// Pure-JS mirror of crates/traderview-core/src/bollinger_oscillators.rs::compute.
export function localCompute(closes, period, k) {
    const n = closes.length;
    const out = {
        percent_b: new Array(n).fill(null),
        bandwidth: new Array(n).fill(null),
        middle:    new Array(n).fill(null),
        upper:     new Array(n).fill(null),
        lower:     new Array(n).fill(null),
    };
    if (period === 0 || !Number.isFinite(k) || k < 0 || n < period) return out;
    for (let i = period - 1; i < n; i++) {
        let allFinite = true;
        let sum = 0;
        for (let j = i + 1 - period; j <= i; j++) {
            if (!Number.isFinite(closes[j])) { allFinite = false; break; }
            sum += closes[j];
        }
        if (!allFinite || !Number.isFinite(closes[i])) continue;
        const mean = sum / period;
        let v_acc = 0;
        for (let j = i + 1 - period; j <= i; j++) v_acc += (closes[j] - mean) ** 2;
        const variance = v_acc / period;
        const stdev = Math.sqrt(Math.max(0, variance));
        const upper = mean + k * stdev;
        const lower = mean - k * stdev;
        out.middle[i] = mean;
        out.upper[i] = upper;
        out.lower[i] = lower;
        const band_width = upper - lower;
        if (band_width > 0) {
            out.percent_b[i] = (closes[i] - lower) / band_width;
        }
        if (Math.abs(mean) > 1e-18) {
            out.bandwidth[i] = band_width / mean;
        }
    }
    return out;
}

// Parse positive prices.
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
        const v = Number(tokens[i].replace(/[\$,]/g, ''));
        if (!Number.isFinite(v) || v <= 0) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not a positive finite price` });
            continue;
        }
        out.closes.push(v);
    }
    return out;
}

export function closesToBlob(closes) {
    return closes.join('\n');
}

// %B zone badge (7 tiers).
export function pbBadge(pb) {
    if (pb == null || !Number.isFinite(pb)) return { key: 'view.bbosc.pb.unknown', cls: '' };
    if (pb >= 1.0)   return { key: 'view.bbosc.pb.breakout',   cls: 'pos' };
    if (pb >= 0.8)   return { key: 'view.bbosc.pb.near_upper', cls: 'pos' };
    if (pb >= 0.55)  return { key: 'view.bbosc.pb.upper_half', cls: 'pos' };
    if (pb >= 0.45)  return { key: 'view.bbosc.pb.middle',     cls: '' };
    if (pb >= 0.2)   return { key: 'view.bbosc.pb.lower_half', cls: 'neg' };
    if (pb >= 0)     return { key: 'view.bbosc.pb.near_lower', cls: 'neg' };
    return { key: 'view.bbosc.pb.breakdown', cls: 'neg' };
}

// Bandwidth-squeeze badge using percentile rank within history.
export function bwBadge(bandwidth, lookback = 60) {
    if (!Array.isArray(bandwidth) || bandwidth.length === 0) {
        return { key: 'view.bbosc.bw.unknown', cls: '' };
    }
    const tail = [];
    for (let i = bandwidth.length - 1; i >= 0 && tail.length < lookback; i--) {
        const v = bandwidth[i];
        if (v != null && Number.isFinite(v)) tail.unshift(v);
    }
    if (tail.length < 5) return { key: 'view.bbosc.bw.unknown', cls: '' };
    const last = tail[tail.length - 1];
    const sorted = [...tail].sort((a, b) => a - b);
    const rank = sorted.indexOf(last);
    const pctile = rank / (sorted.length - 1);
    if (pctile <= 0.10) return { key: 'view.bbosc.bw.tight_squeeze', cls: 'pos' };
    if (pctile <= 0.30) return { key: 'view.bbosc.bw.compression',    cls: '' };
    if (pctile <= 0.70) return { key: 'view.bbosc.bw.normal',         cls: '' };
    if (pctile <= 0.90) return { key: 'view.bbosc.bw.expansion',      cls: '' };
    return { key: 'view.bbosc.bw.extreme_expansion', cls: 'neg' };
}

// Trend over recent %B (rising = momentum building toward upper).
export function pbTrendBadge(percent_b, lookback = 10) {
    if (!Array.isArray(percent_b) || percent_b.length === 0) {
        return { key: 'view.bbosc.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = percent_b.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (percent_b[i] != null && Number.isFinite(percent_b[i])) tail.unshift(percent_b[i]);
    }
    if (tail.length < 2) return { key: 'view.bbosc.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.bbosc.trend.flat',         cls: '' };
    if (slope > range * 0.5)       return { key: 'view.bbosc.trend.rising_fast', cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.bbosc.trend.rising',      cls: 'pos' };
    if (slope < -range * 0.5)      return { key: 'view.bbosc.trend.falling_fast', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.bbosc.trend.falling',     cls: 'neg' };
    return { key: 'view.bbosc.trend.flat', cls: '' };
}

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

export function makeDemoInput(kind = 'normal-trend') {
    switch (kind) {
        case 'normal-trend': {
            const rand = lcg(42n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.3 + (rand() - 0.5) * 1),
                period: 20, k: 2.0,
            };
        }
        case 'ttm-squeeze': {
            // Tight chop → expansion → classic TTM squeeze setup.
            const rand = lcg(7n);
            const closes = [];
            for (let i = 0; i < 40; i++) closes.push(100 + (rand() - 0.5) * 0.15);
            for (let i = 0; i < 40; i++) closes.push(100 + i * 0.5 + (rand() - 0.5) * 1);
            return { closes, period: 20, k: 2.0 };
        }
        case 'walking-upper': {
            const rand = lcg(11n);
            const closes = [100];
            for (let i = 1; i < 80; i++) closes.push(closes[i - 1] + 0.5 + (rand() - 0.5) * 0.1);
            return { closes, period: 20, k: 2.0 };
        }
        case 'walking-lower': {
            const rand = lcg(13n);
            const closes = [180];
            for (let i = 1; i < 80; i++) closes.push(closes[i - 1] - 0.5 + (rand() - 0.5) * 0.1);
            return { closes, period: 20, k: 2.0 };
        }
        case 'oscillating': {
            const rand = lcg(21n);
            return {
                closes: Array.from({ length: 80 }, (_, i) =>
                    100 + Math.sin(i * 0.5) * 5 + (rand() - 0.5) * 0.5),
                period: 20, k: 2.0,
            };
        }
        case 'flat': {
            return {
                closes: new Array(40).fill(100),
                period: 20, k: 2.0,
            };
        }
        case 'wide-bands': {
            // k=3 → bandwidth larger
            const rand = lcg(33n);
            return {
                closes: Array.from({ length: 80 }, (_, i) =>
                    100 + Math.sin(i * 0.4) * 3 + (rand() - 0.5) * 1),
                period: 20, k: 3.0,
            };
        }
        case 'tight-bands': {
            const rand = lcg(57n);
            return {
                closes: Array.from({ length: 80 }, (_, i) =>
                    100 + Math.sin(i * 0.4) * 3 + (rand() - 0.5) * 1),
                period: 20, k: 1.0,
            };
        }
        default: return makeDemoInput('normal-trend');
    }
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtNum(v, d = 4) {
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
