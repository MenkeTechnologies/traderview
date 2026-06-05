// IRC § 48C — Qualified Advanced Energy Project Credit.
// Competitive allocation by DOE: 30% (with wage/apprentice) for energy property + advanced manufacturing.
// IRA 2022 reauthorized w/ $10B (40% reserved for energy communities).
// Allocations Round 1: $4B (Mar 2023); Round 2: $6B (Jan 2024).
// Forms 3468 + 3800 + DOE concept paper + full application.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    project_basis: 0,
    project_category: 'renewable_energy_property',
    prevailing_wage: true,
    apprenticeship: true,
    energy_community_reserved: false,
    placed_in_service_year: 2024,
    doe_allocation_received: false,
    allocation_amount: 0,
    is_critical_minerals: false,
    domestic_content_compliant: true,
    elect_direct_pay: false,
    elect_transferability: false,
};

export async function renderSection48C(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s48C.h1.title">// § 48C ADVANCED ENERGY</span></h1>
        <p class="muted small" data-i18n="view.s48C.hint.intro">
            <strong>Competitive allocation</strong> by DOE: <strong>30%</strong> ITC (with wage + apprenticeship)
            on energy property + advanced manufacturing equipment. Reauthorized IRA 2022 with <strong>$10B</strong>
            (40% reserved for <strong>energy communities</strong>). <strong>Round 1: $4B</strong> (Mar 2023);
            <strong>Round 2: $6B</strong> (Jan 2024). Reserved for: clean energy manufacturing + critical minerals
            + industrial decarbonization. <strong>2-yr placed-in-service</strong> after allocation. Forms 3468
            + 3800 + DOE concept paper + full app. <strong>§ 6418 transferability</strong> allowed; <strong>NO
            § 6417 direct pay</strong> for § 48C.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s48C.h2.inputs">Inputs</h2>
            <form id="s48C-form" class="inline-form">
                <label><span data-i18n="view.s48C.label.basis">Project basis ($)</span>
                    <input type="number" step="0.01" name="project_basis" value="${state.project_basis}"></label>
                <label><span data-i18n="view.s48C.label.category">Project category</span>
                    <select name="project_category">
                        <option value="renewable_energy_property" ${state.project_category === 'renewable_energy_property' ? 'selected' : ''}>Renewable energy property</option>
                        <option value="energy_storage" ${state.project_category === 'energy_storage' ? 'selected' : ''}>Energy storage</option>
                        <option value="grid_modernization" ${state.project_category === 'grid_modernization' ? 'selected' : ''}>Grid modernization</option>
                        <option value="ccus" ${state.project_category === 'ccus' ? 'selected' : ''}>Carbon capture (CCUS)</option>
                        <option value="hydrogen_equipment" ${state.project_category === 'hydrogen_equipment' ? 'selected' : ''}>Hydrogen production equip</option>
                        <option value="ev_components" ${state.project_category === 'ev_components' ? 'selected' : ''}>Electric vehicle components</option>
                        <option value="industrial_decarb" ${state.project_category === 'industrial_decarb' ? 'selected' : ''}>Industrial decarbonization (≥ 20% reduction)</option>
                        <option value="critical_minerals" ${state.project_category === 'critical_minerals' ? 'selected' : ''}>Critical minerals processing</option>
                    </select>
                </label>
                <label><span data-i18n="view.s48C.label.wage">Prevailing wage compliant?</span>
                    <input type="checkbox" name="prevailing_wage" ${state.prevailing_wage ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48C.label.apprentice">Apprenticeship compliant?</span>
                    <input type="checkbox" name="apprenticeship" ${state.apprenticeship ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48C.label.energy_comm">Energy community ($4B reserve)?</span>
                    <input type="checkbox" name="energy_community_reserved" ${state.energy_community_reserved ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48C.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s48C.label.allocation">DOE allocation received?</span>
                    <input type="checkbox" name="doe_allocation_received" ${state.doe_allocation_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48C.label.amount">Allocation amount ($)</span>
                    <input type="number" step="0.01" name="allocation_amount" value="${state.allocation_amount}"></label>
                <label><span data-i18n="view.s48C.label.minerals">Critical minerals?</span>
                    <input type="checkbox" name="is_critical_minerals" ${state.is_critical_minerals ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48C.label.domestic">Domestic content compliant?</span>
                    <input type="checkbox" name="domestic_content_compliant" ${state.domestic_content_compliant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48C.label.direct">§ 6417 direct pay (NOT for § 48C)?</span>
                    <input type="checkbox" name="elect_direct_pay" ${state.elect_direct_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s48C.label.transfer">§ 6418 transferability?</span>
                    <input type="checkbox" name="elect_transferability" ${state.elect_transferability ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s48C.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s48C-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s48C.h2.process">Application process (multi-stage)</h2>
            <ol class="muted small">
                <li data-i18n="view.s48C.proc.concept">Step 1: 8-page DOE concept paper (energy.gov 48C-PORTAL)</li>
                <li data-i18n="view.s48C.proc.encourage">Step 2: DOE "Encouragement Letter" — proceed with full application</li>
                <li data-i18n="view.s48C.proc.full_app">Step 3: 30-page full application + technical details + commercial plan</li>
                <li data-i18n="view.s48C.proc.allocation">Step 4: DOE selects + Treasury allocates credit amount</li>
                <li data-i18n="view.s48C.proc.placed">Step 5: Place in service within 2 years of allocation (extension up to 1 yr)</li>
                <li data-i18n="view.s48C.proc.cert">Step 6: 5-year certification — annual compliance reporting</li>
                <li data-i18n="view.s48C.proc.recapture">Step 7: 5-yr recapture period (sale, abandonment, change of use)</li>
                <li data-i18n="view.s48C.proc.evaluation_criteria">Evaluation: (1) commercial viability, (2) job creation, (3) GHG reduction, (4) supply chain diversification</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s48C.h2.rounds">Round 1 + Round 2 IRA reauthorization</h2>
            <ul class="muted small">
                <li data-i18n="view.s48C.round.r1_amount">Round 1: $4B allocated (Mar 2023); 80 projects selected from 250+ applicants</li>
                <li data-i18n="view.s48C.round.r1_energy">Round 1: $1.5B reserved for energy communities (40%)</li>
                <li data-i18n="view.s48C.round.r2_amount">Round 2: $6B allocated (Jan 2024); 230+ projects</li>
                <li data-i18n="view.s48C.round.r2_energy">Round 2: $2.5B for energy communities + 80% domestic content bonus</li>
                <li data-i18n="view.s48C.round.r3_pending">Round 3: pending congressional authorization for additional funding</li>
                <li data-i18n="view.s48C.round.beats_45x">Stacking: § 48C + § 45X possible if separate qualifying expenditures (different basis)</li>
                <li data-i18n="view.s48C.round.timing">Timing: concept papers due ~Q1; full applications ~Q3; allocations ~Q4-Q1 next yr</li>
                <li data-i18n="view.s48C.round.success_rate">Award rate: ~30-40% of full applicants — competitive</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s48C.h2.compare">§ 48 vs § 48C vs § 45X</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s48C.th.code">Code</th>
                    <th data-i18n="view.s48C.th.target">Target</th>
                    <th data-i18n="view.s48C.th.allocation">Allocation</th>
                    <th data-i18n="view.s48C.th.direct">Direct pay?</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 48 ITC</td><td>Energy property (solar, etc.)</td><td>No cap — automatic</td><td>Tax-exempt only</td></tr>
                    <tr><td>§ 48C Advanced Energy</td><td>Manufacturing of clean tech + decarb</td><td>Competitive DOE</td><td>NO — only transferability</td></tr>
                    <tr><td>§ 45X Adv Mfg PTC</td><td>Production of clean energy components</td><td>No cap — automatic per unit</td><td>5-yr taxable + perm tax-exempt</td></tr>
                    <tr><td>§ 45V Hydrogen</td><td>Hydrogen production</td><td>No cap — automatic per kg</td><td>5-yr taxable + perm tax-exempt</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s48C-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.project_basis = Number(fd.get('project_basis')) || 0;
        state.project_category = fd.get('project_category');
        state.prevailing_wage = !!fd.get('prevailing_wage');
        state.apprenticeship = !!fd.get('apprenticeship');
        state.energy_community_reserved = !!fd.get('energy_community_reserved');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || 0;
        state.doe_allocation_received = !!fd.get('doe_allocation_received');
        state.allocation_amount = Number(fd.get('allocation_amount')) || 0;
        state.is_critical_minerals = !!fd.get('is_critical_minerals');
        state.domestic_content_compliant = !!fd.get('domestic_content_compliant');
        state.elect_direct_pay = !!fd.get('elect_direct_pay');
        state.elect_transferability = !!fd.get('elect_transferability');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s48C-output');
    if (!el) return;
    const fullRate = state.prevailing_wage && state.apprenticeship ? 0.30 : 0.06;
    const allocationCap = state.allocation_amount;
    const computedCredit = state.project_basis * fullRate;
    const allowedCredit = state.doe_allocation_received ? Math.min(computedCredit, allocationCap) : 0;
    const transferProceeds = state.elect_transferability ? allowedCredit * 0.93 : allowedCredit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s48C.h2.result">§ 48C credit computation</h2>
            <div class="cards">
                <div class="card ${state.doe_allocation_received ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s48C.card.allocated">DOE allocation?</div>
                    <div class="value">${state.doe_allocation_received ? esc(t('view.s48C.status.yes')) : esc(t('view.s48C.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s48C.card.rate">Credit rate</div>
                    <div class="value">${(fullRate * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s48C.card.computed">Computed (basis × rate)</div>
                    <div class="value">$${computedCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s48C.card.allocation_cap">Allocation cap</div>
                    <div class="value">$${allocationCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s48C.card.allowed">Allowed credit</div>
                    <div class="value">$${allowedCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s48C.card.transfer">Transfer proceeds (93%)</div>
                    <div class="value">$${transferProceeds.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!state.doe_allocation_received ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s48C.no_alloc_note">
                    NO § 48C credit without DOE allocation. Apply at energy.gov 48C-PORTAL: 8-page concept
                    paper → encouragement letter → 30-page full application → allocation. Process takes 6-9
                    months. Consider § 48 ITC (automatic, no allocation) as alternative.
                </p>
            ` : ''}
        </div>
    `;
}
