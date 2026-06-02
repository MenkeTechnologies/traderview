// IRC § 7811 — Taxpayer Assistance Order (TAO) via Taxpayer Advocate Service (TAS).
// TAO orders IRS to release levy, return seized property, expedite refund.
// TAS gets involved when taxpayer faces "significant hardship" — economic, irreparable injury,
// long delays, lack of system response. Form 911 or 1-877-777-4778.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    economic_hardship: false,
    immediate_threat: false,
    long_irs_delay_days: 0,
    no_response_to_inquiries: false,
    system_failure: false,
    health_threat: false,
    eviction_foreclosure_imminent: false,
    family_significant_negative: false,
    grievance_type: 'levy_release',
    estimated_loss_amount: 0,
    multiple_irs_contacts: false,
};

export async function renderSection7811(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7811.h1.title">// § 7811 TAXPAYER ASSISTANCE ORDER</span></h1>
        <p class="muted small" data-i18n="view.s7811.hint.intro">
            Taxpayer Advocate Service (TAS) issues TAO ordering IRS action when "significant
            hardship" exists. <strong>4 criteria:</strong> (1) immediate threat of adverse action,
            (2) delay &gt; 30 days, (3) cost (including professional fees), (4) irreparable injury
            / long-term adverse impact. Free service. <strong>Form 911</strong> or
            <strong>1-877-777-4778</strong>. TAS can override IRS Operations Manual.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7811.h2.inputs">Inputs</h2>
            <form id="s7811-form" class="inline-form">
                <label><span data-i18n="view.s7811.label.economic">Economic hardship?</span>
                    <input type="checkbox" name="economic_hardship" ${state.economic_hardship ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7811.label.threat">Immediate threat of adverse action?</span>
                    <input type="checkbox" name="immediate_threat" ${state.immediate_threat ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7811.label.delay">IRS delay (days)</span>
                    <input type="number" step="1" name="long_irs_delay_days" value="${state.long_irs_delay_days}"></label>
                <label><span data-i18n="view.s7811.label.no_response">No response to inquiries?</span>
                    <input type="checkbox" name="no_response_to_inquiries" ${state.no_response_to_inquiries ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7811.label.system">IRS system failure?</span>
                    <input type="checkbox" name="system_failure" ${state.system_failure ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7811.label.health">Health threat?</span>
                    <input type="checkbox" name="health_threat" ${state.health_threat ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7811.label.eviction">Eviction / foreclosure imminent?</span>
                    <input type="checkbox" name="eviction_foreclosure_imminent" ${state.eviction_foreclosure_imminent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7811.label.family">Significant family / business impact?</span>
                    <input type="checkbox" name="family_significant_negative" ${state.family_significant_negative ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7811.label.grievance">Grievance type</span>
                    <select name="grievance_type">
                        <option value="levy_release">Release of levy / wage garnishment</option>
                        <option value="lien_subordination">Lien subordination / discharge</option>
                        <option value="refund_delay">Refund delayed</option>
                        <option value="audit_delay">Audit / examination delay</option>
                        <option value="processing_error">Processing error</option>
                        <option value="balance_due_dispute">Balance due dispute</option>
                        <option value="identity_theft">Identity theft</option>
                        <option value="property_return">Return of seized property</option>
                        <option value="oic_processing">OIC processing delay</option>
                        <option value="other">Other systemic issue</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7811.label.loss">Estimated loss amount ($)</span>
                    <input type="number" step="1000" name="estimated_loss_amount" value="${state.estimated_loss_amount}"></label>
                <label><span data-i18n="view.s7811.label.multiple">Multiple IRS contacts tried?</span>
                    <input type="checkbox" name="multiple_irs_contacts" ${state.multiple_irs_contacts ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s7811.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7811-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7811.h2.tas_help">TAS can help with</h2>
            <ul class="muted small">
                <li data-i18n="view.s7811.tas.levy">Release wage / bank levies</li>
                <li data-i18n="view.s7811.tas.lien">Subordinate / withdraw federal tax liens</li>
                <li data-i18n="view.s7811.tas.refund">Expedite refunds in legitimate hardship</li>
                <li data-i18n="view.s7811.tas.return_property">Return seized property / vehicles</li>
                <li data-i18n="view.s7811.tas.fix_processing">Fix processing errors + restore correct records</li>
                <li data-i18n="view.s7811.tas.id_theft">Identity theft resolution + IP PIN</li>
                <li data-i18n="view.s7811.tas.systemic">Systemic Advocacy: address recurring IRS process failures</li>
                <li data-i18n="view.s7811.tas.research_change">Research + policy change recommendations via Annual Report to Congress</li>
                <li data-i18n="view.s7811.tas.legal_advice">NOT legal advice or tax planning</li>
                <li data-i18n="view.s7811.tas.appeals">NOT formal appeals processing</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7811.h2.tao_powers">TAO powers + limits</h2>
            <ul class="muted small">
                <li data-i18n="view.s7811.tao.compel">Compels IRS to act (release levy, issue refund, etc.) within specified time</li>
                <li data-i18n="view.s7811.tao.override">Can override IRM but NOT statutory mandates</li>
                <li data-i18n="view.s7811.tao.appeal">IRS can appeal to TAS Deputy Chief; TAO stays in effect during appeal</li>
                <li data-i18n="view.s7811.tao.can_recommend">Cannot recommend tax position; only procedural relief</li>
                <li data-i18n="view.s7811.tao.no_court">No automatic court enforcement (TAS is administrative)</li>
                <li data-i18n="view.s7811.tao.timeline">Acknowledgment within 1 business day; case manager within 5 days</li>
                <li data-i18n="view.s7811.tao.relief_types">Common relief: levy release within 24 hrs in true emergencies</li>
            </ul>
        </div>
    `;
    document.getElementById('s7811-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.economic_hardship = !!fd.get('economic_hardship');
        state.immediate_threat = !!fd.get('immediate_threat');
        state.long_irs_delay_days = Number(fd.get('long_irs_delay_days')) || 0;
        state.no_response_to_inquiries = !!fd.get('no_response_to_inquiries');
        state.system_failure = !!fd.get('system_failure');
        state.health_threat = !!fd.get('health_threat');
        state.eviction_foreclosure_imminent = !!fd.get('eviction_foreclosure_imminent');
        state.family_significant_negative = !!fd.get('family_significant_negative');
        state.grievance_type = fd.get('grievance_type');
        state.estimated_loss_amount = Number(fd.get('estimated_loss_amount')) || 0;
        state.multiple_irs_contacts = !!fd.get('multiple_irs_contacts');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7811-output');
    if (!el) return;
    let score = 0;
    if (state.economic_hardship) score += 3;
    if (state.immediate_threat) score += 3;
    if (state.eviction_foreclosure_imminent) score += 3;
    if (state.health_threat) score += 3;
    if (state.long_irs_delay_days > 30) score += 2;
    if (state.no_response_to_inquiries) score += 2;
    if (state.system_failure) score += 2;
    if (state.family_significant_negative) score += 1;
    if (state.multiple_irs_contacts) score += 1;
    let qualifies, priority;
    if (score >= 8) { qualifies = true; priority = 'view.s7811.priority.urgent'; }
    else if (score >= 5) { qualifies = true; priority = 'view.s7811.priority.high'; }
    else if (score >= 2) { qualifies = true; priority = 'view.s7811.priority.normal'; }
    else { qualifies = false; priority = 'view.s7811.priority.low'; }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7811.h2.result">TAS eligibility</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s7811.card.score">Hardship score</div>
                    <div class="value">${score}</div>
                </div>
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7811.card.qualifies">Likely qualifies for TAS</div>
                    <div class="value">${qualifies ? esc(t('view.s7811.status.yes')) : esc(t('view.s7811.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7811.card.priority">Priority level</div>
                    <div class="value">${esc(t(priority))}</div>
                </div>
            </div>
            ${qualifies ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s7811.note.file_now">
                    File Form 911 (Application for Taxpayer Assistance Order) or call
                    1-877-777-4778. Free service. Acknowledgment within 1 business day; case
                    assigned within 5 days. Bring all IRS correspondence + documentation of hardship.
                </p>
            ` : ''}
        </div>
    `;
}
