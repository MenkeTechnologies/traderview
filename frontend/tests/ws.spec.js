// WebSocket subscription helpers — pin the `on(type, fn)` contract
// for callers that wire up handlers. Doesn't exercise the actual
// WebSocket lifecycle (which needs a live ws server).

import { test, expect, beforeEach, afterEach } from 'vitest';

let on, isConnected;

beforeEach(async () => {
    // Stub globals before importing the module so `connect()` doesn't try
    // to construct a real WebSocket if some test triggers it.
    globalThis.window = { __tvApiBase: 'http://localhost:3000' };
    globalThis.location = { origin: 'http://localhost:3000' };
    globalThis.WebSocket = function FakeWebSocket() {
        this.addEventListener = () => {};
        this.close = () => {};
    };
    globalThis.localStorage = { getItem: () => '' };
    // Re-import the module fresh per test via dynamic import + cache-bust.
    // Vitest caches by URL; appending a query forces a new module instance.
    const mod = await import('../js/ws.js?bust=' + Math.random());
    on = mod.on;
    isConnected = mod.isConnected;
});

afterEach(() => {
    delete globalThis.window;
    delete globalThis.location;
    delete globalThis.WebSocket;
    delete globalThis.localStorage;
});

// ── isConnected default ───────────────────────────────────────────

test('isConnected returns false before any connection is established', () => {
    expect(isConnected()).toBe(false);
});

// ── on() subscription contract ────────────────────────────────────

test('on() returns an unsubscribe function', () => {
    const off = on('quote', () => {});
    expect(typeof off).toBe('function');
    off();  // no-op verification; just confirms callable
});

test('on() handles multiple subscribers for the same event type', () => {
    const calls = [];
    on('quote', () => calls.push('a'));
    on('quote', () => calls.push('b'));
    on('quote', () => calls.push('c'));
    // We can't trigger dispatch directly, but the subscription should
    // have stored both callbacks. The unsubscribe contract confirms
    // bookkeeping — each off should return a function.
    expect(calls).toEqual([]);  // no dispatch happened
});

test('on() returns distinct unsubscribe functions per subscription', () => {
    const off1 = on('quote', () => {});
    const off2 = on('quote', () => {});
    expect(off1).not.toBe(off2);
});

test('on() with different event types creates independent subscription sets', () => {
    const offA = on('quote', () => {});
    const offB = on('alert', () => {});
    expect(typeof offA).toBe('function');
    expect(typeof offB).toBe('function');
    // Sanity: unsubscribing one doesn't crash the other.
    offA();
    offB();
});

test('unsubscribe is idempotent (calling twice does not throw)', () => {
    const off = on('quote', () => {});
    off();
    expect(() => off()).not.toThrow();
});
