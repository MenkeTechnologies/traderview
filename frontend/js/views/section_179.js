// Section 179 Calculator — IRC § 179 immediate expensing.
// Lets you deduct up to $1,160,000 (2024) of qualifying equipment in year-1
// instead of depreciating, subject to business income cap + phase-out.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

// 2024 figures per IRS Rev. Proc. 2023-34
const LIMITS = {
    2024: { max: 1_160_000, phaseout: 2_890_000, suv_cap: 28_900, bonus_rate: 0.60 },
    2025: { max: 1_220_000, phaseout: 3_050_000, suv_cap: 30_500, bonus_rate: 0.40 },
    2026: { max: 1_250_000, phaseout: 3_130_000, suv_cap: 31_300, bonus_rate: 0.20 },
};

const LS_KEY = 'tv-section179-items-v1';

function load() {
    try {
        const raw = localStorage.getItem(LS_KEY);
        return raw ? JSON.parse(raw) : [];
    } catch { return []; }
}
function save(items) {
    try { localStorage.setItem(LS_KEY, JSON.stringify(items)); } catch { /* private mode */ }
}

let state = {
    items: load(),
    biz_income: 0,
    year: new Date().getFullYear(),
};

export async function renderSection179(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sec179.h1.title">// SECTION 179 CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.sec179.hint.intro">
            IRC § 179 — deduct qualifying business equipment immediately instead of
            depreciating over 5-7 years. Big tax win in profitable years. SUV cap
            $28,900 (2024). Add items below and see year-1 deduction vs. carryover.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.sec179.h2.add_item">Add equipment</h2>
            <form id="s179-form" class="inline-form">
                <label><span data-i18n="view.sec179.label.description">Description</span>
                    <input type="text" name="description" placeholder="MacBook Pro / Trading PC / Office furniture" required></label>
                <label><span data-i18n="view.sec179.label.cost">Cost ($)</span>
                    <input type="number" step="0.01" name="cost" min="0" required></label>
                <label><span data-i18n="view.sec179.label.biz_pct">Business use %</span>
                    <input type="number" step="1" name="business_pct" value="100" min="50" max="100"></label>
                <label><span data-i18n="view.sec179.label.is_suv">Is heavy SUV (6000-14000 lbs)?</span>
                    <input type="checkbox" name="is_suv"></label>
                <label><span data-i18n="view.sec179.label.placed_year">Placed in service</span>
                    <select name="placed_year">
                        ${Object.keys(LIMITS).map(y =>
                            `<option value="${y}" ${Number(y) === state.year ? 'selected' : ''}>${y}</option>`
                        ).join('')}
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.sec179.btn.add">Add item</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.sec179.h2.business_income">Business income for limit</h2>
            <form id="s179-biz" class="inline-form">
                <label><span data-i18n="view.sec179.label.biz_income">Net business income ($)</span>
                    <input type="number" step="0.01" name="biz_income" value="${state.biz_income}"></label>
                <label><span data-i18n="view.sec179.label.year">Tax year</span>
                    <select name="year">
                        ${Object.keys(LIMITS).map(y =>
                            `<option value="${y}" ${Number(y) === state.year ? 'selected' : ''}>${y}</option>`
                        ).join('')}
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.sec179.btn.recompute">Recompute</button>
            </form>
        </div>
        <div id="s179-summary"></div>
        <div id="s179-table" class="chart-panel"></div>
    `;
    document.getElementById('s179-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const item = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            description: fd.get('description'),
            cost: Number(fd.get('cost')),
            business_pct: Number(fd.get('business_pct')) || 100,
            is_suv: !!fd.get('is_suv'),
            placed_year: Number(fd.get('placed_year')),
            created_at: new Date().toISOString(),
        };
        state.items.push(item);
        save(state.items);
        e.target.reset();
        e.target.querySelector('[name="business_pct"]').value = 100;
        showToast(t('view.sec179.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s179-biz').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.biz_income = Number(fd.get('biz_income')) || 0;
        state.year = Number(fd.get('year'));
        render();
    });
    render();
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s179-summary');
    if (!el) return;
    const limits = LIMITS[state.year] || LIMITS[2024];
    const yearItems = state.items.filter(i => i.placed_year === state.year);
    const grossCost = yearItems.reduce((s, i) =>
        s + (i.is_suv ? Math.min(i.cost * (i.business_pct / 100), limits.suv_cap) : i.cost * (i.business_pct / 100)), 0);
    // Phase-out: $1-for-$1 reduction once total purchases exceed threshold.
    const phaseOutAmount = Math.max(0, grossCost - limits.phaseout);
    const maxAfterPhaseout = Math.max(0, limits.max - phaseOutAmount);
    const requested = Math.min(grossCost, maxAfterPhaseout);
    // Business income cap.
    const allowed = Math.min(requested, Math.max(0, state.biz_income));
    const carryover = requested - allowed;
    // Bonus depreciation on whatever doesn't fit Section 179.
    const remaining = grossCost - allowed - carryover;
    const bonus = remaining * limits.bonus_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.sec179.h2.summary">Year-${state.year} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.sec179.card.gross_cost">Gross qualifying cost</div>
                    <div class="value">$${grossCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.sec179.card.section_179">Section 179 deduction</div>
                    <div class="value">$${allowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.sec179.card.bonus_depr">Bonus depreciation</div>
                    <div class="value">$${bonus.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sec179.card.carryover">Carryover to next year</div>
                    <div class="value">$${carryover.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sec179.card.phaseout">Phase-out reduction</div>
                    <div class="value">$${phaseOutAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.sec179.card.year_1_total">Year-1 total deduction</div>
                    <div class="value">$${(allowed + bonus).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            <p class="muted small" data-i18n="view.sec179.note">
                Section 179 capped at net business income; excess carries forward.
                Bonus depreciation has NO income cap and can create a NOL.
            </p>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s179-table');
    if (!el) return;
    if (!state.items.length) {
        el.innerHTML = `<h2 data-i18n="view.sec179.h2.items">Equipment items</h2>
            <p class="muted" data-i18n="view.sec179.empty">No items yet.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.sec179.h2.items">Equipment items</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.sec179.th.year">Year</th>
                <th data-i18n="view.sec179.th.description">Description</th>
                <th data-i18n="view.sec179.th.cost">Cost</th>
                <th data-i18n="view.sec179.th.biz_pct">Biz %</th>
                <th data-i18n="view.sec179.th.suv">SUV</th>
                <th data-i18n="view.sec179.th.qualifying">Qualifying</th>
                <th data-i18n="view.sec179.th.actions">Actions</th>
            </tr></thead>
            <tbody>${[...state.items].sort((a, b) => b.placed_year - a.placed_year).map(i => {
                const limits = LIMITS[i.placed_year] || LIMITS[2024];
                const qual = i.is_suv
                    ? Math.min(i.cost * (i.business_pct / 100), limits.suv_cap)
                    : i.cost * (i.business_pct / 100);
                return `<tr>
                    <td>${i.placed_year}</td>
                    <td>${esc(i.description)}</td>
                    <td>$${i.cost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${i.business_pct}%</td>
                    <td>${i.is_suv ? '✓' : ''}</td>
                    <td class="pos">$${qual.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(i.id)}" data-i18n="view.sec179.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.items = state.items.filter(it => it.id !== btn.dataset.del);
            save(state.items);
            render();
        });
    });
}
