// IRC § 67 — 2% Floor on Miscellaneous Itemized Deductions.
// Pre-TCJA: misc itemized deductions allowed only to extent > 2% of AGI.
// TCJA § 67(g): SUSPENDED misc itemized deductions entirely for 2018-2025.
// Sunsets Dec 31, 2025 — reverts to 2% AGI floor unless OBBBA extension.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    tax_year: 2024,
    agi: 0,
    misc_itemized_2pct_total: 0,
    is_tcja_suspended_year: true,
    s67_g_suspension_active: true,
    s67_g_sunset_date: '2025-12-31',
    is_post_tcja_extension: false,
    obbba_extension: false,
    misc_2pct_unreimbursed_employee_business: 0,
    misc_2pct_safe_deposit_box: 0,
    misc_2pct_tax_preparation_fees: 0,
    misc_2pct_investment_expenses: 0,
    misc_2pct_legal_fees_taxable: 0,
    misc_2pct_estate_tax_iif: 0,
    misc_2pct_dues_professional: 0,
    misc_2pct_education_work_related: 0,
    misc_2pct_other: 0,
    s67_b_above_2pct_floor: 0,
    s67_b_1_estate_admin: 0,
    s67_b_2_gambling_loss: 0,
    s67_b_3_impairment_employee: 0,
    s67_b_4_estate_tax_iif_partial: 0,
    s67_b_5_personal_property_loss_casualty: 0,
    s67_b_6_terminated_unit: 0,
    s67_b_7_short_sale_securities: 0,
    s67_b_8_amortizable_bond_premium: 0,
    s67_b_9_repayments_under_claim_right: 0,
    s67_b_10_unrecovered_pension_basis: 0,
    s67_b_11_impairment_disabled: 0,
    s67_b_12_other_specifically_listed: 0,
    s67_e_estate_trust_fiduciary: 0,
    s67_e_trustee_fees: 0,
    s67_e_separately_stated: false,
    is_estate_or_trust: false,
    s67_b_5_casualty_fdda: false,
    is_unreimbursed_employee_expense: false,
    is_qualified_performing_artist: false,
    is_reservist: false,
    is_state_local_govt_official: false,
    s62_a_2_above_line_special: 0,
    s212_investment_expenses: 0,
    is_trader_in_securities_s475: false,
    s475_f_trader_above_line: 0,
};

export async function renderSection67(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s67.h1.title">// § 67 2% MISC ITEMIZED FLOOR + TCJA SUSPENSION</span></h1>
        <p class="muted small" data-i18n="view.s67.hint.intro">
            <strong>§ 67(a)</strong> — pre-TCJA limit on MISCELLANEOUS ITEMIZED DEDUCTIONS: allowed
            only to extent &gt; 2% of AGI. <strong>§ 67(g) TCJA SUSPENSION:</strong> ALL misc itemized
            deductions DISALLOWED for 2018-2025 (sunsets Dec 31, 2025). <strong>§ 67(b) deductions
            NOT subject to 2% floor</strong> (always allowed): gambling loss to extent of winnings,
            impairment-related work expenses (handicapped), estate tax deduction (income in respect of
            decedent), short-sale-of-securities expenses, amortizable bond premium, repayments under
            claim of right, etc. <strong>§ 67(e) estates + trusts:</strong> fiduciary fees + admin
            expenses unique to estate/trust DEDUCTIBLE above line (NOT subject to suspension).
            <strong>§ 62(a) above-line deductions</strong> remain unchanged (e.g., qualified
            performing artist, reservist 100-mile, state/local government official).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s67.h2.inputs">Inputs</h2>
            <form id="s67-form" class="inline-form">
                <label><span data-i18n="view.s67.label.year">Year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s67.label.agi">AGI ($)</span>
                    <input type="number" step="0.01" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.s67.label.misc_2pct">Misc 2% total ($)</span>
                    <input type="number" step="0.01" name="misc_itemized_2pct_total" value="${state.misc_itemized_2pct_total}"></label>
                <label><span data-i18n="view.s67.label.tcja_suspended">TCJA suspended?</span>
                    <input type="checkbox" name="is_tcja_suspended_year" ${state.is_tcja_suspended_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.s67g">§ 67(g) active?</span>
                    <input type="checkbox" name="s67_g_suspension_active" ${state.s67_g_suspension_active ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.sunset">§ 67(g) sunset</span>
                    <input type="date" name="s67_g_sunset_date" value="${state.s67_g_sunset_date}"></label>
                <label><span data-i18n="view.s67.label.post_tcja">Post-TCJA ext?</span>
                    <input type="checkbox" name="is_post_tcja_extension" ${state.is_post_tcja_extension ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.obbba">OBBBA ext?</span>
                    <input type="checkbox" name="obbba_extension" ${state.obbba_extension ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.unreim_emp">Unreim emp biz ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_unreimbursed_employee_business" value="${state.misc_2pct_unreimbursed_employee_business}"></label>
                <label><span data-i18n="view.s67.label.safe">Safe deposit ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_safe_deposit_box" value="${state.misc_2pct_safe_deposit_box}"></label>
                <label><span data-i18n="view.s67.label.tax_prep">Tax prep fees ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_tax_preparation_fees" value="${state.misc_2pct_tax_preparation_fees}"></label>
                <label><span data-i18n="view.s67.label.invest_exp">Investment exp ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_investment_expenses" value="${state.misc_2pct_investment_expenses}"></label>
                <label><span data-i18n="view.s67.label.legal">Legal fees taxable ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_legal_fees_taxable" value="${state.misc_2pct_legal_fees_taxable}"></label>
                <label><span data-i18n="view.s67.label.estate_iif">Estate tax IRD ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_estate_tax_iif" value="${state.misc_2pct_estate_tax_iif}"></label>
                <label><span data-i18n="view.s67.label.dues">Prof dues ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_dues_professional" value="${state.misc_2pct_dues_professional}"></label>
                <label><span data-i18n="view.s67.label.edu">Work edu ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_education_work_related" value="${state.misc_2pct_education_work_related}"></label>
                <label><span data-i18n="view.s67.label.other">Other 2% ($)</span>
                    <input type="number" step="0.01" name="misc_2pct_other" value="${state.misc_2pct_other}"></label>
                <label><span data-i18n="view.s67.label.above_2pct">§ 67(b) above-floor ($)</span>
                    <input type="number" step="0.01" name="s67_b_above_2pct_floor" value="${state.s67_b_above_2pct_floor}"></label>
                <label><span data-i18n="view.s67.label.s67_b1">§ 67(b)(1) estate admin ($)</span>
                    <input type="number" step="0.01" name="s67_b_1_estate_admin" value="${state.s67_b_1_estate_admin}"></label>
                <label><span data-i18n="view.s67.label.s67_b2">§ 67(b)(2) gambling ($)</span>
                    <input type="number" step="0.01" name="s67_b_2_gambling_loss" value="${state.s67_b_2_gambling_loss}"></label>
                <label><span data-i18n="view.s67.label.s67_b3">§ 67(b)(3) impair emp ($)</span>
                    <input type="number" step="0.01" name="s67_b_3_impairment_employee" value="${state.s67_b_3_impairment_employee}"></label>
                <label><span data-i18n="view.s67.label.s67_b4">§ 67(b)(4) IRD partial ($)</span>
                    <input type="number" step="0.01" name="s67_b_4_estate_tax_iif_partial" value="${state.s67_b_4_estate_tax_iif_partial}"></label>
                <label><span data-i18n="view.s67.label.s67_b5">§ 67(b)(5) casualty FDDA ($)</span>
                    <input type="number" step="0.01" name="s67_b_5_personal_property_loss_casualty" value="${state.s67_b_5_personal_property_loss_casualty}"></label>
                <label><span data-i18n="view.s67.label.s67_b6">§ 67(b)(6) terminated ($)</span>
                    <input type="number" step="0.01" name="s67_b_6_terminated_unit" value="${state.s67_b_6_terminated_unit}"></label>
                <label><span data-i18n="view.s67.label.s67_b7">§ 67(b)(7) short sale ($)</span>
                    <input type="number" step="0.01" name="s67_b_7_short_sale_securities" value="${state.s67_b_7_short_sale_securities}"></label>
                <label><span data-i18n="view.s67.label.s67_b8">§ 67(b)(8) bond prem ($)</span>
                    <input type="number" step="0.01" name="s67_b_8_amortizable_bond_premium" value="${state.s67_b_8_amortizable_bond_premium}"></label>
                <label><span data-i18n="view.s67.label.s67_b9">§ 67(b)(9) repayment ($)</span>
                    <input type="number" step="0.01" name="s67_b_9_repayments_under_claim_right" value="${state.s67_b_9_repayments_under_claim_right}"></label>
                <label><span data-i18n="view.s67.label.s67_b10">§ 67(b)(10) unrec pen ($)</span>
                    <input type="number" step="0.01" name="s67_b_10_unrecovered_pension_basis" value="${state.s67_b_10_unrecovered_pension_basis}"></label>
                <label><span data-i18n="view.s67.label.s67_b11">§ 67(b)(11) impair dis ($)</span>
                    <input type="number" step="0.01" name="s67_b_11_impairment_disabled" value="${state.s67_b_11_impairment_disabled}"></label>
                <label><span data-i18n="view.s67.label.s67_b12">§ 67(b)(12) other ($)</span>
                    <input type="number" step="0.01" name="s67_b_12_other_specifically_listed" value="${state.s67_b_12_other_specifically_listed}"></label>
                <label><span data-i18n="view.s67.label.s67_e_trust">§ 67(e) estate/trust ($)</span>
                    <input type="number" step="0.01" name="s67_e_estate_trust_fiduciary" value="${state.s67_e_estate_trust_fiduciary}"></label>
                <label><span data-i18n="view.s67.label.trustee">Trustee fees ($)</span>
                    <input type="number" step="0.01" name="s67_e_trustee_fees" value="${state.s67_e_trustee_fees}"></label>
                <label><span data-i18n="view.s67.label.separate">Separately stated?</span>
                    <input type="checkbox" name="s67_e_separately_stated" ${state.s67_e_separately_stated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.estate_trust">Estate or trust?</span>
                    <input type="checkbox" name="is_estate_or_trust" ${state.is_estate_or_trust ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.fdda">FDDA?</span>
                    <input type="checkbox" name="s67_b_5_casualty_fdda" ${state.s67_b_5_casualty_fdda ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.unreim_emp_b">Unreim emp?</span>
                    <input type="checkbox" name="is_unreimbursed_employee_expense" ${state.is_unreimbursed_employee_expense ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.qpa">QPA?</span>
                    <input type="checkbox" name="is_qualified_performing_artist" ${state.is_qualified_performing_artist ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.reservist">Reservist?</span>
                    <input type="checkbox" name="is_reservist" ${state.is_reservist ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.govt">Govt official?</span>
                    <input type="checkbox" name="is_state_local_govt_official" ${state.is_state_local_govt_official ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.s62a2">§ 62(a)(2) above ($)</span>
                    <input type="number" step="0.01" name="s62_a_2_above_line_special" value="${state.s62_a_2_above_line_special}"></label>
                <label><span data-i18n="view.s67.label.s212">§ 212 inv exp ($)</span>
                    <input type="number" step="0.01" name="s212_investment_expenses" value="${state.s212_investment_expenses}"></label>
                <label><span data-i18n="view.s67.label.s475">§ 475 trader?</span>
                    <input type="checkbox" name="is_trader_in_securities_s475" ${state.is_trader_in_securities_s475 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s67.label.s475_above">§ 475 above ($)</span>
                    <input type="number" step="0.01" name="s475_f_trader_above_line" value="${state.s475_f_trader_above_line}"></label>
                <button class="primary" type="submit" data-i18n="view.s67.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s67-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s67.h2.suspended">§ 67(g) TCJA-suspended items (NOT deductible 2018-2025)</h2>
            <ul class="muted small">
                <li data-i18n="view.s67.susp.unreim">Unreimbursed employee business expenses (Form 2106 — suspended)</li>
                <li data-i18n="view.s67.susp.tax_prep">Tax preparation fees</li>
                <li data-i18n="view.s67.susp.investment">Investment expenses (§ 212) — NOT for trader (§ 475)</li>
                <li data-i18n="view.s67.susp.legal_taxable">Legal fees in connection with TAXABLE income</li>
                <li data-i18n="view.s67.susp.safe">Safe deposit box rental for taxable securities</li>
                <li data-i18n="view.s67.susp.appraisal">Appraisal fees (charitable contribution-related: still requires appraisal but fee not deductible)</li>
                <li data-i18n="view.s67.susp.iif_estate">Estate tax deduction allocated to income in respect of decedent (IRD) — § 691(c)</li>
                <li data-i18n="view.s67.susp.union">Union dues + professional dues</li>
                <li data-i18n="view.s67.susp.uniforms">Work clothing + uniforms (employer-required)</li>
                <li data-i18n="view.s67.susp.education">Work-related education expenses</li>
                <li data-i18n="view.s67.susp.tools">Tools + work supplies</li>
                <li data-i18n="view.s67.susp.home_office_employee">Home office (employee — NOT self-employed)</li>
                <li data-i18n="view.s67.susp.travel_unreim">Unreimbursed travel + meals (work-related)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s67.h2.not_suspended">§ 67(b) deductions NOT subject to 2% floor (still allowed)</h2>
            <ol class="muted small">
                <li data-i18n="view.s67.ns.gambling">Gambling losses to extent of winnings (§ 165(d))</li>
                <li data-i18n="view.s67.ns.impair_employee">Impairment-related work expenses (handicapped employees)</li>
                <li data-i18n="view.s67.ns.iif">Estate tax deduction on IRD — § 691(c) (the FULL deduction, not the portion above 2%)</li>
                <li data-i18n="view.s67.ns.short_sale">Short-sale-of-securities expenses</li>
                <li data-i18n="view.s67.ns.bond_premium">Amortizable bond premium</li>
                <li data-i18n="view.s67.ns.repayments_claim_right">Repayments under claim of right (§ 1341)</li>
                <li data-i18n="view.s67.ns.unrecovered_pension">Unrecovered investment in pension (final return)</li>
                <li data-i18n="view.s67.ns.casualty_FDDA">Casualty + theft (post-TCJA: ONLY federally declared disasters)</li>
                <li data-i18n="view.s67.ns.section_72_b">§ 72(b)(3) — annuity premium ratio</li>
                <li data-i18n="view.s67.ns.s170_charitable">§ 170 charitable contributions (subject to AGI %, NOT 2% misc floor)</li>
                <li data-i18n="view.s67.ns.s213_medical">§ 213 medical &gt; 7.5% AGI (NOT § 67 — separate floor)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s67.h2.above_line">§ 62 above-the-line preserved alternatives</h2>
            <ul class="muted small">
                <li data-i18n="view.s67.above.qpa">§ 62(b)(1) Qualified Performing Artist (limited income + multi-employer)</li>
                <li data-i18n="view.s67.above.reservist">§ 62(a)(2)(E) Armed Forces reservist (50+ miles + overnight)</li>
                <li data-i18n="view.s67.above.govt">§ 62(a)(2)(C) State/local government official (fee-basis)</li>
                <li data-i18n="view.s67.above.disabled_work">§ 62(a)(2)(D) Disabled-related work expenses (employee with disability)</li>
                <li data-i18n="view.s67.above.educator">§ 62(a)(2)(D) Eligible educator ($300 limit — supplies)</li>
                <li data-i18n="view.s67.above.s162_se">§ 162 Self-employed Schedule C — fully deductible above-line</li>
                <li data-i18n="view.s67.above.s475_trader">§ 475(f) trader: business expenses above-line (Schedule C)</li>
                <li data-i18n="view.s67.above.s62_a_15">§ 62(a)(15) — attorney fees for civil rights / qui tam recoveries</li>
                <li data-i18n="view.s67.above.s62_a_20_21">§ 62(a)(20-21) — discrimination + whistleblower attorney fees</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s67.h2.s67_e_estates">§ 67(e) estates + trusts</h2>
            <ul class="muted small">
                <li data-i18n="view.s67.s67e.fiduciary">Fiduciary fees + administration expenses UNIQUE to estate/trust: ABOVE-LINE</li>
                <li data-i18n="view.s67.s67e.unique">Test: would an INDIVIDUAL have incurred this expense?</li>
                <li data-i18n="view.s67.s67e.unique_yes">If YES → 2% floor (suspended now)</li>
                <li data-i18n="view.s67.s67e.unique_no">If NO (unique to fiduciary) → § 67(e) above-line</li>
                <li data-i18n="view.s67.s67e.knight">Knight v. Comm. (SCOTUS 2008) — investment advisor fees subject to 2% floor</li>
                <li data-i18n="view.s67.s67e.reg_2014">Reg § 1.67-4 (2014) — clarifies post-Knight: bundled fee unbundling required</li>
                <li data-i18n="view.s67.s67e.notice_2018_61">Notice 2018-61 — § 67(g) does NOT suspend § 67(e) deductions for estates/trusts</li>
                <li data-i18n="view.s67.s67e.beneficiaries">Excess deductions on termination: pass to beneficiaries (post-2017 fix per § 67(g))</li>
                <li data-i18n="view.s67.s67e.bundled">Bundled fees: portion attributable to investment advice (§ 212) NOT deductible</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s67.h2.planning">Planning post-TCJA</h2>
            <ul class="muted small">
                <li data-i18n="view.s67.plan.s475">Securities traders: § 475(f) MTM election converts § 212 expenses to § 162 (above-line)</li>
                <li data-i18n="view.s67.plan.employer">Employees: get expenses REIMBURSED by employer (accountable plan) — avoids § 67(g)</li>
                <li data-i18n="view.s67.plan.accountable">Accountable plan: substantiate + return excess + business-purpose</li>
                <li data-i18n="view.s67.plan.s125">Cafeteria plan / DCAP / HSA — exclude expenses from income</li>
                <li data-i18n="view.s67.plan.work_home_office">Self-employment / S-corp salary mix: convert to deductible above-line</li>
                <li data-i18n="view.s67.plan.qualified_disaster">Casualty losses: only federally declared disasters deductible</li>
                <li data-i18n="view.s67.plan.s162">S-corp: pay business expenses + reimburse via accountable plan</li>
                <li data-i18n="view.s67.plan.investment_management">Investment management fees: NOT deductible UNTIL § 67(g) sunsets</li>
                <li data-i18n="view.s67.plan.sunset_2026">Plan for sunset Dec 31, 2025 → 2% floor returns 2026</li>
                <li data-i18n="view.s67.plan.tax_court">Tax court appeals: § 67(g) suspension argument upheld (limited exceptions)</li>
            </ul>
        </div>
    `;
    document.getElementById('s67-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.agi = Number(fd.get('agi')) || 0;
        state.misc_itemized_2pct_total = Number(fd.get('misc_itemized_2pct_total')) || 0;
        state.is_tcja_suspended_year = !!fd.get('is_tcja_suspended_year');
        state.s67_g_suspension_active = !!fd.get('s67_g_suspension_active');
        state.s67_g_sunset_date = fd.get('s67_g_sunset_date') || '';
        state.is_post_tcja_extension = !!fd.get('is_post_tcja_extension');
        state.obbba_extension = !!fd.get('obbba_extension');
        state.misc_2pct_unreimbursed_employee_business = Number(fd.get('misc_2pct_unreimbursed_employee_business')) || 0;
        state.misc_2pct_safe_deposit_box = Number(fd.get('misc_2pct_safe_deposit_box')) || 0;
        state.misc_2pct_tax_preparation_fees = Number(fd.get('misc_2pct_tax_preparation_fees')) || 0;
        state.misc_2pct_investment_expenses = Number(fd.get('misc_2pct_investment_expenses')) || 0;
        state.misc_2pct_legal_fees_taxable = Number(fd.get('misc_2pct_legal_fees_taxable')) || 0;
        state.misc_2pct_estate_tax_iif = Number(fd.get('misc_2pct_estate_tax_iif')) || 0;
        state.misc_2pct_dues_professional = Number(fd.get('misc_2pct_dues_professional')) || 0;
        state.misc_2pct_education_work_related = Number(fd.get('misc_2pct_education_work_related')) || 0;
        state.misc_2pct_other = Number(fd.get('misc_2pct_other')) || 0;
        state.s67_b_above_2pct_floor = Number(fd.get('s67_b_above_2pct_floor')) || 0;
        state.s67_b_1_estate_admin = Number(fd.get('s67_b_1_estate_admin')) || 0;
        state.s67_b_2_gambling_loss = Number(fd.get('s67_b_2_gambling_loss')) || 0;
        state.s67_b_3_impairment_employee = Number(fd.get('s67_b_3_impairment_employee')) || 0;
        state.s67_b_4_estate_tax_iif_partial = Number(fd.get('s67_b_4_estate_tax_iif_partial')) || 0;
        state.s67_b_5_personal_property_loss_casualty = Number(fd.get('s67_b_5_personal_property_loss_casualty')) || 0;
        state.s67_b_6_terminated_unit = Number(fd.get('s67_b_6_terminated_unit')) || 0;
        state.s67_b_7_short_sale_securities = Number(fd.get('s67_b_7_short_sale_securities')) || 0;
        state.s67_b_8_amortizable_bond_premium = Number(fd.get('s67_b_8_amortizable_bond_premium')) || 0;
        state.s67_b_9_repayments_under_claim_right = Number(fd.get('s67_b_9_repayments_under_claim_right')) || 0;
        state.s67_b_10_unrecovered_pension_basis = Number(fd.get('s67_b_10_unrecovered_pension_basis')) || 0;
        state.s67_b_11_impairment_disabled = Number(fd.get('s67_b_11_impairment_disabled')) || 0;
        state.s67_b_12_other_specifically_listed = Number(fd.get('s67_b_12_other_specifically_listed')) || 0;
        state.s67_e_estate_trust_fiduciary = Number(fd.get('s67_e_estate_trust_fiduciary')) || 0;
        state.s67_e_trustee_fees = Number(fd.get('s67_e_trustee_fees')) || 0;
        state.s67_e_separately_stated = !!fd.get('s67_e_separately_stated');
        state.is_estate_or_trust = !!fd.get('is_estate_or_trust');
        state.s67_b_5_casualty_fdda = !!fd.get('s67_b_5_casualty_fdda');
        state.is_unreimbursed_employee_expense = !!fd.get('is_unreimbursed_employee_expense');
        state.is_qualified_performing_artist = !!fd.get('is_qualified_performing_artist');
        state.is_reservist = !!fd.get('is_reservist');
        state.is_state_local_govt_official = !!fd.get('is_state_local_govt_official');
        state.s62_a_2_above_line_special = Number(fd.get('s62_a_2_above_line_special')) || 0;
        state.s212_investment_expenses = Number(fd.get('s212_investment_expenses')) || 0;
        state.is_trader_in_securities_s475 = !!fd.get('is_trader_in_securities_s475');
        state.s475_f_trader_above_line = Number(fd.get('s475_f_trader_above_line')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s67-output');
    if (!el) return;
    const total_2pct = state.misc_2pct_unreimbursed_employee_business + state.misc_2pct_safe_deposit_box + state.misc_2pct_tax_preparation_fees + state.misc_2pct_investment_expenses + state.misc_2pct_legal_fees_taxable + state.misc_2pct_dues_professional + state.misc_2pct_education_work_related + state.misc_2pct_other;
    const floor = state.agi * 0.02;
    const allowed_pre_tcja = Math.max(0, total_2pct - floor);
    const allowed_now = state.s67_g_suspension_active ? 0 : allowed_pre_tcja;
    const above_line = state.s62_a_2_above_line_special + state.s475_f_trader_above_line;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s67.h2.result">§ 67 misc itemized analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s67.card.total">Total 2% misc</div><div class="value">$${total_2pct.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s67.card.floor">2% AGI floor</div><div class="value">$${floor.toLocaleString()}</div></div>
                <div class="card ${state.s67_g_suspension_active ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s67.card.suspension">§ 67(g) suspension</div><div class="value">${state.s67_g_suspension_active ? 'ACTIVE (zero)' : 'EXPIRED'}</div></div>
                <div class="card ${allowed_now > 0 ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s67.card.allowed">Currently allowed</div><div class="value">$${allowed_now.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s67.card.above">Above-line preserved</div><div class="value">$${above_line.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
