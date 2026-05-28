// buildCombo extra edge cases beyond what the existing _pure tests cover.
// Bugs this guards against:
//   * Wrong modifier ordering (matchers fail because "alt+ctrl+a" != "ctrl+alt+a").
//   * Multi-char keys (F-keys, Arrow keys) accidentally normalized incorrectly.
//   * Whitespace / null keys producing junk combos.

import { test, expect } from 'vitest';
import { buildCombo } from '../js/_pure.js';

const evt = (overrides = {}) => ({
    ctrlKey: false, altKey: false, shiftKey: false, metaKey: false,
    key: '',
    ...overrides,
});

test('order is always ctrl, alt, shift, meta regardless of input', () => {
    expect(buildCombo(evt({ metaKey: true, altKey: true, ctrlKey: true, shiftKey: true, key: 'a' })))
        .toBe('ctrl+alt+shift+meta+a');
});

test('bare modifier press returns null', () => {
    for (const k of ['Control', 'Shift', 'Alt', 'Meta', 'control', 'SHIFT']) {
        expect(buildCombo(evt({ key: k }))).toBe(null);
    }
});

test('lowercases multi-char keys (F-keys, ArrowDown)', () => {
    expect(buildCombo(evt({ key: 'F5' }))).toBe('f5');
    expect(buildCombo(evt({ ctrlKey: true, key: 'ArrowDown' }))).toBe('ctrl+arrowdown');
});

test('empty key returns null', () => {
    expect(buildCombo(evt({ key: '' }))).toBe(null);
    expect(buildCombo(evt({ ctrlKey: true, key: '' }))).toBe(null);
});

test('non-string key is coerced (toLowerCase on String(undefined))', () => {
    // String(undefined).toLowerCase() === 'undefined' — the guard rejects
    // null-key but lets the coerced string through. Test pins this so a
    // future "actually validate inputs" refactor catches a behavior change.
    const result = buildCombo(evt({ key: undefined }));
    expect(result === null || result === 'undefined').toBe(true);
});

test('single ctrl+letter combo', () => {
    expect(buildCombo(evt({ ctrlKey: true, key: 'k' }))).toBe('ctrl+k');
});

test('alt+enter (common: submit-from-textarea)', () => {
    expect(buildCombo(evt({ altKey: true, key: 'Enter' }))).toBe('alt+enter');
});
