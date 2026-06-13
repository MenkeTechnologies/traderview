// Break-even / CVP analysis — how many units cover fixed costs, plus
// target-profit volume and margin of safety, via /calc/break-even.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const FIELDS = [
    ['fixed_costs_usd', 'Fixed costs ($/period)', 10000],
    ['price_per_unit_usd', 'Price per unit ($)', 50],
    ['variable_cost_per_unit_usd', 'Variable cost per unit ($)', 30],
    ['target_profit_usd', 'Target profit ($, optional)', 0],
    ['expected_units', 'Expected units sold (optional)', 800],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const units = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 });
const dash = (v, fn) => (v == null ? '—' : fn(v));

export async function renderBreakEven(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.breakeven.h1.title">// BREAK-EVEN ANALYSIS</span></h1>
        <p class="muted small" data-i18n="view.breakeven.hint.intro">
            The most basic small-business question: how many units must you sell to cover
            your costs? Each unit contributes price − variable cost toward the fixed costs
            (its contribution margin); break-even is where that accumulated contribution
            exactly covers them. Add a target profit to see the volume that earns it, and
            your expected sales to see the margin of safety. If the contribution margin is
            zero or negative, no volume ever breaks even — selling more only loses more.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.breakeven.h2.inputs">Your numbers</h2>
            <form id="be-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.breakeven.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                <button class="primary" type="submit" data-i18n="view.breakeven.btn.run">Calculate</button>
            </form>
        </div>
        <div id="be-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#be-form');
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcBreakEven(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.breakeven.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function renderResult(mount, r) {
    const el = mount.querySelector('#be-result');
    const cmRatio = r.contribution_margin_ratio == null
        ? '—'
        : (Number(r.contribution_margin_ratio) * 100).toFixed(1) + '%';

    if (r.no_break_even) {
        el.innerHTML = `
            <div class="chart-panel">
                <h2 data-i18n="view.breakeven.h2.result">The result</h2>
                <div class="cards">
                    <div class="card neg"><div class="label" data-i18n="view.breakeven.card.cm">Contribution margin / unit</div>
                        <div class="value neg">${money(r.contribution_margin_usd)}</div></div>
                    <div class="card neg"><div class="label" data-i18n="view.breakeven.card.be_units">Break-even units</div>
                        <div class="value neg" data-i18n="view.breakeven.never">never</div></div>
                </div>
                <p class="muted small" data-i18n="view.breakeven.warn.no_be">Price is at or below variable cost — every unit loses money before fixed costs. Raise price or cut variable cost.</p>
            </div>
        `;
        applyUiI18n(el);
        return;
    }

    const mos = r.margin_of_safety_units;
    const mosCls = mos == null ? '' : mos >= 0 ? 'pos' : 'neg';
    const profit = r.profit_at_expected_usd;
    const profitCls = profit == null ? '' : profit >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.breakeven.h2.result">The result</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.breakeven.card.be_units">Break-even units</div>
                    <div class="value">${dash(r.break_even_units, units)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.breakeven.card.be_revenue">Break-even revenue</div>
                    <div class="value">${dash(r.break_even_revenue_usd, money)}</div></div>
                <div class="card"><div class="label" data-i18n="view.breakeven.card.cm">Contribution margin / unit</div>
                    <div class="value">${money(r.contribution_margin_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.breakeven.card.cm_ratio">Contribution margin ratio</div>
                    <div class="value">${cmRatio}</div></div>
                <div class="card"><div class="label" data-i18n="view.breakeven.card.target_units">Units for target profit</div>
                    <div class="value">${dash(r.units_for_target_profit, units)}</div></div>
                <div class="card"><div class="label" data-i18n="view.breakeven.card.target_revenue">Revenue for target profit</div>
                    <div class="value">${dash(r.revenue_for_target_profit_usd, money)}</div></div>
                <div class="card ${mosCls}"><div class="label" data-i18n="view.breakeven.card.mos_units">Margin of safety (units)</div>
                    <div class="value ${mosCls}">${dash(mos, units)}</div></div>
                <div class="card ${mosCls}"><div class="label" data-i18n="view.breakeven.card.mos_pct">Margin of safety (%)</div>
                    <div class="value ${mosCls}">${r.margin_of_safety_pct == null ? '—' : Number(r.margin_of_safety_pct).toFixed(1) + '%'}</div></div>
                <div class="card ${profitCls}"><div class="label" data-i18n="view.breakeven.card.profit">Profit at expected volume</div>
                    <div class="value ${profitCls}">${dash(profit, money)}</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
