// Kalman Dynamic Beta pure helpers: validator, payload shape,
// summarizer of the β trace.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody, summarizeBetaTrace, fmtBeta,
} from '../js/_kalman_beta_inputs.js';

const goodParams = { process_noise_q: 1e-4, obs_noise_r: 1e-4, beta0: 1.0, p0: 1.0 };

// ── validateInputs ─────────────────────────────────────────────────

test('validate rejects too-short series', () => {
    expect(validateInputs([1, 2], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], goodParams))
        .toMatch(/asset returns/);
    expect(validateInputs([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], [1, 2], goodParams))
        .toMatch(/bench returns/);
});

test('validate rejects mismatched lengths', () => {
    const a = Array(20).fill(0.01);
    const b = Array(15).fill(0.01);
    expect(validateInputs(a, b, goodParams)).toMatch(/same length/);
});

test('validate rejects non-finite values', () => {
    const a = Array(15).fill(0.01); a[3] = NaN;
    const b = Array(15).fill(0.01);
    expect(validateInputs(a, b, goodParams)).toMatch(/asset.*non-finite/);
});

test('validate rejects bad Q / R', () => {
    const a = Array(15).fill(0.01); const b = Array(15).fill(0.01);
    expect(validateInputs(a, b, { ...goodParams, process_noise_q: -1 })).toMatch(/Q/);
    expect(validateInputs(a, b, { ...goodParams, obs_noise_r: 0 })).toMatch(/R/);
});

test('validate rejects bad β₀ / P₀', () => {
    const a = Array(15).fill(0.01); const b = Array(15).fill(0.01);
    expect(validateInputs(a, b, { ...goodParams, beta0: NaN })).toMatch(/β₀/);
    expect(validateInputs(a, b, { ...goodParams, p0: 0 })).toMatch(/P₀/);
});

test('validate accepts good input', () => {
    const a = Array(15).fill(0.01); const b = Array(15).fill(0.01);
    expect(validateInputs(a, b, goodParams)).toBe(null);
});

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits the full backend shape', () => {
    const b = buildBody([1, 2, 3], [4, 5, 6], goodParams);
    expect(b).toEqual({
        asset: [1, 2, 3],
        bench: [4, 5, 6],
        process_noise_q: 1e-4,
        obs_noise_r: 1e-4,
        beta0: 1.0,
        p0: 1.0,
    });
});

// ── summarizeBetaTrace ─────────────────────────────────────────────

test('summarizeBetaTrace returns null for empty / all-null traces', () => {
    expect(summarizeBetaTrace([])).toBe(null);
    expect(summarizeBetaTrace([null, NaN, undefined])).toBe(null);
});

test('summarizeBetaTrace computes mean / min / max correctly', () => {
    const s = summarizeBetaTrace([1.0, 2.0, 3.0]);
    expect(s.mean).toBeCloseTo(2, 12);
    expect(s.min).toBe(1);
    expect(s.max).toBe(3);
    expect(s.latest).toBe(3);
    expect(s.first).toBe(1);
    expect(s.drift).toBe(2);
    expect(s.count).toBe(3);
});

test('summarizeBetaTrace stdev matches population formula', () => {
    // For [1, 2, 3]: mean=2, variance=((1)²+0+(1)²)/3=2/3 → stdev≈0.8165.
    const s = summarizeBetaTrace([1, 2, 3]);
    expect(s.stdev).toBeCloseTo(Math.sqrt(2 / 3), 9);
});

test('summarizeBetaTrace skips null/NaN entries', () => {
    const s = summarizeBetaTrace([null, 1.0, NaN, 2.0, undefined, 3.0]);
    expect(s.count).toBe(3);
    expect(s.mean).toBeCloseTo(2, 12);
    expect(s.first).toBe(1);
    expect(s.latest).toBe(3);
});

test('summarizeBetaTrace single-value trace has zero stdev and zero drift', () => {
    const s = summarizeBetaTrace([1.5]);
    expect(s.mean).toBe(1.5);
    expect(s.stdev).toBe(0);
    expect(s.drift).toBe(0);
});

// ── fmtBeta ────────────────────────────────────────────────────────

test('fmtBeta defaults to 4 decimals', () => {
    expect(fmtBeta(1.23456789)).toBe('1.2346');
});

test('fmtBeta returns "—" on non-finite', () => {
    expect(fmtBeta(NaN)).toBe('—');
    expect(fmtBeta(null)).toBe('—');
});
