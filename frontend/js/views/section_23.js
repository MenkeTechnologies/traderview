// IRC § 23 — Adoption Credit + § 137 Employer-Provided Adoption Assistance.
// Up to $16,810 (2024) per child credit for qualified adoption expenses.
// MAGI phase-out: $252,150-$292,150 (2024). Carryforward 5 years.
// Special needs adoption: FULL credit regardless of actual expenses incurred.
// § 137: employer-paid adoption assistance up to $16,810 excluded from income (separate from credit, not double-dip).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const CREDIT_2024 = 16_810;
const PHASEOUT_START_2024 = 252_150;
const PHASEOUT_END_2024 = 292_150;

let state = {
    qualified_expenses: 0,
    magi: 0,
    is_special_needs: false,
    employer_assistance: 0,
    adoption_finalized: true,
    domestic_adoption: true,
    failed_adoption: false,
    fed_tax_liability: 0,
};

export async function renderSection23(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s23.h1.title">// § 23 ADOPTION CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s23.hint.intro">
            Up to <strong>$16,810 (2024)</strong> credit per child for qualified adoption
            expenses. MAGI phase-out: <strong>$252,150-$292,150 (2024)</strong>. <strong>5-yr
            carryforward</strong> (non-refundable). <strong>Special needs adoption: FULL credit
            regardless of expenses</strong> incurred. <strong>§ 137:</strong> employer-paid adoption
            assistance up to $16,810 excluded from income (separate, NOT double-dip).
            Form 8839.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s23.h2.inputs">Inputs</h2>
            <form id="s23-form" class="inline-form">
                <label><span data-i18n="view.s23.label.expenses">Qualified adoption expenses ($)</span>
                    <input type="number" step="100" name="qualified_expenses" value="${state.qualified_expenses}"></label>
                <label><span data-i18n="view.s23.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s23.label.special_needs">Special needs adoption?</span>
                    <input type="checkbox" name="is_special_needs" ${state.is_special_needs ? 'checked' : ''}></label>
                <label><span data-i18n="view.s23.label.employer">Employer-provided assistance ($)</span>
                    <input type="number" step="100" name="employer_assistance" value="${state.employer_assistance}"></label>
                <label><span data-i18n="view.s23.label.finalized">Adoption finalized?</span>
                    <input type="checkbox" name="adoption_finalized" ${state.adoption_finalized ? 'checked' : ''}></label>
                <label><span data-i18n="view.s23.label.domestic">Domestic adoption?</span>
                    <input type="checkbox" name="domestic_adoption" ${state.domestic_adoption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s23.label.failed">Adoption attempt that failed?</span>
                    <input type="checkbox" name="failed_adoption" ${state.failed_adoption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s23.label.tax">Federal tax liability ($)</span>
                    <input type="number" step="100" name="fed_tax_liability" value="${state.fed_tax_liability}"></label>
                <button class="primary" type="submit" data-i18n="view.s23.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s23-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s23.h2.qualified">Qualified adoption expenses</h2>
            <ul class="muted small">
                <li data-i18n="view.s23.qual.adoption_fees">Adoption fees</li>
                <li data-i18n="view.s23.qual.attorney">Court costs + attorney fees</li>
                <li data-i18n="view.s23.qual.travel">Travel + lodging + meals (while traveling for adoption)</li>
                <li data-i18n="view.s23.qual.placement">Adoption placement agency fees</li>
                <li data-i18n="view.s23.qual.home_study">Home study fees</li>
                <li data-i18n="view.s23.qual.surrogate_no">NOT: surrogate parenting expenses</li>
                <li data-i18n="view.s23.qual.spouse_child">NOT: adopting spouse's biological child</li>
                <li data-i18n="view.s23.qual.illegal">NOT: expenses violating state law / public policy</li>
                <li data-i18n="view.s23.qual.reimbursed">NOT: expenses reimbursed by employer / charity / gov't program</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s23.h2.special_needs_definition">Special needs determination (state)</h2>
            <ul class="muted small">
                <li data-i18n="view.s23.sn.us_citizen">US citizen or resident at time adoption efforts began</li>
                <li data-i18n="view.s23.sn.state_determined">State determines child cannot return to parents' home</li>
                <li data-i18n="view.s23.sn.harder_to_place">State determines specific factor makes child harder to place</li>
                <li data-i18n="view.s23.sn.factors">Factors: ethnic background, age, sibling group, medical/emotional condition</li>
                <li data-i18n="view.s23.sn.tax_id">Use Adoption Taxpayer ID Number (ATIN) before SSN issued</li>
                <li data-i18n="view.s23.sn.title_iv_e">Title IV-E foster care payments don't reduce credit</li>
                <li data-i18n="view.s23.sn.declaration">Adoption assistance agreement is documentation</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s23.h2.related">Related child / family credits</h2>
            <ul class="muted small">
                <li data-i18n="view.s23.rel.ctc">§ 24 Child Tax Credit: $2,000 per qualifying child &lt; 17</li>
                <li data-i18n="view.s23.rel.actc">§ 24 Additional CTC: refundable portion up to $1,700 (2024)</li>
                <li data-i18n="view.s23.rel.cdcc">§ 21 Child + Dependent Care Credit: 20-35% on up to $3k/$6k expenses</li>
                <li data-i18n="view.s23.rel.eic">§ 32 Earned Income Credit (EIC)</li>
                <li data-i18n="view.s23.rel.coverdell">§ 530 Coverdell ESA for adopted children</li>
                <li data-i18n="view.s23.rel.adoption_fmla">FMLA adoption leave provisions</li>
                <li data-i18n="view.s23.rel.section_137">§ 137 employer adoption assistance excluded ($16,810 limit)</li>
            </ul>
        </div>
    `;
    document.getElementById('s23-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.qualified_expenses = Number(fd.get('qualified_expenses')) || 0;
        state.magi = Number(fd.get('magi')) || 0;
        state.is_special_needs = !!fd.get('is_special_needs');
        state.employer_assistance = Number(fd.get('employer_assistance')) || 0;
        state.adoption_finalized = !!fd.get('adoption_finalized');
        state.domestic_adoption = !!fd.get('domestic_adoption');
        state.failed_adoption = !!fd.get('failed_adoption');
        state.fed_tax_liability = Number(fd.get('fed_tax_liability')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s23-output');
    if (!el) return;
    const baseAmount = state.is_special_needs ? CREDIT_2024 : Math.min(state.qualified_expenses, CREDIT_2024);
    const nonCreditedExpenses = Math.min(state.qualified_expenses, state.employer_assistance);
    const finalExpenses = Math.max(0, state.qualified_expenses - nonCreditedExpenses);
    const creditableAmount = state.is_special_needs
        ? CREDIT_2024
        : Math.min(finalExpenses, CREDIT_2024);
    let factor;
    if (state.magi <= PHASEOUT_START_2024) factor = 1;
    else if (state.magi >= PHASEOUT_END_2024) factor = 0;
    else factor = (PHASEOUT_END_2024 - state.magi) / (PHASEOUT_END_2024 - PHASEOUT_START_2024);
    const phasedCredit = creditableAmount * factor;
    const employerExclusion = Math.min(state.employer_assistance, CREDIT_2024);
    const usedNow = Math.min(phasedCredit, state.fed_tax_liability);
    const carryforward = phasedCredit - usedNow;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s23.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s23.card.base">Base credit amount</div>
                    <div class="value">$${baseAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s23.card.factor">Phase-out factor</div>
                    <div class="value">${(factor * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s23.card.credit">Credit available</div>
                    <div class="value">$${phasedCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s23.card.exclusion">§ 137 employer exclusion</div>
                    <div class="value">$${employerExclusion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s23.card.used_now">Used this year</div>
                    <div class="value">$${usedNow.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s23.card.carryforward">5-yr carryforward</div>
                    <div class="value">$${carryforward.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
