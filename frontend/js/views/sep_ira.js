// SEP IRA — Simplified Employee Pension.
// Employer-only contribution: 25% W-2 wages (effective ~20% SE earnings).
// 2024 cap $69,000. No employee deferral, no catch-up, no Roth (until SECURE 2.0).
// Easiest plan; setup AND fund by tax return due date (incl. extensions).
// Must cover ALL eligible employees (3 of last 5 yrs, age 21+, $750+ wages).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LIMIT_2024 = 69_000;
const W2_PCT = 0.25;
const SE_PCT = 0.20;
const COMP_CAP_2024 = 345_000;  // § 401(a)(17) compensation limit

let state = {
    business_type: 'sole_prop',
    net_se_earnings: 0,
    w2_wages: 0,
    employee_count: 0,
    average_employee_wages: 0,
    elect_roth: false,
    marginal_rate: 0.32,
};

export async function renderSepIra(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sep.h1.title">// SEP IRA CONTRIBUTION CALC</span></h1>
        <p class="muted small" data-i18n="view.sep.hint.intro">
            <strong>2024 cap:</strong> $69,000. Employer contributes up to <strong>25% of W-2</strong>
            comp or ~<strong>20% of net SE</strong> earnings (after half-SE-tax). NO employee
            deferral. NO catch-up. Comp cap $345,000. Setup AND fund by tax return due date
            (incl. extensions). Easiest plan to run — single form 5305-SEP.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.sep.h2.inputs">Inputs</h2>
            <form id="sep-form" class="inline-form">
                <label><span data-i18n="view.sep.label.business_type">Business type</span>
                    <select name="business_type">
                        <option value="sole_prop" ${state.business_type === 'sole_prop' ? 'selected' : ''}>Sole prop / SMLLC</option>
                        <option value="s_corp" ${state.business_type === 's_corp' ? 'selected' : ''}>S-corp (W-2)</option>
                        <option value="c_corp" ${state.business_type === 'c_corp' ? 'selected' : ''}>C-corp (W-2)</option>
                        <option value="partnership" ${state.business_type === 'partnership' ? 'selected' : ''}>Partnership</option>
                    </select>
                </label>
                <label><span data-i18n="view.sep.label.net_se">Net SE earnings ($)</span>
                    <input type="number" step="1000" name="net_se_earnings" value="${state.net_se_earnings}"></label>
                <label><span data-i18n="view.sep.label.w2_wages">W-2 wages ($)</span>
                    <input type="number" step="1000" name="w2_wages" value="${state.w2_wages}"></label>
                <label><span data-i18n="view.sep.label.employee_count">Other eligible employees</span>
                    <input type="number" step="1" name="employee_count" value="${state.employee_count}"></label>
                <label><span data-i18n="view.sep.label.avg_emp_wages">Avg employee W-2 ($)</span>
                    <input type="number" step="1000" name="average_employee_wages" value="${state.average_employee_wages}"></label>
                <label><span data-i18n="view.sep.label.roth">Roth SEP (SECURE 2.0)?</span>
                    <input type="checkbox" name="elect_roth" ${state.elect_roth ? 'checked' : ''}></label>
                <label><span data-i18n="view.sep.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.sep.btn.compute">Compute</button>
            </form>
        </div>
        <div id="sep-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.sep.h2.coverage">Mandatory coverage</h2>
            <ul class="muted small">
                <li data-i18n="view.sep.cov.three_of_five">Must cover any employee who worked in 3 of last 5 yrs</li>
                <li data-i18n="view.sep.cov.age_21">Must be age 21+</li>
                <li data-i18n="view.sep.cov.wages_750">Must have earned ≥ $750 (2024) in current year</li>
                <li data-i18n="view.sep.cov.uniform">Contribution % must be UNIFORM across all eligible employees</li>
                <li data-i18n="view.sep.cov.no_skip">Cannot exclude long-term employees; can't favor highly compensated</li>
                <li data-i18n="view.sep.cov.same_for_owner">Owner gets SAME percentage rate as employees</li>
            </ul>
        </div>
    `;
    document.getElementById('sep-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.business_type = fd.get('business_type');
        state.net_se_earnings = Number(fd.get('net_se_earnings')) || 0;
        state.w2_wages = Number(fd.get('w2_wages')) || 0;
        state.employee_count = Number(fd.get('employee_count')) || 0;
        state.average_employee_wages = Number(fd.get('average_employee_wages')) || 0;
        state.elect_roth = !!fd.get('elect_roth');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('sep-output');
    if (!el) return;
    const isSE = state.business_type === 'sole_prop' || state.business_type === 'partnership';
    const comp = isSE
        ? Math.min(state.net_se_earnings - state.net_se_earnings * 0.07065, COMP_CAP_2024)
        : Math.min(state.w2_wages, COMP_CAP_2024);
    const rate = isSE ? SE_PCT : W2_PCT;
    const yourContribution = Math.min(comp * rate, LIMIT_2024);
    const ownerSavings = state.elect_roth ? 0 : yourContribution * state.marginal_rate;
    const employeeContributions = state.employee_count > 0
        ? state.average_employee_wages * rate * state.employee_count
        : 0;
    const totalOutflow = yourContribution + employeeContributions;
    const netCost = totalOutflow - ownerSavings;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.sep.h2.result">Contribution maximum</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.sep.card.your_contribution">Your contribution</div>
                    <div class="value">$${yourContribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sep.card.eff_rate">Effective rate</div>
                    <div class="value">${(rate * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sep.card.cap">2024 cap</div>
                    <div class="value">$${LIMIT_2024.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.sep.card.your_savings">Your tax savings</div>
                    <div class="value">$${ownerSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${employeeContributions > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.sep.card.emp_contributions">Required employee contributions</div>
                        <div class="value">$${employeeContributions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card ${netCost > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.sep.card.net_cost">Net cost to you</div>
                    <div class="value">$${netCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.employee_count > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.sep.warning.employees">
                    With eligible employees, SEP-IRA becomes EXPENSIVE — same % must go to them.
                    Consider Solo 401(k) (no employees) or Safe Harbor 401(k) (lower employer cost
                    via employee deferral structure).
                </p>
            ` : ''}
        </div>
    `;
}
