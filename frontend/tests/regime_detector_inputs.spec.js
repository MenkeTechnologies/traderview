// Regime Detector pure helpers: validator + Markov-derived statistics
// (annualization, stationary distribution, expected dwell).

import { test, expect } from 'vitest';
import {
    validateReturns, annualizeStdev, annualizeMean,
    stationaryDistribution, expectedDwell, highVolBarFraction,
} from '../js/_regime_detector_inputs.js';

// ── validateReturns ─────────────────────────────────────────────────

test('validateReturns rejects fewer than 30 observations', () => {
    expect(validateReturns(Array(20).fill(0.01))).toMatch(/at least 30/);
});

test('validateReturns rejects non-finite values', () => {
    const r = Array(40).fill(0.01); r[5] = NaN;
    expect(validateReturns(r)).toMatch(/non-finite/);
});

test('validateReturns rejects flat series', () => {
    expect(validateReturns(Array(40).fill(0.01))).toMatch(/flat/);
});

test('validateReturns accepts varied 30+ series', () => {
    const r = Array.from({ length: 40 }, (_, i) => (i - 20) / 1000);
    expect(validateReturns(r)).toBe(null);
});

// ── annualizeStdev / annualizeMean ──────────────────────────────────

test('annualizeStdev applies sqrt-N scaling', () => {
    // 1% daily σ × √252 ≈ 15.87% annual
    expect(annualizeStdev(0.01, 252)).toBeCloseTo(0.01 * Math.sqrt(252), 12);
});

test('annualizeMean applies linear scaling', () => {
    // 0.05% daily × 252 = 12.6% annual
    expect(annualizeMean(0.0005, 252)).toBeCloseTo(0.126, 12);
});

test('annualize functions return NaN on bad inputs', () => {
    expect(Number.isNaN(annualizeStdev(NaN, 252))).toBe(true);
    expect(Number.isNaN(annualizeStdev(0.01, -1))).toBe(true);
    expect(Number.isNaN(annualizeMean(NaN, 252))).toBe(true);
    expect(Number.isNaN(annualizeMean(0.0005, 0))).toBe(true);
});

// ── stationaryDistribution ──────────────────────────────────────────

test('stationaryDistribution is 50/50 for symmetric chain', () => {
    const d = stationaryDistribution(0.95, 0.95);
    expect(d.p_state0).toBeCloseTo(0.5, 12);
    expect(d.p_state1).toBeCloseTo(0.5, 12);
});

test('stationary skews to sticky state when p_kk asymmetric', () => {
    // p00=0.99, p11=0.80 → state 0 stickier → π_0 should dominate.
    const d = stationaryDistribution(0.99, 0.80);
    expect(d.p_state0).toBeGreaterThan(d.p_state1);
    expect(d.p_state0 + d.p_state1).toBeCloseTo(1, 12);
});

test('stationary falls back to 50/50 on bad inputs', () => {
    expect(stationaryDistribution(NaN, 0.9)).toEqual({ p_state0: 0.5, p_state1: 0.5 });
    expect(stationaryDistribution(1, 1)).toEqual({ p_state0: 0.5, p_state1: 0.5 });
});

test('stationary clamps to [0,1] even with edge p_kk', () => {
    const d = stationaryDistribution(0.0, 1.0);
    expect(d.p_state0).toBeGreaterThanOrEqual(0);
    expect(d.p_state0).toBeLessThanOrEqual(1);
});

// ── expectedDwell ───────────────────────────────────────────────────

test('expectedDwell follows 1/(1-p)', () => {
    expect(expectedDwell(0.99)).toBeCloseTo(100, 9);
    expect(expectedDwell(0.95)).toBeCloseTo(20, 9);
});

test('expectedDwell returns Infinity for absorbing state', () => {
    expect(expectedDwell(1)).toBe(Infinity);
});

test('expectedDwell returns 1 for instantaneous-exit state', () => {
    expect(expectedDwell(0)).toBe(1);
});

test('expectedDwell returns NaN for out-of-range inputs', () => {
    expect(Number.isNaN(expectedDwell(-0.1))).toBe(true);
    expect(Number.isNaN(expectedDwell(1.1))).toBe(true);
    expect(Number.isNaN(expectedDwell(NaN))).toBe(true);
});

// ── highVolBarFraction ──────────────────────────────────────────────

test('highVolBarFraction counts bars above threshold', () => {
    expect(highVolBarFraction([0.1, 0.6, 0.4, 0.9], 0.5)).toBe(0.5);
});

test('highVolBarFraction default threshold is 0.5', () => {
    expect(highVolBarFraction([0.49, 0.51, 0.50])).toBeCloseTo(1 / 3, 12);
});

test('highVolBarFraction returns 0 for empty / non-array', () => {
    expect(highVolBarFraction([])).toBe(0);
    expect(highVolBarFraction(null)).toBe(0);
});

test('highVolBarFraction skips non-finite probs', () => {
    expect(highVolBarFraction([0.1, NaN, 0.9, Infinity], 0.5)).toBe(0.25);
});
