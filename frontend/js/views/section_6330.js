// IRC § 6330 — Notice + Opportunity for Hearing Before Levy (Collection Due Process - CDP).
// IRS must provide 30-day notice before levy.
// Taxpayer entitled to CDP hearing with IRS Office of Appeals (independent of original determination).
// Tax Court jurisdiction over CDP determination.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    final_notice_of_intent_received: false,
    cp90_letter_1058_received: false,
    notice_date: '',
    days_since_notice: 0,
    s6330_30_day_window: 30,
    cdp_hearing_request_filed: false,
    form_12153_filed: false,
    is_timely_filing: false,
    s6330_a_levy_proposed: false,
    levy_property_type: 'bank',
    s6330_b_independent_appeals_officer: false,
    appeals_officer_neutral: false,
    s6330_c_hearing_matters: false,
    challenge_underlying_liability: false,
    received_prior_opportunity_to_challenge: false,
    s6330_c_2_b_collection_alternatives: false,
    requested_installment_agreement: false,
    requested_offer_in_compromise: false,
    requested_currently_not_collectible: false,
    spousal_defenses: false,
    s6015_innocent_spouse_raised: false,
    s6330_d_tax_court_jurisdiction: false,
    notice_of_determination_received: false,
    petition_filed_tax_court: false,
    days_to_petition_30: 30,
    s7345_passport_revocation_overlap: false,
    s6320_lien_cdp_already_filed: false,
    s6330_e_suspension_collection: false,
    s6330_e_2_levy_suspended: false,
    collection_sol_tolled: false,
    s6502_collection_sol_10_year: 0,
    days_tolling: 0,
    equivalent_hearing_requested: false,
    is_jeopardy_levy: false,
    s6331_b_jeopardy_assessment: false,
    s7429_administrative_review: false,
    abuse_of_discretion_standard: false,
    de_novo_review_underlying: false,
    settled_offer_compromise: false,
    settled_installment_amount: 0,
    audit_reconsideration_pending: false,
};

export async function renderSection6330(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6330.h1.title">// § 6330 CDP LEVY HEARING</span></h1>
        <p class="muted small" data-i18n="view.s6330.hint.intro">
            <strong>§ 6330</strong> — IRS must provide 30-day Final Notice of Intent to Levy (Letter
            1058 / CP90) before levying property. <strong>Taxpayer's right:</strong> request CDP
            HEARING with IRS Office of Appeals (independent appeals officer who has had NO prior
            involvement). <strong>Form 12153</strong> filed within 30 days of notice date.
            <strong>Hearing matters (§ 6330(c)):</strong> (1) collection alternatives — installment
            agreement, OIC, currently-not-collectible, (2) challenges to underlying liability (only
            if NOT previously had opportunity), (3) spousal defenses (§ 6015), (4) appropriateness
            of collection action. <strong>§ 6330(d):</strong> Notice of Determination → 30 days to
            petition Tax Court. <strong>§ 6330(e):</strong> levy SUSPENDED + collection SOL TOLLED
            during proceedings. <strong>Standard of review:</strong> abuse of discretion (most issues)
            / de novo (underlying liability if eligible). <strong>§ 6320:</strong> parallel CDP for
            LIEN notice (post-filing, within 5 business days).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6330.h2.inputs">Inputs</h2>
            <form id="s6330-form" class="inline-form">
                <label><span data-i18n="view.s6330.label.final_notice">Final notice received?</span>
                    <input type="checkbox" name="final_notice_of_intent_received" ${state.final_notice_of_intent_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.cp90">CP90/Letter 1058?</span>
                    <input type="checkbox" name="cp90_letter_1058_received" ${state.cp90_letter_1058_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.notice_date">Notice date</span>
                    <input type="date" name="notice_date" value="${state.notice_date}"></label>
                <label><span data-i18n="view.s6330.label.days">Days since notice</span>
                    <input type="number" step="1" name="days_since_notice" value="${state.days_since_notice}"></label>
                <label><span data-i18n="view.s6330.label.window">30-day window</span>
                    <input type="number" step="1" name="s6330_30_day_window" value="${state.s6330_30_day_window}"></label>
                <label><span data-i18n="view.s6330.label.requested">CDP requested?</span>
                    <input type="checkbox" name="cdp_hearing_request_filed" ${state.cdp_hearing_request_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.f12153">Form 12153 filed?</span>
                    <input type="checkbox" name="form_12153_filed" ${state.form_12153_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.timely">Timely?</span>
                    <input type="checkbox" name="is_timely_filing" ${state.is_timely_filing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.levy_proposed">Levy proposed?</span>
                    <input type="checkbox" name="s6330_a_levy_proposed" ${state.s6330_a_levy_proposed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.property">Levy property</span>
                    <select name="levy_property_type">
                        <option value="bank" ${state.levy_property_type === 'bank' ? 'selected' : ''}>Bank account</option>
                        <option value="wages" ${state.levy_property_type === 'wages' ? 'selected' : ''}>Wages</option>
                        <option value="ar" ${state.levy_property_type === 'ar' ? 'selected' : ''}>A/R (3rd-party debtor)</option>
                        <option value="real_estate" ${state.levy_property_type === 'real_estate' ? 'selected' : ''}>Real estate</option>
                        <option value="retirement" ${state.levy_property_type === 'retirement' ? 'selected' : ''}>Retirement account</option>
                        <option value="ssi" ${state.levy_property_type === 'ssi' ? 'selected' : ''}>Social Security</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6330.label.indep">Independent appeals?</span>
                    <input type="checkbox" name="s6330_b_independent_appeals_officer" ${state.s6330_b_independent_appeals_officer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.neutral">Neutral AO?</span>
                    <input type="checkbox" name="appeals_officer_neutral" ${state.appeals_officer_neutral ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.matters">Hearing matters?</span>
                    <input type="checkbox" name="s6330_c_hearing_matters" ${state.s6330_c_hearing_matters ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.under_liab">Challenge liability?</span>
                    <input type="checkbox" name="challenge_underlying_liability" ${state.challenge_underlying_liability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.prior">Prior opportunity?</span>
                    <input type="checkbox" name="received_prior_opportunity_to_challenge" ${state.received_prior_opportunity_to_challenge ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.alternatives">Collection alternatives?</span>
                    <input type="checkbox" name="s6330_c_2_b_collection_alternatives" ${state.s6330_c_2_b_collection_alternatives ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.ia">Installment Agreement?</span>
                    <input type="checkbox" name="requested_installment_agreement" ${state.requested_installment_agreement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.oic">Offer in Compromise?</span>
                    <input type="checkbox" name="requested_offer_in_compromise" ${state.requested_offer_in_compromise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.cnc">CNC requested?</span>
                    <input type="checkbox" name="requested_currently_not_collectible" ${state.requested_currently_not_collectible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.spousal">Spousal defenses?</span>
                    <input type="checkbox" name="spousal_defenses" ${state.spousal_defenses ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.s6015">§ 6015 innocent spouse?</span>
                    <input type="checkbox" name="s6015_innocent_spouse_raised" ${state.s6015_innocent_spouse_raised ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.tc_jurisdiction">TC jurisdiction?</span>
                    <input type="checkbox" name="s6330_d_tax_court_jurisdiction" ${state.s6330_d_tax_court_jurisdiction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.notice_det">Notice of Det received?</span>
                    <input type="checkbox" name="notice_of_determination_received" ${state.notice_of_determination_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.petition">Petition filed?</span>
                    <input type="checkbox" name="petition_filed_tax_court" ${state.petition_filed_tax_court ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.30day">30-day petition</span>
                    <input type="number" step="1" name="days_to_petition_30" value="${state.days_to_petition_30}"></label>
                <label><span data-i18n="view.s6330.label.passport">§ 7345 passport?</span>
                    <input type="checkbox" name="s7345_passport_revocation_overlap" ${state.s7345_passport_revocation_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.s6320">§ 6320 lien CDP?</span>
                    <input type="checkbox" name="s6320_lien_cdp_already_filed" ${state.s6320_lien_cdp_already_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.suspension">§ 6330(e) suspension?</span>
                    <input type="checkbox" name="s6330_e_suspension_collection" ${state.s6330_e_suspension_collection ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.levy_suspended">Levy suspended?</span>
                    <input type="checkbox" name="s6330_e_2_levy_suspended" ${state.s6330_e_2_levy_suspended ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.sol_tolled">SOL tolled?</span>
                    <input type="checkbox" name="collection_sol_tolled" ${state.collection_sol_tolled ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.s6502">§ 6502 SOL (days)</span>
                    <input type="number" step="1" name="s6502_collection_sol_10_year" value="${state.s6502_collection_sol_10_year}"></label>
                <label><span data-i18n="view.s6330.label.tolling">Days tolling</span>
                    <input type="number" step="1" name="days_tolling" value="${state.days_tolling}"></label>
                <label><span data-i18n="view.s6330.label.equiv">Equivalent hearing?</span>
                    <input type="checkbox" name="equivalent_hearing_requested" ${state.equivalent_hearing_requested ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.jeopardy">Jeopardy levy?</span>
                    <input type="checkbox" name="is_jeopardy_levy" ${state.is_jeopardy_levy ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.s6331b">§ 6331(b) jeopardy?</span>
                    <input type="checkbox" name="s6331_b_jeopardy_assessment" ${state.s6331_b_jeopardy_assessment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.s7429">§ 7429 review?</span>
                    <input type="checkbox" name="s7429_administrative_review" ${state.s7429_administrative_review ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.abuse">Abuse of discretion?</span>
                    <input type="checkbox" name="abuse_of_discretion_standard" ${state.abuse_of_discretion_standard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.de_novo">De novo review?</span>
                    <input type="checkbox" name="de_novo_review_underlying" ${state.de_novo_review_underlying ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.settled_oic">Settled OIC?</span>
                    <input type="checkbox" name="settled_offer_compromise" ${state.settled_offer_compromise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6330.label.settled_ia">Settled IA amt ($)</span>
                    <input type="number" step="0.01" name="settled_installment_amount" value="${state.settled_installment_amount}"></label>
                <label><span data-i18n="view.s6330.label.audit_recon">Audit recon pending?</span>
                    <input type="checkbox" name="audit_reconsideration_pending" ${state.audit_reconsideration_pending ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6330.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6330-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6330.h2.process">CDP process timeline</h2>
            <ol class="muted small">
                <li data-i18n="view.s6330.proc.notice">IRS issues Final Notice of Intent to Levy + Right to a Hearing (Letter 1058 / CP90 / Letter 11)</li>
                <li data-i18n="view.s6330.proc.30_days">Taxpayer has 30 DAYS from notice DATE to request CDP hearing</li>
                <li data-i18n="view.s6330.proc.f12153">File Form 12153 — Request for Collection Due Process Hearing</li>
                <li data-i18n="view.s6330.proc.suspension">Filing SUSPENDS collection (no levy) until hearing complete + appeals exhausted</li>
                <li data-i18n="view.s6330.proc.sol_tolled">Collection SOL (§ 6502 — 10 years) is TOLLED during hearing + appeal</li>
                <li data-i18n="view.s6330.proc.appeals">Appeals officer conducts hearing — taxpayer can raise § 6330(c) matters</li>
                <li data-i18n="view.s6330.proc.notice_det">Notice of Determination issued by Appeals — concludes hearing</li>
                <li data-i18n="view.s6330.proc.petition_30">30 days from Notice of Determination to petition Tax Court (§ 6330(d))</li>
                <li data-i18n="view.s6330.proc.tax_court">Tax Court hearing — reviews CDP determination</li>
                <li data-i18n="view.s6330.proc.collection_resumes">If no petition or after Tax Court rules: collection action may resume</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6330.h2.matters">§ 6330(c) hearing matters</h2>
            <ul class="muted small">
                <li data-i18n="view.s6330.m.collection">Collection alternatives — IA, OIC, currently-not-collectible (CNC)</li>
                <li data-i18n="view.s6330.m.underlying">Challenges to underlying liability — ONLY if no prior opportunity</li>
                <li data-i18n="view.s6330.m.spousal">Spousal defenses — § 6015 innocent spouse relief</li>
                <li data-i18n="view.s6330.m.appropriate">Appropriateness of collection action — alternatives vs intrusion</li>
                <li data-i18n="view.s6330.m.s6320">§ 6320 also lien-related issues (subordination, withdrawal, release)</li>
                <li data-i18n="view.s6330.m.spousal_relief_first_time">Spousal defense allowed even if previously could have raised</li>
                <li data-i18n="view.s6330.m.under_liab_test">"Prior opportunity" = received Statutory Notice of Deficiency OR participated in pre-assessment audit</li>
                <li data-i18n="view.s6330.m.s7521">§ 7521 — right to representation + record</li>
                <li data-i18n="view.s6330.m.proc_irregular">Procedural irregularities (e.g., § 6751(b) supervisor approval)</li>
                <li data-i18n="view.s6330.m.s6751_b">§ 6751(b) supervisor approval challenge — often successful</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6330.h2.alternatives">Collection alternatives</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6330.tbl.alt">Alternative</th><th data-i18n="view.s6330.tbl.description">Description</th><th data-i18n="view.s6330.tbl.notes">Notes</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6330.tbl.ia">Installment Agreement (§ 6159)</td><td data-i18n="view.s6330.tbl.ia_desc">Monthly payment plan</td><td data-i18n="view.s6330.tbl.ia_note">Streamlined ≤$50K / Direct Debit / Partial Pay IA</td></tr>
                    <tr><td data-i18n="view.s6330.tbl.oic">Offer in Compromise (§ 7122)</td><td data-i18n="view.s6330.tbl.oic_desc">Settle for less than full</td><td data-i18n="view.s6330.tbl.oic_note">Doubt as to liability / collectibility / ETA</td></tr>
                    <tr><td data-i18n="view.s6330.tbl.cnc">Currently Not Collectible</td><td data-i18n="view.s6330.tbl.cnc_desc">Hardship status — no collection action</td><td data-i18n="view.s6330.tbl.cnc_note">CSED runs; financial hardship documented</td></tr>
                    <tr><td data-i18n="view.s6330.tbl.subordination">Lien subordination (§ 6325(d))</td><td data-i18n="view.s6330.tbl.subord_desc">Subordinate to other creditor</td><td data-i18n="view.s6330.tbl.subord_note">For refinancing / business operations</td></tr>
                    <tr><td data-i18n="view.s6330.tbl.withdrawal">Lien withdrawal (§ 6323(j))</td><td data-i18n="view.s6330.tbl.withdrawal_desc">Remove lien from public records</td><td data-i18n="view.s6330.tbl.withdrawal_note">Limited grounds — premature, DDIA, etc.</td></tr>
                    <tr><td data-i18n="view.s6330.tbl.discharge">Lien discharge (§ 6325(b))</td><td data-i18n="view.s6330.tbl.discharge_desc">Release specific property from lien</td><td data-i18n="view.s6330.tbl.discharge_note">For sale of property</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6330.h2.standards">Standards of review</h2>
            <ul class="muted small">
                <li data-i18n="view.s6330.std.abuse">"Abuse of discretion" — standard for collection alternatives, spousal defenses, appropriateness</li>
                <li data-i18n="view.s6330.std.de_novo">"De novo" — review of underlying liability (when properly at issue)</li>
                <li data-i18n="view.s6330.std.factors">Abuse of discretion factors: arbitrary, capricious, without rational basis</li>
                <li data-i18n="view.s6330.std.s6330_c_3">§ 6330(c)(3) — Appeals officer required to balance need for collection vs intrusion</li>
                <li data-i18n="view.s6330.std.lunsford">Lunsford v. Comm. — equivalent hearing not subject to Tax Court review (vs CDP)</li>
                <li data-i18n="view.s6330.std.davis">Davis v. Comm. — Tax Court reviews Appeals' OIC denial for abuse of discretion</li>
                <li data-i18n="view.s6330.std.thomas">Thomas v. Comm. — taxpayer must offer alternative + cooperate (cannot simply refuse)</li>
                <li data-i18n="view.s6330.std.administrative_record">Administrative record review (no new evidence outside hearing typically)</li>
                <li data-i18n="view.s6330.std.s6330_c_2_a">§ 6330(c)(2)(A)(i) — verification IRS followed all law + admin procedure</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6330.h2.coordination">Coordination + special situations</h2>
            <ul class="muted small">
                <li data-i18n="view.s6330.coord.s6320">§ 6320 — parallel CDP for LIEN notice (within 5 business days of NFTL)</li>
                <li data-i18n="view.s6330.coord.s6502">§ 6502 — collection SOL (10 years) TOLLED during CDP + appeal</li>
                <li data-i18n="view.s6330.coord.s7508a">§ 7508A — disaster declaration may suspend collection</li>
                <li data-i18n="view.s6330.coord.jeopardy">Jeopardy levy: § 6330(f) NO CDP — § 7429 separate review</li>
                <li data-i18n="view.s6330.coord.state_levy">State refund levy: NO CDP (§ 6330(f) exception)</li>
                <li data-i18n="view.s6330.coord.federal_employee_pay">Federal employee pay levy: NO CDP for some IRS-determined cases</li>
                <li data-i18n="view.s6330.coord.equivalent">Equivalent hearing: missed 30-day window — within 1 year possible (NO Tax Court review)</li>
                <li data-i18n="view.s6330.coord.s7345">§ 7345 passport revocation for seriously delinquent tax debts ($62K+ in 2024)</li>
                <li data-i18n="view.s6330.coord.audit_recon">Audit reconsideration: separately request review of underlying tax</li>
                <li data-i18n="view.s6330.coord.tco">Taxpayer Advocate Service (TAO) — collaborative review when systemic burden</li>
                <li data-i18n="view.s6330.coord.bankruptcy">Bankruptcy filing: automatic stay (§ 362) suspends collection + CDP</li>
            </ul>
        </div>
    `;
    document.getElementById('s6330-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.final_notice_of_intent_received = !!fd.get('final_notice_of_intent_received');
        state.cp90_letter_1058_received = !!fd.get('cp90_letter_1058_received');
        state.notice_date = fd.get('notice_date') || '';
        state.days_since_notice = Number(fd.get('days_since_notice')) || 0;
        state.s6330_30_day_window = Number(fd.get('s6330_30_day_window')) || 0;
        state.cdp_hearing_request_filed = !!fd.get('cdp_hearing_request_filed');
        state.form_12153_filed = !!fd.get('form_12153_filed');
        state.is_timely_filing = !!fd.get('is_timely_filing');
        state.s6330_a_levy_proposed = !!fd.get('s6330_a_levy_proposed');
        state.levy_property_type = fd.get('levy_property_type');
        state.s6330_b_independent_appeals_officer = !!fd.get('s6330_b_independent_appeals_officer');
        state.appeals_officer_neutral = !!fd.get('appeals_officer_neutral');
        state.s6330_c_hearing_matters = !!fd.get('s6330_c_hearing_matters');
        state.challenge_underlying_liability = !!fd.get('challenge_underlying_liability');
        state.received_prior_opportunity_to_challenge = !!fd.get('received_prior_opportunity_to_challenge');
        state.s6330_c_2_b_collection_alternatives = !!fd.get('s6330_c_2_b_collection_alternatives');
        state.requested_installment_agreement = !!fd.get('requested_installment_agreement');
        state.requested_offer_in_compromise = !!fd.get('requested_offer_in_compromise');
        state.requested_currently_not_collectible = !!fd.get('requested_currently_not_collectible');
        state.spousal_defenses = !!fd.get('spousal_defenses');
        state.s6015_innocent_spouse_raised = !!fd.get('s6015_innocent_spouse_raised');
        state.s6330_d_tax_court_jurisdiction = !!fd.get('s6330_d_tax_court_jurisdiction');
        state.notice_of_determination_received = !!fd.get('notice_of_determination_received');
        state.petition_filed_tax_court = !!fd.get('petition_filed_tax_court');
        state.days_to_petition_30 = Number(fd.get('days_to_petition_30')) || 0;
        state.s7345_passport_revocation_overlap = !!fd.get('s7345_passport_revocation_overlap');
        state.s6320_lien_cdp_already_filed = !!fd.get('s6320_lien_cdp_already_filed');
        state.s6330_e_suspension_collection = !!fd.get('s6330_e_suspension_collection');
        state.s6330_e_2_levy_suspended = !!fd.get('s6330_e_2_levy_suspended');
        state.collection_sol_tolled = !!fd.get('collection_sol_tolled');
        state.s6502_collection_sol_10_year = Number(fd.get('s6502_collection_sol_10_year')) || 0;
        state.days_tolling = Number(fd.get('days_tolling')) || 0;
        state.equivalent_hearing_requested = !!fd.get('equivalent_hearing_requested');
        state.is_jeopardy_levy = !!fd.get('is_jeopardy_levy');
        state.s6331_b_jeopardy_assessment = !!fd.get('s6331_b_jeopardy_assessment');
        state.s7429_administrative_review = !!fd.get('s7429_administrative_review');
        state.abuse_of_discretion_standard = !!fd.get('abuse_of_discretion_standard');
        state.de_novo_review_underlying = !!fd.get('de_novo_review_underlying');
        state.settled_offer_compromise = !!fd.get('settled_offer_compromise');
        state.settled_installment_amount = Number(fd.get('settled_installment_amount')) || 0;
        state.audit_reconsideration_pending = !!fd.get('audit_reconsideration_pending');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6330-output');
    if (!el) return;
    const timely = state.days_since_notice > 0 && state.days_since_notice <= 30;
    const suspended = state.cdp_hearing_request_filed && timely;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6330.h2.result">§ 6330 CDP status</h2>
            <div class="cards">
                <div class="card ${state.final_notice_of_intent_received ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s6330.card.notice">Final notice?</div><div class="value">${state.final_notice_of_intent_received ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6330.card.days">Days since</div><div class="value">${state.days_since_notice}</div></div>
                <div class="card ${timely ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6330.card.timely">Timely?</div><div class="value">${timely ? 'YES (≤30d)' : 'NO (equiv only)'}</div></div>
                <div class="card ${suspended ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6330.card.suspended">Levy suspended?</div><div class="value">${suspended ? 'YES' : 'NO'}</div></div>
                <div class="card ${state.s6330_d_tax_court_jurisdiction ? 'pos' : ''}"><div class="label" data-i18n="view.s6330.card.tc">Tax Court right?</div><div class="value">${state.s6330_d_tax_court_jurisdiction ? 'YES' : 'NO (equiv hrg)'}</div></div>
            </div>
        </div>
    `;
}
