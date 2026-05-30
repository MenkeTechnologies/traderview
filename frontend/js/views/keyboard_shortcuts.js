// Keyboard shortcuts cheat-sheet view. Lists every registered shortcut
// from the shortcuts registry, searchable by id / action / key.
// Reachable via `?` (tv:open-help binds to navigate here), the topbar
// "?" link, or the context menu's "Keyboard Shortcuts…" item.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { listShortcuts } from '../shortcuts.js';
import { formatKey } from '../_shortcuts.js';
import { GLOBAL_ITEMS, ALL_SCOPED_ITEMS } from '../_context_menu.js';

let _query = '';

export async function renderKeyboardShortcuts(mount, _appState) {
    const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);
    mount.innerHTML = `
        <h1 data-i18n="view.keyboard_shortcuts.title" class="view-title">// KEYBOARD SHORTCUTS</h1>

        <div class="chart-panel">
            <div class="inline-form">
                <input id="ks-filter" type="text" autocomplete="off" spellcheck="false"
                       data-i18n-placeholder="view.keyboard_shortcuts.placeholder"
                       placeholder="Filter shortcuts…"
                       value="${esc(_query)}"
                       style="flex:1 0 200px">
            </div>
            <p data-i18n="view.keyboard_shortcuts.hint" class="muted">Type to filter by id, action, or key. Rebind via Preferences (coming soon).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.keyboard_shortcuts.section.shortcuts">Keyboard shortcuts</h2>
            <div id="ks-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.keyboard_shortcuts.section.context_menu">Right-click context menu</h2>
            <p data-i18n="view.keyboard_shortcuts.context_hint" class="muted">Right-click anywhere (or hold Shift to get the browser default) to open the menu. Hold Shift then right-click to escape to the native menu.</p>
            <div id="ks-ctxmenu"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.keyboard_shortcuts.section.scoped_ctx">Scope-specific context menus</h2>
            <p data-i18n="view.keyboard_shortcuts.scoped_hint" class="muted">Right-click a row or tile to get scope-specific actions on top of the global ones above.</p>
            <div id="ks-scoped-ctx"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.keyboard_shortcuts.section.scope_chart">Shortcuts per scope</h2>
            <div id="ks-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    const input = document.getElementById('ks-filter');
    if (input) {
        input.addEventListener('input', (e) => { _query = e.target.value; paint(isMac); });
        input.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && input.value) {
                e.stopPropagation();   // don't let the global tv:escape also fire
                input.value = '';
                _query = '';
                paint(isMac);
            }
        });
        requestAnimationFrame(() => input.focus());
    }
    paint(isMac);
}

function paint(isMac) {
    paintShortcuts(isMac);
    paintCtxMenu();
    paintScopedCtx();
    paintScopeChart();
}

function paintScopeChart() {
    const el = document.getElementById('ks-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const all = listShortcuts();
    if (!all.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.keyboard_shortcuts.empty_chart">${esc(t('view.keyboard_shortcuts.empty_chart'))}</div>`;
        return;
    }
    const counts = new Map();
    for (const sc of all) {
        const scope = sc.scope || 'global';
        counts.set(scope, (counts.get(scope) || 0) + 1);
    }
    const pairs = Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
    const labels = pairs.map(([k]) => k);
    const ys = pairs.map(([, n]) => n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.keyboard_shortcuts.chart.scope_idx') },
            { label: t('view.keyboard_shortcuts.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function paintShortcuts(isMac) {
    const wrap = document.getElementById('ks-table');
    if (!wrap) return;
    const all = listShortcuts();
    const q = _query.trim().toLowerCase();
    const filtered = all.filter(sc => {
        if (!q) return true;
        const blob = [sc.id, sc.descKey, sc.scope, formatKey(sc, isMac)].join(' ').toLowerCase();
        return blob.includes(q);
    });
    if (filtered.length === 0) {
        const key = q ? 'view.keyboard_shortcuts.empty_filter' : 'view.keyboard_shortcuts.empty';
        wrap.innerHTML = `<div class="muted" data-i18n="${key}">${esc(t(key))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.keyboard_shortcuts.column.key">Key</th>
                <th data-i18n="view.keyboard_shortcuts.column.action">Action</th>
                <th data-i18n="view.keyboard_shortcuts.column.scope">Scope</th>
                <th data-i18n="view.keyboard_shortcuts.column.id">Id</th>
            </tr></thead>
            <tbody>
                ${filtered.map(sc => `<tr>
                    <td><kbd>${esc(formatKey(sc, isMac))}</kbd></td>
                    <td>${sc.descKey ? esc(t(sc.descKey)) : ''}</td>
                    <td class="muted">${esc(sc.scope || 'global')}</td>
                    <td class="muted"><code>${esc(sc.id)}</code></td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function paintScopedCtx() {
    const wrap = document.getElementById('ks-scoped-ctx');
    if (!wrap) return;
    const q = _query.trim().toLowerCase();
    // Flatten ALL_SCOPED_ITEMS [scope, items] pairs into one row per
    // (scope, item) so the filter operates uniformly.
    const rows = [];
    for (const [scope, items] of ALL_SCOPED_ITEMS) {
        for (const it of items) {
            if (it && it.kind !== 'separator') rows.push({ scope, item: it });
        }
    }
    const filtered = rows.filter(({ scope, item }) => {
        if (!q) return true;
        const label = item.labelKey ? t(item.labelKey) : '';
        const blob = [scope, item.id, item.labelKey, label, item.actionKey || '']
            .join(' ').toLowerCase();
        return blob.includes(q);
    });
    if (filtered.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.keyboard_shortcuts.empty_filter">${esc(t('view.keyboard_shortcuts.empty_filter'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.keyboard_shortcuts.column.scope">Scope</th>
                <th data-i18n="view.keyboard_shortcuts.column.label">Label</th>
                <th data-i18n="view.keyboard_shortcuts.column.action">Action</th>
                <th data-i18n="view.keyboard_shortcuts.column.id">Id</th>
            </tr></thead>
            <tbody>
                ${filtered.map(({ scope, item }) => `<tr>
                    <td class="muted"><code>${esc(scope)}</code></td>
                    <td>${item.labelKey ? esc(t(item.labelKey)) : ''}</td>
                    <td class="muted"><code>${esc(item.actionKey || '')}</code></td>
                    <td class="muted"><code>${esc(item.id || '')}</code></td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function paintCtxMenu() {
    const wrap = document.getElementById('ks-ctxmenu');
    if (!wrap) return;
    const q = _query.trim().toLowerCase();
    const items = GLOBAL_ITEMS.filter(it => it && it.kind !== 'separator');
    const filtered = items.filter(it => {
        if (!q) return true;
        const label = it.labelKey ? t(it.labelKey) : '';
        const blob = [it.id, it.labelKey, label, it.actionKey || '', it.navTo || '', it.section || '']
            .join(' ').toLowerCase();
        return blob.includes(q);
    });
    if (filtered.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.keyboard_shortcuts.empty_filter">${esc(t('view.keyboard_shortcuts.empty_filter'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.keyboard_shortcuts.column.label">Label</th>
                <th data-i18n="view.keyboard_shortcuts.column.action">Action</th>
                <th data-i18n="view.keyboard_shortcuts.column.section">Section</th>
                <th data-i18n="view.keyboard_shortcuts.column.id">Id</th>
            </tr></thead>
            <tbody>
                ${filtered.map(it => `<tr>
                    <td>${it.labelKey ? esc(t(it.labelKey)) : ''}</td>
                    <td class="muted"><code>${esc(it.actionKey || (it.navTo ? `#${it.navTo}` : ''))}</code></td>
                    <td class="muted">${esc(it.section || '')}</td>
                    <td class="muted"><code>${esc(it.id || '')}</code></td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}
