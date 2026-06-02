// IRC § 7491 — Burden of Proof in Tax Court Proceedings.
// IRS Restructuring + Reform Act 1998: shifted burden to IRS in certain cases.
// Burden shifts: (1) taxpayer satisfies record-keeping, (2) cooperates with IRS, (3) reasonable case provided to IRS in administrative review.
// EXCEPTIONS: criminal cases, foreign-source income, partner audit issues, controlled foreign corp issues.
// § 7491(c): IRS bears burden on PENALTY of $1M+ income (and certain dollar-defined penalties).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    case_type: 'income_tax_deficiency',
    record_keeping_satisfied: true,
    cooperated_with_irs: true,
    reasonable_case_provided: true,
    is_foreign_source: false,
    is_partnership_audit: false,
    is_cfc_issue: false,
    is_criminal: false,
    penalty_amount_at_issue: 0,
    is_individual_under_1m_agi: false,
    is_corporation: false,
    net_worth_under_7m: false,
    employees_under_500: false,
    is_high_income: false,
    statutory_presumption: false,
    factual_dispute_only: true,
    summary_judgment_pending: false,
};

export async function renderSection7491(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7491.h1.title">// § 7491 BURDEN OF PROOF</span></h1>
        <p class="muted small" data-i18n="view.s7491.hint.intro">
            <strong>IRS Restructuring + Reform Act 1998:</strong> shifted burden to IRS in certain cases.
            <strong>Burden shifts</strong> if: (1) taxpayer satisfies <strong>RECORD-KEEPING</strong>,
            (2) <strong>COOPERATES</strong> with IRS, (3) provides reasonable case to IRS administratively.
            <strong>EXCEPTIONS:</strong> criminal cases, foreign-source income, partnership audit issues,
            controlled foreign corp issues. <strong>§ 7491(c):</strong> IRS bears burden on PENALTY for
            $1M+ individual income (and certain dollar-defined penalties). <strong>§ 7491(a)(2)(A):</strong>
            individual + net worth requirements for full shift.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7491.h2.inputs">Inputs</h2>
            <form id="s7491-form" class="inline-form">
                <label><span data-i18n="view.s7491.label.case_type">Case type</span>
                    <select name="case_type">
                        <option value="income_tax_deficiency" ${state.case_type === 'income_tax_deficiency' ? 'selected' : ''}>Income tax deficiency</option>
                        <option value="penalty_dispute" ${state.case_type === 'penalty_dispute' ? 'selected' : ''}>Penalty dispute</option>
                        <option value="refund_suit" ${state.case_type === 'refund_suit' ? 'selected' : ''}>Refund suit (district court)</option>
                        <option value="injunction" ${state.case_type === 'injunction' ? 'selected' : ''}>Injunction relief</option>
                        <option value="cdp_hearing" ${state.case_type === 'cdp_hearing' ? 'selected' : ''}>CDP hearing</option>
                        <option value="innocent_spouse" ${state.case_type === 'innocent_spouse' ? 'selected' : ''}>Innocent spouse</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7491.label.records">Record-keeping satisfied?</span>
                    <input type="checkbox" name="record_keeping_satisfied" ${state.record_keeping_satisfied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.cooperated">Cooperated with IRS?</span>
                    <input type="checkbox" name="cooperated_with_irs" ${state.cooperated_with_irs ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.reasonable">Reasonable case provided?</span>
                    <input type="checkbox" name="reasonable_case_provided" ${state.reasonable_case_provided ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.foreign">Foreign-source income?</span>
                    <input type="checkbox" name="is_foreign_source" ${state.is_foreign_source ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.partnership">Partnership audit?</span>
                    <input type="checkbox" name="is_partnership_audit" ${state.is_partnership_audit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.cfc">CFC issue?</span>
                    <input type="checkbox" name="is_cfc_issue" ${state.is_cfc_issue ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.criminal">Criminal case?</span>
                    <input type="checkbox" name="is_criminal" ${state.is_criminal ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.penalty">Penalty amount at issue ($)</span>
                    <input type="number" step="10000" name="penalty_amount_at_issue" value="${state.penalty_amount_at_issue}"></label>
                <label><span data-i18n="view.s7491.label.individual">Individual under $1M AGI?</span>
                    <input type="checkbox" name="is_individual_under_1m_agi" ${state.is_individual_under_1m_agi ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.corp">Corporation?</span>
                    <input type="checkbox" name="is_corporation" ${state.is_corporation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.net_worth">Net worth under $7M (corp)?</span>
                    <input type="checkbox" name="net_worth_under_7m" ${state.net_worth_under_7m ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.employees">≤ 500 employees?</span>
                    <input type="checkbox" name="employees_under_500" ${state.employees_under_500 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.high">High-income taxpayer?</span>
                    <input type="checkbox" name="is_high_income" ${state.is_high_income ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.statutory">Statutory presumption applies?</span>
                    <input type="checkbox" name="statutory_presumption" ${state.statutory_presumption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.factual">Factual dispute only?</span>
                    <input type="checkbox" name="factual_dispute_only" ${state.factual_dispute_only ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7491.label.summary">Summary judgment pending?</span>
                    <input type="checkbox" name="summary_judgment_pending" ${state.summary_judgment_pending ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s7491.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7491-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7491.h2.requirements">§ 7491(a) shift requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s7491.req.records">RECORD-KEEPING: produced contemporaneous records adequate to substantiate position</li>
                <li data-i18n="view.s7491.req.cooperation">COOPERATION: with reasonable IRS information requests + substantiation</li>
                <li data-i18n="view.s7491.req.administrative">ADMINISTRATIVE: provided reasonable case to IRS BEFORE Tax Court (not just first there)</li>
                <li data-i18n="view.s7491.req.factual_issue">FACTUAL ISSUE: applies to factual issues, not legal questions</li>
                <li data-i18n="view.s7491.req.individual_corp">INDIVIDUAL or CORP: limited to individuals + small corps (≤ $7M net worth + ≤ 500 employees)</li>
                <li data-i18n="view.s7491.req.partnership_excluded">PARTNERSHIPS: NOT eligible (separate audit regime)</li>
                <li data-i18n="view.s7491.req.estate_trust">ESTATES + TRUSTS: not eligible</li>
                <li data-i18n="view.s7491.req.cooperative_meaning">"Cooperative" = same as discovery + reasonable response time</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7491.h2.exceptions">§ 7491 exceptions (burden stays with taxpayer)</h2>
            <ul class="muted small">
                <li data-i18n="view.s7491.exc.foreign_source">Foreign-source income / treaty issues</li>
                <li data-i18n="view.s7491.exc.partnership_TEFRA">TEFRA / BBA partnership audits</li>
                <li data-i18n="view.s7491.exc.cfc">Controlled Foreign Corp / subpart F issues</li>
                <li data-i18n="view.s7491.exc.criminal">Criminal proceedings (Boyle case)</li>
                <li data-i18n="view.s7491.exc.civil_fraud">§ 6663 civil fraud (75% penalty) — IRS always has burden</li>
                <li data-i18n="view.s7491.exc.transferee">Transferee liability</li>
                <li data-i18n="view.s7491.exc.refund_suit">Refund suit (district court / claims court) — burden on plaintiff</li>
                <li data-i18n="view.s7491.exc.injunction">Injunction proceedings against IRS — different standards</li>
                <li data-i18n="view.s7491.exc.summary">Summary judgment: burden under FRCP 56 standards</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7491.h2.s7491c">§ 7491(c) penalty burden (1998 reform)</h2>
            <ul class="muted small">
                <li data-i18n="view.s7491.c.basic">IRS bears burden of production on penalties / additions to tax</li>
                <li data-i18n="view.s7491.c.individual">Individual: applies when penalty ≥ $1M income year</li>
                <li data-i18n="view.s7491.c.s6664">§ 6664(c) reasonable cause exception: taxpayer still bears burden on cure</li>
                <li data-i18n="view.s7491.c.fraud_penalty">§ 6663 fraud: IRS has burden of PROOF (not just production)</li>
                <li data-i18n="view.s7491.c.s6651">§ 6651 failure to file / pay: IRS production only</li>
                <li data-i18n="view.s7491.c.s7491c_2">§ 7491(c)(2): supervisor approval requirement for penalty assertion</li>
                <li data-i18n="view.s7491.c.graev_chai">Graev v. Comm'r (2017) + Chai v. Comm'r (2017): super approval = invalidating defective penalties</li>
                <li data-i18n="view.s7491.c.cure_window">Reasonable cause: taxpayer + facts; IRS gets to rebut</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7491.h2.attorney_fees">§ 7430 attorney's fees recovery</h2>
            <ul class="muted small">
                <li data-i18n="view.s7491.fees.purpose">Recover attorney's fees if IRS position NOT SUBSTANTIALLY JUSTIFIED</li>
                <li data-i18n="view.s7491.fees.prevailing">Taxpayer must PREVAIL on substantive issues</li>
                <li data-i18n="view.s7491.fees.qualifications">Qualifications: small entity + reasonable cost</li>
                <li data-i18n="view.s7491.fees.cap">Cap: $230/hr (2024, indexed) + reasonable hours</li>
                <li data-i18n="view.s7491.fees.tax_court">Tax Court awards fees only after final disposition</li>
                <li data-i18n="view.s7491.fees.district_court">District court: separate § 7430 application</li>
                <li data-i18n="view.s7491.fees.administrative">Administrative proceedings: also covered</li>
                <li data-i18n="view.s7491.fees.section_6402">Coordinate with § 6402 set-off + refund offset rules</li>
            </ul>
        </div>
    `;
    document.getElementById('s7491-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.case_type = fd.get('case_type');
        state.record_keeping_satisfied = !!fd.get('record_keeping_satisfied');
        state.cooperated_with_irs = !!fd.get('cooperated_with_irs');
        state.reasonable_case_provided = !!fd.get('reasonable_case_provided');
        state.is_foreign_source = !!fd.get('is_foreign_source');
        state.is_partnership_audit = !!fd.get('is_partnership_audit');
        state.is_cfc_issue = !!fd.get('is_cfc_issue');
        state.is_criminal = !!fd.get('is_criminal');
        state.penalty_amount_at_issue = Number(fd.get('penalty_amount_at_issue')) || 0;
        state.is_individual_under_1m_agi = !!fd.get('is_individual_under_1m_agi');
        state.is_corporation = !!fd.get('is_corporation');
        state.net_worth_under_7m = !!fd.get('net_worth_under_7m');
        state.employees_under_500 = !!fd.get('employees_under_500');
        state.is_high_income = !!fd.get('is_high_income');
        state.statutory_presumption = !!fd.get('statutory_presumption');
        state.factual_dispute_only = !!fd.get('factual_dispute_only');
        state.summary_judgment_pending = !!fd.get('summary_judgment_pending');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7491-output');
    if (!el) return;
    const meets_requirements = state.record_keeping_satisfied && state.cooperated_with_irs && state.reasonable_case_provided && state.factual_dispute_only;
    const exception_applies = state.is_foreign_source || state.is_partnership_audit || state.is_cfc_issue || state.is_criminal;
    const qualifying_corp = state.is_corporation && state.net_worth_under_7m && state.employees_under_500;
    const eligible_taxpayer = (state.is_individual_under_1m_agi && !state.is_high_income) || qualifying_corp;
    const burden_shifts = meets_requirements && !exception_applies && eligible_taxpayer;
    const penalty_shift = state.penalty_amount_at_issue >= 1_000_000;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7491.h2.result">§ 7491 burden analysis</h2>
            <div class="cards">
                <div class="card ${meets_requirements ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7491.card.requirements">Requirements met?</div>
                    <div class="value">${meets_requirements ? esc(t('view.s7491.status.yes')) : esc(t('view.s7491.status.no'))}</div>
                </div>
                <div class="card ${exception_applies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7491.card.exception">Exception applies?</div>
                    <div class="value">${exception_applies ? esc(t('view.s7491.status.yes')) : esc(t('view.s7491.status.no'))}</div>
                </div>
                <div class="card ${eligible_taxpayer ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7491.card.eligible">Eligible taxpayer?</div>
                    <div class="value">${eligible_taxpayer ? esc(t('view.s7491.status.yes')) : esc(t('view.s7491.status.no'))}</div>
                </div>
                <div class="card ${burden_shifts ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7491.card.shift">Burden shifts to IRS?</div>
                    <div class="value">${burden_shifts ? esc(t('view.s7491.status.yes')) : esc(t('view.s7491.status.no'))}</div>
                </div>
                <div class="card ${penalty_shift ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7491.card.penalty">Penalty burden shift (§ 7491(c))?</div>
                    <div class="value">${penalty_shift ? esc(t('view.s7491.status.yes')) : esc(t('view.s7491.status.no'))}</div>
                </div>
            </div>
            ${burden_shifts ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s7491.shifted_note">
                    BURDEN SHIFTED to IRS. Strong tactical advantage in Tax Court. Affidavits + business
                    records become powerful evidence. IRS examiner now must prove deficiency rather than
                    taxpayer disproving it. Combine with § 7430 attorney's fees recovery if IRS position
                    not substantially justified.
                </p>
            ` : ''}
        </div>
    `;
}
