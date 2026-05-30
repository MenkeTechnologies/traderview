// Black-Litterman portfolio model helpers (He-Litterman 1999 posterior).
//
// Backend body: { inputs: { covariance, equilibrium_returns, view_loadings,
//   view_returns, view_confidence, tau } }
// Returns: { posterior_returns, posterior_covariance } | null
//
// Posterior:
//   A = (τΣ)⁻¹ + PᵀΩ⁻¹P
//   μ_bl = A⁻¹ · [(τΣ)⁻¹ π + PᵀΩ⁻¹ Q]
//   Σ_bl = Σ + A⁻¹

import { t } from './i18n.js';

export const DEFAULT_TAU = 0.05;

export const DEFAULT_INPUTS = {
    inputs: {
        covariance: [[0.04, 0.01], [0.01, 0.09]],
        equilibrium_returns: [0.05, 0.07],
        view_loadings: [[1.0, -1.0]],
        view_returns: [0.02],
        view_confidence: [[0.001]],
        tau: DEFAULT_TAU,
        labels: ['A', 'B'],
        view_labels: ['view_1'],
    },
};

export function validateInputs(input) {
    const inp = input.inputs;
    if (!inp) return t('view.black_litterman.validate.inputs_missing');
    if (!Array.isArray(inp.equilibrium_returns) || inp.equilibrium_returns.length === 0)
                                                                  return t('view.black_litterman.validate.eq_array');
    const n = inp.equilibrium_returns.length;
    for (let i = 0; i < n; i++) {
        if (!Number.isFinite(inp.equilibrium_returns[i]))         return t('view.black_litterman.validate.eq_finite', { i });
    }
    if (!Array.isArray(inp.covariance) || inp.covariance.length !== n)
                                                                  return t('view.black_litterman.validate.cov_dims', { n });
    for (let i = 0; i < n; i++) {
        if (!Array.isArray(inp.covariance[i]) || inp.covariance[i].length !== n)
                                                                  return t('view.black_litterman.validate.cov_row', { i, n });
        for (let j = 0; j < n; j++) {
            if (!Number.isFinite(inp.covariance[i][j]))           return t('view.black_litterman.validate.cov_finite', { i, j });
        }
    }
    if (!Number.isFinite(inp.tau) || inp.tau <= 0)                return t('view.black_litterman.validate.tau');
    // Views are optional: k can be 0.
    const k = Array.isArray(inp.view_returns) ? inp.view_returns.length : 0;
    if (k > 0) {
        if (!Array.isArray(inp.view_loadings) || inp.view_loadings.length !== k)
                                                                  return t('view.black_litterman.validate.loadings_dims', { k, n });
        for (let i = 0; i < k; i++) {
            if (!Array.isArray(inp.view_loadings[i]) || inp.view_loadings[i].length !== n)
                                                                  return t('view.black_litterman.validate.loadings_row', { i, n });
            for (let j = 0; j < n; j++) {
                if (!Number.isFinite(inp.view_loadings[i][j]))    return t('view.black_litterman.validate.loadings_finite', { i, j });
            }
        }
        if (!Array.isArray(inp.view_confidence) || inp.view_confidence.length !== k)
                                                                  return t('view.black_litterman.validate.conf_dims', { k });
        for (let i = 0; i < k; i++) {
            if (!Array.isArray(inp.view_confidence[i]) || inp.view_confidence[i].length !== k)
                                                                  return t('view.black_litterman.validate.conf_row', { i, k });
            for (let j = 0; j < k; j++) {
                if (!Number.isFinite(inp.view_confidence[i][j])) return t('view.black_litterman.validate.conf_finite', { i, j });
            }
        }
        for (let i = 0; i < k; i++) {
            if (!Number.isFinite(inp.view_returns[i]))           return t('view.black_litterman.validate.view_finite', { i });
        }
    }
    return null;
}

export function buildBody(input) {
    const inp = input.inputs;
    return {
        inputs: {
            covariance:           inp.covariance,
            equilibrium_returns:  inp.equilibrium_returns,
            view_loadings:        inp.view_loadings,
            view_returns:         inp.view_returns,
            view_confidence:      inp.view_confidence,
            tau:                  inp.tau,
        },
    };
}

// ── Pure-JS matrix helpers (mirror Rust scale/transpose/matmul/matvec/matadd/invert) ──

function scale(m, s) {
    return m.map(r => r.map(v => v * s));
}

function transpose(m) {
    if (m.length === 0) return [];
    const rows = m.length, cols = m[0].length;
    const out = Array.from({ length: cols }, () => new Array(rows).fill(0));
    for (let i = 0; i < rows; i++) for (let j = 0; j < cols; j++) out[j][i] = m[i][j];
    return out;
}

function matmul(a, b) {
    const ar = a.length, ac = a[0]?.length ?? 0, bc = b[0]?.length ?? 0;
    const out = Array.from({ length: ar }, () => new Array(bc).fill(0));
    for (let i = 0; i < ar; i++) {
        for (let j = 0; j < bc; j++) {
            let s = 0;
            for (let k = 0; k < ac; k++) s += a[i][k] * b[k][j];
            out[i][j] = s;
        }
    }
    return out;
}

function matvec(m, v) {
    return m.map(r => {
        let s = 0;
        for (let i = 0; i < v.length; i++) s += r[i] * v[i];
        return s;
    });
}

function matadd(a, b) {
    return a.map((ra, i) => ra.map((x, j) => x + b[i][j]));
}

// Gauss-Jordan invert with partial pivoting; returns null on singular matrix.
function invert(m) {
    const n = m.length;
    if (n === 0 || m.some(r => r.length !== n)) return null;
    const aug = Array.from({ length: n }, () => new Array(2 * n).fill(0));
    for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
            aug[i][j] = m[i][j];
            aug[i][n + j] = (i === j) ? 1 : 0;
        }
    }
    for (let i = 0; i < n; i++) {
        let pivot = i;
        for (let r = i + 1; r < n; r++) {
            if (Math.abs(aug[r][i]) > Math.abs(aug[pivot][i])) pivot = r;
        }
        if (Math.abs(aug[pivot][i]) < 1e-18) return null;
        if (pivot !== i) { const t = aug[i]; aug[i] = aug[pivot]; aug[pivot] = t; }
        const div = aug[i][i];
        for (let j = 0; j < 2 * n; j++) aug[i][j] /= div;
        for (let r = 0; r < n; r++) {
            if (r === i) continue;
            const f = aug[r][i];
            if (f === 0) continue;
            for (let j = 0; j < 2 * n; j++) aug[r][j] -= f * aug[i][j];
        }
    }
    return aug.map(r => r.slice(n));
}

// Pure-JS mirror of crates/traderview-core/src/black_litterman.rs::solve.
export function localSolve(inp) {
    const err = validateInputs({ inputs: inp });
    if (err) return null;
    const n = inp.equilibrium_returns.length;
    const k = inp.view_returns.length;
    if (k === 0) {
        return {
            posterior_returns:    inp.equilibrium_returns.slice(),
            posterior_covariance: inp.covariance.map(r => r.slice()),
        };
    }
    const tauSigma    = scale(inp.covariance, inp.tau);
    const tauSigmaInv = invert(tauSigma);
    if (!tauSigmaInv) return null;
    const omegaInv = invert(inp.view_confidence);
    if (!omegaInv) return null;
    const pt          = transpose(inp.view_loadings);
    const ptOmegaInv  = matmul(pt, omegaInv);
    const ptOmegaInvP = matmul(ptOmegaInv, inp.view_loadings);
    const A           = matadd(tauSigmaInv, ptOmegaInvP);
    const Ainv        = invert(A);
    if (!Ainv) return null;
    const tsInvPi  = matvec(tauSigmaInv, inp.equilibrium_returns);
    const ptOiQ    = matvec(ptOmegaInv, inp.view_returns);
    const rhs      = tsInvPi.map((v, i) => v + ptOiQ[i]);
    const posterior_returns    = matvec(Ainv, rhs);
    const posterior_covariance = matadd(inp.covariance, Ainv);
    void n;
    return { posterior_returns, posterior_covariance };
}

// Parse a multi-section blob:
//   ASSETS / LABELS line: comma/whitespace separated asset names (1 row)
//   blank line
//   EQUILIBRIUM line: returns (decimal or pct-suffix)
//   blank line
//   COVARIANCE: n × n matrix
//   blank line
//   τ line: "tau 0.05"
//   blank line (optional)
//   VIEWS section: each view = "view_name p_0 p_1 ... p_n q ω"
export function parseBlackLittermanBlob(blob) {
    const out = {
        labels: [], equilibrium_returns: [],
        covariance: [],
        view_loadings: [], view_returns: [], view_confidence: [], view_labels: [],
        tau: DEFAULT_TAU,
        errors: [],
    };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const sections = blob.split(/\n\s*\n/).map(s => s.trim()).filter(s => s.length > 0);
    if (sections.length < 4) {
        out.errors.push({ line_no: 0, message: t('view.black_litterman.parse.expected_sections') });
        return out;
    }
    // Section 1: labels.
    out.labels = sections[0].split('\n')[0].split(/[\s,]+/).filter(t => t.length > 0);
    // Section 2: equilibrium returns.
    out.equilibrium_returns = sections[1].split('\n')[0].split(/[\s,]+/).filter(t => t.length > 0).map(pctOrDec);
    if (out.equilibrium_returns.some(v => !Number.isFinite(v))) {
        out.errors.push({ line_no: 0, message: t('view.black_litterman.parse.equilibrium_finite') });
    }
    // Section 3: covariance matrix.
    for (const line of sections[2].split('\n')) {
        const row = line.split('#')[0].trim();
        if (!row) continue;
        const cells = row.split(/[\s,]+/).filter(t => t.length > 0).map(Number);
        if (cells.some(v => !Number.isFinite(v))) {
            out.errors.push({ line_no: 0, message: t('view.black_litterman.parse.covariance_finite') });
            continue;
        }
        out.covariance.push(cells);
    }
    // Section 4: tau.
    const tauLine = sections[3].split('\n')[0].trim();
    const tauToks = tauLine.split(/[\s,]+/);
    const tauTok = tauToks[tauToks.length - 1];
    out.tau = pctOrDec(tauTok);
    if (!Number.isFinite(out.tau)) {
        out.errors.push({ line_no: 0, message: t('view.black_litterman.parse.tau_not_finite') });
    }
    // Section 5+: views (optional).
    if (sections.length > 4) {
        const n = out.equilibrium_returns.length;
        for (const section of sections.slice(4)) {
            for (const rawLine of section.split('\n')) {
                const line = rawLine.split('#')[0].trim();
                if (!line) continue;
                const toks = line.split(/[\s,]+/).filter(t => t.length > 0);
                // Expect: name p_0 p_1 ... p_{n-1} q ω → n + 3 tokens.
                if (toks.length !== n + 3) {
                    out.errors.push({ line_no: 0, message: `view row needs ${n + 3} tokens (name, ${n}×p, q, ω); got ${toks.length}` });
                    continue;
                }
                const name = toks[0];
                const p_row = toks.slice(1, 1 + n).map(pctOrDec);
                const q = pctOrDec(toks[1 + n]);
                const omega = pctOrDec(toks[2 + n]);
                if (p_row.some(v => !Number.isFinite(v)) || !Number.isFinite(q) || !Number.isFinite(omega)) {
                    out.errors.push({ line_no: 0, message: `view "${name}" has non-finite value` });
                    continue;
                }
                out.view_labels.push(name);
                out.view_loadings.push(p_row);
                out.view_returns.push(q);
            }
        }
        // Build diagonal Ω from collected omegas — we re-read since we need each row's ω again.
        // Easier: re-parse just the ω column.
        out.view_confidence = [];
        let idx = 0;
        for (const section of sections.slice(4)) {
            for (const rawLine of section.split('\n')) {
                const line = rawLine.split('#')[0].trim();
                if (!line) continue;
                const toks = line.split(/[\s,]+/).filter(t => t.length > 0);
                if (toks.length !== n + 3) continue;
                const omega = pctOrDec(toks[2 + n]);
                const row = new Array(out.view_labels.length).fill(0);
                row[idx] = omega;
                out.view_confidence.push(row);
                idx++;
            }
        }
    }
    return out;
}

function pctOrDec(tok) {
    if (typeof tok === 'string' && tok.endsWith('%')) {
        const v = Number(tok.slice(0, -1));
        return Number.isFinite(v) ? v / 100 : NaN;
    }
    return Number(tok);
}

export function blToBlob(inp) {
    const tauPart = `tau ${inp.tau}`;
    let blob = (inp.labels || []).join(' ') + '\n\n'
        + inp.equilibrium_returns.join(' ') + '\n\n'
        + inp.covariance.map(r => r.join(', ')).join('\n') + '\n\n'
        + tauPart;
    if (inp.view_returns && inp.view_returns.length > 0) {
        const viewLines = [];
        for (let i = 0; i < inp.view_returns.length; i++) {
            const name = (inp.view_labels && inp.view_labels[i]) || `view_${i + 1}`;
            const p = inp.view_loadings[i].join(' ');
            const q = inp.view_returns[i];
            const omega = inp.view_confidence[i][i];
            viewLines.push(`${name} ${p} ${q} ${omega}`);
        }
        blob += '\n\n' + viewLines.join('\n');
    }
    return blob;
}

// Confidence verdict for the average diagonal ω.
export function confidenceBadge(view_confidence) {
    if (!Array.isArray(view_confidence) || view_confidence.length === 0)
        return { key: 'view.blit.badge.no_views', cls: '' };
    let sum = 0, n = 0;
    for (let i = 0; i < view_confidence.length; i++) {
        const v = view_confidence[i][i];
        if (Number.isFinite(v) && v > 0) { sum += v; n++; }
    }
    if (n === 0) return { key: 'view.blit.badge.unknown', cls: '' };
    const avg = sum / n;
    if (avg < 1e-5)  return { key: 'view.blit.badge.very_high', cls: 'pos' };
    if (avg < 1e-3)  return { key: 'view.blit.badge.high',      cls: 'pos' };
    if (avg < 1e-1)  return { key: 'view.blit.badge.medium',    cls: '' };
    return { key: 'view.blit.badge.low', cls: 'neg' };
}

// Per-asset tilt vs equilibrium.
export function tiltBadge(diff) {
    if (!Number.isFinite(diff)) return { key: 'view.blit.tilt.unknown', cls: '' };
    if (Math.abs(diff) < 1e-5) return { key: 'view.blit.tilt.unchanged', cls: '' };
    if (diff > 0.01)  return { key: 'view.blit.tilt.strong_up',  cls: 'pos' };
    if (diff > 0)     return { key: 'view.blit.tilt.up',          cls: 'pos' };
    if (diff < -0.01) return { key: 'view.blit.tilt.strong_down', cls: 'neg' };
    return { key: 'view.blit.tilt.down', cls: 'neg' };
}

// Demos.
export function makeDemoInput(kind = 'two-asset-view') {
    switch (kind) {
        case 'two-asset-view':
            return { inputs: { ...DEFAULT_INPUTS.inputs } };
        case 'no-views':
            return { inputs: { ...DEFAULT_INPUTS.inputs,
                view_loadings: [], view_returns: [], view_confidence: [], view_labels: [] } };
        case 'very-confident': {
            const base = { ...DEFAULT_INPUTS.inputs };
            return { inputs: { ...base, view_confidence: [[1e-8]] } };
        }
        case 'very-loose': {
            const base = { ...DEFAULT_INPUTS.inputs };
            return { inputs: { ...base, view_confidence: [[1e8]] } };
        }
        case 'three-asset':
            return { inputs: {
                labels: ['SPY', 'AGG', 'GLD'],
                equilibrium_returns: [0.06, 0.03, 0.04],
                covariance: [
                    [0.04, 0.005, 0.005],
                    [0.005, 0.01, 0.0],
                    [0.005, 0.0, 0.03],
                ],
                view_loadings: [
                    [1, -1, 0],     // SPY beats AGG by Q
                    [0, 0, 1],      // GLD absolute view
                ],
                view_returns: [0.04, 0.06],
                view_confidence: [
                    [0.0005, 0],
                    [0, 0.001],
                ],
                tau: 0.05,
                view_labels: ['SPY_vs_AGG', 'GLD_abs'],
            }};
        case 'two-views-conflict': {
            return { inputs: {
                labels: ['A', 'B'],
                equilibrium_returns: [0.05, 0.07],
                covariance: [[0.04, 0.01], [0.01, 0.09]],
                view_loadings: [[1, 0], [0, 1]],
                view_returns: [0.08, 0.05],   // contradicts equilibrium
                view_confidence: [[0.0001], [0.0001]]
                    .map((r, i, all) => Array.from({ length: all.length }, (_, j) => i === j ? r[0] : 0)),
                tau: 0.05,
                view_labels: ['A_abs', 'B_abs'],
            }};
        }
        case 'low-tau': {
            const base = { ...DEFAULT_INPUTS.inputs };
            return { inputs: { ...base, tau: 0.001 } };
        }
        case 'large-tau': {
            const base = { ...DEFAULT_INPUTS.inputs };
            return { inputs: { ...base, tau: 0.5 } };
        }
        default: return makeDemoInput('two-asset-view');
    }
}

export function fmtPctSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 6) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtSci(v) {
    if (!Number.isFinite(v)) return '—';
    if (v === 0) return '0';
    if (Math.abs(v) >= 1e-4) return v.toFixed(6);
    return v.toExponential(3);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function assetLabel(labels, i) {
    if (Array.isArray(labels) && i >= 0 && i < labels.length && labels[i] != null) return String(labels[i]);
    return String.fromCharCode(65 + i);
}
