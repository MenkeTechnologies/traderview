// Webull personal-broker integration — paste session tokens from browser
// DevTools, poll positions / orders / account every 5s.

import { api, wsUrl } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, pnlClass } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let ws = null;
let viewTok = 0;

export async function renderWebull(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// WEBULL · LIVE BROKER
            <span class="status-dot" id="wb-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
        </h1>

        <details class="chart-panel" id="wb-creds-panel">
            <summary data-i18n-html="view.webull.summary.connect"><strong>Connect</strong> — paste tokens from your browser session</summary>
            <p class="muted small" data-i18n-html="view.webull.summary.instructions">
                1. Open <code>webull.com</code> in another tab and log in (complete MFA / trade pin).<br>
                2. Open DevTools → <strong>Network</strong> → click any <code>tradeapi.webullbroker.com</code> request.<br>
                3. Copy the request headers <code>did</code>, <code>access_token</code>, and <code>t_token</code>.<br>
                4. Paste below. Tokens are held in process memory only — never written to disk.
            </p>
            <form id="wb-form" class="inline-form">
                <label>did <input name="did" type="text" required style="min-width:280px"></label>
                <label>access_token <input name="access_token" type="password" required style="min-width:340px"></label>
                <label>t_token (trade actions; optional for read-only) <input name="t_token" type="password" style="min-width:340px"></label>
                <label>account_id (optional override) <input name="account_id" type="text" style="min-width:140px"></label>
                <button data-i18n="view.webull.btn.connect" class="primary" type="submit">Connect</button>
            </form>
            <p data-i18n="view.webull.hint.all_endpoints_used_are_read_only_positions_today_s" class="muted small">All endpoints used are read-only: positions, today's orders, account summary. Order entry is intentionally not implemented.</p>
        </details>

        <div class="cards" id="wb-account">
            <div class="card"><div class="label" data-i18n="view.webull.card.net_liquidation">Net Liquidation</div><div class="value" id="wb-nl">—</div></div>
            <div class="card"><div class="label" data-i18n="view.webull.card.cash">Cash</div><div class="value" id="wb-cash">—</div></div>
            <div class="card"><div class="label" data-i18n="view.webull.card.day_pnl">Day P/L</div><div class="value" id="wb-daypl">—</div></div>
            <div class="card"><div class="label" data-i18n="view.webull.card.total_unrealized">Total Unrealized</div><div class="value" id="wb-totpl">—</div></div>
            <div class="card"><div class="label" data-i18n="view.webull.card.buying_power">Buying Power</div><div class="value" id="wb-bp">—</div></div>
        </div>

        <div class="chart-panel">
            <h2><span data-i18n="view.webull.h2.open_positions">Open positions</span> <span class="muted small" id="wb-fetched">—</span></h2>
            <table class="trades" id="wb-pos">
                <thead><tr>
                    <th data-i18n="view.webull.th.symbol">Symbol</th><th data-i18n="view.webull.th.side">Side</th><th data-i18n="view.webull.th.asset">Asset</th><th data-i18n="view.webull.th.qty">Qty</th>
                    <th data-i18n="view.webull.th.avg_cost">Avg cost</th><th data-i18n="view.webull.th.last">Last</th><th data-i18n="view.webull.th.mkt_value">Mkt value</th>
                    <th data-i18n="view.webull.th.unrealized">Unrealized</th><th>%</th><th data-i18n="view.webull.th.day_p_l">Day P/L</th>
                </tr></thead>
                <tbody><tr><td colspan="10" class="muted">waiting for first poll…</td></tr></tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webull.h2.today_s_filled_orders">Today's filled orders</h2>
            <table class="trades" id="wb-orders">
                <thead><tr>
                    <th data-i18n="view.webull.th.time">Time</th><th data-i18n="view.webull.th.symbol_2">Symbol</th><th data-i18n="view.webull.th.side_2">Side</th>
                    <th data-i18n="view.webull.th.qty_2">Qty</th><th data-i18n="view.webull.th.avg_fill">Avg fill</th><th data-i18n="view.webull.th.status">Status</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted">waiting for first poll…</td></tr></tbody>
            </table>
        </div>
    `;

    mount.querySelector('#wb-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            did:          fd.get('did').trim(),
            access_token: fd.get('access_token').trim(),
            t_token:      fd.get('t_token').trim() || null,
            account_id:   fd.get('account_id').trim() || null,
        };
        try {
            const r = await api.connectWebull(body);
            if (!viewIsCurrent(viewTok)) return;
            alert(t('view.webull.alert.connected', { hasCreds: r.has_creds }));
            const panel = mount.querySelector('#wb-creds-panel');
            if (panel) panel.open = false;
            connectWs(mount, viewTok);
        } catch (err) {
            alert(t('view.webull.alert.connect_failed', { err: err.message }));
        }
    });

    connectWs(mount, viewTok);
}

function connectWs(mount, tok) {
    try { if (ws) ws.close(); } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#wb-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/webull'));
    ws.addEventListener('open',  () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--green)'; dot.title = t('common.status.connected'); } });
    ws.addEventListener('error', () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--red)';   dot.title = t('common.status.error'); } });
    ws.addEventListener('close', () => {
        if (!viewIsCurrent(tok)) return;
        dot.style.color = 'var(--text-muted)'; dot.title = t('common.status.disconnected');
        setTimeout(() => { if (viewIsCurrent(tok)) connectWs(mount, tok); }, 4000);
    });
    ws.addEventListener('message', (e) => {
        if (!viewIsCurrent(tok)) return;
        try {
            const m = JSON.parse(e.data);
            if (m.type === 'snapshot' && m.snap) render(mount, m.snap);
        } catch (_) {}
    });
}

function render(mount, snap) {
    const fetched = mount.querySelector('#wb-fetched');
    if (fetched) fetched.textContent = snap.fetched_at ? `updated ${fmtDateTime(snap.fetched_at)}` : '';
    const a = snap.account;
    if (a) {
        const nl = mount.querySelector('#wb-nl');     if (nl) nl.textContent = fmtMoney(a.net_liquidation);
        const cash = mount.querySelector('#wb-cash'); if (cash) cash.textContent = fmtMoney(a.cash);
        const dp = mount.querySelector('#wb-daypl');
        if (dp) { dp.textContent = fmtMoney(a.day_pnl); dp.className = `value ${pnlClass(a.day_pnl)}`; }
        const tp = mount.querySelector('#wb-totpl');
        if (tp) { tp.textContent = fmtMoney(a.total_pnl); tp.className = `value ${pnlClass(a.total_pnl)}`; }
        const bp = mount.querySelector('#wb-bp');     if (bp) bp.textContent = fmtMoney(a.buying_power);
    }
    const posBody = mount.querySelector('#wb-pos tbody');
    if (!posBody) return;
    if (snap.positions && snap.positions.length) {
        posBody.innerHTML = snap.positions.map(p => `
            <tr>
                <td><strong style="color:var(--accent)">${esc(p.symbol)}</strong></td>
                <td>${esc(p.side)}</td>
                <td>${esc(p.asset_type)}</td>
                <td>${fmt(p.qty, 0)}</td>
                <td>${fmt(p.avg_cost)}</td>
                <td>${fmt(p.last_price)}</td>
                <td>${fmtMoney(p.market_value)}</td>
                <td class="${pnlClass(p.unrealized_pnl)}">${fmtMoney(p.unrealized_pnl)}</td>
                <td class="${pnlClass(p.unrealized_pct)}">${(p.unrealized_pct).toFixed(2)}%</td>
                <td class="${pnlClass(p.day_pnl)}">${fmtMoney(p.day_pnl)}</td>
            </tr>
        `).join('');
    } else {
        posBody.innerHTML = '<tr><td colspan="10" class="muted">no open positions</td></tr>';
    }
    const ordBody = mount.querySelector('#wb-orders tbody');
    if (!ordBody) return;
    if (snap.orders && snap.orders.length) {
        ordBody.innerHTML = snap.orders.map(o => `
            <tr>
                <td>${o.filled_at ? fmtDateTime(o.filled_at) : (o.created_at ? fmtDateTime(o.created_at) : '—')}</td>
                <td><strong>${esc(o.symbol)}</strong></td>
                <td>${esc(o.side)}</td>
                <td>${fmt(o.filled_qty || o.qty, 0)}</td>
                <td>${fmt(o.avg_fill_price)}</td>
                <td>${esc(o.status)}</td>
            </tr>
        `).join('');
    } else {
        ordBody.innerHTML = '<tr><td colspan="6" class="muted">no filled orders today</td></tr>';
    }
}
