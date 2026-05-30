// Paper-trading simulator — Warrior Trading SimTrader equivalent.
import { api } from '../api.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';

export async function renderPaper(mount) {
    const tok = currentViewToken();
    const accounts = await api.paperAccounts();
    if (!viewIsCurrent(tok)) return;
    if (!accounts.length) {
        await api.paperEnsure();
        if (!viewIsCurrent(tok)) return;
        return renderPaper(mount);
    }
    const acct = accounts[0];
    const [positions, orders] = await Promise.all([
        api.paperPositions(acct.id),
        api.paperOrders(acct.id, 50),
    ]);
    if (!viewIsCurrent(tok)) return;

    // Live unrealized P&L — fetch quotes for held symbols.
    const symList = positions.map(p => p.symbol);
    let quotes = {};
    if (symList.length) {
        try {
            const promises = symList.map(s => api.quote(s).catch(() => null));
            const qs = await Promise.all(promises);
            if (!viewIsCurrent(tok)) return;
            qs.forEach(q => { if (q) quotes[q.symbol] = q; });
        } catch (_) {}
    }
    let posValue = 0, unrealized = 0;
    positions.forEach(p => {
        const q = quotes[p.symbol];
        if (q) {
            const mark = Number(q.price);
            const qty = Number(p.qty);
            posValue += mark * qty;
            unrealized += (mark - Number(p.avg_price)) * qty;
        }
    });
    const cash = Number(acct.cash);
    const equity = cash + posValue;
    const total = equity - Number(acct.starting_cash);
    const totalPct = Number(acct.starting_cash) > 0 ? (total / Number(acct.starting_cash) * 100) : 0;

    mount.innerHTML = `
        <h1 class="view-title">// PAPER TRADING · ${esc(acct.name)}</h1>

        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.paper.card.cash">Cash</div><div class="value">$${fmt(cash)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.position_value">Position value</div><div class="value">$${fmt(posValue)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.equity">Equity</div><div class="value">$${fmt(equity)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.total_pnl">Total P&L</div>
                <div class="value ${total >= 0 ? 'pos' : 'neg'}">${total >= 0 ? '+' : ''}$${fmt(total)} (${totalPct.toFixed(2)}%)</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.unrealized">Unrealized</div>
                <div class="value ${unrealized >= 0 ? 'pos' : 'neg'}">${unrealized >= 0 ? '+' : ''}$${fmt(unrealized)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.starting_cash">Starting cash</div><div class="value">$${fmt(acct.starting_cash)}</div></div>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.paper.h2.order_ticket">Order ticket</h2>
                <form id="ord-form" class="inline-form">
                    <input name="symbol" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" required style="text-transform:uppercase">
                    <select name="side">
                        <option data-i18n="view.paper.opt.buy" value="buy">BUY</option>
                        <option data-i18n="view.paper.opt.sell" value="sell">SELL</option>
                        <option data-i18n="view.paper.opt.short" value="short">SHORT</option>
                        <option data-i18n="view.paper.opt.cover" value="cover">COVER</option>
                    </select>
                    <input name="qty" type="number" step="any" placeholder="qty" data-i18n-placeholder="common.placeholder.qty" required>
                    <select name="order_type">
                        <option data-i18n="view.paper.opt.market" value="market">market</option>
                        <option data-i18n="view.paper.opt.limit" value="limit">limit</option>
                        <option data-i18n="view.paper.opt.stop" value="stop">stop</option>
                    </select>
                    <input name="limit_price" type="number" step="any" placeholder="limit" data-i18n-placeholder="common.placeholder.limit">
                    <input name="stop_price"  type="number" step="any" placeholder="stop" data-i18n-placeholder="common.placeholder.stop">
                    <button data-i18n="view.paper.btn.submit" class="primary" type="submit">SUBMIT</button>
                </form>
                <button data-i18n="view.paper.btn.reset_account_200k" class="link" id="reset">Reset account ($200k)</button>
            </div>

            <div class="chart-panel">
                <h2 data-i18n="view.paper.h2.open_positions">Open positions</h2>
                ${positions.length ? `<table class="trades">
                    <thead><tr><th data-i18n="view.paper.th.sym">Sym</th><th data-i18n="view.paper.th.qty">Qty</th><th data-i18n="view.paper.th.avg">Avg</th><th data-i18n="view.paper.th.last">Last</th>
                    <th data-i18n="view.paper.th.unrealized">Unrealized</th><th data-i18n="view.paper.th.realized">Realized</th></tr></thead>
                    <tbody>${positions.map(p => {
                        const q = quotes[p.symbol];
                        const last = q ? Number(q.price) : null;
                        const u = last != null ? (last - Number(p.avg_price)) * Number(p.qty) : null;
                        const cls = u != null && u >= 0 ? 'pos' : 'neg';
                        return `<tr data-context-scope="symbol-row" data-symbol="${esc(p.symbol)}">
                            <td><a href="#research/${encodeURIComponent(p.symbol)}">${esc(p.symbol)}</a></td>
                            <td>${fmt(p.qty, 0)}</td>
                            <td>${fmt(p.avg_price)}</td>
                            <td>${last != null ? fmt(last) : '—'}</td>
                            <td class="${cls}">${u != null ? (u >= 0 ? '+' : '') + '$' + fmt(u) : '—'}</td>
                            <td class="${Number(p.realized_pnl) >= 0 ? 'pos' : 'neg'}">$${fmt(p.realized_pnl)}</td>
                        </tr>`;
                    }).join('')}</tbody></table>` : '<p data-i18n="view.paper.hint.no_open_positions" class="muted">No open positions.</p>'}
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.order_history">Order history</h2>
            ${orders.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.paper.th.submitted">Submitted</th><th data-i18n="view.paper.th.symbol">Symbol</th><th data-i18n="view.paper.th.side">Side</th><th data-i18n="view.paper.th.qty_2">Qty</th><th data-i18n="view.paper.th.type">Type</th>
                <th data-i18n="view.paper.th.status">Status</th><th data-i18n="view.paper.th.fill_price">Fill price</th><th data-i18n="view.paper.th.filled">Filled</th></tr></thead>
                <tbody>${orders.map(o => `
                    <tr data-context-scope="symbol-row" data-symbol="${esc(o.symbol)}">
                        <td>${fmtDateTime(o.submitted_at)}</td>
                        <td>${esc(o.symbol)}</td>
                        <td>${o.side}</td>
                        <td>${fmt(o.qty, 0)}</td>
                        <td>${o.order_type}</td>
                        <td class="${o.status === 'filled' ? 'pos' : (o.status === 'rejected' ? 'neg' : '')}">${o.status}</td>
                        <td>${o.filled_price != null ? fmt(o.filled_price) : '—'}</td>
                        <td>${o.filled_at ? fmtDateTime(o.filled_at) : '—'}</td>
                    </tr>`).join('')}</tbody></table>` : '<p data-i18n="view.paper.hint.no_orders_yet" class="muted">No orders yet.</p>'}
        </div>
    `;

    mount.querySelector('#ord-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            symbol: fd.get('symbol').trim().toUpperCase(),
            side: fd.get('side'),
            qty: Number(fd.get('qty')),
            order_type: fd.get('order_type'),
            limit_price: fd.get('limit_price') ? Number(fd.get('limit_price')) : null,
            stop_price: fd.get('stop_price') ? Number(fd.get('stop_price')) : null,
        };
        try {
            const o = await api.paperSubmit(acct.id, body);
            if (!viewIsCurrent(tok)) return;
            if (o.status === 'rejected') showToast(t('view.paper.alert.order_rejected', { reason: o.reject_reason || t('common.empty.unknown') }), { level: 'error' });
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelector('#reset').addEventListener('click', async () => {
        if (!await tConfirm('view.paper.confirm.reset', {}, { level: 'danger' })) return;
        await api.paperReset(acct.id, 200000);
        if (!viewIsCurrent(tok)) return;
        renderPaper(mount);
    });
}
