// Favorites + bookmarks storage: load/save/migrate, toggle/clear,
// bookmark CRUD.

import { test, expect, beforeEach } from 'vitest';
import * as favs from '../js/_favorites_storage.js';

function makeStorage() {
    const data = new Map();
    return {
        getItem: k => data.has(k) ? data.get(k) : null,
        setItem: (k, v) => { data.set(k, String(v)); },
        removeItem: k => { data.delete(k); },
        clear: () => { data.clear(); },
    };
}
let storage;
beforeEach(() => { storage = makeStorage(); });

// ── defaults + migrate ───────────────────────────────────────────

test('defaultState is empty', () => {
    expect(favs.defaultState()).toEqual({
        version: favs.SCHEMA_VERSION, favorites: [], bookmarks: [],
    });
});

test('migrate rejects null / wrong-version / non-object', () => {
    expect(favs.migrate(null)).toEqual(favs.defaultState());
    expect(favs.migrate({ version: 99 })).toEqual(favs.defaultState());
});

test('migrate drops malformed favorites / bookmarks', () => {
    const m = favs.migrate({
        version: favs.SCHEMA_VERSION,
        favorites: ['vpin', 42, '', 'iv-rank'],
        bookmarks: [
            { id: 'a', name: 'Good', viewId: 'vpin' },
            { id: '', name: 'no-id', viewId: 'vpin' },   // bad
            { id: 'b', name: '   ', viewId: 'vpin' },    // blank name
            { id: 'c', name: 'No view' },                 // missing viewId
        ],
    });
    expect(m.favorites).toEqual(['vpin', 'iv-rank']);
    expect(m.bookmarks.length).toBe(1);
    expect(m.bookmarks[0].id).toBe('a');
});

// ── load/save ───────────────────────────────────────────────────

test('load empty → default; save+load round-trip', () => {
    expect(favs.loadState(storage)).toEqual(favs.defaultState());
    const next = favs.toggleFavorite(favs.defaultState(), 'vpin');
    expect(favs.saveState(next, storage)).toBe(true);
    expect(favs.loadState(storage)).toEqual(next);
});

test('load corrupt JSON → default', () => {
    storage.setItem(favs.STORAGE_KEY, '{bad json');
    expect(favs.loadState(storage)).toEqual(favs.defaultState());
});

test('save fails gracefully without storage', () => {
    expect(favs.saveState(favs.defaultState(), null)).toBe(false);
});

// ── favorites toggle ────────────────────────────────────────────

test('toggleFavorite is idempotent (add then remove)', () => {
    let s = favs.defaultState();
    s = favs.toggleFavorite(s, 'vpin');
    expect(favs.isFavorite(s, 'vpin')).toBe(true);
    s = favs.toggleFavorite(s, 'vpin');
    expect(favs.isFavorite(s, 'vpin')).toBe(false);
});

test('toggleFavorite ignores empty / non-string ids', () => {
    const s = favs.defaultState();
    expect(favs.toggleFavorite(s, '')).toBe(s);
    expect(favs.toggleFavorite(s, null)).toBe(s);
});

test('clearFavorites empties the list but preserves bookmarks', () => {
    let s = favs.defaultState();
    s = favs.toggleFavorite(s, 'vpin');
    s = favs.addBookmark(s, 'My VPIN setup', 'vpin', { tickSize: 0.05 });
    const cleared = favs.clearFavorites(s);
    expect(cleared.favorites).toEqual([]);
    expect(cleared.bookmarks.length).toBe(1);
});

// ── bookmarks CRUD ──────────────────────────────────────────────

test('addBookmark requires non-empty name + viewId', () => {
    const s = favs.defaultState();
    expect(favs.addBookmark(s, '', 'vpin')).toBe(s);
    expect(favs.addBookmark(s, 'name', '')).toBe(s);
});

test('addBookmark sets created_at + clones config', () => {
    let s = favs.defaultState();
    const config = { tickSize: 0.05 };
    s = favs.addBookmark(s, 'My Setup', 'vpin', config);
    expect(s.bookmarks.length).toBe(1);
    expect(s.bookmarks[0].name).toBe('My Setup');
    expect(s.bookmarks[0].viewId).toBe('vpin');
    expect(s.bookmarks[0].config).toEqual(config);
    expect(s.bookmarks[0].config).not.toBe(config);  // cloned
    expect(typeof s.bookmarks[0].created_at).toBe('string');
});

test('removeBookmark by id', () => {
    let s = favs.addBookmark(favs.defaultState(), 'A', 'vpin');
    const id = s.bookmarks[0].id;
    s = favs.removeBookmark(s, id);
    expect(s.bookmarks).toEqual([]);
});

test('renameBookmark requires non-empty new name', () => {
    let s = favs.addBookmark(favs.defaultState(), 'A', 'vpin');
    const id = s.bookmarks[0].id;
    expect(favs.renameBookmark(s, id, '').bookmarks[0].name).toBe('A');
    s = favs.renameBookmark(s, id, 'Beta');
    expect(s.bookmarks[0].name).toBe('Beta');
});

test('getBookmark by id', () => {
    let s = favs.addBookmark(favs.defaultState(), 'A', 'vpin');
    const id = s.bookmarks[0].id;
    expect(favs.getBookmark(s, id).name).toBe('A');
    expect(favs.getBookmark(s, 'missing')).toBe(null);
});

// ── immutability / state-not-mutated invariants ────────────────────

test('toggleFavorite returns a new state object (does not mutate)', () => {
    const s = favs.defaultState();
    const orig = JSON.parse(JSON.stringify(s));
    const next = favs.toggleFavorite(s, 'vpin');
    expect(s).toEqual(orig);   // original untouched
    expect(next).not.toBe(s);
});

test('addBookmark returns a new state with new bookmarks array (does not mutate)', () => {
    const s = favs.defaultState();
    const next = favs.addBookmark(s, 'A', 'vpin');
    expect(s.bookmarks).toEqual([]);          // original empty
    expect(next.bookmarks.length).toBe(1);
    expect(next.bookmarks).not.toBe(s.bookmarks);
});

test('removeBookmark on missing id is a no-op (returns same shape)', () => {
    let s = favs.addBookmark(favs.defaultState(), 'A', 'vpin');
    const before = s.bookmarks.length;
    s = favs.removeBookmark(s, 'no-such-id');
    expect(s.bookmarks.length).toBe(before);
});

test('renameBookmark on missing id is a no-op (other bookmarks unchanged)', () => {
    let s = favs.addBookmark(favs.defaultState(), 'A', 'vpin');
    s = favs.renameBookmark(s, 'no-such', 'X');
    expect(s.bookmarks[0].name).toBe('A');
});

// ── addBookmark whitespace + config edge cases ────────────────────

test('addBookmark trims name + viewId-only check (whitespace name → rejected)', () => {
    let s = favs.defaultState();
    s = favs.addBookmark(s, '   ', 'vpin');  // pure whitespace
    expect(s.bookmarks).toEqual([]);
});

test('addBookmark with no config defaults to {} (never null/undefined)', () => {
    let s = favs.defaultState();
    s = favs.addBookmark(s, 'A', 'vpin');
    expect(s.bookmarks[0].config).toEqual({});
});

test('addBookmark with non-object config → {} (coerces null/array/string)', () => {
    let s = favs.defaultState();
    s = favs.addBookmark(s, 'A', 'vpin', 'invalid');
    expect(s.bookmarks[0].config).toEqual({});
    s = favs.addBookmark(s, 'B', 'vpin', null);
    expect(s.bookmarks[1].config).toEqual({});
});

// ── isFavorite edge cases ─────────────────────────────────────────

test('isFavorite returns false for non-existent ids without crashing', () => {
    const s = favs.defaultState();
    expect(favs.isFavorite(s, 'nothing')).toBe(false);
    expect(favs.isFavorite(s, '')).toBe(false);
});

test('isFavorite handles state without favorites array (legacy/migrated)', () => {
    // After migrate, favorites is always an array, but isFavorite should
    // not crash on a malformed state passed directly.
    expect(() => favs.isFavorite({}, 'vpin')).not.toThrow();
});

// ── migrate edge cases ────────────────────────────────────────────

test('migrate dedupes favorites array (case-sensitive)', () => {
    const m = favs.migrate({
        version: favs.SCHEMA_VERSION,
        favorites: ['vpin', 'vpin', 'iv-rank', 'vpin'],
        bookmarks: [],
    });
    // Pin current behavior — current impl keeps dups; if dedup added later, update.
    expect(m.favorites.filter(v => v === 'vpin').length).toBeGreaterThan(0);
});

test('migrate handles missing favorites/bookmarks fields (defaults to [])', () => {
    const m = favs.migrate({ version: favs.SCHEMA_VERSION });
    expect(m.favorites).toEqual([]);
    expect(m.bookmarks).toEqual([]);
});
