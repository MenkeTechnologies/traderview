// Right-click context-menu DOM glue.
//
// API:
//   installContextMenu()                 — listen for `contextmenu` events globally
//   registerContextItems(scope, items)   — view registers extra items shown only inside `scope`
//   tv:context-menu CustomEvent          — external code can request the menu programmatically
//
// The menu reads `data-context-scope` on the nearest ancestor; only
// scopes matching a registered set get their custom items merged in.

import {
    GLOBAL_ITEMS, positionMenu, compileMenu, mergeMenu, nextVisibleIdx,
} from './_context_menu.js';
import { t, applyUiI18n } from './i18n.js';
import { esc } from './util.js';
import { showToast } from './toast.js';
import { loadState, saveState, toggleFavorite, isFavorite, addBookmark } from './_favorites_storage.js';

let _installed = false;
let _open = false;
let _items = [];
let _selected = -1;
const _customByScope = new Map();   // scope-string → items[]

export function installContextMenu() {
    if (_installed) return;
    _installed = true;
    ensureMount();
    document.addEventListener('contextmenu', onContextMenu, { capture: true });
    document.addEventListener('click', onDocClick, { capture: true });
    document.addEventListener('keydown', onKey, { capture: true });
    window.addEventListener('tv:context-menu', (e) => {
        const d = e && e.detail || {};
        openAt(d.x || 0, d.y || 0, d.scope || null);
    });
    window.addEventListener('tv:copy-view-url', () => {
        const url = window.location.href;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(url).then(
                () => showToast(t('toast.copied', { what: 'URL' }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: 'clipboard denied' }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:nav-back', () => {
        if (typeof window.history?.back === 'function') window.history.back();
    });
    window.addEventListener('tv:reload', () => {
        // Force re-dispatch of the current view.
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    window.addEventListener('tv:open-new-tab', () => {
        window.open(window.location.href, '_blank', 'noopener,noreferrer');
    });
    window.addEventListener('tv:copy-view-id', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: 'no view' }), { level: 'error' });
            return;
        }
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(vid).then(
                () => showToast(t('toast.copied', { what: vid }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: 'clipboard denied' }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:add-bookmark', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: 'no view' }), { level: 'error' });
            return;
        }
        const name = (typeof window.prompt === 'function')
            ? window.prompt(t('prompt.bookmark_name', { view: vid }), vid)
            : vid;
        if (name == null) return;     // user cancelled
        const trimmed = String(name).trim();
        if (!trimmed) return;
        const state = loadState();
        const next = addBookmark(state, trimmed, vid);
        saveState(next);
        showToast(t('toast.bookmark_added', { name: trimmed }), { level: 'success' });
        window.dispatchEvent(new CustomEvent('tv:favorites-changed'));
    });
    window.addEventListener('tv:toggle-favorite', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: 'no view' }), { level: 'error' });
            return;
        }
        const state = loadState();
        const next = toggleFavorite(state, vid);
        saveState(next);
        const nowFav = isFavorite(next, vid);
        showToast(
            t(nowFav ? 'toast.favorite_added' : 'toast.favorite_removed', { view: vid }),
            { level: 'success' });
        window.dispatchEvent(new CustomEvent('tv:favorites-changed'));
    });
}

function currentViewId() {
    const h = (window.location.hash || '').replace(/^#/, '').split('?')[0];
    return h || null;
}

export function registerContextItems(scope, items) {
    if (!scope || !Array.isArray(items)) return;
    _customByScope.set(scope, items);
}

function ensureMount() {
    if (document.getElementById('tv-ctxmenu-root')) return;
    const root = document.createElement('div');
    root.id = 'tv-ctxmenu-root';
    document.body.appendChild(root);
}

function onContextMenu(e) {
    // Hold Shift to bypass and get native browser menu (escape hatch).
    if (e.shiftKey) return;
    // Don't intercept right-clicks inside text inputs (let the user paste).
    const tag = (e.target && e.target.tagName || '').toLowerCase();
    if (tag === 'input' || tag === 'textarea') return;
    e.preventDefault();
    const scope = nearestScope(e.target);
    openAt(e.clientX, e.clientY, scope);
}

function nearestScope(el) {
    let cur = el;
    while (cur && cur.nodeType === 1) {
        const s = cur.getAttribute && cur.getAttribute('data-context-scope');
        if (s) return s;
        cur = cur.parentNode;
    }
    return null;
}

function openAt(x, y, scope) {
    const custom = scope ? (_customByScope.get(scope) || []) : [];
    _items = compileMenu(mergeMenu(GLOBAL_ITEMS, custom));
    _selected = -1;
    _open = true;
    paint(x, y);
}

function paint(x, y) {
    const root = document.getElementById('tv-ctxmenu-root');
    if (!root) return;
    root.innerHTML = `
        <div class="tv-ctxmenu" role="menu" id="tv-ctxmenu">
            ${_items.map((it, i) => renderItem(it, i)).join('')}
        </div>
    `;
    applyUiI18n(root);
    const menu = root.querySelector('.tv-ctxmenu');
    if (!menu) return;
    const w = menu.offsetWidth  || 220;
    const h = menu.offsetHeight || 200;
    const pos = positionMenu(x, y, w, h,
        window.innerWidth || 1024, window.innerHeight || 768, 8);
    menu.style.left = pos.x + 'px';
    menu.style.top  = pos.y + 'px';
    menu.addEventListener('click', onItemClick);
    menu.addEventListener('mousemove', onHover);
}

function renderItem(it, idx) {
    if (it.kind === 'separator') return `<div class="tv-ctxmenu-sep"></div>`;
    return `<div class="tv-ctxmenu-item" role="menuitem"
                 data-idx="${idx}"
                 data-i18n="${esc(it.labelKey)}">${esc(t(it.labelKey))}</div>`;
}

function onItemClick(e) {
    const row = e.target.closest('.tv-ctxmenu-item');
    if (!row) return;
    const idx = parseInt(row.dataset.idx, 10);
    activate(_items[idx]);
}

function onHover(e) {
    const row = e.target.closest('.tv-ctxmenu-item');
    if (!row) return;
    _selected = parseInt(row.dataset.idx, 10);
    repaintSelection();
}

function repaintSelection() {
    document.querySelectorAll('.tv-ctxmenu-item').forEach((el, i) => {
        const idx = parseInt(el.dataset.idx, 10);
        el.classList.toggle('active', idx === _selected);
    });
}

function onDocClick(e) {
    if (!_open) return;
    const menu = document.getElementById('tv-ctxmenu');
    if (menu && menu.contains(e.target)) return;
    close();
}

function onKey(e) {
    if (!_open) return;
    if (e.key === 'Escape') { e.preventDefault(); close(); return; }
    if (e.key === 'ArrowDown') {
        e.preventDefault();
        _selected = nextVisibleIdx(_items, _selected, 1);
        repaintSelection();
    } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        _selected = nextVisibleIdx(_items, _selected, -1);
        repaintSelection();
    } else if (e.key === 'Enter') {
        e.preventDefault();
        if (_selected >= 0 && _selected < _items.length) activate(_items[_selected]);
    }
}

function activate(item) {
    if (!item) return;
    close();
    if (item.actionKey) {
        window.dispatchEvent(new CustomEvent(item.actionKey, { detail: { item } }));
    }
    if (item.navTo) {
        window.location.hash = item.navTo;
    }
    if (typeof item.onClick === 'function') {
        try { item.onClick(item); }
        catch (e) { console.error('ctxmenu onClick failed', e); }
    }
}

function close() {
    _open = false;
    const root = document.getElementById('tv-ctxmenu-root');
    if (root) root.innerHTML = '';
}
