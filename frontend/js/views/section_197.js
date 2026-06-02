// IRC § 197 — 15-year amortization of acquired intangibles.
// Goodwill, going concern, workforce in place, customer list, supplier list,
// know-how, books & records, patents/copyrights/film/software (purchased), license,
// covenant not to compete, franchise/trademark/trade name.
// Straight-line over 180 months from acquisition month.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-section-197-v1';
const AMORT_MONTHS = 180;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    intangibles: load(),
    marginal_rate: 0.32,
};

export async function renderSection197(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s197.h1.title">// § 197 INTANGIBLE AMORTIZATION</span></h1>
        <p class="muted small" data-i18n="view.s197.hint.intro">
            Straight-line 15-year amortization on purchased intangibles. Covers goodwill,
            going concern, workforce in place, customer / supplier list, covenants not to
            compete, franchise / trademark / trade name, purchased patents / copyrights /
            software, license / permit / certificate. Self-created goodwill NOT subject to
            § 197 — capitalize as zero basis. <strong>180-month line, mid-month convention.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s197.h2.add">Log intangible</h2>
            <form id="s197-form" class="inline-form">
                <label><span data-i18n="view.s197.label.description">Description</span>
                    <input type="text" name="description" placeholder="Acme Holdings Goodwill" required></label>
                <label><span data-i18n="view.s197.label.category">Category</span>
                    <select name="category">
                        <option value="goodwill">Goodwill</option>
                        <option value="going_concern">Going concern</option>
                        <option value="workforce">Workforce in place</option>
                        <option value="customer_list">Customer list</option>
                        <option value="supplier_list">Supplier list</option>
                        <option value="know_how">Know-how / methods</option>
                        <option value="patents">Patents / copyrights (purchased)</option>
                        <option value="franchise">Franchise / trademark / trade name</option>
                        <option value="license">License / permit</option>
                        <option value="non_compete">Covenant not to compete</option>
                        <option value="software">Purchased software</option>
                        <option value="other">Other § 197</option>
                    </select>
                </label>
                <label><span data-i18n="view.s197.label.cost">Cost basis ($)</span>
                    <input type="number" step="100" name="cost" required></label>
                <label><span data-i18n="view.s197.label.acq_year">Acquisition year</span>
                    <input type="number" step="1" name="acq_year" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.s197.label.acq_month">Acquisition month (1-12)</span>
                    <input type="number" step="1" min="1" max="12" name="acq_month" value="1" required></label>
                <button class="primary" type="submit" data-i18n="view.s197.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.s197.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" id="s197-marginal" value="${state.marginal_rate}"></label>
            </div>
        </div>
        <div id="s197-summary"></div>
        <div id="s197-table" class="chart-panel"></div>
    `;
    document.getElementById('s197-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const i = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            description: fd.get('description'),
            category: fd.get('category'),
            cost: Number(fd.get('cost')) || 0,
            acq_year: Number(fd.get('acq_year')),
            acq_month: Math.max(1, Math.min(12, Number(fd.get('acq_month')))),
        };
        state.intangibles.push(i);
        save(state.intangibles);
        e.target.reset();
        e.target.querySelector('[name="acq_year"]').value = new Date().getFullYear();
        e.target.querySelector('[name="acq_month"]').value = 1;
        showToast(t('view.s197.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s197-marginal').addEventListener('change', e => {
        state.marginal_rate = Number(e.target.value) || 0.32;
        render();
    });
    render();
}

function monthsAmortized(i, now = new Date()) {
    const startMonth = (i.acq_year * 12) + (i.acq_month - 1);
    const nowMonth = (now.getFullYear() * 12) + now.getMonth();
    const months = Math.max(0, nowMonth - startMonth + 1);
    return Math.min(months, AMORT_MONTHS);
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s197-summary');
    if (!el) return;
    const totalCost = state.intangibles.reduce((s, i) => s + i.cost, 0);
    const annualAmort = state.intangibles.reduce((s, i) => s + (i.cost / 15), 0);
    const monthlyAmort = annualAmort / 12;
    const annualSavings = annualAmort * state.marginal_rate;
    const totalRemaining = state.intangibles.reduce((s, i) => {
        const used = monthsAmortized(i);
        return s + (i.cost * (AMORT_MONTHS - used) / AMORT_MONTHS);
    }, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s197.h2.summary">Amortization summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s197.card.intangibles">Intangibles</div>
                    <div class="value">${state.intangibles.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s197.card.total_cost">Total cost basis</div>
                    <div class="value">$${totalCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s197.card.annual_amort">Annual amortization</div>
                    <div class="value">$${annualAmort.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s197.card.monthly_amort">Monthly amortization</div>
                    <div class="value">$${monthlyAmort.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s197.card.annual_savings">Annual tax savings</div>
                    <div class="value">$${annualSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s197.card.remaining">Remaining basis</div>
                    <div class="value">$${totalRemaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s197-table');
    if (!el) return;
    if (!state.intangibles.length) {
        el.innerHTML = `<h2 data-i18n="view.s197.h2.intangibles">Intangibles</h2>
            <p class="muted" data-i18n="view.s197.empty">No § 197 intangibles logged.</p>`;
        return;
    }
    const sorted = [...state.intangibles].sort((a, b) => b.cost - a.cost);
    el.innerHTML = `
        <h2 data-i18n="view.s197.h2.intangibles">Intangibles</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s197.th.description">Description</th>
                <th data-i18n="view.s197.th.category">Category</th>
                <th data-i18n="view.s197.th.acquired">Acquired</th>
                <th data-i18n="view.s197.th.cost">Cost</th>
                <th data-i18n="view.s197.th.annual">Annual</th>
                <th data-i18n="view.s197.th.used_months">Months used</th>
                <th data-i18n="view.s197.th.remaining">Remaining</th>
                <th data-i18n="view.s197.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(i => {
                const monthly = i.cost / AMORT_MONTHS;
                const used = monthsAmortized(i);
                const remaining = i.cost * (AMORT_MONTHS - used) / AMORT_MONTHS;
                return `<tr>
                    <td>${esc(i.description)}</td>
                    <td class="muted">${esc(i.category)}</td>
                    <td class="muted">${i.acq_year}-${String(i.acq_month).padStart(2, '0')}</td>
                    <td>$${i.cost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${(monthly * 12).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${used} / ${AMORT_MONTHS}</td>
                    <td>$${remaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(i.id)}" data-i18n="view.s197.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.intangibles = state.intangibles.filter(i => i.id !== btn.dataset.del);
            save(state.intangibles);
            render();
        });
    });
}
