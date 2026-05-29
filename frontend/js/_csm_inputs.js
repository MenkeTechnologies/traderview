// Centered Smoothed Momentum (Ehlers) helpers.
//
// Backend body: { closes: number[], momentum_period: usize, smooth_period: usize }
// Returns: (number|null)[]  — SuperSmoother-filtered momentum series.

import { t } from './i18n.js';

export const DEFAULT_MOMENTUM = 10;
export const DEFAULT_SMOOTH = 8;
export const MIN_MOMENTUM = 1;
export const MAX_MOMENTUM = 500;
export const MIN_SMOOTH = 4;
export const MAX_SMOOTH = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    momentum_period: DEFAULT_MOMENTUM,
    smooth_period: DEFAULT_SMOOTH,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return t('view.csm.validate.closes_array');
    if (!Number.isInteger(input.momentum_period)
        || input.momentum_period < MIN_MOMENTUM || input.momentum_period > MAX_MOMENTUM)
                                                             return t('view.csm.validate.momentum_range', { min: MIN_MOMENTUM, max: MAX_MOMENTUM });
    if (!Number.isInteger(input.smooth_period)
        || input.smooth_period < MIN_SMOOTH || input.smooth_period > MAX_SMOOTH)
                                                             return t('view.csm.validate.smooth_range', { min: MIN_SMOOTH, max: MAX_SMOOTH });
    if (input.closes.length < input.momentum_period + 3)    return t('view.csm.validate.closes_min', { min: input.momentum_period + 3 });
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return t('view.csm.validate.close_not_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        closes: input.closes.slice(),
        momentum_period: input.momentum_period,
        smooth_period:   input.smooth_period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/ehlers_centered_smoothed_momentum.rs::compute.
export function localCompute(closes, momentum_period, smooth_period) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (momentum_period < 1 || smooth_period < 4 || n < momentum_period + 3) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    const mom = new Array(n).fill(0);
    for (let i = momentum_period; i < n; i++) {
        mom[i] = closes[i] - closes[i - momentum_period];
    }
    const pi = Math.PI;
    const a1 = Math.exp(-1.414 * pi / smooth_period);
    const b1 = 2 * a1 * Math.cos(1.414 * pi / smooth_period);
    const c2 = b1;
    const c3 = -a1 * a1;
    const c1 = 1 - c2 - c3;
    const ss = new Array(n).fill(0);
    for (let i = momentum_period + 2; i < n; i++) {
        ss[i] = c1 * (mom[i] + mom[i - 1]) / 2
              + c2 * ss[i - 1]
              + c3 * ss[i - 2];
        out[i] = ss[i];
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

// 5-tier momentum verdict from CSM value sign + magnitude.
export function momentumBadge(csm_last) {
    if (csm_last == null || !Number.isFinite(csm_last)) {
        return { key: 'view.csm.mom.unknown', cls: '' };
    }
    if (csm_last > 10)   return { key: 'view.csm.mom.strong_up',   cls: 'pos' };
    if (csm_last > 1)    return { key: 'view.csm.mom.up',          cls: 'pos' };
    if (csm_last > -1)   return { key: 'view.csm.mom.neutral',     cls: '' };
    if (csm_last > -10)  return { key: 'view.csm.mom.down',        cls: 'neg' };
    return { key: 'view.csm.mom.strong_down', cls: 'neg' };
}

// Trend over last N populated values.
export function trendBadge(csm, lookback = 10) {
    if (!Array.isArray(csm) || csm.length === 0) {
        return { key: 'view.csm.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = csm.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (csm[i] != null && Number.isFinite(csm[i])) tail.unshift(csm[i]);
    }
    if (tail.length < 2) return { key: 'view.csm.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.csm.trend.flat',          cls: '' };
    if (slope > range * 0.5)       return { key: 'view.csm.trend.rising_fast',  cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.csm.trend.rising',       cls: 'pos' };
    if (slope < -range * 0.5)      return { key: 'view.csm.trend.falling_fast', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.csm.trend.falling',      cls: 'neg' };
    return { key: 'view.csm.trend.flat', cls: '' };
}

// Recent zero-cross detector (CSM crossing through zero = trend turn).
export function crossBadge(csm) {
    if (!Array.isArray(csm)) return { key: 'view.csm.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < csm.length; i++) {
        const v = csm[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 0 && v > 0)      { last_cross = 'up';   last_cross_idx = i; }
            else if (prev >= 0 && v < 0) { last_cross = 'down'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.csm.cross.none', cls: '' };
    const barsAgo = csm.length - 1 - last_cross_idx;
    if (last_cross === 'up') return { key: 'view.csm.cross.up_recent',   cls: 'pos', barsAgo };
    return { key: 'view.csm.cross.down_recent', cls: 'neg', barsAgo };
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

export function makeDemoInput(kind = 'uptrend') {
    switch (kind) {
        case 'uptrend': {
            const rand = lcg(42n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 100 + i + (rand() - 0.5) * 0.5),
                momentum_period: 10, smooth_period: 8,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 180 - i + (rand() - 0.5) * 0.5),
                momentum_period: 10, smooth_period: 8,
            };
        }
        case 'sideways': {
            const rand = lcg(11n);
            return {
                closes: Array.from({ length: 80 }, () => 100 + (rand() - 0.5) * 2),
                momentum_period: 10, smooth_period: 8,
            };
        }
        case 'reversal-up': {
            const rand = lcg(13n);
            const closes = [];
            for (let i = 0; i < 40; i++) closes.push(140 - i + (rand() - 0.5) * 0.5);
            for (let i = 0; i < 40; i++) closes.push(100 + i + (rand() - 0.5) * 0.5);
            return { closes, momentum_period: 10, smooth_period: 8 };
        }
        case 'reversal-down': {
            const rand = lcg(21n);
            const closes = [];
            for (let i = 0; i < 40; i++) closes.push(100 + i + (rand() - 0.5) * 0.5);
            for (let i = 0; i < 40; i++) closes.push(140 - i + (rand() - 0.5) * 0.5);
            return { closes, momentum_period: 10, smooth_period: 8 };
        }
        case 'oscillating': {
            const rand = lcg(33n);
            return {
                closes: Array.from({ length: 100 }, (_, i) => 100 + Math.sin(i * 0.3) * 10 + (rand() - 0.5) * 0.5),
                momentum_period: 10, smooth_period: 8,
            };
        }
        case 'short-smooth': {
            const rand = lcg(57n);
            return {
                closes: Array.from({ length: 60 }, (_, i) => 100 + i * 0.5 + (rand() - 0.5) * 1),
                momentum_period: 5, smooth_period: 4,
            };
        }
        case 'long-momentum': {
            const rand = lcg(99n);
            return {
                closes: Array.from({ length: 120 }, (_, i) => 100 + i * 0.7 + (rand() - 0.5) * 1),
                momentum_period: 25, smooth_period: 15,
            };
        }
        default: return makeDemoInput('uptrend');
    }
}

export function fmtNum(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtNumSigned(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
