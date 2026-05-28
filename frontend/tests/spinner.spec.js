// Spinner HTML generators. The DOM-side helpers (withSpinner, buttonSpinner,
// spinnerOverlay) need a live DOM and are exercised in views; here we cover
// the pure-string surface (spinnerHTML + its escape behavior).
//
// Bugs this guards against:
//   * XSS via unescaped status text reaching innerHTML.
//   * Stripping the aria-label attribute (accessibility regression).
//   * Default text changing without docs updating.

import { test, expect } from 'vitest';
import { spinnerHTML } from '../js/spinner.js';

test('spinnerHTML default text is "loading…"', () => {
    const s = spinnerHTML();
    expect(s).toContain('loading…');
});

test('spinnerHTML wraps custom text in the visible label', () => {
    const s = spinnerHTML('Crunching ticks');
    expect(s).toContain('Crunching ticks');
});

test('spinnerHTML escapes < and > to prevent injection', () => {
    const s = spinnerHTML('<script>alert(1)</script>');
    expect(s).not.toContain('<script>');
    expect(s).toContain('&lt;script&gt;');
});

test('spinnerHTML escapes & before other entities', () => {
    const s = spinnerHTML('R&D');
    expect(s).toContain('R&amp;D');
});

test('spinnerHTML preserves role=status for screen readers', () => {
    const s = spinnerHTML('loading');
    expect(s).toContain('role="status"');
});

test('spinnerHTML emits a top-level tv-spinner-wrap container', () => {
    const s = spinnerHTML();
    expect(s.trim().startsWith('<div class="tv-spinner-wrap">')).toBe(true);
});

test('spinnerHTML aria-label uses the RAW (un-escaped) text', () => {
    // Note: this captures the current behavior. If we ever escape the
    // aria-label too (defense-in-depth), update the test — but right now
    // the label inside the attribute uses the raw string, which means
    // text with quotes will break attribute parsing. The visible label
    // (inside the wrap div) IS escaped.
    const s = spinnerHTML('simple-text');
    expect(s).toContain('aria-label="simple-text"');
});
