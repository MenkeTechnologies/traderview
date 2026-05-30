// Webull personal-broker integration — paste session tokens from browser
// DevTools, poll positions / orders / account every 5s.

import { api, wsUrl } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, pnlClass } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let ws = null;
let viewTok = 0;

export async function renderWebull(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.webull.title">// WEBULL · LIVE BROKER</span>
            <span class="status-dot" id="wb-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <button type="button" class="btn btn-secondary" id="wb-refresh-btn"
                    data-i18n="view.webull.btn.refresh"
                    data-tip="view.webull.tip.refresh"
                    data-shortcut="webull_refresh"
                    style="margin-left:12px;font-size:11px;padding:4px 10px;vertical-align:middle">⟳ Refresh</button>
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
                <label>${esc(t('view.webull.label.t_token'))} <input name="t_token" type="password" style="min-width:340px"></label>
                <label>${esc(t('view.webull.label.account_id'))} <input name="account_id" type="text" style="min-width:140px"></label>
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
                <tbody><tr><td colspan="10" class="muted" data-i18n="common.status.waiting_first_poll">waiting for first poll…</td></tr></tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webull.h2.unreal_chart">Unrealized P&L per position</h2>
            <div id="wb-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webull.h2.day_chart">Day P/L per position</h2>
            <div id="wb-day-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webull.h2.mv_chart">Market value per position</h2>
            <div id="wb-mv-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.webull.h2.today_s_filled_orders">Today's filled orders</h2>
            <table class="trades" id="wb-orders">
                <thead><tr>
                    <th data-i18n="view.webull.th.time">Time</th><th data-i18n="view.webull.th.symbol_2">Symbol</th><th data-i18n="view.webull.th.side_2">Side</th>
                    <th data-i18n="view.webull.th.qty_2">Qty</th><th data-i18n="view.webull.th.avg_fill">Avg fill</th><th data-i18n="view.webull.th.status">Status</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted" data-i18n="common.status.waiting_first_poll">waiting for first poll…</td></tr></tbody>
            </table>
        </div>
    `;

    const refreshBtn = mount.querySelector('#wb-refresh-btn');
    if (refreshBtn) refreshBtn.addEventListener('click', () =>
        window.dispatchEvent(new HashChangeEvent('hashchange')));
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
            showToast(t('view.webull.alert.connected', { hasCreds: r.has_creds }), { level: 'error' });
            const panel = mount.querySelector('#wb-creds-panel');
            if (panel) panel.open = false;
            connectWs(mount, viewTok);
        } catch (err) {
            showToast(t('view.webull.alert.connect_failed', { err: err.message }), { level: 'error' });
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

function renderMvChart(positions) {
    const el = document.getElementById('wb-mv-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (positions || []).filter(p => Number.isFinite(Number(p.market_value)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.webull.empty_mv_chart">${esc(t('view.webull.empty_mv_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.market_value) - Number(a.market_value));
    const labels = rows.map(p => p.symbol);
    const xs = labels.map((_, i) => i + 1);
    const ys = rows.map(p => Number(p.market_value));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.webull.chart.symbol') },
            { label: t('view.webull.chart.market_value'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderDayChart(positions) {
    const el = document.getElementById('wb-day-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (positions || []).filter(p => Number.isFinite(Number(p.day_pnl)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.webull.empty_day_chart">${esc(t('view.webull.empty_day_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.day_pnl) - Number(a.day_pnl));
    const labels = rows.map(p => p.symbol);
    const xs = labels.map((_, i) => i + 1);
    const winY  = rows.map(p => Number(p.day_pnl) >= 0 ? Number(p.day_pnl) : null);
    const loseY = rows.map(p => Number(p.day_pnl) <  0 ? Number(p.day_pnl) : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.webull.chart.symbol') },
            { label: t('view.webull.chart.day_up'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.webull.chart.day_down'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.webull.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, winY, loseY, zero], el);
}

function renderUnrealChart(positions) {
    const el = document.getElementById('wb-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (positions || []).filter(p => Number.isFinite(Number(p.unrealized_pnl)));
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.webull.empty_chart">${esc(t('view.webull.empty_chart'))}</div>`;
        return;
    }
    rows.sort((a, b) => Number(b.unrealized_pnl) - Number(a.unrealized_pnl));
    const labels = rows.map(p => p.symbol);
    const xs = labels.map((_, i) => i + 1);
    const winY  = rows.map(p => Number(p.unrealized_pnl) >= 0 ? Number(p.unrealized_pnl) : null);
    const loseY = rows.map(p => Number(p.unrealized_pnl) <  0 ? Number(p.unrealized_pnl) : null);
    const zero  = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.webull.chart.symbol') },
            { label: t('view.webull.chart.win'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.webull.chart.lose'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.webull.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, winY, loseY, zero], el);
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
    renderUnrealChart(snap.positions || []);
    renderDayChart(snap.positions || []);
    renderMvChart(snap.positions || []);
    const posBody = mount.querySelector('#wb-pos tbody');
    if (!posBody) return;
    if (snap.positions && snap.positions.length) {
        posBody.innerHTML = snap.positions.map(p => `
            <tr data-context-scope="symbol-row" data-symbol="${esc(p.symbol)}">
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
        posBody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.webull.empty.no_positions'))}</td></tr>`;
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
        ordBody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('view.webull.empty.no_orders'))}</td></tr>`;
    }
}
