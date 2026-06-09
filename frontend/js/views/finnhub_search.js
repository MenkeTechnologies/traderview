// Universal Finnhub Symbol Search — /search consumer. Lookup any ticker
// across global exchanges and jump to research.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { q: '', rows: [] };

export async function renderFinnhubSearch(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.finnhub_search.h1.title">// SYMBOL SEARCH</span></h1>
        <p class="muted small" data-i18n="view.finnhub_search.hint.intro">
            Look up any ticker globally — Finnhub /search. Click a row to open research.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="fs-form">
                <label><span data-i18n="view.finnhub_search.label.query">Query</span>
                    <input type="text" name="q" value="${esc(state.q)}" placeholder="apple, nvda, voo" required></label>
                <button class="primary" type="submit" data-i18n="view.finnhub_search.btn.search">Search</button>
            </form>
            <div id="fs-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fs-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.q = (fd.get('q') || '').trim();
        void load(tok);
    });
    if (state.q) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('fs-result');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.finnhubSymbolLookup(state.q);
        if (!viewIsCurrent(tok)) return;
        const rows = data?.result || [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.finnhub_search.empty">No matches.</p>`;
            return;
        }
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.finnhub_search.th.symbol">Symbol</th>
                <th data-i18n="view.finnhub_search.th.display">Display</th>
                <th data-i18n="view.finnhub_search.th.description">Description</th>
                <th data-i18n="view.finnhub_search.th.type">Type</th>
            </tr></thead>
            <tbody>${rows.slice(0, 100).map(r => `
                <tr>
                    <td><a class="link" href="#research/${esc(r.symbol || '')}">${esc(r.symbol || '—')}</a></td>
                    <td>${esc(r.displaySymbol || '—')}</td>
                    <td class="muted">${esc(r.description || '—')}</td>
                    <td class="muted">${esc(r.type || '—')}</td>
                </tr>
            `).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.finnhub_search.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.finnhub_search.toast.failed'), { level: 'error' });
    }
}
