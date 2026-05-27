// Paper-trading simulator — Warrior Trading SimTrader equivalent.
import { api } from '../api.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

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
            <div class="card"><div class="label">Cash</div><div class="value">$${fmt(cash)}</div></div>
            <div class="card"><div class="label">Position value</div><div class="value">$${fmt(posValue)}</div></div>
            <div class="card"><div class="label">Equity</div><div class="value">$${fmt(equity)}</div></div>
            <div class="card"><div class="label">Total P&L</div>
                <div class="value ${total >= 0 ? 'pos' : 'neg'}">${total >= 0 ? '+' : ''}$${fmt(total)} (${totalPct.toFixed(2)}%)</div></div>
            <div class="card"><div class="label">Unrealized</div>
                <div class="value ${unrealized >= 0 ? 'pos' : 'neg'}">${unrealized >= 0 ? '+' : ''}$${fmt(unrealized)}</div></div>
            <div class="card"><div class="label">Starting cash</div><div class="value">$${fmt(acct.starting_cash)}</div></div>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2>Order ticket</h2>
                <form id="ord-form" class="inline-form">
                    <input name="symbol" placeholder="symbol" required style="text-transform:uppercase">
                    <select name="side">
                        <option value="buy">BUY</option>
                        <option value="sell">SELL</option>
                        <option value="short">SHORT</option>
                        <option value="cover">COVER</option>
                    </select>
                    <input name="qty" type="number" step="any" placeholder="qty" required>
                    <select name="order_type">
                        <option value="market">market</option>
                        <option value="limit">limit</option>
                        <option value="stop">stop</option>
                    </select>
                    <input name="limit_price" type="number" step="any" placeholder="limit">
                    <input name="stop_price"  type="number" step="any" placeholder="stop">
                    <button class="primary" type="submit">SUBMIT</button>
                </form>
                <button class="link" id="reset">Reset account ($200k)</button>
            </div>

            <div class="chart-panel">
                <h2>Open positions</h2>
                ${positions.length ? `<table class="trades">
                    <thead><tr><th>Sym</th><th>Qty</th><th>Avg</th><th>Last</th>
                    <th>Unrealized</th><th>Realized</th></tr></thead>
                    <tbody>${positions.map(p => {
                        const q = quotes[p.symbol];
                        const last = q ? Number(q.price) : null;
                        const u = last != null ? (last - Number(p.avg_price)) * Number(p.qty) : null;
                        const cls = u != null && u >= 0 ? 'pos' : 'neg';
                        return `<tr>
                            <td><a href="#research/${encodeURIComponent(p.symbol)}">${esc(p.symbol)}</a></td>
                            <td>${fmt(p.qty, 0)}</td>
                            <td>${fmt(p.avg_price)}</td>
                            <td>${last != null ? fmt(last) : '—'}</td>
                            <td class="${cls}">${u != null ? (u >= 0 ? '+' : '') + '$' + fmt(u) : '—'}</td>
                            <td class="${Number(p.realized_pnl) >= 0 ? 'pos' : 'neg'}">$${fmt(p.realized_pnl)}</td>
                        </tr>`;
                    }).join('')}</tbody></table>` : '<p class="muted">No open positions.</p>'}
            </div>
        </div>

        <div class="chart-panel">
            <h2>Order history</h2>
            ${orders.length ? `<table class="trades">
                <thead><tr><th>Submitted</th><th>Symbol</th><th>Side</th><th>Qty</th><th>Type</th>
                <th>Status</th><th>Fill price</th><th>Filled</th></tr></thead>
                <tbody>${orders.map(o => `
                    <tr>
                        <td>${fmtDateTime(o.submitted_at)}</td>
                        <td>${esc(o.symbol)}</td>
                        <td>${o.side}</td>
                        <td>${fmt(o.qty, 0)}</td>
                        <td>${o.order_type}</td>
                        <td class="${o.status === 'filled' ? 'pos' : (o.status === 'rejected' ? 'neg' : '')}">${o.status}</td>
                        <td>${o.filled_price != null ? fmt(o.filled_price) : '—'}</td>
                        <td>${o.filled_at ? fmtDateTime(o.filled_at) : '—'}</td>
                    </tr>`).join('')}</tbody></table>` : '<p class="muted">No orders yet.</p>'}
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
            if (o.status === 'rejected') alert('Order rejected: ' + (o.reject_reason || 'unknown'));
            renderPaper(mount);
        } catch (err) { alert('Error: ' + err.message); }
    });
    mount.querySelector('#reset').addEventListener('click', async () => {
        if (!confirm('Wipe orders + positions and reset cash to $200,000?')) return;
        await api.paperReset(acct.id, 200000);
        if (!viewIsCurrent(tok)) return;
        renderPaper(mount);
    });
}
