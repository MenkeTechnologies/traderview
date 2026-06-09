// Symbol Changes — Finnhub /ca/symbol-change consumer. Recent ticker
// renames + ISIN changes. Useful for catching corporate actions that
// would otherwise break watchlists silently.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const HORIZONS = [
    { value: 30,  key: 'view.symbol_changes.horizon.last_30_days' },
    { value: 90,  key: 'view.symbol_changes.horizon.last_90_days' },
    { value: 180, key: 'view.symbol_changes.horizon.last_180_days' },
];

let state = { horizon: 90 };

export async function renderSymbolChanges(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.symbol_changes.h1.title">// SYMBOL + ISIN CHANGES</span></h1>
        <p class="muted small" data-i18n="view.symbol_changes.hint.intro">
            Recent ticker renames + ISIN changes from Finnhub. Audit your watchlists
            here whenever you see "no data" on a position that should exist.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.symbol_changes.label.horizon">Horizon</span>
                    <select id="sc-horizon">${HORIZONS.map(h =>
                        `<option value="${h.value}" ${h.value === state.horizon ? 'selected' : ''}>${esc(t(h.key))}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="sc-refresh" type="button" data-i18n="view.symbol_changes.btn.refresh">Refresh</button>
            </div>
            <div class="panel-grid" style="margin-top:10px">
                <div class="chart-panel">
                    <h2 data-i18n="view.symbol_changes.h2.symbols">Symbol changes</h2>
                    <div id="sc-symbols"></div>
                </div>
                <div class="chart-panel">
                    <h2 data-i18n="view.symbol_changes.h2.isins">ISIN changes</h2>
                    <div id="sc-isins"></div>
                </div>
            </div>
        </div>
    `;
    document.getElementById('sc-horizon').addEventListener('change', e => {
        state.horizon = Number(e.target.value);
        void load(tok);
    });
    document.getElementById('sc-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}

async function load(tok) {
    const today = new Date();
    const from = new Date(today);
    from.setDate(from.getDate() - state.horizon);
    const [symEl, isinEl] = ['sc-symbols', 'sc-isins'].map(id => document.getElementById(id));
    if (symEl) symEl.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    if (isinEl) isinEl.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const [symData, isinData] = await Promise.all([
            api.finnhubSymbolChange(fmtDay(from), fmtDay(today)).catch(() => null),
            api.finnhubIsinChange(fmtDay(from), fmtDay(today)).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderSymbolTable(symEl, symData?.data || []);
        renderIsinTable(isinEl, isinData?.data || []);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.symbol_changes.toast.failed'), { level: 'error' });
    }
}

function renderSymbolTable(el, rows) {
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.symbol_changes.empty">No symbol changes in window.</p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.symbol_changes.th.date">Date</th>
            <th data-i18n="view.symbol_changes.th.from">From</th>
            <th data-i18n="view.symbol_changes.th.to">To</th>
            <th data-i18n="view.symbol_changes.th.name">Name</th>
        </tr></thead>
        <tbody>${rows.map(r => `
            <tr>
                <td class="muted">${esc(r.atDate || '—')}</td>
                <td>${esc(r.oldSymbol || '—')}</td>
                <td><a class="link" href="#research/${esc(r.newSymbol || '')}">${esc(r.newSymbol || '—')}</a></td>
                <td class="muted">${esc(r.name || '—')}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function renderIsinTable(el, rows) {
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.symbol_changes.empty_isin">No ISIN changes in window.</p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.symbol_changes.th.date_2">Date</th>
            <th data-i18n="view.symbol_changes.th.old_isin">Old ISIN</th>
            <th data-i18n="view.symbol_changes.th.new_isin">New ISIN</th>
            <th data-i18n="view.symbol_changes.th.symbol">Symbol</th>
        </tr></thead>
        <tbody>${rows.map(r => `
            <tr>
                <td class="muted">${esc(r.atDate || '—')}</td>
                <td class="muted">${esc(r.oldIsin || '—')}</td>
                <td>${esc(r.newIsin || '—')}</td>
                <td>${esc(r.symbol || '—')}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}
