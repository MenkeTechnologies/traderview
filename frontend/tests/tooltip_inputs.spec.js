// Tooltip pure helpers: tipAttrsFor, tipSelectors, tipKey.

import { test, expect } from 'vitest';
import { tipAttrsFor, tipSelectors, tipKey, shortcutId, composeTooltip } from '../js/_tooltip.js';

test('tipAttrsFor: emits data-i18n-title attribute pair', () => {
    expect(tipAttrsFor('topbar.tip.palette')).toEqual({
        'data-i18n-title': 'topbar.tip.palette',
    });
});

test('tipAttrsFor: empty / non-string → null', () => {
    expect(tipAttrsFor('')).toBeNull();
    expect(tipAttrsFor(null)).toBeNull();
    expect(tipAttrsFor(42)).toBeNull();
});

test('tipSelectors: covers data-tip / data-tooltip / data-tip-key', () => {
    expect(tipSelectors()).toEqual(['[data-tip]', '[data-tooltip]', '[data-tip-key]']);
});

test('tipKey: pulls from each supported attribute name', () => {
    expect(tipKey({ dataset: { tip: 'k1' } })).toBe('k1');
    expect(tipKey({ dataset: { tooltip: 'k2' } })).toBe('k2');
    expect(tipKey({ dataset: { tipKey: 'k3' } })).toBe('k3');
});

test('tipKey: first-defined wins (tip > tooltip > tipKey)', () => {
    expect(tipKey({ dataset: { tip: 'a', tooltip: 'b', tipKey: 'c' } })).toBe('a');
});

test('tipKey: null on missing element / dataset', () => {
    expect(tipKey(null)).toBeNull();
    expect(tipKey({})).toBeNull();
    expect(tipKey({ dataset: {} })).toBeNull();
});

// ── shortcutId ─────────────────────────────────────────────────────

test('shortcutId: pulls from data-shortcut', () => {
    expect(shortcutId({ dataset: { shortcut: 'command_palette' } })).toBe('command_palette');
});

test('shortcutId: null on missing element / dataset / attr', () => {
    expect(shortcutId(null)).toBeNull();
    expect(shortcutId({})).toBeNull();
    expect(shortcutId({ dataset: {} })).toBeNull();
});

// ── composeTooltip ─────────────────────────────────────────────────

test('composeTooltip: tip + chip joined with two spaces and parens', () => {
    expect(composeTooltip('Open palette', '⌘K')).toBe('Open palette  (⌘K)');
});

test('composeTooltip: trims surrounding whitespace on both sides', () => {
    expect(composeTooltip('  Open palette  ', '  ⌘K  ')).toBe('Open palette  (⌘K)');
});

test('composeTooltip: tip alone → tip', () => {
    expect(composeTooltip('Open palette', '')).toBe('Open palette');
    expect(composeTooltip('Open palette', null)).toBe('Open palette');
});

test('composeTooltip: chip alone → chip', () => {
    expect(composeTooltip('', '⌘K')).toBe('⌘K');
    expect(composeTooltip(null, '⌘K')).toBe('⌘K');
});

test('composeTooltip: empty both → empty string', () => {
    expect(composeTooltip('', '')).toBe('');
    expect(composeTooltip(null, null)).toBe('');
});
