// Market Profile (TPO) helpers: bracket parser, validator, body shape,
// letter mapping, tier classifier, level letters, tier counts, demo
// invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseBracketBlob, validateInputs, buildBody,
    bracketLetter, TPO_LETTERS, levelTier, levelLetters,
    tierCounts, makeDemoBrackets, fmtN, fmtInt,
} from '../js/_market_profile_inputs.js';

// ── parseBracketBlob ───────────────────────────────────────────────

test('parseBracketBlob accepts whitespace + commas + comments', () => {
    const r = parseBracketBlob('# header\n0 102.5 101.0\n1, 101.5, 100.0');
    expect(r.errors).toEqual([]);
    expect(r.brackets).toEqual([
        { bracket_index: 0, high: 102.5, low: 101.0 },
        { bracket_index: 1, high: 101.5, low: 100.0 },
    ]);
});

test('parseBracketBlob rejects wrong token count', () => {
    expect(parseBracketBlob('0 100').errors[0].message).toMatch(/expected 3 tokens/);
});

test('parseBracketBlob rejects non-integer / negative bracket_index', () => {
    expect(parseBracketBlob('1.5 100 99').errors[0].message).toMatch(/bracket_index/);
    expect(parseBracketBlob('-1 100 99').errors[0].message).toMatch(/bracket_index/);
});

test('parseBracketBlob rejects non-positive high or low', () => {
    expect(parseBracketBlob('0 0 99').errors[0].message).toMatch(/high/);
    expect(parseBracketBlob('0 100 0').errors[0].message).toMatch(/low/);
});

test('parseBracketBlob rejects high < low', () => {
    expect(parseBracketBlob('0 99 100').errors[0].message).toMatch(/high must be ≥ low/);
});

test('parseBracketBlob accepts high == low (single-tick bracket)', () => {
    const r = parseBracketBlob('0 100 100');
    expect(r.errors).toEqual([]);
    expect(r.brackets).toEqual([{ bracket_index: 0, high: 100, low: 100 }]);
});

test('parseBracketBlob non-string returns 1 error', () => {
    expect(parseBracketBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([{ bracket_index: 0, high: 100, low: 99 }], 0.5)).toBe(null);
});

test('validate rejects empty brackets', () => {
    expect(validateInputs([], 0.5)).toMatch(/at least 1 bracket/);
});

test('validate rejects non-positive tick_size', () => {
    expect(validateInputs([{ bracket_index: 0, high: 100, low: 99 }], 0)).toMatch(/tick_size/);
    expect(validateInputs([{ bracket_index: 0, high: 100, low: 99 }], -0.5)).toMatch(/tick_size/);
});

test('buildBody emits backend MarketProfileBody shape', () => {
    const br = [{ bracket_index: 0, high: 100, low: 99 }];
    expect(buildBody(br, 0.5)).toEqual({ brackets: br, tick_size: 0.5 });
});

// ── TPO_LETTERS / bracketLetter ───────────────────────────────────

test('TPO_LETTERS has 52 entries (A-Z + a-z)', () => {
    expect(TPO_LETTERS.length).toBe(52);
    expect(TPO_LETTERS[0]).toBe('A');
    expect(TPO_LETTERS[25]).toBe('Z');
    expect(TPO_LETTERS[26]).toBe('a');
    expect(TPO_LETTERS[51]).toBe('z');
});

test('bracketLetter maps idx → letter, wraps at 52', () => {
    expect(bracketLetter(0)).toBe('A');
    expect(bracketLetter(7)).toBe('H');
    expect(bracketLetter(26)).toBe('a');
    expect(bracketLetter(52)).toBe('A');     // wrap
    expect(bracketLetter(-1)).toBe('?');
    expect(bracketLetter(1.5)).toBe('?');
});

// ── levelTier / levelLetters ──────────────────────────────────────

const sampleReport = {
    poc_price: 100,
    value_area_high: 101,
    value_area_low: 99,
    levels: [
        { price: 100, tpo_count: 5, single_print: false, brackets: [0, 1, 2, 3, 4] },
        { price: 100.5, tpo_count: 3, single_print: false, brackets: [1, 2, 3] },
        { price: 99.5,  tpo_count: 2, single_print: false, brackets: [4, 5] },
        { price: 102,   tpo_count: 1, single_print: true,  brackets: [6] },
        { price: 96,    tpo_count: 1, single_print: true,  brackets: [12] },
    ],
};

test('levelTier classifies POC / value / single / normal correctly', () => {
    expect(levelTier(sampleReport.levels[0], sampleReport)).toBe('poc');
    expect(levelTier(sampleReport.levels[1], sampleReport)).toBe('value');
    expect(levelTier(sampleReport.levels[2], sampleReport)).toBe('value');
    expect(levelTier(sampleReport.levels[3], sampleReport)).toBe('single');
    expect(levelTier(sampleReport.levels[4], sampleReport)).toBe('single');
});

test('levelTier falls through to normal for level outside value area without single print', () => {
    const outside = { price: 95, tpo_count: 2, single_print: false, brackets: [10, 11] };
    expect(levelTier(outside, sampleReport)).toBe('normal');
});

test('levelTier handles null inputs', () => {
    expect(levelTier(null, sampleReport)).toBe('normal');
    expect(levelTier(sampleReport.levels[0], null)).toBe('normal');
});

test('levelLetters joins bracket indices as letters', () => {
    expect(levelLetters({ brackets: [0, 1, 7, 26] })).toBe('ABHa');
});

test('levelLetters returns empty string for malformed level', () => {
    expect(levelLetters(null)).toBe('');
    expect(levelLetters({ brackets: null })).toBe('');
});

// ── tierCounts ────────────────────────────────────────────────────

test('tierCounts buckets levels by tier', () => {
    const c = tierCounts(sampleReport);
    expect(c.poc).toBe(1);
    expect(c.value).toBe(2);
    expect(c.single).toBe(2);
    expect(c.normal).toBe(0);
});

test('tierCounts returns zeros on null report', () => {
    expect(tierCounts(null)).toEqual({ poc: 0, value: 0, single: 0, normal: 0 });
});

// ── makeDemoBrackets ──────────────────────────────────────────────

test('makeDemoBrackets returns 13 brackets indexed 0-12', () => {
    const b = makeDemoBrackets();
    expect(b.length).toBe(13);
    expect(b.map(x => x.bracket_index)).toEqual([0,1,2,3,4,5,6,7,8,9,10,11,12]);
});

test('makeDemoBrackets has high > low for every bracket', () => {
    const b = makeDemoBrackets();
    expect(b.every(x => x.high >= x.low && x.high > 0 && x.low > 0)).toBe(true);
});

// ── formatters ────────────────────────────────────────────────────

test('fmtN handles non-finite + digit override', () => {
    expect(fmtN(100.123)).toBe('100.12');
    expect(fmtN(100.123, 4)).toBe('100.1230');
    expect(fmtN(NaN)).toBe('—');
});

test('fmtInt locale-formats with no decimals', () => {
    expect(fmtInt(1234567)).toBe('1,234,567');
    expect(fmtInt(NaN)).toBe('—');
});
