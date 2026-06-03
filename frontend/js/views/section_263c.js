// IRC § 263(c) — Intangible Drilling Costs (IDC) Election.
// Operators / working interest owners can ELECT to deduct IDCs CURRENTLY (vs capitalize).
// IDCs: labor, fuel, repairs, hauling, supplies in drilling / development of wells.
// Independent producers + royalty owners: 100% deductible. Integrated oil cos: 70% deductible / 30% amortized 60 months.
// AMT preference item: excess IDC over 65% of net oil + gas income.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    idc_total: 0,
    is_independent_producer: true,
    is_integrated_oil_co: false,
    is_royalty_owner: false,
    is_working_interest: true,
    net_oil_gas_income: 0,
    well_type: 'developmental',
    elect_current_deduction: true,
    amt_taxable_income: 0,
    section_469_passive: false,
    s291_recapture_amount: 0,
    geological_geophysical_amount: 0,
    well_dry_hole: false,
    production_year: 2024,
    enhanced_oil_recovery: false,
};

export async function renderSection263C(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s263c.h1.title">// § 263(c) IDC ELECTION</span></h1>
        <p class="muted small" data-i18n="view.s263c.hint.intro">
            Operators / working interest owners can <strong>ELECT</strong> to deduct <strong>Intangible Drilling
            Costs (IDC) CURRENTLY</strong> instead of capitalizing. <strong>IDCs:</strong> labor, fuel, repairs,
            hauling, supplies in <strong>drilling / development</strong> of wells. <strong>Independent producers</strong>
            + royalty owners: <strong>100% deductible</strong>. <strong>Integrated oil cos:</strong> 70% deductible
            / 30% amortized 60 months. <strong>AMT preference:</strong> excess IDC over 65% of net oil + gas
            income. <strong>§ 1254 recapture</strong> upon disposition. Reg § 1.612-4 election.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s263c.h2.inputs">Inputs</h2>
            <form id="s263c-form" class="inline-form">
                <label><span data-i18n="view.s263c.label.idc">IDC total ($)</span>
                    <input type="number" step="10000" name="idc_total" value="${state.idc_total}"></label>
                <label><span data-i18n="view.s263c.label.independent">Independent producer?</span>
                    <input type="checkbox" name="is_independent_producer" ${state.is_independent_producer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s263c.label.integrated">Integrated oil co?</span>
                    <input type="checkbox" name="is_integrated_oil_co" ${state.is_integrated_oil_co ? 'checked' : ''}></label>
                <label><span data-i18n="view.s263c.label.royalty">Royalty owner (non-working)?</span>
                    <input type="checkbox" name="is_royalty_owner" ${state.is_royalty_owner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s263c.label.working">Working interest owner?</span>
                    <input type="checkbox" name="is_working_interest" ${state.is_working_interest ? 'checked' : ''}></label>
                <label><span data-i18n="view.s263c.label.net_income">Net oil + gas income ($)</span>
                    <input type="number" step="10000" name="net_oil_gas_income" value="${state.net_oil_gas_income}"></label>
                <label><span data-i18n="view.s263c.label.well_type">Well type</span>
                    <select name="well_type">
                        <option value="developmental" ${state.well_type === 'developmental' ? 'selected' : ''}>Developmental</option>
                        <option value="exploratory" ${state.well_type === 'exploratory' ? 'selected' : ''}>Exploratory</option>
                        <option value="wildcat" ${state.well_type === 'wildcat' ? 'selected' : ''}>Wildcat</option>
                        <option value="enhanced_recovery" ${state.well_type === 'enhanced_recovery' ? 'selected' : ''}>Enhanced recovery</option>
                        <option value="offshore" ${state.well_type === 'offshore' ? 'selected' : ''}>Offshore</option>
                        <option value="horizontal" ${state.well_type === 'horizontal' ? 'selected' : ''}>Horizontal / fracking</option>
                    </select>
                </label>
                <label><span data-i18n="view.s263c.label.elect">Elect current deduction?</span>
                    <input type="checkbox" name="elect_current_deduction" ${state.elect_current_deduction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s263c.label.amt">AMT taxable income ($)</span>
                    <input type="number" step="10000" name="amt_taxable_income" value="${state.amt_taxable_income}"></label>
                <label><span data-i18n="view.s263c.label.passive">§ 469 passive activity?</span>
                    <input type="checkbox" name="section_469_passive" ${state.section_469_passive ? 'checked' : ''}></label>
                <label><span data-i18n="view.s263c.label.recapture">§ 1254 recapture amount ($)</span>
                    <input type="number" step="10000" name="s291_recapture_amount" value="${state.s291_recapture_amount}"></label>
                <label><span data-i18n="view.s263c.label.gg">Geological + geophysical ($)</span>
                    <input type="number" step="10000" name="geological_geophysical_amount" value="${state.geological_geophysical_amount}"></label>
                <label><span data-i18n="view.s263c.label.dry">Dry hole?</span>
                    <input type="checkbox" name="well_dry_hole" ${state.well_dry_hole ? 'checked' : ''}></label>
                <label><span data-i18n="view.s263c.label.year">Production year</span>
                    <input type="number" step="1" name="production_year" value="${state.production_year}"></label>
                <label><span data-i18n="view.s263c.label.enhanced">Enhanced oil recovery?</span>
                    <input type="checkbox" name="enhanced_oil_recovery" ${state.enhanced_oil_recovery ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s263c.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s263c-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s263c.h2.idc_components">IDC components</h2>
            <ul class="muted small">
                <li data-i18n="view.s263c.comp.labor">Labor: drilling crew, supervision, geological labor</li>
                <li data-i18n="view.s263c.comp.fuel">Fuel + power: drilling rig operation</li>
                <li data-i18n="view.s263c.comp.repairs">Repairs to drilling equipment</li>
                <li data-i18n="view.s263c.comp.hauling">Hauling: trucking, cementing, casing services</li>
                <li data-i18n="view.s263c.comp.supplies">Supplies: drilling mud, fluids, chemicals, abrasives</li>
                <li data-i18n="view.s263c.comp.contractor">Contract drilling services</li>
                <li data-i18n="view.s263c.comp.dry_hole">Dry hole costs: §263(c) currently deductible (or capitalize)</li>
                <li data-i18n="view.s263c.comp.tangible">EXCLUDED: tangible equipment (capitalize + depreciate 7-yr MACRS)</li>
                <li data-i18n="view.s263c.comp.geological">EXCLUDED: geological + geophysical (separate § 167(h) 24-mo amortization)</li>
                <li data-i18n="view.s263c.comp.lease_costs">EXCLUDED: lease acquisition costs (§ 263 capitalize)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s263c.h2.treatment_categories">Treatment by taxpayer category</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s263c.th.type">Taxpayer type</th>
                    <th data-i18n="view.s263c.th.deductible">Current deduction</th>
                    <th data-i18n="view.s263c.th.amortize">Amortize portion</th>
                </tr></thead>
                <tbody>
                    <tr><td>Independent producer (75K bbl/day cap)</td><td>100%</td><td>None</td></tr>
                    <tr><td>Royalty owner (working interest)</td><td>100%</td><td>None</td></tr>
                    <tr><td>Integrated oil company</td><td>70%</td><td>30% over 60 months</td></tr>
                    <tr><td>Royalty owner (non-working)</td><td>Capitalize as part of lease cost</td><td>§ 1254 depletion</td></tr>
                    <tr><td>Foreign IDC</td><td>10-year amortization (§ 263(i))</td><td>—</td></tr>
                    <tr><td>EOR enhanced oil recovery</td><td>§ 43 credit (separate)</td><td>15% of qualifying costs</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s263c.h2.amt">§ 57(a)(2) AMT preference</h2>
            <ul class="muted small">
                <li data-i18n="view.s263c.amt.preference">AMT preference item: excess of IDC deduction over IDC-amortized basis</li>
                <li data-i18n="view.s263c.amt.formula">Preference = current IDC deduction - (current IDC if amortized over 10 yrs)</li>
                <li data-i18n="view.s263c.amt.65pct">Adjustment limited to 65% of net oil + gas income for the year</li>
                <li data-i18n="view.s263c.amt.independent_exemption">Independent producers: § 57(a)(2)(E) exempt from preference (post-1992)</li>
                <li data-i18n="view.s263c.amt.integrated">Integrated oil co: still subject to AMT preference</li>
                <li data-i18n="view.s263c.amt.regular_tax">Preference adds back to AMTI; not a tax — feeds into AMT calculation</li>
                <li data-i18n="view.s263c.amt.individual_corp">Both individual + corp AMT (TCJA repealed corp AMT 2018-2022; now CAMT)</li>
                <li data-i18n="view.s263c.amt.s56">§ 56(g) related: passive activity loss disallowance for IDC</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s263c.h2.recapture">§ 1254 recapture upon disposition</h2>
            <ul class="muted small">
                <li data-i18n="view.s263c.rec.purpose">Recapture intangible costs deducted on disposition (parallel § 1245 / § 1250)</li>
                <li data-i18n="view.s263c.rec.amount">Ordinary income up to amount of prior IDC + depletion deductions</li>
                <li data-i18n="view.s263c.rec.basis">Recapture basis: cumulative IDC + depletion above mineral basis</li>
                <li data-i18n="view.s263c.rec.character">Character: ordinary income vs § 1231 capital</li>
                <li data-i18n="view.s263c.rec.disposition_types">Sale, exchange, abandonment, distribution all trigger</li>
                <li data-i18n="view.s263c.rec.holding_period_gain">Beyond recapture: § 1231 gain (capital) vs § 1245 if equipment recapture</li>
                <li data-i18n="view.s263c.rec.related_party">Related party: § 1239 may recharacterize as ordinary income (if depreciable to buyer)</li>
                <li data-i18n="view.s263c.rec.s291_corp">§ 291 corporate cutback: 20% of S corp / C corp § 1254 recapture (parallel)</li>
            </ul>
        </div>
    `;
    document.getElementById('s263c-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.idc_total = Number(fd.get('idc_total')) || 0;
        state.is_independent_producer = !!fd.get('is_independent_producer');
        state.is_integrated_oil_co = !!fd.get('is_integrated_oil_co');
        state.is_royalty_owner = !!fd.get('is_royalty_owner');
        state.is_working_interest = !!fd.get('is_working_interest');
        state.net_oil_gas_income = Number(fd.get('net_oil_gas_income')) || 0;
        state.well_type = fd.get('well_type');
        state.elect_current_deduction = !!fd.get('elect_current_deduction');
        state.amt_taxable_income = Number(fd.get('amt_taxable_income')) || 0;
        state.section_469_passive = !!fd.get('section_469_passive');
        state.s291_recapture_amount = Number(fd.get('s291_recapture_amount')) || 0;
        state.geological_geophysical_amount = Number(fd.get('geological_geophysical_amount')) || 0;
        state.well_dry_hole = !!fd.get('well_dry_hole');
        state.production_year = Number(fd.get('production_year')) || 0;
        state.enhanced_oil_recovery = !!fd.get('enhanced_oil_recovery');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s263c-output');
    if (!el) return;
    let current_pct = 0;
    if (state.is_independent_producer || state.is_royalty_owner) current_pct = 1.0;
    else if (state.is_integrated_oil_co) current_pct = 0.70;
    const current_deduction = state.idc_total * current_pct * (state.elect_current_deduction ? 1 : 0);
    const amortized_portion = state.idc_total - current_deduction;
    const annual_amortization = amortized_portion / 5;
    const tax_savings_current = current_deduction * 0.21;
    const amt_preference = state.is_integrated_oil_co ? Math.min(current_deduction * 0.5, 0.65 * state.net_oil_gas_income) : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s263c.h2.result">§ 263(c) IDC computation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s263c.card.current_pct">Current deduction %</div>
                    <div class="value">${(current_pct * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s263c.card.current">Current deduction</div>
                    <div class="value">$${current_deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s263c.card.amortized">Amortized portion</div>
                    <div class="value">$${amortized_portion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s263c.card.annual">Annual amortization (5 yrs)</div>
                    <div class="value">$${annual_amortization.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s263c.card.savings">Current tax savings (21%)</div>
                    <div class="value">$${tax_savings_current.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s263c.card.amt">AMT preference</div>
                    <div class="value">$${amt_preference.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_independent_producer ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s263c.indep_note">
                    Independent producer: 100% IDC current deduction + § 57(a)(2)(E) AMT exemption.
                    Critical tax benefit for small / mid-size E&P companies. Limit: 75,000 bbl/day production.
                    Combined with § 613A small producer depletion deduction (15%): substantial tax shelter
                    for working interest investors.
                </p>
            ` : ''}
        </div>
    `;
}
