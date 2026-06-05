// Net Operating Loss (NOL) Tracker — IRC § 172 post-TCJA.
// 2018+: NOLs carry forward indefinitely (no carryback for most taxpayers),
// limited to 80% of taxable income in any future year (TCJA limitation).
// CARES Act temporarily restored 5-year carryback for 2018-2020; that's
// expired. CURRENT RULE: only carryforward, 80% TI cap.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-nol-history-v1';
const TAXABLE_INCOME_LIMIT = 0.80;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    rows: load(),  // [{ year, taxable_income_before_nol, generated, used_this_year }]
};

export async function renderNolTracker(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.nol.h1.title">// NOL TRACKER</span></h1>
        <p class="muted small" data-i18n="view.nol.hint.intro">
            IRC § 172 post-TCJA: NOLs carry forward indefinitely, no carryback. Each
            year you can use NOLs to offset up to 80% of taxable income. Track yearly
            generation + utilization to see your remaining carryforward.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.nol.h2.add">Add year</h2>
            <form id="nol-form" class="inline-form">
                <label><span data-i18n="view.nol.label.year">Tax year</span>
                    <input type="number" step="1" name="year" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.nol.label.income_before">Taxable income BEFORE NOL ($)</span>
                    <input type="number" step="0.01" name="taxable_income_before_nol" required></label>
                <label><span data-i18n="view.nol.label.notes">Notes</span>
                    <input type="text" name="notes" placeholder="Trading loss year / catastrophic drawdown"></label>
                <button class="primary" type="submit" data-i18n="view.nol.btn.add">Add</button>
            </form>
        </div>
        <div id="nol-summary"></div>
        <div id="nol-table" class="chart-panel"></div>
    `;
    document.getElementById('nol-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const row = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            taxable_income_before_nol: Number(fd.get('taxable_income_before_nol')),
            notes: fd.get('notes') || '',
        };
        state.rows.push(row);
        save(state.rows);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = new Date().getFullYear();
        showToast(t('view.nol.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function computeYears() {
    const sorted = [...state.rows].sort((a, b) => a.year - b.year);
    let carryforward = 0;
    const result = [];
    for (const r of sorted) {
        const ti = r.taxable_income_before_nol;
        let generated = 0;
        let used = 0;
        let endCarry = carryforward;
        if (ti < 0) {
            generated = -ti;
            endCarry = carryforward + generated;
        } else if (ti > 0 && carryforward > 0) {
            const cap = ti * TAXABLE_INCOME_LIMIT;
            used = Math.min(carryforward, cap);
            endCarry = carryforward - used;
        }
        const taxableAfter = Math.max(0, ti - used);
        result.push({
            ...r,
            generated, used, taxable_after_nol: taxableAfter,
            carryforward_end: endCarry,
        });
        carryforward = endCarry;
    }
    return { rows: result, carryforward_end: carryforward };
}

function render() {
    const { rows, carryforward_end } = computeYears();
    renderSummary(carryforward_end, rows);
    renderTable(rows);
}

function renderSummary(carry, rows) {
    const el = document.getElementById('nol-summary');
    if (!el) return;
    const totalGenerated = rows.reduce((s, r) => s + r.generated, 0);
    const totalUsed = rows.reduce((s, r) => s + r.used, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.nol.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card ${carry > 0 ? 'pos' : ''}">
                    <div class="label" data-i18n="view.nol.card.carryforward">Carryforward available</div>
                    <div class="value">$${carry.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.nol.card.total_generated">Lifetime NOLs generated</div>
                    <div class="value">$${totalGenerated.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.nol.card.total_used">Lifetime NOLs used</div>
                    <div class="value">$${totalUsed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.nol.card.years_tracked">Years tracked</div>
                    <div class="value">${rows.length}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(rows) {
    const el = document.getElementById('nol-table');
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<h2 data-i18n="view.nol.h2.history">History</h2>
            <p class="muted" data-i18n="view.nol.empty">No years tracked yet.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.nol.h2.history">History</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.nol.th.year">Year</th>
                <th data-i18n="view.nol.th.income_before">TI before NOL</th>
                <th data-i18n="view.nol.th.generated">NOL generated</th>
                <th data-i18n="view.nol.th.used">NOL used</th>
                <th data-i18n="view.nol.th.income_after">TI after NOL</th>
                <th data-i18n="view.nol.th.carryforward">Carryforward end</th>
                <th data-i18n="view.nol.th.notes">Notes</th>
                <th data-i18n="view.nol.th.actions">Actions</th>
            </tr></thead>
            <tbody>${rows.map(r => `
                <tr>
                    <td><strong>${r.year}</strong></td>
                    <td>$${r.taxable_income_before_nol.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${r.generated > 0 ? 'neg' : ''}">${r.generated > 0 ? '$' + r.generated.toLocaleString(undefined, { maximumFractionDigits: 0 }) : '—'}</td>
                    <td class="${r.used > 0 ? 'pos' : ''}">${r.used > 0 ? '$' + r.used.toLocaleString(undefined, { maximumFractionDigits: 0 }) : '—'}</td>
                    <td>$${r.taxable_after_nol.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>${r.carryforward_end > 0 ? '$' + r.carryforward_end.toLocaleString(undefined, { maximumFractionDigits: 0 }) : '—'}</td>
                    <td class="muted">${esc(r.notes || '')}</td>
                    <td><button class="link neg" data-del="${esc(r.id)}" data-i18n="view.nol.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:10px" data-i18n="view.nol.note">
            Note: TCJA 80% taxable-income cap means even a large NOL can't fully offset
            a profitable year. Plan trade timing accordingly — a Section 475(f)
            mark-to-market election would convert this trading loss to ordinary,
            which generates QBI-eligible NOLs.
        </p>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.rows = state.rows.filter(r => r.id !== btn.dataset.del);
            save(state.rows);
            render();
        });
    });
}
