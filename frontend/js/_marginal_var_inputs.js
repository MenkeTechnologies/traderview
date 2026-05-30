// Marginal / Component VaR (risk-budgeting) helpers.
//
// Backend body: { portfolio: { weights, covariance }, z_alpha }
// Returns: { portfolio_var, portfolio_vol, marginal_var, component_var,
//   pct_contribution } | null
//
// Identities:
//   port_var(α) = vol · z_α
//   marginal_i  = z_α · (Σw)_i / vol
//   component_i = w_i · marginal_i
//   pct_i       = component_i / port_var * 100
//   Σ component = port_var       (decomposition identity)
//   Σ pct       = 100%

import { t } from './i18n.js';

export const DEFAULT_Z_ALPHA = 1.645;        // 95% normal-tail z

// Common z-scores for quick reference / dropdown labels.
export const Z_CONFIDENCE_LEVELS = [
    { z: 1.645, label: '95%' },
    { z: 1.960, label: '97.5%' },
    { z: 2.326, label: '99%' },
    { z: 2.576, label: '99.5%' },
    { z: 3.090, label: '99.9%' },
];

export const DEFAULT_INPUTS = {
    portfolio: {
        weights: [0.3, 0.4, 0.3],
        covariance: [
            [0.04, 0.01, 0.005],
            [0.01, 0.09, 0.02],
            [0.005, 0.02, 0.16],
        ],
        labels: ['A', 'B', 'C'],
    },
    z_alpha: DEFAULT_Z_ALPHA,
};

export function validateInputs(input) {
    const p = input.portfolio;
    if (!p)                                                  return t('view.marginal_var.validate.portfolio_required');
    if (!Array.isArray(p.weights))                            return t('view.marginal_var.validate.weights_array');
    if (p.weights.length === 0)                               return t('view.marginal_var.validate.weights_empty');
    for (let i = 0; i < p.weights.length; i++) {
        if (!Number.isFinite(p.weights[i]))                   return t('view.marginal_var.validate.weight_finite', { i });
    }
    if (!Array.isArray(p.covariance))                         return t('view.marginal_var.validate.cov_array');
    const k = p.weights.length;
    if (p.covariance.length !== k)                            return t('view.marginal_var.validate.cov_dims', { k, got: p.covariance.length });
    for (let i = 0; i < k; i++) {
        if (!Array.isArray(p.covariance[i]) || p.covariance[i].length !== k)
                                                              return t('view.marginal_var.validate.cov_row', { i, k });
        for (let j = 0; j < k; j++) {
            if (!Number.isFinite(p.covariance[i][j]))         return t('view.marginal_var.validate.cov_finite', { i, j });
        }
    }
    if (p.labels != null && (!Array.isArray(p.labels) || p.labels.length !== k))
                                                              return t('view.marginal_var.validate.labels', { k });
    if (!Number.isFinite(input.z_alpha) || input.z_alpha <= 0) return t('view.marginal_var.validate.z_alpha');
    return null;
}

export function buildBody(input) {
    return {
        portfolio: {
            weights:    input.portfolio.weights,
            covariance: input.portfolio.covariance,
        },
        z_alpha: input.z_alpha,
    };
}

// Pure-JS mirror of crates/traderview-core/src/marginal_var.rs::analyze.
// Returns null on validation failure / negative variance.
// Returns all-zero report for fully-hedged portfolios (vol == 0).
export function localAnalyze(portfolio, z_alpha) {
    const err = validateInputs({ portfolio, z_alpha });
    if (err) return null;
    const k = portfolio.weights.length;
    const sigma_w = new Array(k);
    for (let i = 0; i < k; i++) {
        let s = 0;
        for (let j = 0; j < k; j++) s += portfolio.covariance[i][j] * portfolio.weights[j];
        sigma_w[i] = s;
    }
    let var_p = 0;
    for (let i = 0; i < k; i++) var_p += portfolio.weights[i] * sigma_w[i];
    if (!Number.isFinite(var_p) || var_p < 0) return null;
    const vol = Math.sqrt(var_p);
    if (vol === 0) {
        const zeros = new Array(k).fill(0);
        return {
            portfolio_var:    0,
            portfolio_vol:    0,
            marginal_var:     zeros.slice(),
            component_var:    zeros.slice(),
            pct_contribution: zeros.slice(),
        };
    }
    const port_var_alpha = vol * z_alpha;
    const marginal = new Array(k);
    const component = new Array(k);
    const pct = new Array(k);
    for (let i = 0; i < k; i++) {
        marginal[i] = z_alpha * sigma_w[i] / vol;
        component[i] = portfolio.weights[i] * marginal[i];
        pct[i] = component[i] / port_var_alpha * 100;
    }
    return {
        portfolio_var:    port_var_alpha,
        portfolio_vol:    vol,
        marginal_var:     marginal,
        component_var:    component,
        pct_contribution: pct,
    };
}

// Build covariance from per-asset vols + correlation matrix (small entry helper).
export function covFromVolsAndCorr(vols, corr) {
    const n = vols.length;
    const out = Array.from({ length: n }, () => new Array(n).fill(0));
    for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) out[i][j] = vols[i] * vols[j] * corr[i][j];
    }
    return out;
}

// Parse the dual-input blob format used by the view:
//   line per asset: "label weight"
// followed by an empty line, then a square matrix (one row per line,
// comma/whitespace-separated cells).
export function parsePortfolioBlob(blob) {
    const out = { weights: [], labels: [], covariance: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const sections = blob.split(/\n\s*\n/).map(s => s.trim()).filter(s => s.length > 0);
    if (sections.length < 2) {
        out.errors.push({ line_no: 0, message: 'expected 2 sections separated by a blank line (weights, then covariance matrix)' });
        return out;
    }
    // Weights section.
    const wLines = sections[0].split('\n');
    for (let i = 0; i < wLines.length; i++) {
        const raw = wLines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        let label, weight;
        if (toks.length === 1) {
            label = `pos_${out.weights.length + 1}`;
            weight = Number(toks[0]);
        } else if (toks.length === 2) {
            label = toks[0];
            weight = Number(toks[1]);
        } else {
            out.errors.push({ line_no: i + 1, message: 'weight row needs 1 or 2 tokens' });
            continue;
        }
        if (!Number.isFinite(weight)) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.weight_not_finite') });
            continue;
        }
        out.weights.push(weight);
        out.labels.push(label);
    }
    // Covariance section.
    const mLines = sections[1].split('\n');
    for (let i = 0; i < mLines.length; i++) {
        const raw = mLines[i].split('#')[0].trim();
        if (!raw) continue;
        const cells = raw.split(/[\s,]+/).filter(t => t.length > 0).map(Number);
        if (cells.some(v => !Number.isFinite(v))) {
            out.errors.push({ line_no: i + 1, message: 'non-finite cell in matrix' });
            continue;
        }
        out.covariance.push(cells);
    }
    return out;
}

export function portfolioToBlob(labels, weights, covariance) {
    const wPart = weights.map((w, i) => `${labels[i]} ${w}`).join('\n');
    const mPart = covariance.map(row => row.join(', ')).join('\n');
    return wPart + '\n\n' + mPart;
}

// 5-tier concentration verdict from pct_contribution vector.
export function concentrationBadge(pctVec) {
    if (!Array.isArray(pctVec) || pctVec.length === 0) return { key: 'view.mvar.badge.unknown', cls: '' };
    let maxAbs = 0;
    for (const v of pctVec) {
        const a = Math.abs(v);
        if (Number.isFinite(a) && a > maxAbs) maxAbs = a;
    }
    if (maxAbs >= 80) return { key: 'view.mvar.badge.extreme',     cls: 'neg' };
    if (maxAbs >= 60) return { key: 'view.mvar.badge.concentrated', cls: 'neg' };
    if (maxAbs >= 40) return { key: 'view.mvar.badge.tilted',      cls: '' };
    if (maxAbs >= 25) return { key: 'view.mvar.badge.balanced',    cls: 'pos' };
    return { key: 'view.mvar.badge.well_diversified', cls: 'pos' };
}

// Single-position alarm — flags when a single name dominates.
export function positionBadge(pct, n) {
    if (!Number.isFinite(pct) || !Number.isFinite(n) || n <= 0) return { key: 'view.mvar.pos.unknown', cls: '' };
    const equal = 100 / n;
    const ratio = Math.abs(pct) / equal;
    if (ratio < 0.5)  return { key: 'view.mvar.pos.under',    cls: '' };
    if (ratio < 1.5)  return { key: 'view.mvar.pos.fair',     cls: 'pos' };
    if (ratio < 3)    return { key: 'view.mvar.pos.over',     cls: '' };
    return { key: 'view.mvar.pos.dominant', cls: 'neg' };
}

// Synthetic demos.
export function makeDemoInput(kind = 'mixed-3') {
    switch (kind) {
        case 'mixed-3':
            return { portfolio: {
                weights: [0.3, 0.4, 0.3],
                covariance: [
                    [0.04, 0.01, 0.005],
                    [0.01, 0.09, 0.02],
                    [0.005, 0.02, 0.16],
                ],
                labels: ['SPY', 'EMB', 'GLD'],
            }, z_alpha: DEFAULT_Z_ALPHA };
        case 'equal-uncorr':
            return { portfolio: {
                weights: [1 / 3, 1 / 3, 1 / 3],
                covariance: diag([0.04, 0.04, 0.04]),
                labels: ['A', 'B', 'C'],
            }, z_alpha: DEFAULT_Z_ALPHA };
        case 'concentrated': {
            // 70% one name, 10% × 3.
            return { portfolio: {
                weights: [0.70, 0.10, 0.10, 0.10],
                covariance: diag([0.09, 0.04, 0.04, 0.04]),
                labels: ['NVDA', 'SPY', 'AGG', 'GLD'],
            }, z_alpha: DEFAULT_Z_ALPHA };
        }
        case 'hedged-pair':
            return { portfolio: {
                weights: [1, -1],
                covariance: [[0.04, 0.04], [0.04, 0.04]],
                labels: ['long', 'short'],
            }, z_alpha: DEFAULT_Z_ALPHA };
        case 'two-asset-corr': {
            // Highly correlated 2-asset.
            return { portfolio: {
                weights: [0.5, 0.5],
                covariance: covFromVolsAndCorr([0.2, 0.2], [[1, 0.9], [0.9, 1]]),
                labels: ['stocks_A', 'stocks_B'],
            }, z_alpha: DEFAULT_Z_ALPHA };
        }
        case 'diversifier': {
            // 4 corr=0.7 risk assets + 1 negatively-correlated diversifier.
            const vols = [0.18, 0.18, 0.18, 0.18, 0.10];
            const corr = [
                [1.0,  0.7,  0.7,  0.7, -0.30],
                [0.7,  1.0,  0.7,  0.7, -0.30],
                [0.7,  0.7,  1.0,  0.7, -0.30],
                [0.7,  0.7,  0.7,  1.0, -0.30],
                [-0.30, -0.30, -0.30, -0.30, 1.0],
            ];
            return { portfolio: {
                weights: [0.225, 0.225, 0.225, 0.225, 0.10],
                covariance: covFromVolsAndCorr(vols, corr),
                labels: ['EQ1','EQ2','EQ3','EQ4','BONDS'],
            }, z_alpha: DEFAULT_Z_ALPHA };
        }
        case '99-pct-vad':
            return { ...makeDemoInput('mixed-3'), z_alpha: 2.326 };
        case 'tight-99-9':
            return { ...makeDemoInput('mixed-3'), z_alpha: 3.090 };
        default:
            return makeDemoInput('mixed-3');
    }
}

function diag(vals) {
    const n = vals.length;
    const m = Array.from({ length: n }, () => new Array(n).fill(0));
    for (let i = 0; i < n; i++) m[i][i] = vals[i];
    return m;
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function fmtPctNum(v, d = 2) {
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

export function assetLabel(labels, i) {
    if (Array.isArray(labels) && i >= 0 && i < labels.length && labels[i] != null) return String(labels[i]);
    if (i < 26) return String.fromCharCode(65 + i);
    return `A${i - 25}`;
}
