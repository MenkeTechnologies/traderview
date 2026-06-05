// IRC § 1276 + § 1278 — Market Discount on Bonds.
// Purchased bond below issue price (market discount): gain on sale / accrued at maturity =
// ORDINARY INCOME (not cap gain). De minimis: < 0.25% × yrs to maturity. Choose:
// (1) ratable accrual + recognize as interest annually, OR (2) constant yield method.
// § 1278(b) election: include in income currently → reduce capital gain at sale.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const DE_MINIMIS_PCT_PER_YR = 0.0025;

let state = {
    face_value: 0,
    purchase_price: 0,
    years_to_maturity: 0,
    coupon_rate: 0,
    yield_to_maturity: 0,
    years_held: 0,
    election_1278b: false,
    is_tax_exempt_bond: false,
    marginal_rate: 0.32,
    ltcg_rate: 0.20,
};

export async function renderSection1276(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1276.h1.title">// § 1276 MARKET DISCOUNT</span></h1>
        <p class="muted small" data-i18n="view.s1276.hint.intro">
            Buy bond below face value: <strong>accrued discount = ORDINARY income</strong>, not
            cap gain. <strong>De minimis safe harbor:</strong> &lt; 0.25% × years to maturity.
            Accrual method: ratable OR constant-yield. <strong>§ 1278(b) election:</strong>
            include currently → reduce gain at sale. Tax-exempt muni bonds: market discount also
            ordinary income (federal), but not always state. Reported on 1099-OID or 1099-INT.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1276.h2.inputs">Inputs</h2>
            <form id="s1276-form" class="inline-form">
                <label><span data-i18n="view.s1276.label.face">Face value ($)</span>
                    <input type="number" step="0.01" name="face_value" value="${state.face_value}"></label>
                <label><span data-i18n="view.s1276.label.purchase">Purchase price ($)</span>
                    <input type="number" step="0.01" name="purchase_price" value="${state.purchase_price}"></label>
                <label><span data-i18n="view.s1276.label.maturity">Years to maturity at purchase</span>
                    <input type="number" step="0.25" name="years_to_maturity" value="${state.years_to_maturity}"></label>
                <label><span data-i18n="view.s1276.label.coupon">Coupon rate</span>
                    <input type="number" step="0.001" name="coupon_rate" value="${state.coupon_rate}"></label>
                <label><span data-i18n="view.s1276.label.ytm">Yield to maturity</span>
                    <input type="number" step="0.001" name="yield_to_maturity" value="${state.yield_to_maturity}"></label>
                <label><span data-i18n="view.s1276.label.held">Years held</span>
                    <input type="number" step="0.25" name="years_held" value="${state.years_held}"></label>
                <label><span data-i18n="view.s1276.label.election">§ 1278(b) election (current inclusion)?</span>
                    <input type="checkbox" name="election_1278b" ${state.election_1278b ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1276.label.muni">Tax-exempt municipal bond?</span>
                    <input type="checkbox" name="is_tax_exempt_bond" ${state.is_tax_exempt_bond ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1276.label.marginal">Marginal ordinary %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s1276.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s1276.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1276-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1276.h2.related">Related bond doctrines</h2>
            <ul class="muted small">
                <li data-i18n="view.s1276.rel.oid">Original Issue Discount (§ 1271-1275): bonds issued below par; OID accrues as interest</li>
                <li data-i18n="view.s1276.rel.amortizable">Bond Premium § 171: paid > face = elect to amortize (offset interest)</li>
                <li data-i18n="view.s1276.rel.acquisition">§ 1278(a)(2)(B) Acquisition Premium: amortize against OID accrual</li>
                <li data-i18n="view.s1276.rel.short_term">Short-term (≤ 1 yr maturity): § 1281 mandatory current accrual</li>
                <li data-i18n="view.s1276.rel.stripped">Stripped bonds (Treasury STRIPS): § 1286 OID treatment</li>
                <li data-i18n="view.s1276.rel.contingent">Contingent payment debt: Reg § 1.1275-4 OID-like accrual</li>
            </ul>
        </div>
    `;
    document.getElementById('s1276-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.face_value = Number(fd.get('face_value')) || 0;
        state.purchase_price = Number(fd.get('purchase_price')) || 0;
        state.years_to_maturity = Number(fd.get('years_to_maturity')) || 0;
        state.coupon_rate = Number(fd.get('coupon_rate')) || 0;
        state.yield_to_maturity = Number(fd.get('yield_to_maturity')) || 0;
        state.years_held = Number(fd.get('years_held')) || 0;
        state.election_1278b = !!fd.get('election_1278b');
        state.is_tax_exempt_bond = !!fd.get('is_tax_exempt_bond');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1276-output');
    if (!el) return;
    const totalDiscount = Math.max(0, state.face_value - state.purchase_price);
    const deMinimisAllowance = state.face_value * DE_MINIMIS_PCT_PER_YR * state.years_to_maturity;
    const isDeMinimis = totalDiscount <= deMinimisAllowance;
    const ratableAccrued = state.years_to_maturity > 0
        ? totalDiscount * (state.years_held / state.years_to_maturity)
        : 0;
    const accruedOrdinary = isDeMinimis ? 0 : ratableAccrued;
    const remainingDiscountAtMaturity = totalDiscount - accruedOrdinary;
    const ordinaryTax = accruedOrdinary * state.marginal_rate;
    const capGainAtSale = Math.max(0, state.face_value - state.purchase_price - accruedOrdinary);
    const capGainTax = capGainAtSale * state.ltcg_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1276.h2.result">Tax breakdown</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1276.card.discount">Total market discount</div>
                    <div class="value">$${totalDiscount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1276.card.de_minimis_allow">De minimis allowance</div>
                    <div class="value">$${deMinimisAllowance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${isDeMinimis ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1276.card.de_minimis">Within de minimis</div>
                    <div class="value">${isDeMinimis ? esc(t('view.s1276.status.yes')) : esc(t('view.s1276.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1276.card.accrued_ord">Accrued as ordinary</div>
                    <div class="value">$${accruedOrdinary.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1276.card.remaining">Remaining at maturity</div>
                    <div class="value">$${remainingDiscountAtMaturity.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1276.card.ord_tax">Ordinary tax</div>
                    <div class="value">$${ordinaryTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1276.card.cap_gain">Remaining cap gain</div>
                    <div class="value">$${capGainAtSale.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1276.card.cap_tax">Cap gain tax</div>
                    <div class="value">$${capGainTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
