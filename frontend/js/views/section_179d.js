// IRC § 179D — Energy Efficient Commercial Building Deduction.
// IRA 2022 redesigned: $0.50-$5.65/sqft (2024) based on efficiency vs ASHRAE 90.1 baseline.
// Base ($0.50/sqft) + per-percentage-point (up to 25% lift): $0.02/sqft each.
// Prevailing wage + apprenticeship 5× multiplier. Allocation possible to designer (architect / engineer).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const BASE_PER_SQFT_2024 = 0.50;
const PER_PCT_INCREMENT = 0.02;
const MAX_LIFT_PCT = 100;  // 25 percentage points × per-pct (limit)
const PW_MULTIPLIER = 5;
const MAX_DEDUCTION_PER_SQFT = 5.65;

let state = {
    building_sqft: 0,
    energy_efficiency_pct_lift: 0,
    prevailing_wage_apprenticeship: false,
    is_government_or_tax_exempt: false,
    is_designer: false,
    cost_of_property: 0,
    building_marginal_rate: 0.21,
};

export async function renderSection179d(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s179d.h1.title">// § 179D COMMERCIAL BUILDING ENERGY</span></h1>
        <p class="muted small" data-i18n="view.s179d.hint.intro">
            <strong>IRA 2022 redesigned:</strong> $0.50-$5.65/sqft (2024) based on efficiency vs
            ASHRAE 90.1. <strong>Base $0.50/sqft</strong> + per-percentage-point lift over 25%
            ($0.02/sqft × pp). <strong>Prevailing wage + apprenticeship 5× multiplier</strong>.
            <strong>Allocation possible to designer</strong> (architect / engineer) for govt /
            non-profit buildings. Form 7205.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s179d.h2.inputs">Inputs</h2>
            <form id="s179d-form" class="inline-form">
                <label><span data-i18n="view.s179d.label.sqft">Building square footage</span>
                    <input type="number" step="1000" name="building_sqft" value="${state.building_sqft}"></label>
                <label><span data-i18n="view.s179d.label.efficiency_lift">Energy savings % (over ASHRAE 90.1)</span>
                    <input type="number" step="1" min="25" max="100" name="energy_efficiency_pct_lift" value="${state.energy_efficiency_pct_lift}"></label>
                <label><span data-i18n="view.s179d.label.pw">Prevailing wage + apprenticeship?</span>
                    <input type="checkbox" name="prevailing_wage_apprenticeship" ${state.prevailing_wage_apprenticeship ? 'checked' : ''}></label>
                <label><span data-i18n="view.s179d.label.govt">Government / tax-exempt building?</span>
                    <input type="checkbox" name="is_government_or_tax_exempt" ${state.is_government_or_tax_exempt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s179d.label.designer">Designer claiming allocation?</span>
                    <input type="checkbox" name="is_designer" ${state.is_designer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s179d.label.cost">Total qualified property cost ($)</span>
                    <input type="number" step="1000" name="cost_of_property" value="${state.cost_of_property}"></label>
                <label><span data-i18n="view.s179d.label.marginal">Building owner marginal rate</span>
                    <input type="number" step="0.01" name="building_marginal_rate" value="${state.building_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s179d.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s179d-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s179d.h2.qualifying">Qualifying property</h2>
            <ul class="muted small">
                <li data-i18n="view.s179d.q.envelope">Building envelope: insulation, windows, doors, roof</li>
                <li data-i18n="view.s179d.q.hvac">HVAC + hot water systems</li>
                <li data-i18n="view.s179d.q.interior_lighting">Interior lighting + controls</li>
                <li data-i18n="view.s179d.q.commercial">Commercial buildings (4+ floors); 4+ story mixed use</li>
                <li data-i18n="view.s179d.q.govt">Government buildings (federal, state, local) — allocated to designer</li>
                <li data-i18n="view.s179d.q.tax_exempt">Non-profit / tax-exempt buildings (since IRA 2022) — allocated to designer</li>
                <li data-i18n="view.s179d.q.tribal">Tribal government / Alaska Native corp / Indian housing authority</li>
                <li data-i18n="view.s179d.q.retrofits">Retrofits (3-year minimum compliance period)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s179d.h2.designer_allocation">Designer allocation (post-2023)</h2>
            <p class="muted small" data-i18n="view.s179d.designer.body">
                For government + non-profit buildings, building owner CANNOT claim § 179D
                (no tax liability). Owner can ALLOCATE deduction to qualifying designer
                (architect, engineer, contractor). Allocation by written certification.
                Designer treats as ordinary income recovery (basis reduction). One designer
                only per project; allocation does not affect cost basis to owner.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s179d.h2.combination">Combination with other incentives</h2>
            <ul class="muted small">
                <li data-i18n="view.s179d.combo.bonus">§ 168(k) Bonus depreciation: still allowed on basis after § 179D</li>
                <li data-i18n="view.s179d.combo.179">§ 179 Expensing: separate from § 179D, can combine but reduces basis</li>
                <li data-i18n="view.s179d.combo.45L">§ 45L New Energy Efficient Home: different — for residential / dwelling units</li>
                <li data-i18n="view.s179d.combo.6418">§ 6418 Transferability: § 179D NOT in transferable credit list (deduction, not credit)</li>
                <li data-i18n="view.s179d.combo.itc">§ 48 ITC: separate solar investment tax credit overlap allowed</li>
                <li data-i18n="view.s179d.combo.state">State + local utility energy rebates often stack</li>
            </ul>
        </div>
    `;
    document.getElementById('s179d-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.building_sqft = Number(fd.get('building_sqft')) || 0;
        state.energy_efficiency_pct_lift = Number(fd.get('energy_efficiency_pct_lift')) || 0;
        state.prevailing_wage_apprenticeship = !!fd.get('prevailing_wage_apprenticeship');
        state.is_government_or_tax_exempt = !!fd.get('is_government_or_tax_exempt');
        state.is_designer = !!fd.get('is_designer');
        state.cost_of_property = Number(fd.get('cost_of_property')) || 0;
        state.building_marginal_rate = Number(fd.get('building_marginal_rate')) || 0.21;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s179d-output');
    if (!el) return;
    const lift = Math.max(0, state.energy_efficiency_pct_lift - 25);
    let perSqftBase = BASE_PER_SQFT_2024 + lift * PER_PCT_INCREMENT;
    perSqftBase = Math.min(perSqftBase, MAX_DEDUCTION_PER_SQFT / (state.prevailing_wage_apprenticeship ? PW_MULTIPLIER : 1));
    const perSqftAdjusted = state.prevailing_wage_apprenticeship ? perSqftBase * PW_MULTIPLIER : perSqftBase;
    const cappedPerSqft = Math.min(perSqftAdjusted, MAX_DEDUCTION_PER_SQFT);
    const totalDeduction = Math.min(cappedPerSqft * state.building_sqft, state.cost_of_property);
    const taxSavings = totalDeduction * state.building_marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s179d.h2.result">§ 179D deduction</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s179d.card.per_sqft">$ per sqft</div>
                    <div class="value">$${cappedPerSqft.toFixed(2)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s179d.card.sqft">Sqft</div>
                    <div class="value">${state.building_sqft.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s179d.card.deduction">Total deduction</div>
                    <div class="value">$${totalDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s179d.card.tax_savings">Year-1 tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.is_government_or_tax_exempt && state.is_designer ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.s179d.card.designer">Designer claim (govt building)</div>
                        <div class="value">${esc(t('view.s179d.status.yes'))}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
