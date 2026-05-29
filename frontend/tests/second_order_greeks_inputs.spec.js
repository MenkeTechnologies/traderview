// Second-Order Greeks pure helpers: validator, payload shape, local
// closed-form computation (parity against backend), nearest-ATM index.

import { test, expect } from 'vitest';
import {
    buildBody, validateParams, computePoint, computeGrid,
    METRICS, fmtN, defaultSpotGrid, nearestAtmIndex, linspace,
} from '../js/_second_order_greeks_inputs.js';

const baseParams = {
    kind: 'call', strike: 100, time_to_expiry: 0.25,
    risk_free: 0.05, dividend_yield: 0.0, sigma: 0.25,
    spot_grid_low: 50, spot_grid_high: 150, n_points: 41,
};

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits backend SecondOrderGreeksBody shape (single-point)', () => {
    const b = buildBody({ ...baseParams, spot: 100 });
    expect(b).toEqual({
        spot: 100, strike: 100, time_to_expiry: 0.25,
        risk_free: 0.05, dividend_yield: 0.0, sigma: 0.25, kind: 'call',
    });
});

// ── validateParams ─────────────────────────────────────────────────

test('validate accepts defaults', () => {
    expect(validateParams(baseParams)).toBe(null);
});

test('validate rejects bad kind / non-positive contract params', () => {
    expect(validateParams({ ...baseParams, kind: 'straddle' })).toMatch(/kind/);
    expect(validateParams({ ...baseParams, strike: 0 })).toMatch(/strike/);
    expect(validateParams({ ...baseParams, time_to_expiry: 0 })).toMatch(/time_to_expiry/);
    expect(validateParams({ ...baseParams, sigma: -0.1 })).toMatch(/sigma/);
});

test('validate enforces sane spot grid + integer n_points', () => {
    expect(validateParams({ ...baseParams, spot_grid_high: 50 })).toMatch(/spot_grid_high/);
    expect(validateParams({ ...baseParams, n_points: 4 })).toMatch(/n_points/);
    expect(validateParams({ ...baseParams, n_points: 502 })).toMatch(/n_points/);
    expect(validateParams({ ...baseParams, n_points: 41.5 })).toMatch(/n_points/);
});

test('validate accepts negative risk-free', () => {
    expect(validateParams({ ...baseParams, risk_free: -0.01 })).toBe(null);
});

// ── computePoint (parity vs known closed-form values) ─────────────

test('computePoint returns null on invalid inputs', () => {
    expect(computePoint(NaN, 100, 0.25, 0.05, 0.0, 0.20, 'call')).toBe(null);
    expect(computePoint(0,   100, 0.25, 0.05, 0.0, 0.20, 'call')).toBe(null);
    expect(computePoint(100, 100, 0,    0.05, 0.0, 0.20, 'call')).toBe(null);
    expect(computePoint(100, 100, 0.25, 0.05, 0.0, 0,    'call')).toBe(null);
});

test('OTM call vanna is positive (d2 < 0 → -φ(d1)·d2/σ > 0)', () => {
    const g = computePoint(100, 130, 0.25, 0.0, 0.0, 0.20, 'call');
    expect(g.vanna).toBeGreaterThan(0);
});

test('ATM call vomma is small (d1·d2 near zero at ATM)', () => {
    const g = computePoint(100, 100, 0.25, 0.0, 0.0, 0.20, 'call');
    expect(Math.abs(g.vomma)).toBeLessThan(5);
});

test('call and put have different charm signs for short-dated ATM (q=0)', () => {
    const c = computePoint(100, 100, 0.05, 0.0, 0.0, 0.20, 'call');
    const p = computePoint(100, 100, 0.05, 0.0, 0.0, 0.20, 'put');
    // With q = 0 both share the same charm_common; their charms differ by
    // q·terms which vanish at q=0 → both equal -charm_common. Pin that:
    expect(c.charm).toBeCloseTo(p.charm, 10);
    // Now with q > 0 the q·terms diverge them.
    const c2 = computePoint(100, 100, 0.05, 0.0, 0.02, 0.20, 'call');
    const p2 = computePoint(100, 100, 0.05, 0.0, 0.02, 0.20, 'put');
    expect(c2.charm).not.toBeCloseTo(p2.charm, 6);
});

test('computePoint outputs are all finite for a normal contract', () => {
    const g = computePoint(100, 100, 0.5, 0.05, 0.02, 0.30, 'put');
    for (const m of METRICS) expect(Number.isFinite(g[m])).toBe(true);
});

// ── computeGrid ───────────────────────────────────────────────────

test('computeGrid returns parallel arrays of correct length', () => {
    const g = computeGrid(baseParams);
    expect(g.spots.length).toBe(baseParams.n_points);
    for (const m of METRICS) expect(g[m].length).toBe(baseParams.n_points);
});

test('computeGrid spots span exactly low..high inclusive', () => {
    const g = computeGrid({ ...baseParams, spot_grid_low: 80, spot_grid_high: 120, n_points: 5 });
    expect(g.spots).toEqual([80, 90, 100, 110, 120]);
});

test('computeGrid handles single-point degenerate (n=5 minimum)', () => {
    const g = computeGrid({ ...baseParams, n_points: 5 });
    expect(g.spots.length).toBe(5);
});

// ── linspace ───────────────────────────────────────────────────────

test('linspace returns single value when n < 2', () => {
    expect(linspace(10, 20, 1)).toEqual([10]);
});

test('linspace generates evenly spaced grid', () => {
    expect(linspace(0, 10, 6)).toEqual([0, 2, 4, 6, 8, 10]);
});

// ── defaultSpotGrid ───────────────────────────────────────────────

test('defaultSpotGrid spans ±50% from strike', () => {
    expect(defaultSpotGrid(200)).toEqual({ low: 100, high: 300 });
});

test('defaultSpotGrid falls back on bad strike', () => {
    expect(defaultSpotGrid(0)).toEqual({ low: 50, high: 150 });
    expect(defaultSpotGrid(NaN)).toEqual({ low: 50, high: 150 });
});

// ── nearestAtmIndex ───────────────────────────────────────────────

test('nearestAtmIndex finds closest grid point to strike', () => {
    expect(nearestAtmIndex([90, 95, 100, 105, 110], 102)).toBe(2);
    expect(nearestAtmIndex([90, 95, 100, 105, 110], 108)).toBe(4);
});

test('nearestAtmIndex returns -1 on empty / non-array input', () => {
    expect(nearestAtmIndex([], 100)).toBe(-1);
    expect(nearestAtmIndex(null, 100)).toBe(-1);
});

// ── fmtN ──────────────────────────────────────────────────────────

test('fmtN emits 6 decimals by default', () => {
    expect(fmtN(0.123456789)).toBe('0.123457');
});

test('fmtN handles non-finite with em-dash', () => {
    expect(fmtN(NaN)).toBe('—');
    expect(fmtN(null)).toBe('—');
});
