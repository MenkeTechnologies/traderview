// Multi-account aggregation — per-account stats + grand total row.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t, applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

export async function renderAccountsOverview(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.accounts_overview.h1.accounts_overview" class="view-title">// ACCOUNTS OVERVIEW</h1>
        <p class="muted small" data-i18n="view.accounts_overview.hint.intro">Side-by-side snapshot of every account you own. P/L windows (today / MTD / YTD) are computed from trades.opened_at UTC dates. Open-position fields pull fresh quotes (60s server cache) and aggregate the same way the Live P/L tab does per-account.</p>

        <div id="ao-out"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
    `;
    try {
        const r = await api.accountsOverview();
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const out = mount.querySelector('#ao-out');
        if (out) out.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
        showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
    }
}

function render(r, mount) {
    const cls = (v) => v == null ? '' : v >= 0 ? 'pos' : 'neg';
    const g = r.grand_total;
    const out = mount.querySelector('#ao-out');
    if (!out) return;
    out.innerHTML = `
        <div class="cards">
            <div class="card" data-tip="view.accounts_overview.tip.accounts"><div class="label" data-i18n="view.accounts_overview.card.accounts">Accounts</div>
                <div class="value">${g.accounts}</div></div>
            <div class="card" data-tip="view.accounts_overview.tip.closed_trades"><div class="label" data-i18n="view.accounts_overview.card.closed_trades">Closed trades (∑)</div>
                <div class="value">${g.trade_count}</div>
                <div class="small muted">${g.win_count}W / ${g.loss_count}L</div></div>
            <div class="card" data-tip="view.accounts_overview.tip.today_pnl"><div class="label" data-i18n="view.accounts_overview.card.today_pnl">Today P/L (∑)</div>
                <div class="value ${cls(g.today_pnl)}">$${fmt(g.today_pnl)}</div></div>
            <div class="card" data-tip="view.accounts_overview.tip.mtd_pnl"><div class="label" data-i18n="view.accounts_overview.card.mtd_pnl">MTD P/L (∑)</div>
                <div class="value ${cls(g.mtd_pnl)}">$${fmt(g.mtd_pnl)}</div></div>
            <div class="card" data-tip="view.accounts_overview.tip.ytd_pnl"><div class="label" data-i18n="view.accounts_overview.card.ytd_pnl">YTD P/L (∑)</div>
                <div class="value ${cls(g.ytd_pnl)}">$${fmt(g.ytd_pnl)}</div></div>
            <div class="card" data-tip="view.accounts_overview.tip.alltime_pnl"><div class="label" data-i18n="view.accounts_overview.card.alltime_pnl">All-time P/L (∑)</div>
                <div class="value ${cls(g.total_closed_pnl)}">$${fmt(g.total_closed_pnl)}</div></div>
            <div class="card" data-tip="view.accounts_overview.tip.open_positions"><div class="label" data-i18n="view.accounts_overview.card.open_positions">Open positions (∑)</div>
                <div class="value">${g.open_positions_count}</div>
                <div class="small muted">${esc(t('view.accounts_overview.card.notional', { notional: fmt(g.open_notional) }))}</div></div>
            <div class="card" data-tip="view.accounts_overview.tip.unrealized"><div class="label" data-i18n="view.accounts_overview.card.unrealized">Unrealized (∑)</div>
                <div class="value ${cls(g.open_unrealized_pnl)}">$${fmt(g.open_unrealized_pnl)}</div>
                <div class="small muted">${esc(t('view.accounts_overview.card.day_delta', { sign: g.open_day_pnl >= 0 ? '+' : '', amount: fmt(g.open_day_pnl) }))}</div></div>
        </div>

        <div class="chart-panel">
            <h2>${esc(t('view.accounts_overview.h2.breakdown', { count: r.accounts.length }))}</h2>
            ${accountTable(r.accounts)}
            <p class="muted small">${esc(t('view.accounts_overview.hint.updated', { time: new Date(r.computed_at).toLocaleString() }))}</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.accounts_overview.h2.pnl_chart">All-time P&L per account</h2>
            <div id="ao-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.accounts_overview.h2.winrate_chart">Win rate per account vs 50% baseline</h2>
            <div id="ao-wr-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    try { applyUiI18n(out); } catch (_) {}
    renderPnlChart(r.accounts);
    renderWinRateChart(r.accounts);
}

function renderPnlChart(accounts) {
    const el = document.getElementById('ao-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!Array.isArray(accounts) || accounts.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.accounts_overview.empty_chart">${esc(t('view.accounts_overview.empty_chart'))}</div>`;
        return;
    }
    const labels = accounts.map(a => `${a.broker}·${a.name}`);
    const pnl = accounts.map(a => Number.isFinite(a.total_closed_pnl) ? a.total_closed_pnl : null);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.accounts_overview.chart.acct_idx') },
            { label: t('view.accounts_overview.chart.pnl'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.accounts_overview.chart.zero'),
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
    }, [xs, pnl, zero], el);
}

function renderWinRateChart(accounts) {
    const el = document.getElementById('ao-wr-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!Array.isArray(accounts) || accounts.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.accounts_overview.empty_wr_chart">${esc(t('view.accounts_overview.empty_wr_chart'))}</div>`;
        return;
    }
    const labels = accounts.map(a => `${a.broker}·${a.name}`);
    const wr = accounts.map(a => Number.isFinite(a.win_rate) ? a.win_rate * 100 : null);
    const xs = labels.map((_, i) => i + 1);
    const fifty = xs.map(() => 50);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.accounts_overview.chart.acct_idx') },
            { label: t('view.accounts_overview.chart.win_rate'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.accounts_overview.chart.baseline_50'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, wr, fifty], el);
}

function accountTable(accounts) {
    if (!accounts.length) return '<p data-i18n="view.accounts_overview.hint.no_accounts" class="muted small">No accounts.</p>';
    const cls = (v) => v == null ? '' : v >= 0 ? 'pos' : 'neg';
    return `<table class="trades">
        <thead><tr>
            <th data-i18n="view.accounts_overview.th.broker_name">Broker · Name</th><th data-i18n="view.accounts_overview.th.ccy">Ccy</th>
            <th data-i18n="view.accounts_overview.th.today">Today</th><th data-i18n="view.accounts_overview.th.mtd">MTD</th><th data-i18n="view.accounts_overview.th.ytd">YTD</th><th data-i18n="view.accounts_overview.th.all_time">All-time</th>
            <th data-i18n="view.accounts_overview.th.trades">Trades</th><th data-i18n="view.accounts_overview.th.win">Win%</th>
            <th data-i18n="view.accounts_overview.th.best">Best</th><th data-i18n="view.accounts_overview.th.worst">Worst</th>
            <th data-i18n="view.accounts_overview.th.open">Open</th><th data-i18n="view.accounts_overview.th.unrealized">Unrealized</th>
        </tr></thead>
        <tbody>
        ${accounts.map(a => `<tr>
            <td><strong>${esc(a.broker)}</strong>
                <div class="muted small">${esc(a.name)}</div></td>
            <td class="small">${esc(a.base_currency || 'USD')}</td>
            <td class="${cls(a.today_pnl)}">$${fmt(a.today_pnl)}</td>
            <td class="${cls(a.mtd_pnl)}">$${fmt(a.mtd_pnl)}</td>
            <td class="${cls(a.ytd_pnl)}">$${fmt(a.ytd_pnl)}</td>
            <td class="${cls(a.total_closed_pnl)}">$${fmt(a.total_closed_pnl)}</td>
            <td>${a.trade_count}</td>
            <td>${(a.win_rate * 100).toFixed(1)}%</td>
            <td class="small">${a.best_trade_pnl != null
                ? `<span class="pos">$${fmt(a.best_trade_pnl)}</span> · ${esc(a.best_trade_symbol || '')}`
                : '—'}</td>
            <td class="small">${a.worst_trade_pnl != null
                ? `<span class="neg">$${fmt(a.worst_trade_pnl)}</span> · ${esc(a.worst_trade_symbol || '')}`
                : '—'}</td>
            <td>${a.open_positions_count}
                <div class="small muted">$${fmt(a.open_notional)}</div></td>
            <td class="${cls(a.open_unrealized_pnl)}">$${fmt(a.open_unrealized_pnl)}
                <div class="small muted ${cls(a.open_day_pnl)}">${esc(t('view.accounts_overview.row.day_pnl', { sign: a.open_day_pnl >= 0 ? '+' : '', amount: fmt(a.open_day_pnl) }))}</div></td>
        </tr>`).join('')}
        </tbody></table>`;
}
