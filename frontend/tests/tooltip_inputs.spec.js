// Tooltip pure helpers: tipAttrsFor, tipSelectors, tipKey.

import { test, expect } from 'vitest';
import {
    tipAttrsFor, tipSelectors, tipKey, shortcutId, composeTooltip,
    interactiveSelectors, normalizeTitle, deriveAutoTitle, shouldAutoTitle,
} from '../js/_tooltip.js';

// Minimal fake element factory — covers what _tooltip.js touches without
// pulling in jsdom.
function makeEl(spec = {}) {
    const attrs = { ...(spec.attrs || {}) };
    const ds    = { ...(spec.dataset || {}) };
    return {
        nodeType: 1,
        tagName: (spec.tag || 'button').toUpperCase(),
        textContent: spec.textContent || '',
        isContentEditable: !!spec.contentEditable,
        dataset: ds,
        getAttribute(k) { return Object.prototype.hasOwnProperty.call(attrs, k) ? attrs[k] : null; },
        setAttribute(k, v) { attrs[k] = String(v); },
        hasAttribute(k) { return Object.prototype.hasOwnProperty.call(attrs, k); },
    };
}

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

// ── interactiveSelectors ─────────────────────────────────────────

test('interactiveSelectors: covers every interactive role used in the app', () => {
    const sel = interactiveSelectors();
    for (const want of [
        'button', 'a[href]',
        '[role="button"]', '[role="tab"]', '[role="menuitem"]',
        '[role="menuitemcheckbox"]', '[role="menuitemradio"]',
        '[role="option"]', '[role="switch"]', '[role="checkbox"]',
        'summary', 'select',
        'input[type="checkbox"]', 'input[type="radio"]',
        'input[type="search"]', 'input[type="email"]', 'input[type="tel"]',
        'input[type="url"]', 'input[type="date"]', 'input[type="time"]',
        'input[type="range"]', 'input[type="color"]', 'input[type="file"]',
        'label[for]',
        '[onclick]',
        '[tabindex]:not([tabindex="-1"])',
    ]) {
        expect(sel.includes(want)).toBe(true);
    }
});

test('interactiveSelectors: excludes tabindex="-1" (programmatic focus only)', () => {
    const sel = interactiveSelectors();
    expect(sel.includes('[tabindex]:not([tabindex="-1"])')).toBe(true);
    // sanity: the raw [tabindex] alone (without the :not exclusion) shouldn't appear
    expect(sel.split(',').some(s => s.trim() === '[tabindex]')).toBe(false);
});

// ── normalizeTitle ───────────────────────────────────────────────

test('normalizeTitle: collapses whitespace + trims', () => {
    expect(normalizeTitle('  Save  trade  ')).toBe('Save trade');
    expect(normalizeTitle('a\n\tb')).toBe('a b');
});

test('normalizeTitle: truncates past max with ellipsis', () => {
    const long = 'x'.repeat(120);
    const out  = normalizeTitle(long, 20);
    expect(out.length).toBe(20);
    expect(out.endsWith('…')).toBe(true);
});

test('normalizeTitle: null / undefined → empty', () => {
    expect(normalizeTitle(null)).toBe('');
    expect(normalizeTitle(undefined)).toBe('');
});

// ── deriveAutoTitle ──────────────────────────────────────────────

test('deriveAutoTitle: existing title wins', () => {
    const el = makeEl({ attrs: { title: 'Hi' }, textContent: 'fallback' });
    expect(deriveAutoTitle(el)).toBe('Hi');
});

test('deriveAutoTitle: aria-label beats text', () => {
    const el = makeEl({ attrs: { 'aria-label': 'Close' }, textContent: '✕' });
    expect(deriveAutoTitle(el)).toBe('Close');
});

test('deriveAutoTitle: placeholder used on inputs with no text', () => {
    const el = makeEl({ tag: 'input', attrs: { placeholder: 'Symbol' } });
    expect(deriveAutoTitle(el)).toBe('Symbol');
});

test('deriveAutoTitle: falls back to text content', () => {
    const el = makeEl({ textContent: '  Save trade ' });
    expect(deriveAutoTitle(el)).toBe('Save trade');
});

test('deriveAutoTitle: aria-labelledby resolves via callback', () => {
    const el = makeEl({ attrs: { 'aria-labelledby': 'l1' } });
    const labels = { l1: { textContent: 'Volume' } };
    expect(deriveAutoTitle(el, id => labels[id] || null)).toBe('Volume');
});

test('deriveAutoTitle: nothing meaningful → empty', () => {
    const el = makeEl({});
    expect(deriveAutoTitle(el)).toBe('');
});

test('deriveAutoTitle: null safe', () => {
    expect(deriveAutoTitle(null)).toBe('');
});

// ── shouldAutoTitle ──────────────────────────────────────────────

test('shouldAutoTitle: skips when title already set', () => {
    expect(shouldAutoTitle(makeEl({ attrs: { title: 'x' } }))).toBe(false);
});

test('shouldAutoTitle: skips when data-i18n-title already set', () => {
    expect(shouldAutoTitle(makeEl({ attrs: { 'data-i18n-title': 'k' } }))).toBe(false);
});

test('shouldAutoTitle: skips when data-tip set (i18n path handles it)', () => {
    expect(shouldAutoTitle(makeEl({ dataset: { tip: 'a.b' } }))).toBe(false);
    expect(shouldAutoTitle(makeEl({ dataset: { tooltip: 'a.b' } }))).toBe(false);
    expect(shouldAutoTitle(makeEl({ dataset: { tipKey: 'a.b' } }))).toBe(false);
});

test('shouldAutoTitle: skips opt-out via data-no-tip', () => {
    expect(shouldAutoTitle(makeEl({ dataset: { noTip: '1' } }))).toBe(false);
    expect(shouldAutoTitle(makeEl({ dataset: { noTip: 'true' } }))).toBe(false);
});

test('shouldAutoTitle: skips when already auto-titled (idempotency)', () => {
    expect(shouldAutoTitle(makeEl({ dataset: { autoTitle: '1' } }))).toBe(false);
});

test('shouldAutoTitle: accepts a bare button', () => {
    expect(shouldAutoTitle(makeEl({}))).toBe(true);
});

test('shouldAutoTitle: null safe', () => {
    expect(shouldAutoTitle(null)).toBe(false);
});
