// IRC § 170 — Charitable Contributions Deduction.
// Itemized deduction with AGI percentage limits varying by donee + property type.
// 60% AGI for cash to public charities (50% pre-2018 + 2026+).
// 30% AGI for cash to private foundations.
// 30% AGI for appreciated long-term capital gain property to public charities (20% to PFs).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    donor_agi: 0,
    cash_to_public_charity: 0,
    cash_to_private_foundation: 0,
    ltcg_property_to_public: 0,
    ltcg_property_to_pf: 0,
    ordinary_income_property: 0,
    ltcg_property_fmv: 0,
    ltcg_property_basis: 0,
    qualified_appreciated_stock: 0,
    is_dafs_donation: false,
    daf_amount: 0,
    is_qcd: false,
    qcd_amount: 0,
    s170b_election_basis_only: false,
    is_partial_interest: false,
    crat_amount: 0,
    clat_amount: 0,
    bargain_sale_amount: 0,
    bargain_sale_fmv: 0,
    quid_pro_quo: false,
    benefit_value: 0,
    is_substantiation_compliant: false,
    is_appraisal_required: false,
    is_form_8283_filed: false,
    receipt_obtained: false,
    carryover_to_next_year_pct: 0,
    prior_year_carryover: 0,
    year: 2024,
};

export async function renderSection170(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s170.h1.title">// § 170 CHARITABLE CONTRIBUTIONS</span></h1>
        <p class="muted small" data-i18n="view.s170.hint.intro">
            <strong>Itemized deduction</strong> for contributions to qualified organizations.
            <strong>AGI limits:</strong> 60% cash to public charity (2018-2025, then back to 50%) /
            30% cash to private foundation / 30% LTCG property to public / 20% LTCG to PF.
            <strong>Carryover:</strong> 5 years. <strong>Substantiation:</strong> $250+ requires
            written acknowledgment; non-cash > $500 → Form 8283; > $5,000 → qualified appraisal;
            > $500,000 → attach appraisal. <strong>QCD § 408(d)(8):</strong> $105K direct IRA-to-charity
            counts as RMD (NOT deductible but no income inclusion). <strong>Bargain sale § 1011(b):</strong>
            allocates basis between sale + gift portions.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s170.h2.inputs">Inputs</h2>
            <form id="s170-form" class="inline-form">
                <label><span data-i18n="view.s170.label.agi">Donor AGI ($)</span>
                    <input type="number" step="10000" name="donor_agi" value="${state.donor_agi}"></label>
                <label><span data-i18n="view.s170.label.cash_public">Cash to public charity ($)</span>
                    <input type="number" step="1000" name="cash_to_public_charity" value="${state.cash_to_public_charity}"></label>
                <label><span data-i18n="view.s170.label.cash_pf">Cash to private foundation ($)</span>
                    <input type="number" step="1000" name="cash_to_private_foundation" value="${state.cash_to_private_foundation}"></label>
                <label><span data-i18n="view.s170.label.ltcg_pub">LTCG property to public ($)</span>
                    <input type="number" step="1000" name="ltcg_property_to_public" value="${state.ltcg_property_to_public}"></label>
                <label><span data-i18n="view.s170.label.ltcg_pf">LTCG property to PF ($)</span>
                    <input type="number" step="1000" name="ltcg_property_to_pf" value="${state.ltcg_property_to_pf}"></label>
                <label><span data-i18n="view.s170.label.ordinary">Ordinary income property ($)</span>
                    <input type="number" step="1000" name="ordinary_income_property" value="${state.ordinary_income_property}"></label>
                <label><span data-i18n="view.s170.label.ltcg_fmv">LTCG FMV ($)</span>
                    <input type="number" step="1000" name="ltcg_property_fmv" value="${state.ltcg_property_fmv}"></label>
                <label><span data-i18n="view.s170.label.ltcg_basis">LTCG basis ($)</span>
                    <input type="number" step="1000" name="ltcg_property_basis" value="${state.ltcg_property_basis}"></label>
                <label><span data-i18n="view.s170.label.qas">Qualified appreciated stock ($)</span>
                    <input type="number" step="1000" name="qualified_appreciated_stock" value="${state.qualified_appreciated_stock}"></label>
                <label><span data-i18n="view.s170.label.daf">DAF donation?</span>
                    <input type="checkbox" name="is_dafs_donation" ${state.is_dafs_donation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.daf_amt">DAF amount ($)</span>
                    <input type="number" step="1000" name="daf_amount" value="${state.daf_amount}"></label>
                <label><span data-i18n="view.s170.label.qcd">QCD?</span>
                    <input type="checkbox" name="is_qcd" ${state.is_qcd ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.qcd_amt">QCD amount ($)</span>
                    <input type="number" step="1000" name="qcd_amount" value="${state.qcd_amount}"></label>
                <label><span data-i18n="view.s170.label.basis_election">§ 170(b)(1)(C)(iii) basis election?</span>
                    <input type="checkbox" name="s170b_election_basis_only" ${state.s170b_election_basis_only ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.partial">Partial interest?</span>
                    <input type="checkbox" name="is_partial_interest" ${state.is_partial_interest ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.crat">CRAT amount ($)</span>
                    <input type="number" step="10000" name="crat_amount" value="${state.crat_amount}"></label>
                <label><span data-i18n="view.s170.label.clat">CLAT amount ($)</span>
                    <input type="number" step="10000" name="clat_amount" value="${state.clat_amount}"></label>
                <label><span data-i18n="view.s170.label.bargain">Bargain sale amount ($)</span>
                    <input type="number" step="10000" name="bargain_sale_amount" value="${state.bargain_sale_amount}"></label>
                <label><span data-i18n="view.s170.label.bargain_fmv">Bargain sale FMV ($)</span>
                    <input type="number" step="10000" name="bargain_sale_fmv" value="${state.bargain_sale_fmv}"></label>
                <label><span data-i18n="view.s170.label.qpq">Quid pro quo?</span>
                    <input type="checkbox" name="quid_pro_quo" ${state.quid_pro_quo ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.benefit">Benefit value ($)</span>
                    <input type="number" step="100" name="benefit_value" value="${state.benefit_value}"></label>
                <label><span data-i18n="view.s170.label.subst">Substantiated?</span>
                    <input type="checkbox" name="is_substantiation_compliant" ${state.is_substantiation_compliant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.appraisal">Appraisal required?</span>
                    <input type="checkbox" name="is_appraisal_required" ${state.is_appraisal_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.f8283">Form 8283 filed?</span>
                    <input type="checkbox" name="is_form_8283_filed" ${state.is_form_8283_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.receipt">Receipt obtained?</span>
                    <input type="checkbox" name="receipt_obtained" ${state.receipt_obtained ? 'checked' : ''}></label>
                <label><span data-i18n="view.s170.label.carry_pct">Carryover %</span>
                    <input type="number" step="1" name="carryover_to_next_year_pct" value="${state.carryover_to_next_year_pct}"></label>
                <label><span data-i18n="view.s170.label.prior_carry">Prior year carryover ($)</span>
                    <input type="number" step="1000" name="prior_year_carryover" value="${state.prior_year_carryover}"></label>
                <label><span data-i18n="view.s170.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <button class="primary" type="submit" data-i18n="view.s170.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s170-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s170.h2.limits">AGI percentage limits</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s170.tbl.type">Gift type</th><th data-i18n="view.s170.tbl.public">To 50% charity (public)</th><th data-i18n="view.s170.tbl.pf">To 30% charity (PF)</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s170.tbl.cash">Cash (2018-2025)</td><td>60% AGI</td><td>30% AGI</td></tr>
                    <tr><td data-i18n="view.s170.tbl.cash_post">Cash (post-2025 or pre-2018)</td><td>50% AGI</td><td>30% AGI</td></tr>
                    <tr><td data-i18n="view.s170.tbl.ltcg">LTCG property (FMV)</td><td>30% AGI</td><td>20% AGI</td></tr>
                    <tr><td data-i18n="view.s170.tbl.ltcg_basis">LTCG property (basis election)</td><td>50% AGI</td><td>30% AGI</td></tr>
                    <tr><td data-i18n="view.s170.tbl.ordinary">Ordinary income property</td><td>50% AGI (basis only)</td><td>30% AGI (basis only)</td></tr>
                    <tr><td data-i18n="view.s170.tbl.qas">Qualified appreciated stock to PF</td><td>N/A</td><td>20% AGI (FMV)</td></tr>
                    <tr><td data-i18n="view.s170.tbl.carryover">Carryover period</td><td>5 years</td><td>5 years</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s170.h2.substantiation">Substantiation requirements</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s170.tbl.amount">Amount</th><th data-i18n="view.s170.tbl.requirement">Requirement</th></tr></thead>
                <tbody>
                    <tr><td>Under $250</td><td data-i18n="view.s170.tbl.bank_record">Bank record or written communication</td></tr>
                    <tr><td>$250+</td><td data-i18n="view.s170.tbl.contemporaneous">Contemporaneous written acknowledgment from donee</td></tr>
                    <tr><td data-i18n="view.s170.tbl.non_cash_500">$500+ non-cash</td><td>Form 8283 Section A</td></tr>
                    <tr><td data-i18n="view.s170.tbl.non_cash_5k">$5,000+ non-cash</td><td>Form 8283 Section B + qualified appraisal</td></tr>
                    <tr><td data-i18n="view.s170.tbl.non_cash_500k">$500,000+ non-cash</td><td>Attach appraisal to return</td></tr>
                    <tr><td data-i18n="view.s170.tbl.s170f8">§ 170(f)(8) acknowledgment</td><td data-i18n="view.s170.tbl.ack_content">Must state amount + whether donor received goods/services + value</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s170.h2.daf">Donor-Advised Funds (DAF)</h2>
            <ul class="muted small">
                <li data-i18n="view.s170.daf.s4966">§ 4966 supporting organization treatment</li>
                <li data-i18n="view.s170.daf.deduction_timing">Deduction in year of DAF contribution (NOT distribution)</li>
                <li data-i18n="view.s170.daf.no_partial">Donor retains advisory rights, NOT control — § 170(f)(18)</li>
                <li data-i18n="view.s170.daf.qas_eligible">Qualified appreciated stock to DAF: FMV deduction (treated as public)</li>
                <li data-i18n="view.s170.daf.s4943">§ 4943(e) post-2007 PRIVATE BUSINESS HOLDINGS limit</li>
                <li data-i18n="view.s170.daf.s4958">§ 4958 excise tax on disqualified person benefits</li>
                <li data-i18n="view.s170.daf.bunching">"Bunching" strategy: front-load multiple years into one</li>
                <li data-i18n="view.s170.daf.standard_alternative">Useful when SALT cap + standard deduction limit benefit of small donations</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s170.h2.qcd">QCD § 408(d)(8)</h2>
            <ul class="muted small">
                <li data-i18n="view.s170.qcd.age">Age 70½+ required</li>
                <li data-i18n="view.s170.qcd.amount">2024 limit: $105,000 (indexed)</li>
                <li data-i18n="view.s170.qcd.no_daf">Cannot go to DAF or supporting organization</li>
                <li data-i18n="view.s170.qcd.satisfies_rmd">Counts toward RMD (if age 73+)</li>
                <li data-i18n="view.s170.qcd.no_deduction">NOT deductible — excludes from AGI instead</li>
                <li data-i18n="view.s170.qcd.medicare">AGI reduction benefits: Medicare IRMAA + SS taxation + § 1411 NIIT</li>
                <li data-i18n="view.s170.qcd.s_qcd_split">SECURE 2.0: $50K one-time QCD to CRT or CGA (counted within limit)</li>
                <li data-i18n="view.s170.qcd.gold_age">Strategy: gold for high-income retirees with no itemized benefit</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s170.h2.split">Split-interest gifts</h2>
            <ul class="muted small">
                <li data-i18n="view.s170.split.crat">CRAT: charitable remainder annuity trust — fixed annuity to non-charity</li>
                <li data-i18n="view.s170.split.crut">CRUT: charitable remainder unitrust — % of FMV revalued annually</li>
                <li data-i18n="view.s170.split.crat_min_pct">5%-50% annual payment; 10% remainder minimum (Rev. Rul. 77-374)</li>
                <li data-i18n="view.s170.split.clat">CLAT: charitable lead annuity — annuity to charity first, remainder to family</li>
                <li data-i18n="view.s170.split.clut">CLUT: charitable lead unitrust</li>
                <li data-i18n="view.s170.split.pooled">Pooled income fund (PIF) — donor's gift pooled with others</li>
                <li data-i18n="view.s170.split.qci">§ 170(f)(2) Qualified Conservation Easement — perpetual restriction</li>
                <li data-i18n="view.s170.split.gst">GST tax may apply to non-charitable remainderpersons</li>
                <li data-i18n="view.s170.split.partial_interest">§ 170(f)(3) generally disallows partial interest (split-interest exception)</li>
            </ul>
        </div>
    `;
    document.getElementById('s170-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.donor_agi = Number(fd.get('donor_agi')) || 0;
        state.cash_to_public_charity = Number(fd.get('cash_to_public_charity')) || 0;
        state.cash_to_private_foundation = Number(fd.get('cash_to_private_foundation')) || 0;
        state.ltcg_property_to_public = Number(fd.get('ltcg_property_to_public')) || 0;
        state.ltcg_property_to_pf = Number(fd.get('ltcg_property_to_pf')) || 0;
        state.ordinary_income_property = Number(fd.get('ordinary_income_property')) || 0;
        state.ltcg_property_fmv = Number(fd.get('ltcg_property_fmv')) || 0;
        state.ltcg_property_basis = Number(fd.get('ltcg_property_basis')) || 0;
        state.qualified_appreciated_stock = Number(fd.get('qualified_appreciated_stock')) || 0;
        state.is_dafs_donation = !!fd.get('is_dafs_donation');
        state.daf_amount = Number(fd.get('daf_amount')) || 0;
        state.is_qcd = !!fd.get('is_qcd');
        state.qcd_amount = Number(fd.get('qcd_amount')) || 0;
        state.s170b_election_basis_only = !!fd.get('s170b_election_basis_only');
        state.is_partial_interest = !!fd.get('is_partial_interest');
        state.crat_amount = Number(fd.get('crat_amount')) || 0;
        state.clat_amount = Number(fd.get('clat_amount')) || 0;
        state.bargain_sale_amount = Number(fd.get('bargain_sale_amount')) || 0;
        state.bargain_sale_fmv = Number(fd.get('bargain_sale_fmv')) || 0;
        state.quid_pro_quo = !!fd.get('quid_pro_quo');
        state.benefit_value = Number(fd.get('benefit_value')) || 0;
        state.is_substantiation_compliant = !!fd.get('is_substantiation_compliant');
        state.is_appraisal_required = !!fd.get('is_appraisal_required');
        state.is_form_8283_filed = !!fd.get('is_form_8283_filed');
        state.receipt_obtained = !!fd.get('receipt_obtained');
        state.carryover_to_next_year_pct = Number(fd.get('carryover_to_next_year_pct')) || 0;
        state.prior_year_carryover = Number(fd.get('prior_year_carryover')) || 0;
        state.year = Number(fd.get('year')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s170-output');
    if (!el) return;
    const limit_60 = state.donor_agi * 0.60;
    const limit_30 = state.donor_agi * 0.30;
    const limit_20 = state.donor_agi * 0.20;
    const cash_pub_allowed = Math.min(state.cash_to_public_charity, limit_60);
    const cash_pf_allowed = Math.min(state.cash_to_private_foundation, limit_30);
    const ltcg_pub_allowed = Math.min(state.ltcg_property_to_public, limit_30);
    const ltcg_pf_allowed = Math.min(state.ltcg_property_to_pf, limit_20);
    const total_deduction = cash_pub_allowed + cash_pf_allowed + ltcg_pub_allowed + ltcg_pf_allowed;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s170.h2.result">§ 170 deduction</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s170.card.cash_pub">Cash → public</div><div class="value">$${cash_pub_allowed.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s170.card.cash_pf">Cash → PF</div><div class="value">$${cash_pf_allowed.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s170.card.ltcg_pub">LTCG → public</div><div class="value">$${ltcg_pub_allowed.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s170.card.ltcg_pf">LTCG → PF</div><div class="value">$${ltcg_pf_allowed.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s170.card.total">Total § 170 deduction</div><div class="value">$${total_deduction.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
