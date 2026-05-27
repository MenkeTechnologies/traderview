// Trade replay — Warrior-style scrub through a historical session's executions
// against a 1m bar chart.
import { api } from '../api.js';
import { ohlcChart } from '../charts.js';
import { esc, fmt, fmtDateTime } from '../util.js';

export async function renderReplay(mount, state, day) {
    if (!state.accountId) {
        mount.innerHTML = '<p class="boot">No account.</p>';
        return;
    }
    if (!day) day = new Date().toISOString().slice(0, 10);
    const trades = await api.trades(state.accountId, { date_from: day, date_to: day, limit: 500 });
    mount.innerHTML = `
        <h1 class="view-title">// REPLAY ·
            <input type="date" id="day" value="${day}">
        </h1>
        ${trades.length ? `
            <div class="chart-panel">
                <h2>Trades on ${esc(day)} — ${trades.length}</h2>
                <select id="trade-pick">
                    ${trades.map(t => `<option value="${t.id}">${esc(t.symbol)} · ${t.side} · ${fmtDateTime(t.opened_at).slice(11)} → ${t.closed_at ? fmtDateTime(t.closed_at).slice(11) : 'open'}</option>`).join('')}
                </select>
            </div>
            <div class="chart-panel">
                <h2 id="replay-title">Chart</h2>
                <div id="replay-chart"></div>
                <div id="replay-execs"></div>
            </div>
        ` : '<p class="muted">No closed trades on this day.</p>'}
    `;
    document.getElementById('day').addEventListener('change', (e) => {
        renderReplay(mount, state, e.target.value);
    });
    const pick = document.getElementById('trade-pick');
    if (pick) {
        pick.addEventListener('change', () => loadTrade(pick.value));
        loadTrade(pick.value);
    }

    async function loadTrade(id) {
        const trade = await api.trade(id);
        const execs = await api.executionsForTrade(id);
        const from = Math.floor(new Date(trade.opened_at).getTime() / 1000) - 1800;
        const to   = Math.floor((trade.closed_at ? new Date(trade.closed_at).getTime() : Date.now()) / 1000) + 1800;
        const span = to - from;
        const iv = span < 3600 ? '5m' : span < 86400 ? '15m' : '1d';
        const bars = await api.bars(trade.symbol, iv, from, to).catch(() => ({ bars: [] }));
        document.getElementById('replay-title').textContent =
            `${trade.symbol} · ${trade.side} · entry ${fmt(trade.entry_avg)} · ` +
            (trade.exit_avg != null ? `exit ${fmt(trade.exit_avg)} · net $${fmt(trade.net_pnl)}` : 'open');
        const marks = execs.map(e => ({
            x: new Date(e.executed_at).getTime() / 1000,
            y: Number(e.price),
            side: e.side === 'buy' || e.side === 'cover' ? 'buy' : 'sell',
        }));
        ohlcChart(document.getElementById('replay-chart'), bars.bars || [], marks, { height: 380 });
        document.getElementById('replay-execs').innerHTML = `
            <table class="trades">
                <thead><tr><th>Time</th><th>Side</th><th>Qty</th><th>Price</th><th>Fee</th></tr></thead>
                <tbody>${execs.map(e => `
                    <tr><td>${fmtDateTime(e.executed_at)}</td>
                    <td>${e.side}</td><td>${fmt(e.qty, 0)}</td>
                    <td>${fmt(e.price)}</td><td>${fmt(e.fee)}</td></tr>`).join('')}
                </tbody></table>
        `;
    }
}
