// Command Palette overlay (Cmd+K). Listens for `tv:open-palette`,
// renders a centered overlay, filters TILES + favorites + bookmarks
// live, navigates by ↑↓ + Enter, dismisses by Escape.

import { TILES, CATEGORIES } from './views/launcher.js';
import { loadState } from './_favorites_storage.js';
import {
    buildTileItems, buildFavoriteItems, buildBookmarkItems, buildActionItems,
    categoriesByViewId, tilesByViewId,
    filterAndRank, highlightLabel, moveSelection,
} from './_command_palette_inputs.js';
import { listShortcuts } from './shortcuts.js';
import { formatKey } from './_shortcuts.js';
import { loadState as loadRecents, listRecents, buildRecentItems } from './_recents_storage.js';
import { t, applyUiI18n } from './i18n.js';
import { esc } from './util.js';

let _open = false;
let _query = '';
let _results = [];
let _selected = 0;

export function installCommandPalette() {
    ensureMount();
    window.addEventListener('tv:open-palette', open);
    window.addEventListener('tv:escape', () => { if (_open) close(); });
}

function ensureMount() {
    if (document.getElementById('palette-root')) return;
    const div = document.createElement('div');
    div.id = 'palette-root';
    document.body.appendChild(div);
}

function open() {
    if (_open) return;
    _open = true;
    _query = '';
    _selected = 0;
    paint();
    requestAnimationFrame(() => {
        const input = document.getElementById('palette-input');
        if (input) input.focus();
    });
}

function close() {
    _open = false;
    const root = document.getElementById('palette-root');
    if (root) root.innerHTML = '';
}

function paint() {
    const root = document.getElementById('palette-root');
    if (!root) return;
    // Capture caret position from the live input so a full re-render
    // (which rebuilds the <input> element) doesn't silently drop focus —
    // losing focus lets keystrokes bubble to the document and trip the
    // global symbol hotkey.
    const prevInput = document.getElementById('palette-input');
    const hadFocus = prevInput && document.activeElement === prevInput;
    const caret = prevInput ? prevInput.selectionStart : null;
    const items = buildAllItems();
    // No result cap — every tile must be reachable from the palette, so the
    // full ranked match set renders (the results list is scrollable). With a
    // query the match set is naturally small; an empty query lists everything.
    _results = filterAndRank(items, _query, items.length);
    if (_selected >= _results.length) _selected = 0;
    root.innerHTML = `
        <div class="palette-overlay" role="dialog" aria-modal="true" aria-label="${esc(t('palette.title'))}">
            <div class="palette-card">
                <input id="palette-input"
                       class="palette-input"
                       type="text"
                       autocomplete="off"
                       spellcheck="false"
                       placeholder="${esc(t('palette.placeholder'))}"
                       value="${esc(_query)}">
                <div class="palette-results" id="palette-results">
                    ${_results.length === 0
                        ? `<div class="palette-empty">${esc(t('palette.empty'))}</div>`
                        : _results.map(renderRow).join('')}
                </div>
                <div class="palette-hints">
                    <span>${esc(t('palette.hint.up_down'))}</span>
                    <span>${esc(t('palette.hint.enter'))}</span>
                    <span>${esc(t('palette.hint.esc'))}</span>
                </div>
            </div>
        </div>
    `;
    applyUiI18n(root);
    const input = document.getElementById('palette-input');
    if (input) {
        input.addEventListener('input', onInput);
        input.addEventListener('keydown', onInputKey);
        // Re-focus after the rebuild so typing stays in the palette
        // instead of leaking to the global symbol hotkey.
        if (hadFocus) {
            input.focus();
            if (caret != null) {
                try { input.setSelectionRange(caret, caret); } catch (_) { /* noop */ }
            }
        }
    }
    const list = document.getElementById('palette-results');
    if (list) {
        list.addEventListener('click', onRowClick);
    }
    const overlay = root.querySelector('.palette-overlay');
    if (overlay) {
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) close();
        });
    }
    repaintHighlight();
}

function renderRow(item, i) {
    const segs = highlightLabel(item.label, _query)
        .map(s => s.hit ? `<mark>${esc(s.ch)}</mark>` : esc(s.ch)).join('');
    const cat = item.category ? `<span class="palette-cat">${esc(item.category)}</span>` : '';
    // Action rows put the chip in <kbd> for visual hierarchy; view rows
    // keep the description in <span class="palette-hint">.
    const hintHtml = item.hint
        ? (item.kind === 'action'
            ? `<kbd class="palette-kbd">${esc(item.hint)}</kbd>`
            : `<span class="palette-hint">${esc(item.hint)}</span>`)
        : '';
    return `<div class="palette-row" data-idx="${i}" data-view="${esc(item.viewId)}">
        <span class="palette-icon">${esc(item.icon || '·')}</span>
        <span class="palette-label">${segs}</span>
        ${cat}
        ${hintHtml}
    </div>`;
}

function repaintHighlight() {
    const rows = document.querySelectorAll('.palette-row');
    rows.forEach((r, i) => r.classList.toggle('active', i === _selected));
    const sel = rows[_selected];
    if (sel && typeof sel.scrollIntoView === 'function') {
        sel.scrollIntoView({ block: 'nearest' });
    }
}

function onInput(e) {
    _query = e.target.value;
    _selected = 0;
    paint();
}

function onInputKey(e) {
    if (e.key === 'ArrowDown') { e.preventDefault(); _selected = moveSelection(_selected,  1, _results.length); repaintHighlight(); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); _selected = moveSelection(_selected, -1, _results.length); repaintHighlight(); }
    else if (e.key === 'Enter') {
        e.preventDefault();
        const it = _results[_selected];
        if (it) activate(it);
    }
    else if (e.key === 'Escape') {
        e.preventDefault();
        close();
    }
}

function onRowClick(e) {
    const row = e.target.closest('.palette-row');
    if (!row) return;
    const idx = parseInt(row.dataset.idx, 10);
    const it = _results[idx];
    if (it) activate(it);
}

function activate(it) {
    if (!it) return;
    close();
    if (it.kind === 'action' && it.actionKey) {
        window.dispatchEvent(new CustomEvent(it.actionKey, { detail: { source: 'palette', item: it } }));
        return;
    }
    if (!it.viewId) return;
    if (it.viewId !== window.location.hash.replace(/^#/, '').split('?')[0]) {
        window.location.hash = it.viewId;
    } else {
        // Same view re-selected — kick a re-render via hashchange-fire trick.
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    }
}

function buildAllItems() {
    const byVid = tilesByViewId(TILES);
    const cats = categoriesByViewId(CATEGORIES, t);
    const tileItems = buildTileItems(TILES, cats, t);
    let favs = [], bms = [];
    try {
        const fav = loadState();
        favs = buildFavoriteItems(fav.favorites || [], byVid, t);
        bms  = buildBookmarkItems(fav.bookmarks || [], byVid, t);
    } catch { /* favorites module unavailable; fine */ }
    let recents = [];
    try {
        const r = loadRecents();
        const currentView = (window.location.hash || '').replace(/^#/, '').split('/')[0];
        recents = buildRecentItems(listRecents(r, currentView), byVid, t);
    } catch { /* recents module unavailable; fine */ }
    const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);
    const actions = buildActionItems(listShortcuts(), t, (sc) => formatKey(sc, isMac));
    return [...recents, ...favs, ...bms, ...actions, ...tileItems];
}
