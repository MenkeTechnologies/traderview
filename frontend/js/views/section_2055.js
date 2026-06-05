// IRC § 2055 — Estate Tax Charitable Deduction.
// Unlimited deduction for bequests to qualifying charities (501(c)(3), governments, certain trusts).
// Must be ASCERTAINABLE amount at death; split-interest with non-charitable interests require specific forms.
// Common vehicles: outright bequest, CRT (Charitable Remainder Trust), CLT (Charitable Lead Trust).
// Coordinate with § 170 income tax deduction (multi-faceted estate planning).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    bequest_amount: 0,
    type_of_bequest: 'outright',
    charity_type: '501c3_public',
    is_qualified_charity: true,
    split_interest_charitable_remainder: 0,
    split_interest_charitable_lead: 0,
    actuarial_remainder_value: 0,
    is_crat: false,
    is_crut: false,
    is_clt: false,
    is_pooled_income_fund: false,
    annual_payout_rate_pct: 0,
    trust_term_years: 0,
    is_private_foundation: false,
    private_foundation_30pct: false,
    grant_5yr_carryover: 0,
    estate_total_value: 0,
};

export async function renderSection2055(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s2055.h1.title">// § 2055 ESTATE CHARITABLE DED</span></h1>
        <p class="muted small" data-i18n="view.s2055.hint.intro">
            <strong>UNLIMITED</strong> deduction for bequests to qualifying charities: 501(c)(3), governments,
            certain trusts. Must be <strong>ASCERTAINABLE amount at death</strong>; split-interest with
            non-charitable interests require specific forms. <strong>Common vehicles:</strong> outright bequest,
            <strong>CRT</strong> (Charitable Remainder Trust), <strong>CLT</strong> (Charitable Lead Trust),
            pooled income fund. <strong>Coordinate with § 170</strong> income tax deduction (multi-faceted
            estate planning). <strong>Schedule O</strong> of Form 706 lists charitable transfers.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s2055.h2.inputs">Inputs</h2>
            <form id="s2055-form" class="inline-form">
                <label><span data-i18n="view.s2055.label.amount">Bequest amount ($)</span>
                    <input type="number" step="0.01" name="bequest_amount" value="${state.bequest_amount}"></label>
                <label><span data-i18n="view.s2055.label.type">Type of bequest</span>
                    <select name="type_of_bequest">
                        <option value="outright" ${state.type_of_bequest === 'outright' ? 'selected' : ''}>Outright bequest</option>
                        <option value="crat" ${state.type_of_bequest === 'crat' ? 'selected' : ''}>CRAT (Charitable Remainder Annuity Trust)</option>
                        <option value="crut" ${state.type_of_bequest === 'crut' ? 'selected' : ''}>CRUT (Charitable Remainder Unitrust)</option>
                        <option value="clat" ${state.type_of_bequest === 'clat' ? 'selected' : ''}>CLAT (Charitable Lead Annuity Trust)</option>
                        <option value="clut" ${state.type_of_bequest === 'clut' ? 'selected' : ''}>CLUT (Charitable Lead Unitrust)</option>
                        <option value="pif" ${state.type_of_bequest === 'pif' ? 'selected' : ''}>Pooled Income Fund</option>
                        <option value="donor_advised" ${state.type_of_bequest === 'donor_advised' ? 'selected' : ''}>Donor-advised fund</option>
                        <option value="private_foundation" ${state.type_of_bequest === 'private_foundation' ? 'selected' : ''}>Private foundation</option>
                    </select>
                </label>
                <label><span data-i18n="view.s2055.label.charity">Charity type</span>
                    <select name="charity_type">
                        <option value="501c3_public" ${state.charity_type === '501c3_public' ? 'selected' : ''}>501(c)(3) public charity</option>
                        <option value="501c3_private_op" ${state.charity_type === '501c3_private_op' ? 'selected' : ''}>501(c)(3) private operating foundation</option>
                        <option value="501c3_private_grant" ${state.charity_type === '501c3_private_grant' ? 'selected' : ''}>501(c)(3) private grantmaking foundation</option>
                        <option value="government" ${state.charity_type === 'government' ? 'selected' : ''}>US Government / state / local</option>
                        <option value="cemetery" ${state.charity_type === 'cemetery' ? 'selected' : ''}>Cemetery company (501(c)(13))</option>
                        <option value="veterans" ${state.charity_type === 'veterans' ? 'selected' : ''}>Veterans (501(c)(19))</option>
                        <option value="religious_order" ${state.charity_type === 'religious_order' ? 'selected' : ''}>Religious order / mission</option>
                    </select>
                </label>
                <label><span data-i18n="view.s2055.label.qualified">Is qualified charity?</span>
                    <input type="checkbox" name="is_qualified_charity" ${state.is_qualified_charity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2055.label.remainder">CRT charitable remainder ($)</span>
                    <input type="number" step="0.01" name="split_interest_charitable_remainder" value="${state.split_interest_charitable_remainder}"></label>
                <label><span data-i18n="view.s2055.label.lead">CLT charitable lead ($)</span>
                    <input type="number" step="0.01" name="split_interest_charitable_lead" value="${state.split_interest_charitable_lead}"></label>
                <label><span data-i18n="view.s2055.label.actuarial">Actuarial remainder value ($)</span>
                    <input type="number" step="0.01" name="actuarial_remainder_value" value="${state.actuarial_remainder_value}"></label>
                <label><span data-i18n="view.s2055.label.crat">Is CRAT?</span>
                    <input type="checkbox" name="is_crat" ${state.is_crat ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2055.label.crut">Is CRUT?</span>
                    <input type="checkbox" name="is_crut" ${state.is_crut ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2055.label.clt">Is CLT?</span>
                    <input type="checkbox" name="is_clt" ${state.is_clt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2055.label.pif">Is Pooled Income Fund?</span>
                    <input type="checkbox" name="is_pooled_income_fund" ${state.is_pooled_income_fund ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2055.label.payout">Annual payout rate %</span>
                    <input type="number" step="0.1" name="annual_payout_rate_pct" value="${state.annual_payout_rate_pct}"></label>
                <label><span data-i18n="view.s2055.label.term">Trust term years</span>
                    <input type="number" step="1" name="trust_term_years" value="${state.trust_term_years}"></label>
                <label><span data-i18n="view.s2055.label.pf">Private foundation?</span>
                    <input type="checkbox" name="is_private_foundation" ${state.is_private_foundation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2055.label.30pct">Private foundation 30% AGI cap?</span>
                    <input type="checkbox" name="private_foundation_30pct" ${state.private_foundation_30pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2055.label.carryover">5-yr carryover available ($)</span>
                    <input type="number" step="0.01" name="grant_5yr_carryover" value="${state.grant_5yr_carryover}"></label>
                <label><span data-i18n="view.s2055.label.estate">Estate total value ($)</span>
                    <input type="number" step="0.01" name="estate_total_value" value="${state.estate_total_value}"></label>
                <button class="primary" type="submit" data-i18n="view.s2055.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s2055-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2055.h2.crt">CRT (Charitable Remainder Trust) variants</h2>
            <ul class="muted small">
                <li data-i18n="view.s2055.crt.crat">CRAT — fixed annual payment (annuity); payout 5% min - 50% max of initial value</li>
                <li data-i18n="view.s2055.crt.crut">CRUT — variable annual payment % of annual value; same payout limits</li>
                <li data-i18n="view.s2055.crt.charitable_remainder">Charitable remainder: ≥ 10% of fair market value at inception</li>
                <li data-i18n="view.s2055.crt.income">Income beneficiaries: get annuity / unitrust payments during term</li>
                <li data-i18n="view.s2055.crt.term">Term: lifetime + ≤ 20 years OR fixed term ≤ 20 years</li>
                <li data-i18n="view.s2055.crt.benefits">Benefits: estate deduction + tax-free trust growth + income for life</li>
                <li data-i18n="view.s2055.crt.s664">§ 664 requirements: written trust + qualifying charitable beneficiary</li>
                <li data-i18n="view.s2055.crt.flip_clut">FLIP CLUT: starts as straight CLUT, "flips" to CRUT on specific event</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2055.h2.clt">CLT (Charitable Lead Trust) variants</h2>
            <ul class="muted small">
                <li data-i18n="view.s2055.clt.opposite">Opposite of CRT: charity gets income FIRST, family gets remainder</li>
                <li data-i18n="view.s2055.clt.clat">CLAT — fixed annual charitable payment (annuity)</li>
                <li data-i18n="view.s2055.clt.clut">CLUT — variable annual % of trust value</li>
                <li data-i18n="view.s2055.clt.income_tax_treatment">Income tax: grantor CLT vs non-grantor CLT (different income recognition)</li>
                <li data-i18n="view.s2055.clt.estate_tax">Estate tax deduction: equal to PV of charitable lead at IRS 7520 rate</li>
                <li data-i18n="view.s2055.clt.zero_out">"Zero-out" technique: payments to charity = PV of property → no remainder taxable</li>
                <li data-i18n="view.s2055.clt.low_rate_benefit">Low 7520 rate benefits remainder beneficiary (more accumulates)</li>
                <li data-i18n="view.s2055.clt.gst_planning">GST planning: zero-out CLT may avoid GST on remainder to grandchildren</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2055.h2.coordination">§ 170 income tax coordination</h2>
            <ul class="muted small">
                <li data-i18n="view.s2055.coord.same_charity">Same charity may receive estate (§ 2055) + lifetime (§ 170) gifts</li>
                <li data-i18n="view.s2055.coord.public_60pct">Public charity: 60% AGI cap for cash; 30% for appreciated property</li>
                <li data-i18n="view.s2055.coord.private_30pct">Private foundation: 30% / 20% AGI caps (more restrictive)</li>
                <li data-i18n="view.s2055.coord.5yr_carryover">§ 170 5-year carryover for amounts above AGI limit</li>
                <li data-i18n="view.s2055.coord.crt_lifetime">CRT during life: § 170 deduction = PV of remainder + § 2055 at death if applicable</li>
                <li data-i18n="view.s2055.coord.daf">Donor-Advised Fund: § 170 deduction immediate; estate deduction at death if balance remains</li>
                <li data-i18n="view.s2055.coord.pf_self_dealing">Private foundation: § 4941 self-dealing rules + § 4942 distribution requirement</li>
                <li data-i18n="view.s2055.coord.qcd">QCD (§ 408(d)(8)): IRA → charity directly ($105K limit 2024) — separate income tax benefit</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2055.h2.split_interest">Split-interest rules (Reg 25.2522(c)-3)</h2>
            <ul class="muted small">
                <li data-i18n="view.s2055.si.requirements">Split-interest: combo of charitable + non-charitable beneficiaries</li>
                <li data-i18n="view.s2055.si.specific_forms">Only certain forms qualify for deduction: CRAT, CRUT, CLT, PIF, qualified PRI</li>
                <li data-i18n="view.s2055.si.valuation">Charitable portion valued under § 7520 / Reg 20.2031-7 actuarial tables</li>
                <li data-i18n="view.s2055.si.7520_rate">§ 7520 rate (4-1/2% Federal Mid-Term × 120%): updates monthly</li>
                <li data-i18n="view.s2055.si.special_factors">Special factors: payment timing, asset volatility, term length</li>
                <li data-i18n="view.s2055.si.deeded">Deeded interest in real property: limited; partial gift rules</li>
                <li data-i18n="view.s2055.si.qualified_pri">Qualified Personal Residence Interest (QPRI): limited estate deduction</li>
                <li data-i18n="view.s2055.si.non_qualifying">Non-qualifying split-interest: NO deduction (charitable portion lost)</li>
            </ul>
        </div>
    `;
    document.getElementById('s2055-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.bequest_amount = Number(fd.get('bequest_amount')) || 0;
        state.type_of_bequest = fd.get('type_of_bequest');
        state.charity_type = fd.get('charity_type');
        state.is_qualified_charity = !!fd.get('is_qualified_charity');
        state.split_interest_charitable_remainder = Number(fd.get('split_interest_charitable_remainder')) || 0;
        state.split_interest_charitable_lead = Number(fd.get('split_interest_charitable_lead')) || 0;
        state.actuarial_remainder_value = Number(fd.get('actuarial_remainder_value')) || 0;
        state.is_crat = !!fd.get('is_crat');
        state.is_crut = !!fd.get('is_crut');
        state.is_clt = !!fd.get('is_clt');
        state.is_pooled_income_fund = !!fd.get('is_pooled_income_fund');
        state.annual_payout_rate_pct = Number(fd.get('annual_payout_rate_pct')) || 0;
        state.trust_term_years = Number(fd.get('trust_term_years')) || 0;
        state.is_private_foundation = !!fd.get('is_private_foundation');
        state.private_foundation_30pct = !!fd.get('private_foundation_30pct');
        state.grant_5yr_carryover = Number(fd.get('grant_5yr_carryover')) || 0;
        state.estate_total_value = Number(fd.get('estate_total_value')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s2055-output');
    if (!el) return;
    let deductible = 0;
    if (state.type_of_bequest === 'outright' && state.is_qualified_charity) {
        deductible = state.bequest_amount;
    } else if (state.is_crat || state.is_crut) {
        deductible = state.actuarial_remainder_value;
    } else if (state.is_clt) {
        deductible = state.split_interest_charitable_lead;
    } else if (state.is_pooled_income_fund) {
        deductible = state.actuarial_remainder_value;
    } else {
        deductible = state.bequest_amount;
    }
    const estate_tax_saved = deductible * 0.40;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s2055.h2.result">§ 2055 estate deduction</h2>
            <div class="cards">
                <div class="card ${state.is_qualified_charity ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2055.card.qualified">Qualified charity?</div>
                    <div class="value">${state.is_qualified_charity ? esc(t('view.s2055.status.yes')) : esc(t('view.s2055.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2055.card.deductible">§ 2055 deductible</div>
                    <div class="value">$${deductible.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2055.card.saved">Estate tax saved (40%)</div>
                    <div class="value">$${estate_tax_saved.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2055.card.net">Net to non-charity</div>
                    <div class="value">$${(state.bequest_amount - deductible).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_crat || state.is_crut || state.is_clt ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s2055.split_note">
                    Split-interest trust: only charitable portion (PV of remainder for CRT or PV of lead for
                    CLT) deductible under § 2055. Actuarial computation under § 7520 rate. Monitor low rate
                    environment for CLT planning (more residual to family); high rate environment for CRT
                    (more charitable remainder).
                </p>
            ` : ''}
        </div>
    `;
}
