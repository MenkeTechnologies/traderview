// IRC § 2503 — Gift Tax Annual Exclusion + Future Interest Doctrine.
// $18,000 / donee / year (2024); $19,000 (2025) — indexed annually.
// Must be PRESENT INTEREST gift; future interest excluded from annual exclusion.
// § 2503(b): general annual exclusion.
// § 2503(c): minor's trusts (Crummey trusts).
// § 2503(e): unlimited medical / educational direct payments.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    total_gift_amount: 0,
    annual_exclusion_per_donee: 19_000,
    number_of_donees: 1,
    is_spouse_donor: false,
    spousal_gift_split: false,
    is_present_interest: true,
    is_future_interest: false,
    is_crummey_trust: false,
    crummey_withdrawal_period_days: 30,
    educational_direct_payment: 0,
    medical_direct_payment: 0,
    lifetime_exemption_used: 0,
    lifetime_exemption_2025: 13_990_000,
    s529_5_year_election: false,
    qualified_terminable_interest: false,
};

export async function renderSection2503(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s2503.h1.title">// § 2503 GIFT ANNUAL EXCLUSION</span></h1>
        <p class="muted small" data-i18n="view.s2503.hint.intro">
            <strong>$18,000</strong> / donee / year (2024); <strong>$19,000</strong> (2025) — indexed annually.
            Must be <strong>PRESENT INTEREST</strong> gift; future interest excluded from annual exclusion.
            <strong>§ 2503(b):</strong> general annual exclusion. <strong>§ 2503(c):</strong> minor's trusts
            (Crummey trusts). <strong>§ 2503(e):</strong> <strong>UNLIMITED</strong> medical / educational direct
            payments. <strong>Spousal gift splitting</strong>: $36K / $38K. <strong>§ 529 plan 5-year
            election:</strong> $90K / $95K front-loaded annual exclusion. <strong>Form 709</strong> required
            if &gt; annual exclusion.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s2503.h2.inputs">Inputs</h2>
            <form id="s2503-form" class="inline-form">
                <label><span data-i18n="view.s2503.label.amount">Total gift amount ($)</span>
                    <input type="number" step="1000" name="total_gift_amount" value="${state.total_gift_amount}"></label>
                <label><span data-i18n="view.s2503.label.exclusion">Annual exclusion / donee ($)</span>
                    <input type="number" step="500" name="annual_exclusion_per_donee" value="${state.annual_exclusion_per_donee}"></label>
                <label><span data-i18n="view.s2503.label.donees">Number of donees</span>
                    <input type="number" step="1" name="number_of_donees" value="${state.number_of_donees}"></label>
                <label><span data-i18n="view.s2503.label.spouse">Spouse donor?</span>
                    <input type="checkbox" name="is_spouse_donor" ${state.is_spouse_donor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2503.label.split">Spousal gift splitting?</span>
                    <input type="checkbox" name="spousal_gift_split" ${state.spousal_gift_split ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2503.label.present">Present interest?</span>
                    <input type="checkbox" name="is_present_interest" ${state.is_present_interest ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2503.label.future">Future interest?</span>
                    <input type="checkbox" name="is_future_interest" ${state.is_future_interest ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2503.label.crummey">Crummey trust?</span>
                    <input type="checkbox" name="is_crummey_trust" ${state.is_crummey_trust ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2503.label.crummey_days">Crummey withdrawal period (days)</span>
                    <input type="number" step="1" name="crummey_withdrawal_period_days" value="${state.crummey_withdrawal_period_days}"></label>
                <label><span data-i18n="view.s2503.label.educational">Educational direct payment ($)</span>
                    <input type="number" step="100" name="educational_direct_payment" value="${state.educational_direct_payment}"></label>
                <label><span data-i18n="view.s2503.label.medical">Medical direct payment ($)</span>
                    <input type="number" step="100" name="medical_direct_payment" value="${state.medical_direct_payment}"></label>
                <label><span data-i18n="view.s2503.label.lifetime_used">Lifetime exemption used ($)</span>
                    <input type="number" step="10000" name="lifetime_exemption_used" value="${state.lifetime_exemption_used}"></label>
                <label><span data-i18n="view.s2503.label.lifetime_2025">Lifetime exemption 2025 ($)</span>
                    <input type="number" step="10000" name="lifetime_exemption_2025" value="${state.lifetime_exemption_2025}"></label>
                <label><span data-i18n="view.s2503.label.s529">§ 529 5-year election?</span>
                    <input type="checkbox" name="s529_5_year_election" ${state.s529_5_year_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2503.label.qtip">QTIP qualifying trust?</span>
                    <input type="checkbox" name="qualified_terminable_interest" ${state.qualified_terminable_interest ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s2503.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s2503-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2503.h2.amounts">Annual exclusion amounts</h2>
            <ul class="muted small">
                <li data-i18n="view.s2503.amt.2024">2024: $18,000 / donee / year</li>
                <li data-i18n="view.s2503.amt.2025">2025: $19,000 / donee / year (indexed)</li>
                <li data-i18n="view.s2503.amt.spouse_split">Spousal gift splitting (consent both spouses): $36K / $38K combined</li>
                <li data-i18n="view.s2503.amt.s529_forward">§ 529 plan 5-year election: $90K / $95K front-loaded (cannot exceed in next 4 yrs)</li>
                <li data-i18n="view.s2503.amt.non_us_spouse">Non-US citizen spouse: $185K (2024), $190K (2025) (vs unlimited marital deduction)</li>
                <li data-i18n="view.s2503.amt.lifetime_estate">Lifetime estate + gift exemption 2024: $13.61M / 2025: $13.99M (combined unified)</li>
                <li data-i18n="view.s2503.amt.gst">GST exemption: $13.61M / $13.99M (separate from estate + gift)</li>
                <li data-i18n="view.s2503.amt.sunset">2026 sunset: lifetime exemption potentially drops ~$7M if TCJA expires</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2503.h2.present_vs_future">Present vs Future Interest</h2>
            <ul class="muted small">
                <li data-i18n="view.s2503.pf.present">PRESENT INTEREST: donee has immediate, unrestricted right to use / enjoy / possess</li>
                <li data-i18n="view.s2503.pf.future">FUTURE INTEREST: donee's enjoyment delayed (e.g., remainder after life estate)</li>
                <li data-i18n="view.s2503.pf.crummey">Crummey trust: future interest converted to present via Crummey withdrawal right</li>
                <li data-i18n="view.s2503.pf.s529">§ 529 plan: present interest by statute (§ 529(c)(2)(B))</li>
                <li data-i18n="view.s2503.pf.minor_trust">§ 2503(c) minor's trust: future converted via specific rules (age 21 vesting)</li>
                <li data-i18n="view.s2503.pf.life_estate">Life estate to grandchild: future interest, no annual exclusion</li>
                <li data-i18n="view.s2503.pf.discretionary_distrib">Discretionary distrib: future interest unless Crummey power exists</li>
                <li data-i18n="view.s2503.pf.actuarial">Actuarial valuation for split-interest gifts (§ 7520 tables)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2503.h2.crummey">Crummey Trusts + Withdrawal Powers</h2>
            <ul class="muted small">
                <li data-i18n="view.s2503.cr.theory">Theory: brief withdrawal right makes future interest = present interest</li>
                <li data-i18n="view.s2503.cr.notification">Crummey notice: beneficiary must be NOTIFIED of withdrawal right</li>
                <li data-i18n="view.s2503.cr.duration">Duration: 30 days typical; minimum 30 days IRS approved</li>
                <li data-i18n="view.s2503.cr.amount">Withdrawal amount: ≤ lesser of $5K or 5% of trust (5-or-5 rule § 678)</li>
                <li data-i18n="view.s2503.cr.lapse">If lapses ≥ 5-or-5 amount → grantor power retained → § 678</li>
                <li data-i18n="view.s2503.cr.commissioner_v_crummey">Crummey v. Comm'r (9th Cir. 1968): authorized this technique</li>
                <li data-i18n="view.s2503.cr.estate_planning">Estate planning workhorse: ILITs, family trusts</li>
                <li data-i18n="view.s2503.cr.documentation">Document: written notice + retain copies + actually withhold withdrawal if requested</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2503.h2.s2503_e">§ 2503(e) — Unlimited educational + medical</h2>
            <ul class="muted small">
                <li data-i18n="view.s2503.e.educational">Education: tuition payments DIRECTLY to school (any level)</li>
                <li data-i18n="view.s2503.e.medical">Medical: payments DIRECTLY to medical provider (any qualifying medical)</li>
                <li data-i18n="view.s2503.e.no_limit">NO LIMIT on amount</li>
                <li data-i18n="view.s2503.e.direct_only">DIRECT payment required — pay school / hospital, NOT donee</li>
                <li data-i18n="view.s2503.e.tuition_only">Education: tuition only; NOT room + board, books, fees</li>
                <li data-i18n="view.s2503.e.medical_qualifying">Medical: § 213 qualifying expenses</li>
                <li data-i18n="view.s2503.e.combined_strategy">Combined: $19K annual + unlimited § 2503(e) = optimal grandparent gifting</li>
                <li data-i18n="view.s2503.e.alternative">Alternative: 529 plan for general education (more flexible)</li>
            </ul>
        </div>
    `;
    document.getElementById('s2503-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_gift_amount = Number(fd.get('total_gift_amount')) || 0;
        state.annual_exclusion_per_donee = Number(fd.get('annual_exclusion_per_donee')) || 0;
        state.number_of_donees = Number(fd.get('number_of_donees')) || 0;
        state.is_spouse_donor = !!fd.get('is_spouse_donor');
        state.spousal_gift_split = !!fd.get('spousal_gift_split');
        state.is_present_interest = !!fd.get('is_present_interest');
        state.is_future_interest = !!fd.get('is_future_interest');
        state.is_crummey_trust = !!fd.get('is_crummey_trust');
        state.crummey_withdrawal_period_days = Number(fd.get('crummey_withdrawal_period_days')) || 0;
        state.educational_direct_payment = Number(fd.get('educational_direct_payment')) || 0;
        state.medical_direct_payment = Number(fd.get('medical_direct_payment')) || 0;
        state.lifetime_exemption_used = Number(fd.get('lifetime_exemption_used')) || 0;
        state.lifetime_exemption_2025 = Number(fd.get('lifetime_exemption_2025')) || 0;
        state.s529_5_year_election = !!fd.get('s529_5_year_election');
        state.qualified_terminable_interest = !!fd.get('qualified_terminable_interest');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s2503-output');
    if (!el) return;
    const split_multiplier = state.spousal_gift_split ? 2 : 1;
    const annual_exclusion_total = state.annual_exclusion_per_donee * state.number_of_donees * split_multiplier;
    const s529_multiplier = state.s529_5_year_election ? 5 : 1;
    const s529_exclusion = state.s529_5_year_election ? state.annual_exclusion_per_donee * s529_multiplier * state.number_of_donees * split_multiplier : 0;
    const present_qualifies = state.is_present_interest || state.is_crummey_trust;
    const annual_excluded = present_qualifies ? Math.min(state.total_gift_amount, annual_exclusion_total) : 0;
    const unlimited_excluded = state.educational_direct_payment + state.medical_direct_payment;
    const taxable_gift = Math.max(0, state.total_gift_amount - annual_excluded - unlimited_excluded);
    const lifetime_remaining = Math.max(0, state.lifetime_exemption_2025 - state.lifetime_exemption_used);
    const uses_lifetime = Math.min(taxable_gift, lifetime_remaining);
    const taxable_after_lifetime = Math.max(0, taxable_gift - uses_lifetime);
    const gift_tax = taxable_after_lifetime * 0.40;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s2503.h2.result">§ 2503 gift tax computation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s2503.card.annual">Annual exclusion total</div>
                    <div class="value">$${annual_exclusion_total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2503.card.s529">§ 529 5-yr exclusion</div>
                    <div class="value">$${s529_exclusion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2503.card.unlimited">Unlimited § 2503(e)</div>
                    <div class="value">$${unlimited_excluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${taxable_gift > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s2503.card.taxable">Taxable gift</div>
                    <div class="value">$${taxable_gift.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s2503.card.uses_lifetime">Uses lifetime exemption</div>
                    <div class="value">$${uses_lifetime.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s2503.card.remaining">Lifetime remaining</div>
                    <div class="value">$${(lifetime_remaining - uses_lifetime).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s2503.card.tax">Gift tax (40%)</div>
                    <div class="value">$${gift_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_future_interest && !state.is_crummey_trust ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s2503.future_note">
                    FUTURE INTEREST: annual exclusion NOT available. Entire gift uses lifetime exemption (or
                    triggers tax if exhausted). Workarounds: (1) Crummey withdrawal right, (2) § 2503(c)
                    minor's trust until age 21, (3) outright gift instead of trust, (4) § 529 plan (statutory
                    present interest exception).
                </p>
            ` : ''}
        </div>
    `;
}
