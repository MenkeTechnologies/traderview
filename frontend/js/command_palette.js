// Command Palette overlay (Cmd+K). Listens for `tv:open-palette`,
// renders a centered overlay, filters TILES + favorites + bookmarks
// live, navigates by ↑↓ + Enter, dismisses by Escape.

import { TILES, CATEGORIES } from './views/launcher.js';
import { loadState } from './_favorites_storage.js';
import {
    buildTileItems, buildFavoriteItems, buildBookmarkItems,
    categoriesByViewId, tilesByViewId,
    filterAndRank, highlightLabel, moveSelection,
} from './_command_palette_inputs.js';
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
    const items = buildAllItems();
    _results = filterAndRank(items, _query, 50);
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
    return `<div class="palette-row" data-idx="${i}" data-view="${esc(item.viewId)}">
        <span class="palette-icon">${esc(item.icon || '·')}</span>
        <span class="palette-label">${segs}</span>
        ${cat}
        ${item.hint ? `<span class="palette-hint">${esc(item.hint)}</span>` : ''}
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
    if (!it || !it.viewId) return;
    close();
    if (it.viewId !== window.location.hash.replace(/^#/, '').split('?')[0]) {
        window.location.hash = it.viewId;
    } else {
        // Same view re-selected — kick a re-render via hashchange-fire trick.
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    }
}

function buildAllItems() {
    const byVid = tilesByViewId(TILES);
    const cats = categoriesByViewId(CATEGORIES);
    const tileItems = buildTileItems(TILES, cats);
    let favs = [], bms = [];
    try {
        const fav = loadState();
        favs = buildFavoriteItems(fav.favorites || [], byVid);
        bms  = buildBookmarkItems(fav.bookmarks || [], byVid);
    } catch { /* favorites module unavailable; fine */ }
    return [...favs, ...bms, ...tileItems];
}
