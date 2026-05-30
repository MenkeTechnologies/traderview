// Risk dashboard — Warrior-style daily P&L tracker vs goal + max loss.
import { api } from '../api.js';
import { esc, fmt, fmtMoney } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

export async function renderRisk(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = '<p data-i18n="view.risk.hint.no_account" class="boot">No account.</p>';
        return;
    }
    const [s, summary, eq] = await Promise.all([
        api.settings(),
        api.summary(state.accountId),
        api.equity(state.accountId),
    ]);
    if (!viewIsCurrent(tok)) return;
    const today = new Date().toISOString().slice(0, 10);
    const todayPnl = (eq || []).find(p => p.day === today)?.day_net_pnl ?? 0;
    const todayNum = Number(todayPnl);
    const goal = Number(s.daily_profit_goal || 0);
    const maxLoss = Number(s.daily_max_loss || 0);
    const goalPct  = goal    > 0 ? Math.max(0, Math.min(100, (todayNum / goal) * 100)) : 0;
    const lossPct  = maxLoss > 0 ? Math.max(0, Math.min(100, (-todayNum / maxLoss) * 100)) : 0;
    const hitGoal  = goal > 0 && todayNum >= goal;
    const hitMax   = maxLoss > 0 && todayNum <= -maxLoss;

    mount.innerHTML = `
        <h1 class="view-title">// RISK · ${esc(today)}</h1>

        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.risk.card.today_pnl">Today P&L</div>
                <div class="value ${todayNum >= 0 ? 'pos' : 'neg'}">${fmtMoney(todayNum)}</div></div>
            <div class="card"><div class="label" data-i18n="view.risk.card.daily_goal">Daily goal</div>
                <div class="value">${fmtMoney(goal)}</div></div>
            <div class="card"><div class="label" data-i18n="view.risk.card.max_loss">Max loss</div>
                <div class="value">${fmtMoney(maxLoss)}</div></div>
            <div class="card"><div class="label" data-i18n="view.risk.card.alltime_net">All-time net</div>
                <div class="value ${Number(summary.net_pnl) >= 0 ? 'pos' : 'neg'}">${fmtMoney(summary.net_pnl)}</div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk.h2.goal_progress">Goal progress</h2>
            ${goal > 0 ? `
                <div class="risk-bar-wrap">
                    <div class="risk-bar pos" style="width:${goalPct.toFixed(1)}%"></div>
                    <span class="risk-bar-label">${goalPct.toFixed(1)}% of ${fmtMoney(goal)}</span>
                </div>
                ${hitGoal ? `<p class="pos"><strong>${t('view.risk.alert.goal_hit')}</strong></p>` : ''}
            ` : '<p data-i18n="view.risk.hint.set_a_daily_profit_goal_under_settings_to_enable_p" class="muted">Set a daily profit goal under Settings to enable progress tracking.</p>'}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk.h2.max_loss_tracker">Max-loss tracker</h2>
            ${maxLoss > 0 ? `
                <div class="risk-bar-wrap">
                    <div class="risk-bar neg" style="width:${lossPct.toFixed(1)}%"></div>
                    <span class="risk-bar-label">${lossPct.toFixed(1)}% of -${fmtMoney(maxLoss)}</span>
                </div>
                ${hitMax ? `<p class="neg"><strong>${t('view.risk.alert.max_loss_hit')}</strong></p>` :
                  (lossPct > 60 ? `<p class="neg"><strong>${t('view.risk.alert.approaching_max')}</strong></p>` : '')}
            ` : '<p data-i18n="view.risk.hint.set_a_daily_max_loss_under_settings_to_enable_the_" class="muted">Set a daily max loss under Settings to enable the tracker.</p>'}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk.h2.adjust_limits">Adjust limits</h2>
            <form id="risk-form" class="inline-form">
                <label><span data-i18n="view.risk.label.goal">Daily profit goal</span>
                    <input name="goal" type="number" step="any" value="${goal}"></label>
                <label><span data-i18n="view.risk.label.max_loss">Daily max loss</span>
                    <input name="max" type="number" step="any" value="${maxLoss}"></label>
                <button data-i18n="view.risk.btn.save" class="primary" type="submit">Save</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.risk.h2.daily_pnl_chart">Recent daily P&L vs goal / max-loss</h2>
            <div id="rk-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    renderDailyPnlChart(eq, goal, maxLoss);
    mount.querySelector('#risk-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = Object.assign({}, s, {
            daily_profit_goal: Number(fd.get('goal') || 0),
            daily_max_loss:    Number(fd.get('max')  || 0),
        });
        await api.updateSettings(body);
        if (!viewIsCurrent(tok)) return;
        renderRisk(mount, state);
    });
    void fmt;
}

function renderDailyPnlChart(eq, goal, maxLoss) {
    const el = document.getElementById('rk-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (eq || [])
        .filter(p => p.day && Number.isFinite(Number(p.day_net_pnl)))
        .slice(-30);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.risk.empty_chart">${esc(t('view.risk.empty_chart'))}</div>`;
        return;
    }
    const labels = rows.map(p => p.day);
    const xs = labels.map((_, i) => i + 1);
    const winY  = rows.map(p => Number(p.day_net_pnl) >= 0 ? Number(p.day_net_pnl) : null);
    const loseY = rows.map(p => Number(p.day_net_pnl) <  0 ? Number(p.day_net_pnl) : null);
    const goalY = xs.map(() => goal > 0 ?  goal : null);
    const lossY = xs.map(() => maxLoss > 0 ? -maxLoss : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.risk.chart.day') },
            { label: t('view.risk.chart.win'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.risk.chart.lose'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.risk.chart.goal'),
              stroke: '#00e5ff', width: 1.2, dash: [4, 4],
              points: { show: false } },
            { label: t('view.risk.chart.max_loss'),
              stroke: '#ffd84a', width: 1.2, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 56 },
        ],
        legend: { show: true },
    }, [xs, winY, loseY, goalY, lossY], el);
}
