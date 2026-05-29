// TWAP helpers: parser, validator, body shape, local TWAP parity,
// rolling TWAP, tagged-resp unwrap, Decimal coercion, demo invariants,
// formatters.

import { test, expect } from 'vitest';
import {
    parseTypicals, validateInputs, buildBody,
    localTwap, rollingTwap, decToNum, unwrapResponse,
    makeDemoData, fmtN, fmtBps,
} from '../js/_twap_inputs.js';

// ── parseTypicals (delegates to shared nonNegative parser) ────────

test('parseTypicals accepts decimals + rejects negatives via nonNegative gate', () => {
    const r = parseTypicals('100.5\n100.8\n-1\n# comment\n101');
    expect(r.value).toEqual([100.5, 100.8, 101]);
    expect(r.errors.length).toBe(1);
});

// ── validateInputs ────────────────────────────────────────────────

const okT = [100, 110, 120];

test('validate accepts good inputs', () => {
    expect(validateInputs('long', 105, okT)).toBe(null);
});

test('validate rejects bad side', () => {
    expect(validateInputs('flat', 100, okT)).toMatch(/side/);
});

test('validate rejects non-positive fill_price', () => {
    expect(validateInputs('long', 0, okT)).toMatch(/fill_price/);
});

test('validate rejects empty typicals', () => {
    expect(validateInputs('long', 100, [])).toMatch(/at least 1/);
});

test('validate rejects non-positive typical entries', () => {
    expect(validateInputs('long', 100, [100, 0, 110])).toMatch(/typical_prices/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody stringifies all Decimal scalars', () => {
    expect(buildBody('long', 105.5, [100.5, 110.5])).toEqual({
        side: 'long',
        fill_price: '105.5',
        typical_prices: ['100.5', '110.5'],
    });
});

// ── localTwap (parity with backend) ───────────────────────────────

test('localTwap arithmetic mean (backend parity example)', () => {
    // Backend test: typicals = [100, 110, 120] → TWAP = 110.
    expect(localTwap([100, 110, 120])).toBeCloseTo(110, 10);
});

test('localTwap single-bar = the price', () => {
    expect(localTwap([100])).toBe(100);
});

test('localTwap drops non-finite', () => {
    expect(localTwap([100, NaN, 110, Infinity, 120])).toBeCloseTo(110, 10);
});

test('localTwap empty/non-array → NaN', () => {
    expect(localTwap([])).toBeNaN();
    expect(localTwap(null)).toBeNaN();
});

// ── rollingTwap ───────────────────────────────────────────────────

test('rollingTwap accumulates arithmetic mean correctly', () => {
    const r = rollingTwap([100, 110, 120]);
    expect(r[0]).toBeCloseTo(100, 10);
    expect(r[1]).toBeCloseTo(105, 10);
    expect(r[2]).toBeCloseTo(110, 10);
});

test('rollingTwap null-on-non-finite + safe on non-array', () => {
    const r = rollingTwap([100, NaN, 110]);
    expect(r[1]).toBe(null);
    // After the null, the accumulator continues from the prior 100.
    // Bar 2 valid → sum = 100+110 = 210, count = 2 → 105.
    expect(r[2]).toBeCloseTo(105, 10);
    expect(rollingTwap(null)).toEqual([]);
});

// ── unwrapResponse ───────────────────────────────────────────────

test('unwrapResponse handles wrapped result', () => {
    const u = unwrapResponse({ result: { twap: '100', beat_twap: true } });
    expect(u.ok).toBe(true);
    expect(u.result.twap).toBe('100');
});

test('unwrapResponse: null result → ok=false with reason', () => {
    const u = unwrapResponse({ result: null });
    expect(u.ok).toBe(false);
    expect(u.reason).toMatch(/null/);
});

test('unwrapResponse rejects malformed', () => {
    expect(unwrapResponse(null).ok).toBe(false);
});

// ── decToNum ──────────────────────────────────────────────────────

test('decToNum: Decimal-string + number + null + garbage', () => {
    expect(decToNum('100.5')).toBe(100.5);
    expect(decToNum(100.5)).toBe(100.5);
    expect(decToNum(null)).toBeNaN();
    expect(decToNum('xyz')).toBeNaN();
});

// ── makeDemoData ──────────────────────────────────────────────────

test('makeDemoData deterministic + exactly 200 typicals + long side', () => {
    const a = makeDemoData(42);
    const b = makeDemoData(42);
    expect(a).toEqual(b);
    expect(a.typicals.length).toBe(200);
    expect(a.side).toBe('long');
});

test('makeDemoData fill_price below mean across seeds (beat-TWAP invariant)', () => {
    for (const seed of [1, 7, 42, 1337]) {
        const { fill_price, typicals } = makeDemoData(seed);
        const mean = typicals.reduce((a, b) => a + b, 0) / typicals.length;
        expect(fill_price).toBeLessThan(mean);
    }
});

// ── formatters ────────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(100.12345)).toBe('100.1235');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtBps(12.3)).toBe('+12.3 bps');
    expect(fmtBps(-5.4)).toBe('-5.4 bps');
    expect(fmtBps(NaN)).toBe('—');
});
