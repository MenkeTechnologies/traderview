// IRC § 897 — FIRPTA (Foreign Investment in Real Property Tax Act).
// Foreign person's gain on USRPI (US Real Property Interest) treated as ECI.
// USRPI includes: direct US real estate + interests in USRPHC (Real Prop Holding Corp).
// USRPHC = 50%+ FMV is USRPI for 5-yr testing period.
// § 1445 withholding agent: 15% of amount realized (10% personal residence ≤ $1M).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    sale_price: 0,
    adjusted_basis: 0,
    usrpi_type: 'direct',
    is_personal_residence: false,
    is_publicly_traded: false,
    ownership_pct: 0,
    qfp_election: false,
    treaty_benefit: false,
    seller_country: '',
    held_5_years: false,
};

export async function renderSection897(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s897.h1.title">// § 897 FIRPTA</span></h1>
        <p class="muted small" data-i18n="view.s897.hint.intro">
            Foreign person's gain on <strong>USRPI</strong> (US Real Property Interest) treated as
            <strong>ECI</strong> — taxed at regular US rates (37% individual / 21% corporate).
            USRPI includes direct US real estate + interests in <strong>USRPHC</strong> (US Real Property
            Holding Corp = 50%+ FMV in USRPI for 5-yr testing period). <strong>Publicly traded exception</strong>:
            ≤ 5% owners. <strong>§ 1445 withholding</strong>: 15% (10% personal residence ≤ $1M).
            Forms 8288, 8288-A, 8288-B.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s897.h2.inputs">Inputs</h2>
            <form id="s897-form" class="inline-form">
                <label><span data-i18n="view.s897.label.sale">Sale price ($)</span>
                    <input type="number" step="10000" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s897.label.basis">Adjusted basis ($)</span>
                    <input type="number" step="10000" name="adjusted_basis" value="${state.adjusted_basis}"></label>
                <label><span data-i18n="view.s897.label.type">USRPI type</span>
                    <select name="usrpi_type">
                        <option value="direct" ${state.usrpi_type === 'direct' ? 'selected' : ''}>Direct real estate</option>
                        <option value="usrphc" ${state.usrpi_type === 'usrphc' ? 'selected' : ''}>USRPHC stock</option>
                        <option value="usrpi_partnership" ${state.usrpi_type === 'usrpi_partnership' ? 'selected' : ''}>Partnership interest</option>
                        <option value="usrpi_trust" ${state.usrpi_type === 'usrpi_trust' ? 'selected' : ''}>Trust beneficial interest</option>
                    </select>
                </label>
                <label><span data-i18n="view.s897.label.residence">Personal residence?</span>
                    <input type="checkbox" name="is_personal_residence" ${state.is_personal_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s897.label.public">Publicly traded USRPHC?</span>
                    <input type="checkbox" name="is_publicly_traded" ${state.is_publicly_traded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s897.label.ownership">Ownership % (USRPHC)</span>
                    <input type="number" step="0.1" name="ownership_pct" value="${state.ownership_pct}"></label>
                <label><span data-i18n="view.s897.label.qfp">Qualified Foreign Pension election?</span>
                    <input type="checkbox" name="qfp_election" ${state.qfp_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s897.label.treaty">Treaty benefit available?</span>
                    <input type="checkbox" name="treaty_benefit" ${state.treaty_benefit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s897.label.country">Seller country</span>
                    <input type="text" name="seller_country" value="${esc(state.seller_country)}"></label>
                <label><span data-i18n="view.s897.label.held5">Held 5+ years?</span>
                    <input type="checkbox" name="held_5_years" ${state.held_5_years ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s897.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s897-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s897.h2.usrpi">USRPI scope</h2>
            <ul class="muted small">
                <li data-i18n="view.s897.usrpi.direct">Direct interest in US real property (fee simple, leases, mineral rights, royalties)</li>
                <li data-i18n="view.s897.usrpi.usrphc">USRPHC stock: domestic corp w/ 50%+ FMV in USRPI during 5-yr testing</li>
                <li data-i18n="view.s897.usrpi.partnership">Partnership interest where partnership is USRPHC equivalent</li>
                <li data-i18n="view.s897.usrpi.publicly">Publicly traded USRPHC: 5%-or-less foreign holder exempt (Class B exception)</li>
                <li data-i18n="view.s897.usrpi.reit">REIT stock: special rules — domestic REITs USRPI but exceptions for public + small-holder</li>
                <li data-i18n="view.s897.usrpi.regular_trade">Inventory + securities held in regular trade — NOT USRPI</li>
                <li data-i18n="view.s897.usrpi.deemed_sale">Deemed sales: corp liquidation distributing USRPI triggers § 897(d)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s897.h2.exceptions">FIRPTA exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s897.exc.personal">Personal residence ≤ $300K — full withholding exemption (buyer's affidavit)</li>
                <li data-i18n="view.s897.exc.300_1m">Personal residence $300K-$1M — 10% withholding (not 15%)</li>
                <li data-i18n="view.s897.exc.qfp">Qualified Foreign Pension (QFP) — § 897(l) exemption (PATH Act 2015)</li>
                <li data-i18n="view.s897.exc.publicly_traded">Publicly traded USRPHC + ≤ 5% ownership — exempt from § 897</li>
                <li data-i18n="view.s897.exc.cleansing">Cleansing rule § 897(c)(1)(B): no USRPI in last 5 yrs — escapes USRPHC</li>
                <li data-i18n="view.s897.exc.s355_reorg">§ 355 distributions + § 897(e) nonrec. transfers may defer (not eliminate)</li>
                <li data-i18n="view.s897.exc.cnu_election">Form 8288-B: reduced withholding certificate based on actual tax</li>
                <li data-i18n="view.s897.exc.foreign_govt">Foreign sovereign / pension (CFTC-regulated): exempt under § 892</li>
            </ul>
        </div>
    `;
    document.getElementById('s897-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.adjusted_basis = Number(fd.get('adjusted_basis')) || 0;
        state.usrpi_type = fd.get('usrpi_type');
        state.is_personal_residence = !!fd.get('is_personal_residence');
        state.is_publicly_traded = !!fd.get('is_publicly_traded');
        state.ownership_pct = Number(fd.get('ownership_pct')) || 0;
        state.qfp_election = !!fd.get('qfp_election');
        state.treaty_benefit = !!fd.get('treaty_benefit');
        state.seller_country = fd.get('seller_country');
        state.held_5_years = !!fd.get('held_5_years');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s897-output');
    if (!el) return;
    const gain = Math.max(0, state.sale_price - state.adjusted_basis);
    const publiclyTradedExempt = state.is_publicly_traded && state.ownership_pct <= 5;
    const qfpExempt = state.qfp_election;
    const isExempt = publiclyTradedExempt || qfpExempt;
    const personalResidenceLow = state.is_personal_residence && state.sale_price <= 300_000;
    const personalResidenceMid = state.is_personal_residence && state.sale_price <= 1_000_000;
    let withholdingRate = 0.15;
    if (personalResidenceLow) withholdingRate = 0;
    else if (personalResidenceMid) withholdingRate = 0.10;
    if (isExempt) withholdingRate = 0;
    const withholdingDue = withholdingRate * state.sale_price;
    const taxIfLongHeld = state.held_5_years ? gain * 0.20 : gain * 0.37;
    const finalTax = isExempt ? 0 : Math.max(taxIfLongHeld, withholdingDue);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s897.h2.result">FIRPTA computation</h2>
            <div class="cards">
                <div class="card ${isExempt ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s897.card.exempt">FIRPTA exempt?</div>
                    <div class="value">${isExempt ? esc(t('view.s897.status.yes')) : esc(t('view.s897.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s897.card.gain">Realized gain</div>
                    <div class="value">$${gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s897.card.withholding_rate">Withholding rate</div>
                    <div class="value">${(withholdingRate * 100).toFixed(1)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s897.card.withholding_due">Withholding due (Form 8288)</div>
                    <div class="value">$${withholdingDue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s897.card.tax">Tax on gain</div>
                    <div class="value">$${taxIfLongHeld.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s897.card.final">Final tax / withholding</div>
                    <div class="value">$${finalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${withholdingDue > taxIfLongHeld && !isExempt ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s897.refund_note">
                    Withholding exceeds tax due — file Form 8288-B (Withholding Certificate) at or before
                    closing for reduced rate, OR claim refund via Form 1040NR (individual) / 1120-F (corp).
                </p>
            ` : ''}
        </div>
    `;
}
