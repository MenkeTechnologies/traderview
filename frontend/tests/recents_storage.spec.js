// Recently-visited views tracker — pure helpers.

import { test, expect } from 'vitest';
import {
    STORAGE_KEY, SCHEMA_VERSION, MAX_RECENTS, SKIP_VIEWS,
    defaultState, migrate, loadState, saveState,
    push, listRecents, buildRecentItems, clearRecents,
} from '../js/_recents_storage.js';

// ── defaults ─────────────────────────────────────────────────────

test('defaultState: empty recents + correct version', () => {
    const s = defaultState();
    expect(s.version).toBe(SCHEMA_VERSION);
    expect(s.recents).toEqual([]);
});

test('constants: MAX_RECENTS = 10, STORAGE_KEY versioned', () => {
    expect(MAX_RECENTS).toBe(10);
    expect(STORAGE_KEY).toBe('tv-recents-v1');
});

test('SKIP_VIEWS: contains transient destinations', () => {
    expect(SKIP_VIEWS.has('launcher')).toBe(true);
    expect(SKIP_VIEWS.has('keyboard-shortcuts')).toBe(true);
    expect(SKIP_VIEWS.has('')).toBe(true);
});

// ── migrate ──────────────────────────────────────────────────────

test('migrate: null / non-object → defaultState', () => {
    expect(migrate(null)).toEqual(defaultState());
    expect(migrate(42)).toEqual(defaultState());
});

test('migrate: drops malformed entries', () => {
    const raw = { recents: [
        { viewId: 'good', at: 1000 },
        { viewId: 123,    at: 2000 },       // viewId wrong type
        { viewId: 'no-ts' },                // missing at
        { viewId: '',     at: 1000 },       // empty viewId
        null,
    ] };
    const m = migrate(raw);
    expect(m.recents).toEqual([{ viewId: 'good', at: 1000 }]);
});

test('migrate: caps at MAX_RECENTS', () => {
    const raw = { recents: Array.from({ length: MAX_RECENTS + 5 },
        (_, i) => ({ viewId: `v${i}`, at: i })) };
    expect(migrate(raw).recents.length).toBe(MAX_RECENTS);
});

// ── push ────────────────────────────────────────────────────────

test('push: appends new viewId at front with timestamp', () => {
    const s = push(defaultState(), 'charts', 1000);
    expect(s.recents).toEqual([{ viewId: 'charts', at: 1000 }]);
});

test('push: existing viewId → moved to front with fresh timestamp', () => {
    const s1 = push(defaultState(), 'charts',   1000);
    const s2 = push(s1,             'screener', 2000);
    const s3 = push(s2,             'charts',   3000);
    expect(s3.recents.length).toBe(2);
    expect(s3.recents[0]).toEqual({ viewId: 'charts',   at: 3000 });
    expect(s3.recents[1]).toEqual({ viewId: 'screener', at: 2000 });
});

test('push: caps at MAX_RECENTS', () => {
    let s = defaultState();
    for (let i = 0; i < MAX_RECENTS + 5; i++) s = push(s, `v${i}`, i);
    expect(s.recents.length).toBe(MAX_RECENTS);
    // Newest at front.
    expect(s.recents[0].viewId).toBe(`v${MAX_RECENTS + 4}`);
});

test('push: skip list ignored', () => {
    let s = defaultState();
    s = push(s, 'launcher',           1000);
    s = push(s, 'keyboard-shortcuts', 2000);
    s = push(s, '',                   3000);
    expect(s.recents).toEqual([]);
});

test('push: bad viewId safe', () => {
    expect(push(null, null).recents).toEqual([]);
    expect(push(undefined, 42).recents).toEqual([]);
    expect(push(defaultState(), '').recents).toEqual([]);
});

test('push: does not mutate input state', () => {
    const s0 = defaultState();
    const s1 = push(s0, 'charts', 1000);
    expect(s0.recents).toEqual([]);  // unchanged
    expect(s1).not.toBe(s0);
});

// ── listRecents ─────────────────────────────────────────────────

test('listRecents: returns copy of recents', () => {
    const s = push(defaultState(), 'charts', 1000);
    const got = listRecents(s);
    expect(got).toEqual(s.recents);
    expect(got).not.toBe(s.recents);  // shallow copy, not same ref
});

test('listRecents: excludes a given viewId', () => {
    let s = defaultState();
    s = push(s, 'screener', 1000);
    s = push(s, 'charts',   2000);
    expect(listRecents(s, 'charts')).toEqual([{ viewId: 'screener', at: 1000 }]);
});

test('listRecents: null/undefined safe', () => {
    expect(listRecents()).toEqual([]);
    expect(listRecents(null)).toEqual([]);
    expect(listRecents({})).toEqual([]);
});

// ── buildRecentItems ────────────────────────────────────────────

test('buildRecentItems: maps recents via tiles map, drops unknowns', () => {
    const recents = [{ viewId: 'charts', at: 1000 }, { viewId: 'missing', at: 2000 }];
    const tilesByVid = new Map([
        ['charts', ['charts', 'Charts', '📈', 'Live charts', null]],
    ]);
    const items = buildRecentItems(recents, tilesByVid);
    expect(items.length).toBe(1);
    expect(items[0]).toMatchObject({
        id: 'recent:charts',
        kind: 'recent',
        viewId: 'charts',
        label: 'Charts',
        icon: '🕒',
        hint: 'Live charts',
        category: 'Recent',
    });
});

test('buildRecentItems: non-array input → []', () => {
    expect(buildRecentItems(null, new Map())).toEqual([]);
});

// ── clearRecents ────────────────────────────────────────────────

test('clearRecents: returns empty recents while preserving version', () => {
    const s = push(defaultState(), 'charts', 1000);
    const c = clearRecents(s);
    expect(c.recents).toEqual([]);
    expect(c.version).toBe(SCHEMA_VERSION);
});

// ── loadState / saveState with fake storage ────────────────────

function makeStorage() {
    const m = new Map();
    return {
        getItem: (k) => m.has(k) ? m.get(k) : null,
        setItem: (k, v) => m.set(k, v),
    };
}

test('saveState → loadState round-trips', () => {
    const storage = makeStorage();
    const s = push(defaultState(), 'charts', 1000);
    saveState(s, storage);
    expect(loadState(storage)).toEqual(s);
});

test('loadState: missing key → defaultState', () => {
    expect(loadState(makeStorage())).toEqual(defaultState());
});

test('loadState: malformed JSON → defaultState (no throw)', () => {
    const storage = makeStorage();
    storage.setItem(STORAGE_KEY, '{not json');
    expect(loadState(storage)).toEqual(defaultState());
});

test('saveState: storage throwing is silently swallowed', () => {
    const throws = { setItem: () => { throw new Error('quota'); } };
    expect(() => saveState(defaultState(), throws)).not.toThrow();
});
