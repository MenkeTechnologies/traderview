// Series Smoother pure helpers: parser, validator, index axis,
// payload shaper, Theil-Sen fitted-y, option defaults + validation.

import { test, expect } from 'vitest';
import {
    parseSeries, validateSeries, indexAxis,
    buildSmootherPayloads, theilSenFittedY,
    defaultOptions, validateOptions,
} from '../js/_series_smoother_inputs.js';

// ── parseSeries ──────────────────────────────────────────────────────

test('parseSeries accepts one-per-line', () => {
    const r = parseSeries('100\n101\n102');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([100, 101, 102]);
});

test('parseSeries accepts comma + space mixed', () => {
    const r = parseSeries('100, 101 102\n103,104');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([100, 101, 102, 103, 104]);
});

test('parseSeries skips blank + # lines', () => {
    const r = parseSeries('# label\n\n100\n101\n# end');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([100, 101]);
});

test('parseSeries reports bad tokens with line numbers', () => {
    const r = parseSeries('100\nbad\n101');
    expect(r.value).toEqual([100, 101]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].line_no).toBe(2);
});

test('parseSeries handles per-token bad mixes on one line', () => {
    const r = parseSeries('100 oops 102 nope');
    expect(r.value).toEqual([100, 102]);
    expect(r.errors.length).toBe(2);
});

// ── validateSeries ──────────────────────────────────────────────────

test('validateSeries rejects fewer than minLen values', () => {
    expect(validateSeries([1, 2, 3])).toMatch(/at least 10/);
});

test('validateSeries rejects non-finite values', () => {
    const s = Array(20).fill(1); s[0] = NaN;
    expect(validateSeries(s)).toMatch(/non-finite/);
});

test('validateSeries accepts well-formed series', () => {
    expect(validateSeries(Array(20).fill(0).map((_, i) => i))).toBe(null);
});

test('validateSeries respects custom minLen', () => {
    expect(validateSeries([1, 2], 2)).toBe(null);
    expect(validateSeries([1], 2)).toMatch(/at least 2/);
});

// ── indexAxis ────────────────────────────────────────────────────────

test('indexAxis emits 0..n-1', () => {
    expect(indexAxis(5)).toEqual([0, 1, 2, 3, 4]);
});

test('indexAxis returns empty for 0', () => {
    expect(indexAxis(0)).toEqual([]);
});

// ── buildSmootherPayloads ────────────────────────────────────────────

test('lowess payload has x, y, frac, robustness_iter', () => {
    const p = buildSmootherPayloads([100, 101, 102], defaultOptions());
    expect(p.lowess.x).toEqual([0, 1, 2]);
    expect(p.lowess.y).toEqual([100, 101, 102]);
    expect(p.lowess.frac).toBe(0.3);
    expect(p.lowess.robustness_iter).toBe(0);
});

test('kalman_rts payload sets x0 to first observation', () => {
    const p = buildSmootherPayloads([42, 43, 44], defaultOptions());
    expect(p.kalman_rts.observations).toEqual([42, 43, 44]);
    expect(p.kalman_rts.x0).toBe(42);
    expect(p.kalman_rts.p0).toBe(1.0);
});

test('theil_sen payload has x, y only', () => {
    const p = buildSmootherPayloads([10, 20, 30], defaultOptions());
    expect(p.theil_sen).toEqual({ x: [0, 1, 2], y: [10, 20, 30] });
});

test('polynomial payload carries degree', () => {
    const opts = defaultOptions(); opts.poly_degree = 5;
    const p = buildSmootherPayloads([1, 2, 3], opts);
    expect(p.polynomial.degree).toBe(5);
});

// ── theilSenFittedY ──────────────────────────────────────────────────

test('theilSenFittedY evaluates slope*x + intercept', () => {
    const y = theilSenFittedY([0, 1, 2, 3], 2, 10);
    expect(y).toEqual([10, 12, 14, 16]);
});

test('theilSenFittedY with zero slope yields constant intercept', () => {
    const y = theilSenFittedY([0, 5, 10], 0, 42);
    expect(y).toEqual([42, 42, 42]);
});

// ── defaultOptions / validateOptions ────────────────────────────────

test('defaultOptions returns sensible starting values', () => {
    const o = defaultOptions();
    expect(o.lowess_frac).toBeGreaterThan(0);
    expect(o.lowess_frac).toBeLessThanOrEqual(1);
    expect(o.poly_degree).toBeGreaterThanOrEqual(1);
});

test('defaultOptions returns a fresh object per call', () => {
    const a = defaultOptions(); a.poly_degree = 99;
    const b = defaultOptions();
    expect(b.poly_degree).not.toBe(99);
});

test('validateOptions rejects bad LOWESS frac', () => {
    const o = defaultOptions(); o.lowess_frac = 0;
    expect(validateOptions(o)).toMatch(/LOWESS frac/);
    o.lowess_frac = 1.5;
    expect(validateOptions(o)).toMatch(/LOWESS frac/);
});

test('validateOptions rejects bad LOWESS robust iter', () => {
    const o = defaultOptions(); o.lowess_robust = 2.5;
    expect(validateOptions(o)).toMatch(/robustness/);
    o.lowess_robust = -1;
    expect(validateOptions(o)).toMatch(/robustness/);
});

test('validateOptions rejects negative kalman q / non-positive r', () => {
    const o = defaultOptions(); o.kalman_q = -1;
    expect(validateOptions(o)).toMatch(/process noise/);
    o.kalman_q = 0; o.kalman_r = 0;
    expect(validateOptions(o)).toMatch(/observation noise/);
});

test('validateOptions rejects non-integer / non-positive polynomial degree', () => {
    const o = defaultOptions(); o.poly_degree = 0;
    expect(validateOptions(o)).toMatch(/polynomial degree/);
    o.poly_degree = 2.5;
    expect(validateOptions(o)).toMatch(/polynomial degree/);
});

test('validateOptions returns null on defaults', () => {
    expect(validateOptions(defaultOptions())).toBe(null);
});
