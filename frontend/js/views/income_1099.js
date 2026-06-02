// 1099 Income Tracker — reconcile 1099-NEC, MISC, K, INT, DIV, B.
// Per-payer log + YTD totals + flag delivery dates + comparison vs your
// own bookkeeping. Match against incoming 1099s in February to spot
// over/under-reporting before filing.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-1099-records-v1';

const FORM_TYPES = [
    { value: 'NEC',  label: '1099-NEC (nonemployee comp)' },
    { value: 'MISC', label: '1099-MISC (other income)' },
    { value: 'K',    label: '1099-K (payment apps)' },
    { value: 'INT',  label: '1099-INT (interest)' },
    { value: 'DIV',  label: '1099-DIV (dividends)' },
    { value: 'B',    label: '1099-B (brokerage)' },
    { value: 'R',    label: '1099-R (retirement)' },
    { value: 'G',    label: '1099-G (govt payments)' },
    { value: 'SA',   label: '1099-SA (HSA distributions)' },
    { value: 'C',    label: '1099-C (cancelled debt)' },
];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(records) {
    try { localStorage.setItem(LS_KEY, JSON.stringify(records)); }
    catch { /* private mode */ }
}

let state = {
    records: load(),
    filterYear: new Date().getFullYear(),
};

export async function renderIncome1099(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.t1099.h1.title">// 1099 INCOME TRACKER</span></h1>
        <p class="muted small" data-i18n="view.t1099.hint.intro">
            Log every 1099 you receive (or expect to receive) by payer + form type.
            Match against your books in February to spot mismatches BEFORE filing —
            the IRS sees both copies, so unreported 1099 income triggers a CP2000.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.t1099.h2.add">Add 1099 record</h2>
            <form id="t1099-form" class="inline-form">
                <label><span data-i18n="view.t1099.label.tax_year">Tax year</span>
                    <input type="number" name="tax_year" value="${state.filterYear}" min="2010" required></label>
                <label><span data-i18n="view.t1099.label.form_type">Form type</span>
                    <select name="form_type">${FORM_TYPES.map(f =>
                        `<option value="${f.value}">${esc(f.label)}</option>`
                    ).join('')}</select>
                </label>
                <label><span data-i18n="view.t1099.label.payer">Payer (EIN / name)</span>
                    <input type="text" name="payer" placeholder="Webull / IBKR / Stripe" required></label>
                <label><span data-i18n="view.t1099.label.amount">Amount ($)</span>
                    <input type="number" step="0.01" name="amount" required></label>
                <label><span data-i18n="view.t1099.label.federal_withheld">Federal withheld ($)</span>
                    <input type="number" step="0.01" name="federal_withheld" value="0"></label>
                <label><span data-i18n="view.t1099.label.state_withheld">State withheld ($)</span>
                    <input type="number" step="0.01" name="state_withheld" value="0"></label>
                <label><span data-i18n="view.t1099.label.received">Received?</span>
                    <input type="checkbox" name="received"></label>
                <label><span data-i18n="view.t1099.label.matches_books">Matches books?</span>
                    <input type="checkbox" name="matches_books"></label>
                <label><span data-i18n="view.t1099.label.notes">Notes</span>
                    <input type="text" name="notes"></label>
                <button class="primary" type="submit" data-i18n="view.t1099.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.t1099.label.filter_year">View year</span>
                    <input type="number" id="t1099-filter" value="${state.filterYear}" min="2010"></label>
            </div>
        </div>
        <div id="t1099-summary"></div>
        <div id="t1099-table" class="chart-panel"></div>
    `;
    document.getElementById('t1099-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const rec = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            tax_year: Number(fd.get('tax_year')),
            form_type: fd.get('form_type'),
            payer: fd.get('payer'),
            amount: Number(fd.get('amount')),
            federal_withheld: Number(fd.get('federal_withheld')) || 0,
            state_withheld: Number(fd.get('state_withheld')) || 0,
            received: !!fd.get('received'),
            matches_books: !!fd.get('matches_books'),
            notes: fd.get('notes') || '',
            added_at: new Date().toISOString(),
        };
        state.records.push(rec);
        save(state.records);
        e.target.reset();
        e.target.querySelector('[name="tax_year"]').value = state.filterYear;
        e.target.querySelector('[name="federal_withheld"]').value = 0;
        e.target.querySelector('[name="state_withheld"]').value = 0;
        showToast(t('view.t1099.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('t1099-filter').addEventListener('change', e => {
        state.filterYear = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    render();
}

function render() {
    const yearRecs = state.records.filter(r => r.tax_year === state.filterYear);
    renderSummary(yearRecs);
    renderTable(yearRecs);
}

function renderSummary(yearRecs) {
    const el = document.getElementById('t1099-summary');
    if (!el) return;
    const totalIncome = yearRecs.reduce((s, r) => s + Number(r.amount || 0), 0);
    const totalFed = yearRecs.reduce((s, r) => s + Number(r.federal_withheld || 0), 0);
    const totalState = yearRecs.reduce((s, r) => s + Number(r.state_withheld || 0), 0);
    const notReceived = yearRecs.filter(r => !r.received).length;
    const mismatched = yearRecs.filter(r => r.received && !r.matches_books).length;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.t1099.h2.summary">${state.filterYear} summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.t1099.card.total_income">Total 1099 income</div>
                    <div class="value">$${totalIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.t1099.card.fed_withheld">Federal withheld</div>
                    <div class="value">$${totalFed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.t1099.card.state_withheld">State withheld</div>
                    <div class="value">$${totalState.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.t1099.card.records">Records</div>
                    <div class="value">${yearRecs.length}</div>
                </div>
                <div class="card ${notReceived ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.t1099.card.not_received">Not received</div>
                    <div class="value">${notReceived}</div>
                </div>
                <div class="card ${mismatched ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.t1099.card.mismatched">Mismatched vs books</div>
                    <div class="value">${mismatched}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(yearRecs) {
    const el = document.getElementById('t1099-table');
    if (!el) return;
    if (!yearRecs.length) {
        el.innerHTML = `<h2 data-i18n="view.t1099.h2.records">Records</h2>
            <p class="muted" data-i18n="view.t1099.empty">No 1099 records for this year yet.</p>`;
        return;
    }
    const sorted = [...yearRecs].sort((a, b) => (b.amount || 0) - (a.amount || 0));
    el.innerHTML = `
        <h2 data-i18n="view.t1099.h2.records">Records</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.t1099.th.form">Form</th>
                <th data-i18n="view.t1099.th.payer">Payer</th>
                <th data-i18n="view.t1099.th.amount">Amount</th>
                <th data-i18n="view.t1099.th.fed">Fed WH</th>
                <th data-i18n="view.t1099.th.state">State WH</th>
                <th data-i18n="view.t1099.th.received">Received</th>
                <th data-i18n="view.t1099.th.matched">Matched</th>
                <th data-i18n="view.t1099.th.notes">Notes</th>
                <th data-i18n="view.t1099.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(r => `
                <tr>
                    <td>1099-${esc(r.form_type)}</td>
                    <td>${esc(r.payer)}</td>
                    <td>$${Number(r.amount || 0).toLocaleString(undefined, { maximumFractionDigits: 2 })}</td>
                    <td>${r.federal_withheld ? '$' + Number(r.federal_withheld).toFixed(2) : '—'}</td>
                    <td>${r.state_withheld ? '$' + Number(r.state_withheld).toFixed(2) : '—'}</td>
                    <td class="${r.received ? 'pos' : 'neg'}">${r.received ? '✓' : '×'}</td>
                    <td class="${r.matches_books ? 'pos' : 'neg'}">${r.matches_books ? '✓' : '×'}</td>
                    <td class="muted">${esc(r.notes || '')}</td>
                    <td><button class="link neg" data-del="${esc(r.id)}" data-i18n="view.t1099.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.records = state.records.filter(r => r.id !== btn.dataset.del);
            save(state.records);
            render();
        });
    });
}
