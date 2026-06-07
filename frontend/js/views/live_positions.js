// Live P/L tracker — polls every 30s, highlights biggest mover, shows
// account-level day delta.

import { api, ApiError } from '../api.js';
import { esc, fmt } from '../util.js';
import { go, currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n, t } from '../i18n.js';
import { showToast } from '../toast.js';

let timer = null;

export async function renderLivePositions(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) {
        mount.innerHTML = `<p data-i18n="view.live_positions.hint.no_account_selected" class="boot">No account selected.</p>`;
        return;
    }
    mount.innerHTML = `
        <h1 class="view-title">// LIVE P/L — ${esc(acct.broker)} · ${esc(acct.name)}
            <button type="button" class="btn btn-secondary" id="live-refresh-btn"
                    data-i18n="view.live_positions.btn.refresh"
                    data-tip="view.live_positions.tip.refresh"
                    data-shortcut="live_refresh"
                    style="margin-left:12px;font-size:11px;padding:4px 10px;vertical-align:middle">⟳ Refresh</button>
        </h1>
        <p data-i18n="view.live_positions.hint.snapshot_of_every_open_trade_with_fresh_yahoo_quot" class="muted small">Snapshot of every open trade with fresh Yahoo quotes (60s
            server-cached). Unrealized P/L honors multiplier and side (long/short). Day P/L
            uses the quote's prev_close. Refreshes every 30 seconds.</p>

        <div id="lp-cards" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
        <div id="lp-table"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.live_positions.h2.upnl_chart">Unrealized P/L per position</h2>
            <div id="lp-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.live_positions.h2.notional_chart">Notional exposure per position</h2>
            <div id="lp-notional-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.live_positions.hint.notional" class="muted small">Per-symbol notional size. Reveals capital concentration independent of P/L performance — a 60% notional in one name is concentration risk even if it's green today.</p>
        </div>
        <p class="muted small" id="lp-status"></p>
    `;
    const refreshBtn = mount.querySelector('#live-refresh-btn');
    if (refreshBtn) refreshBtn.addEventListener('click', () => refresh(acct.id, mount, tok));
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(acct.id, mount, tok);
    }, 30_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#live')) {
            clearInterval(timer); timer = null;
        }
    }, { once: true });
    await refresh(acct.id, mount, tok);
}

async function refresh(accountId, mount, tok) {
    try {
        const r = await api.livePositions(accountId);
        if (!viewIsCurrent(tok)) return;
        renderCards(r, mount);
        renderTable(r, mount);
        renderUpnlChart(r);
        renderNotionalChart(r);
        const st = mount.querySelector('#lp-status');
        if (st) st.textContent = t('view.live_positions.status.updated', {
            time: new Date(r.fetched_at).toLocaleTimeString(undefined, { hour12: false }),
            n: r.position_count,
        });
    } catch (e) {
        if (e instanceof ApiError && e.status === 401) return;
        if (!viewIsCurrent(tok)) return;
        const st = mount.querySelector('#lp-status');
        if (st) st.textContent = t('common.error', { err: e.message });
        showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
    }
}

function renderCards(r, mount) {
    const cls = (v) => v == null ? '' : v >= 0 ? 'pos' : 'neg';
    const el = mount.querySelector('#lp-cards');
    if (!el) return;
    el.innerHTML = `
        <div class="card" data-tip="view.live_positions.tip.open_positions"><div class="label" data-i18n="view.live_positions.card.open_positions">Open positions</div>
            <div class="value">${r.position_count}</div></div>
        <div class="card" data-tip="view.live_positions.tip.total_notional"><div class="label" data-i18n="view.live_positions.card.total_notional">Total notional</div>
            <div class="value">$${fmt(r.total_notional)}</div></div>
        <div class="card" data-tip="view.live_positions.tip.unrealized_pnl"><div class="label" data-i18n="view.live_positions.card.unrealized_pnl">Unrealized P/L</div>
            <div class="value ${cls(r.total_unrealized_pnl)}">${signed$(r.total_unrealized_pnl)}</div></div>
        <div class="card" data-tip="view.live_positions.tip.day_pnl"><div class="label" data-i18n="view.live_positions.card.day_pnl">Day P/L</div>
            <div class="value ${cls(r.total_day_pnl)}">${signed$(r.total_day_pnl)}</div></div>
        <div class="card" data-tip="view.live_positions.tip.biggest_winner"><div class="label" data-i18n="view.live_positions.card.biggest_winner">Biggest winner</div>
            <div class="value pos">${esc(r.biggest_winner || '—')}</div></div>
        <div class="card" data-tip="view.live_positions.tip.biggest_loser"><div class="label" data-i18n="view.live_positions.card.biggest_loser">Biggest loser</div>
            <div class="value neg">${esc(r.biggest_loser || '—')}</div></div>
    `;
    try { applyUiI18n(el); } catch (_) {}
}

function renderTable(r, mount) {
    const tbl = mount.querySelector('#lp-table');
    if (!tbl) return;
    if (!r.positions.length) {
        tbl.innerHTML =
            '<div class="chart-panel"><p data-i18n="view.live_positions.hint.no_open_positions" class="muted small">No open positions.</p></div>';
        return;
    }
    // Sort by absolute unrealized P/L desc to surface movers.
    const sorted = [...r.positions].sort(
        (a, b) => Math.abs(b.unrealized_pnl) - Math.abs(a.unrealized_pnl));
    const winner = r.biggest_winner;
    const loser = r.biggest_loser;
    const row = (p) => {
        const tag = p.symbol === winner ? '🟢' : p.symbol === loser ? '🔴' : '';
        const upnlCls = p.unrealized_pnl >= 0 ? 'pos' : 'neg';
        const dayCls  = (p.day_pnl ?? 0) >= 0 ? 'pos' : 'neg';
        return `<tr data-context-scope="position-row"
                     data-symbol="${esc(p.symbol)}"
                     data-id="${esc(p.trade_id)}">
            <td>${tag} <a href="#trade/${p.trade_id}">${esc(p.symbol)}</a>
                <span class="muted small">${esc(p.side)} · ${esc(p.asset_class)}</span></td>
            <td>${fmt(p.qty, 0)}${p.multiplier !== 1 ? `×${fmt(p.multiplier, 0)}` : ''}</td>
            <td>${fmt(p.entry_avg, p.entry_avg < 10 ? 4 : 2)}</td>
            <td>${fmt(p.last_price, p.last_price < 10 ? 4 : 2)}</td>
            <td class="${dayCls}">${p.change_pct == null ? '—' : (p.change_pct >= 0 ? '+' : '') + p.change_pct.toFixed(2) + '%'}</td>
            <td>$${fmt(p.notional)}</td>
            <td class="${upnlCls}">${signed$(p.unrealized_pnl)}</td>
            <td class="${upnlCls}">${(p.unrealized_pct >= 0 ? '+' : '') + p.unrealized_pct.toFixed(2)}%</td>
            <td class="${dayCls}">${p.day_pnl == null ? '—' : signed$(p.day_pnl)}</td>
            <td class="small muted">${esc((p.market_state || '').toLowerCase())}</td>
        </tr>`;
    };
    tbl.innerHTML = `
        <div class="chart-panel">
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.live_positions.th.symbol">Symbol</th><th data-i18n="view.live_positions.th.qty">Qty</th><th data-i18n="view.live_positions.th.entry">Entry</th><th data-i18n="view.live_positions.th.last">Last</th>
                    <th data-i18n="view.live_positions.th.today">Δ today</th><th data-i18n="view.live_positions.th.notional">Notional</th><th data-i18n="view.live_positions.th.upnl">UPnL</th><th data-i18n="view.live_positions.th.upnl_2">UPnL %</th>
                    <th data-i18n="view.live_positions.th.day_p_l">Day P/L</th><th data-i18n="view.live_positions.th.state">State</th>
                </tr></thead>
                <tbody>${sorted.map(row).join('')}</tbody>
            </table>
        </div>
    `;
    // No-op reference to silence unused-import linters in some bundlers.
    void go;
}

function signed$(v) {
    if (v == null) return '—';
    const s = (v >= 0 ? '+' : '-') + '$' + fmt(Math.abs(v));
    return s;
}

function renderUpnlChart(r) {
    const el = document.getElementById('lp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (r.positions || []).filter(p => Number.isFinite(Number(p.unrealized_pnl)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.live_positions.empty_chart">${esc(t('view.live_positions.empty_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(b.unrealized_pnl) - Number(a.unrealized_pnl));
    const labels = valid.map(p => p.symbol);
    const ys = valid.map(p => Number(p.unrealized_pnl));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.live_positions.chart.symbol_idx') },
            { label: t('view.live_positions.chart.upnl'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.live_positions.chart.zero'),
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

function renderNotionalChart(r) {
    const el = document.getElementById('lp-notional-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (r.positions || []).filter(p => Number.isFinite(Number(p.notional)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.live_positions.empty_notional_chart">${esc(t('view.live_positions.empty_notional_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(b.notional) - Number(a.notional));
    const labels = valid.map(p => p.symbol);
    const ys = valid.map(p => Number(p.notional));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.live_positions.chart.symbol_idx') },
            { label: t('view.live_positions.chart.notional'),
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
