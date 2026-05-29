// CUSUM helpers: series parser, validator, body shape, mean+stdev,
// event-marker spreader, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseSeries, validateInputs, buildBody,
    meanStdev, eventMarkers, makeDemoSeries,
    fmtN, dirCss,
} from '../js/_cusum_inputs.js';

// ── parseSeries (delegates) ────────────────────────────────────────

test('parseSeries accepts signed values + comments', () => {
    const r = parseSeries('# header\n0.005\n-0.003\n0.001');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([0.005, -0.003, 0.001]);
});

// ── validateInputs ─────────────────────────────────────────────────

const okCfg = { reference_mean: 0, reference_stdev: 1, threshold_stdevs: 5, slack: 0.5 };

test('validate accepts ≥10 series + good config', () => {
    expect(validateInputs(Array(20).fill(0.01), okCfg)).toBe(null);
});

test('validate rejects < 10 observations', () => {
    expect(validateInputs(Array(5).fill(0.01), okCfg)).toMatch(/at least 10/);
});

test('validate rejects non-finite series values', () => {
    const xs = [...Array(20).fill(0.01)];
    xs[5] = NaN;
    expect(validateInputs(xs, okCfg)).toMatch(/series must be finite/);
});

test('validate rejects bad config scalars', () => {
    const xs = Array(20).fill(0.01);
    expect(validateInputs(xs, { ...okCfg, reference_mean: NaN })).toMatch(/reference_mean/);
    expect(validateInputs(xs, { ...okCfg, reference_stdev: 0 })).toMatch(/reference_stdev/);
    expect(validateInputs(xs, { ...okCfg, reference_stdev: -1 })).toMatch(/reference_stdev/);
    expect(validateInputs(xs, { ...okCfg, threshold_stdevs: 0 })).toMatch(/threshold_stdevs/);
    expect(validateInputs(xs, { ...okCfg, slack: -0.1 })).toMatch(/slack/);
});

test('validate accepts slack = 0 (disable drift)', () => {
    expect(validateInputs(Array(20).fill(0.01), { ...okCfg, slack: 0 })).toBe(null);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody emits backend CusumBody shape', () => {
    expect(buildBody([1, 2], okCfg)).toEqual({ series: [1, 2], config: okCfg });
});

// ── meanStdev (Welford) ───────────────────────────────────────────

test('meanStdev computes sample stdev (n-1 denominator)', () => {
    const { mean, stdev } = meanStdev([2, 4, 4, 4, 5, 5, 7, 9]);
    expect(mean).toBeCloseTo(5, 10);
    expect(stdev).toBeCloseTo(2.138089935, 6);
});

test('meanStdev with n < 2 returns NaN', () => {
    expect(meanStdev([]).mean).toBe(NaN);
    expect(meanStdev([5]).mean).toBe(NaN);
});

test('meanStdev drops non-finite from the input', () => {
    const { mean } = meanStdev([1, NaN, 2, Infinity, 3]);
    expect(mean).toBeCloseTo(2, 10);
});

// ── eventMarkers ──────────────────────────────────────────────────

test('eventMarkers spreads events into parallel up/down series', () => {
    const events = [
        { bar_index: 2, direction: 'up',   cusum_value: 6.5 },
        { bar_index: 5, direction: 'down', cusum_value: 5.8 },
    ];
    const { up, dn } = eventMarkers(events, 7);
    expect(up).toEqual([null, null, 6.5, null, null, null, null]);
    expect(dn).toEqual([null, null, null, null, null, 5.8, null]);
});

test('eventMarkers ignores out-of-bounds bar_index', () => {
    const { up, dn } = eventMarkers([
        { bar_index: -1, direction: 'up', cusum_value: 1 },
        { bar_index: 10, direction: 'down', cusum_value: 1 },
    ], 5);
    expect(up.every(v => v === null)).toBe(true);
    expect(dn.every(v => v === null)).toBe(true);
});

test('eventMarkers ignores non-array input', () => {
    const { up, dn } = eventMarkers(null, 3);
    expect(up).toEqual([null, null, null]);
    expect(dn).toEqual([null, null, null]);
});

// ── makeDemoSeries ────────────────────────────────────────────────

test('makeDemoSeries deterministic for fixed seed + exactly 200 bars', () => {
    const a = makeDemoSeries(42);
    const b = makeDemoSeries(42);
    expect(a).toEqual(b);
    expect(a.length).toBe(200);
});

test('makeDemoSeries first-half mean > second-half mean (regime flip)', () => {
    const xs = makeDemoSeries(1);
    const m1 = xs.slice(0, 100).reduce((a, b) => a + b, 0) / 100;
    const m2 = xs.slice(100).reduce((a, b) => a + b, 0) / 100;
    expect(m1).toBeGreaterThan(0);
    expect(m2).toBeLessThan(0);
    expect(m1).toBeGreaterThan(m2 + 0.01);   // separation ≥ 1%
});

// ── formatters / dirCss ───────────────────────────────────────────

test('fmtN handles non-finite + digit override', () => {
    expect(fmtN(NaN)).toBe('—');
    expect(fmtN(1.23456)).toBe('1.2346');
    expect(fmtN(1.5, 1)).toBe('1.5');
});

test('dirCss maps up/down/other', () => {
    expect(dirCss('up')).toBe('pos');
    expect(dirCss('down')).toBe('neg');
    expect(dirCss('sideways')).toBe('');
});
