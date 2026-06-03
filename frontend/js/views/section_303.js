// IRC § 303 — Distributions in Redemption of Stock to Pay Death Taxes.
// Allows estate to redeem closely-held stock TAX-FREE (sale treatment) up to amount of estate tax + funeral / admin expenses.
// Pairs with § 6166 (estate tax installment) for closely-held business succession.
// Requires stock value > 35% of adjusted gross estate.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    decedent_estate_value: 0,
    closely_held_stock_value: 0,
    adjusted_gross_estate: 0,
    s303_b_2_35pct_threshold_met: false,
    stock_value_pct_of_age: 0,
    federal_estate_tax: 0,
    state_estate_tax: 0,
    funeral_expenses: 0,
    estate_administration_expenses: 0,
    total_qualifying_expenses: 0,
    s303_redemption_amount: 0,
    excess_over_qualifying: 0,
    s303_b_1_3_year_3_month_timing: false,
    s303_b_1_4_year_timing: false,
    redemption_made_within_period: false,
    s303_c_required_capital_gain: false,
    is_qualifying_redemption: false,
    s302_b_normally_taxed_dividend: false,
    s303_avoidance_of_dividend: false,
    s6166_installment_election: false,
    s6166_2pct_special_interest: false,
    closely_held_corporation_test_met: false,
    s6166_b_1_a_test_more_than_35pct: false,
    decedent_ownership_pct: 0,
    is_active_business: false,
    s6166_b_8_active_business_test: false,
    s6166_15_year_payment_schedule: false,
    s6166_4_year_grace_period: false,
    multiple_corp_aggregation_s303_b3: 0,
    multiple_corps_each_20pct: false,
    estate_includable_interest_total: 0,
    estate_includable_interest_pct: 0,
    surviving_spouse_redemption: false,
    s303_d_1_succession_carryover: false,
};

export async function renderSection303(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s303.h1.title">// § 303 ESTATE TAX REDEMPTION</span></h1>
        <p class="muted small" data-i18n="view.s303.hint.intro">
            <strong>§ 303</strong> allows estate (or beneficiary) to REDEEM closely-held corporation
            stock TAX-FREE (treated as SALE / EXCHANGE rather than § 301 dividend) up to amount of
            ESTATE TAX + ALLOWABLE FUNERAL/ADMINISTRATION EXPENSES. <strong>Threshold:</strong>
            closely-held stock value must EXCEED 35% of ADJUSTED GROSS ESTATE (§ 303(b)(2)(A)).
            <strong>Timing:</strong> redemption within 3 years + 90 days after estate tax payment
            extension (longer windows for § 6166 installment payments). <strong>§ 303(b)(3)
            aggregation:</strong> multiple corps aggregate IF each &gt; 20% of total. <strong>Stepped-up
            basis</strong> at death (§ 1014) means redemption typically yields near-ZERO capital gain.
            <strong>Pairs with § 6166</strong> estate tax 15-yr installment + 4-yr grace + 2% special
            interest rate. <strong>Purpose:</strong> facilitate succession of closely-held business
            without forced sale + avoid § 301 dividend treatment trap.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s303.h2.inputs">Inputs</h2>
            <form id="s303-form" class="inline-form">
                <label><span data-i18n="view.s303.label.estate_val">Estate value ($)</span>
                    <input type="number" step="100000" name="decedent_estate_value" value="${state.decedent_estate_value}"></label>
                <label><span data-i18n="view.s303.label.stock_val">Stock value ($)</span>
                    <input type="number" step="100000" name="closely_held_stock_value" value="${state.closely_held_stock_value}"></label>
                <label><span data-i18n="view.s303.label.age">Adj gross estate ($)</span>
                    <input type="number" step="100000" name="adjusted_gross_estate" value="${state.adjusted_gross_estate}"></label>
                <label><span data-i18n="view.s303.label.35pct">35% threshold met?</span>
                    <input type="checkbox" name="s303_b_2_35pct_threshold_met" ${state.s303_b_2_35pct_threshold_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.pct">Stock % of AGE</span>
                    <input type="number" step="0.1" name="stock_value_pct_of_age" value="${state.stock_value_pct_of_age}"></label>
                <label><span data-i18n="view.s303.label.fed_tax">Federal estate tax ($)</span>
                    <input type="number" step="10000" name="federal_estate_tax" value="${state.federal_estate_tax}"></label>
                <label><span data-i18n="view.s303.label.state_tax">State estate tax ($)</span>
                    <input type="number" step="10000" name="state_estate_tax" value="${state.state_estate_tax}"></label>
                <label><span data-i18n="view.s303.label.funeral">Funeral exp ($)</span>
                    <input type="number" step="1000" name="funeral_expenses" value="${state.funeral_expenses}"></label>
                <label><span data-i18n="view.s303.label.admin">Admin exp ($)</span>
                    <input type="number" step="1000" name="estate_administration_expenses" value="${state.estate_administration_expenses}"></label>
                <label><span data-i18n="view.s303.label.qual_total">Qualifying total ($)</span>
                    <input type="number" step="10000" name="total_qualifying_expenses" value="${state.total_qualifying_expenses}"></label>
                <label><span data-i18n="view.s303.label.redemption">Redemption amount ($)</span>
                    <input type="number" step="10000" name="s303_redemption_amount" value="${state.s303_redemption_amount}"></label>
                <label><span data-i18n="view.s303.label.excess">Excess over qual ($)</span>
                    <input type="number" step="10000" name="excess_over_qualifying" value="${state.excess_over_qualifying}"></label>
                <label><span data-i18n="view.s303.label.3_3_timing">3yr 3mo timing?</span>
                    <input type="checkbox" name="s303_b_1_3_year_3_month_timing" ${state.s303_b_1_3_year_3_month_timing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.4yr">4-yr timing?</span>
                    <input type="checkbox" name="s303_b_1_4_year_timing" ${state.s303_b_1_4_year_timing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.within">Within period?</span>
                    <input type="checkbox" name="redemption_made_within_period" ${state.redemption_made_within_period ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s303c">§ 303(c) cap gain?</span>
                    <input type="checkbox" name="s303_c_required_capital_gain" ${state.s303_c_required_capital_gain ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.qualifying">Qualifying redemption?</span>
                    <input type="checkbox" name="is_qualifying_redemption" ${state.is_qualifying_redemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.normally_div">Normally dividend?</span>
                    <input type="checkbox" name="s302_b_normally_taxed_dividend" ${state.s302_b_normally_taxed_dividend ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s303_div">§ 303 avoids div?</span>
                    <input type="checkbox" name="s303_avoidance_of_dividend" ${state.s303_avoidance_of_dividend ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s6166">§ 6166 election?</span>
                    <input type="checkbox" name="s6166_installment_election" ${state.s6166_installment_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s6166_2pct">§ 6166 2% rate?</span>
                    <input type="checkbox" name="s6166_2pct_special_interest" ${state.s6166_2pct_special_interest ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.closely_held">Closely-held test?</span>
                    <input type="checkbox" name="closely_held_corporation_test_met" ${state.closely_held_corporation_test_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s6166_b1a">§ 6166(b)(1)(A) test?</span>
                    <input type="checkbox" name="s6166_b_1_a_test_more_than_35pct" ${state.s6166_b_1_a_test_more_than_35pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.dec_pct">Decedent %</span>
                    <input type="number" step="0.1" name="decedent_ownership_pct" value="${state.decedent_ownership_pct}"></label>
                <label><span data-i18n="view.s303.label.active">Active business?</span>
                    <input type="checkbox" name="is_active_business" ${state.is_active_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s6166_active">§ 6166(b)(8) active test?</span>
                    <input type="checkbox" name="s6166_b_8_active_business_test" ${state.s6166_b_8_active_business_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s6166_15">§ 6166 15-yr schedule?</span>
                    <input type="checkbox" name="s6166_15_year_payment_schedule" ${state.s6166_15_year_payment_schedule ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s6166_4yr">§ 6166 4-yr grace?</span>
                    <input type="checkbox" name="s6166_4_year_grace_period" ${state.s6166_4_year_grace_period ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.multi_corp">Multi-corp aggregation ($)</span>
                    <input type="number" step="10000" name="multiple_corp_aggregation_s303_b3" value="${state.multiple_corp_aggregation_s303_b3}"></label>
                <label><span data-i18n="view.s303.label.each_20pct">Each ≥ 20%?</span>
                    <input type="checkbox" name="multiple_corps_each_20pct" ${state.multiple_corps_each_20pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.estate_int">Estate includable ($)</span>
                    <input type="number" step="10000" name="estate_includable_interest_total" value="${state.estate_includable_interest_total}"></label>
                <label><span data-i18n="view.s303.label.estate_pct">Estate includable %</span>
                    <input type="number" step="0.1" name="estate_includable_interest_pct" value="${state.estate_includable_interest_pct}"></label>
                <label><span data-i18n="view.s303.label.spouse">Surviving spouse?</span>
                    <input type="checkbox" name="surviving_spouse_redemption" ${state.surviving_spouse_redemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s303.label.s303_d1">§ 303(d)(1) carryover?</span>
                    <input type="checkbox" name="s303_d_1_succession_carryover" ${state.s303_d_1_succession_carryover ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s303.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s303-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s303.h2.requirements">§ 303 statutory requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s303.req.estate_inclusion">Stock includable in decedent's gross estate</li>
                <li data-i18n="view.s303.req.35pct">Stock value &gt; 35% of ADJUSTED gross estate (§ 303(b)(2)(A))</li>
                <li data-i18n="view.s303.req.qualifying">Distribution must be for tax payment / funeral / administration expenses</li>
                <li data-i18n="view.s303.req.timing">Within 3 yrs 90 days OR § 6166 election period</li>
                <li data-i18n="view.s303.req.shareholder">Shareholder must bear estate tax burden (or executor for estate)</li>
                <li data-i18n="view.s303.req.s303_b_3">§ 303(b)(3) — multiple corps treated as 1 if each &gt; 20%</li>
                <li data-i18n="view.s303.req.character">Treated as SALE / EXCHANGE — capital gain (typically minimal with § 1014 step-up)</li>
                <li data-i18n="view.s303.req.no_s302">Avoids § 302 / § 301 dividend treatment entirely</li>
                <li data-i18n="view.s303.req.s303_c">§ 303(c) — character of redemption preserved (capital)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s303.h2.computation">Qualifying redemption amount</h2>
            <ul class="muted small">
                <li data-i18n="view.s303.comp.amount">Amount = SUM of: (a) federal + state DEATH taxes, (b) funeral expenses, (c) admin expenses</li>
                <li data-i18n="view.s303.comp.estate_tax_deduction">Federal estate tax already reduces taxable estate (§ 2053)</li>
                <li data-i18n="view.s303.comp.generation_skip">GST tax included if applicable (§ 2603)</li>
                <li data-i18n="view.s303.comp.deductions_allowed">Admin expenses deductible from estate (§ 2053) — coordination required</li>
                <li data-i18n="view.s303.comp.excess">Excess of redemption over qualifying amount → § 301 dividend (potentially)</li>
                <li data-i18n="view.s303.comp.multiple_redemptions">Multiple redemptions: cumulative limit on qualifying amount</li>
                <li data-i18n="view.s303.comp.partial">Partial redemption: applied to qualifying expenses first, excess separately tested</li>
                <li data-i18n="view.s303.comp.coordination_s6166">Coordination with § 6166: installment payments included as qualifying</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s303.h2.timing">Timing window</h2>
            <ul class="muted small">
                <li data-i18n="view.s303.t.standard">Standard: within 3 YEARS + 90 DAYS after due date of estate tax return</li>
                <li data-i18n="view.s303.t.extension">§ 6161 extension to pay: extended period</li>
                <li data-i18n="view.s303.t.s6166_installment">§ 6166 15-yr installment: redemption may occur during installment period</li>
                <li data-i18n="view.s303.t.4yr">§ 303(b)(1)(B): 4 years if Tax Court petition filed</li>
                <li data-i18n="view.s303.t.s303_b_1_C">§ 303(b)(1)(C): IRS-allowed delay periods</li>
                <li data-i18n="view.s303.t.late_redemption">Late redemption: § 301 dividend treatment (no § 303)</li>
                <li data-i18n="view.s303.t.s303_b_4">§ 303(b)(4): extension during § 6166 deferral period</li>
                <li data-i18n="view.s303.t.estate_tax_paid">Period runs from estate tax actually paid (when § 6166 elected)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s303.h2.aggregation">§ 303(b)(3) multiple corps</h2>
            <ul class="muted small">
                <li data-i18n="view.s303.agg.purpose">Aggregate multiple closely-held corporations as single for 35% test</li>
                <li data-i18n="view.s303.agg.each_20">Each corp must be &gt; 20% of decedent's interest</li>
                <li data-i18n="view.s303.agg.threshold">Combined value must exceed 35% AGE</li>
                <li data-i18n="view.s303.agg.attribution">§ 318 attribution applies to ownership testing</li>
                <li data-i18n="view.s303.agg.s303_b_4">§ 303(b)(4) — surviving spouse joint tenancy treated as decedent's</li>
                <li data-i18n="view.s303.agg.s303_b_5">§ 303(b)(5) — community property treated as decedent's per state law</li>
                <li data-i18n="view.s303.agg.holding_company">Holding company structure: NOT a corporation for § 303 (look-through)</li>
                <li data-i18n="view.s303.agg.s6166_aggregation">§ 6166(c) parallel aggregation rule (45% test for installment)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s303.h2.coordination_s6166">Coordination with § 6166</h2>
            <ul class="muted small">
                <li data-i18n="view.s303.s6166.purpose">§ 6166 — 15-year installment for estate tax on closely-held interest</li>
                <li data-i18n="view.s303.s6166.4yr_grace">4-year grace period before principal payments begin</li>
                <li data-i18n="view.s303.s6166.10yr_installment">10-year installment after grace</li>
                <li data-i18n="view.s303.s6166.2pct_special">§ 6601(j) 2% special interest rate on first ~$1.6M of deferred tax</li>
                <li data-i18n="view.s303.s6166.requirement">Closely-held business value &gt; 35% adjusted gross estate</li>
                <li data-i18n="view.s303.s6166.acceleration">Acceleration triggers: 50%+ business sold, liquidation, defaulted payment</li>
                <li data-i18n="view.s303.s6166.election">Election on Form 706 by extended due date</li>
                <li data-i18n="view.s303.s6166.s303_combined">§ 303 + § 6166 = preserve closely-held business without forced sale</li>
                <li data-i18n="view.s303.s6166.bond">IRS may require lien or bond (§ 6324A) for installments</li>
                <li data-i18n="view.s303.s6166.s2032a">§ 2032A special-use valuation may also apply (additional planning)</li>
            </ul>
        </div>
    `;
    document.getElementById('s303-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.decedent_estate_value = Number(fd.get('decedent_estate_value')) || 0;
        state.closely_held_stock_value = Number(fd.get('closely_held_stock_value')) || 0;
        state.adjusted_gross_estate = Number(fd.get('adjusted_gross_estate')) || 0;
        state.s303_b_2_35pct_threshold_met = !!fd.get('s303_b_2_35pct_threshold_met');
        state.stock_value_pct_of_age = Number(fd.get('stock_value_pct_of_age')) || 0;
        state.federal_estate_tax = Number(fd.get('federal_estate_tax')) || 0;
        state.state_estate_tax = Number(fd.get('state_estate_tax')) || 0;
        state.funeral_expenses = Number(fd.get('funeral_expenses')) || 0;
        state.estate_administration_expenses = Number(fd.get('estate_administration_expenses')) || 0;
        state.total_qualifying_expenses = Number(fd.get('total_qualifying_expenses')) || 0;
        state.s303_redemption_amount = Number(fd.get('s303_redemption_amount')) || 0;
        state.excess_over_qualifying = Number(fd.get('excess_over_qualifying')) || 0;
        state.s303_b_1_3_year_3_month_timing = !!fd.get('s303_b_1_3_year_3_month_timing');
        state.s303_b_1_4_year_timing = !!fd.get('s303_b_1_4_year_timing');
        state.redemption_made_within_period = !!fd.get('redemption_made_within_period');
        state.s303_c_required_capital_gain = !!fd.get('s303_c_required_capital_gain');
        state.is_qualifying_redemption = !!fd.get('is_qualifying_redemption');
        state.s302_b_normally_taxed_dividend = !!fd.get('s302_b_normally_taxed_dividend');
        state.s303_avoidance_of_dividend = !!fd.get('s303_avoidance_of_dividend');
        state.s6166_installment_election = !!fd.get('s6166_installment_election');
        state.s6166_2pct_special_interest = !!fd.get('s6166_2pct_special_interest');
        state.closely_held_corporation_test_met = !!fd.get('closely_held_corporation_test_met');
        state.s6166_b_1_a_test_more_than_35pct = !!fd.get('s6166_b_1_a_test_more_than_35pct');
        state.decedent_ownership_pct = Number(fd.get('decedent_ownership_pct')) || 0;
        state.is_active_business = !!fd.get('is_active_business');
        state.s6166_b_8_active_business_test = !!fd.get('s6166_b_8_active_business_test');
        state.s6166_15_year_payment_schedule = !!fd.get('s6166_15_year_payment_schedule');
        state.s6166_4_year_grace_period = !!fd.get('s6166_4_year_grace_period');
        state.multiple_corp_aggregation_s303_b3 = Number(fd.get('multiple_corp_aggregation_s303_b3')) || 0;
        state.multiple_corps_each_20pct = !!fd.get('multiple_corps_each_20pct');
        state.estate_includable_interest_total = Number(fd.get('estate_includable_interest_total')) || 0;
        state.estate_includable_interest_pct = Number(fd.get('estate_includable_interest_pct')) || 0;
        state.surviving_spouse_redemption = !!fd.get('surviving_spouse_redemption');
        state.s303_d_1_succession_carryover = !!fd.get('s303_d_1_succession_carryover');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s303-output');
    if (!el) return;
    const pct = state.adjusted_gross_estate > 0 ? (state.closely_held_stock_value / state.adjusted_gross_estate) * 100 : 0;
    const threshold_met = pct > 35;
    const qualifying = state.federal_estate_tax + state.state_estate_tax + state.funeral_expenses + state.estate_administration_expenses;
    const qualifying_redemption = Math.min(state.s303_redemption_amount, qualifying);
    const excess = state.s303_redemption_amount > qualifying ? state.s303_redemption_amount - qualifying : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s303.h2.result">§ 303 redemption analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s303.card.pct">Stock % of AGE</div><div class="value">${pct.toFixed(1)}%</div></div>
                <div class="card ${threshold_met ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s303.card.threshold">35% threshold?</div><div class="value">${threshold_met ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s303.card.qualifying">Qualifying expenses</div><div class="value">$${qualifying.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s303.card.redemption">§ 303 sale treatment</div><div class="value">$${qualifying_redemption.toLocaleString()}</div></div>
                <div class="card ${excess > 0 ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s303.card.excess">Excess (§ 301 risk)</div><div class="value">$${excess.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
