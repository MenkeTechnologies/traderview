// IRC § 1273 — Original Issue Discount (OID).
// OID = Stated redemption price at maturity MINUS issue price.
// Accrued ratably over life of bond → INCLUDED IN INCOME each year (even without cash receipt).
// "Zero coupon" bonds: maximum OID since all yield is OID.
// Tax-exempt OID: still accrued + taxed-free (for municipal bonds).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    issue_price: 0,
    stated_redemption_at_maturity: 0,
    years_to_maturity: 0,
    current_holding_period_years: 0,
    stated_interest_rate_pct: 0,
    yield_to_maturity_pct: 0,
    issue_date: '',
    maturity_date: '',
    is_tax_exempt: false,
    is_short_term_obligation: false,
    de_minimis_test: false,
    elect_constant_yield: false,
    is_market_discount_bond: false,
    market_discount_election_s1278: false,
    accrued_oid_to_date: 0,
    is_us_treasury: false,
};

export async function renderSection1273(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1273.h1.title">// § 1273 ORIGINAL ISSUE DISCOUNT</span></h1>
        <p class="muted small" data-i18n="view.s1273.hint.intro">
            <strong>OID = Stated Redemption Price at Maturity − Issue Price</strong>. Accrued ratably over
            life of bond → INCLUDED IN INCOME each year (<strong>even WITHOUT cash receipt</strong> — zero
            coupon problem). <strong>De minimis test:</strong> OID &lt; 0.25% × years to maturity → not OID.
            <strong>Constant yield method</strong> standard (compound basis, like loan amortization).
            <strong>Form 1099-OID</strong> reporting if accrual ≥ $10. <strong>Tax-exempt municipal OID:</strong>
            still accrued but tax-free. <strong>§ 1278 market discount</strong> separate concept (post-issuance).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1273.h2.inputs">Inputs</h2>
            <form id="s1273-form" class="inline-form">
                <label><span data-i18n="view.s1273.label.issue">Issue price ($)</span>
                    <input type="number" step="0.01" name="issue_price" value="${state.issue_price}"></label>
                <label><span data-i18n="view.s1273.label.redemption">Stated redemption at maturity ($)</span>
                    <input type="number" step="0.01" name="stated_redemption_at_maturity" value="${state.stated_redemption_at_maturity}"></label>
                <label><span data-i18n="view.s1273.label.years">Years to maturity (at issue)</span>
                    <input type="number" step="0.5" name="years_to_maturity" value="${state.years_to_maturity}"></label>
                <label><span data-i18n="view.s1273.label.holding">Current holding period years</span>
                    <input type="number" step="0.5" name="current_holding_period_years" value="${state.current_holding_period_years}"></label>
                <label><span data-i18n="view.s1273.label.coupon">Stated interest rate %</span>
                    <input type="number" step="0.01" name="stated_interest_rate_pct" value="${state.stated_interest_rate_pct}"></label>
                <label><span data-i18n="view.s1273.label.ytm">Yield to maturity %</span>
                    <input type="number" step="0.01" name="yield_to_maturity_pct" value="${state.yield_to_maturity_pct}"></label>
                <label><span data-i18n="view.s1273.label.issue_date">Issue date</span>
                    <input type="date" name="issue_date" value="${state.issue_date}"></label>
                <label><span data-i18n="view.s1273.label.maturity">Maturity date</span>
                    <input type="date" name="maturity_date" value="${state.maturity_date}"></label>
                <label><span data-i18n="view.s1273.label.tax_exempt">Tax-exempt municipal?</span>
                    <input type="checkbox" name="is_tax_exempt" ${state.is_tax_exempt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1273.label.short_term">Short-term ≤ 1 yr?</span>
                    <input type="checkbox" name="is_short_term_obligation" ${state.is_short_term_obligation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1273.label.de_minimis">De minimis test passes?</span>
                    <input type="checkbox" name="de_minimis_test" ${state.de_minimis_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1273.label.constant_yield">Elect constant yield method?</span>
                    <input type="checkbox" name="elect_constant_yield" ${state.elect_constant_yield ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1273.label.market_discount">Market discount bond?</span>
                    <input type="checkbox" name="is_market_discount_bond" ${state.is_market_discount_bond ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1273.label.s1278">§ 1278(b) MD accrual election?</span>
                    <input type="checkbox" name="market_discount_election_s1278" ${state.market_discount_election_s1278 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1273.label.accrued">Accrued OID to date ($)</span>
                    <input type="number" step="0.01" name="accrued_oid_to_date" value="${state.accrued_oid_to_date}"></label>
                <label><span data-i18n="view.s1273.label.treasury">US Treasury?</span>
                    <input type="checkbox" name="is_us_treasury" ${state.is_us_treasury ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1273.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1273-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1273.h2.computation">Constant yield computation</h2>
            <ol class="muted small">
                <li data-i18n="view.s1273.comp.step1">Step 1: Find Yield to Maturity (YTM) that makes PV of cash flows = issue price</li>
                <li data-i18n="view.s1273.comp.step2">Step 2: Period accrual = (adjusted issue price × YTM) − stated interest</li>
                <li data-i18n="view.s1273.comp.step3">Step 3: Adjusted issue price t+1 = adjusted issue price t + OID accrual</li>
                <li data-i18n="view.s1273.comp.step4">Step 4: Cumulative accrual = sum of all period accruals</li>
                <li data-i18n="view.s1273.comp.step5">Step 5: Holder's basis = original cost + cumulative OID accrual</li>
                <li data-i18n="view.s1273.comp.zero_coupon">Zero coupon: stated interest = 0; all accrual is OID</li>
                <li data-i18n="view.s1273.comp.daily">Daily accrual: prorate the period accrual</li>
                <li data-i18n="view.s1273.comp.partial_year">Partial-year holding: prorate based on days held</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1273.h2.de_minimis">De minimis test</h2>
            <ul class="muted small">
                <li data-i18n="view.s1273.dm.formula">De minimis = stated redemption × 0.25% × years to maturity</li>
                <li data-i18n="view.s1273.dm.example">Example: $1,000 par × 0.25% × 10 yrs = $25 de minimis threshold</li>
                <li data-i18n="view.s1273.dm.below">If actual OID &lt; de minimis → NOT treated as OID</li>
                <li data-i18n="view.s1273.dm.character">Below-threshold: gain on sale = capital gain (not ordinary OID)</li>
                <li data-i18n="view.s1273.dm.installment_obligation">Installment obligations: de minimis = 0.5% × years</li>
                <li data-i18n="view.s1273.dm.partial_principle">Partial principal payments bonds: 1.75% × years</li>
                <li data-i18n="view.s1273.dm.purpose">Purpose: prevent OID complexity on small discounts</li>
                <li data-i18n="view.s1273.dm.practical">Practical: short-maturity bonds rarely have significant OID</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1273.h2.special_situations">Special situations</h2>
            <ul class="muted small">
                <li data-i18n="view.s1273.spec.tax_exempt">Tax-exempt OID: accrue normally but income excluded (Box 11 of 1099-OID)</li>
                <li data-i18n="view.s1273.spec.short_term">Short-term obligations ≤ 1 yr: no OID accrual; gain at sale = ordinary</li>
                <li data-i18n="view.s1273.spec.treasury_oid">US Treasury OID: state tax exempt (Box 8 of 1099-OID)</li>
                <li data-i18n="view.s1273.spec.zero_coupon_strips">Zero-coupon Treasury STRIPS: substantial annual OID accrual problem</li>
                <li data-i18n="view.s1273.spec.high_yield_oid">High Yield OID Discount Obligations § 163(e)(5): payer deduction limited</li>
                <li data-i18n="view.s1273.spec.aha">Applicable High-Yield Discount Obligation: > 5 yrs maturity + > AFR + 5%</li>
                <li data-i18n="view.s1273.spec.modification">§ 1.1001-3: significant modification of debt = deemed new instrument with new OID</li>
                <li data-i18n="view.s1273.spec.distress">Distressed debt: deep discount may be OID, not market discount</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1273.h2.oid_vs_market_discount">OID vs Market Discount (§ 1278)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1273.th.aspect">Aspect</th>
                    <th data-i18n="view.s1273.th.oid">OID (§ 1273)</th>
                    <th data-i18n="view.s1273.th.md">Market Discount (§ 1278)</th>
                </tr></thead>
                <tbody>
                    <tr><td>Origin</td><td>Bought at issuance (or above issue price)</td><td>Bought in secondary market below par</td></tr>
                    <tr><td>Required accrual</td><td>Yes — annual income inclusion (§ 1272)</td><td>Optional — § 1278(b) election</td></tr>
                    <tr><td>Default character at sale</td><td>Ordinary income (OID accrued)</td><td>Ordinary up to MD; capital above</td></tr>
                    <tr><td>De minimis</td><td>0.25% × years to maturity</td><td>0.25% × years to maturity (from purchase)</td></tr>
                    <tr><td>Tax-exempt OID</td><td>Tax-free if muni</td><td>Tax-free if muni</td></tr>
                    <tr><td>Reporting</td><td>1099-OID Box 1</td><td>1099-INT Box 10 (if exit)</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s1273-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.issue_price = Number(fd.get('issue_price')) || 0;
        state.stated_redemption_at_maturity = Number(fd.get('stated_redemption_at_maturity')) || 0;
        state.years_to_maturity = Number(fd.get('years_to_maturity')) || 0;
        state.current_holding_period_years = Number(fd.get('current_holding_period_years')) || 0;
        state.stated_interest_rate_pct = Number(fd.get('stated_interest_rate_pct')) || 0;
        state.yield_to_maturity_pct = Number(fd.get('yield_to_maturity_pct')) || 0;
        state.issue_date = fd.get('issue_date');
        state.maturity_date = fd.get('maturity_date');
        state.is_tax_exempt = !!fd.get('is_tax_exempt');
        state.is_short_term_obligation = !!fd.get('is_short_term_obligation');
        state.de_minimis_test = !!fd.get('de_minimis_test');
        state.elect_constant_yield = !!fd.get('elect_constant_yield');
        state.is_market_discount_bond = !!fd.get('is_market_discount_bond');
        state.market_discount_election_s1278 = !!fd.get('market_discount_election_s1278');
        state.accrued_oid_to_date = Number(fd.get('accrued_oid_to_date')) || 0;
        state.is_us_treasury = !!fd.get('is_us_treasury');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1273-output');
    if (!el) return;
    const total_oid = state.stated_redemption_at_maturity - state.issue_price;
    const de_minimis_threshold = state.stated_redemption_at_maturity * 0.0025 * state.years_to_maturity;
    const is_de_minimis = total_oid < de_minimis_threshold;
    const annual_accrual_straight = state.years_to_maturity > 0 ? total_oid / state.years_to_maturity : 0;
    const current_year_inclusion = is_de_minimis || state.is_short_term_obligation ? 0 : annual_accrual_straight;
    const tax_on_current = state.is_tax_exempt ? 0 : current_year_inclusion * 0.37;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1273.h2.result">§ 1273 OID computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1273.card.total_oid">Total OID</div>
                    <div class="value">$${total_oid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1273.card.threshold">De minimis threshold</div>
                    <div class="value">$${de_minimis_threshold.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${is_de_minimis ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s1273.card.de_minimis">De minimis?</div>
                    <div class="value">${is_de_minimis ? esc(t('view.s1273.status.yes')) : esc(t('view.s1273.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1273.card.annual">Annual accrual (straight line)</div>
                    <div class="value">$${annual_accrual_straight.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1273.card.current_year">Current year inclusion</div>
                    <div class="value">$${current_year_inclusion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1273.card.tax">Current tax (37%)</div>
                    <div class="value">$${tax_on_current.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${current_year_inclusion > 0 && state.stated_interest_rate_pct === 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1273.zero_coupon_note">
                    ZERO COUPON BOND: substantial annual OID accrual without cash receipts. Pay tax NOW on
                    income you HAVEN'T received. Common problem for retail investors. Consider holding
                    zero-coupon bonds in tax-deferred accounts (IRA, 401(k)) OR tax-exempt accounts (Roth IRA).
                    Tax-exempt municipal zero-coupons (e.g., Build America Bonds, BABs): solved this issue.
                </p>
            ` : ''}
        </div>
    `;
}
