// Portfolio Allocator pure helpers: matrix parser, float-list parser,
// label parser, defaulters, covariance validator. View is DOM-bound and
// not unit-testable.

import { test, expect } from 'vitest';
import {
    parseMatrix, parseFloatList, parseLabelList,
    normalizeLabels, defaultLabels, defaultExcessReturns,
    validateCovariance, formatMatrix,
} from '../js/_portfolio_allocator_inputs.js';

// ── parseMatrix ──────────────────────────────────────────────────────

test('parseMatrix handles space-separated rows', () => {
    const r = parseMatrix('0.04 0.01\n0.01 0.09');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([[0.04, 0.01], [0.01, 0.09]]);
});

test('parseMatrix handles comma-separated rows', () => {
    const r = parseMatrix('0.04,0.01\n0.01,0.09');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([[0.04, 0.01], [0.01, 0.09]]);
});

test('parseMatrix ignores blank lines and # comments', () => {
    const r = parseMatrix('# header\n\n1 2\n# inline\n3 4');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([[1, 2], [3, 4]]);
});

test('parseMatrix flags non-square rows', () => {
    const r = parseMatrix('1 2\n3 4 5');
    expect(r.value).toEqual([[1, 2]]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/expected 2 columns/);
});

test('parseMatrix flags non-numeric tokens', () => {
    const r = parseMatrix('1 foo\n3 4');
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].line_no).toBe(1);
});

test('parseMatrix empty input returns empty value + no errors', () => {
    const r = parseMatrix('');
    expect(r.value).toEqual([]);
    expect(r.errors).toEqual([]);
});

// ── parseFloatList ──────────────────────────────────────────────────

test('parseFloatList accepts one-per-line and multi-per-line', () => {
    const r = parseFloatList('0.06\n0.08 0.03');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([0.06, 0.08, 0.03]);
});

test('parseFloatList reports bad token line', () => {
    const r = parseFloatList('0.06\nbad\n0.03');
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].line_no).toBe(2);
});

test('parseFloatList empty input returns empty value + no errors', () => {
    expect(parseFloatList('')).toEqual({ value: [], errors: [] });
    expect(parseFloatList('   ')).toEqual({ value: [], errors: [] });
});

// ── parseLabelList / defaultLabels / normalizeLabels ─────────────────

test('parseLabelList strips blank + # lines', () => {
    expect(parseLabelList('SPY\n# note\n\nQQQ')).toEqual(['SPY', 'QQQ']);
});

test('defaultLabels generates A1..An', () => {
    expect(defaultLabels(3)).toEqual(['A1', 'A2', 'A3']);
});

test('normalizeLabels right-pads short user-supplied lists', () => {
    expect(normalizeLabels(['SPY'], 3)).toEqual(['SPY', 'A2', 'A3']);
});

test('normalizeLabels trims long lists to N', () => {
    expect(normalizeLabels(['a', 'b', 'c', 'd'], 2)).toEqual(['a', 'b']);
});

test('normalizeLabels falls back to defaults when list is empty', () => {
    expect(normalizeLabels([], 3)).toEqual(['A1', 'A2', 'A3']);
});

// ── defaultExcessReturns ─────────────────────────────────────────────

test('defaultExcessReturns gives a flat 5% per asset', () => {
    expect(defaultExcessReturns(3)).toEqual([0.05, 0.05, 0.05]);
});

// ── validateCovariance ───────────────────────────────────────────────

test('validateCovariance accepts a clean symmetric PSD matrix', () => {
    expect(validateCovariance([[0.04, 0.01], [0.01, 0.09]])).toBe(null);
});

test('validateCovariance rejects too-small input', () => {
    expect(validateCovariance([])).toMatch(/at least 2/);
    expect(validateCovariance([[1]])).toMatch(/at least 2/);
});

test('validateCovariance rejects non-square rows', () => {
    expect(validateCovariance([[1, 2], [3]])).toMatch(/must have 2 columns/);
});

test('validateCovariance rejects zero/negative diagonal', () => {
    expect(validateCovariance([[0, 0], [0, 1]])).toMatch(/diagonal/);
    expect(validateCovariance([[-0.01, 0], [0, 1]])).toMatch(/diagonal/);
});

test('validateCovariance rejects asymmetry beyond float tolerance', () => {
    expect(validateCovariance([[1, 0.5], [0.4, 1]])).toMatch(/not symmetric/);
});

test('validateCovariance accepts symmetric within float tolerance', () => {
    expect(validateCovariance([[1, 0.5 + 1e-12], [0.5, 1]])).toBe(null);
});

// ── formatMatrix ─────────────────────────────────────────────────────

test('formatMatrix round-trips through parseMatrix', () => {
    const m = [[0.04, 0.01], [0.01, 0.09]];
    const s = formatMatrix(m, 2);
    const r = parseMatrix(s);
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([[0.04, 0.01], [0.01, 0.09]]);
});
