// Breusch-Godfrey Serial Correlation LM Test helpers.
//
// Backend body: { x: number[], y: number[], lag_order: usize }
// Returns: {
//   lm_statistic, p_value, r_squared_auxiliary, lag_order,
//   n_observations, reject_at_5pct,
// } | null

import { t } from './i18n.js';

export const DEFAULT_LAG = 4;
export const MIN_LAG = 1;
export const MAX_LAG = 50;

export const DEFAULT_INPUTS = {
    x: [],
    y: [],
    lag_order: DEFAULT_LAG,
};

export function validateInputs(input) {
    if (!Array.isArray(input.x))                              return t('view.bg.validate.x_array');
    if (!Array.isArray(input.y))                              return t('view.bg.validate.y_array');
    if (!Number.isInteger(input.lag_order)
        || input.lag_order < MIN_LAG || input.lag_order > MAX_LAG)
                                                              return t('view.bg.validate.lag_range', { min: MIN_LAG, max: MAX_LAG });
    if (input.x.length !== input.y.length)                    return t('view.bg.validate.length_mismatch');
    if (input.x.length < input.lag_order + 8)                 return t('view.bg.validate.min_pairs', { n: input.lag_order + 8 });
    for (let i = 0; i < input.x.length; i++) {
        if (!Number.isFinite(input.x[i]))                     return t('view.bg.validate.x_finite', { i });
        if (!Number.isFinite(input.y[i]))                     return t('view.bg.validate.y_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        x: input.x.slice(),
        y: input.y.slice(),
        lag_order: input.lag_order,
    };
}

// Pure-JS mirror of crates/traderview-core/src/breusch_godfrey.rs::test.
export function localTest(x, y, lag_order) {
    const n = x.length;
    if (n < lag_order + 8 || y.length !== n || lag_order === 0) return null;
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
    const resid = new Array(n);
    for (let i = 0; i < n; i++) resid[i] = y[i] - alpha - beta * x[i];
    const p = 2 + lag_order;
    const n_aux = n - lag_order;
    if (n_aux < p + 2) return null;
    const xtx = Array.from({ length: p }, () => new Array(p).fill(0));
    const xty = new Array(p).fill(0);
    let sum_y = 0, sum_y_sq = 0;
    for (let t = lag_order; t < n; t++) {
        const row = new Array(p).fill(0);
        row[0] = 1;
        row[1] = x[t];
        for (let l = 0; l < lag_order; l++) row[2 + l] = resid[t - 1 - l];
        const yt = resid[t];
        sum_y += yt;
        sum_y_sq += yt * yt;
        for (let j = 0; j < p; j++) {
            xty[j] += row[j] * yt;
            for (let k = 0; k < p; k++) xtx[j][k] += row[j] * row[k];
        }
    }
    const coef = solveLinear(xtx, xty);
    if (!coef) return null;
    const mean_y = sum_y / n_aux;
    const tss = sum_y_sq - n_aux * mean_y * mean_y;
    let ssr = 0;
    for (let t = lag_order; t < n; t++) {
        let yhat = coef[0] + coef[1] * x[t];
        for (let l = 0; l < lag_order; l++) yhat += coef[2 + l] * resid[t - 1 - l];
        ssr += (resid[t] - yhat) ** 2;
    }
    const r_sq = tss > 1e-18 ? 1 - ssr / tss : 0;
    const lm = n_aux * Math.max(0, r_sq);
    const p_value = chiSquaredUpperTail(lm, lag_order);
    const crit = chiSquared5pctCritical(lag_order);
    return {
        lm_statistic: lm,
        p_value,
        r_squared_auxiliary: r_sq,
        lag_order,
        n_observations: n,
        reject_at_5pct: lm > crit,
    };
}

export function solveLinear(m, y) {
    const n = m.length;
    if (n === 0 || y.length !== n) return null;
    const aug = Array.from({ length: n }, (_, i) => {
        const r = new Array(n + 1).fill(0);
        for (let j = 0; j < n; j++) r[j] = m[i][j];
        r[n] = y[i];
        return r;
    });
    for (let i = 0; i < n; i++) {
        let pivot = i;
        for (let r = i + 1; r < n; r++) {
            if (Math.abs(aug[r][i]) > Math.abs(aug[pivot][i])) pivot = r;
        }
        if (Math.abs(aug[pivot][i]) < 1e-18) return null;
        if (pivot !== i) { const tmp = aug[i]; aug[i] = aug[pivot]; aug[pivot] = tmp; }
        const div = aug[i][i];
        for (let j = 0; j < n + 1; j++) aug[i][j] /= div;
        for (let r = 0; r < n; r++) {
            if (r === i) continue;
            const f = aug[r][i];
            if (f === 0) continue;
            const pivot_row = aug[i].slice();
            for (let j = 0; j < n + 1; j++) aug[r][j] -= f * pivot_row[j];
        }
    }
    return Array.from({ length: n }, (_, i) => aug[i][n]);
}

export function chiSquaredUpperTail(x, k) {
    if (x <= 0 || k <= 0) return 1;
    const z = (Math.pow(x / k, 1/3) - (1 - 2 / (9 * k))) / Math.sqrt(2 / (9 * k));
    return 1 - standardNormalCdf(z);
}

export function chiSquared5pctCritical(k) {
    const tbl = { 1: 3.841, 2: 5.991, 3: 7.815, 4: 9.488, 5: 11.070,
                  6: 12.592, 7: 14.067, 8: 15.507, 9: 16.919, 10: 18.307 };
    if (tbl[k] != null) return tbl[k];
    return k + 2 * Math.sqrt(2 * k);
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

// Parse "x y" pairs per line.
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
    if (!report) return { key: 'view.bg.verdict.unknown', cls: '' };
    const p = report.p_value;
    if (!Number.isFinite(p)) return { key: 'view.bg.verdict.unknown', cls: '' };
    if (p < 0.01)  return { key: 'view.bg.verdict.strong_reject', cls: 'neg' };
    if (p < 0.05)  return { key: 'view.bg.verdict.reject',        cls: 'neg' };
    if (p < 0.10)  return { key: 'view.bg.verdict.borderline',    cls: '' };
    return { key: 'view.bg.verdict.no_correlation', cls: 'pos' };
}

// Auxiliary R² strength badge.
export function r2Badge(r_sq) {
    if (r_sq == null || !Number.isFinite(r_sq)) return { key: 'view.bg.r2.unknown', cls: '' };
    if (r_sq >= 0.20) return { key: 'view.bg.r2.very_strong', cls: 'neg' };
    if (r_sq >= 0.10) return { key: 'view.bg.r2.strong',      cls: 'neg' };
    if (r_sq >= 0.04) return { key: 'view.bg.r2.moderate',    cls: '' };
    if (r_sq >= 0.01) return { key: 'view.bg.r2.weak',        cls: '' };
    return { key: 'view.bg.r2.negligible', cls: 'pos' };
}

// Sample-size adequacy: n / lag_order ratio guide.
export function sampleBadge(report) {
    if (!report || !Number.isInteger(report.n_observations)) {
        return { key: 'view.bg.sample.unknown', cls: '' };
    }
    const n = report.n_observations;
    const p = report.lag_order;
    if (p === 0) return { key: 'view.bg.sample.unknown', cls: '' };
    const ratio = n / p;
    if (ratio >= 50) return { key: 'view.bg.sample.large',    cls: 'pos' };
    if (ratio >= 20) return { key: 'view.bg.sample.medium',   cls: '' };
    if (ratio >= 10) return { key: 'view.bg.sample.small',    cls: 'neg' };
    return { key: 'view.bg.sample.too_small', cls: 'neg' };
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

export function makeDemoInput(kind = 'iid-residuals') {
    switch (kind) {
        case 'iid-residuals': {
            const rand = lcg(42n);
            const x = Array.from({ length: 200 }, (_, i) => i);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * 0.5);
            return { x, y, lag_order: 4 };
        }
        case 'ar1-residuals': {
            // y = 2x + e, e_t = 0.8 · e_{t-1} + η — strong serial correlation.
            const rand = lcg(11n);
            const x = Array.from({ length: 300 }, (_, i) => i);
            const e = new Array(300).fill(0);
            for (let i = 1; i < 300; i++) e[i] = 0.8 * e[i - 1] + (rand() - 0.5) * 5;
            const y = x.map((xi, i) => 2 * xi + e[i]);
            return { x, y, lag_order: 2 };
        }
        case 'ar2-residuals': {
            const rand = lcg(13n);
            const x = Array.from({ length: 300 }, (_, i) => i);
            const e = new Array(300).fill(0);
            for (let i = 2; i < 300; i++) {
                e[i] = 0.6 * e[i - 1] + 0.3 * e[i - 2] + (rand() - 0.5) * 3;
            }
            const y = x.map((xi, i) => 2 * xi + e[i]);
            return { x, y, lag_order: 4 };
        }
        case 'mild-ar1': {
            const rand = lcg(21n);
            const x = Array.from({ length: 200 }, (_, i) => i);
            const e = new Array(200).fill(0);
            for (let i = 1; i < 200; i++) e[i] = 0.25 * e[i - 1] + (rand() - 0.5) * 2;
            const y = x.map((xi, i) => 2 * xi + e[i]);
            return { x, y, lag_order: 4 };
        }
        case 'cyclical-residuals': {
            // sin-wave residuals → strong serial correlation but not AR.
            const x = Array.from({ length: 200 }, (_, i) => i);
            const y = x.map((xi, i) => 2 * xi + Math.sin(i * 0.3) * 2);
            return { x, y, lag_order: 6 };
        }
        case 'high-lag': {
            // lag_order = 10 stress test.
            const rand = lcg(33n);
            const x = Array.from({ length: 300 }, (_, i) => i);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * 1);
            return { x, y, lag_order: 10 };
        }
        case 'short-series': {
            // Just above lag_order + 8 minimum.
            const rand = lcg(57n);
            const x = Array.from({ length: 15 }, (_, i) => i);
            const y = x.map(xi => 2 * xi + (rand() - 0.5) * 1);
            return { x, y, lag_order: 2 };
        }
        case 'price-vs-return': {
            // Random walk price + iid returns — should NOT reject.
            const rand = lcg(99n);
            const x = [];
            const y = [];
            let p = 100;
            for (let i = 0; i < 250; i++) {
                p += (rand() - 0.5) * 1;
                x.push(p);
                y.push((rand() - 0.5) * 2);
            }
            return { x, y, lag_order: 4 };
        }
        default: return makeDemoInput('iid-residuals');
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
