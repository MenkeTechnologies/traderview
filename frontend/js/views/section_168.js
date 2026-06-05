// IRC § 168 MACRS Depreciation Schedule generator.
// Standard recovery periods: 3, 5, 7, 10, 15, 20-yr (200% DB); 27.5-yr (residential
// rental, SL); 39-yr (nonres real, SL). Half-year vs mid-quarter vs mid-month conventions.
// QIP fixed by CARES Act to 15-yr from 39-yr.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-macrs-schedule-v1';

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    assets: load(),
    marginal_rate: 0.32,
};

const TABLE_HY = {
    3: [0.3333, 0.4445, 0.1481, 0.0741],
    5: [0.20, 0.32, 0.192, 0.1152, 0.1152, 0.0576],
    7: [0.1429, 0.2449, 0.1749, 0.1249, 0.0893, 0.0892, 0.0893, 0.0446],
    10: [0.10, 0.18, 0.144, 0.1152, 0.0922, 0.0737, 0.0655, 0.0655, 0.0656, 0.0655, 0.0328],
    15: [0.05, 0.095, 0.0855, 0.077, 0.0693, 0.0623, 0.059, 0.059, 0.0591, 0.059, 0.0591, 0.059, 0.0591, 0.059, 0.0591, 0.0295],
    20: [0.0375, 0.07219, 0.06677, 0.06177, 0.05713, 0.05285, 0.04888, 0.04522, 0.04462, 0.04461, 0.04462, 0.04461, 0.04462, 0.04461, 0.04462, 0.04461, 0.04462, 0.04461, 0.04462, 0.04461, 0.02231],
    275: 27.5,  // straight-line
    39: 39,     // straight-line
};

export async function renderSection168(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s168.h1.title">// § 168 MACRS DEPRECIATION SCHEDULE</span></h1>
        <p class="muted small" data-i18n="view.s168.hint.intro">
            Standard MACRS recovery periods: <strong>3-yr (tools), 5-yr (computers, autos),
            7-yr (office furniture), 15-yr (QIP, land improvements), 20-yr (farm buildings),
            27.5-yr (residential rental, SL), 39-yr (non-residential real, SL)</strong>.
            Half-year convention for most personal property; mid-quarter if &gt; 40% Q4;
            mid-month for real property. QIP fixed by CARES Act 2020.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s168.h2.add">Log asset</h2>
            <form id="s168-form" class="inline-form">
                <label><span data-i18n="view.s168.label.description">Description</span>
                    <input type="text" name="description" required></label>
                <label><span data-i18n="view.s168.label.cost">Cost basis ($)</span>
                    <input type="number" step="0.01" name="cost" required></label>
                <label><span data-i18n="view.s168.label.placed_year">Placed in service year</span>
                    <input type="number" step="1" name="placed_year" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.s168.label.placed_month">Placed in service month</span>
                    <input type="number" step="1" min="1" max="12" name="placed_month" value="1" required></label>
                <label><span data-i18n="view.s168.label.class_years">MACRS class</span>
                    <select name="class_years">
                        <option value="3">3-yr (200% DB) — tools, racehorses</option>
                        <option value="5" selected>5-yr (200% DB) — computers, autos, trucks</option>
                        <option value="7">7-yr (200% DB) — office furniture, equipment</option>
                        <option value="10">10-yr — boats, single-purpose ag</option>
                        <option value="15">15-yr (150% DB) — QIP, land improvements</option>
                        <option value="20">20-yr (150% DB) — farm buildings</option>
                        <option value="275">27.5-yr SL — residential rental</option>
                        <option value="39">39-yr SL — non-residential real</option>
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.s168.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.s168.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" id="s168-marginal" value="${state.marginal_rate}"></label>
            </div>
        </div>
        <div id="s168-summary"></div>
        <div id="s168-assets" class="chart-panel"></div>
    `;
    document.getElementById('s168-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const a = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            description: fd.get('description'),
            cost: Number(fd.get('cost')) || 0,
            placed_year: Number(fd.get('placed_year')),
            placed_month: Math.max(1, Math.min(12, Number(fd.get('placed_month')) || 1)),
            class_years: Number(fd.get('class_years')) || 5,
        };
        state.assets.push(a);
        save(state.assets);
        e.target.reset();
        e.target.querySelector('[name="placed_year"]').value = new Date().getFullYear();
        e.target.querySelector('[name="placed_month"]').value = 1;
        showToast(t('view.s168.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s168-marginal').addEventListener('change', e => {
        state.marginal_rate = Number(e.target.value) || 0.32;
        render();
    });
    render();
}

function depForYear(asset, taxYear) {
    const yearOffset = taxYear - asset.placed_year + 1;
    if (yearOffset < 1) return 0;
    const t = TABLE_HY[asset.class_years];
    if (typeof t === 'number') {
        // Straight-line for real property
        const totalLife = t;
        if (yearOffset > totalLife + 1) return 0;
        const monthsRemainingY1 = 12 - asset.placed_month + 0.5;
        if (yearOffset === 1) return asset.cost / totalLife * (monthsRemainingY1 / 12);
        if (yearOffset > totalLife) {
            const usedY1Months = monthsRemainingY1;
            const remainingMonths = 12 - usedY1Months;
            return asset.cost / totalLife * (remainingMonths / 12);
        }
        return asset.cost / totalLife;
    }
    return (t[yearOffset - 1] || 0) * asset.cost;
}

function render() {
    renderSummary();
    renderAssets();
}

function renderSummary() {
    const el = document.getElementById('s168-summary');
    if (!el) return;
    const currentYear = new Date().getFullYear();
    let totalCost = 0, totalYearDep = 0, totalAccumDep = 0;
    for (const a of state.assets) {
        totalCost += a.cost;
        totalYearDep += depForYear(a, currentYear);
        for (let y = a.placed_year; y <= currentYear; y++) {
            totalAccumDep += depForYear(a, y);
        }
    }
    const yearSavings = totalYearDep * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s168.h2.summary">Summary (${currentYear})</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s168.card.asset_count">Assets</div>
                    <div class="value">${state.assets.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s168.card.total_cost">Total cost</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168.card.year_dep">Year depreciation</div>
                    <div class="value">$${totalYearDep.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s168.card.accum_dep">Accum depreciation</div>
                    <div class="value">$${totalAccumDep.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s168.card.year_savings">Year tax savings</div>
                    <div class="value">$${yearSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderAssets() {
    const el = document.getElementById('s168-assets');
    if (!el) return;
    if (!state.assets.length) {
        el.innerHTML = `<h2 data-i18n="view.s168.h2.assets">Assets</h2>
            <p class="muted" data-i18n="view.s168.empty">No assets logged.</p>`;
        return;
    }
    const currentYear = new Date().getFullYear();
    const sorted = [...state.assets].sort((a, b) => b.cost - a.cost);
    el.innerHTML = `
        <h2 data-i18n="view.s168.h2.assets">Assets</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s168.th.description">Description</th>
                <th data-i18n="view.s168.th.cost">Cost</th>
                <th data-i18n="view.s168.th.placed">Placed</th>
                <th data-i18n="view.s168.th.class">Class</th>
                <th data-i18n="view.s168.th.year_dep">${currentYear} Dep.</th>
                <th data-i18n="view.s168.th.savings">Tax savings</th>
                <th data-i18n="view.s168.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(a => {
                const yd = depForYear(a, currentYear);
                return `<tr>
                    <td>${esc(a.description)}</td>
                    <td>$${a.cost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${a.placed_year}-${String(a.placed_month).padStart(2, '0')}</td>
                    <td class="muted">${a.class_years === 275 ? '27.5' : a.class_years}-yr</td>
                    <td class="pos">$${yd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${(yd * state.marginal_rate).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(a.id)}" data-i18n="view.s168.btn.delete">delete</button></td>
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
