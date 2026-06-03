// Mega Backdoor Roth Strategy.
// After-tax 401(k) contributions converted to Roth IRA via in-service rollover
// OR converted in-plan to Roth 401(k). 2024 total 415(c) limit: $69,000.
// $69k - $23k employee deferral - employer match = available after-tax space.
// Up to $46,000/yr extra Roth space ON TOP of regular limits.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LIMITS = {
    2024: { total_415c: 69_000, employee_deferral: 23_000, catchup_50: 7_500 },
    2025: { total_415c: 70_000, employee_deferral: 23_500, catchup_50: 7_500 },
    2026: { total_415c: 72_000, employee_deferral: 24_000, catchup_50: 7_500 },
};

let state = {
    year: new Date().getFullYear(),
    age: 35,
    salary: 200_000,
    employee_deferral: 23_000,
    employer_match: 6_000,
    plan_allows_after_tax: true,
    plan_allows_in_service_conv: true,
    expected_return: 0.07,
    years_to_retirement: 30,
};

export async function renderMegaBackdoorRoth(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mbr.h1.title">// MEGA BACKDOOR ROTH</span></h1>
        <p class="muted small" data-i18n="view.mbr.hint.intro">
            After-tax 401(k) contributions converted to Roth via in-service rollover OR
            in-plan Roth conversion. 2024 total 415(c) cap: $69,000. Subtract employee
            deferral + employer match = available after-tax space. <strong>Up to
            $46,000/yr extra Roth space</strong> on top of the regular $7k IRA limits.
            Requires your 401(k) plan document to allow after-tax + conversions.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.mbr.h2.inputs">Inputs</h2>
            <form id="mbr-form" class="inline-form">
                <label><span data-i18n="view.mbr.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.mbr.label.age">Age</span>
                    <input type="number" step="1" name="age" value="${state.age}" min="18" max="80"></label>
                <label><span data-i18n="view.mbr.label.salary">Salary ($)</span>
                    <input type="number" step="1000" name="salary" value="${state.salary}"></label>
                <label><span data-i18n="view.mbr.label.employee_deferral">Employee deferral ($)</span>
                    <input type="number" step="500" name="employee_deferral" value="${state.employee_deferral}"></label>
                <label><span data-i18n="view.mbr.label.employer_match">Employer match ($)</span>
                    <input type="number" step="500" name="employer_match" value="${state.employer_match}"></label>
                <label><span data-i18n="view.mbr.label.plan_allows_after_tax">Plan allows after-tax contributions?</span>
                    <input type="checkbox" name="plan_allows_after_tax" ${state.plan_allows_after_tax ? 'checked' : ''}></label>
                <label><span data-i18n="view.mbr.label.plan_allows_in_service_conv">Plan allows in-service Roth conversion?</span>
                    <input type="checkbox" name="plan_allows_in_service_conv" ${state.plan_allows_in_service_conv ? 'checked' : ''}></label>
                <label><span data-i18n="view.mbr.label.expected_return">Expected return %</span>
                    <input type="number" step="0.5" name="expected_return" value="${(state.expected_return * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.mbr.label.years_to_retirement">Years to retirement</span>
                    <input type="number" step="1" name="years_to_retirement" value="${state.years_to_retirement}"></label>
                <button class="primary" type="submit" data-i18n="view.mbr.btn.compute">Compute</button>
            </form>
        </div>
        <div id="mbr-output"></div>
    `;
    document.getElementById('mbr-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(fd.get('year'));
        state.age = Number(fd.get('age'));
        state.salary = Number(fd.get('salary')) || 0;
        state.employee_deferral = Number(fd.get('employee_deferral')) || 0;
        state.employer_match = Number(fd.get('employer_match')) || 0;
        state.plan_allows_after_tax = !!fd.get('plan_allows_after_tax');
        state.plan_allows_in_service_conv = !!fd.get('plan_allows_in_service_conv');
        state.expected_return = (Number(fd.get('expected_return')) || 7) / 100;
        state.years_to_retirement = Number(fd.get('years_to_retirement')) || 30;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('mbr-output');
    if (!el) return;
    const limits = LIMITS[state.year] || LIMITS[2024];
    const totalCap = limits.total_415c + (state.age >= 50 ? limits.catchup_50 : 0);
    const afterTaxRoom = Math.max(0, totalCap - state.employee_deferral - state.employer_match);
    const eligible = state.plan_allows_after_tax && state.plan_allows_in_service_conv;
    const actualMegaRoth = eligible ? afterTaxRoom : 0;
    // 30-year compound FV
    let fv = 0;
    for (let y = 0; y < state.years_to_retirement; y++) {
        fv = (fv + actualMegaRoth) * (1 + state.expected_return);
    }
    el.innerHTML = `
        <div class="chart-panel ${eligible ? 'pos' : 'neg'}">
            <h2 data-i18n="view.mbr.h2.summary">${state.year} mega backdoor space</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.mbr.card.after_tax_room">Available after-tax space</div>
                    <div class="value">$${actualMegaRoth.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.mbr.card.total_cap">Total 415(c) cap</div>
                    <div class="value">$${totalCap.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.mbr.card.used_by_others">Used by deferral + match</div>
                    <div class="value">$${(state.employee_deferral + state.employer_match).toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.mbr.card.fv">FV at retirement</div>
                    <div class="value">$${fv.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!eligible ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.mbr.not_eligible">
                    Your plan must allow BOTH after-tax contributions AND in-service Roth
                    conversions for the mega backdoor to work. Check your Summary Plan
                    Description (SPD) or ask HR. About 50% of large-company plans now
                    permit this since 2018-ish.
                </p>
            ` : ''}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mbr.h2.mechanics">Mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.mbr.step.contribute">Contribute up to $${afterTaxRoom.toLocaleString()} as AFTER-TAX (not Roth) to 401(k)</li>
                <li data-i18n="view.mbr.step.convert">Within days/weeks: convert to Roth 401(k) or rollover to Roth IRA</li>
                <li data-i18n="view.mbr.step.minimize_growth">Convert ASAP to minimize taxable growth between contribution and conversion</li>
                <li data-i18n="view.mbr.step.no_tax">After-tax dollars: no tax owed on conversion (basis = contribution)</li>
                <li data-i18n="view.mbr.step.growth_taxed">Any growth between contribution & conversion = taxable ordinary on conversion</li>
                <li data-i18n="view.mbr.step.no_income_cap">NO income phase-out (unlike regular Roth IRA contribution)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.mbr.h2.target_plans">Plans known to allow this</h2>
            <p class="muted small" data-i18n="view.mbr.target_plans_body">
                Microsoft, Google, Meta, Amazon, NVIDIA, Apple, Tesla, Salesforce — most
                tech FAANG+ allow this. Federal employees: TSP does NOT allow it. Most
                small employer 401(k)s don't allow it. Check your SPD or call provider.
            </p>
        </div>
    `;
}
