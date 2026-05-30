// Cholesky decomposition helpers.
//
// Backend body: { matrix: number[][] }  — symmetric positive-definite matrix.
// Returns: { l: number[][], sqrt_determinant: number } | null

import { t } from './i18n.js';

export const MIN_N = 1;
export const MAX_N = 50;

export const DEFAULT_INPUTS = {
    matrix: [
        [4, 12, -16],
        [12, 37, -43],
        [-16, -43, 98],
    ],
};

export function validateInputs(input) {
    if (!Array.isArray(input.matrix))                       return t('view.cholesky.validate.matrix_array');
    const n = input.matrix.length;
    if (n < MIN_N || n > MAX_N)                             return t('view.cholesky.validate.size_range', { min: MIN_N, max: MAX_N });
    for (let i = 0; i < n; i++) {
        const row = input.matrix[i];
        if (!Array.isArray(row))                            return t('view.cholesky.validate.row_array', { i });
        if (row.length !== n)                               return t('view.cholesky.validate.row_length', { i, got: row.length, n });
        for (let j = 0; j < n; j++) {
            if (typeof row[j] !== 'number' || !Number.isFinite(row[j]))
                                                              return t('view.cholesky.validate.cell_finite', { i, j });
        }
    }
    return null;
}

export function buildBody(input) {
    return { matrix: input.matrix.map(r => r.slice()) };
}

// Pure-JS mirror of crates/traderview-core/src/cholesky.rs::decompose.
export function localDecompose(a) {
    const n = a.length;
    if (n === 0) return null;
    for (const row of a) if (row.length !== n) return null;
    for (const row of a) for (const c of row) if (!Number.isFinite(c)) return null;
    // Symmetry check.
    for (let i = 0; i < n; i++) {
        for (let j = i + 1; j < n; j++) {
            const tol = 1e-9 * (1 + Math.abs(a[i][j]) + Math.abs(a[j][i]));
            if (Math.abs(a[i][j] - a[j][i]) > tol) return null;
        }
    }
    const l = Array.from({ length: n }, () => new Array(n).fill(0));
    for (let i = 0; i < n; i++) {
        for (let j = 0; j <= i; j++) {
            let sum = 0;
            for (let k = 0; k < j; k++) sum += l[i][k] * l[j][k];
            if (i === j) {
                const diag = a[i][i] - sum;
                if (diag <= 0 || !Number.isFinite(diag)) return null;
                l[i][j] = Math.sqrt(diag);
            } else {
                if (Math.abs(l[j][j]) < 1e-18) return null;
                l[i][j] = (a[i][j] - sum) / l[j][j];
            }
        }
    }
    let sqrt_det = 1;
    for (let i = 0; i < n; i++) sqrt_det *= l[i][i];
    return { l, sqrt_determinant: sqrt_det };
}

// L · z product (correlated-draw transform).
export function localMultiply(l, z) {
    const n = l.length;
    if (z.length !== n) return null;
    for (const row of l) if (row.length !== n) return null;
    const out = new Array(n).fill(0);
    for (let i = 0; i < n; i++) {
        for (let j = 0; j <= i; j++) out[i] += l[i][j] * z[j];
    }
    return out;
}

// Parse blob: one row per line, tokens = numeric entries.
export function parseMatrixBlob(blob) {
    const out = { matrix: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        const row = [];
        let bad = false;
        for (let j = 0; j < parts.length; j++) {
            const v = Number(parts[j].replace(/[\$,]/g, ''));
            if (!Number.isFinite(v)) {
                out.errors.push({ line_no: i + 1, message: `token "${parts[j]}" not finite` });
                bad = true;
                break;
            }
            row.push(v);
        }
        if (!bad && row.length > 0) out.matrix.push(row);
    }
    if (out.matrix.length > 0 && out.matrix.some(r => r.length !== out.matrix.length)) {
        out.errors.push({ line_no: 0, message: t('view.cholesky.parse.matrix_square') });
    }
    return out;
}

export function matrixToBlob(matrix) {
    return matrix.map(r => r.join(' ')).join('\n');
}

// Definite-positiveness verdict.
export function statusBadge(report) {
    if (!report) return { key: 'view.chol.status.not_pd', cls: 'neg' };
    if (!Number.isFinite(report.sqrt_determinant)) return { key: 'view.chol.status.unknown', cls: '' };
    return { key: 'view.chol.status.pd', cls: 'pos' };
}

// Conditioning verdict from L's diagonal ratio (max/min |l_ii|).
export function conditionBadge(report) {
    if (!report || !Array.isArray(report.l)) return { key: 'view.chol.cond.unknown', cls: '' };
    const n = report.l.length;
    if (n === 0) return { key: 'view.chol.cond.unknown', cls: '' };
    let mx = 0, mn = Infinity;
    for (let i = 0; i < n; i++) {
        const d = Math.abs(report.l[i][i]);
        if (d > mx) mx = d;
        if (d < mn) mn = d;
    }
    if (mn === 0 || !Number.isFinite(mn)) return { key: 'view.chol.cond.singular', cls: 'neg' };
    const ratio = mx / mn;
    if (ratio < 10)    return { key: 'view.chol.cond.well_cond',  cls: 'pos' };
    if (ratio < 100)   return { key: 'view.chol.cond.moderate',   cls: '' };
    if (ratio < 10000) return { key: 'view.chol.cond.ill_cond',   cls: 'neg' };
    return { key: 'view.chol.cond.severely_ill', cls: 'neg' };
}

// Off-diagonal scale (Frobenius norm of strict lower triangle / diagonal norm).
export function offDiagScale(l) {
    if (!Array.isArray(l) || l.length === 0) return NaN;
    const n = l.length;
    let off = 0, diag = 0;
    for (let i = 0; i < n; i++) {
        diag += l[i][i] * l[i][i];
        for (let j = 0; j < i; j++) off += l[i][j] * l[i][j];
    }
    if (diag === 0) return NaN;
    return Math.sqrt(off) / Math.sqrt(diag);
}

// Verify L·Lᵀ ≈ A and return max relative error.
export function reconstructionError(a, l) {
    if (!Array.isArray(a) || !Array.isArray(l)) return NaN;
    const n = a.length;
    if (l.length !== n) return NaN;
    let maxErr = 0;
    for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
            let s = 0;
            for (let k = 0; k < n; k++) s += l[i][k] * l[j][k];
            const err = Math.abs(s - a[i][j]);
            const ref = Math.max(1e-12, Math.abs(a[i][j]));
            maxErr = Math.max(maxErr, err / ref);
        }
    }
    return maxErr;
}

export function summarizeMatrix(a) {
    if (!Array.isArray(a) || a.length === 0) {
        return { n: 0, trace: NaN, max_diag: NaN, min_diag: NaN, max_abs_off: NaN };
    }
    const n = a.length;
    let trace = 0, mxD = -Infinity, mnD = Infinity, mxOff = 0;
    for (let i = 0; i < n; i++) {
        const d = a[i][i];
        trace += d;
        if (d > mxD) mxD = d;
        if (d < mnD) mnD = d;
        for (let j = 0; j < n; j++) {
            if (i === j) continue;
            const v = Math.abs(a[i][j]);
            if (v > mxOff) mxOff = v;
        }
    }
    return {
        n,
        trace,
        max_diag: Number.isFinite(mxD) ? mxD : NaN,
        min_diag: Number.isFinite(mnD) ? mnD : NaN,
        max_abs_off: mxOff,
    };
}

export function makeDemoInput(kind = 'kershaw') {
    switch (kind) {
        case 'kershaw': {
            // Classic Kershaw 3×3 example.
            return { matrix: [
                [4, 12, -16],
                [12, 37, -43],
                [-16, -43, 98],
            ] };
        }
        case 'identity-4': {
            const m = Array.from({ length: 4 }, (_, i) =>
                Array.from({ length: 4 }, (_, j) => (i === j ? 1 : 0)));
            return { matrix: m };
        }
        case 'diagonal-3': {
            return { matrix: [
                [4, 0, 0],
                [0, 9, 0],
                [0, 0, 16],
            ] };
        }
        case 'correlation-2x2': {
            return { matrix: [
                [1.0, 0.5],
                [0.5, 1.0],
            ] };
        }
        case 'covariance-3x3': {
            return { matrix: [
                [1.0,  0.6,  0.3],
                [0.6,  4.0,  1.5],
                [0.3,  1.5,  9.0],
            ] };
        }
        case 'not-pd': {
            // Indefinite — should fail decomposition.
            return { matrix: [
                [1, 2],
                [2, 1],
            ] };
        }
        case 'asymmetric': {
            return { matrix: [
                [1.0, 0.5],
                [0.7, 1.0],
            ] };
        }
        case 'large-cov-5': {
            // 5×5 covariance-like SPD matrix.
            return { matrix: [
                [4.0,  1.2,  0.8,  0.5,  0.3],
                [1.2,  9.0,  2.1,  1.0,  0.7],
                [0.8,  2.1, 16.0,  3.0,  1.5],
                [0.5,  1.0,  3.0, 25.0,  4.0],
                [0.3,  0.7,  1.5,  4.0, 36.0],
            ] };
        }
        default: return makeDemoInput('kershaw');
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

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtSci(v, d = 3) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toExponential(d);
}
