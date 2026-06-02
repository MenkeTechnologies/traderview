// Residential Clean Energy Credit § 25D.
// 30% of qualified expenditures: solar PV, solar water heaters, geothermal
// heat pumps, residential wind, fuel cells, battery storage (3+ kWh).
// Uncapped except fuel cells ($500/kW). 30% rate through 2032, then steps
// down. Lifetime carryforward of unused credit.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-25d-v1';

const RATE_BY_YEAR = {
    2022: 0.30, 2023: 0.30, 2024: 0.30, 2025: 0.30, 2026: 0.30,
    2027: 0.30, 2028: 0.30, 2029: 0.30, 2030: 0.30, 2031: 0.30, 2032: 0.30,
    2033: 0.26, 2034: 0.22, 2035: 0.00,
};

const CATEGORIES = [
    { value: 'solar_pv',           label: 'Solar PV (panels)' },
    { value: 'solar_water_heater', label: 'Solar water heater' },
    { value: 'geothermal',         label: 'Geothermal heat pump' },
    { value: 'residential_wind',   label: 'Residential wind turbine' },
    { value: 'fuel_cell',          label: 'Fuel cell ($500/kW cap)' },
    { value: 'battery_storage',    label: 'Battery storage (3+ kWh)' },
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    projects: load(),
    tax_liability: 30_000,
    prior_year_carryforward: 0,
};

export async function renderCleanEnergy25D(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ce25d.h1.title">// § 25D CLEAN ENERGY CREDIT</span></h1>
        <p class="muted small" data-i18n="view.ce25d.hint.intro">
            <strong>30% federal credit</strong> on solar PV, solar water heaters, geothermal,
            residential wind, fuel cells, battery storage (3+ kWh). Uncapped except fuel cells
            ($500/kW). Rate stays 30% through 2032, then 26% (2033), 22% (2034), 0% (2035+).
            Excess credit carries forward indefinitely against future tax.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.ce25d.h2.add">Add project</h2>
            <form id="ce-form" class="inline-form">
                <label><span data-i18n="view.ce25d.label.year_installed">Year installed</span>
                    <input type="number" step="1" name="year_installed" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.ce25d.label.category">Category</span>
                    <select name="category">${CATEGORIES.map(c =>
                        `<option value="${c.value}">${esc(c.label)}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.ce25d.label.cost">Total qualified cost ($)</span>
                    <input type="number" step="100" name="cost" required></label>
                <label><span data-i18n="view.ce25d.label.kw">kW capacity (fuel cell only)</span>
                    <input type="number" step="0.1" name="kw" value="0"></label>
                <label><span data-i18n="view.ce25d.label.description">Description</span>
                    <input type="text" name="description" placeholder="11.4 kW Enphase / Tesla Powerwall 3"></label>
                <button class="primary" type="submit" data-i18n="view.ce25d.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.ce25d.h2.tax_context">Tax context</h2>
            <form id="ce-tax-form" class="inline-form">
                <label><span data-i18n="view.ce25d.label.tax_liability">Current year tax liability ($)</span>
                    <input type="number" step="100" name="tax_liability" value="${state.tax_liability}"></label>
                <label><span data-i18n="view.ce25d.label.prior_carryforward">Prior carryforward ($)</span>
                    <input type="number" step="100" name="prior_year_carryforward" value="${state.prior_year_carryforward}"></label>
                <button class="primary" type="submit" data-i18n="view.ce25d.btn.update">Update</button>
            </form>
        </div>
        <div id="ce-summary"></div>
        <div id="ce-table" class="chart-panel"></div>
    `;
    document.getElementById('ce-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const p = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year_installed: Number(fd.get('year_installed')),
            category: fd.get('category'),
            cost: Number(fd.get('cost')),
            kw: Number(fd.get('kw')) || 0,
            description: fd.get('description') || '',
        };
        state.projects.push(p);
        save(state.projects);
        e.target.reset();
        e.target.querySelector('[name="year_installed"]').value = new Date().getFullYear();
        e.target.querySelector('[name="kw"]').value = 0;
        showToast(t('view.ce25d.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('ce-tax-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tax_liability = Number(fd.get('tax_liability')) || 0;
        state.prior_year_carryforward = Number(fd.get('prior_year_carryforward')) || 0;
        render();
    });
    render();
}

function projectCredit(p) {
    const rate = RATE_BY_YEAR[p.year_installed] || 0;
    if (p.category === 'fuel_cell') {
        const cap = (p.kw || 0) * 500;
        return Math.min(p.cost * rate, cap);
    }
    return p.cost * rate;
}

function render() {
    const currentYear = new Date().getFullYear();
    const yearTotal = state.projects.filter(p => p.year_installed === currentYear)
        .reduce((s, p) => s + projectCredit(p), 0);
    const lifetime = state.projects.reduce((s, p) => s + projectCredit(p), 0);
    const availableCredit = yearTotal + state.prior_year_carryforward;
    const usedThisYear = Math.min(availableCredit, state.tax_liability);
    const carryforwardNext = availableCredit - usedThisYear;
    renderSummary({ yearTotal, lifetime, availableCredit, usedThisYear, carryforwardNext, currentYear });
    renderTable();
}

function renderSummary({ yearTotal, lifetime, availableCredit, usedThisYear, carryforwardNext, currentYear }) {
    const el = document.getElementById('ce-summary');
    if (!el) return;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ce25d.h2.summary">${currentYear} credit summary</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.ce25d.card.year_credit">${currentYear} credit generated</div>
                    <div class="value">$${yearTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ce25d.card.prior_carry">Prior carryforward</div>
                    <div class="value">$${state.prior_year_carryforward.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ce25d.card.available">Available</div>
                    <div class="value">$${availableCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ce25d.card.used_this_year">Used this year</div>
                    <div class="value">$${usedThisYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${carryforwardNext > 0 ? '' : 'pos'}">
                    <div class="label" data-i18n="view.ce25d.card.carryforward">Carryforward to next year</div>
                    <div class="value">$${carryforwardNext.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ce25d.card.lifetime">Lifetime credits earned</div>
                    <div class="value">$${lifetime.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('ce-table');
    if (!el) return;
    if (!state.projects.length) {
        el.innerHTML = `<h2 data-i18n="view.ce25d.h2.projects">Projects</h2>
            <p class="muted" data-i18n="view.ce25d.empty">No projects logged yet.</p>`;
        return;
    }
    const sorted = [...state.projects].sort((a, b) => b.year_installed - a.year_installed);
    el.innerHTML = `
        <h2 data-i18n="view.ce25d.h2.projects">Projects</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.ce25d.th.year">Year</th>
                <th data-i18n="view.ce25d.th.category">Category</th>
                <th data-i18n="view.ce25d.th.cost">Cost</th>
                <th data-i18n="view.ce25d.th.rate">Rate</th>
                <th data-i18n="view.ce25d.th.credit">Credit</th>
                <th data-i18n="view.ce25d.th.description">Description</th>
                <th data-i18n="view.ce25d.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(p => {
                const cat = CATEGORIES.find(c => c.value === p.category) || CATEGORIES[0];
                const rate = RATE_BY_YEAR[p.year_installed] || 0;
                return `<tr>
                    <td>${p.year_installed}</td>
                    <td class="muted">${esc(cat.label)}</td>
                    <td>$${p.cost.toLocaleString()}</td>
                    <td>${(rate * 100).toFixed(0)}%</td>
                    <td class="pos">$${projectCredit(p).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(p.description || '')}</td>
                    <td><button class="link neg" data-del="${esc(p.id)}" data-i18n="view.ce25d.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.projects = state.projects.filter(p => p.id !== btn.dataset.del);
            save(state.projects);
            render();
        });
    });
}
