// IRC § 25D — Residential Clean Energy Credit (home solar / battery).
// 30% credit on residential property: solar PV, solar thermal, fuel cells, geothermal, small wind, battery storage.
// IRA 2022 extended through 2034 (phase-out 26% 2033, 22% 2034); reinstated at 30%.
// Battery storage ≥ 3 kWh: added IRA 2022 as standalone qualified (no solar pairing required).
// NON-refundable but UNLIMITED carryforward.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    solar_pv_cost: 0,
    solar_thermal_cost: 0,
    geothermal_cost: 0,
    fuel_cell_cost: 0,
    small_wind_cost: 0,
    battery_storage_cost: 0,
    battery_kwh: 0,
    placed_in_service_year: 2024,
    is_residence: true,
    is_principal_residence: true,
    is_secondary_residence: false,
    is_new_construction: false,
    tax_liability: 0,
    carryforward_prior: 0,
};

export async function renderSection25D(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s25D.h1.title">// § 25D RESIDENTIAL CLEAN ENERGY</span></h1>
        <p class="muted small" data-i18n="view.s25D.hint.intro">
            <strong>30% credit</strong> on residential property: solar PV, solar thermal water, fuel cells,
            geothermal heat pump, small wind, <strong>battery storage ≥ 3 kWh</strong>. IRA 2022 extended
            through 2034 (<strong>26% in 2033, 22% in 2034</strong>); 30% through 2032. Battery storage
            added 2023 as <strong>standalone</strong> qualified (no solar pairing). <strong>Non-refundable
            but UNLIMITED carryforward</strong> (unused offsets future tax). <strong>Form 5695</strong>.
            Principal residence + 2nd home eligible; rentals NO. Includes labor + installation.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s25D.h2.inputs">Inputs</h2>
            <form id="s25D-form" class="inline-form">
                <label><span data-i18n="view.s25D.label.solar_pv">Solar PV cost ($)</span>
                    <input type="number" step="0.01" name="solar_pv_cost" value="${state.solar_pv_cost}"></label>
                <label><span data-i18n="view.s25D.label.solar_thermal">Solar thermal water cost ($)</span>
                    <input type="number" step="0.01" name="solar_thermal_cost" value="${state.solar_thermal_cost}"></label>
                <label><span data-i18n="view.s25D.label.geothermal">Geothermal heat pump cost ($)</span>
                    <input type="number" step="0.01" name="geothermal_cost" value="${state.geothermal_cost}"></label>
                <label><span data-i18n="view.s25D.label.fuel_cell">Fuel cell cost ($)</span>
                    <input type="number" step="0.01" name="fuel_cell_cost" value="${state.fuel_cell_cost}"></label>
                <label><span data-i18n="view.s25D.label.wind">Small wind cost ($)</span>
                    <input type="number" step="0.01" name="small_wind_cost" value="${state.small_wind_cost}"></label>
                <label><span data-i18n="view.s25D.label.battery">Battery storage cost ($)</span>
                    <input type="number" step="0.01" name="battery_storage_cost" value="${state.battery_storage_cost}"></label>
                <label><span data-i18n="view.s25D.label.kwh">Battery kWh capacity</span>
                    <input type="number" step="0.1" name="battery_kwh" value="${state.battery_kwh}"></label>
                <label><span data-i18n="view.s25D.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s25D.label.residence">Residence (not rental)?</span>
                    <input type="checkbox" name="is_residence" ${state.is_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25D.label.principal">Principal residence?</span>
                    <input type="checkbox" name="is_principal_residence" ${state.is_principal_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25D.label.secondary">Secondary residence?</span>
                    <input type="checkbox" name="is_secondary_residence" ${state.is_secondary_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25D.label.new_construction">New construction (allowed)?</span>
                    <input type="checkbox" name="is_new_construction" ${state.is_new_construction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25D.label.liability">Current year tax liability ($)</span>
                    <input type="number" step="0.01" name="tax_liability" value="${state.tax_liability}"></label>
                <label><span data-i18n="view.s25D.label.carry">Prior carryforward ($)</span>
                    <input type="number" step="0.01" name="carryforward_prior" value="${state.carryforward_prior}"></label>
                <button class="primary" type="submit" data-i18n="view.s25D.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s25D-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25D.h2.eligibility">Eligible property categories</h2>
            <ul class="muted small">
                <li data-i18n="view.s25D.elig.solar_pv">Solar PV: panels, inverters, wiring, mounting, installation labor</li>
                <li data-i18n="view.s25D.elig.solar_thermal">Solar thermal water heating: NOT pool / hot tub heating</li>
                <li data-i18n="view.s25D.elig.battery_storage">Battery storage ≥ 3 kWh — added IRA 2022 as standalone (no solar pairing required)</li>
                <li data-i18n="view.s25D.elig.fuel_cell">Fuel cell: $500/0.5 kW capacity limit (max $1,667 / household per yr × 30% = $500)</li>
                <li data-i18n="view.s25D.elig.geothermal">Geothermal heat pump: Energy Star certified; loop + heat exchanger + pump</li>
                <li data-i18n="view.s25D.elig.wind">Small wind: ≤ 100 kW capacity, residential property</li>
                <li data-i18n="view.s25D.elig.labor">Labor + installation INCLUDED in credit basis</li>
                <li data-i18n="view.s25D.elig.permits">Permits + inspection fees INCLUDED</li>
                <li data-i18n="view.s25D.elig.roof">NOT eligible: structural roof reinforcement, general roof replacement (unless integrated PV)</li>
                <li data-i18n="view.s25D.elig.financing">Financing costs (interest) NOT in basis; deductible separately</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25D.h2.timeline">Credit rate timeline</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s25D.th.year">Year placed in service</th>
                    <th data-i18n="view.s25D.th.rate">Credit %</th>
                    <th data-i18n="view.s25D.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>2022</td><td>30% (retroactive)</td><td>IRA restored</td></tr>
                    <tr><td>2023-2032</td><td>30%</td><td>Battery storage added 2023</td></tr>
                    <tr><td>2033</td><td>26%</td><td>Phase-out begins</td></tr>
                    <tr><td>2034</td><td>22%</td><td>Last year</td></tr>
                    <tr><td>2035+</td><td>0%</td><td>Sunset</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25D.h2.optimization">Optimization strategies</h2>
            <ul class="muted small">
                <li data-i18n="view.s25D.opt.timing">Time installation to maximize 30% credit (2023-2032)</li>
                <li data-i18n="view.s25D.opt.bundling">Bundle with state + utility rebates — § 25D applies to cost AFTER rebates (federal subtracts state)</li>
                <li data-i18n="view.s25D.opt.battery_alone">Standalone battery now qualifies — install without solar if grid backup desired</li>
                <li data-i18n="view.s25D.opt.tax_planning">Coordinate with income year to use credit (Roth conv, capital gains realization)</li>
                <li data-i18n="view.s25D.opt.no_lease">Solar lease / PPA: NOT eligible — only OWNED systems qualify</li>
                <li data-i18n="view.s25D.opt.battery_sizing">Battery 3+ kWh: avoid sub-3 kWh batteries (no credit)</li>
                <li data-i18n="view.s25D.opt.divided_ownership">Co-owners: each claims proportionate share based on % paid</li>
                <li data-i18n="view.s25D.opt.basis_reduce">Basis reduction: 50% — applies to depreciation IF business use portion</li>
            </ul>
        </div>
    `;
    document.getElementById('s25D-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.solar_pv_cost = Number(fd.get('solar_pv_cost')) || 0;
        state.solar_thermal_cost = Number(fd.get('solar_thermal_cost')) || 0;
        state.geothermal_cost = Number(fd.get('geothermal_cost')) || 0;
        state.fuel_cell_cost = Number(fd.get('fuel_cell_cost')) || 0;
        state.small_wind_cost = Number(fd.get('small_wind_cost')) || 0;
        state.battery_storage_cost = Number(fd.get('battery_storage_cost')) || 0;
        state.battery_kwh = Number(fd.get('battery_kwh')) || 0;
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.is_residence = !!fd.get('is_residence');
        state.is_principal_residence = !!fd.get('is_principal_residence');
        state.is_secondary_residence = !!fd.get('is_secondary_residence');
        state.is_new_construction = !!fd.get('is_new_construction');
        state.tax_liability = Number(fd.get('tax_liability')) || 0;
        state.carryforward_prior = Number(fd.get('carryforward_prior')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s25D-output');
    if (!el) return;
    let rate = 0.30;
    if (state.placed_in_service_year === 2033) rate = 0.26;
    else if (state.placed_in_service_year === 2034) rate = 0.22;
    else if (state.placed_in_service_year >= 2035) rate = 0;
    const batteryEligible = state.battery_kwh >= 3;
    const eligibleCosts = state.solar_pv_cost + state.solar_thermal_cost + state.geothermal_cost +
        state.fuel_cell_cost + state.small_wind_cost + (batteryEligible ? state.battery_storage_cost : 0);
    const grossCredit = eligibleCosts * rate;
    const totalCreditAvailable = grossCredit + state.carryforward_prior;
    const allowedCurrent = Math.min(totalCreditAvailable, state.tax_liability);
    const newCarryforward = Math.max(0, totalCreditAvailable - allowedCurrent);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s25D.h2.result">§ 25D credit computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s25D.card.rate">Rate</div>
                    <div class="value">${(rate * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25D.card.eligible">Eligible cost basis</div>
                    <div class="value">$${eligibleCosts.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${batteryEligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s25D.card.battery_elig">Battery ≥ 3 kWh?</div>
                    <div class="value">${batteryEligible ? esc(t('view.s25D.status.yes')) : esc(t('view.s25D.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25D.card.gross">Gross credit</div>
                    <div class="value">$${grossCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25D.card.allowed">Used this year</div>
                    <div class="value">$${allowedCurrent.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25D.card.carry">Carry forward (unlimited)</div>
                    <div class="value">$${newCarryforward.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${newCarryforward > 0 ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s25D.carry_note">
                    Unused credit carries forward INDEFINITELY — Roth conversion, IRA distribution, or
                    other taxable event can use credit. § 25D is non-refundable but never expires.
                    Unlike § 38 GBC (20-yr limit), § 25D has no time limit on carryforward.
                </p>
            ` : ''}
        </div>
    `;
}
