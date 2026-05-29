// Carhart 4-Factor Model helpers (Mkt + SMB + HML + WML).
//
// Backend body: {
//   portfolio_returns, market_excess, smb, hml, wml, risk_free: number[]
// }  — all same length, ≥ 10 valid observations after NaN-filter.
// Returns: {
//   alpha, beta_mkt, beta_smb, beta_hml, beta_wml,
//   alpha_se, beta_mkt_se, beta_smb_se, beta_hml_se, beta_wml_se,
//   alpha_tstat, r_squared, n_observations,
// } | null

import { t } from './i18n.js';

export const MIN_OBS = 10;

export const DEFAULT_INPUTS = {
    portfolio_returns: [],
    market_excess: [],
    smb: [],
    hml: [],
    wml: [],
    risk_free: [],
};

const FIELDS = ['portfolio_returns', 'market_excess', 'smb', 'hml', 'wml', 'risk_free'];

export function validateInputs(input) {
    for (const k of FIELDS) {
        if (!Array.isArray(input[k]))                          return t('view.car4.validate.field_array', { field: k });
    }
    const n = input.portfolio_returns.length;
    for (const k of FIELDS) {
        if (input[k].length !== n)                             return t('view.car4.validate.field_len', { field: k, len: input[k].length, n });
    }
    if (n < MIN_OBS)                                           return t('view.car4.validate.obs_min', { n: MIN_OBS });
    for (const k of FIELDS) {
        for (let i = 0; i < n; i++) {
            if (typeof input[k][i] !== 'number')               return t('view.car4.validate.field_number', { field: k, i });
            // NaN is allowed per Rust impl — those rows get filtered.
        }
    }
    return null;
}

export function buildBody(input) {
    return {
        portfolio_returns: input.portfolio_returns.slice(),
        market_excess:     input.market_excess.slice(),
        smb:               input.smb.slice(),
        hml:               input.hml.slice(),
        wml:               input.wml.slice(),
        risk_free:         input.risk_free.slice(),
    };
}

// Pure-JS mirror of crates/traderview-core/src/factor_models.rs::carhart4.
export function localCompute(inputs) {
    const n = inputs.portfolio_returns.length;
    if (inputs.market_excess.length !== n
        || inputs.smb.length !== n
        || inputs.hml.length !== n
        || inputs.wml.length !== n
        || inputs.risk_free.length !== n
        || n < MIN_OBS) return null;
    const y = [], x_mkt = [], x_smb = [], x_hml = [], x_wml = [];
    for (let i = 0; i < n; i++) {
        const p = inputs.portfolio_returns[i];
        const m = inputs.market_excess[i];
        const s = inputs.smb[i];
        const h = inputs.hml[i];
        const w = inputs.wml[i];
        const rf = inputs.risk_free[i];
        if (!Number.isFinite(p) || !Number.isFinite(m) || !Number.isFinite(s)
            || !Number.isFinite(h) || !Number.isFinite(w) || !Number.isFinite(rf)) continue;
        y.push(p - rf);
        x_mkt.push(m);
        x_smb.push(s);
        x_hml.push(h);
        x_wml.push(w);
    }
    const n_obs = y.length;
    if (n_obs < MIN_OBS) return null;
    const cols = [new Array(n_obs).fill(1), x_mkt, x_smb, x_hml, x_wml];
    const r = olsWithSe(cols, y);
    if (!r) return null;
    const { beta: b, se } = r;
    if (b.length !== 5 || se.length !== 5) return null;
    let ySum = 0;
    for (const v of y) ySum += v;
    const y_mean = ySum / n_obs;
    let ss_tot = 0, ss_res = 0;
    for (let k = 0; k < n_obs; k++) {
        const pred = b[0] + b[1] * cols[1][k] + b[2] * cols[2][k] + b[3] * cols[3][k] + b[4] * cols[4][k];
        ss_tot += (y[k] - y_mean) ** 2;
        ss_res += (y[k] - pred) ** 2;
    }
    const r2 = ss_tot > 0 ? 1 - ss_res / ss_tot : 0;
    return {
        alpha: b[0], beta_mkt: b[1], beta_smb: b[2], beta_hml: b[3], beta_wml: b[4],
        alpha_se: se[0], beta_mkt_se: se[1], beta_smb_se: se[2],
        beta_hml_se: se[3], beta_wml_se: se[4],
        alpha_tstat: se[0] > 0 ? b[0] / se[0] : 0,
        r_squared: r2,
        n_observations: n_obs,
    };
}

// OLS with standard errors via Gauss-Jordan on augmented (X'X | I | X'y).
export function olsWithSe(x, y) {
    const p = x.length;
    const n = y.length;
    if (p === 0 || n === 0) return null;
    for (const c of x) if (c.length !== n) return null;
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
    const aug = Array.from({ length: p }, () => new Array(2 * p + 1).fill(0));
    for (let i = 0; i < p; i++) {
        for (let j = 0; j < p; j++) {
            aug[i][j] = xtx[i][j];
            aug[i][p + j] = i === j ? 1 : 0;
        }
        aug[i][2 * p] = xty[i];
    }
    for (let i = 0; i < p; i++) {
        let pivot = i;
        for (let r = i + 1; r < p; r++) {
            if (Math.abs(aug[r][i]) > Math.abs(aug[pivot][i])) pivot = r;
        }
        if (Math.abs(aug[pivot][i]) < 1e-18) return null;
        if (pivot !== i) { const tmp = aug[i]; aug[i] = aug[pivot]; aug[pivot] = tmp; }
        const div = aug[i][i];
        for (let j = 0; j < 2 * p + 1; j++) aug[i][j] /= div;
        for (let r = 0; r < p; r++) {
            if (r === i) continue;
            const f = aug[r][i];
            if (f === 0) continue;
            const pivot_row = aug[i].slice();
            for (let j = 0; j < 2 * p + 1; j++) aug[r][j] -= f * pivot_row[j];
        }
    }
    const beta = Array.from({ length: p }, (_, i) => aug[i][2 * p]);
    let ss_res = 0;
    for (let k = 0; k < n; k++) {
        let yh = 0;
        for (let i = 0; i < p; i++) yh += beta[i] * x[i][k];
        ss_res += (y[k] - yh) ** 2;
    }
    const dof = Math.max(1, n - p);
    const sigma2 = ss_res / dof;
    const se = new Array(p);
    for (let i = 0; i < p; i++) {
        const variance = sigma2 * aug[i][p + i];
        se[i] = variance > 0 ? Math.sqrt(variance) : 0;
    }
    return { beta, se };
}

// Parse blob: each line "p mkt smb hml wml rf" → 6 tokens.
export function parseSeriesBlob(blob) {
    const out = {
        portfolio_returns: [], market_excess: [],
        smb: [], hml: [], wml: [], risk_free: [],
        errors: [],
    };
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
        if (parts.length !== 6) {
            out.errors.push({ line_no: i + 1, message: `expected 6 tokens (port mkt smb hml wml rf), got ${parts.length}` });
            continue;
        }
        const vals = parts.map(p => Number(p.replace(/[\$%,]/g, '')));
        if (vals.some(v => !Number.isFinite(v))) {
            out.errors.push({ line_no: i + 1, message: `tokens must be finite` });
            continue;
        }
        out.portfolio_returns.push(vals[0]);
        out.market_excess.push(vals[1]);
        out.smb.push(vals[2]);
        out.hml.push(vals[3]);
        out.wml.push(vals[4]);
        out.risk_free.push(vals[5]);
    }
    return out;
}

export function seriesToBlob(input) {
    const n = input.portfolio_returns.length;
    const lines = [];
    for (let i = 0; i < n; i++) {
        lines.push([
            input.portfolio_returns[i],
            input.market_excess[i],
            input.smb[i],
            input.hml[i],
            input.wml[i],
            input.risk_free[i],
        ].join(' '));
    }
    return lines.join('\n');
}

// Alpha verdict using |t-stat|: > 2 = significant.
export function alphaBadge(report) {
    if (!report) return { key: 'view.car4.alpha.unknown', cls: '' };
    const t = Math.abs(report.alpha_tstat);
    const sign = Math.sign(report.alpha);
    if (!Number.isFinite(t)) return { key: 'view.car4.alpha.unknown', cls: '' };
    if (t >= 2.58 && sign > 0)  return { key: 'view.car4.alpha.strong_pos',  cls: 'pos' };
    if (t >= 1.96 && sign > 0)  return { key: 'view.car4.alpha.significant_pos', cls: 'pos' };
    if (t >= 2.58 && sign < 0)  return { key: 'view.car4.alpha.strong_neg',  cls: 'neg' };
    if (t >= 1.96 && sign < 0)  return { key: 'view.car4.alpha.significant_neg', cls: 'neg' };
    return { key: 'view.car4.alpha.insignificant', cls: '' };
}

// Style tilts from the 4 betas.
export function styleBadge(report) {
    if (!report) return { key: 'view.car4.style.unknown', cls: '' };
    const tilts = [];
    if (report.beta_smb >  0.20) tilts.push('small');
    if (report.beta_smb < -0.20) tilts.push('large');
    if (report.beta_hml >  0.20) tilts.push('value');
    if (report.beta_hml < -0.20) tilts.push('growth');
    if (report.beta_wml >  0.20) tilts.push('momentum');
    if (report.beta_wml < -0.20) tilts.push('contrarian');
    if (tilts.length === 0)      return { key: 'view.car4.style.market_neutral', cls: '' };
    if (tilts.length === 1)      return { key: `view.car4.style.${tilts[0]}`, cls: '' };
    return { key: 'view.car4.style.multi', cls: '' };
}

// R² quality.
export function fitBadge(r_sq) {
    if (r_sq == null || !Number.isFinite(r_sq)) return { key: 'view.car4.fit.unknown', cls: '' };
    if (r_sq >= 0.90) return { key: 'view.car4.fit.excellent', cls: 'pos' };
    if (r_sq >= 0.70) return { key: 'view.car4.fit.good',      cls: 'pos' };
    if (r_sq >= 0.40) return { key: 'view.car4.fit.moderate',  cls: '' };
    if (r_sq >= 0.10) return { key: 'view.car4.fit.weak',      cls: 'neg' };
    return { key: 'view.car4.fit.poor', cls: 'neg' };
}

// Market beta classification.
export function marketBetaBadge(b) {
    if (b == null || !Number.isFinite(b)) return { key: 'view.car4.mkt.unknown', cls: '' };
    if (b < -0.5)  return { key: 'view.car4.mkt.inverse',    cls: 'neg' };
    if (b < 0.5)   return { key: 'view.car4.mkt.low',        cls: '' };
    if (b < 1.5)   return { key: 'view.car4.mkt.market',     cls: '' };
    if (b < 2.5)   return { key: 'view.car4.mkt.high',       cls: '' };
    return { key: 'view.car4.mkt.leveraged', cls: '' };
}

export function summarizeSeries(input) {
    const n = input.portfolio_returns.length;
    if (n === 0) return { n: 0, mean_p: NaN, mean_m: NaN, mean_rf: NaN };
    let sp = 0, sm = 0, sr = 0;
    for (let i = 0; i < n; i++) {
        sp += input.portfolio_returns[i];
        sm += input.market_excess[i];
        sr += input.risk_free[i];
    }
    return {
        n,
        mean_p:  sp / n,
        mean_m:  sm / n,
        mean_rf: sr / n,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function makeSynth(n, betas, alpha, rfRate, seed, sigma = 0.005) {
    const rand = lcg(seed);
    const out = {
        portfolio_returns: [], market_excess: [],
        smb: [], hml: [], wml: [], risk_free: [],
    };
    for (let i = 0; i < n; i++) {
        const m  = (rand() - 0.5) * 0.04;
        const s  = (rand() - 0.5) * 0.03;
        const h  = (rand() - 0.5) * 0.025;
        const w  = (rand() - 0.5) * 0.02;
        const rf = rfRate;
        const eps = (rand() - 0.5) * sigma;
        const p = rf + alpha + betas[0] * m + betas[1] * s + betas[2] * h + betas[3] * w + eps;
        out.portfolio_returns.push(p);
        out.market_excess.push(m);
        out.smb.push(s);
        out.hml.push(h);
        out.wml.push(w);
        out.risk_free.push(rf);
    }
    return out;
}

export function makeDemoInput(kind = 'market-only') {
    switch (kind) {
        case 'market-only':       return makeSynth(252, [1.0,  0,   0,    0],   0.0001, 0.00005, 42n);
        case 'small-cap-tilt':    return makeSynth(252, [1.0,  0.7, 0.0,  0.0],  0.0,    0.00005, 7n);
        case 'value-tilt':        return makeSynth(252, [1.0,  0.0, 0.6,  0.0],  0.0,    0.00005, 11n);
        case 'momentum-tilt':     return makeSynth(252, [1.0,  0.0, 0.0,  0.8],  0.0,    0.00005, 13n);
        case 'growth-tilt':       return makeSynth(252, [1.0,  0.0, -0.6, 0.0],  0.0,    0.00005, 21n);
        case 'positive-alpha':    return makeSynth(252, [0.9,  0.2, 0.1,  0.3],  0.002,  0.00005, 33n);
        case 'negative-alpha':    return makeSynth(252, [1.0,  0.0, 0.0,  0.0], -0.002,  0.00005, 57n);
        case 'small-sample':      return makeSynth(15,  [1.0,  0.0, 0.0,  0.0],  0.0,    0.00005, 99n);
        default: return makeDemoInput('market-only');
    }
}

export function fmtBeta(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtBetaSigned(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtTStat(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(2);
}
