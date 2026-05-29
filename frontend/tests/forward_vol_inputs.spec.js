// Forward Vol Curve pure helpers: tenor parser, term-structure
// parser, duplicate-tenor checker, validator, payload builder, step
// series generator.

import { test, expect } from 'vitest';
import {
    parseTenor, parseTermStructure, sortRowsByTenor,
    checkUniqueTenors, validateTermStructure, buildBody,
    forwardVolStepSeries,
} from '../js/_forward_vol_inputs.js';

// ── parseTenor ──────────────────────────────────────────────────────

test('parseTenor accepts bare years', () => {
    expect(parseTenor('0.25')).toBe(0.25);
    expect(parseTenor('1.5')).toBe(1.5);
});

test('parseTenor accepts "D" (days)', () => {
    expect(parseTenor('1D')).toBeCloseTo(1 / 365, 12);
    expect(parseTenor('30D')).toBeCloseTo(30 / 365, 12);
});

test('parseTenor accepts "W" (weeks)', () => {
    expect(parseTenor('1W')).toBeCloseTo(7 / 365, 12);
});

test('parseTenor accepts "M" (months)', () => {
    expect(parseTenor('1M')).toBeCloseTo(1 / 12, 12);
    expect(parseTenor('3M')).toBeCloseTo(3 / 12, 12);
});

test('parseTenor accepts "Y" (years)', () => {
    expect(parseTenor('1Y')).toBe(1);
    expect(parseTenor('2Y')).toBe(2);
});

test('parseTenor accepts fractional units', () => {
    expect(parseTenor('1.5Y')).toBe(1.5);
    expect(parseTenor('0.5M')).toBeCloseTo(0.5 / 12, 12);
});

test('parseTenor is case-insensitive for units', () => {
    expect(parseTenor('1m')).toBeCloseTo(1 / 12, 12);
    expect(parseTenor('1y')).toBe(1);
});

test('parseTenor returns NaN on garbage', () => {
    expect(Number.isNaN(parseTenor(''))).toBe(true);
    expect(Number.isNaN(parseTenor('foo'))).toBe(true);
    expect(Number.isNaN(parseTenor('1Z'))).toBe(true);   // unknown unit
});

// ── parseTermStructure ──────────────────────────────────────────────

test('parseTermStructure accepts tenor + IV pairs', () => {
    const r = parseTermStructure('1M 20%\n3M 22%\n6M 23%');
    expect(r.errors).toEqual([]);
    expect(r.value.length).toBe(3);
    expect(r.value[0].iv).toBeCloseTo(0.20, 12);
    expect(r.value[0].tenor_years).toBeCloseTo(1 / 12, 12);
});

test('parseTermStructure handles comma separator', () => {
    const r = parseTermStructure('1M,20\n3M,22');
    expect(r.errors).toEqual([]);
    expect(r.value.length).toBe(2);
});

test('parseTermStructure skips blanks and # comments', () => {
    expect(parseTermStructure('# hdr\n\n1M 20%\n# inline\n3M 22%').value.length).toBe(2);
});

test('parseTermStructure reports bad tenor', () => {
    const r = parseTermStructure('1Q 20%');
    expect(r.value.length).toBe(0);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/bad tenor/);
});

test('parseTermStructure reports bad IV', () => {
    const r = parseTermStructure('1M abc');
    expect(r.value.length).toBe(0);
    expect(r.errors[0].message).toMatch(/bad IV/);
});

test('parseTermStructure reports missing-second-field', () => {
    const r = parseTermStructure('1M');
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/two fields/);
});

// ── sortRowsByTenor ─────────────────────────────────────────────────

test('sortRowsByTenor sorts ascending and does not mutate input', () => {
    const rows = [
        { tenor_years: 0.5, iv: 0.22, line_no: 1, raw_tenor: '6M' },
        { tenor_years: 0.08, iv: 0.20, line_no: 2, raw_tenor: '1M' },
    ];
    const sorted = sortRowsByTenor(rows);
    expect(sorted.map(r => r.tenor_years)).toEqual([0.08, 0.5]);
    expect(rows[0].tenor_years).toBe(0.5);    // original untouched
});

// ── checkUniqueTenors ──────────────────────────────────────────────

test('checkUniqueTenors accepts strictly-increasing', () => {
    expect(checkUniqueTenors([
        { tenor_years: 0.1, line_no: 1 },
        { tenor_years: 0.5, line_no: 2 },
        { tenor_years: 1.0, line_no: 3 },
    ])).toBe(null);
});

test('checkUniqueTenors rejects duplicates', () => {
    expect(checkUniqueTenors([
        { tenor_years: 0.5, line_no: 1 },
        { tenor_years: 0.5, line_no: 2 },
    ])).toMatch(/line 2/);
});

test('checkUniqueTenors rejects regressing tenors', () => {
    expect(checkUniqueTenors([
        { tenor_years: 1.0, line_no: 1 },
        { tenor_years: 0.5, line_no: 2 },
    ])).toMatch(/line 2/);
});

// ── validateTermStructure ───────────────────────────────────────────

test('validateTermStructure rejects fewer than 2 rows', () => {
    expect(validateTermStructure([{ tenor_years: 1, iv: 0.2 }])).toMatch(/at least 2/);
});

test('validateTermStructure rejects non-positive tenor', () => {
    expect(validateTermStructure([
        { tenor_years: 0, iv: 0.2 },
        { tenor_years: 1, iv: 0.2 },
    ])).toMatch(/tenors/);
});

test('validateTermStructure rejects negative IV', () => {
    expect(validateTermStructure([
        { tenor_years: 0.5, iv: -0.01 },
        { tenor_years: 1.0, iv: 0.20 },
    ])).toMatch(/IVs/);
});

test('validateTermStructure accepts good input', () => {
    expect(validateTermStructure([
        { tenor_years: 0.5, iv: 0.20 },
        { tenor_years: 1.0, iv: 0.22 },
    ])).toBe(null);
});

// ── buildBody ───────────────────────────────────────────────────────

test('buildBody extracts parallel arrays', () => {
    const b = buildBody([
        { tenor_years: 0.5, iv: 0.20 },
        { tenor_years: 1.0, iv: 0.22 },
    ]);
    expect(b).toEqual({ expiries: [0.5, 1.0], spot_iv: [0.20, 0.22] });
});

// ── forwardVolStepSeries ────────────────────────────────────────────

test('forwardVolStepSeries emits 2 points per forward interval', () => {
    const rows = [
        { tenor_years: 0.25 }, { tenor_years: 0.5 }, { tenor_years: 1.0 },
    ];
    const fwd = [0.21, 0.23];
    const { xs, ys } = forwardVolStepSeries(rows, fwd);
    // Interval 1: [0.25, 0.5] @ 0.21; interval 2: [0.5, 1.0] @ 0.23
    expect(xs).toEqual([0.25, 0.5, 0.5, 1.0]);
    expect(ys).toEqual([0.21, 0.21, 0.23, 0.23]);
});

test('forwardVolStepSeries handles empty inputs', () => {
    expect(forwardVolStepSeries([], [])).toEqual({ xs: [], ys: [] });
    expect(forwardVolStepSeries(null, null)).toEqual({ xs: [], ys: [] });
});

test('forwardVolStepSeries stops at min(forwardVols.len, rows.len-1)', () => {
    // 2 rows but 5 forward vols → emit only 1 interval.
    const rows = [{ tenor_years: 0 }, { tenor_years: 1 }];
    const fwd = [0.2, 0.3, 0.4, 0.5, 0.6];
    const { xs, ys } = forwardVolStepSeries(rows, fwd);
    expect(xs).toEqual([0, 1]);
    expect(ys).toEqual([0.2, 0.2]);
});
