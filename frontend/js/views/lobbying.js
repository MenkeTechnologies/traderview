// Per-symbol lobbying disclosures — Finnhub /stock/lobbying. Free tier.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderLobbying(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lobbying.h1.title">// LOBBYING DISCLOSURES</span></h1>
        <p class="muted small" data-i18n="view.lobbying.hint.intro">
            Per-symbol Senate lobbying disclosure filings (LD-1, LD-2). Surfaces
            issues a company is lobbying on + dollar spend — useful for catalyst
            anticipation around regulatory events.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="lob-form">
                <label><span data-i18n="view.lobbying.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <button class="primary" type="submit" data-i18n="view.lobbying.btn.load">Load</button>
            </form>
            <div id="lob-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('lob-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('lob-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const from = new Date();
        const to = new Date(from);
        from.setDate(from.getDate() - 365);
        const data = await api.symbolLobbying(state.symbol, fmtDay(from), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.lobbying.empty">No lobbying disclosures.</p>`;
            return;
        }
        const totalSpend = rows.reduce((s, r) => s + (Number(r.income) || Number(r.expenses) || 0), 0);
        el.innerHTML = `
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.lobbying.card.filings">Filings (last 1y)</div>
                    <div class="value">${rows.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.lobbying.card.total_spend">Total spend</div>
                    <div class="value">${totalSpend ? '$' + totalSpend.toLocaleString() : '—'}</div>
                </div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.lobbying.th.date">Date</th>
                    <th data-i18n="view.lobbying.th.client">Client</th>
                    <th data-i18n="view.lobbying.th.amount">Amount</th>
                    <th data-i18n="view.lobbying.th.period">Period</th>
                    <th data-i18n="view.lobbying.th.description">Description</th>
                </tr></thead>
                <tbody>${rows.slice(0, 50).map(r => `
                    <tr>
                        <td class="muted">${esc(r.postedDate || r.year || '—')}</td>
                        <td>${esc(r.clientName || r.name || '—')}</td>
                        <td>${(Number(r.income) || Number(r.expenses) || 0)
                            ? '$' + Number(r.income || r.expenses).toLocaleString() : '—'}</td>
                        <td class="muted">${esc(r.period || '—')}</td>
                        <td class="muted">${esc((r.description || r.specificIssues || '').slice(0, 120))}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.lobbying.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.lobbying.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
