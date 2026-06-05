// IRC § 25C — Energy Efficient Home Improvement Credit.
// IRA 2022 expanded: 30% of qualified expenses. Annual caps: $1,200 general + $2,000 heat pumps + biomass.
// Specific caps: $600 windows, $250 doors ($500 total), $150 home energy audit.
// Through 12/31/2032. PIN reporting required from 2025.

import { currentViewToken, viewIsCurrent } from '../app.js';

const CREDIT_RATE = 0.30;
const ANNUAL_LIMIT_GENERAL = 1_200;
const ANNUAL_LIMIT_HEAT_PUMP = 2_000;
const WINDOWS_LIMIT = 600;
const DOORS_TOTAL_LIMIT = 500;
const DOORS_PER_DOOR_LIMIT = 250;
const AUDIT_LIMIT = 150;
const INSULATION_LIMIT = 1_200;

let state = {
    windows_skylights_cost: 0,
    doors_cost: 0,
    insulation_cost: 0,
    central_ac_cost: 0,
    furnace_cost: 0,
    heat_pump_cost: 0,
    biomass_stove_cost: 0,
    home_energy_audit_cost: 0,
    electrical_panel_cost: 0,
    fed_tax_liability: 0,
    is_existing_home: true,
    is_primary_residence: true,
};

export async function renderSection25c(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s25c.h1.title">// § 25C ENERGY EFFICIENT HOME</span></h1>
        <p class="muted small" data-i18n="view.s25c.hint.intro">
            <strong>IRA 2022 expanded:</strong> 30% of qualified expenses. <strong>Annual caps:</strong>
            $1,200 general + $2,000 heat pumps / heat pump water heaters / biomass stoves.
            Specific caps: $600 windows, $500 doors total ($250 each), $150 home energy audit,
            $600 panel. Through 12/31/2032. Existing home + primary residence required.
            <strong>PIN reporting required 2025+</strong> via Form 8908.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s25c.h2.inputs">Inputs</h2>
            <form id="s25c-form" class="inline-form">
                <label><span data-i18n="view.s25c.label.windows">Windows / skylights ($)</span>
                    <input type="number" step="0.01" name="windows_skylights_cost" value="${state.windows_skylights_cost}"></label>
                <label><span data-i18n="view.s25c.label.doors">Exterior doors ($)</span>
                    <input type="number" step="0.01" name="doors_cost" value="${state.doors_cost}"></label>
                <label><span data-i18n="view.s25c.label.insulation">Insulation / air sealing ($)</span>
                    <input type="number" step="0.01" name="insulation_cost" value="${state.insulation_cost}"></label>
                <label><span data-i18n="view.s25c.label.ac">Central AC ($)</span>
                    <input type="number" step="0.01" name="central_ac_cost" value="${state.central_ac_cost}"></label>
                <label><span data-i18n="view.s25c.label.furnace">Furnace / boiler ($)</span>
                    <input type="number" step="0.01" name="furnace_cost" value="${state.furnace_cost}"></label>
                <label><span data-i18n="view.s25c.label.heat_pump">Heat pump / HP water heater ($)</span>
                    <input type="number" step="0.01" name="heat_pump_cost" value="${state.heat_pump_cost}"></label>
                <label><span data-i18n="view.s25c.label.biomass">Biomass stove ($)</span>
                    <input type="number" step="0.01" name="biomass_stove_cost" value="${state.biomass_stove_cost}"></label>
                <label><span data-i18n="view.s25c.label.audit">Home energy audit ($)</span>
                    <input type="number" step="0.01" name="home_energy_audit_cost" value="${state.home_energy_audit_cost}"></label>
                <label><span data-i18n="view.s25c.label.panel">Electrical panel upgrade ($)</span>
                    <input type="number" step="0.01" name="electrical_panel_cost" value="${state.electrical_panel_cost}"></label>
                <label><span data-i18n="view.s25c.label.tax_liability">Federal tax liability ($)</span>
                    <input type="number" step="0.01" name="fed_tax_liability" value="${state.fed_tax_liability}"></label>
                <label><span data-i18n="view.s25c.label.existing">Existing home (not new construction)?</span>
                    <input type="checkbox" name="is_existing_home" ${state.is_existing_home ? 'checked' : ''}></label>
                <label><span data-i18n="view.s25c.label.primary">Primary residence?</span>
                    <input type="checkbox" name="is_primary_residence" ${state.is_primary_residence ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s25c.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s25c-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25c.h2.efficiency_requirements">Efficiency requirements</h2>
            <ul class="muted small">
                <li data-i18n="view.s25c.eff.windows">Windows / skylights: ENERGY STAR Most Efficient + meet IECC 2021</li>
                <li data-i18n="view.s25c.eff.doors">Doors: ENERGY STAR + meet IECC 2021</li>
                <li data-i18n="view.s25c.eff.insulation">Insulation: IECC 2021 climate zone requirements</li>
                <li data-i18n="view.s25c.eff.ac">Central AC: ENERGY STAR Most Efficient</li>
                <li data-i18n="view.s25c.eff.furnace">Furnace: ≥ 97% AFUE</li>
                <li data-i18n="view.s25c.eff.heat_pump">Heat pump: ENERGY STAR Most Efficient</li>
                <li data-i18n="view.s25c.eff.biomass">Biomass stove: ≥ 75% thermal efficiency</li>
                <li data-i18n="view.s25c.eff.panel">Panel: ≥ 200 amp + supports clean energy upgrade</li>
                <li data-i18n="view.s25c.eff.products_qualified">Manufacturer must certify (PIN to be required by IRS 2025+)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25c.h2.related">Related federal energy benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.s25c.rel.25d">§ 25D Residential Clean Energy: 30% solar / battery / geothermal (no cap)</li>
                <li data-i18n="view.s25c.rel.30d">§ 30D Clean Vehicle Credit: up to $7,500 EVs</li>
                <li data-i18n="view.s25c.rel.25e">§ 25E Used EV Credit: up to $4,000</li>
                <li data-i18n="view.s25c.rel.30c">§ 30C Alternative Fuel Refueling: 30% EV charger up to $1,000</li>
                <li data-i18n="view.s25c.rel.45l">§ 45L New Energy Efficient Home (builder): $2,500-$5,000</li>
                <li data-i18n="view.s25c.rel.179d">§ 179D Commercial Building Deduction (post-IRA: up to $5.65/sqft)</li>
                <li data-i18n="view.s25c.rel.heera">HEEHRA + HOMES Rebates (DOE, separate state-administered programs)</li>
                <li data-i18n="view.s25c.rel.state_credits">State + utility energy credits + rebates (varies)</li>
            </ul>
        </div>
    `;
    document.getElementById('s25c-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.windows_skylights_cost = Number(fd.get('windows_skylights_cost')) || 0;
        state.doors_cost = Number(fd.get('doors_cost')) || 0;
        state.insulation_cost = Number(fd.get('insulation_cost')) || 0;
        state.central_ac_cost = Number(fd.get('central_ac_cost')) || 0;
        state.furnace_cost = Number(fd.get('furnace_cost')) || 0;
        state.heat_pump_cost = Number(fd.get('heat_pump_cost')) || 0;
        state.biomass_stove_cost = Number(fd.get('biomass_stove_cost')) || 0;
        state.home_energy_audit_cost = Number(fd.get('home_energy_audit_cost')) || 0;
        state.electrical_panel_cost = Number(fd.get('electrical_panel_cost')) || 0;
        state.fed_tax_liability = Number(fd.get('fed_tax_liability')) || 0;
        state.is_existing_home = !!fd.get('is_existing_home');
        state.is_primary_residence = !!fd.get('is_primary_residence');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s25c-output');
    if (!el) return;
    if (!state.is_existing_home || !state.is_primary_residence) {
        el.innerHTML = `<div class="chart-panel"><p class="muted small neg" data-i18n="view.s25c.warning.eligibility">§ 25C only applies to existing primary residences. New construction + secondary homes ineligible.</p></div>`;
        return;
    }
    const windowsCredit = Math.min(state.windows_skylights_cost * CREDIT_RATE, WINDOWS_LIMIT);
    const doorsCredit = Math.min(state.doors_cost * CREDIT_RATE, DOORS_TOTAL_LIMIT);
    const insulationCredit = state.insulation_cost * CREDIT_RATE;
    const acCredit = state.central_ac_cost * CREDIT_RATE;
    const furnaceCredit = state.furnace_cost * CREDIT_RATE;
    const heatPumpCredit = Math.min(state.heat_pump_cost * CREDIT_RATE + state.biomass_stove_cost * CREDIT_RATE, ANNUAL_LIMIT_HEAT_PUMP);
    const auditCredit = Math.min(state.home_energy_audit_cost * CREDIT_RATE, AUDIT_LIMIT);
    const panelCredit = state.electrical_panel_cost * CREDIT_RATE;
    const generalTotal = windowsCredit + doorsCredit + insulationCredit + acCredit + furnaceCredit + auditCredit + panelCredit;
    const generalCapped = Math.min(generalTotal, ANNUAL_LIMIT_GENERAL);
    const totalCredit = generalCapped + heatPumpCredit;
    const usableCredit = Math.min(totalCredit, state.fed_tax_liability);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s25c.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s25c.card.windows">Windows</div>
                    <div class="value">$${windowsCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25c.card.doors">Doors</div>
                    <div class="value">$${doorsCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25c.card.insulation">Insulation</div>
                    <div class="value">$${insulationCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25c.card.hvac">Central AC + furnace</div>
                    <div class="value">$${(acCredit + furnaceCredit).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25c.card.heat_pump">Heat pump + biomass</div>
                    <div class="value">$${heatPumpCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25c.card.audit">Energy audit</div>
                    <div class="value">$${auditCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s25c.card.general_total">General total (cap $1,200)</div>
                    <div class="value">$${generalCapped.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25c.card.total">Total credit</div>
                    <div class="value">$${totalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25c.card.usable">Usable this year (non-refundable)</div>
                    <div class="value">$${usableCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
