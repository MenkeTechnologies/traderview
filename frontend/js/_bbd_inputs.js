// Bollinger Band Distance (BBD) helpers.
//
// Backend body: { closes: number[], period: usize, n_stdev: f64 }
// Returns: (number|null)[]  — min(|c-upper|, |c-lower|) / band_width per bar.
// Output ∈ [0, 0.5] when close is between bands; > 0.5 only via extreme
// breakouts (close further from nearer band than half the width).
//
// 0 = close at band; 0.5 = close at midline; large values = extreme break.

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 20;
export const DEFAULT_N_STDEV = 2.0;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    period: DEFAULT_PERIOD,
    n_stdev: DEFAULT_N_STDEV,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return t('view.bbd.validate.closes_array');
    if (!Number.isInteger(input.period))                    return t('view.bbd.validate.period_int');
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                             return t('view.bbd.validate.period_range', { min: MIN_PERIOD, max: MAX_PERIOD });
    if (!Number.isFinite(input.n_stdev) || input.n_stdev <= 0)
                                                             return t('view.bbd.validate.n_stdev_pos');
    if (input.closes.length < input.period)                 return t('view.bbd.validate.closes_min_period', { period: input.period });
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return t('view.bbd.validate.close_not_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return { closes: input.closes.slice(), period: input.period, n_stdev: input.n_stdev };
}

// Pure-JS mirror of crates/traderview-core/src/bollinger_band_distance.rs::compute.
export function localCompute(closes, period, n_stdev) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (period < 2 || !Number.isFinite(n_stdev) || n_stdev <= 0 || n < period) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    const p_f = period;
    for (let i = period - 1; i < n; i++) {
        let sum = 0;
        for (let j = i + 1 - period; j <= i; j++) sum += closes[j];
        const mean = sum / p_f;
        let v_acc = 0;
        for (let j = i + 1 - period; j <= i; j++) v_acc += (closes[j] - mean) ** 2;
        const variance = v_acc / p_f;
        const std = Math.sqrt(Math.max(0, variance));
        const band_width = 2 * n_stdev * std;
        if (band_width > 0) {
            const upper = mean + n_stdev * std;
            const lower = mean - n_stdev * std;
            const dist = Math.min(Math.abs(closes[i] - upper), Math.abs(closes[i] - lower));
            out[i] = dist / band_width;
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

// Position verdict: where is close relative to nearer band?
// d ≈ 0.5 → at midline (most neutral)
// d ≈ 0 → at one of the bands (extreme position)
// d > 0.5 → outside bands (breakout / breakdown)
export function positionBadge(d_last) {
    if (d_last == null || !Number.isFinite(d_last)) {
        return { key: 'view.bbd.pos.unknown', cls: '' };
    }
    if (d_last > 0.5)   return { key: 'view.bbd.pos.outside_band', cls: 'neg' };
    if (d_last >= 0.45) return { key: 'view.bbd.pos.midline',      cls: '' };
    if (d_last >= 0.30) return { key: 'view.bbd.pos.mid_zone',     cls: '' };
    if (d_last >= 0.15) return { key: 'view.bbd.pos.toward_band',  cls: '' };
    if (d_last >= 0.05) return { key: 'view.bbd.pos.near_band',    cls: 'pos' };
    return { key: 'view.bbd.pos.at_band', cls: 'pos' };
}

// Trend over last N populated values.
export function trendBadge(bbd, lookback = 10) {
    if (!Array.isArray(bbd) || bbd.length === 0) {
        return { key: 'view.bbd.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = bbd.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (bbd[i] != null && Number.isFinite(bbd[i])) tail.unshift(bbd[i]);
    }
    if (tail.length < 2) return { key: 'view.bbd.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.bbd.trend.flat',                cls: '' };
    if (slope > range * 0.5)       return { key: 'view.bbd.trend.toward_midline_fast', cls: '' };
    if (slope > range * 0.1)       return { key: 'view.bbd.trend.toward_midline',     cls: '' };
    if (slope < -range * 0.5)      return { key: 'view.bbd.trend.toward_band_fast',   cls: 'pos' };
    if (slope < -range * 0.1)      return { key: 'view.bbd.trend.toward_band',        cls: 'pos' };
    return { key: 'view.bbd.trend.flat', cls: '' };
}

// Detection of "kiss" — recent visit to a band (dist ≤ 0.05 in window).
export function kissBadge(bbd, lookback = 5) {
    if (!Array.isArray(bbd) || bbd.length === 0) {
        return { key: 'view.bbd.kiss.unknown', cls: '' };
    }
    const tail = [];
    for (let i = bbd.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (bbd[i] != null && Number.isFinite(bbd[i])) tail.unshift(bbd[i]);
    }
    if (tail.length === 0) return { key: 'view.bbd.kiss.unknown', cls: '' };
    const min = Math.min(...tail);
    if (min < 0.001) return { key: 'view.bbd.kiss.touched',     cls: 'pos' };
    if (min < 0.05)  return { key: 'view.bbd.kiss.kissed_band', cls: 'pos' };
    if (min < 0.15)  return { key: 'view.bbd.kiss.approached',  cls: '' };
    return { key: 'view.bbd.kiss.no_visit', cls: '' };
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

export function makeDemoInput(kind = 'oscillating') {
    switch (kind) {
        case 'oscillating': {
            // %B cycles → BBD also oscillates between 0 (band) and 0.5 (mid).
            const rand = lcg(42n);
            return {
                closes: Array.from({ length: 80 }, (_, i) =>
                    100 + Math.sin(i * 0.5) * 5 + (rand() - 0.5) * 0.5),
                period: 20, n_stdev: 2.0,
            };
        }
        case 'midline-walk': {
            // Random walk that stays near midline → BBD near 0.5.
            const rand = lcg(7n);
            const closes = [100];
            for (let i = 1; i < 80; i++) closes.push(closes[i - 1] + (rand() - 0.5) * 0.3);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'band-walking': {
            // Trending price hugs upper band → BBD near 0.
            const rand = lcg(11n);
            const closes = [100];
            for (let i = 1; i < 80; i++) closes.push(closes[i - 1] + 0.6 + (rand() - 0.5) * 0.1);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'breakout-up': {
            // Flat → spike past upper band → BBD spikes > 0.5 then settles.
            const rand = lcg(13n);
            const closes = [];
            for (let i = 0; i < 30; i++) closes.push(100 + (rand() - 0.5) * 0.4);
            for (let i = 0; i < 20; i++) closes.push(100 + i * 1.0 + (rand() - 0.5) * 0.3);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'breakdown': {
            const rand = lcg(21n);
            const closes = [];
            for (let i = 0; i < 30; i++) closes.push(100 + (rand() - 0.5) * 0.4);
            for (let i = 0; i < 20; i++) closes.push(100 - i * 1.0 + (rand() - 0.5) * 0.3);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'wide-bands': {
            // k=3 → wider bands; BBD seldom small.
            const rand = lcg(33n);
            return {
                closes: Array.from({ length: 80 }, (_, i) =>
                    100 + Math.sin(i * 0.4) * 2 + (rand() - 0.5) * 0.5),
                period: 20, n_stdev: 3.0,
            };
        }
        case 'tight-bands': {
            // k=1 → tighter bands; close more often near/outside bands.
            const rand = lcg(57n);
            return {
                closes: Array.from({ length: 80 }, (_, i) =>
                    100 + Math.sin(i * 0.4) * 2 + (rand() - 0.5) * 0.5),
                period: 20, n_stdev: 1.0,
            };
        }
        case 'flat': {
            return {
                closes: new Array(40).fill(100),
                period: 20, n_stdev: 2.0,
            };
        }
        default: return makeDemoInput('oscillating');
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
