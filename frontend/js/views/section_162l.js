// IRC § 162(l) — Self-Employed Health Insurance above-the-line deduction.
// Sole props, partners, > 2% S-corp shareholders deduct medical / dental / LTC premiums.
// Limit: lesser of premiums OR net SE earnings AFTER half-SE-tax and pension deduction.
// S-corp specific: premiums must be paid by S-corp + reported on W-2 box 1 + Box 14.
// Cannot deduct if eligible for spouse's subsidized employer plan.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    business_type: 'sole_prop',
    net_se_earnings: 0,
    s_corp_w2_wages: 0,
    medical_premiums: 0,
    dental_premiums: 0,
    ltc_premiums: 0,
    age: 40,
    spouse_age: 40,
    eligible_for_spouse_plan: false,
    marginal_rate: 0.32,
};

const LTC_LIMITS_2024 = {
    age_40_below: 480,
    age_41_50: 900,
    age_51_60: 1_800,
    age_61_70: 4_810,
    age_71_above: 6_020,
};

function ltcLimit(age) {
    if (age <= 40) return LTC_LIMITS_2024.age_40_below;
    if (age <= 50) return LTC_LIMITS_2024.age_41_50;
    if (age <= 60) return LTC_LIMITS_2024.age_51_60;
    if (age <= 70) return LTC_LIMITS_2024.age_61_70;
    return LTC_LIMITS_2024.age_71_above;
}

export async function renderSection162l(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s162l.h1.title">// § 162(l) SE HEALTH INSURANCE</span></h1>
        <p class="muted small" data-i18n="view.s162l.hint.intro">
            <strong>Above-the-line</strong> deduction (Schedule 1) for medical / dental / LTC
            premiums paid by self-employed, partners, and > 2% S-corp shareholders.
            Reduces AGI (not just Schedule A). LTC limited by age table. <strong>S-corp:</strong>
            premiums must be paid by corp + included in W-2 Box 1 wages + reported in Box 14
            (not Box 3/5). DISALLOWED if eligible for spouse's subsidized employer plan.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s162l.h2.inputs">Inputs</h2>
            <form id="s162l-form" class="inline-form">
                <label><span data-i18n="view.s162l.label.business_type">Business type</span>
                    <select name="business_type">
                        <option value="sole_prop" ${state.business_type === 'sole_prop' ? 'selected' : ''}>Sole prop</option>
                        <option value="partnership" ${state.business_type === 'partnership' ? 'selected' : ''}>Partner (LLC / GP)</option>
                        <option value="s_corp" ${state.business_type === 's_corp' ? 'selected' : ''}>S-corp (>2% shareholder)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162l.label.net_se">Net SE earnings ($)</span>
                    <input type="number" step="1000" name="net_se_earnings" value="${state.net_se_earnings}"></label>
                <label><span data-i18n="view.s162l.label.s_corp_w2">S-corp W-2 wages ($)</span>
                    <input type="number" step="1000" name="s_corp_w2_wages" value="${state.s_corp_w2_wages}"></label>
                <label><span data-i18n="view.s162l.label.medical">Medical premiums ($)</span>
                    <input type="number" step="100" name="medical_premiums" value="${state.medical_premiums}"></label>
                <label><span data-i18n="view.s162l.label.dental">Dental / vision premiums ($)</span>
                    <input type="number" step="100" name="dental_premiums" value="${state.dental_premiums}"></label>
                <label><span data-i18n="view.s162l.label.ltc">LTC premiums (age-limited) ($)</span>
                    <input type="number" step="100" name="ltc_premiums" value="${state.ltc_premiums}"></label>
                <label><span data-i18n="view.s162l.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.s162l.label.spouse_age">Spouse age</span>
                    <input type="number" step="1" name="spouse_age" value="${state.spouse_age}"></label>
                <label><span data-i18n="view.s162l.label.spouse_plan">Eligible for spouse's subsidized plan?</span>
                    <input type="checkbox" name="eligible_for_spouse_plan" ${state.eligible_for_spouse_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162l.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s162l.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s162l-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162l.h2.s_corp_rules">S-corp specific rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s162l.s_corp.payor">Premiums must be paid by S-corp (not personally)</li>
                <li data-i18n="view.s162l.s_corp.w2">Must be added to W-2 Box 1 (taxable income) + Box 14 (informational)</li>
                <li data-i18n="view.s162l.s_corp.box_3_5">NOT included in Box 3/5 (FICA-exempt)</li>
                <li data-i18n="view.s162l.s_corp.shareholder">Owner deducts on personal return Schedule 1 line 17</li>
                <li data-i18n="view.s162l.s_corp.spouses">Spouse / dependents' premiums also qualify</li>
                <li data-i18n="view.s162l.s_corp.reasonable_comp">Must have reasonable comp before § 162(l) deduction allowed</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162l.h2.ltc_table">2024 LTC age limit table</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s162l.th.age">Age band</th>
                    <th data-i18n="view.s162l.th.limit">Max deduction</th>
                </tr></thead>
                <tbody>
                    <tr><td>40 and under</td><td>$480</td></tr>
                    <tr><td>41-50</td><td>$900</td></tr>
                    <tr><td>51-60</td><td>$1,800</td></tr>
                    <tr><td>61-70</td><td>$4,810</td></tr>
                    <tr><td>71 and over</td><td>$6,020</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s162l-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.business_type = fd.get('business_type');
        state.net_se_earnings = Number(fd.get('net_se_earnings')) || 0;
        state.s_corp_w2_wages = Number(fd.get('s_corp_w2_wages')) || 0;
        state.medical_premiums = Number(fd.get('medical_premiums')) || 0;
        state.dental_premiums = Number(fd.get('dental_premiums')) || 0;
        state.ltc_premiums = Number(fd.get('ltc_premiums')) || 0;
        state.age = Number(fd.get('age')) || 40;
        state.spouse_age = Number(fd.get('spouse_age')) || 40;
        state.eligible_for_spouse_plan = !!fd.get('eligible_for_spouse_plan');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s162l-output');
    if (!el) return;
    if (state.eligible_for_spouse_plan) {
        el.innerHTML = `
            <div class="chart-panel">
                <p class="muted small neg" data-i18n="view.s162l.warning.spouse_plan">
                    DISALLOWED: § 162(l)(2)(B) blocks the deduction in any month you're eligible
                    for a subsidized employer plan via spouse. Pay premiums + take Schedule A 7.5%
                    AGI floor instead.
                </p>
            </div>
        `;
        return;
    }
    const ltcLimitTotal = ltcLimit(state.age) + ltcLimit(state.spouse_age);
    const ltcAllowed = Math.min(state.ltc_premiums, ltcLimitTotal);
    const medDental = state.medical_premiums + state.dental_premiums;
    const totalPremiums = medDental + ltcAllowed;
    const cap = state.business_type === 's_corp'
        ? state.s_corp_w2_wages
        : Math.max(0, state.net_se_earnings - state.net_se_earnings * 0.07065);
    const deduction = Math.min(totalPremiums, cap);
    const taxSavings = deduction * state.marginal_rate;
    const ltcLost = state.ltc_premiums - ltcAllowed;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s162l.h2.result">Deduction calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s162l.card.med_dental">Medical + dental</div>
                    <div class="value">$${medDental.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s162l.card.ltc_limit">LTC age-limited cap</div>
                    <div class="value">$${ltcLimitTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s162l.card.ltc_allowed">LTC allowed</div>
                    <div class="value">$${ltcAllowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s162l.card.income_cap">Income cap</div>
                    <div class="value">$${cap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s162l.card.deduction">Above-the-line deduction</div>
                    <div class="value">$${deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s162l.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${ltcLost > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s162l.card.ltc_lost">LTC over-age-limit lost</div>
                        <div class="value">$${ltcLost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
