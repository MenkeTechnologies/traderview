// Per-symbol congressional trading — Finnhub /stock/congressional-trading.
// Discloses trades by US Senators / Representatives in a given ticker.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderCongressionalTrading(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cong_trading.h1.title">// CONGRESSIONAL TRADING</span></h1>
        <p class="muted small" data-i18n="view.cong_trading.hint.intro">
            Per-symbol US Senate / House trades disclosed under the STOCK Act.
            High-volume buys before catalysts = signal worth investigating.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="ct-form">
                <label><span data-i18n="view.cong_trading.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="NVDA" required></label>
                <button class="primary" type="submit" data-i18n="view.cong_trading.btn.load">Load</button>
            </form>
            <div id="ct-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('ct-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('ct-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const from = new Date();
        const to = new Date(from);
        from.setDate(from.getDate() - 365);
        const data = await api.symbolCongressionalTrading(state.symbol, fmtDay(from), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.cong_trading.empty">No congressional trades on this symbol.</p>`;
            return;
        }
        const buys = rows.filter(r => (r.transactionType || '').toLowerCase().includes('purchase')).length;
        const sells = rows.filter(r => (r.transactionType || '').toLowerCase().includes('sale')).length;
        el.innerHTML = `
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.cong_trading.card.buys">Buys</div>
                    <div class="value">${buys}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.cong_trading.card.sells">Sells</div>
                    <div class="value">${sells}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.cong_trading.card.total">Total disclosures</div>
                    <div class="value">${rows.length}</div>
                </div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.cong_trading.th.transaction_date">Tx Date</th>
                    <th data-i18n="view.cong_trading.th.disclosure_date">Disclosed</th>
                    <th data-i18n="view.cong_trading.th.name">Name</th>
                    <th data-i18n="view.cong_trading.th.house">Chamber</th>
                    <th data-i18n="view.cong_trading.th.type">Type</th>
                    <th data-i18n="view.cong_trading.th.amount">Amount</th>
                    <th data-i18n="view.cong_trading.th.owner">Owner</th>
                </tr></thead>
                <tbody>${rows.slice(0, 100).map(r => {
                    const tx = (r.transactionType || '').toLowerCase();
                    const cls = tx.includes('purchase') ? 'pos' : tx.includes('sale') ? 'neg' : '';
                    return `<tr>
                        <td class="muted">${esc(r.transactionDate || '—')}</td>
                        <td class="muted">${esc(r.filingDate || '—')}</td>
                        <td>${esc(r.name || '—')}</td>
                        <td>${esc(r.position || r.chamber || '—')}</td>
                        <td class="${cls}">${esc(r.transactionType || '—')}</td>
                        <td>${esc(r.amountFrom != null
                            ? '$' + Number(r.amountFrom).toLocaleString()
                              + (r.amountTo ? '–$' + Number(r.amountTo).toLocaleString() : '')
                            : '—')}</td>
                        <td class="muted">${esc(r.ownerType || '—')}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        // Plan-restricted endpoint → HTTP 403 from the backend's
        // `map_fh_err`. Render a clean premium-required affordance
        // instead of the generic error toast.
        if (e && e.status === 403) {
            if (el) {
                el.innerHTML = `
                    <p class="muted neg">${esc(t('view.cong_trading.error.premium_required'))}</p>
                    <p class="muted small">${esc(t('view.cong_trading.error.premium_hint'))}</p>
                    <p><a href="https://finnhub.io/pricing" target="_blank" rel="noopener" class="btn btn-secondary btn-compact">${esc(t('view.cong_trading.error.premium_link'))}</a></p>
                `;
            }
            return;
        }
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.cong_trading.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.cong_trading.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
