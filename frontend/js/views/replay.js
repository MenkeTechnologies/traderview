// Trade replay — Warrior-style scrub through a historical session's executions
// against a 1m bar chart.
import { api } from '../api.js';
import { ohlcChart } from '../charts.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

export async function renderReplay(mount, state, day) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = '<p data-i18n="view.replay.hint.no_account" class="boot">No account.</p>';
        return;
    }
    if (!day) day = new Date().toISOString().slice(0, 10);
    const trades = await api.trades(state.accountId, { date_from: day, date_to: day, limit: 500 });
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.replay.title">// REPLAY ·</span>
            <input type="date" id="day" value="${day}" data-tip="view.replay.tip.day">
        </h1>
        ${trades.length ? `
            <div class="chart-panel">
                <h2>${esc(t('view.replay.h2.trades_on', { day, count: trades.length }))}</h2>
                <select id="trade-pick" data-tip="view.replay.tip.trade_pick">
                    ${trades.map(t => `<option value="${t.id}">${esc(t.symbol)} · ${t.side} · ${fmtDateTime(t.opened_at).slice(11)} → ${t.closed_at ? fmtDateTime(t.closed_at).slice(11) : 'open'}</option>`).join('')}
                </select>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.replay.h2.chart" id="replay-title">Chart</h2>
                <div id="replay-chart"></div>
                <div id="replay-execs"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.replay.h2.exec_chart">Execution price by order #</h2>
                <div id="replay-exec-chart" style="width:100%;height:240px"></div>
            </div>
        ` : '<p data-i18n="view.replay.hint.no_closed_trades_on_this_day" class="muted">No closed trades on this day.</p>'}
    `;
    const dayEl = mount.querySelector('#day');
    if (dayEl) dayEl.addEventListener('change', (e) => {
        renderReplay(mount, state, e.target.value);
    });
    const pick = mount.querySelector('#trade-pick');
    if (pick) {
        pick.addEventListener('change', () => loadTrade(pick.value));
        loadTrade(pick.value);
    }

    async function loadTrade(id) {
        let trade, execs;
        try {
            trade = await api.trade(id);
            if (!viewIsCurrent(tok)) return;
            execs = await api.executionsForTrade(id);
            if (!viewIsCurrent(tok)) return;
        } catch (err) {
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
            return;
        }
        const from = Math.floor(new Date(trade.opened_at).getTime() / 1000) - 1800;
        const to   = Math.floor((trade.closed_at ? new Date(trade.closed_at).getTime() : Date.now()) / 1000) + 1800;
        const span = to - from;
        const iv = span < 3600 ? '5m' : span < 86400 ? '15m' : '1d';
        const bars = await api.bars(trade.symbol, iv, from, to).catch(() => ({ bars: [] }));
        if (!viewIsCurrent(tok)) return;
        const title = mount.querySelector('#replay-title');
        if (title) title.textContent =
            `${trade.symbol} · ${trade.side} · entry ${fmt(trade.entry_avg)} · ` +
            (trade.exit_avg != null ? `exit ${fmt(trade.exit_avg)} · net $${fmt(trade.net_pnl)}` : 'open');
        const marks = execs.map(e => ({
            x: new Date(e.executed_at).getTime() / 1000,
            y: Number(e.price),
            side: e.side === 'buy' || e.side === 'cover' ? 'buy' : 'sell',
        }));
        const chart = mount.querySelector('#replay-chart');
        if (chart) ohlcChart(chart, bars.bars || [], marks, { height: 380 });
        const execsEl = mount.querySelector('#replay-execs');
        if (!execsEl) return;
        execsEl.innerHTML = `
            <table class="trades">
                <thead><tr><th data-i18n="view.replay.th.time">Time</th><th data-i18n="view.replay.th.side">Side</th><th data-i18n="view.replay.th.qty">Qty</th><th data-i18n="view.replay.th.price">Price</th><th data-i18n="view.replay.th.fee">Fee</th></tr></thead>
                <tbody>${execs.map(e => `
                    <tr><td>${fmtDateTime(e.executed_at)}</td>
                    <td>${e.side}</td><td>${fmt(e.qty, 0)}</td>
                    <td>${fmt(e.price)}</td><td>${fmt(e.fee)}</td></tr>`).join('')}
                </tbody></table>
        `;
        renderExecChart(execs);
    }
}

function renderExecChart(execs) {
    const el = document.getElementById('replay-exec-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (execs || []).filter(e => Number.isFinite(Number(e.price)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.replay.empty_chart">${esc(t('view.replay.empty_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => new Date(a.executed_at) - new Date(b.executed_at));
    const xs = valid.map((_, i) => i + 1);
    const buys = valid.map(e => (e.side === 'buy' || e.side === 'cover') ? Number(e.price) : null);
    const sells = valid.map(e => (e.side === 'sell' || e.side === 'short') ? Number(e.price) : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.replay.chart.exec_idx') },
            { label: t('view.replay.chart.buy'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.replay.chart.sell'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 } ],
        legend: { show: true },
    }, [xs, buys, sells], el);
}
