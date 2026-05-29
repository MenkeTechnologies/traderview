// Greeks Profile pure helpers: validator, payload shape, metric
// splitter, defaults helper, formatters.

import { test, expect } from 'vitest';
import {
    buildBody, validateParams, splitMetricSeries, METRICS,
    fmtN, defaultSpotGrid,
} from '../js/_greeks_profile_inputs.js';

const baseParams = {
    kind: 'call', strike: 100, time_to_expiry: 0.25,
    risk_free: 0.05, dividend_yield: 0.0, sigma: 0.25,
    spot_grid_low: 50, spot_grid_high: 150, n_points: 41,
};

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody passes all 9 fields through', () => {
    const b = buildBody(baseParams);
    expect(b).toEqual(baseParams);
});

// ── validateParams ─────────────────────────────────────────────────

test('validate accepts good defaults', () => {
    expect(validateParams(baseParams)).toBe(null);
});

test('validate rejects bad kind / non-positive strike / T', () => {
    expect(validateParams({ ...baseParams, kind: 'straddle' })).toMatch(/kind/);
    expect(validateParams({ ...baseParams, strike: 0 })).toMatch(/strike/);
    expect(validateParams({ ...baseParams, time_to_expiry: -1 })).toMatch(/time_to_expiry/);
});

test('validate accepts negative risk_free', () => {
    expect(validateParams({ ...baseParams, risk_free: -0.005 })).toBe(null);
});

test('validate rejects negative dividend / non-positive σ', () => {
    expect(validateParams({ ...baseParams, dividend_yield: -0.01 })).toMatch(/dividend_yield/);
    expect(validateParams({ ...baseParams, sigma: 0 })).toMatch(/sigma/);
});

test('validate enforces sane spot grid (high > low)', () => {
    expect(validateParams({ ...baseParams, spot_grid_low: 100, spot_grid_high: 100 })).toMatch(/spot_grid_high/);
    expect(validateParams({ ...baseParams, spot_grid_low: 150, spot_grid_high: 100 })).toMatch(/spot_grid_high/);
});

test('validate bounds n_points to [5, 501]', () => {
    expect(validateParams({ ...baseParams, n_points: 4 })).toMatch(/n_points/);
    expect(validateParams({ ...baseParams, n_points: 502 })).toMatch(/n_points/);
    expect(validateParams({ ...baseParams, n_points: 41.5 })).toMatch(/n_points/);
});

// ── splitMetricSeries ─────────────────────────────────────────────

test('splitMetricSeries returns parallel arrays for each metric', () => {
    const points = [
        { spot: 90, price: 1.5, delta: 0.3, gamma: 0.02, vega: 12, theta: -5, rho: 4 },
        { spot: 100, price: 4.5, delta: 0.5, gamma: 0.05, vega: 18, theta: -8, rho: 6 },
    ];
    const s = splitMetricSeries(points);
    expect(s.spots).toEqual([90, 100]);
    for (const m of METRICS) expect(s[m].length).toBe(2);
    expect(s.delta).toEqual([0.3, 0.5]);
    expect(s.theta).toEqual([-5, -8]);
});

test('splitMetricSeries maps non-finite metric values to null', () => {
    const s = splitMetricSeries([
        { spot: 100, price: NaN, delta: 0.5, gamma: 0.05, vega: 18, theta: -8, rho: 6 },
    ]);
    expect(s.price).toEqual([null]);
    expect(s.delta).toEqual([0.5]);
});

test('splitMetricSeries returns empty arrays for non-array input', () => {
    const s = splitMetricSeries(null);
    expect(s.spots).toEqual([]);
    for (const m of METRICS) expect(s[m]).toEqual([]);
});

test('splitMetricSeries skips malformed point entries', () => {
    const s = splitMetricSeries([
        null,
        'garbage',
        { spot: 100, price: 1, delta: 0, gamma: 0, vega: 0, theta: 0, rho: 0 },
    ]);
    expect(s.spots).toEqual([100]);
});

// ── defaultSpotGrid ───────────────────────────────────────────────

test('defaultSpotGrid spans ±50% from strike', () => {
    expect(defaultSpotGrid(100)).toEqual({ low: 50, high: 150 });
    expect(defaultSpotGrid(50)).toEqual({ low: 25, high: 75 });
});

test('defaultSpotGrid returns a safe default on bad strike', () => {
    expect(defaultSpotGrid(0)).toEqual({ low: 50, high: 150 });
    expect(defaultSpotGrid(NaN)).toEqual({ low: 50, high: 150 });
});

// ── fmtN ──────────────────────────────────────────────────────────

test('fmtN emits 4 decimals by default', () => {
    expect(fmtN(1.23456)).toBe('1.2346');
});

test('fmtN supports custom digits', () => {
    expect(fmtN(0.000001, 7)).toBe('0.0000010');
});

test('fmtN returns "—" on non-finite', () => {
    expect(fmtN(NaN)).toBe('—');
    expect(fmtN(null)).toBe('—');
});
