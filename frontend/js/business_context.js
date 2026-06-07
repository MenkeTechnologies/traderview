// Global business-entity context — every expense view reads from this
// to scope queries to one business or to the aggregated "all" view.
//
// Persistence: the selected business_id lives in localStorage AND is
// reflected in the URL hash as `&business=...` so deep-links carry it.
//
// Subscriber pattern: views call onChange(fn) to re-render when the
// user picks a different business.

import { api } from './api.js';
import { showToast } from './toast.js';
import { t } from './i18n.js';
import { openSetupWizard } from './setup_wizard.js';

const STORE_KEY = 'expense_business_id';
const ALL_SENTINEL = 'all';

const subscribers = new Set();
let cached = null; // { businesses: [], selected: 'all' | uuid }

function readSaved() {
    try {
        return localStorage.getItem(STORE_KEY) || ALL_SENTINEL;
    } catch {
        return ALL_SENTINEL;
    }
}

function writeSaved(id) {
    try {
        if (id && id !== ALL_SENTINEL) localStorage.setItem(STORE_KEY, id);
        else localStorage.removeItem(STORE_KEY);
    } catch {}
}

/** Returns the active business id, or null when "All". */
export function activeBusinessId() {
    const id = cached?.selected ?? readSaved();
    return id && id !== ALL_SENTINEL ? id : null;
}

/** Returns all businesses for the current user (cached after first fetch). */
export async function listBusinesses() {
    if (cached) return cached.businesses;
    let businesses = [];
    try { businesses = await api.businessesList(); } catch (e) {
        console.warn('businesses fetch failed', e.message);
    }
    cached = {
        businesses,
        selected: readSaved(),
    };
    return businesses;
}

/**
 * Drop the cache + notify subscribers. Mirrors broker_context's
 * refreshBrokers — call after CRUD on the business entity from
 * anywhere so the topbar selector, expense views, and dashboard
 * tiles all refetch.
 */
export async function refreshBusinesses() {
    cached = null;
    const fresh = await listBusinesses();
    const sel = readSaved();
    if (sel && sel !== ALL_SENTINEL && !fresh.some(b => b.id === sel)) {
        writeSaved(null);
        if (cached) cached.selected = ALL_SENTINEL;
    }
    notify();
    return fresh;
}

export function onChange(fn) {
    subscribers.add(fn);
    return () => subscribers.delete(fn);
}

function notify() {
    for (const fn of subscribers) {
        try { fn(activeBusinessId()); } catch (e) { console.warn(e); }
    }
}

/** Switch the active business and broadcast. `id` may be `null` or `'all'` to clear. */
export function setActiveBusiness(id) {
    const norm = id && id !== ALL_SENTINEL ? id : ALL_SENTINEL;
    if (cached) cached.selected = norm;
    writeSaved(norm);
    notify();
}

/** Convenience: append `business_id=...` to a params object when one is selected. */
export function withBusinessParam(params = {}) {
    const id = activeBusinessId();
    return id ? { ...params, business_id: id } : params;
}

/**
 * Mount a `<select>` inside the given element. Updates the global
 * selection on change, re-renders the selector when businesses are
 * added/removed. Returns a teardown function.
 */
export async function mountBusinessSelector(host) {
    const businesses = await listBusinesses();
    const selected = activeBusinessId();
    const opts = [
        `<option value="${ALL_SENTINEL}"${!selected ? ' selected' : ''}>${esc(t('biz.all_businesses'))}</option>`,
        ...businesses.map((b) =>
            `<option value="${esc(b.id)}"${selected === b.id ? ' selected' : ''}>${esc(b.name)}</option>`),
        `<option value="__new__">${esc(t('biz.add_new'))}…</option>`,
    ].join('');
    // Flat select to match `.account-select` + `.broker-select` —
    // mirrors the topbar's other inline pickers so vertical centering
    // stays consistent in the topbar's flex row.
    host.innerHTML = `<select class="biz-select account-select" aria-label="${esc(t('biz.label'))}">${opts}</select>`;
    const sel = host.querySelector('.biz-select');
    sel.addEventListener('change', async () => {
        if (sel.value === '__new__') {
            // Reset the visible selection BEFORE the modal opens so a
            // cancel doesn't leave the dropdown stuck on "+ New business".
            const prev = activeBusinessId();
            sel.value = prev || ALL_SENTINEL;
            const created = await openSetupWizard({ kind: 'business' });
            if (created) {
                cached = null;
                await listBusinesses();
                setActiveBusiness(created.id);
                showToast(t('biz.created', { name: created.name }), { level: 'success' });
            }
            await mountBusinessSelector(host);
            return;
        }
        setActiveBusiness(sel.value);
        const label = sel.options[sel.selectedIndex]?.text || '';
        try { showToast(t('biz.switched', { name: label }), { level: 'info' }); } catch {}
    });
}

function esc(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}
