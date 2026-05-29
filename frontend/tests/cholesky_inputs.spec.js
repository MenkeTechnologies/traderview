// Cholesky decomposition helpers: parser, validator, localDecompose parity, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, MIN_N, MAX_N,
    parseMatrixBlob, matrixToBlob, validateInputs, buildBody,
    localDecompose, localMultiply,
    statusBadge, conditionBadge, offDiagScale, reconstructionError, summarizeMatrix,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtInt, fmtSci,
} from '../js/_cholesky_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseMatrixBlob: simple 3×3', () => {
    const r = parseMatrixBlob('4 12 -16\n12 37 -43\n-16 -43 98');
    expect(r.errors).toEqual([]);
    expect(r.matrix).toEqual([[4, 12, -16], [12, 37, -43], [-16, -43, 98]]);
});

test('parseMatrixBlob: comments + blank lines ignored', () => {
    const r = parseMatrixBlob('# header\n4 0\n\n0 9 # diag\n');
    expect(r.errors).toEqual([]);
    expect(r.matrix).toEqual([[4, 0], [0, 9]]);
});

test('parseMatrixBlob: non-numeric token', () => {
    const r = parseMatrixBlob('4 bad\n0 9');
    expect(r.errors[0].message).toMatch(/not finite/);
});

test('parseMatrixBlob: non-square flagged', () => {
    const r = parseMatrixBlob('1 2 3\n4 5');
    expect(r.errors.some(e => /square/.test(e.message))).toBe(true);
});

test('parseMatrixBlob: non-string returns 1 error', () => {
    const r = parseMatrixBlob(undefined);
    expect(r.errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default', () => {
    expect(validateInputs(DEFAULT_INPUTS)).toBe(null);
});

test('validate rejects bad shapes', () => {
    expect(validateInputs({ matrix: 'no' })).toMatch(/2D array/);
    expect(validateInputs({ matrix: [] })).toMatch(/matrix size/);
    expect(validateInputs({ matrix: [[1, 2]] })).toMatch(/has length 2/);
    expect(validateInputs({ matrix: [[NaN]] })).toMatch(/not finite/);
});

test('validate rejects too large', () => {
    const big = Array.from({ length: MAX_N + 1 }, () => new Array(MAX_N + 1).fill(0));
    expect(validateInputs({ matrix: big })).toMatch(/matrix size/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody clones rows', () => {
    const inp = { matrix: [[1, 2], [3, 4]] };
    const body = buildBody(inp);
    body.matrix[0][0] = 99;
    expect(inp.matrix[0][0]).toBe(1);
});

// ── localDecompose parity (mirrors every Rust #[test]) ────────────

test('local empty returns null', () => {
    expect(localDecompose([])).toBe(null);
});

test('local non-square returns null', () => {
    expect(localDecompose([[1, 0.5]])).toBe(null);
});

test('local asymmetric returns null', () => {
    expect(localDecompose([[1, 0.5], [0.7, 1]])).toBe(null);
});

test('local non-PD returns null (indefinite)', () => {
    expect(localDecompose([[1, 2], [2, 1]])).toBe(null);
});

test('local NaN input returns null', () => {
    expect(localDecompose([[1, NaN], [NaN, 1]])).toBe(null);
});

test('local identity yields identity', () => {
    const a = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
    const r = localDecompose(a);
    for (let i = 0; i < 3; i++) {
        for (let j = 0; j < 3; j++) {
            const expected = i === j ? 1 : 0;
            expect(Math.abs(r.l[i][j] - expected)).toBeLessThan(1e-12);
        }
    }
    expect(Math.abs(r.sqrt_determinant - 1)).toBeLessThan(1e-12);
});

test('local diagonal yields sqrt diagonal', () => {
    const r = localDecompose([[4, 0], [0, 9]]);
    expect(Math.abs(r.l[0][0] - 2)).toBeLessThan(1e-12);
    expect(Math.abs(r.l[1][1] - 3)).toBeLessThan(1e-12);
    expect(Math.abs(r.sqrt_determinant - 6)).toBeLessThan(1e-12);
});

test('local LLᵀ = A for Kershaw 3×3', () => {
    const a = [[4, 12, -16], [12, 37, -43], [-16, -43, 98]];
    const r = localDecompose(a);
    for (let i = 0; i < 3; i++) {
        for (let j = 0; j < 3; j++) {
            let s = 0;
            for (let k = 0; k < 3; k++) s += r.l[i][k] * r.l[j][k];
            expect(Math.abs(s - a[i][j])).toBeLessThan(1e-9);
        }
    }
});

test('local upper triangle is zero', () => {
    const r = localDecompose([[4, 2], [2, 5]]);
    expect(r.l[0][1]).toBe(0);
});

test('local Kershaw exact factor', () => {
    // Known result from textbook: L = [[2,0,0],[6,1,0],[-8,5,3]].
    const r = localDecompose([[4, 12, -16], [12, 37, -43], [-16, -43, 98]]);
    expect(Math.abs(r.l[0][0] - 2)).toBeLessThan(1e-9);
    expect(Math.abs(r.l[1][0] - 6)).toBeLessThan(1e-9);
    expect(Math.abs(r.l[1][1] - 1)).toBeLessThan(1e-9);
    expect(Math.abs(r.l[2][0] - (-8))).toBeLessThan(1e-9);
    expect(Math.abs(r.l[2][1] - 5)).toBeLessThan(1e-9);
    expect(Math.abs(r.l[2][2] - 3)).toBeLessThan(1e-9);
    expect(Math.abs(r.sqrt_determinant - 6)).toBeLessThan(1e-9);
});

test('local multiply with correlated draws', () => {
    // σ² = (4, 9), ρ = 0.5 → cov = [[4,3],[3,9]].
    const r = localDecompose([[4, 3], [3, 9]]);
    const out = localMultiply(r.l, [1, 1]);
    expect(out.length).toBe(2);
    for (const v of out) expect(Number.isFinite(v)).toBe(true);
});

test('local multiply dim mismatch returns null', () => {
    const l = [[1, 0], [0.5, 0.8]];
    expect(localMultiply(l, [1])).toBe(null);
});

// ── badges ────────────────────────────────────────────────────────

test('statusBadge: pd / not_pd / unknown', () => {
    const ok  = localDecompose([[4, 0], [0, 9]]);
    const bad = localDecompose([[1, 2], [2, 1]]);
    expect(statusBadge(ok).key).toMatch(/pd$/);
    expect(statusBadge(bad).key).toMatch(/not_pd/);
    expect(statusBadge({ sqrt_determinant: NaN, l: [[1]] }).key).toMatch(/unknown/);
});

test('conditionBadge: well_cond / moderate / ill_cond / severely_ill / singular / unknown', () => {
    expect(conditionBadge(null).key).toMatch(/unknown/);
    expect(conditionBadge({ l: [[1, 0], [0, 2]],     sqrt_determinant: 2 }).key).toMatch(/well_cond/);
    expect(conditionBadge({ l: [[1, 0], [0, 50]],    sqrt_determinant: 50 }).key).toMatch(/moderate/);
    expect(conditionBadge({ l: [[1, 0], [0, 5000]],  sqrt_determinant: 5000 }).key).toMatch(/ill_cond/);
    expect(conditionBadge({ l: [[1, 0], [0, 1e6]],   sqrt_determinant: 1e6 }).key).toMatch(/severely_ill/);
    expect(conditionBadge({ l: [[0, 0], [0, 1]],     sqrt_determinant: 0 }).key).toMatch(/singular/);
});

test('offDiagScale: identity → 0, dense triangular > 0', () => {
    expect(offDiagScale([[1, 0], [0, 1]])).toBe(0);
    expect(offDiagScale([[1, 0], [0.5, 0.8]])).toBeGreaterThan(0);
});

test('offDiagScale: empty → NaN', () => {
    expect(Number.isNaN(offDiagScale([]))).toBe(true);
});

test('reconstructionError: exact Kershaw → ≈ 0', () => {
    const a = [[4, 12, -16], [12, 37, -43], [-16, -43, 98]];
    const r = localDecompose(a);
    expect(reconstructionError(a, r.l)).toBeLessThan(1e-9);
});

test('reconstructionError: bad dims → NaN', () => {
    expect(Number.isNaN(reconstructionError([[1]], [[1], [2]]))).toBe(true);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeMatrix: 3×3 Kershaw', () => {
    const s = summarizeMatrix([[4, 12, -16], [12, 37, -43], [-16, -43, 98]]);
    expect(s.n).toBe(3);
    expect(s.trace).toBe(4 + 37 + 98);
    expect(s.max_diag).toBe(98);
    expect(s.min_diag).toBe(4);
    expect(s.max_abs_off).toBe(43);
});

test('summarizeMatrix: empty → NaN', () => {
    const s = summarizeMatrix([]);
    expect(s.n).toBe(0);
    expect(Number.isNaN(s.trace)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: every preset validates', () => {
    for (const k of ['kershaw', 'identity-4', 'diagonal-3', 'correlation-2x2',
                     'covariance-3x3', 'not-pd', 'asymmetric', 'large-cov-5']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
    }
});

test('demos: positive-definite presets decompose', () => {
    for (const k of ['kershaw', 'identity-4', 'diagonal-3',
                     'correlation-2x2', 'covariance-3x3', 'large-cov-5']) {
        const r = localDecompose(makeDemoInput(k).matrix);
        expect(r).not.toBe(null);
    }
});

test('demos: not-pd preset returns null', () => {
    expect(localDecompose(makeDemoInput('not-pd').matrix)).toBe(null);
});

test('demos: asymmetric preset returns null', () => {
    expect(localDecompose(makeDemoInput('asymmetric').matrix)).toBe(null);
});

test('demo identity-4: factor is identity', () => {
    const r = localDecompose(makeDemoInput('identity-4').matrix);
    for (let i = 0; i < 4; i++) {
        for (let j = 0; j < 4; j++) {
            const expected = i === j ? 1 : 0;
            expect(Math.abs(r.l[i][j] - expected)).toBeLessThan(1e-12);
        }
    }
});

test('demo correlation-2x2: known factor', () => {
    // [[1,0.5],[0.5,1]] → L[0][0]=1, L[1][0]=0.5, L[1][1]=√0.75.
    const r = localDecompose(makeDemoInput('correlation-2x2').matrix);
    expect(Math.abs(r.l[0][0] - 1)).toBeLessThan(1e-12);
    expect(Math.abs(r.l[1][0] - 0.5)).toBeLessThan(1e-12);
    expect(Math.abs(r.l[1][1] - Math.sqrt(0.75))).toBeLessThan(1e-12);
});

// ── formatters / roundtrip ────────────────────────────────────────

test('matrixToBlob round-trips through parseMatrixBlob', () => {
    const m = [[4, 12, -16], [12, 37, -43], [-16, -43, 98]];
    const back = parseMatrixBlob(matrixToBlob(m));
    expect(back.errors).toEqual([]);
    expect(back.matrix).toEqual(m);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtNumSigned(1.5)).toBe('+1.5000');
    expect(fmtNumSigned(-1.5)).toBe('-1.5000');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtSci(1234.5)).toBe('1.235e+3');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtNumSigned(Infinity)).toBe('—');
    expect(fmtInt(NaN)).toBe('—');
    expect(fmtSci(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(MIN_N).toBe(1);
    expect(MAX_N).toBe(50);
    expect(DEFAULT_INPUTS.matrix.length).toBe(3);
});
