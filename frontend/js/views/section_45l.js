// IRC § 45L — New Energy Efficient Home Credit (Builder).
// IRA 2022 redesigned: $2,500 (ENERGY STAR single-family) / $5,000 (ZERH) / $2,500 (ES manuf housing)
// / $5,000 (ZERH manuf) / $500-$1,000 per unit multifamily; up to $2,500 / $5,000 with prevailing wage.
// Builder/eligible contractor claims. Through 12/31/2032.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SF_ENERGY_STAR = 2_500;
const SF_ZERH = 5_000;
const MF_BASE_ENERGY_STAR = 500;
const MF_BASE_ZERH = 1_000;
const MF_PW_ENERGY_STAR = 2_500;
const MF_PW_ZERH = 5_000;
const MANUF_ENERGY_STAR = 2_500;
const MANUF_ZERH = 5_000;

let state = {
    dwelling_type: 'single_family',
    units_count: 1,
    certification_level: 'energy_star',
    prevailing_wage_apprenticeship: false,
    placed_in_service_year: 2024,
    builder_marginal_rate: 0.21,
    base_method: false,
};

export async function renderSection45l(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s45l.h1.title">// § 45L NEW ENERGY EFFICIENT HOME</span></h1>
        <p class="muted small" data-i18n="view.s45l.hint.intro">
            <strong>IRA 2022 redesigned:</strong> $2,500 ENERGY STAR single-family / $5,000 ZERH
            (Zero Energy Ready Home) / multifamily $500-$1,000 per unit ($2,500-$5,000 with
            prevailing wage + apprenticeship). Builder / eligible contractor claims. <strong>Through
            12/31/2032</strong>. Certification by qualified rater. Form 8908.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s45l.h2.inputs">Inputs</h2>
            <form id="s45l-form" class="inline-form">
                <label><span data-i18n="view.s45l.label.dwelling">Dwelling type</span>
                    <select name="dwelling_type">
                        <option value="single_family" ${state.dwelling_type === 'single_family' ? 'selected' : ''}>Single family</option>
                        <option value="manufactured" ${state.dwelling_type === 'manufactured' ? 'selected' : ''}>Manufactured</option>
                        <option value="multifamily" ${state.dwelling_type === 'multifamily' ? 'selected' : ''}>Multifamily</option>
                    </select>
                </label>
                <label><span data-i18n="view.s45l.label.units">Units count</span>
                    <input type="number" step="1" name="units_count" value="${state.units_count}"></label>
                <label><span data-i18n="view.s45l.label.cert">Certification level</span>
                    <select name="certification_level">
                        <option value="energy_star" ${state.certification_level === 'energy_star' ? 'selected' : ''}>ENERGY STAR</option>
                        <option value="zerh" ${state.certification_level === 'zerh' ? 'selected' : ''}>Zero Energy Ready Home (ZERH)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s45l.label.pw">Prevailing wage + apprenticeship?</span>
                    <input type="checkbox" name="prevailing_wage_apprenticeship" ${state.prevailing_wage_apprenticeship ? 'checked' : ''}></label>
                <label><span data-i18n="view.s45l.label.year">Placed in service year</span>
                    <input type="number" step="1" name="placed_in_service_year" value="${state.placed_in_service_year}"></label>
                <label><span data-i18n="view.s45l.label.marginal">Builder marginal rate</span>
                    <input type="number" step="0.01" name="builder_marginal_rate" value="${state.builder_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s45l.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s45l-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45l.h2.credit_amounts">Credit amounts (post-IRA)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s45l.th.type">Type</th>
                    <th data-i18n="view.s45l.th.energy_star">ENERGY STAR</th>
                    <th data-i18n="view.s45l.th.zerh">Zero Energy Ready</th>
                </tr></thead>
                <tbody>
                    <tr><td>Single family</td><td>$2,500</td><td>$5,000</td></tr>
                    <tr><td>Manufactured home</td><td>$2,500</td><td>$5,000</td></tr>
                    <tr><td>Multifamily (base)</td><td>$500/unit</td><td>$1,000/unit</td></tr>
                    <tr><td>Multifamily (prevailing wage + apprenticeship)</td><td>$2,500/unit</td><td>$5,000/unit</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45l.h2.who_qualifies">Who qualifies</h2>
            <ul class="muted small">
                <li data-i18n="view.s45l.who.builder">Eligible contractor: built / produced + sold or leased</li>
                <li data-i18n="view.s45l.who.taxpayer">Single taxpayer/entity per home (no double-claiming)</li>
                <li data-i18n="view.s45l.who.manufacturer">Mobile home: manufacturer who produced</li>
                <li data-i18n="view.s45l.who.transactions">Multifamily: developer or original purchaser</li>
                <li data-i18n="view.s45l.who.rater">Independent qualified certifier required</li>
                <li data-i18n="view.s45l.who.cert_doc">Certification documentation retained for at least 3 yrs</li>
                <li data-i18n="view.s45l.who.lihtc">LIHTC interaction: § 45L credit may reduce eligible basis</li>
                <li data-i18n="view.s45l.who.transfer">Credit transferable to third party (IRA 2022 § 6418)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s45l.h2.zerh_requirements">ZERH (Zero Energy Ready Home) requirements</h2>
            <p class="muted small" data-i18n="view.s45l.zerh.body">
                DOE Zero Energy Ready Home: ~40-50% more efficient than typical new construction.
                Must meet ENERGY STAR + EPA Indoor airPLUS + WaterSense + EPA Renewable Energy
                Ready Home (RERH) checklist. Renewable energy system (solar) typically required.
                Targets net-zero energy use when paired with renewable system.
            </p>
        </div>
    `;
    document.getElementById('s45l-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.dwelling_type = fd.get('dwelling_type');
        state.units_count = Number(fd.get('units_count')) || 1;
        state.certification_level = fd.get('certification_level');
        state.prevailing_wage_apprenticeship = !!fd.get('prevailing_wage_apprenticeship');
        state.placed_in_service_year = Number(fd.get('placed_in_service_year')) || new Date().getFullYear();
        state.builder_marginal_rate = Number(fd.get('builder_marginal_rate')) || 0.21;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s45l-output');
    if (!el) return;
    let creditPerUnit;
    const isZerh = state.certification_level === 'zerh';
    if (state.dwelling_type === 'single_family') {
        creditPerUnit = isZerh ? SF_ZERH : SF_ENERGY_STAR;
    } else if (state.dwelling_type === 'manufactured') {
        creditPerUnit = isZerh ? MANUF_ZERH : MANUF_ENERGY_STAR;
    } else if (state.dwelling_type === 'multifamily') {
        if (state.prevailing_wage_apprenticeship) {
            creditPerUnit = isZerh ? MF_PW_ZERH : MF_PW_ENERGY_STAR;
        } else {
            creditPerUnit = isZerh ? MF_BASE_ZERH : MF_BASE_ENERGY_STAR;
        }
    } else {
        creditPerUnit = 0;
    }
    const totalCredit = creditPerUnit * state.units_count;
    const cashValue = totalCredit * state.builder_marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s45l.h2.result">§ 45L credit</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s45l.card.per_unit">Credit per unit</div>
                    <div class="value">$${creditPerUnit.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s45l.card.units">Units</div>
                    <div class="value">${state.units_count}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45l.card.total">Total credit</div>
                    <div class="value">$${totalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s45l.card.cash_value">After-tax cash value</div>
                    <div class="value">$${cashValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
