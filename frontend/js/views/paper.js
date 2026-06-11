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
                    <input name="symbol" data-shortcut="focus_search" data-tip="view.paper.tip.symbol" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" required style="text-transform:uppercase">
                    <select name="side" data-tip="view.paper.tip.side">
                        <option data-i18n="view.paper.opt.buy" value="buy">BUY</option>
                        <option data-i18n="view.paper.opt.sell" value="sell">SELL</option>
                        <option data-i18n="view.paper.opt.short" value="short">SHORT</option>
                        <option data-i18n="view.paper.opt.cover" value="cover">COVER</option>
                    </select>
                    <input name="qty" type="number" step="0.01" placeholder="qty" data-i18n-placeholder="common.placeholder.qty" data-tip="view.paper.tip.qty" required>
                    <select name="order_type" data-tip="view.paper.tip.order_type">
                        <option data-i18n="view.paper.opt.market" value="market">market</option>
                        <option data-i18n="view.paper.opt.limit" value="limit">limit</option>
                        <option data-i18n="view.paper.opt.stop" value="stop">stop</option>
                        <option data-i18n="view.paper.opt.trailing" value="trailing">trailing</option>
                    </select>
                    <input name="limit_price" type="number" step="0.01" placeholder="limit" data-i18n-placeholder="common.placeholder.limit" data-tip="view.paper.tip.limit">
                    <input name="stop_price"  type="number" step="0.01" placeholder="stop" data-i18n-placeholder="common.placeholder.stop" data-tip="view.paper.tip.stop">
                    <input name="trail_value" type="number" step="0.01" min="0" placeholder="trail" data-i18n-placeholder="common.placeholder.trail" data-tip="view.paper.tip.trail">
                    <select name="trail_unit" data-tip="view.paper.tip.trail_unit">
                        <option value="usd">$</option>
                        <option value="pct">%</option>
                    </select>
                    <button data-i18n="view.paper.btn.submit" data-tip="view.paper.tip.submit" data-shortcut="paper_submit" class="primary" type="submit">SUBMIT</button>
                </form>
                <button data-i18n="view.paper.btn.reset_account_200k" data-tip="view.paper.tip.reset" class="link" id="reset">Reset account ($200k)</button>
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
            <h2 data-i18n="view.paper.h2.unrealized_chart">Unrealized P&L per open position</h2>
            <div id="paper-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.notional_chart">Position notional per symbol</h2>
            <div id="paper-notional-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.paper.hint.notional" class="muted small">Per-symbol capital allocation (qty × last). Reveals concentration risk independent of P/L — a 60% notional in one name is concentration even if it's green today.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.order_history">Order history</h2>
            ${orders.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.paper.th.submitted">Submitted</th><th data-i18n="view.paper.th.symbol">Symbol</th><th data-i18n="view.paper.th.side">Side</th><th data-i18n="view.paper.th.qty_2">Qty</th><th data-i18n="view.paper.th.type">Type</th>
                <th data-i18n="view.paper.th.status">Status</th><th data-i18n="view.paper.th.fill_price">Fill price</th><th data-i18n="view.paper.th.filled">Filled</th><th></th></tr></thead>
                <tbody>${orders.map(o => `
                    <tr data-context-scope="symbol-row" data-symbol="${esc(o.symbol)}">
                        <td>${fmtDateTime(o.submitted_at)}</td>
                        <td>${esc(o.symbol)}</td>
                        <td>${o.side}</td>
                        <td>${fmt(o.qty, 0)}</td>
                        <td>${o.order_type}${o.limit_price != null ? ' @' + fmt(o.limit_price) : ''}${o.stop_price != null ? ' stop ' + fmt(o.stop_price) : ''}${o.trail_value != null ? ' ' + (o.trail_is_pct ? (Number(o.trail_value) * 100).toFixed(1) + '%' : '$' + fmt(o.trail_value)) + (o.status === 'pending' && o.trail_extreme != null ? ' (hwm ' + fmt(o.trail_extreme) + ')' : '') : ''}</td>
                        <td class="${o.status === 'filled' ? 'pos' : (o.status === 'rejected' ? 'neg' : '')}">${o.status}</td>
                        <td>${o.filled_price != null ? fmt(o.filled_price) : '—'}</td>
                        <td>${o.filled_at ? fmtDateTime(o.filled_at) : '—'}</td>
                        <td>${o.status === 'pending' ? `<button class="ord-cancel" data-id="${esc(o.id)}" data-i18n="common.btn.cancel">${esc(t('common.btn.cancel'))}</button>` : ''}</td>
                    </tr>`).join('')}</tbody></table>` : '<p data-i18n="view.paper.hint.no_orders_yet" class="muted">No orders yet.</p>'}
        </div>
    `;

    renderUnrealizedChart(positions, quotes);
    renderNotionalChart(positions, quotes);

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
            // A "5" with % selected means 5% — the engine wants 0.05.
            trail_value: fd.get('trail_value')
                ? Number(fd.get('trail_value')) / (fd.get('trail_unit') === 'pct' ? 100 : 1)
                : null,
            trail_is_pct: fd.get('trail_value') ? fd.get('trail_unit') === 'pct' : null,
        };
        try {
            const o = await api.paperSubmit(acct.id, body);
            if (!viewIsCurrent(tok)) return;
            if (o.status === 'rejected') {
                showToast(t('view.paper.alert.order_rejected', { reason: o.reject_reason || t('common.empty.unknown') }), { level: 'error' });
            } else if (o.status === 'filled') {
                showToast(t('view.paper.toast.filled', {
                    side: body.side, qty: body.qty, symbol: body.symbol,
                    price: o.filled_price != null ? fmt(o.filled_price) : '—',
                }), { level: 'success' });
            } else {
                showToast(t('view.paper.toast.submitted', { side: body.side, qty: body.qty, symbol: body.symbol }), { level: 'info' });
            }
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelectorAll('.ord-cancel').forEach(btn => btn.addEventListener('click', async () => {
        try {
            await api.paperOrderCancel(btn.dataset.id);
            if (!viewIsCurrent(tok)) return;
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    }));
    mount.querySelector('#reset').addEventListener('click', async () => {
        if (!await tConfirm('view.paper.confirm.reset', {}, { level: 'danger' })) return;
        try {
            await api.paperReset(acct.id, 200000);
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.paper.toast.reset'), { level: 'success' });
            renderPaper(mount);
        } catch (err) { showToast(t('toast.error.api', { err: err.message }), { level: 'error' }); }
    });
}

function renderUnrealizedChart(positions, quotes) {
    const el = document.getElementById('paper-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const pts = (positions || []).map(p => {
        const q = quotes[p.symbol];
        if (!q) return null;
        const u = (Number(q.price) - Number(p.avg_price)) * Number(p.qty);
        return Number.isFinite(u) ? { symbol: p.symbol, u } : null;
    }).filter(Boolean);
    if (pts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.paper.empty_chart">${esc(t('view.paper.empty_chart'))}</div>`;
        return;
    }
    pts.sort((a, b) => b.u - a.u);
    const labels = pts.map(p => p.symbol);
    const ys = pts.map(p => p.u);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.paper.chart.symbol_idx') },
            { label: t('view.paper.chart.unrealized'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.paper.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderNotionalChart(positions, quotes) {
    const el = document.getElementById('paper-notional-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const pts = (positions || []).map(p => {
        const q = quotes[p.symbol];
        if (!q) return null;
        const n = Number(q.price) * Number(p.qty);
        return Number.isFinite(n) ? { symbol: p.symbol, n } : null;
    }).filter(Boolean);
    if (pts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.paper.empty_notional_chart">${esc(t('view.paper.empty_notional_chart'))}</div>`;
        return;
    }
    pts.sort((a, b) => b.n - a.n);
    const labels = pts.map(p => p.symbol);
    const ys = pts.map(p => p.n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.paper.chart.symbol_idx') },
            { label: t('view.paper.chart.notional'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
