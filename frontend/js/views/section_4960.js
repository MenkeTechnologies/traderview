// IRC § 4960 — Excise Tax on Tax-Exempt Excess Comp + Parachute.
// 21% excise on tax-exempt employer for: (a) Annual comp > $1M to covered employee, OR
// (b) "Excess parachute payment" (≥ 3× base × disqualified) similar to § 280G.
// Covered employee = top-5 highest-paid for any year since 2017 (sticky list).
// Form 4720. Universities, hospitals, foundations, large nonprofits at risk.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EXCISE_RATE = 0.21;
const COMP_CAP = 1_000_000;

let state = {
    employee_count: 0,
    employees: [],
    medical_dental_excluded: 0,
};

export async function renderSection4960(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4960.h1.title">// § 4960 TAX-EXEMPT $1M COMP</span></h1>
        <p class="muted small" data-i18n="view.s4960.hint.intro">
            <strong>21% excise</strong> on tax-exempt EMPLOYER for: (a) Annual comp &gt; $1M to
            covered employee, OR (b) "Excess parachute payment" (≥ 3× base × disqualified) similar
            to § 280G but on tax-exempt side. <strong>Covered employee = top-5 highest-paid</strong>
            for any year since 2017 (sticky list — once covered, always covered). Form 4720.
            Universities, hospitals, foundations, large nonprofits primary targets.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4960.h2.add">Log covered employee</h2>
            <form id="s4960-form" class="inline-form">
                <label><span data-i18n="view.s4960.label.name">Name / role</span>
                    <input type="text" name="name" required></label>
                <label><span data-i18n="view.s4960.label.kind">Kind</span>
                    <select name="kind">
                        <option value="annual_comp">Annual comp (a)</option>
                        <option value="parachute">Excess parachute (b)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4960.label.total_comp">Total annual comp ($)</span>
                    <input type="number" step="10000" name="total_comp" required></label>
                <label><span data-i18n="view.s4960.label.medical_dental">Medical / dental excluded ($)</span>
                    <input type="number" step="100" name="medical_dental" value="0"></label>
                <label><span data-i18n="view.s4960.label.base_5yr">5-yr W-2 base (for parachute test) ($)</span>
                    <input type="number" step="10000" name="base_5yr"></label>
                <label><span data-i18n="view.s4960.label.parachute_total">Parachute payment total ($)</span>
                    <input type="number" step="10000" name="parachute_total"></label>
                <button class="primary" type="submit" data-i18n="view.s4960.btn.add">Add</button>
            </form>
        </div>
        <div id="s4960-summary"></div>
        <div id="s4960-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4960.h2.covered_employees">Covered employee determination</h2>
            <ul class="muted small">
                <li data-i18n="view.s4960.cov.top_5">Top-5 highest-paid for tax year</li>
                <li data-i18n="view.s4960.cov.sticky">Once covered, ALWAYS covered (sticky list)</li>
                <li data-i18n="view.s4960.cov.related_orgs">Related tax-exempt orgs aggregated for determination</li>
                <li data-i18n="view.s4960.cov.licensed_pros">Medical / veterinary licensed professionals: medical-services portion EXCLUDED</li>
                <li data-i18n="view.s4960.cov.shareholders">Substantial shareholder of related taxable corp counted</li>
                <li data-i18n="view.s4960.cov.notice_2019">Notice 2019-9 + Notice 2021-78 + final regs 2021 for guidance</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4960.h2.exclusions">Comp NOT counted (medical exclusion)</h2>
            <ul class="muted small">
                <li data-i18n="view.s4960.excl.medical">Direct medical / veterinary services (only)</li>
                <li data-i18n="view.s4960.excl.admin">NOT administrative / teaching / research at hospital</li>
                <li data-i18n="view.s4960.excl.allocation">Allocate dual-purpose comp pro-rata across medical vs other</li>
                <li data-i18n="view.s4960.excl.qpr">"Qualified pension plan" benefits not counted (excludes 401(a), 403(b), 457(b))</li>
                <li data-i18n="view.s4960.excl.welfare">Welfare benefit plans excluded</li>
                <li data-i18n="view.s4960.excl.reimbursed">Reimbursed business expenses excluded</li>
            </ul>
        </div>
    `;
    document.getElementById('s4960-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.employees.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            name: fd.get('name'),
            kind: fd.get('kind'),
            total_comp: Number(fd.get('total_comp')) || 0,
            medical_dental: Number(fd.get('medical_dental')) || 0,
            base_5yr: Number(fd.get('base_5yr')) || 0,
            parachute_total: Number(fd.get('parachute_total')) || 0,
        });
        e.target.reset();
        render();
    });
    render();
}

function exciseFor(e) {
    if (e.kind === 'annual_comp') {
        const adjustedComp = e.total_comp - e.medical_dental;
        return Math.max(0, adjustedComp - COMP_CAP) * EXCISE_RATE;
    } else {
        const safeHarbor = e.base_5yr * 3;
        if (e.parachute_total < safeHarbor) return 0;
        const excess = e.parachute_total - e.base_5yr;
        return excess * EXCISE_RATE;
    }
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s4960-summary');
    if (!el) return;
    const totalComp = state.employees.reduce((s, e) => s + e.total_comp, 0);
    const totalExcise = state.employees.reduce((s, e) => s + exciseFor(e), 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4960.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4960.card.count">Covered employees</div>
                    <div class="value">${state.employees.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4960.card.total">Total comp paid</div>
                    <div class="value">$${totalComp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4960.card.excise">Total § 4960 excise</div>
                    <div class="value">$${totalExcise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s4960-table');
    if (!el) return;
    if (!state.employees.length) {
        el.innerHTML = `<h2 data-i18n="view.s4960.h2.employees">Covered employees</h2>
            <p class="muted" data-i18n="view.s4960.empty">No employees logged.</p>`;
        return;
    }
    const sorted = [...state.employees].sort((a, b) => exciseFor(b) - exciseFor(a));
    el.innerHTML = `
        <h2 data-i18n="view.s4960.h2.employees">Covered employees</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s4960.th.name">Name</th>
                <th data-i18n="view.s4960.th.kind">Kind</th>
                <th data-i18n="view.s4960.th.total">Comp</th>
                <th data-i18n="view.s4960.th.excise">Excise</th>
                <th data-i18n="view.s4960.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(e => `
                <tr>
                    <td>${esc(e.name)}</td>
                    <td class="muted">${esc(e.kind)}</td>
                    <td>$${e.total_comp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="neg">$${exciseFor(e).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(e.id)}" data-i18n="view.s4960.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.employees = state.employees.filter(emp => emp.id !== btn.dataset.del);
            render();
        });
    });
}
