// Multi-account aggregation — per-account stats + grand total row.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderAccountsOverview(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.accounts_overview.h1.accounts_overview" class="view-title">// ACCOUNTS OVERVIEW</h1>
        <p class="muted small">Side-by-side snapshot of every account you own. P/L windows
            (today / MTD / YTD) are computed from <code>trades.opened_at</code> UTC dates.
            Open-position fields pull fresh quotes (60s server cache) and aggregate the same
            way the Live P/L tab does per-account.</p>

        <div id="ao-out"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
    `;
    try {
        const r = await api.accountsOverview();
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const out = mount.querySelector('#ao-out');
        if (out) out.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function render(r, mount) {
    const cls = (v) => v == null ? '' : v >= 0 ? 'pos' : 'neg';
    const g = r.grand_total;
    const out = mount.querySelector('#ao-out');
    if (!out) return;
    out.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Accounts</div>
                <div class="value">${g.accounts}</div></div>
            <div class="card"><div class="label">Closed trades (∑)</div>
                <div class="value">${g.trade_count}</div>
                <div class="small muted">${g.win_count}W / ${g.loss_count}L</div></div>
            <div class="card"><div class="label">Today P/L (∑)</div>
                <div class="value ${cls(g.today_pnl)}">$${fmt(g.today_pnl)}</div></div>
            <div class="card"><div class="label">MTD P/L (∑)</div>
                <div class="value ${cls(g.mtd_pnl)}">$${fmt(g.mtd_pnl)}</div></div>
            <div class="card"><div class="label">YTD P/L (∑)</div>
                <div class="value ${cls(g.ytd_pnl)}">$${fmt(g.ytd_pnl)}</div></div>
            <div class="card"><div class="label">All-time P/L (∑)</div>
                <div class="value ${cls(g.total_closed_pnl)}">$${fmt(g.total_closed_pnl)}</div></div>
            <div class="card"><div class="label">Open positions (∑)</div>
                <div class="value">${g.open_positions_count}</div>
                <div class="small muted">$${fmt(g.open_notional)} notional</div></div>
            <div class="card"><div class="label">Unrealized (∑)</div>
                <div class="value ${cls(g.open_unrealized_pnl)}">$${fmt(g.open_unrealized_pnl)}</div>
                <div class="small muted">day Δ ${g.open_day_pnl >= 0 ? '+' : ''}$${fmt(g.open_day_pnl)}</div></div>
        </div>

        <div class="chart-panel">
            <h2>${esc(t('view.accounts_overview.h2.breakdown', { count: r.accounts.length }))}</h2>
            ${accountTable(r.accounts)}
            <p class="muted small">Updated ${new Date(r.computed_at).toLocaleString()}.</p>
        </div>
    `;
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
                <div class="small muted ${cls(a.open_day_pnl)}">day ${a.open_day_pnl >= 0 ? '+' : ''}$${fmt(a.open_day_pnl)}</div></td>
        </tr>`).join('')}
        </tbody></table>`;
}
