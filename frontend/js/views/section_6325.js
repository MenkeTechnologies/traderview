// IRC § 6325 — Release of Federal Tax Lien.
// Lien released when liability satisfied OR uncollectible OR bond posted.
// § 6325(a)(1): full payment / accepted offer in compromise.
// § 6325(b): partial release / discharge / subordination of property.
// § 6326: appeal of lien filing.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    tax_liability_total: 0,
    payment_made: 0,
    type_request: 'release_full',
    property_value: 0,
    interest_in_property: 0,
    other_assets_secure: 0,
    other_liens_outstanding: 0,
    bond_posted: 0,
    days_since_request: 0,
    cdp_hearing_requested: false,
    is_self_releasing_lien: false,
    statute_expired: false,
    oic_accepted: false,
    cnc_status: false,
    refile_request: false,
};

export async function renderSection6325(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6325.h1.title">// § 6325 LIEN RELEASE</span></h1>
        <p class="muted small" data-i18n="view.s6325.hint.intro">
            Lien <strong>RELEASED</strong> when: liability <strong>satisfied</strong>, <strong>uncollectible</strong>,
            OR <strong>bond posted</strong>. <strong>§ 6325(a)(1):</strong> full payment / accepted Offer in
            Compromise. <strong>§ 6325(b)(1):</strong> partial release of specific property — value retained ≥
            2× liability. <strong>§ 6325(b)(2):</strong> discharge if government's interest valueless.
            <strong>§ 6325(b)(3):</strong> bond / cash deposit. <strong>§ 6325(d):</strong> subordination for
            refinancing. <strong>§ 6326:</strong> 5-day appeal of erroneous filing.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6325.h2.inputs">Inputs</h2>
            <form id="s6325-form" class="inline-form">
                <label><span data-i18n="view.s6325.label.liability">Tax liability total ($)</span>
                    <input type="number" step="0.01" name="tax_liability_total" value="${state.tax_liability_total}"></label>
                <label><span data-i18n="view.s6325.label.payment">Payment made ($)</span>
                    <input type="number" step="0.01" name="payment_made" value="${state.payment_made}"></label>
                <label><span data-i18n="view.s6325.label.type">Type of request</span>
                    <select name="type_request">
                        <option value="release_full" ${state.type_request === 'release_full' ? 'selected' : ''}>Full release (paid)</option>
                        <option value="discharge_property" ${state.type_request === 'discharge_property' ? 'selected' : ''}>§ 6325(b)(1) discharge specific property</option>
                        <option value="discharge_valueless" ${state.type_request === 'discharge_valueless' ? 'selected' : ''}>§ 6325(b)(2) valueless to govt</option>
                        <option value="bond_release" ${state.type_request === 'bond_release' ? 'selected' : ''}>§ 6325(b)(3) bond / deposit</option>
                        <option value="subordinate" ${state.type_request === 'subordinate' ? 'selected' : ''}>§ 6325(d) subordination (refi)</option>
                        <option value="withdraw" ${state.type_request === 'withdraw' ? 'selected' : ''}>§ 6323(j) withdraw notice</option>
                        <option value="cdp_appeal" ${state.type_request === 'cdp_appeal' ? 'selected' : ''}>§ 6326 / CDP appeal</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6325.label.value">Property value ($)</span>
                    <input type="number" step="0.01" name="property_value" value="${state.property_value}"></label>
                <label><span data-i18n="view.s6325.label.interest">IRS interest in property ($)</span>
                    <input type="number" step="0.01" name="interest_in_property" value="${state.interest_in_property}"></label>
                <label><span data-i18n="view.s6325.label.assets">Other assets securing lien ($)</span>
                    <input type="number" step="0.01" name="other_assets_secure" value="${state.other_assets_secure}"></label>
                <label><span data-i18n="view.s6325.label.other_liens">Other liens outstanding ($)</span>
                    <input type="number" step="0.01" name="other_liens_outstanding" value="${state.other_liens_outstanding}"></label>
                <label><span data-i18n="view.s6325.label.bond">Bond posted ($)</span>
                    <input type="number" step="0.01" name="bond_posted" value="${state.bond_posted}"></label>
                <label><span data-i18n="view.s6325.label.days">Days since request</span>
                    <input type="number" step="1" name="days_since_request" value="${state.days_since_request}"></label>
                <label><span data-i18n="view.s6325.label.cdp">CDP hearing requested?</span>
                    <input type="checkbox" name="cdp_hearing_requested" ${state.cdp_hearing_requested ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6325.label.self">Self-releasing lien?</span>
                    <input type="checkbox" name="is_self_releasing_lien" ${state.is_self_releasing_lien ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6325.label.statute">Statute of limitations expired?</span>
                    <input type="checkbox" name="statute_expired" ${state.statute_expired ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6325.label.oic">OIC accepted?</span>
                    <input type="checkbox" name="oic_accepted" ${state.oic_accepted ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6325.label.cnc">CNC status?</span>
                    <input type="checkbox" name="cnc_status" ${state.cnc_status ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6325.label.refile">Refile request after release?</span>
                    <input type="checkbox" name="refile_request" ${state.refile_request ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6325.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6325-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6325.h2.release_basis">§ 6325(a) full release conditions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6325.r.paid">Full payment of liability + interest + penalties</li>
                <li data-i18n="view.s6325.r.oic">Offer in Compromise accepted (Form 656)</li>
                <li data-i18n="view.s6325.r.uncollectible">CNC (Currently Not Collectible) status + lien hindering collection</li>
                <li data-i18n="view.s6325.r.bonded">Bond posted equal to liability (Form 12153 if appealing)</li>
                <li data-i18n="view.s6325.r.statute">Statute of limitations expired (10-yr collection)</li>
                <li data-i18n="view.s6325.r.timing">Release issued within 30 days of request / qualifying event</li>
                <li data-i18n="view.s6325.r.self_releasing">Self-releasing lien: certain liens release automatically (Form 668(Y))</li>
                <li data-i18n="view.s6325.r.refile">Refile possible if facts change OR new period within statute</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6325.h2.discharge">§ 6325(b) discharge of specific property</h2>
            <ul class="muted small">
                <li data-i18n="view.s6325.d.b1">§ 6325(b)(1): RETAIN value ≥ 2× liability — value safely covers exposure</li>
                <li data-i18n="view.s6325.d.b2">§ 6325(b)(2): VALUELESS to government — no IRS interest in property</li>
                <li data-i18n="view.s6325.d.b3">§ 6325(b)(3): BOND or cash deposit — substitute security</li>
                <li data-i18n="view.s6325.d.b4">§ 6325(b)(4): 30-day notice required for some sales</li>
                <li data-i18n="view.s6325.d.sale">Sale of property: IRS interest follows proceeds (Subordination)</li>
                <li data-i18n="view.s6325.d.refinance">Refi: § 6325(d) subordination allows new lien priority</li>
                <li data-i18n="view.s6325.d.form_14135">Form 14135 application + supporting documents</li>
                <li data-i18n="view.s6325.d.process_time">Typical: 30-60 days to process discharge application</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6325.h2.appeal">Appeal mechanisms</h2>
            <ul class="muted small">
                <li data-i18n="view.s6325.app.s6326">§ 6326: 5 days to appeal erroneous filing — admin review</li>
                <li data-i18n="view.s6325.app.cdp">§ 6320 CDP hearing: 30 days from Notice of Lien Filing</li>
                <li data-i18n="view.s6325.app.equivalent">Equivalent hearing (post-30-day): less powerful but available 1 yr</li>
                <li data-i18n="view.s6325.app.tax_court">Tax Court appeal after Final Notice of Determination</li>
                <li data-i18n="view.s6325.app.collateral">Collateral attack via refund suit limited</li>
                <li data-i18n="view.s6325.app.form_12277">Form 12277 — withdraw notice if benefits taxpayer (compliance / OIC / IA)</li>
                <li data-i18n="view.s6325.app.administrative">Administrative appeals: Office of Appeals review</li>
                <li data-i18n="view.s6325.app.tax_court_lien">Tax Court can issue determination on lien filing appropriate</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6325.h2.related_collection">Related collection-rights provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6325.coll.s6321">§ 6321: tax lien — attaches to all property + rights</li>
                <li data-i18n="view.s6325.coll.s6322">§ 6322: lien arises at time of assessment</li>
                <li data-i18n="view.s6325.coll.s6323">§ 6323: notice of lien — priority + protection for certain creditors</li>
                <li data-i18n="view.s6325.coll.s6324">§ 6324: estate / gift tax lien</li>
                <li data-i18n="view.s6325.coll.s6325">§ 6325: release / discharge (this section)</li>
                <li data-i18n="view.s6325.coll.s6326">§ 6326: appeal of lien filing</li>
                <li data-i18n="view.s6325.coll.s6331">§ 6331: levy</li>
                <li data-i18n="view.s6325.coll.s7403">§ 7403: foreclose lien via lawsuit (DOJ involvement)</li>
            </ul>
        </div>
    `;
    document.getElementById('s6325-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tax_liability_total = Number(fd.get('tax_liability_total')) || 0;
        state.payment_made = Number(fd.get('payment_made')) || 0;
        state.type_request = fd.get('type_request');
        state.property_value = Number(fd.get('property_value')) || 0;
        state.interest_in_property = Number(fd.get('interest_in_property')) || 0;
        state.other_assets_secure = Number(fd.get('other_assets_secure')) || 0;
        state.other_liens_outstanding = Number(fd.get('other_liens_outstanding')) || 0;
        state.bond_posted = Number(fd.get('bond_posted')) || 0;
        state.days_since_request = Number(fd.get('days_since_request')) || 0;
        state.cdp_hearing_requested = !!fd.get('cdp_hearing_requested');
        state.is_self_releasing_lien = !!fd.get('is_self_releasing_lien');
        state.statute_expired = !!fd.get('statute_expired');
        state.oic_accepted = !!fd.get('oic_accepted');
        state.cnc_status = !!fd.get('cnc_status');
        state.refile_request = !!fd.get('refile_request');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6325-output');
    if (!el) return;
    const remaining = Math.max(0, state.tax_liability_total - state.payment_made);
    const release_full_qualifies = remaining === 0 || state.statute_expired || state.oic_accepted;
    const value_after_other_liens = Math.max(0, state.property_value - state.other_liens_outstanding);
    const discharge_b1_qualifies = state.type_request === 'discharge_property' && value_after_other_liens >= 2 * remaining;
    const discharge_b2_qualifies = state.type_request === 'discharge_valueless' && state.interest_in_property === 0;
    const bond_qualifies = state.type_request === 'bond_release' && state.bond_posted >= remaining;
    const eligible = release_full_qualifies || discharge_b1_qualifies || discharge_b2_qualifies || bond_qualifies;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6325.h2.result">§ 6325 outcome</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6325.card.eligible">Eligible for release / discharge?</div>
                    <div class="value">${eligible ? esc(t('view.s6325.status.yes')) : esc(t('view.s6325.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6325.card.remaining">Remaining liability</div>
                    <div class="value">$${remaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${release_full_qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6325.card.full">Full release?</div>
                    <div class="value">${release_full_qualifies ? esc(t('view.s6325.status.yes')) : esc(t('view.s6325.status.no'))}</div>
                </div>
                <div class="card ${discharge_b1_qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6325.card.b1">§ 6325(b)(1) discharge?</div>
                    <div class="value">${discharge_b1_qualifies ? esc(t('view.s6325.status.yes')) : esc(t('view.s6325.status.no'))}</div>
                </div>
                <div class="card ${discharge_b2_qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6325.card.b2">§ 6325(b)(2) discharge?</div>
                    <div class="value">${discharge_b2_qualifies ? esc(t('view.s6325.status.yes')) : esc(t('view.s6325.status.no'))}</div>
                </div>
                <div class="card ${bond_qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6325.card.bond">§ 6325(b)(3) bond?</div>
                    <div class="value">${bond_qualifies ? esc(t('view.s6325.status.yes')) : esc(t('view.s6325.status.no'))}</div>
                </div>
            </div>
            ${state.days_since_request > 30 && !eligible ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6325.delay_note">
                    Request pending &gt; 30 days. Contact IRS Centralized Lien Office (CLO) at 800-913-6050.
                    Form 911 to Taxpayer Advocate Service if hardship. CDP hearing may be available if appeal
                    rights not exhausted. Consider local TAS if facing financial harm.
                </p>
            ` : ''}
        </div>
    `;
}
