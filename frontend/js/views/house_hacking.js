// House Hacking — buy a small multi-unit, live in one, rent the rest.
// Nets rental income against the carrying cost to show what you actually
// pay to live there, via /calc/house-hacking.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import * as enh from '../calc_enhance.js';

const VIEW = 'house-hacking';
let lastReport = null;
let lastBody = null;

const FIELDS = [
    ['home_price_usd', 'Home price ($)', 400000],
    ['down_payment_usd', 'Down payment ($)', 80000],
    ['apr_pct', 'Mortgage APR (%)', 6],
    ['term_months', 'Term (months)', 360],
    ['total_units', 'Total units', 2],
    ['owner_units', 'Units you occupy', 1],
    ['rent_per_unit_usd', 'Rent per unit ($/mo)', 1500],
    ['monthly_tax_usd', 'Property tax ($/mo)', 400],
    ['monthly_insurance_usd', 'Insurance ($/mo)', 100],
    ['monthly_maintenance_usd', 'Maintenance ($/mo)', 200],
    ['monthly_hoa_usd', 'HOA ($/mo)', 0],
    ['comparable_rent_usd', 'Comparable rent for you ($/mo)', 1800],
];
const INT_FIELDS = new Set(['term_months', 'total_units', 'owner_units']);

const money = (n) => (n < 0 ? '-$' : '$') + Math.abs(Number(n)).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderHouseHacking(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.househack.h1.title">// HOUSE HACKING</span></h1>
        <p class="muted small" data-i18n="view.househack.hint.intro">
            Buy a 2–4 unit property, live in one unit, rent the rest. The tenants' rent
            offsets your carrying cost (mortgage + tax + insurance + maintenance + HOA),
            so the money that would have been rent builds equity instead. This shows what
            you actually pay to live there, how it compares to renting, and the property's
            cash flow once you move out and rent every unit.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.househack.h2.inputs">The deal</h2>
            <form id="hh-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.househack.label.${key}">${label}</span>
                        <input type="number" step="${INT_FIELDS.has(key) ? '1' : '0.01'}" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                <button class="primary" type="submit" data-i18n="view.househack.btn.run">Run the numbers</button>
            </form>
            <div id="hh-tools" class="ce-toolbar"></div>
        </div>
        <div id="hh-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#hh-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        return body;
    };
    enh.mountToolbar(mount.querySelector('#hh-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'house-hacking.csv' });
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const body = readBody();
        try {
            const r = await api.calcHouseHacking(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.househack.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['monthly_pi_usd', r.monthly_pi_usd],
        ['rental_income_usd', r.rental_income_usd],
        ['total_housing_cost_usd', r.total_housing_cost_usd],
        ['net_housing_cost_usd', r.net_housing_cost_usd],
        ['savings_vs_renting_usd', r.savings_vs_renting_usd],
        ['full_rental_cash_flow_usd', r.full_rental_cash_flow_usd],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#hh-result');
    const netCls = Number(r.net_housing_cost_usd) <= 0 ? 'pos' : '';
    // Carrying cost vs rental income vs your net housing cost.
    const chart = enh.svgBarChart([
        { label: 'Carry', value: -r.total_housing_cost_usd },
        { label: 'Rent', value: r.rental_income_usd },
        { label: 'Net cost', value: -r.net_housing_cost_usd },
    ]);
    const saveCls = Number(r.savings_vs_renting_usd) >= 0 ? 'pos' : 'neg';
    const cfCls = Number(r.full_rental_cash_flow_usd) >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.househack.h2.result">The numbers</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.househack.card.pi">Mortgage P&amp;I</div>
                    <div class="value">${money(r.monthly_pi_usd)}/mo</div></div>
                <div class="card"><div class="label" data-i18n="view.househack.card.income">Rental income (${r.rented_units})</div>
                    <div class="value pos">${money(r.rental_income_usd)}/mo</div></div>
                <div class="card"><div class="label" data-i18n="view.househack.card.carry">Total carrying cost</div>
                    <div class="value">${money(r.total_housing_cost_usd)}/mo</div></div>
                <div class="card ${netCls ? 'pos' : ''}"><div class="label" data-i18n="view.househack.card.net">Your net housing cost</div>
                    <div class="value ${netCls}">${money(r.net_housing_cost_usd)}/mo</div></div>
                <div class="card"><div class="label" data-i18n="view.househack.card.savings">Saved vs renting</div>
                    <div class="value ${saveCls}">${money(r.savings_vs_renting_usd)}/mo</div></div>
                <div class="card"><div class="label" data-i18n="view.househack.card.cashflow">Cash flow if you move out</div>
                    <div class="value ${cfCls}">${money(r.full_rental_cash_flow_usd)}/mo</div></div>
            </div>
            ${chart}
            <p class="muted small">${r.rent_covers_pi
                ? `<span class="pos" data-i18n="view.househack.note.covers">One unit's rent already covers the mortgage P&amp;I.</span>`
                : `<span data-i18n="view.househack.note.partial">The rented units cover part of the carrying cost — the rest is your housing budget.</span>`}</p>
        </div>
    `;
    applyUiI18n(el);
}
