// Breusch-Pagan Heteroskedasticity Test helpers.
//
// Backend body: { x: number[], y: number[] }  (paired regressor + response)
// Returns: {
//   lm_statistic, p_value, r_squared_auxiliary, n_observations,
//   reject_at_5pct, reject_at_1pct,
// } | null
//
// Tests H₀: OLS residual variance is independent of x (homoskedasticity).

export const MIN_OBS = 10;

export const DEFAULT_INPUTS = {
    x: [],
    y: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.x))                              return 'x must be an array';
    if (!Array.isArray(input.y))                              return 'y must be an array';
    if (input.x.length !== input.y.length)                    return 'x and y must have equal length';
    if (input.x.length < MIN_OBS)                             return `need at least ${MIN_OBS} pairs`;
    for (let i = 0; i < input.x.length; i++) {
        if (!Number.isFinite(input.x[i]))                     return `x[${i}] not finite`;
        if (!Number.isFinite(input.y[i]))                     return `y[${i}] not finite`;
    }
    return null;
}

export function buildBody(input) {
    return { x: input.x.slice(), y: input.y.slice() };
}

// Pure-JS mirror of crates/traderview-core/src/breusch_pagan_test.rs::test.
export function localTest(x, y) {
    const n = x.length;
    if (n < MIN_OBS || y.length !== n) return null;
    for (let i = 0; i < n; i++) {
        if (!Number.isFinite(x[i]) || !Number.isFinite(y[i])) return null;
    }
    const n_f = n;
    let xSum = 0, ySum = 0;
    for (let i = 0; i < n; i++) { xSum += x[i]; ySum += y[i]; }
    const x_mean = xSum / n_f;
    const y_mean = ySum / n_f;
    let sxx = 0, sxy = 0;
    for (let i = 0; i < n; i++) {
        sxx += (x[i] - x_mean) ** 2;
        sxy += (x[i] - x_mean) * (y[i] - y_mean);
    }
    if (sxx <= 0) return null;
    const beta = sxy / sxx;
    const alpha = y_mean - beta * x_mean;
    const resid_sq = new Array(n);
    for (let i = 0; i < n; i++) {
        const r = y[i] - alpha - beta * x[i];
        resid_sq[i] = r * r;
    }
    let rsSum = 0;
    for (const r of resid_sq) rsSum += r;
    const rs_mean = rsSum / n_f;
    let s_xx_aux = 0, s_xy_aux = 0;
    for (let i = 0; i < n; i++) {
        s_xx_aux += (x[i] - x_mean) ** 2;
        s_xy_aux += (x[i] - x_mean) * (resid_sq[i] - rs_mean);
    }
    if (s_xx_aux <= 0) return null;
    const gamma1 = s_xy_aux / s_xx_aux;
    const gamma0 = rs_mean - gamma1 * x_mean;
    let tss = 0, ssr = 0;
    for (let i = 0; i < n; i++) {
        tss += (resid_sq[i] - rs_mean) ** 2;
        const pred = gamma0 + gamma1 * x[i];
        ssr += (resid_sq[i] - pred) ** 2;
    }
    const r_sq = tss > 1e-18 ? 1 - ssr / tss : 0;
    const lm = n_f * Math.max(0, r_sq);
    const p_value = chiSquaredUpperTail(lm, 1);
    return {
        lm_statistic: lm,
        p_value,
        r_squared_auxiliary: r_sq,
        n_observations: n,
        reject_at_5pct: lm > 3.841,
        reject_at_1pct: lm > 6.635,
    };
}

export function chiSquaredUpperTail(x, k) {
    if (x <= 0 || k <= 0) return 1;
    const z = (Math.pow(x / k, 1/3) - (1 - 2 / (9 * k))) / Math.sqrt(2 / (9 * k));
    return 1 - standardNormalCdf(z);
}

export function standardNormalCdf(z) {
    return 0.5 * (1 + erf(z / Math.SQRT2));
}

export function erf(v) {
    const sign = v < 0 ? -1 : 1;
    v = Math.abs(v);
    const t = 1 / (1 + 0.327_591_1 * v);
    const y = 1 - (((((1.061_405_429 * t - 1.453_152_027) * t)
        + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * Math.exp(-v * v);
    return sign * y;
}

// Parse blob: "x y" pairs per line (2 tokens). Comments / blanks ignored.
export function parsePairsBlob(blob) {
    const out = { x: [], y: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length !== 2) {
            out.errors.push({ line_no: i + 1, message: `expected 2 tokens (x y), got ${parts.length}` });
            continue;
        }
        const xi = Number(parts[0].replace(/[\$,]/g, ''));
        const yi = Number(parts[1].replace(/[\$,]/g, ''));
        if (!Number.isFinite(xi) || !Number.isFinite(yi)) {
            out.errors.push({ line_no: i + 1, message: `x and y must be finite` });
            continue;
        }
        out.x.push(xi);
        out.y.push(yi);
    }
    return out;
}

export function pairsToBlob(x, y) {
    const len = Math.min(x.length, y.length);
    const lines = [];
    for (let i = 0; i < len; i++) lines.push(`${x[i]} ${y[i]}`);
    return lines.join('\n');
}

// 4-tier verdict on p-value.
export function verdictBadge(report) {
    if (!report) return { key: 'view.bp.verdict.unknown', cls: '' };
    const p = report.p_value;
    if (!Number.isFinite(p)) return { key: 'view.bp.verdict.unknown', cls: '' };
    if (p < 0.01) return { key: 'view.bp.verdict.strong_reject', cls: 'neg' };
    if (p < 0.05) return { key: 'view.bp.verdict.reject',        cls: 'neg' };
    if (p < 0.10) return { key: 'view.bp.verdict.borderline',    cls: '' };
    return { key: 'view.bp.verdict.homoskedastic', cls: 'pos' };
}

// Strength badge on auxiliary R² (0–1).
export function r2Badge(r_sq) {
    if (r_sq == null || !Number.isFinite(r_sq)) return { key: 'view.bp.r2.unknown', cls: '' };
    if (r_sq >= 0.20) return { key: 'view.bp.r2.very_strong', cls: 'neg' };
    if (r_sq >= 0.10) return { key: 'view.bp.r2.strong',      cls: 'neg' };
    if (r_sq >= 0.04) return { key: 'view.bp.r2.moderate',    cls: '' };
    if (r_sq >= 0.01) return { key: 'view.bp.r2.weak',        cls: '' };
    return { key: 'view.bp.r2.negligible', cls: 'pos' };
}

// Sample-size adequacy.
export function sampleBadge(n) {
    if (!Number.isInteger(n)) return { key: 'view.bp.sample.unknown', cls: '' };
    if (n >= 200) return { key: 'view.bp.sample.large',    cls: 'pos' };
    if (n >= 50)  return { key: 'view.bp.sample.medium',   cls: '' };
    if (n >= 10)  return { key: 'view.bp.sample.small',    cls: 'neg' };
    return { key: 'view.bp.sample.too_small', cls: 'neg' };
}

export function summarizeData(x, y) {
    const n = x.length;
    if (n === 0) return { n: 0, x_mean: NaN, y_mean: NaN, x_sd: NaN, y_sd: NaN };
    let xSum = 0, ySum = 0;
    for (let i = 0; i < n; i++) { xSum += x[i]; ySum += y[i]; }
    const x_mean = xSum / n;
    const y_mean = ySum / n;
    let xVar = 0, yVar = 0;
    for (let i = 0; i < n; i++) {
        xVar += (x[i] - x_mean) ** 2;
        yVar += (y[i] - y_mean) ** 2;
    }
    const dof = Math.max(1, n - 1);
    return {
        n,
        x_mean, y_mean,
        x_sd: Math.sqrt(xVar / dof),
        y_sd: Math.sqrt(yVar / dof),
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'homoskedastic') {
    switch (kind) {
        case 'homoskedastic': {
            // Constant-variance residuals — should fail to reject.
            const rand = lcg(42n);
            const x = Array.from({ length: 300 }, (_, i) => i);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * 1.0);
            return { x, y };
        }
        case 'variance-increasing-in-x': {
            // Variance ∝ x → classic heteroskedasticity.
            const rand = lcg(11n);
            const x = Array.from({ length: 300 }, (_, i) => i + 1);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * (xi / 30));
            return { x, y };
        }
        case 'variance-decreasing-in-x': {
            const rand = lcg(13n);
            const x = Array.from({ length: 300 }, (_, i) => i + 1);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * (10 / xi));
            return { x, y };
        }
        case 'v-shape-variance': {
            // Variance bigger at extremes, tight in the middle.
            const rand = lcg(21n);
            const x = Array.from({ length: 300 }, (_, i) => i - 150);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * (Math.abs(xi) / 10 + 0.5));
            return { x, y };
        }
        case 'narrow-then-wide': {
            // First half tight, second half loose.
            const rand = lcg(33n);
            const x = Array.from({ length: 200 }, (_, i) => i);
            const y = x.map((xi, i) => 2 * xi + (rand() - 0.5) * (i < 100 ? 0.5 : 5));
            return { x, y };
        }
        case 'small-sample': {
            const rand = lcg(57n);
            const x = Array.from({ length: 12 }, (_, i) => i);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * 1.0);
            return { x, y };
        }
        case 'returns-vs-vol': {
            // Daily returns regressed on lagged price → near-zero R² in aux.
            const rand = lcg(99n);
            const x = [];
            const y = [];
            let p = 100;
            for (let i = 0; i < 250; i++) {
                p += (rand() - 0.5) * 0.5;
                x.push(p);
                y.push((rand() - 0.5) * 2);
            }
            return { x, y };
        }
        case 'extreme-spike-residuals': {
            // Mostly tight, but occasional jumps create huge residuals at specific x.
            const rand = lcg(77n);
            const x = Array.from({ length: 200 }, (_, i) => i);
            const y = x.map((xi, i) => 2 * xi + (i % 25 === 0 ? (rand() - 0.5) * 30 : (rand() - 0.5) * 0.5));
            return { x, y };
        }
        default: return makeDemoInput('homoskedastic');
    }
}

export function fmtNum(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPVal(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (v < 0.0001) return '< 0.0001';
    return v.toFixed(4);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
