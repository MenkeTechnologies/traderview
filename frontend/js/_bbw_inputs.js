// Bollinger Band Width (BBW) + %B helpers.
//
// Backend body: { closes: number[], period: usize, k: f64 }
// Returns: {
//   middle: (number|null)[],
//   upper:  (number|null)[],
//   lower:  (number|null)[],
//   band_width: (number|null)[],
//   percent_b:  (number|null)[],
// }

export const DEFAULT_PERIOD = 20;
export const DEFAULT_K = 2.0;
export const MIN_PERIOD = 2;
export const MAX_PERIOD = 500;

export const DEFAULT_INPUTS = {
    closes: [],
    period: DEFAULT_PERIOD,
    k: DEFAULT_K,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return 'closes must be an array';
    if (!Number.isInteger(input.period))                    return 'period must be an integer';
    if (input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                             return `period must be in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (!Number.isFinite(input.k) || input.k < 0)           return 'k must be non-negative finite';
    if (input.closes.length < input.period)                 return `need at least period (${input.period}) closes`;
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return `closes[${i}] not finite`;
    }
    return null;
}

export function buildBody(input) {
    return { closes: input.closes.slice(), period: input.period, k: input.k };
}

// Pure-JS mirror of crates/traderview-core/src/bollinger_band_width.rs::compute.
export function localCompute(closes, period, k) {
    const n = closes.length;
    const mid = new Array(n).fill(null);
    const up  = new Array(n).fill(null);
    const lo  = new Array(n).fill(null);
    const bbw = new Array(n).fill(null);
    const pb  = new Array(n).fill(null);
    if (period < 2 || n < period || !Number.isFinite(k) || k < 0) {
        return { middle: mid, upper: up, lower: lo, band_width: bbw, percent_b: pb };
    }
    for (let i = period - 1; i < n; i++) {
        // Window: closes[i+1-period .. i] inclusive.
        let allFinite = true;
        let sum = 0;
        for (let j = i + 1 - period; j <= i; j++) {
            const v = closes[j];
            if (!Number.isFinite(v)) { allFinite = false; break; }
            sum += v;
        }
        if (!allFinite) continue;
        const m = sum / period;
        let v_acc = 0;
        for (let j = i + 1 - period; j <= i; j++) v_acc += (closes[j] - m) ** 2;
        const variance = v_acc / period;
        const sd = Math.sqrt(Math.max(0, variance));
        const u = m + k * sd;
        const l = m - k * sd;
        mid[i] = m;
        up[i] = u;
        lo[i] = l;
        if (Math.abs(m) > 1e-18) bbw[i] = (u - l) / m;
        const cur = closes[i];
        const width = u - l;
        pb[i] = width > 0 ? (cur - l) / width : 0.5;
    }
    return { middle: mid, upper: up, lower: lo, band_width: bbw, percent_b: pb };
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

// Squeeze verdict on most recent BBW (compared with rolling history).
export function squeezeBadge(band_width, lookback = 60) {
    if (!Array.isArray(band_width) || band_width.length === 0) {
        return { key: 'view.bbw.squeeze.unknown', cls: '' };
    }
    const tail = [];
    for (let i = band_width.length - 1; i >= 0 && tail.length < lookback; i--) {
        const v = band_width[i];
        if (v != null && Number.isFinite(v)) tail.unshift(v);
    }
    if (tail.length < 5) return { key: 'view.bbw.squeeze.unknown', cls: '' };
    const last = tail[tail.length - 1];
    const sorted = [...tail].sort((a, b) => a - b);
    const rank = sorted.indexOf(last);
    const pctile = rank / (sorted.length - 1);
    if (pctile <= 0.10)  return { key: 'view.bbw.squeeze.tight',     cls: 'pos' };
    if (pctile <= 0.25)  return { key: 'view.bbw.squeeze.narrow',    cls: '' };
    if (pctile <= 0.75)  return { key: 'view.bbw.squeeze.normal',    cls: '' };
    if (pctile <= 0.90)  return { key: 'view.bbw.squeeze.expansion', cls: '' };
    return { key: 'view.bbw.squeeze.extreme', cls: 'neg' };
}

// %B position verdict.
export function percentBBadge(pb) {
    if (pb == null || !Number.isFinite(pb)) return { key: 'view.bbw.pb.unknown', cls: '' };
    if (pb >= 1.0)  return { key: 'view.bbw.pb.above_upper', cls: 'pos' };
    if (pb >= 0.8)  return { key: 'view.bbw.pb.near_upper',  cls: 'pos' };
    if (pb >= 0.55) return { key: 'view.bbw.pb.upper_half',  cls: 'pos' };
    if (pb >= 0.45) return { key: 'view.bbw.pb.middle',      cls: '' };
    if (pb >= 0.2)  return { key: 'view.bbw.pb.lower_half',  cls: 'neg' };
    if (pb >= 0)    return { key: 'view.bbw.pb.near_lower',  cls: 'neg' };
    return { key: 'view.bbw.pb.below_lower', cls: 'neg' };
}

// Width-trend verdict (expanding vs contracting).
export function widthTrendBadge(band_width, lookback = 10) {
    if (!Array.isArray(band_width) || band_width.length === 0) {
        return { key: 'view.bbw.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = band_width.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (band_width[i] != null && Number.isFinite(band_width[i])) tail.unshift(band_width[i]);
    }
    if (tail.length < 2) return { key: 'view.bbw.trend.unknown', cls: '' };
    const slope = tail[tail.length - 1] - tail[0];
    const range = Math.max(...tail) - Math.min(...tail);
    if (range === 0)              return { key: 'view.bbw.trend.steady',       cls: '' };
    if (slope > range * 0.5)       return { key: 'view.bbw.trend.expanding',   cls: '' };
    if (slope > range * 0.1)       return { key: 'view.bbw.trend.widening',    cls: '' };
    if (slope < -range * 0.5)      return { key: 'view.bbw.trend.contracting', cls: 'pos' };
    if (slope < -range * 0.1)      return { key: 'view.bbw.trend.narrowing',   cls: 'pos' };
    return { key: 'view.bbw.trend.steady', cls: '' };
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

export function makeDemoInput(kind = 'normal') {
    switch (kind) {
        case 'normal': {
            const rand = lcg(42n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.2 + (rand() - 0.5) * 2),
                period: 20, k: 2.0,
            };
        }
        case 'squeeze-then-break': {
            // Tight chop → big breakout (classic squeeze setup).
            const rand = lcg(7n);
            const closes = [];
            for (let i = 0; i < 30; i++) closes.push(100 + (rand() - 0.5) * 0.2);
            for (let i = 0; i < 40; i++) closes.push(100 + i * 0.7 + (rand() - 0.5) * 2);
            return { closes, period: 20, k: 2.0 };
        }
        case 'expansion-then-contract': {
            const rand = lcg(11n);
            const closes = [];
            for (let i = 0; i < 40; i++) closes.push(100 + Math.sin(i * 0.4) * 8 + (rand() - 0.5) * 2);
            for (let i = 0; i < 40; i++) closes.push(100 + (rand() - 0.5) * 0.3);
            return { closes, period: 20, k: 2.0 };
        }
        case 'trending-up': {
            const rand = lcg(13n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.8 + (rand() - 0.5) * 1),
                period: 20, k: 2.0,
            };
        }
        case 'trending-down': {
            const rand = lcg(21n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 180 - i * 0.8 + (rand() - 0.5) * 1),
                period: 20, k: 2.0,
            };
        }
        case 'walking-bands': {
            // Price stays near upper band — strong trend, %B > 0.8.
            const rand = lcg(33n);
            const closes = [];
            let p = 100;
            for (let i = 0; i < 80; i++) {
                p += 0.8;
                closes.push(p + (rand() - 0.5) * 0.4);
            }
            return { closes, period: 20, k: 2.0 };
        }
        case 'wide-bands': {
            // k=3 → wider bands; same series as normal.
            const rand = lcg(42n);
            return {
                closes: Array.from({ length: 80 }, (_, i) => 100 + i * 0.2 + (rand() - 0.5) * 2),
                period: 20, k: 3.0,
            };
        }
        case 'flat-window': {
            return {
                closes: new Array(40).fill(100),
                period: 20, k: 2.0,
            };
        }
        default: return makeDemoInput('normal');
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
