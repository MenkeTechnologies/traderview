// Risk-parity weights solver (Spinu 2013 fixed-point) helpers.
//
// Backend body: { covariance: number[][], max_iter: number, tolerance: number }
// Returns: { weights, risk_contributions, portfolio_volatility,
//   iterations, max_contribution_deviation, converged } | null
//
// Equal Risk Contribution iteration:
//   w_i ← b_i · σ_p / (Σw)_i ,  then normalize Σw = 1
// with b_i = 1/n (equal budget).

import { t } from './i18n.js';

export const DEFAULT_MAX_ITER = 500;
export const DEFAULT_TOLERANCE = 1e-8;

export const DEFAULT_INPUTS = {
    covariance: [
        [0.04, 0.01, 0.005],
        [0.01, 0.09, 0.02],
        [0.005, 0.02, 0.16],
    ],
    max_iter: DEFAULT_MAX_ITER,
    tolerance: DEFAULT_TOLERANCE,
};

export function validateInputs(input) {
    const c = input.covariance;
    if (!Array.isArray(c))                              return t('view.risk_parity_solver.validate.cov_array');
    if (c.length < 2)                                    return t('view.risk_parity_solver.validate.assets_min');
    const n = c.length;
    for (let i = 0; i < n; i++) {
        if (!Array.isArray(c[i]) || c[i].length !== n) return t('view.risk_parity_solver.validate.cov_square');
        for (let j = 0; j < n; j++) {
            if (!Number.isFinite(c[i][j]))             return t('view.risk_parity_solver.validate.cov_finite', { i, j });
        }
    }
    if (!Number.isInteger(input.max_iter) || input.max_iter < 1) return t('view.risk_parity_solver.validate.max_iter');
    if (!Number.isFinite(input.tolerance) || input.tolerance <= 0) return t('view.risk_parity_solver.validate.tolerance');
    return null;
}

export function buildBody(input) {
    return {
        covariance: input.covariance,
        max_iter:   input.max_iter,
        tolerance:  input.tolerance,
    };
}

// Pure-JS mirror of crates/traderview-core/src/risk_parity_weights.rs::solve.
// Returns null on validation failure or numerical degeneracy.
export function localSolve(covariance, max_iter, tolerance) {
    const err = validateInputs({ covariance, max_iter, tolerance });
    if (err) return null;
    const n = covariance.length;
    let w = new Array(n).fill(1 / n);
    for (let i = 0; i < n; i++) {
        const sigma_i = Math.sqrt(Math.max(0, covariance[i][i]));
        if (sigma_i > 0) w[i] = 1 / sigma_i;
    }
    normalize(w);
    let iters = 0;
    let max_dev = Infinity;
    const target_budget = 1 / n;
    for (let k = 0; k < max_iter; k++) {
        iters++;
        const sigma_w = matvec(covariance, w);
        let port_var = 0;
        for (let i = 0; i < n; i++) port_var += w[i] * sigma_w[i];
        if (port_var <= 0) return null;
        const port_vol = Math.sqrt(port_var);
        const new_w = new Array(n);
        for (let i = 0; i < n; i++) {
            new_w[i] = sigma_w[i] <= 0 ? w[i] : target_budget * port_vol / sigma_w[i];
        }
        normalize(new_w);
        const new_sigma_w = matvec(covariance, new_w);
        let new_port_var = 0;
        for (let i = 0; i < n; i++) new_port_var += new_w[i] * new_sigma_w[i];
        const new_port_vol = Math.sqrt(Math.max(0, new_port_var));
        if (new_port_vol <= 0) return null;
        const contributions = new Array(n);
        for (let i = 0; i < n; i++) contributions[i] = new_w[i] * new_sigma_w[i] / new_port_vol;
        let sumC = 0;
        for (const c of contributions) sumC += c;
        const target = sumC / n;
        max_dev = 0;
        for (const c of contributions) max_dev = Math.max(max_dev, Math.abs(c - target));
        w = new_w;
        if (max_dev < tolerance) break;
    }
    const sigma_w = matvec(covariance, w);
    let port_var = 0;
    for (let i = 0; i < n; i++) port_var += w[i] * sigma_w[i];
    const port_vol = Math.sqrt(Math.max(0, port_var));
    const risk_contributions = new Array(n);
    for (let i = 0; i < n; i++) {
        risk_contributions[i] = port_vol > 0 ? w[i] * sigma_w[i] / port_vol : 0;
    }
    return {
        weights: w,
        risk_contributions,
        portfolio_volatility: port_vol,
        iterations: iters,
        max_contribution_deviation: max_dev,
        converged: max_dev < tolerance,
    };
}

function matvec(m, v) {
    const n = m.length;
    const out = new Array(n);
    for (let i = 0; i < n; i++) {
        let s = 0;
        for (let j = 0; j < n; j++) s += m[i][j] * v[j];
        out[i] = s;
    }
    return out;
}

function normalize(w) {
    let s = 0;
    for (const x of w) s += x;
    if (s > 0) for (let i = 0; i < w.length; i++) w[i] /= s;
}

// Parse blob: one matrix row per line, comma/whitespace-separated.
// Comments (#) and blanks ignored.
export function parseMatrix(blob) {
    const out = { matrix: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const cells = raw.split(/[\s,]+/).filter(x => x.length > 0).map(Number);
        if (cells.some(v => !Number.isFinite(v))) {
            out.errors.push({ line_no: i + 1, message: 'non-finite cell' });
            continue;
        }
        out.matrix.push(cells);
    }
    if (out.matrix.length > 0) {
        const n = out.matrix.length;
        for (let i = 0; i < n; i++) {
            if (out.matrix[i].length !== n) {
                out.errors.push({ line_no: i + 1, message: `row ${i + 1} has ${out.matrix[i].length} cols, expected ${n}` });
            }
        }
    }
    return out;
}

// Build covariance from per-asset volatilities + (symmetric) correlation matrix.
export function covFromVolsAndCorr(vols, corr) {
    const n = vols.length;
    const out = Array.from({ length: n }, () => new Array(n).fill(0));
    for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
            out[i][j] = vols[i] * vols[j] * corr[i][j];
        }
    }
    return out;
}

export function convergenceBadge(report) {
    if (!report) return { key: 'view.rp_solver.badge.unknown', cls: '' };
    if (!report.converged) return { key: 'view.rp_solver.badge.not_converged', cls: 'neg' };
    if (report.iterations < 50)  return { key: 'view.rp_solver.badge.fast',   cls: 'pos' };
    if (report.iterations < 200) return { key: 'view.rp_solver.badge.normal', cls: '' };
    return { key: 'view.rp_solver.badge.slow', cls: '' };
}

export function rcBadge(rc, totalVol, n) {
    if (!Number.isFinite(rc) || !Number.isFinite(totalVol) || totalVol <= 0)
        return { key: 'view.rp_solver.rc.unknown', cls: '' };
    const target = 1 / n;
    const frac = rc / totalVol;
    const dev = Math.abs(frac - target);
    if (dev < 1e-4) return { key: 'view.rp_solver.rc.balanced', cls: 'pos' };
    if (dev < 1e-2) return { key: 'view.rp_solver.rc.close',    cls: '' };
    return { key: 'view.rp_solver.rc.off', cls: 'neg' };
}

export function makeDemoInput(kind = '60-40-style') {
    switch (kind) {
        case 'equal-vol-uncorr':
            return { covariance: diag([0.04, 0.04, 0.04]),
                     max_iter: DEFAULT_MAX_ITER, tolerance: DEFAULT_TOLERANCE };
        case 'high-vol-pair':
            return { covariance: [[0.01, 0], [0, 0.09]],
                     max_iter: DEFAULT_MAX_ITER, tolerance: 1e-12 };
        case '60-40-style': {
            const vols = [0.16, 0.05, 0.18];
            const corr = [
                [1.0,  -0.05,  0.10],
                [-0.05, 1.0,  -0.20],
                [0.10, -0.20,  1.0],
            ];
            return { covariance: covFromVolsAndCorr(vols, corr),
                     max_iter: DEFAULT_MAX_ITER, tolerance: DEFAULT_TOLERANCE };
        }
        case 'high-correlation': {
            const vols = [0.20, 0.20, 0.20];
            const corr = [
                [1.0, 0.85, 0.85],
                [0.85, 1.0, 0.85],
                [0.85, 0.85, 1.0],
            ];
            return { covariance: covFromVolsAndCorr(vols, corr),
                     max_iter: DEFAULT_MAX_ITER, tolerance: DEFAULT_TOLERANCE };
        }
        case 'diversifier': {
            const vols = [0.20, 0.20, 0.20, 0.30];
            const corr = [
                [1.0,  0.7,  0.7, -0.4],
                [0.7,  1.0,  0.7, -0.4],
                [0.7,  0.7,  1.0, -0.4],
                [-0.4, -0.4, -0.4, 1.0],
            ];
            return { covariance: covFromVolsAndCorr(vols, corr),
                     max_iter: DEFAULT_MAX_ITER, tolerance: DEFAULT_TOLERANCE };
        }
        case 'small-pair':
            return { covariance: [[0.04, 0.012], [0.012, 0.09]],
                     max_iter: DEFAULT_MAX_ITER, tolerance: DEFAULT_TOLERANCE };
        case 'tight-tolerance':
            return { covariance: diag([0.04, 0.09, 0.16, 0.25]),
                     max_iter: 1000, tolerance: 1e-14 };
        case 'loose-tolerance':
            return { covariance: diag([0.04, 0.09, 0.16]),
                     max_iter: 10, tolerance: 1e-2 };
        default:
            return makeDemoInput('60-40-style');
    }
}

function diag(vals) {
    const n = vals.length;
    const out = Array.from({ length: n }, () => new Array(n).fill(0));
    for (let i = 0; i < n; i++) out[i][i] = vals[i];
    return out;
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 6) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtSci(v) {
    if (!Number.isFinite(v)) return '—';
    if (v === 0) return '0';
    if (Math.abs(v) >= 1e-4) return v.toFixed(6);
    return v.toExponential(3);
}

export function assetLabel(i) {
    if (i < 26) return String.fromCharCode(65 + i);
    return `A${i - 25}`;
}

export function matrixToBlob(m) {
    if (!Array.isArray(m) || m.length === 0) return '';
    return m.map(row => row.join(', ')).join('\n');
}
