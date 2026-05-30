// Bollinger Bandwidth Percentile (BBWP) helpers.
//
// Backend body: { closes: number[], bb_period: usize, n_stdev: f64, lookback: usize }
// Returns: (number|null)[]  — percentile rank [0, 100] of current BBW in
// its own rolling lookback window. <10 = compression / squeeze;
// >90 = expansion.

import { t } from './i18n.js';

export const DEFAULT_BB_PERIOD = 20;
export const DEFAULT_N_STDEV = 2.0;
export const DEFAULT_LOOKBACK = 252;
export const MIN_BB_PERIOD = 2;
export const MAX_BB_PERIOD = 500;
export const MIN_LOOKBACK = 2;
export const MAX_LOOKBACK = 2000;

export const DEFAULT_INPUTS = {
    closes: [],
    bb_period: DEFAULT_BB_PERIOD,
    n_stdev: DEFAULT_N_STDEV,
    lookback: DEFAULT_LOOKBACK,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                          return t('view.bbwp.validate.closes_array');
    if (!Number.isInteger(input.bb_period))                    return t('view.bbwp.validate.bb_period_int');
    if (input.bb_period < MIN_BB_PERIOD || input.bb_period > MAX_BB_PERIOD)
                                                                return t('view.bbwp.validate.bb_period_range', { min: MIN_BB_PERIOD, max: MAX_BB_PERIOD });
    if (!Number.isFinite(input.n_stdev) || input.n_stdev <= 0) return t('view.bbwp.validate.n_stdev_pos');
    if (!Number.isInteger(input.lookback))                     return t('view.bbwp.validate.lookback_int');
    if (input.lookback < MIN_LOOKBACK || input.lookback > MAX_LOOKBACK)
                                                                return t('view.bbwp.validate.lookback_range', { min: MIN_LOOKBACK, max: MAX_LOOKBACK });
    if (input.lookback < input.bb_period)                      return t('view.bbwp.validate.lookback_ge_bb', { lookback: input.lookback, bb_period: input.bb_period });
    if (input.closes.length < input.lookback)                  return t('view.bbwp.validate.closes_min_lookback', { lookback: input.lookback });
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))                 return t('view.bbwp.validate.close_not_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        closes: input.closes.slice(),
        bb_period: input.bb_period,
        n_stdev: input.n_stdev,
        lookback: input.lookback,
    };
}

// Pure-JS mirror of crates/traderview-core/src/bollinger_bandwidth_percentile.rs::compute.
export function localCompute(closes, bb_period, n_stdev, lookback) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (bb_period < 2 || lookback < bb_period
        || !Number.isFinite(n_stdev) || n_stdev <= 0 || n < lookback) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    const p_f = bb_period;
    const width = new Array(n).fill(null);
    for (let i = bb_period - 1; i < n; i++) {
        let sum = 0;
        for (let j = i + 1 - bb_period; j <= i; j++) sum += closes[j];
        const mean = sum / p_f;
        let v_acc = 0;
        for (let j = i + 1 - bb_period; j <= i; j++) v_acc += (closes[j] - mean) ** 2;
        const variance = v_acc / p_f;
        const std = Math.sqrt(Math.max(0, variance));
        if (Math.abs(mean) > 0) {
            width[i] = 2 * n_stdev * std / Math.abs(mean) * 100;
        }
    }
    for (let i = lookback - 1; i < n; i++) {
        let anyNull = false;
        for (let j = i + 1 - lookback; j <= i; j++) {
            if (width[j] == null) { anyNull = true; break; }
        }
        if (anyNull) continue;
        const cur = width[i];
        let count = 0;
        for (let j = i + 1 - lookback; j <= i; j++) {
            if (width[j] <= cur) count++;
        }
        out[i] = count / lookback * 100;
    }
    return out;
}

// Parse positive prices, whitespace/comma-separated.
export function parseClosesBlob(blob) {
    const out = { closes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
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

// 6-tier regime verdict from BBWP value.
export function regimeBadge(bbwp_last) {
    if (bbwp_last == null || !Number.isFinite(bbwp_last)) {
        return { key: 'view.bbwp.regime.unknown', cls: '' };
    }
    if (bbwp_last <= 5)   return { key: 'view.bbwp.regime.extreme_squeeze', cls: 'pos' };
    if (bbwp_last <= 20)  return { key: 'view.bbwp.regime.squeeze',         cls: 'pos' };
    if (bbwp_last <= 40)  return { key: 'view.bbwp.regime.low',             cls: '' };
    if (bbwp_last <= 60)  return { key: 'view.bbwp.regime.neutral',         cls: '' };
    if (bbwp_last <= 80)  return { key: 'view.bbwp.regime.elevated',        cls: '' };
    if (bbwp_last <= 95)  return { key: 'view.bbwp.regime.expansion',       cls: 'neg' };
    return { key: 'view.bbwp.regime.extreme_expansion', cls: 'neg' };
}

// Direction verdict over last N populated values.
export function trendBadge(bbwp, lookback = 10) {
    if (!Array.isArray(bbwp) || bbwp.length === 0) {
        return { key: 'view.bbwp.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = bbwp.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (bbwp[i] != null && Number.isFinite(bbwp[i])) tail.unshift(bbwp[i]);
    }
    if (tail.length < 2) return { key: 'view.bbwp.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.bbwp.trend.flat',          cls: '' };
    if (slope > range * 0.5)       return { key: 'view.bbwp.trend.rising_fast',  cls: 'neg' };
    if (slope > range * 0.1)       return { key: 'view.bbwp.trend.rising',       cls: '' };
    if (slope < -range * 0.5)      return { key: 'view.bbwp.trend.falling_fast', cls: 'pos' };
    if (slope < -range * 0.1)      return { key: 'view.bbwp.trend.falling',      cls: '' };
    return { key: 'view.bbwp.trend.flat', cls: '' };
}

// Recent compression-trigger badge: did BBWP recently bottom (≤ 10) and turn up?
export function triggerBadge(bbwp, lookback = 20) {
    if (!Array.isArray(bbwp) || bbwp.length === 0) {
        return { key: 'view.bbwp.trigger.unknown', cls: '' };
    }
    const tail = [];
    for (let i = bbwp.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (bbwp[i] != null && Number.isFinite(bbwp[i])) tail.unshift(bbwp[i]);
    }
    if (tail.length < 3) return { key: 'view.bbwp.trigger.unknown', cls: '' };
    const last = tail[tail.length - 1];
    const min  = Math.min(...tail);
    if (min <= 10 && last > min + 5 && last < 50) {
        return { key: 'view.bbwp.trigger.firing', cls: 'pos' };
    }
    if (min <= 10) return { key: 'view.bbwp.trigger.armed', cls: '' };
    return { key: 'view.bbwp.trigger.none', cls: '' };
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

export function makeDemoInput(kind = 'rising-vol') {
    switch (kind) {
        case 'rising-vol': {
            // 252 quiet bars, 50 volatile bars at end → BBWP climbs.
            const rand = lcg(42n);
            const closes = [];
            for (let i = 0; i < 252; i++) closes.push(100 + (rand() - 0.5) * 0.5);
            for (let i = 0; i < 50;  i++) closes.push(100 + (rand() - 0.5) * 10);
            return { closes, bb_period: 20, n_stdev: 2.0, lookback: 252 };
        }
        case 'squeeze-end': {
            // First half noisy, second half tight → BBWP drops at end.
            const rand = lcg(7n);
            const closes = [];
            for (let i = 0; i < 150; i++) closes.push(100 + (rand() - 0.5) * 8);
            for (let i = 0; i < 130; i++) closes.push(100 + (rand() - 0.5) * 0.3);
            return { closes, bb_period: 20, n_stdev: 2.0, lookback: 252 };
        }
        case 'oscillating': {
            // Volatility cycles → BBWP oscillates.
            const rand = lcg(11n);
            const closes = [];
            for (let i = 0; i < 300; i++) {
                const vol = (Math.sin(i / 30) + 1) * 0.5 + 0.1;   // 0.1..1.1 range
                closes.push(100 + (rand() - 0.5) * vol * 8);
            }
            return { closes, bb_period: 20, n_stdev: 2.0, lookback: 252 };
        }
        case 'steady': {
            // Random walk with constant vol → BBWP wanders around 50.
            const rand = lcg(13n);
            const closes = [100];
            for (let i = 1; i < 280; i++) closes.push(closes[i - 1] + (rand() - 0.5) * 1.5);
            return { closes, bb_period: 20, n_stdev: 2.0, lookback: 252 };
        }
        case 'flat': {
            // Need ≥ lookback + bb_period − 1 = 271 bars to populate any BBWP.
            return {
                closes: new Array(290).fill(100),
                bb_period: 20, n_stdev: 2.0, lookback: 252,
            };
        }
        case 'short-lookback': {
            const rand = lcg(33n);
            const closes = [];
            for (let i = 0; i < 80; i++) closes.push(100 + (rand() - 0.5) * 0.4);
            for (let i = 0; i < 30; i++) closes.push(100 + (rand() - 0.5) * 4);
            return { closes, bb_period: 10, n_stdev: 2.0, lookback: 60 };
        }
        case 'high-stdev': {
            // k = 3 → wider bands, larger BBW magnitudes (% ranks similar).
            const rand = lcg(57n);
            const closes = [];
            for (let i = 0; i < 252; i++) closes.push(100 + (rand() - 0.5) * 1.0);
            for (let i = 0; i < 30;  i++) closes.push(100 + (rand() - 0.5) * 6);
            return { closes, bb_period: 20, n_stdev: 3.0, lookback: 252 };
        }
        case 'spike-and-mean-revert': {
            const rand = lcg(99n);
            const closes = [];
            for (let i = 0; i < 200; i++) closes.push(100 + (rand() - 0.5) * 0.4);
            for (let i = 0; i < 20;  i++) closes.push(100 + (rand() - 0.5) * 8);  // spike
            for (let i = 0; i < 60;  i++) closes.push(100 + (rand() - 0.5) * 0.4); // revert
            return { closes, bb_period: 20, n_stdev: 2.0, lookback: 252 };
        }
        default: return makeDemoInput('rising-vol');
    }
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function fmtNum(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
