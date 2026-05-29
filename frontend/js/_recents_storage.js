// Recently-visited views tracker (pure helpers, no DOM).
//
// Schema (key `tv-recents-v1`):
//   {
//     version: 1,
//     recents: [
//       { viewId: "charts",   at: 1716800000000 },
//       { viewId: "screener", at: 1716799000000 },
//       ...                                          (newest first)
//     ]
//   }
//
// Capped at MAX_RECENTS entries. Self-de-dups by viewId — pushing an
// existing viewId moves it to the front and refreshes its timestamp.

export const STORAGE_KEY = 'tv-recents-v1';
export const SCHEMA_VERSION = 1;
export const MAX_RECENTS = 10;

// Views that never count as a "destination" (transient overlays, launcher
// itself, the keyboard-shortcuts cheat-sheet). Push() ignores these.
export const SKIP_VIEWS = new Set([
    '', 'launcher', 'keyboard-shortcuts', 'login', 'logout', 'auth',
]);

export function defaultState() {
    return { version: SCHEMA_VERSION, recents: [] };
}

export function migrate(raw) {
    if (!raw || typeof raw !== 'object') return defaultState();
    const recents = Array.isArray(raw.recents)
        ? raw.recents
              .map(r => (r && typeof r === 'object'
                  && typeof r.viewId === 'string' && r.viewId.trim()
                  && Number.isFinite(r.at))
                  ? { viewId: r.viewId, at: r.at }
                  : null)
              .filter(Boolean)
              .slice(0, MAX_RECENTS)
        : [];
    return { version: SCHEMA_VERSION, recents };
}

export function loadState(storage = globalThis.localStorage) {
    try {
        const raw = storage && storage.getItem(STORAGE_KEY);
        return migrate(raw ? JSON.parse(raw) : null);
    } catch { return defaultState(); }
}

export function saveState(state, storage = globalThis.localStorage) {
    if (!storage || !state) return;
    try { storage.setItem(STORAGE_KEY, JSON.stringify(state)); }
    catch { /* private mode / quota — ignore */ }
}

// Pure: push a new visit to the front, dedupe, cap. Returns NEW state
// (does not mutate input).
export function push(state, viewId, at = Date.now()) {
    if (!viewId || typeof viewId !== 'string') return state || defaultState();
    if (SKIP_VIEWS.has(viewId)) return state || defaultState();
    const s = state && Array.isArray(state.recents) ? state : defaultState();
    const filtered = s.recents.filter(r => r.viewId !== viewId);
    const recents = [{ viewId, at }, ...filtered].slice(0, MAX_RECENTS);
    return { version: SCHEMA_VERSION, recents };
}

// List recents excluding the currently-active view (so it doesn't show
// up as "recent" while you're on it). Returns array of {viewId, at}.
export function listRecents(state, excludeViewId = null) {
    if (!state || !Array.isArray(state.recents)) return [];
    if (!excludeViewId) return [...state.recents];
    return state.recents.filter(r => r.viewId !== excludeViewId);
}

// Build palette-shaped items from a recents list, joined with the tiles
// map (label/icon/hint come from tiles). Drops entries whose viewId
// isn't in the tiles map.
export function buildRecentItems(recents, tilesByViewId) {
    if (!Array.isArray(recents)) return [];
    return recents.map(r => {
        const t = tilesByViewId.get(r.viewId);
        if (!t) return null;
        return {
            id: `recent:${r.viewId}`,
            kind: 'recent',
            viewId: r.viewId,
            label: t[1] || r.viewId,
            icon: '🕒',
            hint: t[3] || '',
            category: 'Recent',
        };
    }).filter(Boolean);
}

export function clearRecents(state) {
    const s = state || defaultState();
    return { ...s, recents: [] };
}
