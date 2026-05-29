// Favorites + bookmarks store.
//
// Schema (key `tv-favorites-v1`):
//   {
//     version: 1,
//     favorites: ["view-id-1", "view-id-2", ...],
//     bookmarks: [{ id, name, viewId, config, created_at }]
//   }
//
// Favorites = just a flat ordered list of view ids the user starred (no
// per-view state). Bookmarks = saved configurations of a specific view
// — symbol pinned, threshold pre-set, etc. — that the user explicitly
// named and parked for later. Both persist to localStorage.

export const STORAGE_KEY = 'tv-favorites-v1';
export const SCHEMA_VERSION = 1;

export function defaultState() {
    return { version: SCHEMA_VERSION, favorites: [], bookmarks: [] };
}

export function migrate(raw) {
    if (!raw || typeof raw !== 'object' || raw.version !== SCHEMA_VERSION) return defaultState();
    const favorites = Array.isArray(raw.favorites)
        ? raw.favorites.filter(s => typeof s === 'string' && s.trim())
        : [];
    const bookmarks = Array.isArray(raw.bookmarks)
        ? raw.bookmarks.filter(b => b && typeof b === 'object'
            && typeof b.id === 'string' && b.id.trim()
            && typeof b.name === 'string' && b.name.trim()
            && typeof b.viewId === 'string' && b.viewId.trim())
            .map(b => ({
                id: b.id, name: b.name, viewId: b.viewId,
                config: (b.config && typeof b.config === 'object') ? b.config : {},
                created_at: typeof b.created_at === 'string' ? b.created_at : new Date(0).toISOString(),
            }))
        : [];
    return { version: SCHEMA_VERSION, favorites, bookmarks };
}

export function loadState(storage = globalThis.localStorage) {
    if (!storage) return defaultState();
    try {
        const s = storage.getItem(STORAGE_KEY);
        if (!s) return defaultState();
        return migrate(JSON.parse(s));
    } catch { return defaultState(); }
}

export function saveState(state, storage = globalThis.localStorage) {
    if (!storage) return false;
    try {
        storage.setItem(STORAGE_KEY, JSON.stringify(state));
        return true;
    } catch { return false; }
}

// ── Favorites CRUD ──────────────────────────────────────────────

export function isFavorite(state, viewId) {
    return Array.isArray(state?.favorites) && state.favorites.includes(viewId);
}

// Idempotent toggle. Returns a new state with the favorite added or
// removed.
export function toggleFavorite(state, viewId) {
    if (!viewId || typeof viewId !== 'string') return state;
    const set = new Set(state.favorites || []);
    if (set.has(viewId)) set.delete(viewId);
    else                 set.add(viewId);
    return { ...state, favorites: [...set] };
}

export function clearFavorites(state) {
    return { ...state, favorites: [] };
}

// ── Bookmarks CRUD ──────────────────────────────────────────────

function bookmarkId(existing = new Set()) {
    for (let i = 0; i < 50; i++) {
        const id = `bm-${Math.random().toString(36).slice(2, 10)}`;
        if (!existing.has(id)) return id;
    }
    return `bm-${Date.now()}`;
}

export function addBookmark(state, name, viewId, config = {}) {
    if (!name || !String(name).trim()) return state;
    if (!viewId || typeof viewId !== 'string') return state;
    const existing = new Set((state.bookmarks || []).map(b => b.id));
    const id = bookmarkId(existing);
    const next = {
        id, name: String(name).trim(), viewId,
        config: (config && typeof config === 'object') ? { ...config } : {},
        created_at: new Date().toISOString(),
    };
    return { ...state, bookmarks: [...(state.bookmarks || []), next] };
}

export function removeBookmark(state, id) {
    return { ...state, bookmarks: (state.bookmarks || []).filter(b => b.id !== id) };
}

export function renameBookmark(state, id, newName) {
    if (!newName || !String(newName).trim()) return state;
    return {
        ...state,
        bookmarks: (state.bookmarks || []).map(b =>
            b.id === id ? { ...b, name: String(newName).trim() } : b),
    };
}

export function getBookmark(state, id) {
    return (state.bookmarks || []).find(b => b.id === id) || null;
}
