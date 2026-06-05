// IRC § 4942 — Private Foundation Undistributed Income.
// PF must distribute 5% of average non-charitable-use asset value annually for charitable purposes.
// Failure: 30% initial excise on undistributed amount + 100% if not corrected within taxable period.
// "Distributable amount" = minimum investment return - tax + carryover.
// "Qualifying distributions" = grants, program expenses, set-asides, direct charitable activities.

import { currentViewToken, viewIsCurrent } from '../app.js';

const PAYOUT_RATE = 0.05;
const INITIAL_EXCISE = 0.30;
const SECOND_EXCISE = 1.00;

let state = {
    average_assets: 0,
    cash_equivalents_for_charitable_use: 0,
    qualifying_distributions: 0,
    prior_year_carryover: 0,
    next_year_set_aside: 0,
    investment_income_tax_paid: 0,
    is_operating_foundation: false,
    years_uncorrected: 0,
};

export async function renderSection4942(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4942.h1.title">// § 4942 PF UNDISTRIBUTED INCOME</span></h1>
        <p class="muted small" data-i18n="view.s4942.hint.intro">
            PF must distribute <strong>5% of average non-charitable-use asset value</strong>
            annually for charitable purposes. <strong>30% initial excise on undistributed</strong>
            + <strong>100% if not corrected</strong> within taxable period. Carryforward 5 yrs.
            Form 990-PF Part XII. <strong>Operating foundations exempt</strong> (substantially-all
            assets directly used in charitable activities).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4942.h2.inputs">Inputs</h2>
            <form id="s4942-form" class="inline-form">
                <label><span data-i18n="view.s4942.label.assets">Average non-charitable-use assets ($)</span>
                    <input type="number" step="0.01" name="average_assets" value="${state.average_assets}"></label>
                <label><span data-i18n="view.s4942.label.cash">Cash for charitable use ($)</span>
                    <input type="number" step="0.01" name="cash_equivalents_for_charitable_use" value="${state.cash_equivalents_for_charitable_use}"></label>
                <label><span data-i18n="view.s4942.label.qualifying">Qualifying distributions YTD ($)</span>
                    <input type="number" step="0.01" name="qualifying_distributions" value="${state.qualifying_distributions}"></label>
                <label><span data-i18n="view.s4942.label.carryover">Prior year carryover ($)</span>
                    <input type="number" step="0.01" name="prior_year_carryover" value="${state.prior_year_carryover}"></label>
                <label><span data-i18n="view.s4942.label.set_aside">Set-aside for future projects ($)</span>
                    <input type="number" step="0.01" name="next_year_set_aside" value="${state.next_year_set_aside}"></label>
                <label><span data-i18n="view.s4942.label.tax">Investment income tax (§ 4940) paid ($)</span>
                    <input type="number" step="0.01" name="investment_income_tax_paid" value="${state.investment_income_tax_paid}"></label>
                <label><span data-i18n="view.s4942.label.operating">Operating foundation?</span>
                    <input type="checkbox" name="is_operating_foundation" ${state.is_operating_foundation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4942.label.years_uncorr">Years uncorrected</span>
                    <input type="number" step="1" name="years_uncorrected" value="${state.years_uncorrected}"></label>
                <button class="primary" type="submit" data-i18n="view.s4942.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4942-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4942.h2.qualifying">Qualifying distributions</h2>
            <ul class="muted small">
                <li data-i18n="view.s4942.qual.grants">Grants to public charities + foreign organizations (with expenditure responsibility)</li>
                <li data-i18n="view.s4942.qual.direct">Direct charitable activities (educational programs, research, museum operations)</li>
                <li data-i18n="view.s4942.qual.set_aside">Set-asides for specific future projects (5-yr limit + IRS approval)</li>
                <li data-i18n="view.s4942.qual.administrative">Reasonable administrative expenses (Form 990-PF Part IV)</li>
                <li data-i18n="view.s4942.qual.exempt_purpose">Acquisition of property for exempt purposes</li>
                <li data-i18n="view.s4942.qual.amounts_paid">Amounts paid OR set aside for charitable purposes</li>
                <li data-i18n="view.s4942.qual.foreign">Grants to foreign organizations with equivalency determination</li>
                <li data-i18n="view.s4942.qual.daf_no">Grants to DAFs DO count (since SECURE 2.0)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4942.h2.carryforward">Carryover + carryback rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s4942.cf.5_year">Excess distributions: 5-year carryforward</li>
                <li data-i18n="view.s4942.cf.fifo">FIFO usage: oldest carryover absorbed first</li>
                <li data-i18n="view.s4942.cf.no_carryback">NO carryback (unlike NOL)</li>
                <li data-i18n="view.s4942.cf.set_aside">Set-aside elections require IRS approval + 5-yr limit</li>
                <li data-i18n="view.s4942.cf.unusual_grant">Unusual grant exception (Rev. Proc. 76-13) for one-time mega contributions</li>
                <li data-i18n="view.s4942.cf.transition">Newly-converted PF: 4-yr transition rule for full payout requirement</li>
            </ul>
        </div>
    `;
    document.getElementById('s4942-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.average_assets = Number(fd.get('average_assets')) || 0;
        state.cash_equivalents_for_charitable_use = Number(fd.get('cash_equivalents_for_charitable_use')) || 0;
        state.qualifying_distributions = Number(fd.get('qualifying_distributions')) || 0;
        state.prior_year_carryover = Number(fd.get('prior_year_carryover')) || 0;
        state.next_year_set_aside = Number(fd.get('next_year_set_aside')) || 0;
        state.investment_income_tax_paid = Number(fd.get('investment_income_tax_paid')) || 0;
        state.is_operating_foundation = !!fd.get('is_operating_foundation');
        state.years_uncorrected = Number(fd.get('years_uncorrected')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4942-output');
    if (!el) return;
    const netAssets = Math.max(0, state.average_assets - state.cash_equivalents_for_charitable_use);
    const minimumInvestmentReturn = netAssets * PAYOUT_RATE;
    const distributableAmount = Math.max(0, minimumInvestmentReturn - state.investment_income_tax_paid);
    const totalDistributions = state.qualifying_distributions + state.next_year_set_aside + state.prior_year_carryover;
    const undistributed = state.is_operating_foundation ? 0 : Math.max(0, distributableAmount - totalDistributions);
    const exciseInitial = undistributed * INITIAL_EXCISE * Math.max(1, state.years_uncorrected);
    const exciseSecond = state.years_uncorrected >= 2 ? undistributed * SECOND_EXCISE : 0;
    const totalExcise = exciseInitial + exciseSecond;
    const newCarryforward = Math.max(0, totalDistributions - distributableAmount);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4942.h2.result">§ 4942 compliance</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4942.card.assets">Net non-charitable assets</div>
                    <div class="value">$${netAssets.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4942.card.mir">Minimum investment return (5%)</div>
                    <div class="value">$${minimumInvestmentReturn.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4942.card.distributable">Distributable amount</div>
                    <div class="value">$${distributableAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4942.card.distributions">Total distributions</div>
                    <div class="value">$${totalDistributions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${undistributed > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4942.card.undistributed">Undistributed amount</div>
                    <div class="value">$${undistributed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalExcise > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4942.card.excise">§ 4942 excise tax</div>
                    <div class="value">$${totalExcise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s4942.card.carryforward">Carryforward to next year</div>
                    <div class="value">$${newCarryforward.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
