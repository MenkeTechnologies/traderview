// Augmented Dickey-Fuller (ADF) stationarity test helpers.
//
// Backend body: { series: number[], lags: number }
// Returns: { t_statistic, gamma, gamma_se, significance, n_observations, lags } | null
//
// Regression: Δy_t = α + γ·y_{t-1} + Σ φ_i·Δy_{t-i} + ε_t.
// Reject H₀ (unit root, non-stationary) when t-stat < critical:
//   1%:  −3.43
//   5%:  −2.86
//   10%: −2.57

import { t } from './i18n.js';

export const DEFAULT_LAGS = 1;
export const CRIT_1PCT  = -3.43;
export const CRIT_5PCT  = -2.86;
export const CRIT_10PCT = -2.57;
export const SIGNIFICANCES = ['pct1', 'pct5', 'pct10', 'insignificant'];

export const DEFAULT_INPUTS = {
    series: [],
    lags: DEFAULT_LAGS,
};

export function validateInputs(input) {
    if (!Array.isArray(input.series))                       return t('view.adf_test.validate.series_array');
    for (let i = 0; i < input.series.length; i++) {
        if (!Number.isFinite(input.series[i]))              return t('view.adf_test.validate.series_finite', { i });
    }
    if (!Number.isInteger(input.lags))                      return t('view.adf_test.validate.lags_int');
    if (input.lags < 0)                                     return t('view.adf_test.validate.lags_negative');
    const minN = 3 * input.lags + 4;
    if (input.series.length < minN)                         return t('view.adf_test.validate.series_min', { n: minN });
    return null;
}

export function buildBody(input) {
    return {
        series: input.series,
        lags:   input.lags,
    };
}

// Pure-JS mirror of crates/traderview-core/src/adf_standalone.rs::test.
// Returns null on validation failure or singular regression.
export function localTest(series, lags) {
    const n = series.length;
    if (n < 3 * lags + 4) return null;
    for (const v of series) if (!Number.isFinite(v)) return null;
    const diffs = new Array(n).fill(0);
    for (let i = 1; i < n; i++) diffs[i] = series[i] - series[i - 1];
    const start = lags + 1;
    if (n <= start) return null;
    const m = n - start;
    if (m < 2 * lags + 2) return null;
    const p_cols = 2 + lags;
    const x = Array.from({ length: p_cols }, () => new Array(m));
    const y_vec = new Array(m);
    let row = 0;
    for (let i = start; i < n; i++) {
        x[0][row] = 1;
        x[1][row] = series[i - 1];
        for (let k = 0; k < lags; k++) x[2 + k][row] = diffs[i - 1 - k];
        y_vec[row] = diffs[i];
        row++;
    }
    const olsResult = olsWithSe(x, y_vec);
    if (!olsResult) return null;
    const { beta, se } = olsResult;
    if (beta.length !== p_cols || se.length !== p_cols) return null;
    const gamma = beta[1];
    const gamma_se = se[1];
    if (gamma_se <= 0) return null;
    const t_stat = gamma / gamma_se;
    let sig;
    if (t_stat < CRIT_1PCT)       sig = 'pct1';
    else if (t_stat < CRIT_5PCT)  sig = 'pct5';
    else if (t_stat < CRIT_10PCT) sig = 'pct10';
    else                          sig = 'insignificant';
    return {
        t_statistic:    t_stat,
        gamma,
        gamma_se,
        significance:   sig,
        n_observations: m,
        lags,
    };
}

// OLS regression with standard errors via Gauss-Jordan inversion of XᵀX.
function olsWithSe(x, y) {
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
    // Augmented matrix: [XᵀX | I | Xᵀy]
    const aug = Array.from({ length: p }, () => new Array(2 * p + 1).fill(0));
    for (let i = 0; i < p; i++) {
        for (let j = 0; j < p; j++) {
            aug[i][j] = xtx[i][j];
            aug[i][p + j] = (i === j) ? 1 : 0;
        }
        aug[i][2 * p] = xty[i];
    }
    for (let i = 0; i < p; i++) {
        let pivot = i;
        for (let r = i + 1; r < p; r++) {
            if (Math.abs(aug[r][i]) > Math.abs(aug[pivot][i])) pivot = r;
        }
        if (Math.abs(aug[pivot][i]) < 1e-18) return null;
        if (pivot !== i) { const t = aug[i]; aug[i] = aug[pivot]; aug[pivot] = t; }
        const div = aug[i][i];
        for (let j = 0; j < 2 * p + 1; j++) aug[i][j] /= div;
        for (let r = 0; r < p; r++) {
            if (r === i) continue;
            const f = aug[r][i];
            if (f === 0) continue;
            for (let j = 0; j < 2 * p + 1; j++) aug[r][j] -= f * aug[i][j];
        }
    }
    const beta = new Array(p);
    for (let i = 0; i < p; i++) beta[i] = aug[i][2 * p];
    // Residuals + σ² + standard errors from diagonal of (XᵀX)⁻¹ × σ².
    let ss_res = 0;
    for (let k = 0; k < n; k++) {
        let yhat = 0;
        for (let i = 0; i < p; i++) yhat += beta[i] * x[i][k];
        const r = y[k] - yhat;
        ss_res += r * r;
    }
    const dof = Math.max(1, n - p);
    const sigma2 = ss_res / dof;
    const se = new Array(p).fill(0);
    for (let i = 0; i < p; i++) {
        const v = sigma2 * aug[i][p + i];
        se[i] = v > 0 ? Math.sqrt(v) : 0;
    }
    return { beta, se };
}

// Parse comma/whitespace-separated series; ignores blanks + # comments.
export function parseSeriesBlob(blob) {
    const out = { series: [], errors: [] };
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
        const v = Number(tokens[i]);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.series.push(v);
    }
    return out;
}

export function seriesToBlob(series) {
    return series.join('\n');
}

// Verdict for the significance level.
export function significanceBadge(sig) {
    if (sig === 'pct1')           return { key: 'view.adf.badge.pct1',   cls: 'pos' };
    if (sig === 'pct5')           return { key: 'view.adf.badge.pct5',   cls: 'pos' };
    if (sig === 'pct10')          return { key: 'view.adf.badge.pct10',  cls: '' };
    if (sig === 'insignificant')  return { key: 'view.adf.badge.fail',   cls: 'neg' };
    return { key: 'view.adf.badge.unknown', cls: '' };
}

// Interpretive verdict on t-statistic distance from 5% critical value.
export function strengthBadge(t_stat) {
    if (!Number.isFinite(t_stat)) return { key: 'view.adf.strength.unknown', cls: '' };
    if (t_stat < -5)  return { key: 'view.adf.strength.very_strong', cls: 'pos' };
    if (t_stat < CRIT_1PCT)  return { key: 'view.adf.strength.strong',     cls: 'pos' };
    if (t_stat < CRIT_5PCT)  return { key: 'view.adf.strength.moderate',   cls: 'pos' };
    if (t_stat < CRIT_10PCT) return { key: 'view.adf.strength.weak',       cls: '' };
    if (t_stat < -1.0) return { key: 'view.adf.strength.weak_trend',   cls: 'neg' };
    return { key: 'view.adf.strength.unit_root', cls: 'neg' };
}

export function significanceLabelKey(sig) {
    if (sig === 'pct1')          return 'view.adf.sig.pct1';
    if (sig === 'pct5')          return 'view.adf.sig.pct5';
    if (sig === 'pct10')         return 'view.adf.sig.pct10';
    if (sig === 'insignificant') return 'view.adf.sig.insignificant';
    return 'view.adf.sig.unknown';
}

// Deterministic LCG for stable demo data.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF - 0.5;
    };
}

// Synthetic demos. Each is constructed to hit a different significance tier.
export function makeDemoInput(kind = 'random-walk') {
    switch (kind) {
        case 'random-walk': {
            const rand = lcg(42n);
            const s = new Array(500).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = s[i - 1] + rand();
            return { series: s, lags: 1 };
        }
        case 'mean-reverting-strong': {
            // AR(1) φ=0.3 → highly stationary.
            const rand = lcg(999n);
            const s = new Array(500).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = 0.3 * s[i - 1] + rand();
            return { series: s, lags: 2 };
        }
        case 'mean-reverting-weak': {
            // AR(1) φ=0.85 → mildly stationary.
            const rand = lcg(123n);
            const s = new Array(500).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = 0.85 * s[i - 1] + rand();
            return { series: s, lags: 2 };
        }
        case 'trend-stationary': {
            // y = 0.01·t + noise.
            const rand = lcg(7n);
            const s = new Array(500);
            for (let i = 0; i < s.length; i++) s[i] = 0.01 * i + rand();
            return { series: s, lags: 1 };
        }
        case 'pure-noise': {
            const rand = lcg(1n);
            const s = new Array(500);
            for (let i = 0; i < s.length; i++) s[i] = rand();
            return { series: s, lags: 1 };
        }
        case 'high-lags': {
            // Same series as mean-reverting-strong but with lags=5.
            const rand = lcg(999n);
            const s = new Array(500).fill(0);
            for (let i = 1; i < s.length; i++) s[i] = 0.3 * s[i - 1] + rand();
            return { series: s, lags: 5 };
        }
        case 'short-series': {
            // Just barely enough for lags=0.
            return { series: [1, 2, 3, 4, 5, 6], lags: 0 };
        }
        case 'flat': {
            // Constant series → singular regression → null.
            return { series: new Array(50).fill(100), lags: 1 };
        }
        default: return makeDemoInput('random-walk');
    }
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtT(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
