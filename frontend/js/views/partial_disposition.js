// Partial Disposition Election § 1.168(i)-8 — when a building component is replaced,
// elect to dispose of the OLD component, taking loss + ending depreciation.
// Replaces capitalize-and-keep-on-books treatment under old rules.
// Form 3115 required if missed in original year; Rev. Proc. 2015-13 auto change.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-partial-disp-v1';

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    dispositions: load(),
    marginal_rate: 0.32,
};

export async function renderPartialDisposition(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pd.h1.title">// PARTIAL DISPOSITION § 1.168(i)-8</span></h1>
        <p class="muted small" data-i18n="view.pd.hint.intro">
            When replacing a building component (roof, HVAC, windows, plumbing system) on
            commercial / rental property, ELECT to dispose of the OLD component on Form 4797.
            Take loss = remaining undepreciated basis. Stop future depreciation on the disposed
            component. <strong>Election made on return for year of disposition.</strong> Missed?
            File Form 3115 (Rev. Proc. 2015-13 auto change #205) — no IRS consent needed.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.pd.h2.add">Log partial disposition</h2>
            <form id="pd-form" class="inline-form">
                <label><span data-i18n="view.pd.label.year">Year of disposition</span>
                    <input type="number" step="1" name="year" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.pd.label.property">Property / building</span>
                    <input type="text" name="property" required></label>
                <label><span data-i18n="view.pd.label.component">Component disposed</span>
                    <select name="component">
                        <option value="roof">Roof</option>
                        <option value="hvac">HVAC system</option>
                        <option value="windows">Windows</option>
                        <option value="plumbing">Plumbing system</option>
                        <option value="electrical">Electrical system</option>
                        <option value="elevators">Elevators</option>
                        <option value="security">Security system</option>
                        <option value="fire">Fire protection</option>
                        <option value="other">Other</option>
                    </select>
                </label>
                <label><span data-i18n="view.pd.label.original_basis">Original component basis ($)</span>
                    <input type="number" step="100" name="original_basis" required></label>
                <label><span data-i18n="view.pd.label.accumulated_dep">Accumulated depreciation taken ($)</span>
                    <input type="number" step="100" name="accumulated_dep" required></label>
                <label><span data-i18n="view.pd.label.removal_cost">Removal / demolition cost ($)</span>
                    <input type="number" step="100" name="removal_cost" value="0"></label>
                <label><span data-i18n="view.pd.label.salvage">Salvage proceeds ($)</span>
                    <input type="number" step="100" name="salvage" value="0"></label>
                <button class="primary" type="submit" data-i18n="view.pd.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.pd.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" id="pd-marginal" value="${state.marginal_rate}"></label>
            </div>
        </div>
        <div id="pd-summary"></div>
        <div id="pd-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.pd.h2.why">Why elect partial disposition?</h2>
            <ul class="muted small">
                <li data-i18n="view.pd.why.loss">Recognize loss = unrecovered basis on Form 4797</li>
                <li data-i18n="view.pd.why.stop_dep">Stop depreciating phantom basis on a component no longer there</li>
                <li data-i18n="view.pd.why.tpr">Required by Tangible Property Regs to avoid double-depreciation</li>
                <li data-i18n="view.pd.why.no_recapture">No § 1250 recapture (component was fully residential / commercial real)</li>
                <li data-i18n="view.pd.why.3115_missed">Form 3115 for missed years: § 481(a) catch-up, negative income adj.</li>
            </ul>
        </div>
    `;
    document.getElementById('pd-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const d = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            property: fd.get('property'),
            component: fd.get('component'),
            original_basis: Number(fd.get('original_basis')) || 0,
            accumulated_dep: Number(fd.get('accumulated_dep')) || 0,
            removal_cost: Number(fd.get('removal_cost')) || 0,
            salvage: Number(fd.get('salvage')) || 0,
        };
        state.dispositions.push(d);
        save(state.dispositions);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = new Date().getFullYear();
        showToast(t('view.pd.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('pd-marginal').addEventListener('change', e => {
        state.marginal_rate = Number(e.target.value) || 0.32;
        render();
    });
    render();
}

function evaluate(d) {
    const remaining_basis = Math.max(0, d.original_basis - d.accumulated_dep);
    const total_loss = remaining_basis + d.removal_cost - d.salvage;
    const tax_savings = total_loss * state.marginal_rate;
    return { remaining_basis, total_loss, tax_savings };
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('pd-summary');
    if (!el) return;
    const evals = state.dispositions.map(d => ({ d, ev: evaluate(d) }));
    const totalLoss = evals.reduce((s, x) => s + x.ev.total_loss, 0);
    const totalSavings = evals.reduce((s, x) => s + x.ev.tax_savings, 0);
    const missedYears = state.dispositions.filter(d => d.year < new Date().getFullYear()).length;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pd.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.pd.card.count">Dispositions logged</div>
                    <div class="value">${state.dispositions.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.pd.card.total_loss">Total § 4797 loss</div>
                    <div class="value">$${totalLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.pd.card.tax_savings">Tax savings</div>
                    <div class="value">$${totalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${missedYears > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.pd.card.form_3115">Form 3115 (prior years)</div>
                        <div class="value">${missedYears}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('pd-table');
    if (!el) return;
    if (!state.dispositions.length) {
        el.innerHTML = `<h2 data-i18n="view.pd.h2.dispositions">Dispositions</h2>
            <p class="muted" data-i18n="view.pd.empty">No partial dispositions logged.</p>`;
        return;
    }
    const sorted = [...state.dispositions].sort((a, b) => b.year - a.year);
    el.innerHTML = `
        <h2 data-i18n="view.pd.h2.dispositions">Dispositions</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.pd.th.year">Year</th>
                <th data-i18n="view.pd.th.property">Property</th>
                <th data-i18n="view.pd.th.component">Component</th>
                <th data-i18n="view.pd.th.original_basis">Original basis</th>
                <th data-i18n="view.pd.th.accumulated_dep">Accum. dep.</th>
                <th data-i18n="view.pd.th.remaining">Remaining basis</th>
                <th data-i18n="view.pd.th.loss">Loss</th>
                <th data-i18n="view.pd.th.savings">Tax savings</th>
                <th data-i18n="view.pd.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(d => {
                const ev = evaluate(d);
                return `<tr>
                    <td>${d.year}</td>
                    <td>${esc(d.property)}</td>
                    <td class="muted">${esc(d.component)}</td>
                    <td class="muted">$${d.original_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">$${d.accumulated_dep.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${ev.remaining_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${ev.total_loss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="pos">$${ev.tax_savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(d.id)}" data-i18n="view.pd.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.dispositions = state.dispositions.filter(d => d.id !== btn.dataset.del);
            save(state.dispositions);
            render();
        });
    });
}
