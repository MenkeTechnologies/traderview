// Borrow Rate Indicator helpers — annualized hard-to-borrow stress gauge.
//
// Backend body: { rates_pct: number[], period: usize }
// Returns: {
//   change_pct: (number|null)[],
//   stress:     (string|null)[]   // 'low_available' | 'normal' | 'tight' | 'hard_to_borrow' | 'extreme_squeeze'
//   period:     usize,
// }

export const DEFAULT_PERIOD = 5;
export const MIN_PERIOD = 1;
export const MAX_PERIOD = 500;

export const STRESS_LEVELS = [
    'low_available', 'normal', 'tight', 'hard_to_borrow', 'extreme_squeeze',
];

export const DEFAULT_INPUTS = {
    rates_pct: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.rates_pct))                    return 'rates_pct must be an array';
    if (!Number.isInteger(input.period) || input.period < MIN_PERIOD || input.period > MAX_PERIOD)
                                                             return `period must be integer in [${MIN_PERIOD}, ${MAX_PERIOD}]`;
    if (input.rates_pct.length < input.period + 1)          return `need at least period + 1 = ${input.period + 1} rates`;
    for (let i = 0; i < input.rates_pct.length; i++) {
        if (!Number.isFinite(input.rates_pct[i]))           return `rates_pct[${i}] not finite`;
        if (input.rates_pct[i] < 0)                         return `rates_pct[${i}] cannot be negative`;
    }
    return null;
}

export function buildBody(input) {
    return { rates_pct: input.rates_pct.slice(), period: input.period };
}

// Pure-JS mirror of crates/traderview-core/src/borrow_rate_indicator.rs::compute.
export function localCompute(rates_pct, period) {
    const n = rates_pct.length;
    const report = {
        change_pct: new Array(n).fill(null),
        stress:     new Array(n).fill(null),
        period,
    };
    if (period < 1 || n < period + 1) return report;
    for (const v of rates_pct) {
        if (!Number.isFinite(v) || v < 0) return report;
    }
    for (let i = 0; i < n; i++) {
        const cur = rates_pct[i];
        let change_pct = null;
        if (i >= period) {
            const prev = rates_pct[i - period];
            if (prev > 0) change_pct = (cur - prev) / prev * 100;
        }
        report.change_pct[i] = change_pct;
        report.stress[i] = classify(cur, change_pct == null ? 0 : change_pct);
    }
    return report;
}

export function classify(rate, change_pct) {
    if (rate >= 200 || change_pct >= 100) return 'extreme_squeeze';
    if (rate >= 50)                        return 'hard_to_borrow';
    if (rate >= 10)                        return 'tight';
    if (rate >= 1)                         return 'normal';
    return 'low_available';
}

// Parse non-negative rates blob.
export function parseRatesBlob(blob) {
    const out = { rates_pct: [], errors: [] };
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
        const v = Number(tokens[i].replace(/[\$%,]/g, ''));
        if (!Number.isFinite(v) || v < 0) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not non-negative finite` });
            continue;
        }
        out.rates_pct.push(v);
    }
    return out;
}

export function ratesToBlob(rates) {
    return rates.join('\n');
}

// Last-bar stress verdict with color hinting.
export function stressBadge(stress_last) {
    if (stress_last == null) return { key: 'view.borrow.stress.unknown', cls: '' };
    const map = {
        low_available:    { key: 'view.borrow.stress.low_available',    cls: 'pos' },
        normal:           { key: 'view.borrow.stress.normal',           cls: '' },
        tight:            { key: 'view.borrow.stress.tight',            cls: 'neg' },
        hard_to_borrow:   { key: 'view.borrow.stress.hard_to_borrow',   cls: 'neg' },
        extreme_squeeze:  { key: 'view.borrow.stress.extreme_squeeze',  cls: 'neg' },
    };
    return map[stress_last] || { key: 'view.borrow.stress.unknown', cls: '' };
}

// Trend over last N populated change_pct.
export function trendBadge(change_pct, lookback = 5) {
    if (!Array.isArray(change_pct) || change_pct.length === 0) {
        return { key: 'view.borrow.trend.unknown', cls: '' };
    }
    const tail = [];
    for (let i = change_pct.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (change_pct[i] != null && Number.isFinite(change_pct[i])) tail.unshift(change_pct[i]);
    }
    if (tail.length === 0) return { key: 'view.borrow.trend.unknown', cls: '' };
    const last = tail[tail.length - 1];
    if (last >= 50)   return { key: 'view.borrow.trend.spiking', cls: 'neg' };
    if (last >= 10)   return { key: 'view.borrow.trend.rising',  cls: 'neg' };
    if (last > -10)   return { key: 'view.borrow.trend.steady',  cls: '' };
    if (last > -50)   return { key: 'view.borrow.trend.easing',  cls: 'pos' };
    return { key: 'view.borrow.trend.collapsing', cls: 'pos' };
}

// Most-recent transition: did stress escalate vs N bars ago?
export function escalationBadge(stress, lookback = 5) {
    if (!Array.isArray(stress) || stress.length < 2) {
        return { key: 'view.borrow.escal.unknown', cls: '' };
    }
    const tail = [];
    for (let i = stress.length - 1; i >= 0 && tail.length < lookback; i--) {
        if (stress[i] != null) tail.unshift(stress[i]);
    }
    if (tail.length < 2) return { key: 'view.borrow.escal.unknown', cls: '' };
    const first = STRESS_LEVELS.indexOf(tail[0]);
    const last  = STRESS_LEVELS.indexOf(tail[tail.length - 1]);
    if (last > first + 1)  return { key: 'view.borrow.escal.sharp_escalation', cls: 'neg' };
    if (last > first)      return { key: 'view.borrow.escal.escalating',       cls: 'neg' };
    if (last < first - 1)  return { key: 'view.borrow.escal.sharp_relief',     cls: 'pos' };
    if (last < first)      return { key: 'view.borrow.escal.easing',           cls: 'pos' };
    return { key: 'view.borrow.escal.stable', cls: '' };
}

// Distribution of stress levels across the entire series.
export function stressDistribution(stress) {
    const counts = { low_available: 0, normal: 0, tight: 0, hard_to_borrow: 0, extreme_squeeze: 0 };
    if (!Array.isArray(stress)) return counts;
    for (const s of stress) if (s != null && counts[s] !== undefined) counts[s]++;
    return counts;
}

export function summarizeRates(rates_pct) {
    if (!Array.isArray(rates_pct) || rates_pct.length === 0) {
        return { count: 0, last: NaN, min: NaN, max: NaN, mean: NaN };
    }
    let sum = 0, mx = -Infinity, mn = Infinity;
    for (const v of rates_pct) {
        sum += v;
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    return {
        count: rates_pct.length,
        last: rates_pct[rates_pct.length - 1],
        min: Number.isFinite(mn) ? mn : NaN,
        max: Number.isFinite(mx) ? mx : NaN,
        mean: sum / rates_pct.length,
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
            return { rates_pct: Array.from({ length: 60 }, () => 2 + rand() * 3), period: 5 };
        }
        case 'gradually-escalating': {
            const rand = lcg(7n);
            return {
                rates_pct: Array.from({ length: 60 }, (_, i) => 1 + i * 0.4 + rand() * 0.5),
                period: 5,
            };
        }
        case 'sudden-spike': {
            const rand = lcg(11n);
            const rates = Array.from({ length: 30 }, () => 3 + rand() * 1);
            for (let i = 0; i < 5; i++) rates.push(15 + i * 2 + rand() * 2);
            for (let i = 0; i < 25; i++) rates.push(25 + rand() * 3);
            return { rates_pct: rates, period: 5 };
        }
        case 'extreme-squeeze': {
            // Spike to over 200% rate.
            const rand = lcg(13n);
            const rates = Array.from({ length: 30 }, () => 5 + rand() * 2);
            for (let i = 0; i < 20; i++) rates.push(50 + i * 10 + rand() * 5);
            return { rates_pct: rates, period: 5 };
        }
        case 'easy-borrow': {
            // GTC easy-borrow — near-constant tiny rate so % change stays under 100%.
            const rand = lcg(21n);
            return { rates_pct: Array.from({ length: 60 }, () => 0.50 + (rand() - 0.5) * 0.05), period: 5 };
        }
        case 'oscillating': {
            const rand = lcg(33n);
            return {
                rates_pct: Array.from({ length: 60 }, (_, i) => 10 + Math.sin(i * 0.4) * 5 + rand() * 1),
                period: 5,
            };
        }
        case 'spike-and-relax': {
            const rand = lcg(57n);
            const rates = Array.from({ length: 20 }, () => 3 + rand() * 1);
            for (let i = 0; i < 5; i++) rates.push(20 + i * 5 + rand() * 2);   // spike
            for (let i = 0; i < 30; i++) rates.push(Math.max(1, 50 - i * 1.5 + rand() * 1));   // relax
            return { rates_pct: rates, period: 5 };
        }
        case 'short-period': {
            const rand = lcg(99n);
            return {
                rates_pct: Array.from({ length: 30 }, (_, i) => 5 + i * 0.5 + rand() * 1),
                period: 2,
            };
        }
        default: return makeDemoInput('normal');
    }
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function fmtPctSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d) + '%';
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
