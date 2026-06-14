// Rent vs sell — keep the rental (appreciation + reinvested cash flow) vs
// sell now and invest the proceeds, compared as end-of-horizon wealth, via
// /calc/rent-vs-sell. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const VIEW = 'rent-vs-sell';
let lastReport = null;
let lastBody = null;

const FIELDS = [
    ['current_value_usd', 'Current value ($)', 300000],
    ['cost_basis_usd', 'Adjusted cost basis ($)', 200000],
    ['mortgage_balance_usd', 'Mortgage balance ($)', 0],
    ['selling_cost_pct', 'Selling cost (% of price)', 6],
    ['capital_gains_tax_pct', 'Capital-gains rate (%)', 20],
    ['annual_rent_usd', 'Annual rent ($)', 24000],
    ['annual_operating_expenses_usd', 'Annual operating expenses ($)', 8000],
    ['annual_mortgage_payment_usd', 'Annual mortgage payment ($)', 0],
    ['annual_appreciation_pct', 'Appreciation (%/yr)', 4],
    ['annual_rent_growth_pct', 'Rent growth (%/yr)', 3],
    ['alternative_return_pct', 'Alternative return (%/yr)', 7],
    ['years', 'Horizon (years)', 10],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderRentVsSell(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rvs.h1.title">// RENT vs SELL</span></h1>
        <p class="muted small" data-i18n="view.rvs.hint.intro">
            Keep the property as a rental or sell it and invest the proceeds? This compares
            total wealth at the end of your horizon, both assuming you liquidate at the end.
            Sell now: net proceeds (value − selling costs − mortgage − capital-gains tax)
            compounded at your alternative return. Keep: the property appreciates and throws
            off rental cash flow (reinvested), then sells at the horizon. Mortgage paydown is
            ignored (conservative for keeping). Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rvs.h2.inputs">The property</h2>
            <form id="rvs-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.rvs.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="rvs-tools" class="ce-toolbar"></div>
        </div>
        <div id="rvs-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rvs-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        body.years = Math.max(0, Math.round(body.years));
        return body;
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcRentVsSell(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.rvs.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#rvs-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'rent-vs-sell.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['keep_wealth_usd', r.keep_wealth_usd],
        ['sell_wealth_usd', r.sell_wealth_usd],
        ['keep_advantage_usd', r.keep_advantage_usd],
        ['keep_wins', r.keep_wins],
        ['accumulated_cash_flow_usd', r.accumulated_cash_flow_usd],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#rvs-result');
    const winnerCls = r.keep_wins ? 'pos' : 'neg';
    // End-of-horizon wealth: keep the rental vs sell now and invest.
    const chart = enh.svgBarChart([
        { label: 'Keep', value: r.keep_wealth_usd },
        { label: 'Sell', value: r.sell_wealth_usd },
    ]);
    const winner = r.keep_wins ? t('view.rvs.winner.keep') : t('view.rvs.winner.sell');
    const adv = Number(r.keep_advantage_usd);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rvs.h2.result">The decision</h2>
            <div class="cards">
                <div class="card ${winnerCls}"><div class="label" data-i18n="view.rvs.card.winner">Better choice</div>
                    <div class="value ${winnerCls}">${winner}</div></div>
                <div class="card"><div class="label" data-i18n="view.rvs.card.advantage">Keep advantage</div>
                    <div class="value ${adv >= 0 ? 'pos' : 'neg'}">${adv >= 0 ? '+' : '−'}${money(Math.abs(adv))}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.rvs.col.line">Line</th>
                    <th data-i18n="view.rvs.col.keep">Keep</th>
                    <th data-i18n="view.rvs.col.sell">Sell now</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.rvs.row.proceeds">Net sale proceeds</td>
                        <td>${money(r.keep_sale_proceeds_usd)}</td><td>${money(r.sell_now_proceeds_usd)}</td></tr>
                    <tr><td data-i18n="view.rvs.row.future_value">Property value at horizon</td>
                        <td>${money(r.future_value_usd)}</td><td>—</td></tr>
                    <tr><td data-i18n="view.rvs.row.cashflow">Reinvested cash flow</td>
                        <td>${money(r.accumulated_cash_flow_usd)}</td><td>—</td></tr>
                    <tr class="emph"><td data-i18n="view.rvs.row.wealth">Total wealth at horizon</td>
                        <td class="pos">${money(r.keep_wealth_usd)}</td><td class="pos">${money(r.sell_wealth_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
