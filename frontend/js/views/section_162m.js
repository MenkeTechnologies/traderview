// IRC § 162(m) — $1M Executive Compensation Deduction Cap.
// Public C-corps disallowed deduction for comp > $1M paid to "covered employees":
// CEO, CFO, three highest-paid officers, plus any covered employee since 2017
// (sticky list — once covered, always covered). TCJA 2017 eliminated performance-based
// exception. ARPA 2021 expanded to top 5 highest-paid (not just executives) from 2027.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-162m-v1';
const CAP = 1_000_000;
const CORP_RATE = 0.21;

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    employees: load(),
    corp_tax_rate: CORP_RATE,
};

export async function renderSection162m(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s162m.h1.title">// § 162(m) $1M EXEC COMP CAP</span></h1>
        <p class="muted small" data-i18n="view.s162m.hint.intro">
            Public C-corps disallowed deduction for comp &gt; $1M paid to <strong>covered employees</strong>:
            CEO, CFO, three highest-paid officers + any covered employee since 2017 (sticky list —
            <strong>once covered, always covered</strong>). TCJA 2017 eliminated performance-based
            exception (former CEO-perf-bonus loophole). <strong>ARPA 2021 expanded</strong> to top 5
            highest-paid (not just executives) starting 2027.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s162m.h2.add">Log covered employee</h2>
            <form id="s162m-form" class="inline-form">
                <label><span data-i18n="view.s162m.label.name">Name / role</span>
                    <input type="text" name="name" placeholder="CEO Jane Doe" required></label>
                <label><span data-i18n="view.s162m.label.title">Title</span>
                    <select name="title">
                        <option value="ceo">CEO</option>
                        <option value="cfo">CFO</option>
                        <option value="ntop3">Named top-3 executive</option>
                        <option value="legacy">Legacy covered (TCJA sticky)</option>
                        <option value="top5">Top-5 (post-2027 ARPA)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162m.label.total_comp">Total comp ($)</span>
                    <input type="number" step="0.01" name="total_comp" required></label>
                <label><span data-i18n="view.s162m.label.cash">Cash + bonus ($)</span>
                    <input type="number" step="0.01" name="cash_bonus"></label>
                <label><span data-i18n="view.s162m.label.equity">Equity (vested) ($)</span>
                    <input type="number" step="0.01" name="equity"></label>
                <label><span data-i18n="view.s162m.label.deferred">Deferred comp recognized ($)</span>
                    <input type="number" step="0.01" name="deferred_recognized"></label>
                <button class="primary" type="submit" data-i18n="view.s162m.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.s162m.label.corp_rate">Corp tax rate</span>
                    <input type="number" step="0.01" id="s162m-rate" value="${state.corp_tax_rate}"></label>
            </div>
        </div>
        <div id="s162m-summary"></div>
        <div id="s162m-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162m.h2.planning">Planning techniques</h2>
            <ul class="muted small">
                <li data-i18n="view.s162m.plan.private">Private companies NOT subject to § 162(m)</li>
                <li data-i18n="view.s162m.plan.partnership">Partnership / LLC NOT subject</li>
                <li data-i18n="view.s162m.plan.deferred">Defer comp until post-separation year if person no longer in scope</li>
                <li data-i18n="view.s162m.plan.foreign_subs">Foreign sub of public US co: § 162(m) applies if subject to Securities Act reporting</li>
                <li data-i18n="view.s162m.plan.section_280g">Coordinate with § 280G golden parachute disallowance + 20% excise</li>
                <li data-i18n="view.s162m.plan.gross_up">"Gross-up" payments to executives bear § 162(m) limit</li>
                <li data-i18n="view.s162m.plan.q1">SEC Form Summary Comp Table is § 162(m) starting point</li>
                <li data-i18n="view.s162m.plan.ipo">IPO transition relief: 3 yr post-IPO has reduced rules</li>
            </ul>
        </div>
    `;
    document.getElementById('s162m-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.employees.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            name: fd.get('name'),
            title: fd.get('title'),
            total_comp: Number(fd.get('total_comp')) || 0,
            cash_bonus: Number(fd.get('cash_bonus')) || 0,
            equity: Number(fd.get('equity')) || 0,
            deferred_recognized: Number(fd.get('deferred_recognized')) || 0,
        });
        save(state.employees);
        e.target.reset();
        showToast(t('view.s162m.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s162m-rate').addEventListener('change', e => {
        state.corp_tax_rate = Number(e.target.value) || CORP_RATE;
        render();
    });
    render();
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s162m-summary');
    if (!el) return;
    const totalComp = state.employees.reduce((s, e) => s + e.total_comp, 0);
    const totalDisallowed = state.employees.reduce((s, e) => s + Math.max(0, e.total_comp - CAP), 0);
    const lostDeduction = totalDisallowed * state.corp_tax_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s162m.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s162m.card.count">Covered employees</div>
                    <div class="value">${state.employees.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s162m.card.total_comp">Total comp paid</div>
                    <div class="value">$${totalComp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162m.card.disallowed">Disallowed amount</div>
                    <div class="value">$${totalDisallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162m.card.lost">Lost deduction value</div>
                    <div class="value">$${lostDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s162m-table');
    if (!el) return;
    if (!state.employees.length) {
        el.innerHTML = `<h2 data-i18n="view.s162m.h2.employees">Covered employees</h2>
            <p class="muted" data-i18n="view.s162m.empty">No covered employees logged.</p>`;
        return;
    }
    const sorted = [...state.employees].sort((a, b) => b.total_comp - a.total_comp);
    el.innerHTML = `
        <h2 data-i18n="view.s162m.h2.employees">Covered employees</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s162m.th.name">Name</th>
                <th data-i18n="view.s162m.th.title">Title</th>
                <th data-i18n="view.s162m.th.total">Total comp</th>
                <th data-i18n="view.s162m.th.cap">Cap</th>
                <th data-i18n="view.s162m.th.disallowed">Disallowed</th>
                <th data-i18n="view.s162m.th.deductible">Deductible</th>
                <th data-i18n="view.s162m.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(e => {
                const disallowed = Math.max(0, e.total_comp - CAP);
                const deductible = Math.min(e.total_comp, CAP);
                return `<tr>
                    <td>${esc(e.name)}</td>
                    <td class="muted">${esc(e.title)}</td>
                    <td>$${e.total_comp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">$${CAP.toLocaleString()}</td>
                    <td class="${disallowed > 0 ? 'neg' : 'pos'}">$${disallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${deductible.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(e.id)}" data-i18n="view.s162m.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.employees = state.employees.filter(emp => emp.id !== btn.dataset.del);
            save(state.employees);
            render();
        });
    });
}
