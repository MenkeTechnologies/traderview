// Meal Deduction Tracker — IRC § 274(n).
// Default 50% deductible. Specific exceptions: 100% deductible for
// (1) employee meals on premises, (2) office snacks for staff (until
// 2025 under TCJA), (3) meals provided at company events open to all,
// (4) reimbursed meals at retail value, (5) meals included as W-2 wage.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-meals-v1';

const CATEGORIES = [
    { value: 'client', label: 'Client meeting',      deductible: 0.50 },
    { value: 'travel', label: 'Business travel',     deductible: 0.50 },
    { value: 'conf',   label: 'Conference / meal',   deductible: 0.50 },
    { value: 'staff',  label: 'Staff event (100%)',  deductible: 1.00 },
    { value: 'event',  label: 'Office party (100%)', deductible: 1.00 },
    { value: 'reimb',  label: 'Reimbursed retail',   deductible: 1.00 },
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(meals) {
    try { localStorage.setItem(LS_KEY, JSON.stringify(meals)); } catch { /* ignore */ }
}

let state = {
    meals: load(),
    filterYear: new Date().getFullYear(),
};

export async function renderMealDeduction(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.meals.h1.title">// MEAL DEDUCTION TRACKER</span></h1>
        <p class="muted small" data-i18n="view.meals.hint.intro">
            IRC § 274(n) — generally 50% deductible. Track per-meal so audit-ready.
            Required substantiation: date, amount, place, business purpose, names
            of attendees + business relationship.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.meals.h2.add">Add meal</h2>
            <form id="ml-form" class="inline-form">
                <label><span data-i18n="view.meals.label.date">Date</span>
                    <input type="date" name="date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.meals.label.category">Category</span>
                    <select name="category">${CATEGORIES.map(c =>
                        `<option value="${c.value}">${esc(c.label)}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.meals.label.place">Place / vendor</span>
                    <input type="text" name="place" placeholder="The Capital Grille" required></label>
                <label><span data-i18n="view.meals.label.attendees">Attendees</span>
                    <input type="text" name="attendees" placeholder="John Doe (Acme Corp CFO)" required></label>
                <label><span data-i18n="view.meals.label.purpose">Business purpose</span>
                    <input type="text" name="purpose" placeholder="Q3 contract negotiation" required></label>
                <label><span data-i18n="view.meals.label.cost">Total cost ($)</span>
                    <input type="number" step="0.01" name="cost" required></label>
                <label><span data-i18n="view.meals.label.tip">Tip ($, included in cost)</span>
                    <input type="number" step="0.01" name="tip" value="0"></label>
                <button class="primary" type="submit" data-i18n="view.meals.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.meals.label.year">View year</span>
                    <input type="number" id="ml-filter" value="${state.filterYear}"></label>
            </div>
        </div>
        <div id="ml-summary"></div>
        <div id="ml-table" class="chart-panel"></div>
    `;
    document.getElementById('ml-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const cat = CATEGORIES.find(c => c.value === fd.get('category')) || CATEGORIES[0];
        const m = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            category: cat.value,
            deductible_rate: cat.deductible,
            place: fd.get('place'),
            attendees: fd.get('attendees'),
            purpose: fd.get('purpose'),
            cost: Number(fd.get('cost')),
            tip: Number(fd.get('tip')) || 0,
        };
        state.meals.push(m);
        save(state.meals);
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0, 10);
        e.target.querySelector('[name="tip"]').value = 0;
        showToast(t('view.meals.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('ml-filter').addEventListener('change', e => {
        state.filterYear = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    render();
}

function render() {
    const yearMeals = state.meals.filter(m => new Date(m.date).getFullYear() === state.filterYear);
    renderSummary(yearMeals);
    renderTable(yearMeals);
}

function renderSummary(meals) {
    const el = document.getElementById('ml-summary');
    if (!el) return;
    const totalSpend = meals.reduce((s, m) => s + Number(m.cost || 0), 0);
    const totalDeductible = meals.reduce((s, m) =>
        s + Number(m.cost || 0) * Number(m.deductible_rate || 0.5), 0);
    const byCategory = new Map();
    for (const m of meals) {
        byCategory.set(m.category, (byCategory.get(m.category) || 0) + Number(m.cost || 0));
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.meals.h2.summary">${state.filterYear} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.meals.card.count">Meals</div>
                    <div class="value">${meals.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.meals.card.total">Total spend</div>
                    <div class="value">$${totalSpend.toLocaleString(undefined, { maximumFractionDigits: 2 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.meals.card.deductible">Deductible</div>
                    <div class="value">$${totalDeductible.toLocaleString(undefined, { maximumFractionDigits: 2 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.meals.card.avg">Avg per meal</div>
                    <div class="value">$${meals.length ? (totalSpend / meals.length).toFixed(2) : '0.00'}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(meals) {
    const el = document.getElementById('ml-table');
    if (!el) return;
    if (!meals.length) {
        el.innerHTML = `<h2 data-i18n="view.meals.h2.log">Log</h2>
            <p class="muted" data-i18n="view.meals.empty">No meals logged for this year.</p>`;
        return;
    }
    const sorted = [...meals].sort((a, b) =>
        String(b.date).localeCompare(String(a.date)));
    el.innerHTML = `
        <h2 data-i18n="view.meals.h2.log">Log</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.meals.th.date">Date</th>
                <th data-i18n="view.meals.th.category">Category</th>
                <th data-i18n="view.meals.th.place">Place</th>
                <th data-i18n="view.meals.th.attendees">Attendees</th>
                <th data-i18n="view.meals.th.purpose">Purpose</th>
                <th data-i18n="view.meals.th.cost">Cost</th>
                <th data-i18n="view.meals.th.rate">Rate</th>
                <th data-i18n="view.meals.th.deductible">Deductible</th>
                <th data-i18n="view.meals.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(m => {
                const cat = CATEGORIES.find(c => c.value === m.category) || CATEGORIES[0];
                const ded = Number(m.cost || 0) * Number(m.deductible_rate || 0.5);
                return `<tr>
                    <td>${esc(m.date)}</td>
                    <td class="muted">${esc(cat.label.split(' (')[0])}</td>
                    <td>${esc(m.place)}</td>
                    <td class="muted">${esc((m.attendees || '').slice(0, 40))}</td>
                    <td class="muted">${esc((m.purpose || '').slice(0, 40))}</td>
                    <td>$${Number(m.cost || 0).toFixed(2)}</td>
                    <td>${(Number(m.deductible_rate || 0.5) * 100).toFixed(0)}%</td>
                    <td class="pos">$${ded.toFixed(2)}</td>
                    <td><button class="link neg" data-del="${esc(m.id)}" data-i18n="view.meals.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.meals = state.meals.filter(m => m.id !== btn.dataset.del);
            save(state.meals);
            render();
        });
    });
}
