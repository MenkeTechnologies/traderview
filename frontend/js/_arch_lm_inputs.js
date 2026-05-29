// Engle's ARCH-LM test helpers.
//
// Backend body: { returns: number[], lags: usize }
// Returns: { lm_statistic, r_squared, lags, n_observations } | null
//
// Tests H₀: returns have constant variance vs H₁: ARCH effects (time-
// varying variance). LM = (n−q) · R² ~ χ²(q) under H₀. Reject when LM
// exceeds the tabulated critical value (q=5 at 5% → 11.07).

export const DEFAULT_LAGS = 5;
export const MIN_LAGS = 1;
export const MAX_LAGS = 50;

export const DEFAULT_INPUTS = { returns: [], lags: DEFAULT_LAGS };

export function validateInputs(input) {
    if (!Array.isArray(input.returns))               return 'returns must be an array';
    if (!Number.isInteger(input.lags))               return 'lags must be an integer';
    if (input.lags < MIN_LAGS || input.lags > MAX_LAGS) return `lags must be in [${MIN_LAGS}, ${MAX_LAGS}]`;
    const need = 3 * input.lags + 2;
    if (input.returns.length < need)                 return `returns needs ≥ 3·lags + 2 = ${need} obs`;
    for (let i = 0; i < input.returns.length; i++) {
        if (!Number.isFinite(input.returns[i]))      return `returns[${i}] not finite`;
    }
    return null;
}

export function buildBody(input) {
    return { returns: input.returns, lags: input.lags };
}

// Pure-JS mirror of crates/traderview-core/src/arch_lm_test.rs::test.
export function localTest(returns, lags) {
    const n = returns.length;
    if (n < 3 * lags + 2 || lags === 0) return null;
    for (const r of returns) if (!Number.isFinite(r)) return null;
    let sum = 0;
    for (const r of returns) sum += r;
    const mean = sum / n;
    const e_sq = returns.map(r => (r - mean) ** 2);
    const start = lags;
    const m = n - start;
    if (m < lags + 2) return null;
    const x_intercept = new Array(m);
    const x_lags = Array.from({ length: lags }, () => new Array(m));
    const y = new Array(m);
    for (let k = 0; k < m; k++) {
        const t = start + k;
        x_intercept[k] = 1;
        for (let i = 0; i < lags; i++) x_lags[i][k] = e_sq[t - 1 - i];
        y[k] = e_sq[t];
    }
    const cols = [x_intercept, ...x_lags];
    const beta = ols(cols, y);
    if (!beta) return null;
    let ss_res = 0;
    for (let k = 0; k < m; k++) {
        let yh = 0;
        for (let i = 0; i < cols.length; i++) yh += beta[i] * cols[i][k];
        ss_res += (y[k] - yh) ** 2;
    }
    let y_sum = 0;
    for (const v of y) y_sum += v;
    const y_mean = y_sum / m;
    let ss_tot = 0;
    for (const v of y) ss_tot += (v - y_mean) ** 2;
    if (ss_tot <= 0) return null;
    const r2 = 1 - ss_res / ss_tot;
    const lm = m * r2;
    return {
        lm_statistic:   lm,
        r_squared:      r2,
        lags,
        n_observations: m,
    };
}

// OLS by Gauss-Jordan with partial pivoting — mirrors Rust impl bit-for-bit.
export function ols(x, y) {
    const p = x.length;
    const n = y.length;
    if (p === 0 || n === 0) return null;
    for (const col of x) if (col.length !== n) return null;
    const xtx = Array.from({ length: p }, () => new Array(p).fill(0));
    const xty = new Array(p).fill(0);
    for (let i = 0; i < p; i++) {
        for (let j = 0; j < p; j++) {
            let s = 0;
            for (let k = 0; k < n; k++) s += x[i][k] * x[j][k];
            xtx[i][j] = s;
        }
        let s = 0;
        for (let k = 0; k < n; k++) s += x[i][k] * y[k];
        xty[i] = s;
    }
    const aug = Array.from({ length: p }, () => new Array(p + 1).fill(0));
    for (let i = 0; i < p; i++) {
        for (let j = 0; j < p; j++) aug[i][j] = xtx[i][j];
        aug[i][p] = xty[i];
    }
    for (let i = 0; i < p; i++) {
        let pivot = i;
        for (let r = i + 1; r < p; r++) {
            if (Math.abs(aug[r][i]) > Math.abs(aug[pivot][i])) pivot = r;
        }
        if (Math.abs(aug[pivot][i]) < 1e-18) return null;
        if (pivot !== i) { const tmp = aug[i]; aug[i] = aug[pivot]; aug[pivot] = tmp; }
        const div = aug[i][i];
        for (let j = 0; j < p + 1; j++) aug[i][j] /= div;
        for (let r = 0; r < p; r++) {
            if (r === i) continue;
            const f = aug[r][i];
            if (f === 0) continue;
            const pivot_row = aug[i].slice();
            for (let j = 0; j < p + 1; j++) aug[r][j] -= f * pivot_row[j];
        }
    }
    return aug.map(row => row[p]);
}

// Chi-squared critical values: returns table[lag] → {alpha10, alpha5, alpha1}.
// Tabulated values for k=1..15 lags. For higher k, use null and rely on LM
// vs lag heuristic instead.
const CHI2_CRIT = {
    1:  { a10:  2.706, a5:  3.841, a1:  6.635 },
    2:  { a10:  4.605, a5:  5.991, a1:  9.210 },
    3:  { a10:  6.251, a5:  7.815, a1: 11.345 },
    4:  { a10:  7.779, a5:  9.488, a1: 13.277 },
    5:  { a10:  9.236, a5: 11.070, a1: 15.086 },
    6:  { a10: 10.645, a5: 12.592, a1: 16.812 },
    7:  { a10: 12.017, a5: 14.067, a1: 18.475 },
    8:  { a10: 13.362, a5: 15.507, a1: 20.090 },
    9:  { a10: 14.684, a5: 16.919, a1: 21.666 },
    10: { a10: 15.987, a5: 18.307, a1: 23.209 },
    11: { a10: 17.275, a5: 19.675, a1: 24.725 },
    12: { a10: 18.549, a5: 21.026, a1: 26.217 },
    13: { a10: 19.812, a5: 22.362, a1: 27.688 },
    14: { a10: 21.064, a5: 23.685, a1: 29.141 },
    15: { a10: 22.307, a5: 24.996, a1: 30.578 },
};

export function chi2Critical(lags) {
    if (CHI2_CRIT[lags]) return CHI2_CRIT[lags];
    // Wilson-Hilferty cube-root approximation for k > 15.
    const k = lags;
    const z = (p) => {  // approximate inverse standard normal for one-sided p.
        const tbl = { 0.10: 1.2816, 0.05: 1.6449, 0.01: 2.3263 };
        return tbl[p];
    };
    const crit = (alpha) => {
        const zv = z(alpha);
        return k * Math.pow(1 - 2 / (9 * k) + zv * Math.sqrt(2 / (9 * k)), 3);
    };
    return { a10: crit(0.10), a5: crit(0.05), a1: crit(0.01) };
}

// Wilson-Hilferty chi-squared SURVIVAL function approx → p-value.
export function chi2PValue(lm, lags) {
    if (!Number.isFinite(lm) || lm <= 0 || !Number.isFinite(lags) || lags <= 0) return NaN;
    const z = (Math.pow(lm / lags, 1/3) - (1 - 2 / (9 * lags))) / Math.sqrt(2 / (9 * lags));
    return 1 - standardNormalCdf(z);
}

export function standardNormalCdf(z) {
    return 0.5 * (1 + erf(z / Math.SQRT2));
}

export function erf(x) {
    const sign = x < 0 ? -1 : 1;
    x = Math.abs(x);
    const t = 1 / (1 + 0.327_591_1 * x);
    const y = 1 - (((((1.061_405_429 * t - 1.453_152_027) * t)
        + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * Math.exp(-x * x);
    return sign * y;
}

// 4-tier verdict using χ²(lags) critical values.
export function verdictBadge(report) {
    if (!report) return { key: 'view.arch_lm.verdict.unknown', cls: '' };
    const c = chi2Critical(report.lags);
    const lm = report.lm_statistic;
    if (!Number.isFinite(lm)) return { key: 'view.arch_lm.verdict.unknown', cls: '' };
    if (lm > c.a1)  return { key: 'view.arch_lm.verdict.strong',     cls: 'neg' };
    if (lm > c.a5)  return { key: 'view.arch_lm.verdict.moderate',   cls: 'neg' };
    if (lm > c.a10) return { key: 'view.arch_lm.verdict.borderline', cls: '' };
    return { key: 'view.arch_lm.verdict.no_arch', cls: 'pos' };
}

// Magnitude verdict on R² of the auxiliary regression.
export function r2Badge(r2) {
    if (r2 == null || !Number.isFinite(r2)) return { key: 'view.arch_lm.r2.unknown', cls: '' };
    if (r2 >= 0.20) return { key: 'view.arch_lm.r2.strong',   cls: 'neg' };
    if (r2 >= 0.05) return { key: 'view.arch_lm.r2.moderate', cls: '' };
    if (r2 >= 0.01) return { key: 'view.arch_lm.r2.weak',     cls: '' };
    return { key: 'view.arch_lm.r2.none', cls: 'pos' };
}

// Parse whitespace/comma-separated returns.
export function parseReturnsBlob(blob) {
    const out = { returns: [], errors: [] };
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
        const raw = tokens[i];
        let tok = raw;
        let neg = false;
        if (tok.startsWith('(') && tok.endsWith(')')) { neg = true; tok = tok.slice(1, -1); }
        tok = tok.replace(/[\$%]/g, '');
        const v = Number(tok);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${raw}" not finite` });
            continue;
        }
        out.returns.push(neg ? -v : v);
    }
    return out;
}

export function returnsToBlob(returns) {
    return returns.join('\n');
}

// Per-series stats.
export function summarizeReturns(returns) {
    if (!Array.isArray(returns) || returns.length === 0) {
        return { count: 0, mean: NaN, sd: NaN, min: NaN, max: NaN };
    }
    const n = returns.length;
    let sum = 0, mx = -Infinity, mn = Infinity;
    for (const v of returns) {
        sum += v;
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    const mean = sum / n;
    let v_acc = 0;
    for (const v of returns) v_acc += (v - mean) ** 2;
    const variance = n > 1 ? v_acc / (n - 1) : 0;
    const sd = Math.sqrt(Math.max(0, variance));
    return {
        count: n, mean, sd,
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
    const u1 = Math.max(1e-12, rand());
    const u2 = rand();
    return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
}

export function makeDemoInput(kind = 'arch-strong') {
    switch (kind) {
        case 'arch-strong': {
            // ARCH(1): σ²ₜ = 0.01 + 0.8 · r²ₜ₋₁.
            const rand = lcg(42n);
            const n = 1000;
            const r = new Array(n).fill(0);
            for (let t = 1; t < n; t++) {
                const variance = 0.01 + 0.8 * r[t - 1] ** 2;
                r[t] = Math.sqrt(variance) * gaussian(rand);
            }
            return { returns: r, lags: 5 };
        }
        case 'arch-mild': {
            const rand = lcg(11n);
            const n = 800;
            const r = new Array(n).fill(0);
            for (let t = 1; t < n; t++) {
                const variance = 0.01 + 0.3 * r[t - 1] ** 2;
                r[t] = Math.sqrt(variance) * gaussian(rand);
            }
            return { returns: r, lags: 5 };
        }
        case 'garch-like': {
            // GARCH(1,1)-ish: variance persists via own lag.
            const rand = lcg(7n);
            const n = 800;
            const r = new Array(n).fill(0);
            let sigma2 = 0.01;
            for (let t = 1; t < n; t++) {
                sigma2 = 0.01 + 0.1 * r[t - 1] ** 2 + 0.85 * sigma2;
                r[t] = Math.sqrt(sigma2) * gaussian(rand);
            }
            return { returns: r, lags: 5 };
        }
        case 'iid-gauss': {
            const rand = lcg(99n);
            return { returns: Array.from({ length: 800 }, () => 0.01 * gaussian(rand)), lags: 5 };
        }
        case 'iid-laplace': {
            const rand = lcg(13n);
            const r = Array.from({ length: 800 }, () => {
                const u = rand() - 0.5;
                return -Math.sign(u) * Math.log(1 - 2 * Math.abs(u)) * 0.01;
            });
            return { returns: r, lags: 5 };
        }
        case 'short-memory-vol': {
            // Variance switches via Bernoulli (regime).
            const rand = lcg(21n);
            const n = 800;
            const r = new Array(n);
            for (let t = 0; t < n; t++) {
                const sigma = rand() < 0.3 ? 0.05 : 0.01;
                r[t] = sigma * gaussian(rand);
            }
            return { returns: r, lags: 5 };
        }
        case 'few-obs': {
            // Just above 3·lags+2 minimum.
            const rand = lcg(33n);
            return { returns: Array.from({ length: 25 }, () => 0.01 * gaussian(rand)), lags: 3 };
        }
        case 'high-lags': {
            // Test χ²(10) critical value.
            const rand = lcg(57n);
            const n = 600;
            const r = new Array(n).fill(0);
            for (let t = 1; t < n; t++) {
                const variance = 0.01 + 0.5 * r[t - 1] ** 2;
                r[t] = Math.sqrt(variance) * gaussian(rand);
            }
            return { returns: r, lags: 10 };
        }
        default: return makeDemoInput('arch-strong');
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
