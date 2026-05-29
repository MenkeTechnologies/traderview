// Vol-smile fitter pure helpers. The view itself is DOM-bound; this
// covers everything that's testable in isolation: parser, IV
// conversion (decimal vs percent ambiguity), payload shaping,
// validation, ATM skew slope formula.

import { test, expect } from 'vitest';
import {
    parseStrikeIvText, parseIv, buildSviBody, validateSmileInputs,
    sortRowsByStrike, atmSkewSlope,
} from '../js/_vol_smile_inputs.js';

// ── parseIv ──────────────────────────────────────────────────────────

test('parseIv accepts decimal form 0.25', () => {
    expect(parseIv('0.25')).toBe(0.25);
});

test('parseIv treats values ≥ 1 as percent (25 → 0.25)', () => {
    expect(parseIv('25')).toBe(0.25);
});

test('parseIv handles trailing percent sign', () => {
    expect(parseIv('25%')).toBe(0.25);
    expect(parseIv('25.5%')).toBe(0.255);
});

test('parseIv rejects negatives + non-numeric', () => {
    expect(Number.isNaN(parseIv('-0.5'))).toBe(true);
    expect(Number.isNaN(parseIv('abc'))).toBe(true);
});

test('parseIv handles weird whitespace inside percent', () => {
    expect(parseIv(' 25 %')).toBe(0.25);
});

// ── parseStrikeIvText ────────────────────────────────────────────────

test('parser extracts simple whitespace-separated rows', () => {
    const t = `100 25%\n105 24%\n110 25.5%`;
    const r = parseStrikeIvText(t);
    expect(r.errors).toEqual([]);
    expect(r.rows.length).toBe(3);
    expect(r.rows[0]).toEqual({ strike: 100, iv: 0.25, line_no: 1 });
});

test('parser handles commas as separator', () => {
    const r = parseStrikeIvText('100,0.25\n105,0.24');
    expect(r.errors).toEqual([]);
    expect(r.rows.length).toBe(2);
});

test('parser ignores blank lines and # comments', () => {
    const t = `# header\n\n100 0.25\n# another\n105 0.24`;
    const r = parseStrikeIvText(t);
    expect(r.errors).toEqual([]);
    expect(r.rows.length).toBe(2);
});

test('parser reports line number on malformed row', () => {
    const t = `100 0.25\nbroken\n105 0.24`;
    const r = parseStrikeIvText(t);
    expect(r.rows.length).toBe(2);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].line_no).toBe(2);
    expect(r.errors[0].raw).toBe('broken');
});

test('parser reports bad strike with original token', () => {
    const r = parseStrikeIvText('foo 0.25');
    expect(r.rows).toEqual([]);
    expect(r.errors[0].message).toMatch(/bad strike "foo"/);
});

test('parser reports bad iv with original token', () => {
    const r = parseStrikeIvText('100 nope');
    expect(r.rows).toEqual([]);
    expect(r.errors[0].message).toMatch(/bad IV "nope"/);
});

test('parser rejects zero/negative strike', () => {
    const r = parseStrikeIvText('0 0.25\n-10 0.25');
    expect(r.rows).toEqual([]);
    expect(r.errors.length).toBe(2);
});

test('parser ignores trailing fields beyond strike + iv', () => {
    // Real chains have 6+ columns; we only care about first 2.
    const r = parseStrikeIvText('100 0.25 5.50 4.80 1000 250');
    expect(r.errors).toEqual([]);
    expect(r.rows[0]).toEqual({ strike: 100, iv: 0.25, line_no: 1 });
});

// ── buildSviBody ─────────────────────────────────────────────────────

test('buildSviBody computes log-moneyness against forward', () => {
    // Spot 100, rate 0, div 0, t = 1 → forward = 100.
    // Strike 100 → log(100/100) = 0.
    // Strike 110 → log(110/100) ≈ 0.0953.
    const rows = [
        { strike: 100, iv: 0.25, line_no: 1 },
        { strike: 110, iv: 0.30, line_no: 2 },
    ];
    const body = buildSviBody(rows, 100, 0, 0, 1);
    expect(body.log_moneyness[0]).toBeCloseTo(0, 9);
    expect(body.log_moneyness[1]).toBeCloseTo(Math.log(1.1), 9);
});

test('buildSviBody computes total variance as iv²·t', () => {
    const rows = [{ strike: 100, iv: 0.20, line_no: 1 }];
    const body = buildSviBody(rows, 100, 0, 0, 0.5);
    expect(body.total_variance[0]).toBeCloseTo(0.04 * 0.5, 12);
});

test('buildSviBody forward responds to rate and divs', () => {
    // Spot 100, rate 0.05, div 0.01, t = 1 → forward = 100 · exp(0.04) ≈ 104.08.
    const rows = [{ strike: 100, iv: 0.20, line_no: 1 }];
    const body = buildSviBody(rows, 100, 0.05, 0.01, 1);
    const expected = Math.log(100 / (100 * Math.exp(0.04)));
    expect(body.log_moneyness[0]).toBeCloseTo(expected, 9);
});

test('buildSviBody passes through expiry_years', () => {
    const body = buildSviBody([], 100, 0, 0, 0.25);
    expect(body.expiry_years).toBe(0.25);
});

// ── validateSmileInputs ──────────────────────────────────────────────

test('validateSmileInputs requires at least 5 rows', () => {
    const r = [];
    for (let i = 0; i < 4; i++) r.push({ strike: 100 + i, iv: 0.25, line_no: i + 1 });
    expect(validateSmileInputs(r, 100, 1)).toMatch(/at least 5/);
});

test('validateSmileInputs rejects bad spot/t', () => {
    const r = [];
    for (let i = 0; i < 5; i++) r.push({ strike: 100 + i, iv: 0.25, line_no: i + 1 });
    expect(validateSmileInputs(r, 0, 1)).toMatch(/spot/);
    expect(validateSmileInputs(r, 100, 0)).toMatch(/expiry/);
    expect(validateSmileInputs(r, 100, -1)).toMatch(/expiry/);
});

test('validateSmileInputs returns null on good input', () => {
    const r = [];
    for (let i = 0; i < 5; i++) r.push({ strike: 100 + i, iv: 0.25, line_no: i + 1 });
    expect(validateSmileInputs(r, 100, 0.5)).toBe(null);
});

// ── sortRowsByStrike ─────────────────────────────────────────────────

test('sortRowsByStrike sorts ascending by strike', () => {
    const r = [
        { strike: 110, iv: 0.30, line_no: 1 },
        { strike: 100, iv: 0.25, line_no: 2 },
        { strike: 105, iv: 0.28, line_no: 3 },
    ];
    const s = sortRowsByStrike(r);
    expect(s.map(x => x.strike)).toEqual([100, 105, 110]);
});

test('sortRowsByStrike does not mutate input', () => {
    const r = [{ strike: 110, iv: 0.30, line_no: 1 }, { strike: 100, iv: 0.25, line_no: 2 }];
    sortRowsByStrike(r);
    expect(r[0].strike).toBe(110);    // original order preserved
});

// ── atmSkewSlope ─────────────────────────────────────────────────────

test('atmSkewSlope returns 0 when fitted IV is zero', () => {
    const slope = atmSkewSlope({ a: 0, b: 0.1, rho: 0, m: 0, sigma: 0.1 }, 1);
    expect(slope).toBe(0);
});

test('atmSkewSlope is negative when SVI ρ is negative (equity-style skew)', () => {
    const slope = atmSkewSlope({ a: 0.04, b: 0.1, rho: -0.5, m: 0, sigma: 0.1 }, 1);
    expect(slope).toBeLessThan(0);
});

test('atmSkewSlope is positive when SVI ρ is positive', () => {
    const slope = atmSkewSlope({ a: 0.04, b: 0.1, rho: 0.5, m: 0, sigma: 0.1 }, 1);
    expect(slope).toBeGreaterThan(0);
});

test('atmSkewSlope returns 0 for non-positive expiry', () => {
    expect(atmSkewSlope({ a: 0.04, b: 0.1, rho: -0.5, m: 0, sigma: 0.1 }, 0)).toBe(0);
});
