// IRC § 1250 — Depreciation Recapture on Real Property.
// Recaptures EXCESS over straight-line depreciation as ordinary income.
// Post-1986 (MACRS) real property uses straight-line only — § 1250 recapture effectively ZERO.
// BUT § 1(h)(6) "unrecaptured § 1250 gain" taxed at MAX 25% capital gain rate.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    property_type: 'residential_rental',
    original_cost: 0,
    land_basis: 0,
    accumulated_depreciation: 0,
    straight_line_equivalent: 0,
    excess_over_sl: 0,
    sale_proceeds: 0,
    adjusted_basis: 0,
    realized_gain: 0,
    unrecaptured_s1250_gain: 0,
    s1250_recapture_ordinary: 0,
    s1231_capital_balance: 0,
    holding_period_months: 12,
    is_corporate: false,
    s291_corporate_add: 0,
    pre_1986_property: false,
    pre_acrs_pre_1981: false,
    is_qualified_real_property: false,
    qrip_subject: false,
    is_residential_rental: true,
    is_nonresidential: false,
    cost_segregation_split: false,
    s1245_carved_out: 0,
    placed_in_service_date: '',
    sale_date: '',
    s1031_like_kind: false,
    s453_installment: false,
    deferred_gain: 0,
    s1411_niit_applies: false,
    short_sale_against_box: false,
};

export async function renderSection1250(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1250.h1.title">// § 1250 REAL PROPERTY RECAPTURE</span></h1>
        <p class="muted small" data-i18n="view.s1250.hint.intro">
            <strong>§ 1250</strong> recaptures gain on disposition of depreciable real property as
            ORDINARY INCOME only to extent of EXCESS over straight-line depreciation.
            <strong>Post-1986</strong> property (MACRS): straight-line ONLY allowed → § 1250 recapture
            effectively ZERO. <strong>UNRECAPTURED § 1250 GAIN</strong> (§ 1(h)(6)) — the prior
            straight-line depreciation taxed at MAXIMUM 25% capital gain rate (vs 15%/20% LTCG).
            <strong>§ 291 corporate add-on:</strong> 20% of (amount that would be § 1245 if § 1245
            applied) treated as additional ordinary income for C-corps. <strong>Cost segregation:</strong>
            carves out § 1245 components (machinery, fixtures, equipment) — those get FULL § 1245
            recapture treatment. <strong>§ 1231 capital gain</strong> for excess (LTCG if &gt; 1 year).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1250.h2.inputs">Inputs</h2>
            <form id="s1250-form" class="inline-form">
                <label><span data-i18n="view.s1250.label.type">Property type</span>
                    <select name="property_type">
                        <option value="residential_rental" ${state.property_type === 'residential_rental' ? 'selected' : ''}>Residential rental (27.5 yr)</option>
                        <option value="nonresidential" ${state.property_type === 'nonresidential' ? 'selected' : ''}>Nonresidential (39 yr)</option>
                        <option value="qualified_improvement" ${state.property_type === 'qualified_improvement' ? 'selected' : ''}>§ 168(e)(6) QIP (15 yr)</option>
                        <option value="leasehold_improvement" ${state.property_type === 'leasehold_improvement' ? 'selected' : ''}>Leasehold improvement</option>
                        <option value="retail_improvement" ${state.property_type === 'retail_improvement' ? 'selected' : ''}>Retail improvement</option>
                        <option value="restaurant_improvement" ${state.property_type === 'restaurant_improvement' ? 'selected' : ''}>Restaurant improvement</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1250.label.cost">Original cost ($)</span>
                    <input type="number" step="10000" name="original_cost" value="${state.original_cost}"></label>
                <label><span data-i18n="view.s1250.label.land">Land basis ($)</span>
                    <input type="number" step="10000" name="land_basis" value="${state.land_basis}"></label>
                <label><span data-i18n="view.s1250.label.accum">Accumulated depreciation ($)</span>
                    <input type="number" step="1000" name="accumulated_depreciation" value="${state.accumulated_depreciation}"></label>
                <label><span data-i18n="view.s1250.label.sl">Straight-line equivalent ($)</span>
                    <input type="number" step="1000" name="straight_line_equivalent" value="${state.straight_line_equivalent}"></label>
                <label><span data-i18n="view.s1250.label.excess">Excess over SL ($)</span>
                    <input type="number" step="1000" name="excess_over_sl" value="${state.excess_over_sl}"></label>
                <label><span data-i18n="view.s1250.label.proceeds">Sale proceeds ($)</span>
                    <input type="number" step="10000" name="sale_proceeds" value="${state.sale_proceeds}"></label>
                <label><span data-i18n="view.s1250.label.basis">Adjusted basis ($)</span>
                    <input type="number" step="10000" name="adjusted_basis" value="${state.adjusted_basis}"></label>
                <label><span data-i18n="view.s1250.label.gain">Realized gain ($)</span>
                    <input type="number" step="10000" name="realized_gain" value="${state.realized_gain}"></label>
                <label><span data-i18n="view.s1250.label.unrec">Unrecaptured § 1250 gain ($)</span>
                    <input type="number" step="10000" name="unrecaptured_s1250_gain" value="${state.unrecaptured_s1250_gain}"></label>
                <label><span data-i18n="view.s1250.label.ord">§ 1250 ordinary recapture ($)</span>
                    <input type="number" step="10000" name="s1250_recapture_ordinary" value="${state.s1250_recapture_ordinary}"></label>
                <label><span data-i18n="view.s1250.label.cap">§ 1231 capital balance ($)</span>
                    <input type="number" step="10000" name="s1231_capital_balance" value="${state.s1231_capital_balance}"></label>
                <label><span data-i18n="view.s1250.label.holding">Holding (months)</span>
                    <input type="number" step="1" name="holding_period_months" value="${state.holding_period_months}"></label>
                <label><span data-i18n="view.s1250.label.corp">Corporate?</span>
                    <input type="checkbox" name="is_corporate" ${state.is_corporate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.s291">§ 291 corporate add ($)</span>
                    <input type="number" step="1000" name="s291_corporate_add" value="${state.s291_corporate_add}"></label>
                <label><span data-i18n="view.s1250.label.pre1986">Pre-1986 property?</span>
                    <input type="checkbox" name="pre_1986_property" ${state.pre_1986_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.acrs">Pre-ACRS pre-1981?</span>
                    <input type="checkbox" name="pre_acrs_pre_1981" ${state.pre_acrs_pre_1981 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.qualified">Qualified real property?</span>
                    <input type="checkbox" name="is_qualified_real_property" ${state.is_qualified_real_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.qrip">QRIP subject?</span>
                    <input type="checkbox" name="qrip_subject" ${state.qrip_subject ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.residential">Residential rental?</span>
                    <input type="checkbox" name="is_residential_rental" ${state.is_residential_rental ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.non_res">Nonresidential?</span>
                    <input type="checkbox" name="is_nonresidential" ${state.is_nonresidential ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.cost_seg">Cost segregation split?</span>
                    <input type="checkbox" name="cost_segregation_split" ${state.cost_segregation_split ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.s1245">§ 1245 carved out ($)</span>
                    <input type="number" step="1000" name="s1245_carved_out" value="${state.s1245_carved_out}"></label>
                <label><span data-i18n="view.s1250.label.placed">Placed in service date</span>
                    <input type="date" name="placed_in_service_date" value="${state.placed_in_service_date}"></label>
                <label><span data-i18n="view.s1250.label.sold">Sale date</span>
                    <input type="date" name="sale_date" value="${state.sale_date}"></label>
                <label><span data-i18n="view.s1250.label.s1031">§ 1031 like-kind?</span>
                    <input type="checkbox" name="s1031_like_kind" ${state.s1031_like_kind ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.installment">§ 453 installment?</span>
                    <input type="checkbox" name="s453_installment" ${state.s453_installment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.deferred">Deferred gain ($)</span>
                    <input type="number" step="10000" name="deferred_gain" value="${state.deferred_gain}"></label>
                <label><span data-i18n="view.s1250.label.niit">§ 1411 NIIT applies?</span>
                    <input type="checkbox" name="s1411_niit_applies" ${state.s1411_niit_applies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1250.label.short_box">Short sale against box?</span>
                    <input type="checkbox" name="short_sale_against_box" ${state.short_sale_against_box ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1250.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1250-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1250.h2.macrs">MACRS recovery periods</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s1250.tbl.property">Property</th><th data-i18n="view.s1250.tbl.period">Period</th><th data-i18n="view.s1250.tbl.method">Method</th><th data-i18n="view.s1250.tbl.note">Note</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s1250.tbl.residential">Residential rental (incl apartments)</td><td>27.5 yr</td><td>SL</td><td data-i18n="view.s1250.tbl.s168_c">§ 168(c) MM convention</td></tr>
                    <tr><td data-i18n="view.s1250.tbl.nonresidential">Nonresidential (offices, retail, industrial)</td><td>39 yr</td><td>SL</td><td data-i18n="view.s1250.tbl.s168_c_2">§ 168(c) MM convention</td></tr>
                    <tr><td data-i18n="view.s1250.tbl.qip">QIP (qualified improvement property)</td><td>15 yr</td><td>SL</td><td data-i18n="view.s1250.tbl.cares_fix">CARES Act 2020 retroactive fix</td></tr>
                    <tr><td data-i18n="view.s1250.tbl.ads">ADS (alternative depreciation)</td><td>30 yr res / 40 yr non-res</td><td>SL</td><td data-i18n="view.s1250.tbl.elective">Election or required</td></tr>
                    <tr><td data-i18n="view.s1250.tbl.land">Land</td><td data-i18n="view.s1250.tbl.no_dep">N/A</td><td>—</td><td data-i18n="view.s1250.tbl.no_depreciable">Not depreciable</td></tr>
                    <tr><td data-i18n="view.s1250.tbl.pre_1981">Pre-1981 buildings</td><td>Useful life</td><td>SL or 200% DDB</td><td>§ 167 facts &amp; circumstances</td></tr>
                    <tr><td data-i18n="view.s1250.tbl.acrs">1981-1986 ACRS</td><td>15 yr / 18 yr / 19 yr</td><td>175% DDB</td><td>SIGNIFICANT § 1250 recapture for these years</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1250.h2.unrec">Unrecaptured § 1250 gain (25% rate)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1250.unr.amount">Amount = post-1986 straight-line depreciation that ALREADY claimed</li>
                <li data-i18n="view.s1250.unr.rate">Taxed at MAXIMUM 25% — vs 15%/20% LTCG, vs 37% ordinary</li>
                <li data-i18n="view.s1250.unr.s1h6">§ 1(h)(6) special rate</li>
                <li data-i18n="view.s1250.unr.individuals">Individuals only — corporations taxed at flat 21% anyway</li>
                <li data-i18n="view.s1250.unr.netting">Netted against capital losses BEFORE applying rate (Schedule D Worksheet)</li>
                <li data-i18n="view.s1250.unr.calculation">= LESSER of (depreciation taken) OR (gain on sale)</li>
                <li data-i18n="view.s1250.unr.niit">§ 1411 NIIT 3.8% adds on top → effective 28.8% max</li>
                <li data-i18n="view.s1250.unr.s_residential">Residential rental 27.5 yr: $100K depreciation after 27.5 yrs full</li>
                <li data-i18n="view.s1250.unr.recapture_total">Total tax burden on long-held real estate disposition can be significant</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1250.h2.s291">§ 291 corporate add-on</h2>
            <ul class="muted small">
                <li data-i18n="view.s1250.s291.amount">20% × amount that WOULD be § 1245 recapture if § 1245 applied</li>
                <li data-i18n="view.s1250.s291.corp_only">C-corporations only — not individuals or S-corps</li>
                <li data-i18n="view.s1250.s291.calculation">Formula: 20% × (cumulative depreciation taken)</li>
                <li data-i18n="view.s1250.s291.ordinary">Treated as additional ordinary income (not capital gain)</li>
                <li data-i18n="view.s1250.s291.s1239">Combined with § 1250 ordinary recapture (if any)</li>
                <li data-i18n="view.s1250.s291.effective_rate">Effective: 21% × 20% = 4.2% additional federal tax on corporate sales</li>
                <li data-i18n="view.s1250.s291.book_tax">Book vs tax difference — § 291 creates temporary BTD</li>
                <li data-i18n="view.s1250.s291.s1250_corp">Corporate § 1250 + § 291 = approximate § 1245 treatment</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1250.h2.strategies">Strategies + planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s1250.strat.s1031">§ 1031 like-kind exchange defers gain + unrecaptured § 1250 gain</li>
                <li data-i18n="view.s1250.strat.s453">§ 453 installment sale spreads recognition across receipts</li>
                <li data-i18n="view.s1250.strat.cost_seg">Cost segregation accelerates dep (15-yr) but creates § 1245 recapture exposure</li>
                <li data-i18n="view.s1250.strat.s1033">§ 1033 involuntary conversion defers gain + recapture potential preserved</li>
                <li data-i18n="view.s1250.strat.opportunity_zone">QOZ deferral § 1400Z-2 — basis step-up after 10 years</li>
                <li data-i18n="view.s1250.strat.charitable_remainder">CRT split-interest: defer gain over income payment period</li>
                <li data-i18n="view.s1250.strat.death">§ 1014 stepped-up basis at death extinguishes unrealized recapture</li>
                <li data-i18n="view.s1250.strat.gifts">§ 1015 carryover basis on gift preserves recapture potential</li>
                <li data-i18n="view.s1250.strat.partnership">Hold through partnership: § 743(b) step-up may reset for new partners</li>
            </ul>
        </div>
    `;
    document.getElementById('s1250-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.property_type = fd.get('property_type');
        state.original_cost = Number(fd.get('original_cost')) || 0;
        state.land_basis = Number(fd.get('land_basis')) || 0;
        state.accumulated_depreciation = Number(fd.get('accumulated_depreciation')) || 0;
        state.straight_line_equivalent = Number(fd.get('straight_line_equivalent')) || 0;
        state.excess_over_sl = Number(fd.get('excess_over_sl')) || 0;
        state.sale_proceeds = Number(fd.get('sale_proceeds')) || 0;
        state.adjusted_basis = Number(fd.get('adjusted_basis')) || 0;
        state.realized_gain = Number(fd.get('realized_gain')) || 0;
        state.unrecaptured_s1250_gain = Number(fd.get('unrecaptured_s1250_gain')) || 0;
        state.s1250_recapture_ordinary = Number(fd.get('s1250_recapture_ordinary')) || 0;
        state.s1231_capital_balance = Number(fd.get('s1231_capital_balance')) || 0;
        state.holding_period_months = Number(fd.get('holding_period_months')) || 0;
        state.is_corporate = !!fd.get('is_corporate');
        state.s291_corporate_add = Number(fd.get('s291_corporate_add')) || 0;
        state.pre_1986_property = !!fd.get('pre_1986_property');
        state.pre_acrs_pre_1981 = !!fd.get('pre_acrs_pre_1981');
        state.is_qualified_real_property = !!fd.get('is_qualified_real_property');
        state.qrip_subject = !!fd.get('qrip_subject');
        state.is_residential_rental = !!fd.get('is_residential_rental');
        state.is_nonresidential = !!fd.get('is_nonresidential');
        state.cost_segregation_split = !!fd.get('cost_segregation_split');
        state.s1245_carved_out = Number(fd.get('s1245_carved_out')) || 0;
        state.placed_in_service_date = fd.get('placed_in_service_date') || '';
        state.sale_date = fd.get('sale_date') || '';
        state.s1031_like_kind = !!fd.get('s1031_like_kind');
        state.s453_installment = !!fd.get('s453_installment');
        state.deferred_gain = Number(fd.get('deferred_gain')) || 0;
        state.s1411_niit_applies = !!fd.get('s1411_niit_applies');
        state.short_sale_against_box = !!fd.get('short_sale_against_box');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1250-output');
    if (!el) return;
    const realized = state.sale_proceeds - state.adjusted_basis;
    const ordinary_recapture = Math.min(Math.max(0, realized), state.excess_over_sl);
    const unrecaptured = Math.min(Math.max(0, realized - ordinary_recapture), state.straight_line_equivalent);
    const s1231 = Math.max(0, realized - ordinary_recapture - unrecaptured);
    const s291 = state.is_corporate ? (state.accumulated_depreciation * 0.20) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1250.h2.result">§ 1250 + unrecaptured + § 1231</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s1250.card.realized">Realized gain</div><div class="value">$${realized.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s1250.card.ord">§ 1250 ordinary (excess SL)</div><div class="value">$${ordinary_recapture.toLocaleString()}</div></div>
                <div class="card warn"><div class="label" data-i18n="view.s1250.card.unrec">Unrecaptured § 1250 (25%)</div><div class="value">$${unrecaptured.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s1250.card.s1231">§ 1231 LTCG</div><div class="value">$${s1231.toLocaleString()}</div></div>
                <div class="card ${s291 > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.s1250.card.s291">§ 291 corp add</div><div class="value">$${s291.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
