// IRC § 45X — Advanced Manufacturing Production Credit (IRA 2022).
// Per-unit credit on US production of clean energy components: solar cells, wind blades, batteries, critical minerals.
// Production tax credit (PTC) — paid per UNIT produced + sold to unrelated party.
// Phases out: 100% through 2029, 75% in 2030, 50% in 2031, 25% in 2032, 0% in 2033 (except minerals 100%).
// § 6417 direct pay (taxable 5-yr) / § 6418 transferability — major monetization paths.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    component_type: 'solar_module',
    units_sold: 0,
    capacity_per_unit: 0,
    placed_in_service_year: 2024,
    domestic_content_compliant: true,
    sold_to_unrelated_party: true,
    elect_direct_pay: false,
    elect_transferability: false,
    transfer_market_pct: 92,
    is_first_5_years_taxable: true,
    integrated_facility_election: false,
    facility_type: 'production',
};

export async function renderSection45X(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s45X.h1.title">// § 45X ADVANCED MFG CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s45X.hint.intro">
            <strong>Per-unit credit</strong> on US production of clean energy components: <strong>solar
            cells / modules / wafers, wind blades / nacelles / towers, battery cells / modules, inverters,
            critical minerals</strong>. <strong>Phases out:</strong> 100% through 2029, 75% 2030, 50% 2031,
            25% 2032, 0% 2033 (critical minerals stay 100% PERMANENTLY). Sold to <strong>unrelated party</strong>.
            <strong>§ 6417 direct pay</strong> (taxable 5-yr / nonprofit perm); <strong>§ 6418 transferability</strong>
            major monetization. Form 7207.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s45X.h2.inputs">Inputs</h2>
            <form id="s45X-form" class="inline-form">
                <label><span data-i18n="view.s45X.label.component">Component type</span>
                    <select name="component_type">
                        <option value="solar_cell" ${state.component_type === 'solar_cell' ? 'selected' : ''}>Solar cell ($/W)</option>
                        <option value="solar_module" ${state.component_type === 'solar_module' ? 'selected' : ''}>Solar module ($0.07/W)</option>
                        <option value="solar_wafer" ${state.component_type === 'solar_wafer' ? 'selected' : ''}>Solar wafer ($12/m²)</option>
                        <option value="solar_thin_film" ${state.component_type === 'solar_thin_film' ? 'selected' : ''}>Thin film photovoltaic ($/W)</option>
                        <option value="wind_blade" ${state.component_type === 'wind_blade' ? 'selected' : ''}>Wind blade ($0.02/W)</option>
                        <option value="wind_tower" ${state.component_type === 'wind_tower' ? 'selected' : ''}>Wind tower ($0.03/W)</option>
                        <option value="wind_nacelle" ${state.component_type === 'wind_nacelle' ? 'selected' : ''}>Wind nacelle ($0.05/W)</option>
                        <option value="battery_cell" ${state.component_type === 'battery_cell' ? 'selected' : ''}>Battery cell ($35/kWh)</option>
                        <option value="battery_module" ${state.component_type === 'battery_module' ? 'selected' : ''}>Battery module ($10/kWh)</option>
                        <option value="inverter" ${state.component_type === 'inverter' ? 'selected' : ''}>Inverter ($/W)</option>
                        <option value="critical_minerals" ${state.component_type === 'critical_minerals' ? 'selected' : ''}>Critical minerals (10% costs)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s45X.label.units">Units sold / produced</span>
                    <input type="number" step="100" name="units_sold" value="${state.units_sold}"></label>
                <label><span data-i18n="view.s45X.label.capacity">Capacity per unit (W or kWh)</span>
                    <input type="number" step="0.1" name="capacity_per_unit" value="${state.capacity_per_unit}"></label>
                <label><span data-i18n="view.s45X.label.year">Sale / placed-in-service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s45X.label.domestic">Domestic content compliant?</span>
                    <input type="checkbox" name="domestic_content_compliant" ${state.domestic_content_compliant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45X.label.unrelated">Sold to unrelated party?</span>
                    <input type="checkbox" name="sold_to_unrelated_party" ${state.sold_to_unrelated_party ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45X.label.direct">§ 6417 direct pay?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45X.label.transfer">§ 6418 transferability?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45X.label.market">Transfer market %</span>
                    <input type="number" step="0.1" name="transfer_market_pct" value="${state.transfer_market_pct}"></label>
                <label><span data-i18n="view.s45X.label.5yr">In first 5-yr taxable window?</span>
                    <input type="checkbox" name="is_first_5_years_taxable" ${state.is_first_5_years_taxable ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45X.label.integrated">§ 45X(c)(2)(C) integrated facility?</span>
                    <input type="checkbox" name="integrated_facility_election" ${state.integrated_facility_election ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s45X.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s45X-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45X.h2.rates">Per-unit credit rates (IRA 2022)</h2>
            <ul class="muted small">
                <li data-i18n="view.s45X.rate.solar_module">Solar module: $0.07 / direct current watt of capacity</li>
                <li data-i18n="view.s45X.rate.solar_cell">Solar cell: $0.04 / W</li>
                <li data-i18n="view.s45X.rate.solar_thin_film">Thin-film photovoltaic: $0.04 / W</li>
                <li data-i18n="view.s45X.rate.solar_wafer">Solar wafer: $12 / m²</li>
                <li data-i18n="view.s45X.rate.solar_grade">Polysilicon (solar grade): $3 / kg</li>
                <li data-i18n="view.s45X.rate.wind_blade">Wind blade: $0.02 / W of nameplate</li>
                <li data-i18n="view.s45X.rate.wind_tower">Wind tower: $0.03 / W</li>
                <li data-i18n="view.s45X.rate.wind_nacelle">Wind nacelle: $0.05 / W</li>
                <li data-i18n="view.s45X.rate.battery_cell">Battery cell: $35 / kWh</li>
                <li data-i18n="view.s45X.rate.battery_module">Battery module: $10 / kWh ($15 / kWh w/o cells incorporated)</li>
                <li data-i18n="view.s45X.rate.critical_minerals">Critical minerals: 10% of production costs (NEVER phases out)</li>
                <li data-i18n="view.s45X.rate.electrode">Electrode active materials: 10% of production costs</li>
                <li data-i18n="view.s45X.rate.applicable_critical">39 applicable critical minerals (lithium, cobalt, nickel, manganese, graphite, etc.)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45X.h2.phaseout">Phaseout schedule</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s45X.th.year">Year</th>
                    <th data-i18n="view.s45X.th.pct">Credit %</th>
                    <th data-i18n="view.s45X.th.exception">Exception</th>
                </tr></thead>
                <tbody>
                    <tr><td>2023-2029</td><td>100%</td><td>—</td></tr>
                    <tr><td>2030</td><td>75%</td><td>Critical minerals: 100%</td></tr>
                    <tr><td>2031</td><td>50%</td><td>Critical minerals: 100%</td></tr>
                    <tr><td>2032</td><td>25%</td><td>Critical minerals: 100%</td></tr>
                    <tr><td>2033+</td><td>0%</td><td>Critical minerals: 100% PERMANENT</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45X.h2.monetization">§ 6417 + § 6418 monetization details</h2>
            <ul class="muted small">
                <li data-i18n="view.s45X.mon.taxable_5yr">Taxable C-corps: 5-year § 6417 direct pay window starting in production year (2023-2032)</li>
                <li data-i18n="view.s45X.mon.tax_exempt_perm">Tax-exempt + tribal + electric coops: § 6417 permanent</li>
                <li data-i18n="view.s45X.mon.cash_market">Active credit market: 87-94¢ on $1 typical for verified projects</li>
                <li data-i18n="view.s45X.mon.passive_inv">Passive activity rules: § 469 may limit corp partners using PTCs</li>
                <li data-i18n="view.s45X.mon.feedstock_use">Self-use as feedstock: § 45X(c)(2)(C) integrated facility election lets internal use qualify</li>
                <li data-i18n="view.s45X.mon.foreign_entity">FEOC (Foreign Entity of Concern) excluded from claiming + supply chain restrictions</li>
                <li data-i18n="view.s45X.mon.cogs_allocation">COGS allocation reduces taxable income — credit + COGS combo is the planning sweet spot</li>
                <li data-i18n="view.s45X.mon.repurchase">Stockpile + later sale: credit recognized at sale, not production</li>
            </ul>
        </div>
    `;
    document.getElementById('s45X-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.component_type = fd.get('component_type');
        state.units_sold = Number(fd.get('units_sold')) || 0;
        state.capacity_per_unit = Number(fd.get('capacity_per_unit')) || 0;
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.domestic_content_compliant = !!fd.get('domestic_content_compliant');
        state.sold_to_unrelated_party = !!fd.get('sold_to_unrelated_party');
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.elect_transferability = !!fd.get('elect_transferability');
        state.transfer_market_pct = Number(fd.get('transfer_market_pct')) || 0;
        state.is_first_5_years_taxable = !!fd.get('is_first_5_years_taxable');
        state.integrated_facility_election = !!fd.get('integrated_facility_election');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s45X-output');
    if (!el) return;
    let ratePerUnit = 0;
    switch (state.component_type) {
        case 'solar_module': ratePerUnit = 0.07 * state.capacity_per_unit; break;
        case 'solar_cell': case 'solar_thin_film': ratePerUnit = 0.04 * state.capacity_per_unit; break;
        case 'solar_wafer': ratePerUnit = 12; break;
        case 'wind_blade': ratePerUnit = 0.02 * state.capacity_per_unit; break;
        case 'wind_tower': ratePerUnit = 0.03 * state.capacity_per_unit; break;
        case 'wind_nacelle': ratePerUnit = 0.05 * state.capacity_per_unit; break;
        case 'battery_cell': ratePerUnit = 35 * state.capacity_per_unit; break;
        case 'battery_module': ratePerUnit = 10 * state.capacity_per_unit; break;
        case 'critical_minerals': ratePerUnit = 0; break;
    }
    let phaseoutPct = 1.0;
    const isCriticalMin = state.component_type === 'critical_minerals';
    if (!isCriticalMin) {
        if (state.placed_in_service_year === 2030) phaseoutPct = 0.75;
        else if (state.placed_in_service_year === 2031) phaseoutPct = 0.50;
        else if (state.placed_in_service_year === 2032) phaseoutPct = 0.25;
        else if (state.placed_in_service_year >= 2033) phaseoutPct = 0;
    }
    const eligible = state.sold_to_unrelated_party && state.domestic_content_compliant;
    const totalCredit = eligible ? state.units_sold * ratePerUnit * phaseoutPct : 0;
    const transferProceeds = state.elect_transferability ? totalCredit * (state.transfer_market_pct / 100) : totalCredit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s45X.h2.result">§ 45X credit computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s45X.card.eligible">Eligible?</div>
                    <div class="value">${eligible ? esc(t('view.s45X.status.yes')) : esc(t('view.s45X.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45X.card.rate">Rate / unit</div>
                    <div class="value">$${ratePerUnit.toFixed(2)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45X.card.phaseout">Phaseout %</div>
                    <div class="value">${(phaseoutPct * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45X.card.credit">§ 45X credit</div>
                    <div class="value">$${totalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45X.card.transfer">Transfer cash proceeds</div>
                    <div class="value">$${transferProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${isCriticalMin ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s45X.card.permanent">Permanent (critical minerals)?</div>
                    <div class="value">${isCriticalMin ? esc(t('view.s45X.status.yes')) : esc(t('view.s45X.status.no'))}</div>
                </div>
            </div>
            ${isCriticalMin ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s45X.minerals_note">
                    Critical minerals NEVER phase out — 10% of production costs PERMANENTLY through full IRA
                    horizon. Strategically prioritize lithium, cobalt, graphite, manganese, nickel production
                    in US (39 applicable minerals). Combined with § 30D EV credit's mineral sourcing rules,
                    this drives US battery supply chain re-shoring.
                </p>
            ` : ''}
        </div>
    `;
}
