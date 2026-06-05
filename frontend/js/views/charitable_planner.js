// Charitable Contribution Planner — donor-advised funds, appreciated stock
// donations vs cash. Compares after-tax cost of giving $X cash vs $X of
// appreciated stock (which avoids cap-gains tax + still gives full FMV deduction).
// Deduction limits: 60% AGI for cash, 30% AGI for long-term appreciated stock.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const CASH_LIMIT_PCT = 0.60;
const APPRECIATED_LIMIT_PCT = 0.30;

let state = {
    agi: 250_000,
    intended_gift: 25_000,
    cost_basis: 5_000,
    holding_period_long: true,
    marginal_rate: 0.32,
    cap_gains_rate: 0.20,
    niit: 0.038,
    bunch_years: 1,
};

export async function renderCharitablePlanner(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.charity.h1.title">// CHARITABLE PLANNER</span></h1>
        <p class="muted small" data-i18n="view.charity.hint.intro">
            Donating appreciated stock instead of cash = avoid cap-gains tax AND get
            full FMV deduction. Compare side-by-side. Bunching multiple years of giving
            into one tax year + using a Donor Advised Fund (DAF) maximizes itemized
            deductions in alternating years.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.charity.h2.inputs">Inputs</h2>
            <form id="ch-form" class="inline-form">
                <label><span data-i18n="view.charity.label.agi">AGI ($)</span>
                    <input type="number" step="0.01" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.charity.label.intended_gift">Intended gift FMV ($)</span>
                    <input type="number" step="0.01" name="intended_gift" value="${state.intended_gift}"></label>
                <label><span data-i18n="view.charity.label.cost_basis">Your cost basis on the stock</span>
                    <input type="number" step="0.01" name="cost_basis" value="${state.cost_basis}"></label>
                <label><span data-i18n="view.charity.label.holding_period">Held >1 year (long-term)?</span>
                    <input type="checkbox" name="holding_period_long" ${state.holding_period_long ? 'checked' : ''}></label>
                <label><span data-i18n="view.charity.label.marginal_rate">Marginal rate %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.charity.label.cap_gains_rate">LT cap-gains rate %</span>
                    <input type="number" step="0.5" name="cap_gains_rate" value="${(state.cap_gains_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.charity.label.niit">NIIT %</span>
                    <input type="number" step="0.1" name="niit" value="${(state.niit * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.charity.label.bunch_years">DAF bunch years</span>
                    <input type="number" step="1" name="bunch_years" value="${state.bunch_years}" min="1" max="10"></label>
                <button class="primary" type="submit" data-i18n="view.charity.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="ch-output"></div>
    `;
    document.getElementById('ch-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.agi = Number(fd.get('agi')) || 0;
        state.intended_gift = Number(fd.get('intended_gift')) || 0;
        state.cost_basis = Number(fd.get('cost_basis')) || 0;
        state.holding_period_long = !!fd.get('holding_period_long');
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        state.cap_gains_rate = (Number(fd.get('cap_gains_rate')) || 20) / 100;
        state.niit = (Number(fd.get('niit')) || 3.8) / 100;
        state.bunch_years = Number(fd.get('bunch_years')) || 1;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('ch-output');
    if (!el) return;
    const gift = state.intended_gift;
    const gain = gift - state.cost_basis;
    const ltStock = state.holding_period_long;

    // Cash gift
    const cashDeductionCap = state.agi * CASH_LIMIT_PCT;
    const cashDeduction = Math.min(gift, cashDeductionCap);
    const cashTaxSavings = cashDeduction * state.marginal_rate;
    const cashNetCost = gift - cashTaxSavings;

    // Appreciated stock gift (LT)
    const stockDeductionCap = state.agi * (ltStock ? APPRECIATED_LIMIT_PCT : CASH_LIMIT_PCT);
    const stockFMV = gift;  // donated at FMV
    const stockDeduction = ltStock
        ? Math.min(stockFMV, stockDeductionCap)
        : Math.min(state.cost_basis, stockDeductionCap);  // ST limited to basis
    const avoidedCapGainsTax = ltStock && gain > 0
        ? gain * (state.cap_gains_rate + state.niit)
        : 0;
    const stockTaxSavings = stockDeduction * state.marginal_rate + avoidedCapGainsTax;
    const stockNetCost = stockFMV - stockTaxSavings;

    const savings = cashNetCost - stockNetCost;
    const recommendation = savings > 0
        ? t('view.charity.recommend.appreciated')
        : t('view.charity.recommend.cash');

    // DAF bunching: bunch N years into year-1, take standard deduction in years 2..N
    const bunchedGift = gift * state.bunch_years;
    const bunchedDeduction = Math.min(bunchedGift, state.agi * CASH_LIMIT_PCT);
    const bunchedSavings = bunchedDeduction * state.marginal_rate;

    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.charity.h2.comparison">Cash vs appreciated stock</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.charity.card.savings">Savings (stock − cash)</div>
                    <div class="value">$${savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.charity.card.recommendation">Recommendation</div>
                    <div class="value">${esc(recommendation)}</div>
                </div>
            </div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.charity.h2.cash">Cash gift</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.charity.row.gift_amount">Gift amount</td>
                        <td>$${gift.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.charity.row.deduction_cap">Deduction cap (60% AGI)</td>
                        <td>$${cashDeductionCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.charity.row.deduction">Deduction</td>
                        <td>$${cashDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.charity.row.tax_savings">Tax savings</td>
                        <td class="pos">$${cashTaxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td><strong data-i18n="view.charity.row.net_cost">Net cost of gift</strong></td>
                        <td><strong class="neg">$${cashNetCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                </tbody></table>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.charity.h2.appreciated">Appreciated stock gift</h2>
                <table class="trades"><tbody>
                    <tr><td data-i18n="view.charity.row.stock_fmv">Stock FMV</td>
                        <td>$${stockFMV.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.charity.row.embedded_gain">Embedded gain</td>
                        <td>$${gain.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.charity.row.deduction_cap_30">Deduction cap (${ltStock ? '30' : '60'}% AGI)</td>
                        <td>$${stockDeductionCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.charity.row.deduction">Deduction</td>
                        <td>$${stockDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.charity.row.avoided_cap_gains">Avoided cap-gains tax</td>
                        <td class="pos">$${avoidedCapGainsTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td data-i18n="view.charity.row.tax_savings">Total tax savings</td>
                        <td class="pos">$${stockTaxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                    <tr><td><strong data-i18n="view.charity.row.net_cost">Net cost of gift</strong></td>
                        <td><strong class="neg">$${stockNetCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                </tbody></table>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.charity.h2.bunch">DAF bunching strategy</h2>
            <p>
                ${esc(t('view.charity.bunch.body', {
                    n: state.bunch_years,
                    gift: bunchedGift.toLocaleString(),
                    savings: bunchedSavings.toLocaleString(undefined, { maximumFractionDigits: 0 }),
                }))}
            </p>
            <p class="muted small" data-i18n="view.charity.bunch.note">
                Fidelity / Schwab / Vanguard offer free DAFs. Front-load multiple years of
                giving into one year, get the itemized deduction, then take the standard
                deduction in the off-years. Distribute from the DAF to charities at your pace.
            </p>
        </div>
    `;
}
