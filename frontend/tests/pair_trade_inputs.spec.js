// Pair Trade Z-Score pure helpers: validator, local spread+z series,
// crossing counter, signal label/class.

import { test, expect } from 'vitest';
import {
    validateInputs, buildBody,
    spreadAndZSeries, countCrossings,
    fmtSignal, signalCssClass,
} from '../js/_pair_trade_inputs.js';

const goodConfig = { entry_z: 2.0, exit_z: 0.5, stop_z: 3.5 };

// ── validateInputs ─────────────────────────────────────────────────

test('validate rejects too-short legs', () => {
    expect(validateInputs([1, 2], Array(10).fill(1), goodConfig)).toMatch(/y leg/);
    expect(validateInputs(Array(10).fill(1), [1, 2], goodConfig)).toMatch(/x leg/);
});

test('validate rejects mismatched lengths', () => {
    expect(validateInputs(Array(20).fill(1), Array(15).fill(1), goodConfig))
        .toMatch(/same length/);
});

test('validate rejects non-finite values', () => {
    const y = Array(15).fill(1); y[5] = NaN;
    const x = Array(15).fill(1);
    expect(validateInputs(y, x, goodConfig)).toMatch(/y prices.*non-finite/);
});

test('validate rejects exit_z >= entry_z', () => {
    const y = Array(15).fill(1); const x = Array(15).fill(1);
    expect(validateInputs(y, x, { entry_z: 2, exit_z: 2, stop_z: 3.5 })).toMatch(/exit_z must be </);
    expect(validateInputs(y, x, { entry_z: 1, exit_z: 1.5, stop_z: 3 })).toMatch(/exit_z must be </);
});

test('validate rejects stop_z <= entry_z', () => {
    const y = Array(15).fill(1); const x = Array(15).fill(1);
    expect(validateInputs(y, x, { entry_z: 2, exit_z: 0.5, stop_z: 2 })).toMatch(/stop_z must be >/);
});

test('validate rejects non-positive thresholds', () => {
    const y = Array(15).fill(1); const x = Array(15).fill(1);
    expect(validateInputs(y, x, { entry_z: 0, exit_z: 0.5, stop_z: 3.5 })).toMatch(/entry_z/);
    expect(validateInputs(y, x, { entry_z: 2, exit_z: -0.1, stop_z: 3.5 })).toMatch(/exit_z must be > 0/);
});

test('validate accepts good input', () => {
    const y = Array(15).fill(1); const x = Array(15).fill(1);
    expect(validateInputs(y, x, goodConfig)).toBe(null);
});

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits backend shape', () => {
    const b = buildBody([100], [50], goodConfig);
    expect(b).toEqual({ y: [100], x: [50], config: goodConfig });
});

// ── spreadAndZSeries ───────────────────────────────────────────────

test('spreadAndZSeries computes spread = y - β·x', () => {
    const r = spreadAndZSeries([10, 20, 30], [5, 10, 15], 2);
    expect(r.spreads).toEqual([0, 0, 0]);
});

test('spreadAndZSeries zeroes the z when spread is constant', () => {
    const r = spreadAndZSeries([10, 20, 30], [5, 10, 15], 2);
    expect(r.zs).toEqual([0, 0, 0]);
});

test('spreadAndZSeries z-scores variable spreads correctly', () => {
    // spreads: y - 2·x = [1, -1, 1, -1] → mean=0, stdev=1 → z=spread.
    const r = spreadAndZSeries([21, 19, 21, 19], [10, 10, 10, 10], 2);
    expect(r.spreads).toEqual([1, -1, 1, -1]);
    expect(r.spread_mean).toBeCloseTo(0, 12);
    expect(r.spread_stdev).toBeCloseTo(1, 12);
    expect(r.zs).toEqual([1, -1, 1, -1]);
});

test('spreadAndZSeries returns empty on shape mismatch / bad β', () => {
    expect(spreadAndZSeries([1, 2], [1], 2).spreads).toEqual([]);
    expect(spreadAndZSeries([1, 2], [1, 2], NaN).spreads).toEqual([]);
});

// ── countCrossings ─────────────────────────────────────────────────

test('countCrossings counts excursions (one per crossing, not per bar)', () => {
    const zs = [0, 1, 2.5, 1.8, 0.2, -2.3, -2.5, 0];
    // Excursions: bars 2-3 (|z|>2 then >2), bars 5-6 (|z|>2 then >2). 2 crossings.
    expect(countCrossings(zs, 2)).toBe(2);
});

test('countCrossings = 0 when nothing exceeds threshold', () => {
    expect(countCrossings([0.1, -0.5, 0.3, -0.2], 2)).toBe(0);
});

test('countCrossings skips non-finite z values', () => {
    expect(countCrossings([0, 2.5, NaN, 2.5, 0, null, -2.5], 2)).toBe(2);
});

test('countCrossings returns 0 on bad inputs', () => {
    expect(countCrossings(null, 2)).toBe(0);
    expect(countCrossings([1, 2, 3], -1)).toBe(0);
});

// ── fmtSignal / signalCssClass ─────────────────────────────────────

test('fmtSignal labels each enum variant', () => {
    expect(fmtSignal('long_spread')).toMatch(/LONG SPREAD/);
    expect(fmtSignal('short_spread')).toMatch(/SHORT SPREAD/);
    expect(fmtSignal('exit_spread')).toMatch(/EXIT/);
    expect(fmtSignal('stop_out')).toMatch(/STOP OUT/);
    expect(fmtSignal('hold')).toMatch(/HOLD/);
});

test('fmtSignal upper-cases unknown enum variants', () => {
    expect(fmtSignal('whatever')).toBe('WHATEVER');
});

test('signalCssClass: enter signals → pos, stop → neg, else empty', () => {
    expect(signalCssClass('long_spread')).toBe('pos');
    expect(signalCssClass('short_spread')).toBe('pos');
    expect(signalCssClass('stop_out')).toBe('neg');
    expect(signalCssClass('hold')).toBe('');
});
