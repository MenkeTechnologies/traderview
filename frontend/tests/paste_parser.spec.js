// Shared paste-parser spec. The 4 view-specific parsers
// (parseReturns / parseSeries / parseVolumeCurve) all delegate here, so
// covering this once covers all of them. Existing per-view specs still
// run against their thin wrappers as integration tests.

import { test, expect } from 'vitest';
import { parseFloatBlob } from '../js/_paste_parser.js';

test('parses one number per line', () => {
    expect(parseFloatBlob('1\n2\n3').value).toEqual([1, 2, 3]);
});

test('parses multiple numbers per line, mixed delimiters', () => {
    expect(parseFloatBlob('1, 2 3\n4').value).toEqual([1, 2, 3, 4]);
});

test('skips blank lines and # comments', () => {
    expect(parseFloatBlob('# header\n\n1\n# inline\n2').value).toEqual([1, 2]);
});

test('non-numeric tokens are line-anchored errors', () => {
    const r = parseFloatBlob('1\nbad\n2');
    expect(r.value).toEqual([1, 2]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].line_no).toBe(2);
    expect(r.errors[0].raw).toBe('bad');
    expect(r.errors[0].message).toMatch(/non-numeric/);
});

test('mixed-bad lines report one error per bad token', () => {
    const r = parseFloatBlob('1 oops 2 nope');
    expect(r.value).toEqual([1, 2]);
    expect(r.errors.length).toBe(2);
    expect(r.errors.every(e => e.line_no === 1)).toBe(true);
});

test('non-string input returns single descriptive error', () => {
    const r = parseFloatBlob(null);
    expect(r.value).toEqual([]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/not a string/);
});

test('nonNegative option flags negative values', () => {
    const r = parseFloatBlob('100\n-50\n200', { nonNegative: true });
    expect(r.value).toEqual([100, 200]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/negative/);
});

test('nonNegative option allows zero', () => {
    const r = parseFloatBlob('0\n100', { nonNegative: true });
    expect(r.value).toEqual([0, 100]);
    expect(r.errors).toEqual([]);
});

test('without nonNegative, negative numbers are accepted', () => {
    expect(parseFloatBlob('-1\n2').value).toEqual([-1, 2]);
});

test('empty input returns empty value + no errors', () => {
    expect(parseFloatBlob('')).toEqual({ value: [], errors: [] });
});

test('scientific notation is accepted', () => {
    expect(parseFloatBlob('1e-3\n2.5e2').value).toEqual([0.001, 250]);
});

test('Infinity and NaN are rejected as non-numeric', () => {
    const r = parseFloatBlob('1\nNaN\nInfinity\n2');
    expect(r.value).toEqual([1, 2]);
    expect(r.errors.length).toBe(2);
});
