// SIMPLE IRA — Savings Incentive Match Plan for Employees.
// Sub-100-employee plans. 2024 employee deferral $16,000 ($19,500 with catch-up 50+).
// Employer: 3% match OR 2% non-elective for ALL eligible. SECURE 2.0 added 10% increase
// for some employers. 25% early-withdrawal penalty (within 2 yrs of first contribution).
// Must establish by Oct 1.

import { currentViewToken, viewIsCurrent } from '../app.js';

const EMPLOYEE_LIMIT_2024 = 16_000;
const CATCH_UP_50 = 3_500;
const CATCH_UP_60 = 5_250;  // SECURE 2.0 super-catch-up
const MATCH_PCT = 0.03;
const NON_ELECTIVE_PCT = 0.02;
const EARLY_WITHDRAWAL_PENALTY = 0.25;
const EMPLOYEE_COUNT_CAP = 100;

let state = {
    age: 40,
    employee_count: 1,
    annual_compensation: 0,
    employee_deferral: 0,
    employer_choice: 'match',
    establish_year_first: false,
    marginal_rate: 0.32,
};

export async function renderSimpleIra(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.simple.h1.title">// SIMPLE IRA CONTRIBUTION CALC</span></h1>
        <p class="muted small" data-i18n="view.simple.hint.intro">
            For employers with ≤ 100 employees. <strong>2024 employee deferral:
            $16,000</strong> ($19,500 with 50+ catch-up; $21,250 SECURE 2.0 60-63 super-catch-up).
            Employer chooses: <strong>3% match</strong> OR <strong>2% non-elective for ALL eligible</strong>.
            <strong>25% early-withdrawal penalty</strong> (within 2 yrs of first contribution; 10% after).
            Must establish by Oct 1. SECURE 2.0 added Roth option + 10% employer enhancement.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.simple.h2.inputs">Inputs</h2>
            <form id="simple-form" class="inline-form">
                <label><span data-i18n="view.simple.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.simple.label.employees">Total eligible employees</span>
                    <input type="number" step="1" name="employee_count" value="${state.employee_count}"></label>
                <label><span data-i18n="view.simple.label.comp">Your compensation ($)</span>
                    <input type="number" step="0.01" name="annual_compensation" value="${state.annual_compensation}"></label>
                <label><span data-i18n="view.simple.label.deferral">Your deferral ($)</span>
                    <input type="number" step="0.01" name="employee_deferral" value="${state.employee_deferral}"></label>
                <label><span data-i18n="view.simple.label.choice">Employer formula</span>
                    <select name="employer_choice">
                        <option value="match" ${state.employer_choice === 'match' ? 'selected' : ''}>3% match</option>
                        <option value="non_elective" ${state.employer_choice === 'non_elective' ? 'selected' : ''}>2% non-elective</option>
                    </select>
                </label>
                <label><span data-i18n="view.simple.label.first_year">Within 2 yrs of first contribution?</span>
                    <input type="checkbox" name="establish_year_first" ${state.establish_year_first ? 'checked' : ''}></label>
                <label><span data-i18n="view.simple.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.simple.btn.compute">Compute</button>
            </form>
        </div>
        <div id="simple-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.simple.h2.choice">3% match vs 2% non-elective</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.simple.th.feature">Feature</th>
                    <th data-i18n="view.simple.th.match">3% match</th>
                    <th data-i18n="view.simple.th.nonelect">2% non-elective</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.simple.row.who_gets">Who gets it</td><td>Only those who defer</td><td>EVERY eligible employee</td></tr>
                    <tr><td data-i18n="view.simple.row.cap">Salary cap</td><td>Comp × 3%</td><td>$345,000 × 2% = $6,900</td></tr>
                    <tr><td data-i18n="view.simple.row.reduce">Can reduce?</td><td>Down to 1% in 2 of 5 yrs</td><td>NO — fixed 2%</td></tr>
                    <tr><td data-i18n="view.simple.row.best_when">Best when</td><td>Owner alone / high defer</td><td>Many low-defer employees</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('simple-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.age = Number(fd.get('age')) || 40;
        state.employee_count = Number(fd.get('employee_count')) || 1;
        state.annual_compensation = Number(fd.get('annual_compensation')) || 0;
        state.employee_deferral = Number(fd.get('employee_deferral')) || 0;
        state.employer_choice = fd.get('employer_choice');
        state.establish_year_first = !!fd.get('establish_year_first');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('simple-output');
    if (!el) return;
    if (state.employee_count > EMPLOYEE_COUNT_CAP) {
        el.innerHTML = `
            <div class="chart-panel">
                <p class="muted small neg" data-i18n="view.simple.warning.too_big">
                    More than 100 employees — SIMPLE IRA disallowed. Consider Safe Harbor 401(k) instead.
                </p>
            </div>
        `;
        return;
    }
    const catchUp = state.age >= 60 ? CATCH_UP_60 : (state.age >= 50 ? CATCH_UP_50 : 0);
    const employeeCap = EMPLOYEE_LIMIT_2024 + catchUp;
    const employeeContrib = Math.min(state.employee_deferral, employeeCap);
    const employerMatch = state.employer_choice === 'match'
        ? Math.min(employeeContrib, state.annual_compensation * MATCH_PCT)
        : state.annual_compensation * NON_ELECTIVE_PCT;
    const total = employeeContrib + employerMatch;
    const taxSavings = total * state.marginal_rate;
    const penaltyRate = state.establish_year_first ? EARLY_WITHDRAWAL_PENALTY : 0.10;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.simple.h2.result">Contribution maximum</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.simple.card.deferral">Employee deferral</div>
                    <div class="value">$${employeeContrib.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.simple.card.employer">Employer contribution</div>
                    <div class="value">$${employerMatch.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.simple.card.total">Total</div>
                    <div class="value">$${total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.simple.card.savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${state.establish_year_first ? 'neg' : ''}">
                    <div class="label" data-i18n="view.simple.card.penalty">Early-withdrawal penalty</div>
                    <div class="value">${(penaltyRate * 100).toFixed(0)}%</div>
                </div>
            </div>
            ${state.establish_year_first ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.simple.warning.two_year">
                    Within 2 years of first contribution: 25% early-withdrawal penalty (vs normal 10%)
                    AND distributions cannot be rolled to non-SIMPLE IRA. Wait 2 years before rollover.
                </p>
            ` : ''}
        </div>
    `;
}
