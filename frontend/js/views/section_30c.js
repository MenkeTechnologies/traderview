// IRC § 30C — Alternative Fuel Vehicle Refueling Property Credit (EV chargers).
// IRA 2022 expanded: 30% credit (with wage/apprentice) up to $100K business; 30% / $1K personal.
// MUST be located in non-urban area OR low-income community census tract (per 8911 mapper).
// Construction begins before 1/1/2033; placed in service after 12/31/2022.
// § 6418 transferability for business credit; carryforward 20 yrs.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    project_cost: 0,
    is_business: true,
    in_non_urban_area: false,
    in_low_income_community: false,
    prevailing_wage: true,
    apprenticeship: true,
    placed_in_service_year: 2024,
    construction_begin_year: 2024,
    eligible_per_item: true,
    multiple_locations: false,
    item_count: 1,
    transferability_elected: false,
};

export async function renderSection30C(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s30C.h1.title">// § 30C ALT. FUEL CHARGER</span></h1>
        <p class="muted small" data-i18n="view.s30C.hint.intro">
            <strong>EV charger</strong> + alternative fuel infrastructure credit. <strong>Business:</strong>
            30% (with wage/apprenticeship) up to <strong>$100K per item</strong>. <strong>Personal:</strong>
            30% / <strong>$1,000 cap</strong>. <strong>IRA 2022 restriction:</strong> MUST be located in
            <strong>non-urban area</strong> OR <strong>low-income community</strong> census tract (8911 mapper).
            <strong>Construction begin before 1/1/2033</strong>; placed in service after 12/31/2022.
            <strong>§ 6418 transferability</strong> for business; 20-yr carryforward. Form 8911.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s30C.h2.inputs">Inputs</h2>
            <form id="s30C-form" class="inline-form">
                <label><span data-i18n="view.s30C.label.cost">Project cost ($)</span>
                    <input type="number" step="0.01" name="project_cost" value="${state.project_cost}"></label>
                <label><span data-i18n="view.s30C.label.business">Business use?</span>
                    <input type="checkbox" name="is_business" ${state.is_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30C.label.non_urban">Non-urban area?</span>
                    <input type="checkbox" name="in_non_urban_area" ${state.in_non_urban_area ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30C.label.lmi">Low-income community?</span>
                    <input type="checkbox" name="in_low_income_community" ${state.in_low_income_community ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30C.label.wage">Prevailing wage (full 30%)?</span>
                    <input type="checkbox" name="prevailing_wage" ${state.prevailing_wage ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30C.label.apprentice">Apprenticeship?</span>
                    <input type="checkbox" name="apprenticeship" ${state.apprenticeship ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30C.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s30C.label.construct">Construction begin year</span>
                    <input type="number" step="1" name="construction_begin_year" value="${state.construction_begin_year}"></label>
                <label><span data-i18n="view.s30C.label.eligible">Eligible per-item ($100K cap)?</span>
                    <input type="checkbox" name="eligible_per_item" ${state.eligible_per_item ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30C.label.multiple">Multiple locations?</span>
                    <input type="checkbox" name="multiple_locations" ${state.multiple_locations ? 'checked' : ''}></label>
                <label><span data-i18n="view.s30C.label.count">Number of items</span>
                    <input type="number" step="1" name="item_count" value="${state.item_count}"></label>
                <label><span data-i18n="view.s30C.label.transfer">Transferability elected?</span>
                    <input type="checkbox" name="transferability_elected" ${state.transferability_elected ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s30C.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s30C-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s30C.h2.eligibility">Location eligibility (8911 mapper required)</h2>
            <ul class="muted small">
                <li data-i18n="view.s30C.elig.non_urban">Non-urban: NOT in Census Bureau urban area (cities + suburbs)</li>
                <li data-i18n="view.s30C.elig.lic">Low-income community: § 45D(e) Census tract — poverty rate ≥ 20% OR income ≤ 80% statewide</li>
                <li data-i18n="view.s30C.elig.either">EITHER non-urban OR low-income qualifies (not both required)</li>
                <li data-i18n="view.s30C.elig.8911_mapper">Treasury 8911 mapper: irs.gov/credits-deductions/clean-energy/alternative-fuel-vehicle-refueling-property-credit</li>
                <li data-i18n="view.s30C.elig.address">Address-based: enter installation address to verify eligibility</li>
                <li data-i18n="view.s30C.elig.coordinates">Mapper uses geographic coordinates from US Census 2020 + BLS data</li>
                <li data-i18n="view.s30C.elig.50pct_residential">Residential property: 50%+ used for personal — separate $1K limit applies</li>
                <li data-i18n="view.s30C.elig.previously_eligible">Pre-2023: ALL locations eligible — IRA 2022 added geographic restriction</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s30C.h2.eligible_property">Eligible alternative fuel property</h2>
            <ul class="muted small">
                <li data-i18n="view.s30C.prop.ev_charger">EV chargers: Level 2 (residential), DC fast charging (commercial)</li>
                <li data-i18n="view.s30C.prop.hydrogen">Hydrogen refueling: pumps + storage + dispensers</li>
                <li data-i18n="view.s30C.prop.lng">LNG (Liquefied Natural Gas) refueling</li>
                <li data-i18n="view.s30C.prop.cng">CNG (Compressed Natural Gas) refueling</li>
                <li data-i18n="view.s30C.prop.lpg">LPG / Propane refueling</li>
                <li data-i18n="view.s30C.prop.e85">E85 (85% ethanol) fuel pumps</li>
                <li data-i18n="view.s30C.prop.bidirectional">V2G bidirectional charging: eligible if meets criteria</li>
                <li data-i18n="view.s30C.prop.cost_basis">Includes: equipment + installation labor + permits + ancillary costs</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s30C.h2.business_vs_personal">Business vs Personal — different rules</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s30C.th.use">Use type</th>
                    <th data-i18n="view.s30C.th.rate">Rate</th>
                    <th data-i18n="view.s30C.th.cap">Cap</th>
                    <th data-i18n="view.s30C.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>Business (wage + apprentice)</td><td>30%</td><td>$100K per item</td><td>§ 38 GBC, § 6418 transferable</td></tr>
                    <tr><td>Business (base, no wage)</td><td>6%</td><td>$100K per item</td><td>§ 38 GBC</td></tr>
                    <tr><td>Personal residence</td><td>30%</td><td>$1,000 per item</td><td>Non-refundable, no carryforward</td></tr>
                    <tr><td>Mixed use ≥ 50% personal</td><td>Personal limit ($1K)</td><td>$1K</td><td>Combined cap applies</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s30C-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.project_cost = Number(fd.get('project_cost')) || 0;
        state.is_business = !!fd.get('is_business');
        state.in_non_urban_area = !!fd.get('in_non_urban_area');
        state.in_low_income_community = !!fd.get('in_low_income_community');
        state.prevailing_wage = !!fd.get('prevailing_wage');
        state.apprenticeship = !!fd.get('apprenticeship');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.construction_begin_year = Number(fd.get('construction_begin_year')) || 0;
        state.eligible_per_item = !!fd.get('eligible_per_item');
        state.multiple_locations = !!fd.get('multiple_locations');
        state.item_count = Number(fd.get('item_count')) || 0;
        state.transferability_elected = !!fd.get('transferability_elected');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s30C-output');
    if (!el) return;
    const locationOK = state.in_non_urban_area || state.in_low_income_community;
    const construction_eligible = state.construction_begin_year < 2033 && state.placed_in_service_year > 2022;
    const eligible = locationOK && construction_eligible;
    const fullRate = state.prevailing_wage && state.apprenticeship;
    const rate = fullRate ? 0.30 : 0.06;
    const perItemCap = state.is_business ? 100_000 : 1_000;
    const creditPerItem = Math.min(state.project_cost * rate / state.item_count, perItemCap);
    const totalCredit = eligible ? creditPerItem * state.item_count : 0;
    const transferProceeds = state.transferability_elected ? totalCredit * 0.93 : totalCredit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s30C.h2.result">§ 30C credit computation</h2>
            <div class="cards">
                <div class="card ${locationOK ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s30C.card.location">Location eligible?</div>
                    <div class="value">${locationOK ? esc(t('view.s30C.status.yes')) : esc(t('view.s30C.status.no'))}</div>
                </div>
                <div class="card ${construction_eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s30C.card.construction">Construction eligible?</div>
                    <div class="value">${construction_eligible ? esc(t('view.s30C.status.yes')) : esc(t('view.s30C.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s30C.card.rate">Rate</div>
                    <div class="value">${(rate * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s30C.card.cap">Per-item cap</div>
                    <div class="value">$${perItemCap.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s30C.card.per_item">Credit per item</div>
                    <div class="value">$${creditPerItem.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s30C.card.total">Total credit (${state.item_count} items)</div>
                    <div class="value">$${totalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s30C.card.transfer">Transfer cash (93%)</div>
                    <div class="value">$${transferProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!locationOK ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s30C.loc_note">
                    Location FAIL: IRA 2022 restricts § 30C to non-urban areas + low-income communities only.
                    Pre-2023 installations may qualify under older rules. Verify via 8911 mapper at irs.gov.
                    Address-based lookup uses Census 2020 boundaries — borderline addresses may be eligible.
                </p>
            ` : ''}
        </div>
    `;
}
