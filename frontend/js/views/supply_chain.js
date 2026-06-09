// Supply Chain Relationships — Finnhub /stock/supply-chain.
// Surfaces customer + supplier exposure: when AAPL drops, sympathy hits
// its supply chain (chip suppliers, etc).

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderSupplyChain(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.supply_chain.h1.title">// SUPPLY CHAIN</span></h1>
        <p class="muted small" data-i18n="view.supply_chain.hint.intro">
            Customers + suppliers per symbol. Sympathy-play hunting ground — when a hub
            stock moves, its supply chain often follows. <strong>Premium endpoint</strong>.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="sc-form">
                <label><span data-i18n="view.supply_chain.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <button class="primary" type="submit" data-i18n="view.supply_chain.btn.load">Load</button>
            </form>
            <div id="sc-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('sc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('sc-result');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.symbolSupplyChain(state.symbol);
        if (!viewIsCurrent(tok)) return;
        const rels = data?.data || [];
        if (!rels.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.supply_chain.empty">No supply-chain data (or premium required).</p>`;
            return;
        }
        const customers = rels.filter(r => /customer/i.test(r.entityCategory || r.relationship || ''));
        const suppliers = rels.filter(r => /supplier|vendor/i.test(r.entityCategory || r.relationship || ''));
        const others = rels.filter(r =>
            !/customer/i.test(r.entityCategory || r.relationship || '') &&
            !/supplier|vendor/i.test(r.entityCategory || r.relationship || ''));
        el.innerHTML = `
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.supply_chain.card.customers">Customers</div>
                    <div class="value">${customers.length}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.supply_chain.card.suppliers">Suppliers</div>
                    <div class="value">${suppliers.length}</div></div>
                <div class="card"><div class="label" data-i18n="view.supply_chain.card.other">Other relationships</div>
                    <div class="value">${others.length}</div></div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.supply_chain.th.symbol">Symbol</th>
                    <th data-i18n="view.supply_chain.th.name">Name</th>
                    <th data-i18n="view.supply_chain.th.category">Category</th>
                    <th data-i18n="view.supply_chain.th.relationship">Relationship</th>
                    <th data-i18n="view.supply_chain.th.country">Country</th>
                </tr></thead>
                <tbody>${rels.slice(0, 300).map(r => `
                    <tr>
                        <td><a class="link" href="#research/${esc(r.entitySymbol || '')}">${esc(r.entitySymbol || '—')}</a></td>
                        <td>${esc(r.entityName || '—')}</td>
                        <td class="muted">${esc(r.entityCategory || '—')}</td>
                        <td class="muted">${esc(r.relationship || '—')}</td>
                        <td class="muted">${esc(r.country || '—')}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.supply_chain.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.supply_chain.toast.failed'), { level: 'error' });
    }
}
