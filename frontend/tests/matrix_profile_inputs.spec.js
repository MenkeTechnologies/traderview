// Pattern Discovery (Matrix Profile) pure helpers: series parser,
// input validator, overlay-series generator, tuple unpackers, index
// axis.

import { test, expect } from 'vitest';
import {
    parseSeries, validateMatrixProfileInputs,
    overlaySeriesForWindows, unpackMotifPair, unpackDiscords,
    indexAxis,
} from '../js/_matrix_profile_inputs.js';

// ── parseSeries ──────────────────────────────────────────────────────

test('parseSeries handles one-per-line numbers', () => {
    expect(parseSeries('1\n2\n3').value).toEqual([1, 2, 3]);
});

test('parseSeries handles mixed delimiters', () => {
    expect(parseSeries('1, 2 3\n4').value).toEqual([1, 2, 3, 4]);
});

test('parseSeries skips blank + # lines', () => {
    expect(parseSeries('# x\n\n1\n2').value).toEqual([1, 2]);
});

test('parseSeries reports non-numeric tokens with line numbers', () => {
    const r = parseSeries('1\nbad\n2');
    expect(r.value).toEqual([1, 2]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].line_no).toBe(2);
});

test('parseSeries empty input is empty', () => {
    expect(parseSeries('')).toEqual({ value: [], errors: [] });
});

// ── validateMatrixProfileInputs ──────────────────────────────────────

test('validate rejects empty series', () => {
    expect(validateMatrixProfileInputs([], 5)).toMatch(/empty/);
});

test('validate rejects non-finite values', () => {
    const s = Array(20).fill(0); s[0] = NaN;
    expect(validateMatrixProfileInputs(s, 5)).toMatch(/non-finite/);
});

test('validate rejects non-integer / too-small m', () => {
    expect(validateMatrixProfileInputs(Array(50).fill(1), 3)).toMatch(/≥ 4/);
    expect(validateMatrixProfileInputs(Array(50).fill(1), 4.5)).toMatch(/≥ 4/);
});

test('validate rejects series shorter than 2·m', () => {
    expect(validateMatrixProfileInputs(Array(10).fill(1), 6)).toMatch(/too short/);
});

test('validate passes on good inputs', () => {
    expect(validateMatrixProfileInputs(Array(50).fill(0).map((_, i) => i), 10)).toBe(null);
});

// ── overlaySeriesForWindows ──────────────────────────────────────────

test('overlay places original values inside each window, null elsewhere', () => {
    const s = [10, 20, 30, 40, 50, 60];
    const o = overlaySeriesForWindows(s, [{ start: 2 }], 3);
    expect(o).toEqual([null, null, 30, 40, 50, null]);
});

test('overlay supports multiple windows', () => {
    const s = [10, 20, 30, 40, 50, 60];
    const o = overlaySeriesForWindows(s, [{ start: 0 }, { start: 4 }], 2);
    expect(o).toEqual([10, 20, null, null, 50, 60]);
});

test('overlay clips windows that run past the series end', () => {
    const s = [1, 2, 3];
    const o = overlaySeriesForWindows(s, [{ start: 2 }], 5);
    expect(o).toEqual([null, null, 3]);
});

test('overlay returns all nulls for an empty window list', () => {
    const s = [1, 2, 3];
    expect(overlaySeriesForWindows(s, [], 2)).toEqual([null, null, null]);
});

test('overlay silently drops invalid window descriptors', () => {
    const s = [10, 20, 30];
    const o = overlaySeriesForWindows(s, [{ start: -1 }, { start: 1.5 }, { start: 0 }], 2);
    expect(o).toEqual([10, 20, null]);
});

// ── unpackMotifPair / unpackDiscords ─────────────────────────────────

test('unpackMotifPair returns null for null/invalid input', () => {
    expect(unpackMotifPair(null)).toBe(null);
    expect(unpackMotifPair([])).toBe(null);
    expect(unpackMotifPair([1, 2])).toBe(null);                   // too short
    expect(unpackMotifPair([1.5, 2, 0.1])).toBe(null);            // non-integer i
});

test('unpackMotifPair unpacks valid tuple', () => {
    expect(unpackMotifPair([3, 47, 0.012])).toEqual({ i: 3, j: 47, distance: 0.012 });
});

test('unpackDiscords skips malformed entries', () => {
    const raw = [[5, 1.2], [10], [15, NaN], [20, 0.8], 'garbage'];
    expect(unpackDiscords(raw)).toEqual([
        { start: 5, distance: 1.2 },
        { start: 20, distance: 0.8 },
    ]);
});

test('unpackDiscords returns empty for non-array input', () => {
    expect(unpackDiscords(null)).toEqual([]);
    expect(unpackDiscords('nope')).toEqual([]);
});

// ── indexAxis ────────────────────────────────────────────────────────

test('indexAxis(n) returns [0..n-1]', () => {
    expect(indexAxis(4)).toEqual([0, 1, 2, 3]);
    expect(indexAxis(0)).toEqual([]);
});
