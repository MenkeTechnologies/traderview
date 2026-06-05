// IRC § 1445 — FIRPTA Withholding (Mechanical Companion to § 897).
// Buyer withholds 15% of amount realized on USRPI disposition by foreign person.
// 10% for personal residence ≤ $1M; 0% for ≤ $300K personal use.
// Foreign seller claims credit on Form 1040NR or 1120-F.
// Reduced withholding certificate (Form 8288-B) available pre-closing.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    amount_realized: 0,
    buyer_personal_residence: false,
    actual_tax_liability: 0,
    form_8288b_filed: false,
    foreign_seller: true,
    seller_certified_us_person: false,
    qfp_certified: false,
    publicly_traded: false,
    distribution_from_usrphc: false,
    partnership_distribution: false,
};

export async function renderSection1445(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1445.h1.title">// § 1445 FIRPTA WITHHOLDING</span></h1>
        <p class="muted small" data-i18n="view.s1445.hint.intro">
            Buyer (transferee) withholds <strong>15%</strong> of amount realized on USRPI disposition by foreign
            person. <strong>10%</strong> for personal residence ≤ $1M; <strong>0%</strong> for personal use ≤ $300K
            (buyer affidavit required). <strong>Foreign seller</strong> claims credit on 1040NR / 1120-F.
            <strong>Reduced withholding certificate</strong> via Form 8288-B pre-closing. Penalties on buyer
            for failure to withhold. Forms <strong>8288, 8288-A, 8288-B</strong>.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1445.h2.inputs">Inputs</h2>
            <form id="s1445-form" class="inline-form">
                <label><span data-i18n="view.s1445.label.amount">Amount realized ($)</span>
                    <input type="number" step="0.01" name="amount_realized" value="${state.amount_realized}"></label>
                <label><span data-i18n="view.s1445.label.personal">Buyer personal residence?</span>
                    <input type="checkbox" name="buyer_personal_residence" ${state.buyer_personal_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1445.label.actual_tax">Actual tax liability (for 8288-B) ($)</span>
                    <input type="number" step="0.01" name="actual_tax_liability" value="${state.actual_tax_liability}"></label>
                <label><span data-i18n="view.s1445.label.form_8288b">Form 8288-B filed?</span>
                    <input type="checkbox" name="form_8288b_filed" ${state.form_8288b_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1445.label.foreign">Seller is foreign person?</span>
                    <input type="checkbox" name="foreign_seller" ${state.foreign_seller ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1445.label.cert">Seller certified non-foreign (Form W-8 / W-9)?</span>
                    <input type="checkbox" name="seller_certified_us_person" ${state.seller_certified_us_person ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1445.label.qfp">Qualified Foreign Pension certified?</span>
                    <input type="checkbox" name="qfp_certified" ${state.qfp_certified ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1445.label.public">Publicly traded USRPHC ≤ 5%?</span>
                    <input type="checkbox" name="publicly_traded" ${state.publicly_traded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1445.label.usrphc_dist">USRPHC distribution to shareholder?</span>
                    <input type="checkbox" name="distribution_from_usrphc" ${state.distribution_from_usrphc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1445.label.partnership">Partnership distribution?</span>
                    <input type="checkbox" name="partnership_distribution" ${state.partnership_distribution ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1445.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1445-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1445.h2.thresholds">§ 1445 withholding rate thresholds</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1445.th.scenario">Scenario</th>
                    <th data-i18n="view.s1445.th.rate">Withholding rate</th>
                    <th data-i18n="view.s1445.th.affidavit">Buyer affidavit required?</th>
                </tr></thead>
                <tbody>
                    <tr><td>Default — USRPI sale</td><td>15%</td><td>—</td></tr>
                    <tr><td>Personal residence > $1M</td><td>15%</td><td>—</td></tr>
                    <tr><td>Personal residence $300K-$1M</td><td>10%</td><td>YES — § 1445(b)(5) affidavit</td></tr>
                    <tr><td>Personal residence ≤ $300K</td><td>0%</td><td>YES — § 1445(b)(5) + intent to occupy</td></tr>
                    <tr><td>Seller certified US person W-9</td><td>0%</td><td>YES — § 1445(b)(2) certificate</td></tr>
                    <tr><td>Form 8288-B reduced cert</td><td>Actual tax / amount realized</td><td>—</td></tr>
                    <tr><td>QFP certified</td><td>0%</td><td>YES — § 1445(b)(7) (PATH Act 2015)</td></tr>
                    <tr><td>Public USRPHC ≤ 5% holder</td><td>0%</td><td>YES — § 897(c)(3) exception</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1445.h2.process">Withholding process + deadlines</h2>
            <ol class="muted small">
                <li data-i18n="view.s1445.proc.closing">Buyer withholds at closing — deducts from sale proceeds delivered to seller</li>
                <li data-i18n="view.s1445.proc.remit">Remit to IRS within 20 days of closing with Form 8288</li>
                <li data-i18n="view.s1445.proc.statement">Provide Form 8288-A copy to seller — needed for refund claim</li>
                <li data-i18n="view.s1445.proc.8288b">Pre-closing Form 8288-B: reduce withholding to actual tax — IRS has 90 days to respond</li>
                <li data-i18n="view.s1445.proc.escrow">Common practice: escrow withholding until 8288-B determination</li>
                <li data-i18n="view.s1445.proc.refund">Seller refund: file 1040NR (individual) or 1120-F (corp) for actual tax + claim withholding credit</li>
                <li data-i18n="view.s1445.proc.penalty">Buyer penalty: tax liability + interest if fails to withhold</li>
                <li data-i18n="view.s1445.proc.agent">Settlement agent / escrow agent often handles mechanical withholding</li>
            </ol>
        </div>
    `;
    document.getElementById('s1445-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.amount_realized = Number(fd.get('amount_realized')) || 0;
        state.buyer_personal_residence = !!fd.get('buyer_personal_residence');
        state.actual_tax_liability = Number(fd.get('actual_tax_liability')) || 0;
        state.form_8288b_filed = !!fd.get('form_8288b_filed');
        state.foreign_seller = !!fd.get('foreign_seller');
        state.seller_certified_us_person = !!fd.get('seller_certified_us_person');
        state.qfp_certified = !!fd.get('qfp_certified');
        state.publicly_traded = !!fd.get('publicly_traded');
        state.distribution_from_usrphc = !!fd.get('distribution_from_usrphc');
        state.partnership_distribution = !!fd.get('partnership_distribution');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1445-output');
    if (!el) return;
    let rate = 0.15;
    let exempt = false;
    let reason = 'default';
    if (state.seller_certified_us_person) { rate = 0; exempt = true; reason = 'us_person_cert'; }
    else if (state.qfp_certified) { rate = 0; exempt = true; reason = 'qfp_cert'; }
    else if (state.publicly_traded) { rate = 0; exempt = true; reason = 'public_traded'; }
    else if (state.buyer_personal_residence) {
        if (state.amount_realized <= 300_000) { rate = 0; reason = 'personal_300k'; }
        else if (state.amount_realized <= 1_000_000) { rate = 0.10; reason = 'personal_1m'; }
    }
    let withholding = rate * state.amount_realized;
    if (state.form_8288b_filed && state.actual_tax_liability < withholding) {
        withholding = state.actual_tax_liability;
        reason = '8288b';
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1445.h2.result">§ 1445 withholding computation</h2>
            <div class="cards">
                <div class="card ${exempt ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s1445.card.exempt">Exempt?</div>
                    <div class="value">${exempt ? esc(t('view.s1445.status.yes')) : esc(t('view.s1445.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1445.card.rate">Withholding rate</div>
                    <div class="value">${(rate * 100).toFixed(1)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1445.card.basis">Basis</div>
                    <div class="value">${esc(t('view.s1445.basis.' + reason))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1445.card.withhold">Withholding owed</div>
                    <div class="value">$${withholding.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1445.card.net_proceeds">Net to seller</div>
                    <div class="value">$${(state.amount_realized - withholding).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.form_8288b_filed ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s1445.b_note">
                    Form 8288-B pending: IRS issues Withholding Certificate within 90 days; escrow held until
                    determination. Reduce withholding to actual computed tax. Closing proceeds may be deposited
                    in escrow while certificate pending.
                </p>
            ` : ''}
        </div>
    `;
}
