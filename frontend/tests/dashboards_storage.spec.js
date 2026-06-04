// Dashboards storage: default state, migration, CRUD, persistence.

import { test, expect, beforeEach } from 'vitest';
import * as store from '../js/_dashboards_storage.js';

// In-memory storage shim so tests don't pollute localStorage.
function makeStorage() {
    const data = new Map();
    return {
        getItem: k => data.has(k) ? data.get(k) : null,
        setItem: (k, v) => { data.set(k, String(v)); },
        removeItem: k => { data.delete(k); },
        clear: () => { data.clear(); },
        _peek: () => Object.fromEntries(data),
    };
}

let storage;
beforeEach(() => { storage = makeStorage(); });

// ── defaultState / migrate ────────────────────────────────────────

test('defaultState has main dashboard with no tiles', () => {
    const s = store.defaultState();
    expect(s.version).toBe(store.SCHEMA_VERSION);
    expect(s.active).toBe('main');
    expect(s.dashboards.main.id).toBe('main');
    expect(s.dashboards.main.tiles).toEqual([]);
});

test('migrate accepts current-version payload', () => {
    const good = store.defaultState();
    expect(store.migrate(good)).toEqual(good);
});

test('migrate rejects null / non-object / wrong-version → default', () => {
    expect(store.migrate(null).active).toBe('main');
    expect(store.migrate('garbage').active).toBe('main');
    expect(store.migrate({ version: 99, dashboards: {} }).active).toBe('main');
});

test('migrate keeps only well-formed dashboards', () => {
    const raw = {
        version: store.SCHEMA_VERSION,
        active: 'b',
        dashboards: {
            'a': { id: 'a', name: 'A', tiles: [{ id: 't1', viewId: 'x' }] },
            'bad-missing-name': { id: 'x' },
            'b': { id: 'b', name: 'B', tiles: 'not-an-array' },
        },
    };
    const m = store.migrate(raw);
    expect(Object.keys(m.dashboards).sort()).toEqual(['a', 'b']);
    expect(m.dashboards.a.tiles.length).toBe(1);
    expect(m.dashboards.b.tiles).toEqual([]);
});

test('migrate falls back to first dashboard when active is invalid', () => {
    const raw = {
        version: store.SCHEMA_VERSION,
        active: 'nonexistent',
        dashboards: { only: { id: 'only', name: 'Only', tiles: [] } },
    };
    expect(store.migrate(raw).active).toBe('only');
});

// ── load / save round trip ───────────────────────────────────────

test('loadState returns default when key missing', () => {
    expect(store.loadState(storage).active).toBe('main');
});

test('saveState + loadState round trip', () => {
    const a = store.createDashboard(store.defaultState(), 'My Setup');
    expect(store.saveState(a, storage)).toBe(true);
    const b = store.loadState(storage);
    expect(b).toEqual(a);
});

test('loadState returns default on corrupt JSON', () => {
    storage.setItem(store.STORAGE_KEY, '{bad json');
    expect(store.loadState(storage).active).toBe('main');
});

test('saveState gracefully fails when storage is null', () => {
    expect(store.saveState(store.defaultState(), null)).toBe(false);
});

// ── slugifyName ───────────────────────────────────────────────────

test('slugifyName lowercases + dashes + collision-suffixes', () => {
    expect(store.slugifyName('My Setup')).toBe('my-setup');
    expect(store.slugifyName('   Edges & Spaces  ')).toBe('edges-spaces');
    expect(store.slugifyName('A', new Set(['a']))).toBe('a-2');
    expect(store.slugifyName('A', new Set(['a', 'a-2', 'a-3']))).toBe('a-4');
});

test('slugifyName falls back to "dashboard" on empty / weird names', () => {
    expect(store.slugifyName('')).toBe('dashboard');
    expect(store.slugifyName('!!!')).toBe('dashboard');
    expect(store.slugifyName('!!!', new Set(['dashboard']))).toBe('dashboard-2');
});

// ── dashboard CRUD ────────────────────────────────────────────────

test('createDashboard adds + sets active', () => {
    const s = store.createDashboard(store.defaultState(), 'Macro');
    expect(s.dashboards.macro).toBeTruthy();
    expect(s.active).toBe('macro');
});

test('createDashboard rejects empty name', () => {
    const before = store.defaultState();
    expect(store.createDashboard(before, '')).toBe(before);
    expect(store.createDashboard(before, '   ')).toBe(before);
});

test('createDashboard does not mutate input state', () => {
    const before = store.defaultState();
    const beforeJson = JSON.stringify(before);
    store.createDashboard(before, 'X');
    expect(JSON.stringify(before)).toBe(beforeJson);
});

test('renameDashboard updates name', () => {
    const s1 = store.createDashboard(store.defaultState(), 'A');
    const s2 = store.renameDashboard(s1, 'a', 'Aleph');
    expect(s2.dashboards.a.name).toBe('Aleph');
});

test('renameDashboard no-op when id missing or new name empty', () => {
    const s = store.defaultState();
    expect(store.renameDashboard(s, 'no-such', 'X')).toBe(s);
    expect(store.renameDashboard(s, 'main', '')).toBe(s);
});

test('deleteDashboard removes + switches active when needed', () => {
    let s = store.defaultState();
    s = store.createDashboard(s, 'X');
    s = store.createDashboard(s, 'Y');
    expect(s.active).toBe('y');
    s = store.deleteDashboard(s, 'y');
    expect(s.dashboards.y).toBeUndefined();
    expect(s.active).not.toBe('y');
});

test('deleteDashboard returns fresh default when removing the last dashboard', () => {
    const s = store.deleteDashboard(store.defaultState(), 'main');
    expect(s.active).toBe('main');
    expect(s.dashboards.main.tiles).toEqual([]);
});

test('setActive switches when id valid, no-op when invalid', () => {
    let s = store.createDashboard(store.defaultState(), 'X');
    s = store.setActive(s, 'main');
    expect(s.active).toBe('main');
    expect(store.setActive(s, 'no-such')).toBe(s);
});

// ── tile CRUD ────────────────────────────────────────────────────

test('addTile appends with unique id', () => {
    let s = store.addTile(store.defaultState(), 'main', 'vpin');
    expect(s.dashboards.main.tiles).toHaveLength(1);
    expect(s.dashboards.main.tiles[0].viewId).toBe('vpin');
    s = store.addTile(s, 'main', 'oi-change');
    expect(s.dashboards.main.tiles).toHaveLength(2);
    expect(s.dashboards.main.tiles[0].id).not.toBe(s.dashboards.main.tiles[1].id);
});

test('addTile no-ops on missing dashboard or empty viewId', () => {
    const s = store.defaultState();
    expect(store.addTile(s, 'no-such', 'vpin')).toBe(s);
    expect(store.addTile(s, 'main', '')).toBe(s);
});

test('addTile stores per-tile config object', () => {
    const s = store.addTile(store.defaultState(), 'main', 'vpin', { period: 14 });
    expect(s.dashboards.main.tiles[0].config).toEqual({ period: 14 });
});

test('addTile tags new tiles as kind=view for the discriminated union', () => {
    const s = store.addTile(store.defaultState(), 'main', 'vpin');
    // Pins the contract the graph/view dispatch in dashboards.js depends
    // on — if a future refactor drops the `kind` field, graph tiles will
    // start rendering through the launcher view path and silently break.
    expect(s.dashboards.main.tiles[0].kind).toBe('view');
    expect(s.dashboards.main.tiles[0].viewId).toBe('vpin');
});

test('addGraphTile appends a graph-kind tile with graphId, not viewId', () => {
    const s = store.addGraphTile(store.defaultState(), 'main', 'cumulative_pnl');
    expect(s.dashboards.main.tiles).toHaveLength(1);
    expect(s.dashboards.main.tiles[0].kind).toBe('graph');
    expect(s.dashboards.main.tiles[0].graphId).toBe('cumulative_pnl');
    expect(s.dashboards.main.tiles[0].viewId).toBeUndefined();
});

test('addGraphTile no-ops on missing dashboard or empty graphId', () => {
    const s = store.defaultState();
    expect(store.addGraphTile(s, 'no-such', 'cumulative_pnl')).toBe(s);
    expect(store.addGraphTile(s, 'main', '')).toBe(s);
});

test('migrate accepts both legacy view tiles and graph tiles', () => {
    const raw = {
        version: store.SCHEMA_VERSION,
        active: 'm',
        dashboards: {
            m: { id: 'm', name: 'M', tiles: [
                { id: 't1', viewId: 'webull' },                       // legacy view
                { id: 't2', kind: 'view', viewId: 'mlp-k1' },         // explicit view
                { id: 't3', kind: 'graph', graphId: 'cumulative_pnl' }, // graph
                { id: 't4', kind: 'graph' },                          // missing graphId — dropped
            ] },
        },
    };
    const m = store.migrate(raw);
    expect(m.dashboards.m.tiles).toHaveLength(3);
    expect(m.dashboards.m.tiles[0].viewId).toBe('webull');
    expect(m.dashboards.m.tiles[1].viewId).toBe('mlp-k1');
    expect(m.dashboards.m.tiles[2].graphId).toBe('cumulative_pnl');
});

test('removeTile removes by id', () => {
    let s = store.addTile(store.defaultState(), 'main', 'vpin');
    const tid = s.dashboards.main.tiles[0].id;
    s = store.removeTile(s, 'main', tid);
    expect(s.dashboards.main.tiles).toEqual([]);
});

test('removeTile no-op on missing dashboard or unknown tile id', () => {
    const s = store.addTile(store.defaultState(), 'main', 'vpin');
    expect(store.removeTile(s, 'no-such', 'anything')).toBe(s);
});

test('moveTile up swaps with neighbor', () => {
    let s = store.addTile(store.defaultState(), 'main', 'a');
    s = store.addTile(s, 'main', 'b');
    s = store.addTile(s, 'main', 'c');
    const [, b, ] = s.dashboards.main.tiles;
    s = store.moveTile(s, 'main', b.id, -1);
    expect(s.dashboards.main.tiles.map(t => t.viewId)).toEqual(['b', 'a', 'c']);
});

test('moveTile down swaps with neighbor', () => {
    let s = store.addTile(store.defaultState(), 'main', 'a');
    s = store.addTile(s, 'main', 'b');
    const [a] = s.dashboards.main.tiles;
    s = store.moveTile(s, 'main', a.id, 1);
    expect(s.dashboards.main.tiles.map(t => t.viewId)).toEqual(['b', 'a']);
});

test('moveTile no-op when already at edge', () => {
    let s = store.addTile(store.defaultState(), 'main', 'a');
    s = store.addTile(s, 'main', 'b');
    const tiles = s.dashboards.main.tiles;
    const out = store.moveTile(s, 'main', tiles[0].id, -1);
    expect(out).toBe(s);
});

test('moveTile no-op on unknown tile id', () => {
    const s = store.addTile(store.defaultState(), 'main', 'a');
    expect(store.moveTile(s, 'main', 'no-such', 1)).toBe(s);
});

// ── moveTileTo (absolute-index drag-drop) ─────────────────────────

function tileLayout(initial = ['a', 'b', 'c', 'd']) {
    let s = store.defaultState();
    for (const v of initial) s = store.addTile(s, 'main', v);
    return s;
}

test('moveTileTo: drop tile 0 onto gap 3 ends up at index 2 (after splice)', () => {
    let s = tileLayout();
    const aId = s.dashboards.main.tiles[0].id;
    s = store.moveTileTo(s, 'main', aId, 3);
    expect(s.dashboards.main.tiles.map(t => t.viewId)).toEqual(['b', 'c', 'a', 'd']);
});

test('moveTileTo: drop tile 3 onto gap 0 lands at index 0', () => {
    let s = tileLayout();
    const dId = s.dashboards.main.tiles[3].id;
    s = store.moveTileTo(s, 'main', dId, 0);
    expect(s.dashboards.main.tiles.map(t => t.viewId)).toEqual(['d', 'a', 'b', 'c']);
});

test('moveTileTo: drop onto the after-last gap lands at end', () => {
    let s = tileLayout();
    const aId = s.dashboards.main.tiles[0].id;
    s = store.moveTileTo(s, 'main', aId, 4);
    expect(s.dashboards.main.tiles.map(t => t.viewId)).toEqual(['b', 'c', 'd', 'a']);
});

test('moveTileTo: dropping onto own position is a no-op', () => {
    const s = tileLayout();
    const bId = s.dashboards.main.tiles[1].id;
    expect(store.moveTileTo(s, 'main', bId, 1)).toBe(s);
});

test('moveTileTo: out-of-range indexes clamp to valid range', () => {
    let s = tileLayout();
    const aId = s.dashboards.main.tiles[0].id;
    s = store.moveTileTo(s, 'main', aId, 999);
    expect(s.dashboards.main.tiles[s.dashboards.main.tiles.length - 1].viewId).toBe('a');
});

test('moveTileTo: unknown tile id / missing dashboard → no-op', () => {
    const s = tileLayout();
    expect(store.moveTileTo(s, 'main', 'no-such', 0)).toBe(s);
    expect(store.moveTileTo(s, 'missing-dash', 'anything', 0)).toBe(s);
});

test('moveTileTo: non-finite newIndex → no-op', () => {
    const s = tileLayout();
    const aId = s.dashboards.main.tiles[0].id;
    expect(store.moveTileTo(s, 'main', aId, NaN)).toBe(s);
});

// ── duplicateDashboard ────────────────────────────────────────────

test('duplicateDashboard clones name + tiles with fresh ids', () => {
    let s = store.defaultState();
    s = store.addTile(s, 'main', 'vpin');
    s = store.addTile(s, 'main', 'oi-change');
    const originalTileIds = s.dashboards.main.tiles.map(t => t.id);
    s = store.duplicateDashboard(s, 'main');
    expect(Object.keys(s.dashboards).length).toBe(2);
    const copy = Object.values(s.dashboards).find(d => d.id !== 'main');
    expect(copy.name).toBe('Main (copy)');
    expect(copy.tiles.length).toBe(2);
    // Cloned tiles must have NEW ids.
    expect(copy.tiles.every(t => !originalTileIds.includes(t.id))).toBe(true);
    expect(copy.tiles.map(t => t.viewId)).toEqual(['vpin', 'oi-change']);
});

test('duplicateDashboard sets active to the new copy', () => {
    const s = store.duplicateDashboard(store.defaultState(), 'main');
    expect(s.active).not.toBe('main');
    expect(s.dashboards[s.active].name).toBe('Main (copy)');
});

test('duplicateDashboard no-op on missing id', () => {
    const s = store.defaultState();
    expect(store.duplicateDashboard(s, 'no-such')).toBe(s);
});

// ── export / import ──────────────────────────────────────────────

test('exportState round-trips through importState', () => {
    let s = store.createDashboard(store.defaultState(), 'Macro');
    s = store.addTile(s, 'macro', 'vpin');
    const json = store.exportState(s);
    expect(typeof json).toBe('string');
    const imported = store.importState(json);
    expect(imported).toEqual(s);
});

test('importState returns null on non-string / blank / unparseable input', () => {
    expect(store.importState(null)).toBe(null);
    expect(store.importState('')).toBe(null);
    expect(store.importState('   ')).toBe(null);
    expect(store.importState('{not valid json')).toBe(null);
});

test('importState migrates wrong-version payload to defaults', () => {
    const imported = store.importState(JSON.stringify({ version: 99 }));
    expect(imported.active).toBe('main');
});

// ── helpers ──────────────────────────────────────────────────────

test('getActiveDashboard returns active or null', () => {
    const s = store.defaultState();
    expect(store.getActiveDashboard(s).id).toBe('main');
    expect(store.getActiveDashboard({ ...s, active: 'no-such' })).toBe(null);
});

test('listDashboards returns alphabetically-sorted array', () => {
    let s = store.defaultState();
    s = store.createDashboard(s, 'Zeta');
    s = store.createDashboard(s, 'Alpha');
    const list = store.listDashboards(s);
    expect(list.map(d => d.name)).toEqual(['Alpha', 'Main', 'Zeta']);
});
