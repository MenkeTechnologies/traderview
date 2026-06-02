// Equipment Depreciation Tracker — MACRS half-year + mid-quarter convention.
// Computes year-by-year depreciation schedule for assets that didn't
// qualify (or didn't elect) for full Section 179 expensing.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-depreciation-v1';

// MACRS GDS rates — half-year convention.
const MACRS = {
    '3': [0.3333, 0.4445, 0.1481, 0.0741],
    '5': [0.2000, 0.3200, 0.1920, 0.1152, 0.1152, 0.0576],
    '7': [0.1429, 0.2449, 0.1749, 0.1249, 0.0893, 0.0892, 0.0893, 0.0446],
    '10':[0.1000, 0.1800, 0.1440, 0.1152, 0.0922, 0.0737, 0.0655, 0.0655, 0.0656, 0.0655, 0.0328],
    '15':[0.05, 0.095, 0.0855, 0.077, 0.0693, 0.0623, 0.0590, 0.0590, 0.0591, 0.0590, 0.0591, 0.0590, 0.0591, 0.0590, 0.0591, 0.0295],
};

const ASSET_TYPES = [
    { value: '3',  label: '3-yr (some race horses, qualified rent-to-own)' },
    { value: '5',  label: '5-yr (computers, vehicles, office equip)' },
    { value: '7',  label: '7-yr (office furniture, ag machinery)' },
    { value: '10', label: '10-yr (boats, single-purpose ag structures)' },
    { value: '15', label: '15-yr (qualified improvement property)' },
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(assets) { try { localStorage.setItem(LS_KEY, JSON.stringify(assets)); } catch { /* ignore */ } }

let state = { assets: load() };

export async function renderDepreciation(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.depr.h1.title">// EQUIPMENT DEPRECIATION</span></h1>
        <p class="muted small" data-i18n="view.depr.hint.intro">
            MACRS GDS half-year convention schedule. Each asset gets a per-year
            depreciation expense over its class life. Pair with Section 179 for
            year-1 immediate expensing; whatever doesn't fit Sec 179 depreciates here.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.depr.h2.add">Add asset</h2>
            <form id="dp-form" class="inline-form">
                <label><span data-i18n="view.depr.label.description">Description</span>
                    <input type="text" name="description" placeholder="Trading PC / desk / vehicle" required></label>
                <label><span data-i18n="view.depr.label.cost">Cost basis ($)</span>
                    <input type="number" step="0.01" name="cost" required></label>
                <label><span data-i18n="view.depr.label.placed_year">Year placed in service</span>
                    <input type="number" step="1" name="placed_year" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.depr.label.class_life">Class life</span>
                    <select name="class_life">${ASSET_TYPES.map(a =>
                        `<option value="${a.value}">${esc(a.label)}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.depr.label.biz_pct">Business use %</span>
                    <input type="number" step="1" name="business_pct" value="100" min="1" max="100"></label>
                <button class="primary" type="submit" data-i18n="view.depr.btn.add">Add</button>
            </form>
        </div>
        <div id="dp-summary"></div>
        <div id="dp-table" class="chart-panel"></div>
    `;
    document.getElementById('dp-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const a = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            description: fd.get('description'),
            cost: Number(fd.get('cost')),
            placed_year: Number(fd.get('placed_year')),
            class_life: fd.get('class_life'),
            business_pct: Number(fd.get('business_pct')) || 100,
        };
        state.assets.push(a);
        save(state.assets);
        e.target.reset();
        e.target.querySelector('[name="placed_year"]').value = new Date().getFullYear();
        e.target.querySelector('[name="business_pct"]').value = 100;
        showToast(t('view.depr.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function render() {
    const currentYear = new Date().getFullYear();
    const yearTotals = computeYearTotals(currentYear);
    renderSummary(yearTotals, currentYear);
    renderTable(currentYear);
}

function computeYearTotals(currentYear) {
    // Returns { thisYear, nextYear, total_remaining }
    let thisYear = 0;
    let nextYear = 0;
    let totalRemaining = 0;
    for (const a of state.assets) {
        const rates = MACRS[a.class_life] || MACRS['5'];
        const base = a.cost * (a.business_pct / 100);
        const yearIndex = currentYear - a.placed_year;
        if (yearIndex >= 0 && yearIndex < rates.length) {
            thisYear += base * rates[yearIndex];
        }
        if (yearIndex + 1 >= 0 && yearIndex + 1 < rates.length) {
            nextYear += base * rates[yearIndex + 1];
        }
        // Total remaining = sum of all rates from current year forward.
        for (let i = Math.max(0, yearIndex); i < rates.length; i++) {
            totalRemaining += base * rates[i];
        }
    }
    return { thisYear, nextYear, totalRemaining };
}

function renderSummary({ thisYear, nextYear, totalRemaining }, currentYear) {
    const el = document.getElementById('dp-summary');
    if (!el) return;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.depr.h2.summary">Depreciation expense</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label">${esc(`${currentYear}`)}</div>
                    <div class="value">$${thisYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label">${esc(`${currentYear + 1}`)}</div>
                    <div class="value">$${nextYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.depr.card.total_remaining">Total remaining</div>
                    <div class="value">$${totalRemaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.depr.card.assets">Assets tracked</div>
                    <div class="value">${state.assets.length}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(currentYear) {
    const el = document.getElementById('dp-table');
    if (!el) return;
    if (!state.assets.length) {
        el.innerHTML = `<h2 data-i18n="view.depr.h2.schedule">Schedule</h2>
            <p class="muted" data-i18n="view.depr.empty">No assets tracked yet.</p>`;
        return;
    }
    const sorted = [...state.assets].sort((a, b) => b.placed_year - a.placed_year);
    el.innerHTML = `
        <h2 data-i18n="view.depr.h2.schedule">Schedule</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.depr.th.description">Description</th>
                <th data-i18n="view.depr.th.placed_year">Placed</th>
                <th data-i18n="view.depr.th.cost">Cost</th>
                <th data-i18n="view.depr.th.basis">Biz basis</th>
                <th data-i18n="view.depr.th.class">Class</th>
                <th data-i18n="view.depr.th.this_year">${esc(`${currentYear}`)}</th>
                <th data-i18n="view.depr.th.next_year">${esc(`${currentYear + 1}`)}</th>
                <th data-i18n="view.depr.th.accumulated">Accumulated</th>
                <th data-i18n="view.depr.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(a => {
                const rates = MACRS[a.class_life] || MACRS['5'];
                const basis = a.cost * (a.business_pct / 100);
                const yearIndex = currentYear - a.placed_year;
                const thisYear = yearIndex >= 0 && yearIndex < rates.length ? basis * rates[yearIndex] : 0;
                const nextYear = yearIndex + 1 >= 0 && yearIndex + 1 < rates.length ? basis * rates[yearIndex + 1] : 0;
                const accumulated = rates
                    .slice(0, Math.max(0, Math.min(yearIndex + 1, rates.length)))
                    .reduce((s, r) => s + basis * r, 0);
                return `<tr>
                    <td>${esc(a.description)}</td>
                    <td>${a.placed_year}</td>
                    <td>$${a.cost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${a.class_life}-yr</td>
                    <td class="pos">$${thisYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${nextYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">$${accumulated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(a.id)}" data-i18n="view.depr.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.assets = state.assets.filter(a => a.id !== btn.dataset.del);
            save(state.assets);
            render();
        });
    });
}
