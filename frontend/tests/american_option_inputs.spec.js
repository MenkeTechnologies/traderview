// American Option Pricer pure helpers: payload shape, validator,
// local Black-Scholes European, early-exercise premium, normal CDF.

import { test, expect } from 'vitest';
import {
    buildLsmcBody, validateLsmcParams,
    blackScholesEuropean, normCdf,
    earlyExercisePremium, ciHalfWidth, fmtMoney,
} from '../js/_american_option_inputs.js';

const baseParams = {
    kind: 'put',
    spot: 100, strike: 100, t_years: 0.5,
    rate: 0.05, dividend: 0.0, sigma: 0.25,
    steps: 50, paths: 5000, seed: 42,
};

// ── validateLsmcParams ──────────────────────────────────────────────

test('validate accepts good defaults', () => {
    expect(validateLsmcParams(baseParams)).toBe(null);
});

test('validate rejects bad kind', () => {
    expect(validateLsmcParams({ ...baseParams, kind: 'straddle' })).toMatch(/kind/);
});

test('validate rejects non-positive spot/strike/t', () => {
    expect(validateLsmcParams({ ...baseParams, spot: 0 })).toMatch(/spot/);
    expect(validateLsmcParams({ ...baseParams, strike: -1 })).toMatch(/strike/);
    expect(validateLsmcParams({ ...baseParams, t_years: 0 })).toMatch(/t_years/);
});

test('validate rejects negative dividend / sigma', () => {
    expect(validateLsmcParams({ ...baseParams, dividend: -0.01 })).toMatch(/dividend/);
    expect(validateLsmcParams({ ...baseParams, sigma: -1 })).toMatch(/sigma/);
});

test('validate enforces integer steps/paths/seed', () => {
    expect(validateLsmcParams({ ...baseParams, steps: 2.5 })).toMatch(/steps/);
    expect(validateLsmcParams({ ...baseParams, paths: 9 })).toMatch(/paths/);
    expect(validateLsmcParams({ ...baseParams, seed: -1 })).toMatch(/seed/);
});

// ── buildLsmcBody ───────────────────────────────────────────────────

test('buildLsmcBody passes through every param', () => {
    const b = buildLsmcBody(baseParams);
    expect(b).toEqual({
        kind: 'put', spot: 100, strike: 100, t_years: 0.5,
        rate: 0.05, dividend: 0.0, sigma: 0.25,
        steps: 50, paths: 5000, seed: 42,
    });
});

// ── normCdf ─────────────────────────────────────────────────────────

test('normCdf(0) ≈ 0.5', () => {
    expect(normCdf(0)).toBeCloseTo(0.5, 6);
});

test('normCdf is symmetric: CDF(x) + CDF(-x) ≈ 1', () => {
    for (const x of [0.5, 1, 1.96, 3]) {
        expect(normCdf(x) + normCdf(-x)).toBeCloseTo(1, 6);
    }
});

test('normCdf matches standard z-table values', () => {
    expect(normCdf(1.96)).toBeCloseTo(0.975, 3);
    expect(normCdf(-2.33)).toBeCloseTo(0.0099, 3);
});

// ── blackScholesEuropean ────────────────────────────────────────────

test('BS call at-the-money matches known closed-form value', () => {
    // BS call: S=100, K=100, T=1, r=0.05, q=0, σ=0.20 → ≈ 10.4506
    const c = blackScholesEuropean('call', 100, 100, 1, 0.05, 0, 0.20);
    expect(c).toBeCloseTo(10.4506, 3);
});

test('BS put at-the-money matches known closed-form value', () => {
    // BS put: S=100, K=100, T=1, r=0.05, q=0, σ=0.20 → ≈ 5.5735
    const p = blackScholesEuropean('put', 100, 100, 1, 0.05, 0, 0.20);
    expect(p).toBeCloseTo(5.5735, 3);
});

test('BS put-call parity holds: C - P = S·e^(-qT) - K·e^(-rT)', () => {
    const S = 110, K = 100, T = 0.5, r = 0.04, q = 0.01, sigma = 0.30;
    const c = blackScholesEuropean('call', S, K, T, r, q, sigma);
    const p = blackScholesEuropean('put', S, K, T, r, q, sigma);
    const parity = S * Math.exp(-q * T) - K * Math.exp(-r * T);
    expect(c - p).toBeCloseTo(parity, 6);
});

test('BS at expiry (t=0) collapses to intrinsic', () => {
    expect(blackScholesEuropean('call', 110, 100, 0, 0.05, 0, 0.2)).toBe(10);
    expect(blackScholesEuropean('put', 90, 100, 0, 0.05, 0, 0.2)).toBe(10);
    expect(blackScholesEuropean('call', 90, 100, 0, 0.05, 0, 0.2)).toBe(0);
});

test('BS at zero vol collapses to intrinsic', () => {
    // σ=0 + t>0 should still return discounted intrinsic for ITM, but
    // the helper short-circuits to the spot-vs-strike difference.
    expect(blackScholesEuropean('call', 110, 100, 1, 0.05, 0, 0)).toBe(10);
    expect(blackScholesEuropean('put', 90, 100, 1, 0.05, 0, 0)).toBe(10);
});

test('BS deep OTM call is near zero', () => {
    expect(blackScholesEuropean('call', 50, 200, 0.1, 0.05, 0, 0.20)).toBeLessThan(0.01);
});

// ── earlyExercisePremium / ciHalfWidth / fmtMoney ───────────────────

test('earlyExercisePremium is American - European', () => {
    expect(earlyExercisePremium(10, 7.5)).toBe(2.5);
});

test('earlyExercisePremium returns null on bad inputs', () => {
    expect(earlyExercisePremium(NaN, 5)).toBe(null);
    expect(earlyExercisePremium(5, NaN)).toBe(null);
});

test('ciHalfWidth = 1.96 · SE', () => {
    expect(ciHalfWidth(0.10)).toBeCloseTo(0.196, 9);
});

test('ciHalfWidth returns NaN on bad SE', () => {
    expect(Number.isNaN(ciHalfWidth(NaN))).toBe(true);
});

test('fmtMoney emits requested decimals', () => {
    expect(fmtMoney(1.23456789, 2)).toBe('1.23');
    expect(fmtMoney(1.23456789, 4)).toBe('1.2346');
});

test('fmtMoney returns "—" on non-finite', () => {
    expect(fmtMoney(NaN)).toBe('—');
    expect(fmtMoney(Infinity)).toBe('—');
});
