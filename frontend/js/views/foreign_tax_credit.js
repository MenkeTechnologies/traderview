// Foreign Tax Credit § 901 Tracker.
// Reduces US tax dollar-for-dollar by foreign income taxes paid (vs. deduction
// = subtract from income). Common scenarios: dividend WHT on foreign stocks
// (15% per US-most-treaties), capital gains tax on foreign brokerage.
// $300 single / $600 MFJ threshold = no Form 1116 needed (passive cat. only).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-ftc-v1';
const SIMPLIFIED_THRESHOLD_SINGLE = 300;
const SIMPLIFIED_THRESHOLD_MFJ = 600;

const CATEGORIES = [
    { value: 'passive',  label: 'Passive (most common — dividends, interest, royalties)' },
    { value: 'general',  label: 'General (active business / wages)' },
    { value: 'gilti',    label: 'GILTI' },
    { value: 'foreign_branch', label: 'Foreign branch' },
    { value: 'lump_sum_dist',  label: 'Lump-sum distribution' },
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    rows: load(),
    year: new Date().getFullYear(),
    filing: 'mfj',
};

export async function renderForeignTaxCredit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ftc.h1.title">// FOREIGN TAX CREDIT § 901</span></h1>
        <p class="muted small" data-i18n="view.ftc.hint.intro">
            Reduces US tax dollar-for-dollar by foreign income taxes paid. Common
            triggers: dividend WHT on foreign stocks (15% per US treaties),
            capital gains tax on IBKR-UK / Saxo holdings. Under $300 single / $600
            MFJ passive: no Form 1116 needed — claim directly on Schedule 3.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.ftc.h2.add">Add foreign income</h2>
            <form id="ftc-form" class="inline-form">
                <label><span data-i18n="view.ftc.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.ftc.label.country">Country</span>
                    <input type="text" name="country" placeholder="United Kingdom" required></label>
                <label><span data-i18n="view.ftc.label.income_type">Income type</span>
                    <input type="text" name="income_type" placeholder="ASML dividend / HSBC interest" required></label>
                <label><span data-i18n="view.ftc.label.category">Category</span>
                    <select name="category">${CATEGORIES.map(c =>
                        `<option value="${c.value}">${esc(c.label)}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.ftc.label.gross_income">Gross foreign income ($)</span>
                    <input type="number" step="0.01" name="gross_income" required></label>
                <label><span data-i18n="view.ftc.label.foreign_tax_paid">Foreign tax paid ($)</span>
                    <input type="number" step="0.01" name="foreign_tax_paid" required></label>
                <button class="primary" type="submit" data-i18n="view.ftc.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.ftc.label.filing">Filing status</span>
                    <select id="ftc-filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj"    ${state.filing === 'mfj'    ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.ftc.label.view_year">View year</span>
                    <input type="number" id="ftc-year" value="${state.year}"></label>
            </div>
        </div>
        <div id="ftc-summary"></div>
        <div id="ftc-table" class="chart-panel"></div>
    `;
    document.getElementById('ftc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const r = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            country: fd.get('country'),
            income_type: fd.get('income_type'),
            category: fd.get('category'),
            gross_income: Number(fd.get('gross_income')),
            foreign_tax_paid: Number(fd.get('foreign_tax_paid')),
        };
        state.rows.push(r);
        save(state.rows);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        showToast(t('view.ftc.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('ftc-filing').addEventListener('change', e => {
        state.filing = e.target.value;
        render();
    });
    document.getElementById('ftc-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    render();
}

function render() {
    const yearRows = state.rows.filter(r => r.year === state.year);
    renderSummary(yearRows);
    renderTable(yearRows);
}

function renderSummary(yearRows) {
    const el = document.getElementById('ftc-summary');
    if (!el) return;
    const totalIncome = yearRows.reduce((s, r) => s + r.gross_income, 0);
    const totalTaxPaid = yearRows.reduce((s, r) => s + r.foreign_tax_paid, 0);
    const passiveTaxPaid = yearRows.filter(r => r.category === 'passive')
        .reduce((s, r) => s + r.foreign_tax_paid, 0);
    const threshold = state.filing === 'mfj' ? SIMPLIFIED_THRESHOLD_MFJ : SIMPLIFIED_THRESHOLD_SINGLE;
    const needsForm1116 = totalTaxPaid > threshold || yearRows.some(r => r.category !== 'passive');
    const effectiveRate = totalIncome > 0 ? totalTaxPaid / totalIncome : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ftc.h2.summary">${state.year} foreign tax summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.ftc.card.foreign_income">Foreign income</div>
                    <div class="value">$${totalIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ftc.card.foreign_tax">Foreign tax paid</div>
                    <div class="value">$${totalTaxPaid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.ftc.card.credit">Available FTC</div>
                    <div class="value">$${totalTaxPaid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.ftc.card.effective_rate">Effective rate</div>
                    <div class="value">${(effectiveRate * 100).toFixed(1)}%</div>
                </div>
                <div class="card ${needsForm1116 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.ftc.card.form_1116">Form 1116 required?</div>
                    <div class="value">${needsForm1116 ? esc(t('view.ftc.status.yes')) : esc(t('view.ftc.status.no'))}</div>
                </div>
            </div>
            <p class="muted small" style="margin-top:10px" data-i18n="view.ftc.note">
                FTC limit = US tax × (foreign income / total income). Excess carries back 1 year + forward 10.
                For passive-cat income only with foreign tax ≤ threshold, skip Form 1116 — direct claim on Schedule 3.
            </p>
        </div>
    `;
}

function renderTable(yearRows) {
    const el = document.getElementById('ftc-table');
    if (!el) return;
    if (!yearRows.length) {
        el.innerHTML = `<h2 data-i18n="view.ftc.h2.entries">Entries</h2>
            <p class="muted" data-i18n="view.ftc.empty">No foreign tax entries for this year.</p>`;
        return;
    }
    const sorted = [...yearRows].sort((a, b) => b.foreign_tax_paid - a.foreign_tax_paid);
    el.innerHTML = `
        <h2 data-i18n="view.ftc.h2.entries">Entries</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.ftc.th.country">Country</th>
                <th data-i18n="view.ftc.th.income_type">Income type</th>
                <th data-i18n="view.ftc.th.category">Category</th>
                <th data-i18n="view.ftc.th.gross_income">Gross income</th>
                <th data-i18n="view.ftc.th.tax_paid">Tax paid</th>
                <th data-i18n="view.ftc.th.effective">WHT %</th>
                <th data-i18n="view.ftc.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(r => {
                const wht = r.gross_income > 0 ? (r.foreign_tax_paid / r.gross_income) : 0;
                return `<tr>
                    <td>${esc(r.country)}</td>
                    <td>${esc(r.income_type)}</td>
                    <td class="muted">${esc(r.category)}</td>
                    <td>$${r.gross_income.toFixed(2)}</td>
                    <td class="pos">$${r.foreign_tax_paid.toFixed(2)}</td>
                    <td>${(wht * 100).toFixed(1)}%</td>
                    <td><button class="link neg" data-del="${esc(r.id)}" data-i18n="view.ftc.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.rows = state.rows.filter(r => r.id !== btn.dataset.del);
            save(state.rows);
            render();
        });
    });
}
