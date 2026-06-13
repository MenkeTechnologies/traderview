// Trade expectancy — per-trade edge from win rate + avg win/loss, the
// reward:risk ratio, break-even win rate, and expectancy in R, via
// /calc/trade-expectancy. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['win_rate_pct', 'Win rate (%)', 40],
    ['avg_win_usd', 'Average win ($)', 300],
    ['avg_loss_usd', 'Average loss ($)', 100],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });

export async function renderTradeExpectancy(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.te.h1.title">// TRADE EXPECTANCY</span></h1>
        <p class="muted small" data-i18n="view.te.hint.intro">
            Whether a system makes money long-run isn't the win rate alone — it's the win rate
            weighted by how much you win vs lose: expectancy = (win% × avg win) − (loss% × avg
            loss). The break-even win rate is what you'd need at this reward:risk to net zero; a
            high win rate with a poor reward:risk can still lose. Expectancy in R is the per-trade
            edge in risk units. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.te.h2.inputs">Your stats</h2>
            <form id="te-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.te.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="te-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#te-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcTradeExpectancy(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.te.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#te-result');
    const edgeCls = r.has_edge ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.te.h2.result">The edge</h2>
            <div class="cards">
                <div class="card ${edgeCls}"><div class="label" data-i18n="view.te.card.expectancy">Expectancy / trade</div>
                    <div class="value ${edgeCls}">${money(r.expectancy_per_trade_usd)}</div></div>
                <div class="card ${edgeCls}"><div class="label" data-i18n="view.te.card.edge">Edge</div>
                    <div class="value ${edgeCls}">${r.has_edge ? t('view.te.yes') : t('view.te.no')}</div></div>
                <div class="card"><div class="label" data-i18n="view.te.card.rr">Reward : risk</div>
                    <div class="value">${Number(r.reward_risk_ratio).toFixed(2)} : 1</div></div>
                <div class="card"><div class="label" data-i18n="view.te.card.breakeven">Break-even win rate</div>
                    <div class="value">${Number(r.breakeven_win_rate_pct).toFixed(1)}%</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.te.col.line">Line</th><th data-i18n="view.te.col.value">Value</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.te.row.r">Expectancy in R</td><td class="${edgeCls}">${Number(r.expectancy_in_r).toFixed(3)}R</td></tr>
                    <tr><td data-i18n="view.te.row.loss_rate">Loss rate</td><td>${Number(r.loss_rate_pct).toFixed(1)}%</td></tr>
                    <tr class="emph"><td data-i18n="view.te.row.per100">Expected over 100 trades</td><td class="${edgeCls}">${money(r.expectancy_per_100_trades_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
