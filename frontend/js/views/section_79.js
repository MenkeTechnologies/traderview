// IRC § 79 — Group Term Life Insurance > $50,000.
// First $50K of employer-provided group term life is TAX-FREE.
// Coverage > $50K: IMPUTED INCOME based on IRS Uniform Premium Table (Table I, Notice 2021-65).
// Imputed cost = (Coverage − $50K) / 1,000 × IRS rate per $1K × months covered.
// Reported on Form W-2 Box 12 code C.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    total_coverage: 0,
    employee_age: 0,
    months_covered: 12,
    employee_contribution_after_tax: 0,
    is_key_employee: false,
    is_self_insured: false,
    has_carve_out: false,
    plan_discriminatory: false,
    permanent_benefits: 0,
    spousal_coverage: 0,
    dependent_coverage: 0,
};

const UNIFORM_TABLE_I = {
    under_25: 0.05,
    a25_29: 0.06,
    a30_34: 0.08,
    a35_39: 0.09,
    a40_44: 0.10,
    a45_49: 0.15,
    a50_54: 0.23,
    a55_59: 0.43,
    a60_64: 0.66,
    a65_69: 1.27,
    a70_plus: 2.06,
};

export async function renderSection79(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s79.h1.title">// § 79 GROUP TERM LIFE</span></h1>
        <p class="muted small" data-i18n="view.s79.hint.intro">
            First <strong>$50,000</strong> of employer-provided group term life coverage = <strong>TAX-FREE</strong>.
            Coverage <strong>over $50K:</strong> IMPUTED INCOME based on IRS Uniform Premium Table I
            (Notice 2021-65). <strong>Formula:</strong> (Coverage − $50K) / 1,000 × IRS rate per $1K × months
            covered. <strong>Reported W-2 Box 12 code C.</strong> Subject to <strong>Soc Sec + Medicare</strong>
            (but NOT income tax withholding). <strong>Discrimination rules:</strong> if plan favors key
            employees, full coverage taxable to them (no $50K exclusion).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s79.h2.inputs">Inputs</h2>
            <form id="s79-form" class="inline-form">
                <label><span data-i18n="view.s79.label.coverage">Total coverage ($)</span>
                    <input type="number" step="0.01" name="total_coverage" value="${state.total_coverage}"></label>
                <label><span data-i18n="view.s79.label.age">Employee age</span>
                    <input type="number" step="1" name="employee_age" value="${state.employee_age}"></label>
                <label><span data-i18n="view.s79.label.months">Months covered (yr)</span>
                    <input type="number" step="1" name="months_covered" value="${state.months_covered}"></label>
                <label><span data-i18n="view.s79.label.contribution">After-tax employee contribution ($)</span>
                    <input type="number" step="1" name="employee_contribution_after_tax" value="${state.employee_contribution_after_tax}"></label>
                <label><span data-i18n="view.s79.label.key">Key employee?</span>
                    <input type="checkbox" name="is_key_employee" ${state.is_key_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s79.label.self">Self-insured plan?</span>
                    <input type="checkbox" name="is_self_insured" ${state.is_self_insured ? 'checked' : ''}></label>
                <label><span data-i18n="view.s79.label.carve">Has key-employee carve-out?</span>
                    <input type="checkbox" name="has_carve_out" ${state.has_carve_out ? 'checked' : ''}></label>
                <label><span data-i18n="view.s79.label.discrim">Discriminatory plan?</span>
                    <input type="checkbox" name="plan_discriminatory" ${state.plan_discriminatory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s79.label.permanent">Permanent benefits (non-term) ($)</span>
                    <input type="number" step="0.01" name="permanent_benefits" value="${state.permanent_benefits}"></label>
                <label><span data-i18n="view.s79.label.spouse">Spousal coverage ($)</span>
                    <input type="number" step="0.01" name="spousal_coverage" value="${state.spousal_coverage}"></label>
                <label><span data-i18n="view.s79.label.dependent">Dependent coverage ($)</span>
                    <input type="number" step="0.01" name="dependent_coverage" value="${state.dependent_coverage}"></label>
                <button class="primary" type="submit" data-i18n="view.s79.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s79-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s79.h2.uniform">Table I — Uniform Premiums per $1K per month (Notice 2021-65)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s79.th.age">Age</th>
                    <th data-i18n="view.s79.th.rate">Rate per $1K/month</th>
                </tr></thead>
                <tbody>
                    <tr><td>Under 25</td><td>$0.05</td></tr>
                    <tr><td>25-29</td><td>$0.06</td></tr>
                    <tr><td>30-34</td><td>$0.08</td></tr>
                    <tr><td>35-39</td><td>$0.09</td></tr>
                    <tr><td>40-44</td><td>$0.10</td></tr>
                    <tr><td>45-49</td><td>$0.15</td></tr>
                    <tr><td>50-54</td><td>$0.23</td></tr>
                    <tr><td>55-59</td><td>$0.43</td></tr>
                    <tr><td>60-64</td><td>$0.66</td></tr>
                    <tr><td>65-69</td><td>$1.27</td></tr>
                    <tr><td>70+</td><td>$2.06</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s79.h2.discrimination">Discrimination rules (§ 79(d))</h2>
            <ul class="muted small">
                <li data-i18n="view.s79.disc.test_e">Eligibility test: cannot favor key employees in coverage</li>
                <li data-i18n="view.s79.disc.test_b">Benefits test: cannot favor key employees in benefits</li>
                <li data-i18n="view.s79.disc.key_def">Key employee: 5% owner, 1% owner + $150K+, officer + $215K (2025)</li>
                <li data-i18n="view.s79.disc.consequence">Discriminatory plan → KEY EMPLOYEES taxable on FULL coverage cost (no $50K excl)</li>
                <li data-i18n="view.s79.disc.actual_cost">Actual cost = GREATER of actual premium OR Table I uniform cost</li>
                <li data-i18n="view.s79.disc.regular_employees">Non-key employees still get $50K exclusion</li>
                <li data-i18n="view.s79.disc.cure">Cure: equalize coverage / benefits or carve out keys to separate plan</li>
                <li data-i18n="view.s79.disc.s125_cafeteria">§ 125 cafeteria: different discrimination rules apply (parallel)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s79.h2.special">Special rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s79.spec.spousal">Spousal coverage ≤ $2,000: de minimis — no income</li>
                <li data-i18n="view.s79.spec.dependent">Dependent coverage ≤ $2,000: de minimis — no income</li>
                <li data-i18n="view.s79.spec.permanent">Permanent benefits (cash value): full value taxable in year of acquisition</li>
                <li data-i18n="view.s79.spec.retired">Retired employees: continued coverage may still be § 79 subject</li>
                <li data-i18n="view.s79.spec.disabled">Disabled employees: tax-free up to limit</li>
                <li data-i18n="view.s79.spec.beneficiary">Beneficiary received death benefit: tax-free under § 101(a)</li>
                <li data-i18n="view.s79.spec.carry_over">Coverage continuation: months past employment count</li>
                <li data-i18n="view.s79.spec.cafeteria">§ 125 election to pay premiums pre-tax: reduces imputed income</li>
            </ul>
        </div>
    `;
    document.getElementById('s79-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_coverage = Number(fd.get('total_coverage')) || 0;
        state.employee_age = Number(fd.get('employee_age')) || 0;
        state.months_covered = Number(fd.get('months_covered')) || 0;
        state.employee_contribution_after_tax = Number(fd.get('employee_contribution_after_tax')) || 0;
        state.is_key_employee = !!fd.get('is_key_employee');
        state.is_self_insured = !!fd.get('is_self_insured');
        state.has_carve_out = !!fd.get('has_carve_out');
        state.plan_discriminatory = !!fd.get('plan_discriminatory');
        state.permanent_benefits = Number(fd.get('permanent_benefits')) || 0;
        state.spousal_coverage = Number(fd.get('spousal_coverage')) || 0;
        state.dependent_coverage = Number(fd.get('dependent_coverage')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s79-output');
    if (!el) return;
    const age = state.employee_age;
    let rate = UNIFORM_TABLE_I.a70_plus;
    if (age < 25) rate = UNIFORM_TABLE_I.under_25;
    else if (age < 30) rate = UNIFORM_TABLE_I.a25_29;
    else if (age < 35) rate = UNIFORM_TABLE_I.a30_34;
    else if (age < 40) rate = UNIFORM_TABLE_I.a35_39;
    else if (age < 45) rate = UNIFORM_TABLE_I.a40_44;
    else if (age < 50) rate = UNIFORM_TABLE_I.a45_49;
    else if (age < 55) rate = UNIFORM_TABLE_I.a50_54;
    else if (age < 60) rate = UNIFORM_TABLE_I.a55_59;
    else if (age < 65) rate = UNIFORM_TABLE_I.a60_64;
    else if (age < 70) rate = UNIFORM_TABLE_I.a65_69;
    const excludeBase = (state.is_key_employee && state.plan_discriminatory) ? 0 : 50_000;
    const excessCoverage = Math.max(0, state.total_coverage - excludeBase);
    const monthlyImputed = (excessCoverage / 1000) * rate;
    const annualImputed = monthlyImputed * state.months_covered;
    const netImputed = Math.max(0, annualImputed - state.employee_contribution_after_tax);
    const fica = netImputed * 0.0765;
    const incomeTax = netImputed * 0.24;
    const spousalImputed = state.spousal_coverage > 2000 ? (state.spousal_coverage / 1000 * rate * state.months_covered) : 0;
    const dependentImputed = state.dependent_coverage > 2000 ? (state.dependent_coverage / 1000 * rate * state.months_covered) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s79.h2.result">§ 79 imputed income</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s79.card.exclude">Tax-free exclusion</div>
                    <div class="value">$${excludeBase.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s79.card.excess">Excess coverage</div>
                    <div class="value">$${excessCoverage.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s79.card.rate">Rate/$1K/mo</div>
                    <div class="value">$${rate.toFixed(2)}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s79.card.imputed">Annual imputed (W-2 C)</div>
                    <div class="value">$${netImputed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s79.card.fica">FICA on imputed (7.65%)</div>
                    <div class="value">$${fica.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s79.card.income">Income tax (24%)</div>
                    <div class="value">$${incomeTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s79.card.spousal">Spousal imputed</div>
                    <div class="value">$${spousalImputed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s79.card.dependent">Dependent imputed</div>
                    <div class="value">$${dependentImputed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_key_employee && state.plan_discriminatory ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s79.disc_note">
                    Discriminatory plan + key employee: lose $50K exclusion → FULL coverage cost taxable
                    (GREATER of actual premium or Table I uniform cost). Cure: carve out keys to separate plan
                    OR equalize coverage / benefits across all employees.
                </p>
            ` : ''}
        </div>
    `;
}
