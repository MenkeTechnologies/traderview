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
    expect(r.errors[0].message).toMatch(/must be a string/);
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

// ── edge cases ─────────────────────────────────────────────────────

test('CRLF line endings handled identically to LF', () => {
    expect(parseFloatBlob('1\r\n2\r\n3').value).toEqual([1, 2, 3]);
});

test('comment lines with leading whitespace before # are still skipped', () => {
    expect(parseFloatBlob('  # leading-space comment\n1').value).toEqual([1]);
});

test('inline # is NOT treated as comment (only line-leading #)', () => {
    // The check is `stripped.startsWith("#")`. Comments mid-line are parsed
    // as tokens — `#bad` would be a non-numeric token.
    const r = parseFloatBlob('1 #inline 2');
    expect(r.value).toEqual([1, 2]);
    expect(r.errors.length).toBe(1);
    expect(r.errors[0].message).toMatch(/#inline/);
});

test('mixed delimiters on one line: comma, space, tab', () => {
    expect(parseFloatBlob('1,2\t3 4').value).toEqual([1, 2, 3, 4]);
});

test('trailing delimiter (comma) does not produce a blank-token error', () => {
    // filter(Boolean) drops the empty trailing chunk after trim.
    expect(parseFloatBlob('1, 2,').value).toEqual([1, 2]);
});

test('line number reflects the source LINE, not the value index', () => {
    const r = parseFloatBlob('# c1\n# c2\nbad');
    expect(r.errors[0].line_no).toBe(3);
});

test('hex and octal literals: 0x10 is accepted by Number(), 0o7 too', () => {
    // Pins JS semantics for Number(): hex/octal/binary string parsing.
    expect(parseFloatBlob('0x10').value).toEqual([16]);
    expect(parseFloatBlob('0o7').value).toEqual([7]);
});

test('nonNegative option does NOT reject NaN tokens (caught earlier as non-numeric)', () => {
    const r = parseFloatBlob('NaN\n-1', { nonNegative: true });
    expect(r.value).toEqual([]);
    expect(r.errors.length).toBe(2);
    expect(r.errors[0].message).toMatch(/non-numeric/);
    expect(r.errors[1].message).toMatch(/negative/);
});

test('options object is optional — undefined opts behaves as defaults', () => {
    expect(parseFloatBlob('1 2 3', undefined).value).toEqual([1, 2, 3]);
});

test('purely-whitespace input returns empty value + empty errors', () => {
    expect(parseFloatBlob('   \n\t\n  ')).toEqual({ value: [], errors: [] });
});
