// Keyboard shortcuts cheat-sheet view. Lists every registered shortcut
// from the shortcuts registry, searchable by id / action / key.
// Reachable via `?` (tv:open-help binds to navigate here), the topbar
// "?" link, or the context menu's "Keyboard Shortcuts…" item.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { listShortcuts } from '../shortcuts.js';
import { formatKey } from '../_shortcuts.js';
import { GLOBAL_ITEMS } from '../_context_menu.js';

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
