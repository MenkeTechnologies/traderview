import { api } from '../api.js';
import { fmt, fmtMoney, fmtPct, fmtSecs, pnlClass, statCard } from '../util.js';
import { equityChart } from '../charts.js';
import { renderWorldMarkets } from './world_map.js';

export async function renderDashboard(mount, state) {
    if (!state.accountId) {
        mount.innerHTML = `
            <h1 class="view-title">// DASHBOARD</h1>
            <div id="world-markets-mount"></div>
            <p class="boot">No account yet. Add one via Accounts, then import or add trades.</p>
        `;
        renderWorldMarkets(document.getElementById('world-markets-mount'));
        return;
    }
    const [summary, equity, cal] = await Promise.all([
        api.summary(state.accountId),
        api.equity(state.accountId),
        api.calendar(state.accountId),
    ]);

    mount.innerHTML = `
        <h1 class="view-title">// DASHBOARD</h1>
        <div id="world-markets-mount"></div>
        <div class="cards">
            ${statCard('Net P&L', fmtMoney(summary.net_pnl), pnlClass(summary.net_pnl))}
            ${statCard('Trades', summary.trade_count)}
            ${statCard('Win rate', fmtPct(summary.win_rate))}
            ${statCard('Profit factor', fmt(summary.profit_factor))}
            ${statCard('Expectancy', fmtMoney(summary.expectancy), pnlClass(summary.expectancy))}
            ${statCard('Avg R', fmt(summary.avg_r))}
            ${statCard('Largest win', fmtMoney(summary.largest_win), 'pos')}
            ${statCard('Largest loss', fmtMoney(summary.largest_loss), 'neg')}
            ${statCard('Max consec wins', summary.max_consec_wins)}
            ${statCard('Max consec losses', summary.max_consec_losses)}
            ${statCard('Avg hold', fmtSecs(summary.avg_hold_seconds))}
            ${statCard('Fees', fmtMoney(summary.fees))}
        </div>

        <div class="chart-panel">
            <h2>Equity Curve</h2>
            <div id="equity-chart"></div>
        </div>

        <div class="chart-panel">
            <h2>Last 90 Days</h2>
            <div class="mini-cal" id="mini-cal"></div>
        </div>
    `;

    equityChart(document.getElementById('equity-chart'), equity);
    renderMiniCalendar(document.getElementById('mini-cal'), cal);
    renderWorldMarkets(document.getElementById('world-markets-mount'));
}

function renderMiniCalendar(el, cells) {
    if (!cells.length) { el.innerHTML = '<div class="boot">No data</div>'; return; }
    const recent = cells.slice(-90);
    const max = Math.max(...recent.map(c => Math.abs(Number(c.net_pnl))), 1);
    el.innerHTML = recent.map(c => {
        const v = Number(c.net_pnl);
        const intensity = Math.min(1, Math.abs(v) / max);
        const color = v >= 0
            ? `rgba(35, 209, 96, ${0.15 + intensity * 0.7})`
            : `rgba(255, 56, 96, ${0.15 + intensity * 0.7})`;
        return `<div class="cal-cell" style="background:${color}"
            title="${c.day} · ${fmtMoney(v)} · ${c.trades} trades"></div>`;
    }).join('');
}
