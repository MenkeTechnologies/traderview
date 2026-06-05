// IRC § 1446(f) — Withholding on Sale of Partnership Interest by Foreign Person.
// Transferee must withhold 10% of amount realized on sale of US trade/biz partnership interest.
// Effective Jan 1 2018 (TCJA); ECI portion deemed sold by foreign partner.
// Certificate of partnership withholding (Form W-8 ECI) to reduce withholding.
// Coordinate with § 1445 (FIRPTA), § 1446(a) (ECI partnership withholding).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    amount_realized: 0,
    transferor_is_foreign: true,
    partnership_is_us_business: true,
    eci_portion_amount: 0,
    seller_certification_us_person: false,
    seller_certification_no_eci: false,
    partnership_certification_low: false,
    publicly_traded_partnership: false,
    withholding_certificate_filed: false,
    actual_tax_liability: 0,
    days_since_closing: 0,
};

export async function renderSection1446F(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1446F.h1.title">// § 1446(f) PS INT. WITHHOLDING</span></h1>
        <p class="muted small" data-i18n="view.s1446F.hint.intro">
            <strong>Transferee withholds 10%</strong> of amount realized on sale of US trade/biz partnership
            interest by foreign person (TCJA 2018). <strong>ECI portion</strong> deemed sold by foreign partner
            → may be subject to higher US tax. <strong>Exceptions:</strong> seller certifies non-foreign (Form
            W-8 + W-9), seller certifies no ECI portion, partnership certifies low %, PTP exception,
            withholding certificate. <strong>Forms 8804 + 8805 + 8288 series</strong>. <strong>Coordinate
            with § 1445 FIRPTA</strong> (real estate).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1446F.h2.inputs">Inputs</h2>
            <form id="s1446F-form" class="inline-form">
                <label><span data-i18n="view.s1446F.label.amount">Amount realized ($)</span>
                    <input type="number" step="0.01" name="amount_realized" value="${state.amount_realized}"></label>
                <label><span data-i18n="view.s1446F.label.foreign">Transferor is foreign person?</span>
                    <input type="checkbox" name="transferor_is_foreign" ${state.transferor_is_foreign ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1446F.label.us_biz">Partnership conducts US business?</span>
                    <input type="checkbox" name="partnership_is_us_business" ${state.partnership_is_us_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1446F.label.eci">ECI portion amount ($)</span>
                    <input type="number" step="0.01" name="eci_portion_amount" value="${state.eci_portion_amount}"></label>
                <label><span data-i18n="view.s1446F.label.cert_us">Seller certified US person (W-9)?</span>
                    <input type="checkbox" name="seller_certification_us_person" ${state.seller_certification_us_person ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1446F.label.cert_no_eci">Seller certified no ECI portion?</span>
                    <input type="checkbox" name="seller_certification_no_eci" ${state.seller_certification_no_eci ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1446F.label.ps_cert">Partnership certified ≤ 25% ECI?</span>
                    <input type="checkbox" name="partnership_certification_low" ${state.partnership_certification_low ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1446F.label.ptp">Publicly Traded Partnership?</span>
                    <input type="checkbox" name="publicly_traded_partnership" ${state.publicly_traded_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1446F.label.cert_wh">Withholding certificate (8288-B equivalent)?</span>
                    <input type="checkbox" name="withholding_certificate_filed" ${state.withholding_certificate_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1446F.label.actual">Actual tax liability ($)</span>
                    <input type="number" step="0.01" name="actual_tax_liability" value="${state.actual_tax_liability}"></label>
                <label><span data-i18n="view.s1446F.label.days">Days since closing</span>
                    <input type="number" step="1" name="days_since_closing" value="${state.days_since_closing}"></label>
                <button class="primary" type="submit" data-i18n="view.s1446F.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1446F-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1446F.h2.scope">§ 1446(f) scope + applicability</h2>
            <ul class="muted small">
                <li data-i18n="view.s1446F.scope.transferee">TRANSFEREE (buyer of partnership interest) must withhold</li>
                <li data-i18n="view.s1446F.scope.foreign_seller">SELLER is foreign person (NRA, foreign corp, foreign partnership)</li>
                <li data-i18n="view.s1446F.scope.ecit">Partnership conducts US trade or business (creates ECI on portion)</li>
                <li data-i18n="view.s1446F.scope.10pct">Default rate: 10% of TOTAL amount realized</li>
                <li data-i18n="view.s1446F.scope.eci_only">Reduced rate available if seller certifies LIMITED ECI exposure</li>
                <li data-i18n="view.s1446F.scope.remit">Remit to IRS within 20 days of payment</li>
                <li data-i18n="view.s1446F.scope.penalty">Penalty: 100% of withholding amount + interest if failure</li>
                <li data-i18n="view.s1446F.scope.partnership_secondary">Partnership becomes secondarily liable if transferee fails</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1446F.h2.exceptions">Withholding exceptions</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1446F.th.basis">Exception basis</th>
                    <th data-i18n="view.s1446F.th.rate">Withholding rate</th>
                    <th data-i18n="view.s1446F.th.cert">Certificate needed</th>
                </tr></thead>
                <tbody>
                    <tr><td>Seller US person</td><td>0%</td><td>Form W-9</td></tr>
                    <tr><td>Seller no ECI in partnership</td><td>0%</td><td>Form 8804-C cert from seller</td></tr>
                    <tr><td>Partnership ≤ 25% ECI</td><td>0%</td><td>Partnership 8804-W certification</td></tr>
                    <tr><td>Treaty (rare — sale of interest)</td><td>0%</td><td>Form W-8BEN + treaty position</td></tr>
                    <tr><td>Publicly Traded Partnership</td><td>10% (broker)</td><td>Broker handles withholding</td></tr>
                    <tr><td>Withholding certificate</td><td>Actual tax / Amount realized</td><td>Application similar to 8288-B</td></tr>
                    <tr><td>Default (no cert)</td><td>10% amount realized</td><td>—</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1446F.h2.eci">ECI portion deemed sold</h2>
            <ul class="muted small">
                <li data-i18n="view.s1446F.eci.deemed">Foreign partner deemed to sell ECI assets pro-rata on sale of interest</li>
                <li data-i18n="view.s1446F.eci.rev_91_32">Reverses Rev. Rul. 91-32 (only US assets treated as sourced US) — TCJA codified deemed sale</li>
                <li data-i18n="view.s1446F.eci.character">Character: same as if assets directly sold (cap gain, ordinary, recapture)</li>
                <li data-i18n="view.s1446F.eci.computation">ECI gain = total gain × FMV of ECI assets / FMV of all assets</li>
                <li data-i18n="view.s1446F.eci.us_rate">Taxed at regular US rates (max 37% individual / 21% corp)</li>
                <li data-i18n="view.s1446F.eci.no_treaty">No treaty exception generally — § 894 limitations</li>
                <li data-i18n="view.s1446F.eci.ftc">Foreign FTC may apply if foreign country also taxes the gain</li>
                <li data-i18n="view.s1446F.eci.s897_combined">Coordinate with § 897 FIRPTA — overlapping for partnerships with US real property</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1446F.h2.ptp">Publicly Traded Partnership special rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s1446F.ptp.broker">Broker (not buyer) handles withholding on sales of PTP units</li>
                <li data-i18n="view.s1446F.ptp.notice">Notice 2018-08 + Notice 2018-29 + final 2020 regs apply</li>
                <li data-i18n="view.s1446F.ptp.cure">No correction filed if broker withholds correctly per Notice 2018-29</li>
                <li data-i18n="view.s1446F.ptp.qualified">"Qualified notice" required: PTP certifies % ECI to broker</li>
                <li data-i18n="view.s1446F.ptp.distributions">PTP distributions to foreign partners: § 1446(a) withholding (37% indiv / 21% corp)</li>
                <li data-i18n="view.s1446F.ptp.exemption">If PTP &lt; 10% ECI: no § 1446(f) withholding on sales</li>
                <li data-i18n="view.s1446F.ptp.report">Form 1042-S issued by broker for withholding</li>
                <li data-i18n="view.s1446F.ptp.major_ptps">Major PTPs: Energy Transfer Partners, Enterprise Products, MPLX, etc.</li>
            </ul>
        </div>
    `;
    document.getElementById('s1446F-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.amount_realized = Number(fd.get('amount_realized')) || 0;
        state.transferor_is_foreign = !!fd.get('transferor_is_foreign');
        state.partnership_is_us_business = !!fd.get('partnership_is_us_business');
        state.eci_portion_amount = Number(fd.get('eci_portion_amount')) || 0;
        state.seller_certification_us_person = !!fd.get('seller_certification_us_person');
        state.seller_certification_no_eci = !!fd.get('seller_certification_no_eci');
        state.partnership_certification_low = !!fd.get('partnership_certification_low');
        state.publicly_traded_partnership = !!fd.get('publicly_traded_partnership');
        state.withholding_certificate_filed = !!fd.get('withholding_certificate_filed');
        state.actual_tax_liability = Number(fd.get('actual_tax_liability')) || 0;
        state.days_since_closing = Number(fd.get('days_since_closing')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1446F-output');
    if (!el) return;
    let rate = 0.10;
    let basis = 'default';
    if (state.seller_certification_us_person) { rate = 0; basis = 'us_person'; }
    else if (state.seller_certification_no_eci) { rate = 0; basis = 'no_eci'; }
    else if (state.partnership_certification_low) { rate = 0; basis = 'ps_low_eci'; }
    else if (!state.transferor_is_foreign) { rate = 0; basis = 'not_foreign'; }
    else if (!state.partnership_is_us_business) { rate = 0; basis = 'no_us_biz'; }
    let withholding = rate * state.amount_realized;
    if (state.withholding_certificate_filed && state.actual_tax_liability < withholding) {
        withholding = state.actual_tax_liability;
        basis = 'certificate';
    }
    const eciTax = state.eci_portion_amount * 0.37;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1446F.h2.result">§ 1446(f) computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1446F.card.rate">Withholding rate</div>
                    <div class="value">${(rate * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1446F.card.basis">Basis</div>
                    <div class="value">${esc(t('view.s1446F.basis.' + basis))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1446F.card.withhold">Withholding due</div>
                    <div class="value">$${withholding.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1446F.card.net">Net to seller</div>
                    <div class="value">$${(state.amount_realized - withholding).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1446F.card.eci_tax">ECI tax exposure (37%)</div>
                    <div class="value">$${eciTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${withholding > 0 && state.actual_tax_liability < withholding ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s1446F.refund_note">
                    Withholding exceeds actual tax liability. File for refund via Form 1040NR / 1120-F.
                    Alternative: pre-closing withholding certificate (similar to § 1445 Form 8288-B) reduces
                    withholding to actual tax. IRS has 90 days to respond — typically escrow held until issued.
                </p>
            ` : ''}
        </div>
    `;
}
