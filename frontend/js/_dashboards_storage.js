// Dashboards storage layer — pure, testable, localStorage-backed.
//
// Schema (key `tv-dashboards-v1`):
// {
//   version: 1,
//   active: <dashboardId>,
//   dashboards: {
//     <dashboardId>: { id, name, tiles: [{ id, viewId, config }] }
//   }
// }
//
// All mutations return a NEW state object (no in-place mutation) so the
// caller can decide when to persist. Storage I/O is wrapped in try/catch
// so private-mode browsers and disabled localStorage degrade to in-memory.

export const STORAGE_KEY = 'tv-dashboards-v1';
export const SCHEMA_VERSION = 1;

// Returns the default state with one empty "main" dashboard. Used both
// for first-run and as the migration target for older / corrupt data.
export function defaultState() {
    const id = 'main';
    return {
        version: SCHEMA_VERSION,
        active: id,
        dashboards: {
            [id]: { id, name: 'Main', tiles: [] },
        },
    };
}

// Validates and migrates an arbitrary payload to the current schema. If
// the payload is bad in any way, returns defaultState().
export function migrate(raw) {
    if (!raw || typeof raw !== 'object') return defaultState();
    if (raw.version !== SCHEMA_VERSION) return defaultState();
    if (!raw.dashboards || typeof raw.dashboards !== 'object') return defaultState();
    const dashboards = {};
    for (const [id, d] of Object.entries(raw.dashboards)) {
        if (!d || typeof d !== 'object' || typeof d.id !== 'string') continue;
        if (typeof d.name !== 'string' || !d.name.trim()) continue;
        const tiles = Array.isArray(d.tiles) ? d.tiles.filter(t =>
            t && typeof t.id === 'string' && typeof t.viewId === 'string'
        ).map(t => ({
            id: t.id, viewId: t.viewId,
            config: (t.config && typeof t.config === 'object') ? t.config : {},
        })) : [];
        dashboards[id] = { id: d.id, name: d.name, tiles };
    }
    if (Object.keys(dashboards).length === 0) return defaultState();
    let active = raw.active;
    if (typeof active !== 'string' || !dashboards[active]) {
        active = Object.keys(dashboards)[0];
    }
    return { version: SCHEMA_VERSION, active, dashboards };
}

// Reads from localStorage with full safety (private-mode, JSON parse,
// schema validation). Optional `storage` argument for testability.
export function loadState(storage = globalThis.localStorage) {
    if (!storage) return defaultState();
    let raw;
    try {
        const s = storage.getItem(STORAGE_KEY);
        if (!s) return defaultState();
        raw = JSON.parse(s);
    } catch {
        return defaultState();
    }
    return migrate(raw);
}

export function saveState(state, storage = globalThis.localStorage) {
    if (!storage) return false;
    try {
        storage.setItem(STORAGE_KEY, JSON.stringify(state));
        return true;
    } catch {
        return false;
    }
}

// Slugified ID generator. Increments a numeric suffix on collision.
export function slugifyName(name, existingIds = new Set()) {
    const base = String(name || 'dashboard')
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-|-$/g, '')
        .slice(0, 32) || 'dashboard';
    if (!existingIds.has(base)) return base;
    for (let i = 2; i < 1000; i++) {
        const id = `${base}-${i}`;
        if (!existingIds.has(id)) return id;
    }
    return `${base}-${Date.now()}`;
}

// Generates a unique tile id within a dashboard. Format `tile-{rand}`.
export function newTileId(existingIds = new Set()) {
    for (let attempt = 0; attempt < 50; attempt++) {
        const id = `tile-${Math.random().toString(36).slice(2, 10)}`;
        if (!existingIds.has(id)) return id;
    }
    return `tile-${Date.now()}`;
}

// Dashboard CRUD ─────────────────────────────────────────────────────

export function createDashboard(state, name) {
    if (!name || !String(name).trim()) return state;
    const existing = new Set(Object.keys(state.dashboards));
    const id = slugifyName(name, existing);
    return {
        ...state,
        active: id,
        dashboards: {
            ...state.dashboards,
            [id]: { id, name: String(name).trim(), tiles: [] },
        },
    };
}

export function renameDashboard(state, id, newName) {
    if (!state.dashboards[id]) return state;
    if (!newName || !String(newName).trim()) return state;
    const d = state.dashboards[id];
    return {
        ...state,
        dashboards: {
            ...state.dashboards,
            [id]: { ...d, name: String(newName).trim() },
        },
    };
}

export function deleteDashboard(state, id) {
    if (!state.dashboards[id]) return state;
    if (Object.keys(state.dashboards).length === 1) {
        // Never leave the user with zero dashboards — replace with empty default.
        return defaultState();
    }
    const next = { ...state.dashboards };
    delete next[id];
    const newActive = state.active === id ? Object.keys(next)[0] : state.active;
    return { ...state, active: newActive, dashboards: next };
}

export function setActive(state, id) {
    if (!state.dashboards[id]) return state;
    return { ...state, active: id };
}

// Duplicates an existing dashboard with a fresh id + name suffix.
// Tile ids are regenerated so the clone's tiles don't share ids with
// the original (preserving the invariant that ids are unique within a
// dashboard, but also across dashboards in the same session for any
// caller that mistakenly compares globally).
export function duplicateDashboard(state, id) {
    const d = state.dashboards[id];
    if (!d) return state;
    const existing = new Set(Object.keys(state.dashboards));
    const slug = slugifyName(`${d.name} copy`, existing);
    const tiles = d.tiles.map(t => ({
        id: newTileId(new Set()),
        viewId: t.viewId,
        config: { ...(t.config || {}) },
    }));
    return {
        ...state,
        active: slug,
        dashboards: {
            ...state.dashboards,
            [slug]: { id: slug, name: `${d.name} (copy)`, tiles },
        },
    };
}

// Returns a JSON-serializable snapshot suitable for the export-to-file
// flow. Equivalent to JSON.stringify(state) but routed through a single
// helper so future schema changes pass through one chokepoint.
export function exportState(state) {
    return JSON.stringify(state, null, 2);
}

// Imports a JSON blob, validates + migrates, and returns the new state.
// Returns null when the blob can't be parsed at all — lets the caller
// surface a friendly error vs. silently resetting to defaults.
export function importState(jsonText) {
    if (typeof jsonText !== 'string' || !jsonText.trim()) return null;
    let raw;
    try { raw = JSON.parse(jsonText); }
    catch { return null; }
    return migrate(raw);
}

// Tile CRUD ──────────────────────────────────────────────────────────

export function addTile(state, dashboardId, viewId, config = {}) {
    const d = state.dashboards[dashboardId];
    if (!d) return state;
    if (!viewId || typeof viewId !== 'string') return state;
    const existing = new Set(d.tiles.map(t => t.id));
    const id = newTileId(existing);
    const tile = { id, viewId, config: { ...config } };
    return {
        ...state,
        dashboards: {
            ...state.dashboards,
            [dashboardId]: { ...d, tiles: [...d.tiles, tile] },
        },
    };
}

export function removeTile(state, dashboardId, tileId) {
    const d = state.dashboards[dashboardId];
    if (!d) return state;
    return {
        ...state,
        dashboards: {
            ...state.dashboards,
            [dashboardId]: { ...d, tiles: d.tiles.filter(t => t.id !== tileId) },
        },
    };
}

// Returns a new state with `tileId` moved by `direction` (-1 = up/left,
// +1 = down/right). No-op when at the edge.
export function moveTile(state, dashboardId, tileId, direction) {
    const d = state.dashboards[dashboardId];
    if (!d) return state;
    const i = d.tiles.findIndex(t => t.id === tileId);
    if (i < 0) return state;
    const j = i + (direction < 0 ? -1 : 1);
    if (j < 0 || j >= d.tiles.length) return state;
    const tiles = [...d.tiles];
    [tiles[i], tiles[j]] = [tiles[j], tiles[i]];
    return {
        ...state,
        dashboards: {
            ...state.dashboards,
            [dashboardId]: { ...d, tiles },
        },
    };
}

// Moves `tileId` to absolute slot `newIndex`. Used by the drag-and-drop
// reorder where the user drops a tile in a specific gap (regardless of
// its original position). `newIndex` is interpreted as the destination
// in the ORIGINAL array (before removing the moving tile), then adjusted
// to account for the splice — so dropping a tile from index 2 onto gap 5
// lands it at gap 4 (which represents "after the 4th tile" in the new
// ordering). No-op when the move resolves to the tile's existing slot.
export function moveTileTo(state, dashboardId, tileId, newIndex) {
    const d = state.dashboards[dashboardId];
    if (!d) return state;
    const i = d.tiles.findIndex(t => t.id === tileId);
    if (i < 0) return state;
    if (!Number.isFinite(newIndex)) return state;
    // Allowed destination range is [0 .. length] — the upper-bound gap
    // is "after the last existing tile."
    let target = Math.max(0, Math.min(d.tiles.length, Math.floor(newIndex)));
    // If dropping past the current position, removing first shifts the
    // gap down by 1; account for that.
    if (target > i) target -= 1;
    if (target === i) return state;
    const tiles = [...d.tiles];
    const [moving] = tiles.splice(i, 1);
    tiles.splice(target, 0, moving);
    return {
        ...state,
        dashboards: {
            ...state.dashboards,
            [dashboardId]: { ...d, tiles },
        },
    };
}

// Helpers ────────────────────────────────────────────────────────────

export function getActiveDashboard(state) {
    return state.dashboards[state.active] || null;
}

export function listDashboards(state) {
    return Object.values(state.dashboards).sort((a, b) =>
        a.name.localeCompare(b.name));
}
