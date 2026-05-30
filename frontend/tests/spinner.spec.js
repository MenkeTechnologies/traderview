// Spinner HTML generators. The DOM-side helpers (withSpinner, buttonSpinner,
// spinnerOverlay) need a live DOM and are exercised in views; here we cover
// the pure-string surface (spinnerHTML + its escape behavior).
//
// Bugs this guards against:
//   * XSS via unescaped status text reaching innerHTML.
//   * Stripping the aria-label attribute (accessibility regression).
//   * Default text changing without docs updating.

import { test, expect } from 'vitest';
import {
    spinnerHTML, withSpinner, spinnerOverlay, buttonSpinner,
} from '../js/spinner.js';

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

test('spinnerHTML aria-label uses the escaped text (preserves XSS guard)', () => {
    const s = spinnerHTML('R&D <script>');
    // Same `safe` value used for aria-label as visible label.
    expect(s).toContain('aria-label="R&amp;D &lt;script&gt;"');
});

// ── withSpinner ───────────────────────────────────────────────────

test('withSpinner: null element passes promise through unchanged', async () => {
    const p = Promise.resolve('result');
    expect(await withSpinner(null, p)).toBe('result');
});

test('withSpinner: replaces innerHTML on entry, leaves caller to render on success', async () => {
    const el = { innerHTML: 'original' };
    const r = await withSpinner(el, Promise.resolve(42));
    expect(r).toBe(42);
    // innerHTML still shows spinner; caller is expected to render after.
    expect(el.innerHTML).toContain('tv-spinner-wrap');
});

test('withSpinner: restores prior innerHTML when promise rejects + rethrows', async () => {
    const el = { innerHTML: 'PRIOR' };
    await expect(withSpinner(el, Promise.reject(new Error('boom'))))
        .rejects.toThrow('boom');
    expect(el.innerHTML).toBe('PRIOR');
});

// ── spinnerOverlay ────────────────────────────────────────────────

test('spinnerOverlay: null element returns noop dispose', () => {
    const dispose = spinnerOverlay(null);
    expect(typeof dispose).toBe('function');
    expect(() => dispose()).not.toThrow();
});

test('spinnerOverlay: appends child overlay; dispose removes it + restores position', () => {
    const removed = [];
    const overlay = { remove() { removed.push(this); } };
    const el = {
        style: { position: 'static' },
        appendChild(child) { Object.assign(child, overlay); },
    };
    // Stub document.createElement so spinnerOverlay can build the overlay.
    const origDoc = globalThis.document;
    globalThis.document = {
        createElement() {
            return { className: '', innerHTML: '', remove() { removed.push('overlay-removed'); } };
        },
    };
    try {
        const dispose = spinnerOverlay(el);
        expect(el.style.position).toBe('relative');  // promoted from static
        expect(typeof dispose).toBe('function');
        dispose();
        expect(el.style.position).toBe('static');     // restored
        expect(removed.length).toBeGreaterThan(0);    // overlay disposed
    } finally {
        globalThis.document = origDoc;
    }
});

test('spinnerOverlay: preserves non-static prior position', () => {
    const el = {
        style: { position: 'absolute' },
        appendChild() {},
    };
    const origDoc = globalThis.document;
    globalThis.document = {
        createElement() { return { className: '', innerHTML: '', remove() {} }; },
    };
    try {
        const dispose = spinnerOverlay(el);
        expect(el.style.position).toBe('absolute');  // NOT promoted
        dispose();
        expect(el.style.position).toBe('absolute');  // unchanged after dispose
    } finally {
        globalThis.document = origDoc;
    }
});

// ── buttonSpinner ─────────────────────────────────────────────────

test('buttonSpinner: null button passes promise through', async () => {
    const r = await buttonSpinner(null, Promise.resolve('done'));
    expect(r).toBe('done');
});

test('buttonSpinner: disables button + swaps label, restores both on success', async () => {
    const btn = { innerHTML: 'Save', disabled: false };
    const r = await buttonSpinner(btn, Promise.resolve('ok'));
    expect(r).toBe('ok');
    expect(btn.innerHTML).toBe('Save');          // restored
    expect(btn.disabled).toBe(false);            // restored
});

test('buttonSpinner: restores button state even when promise rejects', async () => {
    const btn = { innerHTML: 'Submit', disabled: false };
    await expect(buttonSpinner(btn, Promise.reject(new Error('nope'))))
        .rejects.toThrow('nope');
    expect(btn.innerHTML).toBe('Submit');
    expect(btn.disabled).toBe(false);
});

test('buttonSpinner: preserves prior disabled state on restore', async () => {
    const btn = { innerHTML: 'X', disabled: true };  // already disabled
    await buttonSpinner(btn, Promise.resolve());
    expect(btn.disabled).toBe(true);              // stays disabled
});
