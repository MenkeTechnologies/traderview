// IRC § 6404 — Abatements of Tax, Penalties, Interest.
// § 6404(a) general — IRS may abate uncollectible / erroneously assessed.
// § 6404(e) — abatement of interest attributable to IRS error/delay (limited).
// § 6404(f) — abatement of interest + penalty arising from erroneous written IRS advice.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    abatement_type: 's6404_e_irs_delay',
    tax_period: 2024,
    tax_owed: 0,
    interest_assessed: 0,
    penalty_assessed: 0,
    s6404_a_uncollectible: false,
    s6404_a_erroneously_assessed: false,
    s6404_e_irs_delay_18_months: false,
    s6404_e_managerial_or_ministerial_act: false,
    irs_delay_months: 0,
    s6404_e_2_disaster_zone: false,
    s7508a_presidential_disaster: false,
    s6404_f_erroneous_advice_irs: false,
    written_advice_received: false,
    advice_received_date: '',
    advice_relied_upon: false,
    rate_per_irs_error: 0,
    reasonable_cause_demonstrated: false,
    s6651_failure_to_file_penalty: 0,
    s6651_failure_to_pay_penalty: 0,
    s6651_a_3_failure_to_pay_penalty_on_demand: 0,
    s6654_estimated_tax_penalty: 0,
    s6655_corporate_estimated_penalty: 0,
    s6662_accuracy_penalty: 0,
    s6663_fraud_penalty: 0,
    abatement_amount_requested: 0,
    form_843_filed: false,
    form_843_attachment_explanation: false,
    cdp_proceeding_overlap: false,
    refund_claim_filed: false,
    s6511_refund_sol_3_year_2_year: 0,
    audit_recon_pending: false,
    s6020_b_substitute_for_return: false,
    s6404_h_tax_court_jurisdiction: false,
    s6404_h_petition_filed: false,
    s7430_attorney_fees: 0,
    s6404_g_failure_to_provide_notice: false,
    no_36_month_notice: false,
    s6404_g_assessment_date: '',
    notice_required_within_36_months: false,
    days_late_irs_notice: 0,
    taxpayer_acted_reasonably: true,
    abatement_granted: false,
    abated_amount_total: 0,
};

export async function renderSection6404(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6404.h1.title">// § 6404 ABATEMENT OF TAX / INTEREST / PENALTIES</span></h1>
        <p class="muted small" data-i18n="view.s6404.hint.intro">
            <strong>§ 6404</strong> provides several IRS abatement authorities. <strong>§ 6404(a):</strong>
            IRS MAY abate UNCOLLECTIBLE OR ERRONEOUSLY ASSESSED tax (discretionary). <strong>§ 6404(e):</strong>
            abate INTEREST attributable to IRS-caused delay of 18+ months in managerial/ministerial
            act (e.g., audit delays, mishandling refund claim). <strong>§ 6404(e)(2):</strong>
            mandatory abatement for Presidentially-declared disasters. <strong>§ 6404(f):</strong>
            abate INTEREST + PENALTY from RELIANCE ON ERRONEOUS WRITTEN IRS ADVICE — reasonable
            cause + reliance documented + taxpayer provided complete information. <strong>§ 6404(g):</strong>
            INTEREST + PENALTY suspended after 36 months without IRS notice of additional tax.
            <strong>§ 6404(h):</strong> Tax Court has jurisdiction over abatement denial (post-1996).
            <strong>Form 843</strong> claim for abatement / refund. <strong>"Reasonable cause"</strong>
            defense across most penalties (§ 6651, § 6662) — separately from § 6404.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.inputs">Inputs</h2>
            <form id="s6404-form" class="inline-form">
                <label><span data-i18n="view.s6404.label.type">Abatement type</span>
                    <select name="abatement_type">
                        <option value="s6404_a_uncollectible" ${state.abatement_type === 's6404_a_uncollectible' ? 'selected' : ''}>§ 6404(a) uncollectible</option>
                        <option value="s6404_a_erroneous" ${state.abatement_type === 's6404_a_erroneous' ? 'selected' : ''}>§ 6404(a) erroneous</option>
                        <option value="s6404_e_irs_delay" ${state.abatement_type === 's6404_e_irs_delay' ? 'selected' : ''}>§ 6404(e) IRS delay</option>
                        <option value="s6404_e_2_disaster" ${state.abatement_type === 's6404_e_2_disaster' ? 'selected' : ''}>§ 6404(e)(2) disaster</option>
                        <option value="s6404_f_erroneous_advice" ${state.abatement_type === 's6404_f_erroneous_advice' ? 'selected' : ''}>§ 6404(f) erroneous advice</option>
                        <option value="s6404_g_36_month" ${state.abatement_type === 's6404_g_36_month' ? 'selected' : ''}>§ 6404(g) 36-month</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6404.label.year">Tax period</span>
                    <input type="number" step="1" name="tax_period" value="${state.tax_period}"></label>
                <label><span data-i18n="view.s6404.label.tax">Tax owed ($)</span>
                    <input type="number" step="100" name="tax_owed" value="${state.tax_owed}"></label>
                <label><span data-i18n="view.s6404.label.interest">Interest ($)</span>
                    <input type="number" step="100" name="interest_assessed" value="${state.interest_assessed}"></label>
                <label><span data-i18n="view.s6404.label.penalty">Penalty ($)</span>
                    <input type="number" step="100" name="penalty_assessed" value="${state.penalty_assessed}"></label>
                <label><span data-i18n="view.s6404.label.uncollect">Uncollectible?</span>
                    <input type="checkbox" name="s6404_a_uncollectible" ${state.s6404_a_uncollectible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.erroneous">Erroneous?</span>
                    <input type="checkbox" name="s6404_a_erroneously_assessed" ${state.s6404_a_erroneously_assessed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.18m">18-mo IRS delay?</span>
                    <input type="checkbox" name="s6404_e_irs_delay_18_months" ${state.s6404_e_irs_delay_18_months ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.managerial">Managerial / ministerial?</span>
                    <input type="checkbox" name="s6404_e_managerial_or_ministerial_act" ${state.s6404_e_managerial_or_ministerial_act ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.delay_months">Delay months</span>
                    <input type="number" step="1" name="irs_delay_months" value="${state.irs_delay_months}"></label>
                <label><span data-i18n="view.s6404.label.disaster">Disaster zone?</span>
                    <input type="checkbox" name="s6404_e_2_disaster_zone" ${state.s6404_e_2_disaster_zone ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.presidential">Presidential disaster?</span>
                    <input type="checkbox" name="s7508a_presidential_disaster" ${state.s7508a_presidential_disaster ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.f_advice">§ 6404(f) advice?</span>
                    <input type="checkbox" name="s6404_f_erroneous_advice_irs" ${state.s6404_f_erroneous_advice_irs ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.written">Written advice?</span>
                    <input type="checkbox" name="written_advice_received" ${state.written_advice_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.advice_date">Advice date</span>
                    <input type="date" name="advice_received_date" value="${state.advice_received_date}"></label>
                <label><span data-i18n="view.s6404.label.relied">Relied on?</span>
                    <input type="checkbox" name="advice_relied_upon" ${state.advice_relied_upon ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.rate">Per-error rate</span>
                    <input type="number" step="0.1" name="rate_per_irs_error" value="${state.rate_per_irs_error}"></label>
                <label><span data-i18n="view.s6404.label.rc">Reasonable cause?</span>
                    <input type="checkbox" name="reasonable_cause_demonstrated" ${state.reasonable_cause_demonstrated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.ftf">§ 6651 FTF ($)</span>
                    <input type="number" step="100" name="s6651_failure_to_file_penalty" value="${state.s6651_failure_to_file_penalty}"></label>
                <label><span data-i18n="view.s6404.label.ftp">§ 6651 FTP ($)</span>
                    <input type="number" step="100" name="s6651_failure_to_pay_penalty" value="${state.s6651_failure_to_pay_penalty}"></label>
                <label><span data-i18n="view.s6404.label.ftp_demand">§ 6651(a)(3) demand ($)</span>
                    <input type="number" step="100" name="s6651_a_3_failure_to_pay_penalty_on_demand" value="${state.s6651_a_3_failure_to_pay_penalty_on_demand}"></label>
                <label><span data-i18n="view.s6404.label.s6654">§ 6654 est tax ($)</span>
                    <input type="number" step="100" name="s6654_estimated_tax_penalty" value="${state.s6654_estimated_tax_penalty}"></label>
                <label><span data-i18n="view.s6404.label.s6655">§ 6655 corp est ($)</span>
                    <input type="number" step="100" name="s6655_corporate_estimated_penalty" value="${state.s6655_corporate_estimated_penalty}"></label>
                <label><span data-i18n="view.s6404.label.s6662">§ 6662 accuracy ($)</span>
                    <input type="number" step="100" name="s6662_accuracy_penalty" value="${state.s6662_accuracy_penalty}"></label>
                <label><span data-i18n="view.s6404.label.s6663">§ 6663 fraud ($)</span>
                    <input type="number" step="100" name="s6663_fraud_penalty" value="${state.s6663_fraud_penalty}"></label>
                <label><span data-i18n="view.s6404.label.requested">Requested ($)</span>
                    <input type="number" step="100" name="abatement_amount_requested" value="${state.abatement_amount_requested}"></label>
                <label><span data-i18n="view.s6404.label.f843">Form 843 filed?</span>
                    <input type="checkbox" name="form_843_filed" ${state.form_843_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.attached">Explanation attached?</span>
                    <input type="checkbox" name="form_843_attachment_explanation" ${state.form_843_attachment_explanation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.cdp">CDP overlap?</span>
                    <input type="checkbox" name="cdp_proceeding_overlap" ${state.cdp_proceeding_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.refund_claim">Refund claim?</span>
                    <input type="checkbox" name="refund_claim_filed" ${state.refund_claim_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.s6511">§ 6511 SOL</span>
                    <input type="number" step="1" name="s6511_refund_sol_3_year_2_year" value="${state.s6511_refund_sol_3_year_2_year}"></label>
                <label><span data-i18n="view.s6404.label.audit_recon">Audit recon pending?</span>
                    <input type="checkbox" name="audit_recon_pending" ${state.audit_recon_pending ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.sfr">§ 6020(b) SFR?</span>
                    <input type="checkbox" name="s6020_b_substitute_for_return" ${state.s6020_b_substitute_for_return ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.tc_jurisdiction">§ 6404(h) TC juris?</span>
                    <input type="checkbox" name="s6404_h_tax_court_jurisdiction" ${state.s6404_h_tax_court_jurisdiction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.petition">Petition filed?</span>
                    <input type="checkbox" name="s6404_h_petition_filed" ${state.s6404_h_petition_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.s7430">§ 7430 fees ($)</span>
                    <input type="number" step="100" name="s7430_attorney_fees" value="${state.s7430_attorney_fees}"></label>
                <label><span data-i18n="view.s6404.label.s6404g">§ 6404(g) failure?</span>
                    <input type="checkbox" name="s6404_g_failure_to_provide_notice" ${state.s6404_g_failure_to_provide_notice ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.no_36">No 36-mo notice?</span>
                    <input type="checkbox" name="no_36_month_notice" ${state.no_36_month_notice ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.assess_date">Assessment date</span>
                    <input type="date" name="s6404_g_assessment_date" value="${state.s6404_g_assessment_date}"></label>
                <label><span data-i18n="view.s6404.label.notice_36">36-mo notice req?</span>
                    <input type="checkbox" name="notice_required_within_36_months" ${state.notice_required_within_36_months ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.late_notice">Late notice days</span>
                    <input type="number" step="1" name="days_late_irs_notice" value="${state.days_late_irs_notice}"></label>
                <label><span data-i18n="view.s6404.label.reasonable_action">Reasonable action?</span>
                    <input type="checkbox" name="taxpayer_acted_reasonably" ${state.taxpayer_acted_reasonably ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.granted">Granted?</span>
                    <input type="checkbox" name="abatement_granted" ${state.abatement_granted ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6404.label.abated_amt">Abated total ($)</span>
                    <input type="number" step="100" name="abated_amount_total" value="${state.abated_amount_total}"></label>
                <button class="primary" type="submit" data-i18n="view.s6404.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6404-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.s6404_e">§ 6404(e) IRS delay abatement</h2>
            <ul class="muted small">
                <li data-i18n="view.s6404.e.scope">Abate INTEREST attributable to IRS error/delay (managerial / ministerial act)</li>
                <li data-i18n="view.s6404.e.discretionary">DISCRETIONARY — IRS may abate "any portion attributable to" delay</li>
                <li data-i18n="view.s6404.e.18_months">Significant delay: typically 18+ months as benchmark</li>
                <li data-i18n="view.s6404.e.managerial">Managerial act: lost file, supervisor approval, internal review delays</li>
                <li data-i18n="view.s6404.e.ministerial">Ministerial act: typing, computation, routine processing</li>
                <li data-i18n="view.s6404.e.factors">Factors: cause of delay, IRS responsibility, taxpayer cooperation</li>
                <li data-i18n="view.s6404.e.no_substantive">NOT applicable: substantive legal decisions, audit position changes</li>
                <li data-i18n="view.s6404.e.tax_court">§ 6404(h) Tax Court review post-1996 (abuse of discretion)</li>
                <li data-i18n="view.s6404.e.s6404_e_2">§ 6404(e)(2) MANDATORY for Presidentially-declared disasters</li>
                <li data-i18n="view.s6404.e.f843">Form 843 — attach detailed explanation of IRS delay</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.s6404_f">§ 6404(f) erroneous written advice</h2>
            <ol class="muted small">
                <li data-i18n="view.s6404.f.scope">Abate interest + penalty arising from reliance on written IRS advice</li>
                <li data-i18n="view.s6404.f.written">Must be WRITTEN advice (oral advice insufficient)</li>
                <li data-i18n="view.s6404.f.complete">Taxpayer provided COMPLETE + ACCURATE information when seeking advice</li>
                <li data-i18n="view.s6404.f.relied">Relied REASONABLY on the advice</li>
                <li data-i18n="view.s6404.f.advice_subsequently_in_error">Advice subsequently determined erroneous</li>
                <li data-i18n="view.s6404.f.f843_with_supporting">Form 843 + copies of original request + advice received</li>
                <li data-i18n="view.s6404.f.s6694_preparer">Distinguish from preparer's advice (§ 6694)</li>
                <li data-i18n="view.s6404.f.regs">Reg § 301.6404-3 — detailed procedures</li>
                <li data-i18n="view.s6404.f.scope_advice">Limited to advice from IRS DIRECTLY (not website / publications)</li>
                <li data-i18n="view.s6404.f.PLR">PLR + TAM rely on facts presented — narrow window</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.s6404_g">§ 6404(g) 36-month interest + penalty suspension</h2>
            <ul class="muted small">
                <li data-i18n="view.s6404.g.purpose">Suspends interest + penalty if IRS doesn't notify within 36 MONTHS</li>
                <li data-i18n="view.s6404.g.scope">Applies to: § 6651 FTP + interest on amount in notice</li>
                <li data-i18n="view.s6404.g.commencement">Suspension period: from day after due date until 18 days after notice</li>
                <li data-i18n="view.s6404.g.s6404_g_2_a">§ 6404(g)(2)(A) — extensions of time (no suspension during extension period)</li>
                <li data-i18n="view.s6404.g.s6404_g_2_b">§ 6404(g)(2)(B) — gross misstatement (25%+) excludes suspension</li>
                <li data-i18n="view.s6404.g.s6404_g_2_c">§ 6404(g)(2)(C) — listed transactions excluded</li>
                <li data-i18n="view.s6404.g.s6404_g_2_d">§ 6404(g)(2)(D) — fraud excluded (§ 6663)</li>
                <li data-i18n="view.s6404.g.timely">Critical for taxpayers receiving long-delayed adjustment notices</li>
                <li data-i18n="view.s6404.g.notice_requirement">"Notice" requires SPECIFIC reasons + amount</li>
                <li data-i18n="view.s6404.g.30_day_letter">30-day letter or revenue agent report counts</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.first_time">"First-Time Penalty Abatement" (FTA) — administrative</h2>
            <ul class="muted small">
                <li data-i18n="view.s6404.fta.policy">IRS POLICY (NOT § 6404 statute) — IRM 20.1.1.3.6.1</li>
                <li data-i18n="view.s6404.fta.criteria">Eligibility: NO penalties in PRIOR 3 years + currently in compliance + filed all required returns</li>
                <li data-i18n="view.s6404.fta.penalties_eligible">Eligible: § 6651 FTF + FTP, § 6656 deposit penalty</li>
                <li data-i18n="view.s6404.fta.not_eligible">Not eligible: § 6662 accuracy, § 6663 fraud</li>
                <li data-i18n="view.s6404.fta.no_reasonable">No reasonable cause required — automatic 1-time waiver</li>
                <li data-i18n="view.s6404.fta.request">Request via Form 843, phone, or written letter</li>
                <li data-i18n="view.s6404.fta.first_time_only">First-time only — cannot use again for same penalty type</li>
                <li data-i18n="view.s6404.fta.applies_to_period">Applies to ONE tax period</li>
                <li data-i18n="view.s6404.fta.s6651_specific">Most common for § 6651 failure-to-file / failure-to-pay</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.reasonable_cause">Reasonable cause defense (separate from § 6404)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6404.rc.scope">Most penalties (§ 6651, § 6662, § 6677, etc.)</li>
                <li data-i18n="view.s6404.rc.standard">"Reasonable cause + good faith" — facts &amp; circumstances</li>
                <li data-i18n="view.s6404.rc.boyle">United States v. Boyle (1985): cannot rely on agent for filing</li>
                <li data-i18n="view.s6404.rc.factors">Factors: serious illness, family death, natural disaster, fire, etc.</li>
                <li data-i18n="view.s6404.rc.reliance_on_advisor">Reliance on tax advisor for SUBSTANTIVE issue (not filing)</li>
                <li data-i18n="view.s6404.rc.complete_disclosure">Complete disclosure of facts to advisor</li>
                <li data-i18n="view.s6404.rc.competent_advisor">Advisor must be competent</li>
                <li data-i18n="view.s6404.rc.documentation">Written documentation of advice received</li>
                <li data-i18n="view.s6404.rc.irm_20_1">IRM 20.1.1 Penalty Handbook lists examples</li>
                <li data-i18n="view.s6404.rc.f843">Form 843 requesting reasonable cause abatement</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.procedures">Procedures</h2>
            <ol class="muted small">
                <li data-i18n="view.s6404.proc.f843">Form 843 — claim for refund or request for abatement</li>
                <li data-i18n="view.s6404.proc.documentation">Attach: detailed explanation, supporting documents, copies of relevant correspondence</li>
                <li data-i18n="view.s6404.proc.appeal_denial">If denied: IRS Appeals consideration (Letter 854)</li>
                <li data-i18n="view.s6404.proc.tc_review">§ 6404(h) Tax Court review of § 6404(e) denial — 180-day deadline</li>
                <li data-i18n="view.s6404.proc.sol_refund">§ 6511 SOL for refund claims: 3 yrs from return filing OR 2 yrs from payment</li>
                <li data-i18n="view.s6404.proc.cdp_coordination">§ 6320 / § 6330 CDP hearing can include abatement requests</li>
                <li data-i18n="view.s6404.proc.s6402">§ 6402 set-off — refund offsets owed amounts</li>
                <li data-i18n="view.s6404.proc.s7430">§ 7430 attorney fees if prevailing on abuse of discretion claim</li>
                <li data-i18n="view.s6404.proc.tao">Taxpayer Advocate Service (TAS) when systemic burden or unreasonable delay</li>
            </ol>
        </div>
    `;
    document.getElementById('s6404-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.abatement_type = fd.get('abatement_type');
        state.tax_period = Number(fd.get('tax_period')) || 0;
        state.tax_owed = Number(fd.get('tax_owed')) || 0;
        state.interest_assessed = Number(fd.get('interest_assessed')) || 0;
        state.penalty_assessed = Number(fd.get('penalty_assessed')) || 0;
        state.s6404_a_uncollectible = !!fd.get('s6404_a_uncollectible');
        state.s6404_a_erroneously_assessed = !!fd.get('s6404_a_erroneously_assessed');
        state.s6404_e_irs_delay_18_months = !!fd.get('s6404_e_irs_delay_18_months');
        state.s6404_e_managerial_or_ministerial_act = !!fd.get('s6404_e_managerial_or_ministerial_act');
        state.irs_delay_months = Number(fd.get('irs_delay_months')) || 0;
        state.s6404_e_2_disaster_zone = !!fd.get('s6404_e_2_disaster_zone');
        state.s7508a_presidential_disaster = !!fd.get('s7508a_presidential_disaster');
        state.s6404_f_erroneous_advice_irs = !!fd.get('s6404_f_erroneous_advice_irs');
        state.written_advice_received = !!fd.get('written_advice_received');
        state.advice_received_date = fd.get('advice_received_date') || '';
        state.advice_relied_upon = !!fd.get('advice_relied_upon');
        state.rate_per_irs_error = Number(fd.get('rate_per_irs_error')) || 0;
        state.reasonable_cause_demonstrated = !!fd.get('reasonable_cause_demonstrated');
        state.s6651_failure_to_file_penalty = Number(fd.get('s6651_failure_to_file_penalty')) || 0;
        state.s6651_failure_to_pay_penalty = Number(fd.get('s6651_failure_to_pay_penalty')) || 0;
        state.s6651_a_3_failure_to_pay_penalty_on_demand = Number(fd.get('s6651_a_3_failure_to_pay_penalty_on_demand')) || 0;
        state.s6654_estimated_tax_penalty = Number(fd.get('s6654_estimated_tax_penalty')) || 0;
        state.s6655_corporate_estimated_penalty = Number(fd.get('s6655_corporate_estimated_penalty')) || 0;
        state.s6662_accuracy_penalty = Number(fd.get('s6662_accuracy_penalty')) || 0;
        state.s6663_fraud_penalty = Number(fd.get('s6663_fraud_penalty')) || 0;
        state.abatement_amount_requested = Number(fd.get('abatement_amount_requested')) || 0;
        state.form_843_filed = !!fd.get('form_843_filed');
        state.form_843_attachment_explanation = !!fd.get('form_843_attachment_explanation');
        state.cdp_proceeding_overlap = !!fd.get('cdp_proceeding_overlap');
        state.refund_claim_filed = !!fd.get('refund_claim_filed');
        state.s6511_refund_sol_3_year_2_year = Number(fd.get('s6511_refund_sol_3_year_2_year')) || 0;
        state.audit_recon_pending = !!fd.get('audit_recon_pending');
        state.s6020_b_substitute_for_return = !!fd.get('s6020_b_substitute_for_return');
        state.s6404_h_tax_court_jurisdiction = !!fd.get('s6404_h_tax_court_jurisdiction');
        state.s6404_h_petition_filed = !!fd.get('s6404_h_petition_filed');
        state.s7430_attorney_fees = Number(fd.get('s7430_attorney_fees')) || 0;
        state.s6404_g_failure_to_provide_notice = !!fd.get('s6404_g_failure_to_provide_notice');
        state.no_36_month_notice = !!fd.get('no_36_month_notice');
        state.s6404_g_assessment_date = fd.get('s6404_g_assessment_date') || '';
        state.notice_required_within_36_months = !!fd.get('notice_required_within_36_months');
        state.days_late_irs_notice = Number(fd.get('days_late_irs_notice')) || 0;
        state.taxpayer_acted_reasonably = !!fd.get('taxpayer_acted_reasonably');
        state.abatement_granted = !!fd.get('abatement_granted');
        state.abated_amount_total = Number(fd.get('abated_amount_total')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6404-output');
    if (!el) return;
    const eligible_s6404_e = state.s6404_e_irs_delay_18_months && state.s6404_e_managerial_or_ministerial_act;
    const eligible_s6404_f = state.s6404_f_erroneous_advice_irs && state.written_advice_received && state.advice_relied_upon;
    const eligible_s6404_g = state.no_36_month_notice;
    const eligible_disaster = state.s6404_e_2_disaster_zone || state.s7508a_presidential_disaster;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6404.h2.result">§ 6404 abatement eligibility</h2>
            <div class="cards">
                <div class="card ${eligible_s6404_e ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6404.card.s6404e">§ 6404(e) IRS delay?</div><div class="value">${eligible_s6404_e ? 'ELIGIBLE' : 'NO'}</div></div>
                <div class="card ${eligible_s6404_f ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6404.card.s6404f">§ 6404(f) advice?</div><div class="value">${eligible_s6404_f ? 'ELIGIBLE' : 'NO'}</div></div>
                <div class="card ${eligible_s6404_g ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6404.card.s6404g">§ 6404(g) 36-mo?</div><div class="value">${eligible_s6404_g ? 'ELIGIBLE' : 'NO'}</div></div>
                <div class="card ${eligible_disaster ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6404.card.disaster">Disaster (mandatory)?</div><div class="value">${eligible_disaster ? 'ELIGIBLE' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6404.card.requested">Requested abatement</div><div class="value">$${state.abatement_amount_requested.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
