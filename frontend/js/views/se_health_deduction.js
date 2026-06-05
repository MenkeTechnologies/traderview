// SE Health Insurance Deduction — IRC § 162(l).
// Self-employed taxpayers can deduct health / dental / LTC insurance premiums
// ABOVE-the-line (Schedule 1 line 17), reducing AGI. Limited to SE net income
// after ½ SE deduction. Does NOT reduce SE tax base (still pay SE tax on the
// gross). No "double dip" with subsidized employer coverage.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LTC_AGE_LIMITS_2024 = [
    [40, 470], [50, 880], [60, 1_760], [70, 4_710], [Infinity, 5_880],
];

let state = {
    se_net_income: 100_000,
    health_premium: 12_000,
    dental_premium: 0,
    vision_premium: 0,
    ltc_premium: 0,
    age: 45,
    has_employer_subsidized_coverage: false,
    spouse_has_employer_subsidized: false,
    marginal_rate: 0.32,
    state_rate: 0.05,
};

export async function renderSeHealthDeduction(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sehealth.h1.title">// SE HEALTH INSURANCE DEDUCTION</span></h1>
        <p class="muted small" data-i18n="view.sehealth.hint.intro">
            <strong>§ 162(l):</strong> SE taxpayers deduct health / dental / vision / LTC
            premiums ABOVE-the-line (Schedule 1 line 17). LTC capped by age. Limited to
            SE net income after ½ SE deduction. Does NOT reduce SE tax base.
            Disqualified if you (or spouse) had subsidized employer coverage that month.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.sehealth.h2.inputs">Inputs</h2>
            <form id="seh-form" class="inline-form">
                <label><span data-i18n="view.sehealth.label.se_net_income">SE net income ($)</span>
                    <input type="number" step="0.01" name="se_net_income" value="${state.se_net_income}"></label>
                <label><span data-i18n="view.sehealth.label.health_premium">Health premium ($/yr)</span>
                    <input type="number" step="0.01" name="health_premium" value="${state.health_premium}"></label>
                <label><span data-i18n="view.sehealth.label.dental_premium">Dental premium ($/yr)</span>
                    <input type="number" step="0.01" name="dental_premium" value="${state.dental_premium}"></label>
                <label><span data-i18n="view.sehealth.label.vision_premium">Vision premium ($/yr)</span>
                    <input type="number" step="0.01" name="vision_premium" value="${state.vision_premium}"></label>
                <label><span data-i18n="view.sehealth.label.ltc_premium">LTC premium ($/yr)</span>
                    <input type="number" step="0.01" name="ltc_premium" value="${state.ltc_premium}"></label>
                <label><span data-i18n="view.sehealth.label.age">Age (for LTC cap)</span>
                    <input type="number" step="1" name="age" value="${state.age}" min="0" max="100"></label>
                <label><span data-i18n="view.sehealth.label.has_employer">Have employer-subsidized health?</span>
                    <input type="checkbox" name="has_employer_subsidized_coverage" ${state.has_employer_subsidized_coverage ? 'checked' : ''}></label>
                <label><span data-i18n="view.sehealth.label.spouse_has_employer">Spouse has subsidized employer health?</span>
                    <input type="checkbox" name="spouse_has_employer_subsidized" ${state.spouse_has_employer_subsidized ? 'checked' : ''}></label>
                <label><span data-i18n="view.sehealth.label.marginal_rate">Marginal federal %</span>
                    <input type="number" step="0.5" name="marginal_rate" value="${(state.marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.sehealth.label.state_rate">State rate %</span>
                    <input type="number" step="0.5" name="state_rate" value="${(state.state_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.sehealth.btn.compute">Compute</button>
            </form>
        </div>
        <div id="seh-output"></div>
    `;
    document.getElementById('seh-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.se_net_income = Number(fd.get('se_net_income')) || 0;
        state.health_premium = Number(fd.get('health_premium')) || 0;
        state.dental_premium = Number(fd.get('dental_premium')) || 0;
        state.vision_premium = Number(fd.get('vision_premium')) || 0;
        state.ltc_premium = Number(fd.get('ltc_premium')) || 0;
        state.age = Number(fd.get('age')) || 0;
        state.has_employer_subsidized_coverage = !!fd.get('has_employer_subsidized_coverage');
        state.spouse_has_employer_subsidized = !!fd.get('spouse_has_employer_subsidized');
        state.marginal_rate = (Number(fd.get('marginal_rate')) || 32) / 100;
        state.state_rate = (Number(fd.get('state_rate')) || 0) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('seh-output');
    if (!el) return;
    const ltcCap = LTC_AGE_LIMITS_2024.find(([cap, _]) => state.age <= cap)?.[1] || 5_880;
    const deductibleLtc = Math.min(state.ltc_premium, ltcCap);
    const grossPremiums = state.health_premium + state.dental_premium + state.vision_premium + deductibleLtc;
    const disqualified = state.has_employer_subsidized_coverage || state.spouse_has_employer_subsidized;
    // Income cap: SE net income minus ½ SE tax
    const seTaxBase = state.se_net_income * 0.9235;
    const seTax = Math.min(seTaxBase, 168_600) * 0.124 + seTaxBase * 0.029;
    const incomeCap = Math.max(0, state.se_net_income - seTax / 2);
    const deductible = disqualified ? 0 : Math.min(grossPremiums, incomeCap);
    const taxSavings = deductible * (state.marginal_rate + state.state_rate);
    el.innerHTML = `
        <div class="chart-panel ${deductible > 0 ? 'pos' : 'neg'}">
            <h2 data-i18n="view.sehealth.h2.result">Deduction result</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.sehealth.card.deductible">Deductible</div>
                    <div class="value">$${deductible.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sehealth.card.gross_premiums">Gross premiums</div>
                    <div class="value">$${grossPremiums.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sehealth.card.ltc_cap">LTC cap @ age ${state.age}</div>
                    <div class="value">$${ltcCap.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sehealth.card.income_cap">Income cap</div>
                    <div class="value">$${incomeCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.sehealth.card.tax_savings">Combined tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${disqualified ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.sehealth.card.disqualified">DISQUALIFIED</div>
                        <div class="value">${esc(t('view.sehealth.disqualified_msg'))}</div>
                    </div>
                ` : ''}
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.sehealth.h2.notes">Notes</h2>
            <ul class="muted small">
                <li data-i18n="view.sehealth.note.above_line">Above-the-line deduction: reduces AGI (helps unlock other phase-outs)</li>
                <li data-i18n="view.sehealth.note.no_se">Does NOT reduce SE tax base — only income tax</li>
                <li data-i18n="view.sehealth.note.month_by_month">Disqualification is month-by-month (employer subsidy any month = no deduction THAT month)</li>
                <li data-i18n="view.sehealth.note.medicare">Medicare premiums (Parts B, D, Medigap) qualify if you're on Medicare + still SE</li>
                <li data-i18n="view.sehealth.note.ltc_cap_age">LTC age-cap brackets adjust annually for inflation</li>
                <li data-i18n="view.sehealth.note.s_corp">S-corp owners: company pays premiums, reports as W-2 wages, owner deducts via § 162(l) on personal return</li>
            </ul>
        </div>
    `;
}
