// Chande Momentum Oscillator (CMO) helpers.
//
// Backend body: { closes: number[], period: usize }
// Returns: (number|null)[]  — 100 × (SoU − SoD) / (SoU + SoD) ∈ [−100, +100].

export const DEFAULT_PERIOD = 14;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return 'closes must be an array';
    if (!Number.isInteger(input.period))                    return 'period must be an integer';
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                             return `period must be in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (input.closes.length < input.period + 1)             return `need at least period + 1 = ${input.period + 1} closes`;
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return `closes[${i}] not finite`;
    }
    return null;
}

export function buildBody(input) {
    return { closes: input.closes.slice(), period: input.period };
}

// Pure-JS mirror of crates/traderview-core/src/chande_momentum_oscillator.rs::compute.
export function localCompute(closes, period) {
    const n = closes.length;
    const out = new Array(n).fill(null);
    if (period < 2 || n < period + 1) return out;
    for (const v of closes) if (!Number.isFinite(v)) return out;
    for (let i = period; i < n; i++) {
        let sou = 0, sod = 0;
        for (let k = i - period + 1; k <= i; k++) {
            const d = closes[k] - closes[k - 1];
            if (d > 0) sou += d;
            else       sod -= d;
        }
        const denom = sou + sod;
        out[i] = denom > 0 ? 100 * (sou - sod) / denom : 0;
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

// 7-tier zone verdict (centered around 0, classic CMO overbought/oversold).
export function zoneBadge(cmo_last) {
    if (cmo_last == null || !Number.isFinite(cmo_last)) {
        return { key: 'view.cmo.zone.unknown', cls: '' };
    }
    if (cmo_last > 75)  return { key: 'view.cmo.zone.extreme_overbought', cls: 'neg' };
    if (cmo_last > 50)  return { key: 'view.cmo.zone.overbought',         cls: 'neg' };
    if (cmo_last > 20)  return { key: 'view.cmo.zone.bullish_lean',       cls: 'pos' };
    if (cmo_last > -20) return { key: 'view.cmo.zone.neutral',            cls: '' };
    if (cmo_last > -50) return { key: 'view.cmo.zone.bearish_lean',       cls: 'neg' };
    if (cmo_last > -75) return { key: 'view.cmo.zone.oversold',           cls: 'pos' };
    return { key: 'view.cmo.zone.extreme_oversold', cls: 'pos' };
}

// Recent threshold crossing (±50 OB/OS lines + zero-line cross).
export function crossBadge(cmo) {
    if (!Array.isArray(cmo)) return { key: 'view.cmo.cross.unknown', cls: '' };
    let prev = null;
    let last_cross = null;
    let last_cross_idx = -1;
    for (let i = 0; i < cmo.length; i++) {
        const v = cmo[i];
        if (v == null || !Number.isFinite(v)) continue;
        if (prev != null) {
            if (prev <= 50  && v > 50)        { last_cross = 'into_overbought'; last_cross_idx = i; }
            else if (prev >= 50  && v < 50)   { last_cross = 'out_of_overbought'; last_cross_idx = i; }
            else if (prev >= -50 && v < -50)  { last_cross = 'into_oversold'; last_cross_idx = i; }
            else if (prev <= -50 && v > -50)  { last_cross = 'out_of_oversold'; last_cross_idx = i; }
            else if (prev <= 0   && v > 0)    { last_cross = 'zero_up'; last_cross_idx = i; }
            else if (prev >= 0   && v < 0)    { last_cross = 'zero_down'; last_cross_idx = i; }
        }
        prev = v;
    }
    if (last_cross == null) return { key: 'view.cmo.cross.none', cls: '' };
    const barsAgo = cmo.length - 1 - last_cross_idx;
    const map = {
        into_overbought:   { key: 'view.cmo.cross.into_overbought',   cls: 'neg' },
        out_of_overbought: { key: 'view.cmo.cross.out_of_overbought', cls: 'neg' },
        into_oversold:     { key: 'view.cmo.cross.into_oversold',     cls: 'pos' },
        out_of_oversold:   { key: 'view.cmo.cross.out_of_oversold',   cls: 'pos' },
        zero_up:           { key: 'view.cmo.cross.zero_up',           cls: 'pos' },
        zero_down:         { key: 'view.cmo.cross.zero_down',         cls: 'neg' },
    };
    return { ...map[last_cross], barsAgo };
}

// Trend over last N populated values.
export function trendBadge(cmo, lookback = 10) {
    if (!Array.isArray(cmo) || cmo.length === 0) {
        return { key: 'view.cmo.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = cmo.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (cmo[i] != null && Number.isFinite(cmo[i])) tail.unshift(cmo[i]);
    }
    if (tail.length < 2) return { key: 'view.cmo.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.cmo.trend.flat',          cls: '' };
    if (slope > range * 0.5)       return { key: 'view.cmo.trend.rising_fast',  cls: 'pos' };
    if (slope > range * 0.1)       return { key: 'view.cmo.trend.rising',       cls: 'pos' };
    if (slope < -range * 0.5)      return { key: 'view.cmo.trend.falling_fast', cls: 'neg' };
    if (slope < -range * 0.1)      return { key: 'view.cmo.trend.falling',      cls: 'neg' };
    return { key: 'view.cmo.trend.flat', cls: '' };
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
                closes: Array.from({ length: 80 }, (_, i) => 100 + i + (rand() - 0.5) * 0.3),
                period: 14,
            };
        }
        case 'downtrend': {
            const rand = lcg(7n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 180 - i + (rand() - 0.5) * 0.3),
                period: 14,
            };
        }
        case 'flat': {
            return { closes: new Array(40).fill(100), period: 14 };
        }
        case 'alternating': {
            return {
                closes: Array.from({ length: 50 }, (_, i) => 100 + (i % 2)),
                period: 14,
            };
        }
        case 'oscillating': {
            const rand = lcg(11n);
            return {
                closes: Array.from({ length: 100 }, (_, i) => 100 + Math.sin(i * 0.4) * 5 + (rand() - 0.5) * 0.3),
                period: 14,
            };
        }
        case 'reversal-up': {
            const rand = lcg(13n);
            const c = [];
            for (let i = 0; i < 40; i++) c.push(140 - i + (rand() - 0.5) * 0.3);
            for (let i = 0; i < 40; i++) c.push(100 + i + (rand() - 0.5) * 0.3);
            return { closes: c, period: 14 };
        }
        case 'reversal-down': {
            const rand = lcg(21n);
            const c = [];
            for (let i = 0; i < 40; i++) c.push(100 + i + (rand() - 0.5) * 0.3);
            for (let i = 0; i < 40; i++) c.push(140 - i + (rand() - 0.5) * 0.3);
            return { closes: c, period: 14 };
        }
        case 'short-period': {
            const rand = lcg(33n);
            return {
                closes: Array.from({ length: 30 }, (_, i) => 100 + i * 0.5 + (rand() - 0.5) * 0.5),
                period: 5,
            };
        }
        default: return makeDemoInput('uptrend');
    }
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtNumSigned(v, d = 2) {
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
