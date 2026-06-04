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

let state = store.loadState();
let editMode = false;
let _wired = false;

export async function renderDashboards(mount, _appState) {
    state = store.loadState();
    mount.innerHTML = `
        <h1 data-i18n="view.dashboards.h1.dashboards" class="view-title">// DASHBOARDS</h1>
        <div class="db-shell">
            <aside id="db-sidebar" class="db-sidebar"></aside>
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
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.dashboards.chart.tile') },
            { label: t('view.dashboards.chart.usage'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 14, fill: '#ffd84a', stroke: '#ffd84a' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
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
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.dashboards.chart.dashboard') },
            { label: t('view.dashboards.chart.tiles'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
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

async function renderActive() {
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
        const filtered = all.filter(item => {
            if (!q) return true;
            const needle = q.toLowerCase();
            return item.label.toLowerCase().includes(needle)
                || (item.desc || '').toLowerCase().includes(needle);
        });
        grid.innerHTML = filtered.map(item =>
            `<button class="db-pick-tile" data-kind="${item.kind}" data-add="${esc(item.id)}" type="button"
                    title="${esc(item.desc)}">
                <span class="db-pick-glyph">${esc(item.glyph)}</span>
                <span class="db-pick-label">${esc(item.label)}</span>
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
    // Interleave drop-zones between every pair of tiles + at the head and
    // tail. Each drop-zone carries the destination index so the storage
    // layer's `moveTileTo` can resolve it deterministically.
    const dropZone = (idx) =>
        editMode ? `<div class="db-dropzone" data-drop-index="${idx}"></div>` : '';
    let html = dropZone(0);
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
        html += `
            <div class="db-tile ${editMode ? 'db-tile-draggable' : ''}"
                 data-tile-id="${esc(tile.id)}"
                 ${editMode ? `draggable="true"` : ''}>
                <div class="db-tile-head">
                    <span class="db-tile-glyph">${esc(meta.glyph)}</span>
                    <span class="db-tile-label">${esc(meta.label)}</span>
                    ${editMode ? `
                        <span class="db-tile-controls">
                            <span class="db-tile-drag" data-i18n-title="view.dashboards.tip.drag" title="Drag to reorder">⋮⋮</span>
                            <button class="db-tile-btn" data-move="up" data-tile="${esc(tile.id)}"   ${idx === 0 ? 'disabled' : ''} data-i18n-title="view.dashboards.tip.move_up" title="Move up">▲</button>
                            <button class="db-tile-btn" data-move="down" data-tile="${esc(tile.id)}" ${idx === dashboard.tiles.length - 1 ? 'disabled' : ''} data-i18n-title="view.dashboards.tip.move_down" title="Move down">▼</button>
                            <button class="db-tile-btn db-tile-remove" data-remove="${esc(tile.id)}" data-i18n-title="view.dashboards.tip.remove_tile" title="Remove tile">×</button>
                        </span>
                    ` : ''}
                </div>
                <div class="db-tile-body" id="db-tile-body-${esc(tile.id)}"></div>
            </div>
        `;
        html += dropZone(idx + 1);
    }
    gridEl.innerHTML = html;

    if (editMode) {
        gridEl.querySelectorAll('button[data-remove]').forEach(btn => {
            btn.addEventListener('click', async () => {
                state = store.removeTile(state, dashboard.id, btn.dataset.remove);
                persist();
                await renderActive();
            });
        });
        gridEl.querySelectorAll('button[data-move]').forEach(btn => {
            btn.addEventListener('click', async () => {
                const dir = btn.dataset.move === 'up' ? -1 : 1;
                state = store.moveTile(state, dashboard.id, btn.dataset.tile, dir);
                persist();
                await renderActive();
            });
        });
        wireDragAndDrop(dashboard);
    }

    // If any graph tile is present, fetch the analytics bundle ONCE and
    // reuse the same `data` for every widget render — same as the
    // analytics dashboard does.
    const hasGraph = dashboard.tiles.some(t => t.kind === 'graph');
    let analyticsBundle = null;
    if (hasGraph) {
        try {
            // The active account id lives in the app's state but we render
            // tiles in isolation; pull from the global. Falls back to
            // undefined which the API treats as "all accounts".
            const appState = (await import('../app.js')).state || {};
            analyticsBundle = await loadAnalyticsBundle(appState.accountId, 90);
        } catch (e) {
            console.warn('analytics bundle load failed', e);
        }
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

        const fn = renderers[tile.viewId];
        if (!fn) {
            body.innerHTML = `<div class="boot" style="color:var(--red)">${esc(t('view.dashboards.tile.err.not_found', { view: tile.viewId }))}</div>`;
            continue;
        }
        try {
            await fn(body, {});
        } catch (e) {
            body.innerHTML = `<div class="boot" style="color:var(--red)">${esc(t('view.dashboards.tile.err.render_failed', { err: String(e.message || e) }))}</div>`;
        }
    }
}


// HTML5 drag-and-drop wiring. Each .db-tile is the drag source and each
// .db-dropzone is the drop target. We use a dataTransfer payload of
// `tile-id:<id>` so cross-context drags (e.g. from the launcher) can be
// distinguished by prefix in a future cross-context handler.
function wireDragAndDrop(dashboard) {
    const grid = document.getElementById('db-grid');
    if (!grid) return;
    let draggingId = null;
    grid.querySelectorAll('.db-tile-draggable').forEach(tileEl => {
        tileEl.addEventListener('dragstart', (ev) => {
            draggingId = tileEl.dataset.tileId;
            tileEl.classList.add('db-tile-dragging');
            if (ev.dataTransfer) {
                ev.dataTransfer.effectAllowed = 'move';
                ev.dataTransfer.setData('text/plain', `tile-id:${draggingId}`);
            }
        });
        tileEl.addEventListener('dragend', () => {
            tileEl.classList.remove('db-tile-dragging');
            grid.querySelectorAll('.db-dropzone').forEach(z => z.classList.remove('db-dropzone-active'));
            draggingId = null;
        });
    });
    grid.querySelectorAll('.db-dropzone').forEach(zone => {
        zone.addEventListener('dragover', (ev) => {
            if (!draggingId) return;
            ev.preventDefault();
            if (ev.dataTransfer) ev.dataTransfer.dropEffect = 'move';
            zone.classList.add('db-dropzone-active');
        });
        zone.addEventListener('dragleave', () => {
            zone.classList.remove('db-dropzone-active');
        });
        zone.addEventListener('drop', async (ev) => {
            ev.preventDefault();
            zone.classList.remove('db-dropzone-active');
            if (!draggingId) return;
            const dropIndex = parseInt(zone.dataset.dropIndex, 10);
            if (!Number.isInteger(dropIndex)) return;
            state = store.moveTileTo(state, dashboard.id, draggingId, dropIndex);
            persist();
            draggingId = null;
            await renderActive();
        });
    });
}
