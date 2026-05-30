// Global active-ticker store. Covers: get/set/persist round-trip,
// localStorage edge cases (private-browsing throws), event dispatch on
// change, no-op behavior on duplicate set, uppercase normalization.
//
// We stub `window` and `localStorage` per test so the module's lazy
// init re-runs cleanly. `_resetForTests` clears in-memory state.

import { test, expect, beforeEach } from 'vitest';
import {
    getGlobalSymbol, setGlobalSymbol,
    onGlobalSymbolChanged, _resetForTests,
} from '../js/_global_symbol.js';

beforeEach(() => {
    // Fresh stub per test so localStorage state and window listeners
    // don't bleed across runs.
    const store = new Map();
    globalThis.localStorage = {
        getItem(k)    { return store.has(k) ? store.get(k) : null; },
        setItem(k, v) { store.set(k, String(v)); },
        removeItem(k) { store.delete(k); },
        clear()       { store.clear(); },
    };
    const listeners = new Map();
    globalThis.window = {
        addEventListener(type, fn)    {
            if (!listeners.has(type)) listeners.set(type, new Set());
            listeners.get(type).add(fn);
        },
        removeEventListener(type, fn) {
            if (listeners.has(type)) listeners.get(type).delete(fn);
        },
        dispatchEvent(ev) {
            const set = listeners.get(ev.type);
            if (set) for (const fn of set) fn(ev);
            return true;
        },
    };
    globalThis.CustomEvent = function CustomEvent(type, init) {
        return { type, detail: init?.detail };
    };
    _resetForTests();
});

// ── getGlobalSymbol ─────────────────────────────────────────────────

test('initial value is empty string', () => {
    expect(getGlobalSymbol()).toBe('');
});

test('lazy-loads from localStorage', () => {
    globalThis.localStorage.setItem('tv-global-symbol', 'NVDA');
    _resetForTests();
    globalThis.localStorage.setItem('tv-global-symbol', 'NVDA');    // re-set after reset
    expect(getGlobalSymbol()).toBe('NVDA');
});

// ── setGlobalSymbol ─────────────────────────────────────────────────

test('setGlobalSymbol updates the in-memory value', () => {
    setGlobalSymbol('AAPL');
    expect(getGlobalSymbol()).toBe('AAPL');
});

test('setGlobalSymbol uppercases input', () => {
    setGlobalSymbol('aapl');
    expect(getGlobalSymbol()).toBe('AAPL');
});

test('setGlobalSymbol writes to localStorage', () => {
    setGlobalSymbol('TSLA');
    expect(globalThis.localStorage.getItem('tv-global-symbol')).toBe('TSLA');
});

test('setGlobalSymbol returns true when value changes, false otherwise', () => {
    expect(setGlobalSymbol('META')).toBe(true);
    expect(setGlobalSymbol('META')).toBe(false);
    expect(setGlobalSymbol('meta')).toBe(false);    // already 'META' (uppercase)
    expect(setGlobalSymbol('GOOG')).toBe(true);
});

test('setGlobalSymbol no-ops on empty / non-string input', () => {
    expect(setGlobalSymbol('')).toBe(false);
    expect(setGlobalSymbol(null)).toBe(false);
    expect(setGlobalSymbol(undefined)).toBe(false);
    expect(setGlobalSymbol(42)).toBe(false);
    expect(getGlobalSymbol()).toBe('');
});

// ── onGlobalSymbolChanged ───────────────────────────────────────────

test('subscribers receive new symbol on change', () => {
    let last = null;
    onGlobalSymbolChanged(sym => { last = sym; });
    setGlobalSymbol('NVDA');
    expect(last).toBe('NVDA');
});

test('subscribers NOT called when value is unchanged', () => {
    setGlobalSymbol('NVDA');
    let calls = 0;
    onGlobalSymbolChanged(() => { calls++; });
    setGlobalSymbol('NVDA');         // no-op
    setGlobalSymbol('nvda');         // case-only — also no-op
    expect(calls).toBe(0);
});

test('multiple subscribers all fire on change', () => {
    let a = null, b = null;
    onGlobalSymbolChanged(s => { a = s; });
    onGlobalSymbolChanged(s => { b = s; });
    setGlobalSymbol('PLTR');
    expect(a).toBe('PLTR');
    expect(b).toBe('PLTR');
});

test('unsubscribe stops further notifications', () => {
    let last = null;
    const off = onGlobalSymbolChanged(s => { last = s; });
    setGlobalSymbol('AAPL');
    expect(last).toBe('AAPL');
    off();
    setGlobalSymbol('NVDA');
    expect(last).toBe('AAPL');       // unchanged after unsubscribe
});

test('onGlobalSymbolChanged tolerates non-function handler', () => {
    const off = onGlobalSymbolChanged(null);
    expect(typeof off).toBe('function');
    off();    // no-op
});

// ── localStorage failure isolation ──────────────────────────────────

test('setGlobalSymbol still works when localStorage.setItem throws', () => {
    globalThis.localStorage.setItem = () => { throw new Error('private-mode quota'); };
    expect(() => setGlobalSymbol('SAFE')).not.toThrow();
    expect(getGlobalSymbol()).toBe('SAFE');
});

test('getGlobalSymbol survives a localStorage.getItem throw', () => {
    globalThis.localStorage.getItem = () => { throw new Error('private-mode read'); };
    _resetForTests();
    // Re-stub so reset doesn't itself blow up.
    globalThis.localStorage.removeItem = () => {};
    globalThis.localStorage.getItem = () => { throw new Error('private-mode read'); };
    expect(() => getGlobalSymbol()).not.toThrow();
});

// ── additional edge cases ─────────────────────────────────────────

test('whitespace-only string is NOT empty (passes length check), uppercased + stored', () => {
    // The guard is `sym.length === 0`, so "   " passes. Uppercase trims nothing.
    // This captures current behavior — pin so future trim improvement updates test.
    expect(setGlobalSymbol('   ')).toBe(true);
    expect(getGlobalSymbol()).toBe('   ');
});

test('CustomEvent detail.symbol matches uppercased value', () => {
    let captured = null;
    globalThis.window.addEventListener('tv-symbol-changed', (e) => { captured = e.detail; });
    setGlobalSymbol('aapl');
    expect(captured).toEqual({ symbol: 'AAPL' });
});

test('a listener throwing does not stop the value update', () => {
    onGlobalSymbolChanged(() => { throw new Error('listener-broke'); });
    // setGlobalSymbol wraps dispatchEvent in try/catch, so the throw is swallowed.
    expect(() => setGlobalSymbol('SAFE')).not.toThrow();
    expect(getGlobalSymbol()).toBe('SAFE');
});

test('two subscriptions can be unsubscribed independently', () => {
    const calls = { a: 0, b: 0 };
    const offA = onGlobalSymbolChanged(() => { calls.a++; });
    const offB = onGlobalSymbolChanged(() => { calls.b++; });
    setGlobalSymbol('AAPL');
    expect(calls).toEqual({ a: 1, b: 1 });
    offA();
    setGlobalSymbol('NVDA');
    expect(calls).toEqual({ a: 1, b: 2 });
    offB();
    setGlobalSymbol('META');
    expect(calls).toEqual({ a: 1, b: 2 });  // both unsubscribed
});

test('empty stored value does not propagate as current symbol', () => {
    globalThis.localStorage.setItem('tv-global-symbol', '');
    _resetForTests();
    globalThis.localStorage.setItem('tv-global-symbol', '');
    expect(getGlobalSymbol()).toBe('');
});

test('setGlobalSymbol does not write to storage when value unchanged', () => {
    setGlobalSymbol('AAPL');
    let writes = 0;
    const orig = globalThis.localStorage.setItem;
    globalThis.localStorage.setItem = (k, v) => { writes++; orig.call(globalThis.localStorage, k, v); };
    setGlobalSymbol('AAPL');     // unchanged
    setGlobalSymbol('aapl');     // case-only — still unchanged
    expect(writes).toBe(0);
});

test('listener registered AFTER a value-set still fires on next change', () => {
    setGlobalSymbol('AAPL');
    let captured = null;
    onGlobalSymbolChanged(s => { captured = s; });
    setGlobalSymbol('NVDA');
    expect(captured).toBe('NVDA');
});

test('CustomEvent type is exactly "tv-symbol-changed"', () => {
    // Pin event name so consumers wiring window.addEventListener don't break silently.
    let evType = null;
    globalThis.window.addEventListener('tv-symbol-changed', (e) => { evType = e.type; });
    setGlobalSymbol('SPY');
    expect(evType).toBe('tv-symbol-changed');
});
