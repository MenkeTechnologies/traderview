// IRC § 483 — Imputed Interest on Installment Sales.
// Installment sales with inadequate interest: portion of each payment recharacterized as interest.
// Test rate: AFR (Applicable Federal Rate) based on term + compounding period.
// Below-AFR loans: portion of each payment = interest (not capital gain).
// Coordinate with § 7872 (below-market loans) + § 1274 (debt instruments).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    sale_price_total: 0,
    stated_interest_rate_pct: 0,
    afr_test_rate_pct: 0,
    payment_term_years: 0,
    annual_payment: 0,
    installment_sale_election: true,
    is_related_party: false,
    is_seller_financing: true,
    transaction_type: 'real_property',
    is_publicly_traded_debt: false,
    fmv_at_sale: 0,
    seller_cost_basis: 0,
    holding_period_years: 0,
    sale_date: '',
    use_short_term_afr: false,
};

export async function renderSection483(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s483.h1.title">// § 483 IMPUTED INTEREST</span></h1>
        <p class="muted small" data-i18n="view.s483.hint.intro">
            Installment sales with <strong>inadequate interest</strong>: portion of each payment recharacterized
            as <strong>INTEREST</strong>. <strong>Test rate:</strong> AFR (Applicable Federal Rate) based on
            term + compounding period. <strong>Below-AFR loans:</strong> portion of each payment = interest
            (not capital gain). <strong>§ 7872</strong> below-market loans + <strong>§ 1274</strong> debt
            instruments coordinate. <strong>3 AFR rates:</strong> short-term (≤ 3 yrs), mid-term (3-9 yrs),
            long-term (&gt; 9 yrs). <strong>De minimis:</strong> ≤ $10K sales without imputed interest.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s483.h2.inputs">Inputs</h2>
            <form id="s483-form" class="inline-form">
                <label><span data-i18n="view.s483.label.price">Sale price total ($)</span>
                    <input type="number" step="10000" name="sale_price_total" value="${state.sale_price_total}"></label>
                <label><span data-i18n="view.s483.label.stated">Stated interest rate %</span>
                    <input type="number" step="0.01" name="stated_interest_rate_pct" value="${state.stated_interest_rate_pct}"></label>
                <label><span data-i18n="view.s483.label.afr">Test AFR rate %</span>
                    <input type="number" step="0.01" name="afr_test_rate_pct" value="${state.afr_test_rate_pct}"></label>
                <label><span data-i18n="view.s483.label.years">Payment term years</span>
                    <input type="number" step="0.5" name="payment_term_years" value="${state.payment_term_years}"></label>
                <label><span data-i18n="view.s483.label.annual">Annual payment ($)</span>
                    <input type="number" step="100" name="annual_payment" value="${state.annual_payment}"></label>
                <label><span data-i18n="view.s483.label.election">§ 453 installment election?</span>
                    <input type="checkbox" name="installment_sale_election" ${state.installment_sale_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s483.label.related">Related party?</span>
                    <input type="checkbox" name="is_related_party" ${state.is_related_party ? 'checked' : ''}></label>
                <label><span data-i18n="view.s483.label.seller">Seller financing?</span>
                    <input type="checkbox" name="is_seller_financing" ${state.is_seller_financing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s483.label.type">Transaction type</span>
                    <select name="transaction_type">
                        <option value="real_property" ${state.transaction_type === 'real_property' ? 'selected' : ''}>Real property</option>
                        <option value="business" ${state.transaction_type === 'business' ? 'selected' : ''}>Business / partnership interest</option>
                        <option value="personal_property" ${state.transaction_type === 'personal_property' ? 'selected' : ''}>Personal property</option>
                        <option value="services" ${state.transaction_type === 'services' ? 'selected' : ''}>Services contract</option>
                        <option value="lease_with_option" ${state.transaction_type === 'lease_with_option' ? 'selected' : ''}>Lease with option to buy</option>
                    </select>
                </label>
                <label><span data-i18n="view.s483.label.publicly">Publicly traded debt?</span>
                    <input type="checkbox" name="is_publicly_traded_debt" ${state.is_publicly_traded_debt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s483.label.fmv">FMV at sale ($)</span>
                    <input type="number" step="10000" name="fmv_at_sale" value="${state.fmv_at_sale}"></label>
                <label><span data-i18n="view.s483.label.basis">Seller cost basis ($)</span>
                    <input type="number" step="10000" name="seller_cost_basis" value="${state.seller_cost_basis}"></label>
                <label><span data-i18n="view.s483.label.holding">Holding period years</span>
                    <input type="number" step="0.5" name="holding_period_years" value="${state.holding_period_years}"></label>
                <label><span data-i18n="view.s483.label.date">Sale date</span>
                    <input type="date" name="sale_date" value="${state.sale_date}"></label>
                <label><span data-i18n="view.s483.label.short">Use short-term AFR?</span>
                    <input type="checkbox" name="use_short_term_afr" ${state.use_short_term_afr ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s483.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s483-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s483.h2.afr">AFR (Applicable Federal Rate)</h2>
            <ul class="muted small">
                <li data-i18n="view.s483.afr.short">Short-term AFR: applies to debt ≤ 3 years (90-day Treasury)</li>
                <li data-i18n="view.s483.afr.mid">Mid-term AFR: applies to debt > 3 to ≤ 9 years (3-9 year Treasury)</li>
                <li data-i18n="view.s483.afr.long">Long-term AFR: applies to debt > 9 years (long bond yields)</li>
                <li data-i18n="view.s483.afr.published">Published monthly in Rev. Rul. (e.g., Rev. Rul. 2024-X)</li>
                <li data-i18n="view.s483.afr.annual_semi">Annual or semi-annual compounding</li>
                <li data-i18n="view.s483.afr.current">November 2024 AFRs: short-term ~5.1%, mid ~4.3%, long ~4.5%</li>
                <li data-i18n="view.s483.afr.110pct">§ 7872(f)(3) gift loan: 110% of AFR</li>
                <li data-i18n="view.s483.afr.applicable_rate">"Applicable rate" used to test adequate interest</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s483.h2.imputation_rules">Imputation rules</h2>
            <ol class="muted small">
                <li data-i18n="view.s483.imp.test">Test: stated rate ≥ AFR → adequate interest; no imputation</li>
                <li data-i18n="view.s483.imp.below">If stated rate &lt; AFR: portion of each payment = imputed interest</li>
                <li data-i18n="view.s483.imp.formula">Imputed interest = AFR × outstanding balance per period</li>
                <li data-i18n="view.s483.imp.allocation">Allocate to interest first, then to principal recovery</li>
                <li data-i18n="view.s483.imp.character">Interest portion: ORDINARY INCOME for seller, deductible (§ 163) for buyer</li>
                <li data-i18n="view.s483.imp.principal">Principal portion: capital gain (per § 453 installment method)</li>
                <li data-i18n="view.s483.imp.buyer_basis">Buyer's basis = principal portion (NOT total payments)</li>
                <li data-i18n="view.s483.imp.section_1274">§ 1274 may apply instead for "publicly traded" debt instruments</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s483.h2.exceptions">§ 483 exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s483.exc.de_minimis">De minimis: aggregate ≤ $10K sales of property in a year</li>
                <li data-i18n="view.s483.exc.farms_3m">Farm property: aggregate ≤ $3M sales</li>
                <li data-i18n="view.s483.exc.small_personal">Small personal use property: ≤ $250K sales (e.g., cars)</li>
                <li data-i18n="view.s483.exc.publicly_traded">Publicly traded property: § 1274 may apply instead</li>
                <li data-i18n="view.s483.exc.foreign">Foreign government / international org transactions</li>
                <li data-i18n="view.s483.exc.options">Options to purchase: § 483 not applicable (separate § 1234)</li>
                <li data-i18n="view.s483.exc.short_term">Short-term sales (≤ 1 yr): § 483 generally inapplicable</li>
                <li data-i18n="view.s483.exc.cash_method">Cash method seller + small amounts: may opt out</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s483.h2.coordination">Coordination with other Code sections</h2>
            <ul class="muted small">
                <li data-i18n="view.s483.coord.s453">§ 453 installment method: imputed interest part NOT subject to installment</li>
                <li data-i18n="view.s483.coord.s7872">§ 7872 below-market loans: similar imputation for non-sale loans</li>
                <li data-i18n="view.s483.coord.s1274">§ 1274 debt instruments: applies to PUBLICLY TRADED debt (not § 483)</li>
                <li data-i18n="view.s483.coord.s7702">§ 7702 life insurance contracts: separate test, not § 483</li>
                <li data-i18n="view.s483.coord.s1239">§ 1239 related party: combine with imputed interest analysis</li>
                <li data-i18n="view.s483.coord.s453_b_g">§ 453B(g): transferred installment notes to spouse — tax-free under § 1041</li>
                <li data-i18n="view.s483.coord.estate">Estate planning: low-AFR sale to grantor trust (private annuity sale)</li>
                <li data-i18n="view.s483.coord.s2032a">§ 2032A estate freezes use installment notes with AFR</li>
            </ul>
        </div>
    `;
    document.getElementById('s483-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.sale_price_total = Number(fd.get('sale_price_total')) || 0;
        state.stated_interest_rate_pct = Number(fd.get('stated_interest_rate_pct')) || 0;
        state.afr_test_rate_pct = Number(fd.get('afr_test_rate_pct')) || 0;
        state.payment_term_years = Number(fd.get('payment_term_years')) || 0;
        state.annual_payment = Number(fd.get('annual_payment')) || 0;
        state.installment_sale_election = !!fd.get('installment_sale_election');
        state.is_related_party = !!fd.get('is_related_party');
        state.is_seller_financing = !!fd.get('is_seller_financing');
        state.transaction_type = fd.get('transaction_type');
        state.is_publicly_traded_debt = !!fd.get('is_publicly_traded_debt');
        state.fmv_at_sale = Number(fd.get('fmv_at_sale')) || 0;
        state.seller_cost_basis = Number(fd.get('seller_cost_basis')) || 0;
        state.holding_period_years = Number(fd.get('holding_period_years')) || 0;
        state.sale_date = fd.get('sale_date');
        state.use_short_term_afr = !!fd.get('use_short_term_afr');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s483-output');
    if (!el) return;
    const adequate_interest = state.stated_interest_rate_pct >= state.afr_test_rate_pct;
    const imputed_interest_per_year = !adequate_interest ?
        state.sale_price_total * ((state.afr_test_rate_pct - state.stated_interest_rate_pct) / 100) : 0;
    const total_imputed_lifetime = imputed_interest_per_year * state.payment_term_years;
    const capital_gain_per_year = state.annual_payment - imputed_interest_per_year - ((state.sale_price_total - state.seller_cost_basis) / state.payment_term_years);
    const interest_tax = imputed_interest_per_year * 0.37;
    const capital_gain_tax = capital_gain_per_year * 0.20;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s483.h2.result">§ 483 imputed interest computation</h2>
            <div class="cards">
                <div class="card ${adequate_interest ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s483.card.adequate">Adequate stated interest?</div>
                    <div class="value">${adequate_interest ? esc(t('view.s483.status.yes')) : esc(t('view.s483.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s483.card.stated">Stated rate</div>
                    <div class="value">${state.stated_interest_rate_pct.toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s483.card.afr">Test AFR</div>
                    <div class="value">${state.afr_test_rate_pct.toFixed(2)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s483.card.annual_imputed">Annual imputed interest</div>
                    <div class="value">$${imputed_interest_per_year.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s483.card.lifetime">Lifetime imputed interest</div>
                    <div class="value">$${total_imputed_lifetime.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s483.card.int_tax">Annual interest tax (37%)</div>
                    <div class="value">$${interest_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s483.card.cg_tax">Annual cap gain tax (20%)</div>
                    <div class="value">$${capital_gain_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!adequate_interest && state.is_related_party ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s483.related_note">
                    BELOW-AFR + RELATED PARTY: § 483 imputed interest combined w/ § 7872 below-market loan
                    rules + § 1239 related party rules. IRS scrutiny on private installment sales among family.
                    Document at-least-AFR stated rate. Estate planning structure (intra-family sales) requires
                    careful AFR documentation + annual interest reporting.
                </p>
            ` : ''}
        </div>
    `;
}
