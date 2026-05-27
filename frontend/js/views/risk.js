// Risk dashboard — Warrior-style daily P&L tracker vs goal + max loss.
import { api } from '../api.js';
import { esc, fmt, fmtMoney } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderRisk(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) {
        mount.innerHTML = '<p class="boot">No account.</p>';
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
            <div class="card"><div class="label">Today P&L</div>
                <div class="value ${todayNum >= 0 ? 'pos' : 'neg'}">${fmtMoney(todayNum)}</div></div>
            <div class="card"><div class="label">Daily goal</div>
                <div class="value">${fmtMoney(goal)}</div></div>
            <div class="card"><div class="label">Max loss</div>
                <div class="value">${fmtMoney(maxLoss)}</div></div>
            <div class="card"><div class="label">All-time net</div>
                <div class="value ${Number(summary.net_pnl) >= 0 ? 'pos' : 'neg'}">${fmtMoney(summary.net_pnl)}</div></div>
        </div>

        <div class="chart-panel">
            <h2>Goal progress</h2>
            ${goal > 0 ? `
                <div class="risk-bar-wrap">
                    <div class="risk-bar pos" style="width:${goalPct.toFixed(1)}%"></div>
                    <span class="risk-bar-label">${goalPct.toFixed(1)}% of ${fmtMoney(goal)}</span>
                </div>
                ${hitGoal ? '<p class="pos"><strong>🎯 Goal hit — consider stopping for the day.</strong></p>' : ''}
            ` : '<p class="muted">Set a daily profit goal under Settings to enable progress tracking.</p>'}
        </div>

        <div class="chart-panel">
            <h2>Max-loss tracker</h2>
            ${maxLoss > 0 ? `
                <div class="risk-bar-wrap">
                    <div class="risk-bar neg" style="width:${lossPct.toFixed(1)}%"></div>
                    <span class="risk-bar-label">${lossPct.toFixed(1)}% of -${fmtMoney(maxLoss)}</span>
                </div>
                ${hitMax ? '<p class="neg"><strong>🚨 Daily max loss hit — STOP TRADING.</strong></p>' :
                  (lossPct > 60 ? '<p class="neg"><strong>⚠️ Approaching max loss — be cautious.</strong></p>' : '')}
            ` : '<p class="muted">Set a daily max loss under Settings to enable the tracker.</p>'}
        </div>

        <div class="chart-panel">
            <h2>Adjust limits</h2>
            <form id="risk-form" class="inline-form">
                <label>Daily profit goal <input name="goal" type="number" step="any" value="${goal}"></label>
                <label>Daily max loss <input name="max" type="number" step="any" value="${maxLoss}"></label>
                <button class="primary" type="submit">Save</button>
            </form>
        </div>
    `;
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
