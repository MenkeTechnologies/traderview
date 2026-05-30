// Anderson-Darling Normality Test (Stephens 1986) helpers.
//
// Backend body: { sample: number[] }
// Returns: { a_squared, a_squared_adjusted, reject_at_5pct, reject_at_1pct,
//   n_observations } | null
//
// Tests the null H0: data drawn from a normal distribution. Uses the full
// empirical CDF (vs Jarque-Bera which uses first 4 moments only) — more
// powerful at detecting tail-weight deviations. Stephens (1986) small-sample
// correction applied.

import { t } from './i18n.js';

export const MIN_OBS = 8;

export const DEFAULT_INPUTS = { sample: [] };

export function validateInputs(input) {
    if (!Array.isArray(input.sample))               return t('view.ad_normality.validate.sample_array');
    if (input.sample.length < MIN_OBS)              return t('view.ad_normality.validate.sample_min', { n: MIN_OBS });
    for (let i = 0; i < input.sample.length; i++) {
        if (!Number.isFinite(input.sample[i]))      return t('view.ad_normality.validate.sample_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return { sample: input.sample };
}

// Pure-JS mirror of crates/traderview-core/src/anderson_darling_normality.rs::test.
export function localTest(sample) {
    const n = sample.length;
    if (n < MIN_OBS) return null;
    for (const v of sample) if (!Number.isFinite(v)) return null;
    const sorted = [...sample].sort((a, b) => a - b);
    const n_f = n;
    let sum = 0;
    for (const v of sorted) sum += v;
    const mean = sum / n_f;
    let var_acc = 0;
    for (const v of sorted) var_acc += (v - mean) ** 2;
    const variance = var_acc / (n_f - 1);
    if (variance <= 0) return null;
    const sd = Math.sqrt(variance);
    const phi = sorted.map(x => clamp(standardNormalCdf((x - mean) / sd), 1e-12, 1 - 1e-12));
    let acc = 0;
    for (let i = 0; i < n; i++) {
        const i_f = i + 1;
        const z_i = phi[i];
        const z_ni = phi[n - 1 - i];
        acc += (2 * i_f - 1) * (Math.log(z_i) + Math.log(1 - z_ni));
    }
    const a_sq = -n_f - acc / n_f;
    const a_sq_adj = a_sq * (1 + 0.75 / n_f + 2.25 / (n_f * n_f));
    return {
        a_squared:           a_sq,
        a_squared_adjusted:  a_sq_adj,
        reject_at_5pct:      a_sq_adj > 0.752,
        reject_at_1pct:      a_sq_adj > 1.035,
        n_observations:      n,
    };
}

function clamp(x, lo, hi) { return Math.min(hi, Math.max(lo, x)); }

/// Inverse standard-normal CDF (probit). Beasley-Springer-Moro approximation;
/// accurate to about 4-5 decimals — enough for Q-Q plot visualization.
export function invStdNormalCdf(p) {
    if (!Number.isFinite(p) || p <= 0 || p >= 1) return NaN;
    const a = [
        -3.969683028665376e+01,  2.209460984245205e+02,
        -2.759285104469687e+02,  1.383577518672690e+02,
        -3.066479806614716e+01,  2.506628277459239e+00,
    ];
    const b = [
        -5.447609879822406e+01,  1.615858368580409e+02,
        -1.556989798598866e+02,  6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    const c = [
        -7.784894002430293e-03, -3.223964580411365e-01,
        -2.400758277161838e+00, -2.549732539343734e+00,
         4.374664141464968e+00,  2.938163982698783e+00,
    ];
    const d = [
        7.784695709041462e-03,  3.224671290700398e-01,
        2.445134137142996e+00,  3.754408661907416e+00,
    ];
    const plow  = 0.02425;
    const phigh = 1 - plow;
    if (p < plow) {
        const q = Math.sqrt(-2 * Math.log(p));
        return (((((c[0]*q + c[1])*q + c[2])*q + c[3])*q + c[4])*q + c[5])
             / ((((d[0]*q + d[1])*q + d[2])*q + d[3])*q + 1);
    }
    if (p <= phigh) {
        const q = p - 0.5;
        const r = q * q;
        return (((((a[0]*r + a[1])*r + a[2])*r + a[3])*r + a[4])*r + a[5]) * q
             / (((((b[0]*r + b[1])*r + b[2])*r + b[3])*r + b[4])*r + 1);
    }
    const q = Math.sqrt(-2 * Math.log(1 - p));
    return -(((((c[0]*q + c[1])*q + c[2])*q + c[3])*q + c[4])*q + c[5])
         / ((((d[0]*q + d[1])*q + d[2])*q + d[3])*q + 1);
}

/// Build (theoretical, empirical) quantile pairs for a Q-Q plot against
/// the standard normal. Uses Filliben's plotting positions p_i = (i-0.375)/(n+0.25)
/// for a robust mid-rank estimate. Returns { theoretical, empirical, n }
/// with theoretical sorted ascending; empirical is sample sorted ascending.
export function qqPlotData(sample) {
    if (!Array.isArray(sample) || sample.length < 2) {
        return { theoretical: [], empirical: [], n: 0 };
    }
    const sorted = [...sample].filter(Number.isFinite).sort((a, b) => a - b);
    const n = sorted.length;
    if (n < 2) return { theoretical: [], empirical: [], n };
    const theoretical = [];
    for (let i = 1; i <= n; i++) {
        const p = (i - 0.375) / (n + 0.25);
        theoretical.push(invStdNormalCdf(p));
    }
    return { theoretical, empirical: sorted, n };
}

export function standardNormalCdf(z) {
    return 0.5 * (1 + erf(z / Math.SQRT2));
}

// Abramowitz-Stegun-style erf approximation matching Rust impl.
export function erf(x) {
    const sign = x < 0 ? -1 : 1;
    x = Math.abs(x);
    const t = 1 / (1 + 0.327_591_1 * x);
    const y = 1 - (((((1.061_405_429 * t - 1.453_152_027) * t)
        + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * Math.exp(-x * x);
    return sign * y;
}

// Parse whitespace/comma-separated number list (sample observations).
export function parseSampleBlob(blob) {
    const out = { sample: [], errors: [] };
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
        const raw = tokens[i];
        let tok = raw;
        let neg = false;
        if (tok.startsWith('(') && tok.endsWith(')')) {
            neg = true;
            tok = tok.slice(1, -1);
        }
        tok = tok.replace(/[\$%]/g, '');
        const v = Number(tok);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${raw}" not finite` });
            continue;
        }
        out.sample.push(neg ? -v : v);
    }
    return out;
}

export function sampleToBlob(sample) {
    return sample.join('\n');
}

// 4-tier verdict tied to Stephens (1986) critical values.
export function verdictBadge(report) {
    if (!report) return { key: 'view.ad_norm.verdict.unknown', cls: '' };
    const a = report.a_squared_adjusted;
    if (!Number.isFinite(a)) return { key: 'view.ad_norm.verdict.unknown', cls: '' };
    if (a > 1.035) return { key: 'view.ad_norm.verdict.reject_strong', cls: 'neg' };
    if (a > 0.752) return { key: 'view.ad_norm.verdict.reject_5pct',   cls: 'neg' };
    if (a > 0.631) return { key: 'view.ad_norm.verdict.borderline',    cls: '' };
    return { key: 'view.ad_norm.verdict.normal', cls: 'pos' };
}

// p-value approximation using Marsaglia & Marsaglia (2004) approach for
// adjusted A² — gives a continuous tail-probability estimate instead of
// just bucket-bound verdicts. Approximation valid for n ≥ 8.
export function approxPValue(a_sq_adj) {
    if (!Number.isFinite(a_sq_adj) || a_sq_adj <= 0) return NaN;
    const A = a_sq_adj;
    if (A < 0.200) return 1 - Math.exp(-13.436 + 101.14 * A - 223.73 * A * A);
    if (A < 0.340) return 1 - Math.exp(-8.318 + 42.796 * A - 59.938 * A * A);
    if (A < 0.600) return Math.exp(0.9177 - 4.279 * A - 1.38 * A * A);
    if (A < 13.0)  return Math.exp(1.2937 - 5.709 * A + 0.0186 * A * A);
    return 0;
}

// Sample-side stats (raw distribution descriptors).
export function summarizeSample(sample) {
    if (!Array.isArray(sample) || sample.length === 0) {
        return { count: 0, mean: NaN, sd: NaN, skew: NaN, kurt: NaN, min: NaN, max: NaN };
    }
    const n = sample.length;
    let sum = 0, mx = -Infinity, mn = Infinity;
    for (const v of sample) {
        sum += v;
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    const mean = sum / n;
    let m2 = 0, m3 = 0, m4 = 0;
    for (const v of sample) {
        const d = v - mean;
        m2 += d * d;
        m3 += d * d * d;
        m4 += d * d * d * d;
    }
    const variance = n > 1 ? m2 / (n - 1) : 0;
    const sd = Math.sqrt(Math.max(0, variance));
    const skew = sd > 0 && n > 0 ? (m3 / n) / Math.pow(sd, 3) : NaN;
    const kurt = sd > 0 && n > 0 ? (m4 / n) / Math.pow(sd, 4) - 3 : NaN;
    return {
        count: n, mean, sd, skew, kurt,
        min: Number.isFinite(mn) ? mn : NaN,
        max: Number.isFinite(mx) ? mx : NaN,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function gaussian(rand) {
    let u1 = Math.max(1e-12, rand());
    let u2 = rand();
    return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
}

export function makeDemoInput(kind = 'gaussian') {
    switch (kind) {
        case 'gaussian': {
            const rand = lcg(42n);
            return { sample: Array.from({ length: 500 }, () => gaussian(rand)) };
        }
        case 'heavy-tail': {
            // 90% N(0,1), 10% N(0,25) → heavy tails, should reject at 1%.
            const rand = lcg(11n);
            return { sample: Array.from({ length: 800 }, () => {
                const z = gaussian(rand);
                return rand() < 0.1 ? z * 5 : z;
            }) };
        }
        case 'right-skew': {
            // |z| → half-normal, strongly skewed right.
            const rand = lcg(7n);
            return { sample: Array.from({ length: 500 }, () => Math.abs(gaussian(rand))) };
        }
        case 'left-skew': {
            const rand = lcg(7n);
            return { sample: Array.from({ length: 500 }, () => -Math.abs(gaussian(rand))) };
        }
        case 'uniform': {
            const rand = lcg(99n);
            return { sample: Array.from({ length: 500 }, () => rand() * 2 - 1) };
        }
        case 'bimodal': {
            const rand = lcg(13n);
            return { sample: Array.from({ length: 500 }, () => {
                const z = gaussian(rand);
                return rand() < 0.5 ? z - 3 : z + 3;
            }) };
        }
        case 'exponential': {
            const rand = lcg(21n);
            return { sample: Array.from({ length: 500 }, () => -Math.log(Math.max(1e-12, rand()))) };
        }
        case 'small-sample': {
            const rand = lcg(57n);
            return { sample: Array.from({ length: 12 }, () => gaussian(rand)) };
        }
        default: return makeDemoInput('gaussian');
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

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtPVal(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (v < 0.0001) return '< 0.0001';
    return v.toFixed(4);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
