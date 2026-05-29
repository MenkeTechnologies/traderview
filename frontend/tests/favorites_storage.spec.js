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
