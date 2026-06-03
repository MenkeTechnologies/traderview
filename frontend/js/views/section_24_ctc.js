// IRC § 24 — Child Tax Credit (CTC) + Additional CTC (ACTC).
// 2024: $2,000 per qualifying child < 17. Refundable portion (ACTC) up to $1,700.
// MAGI phase-out: $200k single / $400k MFJ. Reduces $50 per $1k over threshold.
// § 24(h)(4) Family Credit: $500 non-refundable for other dependents.
// SSN required by due date.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const CTC_2024 = 2_000;
const REFUNDABLE_2024 = 1_700;
const PHASE_LOW_SINGLE = 200_000;
const PHASE_LOW_MFJ = 400_000;
const FAMILY_CREDIT = 500;
const PHASE_REDUCTION_PER_1K = 50;

let state = {
    filing_status: 'mfj',
    magi: 0,
    qualifying_children_count: 0,
    other_dependents_count: 0,
    earned_income: 0,
    fed_tax_liability_before_credits: 0,
    has_ssn_by_due_date: true,
};

export async function renderSection24Ctc(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ctc.h1.title">// § 24 CHILD TAX CREDIT</span></h1>
        <p class="muted small" data-i18n="view.ctc.hint.intro">
            <strong>$2,000 per qualifying child &lt; 17</strong> (2024). Refundable portion (ACTC)
            up to <strong>$1,700</strong>. MAGI phase-out: <strong>$200k single / $400k MFJ</strong>.
            Reduces $50 per $1k over threshold. <strong>§ 24(h)(4) Family Credit: $500
            non-refundable</strong> for other dependents (parents, college kids, etc.).
            <strong>SSN required by due date</strong> of return.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.ctc.h2.inputs">Inputs</h2>
            <form id="ctc-form" class="inline-form">
                <label><span data-i18n="view.ctc.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.ctc.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.ctc.label.children">Qualifying children &lt; 17</span>
                    <input type="number" step="1" name="qualifying_children_count" value="${state.qualifying_children_count}"></label>
                <label><span data-i18n="view.ctc.label.other_dep">Other dependents</span>
                    <input type="number" step="1" name="other_dependents_count" value="${state.other_dependents_count}"></label>
                <label><span data-i18n="view.ctc.label.earned">Earned income ($)</span>
                    <input type="number" step="100" name="earned_income" value="${state.earned_income}"></label>
                <label><span data-i18n="view.ctc.label.tax_liability">Tax liability before credits ($)</span>
                    <input type="number" step="100" name="fed_tax_liability_before_credits" value="${state.fed_tax_liability_before_credits}"></label>
                <label><span data-i18n="view.ctc.label.ssn">SSNs by due date?</span>
                    <input type="checkbox" name="has_ssn_by_due_date" ${state.has_ssn_by_due_date ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.ctc.btn.compute">Compute</button>
            </form>
        </div>
        <div id="ctc-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.ctc.h2.qualifying_child">Qualifying child tests</h2>
            <ul class="muted small">
                <li data-i18n="view.ctc.qc.relationship">Son, daughter, stepchild, foster, sibling, niece, nephew, grandchild</li>
                <li data-i18n="view.ctc.qc.age">Under 17 at year-end</li>
                <li data-i18n="view.ctc.qc.residency">Lived with you &gt; half the year (limited exceptions)</li>
                <li data-i18n="view.ctc.qc.support">Did not provide &gt; half own support</li>
                <li data-i18n="view.ctc.qc.us_citizen">US citizen, national, or resident alien</li>
                <li data-i18n="view.ctc.qc.dependent">Claimed as dependent on your return</li>
                <li data-i18n="view.ctc.qc.no_joint">Not filing joint return (except for refund only)</li>
                <li data-i18n="view.ctc.qc.ssn">SSN issued before return due date</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.ctc.h2.future">Future CTC changes</h2>
            <ul class="muted small">
                <li data-i18n="view.ctc.future.tcja_sunset">TCJA sunset 12/31/2025: $1,000 + lower phase-out, refundable changes</li>
                <li data-i18n="view.ctc.future.legislative">Multiple 2024 bills propose extending or increasing</li>
                <li data-i18n="view.ctc.future.2021_arpa">ARPA 2021 temporarily: $3,000/$3,600 monthly advance payments (didn't extend)</li>
                <li data-i18n="view.ctc.future.actc_increase">ACTC refundable cap increases by $100 each year through 2026</li>
                <li data-i18n="view.ctc.future.itin">ITIN children: NOT eligible for CTC; eligible for ODC</li>
                <li data-i18n="view.ctc.future.state_ctc">17 states + DC have state CTCs</li>
            </ul>
        </div>
    `;
    document.getElementById('ctc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.magi = Number(fd.get('magi')) || 0;
        state.qualifying_children_count = Number(fd.get('qualifying_children_count')) || 0;
        state.other_dependents_count = Number(fd.get('other_dependents_count')) || 0;
        state.earned_income = Number(fd.get('earned_income')) || 0;
        state.fed_tax_liability_before_credits = Number(fd.get('fed_tax_liability_before_credits')) || 0;
        state.has_ssn_by_due_date = !!fd.get('has_ssn_by_due_date');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('ctc-output');
    if (!el) return;
    if (!state.has_ssn_by_due_date) {
        el.innerHTML = `<div class="chart-panel"><p class="muted small neg" data-i18n="view.ctc.warning.no_ssn">SSN required by return due date — children without SSN ineligible for full $2,000 CTC. Eligible for $500 Other Dependent Credit instead.</p></div>`;
        return;
    }
    const phaseoutThreshold = state.filing_status === 'mfj' ? PHASE_LOW_MFJ : PHASE_LOW_SINGLE;
    const grossCtc = state.qualifying_children_count * CTC_2024;
    const familyCredit = state.other_dependents_count * FAMILY_CREDIT;
    const totalGross = grossCtc + familyCredit;
    const overThreshold = Math.max(0, state.magi - phaseoutThreshold);
    const phaseoutReduction = Math.ceil(overThreshold / 1000) * PHASE_REDUCTION_PER_1K;
    const remainingCredit = Math.max(0, totalGross - phaseoutReduction);
    const ctcAfterPhaseout = Math.min(grossCtc, Math.max(0, remainingCredit));
    const familyAfterPhaseout = remainingCredit - ctcAfterPhaseout;
    const nonRefundableCtc = Math.min(ctcAfterPhaseout, state.fed_tax_liability_before_credits);
    const remainingForRefundable = ctcAfterPhaseout - nonRefundableCtc;
    const earnedIncomeOver2500 = Math.max(0, state.earned_income - 2_500);
    const refundableLimit = earnedIncomeOver2500 * 0.15;
    const refundablePerChild = Math.min(REFUNDABLE_2024, ctcAfterPhaseout / Math.max(1, state.qualifying_children_count));
    const refundableActc = Math.min(remainingForRefundable, refundableLimit, refundablePerChild * state.qualifying_children_count);
    const nonRefundableOdc = Math.min(familyAfterPhaseout, Math.max(0, state.fed_tax_liability_before_credits - nonRefundableCtc));
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ctc.h2.result">CTC + ACTC + ODC</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.ctc.card.gross_ctc">Gross CTC</div>
                    <div class="value">$${grossCtc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ctc.card.family">Other dependent credit</div>
                    <div class="value">$${familyCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${phaseoutReduction > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.ctc.card.phaseout">Phase-out reduction</div>
                    <div class="value">$${phaseoutReduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ctc.card.non_refundable">Non-refundable CTC</div>
                    <div class="value">$${nonRefundableCtc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ctc.card.refundable">Refundable ACTC</div>
                    <div class="value">$${refundableActc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ctc.card.odc">Other Dependent Credit</div>
                    <div class="value">$${nonRefundableOdc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ctc.card.total">Total credit</div>
                    <div class="value">$${(nonRefundableCtc + refundableActc + nonRefundableOdc).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
