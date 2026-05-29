// Vasicek pure helpers: payload shape, validator, closed-form
// long-run statistics + half-life, normal-density curve, formatters.

import { test, expect } from 'vitest';
import {
    buildBody, validateParams,
    halfLifeYears, longRunStdev, horizonYears,
    normalDensityCurve, fmtRatePct, fmtYears,
} from '../js/_vasicek_inputs.js';

const baseParams = {
    r0: 0.05, a: 0.5, b: 0.03, sigma: 0.01,
    dt: 1 / 52, steps: 520, paths: 5000, seed: 42,
};

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody passes every field through', () => {
    const b = buildBody(baseParams);
    expect(b).toEqual(baseParams);
});

// ── validateParams ─────────────────────────────────────────────────

test('validate accepts good defaults', () => {
    expect(validateParams(baseParams)).toBe(null);
});

test('validate rejects non-finite r0', () => {
    expect(validateParams({ ...baseParams, r0: NaN })).toMatch(/r0/);
});

test('validate rejects non-positive a', () => {
    expect(validateParams({ ...baseParams, a: 0 })).toMatch(/mean-reversion/);
    expect(validateParams({ ...baseParams, a: -1 })).toMatch(/mean-reversion/);
});

test('validate accepts negative b (negative rates are legal)', () => {
    expect(validateParams({ ...baseParams, b: -0.005 })).toBe(null);
});

test('validate rejects negative sigma', () => {
    expect(validateParams({ ...baseParams, sigma: -0.01 })).toMatch(/σ/);
});

test('validate rejects non-positive dt', () => {
    expect(validateParams({ ...baseParams, dt: 0 })).toMatch(/dt/);
    expect(validateParams({ ...baseParams, dt: -0.01 })).toMatch(/dt/);
});

test('validate enforces integer steps/paths/seed', () => {
    expect(validateParams({ ...baseParams, steps: 0 })).toMatch(/steps/);
    expect(validateParams({ ...baseParams, steps: 1.5 })).toMatch(/steps/);
    expect(validateParams({ ...baseParams, paths: 5 })).toMatch(/paths/);
    expect(validateParams({ ...baseParams, seed: -1 })).toMatch(/seed/);
});

// ── halfLifeYears ──────────────────────────────────────────────────

test('halfLifeYears = ln(2) / a', () => {
    expect(halfLifeYears(0.5)).toBeCloseTo(Math.LN2 / 0.5, 12);
    expect(halfLifeYears(1.0)).toBeCloseTo(Math.LN2, 12);
});

test('halfLifeYears returns null on bad a', () => {
    expect(halfLifeYears(0)).toBe(null);
    expect(halfLifeYears(-1)).toBe(null);
    expect(halfLifeYears(NaN)).toBe(null);
});

// ── longRunStdev ───────────────────────────────────────────────────

test('longRunStdev = σ / √(2a)', () => {
    expect(longRunStdev(0.5, 0.01)).toBeCloseTo(0.01 / Math.sqrt(1), 12);
    expect(longRunStdev(2, 0.04)).toBeCloseTo(0.04 / 2, 12);
});

test('longRunStdev returns null on bad inputs', () => {
    expect(longRunStdev(0, 0.01)).toBe(null);
    expect(longRunStdev(-1, 0.01)).toBe(null);
    expect(longRunStdev(0.5, -0.01)).toBe(null);
    expect(longRunStdev(0.5, NaN)).toBe(null);
});

// ── horizonYears ──────────────────────────────────────────────────

test('horizonYears = steps · dt', () => {
    expect(horizonYears(520, 1 / 52)).toBeCloseTo(10, 9);
    expect(horizonYears(252, 1 / 252)).toBeCloseTo(1, 9);
});

test('horizonYears returns null on bad inputs', () => {
    expect(horizonYears(0, 0.01)).toBe(null);
    expect(horizonYears(10, 0)).toBe(null);
    expect(horizonYears(2.5, 0.01)).toBe(null);
});

// ── normalDensityCurve ────────────────────────────────────────────

test('normalDensityCurve emits the requested points', () => {
    const { xs, ys } = normalDensityCurve(0, 1, 11);
    expect(xs.length).toBe(11);
    expect(ys.length).toBe(11);
});

test('normalDensityCurve spans approximately ±4σ around mean', () => {
    const { xs } = normalDensityCurve(0.03, 0.01, 9);
    expect(xs[0]).toBeCloseTo(0.03 - 4 * 0.01, 6);
    expect(xs[xs.length - 1]).toBeCloseTo(0.03 + 4 * 0.01, 6);
});

test('normalDensityCurve peaks at the mean', () => {
    const { xs, ys } = normalDensityCurve(0.03, 0.01, 101);
    const peakIdx = ys.indexOf(Math.max(...ys));
    expect(Math.abs(xs[peakIdx] - 0.03)).toBeLessThan(0.0005);
});

test('normalDensityCurve returns empty arrays for non-positive stdev', () => {
    expect(normalDensityCurve(0.03, 0).xs).toEqual([]);
    expect(normalDensityCurve(0.03, -1).xs).toEqual([]);
    expect(normalDensityCurve(NaN, 0.01).xs).toEqual([]);
});

// ── formatters ─────────────────────────────────────────────────────

test('fmtRatePct emits 4-decimal percent by default', () => {
    expect(fmtRatePct(0.0525)).toBe('5.2500%');
});

test('fmtRatePct returns "—" on non-finite', () => {
    expect(fmtRatePct(NaN)).toBe('—');
});

test('fmtYears formats >= 1y as years', () => {
    expect(fmtYears(1.5)).toBe('1.50 years');
});

test('fmtYears formats < 1y as days', () => {
    expect(fmtYears(0.5)).toBe('182.5 days');
});

test('fmtYears returns "—" on non-finite', () => {
    expect(fmtYears(NaN)).toBe('—');
});
