// Finnhub Insider Transactions — Form 4 buys/sells per symbol.
// Premium-graceful: free tier returns 403 → empty state with hint.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderInsiderTransactionsFinnhub(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.insider_fh.h1.title">// INSIDER TRANSACTIONS (FINNHUB)</span></h1>
        <p class="muted small" data-i18n="view.insider_fh.hint.intro">
            Finnhub /stock/insider-transactions — Form 4 buys/sells. Cluster of insider
            buys before earnings = bullish signal. <strong>Premium endpoint</strong> on
            free tier; SEC EDGAR is the free fallback (see Disclosures view).
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="ifh-form">
                <label><span data-i18n="view.insider_fh.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="NVDA" required></label>
                <button class="primary" type="submit" data-i18n="view.insider_fh.btn.load">Load</button>
            </form>
            <div id="ifh-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('ifh-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('ifh-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const from = new Date();
        const to = new Date(from);
        from.setDate(from.getDate() - 180);
        const data = await api.symbolFinnhubInsiders(state.symbol, fmtDay(from), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.insider_fh.empty">No insider transactions (or premium required).</p>`;
            return;
        }
        const buys = rows.filter(r => Number(r.change) > 0);
        const sells = rows.filter(r => Number(r.change) < 0);
        const buyShares = buys.reduce((s, r) => s + Math.abs(Number(r.change) || 0), 0);
        const sellShares = sells.reduce((s, r) => s + Math.abs(Number(r.change) || 0), 0);
        el.innerHTML = `
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.insider_fh.card.buys">Buys</div>
                    <div class="value">${buys.length} <span class="muted small">${buyShares.toLocaleString()} sh</span></div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.insider_fh.card.sells">Sells</div>
                    <div class="value">${sells.length} <span class="muted small">${sellShares.toLocaleString()} sh</span></div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.insider_fh.card.net">Net</div>
                    <div class="value ${buyShares >= sellShares ? 'pos' : 'neg'}">${(buyShares - sellShares).toLocaleString()}</div>
                </div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.insider_fh.th.filing_date">Filing</th>
                    <th data-i18n="view.insider_fh.th.tx_date">Tx Date</th>
                    <th data-i18n="view.insider_fh.th.name">Name</th>
                    <th data-i18n="view.insider_fh.th.position">Position</th>
                    <th data-i18n="view.insider_fh.th.change">Change</th>
                    <th data-i18n="view.insider_fh.th.price">Price</th>
                    <th data-i18n="view.insider_fh.th.value">Value</th>
                    <th data-i18n="view.insider_fh.th.code">Code</th>
                </tr></thead>
                <tbody>${rows.slice(0, 200).map(r => {
                    const ch = Number(r.change) || 0;
                    const cls = ch > 0 ? 'pos' : ch < 0 ? 'neg' : '';
                    const val = (Number(r.transactionPrice) || 0) * Math.abs(ch);
                    return `<tr>
                        <td class="muted">${esc(r.filingDate || '—')}</td>
                        <td class="muted">${esc(r.transactionDate || '—')}</td>
                        <td>${esc(r.name || '—')}</td>
                        <td class="muted">${esc(r.position || '—')}</td>
                        <td class="${cls}">${ch.toLocaleString()}</td>
                        <td>${r.transactionPrice != null ? '$' + Number(r.transactionPrice).toFixed(2) : '—'}</td>
                        <td>${val ? '$' + val.toLocaleString() : '—'}</td>
                        <td class="muted">${esc(r.transactionCode || '—')}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.insider_fh.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.insider_fh.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
