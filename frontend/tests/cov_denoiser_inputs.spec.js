// Cov Denoiser pure helpers: validator (with MP-specific T ≥ N rule),
// payload builder, MP bulk-edge formula, matrix deltas.

import { test, expect } from 'vitest';
import {
    parseCovariance, validateInputs, buildBody,
    marchenkoPasturBulk, maxAbsDelta, frobeniusRelDelta,
} from '../js/_cov_denoiser_inputs.js';

const goodCov = [
    [0.04, 0.01, 0.005],
    [0.01, 0.09, 0.02],
    [0.005, 0.02, 0.16],
];

// ── parseCovariance ────────────────────────────────────────────────

test('parseCovariance reuses the portfolio-allocator matrix parser', () => {
    const r = parseCovariance('0.04 0.01\n0.01 0.09');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([[0.04, 0.01], [0.01, 0.09]]);
});

// ── validateInputs ─────────────────────────────────────────────────

test('validate inherits portfolio-allocator cov checks', () => {
    expect(validateInputs([[1, 2], [3]], 100)).toMatch(/must have/);
    expect(validateInputs([[0, 0], [0, 1]], 100)).toMatch(/diagonal/);
    expect(validateInputs([[1, 0.5], [0.4, 1]], 100)).toMatch(/symmetric/);
});

test('validate rejects bad T (non-integer / non-positive)', () => {
    expect(validateInputs(goodCov, 0)).toMatch(/T must be a positive/);
    expect(validateInputs(goodCov, 2.5)).toMatch(/positive integer/);
});

test('validate rejects T < N (q would exceed 1)', () => {
    expect(validateInputs(goodCov, 2)).toMatch(/T \(2\) must be ≥ N \(3\)/);
});

test('validate accepts T ≥ N with a clean cov', () => {
    expect(validateInputs(goodCov, 3)).toBe(null);
    expect(validateInputs(goodCov, 100)).toBe(null);
});

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits the backend shape', () => {
    expect(buildBody(goodCov, 60)).toEqual({
        covariance: goodCov,
        num_observations: 60,
    });
});

// ── marchenkoPasturBulk ────────────────────────────────────────────

test('MP bulk: q=0 → edges collapse to σ²', () => {
    // Note: q must be > 0 (q=0 means T=∞, well-conditioned, no bulk).
    // The helper returns null for q=0 — verified separately below.
    expect(marchenkoPasturBulk(1, 0)).toBe(null);
});

test('MP bulk: q=1 → λ_min=0, λ_max=4σ²', () => {
    const bulk = marchenkoPasturBulk(1, 1);
    expect(bulk.lambda_min).toBeCloseTo(0, 12);
    expect(bulk.lambda_max).toBeCloseTo(4, 12);
});

test('MP bulk: q=0.25 → λ_min=σ²·0.25, λ_max=σ²·2.25', () => {
    const bulk = marchenkoPasturBulk(0.05, 0.25);
    expect(bulk.lambda_min).toBeCloseTo(0.05 * 0.25, 12);
    expect(bulk.lambda_max).toBeCloseTo(0.05 * 2.25, 12);
});

test('MP bulk rejects σ²≤0 / q out of (0,1]', () => {
    expect(marchenkoPasturBulk(0, 0.5)).toBe(null);
    expect(marchenkoPasturBulk(-1, 0.5)).toBe(null);
    expect(marchenkoPasturBulk(1, 1.5)).toBe(null);
    expect(marchenkoPasturBulk(1, NaN)).toBe(null);
});

// ── maxAbsDelta ────────────────────────────────────────────────────

test('maxAbsDelta finds the largest cell change', () => {
    const a = [[1, 2], [3, 4]];
    const b = [[1, 2.5], [3, 4]];
    expect(maxAbsDelta(a, b)).toBe(0.5);
});

test('maxAbsDelta returns 0 for identical matrices', () => {
    expect(maxAbsDelta([[1, 2], [3, 4]], [[1, 2], [3, 4]])).toBe(0);
});

test('maxAbsDelta returns null on shape mismatch', () => {
    expect(maxAbsDelta([[1, 2]], [[1, 2], [3, 4]])).toBe(null);
    expect(maxAbsDelta([[1, 2]], [[1]])).toBe(null);
    expect(maxAbsDelta(null, [[1]])).toBe(null);
});

// ── frobeniusRelDelta ─────────────────────────────────────────────

test('frobeniusRelDelta is 0 for identical matrices', () => {
    expect(frobeniusRelDelta(goodCov, goodCov)).toBe(0);
});

test('frobeniusRelDelta scales correctly with change magnitude', () => {
    const a = [[1, 0], [0, 1]];
    const b = [[1.1, 0], [0, 1]];
    // ||Δ||_F = 0.1; ||A||_F = √2; ratio = 0.1 / √2.
    expect(frobeniusRelDelta(a, b)).toBeCloseTo(0.1 / Math.sqrt(2), 9);
});

test('frobeniusRelDelta returns null on zero-norm input', () => {
    expect(frobeniusRelDelta([[0, 0], [0, 0]], [[0.1, 0], [0, 0]])).toBe(null);
});

test('frobeniusRelDelta returns null on shape mismatch', () => {
    expect(frobeniusRelDelta([[1, 2]], [[1, 2], [3, 4]])).toBe(null);
});
