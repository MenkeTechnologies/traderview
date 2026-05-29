// Bollinger %B (standalone) helpers.
//
// Backend body: { closes: number[], period: usize, n_stdev: f64 }
// Returns: (number|null)[]  — %B per bar.
//
// %B = (close - lower) / (upper - lower). %B at upper = 1.0; at midline = 0.5;
// at lower = 0.0; >1 = above upper band (breakout); <0 = below lower band.

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
    if (!Array.isArray(input.closes))                       return 'closes must be an array';
    if (!Number.isInteger(input.period))                    return 'period must be an integer';
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                             return `period must be in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (!Number.isFinite(input.n_stdev) || input.n_stdev <= 0)
                                                             return 'n_stdev must be positive finite';
    if (input.closes.length < input.period)                 return `need at least period (${input.period}) closes`;
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return `closes[${i}] not finite`;
    }
    return null;
}

export function buildBody(input) {
    return { closes: input.closes.slice(), period: input.period, n_stdev: input.n_stdev };
}

// Pure-JS mirror of crates/traderview-core/src/bollinger_percent_b.rs::compute.
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
        const band_range = 2 * n_stdev * std;
        if (band_range > 0) {
            const lower = mean - n_stdev * std;
            out[i] = (closes[i] - lower) / band_range;
        } else {
            out[i] = 0.5;
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

// 7-tier zone verdict.
export function zoneBadge(pb_last) {
    if (pb_last == null || !Number.isFinite(pb_last)) {
        return { key: 'view.bbpb.zone.unknown', cls: '' };
    }
    if (pb_last >= 1.0)  return { key: 'view.bbpb.zone.above_upper', cls: 'pos' };
    if (pb_last >= 0.8)  return { key: 'view.bbpb.zone.near_upper',  cls: 'pos' };
    if (pb_last >= 0.55) return { key: 'view.bbpb.zone.upper_half',  cls: 'pos' };
    if (pb_last >= 0.45) return { key: 'view.bbpb.zone.middle',      cls: '' };
    if (pb_last >= 0.2)  return { key: 'view.bbpb.zone.lower_half',  cls: 'neg' };
    if (pb_last >= 0)    return { key: 'view.bbpb.zone.near_lower',  cls: 'neg' };
    return { key: 'view.bbpb.zone.below_lower', cls: 'neg' };
}

// Signal: most recent zero-cross (overbought→neutral or oversold→neutral).
export function crossBadge(pb) {
    if (!Array.isArray(pb)) return { key: 'view.bbpb.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < pb.length; i++) {
        const v = pb[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            // Upper-band cross (close moving up past 1.0).
            if (prev <= 1.0 && v > 1.0) { last_cross = 'breakout';  last_cross_idx = i; }
            else if (prev >= 1.0 && v < 1.0) { last_cross = 'returned_below_upper'; last_cross_idx = i; }
            else if (prev >= 0.0 && v < 0.0) { last_cross = 'breakdown'; last_cross_idx = i; }
            else if (prev <= 0.0 && v > 0.0) { last_cross = 'returned_above_lower'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.bbpb.cross.none', cls: '' };
    const barsAgo = pb.length - 1 - last_cross_idx;
    const map = {
        breakout:              'view.bbpb.cross.breakout',
        breakdown:             'view.bbpb.cross.breakdown',
        returned_below_upper:  'view.bbpb.cross.returned_below_upper',
        returned_above_lower:  'view.bbpb.cross.returned_above_lower',
    };
    const cls = (last_cross === 'breakout' || last_cross === 'returned_above_lower') ? 'pos' : 'neg';
    return { key: map[last_cross], cls, barsAgo };
}

// Trend over last N populated values.
export function trendBadge(pb, lookback = 10) {
    if (!Array.isArray(pb) || pb.length === 0) {
        return { key: 'view.bbpb.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = pb.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (pb[i] != null && Number.isFinite(pb[i])) tail.unshift(pb[i]);
    }
    if (tail.length < 2) return { key: 'view.bbpb.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.bbpb.trend.flat',         cls: '' };
    if (slope > range * 0.5)       return { key: 'view.bbpb.trend.rising_fast', cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.bbpb.trend.rising',      cls: 'pos' };
    if (slope < -range * 0.5)      return { key: 'view.bbpb.trend.falling_fast', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.bbpb.trend.falling',     cls: 'neg' };
    return { key: 'view.bbpb.trend.flat', cls: '' };
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

export function makeDemoInput(kind = 'walking-up') {
    switch (kind) {
        case 'walking-up': {
            // Sustained uptrend → %B near 1.
            const rand = lcg(42n);
            const closes = [100];
            for (let i = 1; i < 80; i++) closes.push(closes[i - 1] + 0.6 + (rand() - 0.5) * 0.2);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'walking-down': {
            const rand = lcg(7n);
            const closes = [180];
            for (let i = 1; i < 80; i++) closes.push(closes[i - 1] - 0.6 + (rand() - 0.5) * 0.2);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'oscillating': {
            // %B cycles between extremes.
            const rand = lcg(11n);
            return {
                closes: Array.from({ length: 80 }, (_, i) =>
                    100 + Math.sin(i * 0.5) * 5 + (rand() - 0.5) * 0.5),
                period: 20, n_stdev: 2.0,
            };
        }
        case 'breakout-up': {
            // Flat then sharp spike → %B > 1.
            const rand = lcg(13n);
            const closes = [];
            for (let i = 0; i < 30; i++) closes.push(100 + (rand() - 0.5) * 0.5);
            for (let i = 0; i < 20; i++) closes.push(100 + i * 0.8 + (rand() - 0.5) * 0.3);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'breakdown': {
            const rand = lcg(21n);
            const closes = [];
            for (let i = 0; i < 30; i++) closes.push(100 + (rand() - 0.5) * 0.5);
            for (let i = 0; i < 20; i++) closes.push(100 - i * 0.8 + (rand() - 0.5) * 0.3);
            return { closes, period: 20, n_stdev: 2.0 };
        }
        case 'mean-revert': {
            // Whippy noise around 100.
            const rand = lcg(33n);
            return {
                closes: Array.from({ length: 80 }, () => 100 + (rand() - 0.5) * 3),
                period: 20, n_stdev: 2.0,
            };
        }
        case 'flat': {
            return {
                closes: new Array(30).fill(100),
                period: 20, n_stdev: 2.0,
            };
        }
        case 'tight-bands': {
            // k = 1.0 → tighter bands, %B saturates faster.
            const rand = lcg(57n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.5 + (rand() - 0.5) * 1),
                period: 20, n_stdev: 1.0,
            };
        }
        default: return makeDemoInput('walking-up');
    }
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPb(v, d = 4) {
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
