// IRC § 6672 — Trust Fund Recovery Penalty (TFRP).
// 100% penalty on RESPONSIBLE PERSON who WILLFULLY fails to collect / account for / pay over trust fund taxes.
// "Trust fund taxes" = withheld income + employee FICA + collected excise taxes.
// Personal liability — pierces corp veil; joint + several with corp.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_responsible_person: false,
    is_corporate_officer: false,
    is_employee_with_authority: false,
    is_outside_director: false,
    is_signature_authority: false,
    is_check_signing_authority: false,
    is_bank_signature_card: false,
    has_payroll_decision_authority: false,
    knows_payroll_taxes_due: false,
    knows_creditors_paid_instead: false,
    willful_failure: false,
    reckless_disregard: false,
    s6672_a_amount: 0,
    s6672_b_notice_required: true,
    s6672_b_designated_payment: 0,
    employer_total_trust_fund: 0,
    withheld_income_tax: 0,
    employee_fica_share: 0,
    employer_fica_share: 0,
    medicare_tax_withheld: 0,
    federal_unemployment_FUTA: 0,
    state_unemployment_SUTA: 0,
    collected_excise_taxes: 0,
    quarter_in_question: 'Q4_2024',
    forms_941_filed: false,
    forms_940_filed: false,
    form_2751_signed: false,
    is_letter_1153_issued: false,
    administrative_appeal: false,
    s6672_b_30_day_protest: 0,
    s7430_attorney_fees: 0,
    s6672_e_partial_payment_designation: false,
    s7501_constructive_trust: false,
    has_business_failed: false,
    business_distress_factor: false,
    payments_to_creditors_during_period: 0,
    s6672_d_joint_several_liability: false,
    multiple_responsible_persons: 0,
    s6672_c_contribution_right: false,
    s6321_lien_attached: false,
    s6331_levy_authorized: false,
    s6320_cdp_notice: false,
    s6330_cdp_hearing: false,
    s7202_criminal_overlap: false,
    s7202_criminal_penalty: 0,
    rea_appeal_received: false,
    paid_in_full_post_appeal: false,
    refund_suit_district_court: false,
    flora_rule_one_quarter_payment: false,
};

export async function renderSection6672(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6672.h1.title">// § 6672 TRUST FUND RECOVERY PENALTY</span></h1>
        <p class="muted small" data-i18n="view.s6672.hint.intro">
            <strong>§ 6672</strong> — 100% PENALTY on RESPONSIBLE PERSON who WILLFULLY fails to
            collect / account for / pay over TRUST FUND TAXES. <strong>"Trust fund taxes"</strong> =
            (a) WITHHELD employee income tax + (b) employee FICA (SS + Medicare) shares + (c) collected
            excise taxes. <strong>NOT</strong> employer FICA share (separate § 3111 employer liability).
            <strong>"Responsible person":</strong> any person who has SIGNATURE / DUTY / AUTHORITY over
            payment of taxes — includes officers, directors with practical authority, signatories on
            corporate accounts, employees with check-signing or decision authority. <strong>"Willful":</strong>
            voluntary + conscious + intentional decision NOT to pay (KNEW taxes owed + had funds +
            chose to pay other creditors). <strong>Personal liability + joint &amp; several</strong>
            among multiple responsible persons. <strong>§ 7501 constructive trust:</strong> employer
            holds withheld funds AS TRUSTEE for government — corp veil pierced.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6672.h2.inputs">Inputs</h2>
            <form id="s6672-form" class="inline-form">
                <label><span data-i18n="view.s6672.label.responsible">Responsible person?</span>
                    <input type="checkbox" name="is_responsible_person" ${state.is_responsible_person ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.officer">Corporate officer?</span>
                    <input type="checkbox" name="is_corporate_officer" ${state.is_corporate_officer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.employee">Employee w/ authority?</span>
                    <input type="checkbox" name="is_employee_with_authority" ${state.is_employee_with_authority ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.outside">Outside director?</span>
                    <input type="checkbox" name="is_outside_director" ${state.is_outside_director ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.signature">Signature authority?</span>
                    <input type="checkbox" name="is_signature_authority" ${state.is_signature_authority ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.check">Check-signing?</span>
                    <input type="checkbox" name="is_check_signing_authority" ${state.is_check_signing_authority ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.bank">Bank sig card?</span>
                    <input type="checkbox" name="is_bank_signature_card" ${state.is_bank_signature_card ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.payroll">Payroll authority?</span>
                    <input type="checkbox" name="has_payroll_decision_authority" ${state.has_payroll_decision_authority ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.knows_due">Knows taxes due?</span>
                    <input type="checkbox" name="knows_payroll_taxes_due" ${state.knows_payroll_taxes_due ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.other_creditors">Paid other creditors?</span>
                    <input type="checkbox" name="knows_creditors_paid_instead" ${state.knows_creditors_paid_instead ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.willful">Willful?</span>
                    <input type="checkbox" name="willful_failure" ${state.willful_failure ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.reckless">Reckless disregard?</span>
                    <input type="checkbox" name="reckless_disregard" ${state.reckless_disregard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.amount">§ 6672(a) amount ($)</span>
                    <input type="number" step="1000" name="s6672_a_amount" value="${state.s6672_a_amount}"></label>
                <label><span data-i18n="view.s6672.label.notice">§ 6672(b) notice required?</span>
                    <input type="checkbox" name="s6672_b_notice_required" ${state.s6672_b_notice_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.designated">Designated payment ($)</span>
                    <input type="number" step="1000" name="s6672_b_designated_payment" value="${state.s6672_b_designated_payment}"></label>
                <label><span data-i18n="view.s6672.label.total">Employer trust fund ($)</span>
                    <input type="number" step="1000" name="employer_total_trust_fund" value="${state.employer_total_trust_fund}"></label>
                <label><span data-i18n="view.s6672.label.withheld">Withheld income tax ($)</span>
                    <input type="number" step="1000" name="withheld_income_tax" value="${state.withheld_income_tax}"></label>
                <label><span data-i18n="view.s6672.label.emp_fica">Employee FICA ($)</span>
                    <input type="number" step="1000" name="employee_fica_share" value="${state.employee_fica_share}"></label>
                <label><span data-i18n="view.s6672.label.er_fica">Employer FICA ($)</span>
                    <input type="number" step="1000" name="employer_fica_share" value="${state.employer_fica_share}"></label>
                <label><span data-i18n="view.s6672.label.medicare">Medicare ($)</span>
                    <input type="number" step="1000" name="medicare_tax_withheld" value="${state.medicare_tax_withheld}"></label>
                <label><span data-i18n="view.s6672.label.futa">FUTA ($)</span>
                    <input type="number" step="100" name="federal_unemployment_FUTA" value="${state.federal_unemployment_FUTA}"></label>
                <label><span data-i18n="view.s6672.label.suta">SUTA ($)</span>
                    <input type="number" step="100" name="state_unemployment_SUTA" value="${state.state_unemployment_SUTA}"></label>
                <label><span data-i18n="view.s6672.label.excise">Collected excise ($)</span>
                    <input type="number" step="100" name="collected_excise_taxes" value="${state.collected_excise_taxes}"></label>
                <label><span data-i18n="view.s6672.label.quarter">Quarter</span>
                    <select name="quarter_in_question">
                        <option value="Q1_2024" ${state.quarter_in_question === 'Q1_2024' ? 'selected' : ''}>Q1 2024</option>
                        <option value="Q2_2024" ${state.quarter_in_question === 'Q2_2024' ? 'selected' : ''}>Q2 2024</option>
                        <option value="Q3_2024" ${state.quarter_in_question === 'Q3_2024' ? 'selected' : ''}>Q3 2024</option>
                        <option value="Q4_2024" ${state.quarter_in_question === 'Q4_2024' ? 'selected' : ''}>Q4 2024</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6672.label.f941">Form 941 filed?</span>
                    <input type="checkbox" name="forms_941_filed" ${state.forms_941_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.f940">Form 940 filed?</span>
                    <input type="checkbox" name="forms_940_filed" ${state.forms_940_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.f2751">Form 2751 signed?</span>
                    <input type="checkbox" name="form_2751_signed" ${state.form_2751_signed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.l1153">Letter 1153 issued?</span>
                    <input type="checkbox" name="is_letter_1153_issued" ${state.is_letter_1153_issued ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.appeal">Admin appeal?</span>
                    <input type="checkbox" name="administrative_appeal" ${state.administrative_appeal ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.protest">30-day protest ($)</span>
                    <input type="number" step="100" name="s6672_b_30_day_protest" value="${state.s6672_b_30_day_protest}"></label>
                <label><span data-i18n="view.s6672.label.s7430">§ 7430 fees ($)</span>
                    <input type="number" step="100" name="s7430_attorney_fees" value="${state.s7430_attorney_fees}"></label>
                <label><span data-i18n="view.s6672.label.partial">Partial pmt designation?</span>
                    <input type="checkbox" name="s6672_e_partial_payment_designation" ${state.s6672_e_partial_payment_designation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.s7501">§ 7501 constr trust?</span>
                    <input type="checkbox" name="s7501_constructive_trust" ${state.s7501_constructive_trust ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.failed">Business failed?</span>
                    <input type="checkbox" name="has_business_failed" ${state.has_business_failed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.distress">Distress factor?</span>
                    <input type="checkbox" name="business_distress_factor" ${state.business_distress_factor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.creditors">Pmts to creditors ($)</span>
                    <input type="number" step="1000" name="payments_to_creditors_during_period" value="${state.payments_to_creditors_during_period}"></label>
                <label><span data-i18n="view.s6672.label.joint">§ 6672(d) joint+sev?</span>
                    <input type="checkbox" name="s6672_d_joint_several_liability" ${state.s6672_d_joint_several_liability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.multi">Multiple resp persons</span>
                    <input type="number" step="1" name="multiple_responsible_persons" value="${state.multiple_responsible_persons}"></label>
                <label><span data-i18n="view.s6672.label.contribution">§ 6672(c) contribution?</span>
                    <input type="checkbox" name="s6672_c_contribution_right" ${state.s6672_c_contribution_right ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.s6321">§ 6321 lien?</span>
                    <input type="checkbox" name="s6321_lien_attached" ${state.s6321_lien_attached ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.s6331">§ 6331 levy?</span>
                    <input type="checkbox" name="s6331_levy_authorized" ${state.s6331_levy_authorized ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.s6320">§ 6320 CDP notice?</span>
                    <input type="checkbox" name="s6320_cdp_notice" ${state.s6320_cdp_notice ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.s6330">§ 6330 CDP hearing?</span>
                    <input type="checkbox" name="s6330_cdp_hearing" ${state.s6330_cdp_hearing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.s7202">§ 7202 criminal?</span>
                    <input type="checkbox" name="s7202_criminal_overlap" ${state.s7202_criminal_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.s7202_pen">§ 7202 penalty ($)</span>
                    <input type="number" step="1000" name="s7202_criminal_penalty" value="${state.s7202_criminal_penalty}"></label>
                <label><span data-i18n="view.s6672.label.rea">REA received?</span>
                    <input type="checkbox" name="rea_appeal_received" ${state.rea_appeal_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.paid_post">Paid post-appeal?</span>
                    <input type="checkbox" name="paid_in_full_post_appeal" ${state.paid_in_full_post_appeal ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.refund">Refund suit?</span>
                    <input type="checkbox" name="refund_suit_district_court" ${state.refund_suit_district_court ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6672.label.flora">Flora rule (1 qtr)?</span>
                    <input type="checkbox" name="flora_rule_one_quarter_payment" ${state.flora_rule_one_quarter_payment ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6672.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6672-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6672.h2.responsible">"Responsible person" — multifactor test</h2>
            <ol class="muted small">
                <li data-i18n="view.s6672.resp.officer">Corporate officer or director with PRACTICAL AUTHORITY (titles alone not dispositive)</li>
                <li data-i18n="view.s6672.resp.signature">Signature authority on corporate bank accounts</li>
                <li data-i18n="view.s6672.resp.check_signing">Check-signing authority (actual exercise, not just title)</li>
                <li data-i18n="view.s6672.resp.payroll_authority">Authority to direct payroll + creditor payments</li>
                <li data-i18n="view.s6672.resp.tax_filing">Authority over filing tax returns + paying taxes</li>
                <li data-i18n="view.s6672.resp.day_to_day">Day-to-day management role</li>
                <li data-i18n="view.s6672.resp.hire_fire">Hire/fire authority over employees</li>
                <li data-i18n="view.s6672.resp.bookkeeper">Bookkeeper / accountant: typically NOT responsible (no authority to disregard)</li>
                <li data-i18n="view.s6672.resp.outside_director">Outside director without practical authority: typically NOT responsible</li>
                <li data-i18n="view.s6672.resp.multiple_persons">Multiple responsible persons common in same corporation</li>
                <li data-i18n="view.s6672.resp.s6672_d">§ 6672(d) — JOINT &amp; SEVERAL liability (IRS chooses target)</li>
                <li data-i18n="view.s6672.resp.contribution">§ 6672(c) — contribution rights AMONG responsible persons</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6672.h2.willfulness">"Willfulness" test</h2>
            <ul class="muted small">
                <li data-i18n="view.s6672.will.knew">KNEW taxes were due</li>
                <li data-i18n="view.s6672.will.had_funds">HAD funds available to pay</li>
                <li data-i18n="view.s6672.will.chose">CHOSE to pay other creditors instead</li>
                <li data-i18n="view.s6672.will.voluntary">Voluntary + conscious + intentional</li>
                <li data-i18n="view.s6672.will.no_evil_motive">No "evil motive" required — paying creditors over taxes IS willful</li>
                <li data-i18n="view.s6672.will.reckless">Reckless disregard counts as willful (Slodov v. United States)</li>
                <li data-i18n="view.s6672.will.knew_after">Person who LEARNS of unpaid taxes after-the-fact + has UNencumbered funds: willful if doesn't pay</li>
                <li data-i18n="view.s6672.will.disregard">Disregard of obvious risk = willful (Mortenson)</li>
                <li data-i18n="view.s6672.will.encumbered">"Encumbered" funds (i.e., security interest holder claim): NOT available — exception</li>
                <li data-i18n="view.s6672.will.partial">Partial payment of trust fund taxes: STILL willful for remainder</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6672.h2.trust_fund_amount">Trust fund taxes — what's covered</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6672.tbl.item">Item</th><th data-i18n="view.s6672.tbl.tfrp">TFRP applies?</th><th data-i18n="view.s6672.tbl.basis">Basis</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6672.tbl.withheld_inc">Withheld income tax</td><td>YES</td><td>§ 3402 + § 7501</td></tr>
                    <tr><td data-i18n="view.s6672.tbl.emp_fica">Employee FICA (SS + Medicare)</td><td>YES</td><td>§ 3102 + § 7501</td></tr>
                    <tr><td data-i18n="view.s6672.tbl.er_fica">Employer FICA share</td><td>NO</td><td>§ 3111 (employer's own liability)</td></tr>
                    <tr><td data-i18n="view.s6672.tbl.futa">FUTA</td><td>NO</td><td>§ 3301 (employer's own)</td></tr>
                    <tr><td data-i18n="view.s6672.tbl.suta">SUTA</td><td>NO (state law)</td><td>State law (separate)</td></tr>
                    <tr><td data-i18n="view.s6672.tbl.excise">Collected excise taxes</td><td>YES</td><td>§ 7501</td></tr>
                    <tr><td data-i18n="view.s6672.tbl.tips">Tipped employee withholding</td><td>YES</td><td>§ 3401(a)(16)</td></tr>
                    <tr><td data-i18n="view.s6672.tbl.medicare_addl">Additional Medicare 0.9%</td><td>YES (employer's withholding obligation)</td><td>§ 3101(b)(2)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6672.h2.process">Administrative + judicial process</h2>
            <ol class="muted small">
                <li data-i18n="view.s6672.proc.investigation">IRS investigation: Form 4180 interview + financial records</li>
                <li data-i18n="view.s6672.proc.letter_1153">Letter 1153 — proposed TFRP determination</li>
                <li data-i18n="view.s6672.proc.form_2751">Form 2751 — Proposed Assessment of Trust Fund Recovery Penalty</li>
                <li data-i18n="view.s6672.proc.protest">30-day protest period (administrative appeal)</li>
                <li data-i18n="view.s6672.proc.appeals">IRS Appeals consideration — independent review</li>
                <li data-i18n="view.s6672.proc.assessment">Assessment — within 10-day extension period</li>
                <li data-i18n="view.s6672.proc.notice_demand">Notice + demand for payment</li>
                <li data-i18n="view.s6672.proc.s6321_lien">§ 6321 lien — automatic on failure to pay</li>
                <li data-i18n="view.s6672.proc.s6320_cdp">§ 6320 CDP hearing — lien notice within 5 business days</li>
                <li data-i18n="view.s6672.proc.s6331_levy">§ 6331 levy — after 30-day final notice</li>
                <li data-i18n="view.s6672.proc.s6330_cdp">§ 6330 CDP hearing — pre-levy</li>
                <li data-i18n="view.s6672.proc.refund">Refund suit: Flora rule — pay 1 quarter + 1 employee's TFRP, sue for refund</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6672.h2.defenses">Defenses + planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s6672.def.not_responsible">"Not responsible person" — limited authority + no payment decision power</li>
                <li data-i18n="view.s6672.def.not_willful">"Not willful" — didn't know taxes due + reasonable belief paid + no unencumbered funds</li>
                <li data-i18n="view.s6672.def.encumbered">"Funds were encumbered" — bank loan covenants required other creditor payment</li>
                <li data-i18n="view.s6672.def.unauthorized">Unauthorized actions by junior employees — not personally responsible</li>
                <li data-i18n="view.s6672.def.early_knowledge">Recently joined + had no opportunity to act before taxes due</li>
                <li data-i18n="view.s6672.def.actual_payments">Made actual payments on tax liability when funds available</li>
                <li data-i18n="view.s6672.def.bona_fide_dispute">Bona fide dispute over employee classification (1099 vs W-2)</li>
                <li data-i18n="view.s6672.def.partial_designation">Designate partial payments to trust fund vs employer portions</li>
                <li data-i18n="view.s6672.def.consider_chapter_11">Chapter 11 bankruptcy can suspend personal collection action</li>
                <li data-i18n="view.s6672.def.s6672_contribution">§ 6672(c) seek contribution from other responsible persons</li>
                <li data-i18n="view.s6672.def.no_bankruptcy_discharge">NOT dischargeable in bankruptcy (§ 523(a)(1)(C))</li>
            </ul>
        </div>
    `;
    document.getElementById('s6672-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_responsible_person = !!fd.get('is_responsible_person');
        state.is_corporate_officer = !!fd.get('is_corporate_officer');
        state.is_employee_with_authority = !!fd.get('is_employee_with_authority');
        state.is_outside_director = !!fd.get('is_outside_director');
        state.is_signature_authority = !!fd.get('is_signature_authority');
        state.is_check_signing_authority = !!fd.get('is_check_signing_authority');
        state.is_bank_signature_card = !!fd.get('is_bank_signature_card');
        state.has_payroll_decision_authority = !!fd.get('has_payroll_decision_authority');
        state.knows_payroll_taxes_due = !!fd.get('knows_payroll_taxes_due');
        state.knows_creditors_paid_instead = !!fd.get('knows_creditors_paid_instead');
        state.willful_failure = !!fd.get('willful_failure');
        state.reckless_disregard = !!fd.get('reckless_disregard');
        state.s6672_a_amount = Number(fd.get('s6672_a_amount')) || 0;
        state.s6672_b_notice_required = !!fd.get('s6672_b_notice_required');
        state.s6672_b_designated_payment = Number(fd.get('s6672_b_designated_payment')) || 0;
        state.employer_total_trust_fund = Number(fd.get('employer_total_trust_fund')) || 0;
        state.withheld_income_tax = Number(fd.get('withheld_income_tax')) || 0;
        state.employee_fica_share = Number(fd.get('employee_fica_share')) || 0;
        state.employer_fica_share = Number(fd.get('employer_fica_share')) || 0;
        state.medicare_tax_withheld = Number(fd.get('medicare_tax_withheld')) || 0;
        state.federal_unemployment_FUTA = Number(fd.get('federal_unemployment_FUTA')) || 0;
        state.state_unemployment_SUTA = Number(fd.get('state_unemployment_SUTA')) || 0;
        state.collected_excise_taxes = Number(fd.get('collected_excise_taxes')) || 0;
        state.quarter_in_question = fd.get('quarter_in_question');
        state.forms_941_filed = !!fd.get('forms_941_filed');
        state.forms_940_filed = !!fd.get('forms_940_filed');
        state.form_2751_signed = !!fd.get('form_2751_signed');
        state.is_letter_1153_issued = !!fd.get('is_letter_1153_issued');
        state.administrative_appeal = !!fd.get('administrative_appeal');
        state.s6672_b_30_day_protest = Number(fd.get('s6672_b_30_day_protest')) || 0;
        state.s7430_attorney_fees = Number(fd.get('s7430_attorney_fees')) || 0;
        state.s6672_e_partial_payment_designation = !!fd.get('s6672_e_partial_payment_designation');
        state.s7501_constructive_trust = !!fd.get('s7501_constructive_trust');
        state.has_business_failed = !!fd.get('has_business_failed');
        state.business_distress_factor = !!fd.get('business_distress_factor');
        state.payments_to_creditors_during_period = Number(fd.get('payments_to_creditors_during_period')) || 0;
        state.s6672_d_joint_several_liability = !!fd.get('s6672_d_joint_several_liability');
        state.multiple_responsible_persons = Number(fd.get('multiple_responsible_persons')) || 0;
        state.s6672_c_contribution_right = !!fd.get('s6672_c_contribution_right');
        state.s6321_lien_attached = !!fd.get('s6321_lien_attached');
        state.s6331_levy_authorized = !!fd.get('s6331_levy_authorized');
        state.s6320_cdp_notice = !!fd.get('s6320_cdp_notice');
        state.s6330_cdp_hearing = !!fd.get('s6330_cdp_hearing');
        state.s7202_criminal_overlap = !!fd.get('s7202_criminal_overlap');
        state.s7202_criminal_penalty = Number(fd.get('s7202_criminal_penalty')) || 0;
        state.rea_appeal_received = !!fd.get('rea_appeal_received');
        state.paid_in_full_post_appeal = !!fd.get('paid_in_full_post_appeal');
        state.refund_suit_district_court = !!fd.get('refund_suit_district_court');
        state.flora_rule_one_quarter_payment = !!fd.get('flora_rule_one_quarter_payment');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6672-output');
    if (!el) return;
    const trust_fund = state.withheld_income_tax + state.employee_fica_share + state.medicare_tax_withheld + state.collected_excise_taxes;
    const tfrp_penalty = (state.is_responsible_person && (state.willful_failure || state.reckless_disregard)) ? trust_fund : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6672.h2.result">§ 6672 TFRP assessment</h2>
            <div class="cards">
                <div class="card ${state.is_responsible_person ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s6672.card.responsible">Responsible?</div><div class="value">${state.is_responsible_person ? 'YES' : 'NO'}</div></div>
                <div class="card ${state.willful_failure || state.reckless_disregard ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s6672.card.willful">Willful?</div><div class="value">${state.willful_failure || state.reckless_disregard ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6672.card.trust_fund">Trust fund taxes</div><div class="value">$${trust_fund.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s6672.card.tfrp">§ 6672 TFRP (100%)</div><div class="value">$${tfrp_penalty.toLocaleString()}</div></div>
                <div class="card warn"><div class="label" data-i18n="view.s6672.card.joint">Joint &amp; several?</div><div class="value">${state.multiple_responsible_persons > 1 ? 'YES — '+state.multiple_responsible_persons+' persons' : 'No (single)'}</div></div>
            </div>
        </div>
    `;
}
