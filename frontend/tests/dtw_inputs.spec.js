// DTW pure helpers: body shape, validator, normalized distance,
// max-stretch reducer, path → series unpacker.

import { test, expect } from 'vitest';
import {
    buildBody, validateInputs,
    normalizedDistance, maxStretch, pathToSeries,
} from '../js/_dtw_inputs.js';

// ── buildBody ──────────────────────────────────────────────────────

test('buildBody emits backend shape', () => {
    const b = buildBody([1, 2, 3], [4, 5, 6], 5);
    expect(b).toEqual({ a: [1, 2, 3], b: [4, 5, 6], band_radius: 5 });
});

test('buildBody defaults bad band_radius to 0', () => {
    expect(buildBody([1], [2], -1).band_radius).toBe(0);
    expect(buildBody([1], [2], NaN).band_radius).toBe(0);
    expect(buildBody([1], [2], 1.5).band_radius).toBe(0);
});

// ── validateInputs ─────────────────────────────────────────────────

test('validate rejects too-short series', () => {
    expect(validateInputs([1], [1, 2], 0)).toMatch(/series A/);
    expect(validateInputs([1, 2], [1], 0)).toMatch(/series B/);
});

test('validate rejects non-finite values', () => {
    expect(validateInputs([NaN, 1], [1, 2], 0)).toMatch(/series A.*non-finite/);
    expect(validateInputs([1, 2], [1, Infinity], 0)).toMatch(/series B.*non-finite/);
});

test('validate rejects bad band_radius', () => {
    expect(validateInputs([1, 2], [1, 2], -1)).toMatch(/band_radius/);
    expect(validateInputs([1, 2], [1, 2], 1.5)).toMatch(/band_radius/);
});

test('validate accepts good input', () => {
    expect(validateInputs([1, 2, 3], [4, 5, 6], 0)).toBe(null);
    expect(validateInputs([1, 2, 3], [4, 5, 6], 2)).toBe(null);
});

// ── normalizedDistance ─────────────────────────────────────────────

test('normalizedDistance divides distance by path length', () => {
    expect(normalizedDistance(15, 3)).toBeCloseTo(5, 12);
});

test('normalizedDistance returns null on bad inputs', () => {
    expect(normalizedDistance(NaN, 3)).toBe(null);
    expect(normalizedDistance(15, 0)).toBe(null);
    expect(normalizedDistance(15, -1)).toBe(null);
    expect(normalizedDistance(15, 2.5)).toBe(null);
});

// ── maxStretch ─────────────────────────────────────────────────────

test('maxStretch is 0 for the diagonal path', () => {
    const path = [[0, 0], [1, 1], [2, 2], [3, 3]];
    expect(maxStretch(path)).toBe(0);
});

test('maxStretch finds the largest |i - j| departure', () => {
    const path = [[0, 0], [1, 1], [2, 4], [3, 5]];
    expect(maxStretch(path)).toBe(2);
});

test('maxStretch returns 0 on empty / non-array', () => {
    expect(maxStretch([])).toBe(0);
    expect(maxStretch(null)).toBe(0);
});

test('maxStretch tolerates malformed pairs (skips them)', () => {
    const path = [[0, 0], [1], 'bad', [2, 5]];
    expect(maxStretch(path)).toBe(3);
});

// ── pathToSeries ───────────────────────────────────────────────────

test('pathToSeries unpacks parallel arrays', () => {
    const { xs, ys } = pathToSeries([[0, 0], [1, 2], [2, 3]]);
    expect(xs).toEqual([0, 1, 2]);
    expect(ys).toEqual([0, 2, 3]);
});

test('pathToSeries skips malformed pairs', () => {
    const { xs, ys } = pathToSeries([[0, 0], [1], null, [2, 3]]);
    expect(xs).toEqual([0, 2]);
    expect(ys).toEqual([0, 3]);
});

test('pathToSeries returns empty for non-array', () => {
    expect(pathToSeries(null)).toEqual({ xs: [], ys: [] });
    expect(pathToSeries('garbage')).toEqual({ xs: [], ys: [] });
});
