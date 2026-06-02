// Accountable Plan Reimbursements — IRC § 62(c) + Treas. Reg. § 1.62-2.
// S-corp owner's secret weapon: business reimburses owner-employee for
// business-purpose expenses (home office, mileage, internet, phone) AT THE
// OWNER LEVEL. Deductible to corp, NON-taxable to owner. Replaces the
// employee 2106 deduction that TCJA killed (2018-2025).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-accountable-v1';

const EXPENSE_CATEGORIES = [
    { value: 'home_office',   label: 'Home office (sqft × utilities/rent ratio)' },
    { value: 'mileage',       label: 'Mileage (IRS rate × business miles)' },
    { value: 'internet_phone', label: 'Internet / cell phone (business %)' },
    { value: 'office_supplies', label: 'Office supplies' },
    { value: 'training',      label: 'Training / education / books' },
    { value: 'subscriptions', label: 'Subscriptions (data feeds, software)' },
    { value: 'travel',        label: 'Travel (airfare, hotel, ground)' },
    { value: 'meals',         label: 'Meals (50% deductible)' },
    { value: 'equipment',     label: 'Equipment (< $2,500 de minimis)' },
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    reimbursements: load(),
    marginal_rate: 0.32,
    se_rate: 0.153,
};

export async function renderAccountablePlan(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.acct.h1.title">// ACCOUNTABLE PLAN REIMBURSEMENTS</span></h1>
        <p class="muted small" data-i18n="view.acct.hint.intro">
            <strong>S-corp owner's secret weapon</strong> — § 62(c) + Treas. Reg. 1.62-2.
            Business reimburses owner-employee for business expenses. Deductible to corp,
            NON-taxable wage exclusion to owner. Replaces the dead 2106 employee deduction
            (killed by TCJA 2018-2025).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.acct.h2.add">Submit expense for reimbursement</h2>
            <form id="ap-form" class="inline-form">
                <label><span data-i18n="view.acct.label.date">Date</span>
                    <input type="date" name="date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.acct.label.category">Category</span>
                    <select name="category">${EXPENSE_CATEGORIES.map(c =>
                        `<option value="${c.value}">${esc(c.label)}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.acct.label.amount">Amount ($)</span>
                    <input type="number" step="0.01" name="amount" required></label>
                <label><span data-i18n="view.acct.label.business_pct">Business use %</span>
                    <input type="number" step="1" name="business_pct" value="100" min="1" max="100"></label>
                <label><span data-i18n="view.acct.label.description">Description</span>
                    <input type="text" name="description" required></label>
                <label><span data-i18n="view.acct.label.receipt_attached">Receipt attached?</span>
                    <input type="checkbox" name="receipt_attached"></label>
                <label><span data-i18n="view.acct.label.reimbursed">Reimbursed?</span>
                    <input type="checkbox" name="reimbursed"></label>
                <button class="primary" type="submit" data-i18n="view.acct.btn.add">Submit</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.acct.h2.requirements">Accountable plan requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.acct.req.business_connection">Business connection — expense incurred while performing services for employer</li>
                <li data-i18n="view.acct.req.substantiation">Substantiation — receipts + business purpose within reasonable time (~60 days)</li>
                <li data-i18n="view.acct.req.return_excess">Return excess advances — within 120 days of expense</li>
                <li data-i18n="view.acct.req.written_plan">WRITTEN plan adopted by corporate resolution before first reimbursement</li>
            </ol>
            <p class="muted small" data-i18n="view.acct.req.warning">
                Without ALL 4 elements: reimbursements become taxable W-2 wages. Worst of both
                worlds — owner pays SE/FICA on the reimbursement.
            </p>
        </div>
        <div id="ap-summary"></div>
        <div id="ap-table" class="chart-panel"></div>
    `;
    document.getElementById('ap-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const amt = Number(fd.get('amount'));
        const pct = Number(fd.get('business_pct')) / 100;
        const r = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            category: fd.get('category'),
            amount: amt,
            business_pct: pct,
            reimbursable: amt * pct,
            description: fd.get('description'),
            receipt_attached: !!fd.get('receipt_attached'),
            reimbursed: !!fd.get('reimbursed'),
        };
        state.reimbursements.push(r);
        save(state.reimbursements);
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0, 10);
        e.target.querySelector('[name="business_pct"]').value = 100;
        showToast(t('view.acct.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function render() {
    const year = new Date().getFullYear();
    const yearItems = state.reimbursements.filter(r => new Date(r.date).getFullYear() === year);
    renderSummary(yearItems, year);
    renderTable(yearItems);
}

function renderSummary(items, year) {
    const el = document.getElementById('ap-summary');
    if (!el) return;
    const total = items.reduce((s, r) => s + r.reimbursable, 0);
    const pending = items.filter(r => !r.reimbursed).reduce((s, r) => s + r.reimbursable, 0);
    const noReceipt = items.filter(r => !r.receipt_attached).length;
    const taxSavings = total * (state.marginal_rate + state.se_rate);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.acct.h2.summary">${year} accountable plan summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.acct.card.items">Items</div>
                    <div class="value">${items.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.acct.card.total">Total reimbursable</div>
                    <div class="value">$${total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.acct.card.pending">Pending reimbursement</div>
                    <div class="value">$${pending.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.acct.card.tax_savings">Combined tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${noReceipt > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.acct.card.no_receipt">Missing receipts</div>
                    <div class="value">${noReceipt}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(items) {
    const el = document.getElementById('ap-table');
    if (!el) return;
    if (!items.length) {
        el.innerHTML = `<h2 data-i18n="view.acct.h2.log">Log</h2>
            <p class="muted" data-i18n="view.acct.empty">No reimbursement items yet.</p>`;
        return;
    }
    const sorted = [...items].sort((a, b) => String(b.date).localeCompare(String(a.date)));
    el.innerHTML = `
        <h2 data-i18n="view.acct.h2.log">Log</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.acct.th.date">Date</th>
                <th data-i18n="view.acct.th.category">Category</th>
                <th data-i18n="view.acct.th.description">Description</th>
                <th data-i18n="view.acct.th.amount">Amount</th>
                <th data-i18n="view.acct.th.biz_pct">Biz %</th>
                <th data-i18n="view.acct.th.reimbursable">Reimbursable</th>
                <th data-i18n="view.acct.th.receipt">Receipt</th>
                <th data-i18n="view.acct.th.reimbursed">Reimbursed</th>
                <th data-i18n="view.acct.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(r => {
                const cat = EXPENSE_CATEGORIES.find(c => c.value === r.category) || EXPENSE_CATEGORIES[0];
                return `<tr>
                    <td>${esc(r.date)}</td>
                    <td class="muted">${esc(cat.label.split(' (')[0])}</td>
                    <td>${esc(r.description)}</td>
                    <td>$${r.amount.toFixed(2)}</td>
                    <td>${(r.business_pct * 100).toFixed(0)}%</td>
                    <td class="pos">$${r.reimbursable.toFixed(2)}</td>
                    <td class="${r.receipt_attached ? 'pos' : 'neg'}">${r.receipt_attached ? '✓' : '×'}</td>
                    <td class="${r.reimbursed ? 'pos' : ''}">${r.reimbursed ? '✓' : '—'}</td>
                    <td><button class="link neg" data-del="${esc(r.id)}" data-i18n="view.acct.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.reimbursements = state.reimbursements.filter(r => r.id !== btn.dataset.del);
            save(state.reimbursements);
            render();
        });
    });
}
