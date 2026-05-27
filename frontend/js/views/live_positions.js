// Live P/L tracker — polls every 30s, highlights biggest mover, shows
// account-level day delta.

import { api, ApiError } from '../api.js';
import { esc, fmt } from '../util.js';
import { go, currentViewToken, viewIsCurrent } from '../app.js';

let timer = null;

export async function renderLivePositions(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) {
        mount.innerHTML = `<p class="boot">No account selected.</p>`;
        return;
    }
    mount.innerHTML = `
        <h1 class="view-title">// LIVE P/L — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small">Snapshot of every open trade with fresh Yahoo quotes (60s
            server-cached). Unrealized P/L honors multiplier and side (long/short). Day P/L
            uses the quote's prev_close. Refreshes every 30 seconds.</p>

        <div id="lp-cards" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>
        <div id="lp-table"></div>
        <p class="muted small" id="lp-status"></p>
    `;
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
        const st = mount.querySelector('#lp-status');
        if (st) st.textContent =
            `Updated ${new Date(r.fetched_at).toLocaleTimeString(undefined, { hour12: false })} · ${r.position_count} positions`;
    } catch (e) {
        if (e instanceof ApiError && e.status === 401) return;
        if (!viewIsCurrent(tok)) return;
        const st = mount.querySelector('#lp-status');
        if (st) st.textContent = 'error: ' + e.message;
    }
}

function renderCards(r, mount) {
    const cls = (v) => v == null ? '' : v >= 0 ? 'pos' : 'neg';
    const el = mount.querySelector('#lp-cards');
    if (!el) return;
    el.innerHTML = `
        <div class="card"><div class="label">Open positions</div>
            <div class="value">${r.position_count}</div></div>
        <div class="card"><div class="label">Total notional</div>
            <div class="value">$${fmt(r.total_notional)}</div></div>
        <div class="card"><div class="label">Unrealized P/L</div>
            <div class="value ${cls(r.total_unrealized_pnl)}">${signed$(r.total_unrealized_pnl)}</div></div>
        <div class="card"><div class="label">Day P/L</div>
            <div class="value ${cls(r.total_day_pnl)}">${signed$(r.total_day_pnl)}</div></div>
        <div class="card"><div class="label">Biggest winner</div>
            <div class="value pos">${esc(r.biggest_winner || '—')}</div></div>
        <div class="card"><div class="label">Biggest loser</div>
            <div class="value neg">${esc(r.biggest_loser || '—')}</div></div>
    `;
}

function renderTable(r, mount) {
    const tbl = mount.querySelector('#lp-table');
    if (!tbl) return;
    if (!r.positions.length) {
        tbl.innerHTML =
            '<div class="chart-panel"><p class="muted small">No open positions.</p></div>';
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
        return `<tr>
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
                    <th>Symbol</th><th>Qty</th><th>Entry</th><th>Last</th>
                    <th>Δ today</th><th>Notional</th><th>UPnL</th><th>UPnL %</th>
                    <th>Day P/L</th><th>State</th>
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
