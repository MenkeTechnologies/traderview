// Murrey Math helpers: HLC parser, validator, body shape, level
// significance map, octave-position classifier, bracketing-level
// finder, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseBarBlob, validateInputs, buildBody,
    LEVEL_SIGNIFICANCE, significanceOf,
    pricePosition, bracketingLevels,
    makeDemoBars, fmtN, fmtPct,
} from '../js/_murrey_math_inputs.js';

// ── parseBarBlob ───────────────────────────────────────────────────

test('parseBarBlob accepts whitespace + commas + comments', () => {
    const r = parseBarBlob('# h\n100.6 99.4 100.0\n100.8, 99.6, 100.2');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([
        { high: 100.6, low: 99.4, close: 100.0 },
        { high: 100.8, low: 99.6, close: 100.2 },
    ]);
});

test('parseBarBlob rejects wrong token count', () => {
    expect(parseBarBlob('100 99').errors[0].message).toMatch(/expected 3 tokens/);
});

test('parseBarBlob rejects non-positive HLC + low>high + close outside', () => {
    expect(parseBarBlob('0 1 1').errors[0].message).toMatch(/HLC/);
    expect(parseBarBlob('99 100 99').errors[0].message).toMatch(/low > high/);
    expect(parseBarBlob('100 99 105').errors[0].message).toMatch(/close outside/);
});

test('parseBarBlob non-string returns 1 error', () => {
    expect(parseBarBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([{ high: 100, low: 99, close: 99.5 }], 64)).toBe(null);
});

test('validate rejects empty + non-positive lookback', () => {
    expect(validateInputs([], 64)).toMatch(/at least 1 bar/);
    expect(validateInputs([{}], 0)).toMatch(/lookback_bars/);
    expect(validateInputs([{}], 1.5)).toMatch(/lookback_bars/);
});

test('buildBody emits backend MurreyMathBody shape', () => {
    const bars = [{ high: 100, low: 99, close: 99.5 }];
    expect(buildBody(bars, 64)).toEqual({ bars, lookback_bars: 64 });
});

// ── LEVEL_SIGNIFICANCE ────────────────────────────────────────────

test('LEVEL_SIGNIFICANCE has all 13 levels from -2/8 to 10/8', () => {
    const labels = Object.keys(LEVEL_SIGNIFICANCE);
    expect(labels.length).toBe(13);
    for (let k = -2; k <= 10; k++) {
        expect(LEVEL_SIGNIFICANCE).toHaveProperty(`${k}/8`);
    }
});

test('LEVEL_SIGNIFICANCE: 0/8, 4/8, 8/8 are critical rank', () => {
    expect(LEVEL_SIGNIFICANCE['0/8'].rank).toBe('critical');
    expect(LEVEL_SIGNIFICANCE['4/8'].rank).toBe('critical');
    expect(LEVEL_SIGNIFICANCE['8/8'].rank).toBe('critical');
});

test('LEVEL_SIGNIFICANCE: 2/8 and 6/8 are major (Murrey pivot zones)', () => {
    expect(LEVEL_SIGNIFICANCE['2/8'].rank).toBe('major');
    expect(LEVEL_SIGNIFICANCE['6/8'].rank).toBe('major');
});

test('LEVEL_SIGNIFICANCE: -2/-1 and 9/10 are extensions', () => {
    expect(LEVEL_SIGNIFICANCE['-2/8'].rank).toBe('extended');
    expect(LEVEL_SIGNIFICANCE['10/8'].rank).toBe('extended');
});

test('significanceOf unknown fallthrough', () => {
    expect(significanceOf('garbage').rank).toBe('unknown');
});

// ── pricePosition ─────────────────────────────────────────────────

const sampleLevels = [
    ['-2/8', 90], ['-1/8', 91.25], ['0/8', 92.5], ['1/8', 93.75], ['2/8', 95],
    ['3/8', 96.25], ['4/8', 97.5], ['5/8', 98.75], ['6/8', 100], ['7/8', 101.25],
    ['8/8', 102.5], ['9/8', 103.75], ['10/8', 105],
];

test('pricePosition: below 0/8 → "below octave"', () => {
    expect(pricePosition(91, sampleLevels)).toBe('below octave');
});

test('pricePosition: above 8/8 → "above octave"', () => {
    expect(pricePosition(103, sampleLevels)).toBe('above octave');
});

test('pricePosition: between 0/8 and 4/8 → "lower half"', () => {
    expect(pricePosition(95, sampleLevels)).toBe('lower half');
});

test('pricePosition: between 4/8 and 8/8 → "upper half"', () => {
    expect(pricePosition(100, sampleLevels)).toBe('upper half');
});

test('pricePosition: empty / non-finite returns unknown', () => {
    expect(pricePosition(95, [])).toBe('unknown');
    expect(pricePosition(NaN, sampleLevels)).toBe('unknown');
});

// ── bracketingLevels ──────────────────────────────────────────────

test('bracketingLevels finds immediate below + above', () => {
    const { below, above } = bracketingLevels(96.5, sampleLevels);
    expect(below[0]).toBe('3/8');
    expect(above[0]).toBe('4/8');
});

test('bracketingLevels: price exactly on a level appears in both sides', () => {
    const { below, above } = bracketingLevels(97.5, sampleLevels);
    expect(below[0]).toBe('4/8');
    expect(above[0]).toBe('4/8');
});

test('bracketingLevels: price below all levels → only above is set', () => {
    const { below, above } = bracketingLevels(50, sampleLevels);
    expect(below).toBe(null);
    expect(above[0]).toBe('-2/8');
});

test('bracketingLevels: price above all levels → only below is set', () => {
    const { below, above } = bracketingLevels(200, sampleLevels);
    expect(above).toBe(null);
    expect(below[0]).toBe('10/8');
});

test('bracketingLevels safe on empty levels + NaN price', () => {
    expect(bracketingLevels(95, [])).toEqual({ below: null, above: null });
    expect(bracketingLevels(NaN, sampleLevels)).toEqual({ below: null, above: null });
});

// ── makeDemoBars ──────────────────────────────────────────────────

test('makeDemoBars returns 80 bars with valid HLC', () => {
    const bars = makeDemoBars();
    expect(bars.length).toBe(80);
    expect(bars.every(b => b.low <= b.high && b.close >= b.low && b.close <= b.high && b.high > 0)).toBe(true);
});

test('makeDemoBars range is bounded (within ~10 points around 100)', () => {
    const bars = makeDemoBars();
    const max = Math.max(...bars.map(b => b.high));
    const min = Math.min(...bars.map(b => b.low));
    expect(max - min).toBeGreaterThan(5);   // non-trivial swing
    expect(max - min).toBeLessThan(15);     // bounded
});

// ── formatters ────────────────────────────────────────────────────

test('formatters', () => {
    expect(fmtN(100.5)).toBe('100.5000');
    expect(fmtN(100.5, 2)).toBe('100.50');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtPct(2.345)).toBe('2.35%');
    expect(fmtPct(NaN)).toBe('—');
});
