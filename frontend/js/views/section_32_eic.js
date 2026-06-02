// IRC § 32 — Earned Income Tax Credit (EITC).
// REFUNDABLE credit for low-to-moderate income workers. 2024:
// - No kids: max $632, phase-out at $19k single / $25k MFJ
// - 1 child: max $4,213, phase-out at $49k / $56k
// - 2 children: max $6,960, phase-out at $55k / $62k
// - 3+ children: max $7,830, phase-out at $59k / $66k
// Investment income limit: $11,600 (2024). Otherwise disqualified.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EIC_TABLE_2024 = {
    0: { max_credit: 632, phaseout_begin_single: 10_330, phaseout_end_single: 18_591, phaseout_begin_mfj: 17_250, phaseout_end_mfj: 25_511 },
    1: { max_credit: 4_213, phaseout_begin_single: 22_720, phaseout_end_single: 49_084, phaseout_begin_mfj: 29_640, phaseout_end_mfj: 56_004 },
    2: { max_credit: 6_960, phaseout_begin_single: 22_720, phaseout_end_single: 55_768, phaseout_begin_mfj: 29_640, phaseout_end_mfj: 62_688 },
    3: { max_credit: 7_830, phaseout_begin_single: 22_720, phaseout_end_single: 59_899, phaseout_begin_mfj: 29_640, phaseout_end_mfj: 66_819 },
};

const INVESTMENT_INCOME_LIMIT_2024 = 11_600;
const MIN_AGE_NO_KIDS = 25;
const MAX_AGE_NO_KIDS = 65;

let state = {
    qualifying_children_count: 0,
    earned_income: 0,
    agi: 0,
    investment_income: 0,
    filing_status: 'single',
    age: 30,
    is_us_citizen: true,
    has_valid_ssn: true,
    is_qualifying_child_of_another: false,
};

export async function renderSection32Eic(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.eic.h1.title">// § 32 EARNED INCOME CREDIT</span></h1>
        <p class="muted small" data-i18n="view.eic.hint.intro">
            REFUNDABLE credit for low-to-moderate income workers. 2024 max:
            <strong>$632 (no kids), $4,213 (1 child), $6,960 (2 children), $7,830 (3+ children)</strong>.
            <strong>Investment income limit: $11,600 (2024)</strong> — exceeds disqualifies. No kids:
            age 25-64 required. Self-employed eligible. <strong>PATH Act:</strong> refunds held until
            mid-Feb. <strong>One of the most under-claimed credits</strong> (20% of eligible miss).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.eic.h2.inputs">Inputs</h2>
            <form id="eic-form" class="inline-form">
                <label><span data-i18n="view.eic.label.children">Qualifying children</span>
                    <input type="number" step="1" min="0" max="3" name="qualifying_children_count" value="${state.qualifying_children_count}"></label>
                <label><span data-i18n="view.eic.label.earned">Earned income ($)</span>
                    <input type="number" step="100" name="earned_income" value="${state.earned_income}"></label>
                <label><span data-i18n="view.eic.label.agi">AGI ($)</span>
                    <input type="number" step="100" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.eic.label.investment">Investment income ($)</span>
                    <input type="number" step="100" name="investment_income" value="${state.investment_income}"></label>
                <label><span data-i18n="view.eic.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.eic.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.eic.label.citizen">US citizen / resident?</span>
                    <input type="checkbox" name="is_us_citizen" ${state.is_us_citizen ? 'checked' : ''}></label>
                <label><span data-i18n="view.eic.label.ssn">Valid SSN?</span>
                    <input type="checkbox" name="has_valid_ssn" ${state.has_valid_ssn ? 'checked' : ''}></label>
                <label><span data-i18n="view.eic.label.qualif_child">Are YOU another's qualifying child?</span>
                    <input type="checkbox" name="is_qualifying_child_of_another" ${state.is_qualifying_child_of_another ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.eic.btn.compute">Compute</button>
            </form>
        </div>
        <div id="eic-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.eic.h2.qualifying_child">Qualifying child tests</h2>
            <ul class="muted small">
                <li data-i18n="view.eic.qc.relationship">Relationship: child, stepchild, grandchild, sibling, niece/nephew, foster</li>
                <li data-i18n="view.eic.qc.age">Age: &lt; 19 OR &lt; 24 if full-time student OR any age if permanently disabled</li>
                <li data-i18n="view.eic.qc.residence">Residency: lived with you in US &gt; half the year</li>
                <li data-i18n="view.eic.qc.joint_return">Not filing joint return (unless only for refund of withholding)</li>
                <li data-i18n="view.eic.qc.support">Support: not providing &gt; 50% own support</li>
                <li data-i18n="view.eic.qc.tiebreaker">Tiebreaker rules if multiple potentials (parent over non-parent)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.eic.h2.path_act">PATH Act timing</h2>
            <p class="muted small" data-i18n="view.eic.path.body">
                Refunds claiming EITC or ACTC held until <strong>mid-February</strong> to allow IRS
                verification (combats fraud). Even if filed Jan 15, refund issued ~Feb 27. Direct
                deposit available. Form 8867 due diligence required for preparers. EITC fraud carries
                10-year ban (§ 32(k)).
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.eic.h2.state_eic">State EIC piggyback</h2>
            <ul class="muted small">
                <li data-i18n="view.eic.state.ca">California CalEITC: up to $3,529 + Young Child Tax Credit + Foster Youth credit</li>
                <li data-i18n="view.eic.state.ny">New York EITC: 30% of federal + NYC EITC 5%</li>
                <li data-i18n="view.eic.state.dc">DC EITC: 70% of federal (highest)</li>
                <li data-i18n="view.eic.state.others">31 states + DC + Puerto Rico have state EIC of various percentages</li>
                <li data-i18n="view.eic.state.refundable">State EIC may be refundable (most) or non-refundable</li>
            </ul>
        </div>
    `;
    document.getElementById('eic-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.qualifying_children_count = Math.min(3, Math.max(0, Number(fd.get('qualifying_children_count')) || 0));
        state.earned_income = Number(fd.get('earned_income')) || 0;
        state.agi = Number(fd.get('agi')) || 0;
        state.investment_income = Number(fd.get('investment_income')) || 0;
        state.filing_status = fd.get('filing_status');
        state.age = Number(fd.get('age')) || 30;
        state.is_us_citizen = !!fd.get('is_us_citizen');
        state.has_valid_ssn = !!fd.get('has_valid_ssn');
        state.is_qualifying_child_of_another = !!fd.get('is_qualifying_child_of_another');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('eic-output');
    if (!el) return;
    const ageOk = state.qualifying_children_count > 0
        || (state.age >= MIN_AGE_NO_KIDS && state.age <= MAX_AGE_NO_KIDS);
    const baseQualifies = state.is_us_citizen && state.has_valid_ssn
        && !state.is_qualifying_child_of_another && ageOk
        && state.investment_income <= INVESTMENT_INCOME_LIMIT_2024
        && state.earned_income > 0;
    const tier = EIC_TABLE_2024[state.qualifying_children_count];
    let credit = 0;
    if (baseQualifies) {
        const phaseoutBegin = state.filing_status === 'mfj' ? tier.phaseout_begin_mfj : tier.phaseout_begin_single;
        const phaseoutEnd = state.filing_status === 'mfj' ? tier.phaseout_end_mfj : tier.phaseout_end_single;
        if (Math.max(state.earned_income, state.agi) <= phaseoutBegin) credit = tier.max_credit;
        else if (Math.max(state.earned_income, state.agi) >= phaseoutEnd) credit = 0;
        else {
            credit = tier.max_credit * (1 - (Math.max(state.earned_income, state.agi) - phaseoutBegin) / (phaseoutEnd - phaseoutBegin));
        }
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.eic.h2.result">EIC calculation</h2>
            <div class="cards">
                <div class="card ${baseQualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.eic.card.qualifies">Qualifies?</div>
                    <div class="value">${baseQualifies ? esc(t('view.eic.status.yes')) : esc(t('view.eic.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.eic.card.max">Max credit</div>
                    <div class="value">$${tier.max_credit.toLocaleString()}</div>
                </div>
                <div class="card ${state.investment_income > INVESTMENT_INCOME_LIMIT_2024 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.eic.card.invest_limit">Investment income</div>
                    <div class="value">$${state.investment_income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${ageOk ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.eic.card.age">Age requirement met?</div>
                    <div class="value">${ageOk ? esc(t('view.eic.status.yes')) : esc(t('view.eic.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.eic.card.credit">EIC (refundable)</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
