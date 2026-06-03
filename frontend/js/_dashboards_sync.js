// Push-only mirror of dashboards localStorage → /dashboards REST API.
//
// localStorage is the source of truth on each device; the backend exists
// as a per-user read-only mirror so the user can recover or migrate their
// layouts between devices. Auth-gated: anonymous sessions stay local.
//
// Reconciliation uses a stable `slug` stored inside `layout.slug` (the
// frontend's dashboard id like "main" / "trading-2"). Backend rows are
// keyed by uuid; we maintain an in-memory slug→uuid map populated lazily
// from the first GET /dashboards. Per-slug signature dedupe skips PUTs
// when the layout hasn't changed since the last sync.
//
// Deletes are scoped to slugs we have *upserted in the current session*
// (tracked in `ownedSlugs`). Backend rows from another device are left
// alone — the first sync on a new device only adds, it doesn't blow
// away rows it didn't create.

import { api, ApiError } from './api.js';

const DEBOUNCE_MS = 500;

let timer = null;
let pending = null;
let mapLoaded = false;
const slugToUuid = new Map();
const sigBySlug = new Map();
const ownedSlugs = new Set();
let inflight = null;

function hasAuth() {
    try {
        if (typeof globalThis.fetch !== 'function') return false;
        return !!globalThis.__tvApiToken;
    } catch {
        return false;
    }
}

function buildLayout(d) {
    return { slug: d.id, tiles: d.tiles };
}

async function ensureMap() {
    if (mapLoaded) return;
    mapLoaded = true;
    try {
        const rows = await api.dashboards();
        for (const r of rows || []) {
            const slug = r && r.layout && typeof r.layout.slug === 'string'
                ? r.layout.slug : null;
            if (slug && r.id) {
                slugToUuid.set(slug, r.id);
                sigBySlug.set(slug, JSON.stringify(r.layout));
            }
        }
    } catch (e) {
        // Network down / 401 / etc. — leave map empty; next push retries.
        mapLoaded = false;
        console.warn('[dashboards-sync] initial list failed:', e?.message || e);
    }
}

async function upsertOne(slug, d) {
    const layout = buildLayout(d);
    const sig = JSON.stringify(layout);
    const body = { name: d.name, layout };
    const uuid = slugToUuid.get(slug);
    if (uuid) {
        if (sigBySlug.get(slug) === sig) {
            ownedSlugs.add(slug);
            return;
        }
        try {
            await api.updateDashboard(uuid, body);
            sigBySlug.set(slug, sig);
            ownedSlugs.add(slug);
        } catch (e) {
            if (e instanceof ApiError && e.status === 404) {
                slugToUuid.delete(slug);
                sigBySlug.delete(slug);
                try {
                    const row = await api.createDashboard(body);
                    if (row?.id) {
                        slugToUuid.set(slug, row.id);
                        sigBySlug.set(slug, sig);
                        ownedSlugs.add(slug);
                    }
                } catch (e2) {
                    console.warn('[dashboards-sync] recreate failed:', e2?.message || e2);
                }
            } else {
                console.warn('[dashboards-sync] update failed:', e?.message || e);
            }
        }
    } else {
        try {
            const row = await api.createDashboard(body);
            if (row?.id) {
                slugToUuid.set(slug, row.id);
                sigBySlug.set(slug, sig);
                ownedSlugs.add(slug);
            }
        } catch (e) {
            console.warn('[dashboards-sync] create failed:', e?.message || e);
        }
    }
}

async function reconcile(state) {
    if (!hasAuth()) return;
    await ensureMap();
    const localSlugs = new Set(Object.keys(state.dashboards));
    for (const [slug, d] of Object.entries(state.dashboards)) {
        await upsertOne(slug, d);
    }
    for (const slug of [...slugToUuid.keys()]) {
        if (localSlugs.has(slug)) continue;
        if (!ownedSlugs.has(slug)) continue;
        const uuid = slugToUuid.get(slug);
        try {
            await api.deleteDashboard(uuid);
        } catch (e) {
            if (!(e instanceof ApiError && e.status === 404)) {
                console.warn('[dashboards-sync] delete failed:', e?.message || e);
            }
        }
        slugToUuid.delete(slug);
        sigBySlug.delete(slug);
        ownedSlugs.delete(slug);
    }
}

export function schedulePush(state) {
    if (!hasAuth()) return;
    pending = state;
    if (timer) return;
    timer = setTimeout(async () => {
        timer = null;
        const s = pending;
        pending = null;
        if (!s) return;
        inflight = reconcile(s).catch(() => {});
        await inflight;
        if (pending) schedulePush(pending);
    }, DEBOUNCE_MS);
}

export async function pushNow(state) {
    if (timer) { clearTimeout(timer); timer = null; }
    pending = null;
    if (!hasAuth()) return;
    return reconcile(state);
}

export function _resetForTest() {
    if (timer) { clearTimeout(timer); timer = null; }
    pending = null;
    mapLoaded = false;
    slugToUuid.clear();
    sigBySlug.clear();
    ownedSlugs.clear();
    inflight = null;
}
