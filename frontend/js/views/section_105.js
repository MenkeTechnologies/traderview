// IRC § 105 — Accident + Health Plans + Self-Employed Health Insurance.
// § 105(a) general rule: amounts received under employer-provided plan EXCLUDED from gross income.
// § 105(b) medical care reimbursement: full exclusion if reimburses medical expenses.
// § 105(h) discrimination: self-insured medical plans — discriminatory amounts taxable to HCEs.
// SE health insurance deduction: § 162(l) covered; HRA / HSA / FSA exclusion governed by § 105.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    employer_plan_payments: 0,
    medical_reimbursement: 0,
    actual_medical_expenses: 0,
    is_self_insured: false,
    is_hce: false,
    is_5_pct_owner: false,
    is_discriminatory: false,
    ssi_disability_pay: 0,
    long_term_disability: 0,
    payments_for_loss_function: 0,
    se_health_premiums: 0,
    se_se_tax_owed: 0,
    s_corp_2_pct_owner: false,
    s_corp_w2_premium: 0,
};

export async function renderSection105(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s105.h1.title">// § 105 ACCIDENT + HEALTH</span></h1>
        <p class="muted small" data-i18n="view.s105.hint.intro">
            <strong>§ 105(a):</strong> Amounts under employer accident / health plan EXCLUDED from gross income.
            <strong>§ 105(b):</strong> Medical care reimbursement — full exclusion. <strong>§ 105(c):</strong>
            permanent loss of bodily function payments — excluded. <strong>§ 105(h):</strong> SELF-INSURED
            medical plans — discriminatory amounts TAXABLE to highly compensated employees (HCE). <strong>§ 162(l)
            SE health</strong> deduction: SE persons + 2% S-corp owners deduct ABOVE-THE-LINE.
            <strong>HRA / HSA / FSA</strong> use § 105 for tax-free reimbursement.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s105.h2.inputs">Inputs</h2>
            <form id="s105-form" class="inline-form">
                <label><span data-i18n="view.s105.label.payments">Employer plan payments to employee ($)</span>
                    <input type="number" step="100" name="employer_plan_payments" value="${state.employer_plan_payments}"></label>
                <label><span data-i18n="view.s105.label.medical">Medical reimbursement ($)</span>
                    <input type="number" step="100" name="medical_reimbursement" value="${state.medical_reimbursement}"></label>
                <label><span data-i18n="view.s105.label.actual">Actual medical expenses incurred ($)</span>
                    <input type="number" step="100" name="actual_medical_expenses" value="${state.actual_medical_expenses}"></label>
                <label><span data-i18n="view.s105.label.self">Self-insured plan?</span>
                    <input type="checkbox" name="is_self_insured" ${state.is_self_insured ? 'checked' : ''}></label>
                <label><span data-i18n="view.s105.label.hce">Highly compensated employee (HCE)?</span>
                    <input type="checkbox" name="is_hce" ${state.is_hce ? 'checked' : ''}></label>
                <label><span data-i18n="view.s105.label.5_pct">5%+ owner?</span>
                    <input type="checkbox" name="is_5_pct_owner" ${state.is_5_pct_owner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s105.label.discrim">Discriminatory plan?</span>
                    <input type="checkbox" name="is_discriminatory" ${state.is_discriminatory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s105.label.ssi">SSI / disability income ($)</span>
                    <input type="number" step="100" name="ssi_disability_pay" value="${state.ssi_disability_pay}"></label>
                <label><span data-i18n="view.s105.label.ltd">Long-term disability ($)</span>
                    <input type="number" step="100" name="long_term_disability" value="${state.long_term_disability}"></label>
                <label><span data-i18n="view.s105.label.function">Loss of function payments ($)</span>
                    <input type="number" step="100" name="payments_for_loss_function" value="${state.payments_for_loss_function}"></label>
                <label><span data-i18n="view.s105.label.se_premium">SE health premiums ($)</span>
                    <input type="number" step="100" name="se_health_premiums" value="${state.se_health_premiums}"></label>
                <label><span data-i18n="view.s105.label.se_tax">SE tax owed ($)</span>
                    <input type="number" step="100" name="se_se_tax_owed" value="${state.se_se_tax_owed}"></label>
                <label><span data-i18n="view.s105.label.s_corp">2%+ S-corp owner?</span>
                    <input type="checkbox" name="s_corp_2_pct_owner" ${state.s_corp_2_pct_owner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s105.label.s_corp_premium">S-corp W-2 premium ($)</span>
                    <input type="number" step="100" name="s_corp_w2_premium" value="${state.s_corp_w2_premium}"></label>
                <button class="primary" type="submit" data-i18n="view.s105.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s105-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s105.h2.subsections">Subsections + tax treatment</h2>
            <ul class="muted small">
                <li data-i18n="view.s105.sub.a">§ 105(a): amounts under employer plan — EXCLUDED from gross income (default rule)</li>
                <li data-i18n="view.s105.sub.b">§ 105(b): medical care reimbursement up to actual expenses — full exclusion</li>
                <li data-i18n="view.s105.sub.c">§ 105(c): permanent loss / disfigurement payments — excluded (regardless of wages)</li>
                <li data-i18n="view.s105.sub.d">§ 105(d): wage / salary replacement (sick pay) — INCLUDED in income</li>
                <li data-i18n="view.s105.sub.e">§ 105(e): trust / employer-funded HRAs — § 105 treatment</li>
                <li data-i18n="view.s105.sub.h">§ 105(h): self-insured plans discrimination — HCE taxable on excess benefits</li>
                <li data-i18n="view.s105.sub.s125">§ 125 cafeteria plan + § 105: pre-tax salary reduction → reimbursement → no income</li>
                <li data-i18n="view.s105.sub.death_benefit">§ 101 death benefit: tax-free (separate from § 105)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s105.h2.s105h_discrim">§ 105(h) self-insured discrimination</h2>
            <ul class="muted small">
                <li data-i18n="view.s105.disc.eligibility">Eligibility test: 70%+ non-HCE OR 80%+ benefits non-HCE</li>
                <li data-i18n="view.s105.disc.benefits">Benefits test: same benefits available to all participants</li>
                <li data-i18n="view.s105.disc.violations">Violations: HCE taxable on "excess reimbursements" + Soc Sec</li>
                <li data-i18n="view.s105.disc.cure">Cure: equalize benefits across HCE / non-HCE</li>
                <li data-i18n="view.s105.disc.formula">Excess = (HCE benefit − average non-HCE benefit) for that HCE</li>
                <li data-i18n="view.s105.disc.timing">Test applied on year-end basis (last day of year)</li>
                <li data-i18n="view.s105.disc.insurance_carrier_safe">Insured plans NOT subject to § 105(h) — bypass via insurance</li>
                <li data-i18n="view.s105.disc.s162m">Public companies: + § 162(m) $1M deduction limit on HCE comp</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s105.h2.s162l">§ 162(l) SE Health Insurance Deduction</h2>
            <ul class="muted small">
                <li data-i18n="view.s105.l.who">Eligible: sole prop, partner, 2%+ S-corp owner, member of multimember LLC</li>
                <li data-i18n="view.s105.l.above_line">ABOVE-THE-LINE deduction — Schedule 1 (not Schedule A)</li>
                <li data-i18n="view.s105.l.spouse_dependents">Includes premiums for self, spouse, dependents, kids &lt; 27 yrs</li>
                <li data-i18n="view.s105.l.limit">Limited to earned income from business (cannot create loss)</li>
                <li data-i18n="view.s105.l.no_em_plan">NOT eligible if employee + spouse employee plan available</li>
                <li data-i18n="view.s105.l.s_corp_w2">S-corp owners: premiums must be ON W-2 (Box 1) to be deductible by shareholder</li>
                <li data-i18n="view.s105.l.no_se">Deduction does NOT reduce SE tax (separate computation)</li>
                <li data-i18n="view.s105.l.aca_marketplace">Marketplace coverage premiums qualify (must self-fund or have employer not offer)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s105.h2.hra_types">HRA + HSA + FSA types</h2>
            <ul class="muted small">
                <li data-i18n="view.s105.types.hra">HRA (Health Reimbursement Arrangement): employer-funded, reimburses medical</li>
                <li data-i18n="view.s105.types.qsehra">QSEHRA (Qualified Small Employer HRA): small biz ≤ 50 employees</li>
                <li data-i18n="view.s105.types.ichra">ICHRA (Individual Coverage HRA): premium reimbursement for individual plans</li>
                <li data-i18n="view.s105.types.hsa">HSA (Health Savings Account): individual + employer combined, requires HDHP</li>
                <li data-i18n="view.s105.types.fsa">FSA (Flexible Spending Account): salary reduction, use-it-or-lose-it</li>
                <li data-i18n="view.s105.types.dcfsa">Dependent Care FSA: $5K limit, child care + elderly care</li>
                <li data-i18n="view.s105.types.cobra">COBRA premium subsidy: § 6432 + ARPA temporary subsidies</li>
                <li data-i18n="view.s105.types.cba">Cafeteria Plan § 125: choice between cash + tax-free benefits</li>
            </ul>
        </div>
    `;
    document.getElementById('s105-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.employer_plan_payments = Number(fd.get('employer_plan_payments')) || 0;
        state.medical_reimbursement = Number(fd.get('medical_reimbursement')) || 0;
        state.actual_medical_expenses = Number(fd.get('actual_medical_expenses')) || 0;
        state.is_self_insured = !!fd.get('is_self_insured');
        state.is_hce = !!fd.get('is_hce');
        state.is_5_pct_owner = !!fd.get('is_5_pct_owner');
        state.is_discriminatory = !!fd.get('is_discriminatory');
        state.ssi_disability_pay = Number(fd.get('ssi_disability_pay')) || 0;
        state.long_term_disability = Number(fd.get('long_term_disability')) || 0;
        state.payments_for_loss_function = Number(fd.get('payments_for_loss_function')) || 0;
        state.se_health_premiums = Number(fd.get('se_health_premiums')) || 0;
        state.se_se_tax_owed = Number(fd.get('se_se_tax_owed')) || 0;
        state.s_corp_2_pct_owner = !!fd.get('s_corp_2_pct_owner');
        state.s_corp_w2_premium = Number(fd.get('s_corp_w2_premium')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s105-output');
    if (!el) return;
    const excludedMedical = Math.min(state.medical_reimbursement, state.actual_medical_expenses);
    const excessReimbursement = Math.max(0, state.medical_reimbursement - state.actual_medical_expenses);
    const discrim_taxable = (state.is_self_insured && state.is_discriminatory && state.is_hce) ? excessReimbursement : 0;
    const c_excluded = state.payments_for_loss_function;
    const se_premium_eligible = (state.se_health_premiums + state.s_corp_w2_premium) - state.se_se_tax_owed;
    const se_deduction = Math.max(0, se_premium_eligible);
    const totalExcluded = excludedMedical + c_excluded;
    const totalTaxable = discrim_taxable + state.ssi_disability_pay + state.long_term_disability;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s105.h2.result">§ 105 outcome</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s105.card.excluded_medical">§ 105(b) medical excluded</div>
                    <div class="value">$${excludedMedical.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s105.card.c_excluded">§ 105(c) loss-of-function excluded</div>
                    <div class="value">$${c_excluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s105.card.discrim_taxable">§ 105(h) discrimination taxable</div>
                    <div class="value">$${discrim_taxable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s105.card.ssi">SSI / disability income</div>
                    <div class="value">$${state.ssi_disability_pay.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s105.card.se_deduction">SE health deduction</div>
                    <div class="value">$${se_deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s105.card.total_excluded">Total excluded</div>
                    <div class="value">$${totalExcluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s105.card.total_taxable">Total taxable</div>
                    <div class="value">$${totalTaxable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${discrim_taxable > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s105.disc_note">
                    § 105(h) discrimination: excess reimbursements TAXABLE to HCE. Includes both regular
                    + Social Security earnings. Cure: equalize benefits across HCE / non-HCE OR switch to
                    INSURED plan (not subject to § 105(h)). Common issue in self-funded health plans of large employers.
                </p>
            ` : ''}
        </div>
    `;
}
