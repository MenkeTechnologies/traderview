// Solo 401(k) — the trader/sole-prop retirement vehicle.
// 2024 limits: employee deferral $23,000 ($30,500 if ≥ 50), employer profit-share
// up to 25% comp (W-2) or 20% self-employed net (after deduction-for-half-SE-tax),
// combined cap $69,000 ($76,500 catch-up). Roth deferrals + Mega Backdoor.
// CRITICAL: must establish by Dec 31; can FUND through return due date.

import { currentViewToken, viewIsCurrent } from '../app.js';

const LIMITS_2024 = {
    employee_deferral: 23_000,
    catch_up_50: 7_500,
    catch_up_60: 11_250,  // SECURE 2.0 enhanced 60-63 catch-up
    combined_cap: 69_000,
    combined_cap_50: 76_500,
    employer_cap_pct: 0.25,  // for W-2 income; effective 20% for SE
    se_employer_cap_pct: 0.20,
};

let state = {
    business_type: 'sole_prop',
    age: 40,
    net_se_earnings: 0,
    w2_wages: 0,
    employee_deferral: 0,
    elect_roth_deferral: false,
    elect_mega_backdoor: false,
    marginal_rate: 0.32,
};

export async function renderSolo401k(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.solo.h1.title">// SOLO 401(k) CONTRIBUTION CALC</span></h1>
        <p class="muted small" data-i18n="view.solo.hint.intro">
            <strong>2024 limits:</strong> employee deferral $23,000 ($30,500 with 50+ catch-up,
            $34,250 with SECURE 2.0 60-63 super-catch-up), employer profit-share up to 25% W-2 comp
            or 20% SE net, combined cap $69,000 ($76,500 / $80,250 with catch-ups). MUST
            ESTABLISH BY DEC 31; can FUND through tax return due date (incl. extensions).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.solo.h2.inputs">Inputs</h2>
            <form id="solo-form" class="inline-form">
                <label><span data-i18n="view.solo.label.business_type">Business type</span>
                    <select name="business_type">
                        <option value="sole_prop" ${state.business_type === 'sole_prop' ? 'selected' : ''}>Sole prop / single-member LLC</option>
                        <option value="s_corp" ${state.business_type === 's_corp' ? 'selected' : ''}>S-corp (W-2)</option>
                        <option value="c_corp" ${state.business_type === 'c_corp' ? 'selected' : ''}>C-corp (W-2)</option>
                        <option value="partnership" ${state.business_type === 'partnership' ? 'selected' : ''}>Partnership / LLC-multi</option>
                    </select>
                </label>
                <label><span data-i18n="view.solo.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.solo.label.net_se">Net SE earnings (Schedule C) ($)</span>
                    <input type="number" step="1000" name="net_se_earnings" value="${state.net_se_earnings}"></label>
                <label><span data-i18n="view.solo.label.w2_wages">W-2 wages (S/C-corp) ($)</span>
                    <input type="number" step="1000" name="w2_wages" value="${state.w2_wages}"></label>
                <label><span data-i18n="view.solo.label.elect_roth">Roth deferral?</span>
                    <input type="checkbox" name="elect_roth_deferral" ${state.elect_roth_deferral ? 'checked' : ''}></label>
                <label><span data-i18n="view.solo.label.mega_backdoor">Mega backdoor (after-tax)?</span>
                    <input type="checkbox" name="elect_mega_backdoor" ${state.elect_mega_backdoor ? 'checked' : ''}></label>
                <label><span data-i18n="view.solo.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.solo.btn.compute">Compute</button>
            </form>
        </div>
        <div id="solo-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.solo.h2.tradeoffs">Solo 401(k) vs SEP IRA vs SIMPLE IRA</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.solo.th.feature">Feature</th>
                    <th>Solo 401(k)</th>
                    <th>SEP IRA</th>
                    <th>SIMPLE IRA</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.solo.row.2024_cap">2024 cap</td><td>$69k ($76.5k catch-up)</td><td>$69k</td><td>$16k ($19.5k catch-up)</td></tr>
                    <tr><td data-i18n="view.solo.row.employee">Employee deferral</td><td>YES ($23k)</td><td>NO</td><td>YES ($16k)</td></tr>
                    <tr><td data-i18n="view.solo.row.roth_avail">Roth deferral</td><td>YES</td><td>SECURE 2.0 yes</td><td>SECURE 2.0 yes</td></tr>
                    <tr><td data-i18n="view.solo.row.mega_back">Mega backdoor</td><td>YES (if plan allows)</td><td>NO</td><td>NO</td></tr>
                    <tr><td data-i18n="view.solo.row.loans">Loan availability</td><td>YES ($50k / 50%)</td><td>NO</td><td>NO</td></tr>
                    <tr><td data-i18n="view.solo.row.setup">Setup deadline</td><td>Dec 31</td><td>Tax return due date</td><td>Oct 1</td></tr>
                    <tr><td data-i18n="view.solo.row.spouse">Spousal contribution</td><td>YES if W-2 by biz</td><td>YES</td><td>YES</td></tr>
                    <tr><td data-i18n="view.solo.row.employees">Allows employees</td><td>NO (becomes regular 401k)</td><td>YES</td><td>YES</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('solo-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.business_type = fd.get('business_type');
        state.age = Number(fd.get('age')) || 40;
        state.net_se_earnings = Number(fd.get('net_se_earnings')) || 0;
        state.w2_wages = Number(fd.get('w2_wages')) || 0;
        state.elect_roth_deferral = !!fd.get('elect_roth_deferral');
        state.elect_mega_backdoor = !!fd.get('elect_mega_backdoor');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('solo-output');
    if (!el) return;
    const catchUp = state.age >= 60 ? LIMITS_2024.catch_up_60 : (state.age >= 50 ? LIMITS_2024.catch_up_50 : 0);
    const employeeCap = LIMITS_2024.employee_deferral + catchUp;
    const isSE = state.business_type === 'sole_prop' || state.business_type === 'partnership';
    const compForEmployer = isSE
        ? state.net_se_earnings - (state.net_se_earnings * 0.07065)  // approx half-SE-tax
        : state.w2_wages;
    const employerCap = isSE
        ? compForEmployer * LIMITS_2024.se_employer_cap_pct
        : state.w2_wages * LIMITS_2024.employer_cap_pct;
    const combinedCap = state.age >= 50 ? LIMITS_2024.combined_cap_50 + (state.age >= 60 ? LIMITS_2024.catch_up_60 - LIMITS_2024.catch_up_50 : 0) : LIMITS_2024.combined_cap;
    const totalMax = Math.min(employeeCap + employerCap, combinedCap + catchUp);
    const employeeContribution = Math.min(employeeCap, totalMax);
    const employerContribution = Math.min(employerCap, totalMax - employeeContribution);
    const megaBackdoorRoom = state.elect_mega_backdoor
        ? Math.max(0, combinedCap - employeeContribution - employerContribution)
        : 0;
    const totalContrib = employeeContribution + employerContribution + megaBackdoorRoom;
    const pretaxContrib = state.elect_roth_deferral ? employerContribution : (employeeContribution + employerContribution);
    const taxSavings = pretaxContrib * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.solo.h2.result">Contribution maximum</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.solo.card.employee">Employee deferral</div>
                    <div class="value">$${employeeContribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.solo.card.employer">Employer profit share</div>
                    <div class="value">$${employerContribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${megaBackdoorRoom > 0 ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.solo.card.mega">Mega backdoor (after-tax)</div>
                        <div class="value">$${megaBackdoorRoom.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card pos">
                    <div class="label" data-i18n="view.solo.card.total">Total maximum</div>
                    <div class="value">$${totalContrib.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.solo.card.catch_up">Catch-up portion</div>
                    <div class="value">$${catchUp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.solo.card.tax_savings">Year-1 tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.solo.card.combined_cap">Combined cap (§ 415)</div>
                    <div class="value">$${(combinedCap + catchUp).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.elect_roth_deferral ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.solo.note.roth">
                    Roth deferral: no year-1 tax saving on employee portion (you pay tax now);
                    qualified withdrawals fully tax-free. Employer match always pre-tax (Roth match
                    SECURE 2.0 optional).
                </p>
            ` : ''}
        </div>
    `;
}
