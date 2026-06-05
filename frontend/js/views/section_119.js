// IRC § 119 — Meals + Lodging Furnished for Convenience of Employer.
// Excluded from employee income if furnished:
// (1) on business premises + (2) for convenience of employer + (3) lodging requires "as condition of employment".
// TCJA 50% disallowance of employer DEDUCTION but employee exclusion preserved.
// Common: hotels for hotel employees, cafeteria meals for hospital workers, on-site housing for camp managers.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    benefit_type: 'meals',
    annual_value: 0,
    on_business_premises: true,
    employer_convenience: true,
    lodging_condition_of_employment: false,
    employer_has_substantial_non_compensatory_reason: true,
    employee_marginal_rate: 0.32,
    fica_rate: 0.0765,
    state_marginal_rate: 0.06,
    employer_marginal_rate: 0.21,
};

export async function renderSection119(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s119.h1.title">// § 119 MEALS + LODGING FOR EMPLOYER CONVENIENCE</span></h1>
        <p class="muted small" data-i18n="view.s119.hint.intro">
            Excluded from employee income if furnished: <strong>(1) on business premises</strong>
            + <strong>(2) for convenience of employer</strong> + <strong>(3) lodging requires
            "as condition of employment"</strong>. TCJA: <strong>50% disallowance of employer
            DEDUCTION</strong> 2018-2025; employee exclusion preserved. Common: hotels, hospital
            cafeteria, on-site housing for camp managers, lighthouse keepers.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s119.h2.inputs">Inputs</h2>
            <form id="s119-form" class="inline-form">
                <label><span data-i18n="view.s119.label.benefit_type">Benefit type</span>
                    <select name="benefit_type">
                        <option value="meals" ${state.benefit_type === 'meals' ? 'selected' : ''}>Meals</option>
                        <option value="lodging" ${state.benefit_type === 'lodging' ? 'selected' : ''}>Lodging</option>
                        <option value="both" ${state.benefit_type === 'both' ? 'selected' : ''}>Both</option>
                    </select>
                </label>
                <label><span data-i18n="view.s119.label.value">Annual value ($)</span>
                    <input type="number" step="0.01" name="annual_value" value="${state.annual_value}"></label>
                <label><span data-i18n="view.s119.label.premises">On business premises?</span>
                    <input type="checkbox" name="on_business_premises" ${state.on_business_premises ? 'checked' : ''}></label>
                <label><span data-i18n="view.s119.label.convenience">For convenience of employer?</span>
                    <input type="checkbox" name="employer_convenience" ${state.employer_convenience ? 'checked' : ''}></label>
                <label><span data-i18n="view.s119.label.condition">Lodging: condition of employment?</span>
                    <input type="checkbox" name="lodging_condition_of_employment" ${state.lodging_condition_of_employment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s119.label.reason">Substantial non-compensatory reason?</span>
                    <input type="checkbox" name="employer_has_substantial_non_compensatory_reason" ${state.employer_has_substantial_non_compensatory_reason ? 'checked' : ''}></label>
                <label><span data-i18n="view.s119.label.emp_rate">Employee marginal rate</span>
                    <input type="number" step="0.01" name="employee_marginal_rate" value="${state.employee_marginal_rate}"></label>
                <label><span data-i18n="view.s119.label.fica">FICA rate (employee + employer)</span>
                    <input type="number" step="0.0001" name="fica_rate" value="${state.fica_rate}"></label>
                <label><span data-i18n="view.s119.label.state">State rate</span>
                    <input type="number" step="0.01" name="state_marginal_rate" value="${state.state_marginal_rate}"></label>
                <label><span data-i18n="view.s119.label.employer_rate">Employer marginal rate</span>
                    <input type="number" step="0.01" name="employer_marginal_rate" value="${state.employer_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s119.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s119-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s119.h2.qualifying_meals">Qualifying meals examples</h2>
            <ul class="muted small">
                <li data-i18n="view.s119.meals.hospital">Hospital cafeteria food for medical staff (Boyd Gaming Corp)</li>
                <li data-i18n="view.s119.meals.short_lunch">Short lunch period requires staying on premises</li>
                <li data-i18n="view.s119.meals.emergency">Available for emergency calls during meal period</li>
                <li data-i18n="view.s119.meals.no_eating">No eating facilities reasonably nearby</li>
                <li data-i18n="view.s119.meals.full_field_staff">Meals furnished for substantial non-compensatory reason</li>
                <li data-i18n="view.s119.meals.de_minimis">Occasional meals = § 132(e) de minimis (separate)</li>
                <li data-i18n="view.s119.meals.taxable_to_employer">Pre-2026: 50% deduction limit for employer (down from 100%)</li>
                <li data-i18n="view.s119.meals.post_2026">2026: 0% employer deduction (TCJA sunset)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s119.h2.qualifying_lodging">Qualifying lodging examples</h2>
            <ul class="muted small">
                <li data-i18n="view.s119.lodging.condition">Employee must accept as a condition of employment</li>
                <li data-i18n="view.s119.lodging.duty">Required to be on call 24/7 at facility</li>
                <li data-i18n="view.s119.lodging.remote">Remote location with no reasonable alternative housing</li>
                <li data-i18n="view.s119.lodging.security">Security / on-site management (apartment manager, lighthouse, camp)</li>
                <li data-i18n="view.s119.lodging.no_cash_option">No cash-in-lieu option permitted</li>
                <li data-i18n="view.s119.lodging.benefit_employer">Significant employer convenience benefit</li>
                <li data-i18n="view.s119.lodging.travel_no">NOT temporary travel lodging (use § 162(a)(2) instead)</li>
                <li data-i18n="view.s119.lodging.company_subsidized">Subsidized housing without true requirement = taxable</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s119.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s119.rel.119a">§ 119(a): meals + lodging exclusion</li>
                <li data-i18n="view.s119.rel.119b">§ 119(b)(3): faculty housing exception ≥ 5% FMV cost or rent</li>
                <li data-i18n="view.s119.rel.119c">§ 119(c): employer's policy + facts/circumstances test</li>
                <li data-i18n="view.s119.rel.107">§ 107 Minister's housing allowance (separate provision)</li>
                <li data-i18n="view.s119.rel.911">§ 911 Foreign housing exclusion for expats</li>
                <li data-i18n="view.s119.rel.132e">§ 132(e) De minimis: occasional supper money + coffee</li>
                <li data-i18n="view.s119.rel.162a2">§ 162(a)(2) Travel away from home (transient)</li>
                <li data-i18n="view.s119.rel.7872">§ 7872 Below-market loan rules for housing assistance</li>
            </ul>
        </div>
    `;
    document.getElementById('s119-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.benefit_type = fd.get('benefit_type');
        state.annual_value = Number(fd.get('annual_value')) || 0;
        state.on_business_premises = !!fd.get('on_business_premises');
        state.employer_convenience = !!fd.get('employer_convenience');
        state.lodging_condition_of_employment = !!fd.get('lodging_condition_of_employment');
        state.employer_has_substantial_non_compensatory_reason = !!fd.get('employer_has_substantial_non_compensatory_reason');
        state.employee_marginal_rate = Number(fd.get('employee_marginal_rate')) || 0.32;
        state.fica_rate = Number(fd.get('fica_rate')) || 0.0765;
        state.state_marginal_rate = Number(fd.get('state_marginal_rate')) || 0.06;
        state.employer_marginal_rate = Number(fd.get('employer_marginal_rate')) || 0.21;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s119-output');
    if (!el) return;
    const mealsQualify = state.on_business_premises && state.employer_convenience && state.employer_has_substantial_non_compensatory_reason;
    const lodgingQualifies = state.on_business_premises && state.employer_convenience && state.lodging_condition_of_employment;
    let qualifies = false;
    if (state.benefit_type === 'meals') qualifies = mealsQualify;
    else if (state.benefit_type === 'lodging') qualifies = lodgingQualifies;
    else qualifies = mealsQualify && lodgingQualifies;
    const employeeSavings = qualifies
        ? state.annual_value * (state.employee_marginal_rate + state.state_marginal_rate + state.fica_rate)
        : 0;
    const employerDeductionRate = state.benefit_type === 'meals' || state.benefit_type === 'both' ? 0.50 : 1.00;
    const employerDeductionValue = qualifies ? state.annual_value * employerDeductionRate * state.employer_marginal_rate : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s119.h2.result">Exclusion analysis</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s119.card.qualifies">Qualifies under § 119?</div>
                    <div class="value">${qualifies ? esc(t('view.s119.status.yes')) : esc(t('view.s119.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s119.card.value">Annual value excluded</div>
                    <div class="value">$${(qualifies ? state.annual_value : 0).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s119.card.employee_savings">Employee tax savings</div>
                    <div class="value">$${employeeSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s119.card.employer_deduction">Employer deduction value</div>
                    <div class="value">$${employerDeductionValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
