// VaR Calculator pure helpers: parser, validator, alpha conversion,
// histogram, formatLoss.

import { test, expect } from 'vitest';
import {
    parseReturns, validateReturns, confidenceToAlpha,
    histogram, formatLoss,
} from '../js/_var_calculator_inputs.js';

// ── parseReturns ─────────────────────────────────────────────────────

test('parseReturns accepts one-per-line', () => {
    const r = parseReturns('-0.01\n0.005\n-0.02');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([-0.01, 0.005, -0.02]);
});

test('parseReturns accepts comma-separated', () => {
    const r = parseReturns('-0.01, 0.005, -0.02');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([-0.01, 0.005, -0.02]);
});

test('parseReturns accepts multiple per line', () => {
    const r = parseReturns('-0.01 0.005 -0.02\n0.01 -0.005');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([-0.01, 0.005, -0.02, 0.01, -0.005]);
});

test('parseReturns ignores blank + # lines', () => {
    const r = parseReturns('# header\n\n-0.01\n# inline\n0.02');
    expect(r.errors).toEqual([]);
    expect(r.value).toEqual([-0.01, 0.02]);
});

test('parseReturns reports bad token line', () => {
    const r = parseReturns('-0.01\nnope\n0.02');
    expect(r.value).toEqual([-0.01, 0.02]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].line_no).toBe(2);
});

test('parseReturns reports all bad tokens on a mixed line', () => {
    const r = parseReturns('0.01 foo 0.02 bar');
    expect(r.value).toEqual([0.01, 0.02]);
    expect(r.errors.length).toBe(2);
});

test('parseReturns empty input returns empty', () => {
    expect(parseReturns('')).toEqual({ value: [], errors: [] });
});

// ── validateReturns ─────────────────────────────────────────────────

test('validateReturns rejects too-few returns', () => {
    expect(validateReturns(Array(10).fill(0.01))).toMatch(/at least 20/);
});

test('validateReturns rejects constant series', () => {
    expect(validateReturns(Array(30).fill(0.01))).toMatch(/constant/);
});

test('validateReturns rejects non-finite', () => {
    const r = Array(30).fill(0.01); r[5] = NaN;
    expect(validateReturns(r)).toMatch(/non-finite/);
});

test('validateReturns returns null on a varied 30-element series', () => {
    const r = Array.from({ length: 30 }, (_, i) => (i - 15) / 1000);
    expect(validateReturns(r)).toBe(null);
});

// ── confidenceToAlpha ───────────────────────────────────────────────

test('confidenceToAlpha converts correctly', () => {
    expect(confidenceToAlpha(0.95)).toBeCloseTo(0.05, 12);
    expect(confidenceToAlpha(0.99)).toBeCloseTo(0.01, 12);
    expect(confidenceToAlpha(0.999)).toBeCloseTo(0.001, 12);
});

// ── histogram ───────────────────────────────────────────────────────

test('histogram counts sum to N', () => {
    const r = Array.from({ length: 100 }, (_, i) => (i - 50) / 1000);
    const h = histogram(r, 20);
    const sum = h.counts.reduce((a, b) => a + b, 0);
    expect(sum).toBe(100);
});

test('histogram centers are evenly spaced', () => {
    const r = Array.from({ length: 100 }, (_, i) => (i - 50) / 1000);
    const h = histogram(r, 10);
    const gaps = h.centers.slice(1).map((c, i) => c - h.centers[i]);
    for (const g of gaps) expect(g).toBeCloseTo(h.binWidth, 12);
});

test('histogram with empty input returns empty', () => {
    expect(histogram([])).toEqual({ centers: [], counts: [], binWidth: 0 });
});

test('histogram with constant series collapses to a single bin', () => {
    const r = Array(50).fill(0.01);
    const h = histogram(r, 20);
    expect(h.centers).toEqual([0.01]);
    expect(h.counts).toEqual([50]);
});

test('histogram rejects nbins < 1', () => {
    expect(histogram([0.01, 0.02], 0)).toEqual({ centers: [], counts: [], binWidth: 0 });
});

// ── formatLoss ──────────────────────────────────────────────────────

test('formatLoss prefixes negative percent', () => {
    expect(formatLoss(0.025)).toBe('-2.50%');
});

test('formatLoss handles non-finite', () => {
    expect(formatLoss(NaN)).toBe('—');
    expect(formatLoss(Infinity)).toBe('—');
    expect(formatLoss(null)).toBe('—');
});

test('formatLoss respects digits arg', () => {
    expect(formatLoss(0.025, 4)).toBe('-2.5000%');
});
