// Global broker filter — every trade view (dashboard, trades, journal,
// reports) reads from here to scope queries to one broker or to the
// aggregated "all accounts" view.
//
// Distinct from `app.js`'s `state.accountId`: accountId picks one
// SPECIFIC account row, broker picks ALL accounts of a given broker.
// You can have either, or neither (truly aggregated).

import { api } from './api.js';
import { showToast } from './toast.js';
import { t } from './i18n.js';
import { openSetupWizard } from './setup_wizard.js';

const STORE_KEY = 'trade_broker_id';
const ALL_SENTINEL = 'all';

const subscribers = new Set();
let cached = null; // { brokers: [], selected: 'all' | uuid }

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

/** Returns the active broker id, or null when "All". */
export function activeBrokerId() {
    const id = cached?.selected ?? readSaved();
    return id && id !== ALL_SENTINEL ? id : null;
}

// Expose a global sync accessor so `api.js::rq()` can thread the active
// broker into every endpoint without taking an import cycle on
// broker_context.js.
try { globalThis.__tvActiveBroker = activeBrokerId; } catch {}

/** Returns the active broker slug (e.g. "webull") or null. */
export async function activeBrokerSlug() {
    const id = activeBrokerId();
    if (!id) return null;
    const brokers = await listBrokers();
    const b = brokers.find((b) => b.id === id);
    return b?.slug || null;
}

export async function listBrokers() {
    if (cached) return cached.brokers;
    let brokers = [];
    try { brokers = await api.brokersList(); } catch (e) {
        console.warn('brokers fetch failed', e.message);
    }
    cached = { brokers, selected: readSaved() };
    return brokers;
}

/**
 * Invalidate the broker cache + notify subscribers so any view that
 * displays brokers (topbar selector, account-strip filter, dashboard
 * tiles) refetches. Call after a broker is created / edited / deleted
 * from anywhere — the management page, the wizard, etc. If the active
 * broker no longer exists in the fresh list, the selection falls back
 * to "All brokers".
 */
export async function refreshBrokers() {
    cached = null;
    const fresh = await listBrokers();
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
        try { fn(activeBrokerId()); } catch (e) { console.warn(e); }
    }
}

export function setActiveBroker(id) {
    const norm = id && id !== ALL_SENTINEL ? id : ALL_SENTINEL;
    if (cached) cached.selected = norm;
    writeSaved(norm);
    notify();
}

/**
 * Mount a `<select>` inside the host element. Switches active broker
 * on change, re-renders on add. Returns nothing — caller relies on
 * `onChange(fn)` for reactivity.
 */
export async function mountBrokerSelector(host) {
    const brokers = await listBrokers();
    const selected = activeBrokerId();
    const opts = [
        `<option value="${ALL_SENTINEL}"${!selected ? ' selected' : ''}>${esc(t('broker.all_brokers'))}</option>`,
        ...brokers.map((b) =>
            `<option value="${esc(b.id)}"${selected === b.id ? ' selected' : ''}>${esc(b.display_name)}</option>`),
        `<option value="__new__">${esc(t('broker.add_new'))}…</option>`,
    ].join('');
    // Flat select that matches `.account-select` — the previous wrapping
    // `<label>` + visible "Broker" hint rendered as a stacked two-line
    // pill in the topbar, breaking vertical centering against the
    // neighboring single-line controls. The aria-label retains the
    // accessibility hint without the visual stack.
    host.innerHTML = `<select class="broker-select account-select" aria-label="${esc(t('broker.label'))}">${opts}</select>`;
    const sel = host.querySelector('.broker-select');
    sel.addEventListener('change', async () => {
        if (sel.value === '__new__') {
            // Reset the visible selection BEFORE the modal opens so a
            // cancel doesn't leave the dropdown stuck on "+ New broker".
            const prev = activeBrokerId();
            sel.value = prev || ALL_SENTINEL;
            const created = await openSetupWizard({ kind: 'broker' });
            if (created) {
                cached = null;
                await listBrokers();
                setActiveBroker(created.id);
                showToast(t('broker.created', { name: created.display_name }), { level: 'success' });
            }
            await mountBrokerSelector(host);
            return;
        }
        setActiveBroker(sel.value);
        const label = sel.options[sel.selectedIndex]?.text || '';
        try { showToast(t('broker.switched', { name: label }), { level: 'info' }); } catch {}
    });
}

function esc(s) {
    return String(s).replace(/[&<>"]/g, (c) => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;',
    }[c]));
}
