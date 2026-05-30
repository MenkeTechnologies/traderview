// Toast runtime tests for showToast + tShowToast. The DOM-touching
// surface that toast_inputs.spec.js doesn't cover. Uses a fake DOM
// + fake `setTimeout` so the test doesn't pull jsdom.

import { test, expect, beforeEach, afterEach } from 'vitest';
import { showToast, tShowToast } from '../js/toast.js';
import { setMap } from '../js/i18n.js';

let mountedRoot;
let _timeoutHandles;

beforeEach(() => {
    _timeoutHandles = [];
    // Fake DOM: minimal root that records appendChild / removeChild.
    let id = 0;
    const makeEl = (tag) => {
        const el = {
            id: `el-${++id}`, tagName: tag.toUpperCase(),
            className: '', textContent: '', innerHTML: '',
            style: {}, dataset: {},
            children: [], parentNode: null,
            listeners: {},
            appendChild(c) { c.parentNode = this; this.children.push(c); return c; },
            removeChild(c) { c.parentNode = null; this.children = this.children.filter(x => x !== c); },
            addEventListener(type, fn) { (this.listeners[type] ||= []).push(fn); },
            removeEventListener(type, fn) {
                if (this.listeners[type]) this.listeners[type] = this.listeners[type].filter(x => x !== fn);
            },
            setAttribute(k, v) { this[k] = v; },
            getAttribute(k) { return this[k]; },
        };
        return el;
    };
    const body = makeEl('body');
    mountedRoot = null;
    globalThis.window = {
        dispatchEvent() { return true; },
        addEventListener() {},
        removeEventListener() {},
    };
    globalThis.CustomEvent = function CustomEvent(type, init) {
        return { type, detail: init?.detail };
    };
    globalThis.document = {
        body,
        getElementById(id) {
            // Search body.children recursively.
            const search = (node) => {
                if (node.id === id) return node;
                for (const c of node.children) {
                    const r = search(c);
                    if (r) return r;
                }
                return null;
            };
            return search(body) || (mountedRoot && mountedRoot.id === id ? mountedRoot : null);
        },
        createElement(tag) { return makeEl(tag); },
    };
    // Patch setTimeout so we can run pending timers manually.
    globalThis.setTimeout = (fn, _ms) => {
        const handle = { fn, cleared: false };
        _timeoutHandles.push(handle);
        return handle;
    };
    globalThis.clearTimeout = (handle) => { if (handle) handle.cleared = true; };
});

afterEach(() => {
    delete globalThis.window;
    delete globalThis.document;
    setMap({});
});

function runTimers() {
    for (const h of _timeoutHandles) {
        if (!h.cleared) h.fn();
    }
    _timeoutHandles = [];
}

// ── showToast core behavior ────────────────────────────────────────

test('showToast returns null without window/document', () => {
    delete globalThis.window;
    expect(showToast('hi')).toBeNull();
});

test('showToast creates a toast div under the mount root', () => {
    const el = showToast('hello');
    expect(el).not.toBeNull();
    expect(el.textContent).toContain('hello');
    expect(el.className).toContain('tv-toast');
});

test('showToast prefixes the message with the level icon', () => {
    const el = showToast('done', { level: 'success' });
    // iconFor('success') is '✓'.
    expect(el.textContent.startsWith('✓ ')).toBe(true);
});

test('showToast with invalid level returns null and logs warning', () => {
    const origWarn = console.warn;
    let warned = null;
    console.warn = (...args) => { warned = args.join(' '); };
    try {
        const r = showToast('hi', { level: 'fatal' });
        expect(r).toBeNull();
        expect(warned).toMatch(/invalid/);
    } finally {
        console.warn = origWarn;
    }
});

test('showToast applies extraClass when valid', () => {
    const el = showToast('hi', { extraClass: 'tv-toast-pin' });
    expect(el.className).toContain('tv-toast-pin');
});

// ── coalescing ────────────────────────────────────────────────────

test('showToast coalesces same message+level: only one DOM element exists', () => {
    const a = showToast('clip denied', { level: 'error' });
    const b = showToast('clip denied', { level: 'error' });
    expect(b).toBe(a);  // returns the existing element
    // Mount root should have only one child.
    const root = globalThis.document.getElementById('tv-toast-root');
    expect(root.children.length).toBe(1);
});

test('showToast: different level → separate toast element', () => {
    const a = showToast('msg', { level: 'info' });
    const b = showToast('msg', { level: 'error' });
    expect(b).not.toBe(a);
    const root = globalThis.document.getElementById('tv-toast-root');
    expect(root.children.length).toBe(2);
});

// ── dismissal ─────────────────────────────────────────────────────

test('clicking the toast dismisses it (removes from DOM)', () => {
    const el = showToast('clickable');
    const root = globalThis.document.getElementById('tv-toast-root');
    expect(root.children.length).toBe(1);
    // Simulate click.
    el.listeners.click[0]();
    expect(root.children.length).toBe(0);
});

test('expiration via timer fires dismiss', () => {
    const el = showToast('expires');
    const root = globalThis.document.getElementById('tv-toast-root');
    expect(root.children.length).toBe(1);
    runTimers();
    expect(root.children.length).toBe(0);
});

test('coalesced re-show resets the timer (prior pending dismiss is cleared)', () => {
    const a = showToast('hello');
    const firstTimer = a._timeout;
    expect(firstTimer.cleared).toBe(false);
    showToast('hello');  // coalesce
    expect(firstTimer.cleared).toBe(true);
});

// ── tShowToast wraps t() ──────────────────────────────────────────

test('tShowToast looks up key via t() then forwards to showToast', () => {
    setMap({ 'toast.hello': 'Hello {name}' });
    const el = tShowToast('toast.hello', { name: 'Jake' });
    expect(el.textContent).toContain('Hello Jake');
});

test('tShowToast missing key falls through (key returned by t())', () => {
    setMap({});
    const el = tShowToast('toast.missing');
    expect(el.textContent).toContain('toast.missing');
});

test('tShowToast with no params still works', () => {
    setMap({ 'toast.plain': 'plain text' });
    const el = tShowToast('toast.plain');
    expect(el.textContent).toContain('plain text');
});

test('tShowToast forwards opts to showToast (level applied)', () => {
    setMap({ 'toast.k': 'k-text' });
    const el = tShowToast('toast.k', {}, { level: 'error' });
    expect(el.textContent.startsWith('✕')).toBe(true);  // iconFor('error')
});
