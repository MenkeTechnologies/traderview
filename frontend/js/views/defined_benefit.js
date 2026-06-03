// Defined Benefit / Cash Balance Plan — for high earners >$300k/yr.
// 2024 § 415(b) max annual benefit: $275,000 at NRA. Contribution can exceed $250k/yr.
// Combine with 401(k) for stack: $69k + $275k = $344k+ tax-deductible per year.
// Requires actuary + Form 5500. Generally for owner-employees age 45+ with low employee headcount.

import { currentViewToken, viewIsCurrent } from '../app.js';

const ANNUAL_BENEFIT_CAP_2024 = 275_000;
const COMP_CAP_2024 = 345_000;

let state = {
    age: 50,
    target_retirement_age: 62,
    annual_compensation: 0,
    has_other_401k: false,
    is_cash_balance: true,
    interest_credit_rate: 0.05,
    funding_discount_rate: 0.05,
    employee_count: 1,
    marginal_rate: 0.37,
};

export async function renderDefinedBenefit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.db.h1.title">// DEFINED BENEFIT / CASH BALANCE</span></h1>
        <p class="muted small" data-i18n="view.db.hint.intro">
            For high earners (&gt; $300k). <strong>2024 § 415(b) annual benefit cap: $275,000</strong>
            at NRA. Annual contribution can EXCEED $250k for older participants. STACK with 401(k):
            $69k + $275k actuarially-determined annual contribution = $300k+ deductible/yr.
            <strong>Cash Balance:</strong> hybrid with notional account + interest credit rate.
            Requires <strong>actuary + Form 5500</strong>. Owner-employees 45+ benefit most.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.db.h2.inputs">Inputs</h2>
            <form id="db-form" class="inline-form">
                <label><span data-i18n="view.db.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.db.label.retire_age">Target retirement age</span>
                    <input type="number" step="1" name="target_retirement_age" value="${state.target_retirement_age}"></label>
                <label><span data-i18n="view.db.label.comp">Annual W-2 / net SE comp ($)</span>
                    <input type="number" step="1000" name="annual_compensation" value="${state.annual_compensation}"></label>
                <label><span data-i18n="view.db.label.has_401k">Have a separate Solo 401(k)?</span>
                    <input type="checkbox" name="has_other_401k" ${state.has_other_401k ? 'checked' : ''}></label>
                <label><span data-i18n="view.db.label.cash_balance">Cash Balance plan?</span>
                    <input type="checkbox" name="is_cash_balance" ${state.is_cash_balance ? 'checked' : ''}></label>
                <label><span data-i18n="view.db.label.icr">Interest Credit Rate (CB)</span>
                    <input type="number" step="0.001" name="interest_credit_rate" value="${state.interest_credit_rate}"></label>
                <label><span data-i18n="view.db.label.discount">Funding discount rate</span>
                    <input type="number" step="0.001" name="funding_discount_rate" value="${state.funding_discount_rate}"></label>
                <label><span data-i18n="view.db.label.employees">Eligible employees</span>
                    <input type="number" step="1" name="employee_count" value="${state.employee_count}"></label>
                <label><span data-i18n="view.db.label.marginal">Marginal rate</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.db.btn.compute">Compute</button>
            </form>
        </div>
        <div id="db-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.db.h2.compare">Defined Benefit vs Solo 401(k) by age</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.db.th.age">Age</th>
                    <th data-i18n="view.db.th.solo">Solo 401(k) max</th>
                    <th data-i18n="view.db.th.db_approx">DB approx max</th>
                    <th data-i18n="view.db.th.combo">Combined</th>
                </tr></thead>
                <tbody>
                    <tr><td>40</td><td>$69k</td><td>~$120k</td><td>~$189k</td></tr>
                    <tr><td>45</td><td>$69k</td><td>~$170k</td><td>~$239k</td></tr>
                    <tr><td>50</td><td>$76.5k</td><td>~$220k</td><td>~$296k</td></tr>
                    <tr><td>55</td><td>$76.5k</td><td>~$260k</td><td>~$336k</td></tr>
                    <tr><td>60</td><td>$80.25k</td><td>~$300k</td><td>~$380k</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.db.h2.cautions">Cautions</h2>
            <ul class="muted small">
                <li data-i18n="view.db.cau.actuary">Requires enrolled actuary + Form 5500-SF / 5500</li>
                <li data-i18n="view.db.cau.permanence">Must intend permanence (3-5+ yrs minimum, PBGC could investigate)</li>
                <li data-i18n="view.db.cau.employee_costs">Employees: must include eligible non-owner employees at coverage level</li>
                <li data-i18n="view.db.cau.testing">§ 401(a)(4) / § 410(b) non-discrimination tests apply</li>
                <li data-i18n="view.db.cau.funding">Minimum funding required even in down years</li>
                <li data-i18n="view.db.cau.overfunding">Reversion penalty 50% on overfunded amounts → use Qualified Replacement Plan</li>
                <li data-i18n="view.db.cau.pbgc">PBGC premium ~ $96/participant/yr (single-employer)</li>
                <li data-i18n="view.db.cau.deductibility">Up-front design fee $3-10k + annual admin $2-5k</li>
            </ul>
        </div>
    `;
    document.getElementById('db-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.age = Number(fd.get('age')) || 50;
        state.target_retirement_age = Number(fd.get('target_retirement_age')) || 62;
        state.annual_compensation = Number(fd.get('annual_compensation')) || 0;
        state.has_other_401k = !!fd.get('has_other_401k');
        state.is_cash_balance = !!fd.get('is_cash_balance');
        state.interest_credit_rate = Number(fd.get('interest_credit_rate')) || 0.05;
        state.funding_discount_rate = Number(fd.get('funding_discount_rate')) || 0.05;
        state.employee_count = Number(fd.get('employee_count')) || 1;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.37;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('db-output');
    if (!el) return;
    const yearsToRetire = Math.max(1, state.target_retirement_age - state.age);
    const cappedComp = Math.min(state.annual_compensation, COMP_CAP_2024);
    const targetBenefit = Math.min(cappedComp, ANNUAL_BENEFIT_CAP_2024);
    // PV of annuity at retirement
    const pvFactor = (1 - Math.pow(1 + state.funding_discount_rate, -20)) / state.funding_discount_rate;
    const pvAtRetirement = targetBenefit * pvFactor;
    // Annual contribution to fund
    const futureValueFactor = (Math.pow(1 + state.funding_discount_rate, yearsToRetire) - 1) / state.funding_discount_rate;
    const annualContribution = pvAtRetirement / futureValueFactor;
    const cappedContribution = Math.min(annualContribution, ANNUAL_BENEFIT_CAP_2024);
    const total401kStack = state.has_other_401k ? 69_000 : 0;
    const grandTotal = cappedContribution + total401kStack;
    const taxSavings = grandTotal * state.marginal_rate;
    const tenYrTaxSavings = taxSavings * Math.min(10, yearsToRetire);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.db.h2.result">Plan economics</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.db.card.years">Years to retirement</div>
                    <div class="value">${yearsToRetire}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.db.card.target_benefit">Target annual benefit</div>
                    <div class="value">$${targetBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.db.card.contribution">Annual DB contribution</div>
                    <div class="value">$${cappedContribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.has_other_401k ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.db.card.401k_stack">Combined 401(k) stack</div>
                        <div class="value">$${total401kStack.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card pos">
                    <div class="label" data-i18n="view.db.card.total">Grand total</div>
                    <div class="value">$${grandTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.db.card.savings">Year-1 tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.db.card.ten_year">10-yr cumulative savings</div>
                    <div class="value">$${tenYrTaxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
