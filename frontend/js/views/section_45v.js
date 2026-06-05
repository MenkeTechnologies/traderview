// IRC § 45V — Clean Hydrogen Production Credit (IRA 2022).
// Tiered by lifecycle GHG emissions: ≤ 0.45 kg CO2e/kg H2 = $3/kg; up to 4.0 = $0.60-$1/kg.
// 10-year credit period from facility placed in service.
// Construction begin before 2033; prevailing wage + apprenticeship 5× multiplier.
// § 6417 direct pay (5-yr taxable / perm tax-exempt); § 6418 transferability.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    kg_hydrogen_produced: 0,
    lifecycle_ghg_kg_co2e: 0,
    prevailing_wage: true,
    apprenticeship: true,
    placed_in_service_year: 2024,
    construction_begin_year: 2024,
    years_in_credit_period: 0,
    elect_direct_pay: false,
    elect_transferability: false,
    elect_45v_or_45q: '45v',
    domestic_content_compliant: true,
    pathway_three_pillars: true,
};

export async function renderSection45V(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s45V.h1.title">// § 45V CLEAN HYDROGEN</span></h1>
        <p class="muted small" data-i18n="view.s45V.hint.intro">
            Tiered by <strong>lifecycle GHG emissions</strong>: ≤ 0.45 kg CO2e/kg H2 = <strong>$3.00/kg</strong>;
            0.45-1.5 = $1.00/kg; 1.5-2.5 = $0.75/kg; 2.5-4.0 = $0.60/kg. <strong>10-year credit period</strong>
            from facility placed in service. <strong>Construction begin before 1/1/2033.</strong> 5× wage +
            apprenticeship multiplier — base rates 1/5 of above. <strong>Three pillars</strong> regulation
            (incrementality + temporal matching + deliverability) under 2024 final regs. <strong>§ 6417 direct
            pay + § 6418 transferability.</strong> <strong>§ 48 ITC election:</strong> may elect § 48 instead.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s45V.h2.inputs">Inputs</h2>
            <form id="s45V-form" class="inline-form">
                <label><span data-i18n="view.s45V.label.kg">kg hydrogen produced / yr</span>
                    <input type="number" step="0.01" name="kg_hydrogen_produced" value="${state.kg_hydrogen_produced}"></label>
                <label><span data-i18n="view.s45V.label.ghg">Lifecycle GHG kg CO2e / kg H2</span>
                    <input type="number" step="0.05" name="lifecycle_ghg_kg_co2e" value="${state.lifecycle_ghg_kg_co2e}"></label>
                <label><span data-i18n="view.s45V.label.wage">Prevailing wage compliant?</span>
                    <input type="checkbox" name="prevailing_wage" ${state.prevailing_wage ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45V.label.apprentice">Apprenticeship compliant?</span>
                    <input type="checkbox" name="apprenticeship" ${state.apprenticeship ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45V.label.placed">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s45V.label.construction">Construction begin year</span>
                    <input type="number" step="1" name="construction_begin_year" value="${state.construction_begin_year}"></label>
                <label><span data-i18n="view.s45V.label.years_credit">Years in 10-yr credit period</span>
                    <input type="number" step="1" name="years_in_credit_period" value="${state.years_in_credit_period}"></label>
                <label><span data-i18n="view.s45V.label.direct">§ 6417 direct pay?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45V.label.transfer">§ 6418 transferability?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45V.label.election">§ 45V or § 48 election</span>
                    <select name="elect_45v_or_45q">
                        <option value="45v" ${state.elect_45v_or_45q === '45v' ? 'selected' : ''}>§ 45V PTC ($/kg/yr × 10)</option>
                        <option value="48" ${state.elect_45v_or_45q === '48' ? 'selected' : ''}>§ 48 ITC (30% basis upfront)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s45V.label.domestic">Domestic content compliant?</span>
                    <input type="checkbox" name="domestic_content_compliant" ${state.domestic_content_compliant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45V.label.three_pillars">Three pillars met (incrementality + matching + deliverability)?</span>
                    <input type="checkbox" name="pathway_three_pillars" ${state.pathway_three_pillars ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s45V.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s45V-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45V.h2.rates">Credit by GHG intensity (with wage + apprentice 5×)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s45V.th.ghg">Lifecycle GHG (kg CO2e/kg H2)</th>
                    <th data-i18n="view.s45V.th.base">Base rate</th>
                    <th data-i18n="view.s45V.th.bonus">Full rate (5×)</th>
                </tr></thead>
                <tbody>
                    <tr><td>≤ 0.45</td><td>$0.60/kg</td><td>$3.00/kg</td></tr>
                    <tr><td>0.45 - 1.5</td><td>$0.20/kg</td><td>$1.00/kg</td></tr>
                    <tr><td>1.5 - 2.5</td><td>$0.15/kg</td><td>$0.75/kg</td></tr>
                    <tr><td>2.5 - 4.0</td><td>$0.12/kg</td><td>$0.60/kg</td></tr>
                    <tr><td>&gt; 4.0</td><td>0 (no credit)</td><td>0 (no credit)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45V.h2.pathways">Hydrogen production pathways</h2>
            <ul class="muted small">
                <li data-i18n="view.s45V.path.electrolysis">Electrolysis (green): renewable electricity + water → H2 + O2; lowest GHG if renewable</li>
                <li data-i18n="view.s45V.path.smr_ccs">Steam Methane Reforming + CCS (blue): natural gas + CCS may qualify if 90%+ capture</li>
                <li data-i18n="view.s45V.path.biogas">Biogas reforming: anaerobic digestion of waste → H2; LCA varies widely</li>
                <li data-i18n="view.s45V.path.biomass">Biomass gasification: agricultural / forestry residues</li>
                <li data-i18n="view.s45V.path.nuclear">Nuclear (pink/red): high-temp electrolysis or thermochemical from nuclear heat</li>
                <li data-i18n="view.s45V.path.methane_pyrolysis">Methane pyrolysis (turquoise): solid carbon byproduct; potentially low-LCA</li>
                <li data-i18n="view.s45V.path.gas_no_ccs">Conventional SMR (gray): &gt; 9 kg CO2e/kg — NO § 45V credit</li>
                <li data-i18n="view.s45V.path.byproduct">Byproduct hydrogen: from chlor-alkali, steam reforming — may qualify partial</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45V.h2.three_pillars">"Three pillars" (2024 final regs)</h2>
            <ul class="muted small">
                <li data-i18n="view.s45V.pillar.incrementality">INCREMENTALITY: electricity from new renewable source (≤ 36 months pre-production)</li>
                <li data-i18n="view.s45V.pillar.temporal">TEMPORAL MATCHING: hourly matching 2030+ (annual matching 2024-2029 phase-in)</li>
                <li data-i18n="view.s45V.pillar.deliverability">DELIVERABILITY: same EIA region or DOE-defined contiguous area</li>
                <li data-i18n="view.s45V.pillar.43_safe_harbor">Safe harbor: nuclear existing capacity (up to 200 MW per facility / 5% national)</li>
                <li data-i18n="view.s45V.pillar.exceptions">Carve-outs: hydropower long-term contracts + ERCOT / California state-specific</li>
                <li data-i18n="view.s45V.pillar.epc">EPCs (Energy Attribute Certificates) used to track renewable sourcing</li>
                <li data-i18n="view.s45V.pillar.eu_comparison">EU has similar 'additionality' standards under Renewable Energy Directive III</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45V.h2.s48_election">§ 48 ITC vs § 45V PTC election</h2>
            <ul class="muted small">
                <li data-i18n="view.s45V.elect.choose">Annual election: choose PTC (§ 45V) OR one-time ITC (§ 48)</li>
                <li data-i18n="view.s45V.elect.itc_appeal">ITC better when: high capex, low utilization, or quick monetization preferred</li>
                <li data-i18n="view.s45V.elect.ptc_appeal">PTC better when: high utilization (8000+ hrs/yr), low capex relative to output</li>
                <li data-i18n="view.s45V.elect.itc_recapture">ITC recapture risk if facility sold w/in 5 yrs</li>
                <li data-i18n="view.s45V.elect.ptc_no_recapture">PTC: no recapture, but credit only earned in years producing qualifying H2</li>
                <li data-i18n="view.s45V.elect.tax_equity">Tax equity market: both monetizable via § 6418 transferability</li>
                <li data-i18n="view.s45V.elect.fundamental">Fundamental tradeoff: certainty (ITC) vs ongoing (PTC); break-even ~5-6 years</li>
            </ul>
        </div>
    `;
    document.getElementById('s45V-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.kg_hydrogen_produced = Number(fd.get('kg_hydrogen_produced')) || 0;
        state.lifecycle_ghg_kg_co2e = Number(fd.get('lifecycle_ghg_kg_co2e')) || 0;
        state.prevailing_wage = !!fd.get('prevailing_wage');
        state.apprenticeship = !!fd.get('apprenticeship');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.construction_begin_year = Number(fd.get('construction_begin_year')) || 0;
        state.years_in_credit_period = Number(fd.get('years_in_credit_period')) || 0;
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.elect_transferability = !!fd.get('elect_transferability');
        state.elect_45v_or_45q = fd.get('elect_45v_or_45q');
        state.domestic_content_compliant = !!fd.get('domestic_content_compliant');
        state.pathway_three_pillars = !!fd.get('pathway_three_pillars');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s45V-output');
    if (!el) return;
    let fullRate = 0;
    if (state.lifecycle_ghg_kg_co2e <= 0.45) fullRate = 3.00;
    else if (state.lifecycle_ghg_kg_co2e <= 1.5) fullRate = 1.00;
    else if (state.lifecycle_ghg_kg_co2e <= 2.5) fullRate = 0.75;
    else if (state.lifecycle_ghg_kg_co2e <= 4.0) fullRate = 0.60;
    else fullRate = 0;
    const wageBonus = state.prevailing_wage && state.apprenticeship;
    const ratePerKg = wageBonus ? fullRate : fullRate * 0.20;
    const constructionEligible = state.construction_begin_year < 2033;
    const inCredit = state.years_in_credit_period < 10;
    const threePillars = state.pathway_three_pillars;
    const eligible = constructionEligible && inCredit && threePillars;
    const annualCredit = eligible ? state.kg_hydrogen_produced * ratePerKg : 0;
    const totalCredit10yr = annualCredit * 10;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s45V.h2.result">§ 45V credit computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s45V.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.s45V.status.yes')) : esc(t('view.s45V.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45V.card.ghg_tier">GHG tier</div>
                    <div class="value">${state.lifecycle_ghg_kg_co2e.toFixed(2)} kg CO2e</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45V.card.rate">Rate / kg</div>
                    <div class="value">$${ratePerKg.toFixed(2)}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45V.card.annual">Annual credit</div>
                    <div class="value">$${annualCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45V.card.ten_year">Total 10-yr credit</div>
                    <div class="value">$${totalCredit10yr.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.lifecycle_ghg_kg_co2e <= 0.45 ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s45V.full_credit_note">
                    Lowest tier achieved: full $3/kg PTC. Requires either truly renewable electrolysis with three
                    pillars OR SMR + 95%+ CCS + appropriate LCA. Verify with verified third-party LCA + GREET
                    (DOE) model. Maintain throughout 10-yr period — re-verify if pathways change.
                </p>
            ` : ''}
        </div>
    `;
}
