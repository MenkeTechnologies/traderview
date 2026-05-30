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
    GLOBAL_ITEMS, EDITING_ITEMS, positionMenu, compileMenu, mergeMenu,
    mergeMenuWithEditing, nextVisibleIdx,
} from './_context_menu.js';
import { t, applyUiI18n } from './i18n.js';
import { esc } from './util.js';
import { showToast } from './toast.js';
import { loadState, saveState, toggleFavorite, isFavorite, addBookmark } from './_favorites_storage.js';
import { getGlobalSymbol } from './_global_symbol.js';

let _installed = false;
let _open = false;
let _items = [];
let _selected = -1;
let _editingTarget = null;          // the input/textarea/CE the menu was opened on
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
        _editingTarget = isTextEntry(d.editingTarget) ? d.editingTarget : null;
        openAt(d.x || 0, d.y || 0, d.scope || null, !!_editingTarget);
    });
    window.addEventListener('tv:copy-view-url', () => {
        const url = window.location.href;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(url).then(
                () => showToast(t('toast.copied', { what: t('toast.what.url') }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
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
            showToast(t('toast.error.api', { err: t('toast.err.no_view') }), { level: 'error' });
            return;
        }
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(vid).then(
                () => showToast(t('toast.copied', { what: vid }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    window.addEventListener('tv:add-bookmark', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: t('toast.err.no_view') }), { level: 'error' });
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
    window.addEventListener('tv:edit-cut',        () => execEdit('cut'));
    window.addEventListener('tv:edit-copy',       () => execEdit('copy'));
    window.addEventListener('tv:edit-paste',      () => void execPaste());
    window.addEventListener('tv:edit-select-all', () => execEdit('selectAll'));
    window.addEventListener('tv:edit-undo',       () => execEdit('undo'));
    window.addEventListener('tv:edit-redo',       () => execEdit('redo'));
    window.addEventListener('tv:copy-symbol', () => {
        const sym = (getGlobalSymbol() || '').toUpperCase();
        if (!sym) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        if (navigator.clipboard && navigator.clipboard.writeText) {
            void navigator.clipboard.writeText(sym).then(
                () => showToast(t('toast.copied', { what: sym }), { level: 'success' }),
                () => showToast(t('toast.error.api', { err: t('toast.err.clipboard_denied') }), { level: 'error' }),
            );
        }
    });
    const navForSymbol = (viewId) => () => {
        const sym = (getGlobalSymbol() || '').toUpperCase();
        if (!sym) {
            showToast(t('toast.error.api', { err: t('toast.err.no_symbol') }), { level: 'error' });
            return;
        }
        window.location.hash = `${viewId}/${sym}`;
    };
    window.addEventListener('tv:open-charts-for-symbol',   navForSymbol('charts'));
    window.addEventListener('tv:open-options-for-symbol',  navForSymbol('options'));
    window.addEventListener('tv:open-research-for-symbol', navForSymbol('research'));
    window.addEventListener('tv:open-earnings-for-symbol', navForSymbol('earnings-iv'));
    // News view doesn't accept a hash-path symbol — it's filtered via
    // its own form. Navigate to the view; the user picks the symbol.
    window.addEventListener('tv:open-news-for-symbol',     () => { window.location.hash = 'news'; });
    window.addEventListener('tv:toggle-favorite', () => {
        const vid = currentViewId();
        if (!vid) {
            showToast(t('toast.error.api', { err: t('toast.err.no_view') }), { level: 'error' });
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
    e.preventDefault();
    const scope = nearestScope(e.target);
    const editing = isTextEntry(e.target) ? e.target : null;
    _editingTarget = editing;
    openAt(e.clientX, e.clientY, scope, !!editing);
}

function isTextEntry(el) {
    if (!el) return false;
    const tag = (el.tagName || '').toLowerCase();
    if (tag === 'textarea') return true;
    if (tag === 'input') {
        const type = (el.getAttribute('type') || 'text').toLowerCase();
        return !['button', 'submit', 'reset', 'checkbox', 'radio', 'file', 'image', 'range', 'color'].includes(type);
    }
    return !!el.isContentEditable;
}

// Run a synchronous edit command (cut/copy/selectAll/undo/redo). Falls
// back silently when the browser refuses (e.g. CSP / cross-origin).
// Target priority: ctx-menu target → document.activeElement (palette
// path) → no-op.
function execEdit(cmd) {
    const tgt = resolveEditTarget();
    if (!tgt) return;
    if (typeof tgt.focus === 'function') tgt.focus();
    try {
        if (typeof document.execCommand === 'function') document.execCommand(cmd);
    } catch (_) { /* ignored */ }
}

// Paste needs the async clipboard API in modern browsers; execCommand
// `paste` is widely blocked outside browser extensions.
async function execPaste() {
    const tgt = resolveEditTarget();
    if (!tgt) return;
    try {
        const txt = await navigator.clipboard.readText();
        if (typeof tgt.setRangeText === 'function' && typeof tgt.selectionStart === 'number') {
            const s = tgt.selectionStart, e = tgt.selectionEnd;
            tgt.setRangeText(txt, s, e, 'end');
            tgt.dispatchEvent(new Event('input', { bubbles: true }));
        } else if (tgt.isContentEditable) {
            tgt.focus();
            document.execCommand('insertText', false, txt);
        }
    } catch (_) { /* clipboard denied */ }
}

// Edit commands can fire from two sources: a right-click on an input
// (ctxmenu sets _editingTarget) or the command palette (no target —
// fall back to whatever has focus). Returns null when neither path
// yields a text-entry element.
function resolveEditTarget() {
    if (_editingTarget && isTextEntry(_editingTarget)) return _editingTarget;
    const ae = (typeof document !== 'undefined') ? document.activeElement : null;
    if (ae && isTextEntry(ae)) return ae;
    return null;
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

function openAt(x, y, scope, editing = false) {
    const custom = scope ? (_customByScope.get(scope) || []) : [];
    const merged = editing
        ? mergeMenuWithEditing(GLOBAL_ITEMS, custom, EDITING_ITEMS)
        : mergeMenu(GLOBAL_ITEMS, custom);
    _items = compileMenu(merged);
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
