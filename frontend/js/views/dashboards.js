// Dashboards view — user-customizable grids of any existing view.
//
// Architecture:
//   * Storage in localStorage via `_dashboards_storage.js` (pure, versioned).
//   * Sidebar lists saved dashboards; main area renders the active one.
//   * Edit mode reveals add-tile picker + per-tile remove + move buttons.
//   * Each tile mounts an existing view by ID via `tileRenderers` —
//     reused identically with the regular page renderers (paste textareas,
//     demo buttons, charts all work; the global ticker propagates through
//     `sym()` for symbol-aware views).

import { esc } from '../util.js';
import * as store from '../_dashboards_storage.js';
import * as favs from '../_favorites_storage.js';
import { TILES } from './launcher.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm, tPrompt } from '../dialog.js';
import { WIDGETS_BY_ID, loadAnalyticsBundle } from './dashboard.js';
import { searchScore, getMatchIndices, highlightWithIndices } from '../fzf.js';
import { initDragReorder, resetDragReorder } from '../drag_reorder.js';

// Re-export so the rest of the app can mount the same renderers in
// other contexts later (e.g., browser extensions, popups).
let TILE_RENDERERS = null;

// Lazily build a map from view-id → renderer function by inspecting the
// existing app.js dispatch table. We import on demand to avoid circular
// dependency (app.js imports this view, this view imports app.js for
// the renderers).
async function getRenderers() {
    if (TILE_RENDERERS) return TILE_RENDERERS;
    const app = await import('../app.js');
    TILE_RENDERERS = app.viewRenderers || {};
    return TILE_RENDERERS;
}

const TILE_INDEX = new Map(TILES.map(t => [t[0], { label: t[1], glyph: t[2], desc: t[3] }]));

// Views that own a WS connection, a sub-second rerender, or another
// hard-to-tear-down side effect. Putting more than one of these (or
// mixing them with other live tiles) makes the dashboard visibly flash
// as they fight for the module-level singletons (ws, setInterval handle).
// Show a "open standalone" hint instead of mounting.
// Only block views that would recursively mount THIS view inside itself —
// every other live view renders normally as a tile, even if its WS / poll
// owns module-level state. (The recursion was the actual flash source.)
const TILE_DENYLIST = new Set([
    'dashboards',
]);

let state = store.loadState();
let editMode = false;
let _wired = false;
// Sidebar collapse — persisted across sessions. Default expanded.
const SIDEBAR_COLLAPSED_KEY = 'tv:dashSidebarCollapsed';
let sidebarCollapsed = (() => {
    try { return localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === '1'; }
    catch { return false; }
})();

// Per-tile teardown callbacks returned by view renderers (squeeze_scanner,
// live_scanner, etc. that open WS + start polling). Without this, every
// renderTiles() re-mount stacks another WS + setTimeout chain on top of
// the previous one — visible as runaway /api/squeeze/candidates polling.
const tileTeardowns = new Map();
function tearDownAllTiles() {
    for (const fn of tileTeardowns.values()) {
        try { fn(); } catch (e) { console.warn('tile teardown threw', e); }
    }
    tileTeardowns.clear();
}

export async function renderDashboards(mount, _appState) {
    state = store.loadState();
    mount.innerHTML = `
        <h1 data-i18n="view.dashboards.h1.dashboards" class="view-title">// DASHBOARDS</h1>
        <div class="db-shell ${sidebarCollapsed ? 'db-shell--collapsed' : ''}" id="db-shell">
            <aside id="db-sidebar" class="db-sidebar"></aside>
            <button id="db-sidebar-toggle" class="db-sidebar-toggle" type="button"
                    title="${esc(t('view.dashboards.tip.toggle_sidebar') || 'Toggle sidebar')}"
                    aria-label="${esc(t('view.dashboards.tip.toggle_sidebar') || 'Toggle sidebar')}">
                <span class="db-sidebar-toggle-glyph">${sidebarCollapsed ? '›' : '‹'}</span>
            </button>
            <section id="db-main" class="db-main"></section>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.dashboards.h2.tiles_chart">Tiles per dashboard</h2>
            <div id="db-chart" style="width:100%;height:200px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.dashboards.h2.tile_freq_chart">Tile usage across dashboards (top 10)</h2>
            <div id="db-freq-chart" style="width:100%;height:200px"></div>
        </div>
    `;
    renderSidebar();
    await renderActive();
    renderTilesChart();
    renderTileFreqChart();
    // Sidebar collapse toggle — flips a class on the shell so the CSS
    // grid template changes column widths; persists across sessions.
    const toggleBtn = mount.querySelector('#db-sidebar-toggle');
    if (toggleBtn) {
        toggleBtn.addEventListener('click', () => {
            sidebarCollapsed = !sidebarCollapsed;
            try { localStorage.setItem(SIDEBAR_COLLAPSED_KEY, sidebarCollapsed ? '1' : '0'); } catch {}
            const shell = mount.querySelector('#db-shell');
            if (shell) shell.classList.toggle('db-shell--collapsed', sidebarCollapsed);
            const glyph = toggleBtn.querySelector('.db-sidebar-toggle-glyph');
            if (glyph) glyph.textContent = sidebarCollapsed ? '›' : '‹';
        });
    }
    if (!_wired) {
        _wired = true;
        // External mutations (e.g. launcher 📌 pin button) wake this view
        // so the active dashboard's tile list refreshes without a manual
        // reload. Only paints when this view is currently active.
        window.addEventListener('tv:dashboards-changed', () => {
            if ((window.location.hash || '').replace(/^#/, '').split('/')[0] !== 'dashboard'
                && (window.location.hash || '').replace(/^#/, '').split('/')[0] !== 'dashboards') return;
            state = store.loadState();
            renderSidebar();
            void renderActive();
        });
    }
}

function renderTileFreqChart() {
    const el = document.getElementById('db-freq-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const counts = new Map();
    for (const d of store.listDashboards(state) || []) {
        for (const tile of d.tiles || []) {
            const k = tile.viewId || tile.id || String(tile);
            counts.set(k, (counts.get(k) || 0) + 1);
        }
    }
    const rows = [...counts.entries()].map(([k, v]) => ({ k, v }))
        .sort((a, b) => b.v - a.v)
        .slice(0, 10);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.dashboards.empty_freq_chart">${esc(t('view.dashboards.empty_freq_chart'))}</div>`;
        return;
    }
    const labels = rows.map(r => r.k);
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(r => r.v);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.dashboards.chart.tile') },
            { label: t('view.dashboards.chart.usage'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 14, fill: '#ffd84a', stroke: '#ffd84a' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTilesChart() {
    const el = document.getElementById('db-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = store.listDashboards(state)
        .map(d => ({ name: d.name, n: d.tiles.length }))
        .filter(r => Number.isFinite(r.n));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.dashboards.empty_chart">${esc(t('view.dashboards.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => b.n - a.n);
    const labels = rows.map(r => r.name);
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(r => r.n);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.dashboards.chart.dashboard') },
            { label: t('view.dashboards.chart.tiles'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function persist() {
    store.saveState(state);
}

function renderSidebar() {
    const wrap = document.getElementById('db-sidebar');
    if (!wrap) return;
    const list = store.listDashboards(state);
    wrap.innerHTML = `
        <div class="db-sidebar-head" data-i18n="view.dashboards.sidebar.head">DASHBOARDS</div>
        <ul class="db-list">
            ${list.map(d => `
                <li class="db-list-item ${d.id === state.active ? 'db-active' : ''}"
                    data-context-scope="dashboard-sidebar-item"
                    data-id="${esc(d.id)}"
                    data-name="${esc(d.name)}">
                    <button class="db-pick" data-pick="${esc(d.id)}" type="button">${esc(d.name)}
                        <span class="muted">${d.tiles.length} tile${d.tiles.length === 1 ? '' : 's'}</span></button>
                </li>
            `).join('')}
        </ul>
        <div class="db-sidebar-actions">
            <input id="db-new-name" type="text" placeholder="new dashboard name" data-i18n-placeholder="view.dashboards.placeholder.new_name"
                   data-tip="view.dashboards.tip.new_name" data-shortcut="dashboards_focus_new">
            <button data-i18n="view.dashboards.btn.create" data-tip="view.dashboards.tip.create" id="db-new" class="primary" type="button">+ Create</button>
        </div>
        <hr>
        <div class="db-sidebar-actions">
            <button data-i18n="view.dashboards.btn.rename_active" data-tip="view.dashboards.tip.rename" id="db-rename"     class="secondary" type="button">Rename active</button>
            <button data-i18n="view.dashboards.btn.duplicate_active" data-tip="view.dashboards.tip.duplicate" id="db-duplicate"  class="secondary" type="button">Duplicate active</button>
            <button data-i18n="view.dashboards.btn.delete_active" data-tip="view.dashboards.tip.delete" id="db-delete"     class="secondary" type="button">Delete active</button>
            <button id="db-edit" data-tip="view.dashboards.tip.edit" data-shortcut="dashboards_toggle_edit"  class="${editMode ? 'primary' : 'secondary'}" type="button">${editMode ? t('view.dashboards.btn.done_editing') : t('view.dashboards.btn.edit_layout')}</button>
        </div>
        <hr>
        <div class="db-sidebar-actions">
            <button data-i18n="view.dashboards.btn.export_all_json" data-tip="view.dashboards.tip.export" id="db-export"     class="secondary" type="button">Export all (JSON)</button>
            <button data-i18n="view.dashboards.btn.import_json" data-tip="view.dashboards.tip.import" id="db-import"     class="secondary" type="button">Import (JSON)</button>
        </div>
        <hr>
        <div id="db-favs-section"></div>
        <p class="muted" data-i18n-html="view.dashboards.storage_hint">All dashboards saved in browser <code>localStorage</code> —
            no backend round-trip. Drag tiles in EDIT mode to reorder.</p>
    `;
    wrap.querySelectorAll('button[data-pick]').forEach(btn => {
        btn.addEventListener('click', () => {
            state = store.setActive(state, btn.dataset.pick);
            persist();
            renderSidebar();
            void renderActive();
        });
    });
    document.getElementById('db-new').addEventListener('click', async () => {
        const n = document.getElementById('db-new-name').value.trim();
        if (!n) {
            showToast(t('view.dashboards.alert.empty_name'), { level: 'warning' });
            return;
        }
        state = store.createDashboard(state, n);
        persist();
        document.getElementById('db-new-name').value = '';
        showToast(t('view.dashboards.toast.created', { name: n }), { level: 'success' });
        renderSidebar();
        await renderActive();
    });
    document.getElementById('db-rename').addEventListener('click', async () => {
        const d = store.getActiveDashboard(state);
        if (!d) return;
        const n = await tPrompt('view.dashboards.prompt.rename', {}, { defaultValue: d.name });
        if (!n || !n.trim()) return;
        state = store.renameDashboard(state, d.id, n);
        persist();
        renderSidebar();
        await renderActive();
    });
    document.getElementById('db-delete').addEventListener('click', async () => {
        const d = store.getActiveDashboard(state);
        if (!d) return;
        if (!await tConfirm('view.dashboards.confirm.delete_named', { name: d.name }, { level: 'danger' })) return;
        state = store.deleteDashboard(state, d.id);
        persist();
        showToast(t('view.dashboards.toast.deleted', { name: d.name }), { level: 'success' });
        renderSidebar();
        await renderActive();
    });
    document.getElementById('db-edit').addEventListener('click', async () => {
        editMode = !editMode;
        renderSidebar();
        await renderActive();
    });
    document.getElementById('db-duplicate').addEventListener('click', async () => {
        const d = store.getActiveDashboard(state);
        if (!d) return;
        state = store.duplicateDashboard(state, d.id);
        persist();
        showToast(t('view.dashboards.toast.duplicated', { name: d.name }), { level: 'success' });
        renderSidebar();
        await renderActive();
    });
    document.getElementById('db-export').addEventListener('click', () => {
        const json = store.exportState(state);
        downloadJsonFile('traderview-dashboards.json', json);
    });
    document.getElementById('db-import').addEventListener('click', async () => {
        const text = await tPrompt('view.dashboards.prompt.paste_import', {});
        if (!text) return;
        const next = store.importState(text);
        if (!next) {
            showToast(t('view.dashboards.alert.import_failed'), { level: 'error' });
            return;
        }
        if (!await tConfirm('view.dashboards.confirm.import_replace', {}, { level: 'warning' })) return;
        state = next;
        persist();
        renderSidebar();
        await renderActive();
    });
    renderFavsSection();
}

function downloadJsonFile(filename, json) {
    if (typeof window === 'undefined' || !window.URL || !document) return;
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    setTimeout(() => {
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }, 100);
}

function renderFavsSection() {
    const wrap = document.getElementById('db-favs-section');
    if (!wrap) return;
    const fState = favs.loadState();
    if (!fState.favorites.length && !fState.bookmarks.length) {
        wrap.innerHTML = `
            <div class="db-sidebar-head">${esc(t('view.dashboards.sidebar.favorites_empty'))}</div>
            <p data-i18n="view.dashboards.hint.no_favorites_yet_click_on_any_launcher_tile_to_fav" class="muted" style="font-size:11px">
                No favorites yet. Click ★ on any launcher tile to favorite it.
                Add this favorite view as a tile here with one click.
            </p>
        `;
        return;
    }
    const favTiles = fState.favorites
        .map(viewId => TILE_INDEX.get(viewId) ? { viewId, ...TILE_INDEX.get(viewId) } : null)
        .filter(Boolean);
    wrap.innerHTML = `
        <div class="db-sidebar-head">${esc(t('view.dashboards.sidebar.favorites', { count: favTiles.length }))}</div>
        <ul class="db-list">
            ${favTiles.map(f => `
                <li class="db-list-item">
                    <button class="db-pick" data-fav-add="${esc(f.viewId)}" type="button"
                            data-i18n-title="view.dashboards.tip.fav_add"
                            title="Click to add this favorite as a tile in the active dashboard">
                        <span>${esc(f.glyph)} ${esc(f.label)}</span>
                        <span class="muted" data-i18n="view.dashboards.label.add_tile">+ tile</span>
                    </button>
                </li>
            `).join('')}
        </ul>
        ${fState.bookmarks.length ? `
            <div class="db-sidebar-head" style="margin-top:8px">${esc(t('view.dashboards.sidebar.bookmarks', { count: fState.bookmarks.length }))}</div>
            <ul class="db-list">
                ${fState.bookmarks.map(b => `
                    <li class="db-list-item">
                        <button class="db-pick" data-bm-add="${esc(b.id)}" type="button"
                                data-i18n-title="view.dashboards.tip.bm_add"
                                title="Add bookmark as a configured tile">
                            <span>${esc(b.name)}</span>
                            <span class="muted">${esc(b.viewId)}</span>
                        </button>
                    </li>
                `).join('')}
            </ul>
        ` : ''}
    `;
    wrap.querySelectorAll('button[data-fav-add]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const active = store.getActiveDashboard(state);
            if (!active) return;
            state = store.addTile(state, active.id, btn.dataset.favAdd);
            persist();
            renderSidebar();
            await renderActive();
        });
    });
    wrap.querySelectorAll('button[data-bm-add]').forEach(btn => {
        btn.addEventListener('click', async () => {
            const active = store.getActiveDashboard(state);
            if (!active) return;
            const bm = favs.getBookmark(favs.loadState(), btn.dataset.bmAdd);
            if (!bm) return;
            state = store.addTile(state, active.id, bm.viewId, bm.config || {});
            persist();
            renderSidebar();
            await renderActive();
        });
    });
}

// Coalesce + drain renderActive() calls. Multiple sources can chain into
// renderActive in the same tick (addTile → persist → renderSidebar →
// renderActive, plus the tv:dashboards-changed listener). Naive serial
// invocation visibly flashes when any tile renderer is slow (the
// analytics-dashboard bundle fetch is the worst offender). The drain
// loop guarantees a final render after the last state change without
// running 60 redundant renders along the way.
// Coalesce + drain renderActive() calls so several chained handlers
// (addTile → persist → renderSidebar → renderActive plus the
// tv:dashboards-changed listener) collapse to one render — and any state
// change that lands DURING a render gets a follow-up pass.
let _renderRunning = false;
let _renderQueued = false;
async function renderActive() {
    if (_renderRunning) { _renderQueued = true; return; }
    _renderRunning = true;
    try {
        do {
            _renderQueued = false;
            await _renderActiveOnce();
        } while (_renderQueued);
    } finally {
        _renderRunning = false;
    }
}
async function _renderActiveOnce() {
    const wrap = document.getElementById('db-main');
    if (!wrap) return;
    const d = store.getActiveDashboard(state);
    if (!d) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.dashboards.empty.active">No active dashboard.</div>`;
        return;
    }
    const header = `
        <div class="db-main-head">
            <h2>${esc(d.name)}</h2>
            <span class="muted">${d.tiles.length} tile${d.tiles.length === 1 ? '' : 's'} · ${editMode ? 'EDIT mode' : 'view mode'}</span>
        </div>
    `;
    const picker = editMode ? renderPicker() : '';
    const grid = d.tiles.length
        ? `<div id="db-grid" class="db-grid"></div>`
        : `<div class="muted" data-i18n="view.dashboards.empty.tiles">No tiles yet. Click Edit layout in the sidebar then add tiles from the picker.</div>`;
    wrap.innerHTML = header + picker + grid;
    if (editMode) wirePicker(d);
    if (d.tiles.length) await renderTiles(d);
}

function renderPicker() {
    return `
        <div class="chart-panel db-picker">
            <h3 data-i18n="view.dashboards.h3.add_tile">+ Add tile</h3>
            <input id="db-pick-search" type="text" data-shortcut="focus_search" placeholder="filter views…" data-i18n-placeholder="view.dashboards.placeholder.filter" class="db-pick-search">
            <div id="db-pick-grid" class="db-pick-grid"></div>
        </div>
    `;
}

function wirePicker(activeDashboard) {
    const grid = document.getElementById('db-pick-grid');
    const search = document.getElementById('db-pick-search');
    // Build the graph-widget catalog once: every entry from the analytics
    // dashboard's WIDGETS_BY_ID, shaped so the same render path handles it
    // alongside the view tiles. Glyph 📊 distinguishes them in the picker.
    const GRAPH_ITEMS = [...WIDGETS_BY_ID.values()].map(w => ({
        kind: 'graph',
        id: w.id,
        label: t(w.titleKey) || w.id,
        glyph: '📊',
        desc: '',
    }));
    const renderGrid = (q) => {
        // Resolve label/desc through the i18n catalog so filter + display
        // both follow the active locale. Falls back to the TILES literal
        // when a key is absent.
        const tr = (key, fallback) => { const v = t(key); return (v && v !== key) ? v : fallback; };
        const viewItems = TILES.map(([id, label, glyph, desc]) => ({
            kind: 'view',
            id,
            label: tr(`tile.${id}.label`, label),
            glyph: glyph || '·',
            desc:  tr(`tile.${id}.desc`,  desc || ''),
        }));
        const all = [...GRAPH_ITEMS, ...viewItems];
        const ranked = q
            ? all
                .map(item => ({ item, score: searchScore(q, [item.label, item.desc || '']) }))
                .filter(x => x.score > 0)
                .sort((a, b) => b.score - a.score)
                .map(x => x.item)
            : all;
        const hlLabel = (s) => q ? highlightWithIndices(s, getMatchIndices(q, s)) : esc(s);
        grid.innerHTML = ranked.map(item =>
            `<button class="db-pick-tile" data-kind="${item.kind}" data-add="${esc(item.id)}" type="button"
                    title="${esc(item.desc)}">
                <span class="db-pick-glyph">${esc(item.glyph)}</span>
                <span class="db-pick-label">${hlLabel(item.label)}</span>
            </button>`
        ).join('') || `<div class="muted" data-i18n="view.dashboards.empty.no_views_match">No views match the filter.</div>`;
        grid.querySelectorAll('button[data-add]').forEach(btn => {
            btn.addEventListener('click', async () => {
                state = btn.dataset.kind === 'graph'
                    ? store.addGraphTile(state, activeDashboard.id, btn.dataset.add)
                    : store.addTile(state, activeDashboard.id, btn.dataset.add);
                persist();
                renderSidebar();
                await renderActive();
            });
        });
    };
    search.addEventListener('input', () => renderGrid(search.value.trim()));
    renderGrid('');
}

async function renderTiles(dashboard) {
    const gridEl = document.getElementById('db-grid');
    const renderers = await getRenderers();
    // Cancel WS / polling / intervals from the previous render BEFORE
    // we rewrite innerHTML — otherwise the orphaned setTimeout chains
    // keep firing and the squeeze/candidates poll runs unbounded.
    tearDownAllTiles();
    let html = '';
    for (let idx = 0; idx < dashboard.tiles.length; idx++) {
        const tile = dashboard.tiles[idx];
        // Graph tiles read meta from the analytics dashboard's WIDGETS
        // table; view tiles read from the launcher's TILES catalog.
        let meta;
        if (tile.kind === 'graph') {
            const w = WIDGETS_BY_ID.get(tile.graphId);
            const labelVal = w ? t(w.titleKey) : tile.graphId;
            meta = { label: labelVal, glyph: '📊' };
        } else {
            meta = TILE_INDEX.get(tile.viewId) || { label: tile.viewId, glyph: '·' };
        }
        // Saved span (config.size.col / .row) — defaults to 1×1.
        const size = (tile.config && tile.config.size) || { col: 1, row: 1 };
        const spanStyle = `grid-column:span ${size.col};grid-row:span ${size.row};`;
        html += `
            <div class="db-tile db-tile-draggable"
                 data-tile-id="${esc(tile.id)}"
                 data-col="${size.col}" data-row="${size.row}"
                 style="${spanStyle}">
                <div class="db-tile-head">
                    <span class="db-tile-grip" title="${esc(t('view.dashboards.tip.drag'))}" aria-hidden="true">⋮⋮</span>
                    <span class="db-tile-glyph">${esc(meta.glyph)}</span>
                    <span class="db-tile-label">${esc(meta.label)}</span>
                    <span class="db-tile-controls">
                        ${editMode ? `
                            <button class="db-tile-btn" data-move="up" data-tile="${esc(tile.id)}"   ${idx === 0 ? 'disabled' : ''} data-i18n-title="view.dashboards.tip.move_up" title="Move up">▲</button>
                            <button class="db-tile-btn" data-move="down" data-tile="${esc(tile.id)}" ${idx === dashboard.tiles.length - 1 ? 'disabled' : ''} data-i18n-title="view.dashboards.tip.move_down" title="Move down">▼</button>
                        ` : ''}
                        <button class="db-tile-btn db-tile-remove" data-remove="${esc(tile.id)}" data-i18n-title="view.dashboards.tip.remove_tile" title="${esc(t('view.dashboards.tip.remove_tile'))}">×</button>
                    </span>
                </div>
                <div class="db-tile-body" id="db-tile-body-${esc(tile.id)}"></div>
                <span class="db-tile-resize"
                      data-resize-tile="${esc(tile.id)}"
                      title="${esc(t('view.dashboards.tip.resize'))}"
                      aria-label="${esc(t('view.dashboards.tip.resize'))}"></span>
            </div>
        `;
    }
    gridEl.innerHTML = html;

    // Remove (×) is always available — no need to enter edit mode just to
    // delete a tile. Move-up / move-down arrows still only appear in edit
    // mode since drag-reorder covers the same need ergonomically.
    gridEl.querySelectorAll('button[data-remove]').forEach(btn => {
        btn.addEventListener('click', async (ev) => {
            ev.stopPropagation();
            state = store.removeTile(state, dashboard.id, btn.dataset.remove);
            persist();
            await renderActive();
        });
    });
    if (editMode) {
        gridEl.querySelectorAll('button[data-move]').forEach(btn => {
            btn.addEventListener('click', async () => {
                const dir = btn.dataset.move === 'up' ? -1 : 1;
                state = store.moveTile(state, dashboard.id, btn.dataset.tile, dir);
                persist();
                await renderActive();
            });
        });
    }
    // Drag-reorder + resize are always-on (not edit-mode gated). Pointer-
    // driven engine works through canvases (uPlot tiles) where the old
    // HTML5 dragstart/dragover/drop path was eaten by the chart.
    wireDragAndResize(dashboard);

    // If any graph tile is present, fetch the analytics bundle ONCE and
    // reuse the same `data` for every widget render — same as the
    // analytics dashboard does.
    const hasGraph = dashboard.tiles.some(t => t.kind === 'graph');
    let analyticsBundle = null;
    // Snapshot the active view token + active account BEFORE awaiting so
    // we can bail if the user navigates away (or switches accounts) mid-
    // fetch. Without this, a slow analytics bundle landing after the
    // user changed accounts paints the OLD account's graphs into the NEW
    // account's tile bodies — silent wrong-data display.
    const app = await import('../app.js');
    const tok = typeof app.currentViewToken === 'function' ? app.currentViewToken() : null;
    const appState = app.state || {};
    const renderAccountId = appState.accountId;
    if (hasGraph) {
        try {
            analyticsBundle = await loadAnalyticsBundle(renderAccountId, 90);
        } catch (e) {
            console.warn('analytics bundle load failed', e);
        }
        if (tok != null && typeof app.viewIsCurrent === 'function' && !app.viewIsCurrent(tok)) {
            return;
        }
        // Account change during fetch — caller's render is stale.
        if ((app.state || {}).accountId !== renderAccountId) return;
    }

    // Mount each tile into its body. Failures in one tile don't block
    // others — render is wrapped so a broken tile shows an inline error
    // rather than blanking the whole dashboard.
    for (const tile of dashboard.tiles) {
        const body = document.getElementById(`db-tile-body-${tile.id}`);
        if (!body) continue;

        if (tile.kind === 'graph') {
            const widget = WIDGETS_BY_ID.get(tile.graphId);
            if (!widget) {
                body.innerHTML = `<div class="boot" style="color:var(--red)">${esc(t('view.dashboards.tile.err.not_found', { view: tile.graphId }))}</div>`;
                continue;
            }
            if (!analyticsBundle) {
                body.innerHTML = `<div class="boot" style="color:var(--red)">${esc(t('view.dashboards.tile.err.render_failed', { err: 'analytics data unavailable' }))}</div>`;
                continue;
            }
            try {
                body.innerHTML = widget.html(analyticsBundle.data);
                if (typeof widget.mount === 'function') {
                    widget.mount(analyticsBundle.data, body);
                }
            } catch (e) {
                body.innerHTML = `<div class="boot" style="color:var(--red)">${esc(t('view.dashboards.tile.err.render_failed', { err: String(e.message || e) }))}</div>`;
            }
            continue;
        }

        if (TILE_DENYLIST.has(tile.viewId)) {
            // Self-recursive view (mounting Dashboards inside a tile inside
            // Dashboards recurses infinitely — verified via stack trace).
            // No other views are blocked here.
            body.innerHTML = `<div class="boot" style="padding:14px;text-align:center;color:var(--text-muted, #aab);font-size:12px">
                Can't embed the Dashboards view inside itself.
            </div>`;
            continue;
        }
        const fn = renderers[tile.viewId];
        if (!fn) {
            body.innerHTML = `<div class="boot" style="color:var(--red)">${esc(t('view.dashboards.tile.err.not_found', { view: tile.viewId }))}</div>`;
            continue;
        }
        try {
            // Pass the REAL app state — many renderers do `state.accounts.find(...)`
            // and crash on an empty object.
            const teardown = await fn(body, appState);
            // Renderers that return a teardown (squeeze_scanner, live_scanner,
            // any WS/poll-owning view) get cleaned up on the next renderTiles().
            if (typeof teardown === 'function') {
                tileTeardowns.set(tile.id, teardown);
            }
        } catch (e) {
            body.innerHTML = `<div class="boot" style="color:var(--red)">${esc(t('view.dashboards.tile.err.render_failed', { err: String(e.message || e) }))}</div>`;
        }
    }
}


// HTML5 drag-and-drop wiring. Each .db-tile is the drag source and each
// .db-dropzone is the drop target. We use a dataTransfer payload of
// `tile-id:<id>` so cross-context drags (e.g. from the launcher) can be
// distinguished by prefix in a future cross-context handler.
// Always-on Trello-style reorder + corner resize. State persistence flows
// through _dashboards_storage (reorderTiles / setTileSize) so the tile
// order + sizes survive view re-renders, dashboard switches, and reloads.
function wireDragAndResize(dashboard) {
    const grid = document.getElementById('db-grid');
    if (!grid) return;
    // Re-arm the drag engine after every render — the grid div is the
    // same element across renders (innerHTML replaces children, not the
    // host), so the `_trelloDragInit` guard would otherwise block a re-init.
    resetDragReorder(grid);
    initDragReorder(grid, '.db-tile', null, {
        direction: 'horizontal',
        // Only the grip / glyph / label start a drag — NOT the controls
        // (× delete, ▲/▼ move) or the body. Without this restriction
        // mousedown on a button triggered drag-init + preventDefault,
        // which suppressed the subsequent click event so the button
        // appeared dead.
        handleSelector: '.db-tile-grip, .db-tile-glyph, .db-tile-label',
        getKey: (el) => el.dataset.tileId,
        // Push new order into authoritative dashboard state. The drag
        // engine already moved the DOM; we mirror that into state and
        // skip a re-render (DOM is already correct).
        onReorder: (newKeys) => {
            const next = store.reorderTiles(state, dashboard.id, newKeys);
            if (next !== state) {
                state = next;
                persist();
            }
        },
        toastKey: 'toast.reordered',
    });

    // Corner-handle resize → grid-column / grid-row span.
    grid.querySelectorAll('.db-tile-resize').forEach(handle => {
        // Capture phase + stopImmediatePropagation so this fires BEFORE the
        // drag-engine's bubble-phase mousedown on the grid container, and
        // before any sibling click handlers — guarantees resize wins over
        // drag-reorder when the user grabs the corner.
        handle.addEventListener('mousedown', (ev) => {
            ev.preventDefault();
            ev.stopImmediatePropagation();
            const tileEl = handle.closest('.db-tile');
            if (!tileEl) return;
            const tileId = tileEl.dataset.tileId;
            // Approximate cell size from the tile's current rendered
            // footprint divided by its current span — robust against
            // future grid template tweaks.
            const startCol = parseInt(tileEl.dataset.col, 10) || 1;
            const startRow = parseInt(tileEl.dataset.row, 10) || 1;
            const rect = tileEl.getBoundingClientRect();
            const cellW = rect.width / startCol;
            const cellH = rect.height / startRow;
            const startX = ev.clientX, startY = ev.clientY;

            tileEl.classList.add('db-tile-resizing');

            const onMove = (mv) => {
                const dx = mv.clientX - startX;
                const dy = mv.clientY - startY;
                const col = Math.max(1, Math.min(6, startCol + Math.round(dx / cellW)));
                const row = Math.max(1, Math.min(6, startRow + Math.round(dy / cellH)));
                tileEl.style.gridColumn = `span ${col}`;
                tileEl.style.gridRow = `span ${row}`;
                tileEl.dataset.col = String(col);
                tileEl.dataset.row = String(row);
            };
            const onUp = () => {
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup', onUp);
                tileEl.classList.remove('db-tile-resizing');
                const col = parseInt(tileEl.dataset.col, 10) || 1;
                const row = parseInt(tileEl.dataset.row, 10) || 1;
                const next = store.setTileSize(state, dashboard.id, tileId, { col, row });
                if (next !== state) {
                    state = next;
                    persist();
                }
            };
            document.addEventListener('mousemove', onMove);
            document.addEventListener('mouseup', onUp);
        }, /* capture */ true);
    });
}
