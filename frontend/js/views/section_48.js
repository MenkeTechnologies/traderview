// IRC § 48 — Investment Tax Credit (ITC, mostly solar / energy property).
// 30% credit on basis of qualified energy property (5× bonus structure: base 6% + 24%).
// IRA 2022 extended; § 48E "Clean Electricity ITC" replaces post-2024.
// Bonus credit adders: 10% domestic content + 10% energy community + 10% LMI bonus.
// Direct pay § 6417 for tax-exempts; transferability § 6418 for taxables.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    project_basis: 0,
    project_type: 'solar',
    prevailing_wage: true,
    apprenticeship: true,
    domestic_content_bonus: false,
    energy_community_bonus: false,
    lmi_bonus: false,
    placed_in_service_year: 2024,
    is_under_1mw: false,
    elect_direct_pay: false,
    elect_transferability: false,
    transfer_discount_pct: 95,
};

export async function renderSection48(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s48.h1.title">// § 48 INVESTMENT TAX CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s48.hint.intro">
            <strong>30% credit</strong> on qualified energy property basis (solar, geothermal, fuel cell,
            wind ≤ 100kW, biogas, CHP, micro-turbine, etc.). <strong>5× bonus structure:</strong> base
            6% + 24% prevailing wage / apprenticeship multiplier. <strong>Adders:</strong> +10% domestic
            content + 10% energy community + 10% LMI bonus (low-income community). Post-2024: § 48E
            "Clean Electricity ITC" replaces. <strong>§ 6417 direct pay</strong> tax-exempt; <strong>§ 6418
            transferability</strong> taxable. <strong>Forms 3468 + 3800.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s48.h2.inputs">Inputs</h2>
            <form id="s48-form" class="inline-form">
                <label><span data-i18n="view.s48.label.basis">Project basis ($)</span>
                    <input type="number" step="10000" name="project_basis" value="${state.project_basis}"></label>
                <label><span data-i18n="view.s48.label.type">Project type</span>
                    <select name="project_type">
                        <option value="solar" ${state.project_type === 'solar' ? 'selected' : ''}>Solar PV</option>
                        <option value="solar_thermal" ${state.project_type === 'solar_thermal' ? 'selected' : ''}>Solar thermal / concentrating</option>
                        <option value="geothermal" ${state.project_type === 'geothermal' ? 'selected' : ''}>Geothermal heat pump</option>
                        <option value="fuel_cell" ${state.project_type === 'fuel_cell' ? 'selected' : ''}>Fuel cell</option>
                        <option value="small_wind" ${state.project_type === 'small_wind' ? 'selected' : ''}>Small wind (≤ 100kW)</option>
                        <option value="biogas" ${state.project_type === 'biogas' ? 'selected' : ''}>Biogas</option>
                        <option value="chp" ${state.project_type === 'chp' ? 'selected' : ''}>Combined heat + power</option>
                        <option value="microgrid" ${state.project_type === 'microgrid' ? 'selected' : ''}>Microgrid controller</option>
                        <option value="storage" ${state.project_type === 'storage' ? 'selected' : ''}>Standalone storage (post-2022)</option>
                        <option value="biomass" ${state.project_type === 'biomass' ? 'selected' : ''}>Biomass</option>
                    </select>
                </label>
                <label><span data-i18n="view.s48.label.wage">Prevailing wage compliant?</span>
                    <input type="checkbox" name="prevailing_wage" ${state.prevailing_wage ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.apprentice">Apprenticeship compliant?</span>
                    <input type="checkbox" name="apprenticeship" ${state.apprenticeship ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.domestic">Domestic content bonus (+10%)?</span>
                    <input type="checkbox" name="domestic_content_bonus" ${state.domestic_content_bonus ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.energy_comm">Energy community bonus (+10%)?</span>
                    <input type="checkbox" name="energy_community_bonus" ${state.energy_community_bonus ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.lmi">LMI / Tribal bonus (+10/+20%)?</span>
                    <input type="checkbox" name="lmi_bonus" ${state.lmi_bonus ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s48.label.1mw">Under 1 MW (no wage requirement)?</span>
                    <input type="checkbox" name="is_under_1mw" ${state.is_under_1mw ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.direct">§ 6417 direct pay?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.transfer">§ 6418 transferability?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48.label.discount">Transfer market price % (typical 88-95)</span>
                    <input type="number" step="0.1" name="transfer_discount_pct" value="${state.transfer_discount_pct}"></label>
                <button class="primary" type="submit" data-i18n="view.s48.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s48-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s48.h2.bonus_structure">5× wage / apprentice + 3 adders</h2>
            <ul class="muted small">
                <li data-i18n="view.s48.bonus.base">BASE rate: 6% (without wage / apprentice OR ≥ 1 MW)</li>
                <li data-i18n="view.s48.bonus.full">FULL rate: 30% (with wage + apprentice OR under 1 MW)</li>
                <li data-i18n="view.s48.bonus.domestic">+10% Domestic Content: ≥ 40% steel + iron domestic; manuf'd products % varies</li>
                <li data-i18n="view.s48.bonus.energy">+10% Energy Community: brownfield / fossil-fuel community / unemployment census</li>
                <li data-i18n="view.s48.bonus.lmi">+10-20% LMI bonus: low-income census tract or affordable housing — competitive allocation</li>
                <li data-i18n="view.s48.bonus.aggregate">Max stacked: 30% + 10% + 10% + 10/20% = 50-60% effective</li>
                <li data-i18n="view.s48.bonus.allocation_required">LMI bonus requires PRE-APPROVAL via Treasury / DOE Capacity Allocation Program</li>
                <li data-i18n="view.s48.bonus.under_1mw">&lt; 1 MW projects: full 30% rate WITHOUT wage / apprentice compliance required</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s48.h2.s48e_transition">§ 48E Clean Electricity ITC (post-2024)</h2>
            <ul class="muted small">
                <li data-i18n="view.s48.s48e.tech_neutral">Technology-neutral: any zero-emission electricity generation qualifies</li>
                <li data-i18n="view.s48.s48e.phase_in">Begins 2025; § 48 phases out for new projects</li>
                <li data-i18n="view.s48.s48e.same_structure">Same 6%/30% base + adders + § 6417 / § 6418 monetization</li>
                <li data-i18n="view.s48.s48e.battery">Storage included (paired or standalone)</li>
                <li data-i18n="view.s48.s48e.phaseout">Phase-out: 100% credit until 2032; phases out as GHG goals met</li>
                <li data-i18n="view.s48.s48e.future">Long-term: extends until US grid hits 25% of 2022 emissions</li>
                <li data-i18n="view.s48.s48e.choice">Choose § 48 (legacy) OR § 48E (post-2024) — projects in service 2025+ default to § 48E</li>
                <li data-i18n="view.s48.s48e.45y_ptc">§ 45Y is PTC counterpart (operating credit per kWh)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s48.h2.recapture">Recapture rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s48.rec.5yr">5-year recapture period from placed-in-service date</li>
                <li data-i18n="view.s48.rec.schedule">Yr 1: 100% / Yr 2: 80% / Yr 3: 60% / Yr 4: 40% / Yr 5: 20%</li>
                <li data-i18n="view.s48.rec.triggers">Triggers: sale, abandonment, theft, casualty, lease change, change of use</li>
                <li data-i18n="view.s48.rec.transferability">Transferability: recapture stays w/ ORIGINAL TAXPAYER (seller); buyer protected</li>
                <li data-i18n="view.s48.rec.casualty_exception">Casualty replacement: no recapture if rebuilt within rules</li>
                <li data-i18n="view.s48.rec.basis_reduce">Basis reduction: 50% of credit (separate basis reduction for adders)</li>
                <li data-i18n="view.s48.rec.recapture_form">Form 4255 to compute recapture if triggered</li>
                <li data-i18n="view.s48.rec.exit_planning">Hold 5+ yrs to avoid recapture — common in tax equity partnerships</li>
            </ul>
        </div>
    `;
    document.getElementById('s48-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.project_basis = Number(fd.get('project_basis')) || 0;
        state.project_type = fd.get('project_type');
        state.prevailing_wage = !!fd.get('prevailing_wage');
        state.apprenticeship = !!fd.get('apprenticeship');
        state.domestic_content_bonus = !!fd.get('domestic_content_bonus');
        state.energy_community_bonus = !!fd.get('energy_community_bonus');
        state.lmi_bonus = !!fd.get('lmi_bonus');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.is_under_1mw = !!fd.get('is_under_1mw');
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.elect_transferability = !!fd.get('elect_transferability');
        state.transfer_discount_pct = Number(fd.get('transfer_discount_pct')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s48-output');
    if (!el) return;
    const fullRateMet = state.is_under_1mw || (state.prevailing_wage && state.apprenticeship);
    const baseRate = fullRateMet ? 0.30 : 0.06;
    const adders = (state.domestic_content_bonus ? 0.10 : 0) + (state.energy_community_bonus ? 0.10 : 0) + (state.lmi_bonus ? 0.10 : 0);
    const totalRate = baseRate + adders;
    const credit = state.project_basis * totalRate;
    const basisReduction = credit * 0.50;
    const transferProceeds = state.elect_transferability ? credit * (state.transfer_discount_pct / 100) : credit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s48.h2.result">§ 48 ITC computation</h2>
            <div class="cards">
                <div class="card ${fullRateMet ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s48.card.full_rate">Full rate (30%)?</div>
                    <div class="value">${fullRateMet ? esc(t('view.s48.status.yes')) : esc(t('view.s48.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s48.card.base">Base rate</div>
                    <div class="value">${(baseRate * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s48.card.adders">Total adders</div>
                    <div class="value">${(adders * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s48.card.total">Total rate</div>
                    <div class="value">${(totalRate * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s48.card.credit">ITC value</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s48.card.basis">Basis reduction (50%)</div>
                    <div class="value">$${basisReduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s48.card.proceeds">Transfer cash proceeds</div>
                    <div class="value">$${transferProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
