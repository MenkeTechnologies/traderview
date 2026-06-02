// IRC § 6112 — Material Advisor List Maintenance.
// Material advisor must maintain list of advisees + reportable transactions for 7 years.
// Must produce within 20 business days of IRS written request.
// § 6708 penalty: $10,000/day after day 21 (NO CAP).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_material_advisor: false,
    transactions_advised_count: 0,
    list_maintained: false,
    list_contents_complete: false,
    s6112_list_retention_years: 7,
    list_age_years: 0,
    irs_request_received: false,
    irs_request_date: '',
    days_since_request: 0,
    business_days_to_production: 20,
    days_late_production: 0,
    list_produced: false,
    list_produced_partially: false,
    rt_category: 'listed',
    list_partial_production: 0,
    s6708_penalty_per_day: 10000,
    s6708_penalty_total: 0,
    intentional_disregard: false,
    cooperative_with_irs: true,
    s6112_b_required_information: 'all',
    s7525_practitioner_privilege: false,
    attorney_client_privilege_claimed: false,
    privilege_log_provided: false,
    s7421_anti_injunction_lifted: false,
    s7408_promoter_injunction: false,
    s6700_promoter_penalty: 0,
    cumulative_advisee_count: 0,
    advisee_tax_benefit_total: 0,
    s7402_summons_issued: false,
    s7609_summons_intervention: false,
    voluntary_compliance: false,
    reasonable_cause_defense: false,
};

export async function renderSection6112(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6112.h1.title">// § 6112 MATERIAL ADVISOR LIST MAINTENANCE</span></h1>
        <p class="muted small" data-i18n="view.s6112.hint.intro">
            <strong>§ 6112</strong> + Reg § 301.6112-1 require material advisor to MAINTAIN list
            of each advisee + reportable transaction. <strong>Required:</strong> name, address, TIN,
            amount invested, copy of advice, all fees, dates of involvement. <strong>Retention:</strong>
            7 YEARS from date last became material advisor. <strong>Production:</strong> within
            <strong>20 BUSINESS DAYS</strong> of IRS written request (Reg § 301.6112-1(e)).
            <strong>§ 6708 penalty:</strong> $10,000 PER DAY starting day 21 after request —
            <strong>NO CAP</strong>. <strong>Coordination:</strong> § 6111 Form 8918 advisor
            disclosure + § 6707A taxpayer Form 8886 + § 7408 injunction + § 7402 summons.
            <strong>§ 7525 limited practitioner privilege</strong> does NOT cover tax shelter advice.
            <strong>Voluntary compliance reduces penalty</strong>; intentional disregard accelerates
            § 7408 injunction relief.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6112.h2.inputs">Inputs</h2>
            <form id="s6112-form" class="inline-form">
                <label><span data-i18n="view.s6112.label.material">Material advisor?</span>
                    <input type="checkbox" name="is_material_advisor" ${state.is_material_advisor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.count">Transactions advised</span>
                    <input type="number" step="1" name="transactions_advised_count" value="${state.transactions_advised_count}"></label>
                <label><span data-i18n="view.s6112.label.maintained">List maintained?</span>
                    <input type="checkbox" name="list_maintained" ${state.list_maintained ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.complete">Contents complete?</span>
                    <input type="checkbox" name="list_contents_complete" ${state.list_contents_complete ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.retention">Retention years</span>
                    <input type="number" step="1" name="s6112_list_retention_years" value="${state.s6112_list_retention_years}"></label>
                <label><span data-i18n="view.s6112.label.age">Age (years)</span>
                    <input type="number" step="1" name="list_age_years" value="${state.list_age_years}"></label>
                <label><span data-i18n="view.s6112.label.request">IRS request received?</span>
                    <input type="checkbox" name="irs_request_received" ${state.irs_request_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.req_date">IRS request date</span>
                    <input type="date" name="irs_request_date" value="${state.irs_request_date}"></label>
                <label><span data-i18n="view.s6112.label.days_since">Days since request</span>
                    <input type="number" step="1" name="days_since_request" value="${state.days_since_request}"></label>
                <label><span data-i18n="view.s6112.label.biz_days">Biz days to produce</span>
                    <input type="number" step="1" name="business_days_to_production" value="${state.business_days_to_production}"></label>
                <label><span data-i18n="view.s6112.label.late">Days late production</span>
                    <input type="number" step="1" name="days_late_production" value="${state.days_late_production}"></label>
                <label><span data-i18n="view.s6112.label.produced">List produced?</span>
                    <input type="checkbox" name="list_produced" ${state.list_produced ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.partial">Partial production?</span>
                    <input type="checkbox" name="list_produced_partially" ${state.list_produced_partially ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.category">RT category</span>
                    <select name="rt_category">
                        <option value="listed" ${state.rt_category === 'listed' ? 'selected' : ''}>Listed</option>
                        <option value="confidential" ${state.rt_category === 'confidential' ? 'selected' : ''}>Confidential</option>
                        <option value="contractual" ${state.rt_category === 'contractual' ? 'selected' : ''}>Contractual protection</option>
                        <option value="loss" ${state.rt_category === 'loss' ? 'selected' : ''}>Loss transaction</option>
                        <option value="toi" ${state.rt_category === 'toi' ? 'selected' : ''}>Transaction of interest</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6112.label.partial_amt">Partial production</span>
                    <input type="number" step="1" name="list_partial_production" value="${state.list_partial_production}"></label>
                <label><span data-i18n="view.s6112.label.per_day">$ per day</span>
                    <input type="number" step="100" name="s6708_penalty_per_day" value="${state.s6708_penalty_per_day}"></label>
                <label><span data-i18n="view.s6112.label.total">Total penalty ($)</span>
                    <input type="number" step="1000" name="s6708_penalty_total" value="${state.s6708_penalty_total}"></label>
                <label><span data-i18n="view.s6112.label.intentional">Intentional disregard?</span>
                    <input type="checkbox" name="intentional_disregard" ${state.intentional_disregard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.coop">Cooperative?</span>
                    <input type="checkbox" name="cooperative_with_irs" ${state.cooperative_with_irs ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.required">Required info</span>
                    <select name="s6112_b_required_information">
                        <option value="all" ${state.s6112_b_required_information === 'all' ? 'selected' : ''}>All required</option>
                        <option value="partial" ${state.s6112_b_required_information === 'partial' ? 'selected' : ''}>Partial</option>
                        <option value="missing" ${state.s6112_b_required_information === 'missing' ? 'selected' : ''}>Missing key items</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6112.label.s7525">§ 7525 privilege?</span>
                    <input type="checkbox" name="s7525_practitioner_privilege" ${state.s7525_practitioner_privilege ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.privilege">Attorney-client claimed?</span>
                    <input type="checkbox" name="attorney_client_privilege_claimed" ${state.attorney_client_privilege_claimed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.priv_log">Privilege log provided?</span>
                    <input type="checkbox" name="privilege_log_provided" ${state.privilege_log_provided ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.s7421">§ 7421 anti-injunction lifted?</span>
                    <input type="checkbox" name="s7421_anti_injunction_lifted" ${state.s7421_anti_injunction_lifted ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.s7408">§ 7408 injunction?</span>
                    <input type="checkbox" name="s7408_promoter_injunction" ${state.s7408_promoter_injunction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.s6700">§ 6700 promoter ($)</span>
                    <input type="number" step="1000" name="s6700_promoter_penalty" value="${state.s6700_promoter_penalty}"></label>
                <label><span data-i18n="view.s6112.label.advisees">Advisee count</span>
                    <input type="number" step="1" name="cumulative_advisee_count" value="${state.cumulative_advisee_count}"></label>
                <label><span data-i18n="view.s6112.label.benefit">Advisee benefit ($)</span>
                    <input type="number" step="10000" name="advisee_tax_benefit_total" value="${state.advisee_tax_benefit_total}"></label>
                <label><span data-i18n="view.s6112.label.s7402">§ 7402 summons issued?</span>
                    <input type="checkbox" name="s7402_summons_issued" ${state.s7402_summons_issued ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.s7609">§ 7609 intervention?</span>
                    <input type="checkbox" name="s7609_summons_intervention" ${state.s7609_summons_intervention ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.voluntary">Voluntary compliance?</span>
                    <input type="checkbox" name="voluntary_compliance" ${state.voluntary_compliance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6112.label.cause">Reasonable cause?</span>
                    <input type="checkbox" name="reasonable_cause_defense" ${state.reasonable_cause_defense ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6112.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6112-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6112.h2.list_contents">§ 6112 list required contents</h2>
            <ol class="muted small">
                <li data-i18n="view.s6112.contents.name">Name + address of each advisee</li>
                <li data-i18n="view.s6112.contents.tin">TIN (SSN, EIN, ITIN)</li>
                <li data-i18n="view.s6112.contents.invested">Amount invested in transaction</li>
                <li data-i18n="view.s6112.contents.advice">COPY of all written advice provided</li>
                <li data-i18n="view.s6112.contents.fees">All fees received from each advisee</li>
                <li data-i18n="view.s6112.contents.dates">Dates of involvement</li>
                <li data-i18n="view.s6112.contents.description">Description of transaction + tax benefit</li>
                <li data-i18n="view.s6112.contents.s6111_reference">Reference to Form 8918 (§ 6111) filing</li>
                <li data-i18n="view.s6112.contents.format">Electronic format permitted (Reg § 301.6112-1(b)(3))</li>
                <li data-i18n="view.s6112.contents.separate">Separate list for each REPORTABLE TRANSACTION category</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6112.h2.production">Production timeline</h2>
            <ul class="muted small">
                <li data-i18n="view.s6112.prod.request">IRS written request from Office of Tax Shelter Analysis (OTSA)</li>
                <li data-i18n="view.s6112.prod.20_days">20 BUSINESS DAYS to produce (Reg § 301.6112-1(e))</li>
                <li data-i18n="view.s6112.prod.day_21">Day 21: § 6708 $10K/day penalty STARTS</li>
                <li data-i18n="view.s6112.prod.no_cap">NO CAP on penalty — can exceed $1M easily</li>
                <li data-i18n="view.s6112.prod.partial">Partial production: count missing names</li>
                <li data-i18n="view.s6112.prod.attorney_general">Attorney general may also pursue § 7402 summons</li>
                <li data-i18n="view.s6112.prod.s7609">§ 7609 third-party summons: advisees may intervene</li>
                <li data-i18n="view.s6112.prod.s6708_b">§ 6708(b) — reasonable cause + good-faith effort defense available</li>
                <li data-i18n="view.s6112.prod.compliance">Voluntary delivery within 21 days completely avoids penalty</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6112.h2.privilege">Privilege concerns</h2>
            <ul class="muted small">
                <li data-i18n="view.s6112.priv.s7525">§ 7525 federally authorized tax practitioner privilege (limited)</li>
                <li data-i18n="view.s6112.priv.s7525_tax_shelter">§ 7525 does NOT cover written tax shelter promotion</li>
                <li data-i18n="view.s6112.priv.attorney_client">Attorney-client: protects legal advice, NOT lists of clients</li>
                <li data-i18n="view.s6112.priv.crime_fraud">Crime-fraud exception: if used in furtherance of fraud (Notice 2009-7)</li>
                <li data-i18n="view.s6112.priv.work_product">Work product: NOT shielded for client lists or fee disclosures</li>
                <li data-i18n="view.s6112.priv.privilege_log">Must provide privilege log + specific identification</li>
                <li data-i18n="view.s6112.priv.in_camera">In camera review by judge possible (limited scope)</li>
                <li data-i18n="view.s6112.priv.identifying">Identifying clients NOT privileged in most circuits (Fisher v. US)</li>
                <li data-i18n="view.s6112.priv.no_privilege_for_fees">Fee disclosures: NOT privileged (Reiserer)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6112.h2.coordination">§ 6112 coordination with other provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6112.coord.s6111">§ 6111 Form 8918 — material advisor disclosure (separate)</li>
                <li data-i18n="view.s6112.coord.s6707">§ 6707 — material advisor penalty for failure to file 8918</li>
                <li data-i18n="view.s6112.coord.s6708">§ 6708 — list maintenance penalty ($10K/day)</li>
                <li data-i18n="view.s6112.coord.s6700">§ 6700 — promoter penalty (organizing abusive shelter)</li>
                <li data-i18n="view.s6112.coord.s6701">§ 6701 — aiding understatement of liability ($1K-$10K per occurrence)</li>
                <li data-i18n="view.s6112.coord.s7408">§ 7408 — injunction against promoter / material advisor</li>
                <li data-i18n="view.s6112.coord.s7402">§ 7402 — summons enforcement actions</li>
                <li data-i18n="view.s6112.coord.s7609">§ 7609 — third-party summons procedures</li>
                <li data-i18n="view.s6112.coord.opr">OPR Circular 230 disciplinary proceedings — license sanctions</li>
                <li data-i18n="view.s6112.coord.criminal">§ 7203 / § 7206 criminal prosecution potential</li>
            </ul>
        </div>
    `;
    document.getElementById('s6112-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_material_advisor = !!fd.get('is_material_advisor');
        state.transactions_advised_count = Number(fd.get('transactions_advised_count')) || 0;
        state.list_maintained = !!fd.get('list_maintained');
        state.list_contents_complete = !!fd.get('list_contents_complete');
        state.s6112_list_retention_years = Number(fd.get('s6112_list_retention_years')) || 0;
        state.list_age_years = Number(fd.get('list_age_years')) || 0;
        state.irs_request_received = !!fd.get('irs_request_received');
        state.irs_request_date = fd.get('irs_request_date') || '';
        state.days_since_request = Number(fd.get('days_since_request')) || 0;
        state.business_days_to_production = Number(fd.get('business_days_to_production')) || 0;
        state.days_late_production = Number(fd.get('days_late_production')) || 0;
        state.list_produced = !!fd.get('list_produced');
        state.list_produced_partially = !!fd.get('list_produced_partially');
        state.rt_category = fd.get('rt_category');
        state.list_partial_production = Number(fd.get('list_partial_production')) || 0;
        state.s6708_penalty_per_day = Number(fd.get('s6708_penalty_per_day')) || 0;
        state.s6708_penalty_total = Number(fd.get('s6708_penalty_total')) || 0;
        state.intentional_disregard = !!fd.get('intentional_disregard');
        state.cooperative_with_irs = !!fd.get('cooperative_with_irs');
        state.s6112_b_required_information = fd.get('s6112_b_required_information');
        state.s7525_practitioner_privilege = !!fd.get('s7525_practitioner_privilege');
        state.attorney_client_privilege_claimed = !!fd.get('attorney_client_privilege_claimed');
        state.privilege_log_provided = !!fd.get('privilege_log_provided');
        state.s7421_anti_injunction_lifted = !!fd.get('s7421_anti_injunction_lifted');
        state.s7408_promoter_injunction = !!fd.get('s7408_promoter_injunction');
        state.s6700_promoter_penalty = Number(fd.get('s6700_promoter_penalty')) || 0;
        state.cumulative_advisee_count = Number(fd.get('cumulative_advisee_count')) || 0;
        state.advisee_tax_benefit_total = Number(fd.get('advisee_tax_benefit_total')) || 0;
        state.s7402_summons_issued = !!fd.get('s7402_summons_issued');
        state.s7609_summons_intervention = !!fd.get('s7609_summons_intervention');
        state.voluntary_compliance = !!fd.get('voluntary_compliance');
        state.reasonable_cause_defense = !!fd.get('reasonable_cause_defense');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6112-output');
    if (!el) return;
    const days_in_violation = Math.max(0, state.days_late_production);
    const penalty = !state.reasonable_cause_defense ? days_in_violation * state.s6708_penalty_per_day : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6112.h2.result">§ 6708 penalty assessment</h2>
            <div class="cards">
                <div class="card ${state.is_material_advisor ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s6112.card.material">Material advisor?</div><div class="value">${state.is_material_advisor ? 'YES' : 'NO'}</div></div>
                <div class="card ${state.list_maintained ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6112.card.list">List maintained?</div><div class="value">${state.list_maintained ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6112.card.days">Days in violation</div><div class="value">${days_in_violation}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s6112.card.penalty">§ 6708 penalty</div><div class="value">$${penalty.toLocaleString()}</div></div>
                <div class="card warn"><div class="label" data-i18n="view.s6112.card.no_cap">No cap</div><div class="value">Cumulative</div></div>
            </div>
        </div>
    `;
}
