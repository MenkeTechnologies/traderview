// Almgren-Chriss pure helpers: validator, body shape, time axis,
// frontier sweep, nearest-λ marker.

import { test, expect } from 'vitest';
import {
    validateParams, buildBody, timeAxis,
    lambdaSweep, nearestLambdaIndex,
    fmtBig, fmtSeconds, fmtPct,
} from '../js/_almgren_chriss_inputs.js';

const baseParams = {
    total_shares: 1_000_000, horizon_seconds: 23_400, n_intervals: 30,
    eta: 2.5e-6, gamma: 2.5e-7, lambda: 1e-6, sigma: 0.95 / Math.sqrt(86_400),
};

// ── validateParams ────────────────────────────────────────────────

test('validate accepts canonical AC demo parameters', () => {
    expect(validateParams(baseParams)).toBe(null);
});

test('validate rejects total_shares = 0', () => {
    expect(validateParams({ ...baseParams, total_shares: 0 })).toMatch(/total_shares/);
});

test('validate accepts negative total_shares (sell trajectory)', () => {
    expect(validateParams({ ...baseParams, total_shares: -500_000 })).toBe(null);
});

test('validate rejects non-positive horizon', () => {
    expect(validateParams({ ...baseParams, horizon_seconds: 0 })).toMatch(/horizon/);
    expect(validateParams({ ...baseParams, horizon_seconds: -1 })).toMatch(/horizon/);
});

test('validate bounds n_intervals to [1, 2000] integer', () => {
    expect(validateParams({ ...baseParams, n_intervals: 0 })).toMatch(/n_intervals/);
    expect(validateParams({ ...baseParams, n_intervals: 2001 })).toMatch(/n_intervals/);
    expect(validateParams({ ...baseParams, n_intervals: 30.5 })).toMatch(/n_intervals/);
});

test('validate enforces non-negativity for γ/λ/σ and positivity for η', () => {
    expect(validateParams({ ...baseParams, eta: 0 })).toMatch(/eta/);
    expect(validateParams({ ...baseParams, gamma: -1 })).toMatch(/gamma/);
    expect(validateParams({ ...baseParams, lambda: -1 })).toMatch(/lambda/);
    expect(validateParams({ ...baseParams, sigma: -0.1 })).toMatch(/sigma/);
});

test('validate accepts γ=0 and λ=0 (TWAP limit)', () => {
    expect(validateParams({ ...baseParams, gamma: 0, lambda: 0 })).toBe(null);
});

// ── buildBody ────────────────────────────────────────────────────

test('buildBody wraps params under "params" key matching backend AlmgrenChrissBody', () => {
    const b = buildBody(baseParams);
    expect(b).toEqual({ params: baseParams });
});

// ── timeAxis ─────────────────────────────────────────────────────

test('timeAxis inventory grid has n+1 endpoints', () => {
    const xs = timeAxis(60, 6, 'inventory');
    expect(xs).toEqual([0, 10, 20, 30, 40, 50, 60]);
});

test('timeAxis schedule grid places points at slice midpoints', () => {
    const xs = timeAxis(60, 6, 'schedule');
    expect(xs).toEqual([5, 15, 25, 35, 45, 55]);
});

test('timeAxis returns empty on bad input', () => {
    expect(timeAxis(0, 6)).toEqual([]);
    expect(timeAxis(60, 0)).toEqual([]);
});

// ── lambdaSweep ──────────────────────────────────────────────────

test('lambdaSweep returns geometric ladder centred on base', () => {
    const lambdas = lambdaSweep(1e-6, 7);
    expect(lambdas.length).toBe(7);
    expect(lambdas[3]).toBeCloseTo(1e-6, 15);
    // Symmetric in log space.
    expect(lambdas[0]).toBeCloseTo(1e-9, 15);
    expect(lambdas[6]).toBeCloseTo(1e-3, 15);
});

test('lambdaSweep clamps points to [3, 21] and odd-count safe', () => {
    expect(lambdaSweep(1e-6, 1).length).toBe(3);
    expect(lambdaSweep(1e-6, 99).length).toBe(21);
});

test('lambdaSweep handles non-positive base by defaulting to 1e-6', () => {
    const a = lambdaSweep(0, 5);
    expect(a[2]).toBeCloseTo(1e-6, 15);
});

// ── nearestLambdaIndex ──────────────────────────────────────────

test('nearestLambdaIndex picks closest in log space', () => {
    const lambdas = [1e-9, 1e-7, 1e-5, 1e-3];
    expect(nearestLambdaIndex(lambdas, 5e-6)).toBe(2);
    expect(nearestLambdaIndex(lambdas, 1e-9)).toBe(0);
    expect(nearestLambdaIndex(lambdas, 1e-3)).toBe(3);
});

test('nearestLambdaIndex returns -1 on empty / non-finite target', () => {
    expect(nearestLambdaIndex([], 1e-6)).toBe(-1);
    expect(nearestLambdaIndex([1e-6], NaN)).toBe(-1);
});

// ── formatters ───────────────────────────────────────────────────

test('fmtBig: B/M/k suffixes', () => {
    expect(fmtBig(2.5e9)).toBe('2.500B');
    expect(fmtBig(4.2e6)).toBe('4.200M');
    expect(fmtBig(15_000)).toBe('15.000k');
    expect(fmtBig(42.5)).toBe('42.500');
    expect(fmtBig(NaN)).toBe('—');
});

test('fmtSeconds: seconds / minutes / hours / days', () => {
    expect(fmtSeconds(45)).toBe('45.0s');
    expect(fmtSeconds(90)).toBe('1.50m');
    expect(fmtSeconds(3600 * 2)).toBe('2.00h');
    expect(fmtSeconds(86400 * 3)).toBe('3.00d');
    expect(fmtSeconds(Infinity)).toBe('—');
    expect(fmtSeconds(-1)).toBe('—');
});

test('fmtPct emits 2-decimal percentage', () => {
    expect(fmtPct(0.1234)).toBe('12.34%');
    expect(fmtPct(NaN)).toBe('—');
});
