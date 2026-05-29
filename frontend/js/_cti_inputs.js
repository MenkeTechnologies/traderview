// Chande Trend Index (CTI) helpers — correlation of closes vs linear ramp.
//
// Backend body: { closes: number[], period: usize }
// Returns: (number|null)[]  — correlation in [−1, +1].

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 14;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return t('view.cti.validate.closes_array');
    if (!Number.isInteger(input.period))                    return t('view.cti.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                             return t('view.cti.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (input.closes.length < input.period)                 return t('view.cti.validate.closes_min', { period: input.period });
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return t('view.cti.validate.close_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return { closes: input.closes.slice(), period: input.period };
}

// Pure-JS mirror of crates/traderview-core/src/chande_trend_index.rs::compute.
export function localCompute(closes, period) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (period < 2 || n < period) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    const n_f = period;
    let sx = 0, sx2 = 0;
    for (let i = 1; i <= period; i++) { sx += i; sx2 += i * i; }
    const xbar = sx / n_f;
    const sxx = sx2 - n_f * xbar * xbar;
    if (sxx <= 0) return out;
    for (let i = period - 1; i < n; i++) {
        let ySum = 0;
        for (let j = i + 1 - period; j <= i; j++) ySum += closes[j];
        const ybar = ySum / n_f;
        let sxy = 0, syy = 0;
        for (let k = 0; k < period; k++) {
            const dx = (k + 1) - xbar;
            const dy = closes[i + 1 - period + k] - ybar;
            sxy += dx * dy;
            syy += dy * dy;
        }
        if (syy > 0) {
            const r = sxy / Math.sqrt(sxx * syy);
            out[i] = Math.max(-1, Math.min(1, r));
        } else {
            out[i] = 0;
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

// 7-tier trend-strength verdict on most recent CTI value.
export function strengthBadge(cti_last) {
    if (cti_last == null || !Number.isFinite(cti_last)) {
        return { key: 'view.cti.strength.unknown', cls: '' };
    }
    if (cti_last > 0.85)  return { key: 'view.cti.strength.perfect_up',  cls: 'pos' };
    if (cti_last > 0.5)   return { key: 'view.cti.strength.strong_up',   cls: 'pos' };
    if (cti_last > 0.2)   return { key: 'view.cti.strength.weak_up',     cls: 'pos' };
    if (cti_last > -0.2)  return { key: 'view.cti.strength.no_trend',    cls: '' };
    if (cti_last > -0.5)  return { key: 'view.cti.strength.weak_down',   cls: 'neg' };
    if (cti_last > -0.85) return { key: 'view.cti.strength.strong_down', cls: 'neg' };
    return { key: 'view.cti.strength.perfect_down', cls: 'neg' };
}

// Recent zero-cross detector (trend turn).
export function crossBadge(cti) {
    if (!Array.isArray(cti)) return { key: 'view.cti.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < cti.length; i++) {
        const v = cti[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 0 && v > 0)      { last_cross = 'up';   last_cross_idx = i; }
            else if (prev >= 0 && v < 0) { last_cross = 'down'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.cti.cross.none', cls: '' };
    const barsAgo = cti.length - 1 - last_cross_idx;
    if (last_cross === 'up') return { key: 'view.cti.cross.up_recent', cls: 'pos', barsAgo };
    return { key: 'view.cti.cross.down_recent', cls: 'neg', barsAgo };
}

// Trend strengthening / weakening over last N populated values.
export function changeBadge(cti, lookback = 10) {
    if (!Array.isArray(cti) || cti.length === 0) {
        return { key: 'view.cti.change.unknown', cls: '' };
    }
    const tail = [];
    for (let i = cti.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (cti[i] != null && Number.isFinite(cti[i])) tail.unshift(cti[i]);
    }
    if (tail.length < 2) return { key: 'view.cti.change.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    if (slope > 0.5)  return { key: 'view.cti.change.strengthening_up',   cls: 'pos' };
    if (slope > 0.1)  return { key: 'view.cti.change.firming_up',         cls: 'pos' };
    if (slope > -0.1) return { key: 'view.cti.change.stable',             cls: '' };
    if (slope > -0.5) return { key: 'view.cti.change.weakening',          cls: 'neg' };
    return { key: 'view.cti.change.strengthening_down', cls: 'neg' };
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
                closes: Array.from({ length: 60 }, (_, i) => 100 + i + (rand() - 0.5) * 0.5),
                period: 14,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                closes: Array.from({ length: 60 }, (_, i) => 160 - i + (rand() - 0.5) * 0.5),
                period: 14,
            };
        }
        case 'flat': {
            return { closes: new Array(40).fill(100), period: 14 };
        }
        case 'noisy-trend': {
            const rand = lcg(11n);
            return {
                closes: Array.from({ length: 60 }, (_, i) => 100 + i * 0.3 + (rand() - 0.5) * 3),
                period: 14,
            };
        }
        case 'oscillating': {
            return {
                closes: Array.from({ length: 60 }, (_, i) => 100 + Math.sin(i * 0.4) * 5),
                period: 14,
            };
        }
        case 'reversal': {
            const rand = lcg(13n);
            const c = [];
            for (let i = 0; i < 30; i++) c.push(100 + i + (rand() - 0.5) * 0.3);
            for (let i = 0; i < 30; i++) c.push(130 - i + (rand() - 0.5) * 0.3);
            return { closes: c, period: 14 };
        }
        case 'chop-then-trend': {
            const rand = lcg(21n);
            const c = [];
            for (let i = 0; i < 30; i++) c.push(100 + (rand() - 0.5) * 1.5);
            for (let i = 0; i < 30; i++) c.push(100 + i + (rand() - 0.5) * 0.5);
            return { closes: c, period: 14 };
        }
        case 'short-period': {
            const rand = lcg(33n);
            return {
                closes: Array.from({ length: 30 }, (_, i) => 100 + i + (rand() - 0.5) * 0.5),
                period: 5,
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
