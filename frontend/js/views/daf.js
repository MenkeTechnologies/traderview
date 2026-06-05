// Donor Advised Fund (DAF) — front-loaded charitable bunching tracker.
// Contribute appreciated stock → 30% AGI cap deduction at FMV. Avoid LTCG tax.
// Bunching: combine multiple years' giving into one tax year to itemize, then
// distribute over time. Major DAF sponsors: Fidelity Charitable, Schwab Charitable,
// Vanguard Charitable. Min: $5k Fidelity, $5k Schwab, $25k Vanguard.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-daf-v1';

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '{}'); }
    catch { return {}; }
}
function save(data) { try { localStorage.setItem(LS_KEY, JSON.stringify(data)); } catch { /* ignore */ } }

let state = Object.assign({
    contributions: [],
    grants: [],
    standard_deduction: 29_200,  // 2024 MFJ
    agi: 0,
    marginal_rate: 0.35,
    ltcg_rate: 0.20,
}, load());

function persist() { save(state); }

export async function renderDaf(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.daf.h1.title">// DAF — DONOR ADVISED FUND</span></h1>
        <p class="muted small" data-i18n="view.daf.hint.intro">
            Front-load charitable giving in high-income years. Contribute appreciated LT stock
            → <strong>FMV deduction</strong> (30% AGI cap) + <strong>avoid LTCG tax</strong>.
            DAF distributes to charity over time at your direction. <strong>Bunching strategy:</strong>
            stack 5 years of giving into one to exceed standard deduction, then itemize.
            Fidelity Charitable / Schwab Charitable / Vanguard Charitable: $5-25k min.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.daf.h2.add_contribution">Log DAF contribution</h2>
            <form id="daf-cform" class="inline-form">
                <label><span data-i18n="view.daf.label.date">Date</span>
                    <input type="date" name="date" value="${new Date().toISOString().slice(0,10)}" required></label>
                <label><span data-i18n="view.daf.label.asset_type">Asset type</span>
                    <select name="asset_type">
                        <option value="cash">Cash</option>
                        <option value="lt_stock">LT appreciated stock</option>
                        <option value="st_stock">ST stock</option>
                        <option value="real_estate">Real estate</option>
                        <option value="private">Private business interest</option>
                        <option value="crypto">Cryptocurrency</option>
                    </select>
                </label>
                <label><span data-i18n="view.daf.label.fmv">Fair market value ($)</span>
                    <input type="number" step="0.01" name="fmv" required></label>
                <label><span data-i18n="view.daf.label.basis">Your cost basis ($)</span>
                    <input type="number" step="0.01" name="basis"></label>
                <button class="primary" type="submit" data-i18n="view.daf.btn.add_contribution">Add contribution</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.daf.h2.add_grant">Log DAF grant out</h2>
            <form id="daf-gform" class="inline-form">
                <label><span data-i18n="view.daf.label.date">Date</span>
                    <input type="date" name="date" value="${new Date().toISOString().slice(0,10)}" required></label>
                <label><span data-i18n="view.daf.label.charity">Charity</span>
                    <input type="text" name="charity" placeholder="Doctors Without Borders" required></label>
                <label><span data-i18n="view.daf.label.amount">Amount ($)</span>
                    <input type="number" step="0.01" name="amount" required></label>
                <button class="primary" type="submit" data-i18n="view.daf.btn.add_grant">Add grant</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.daf.label.agi">AGI ($)</span>
                    <input type="number" step="0.01" id="daf-agi" value="${state.agi}"></label>
                <label><span data-i18n="view.daf.label.std_ded">Standard deduction ($)</span>
                    <input type="number" step="0.01" id="daf-std" value="${state.standard_deduction}"></label>
                <label><span data-i18n="view.daf.label.marginal">Marginal rate %</span>
                    <input type="number" step="0.01" id="daf-marginal" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.daf.label.ltcg">LTCG rate %</span>
                    <input type="number" step="0.01" id="daf-ltcg" value="${state.ltcg_rate}"></label>
            </div>
        </div>
        <div id="daf-summary"></div>
        <div id="daf-tables" class="chart-panel"></div>
    `;
    document.getElementById('daf-cform').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.contributions.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            asset_type: fd.get('asset_type'),
            fmv: Number(fd.get('fmv')) || 0,
            basis: Number(fd.get('basis')) || 0,
        });
        persist();
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0,10);
        showToast(t('view.daf.toast.contribution_added'), { level: 'success' });
        render();
    });
    document.getElementById('daf-gform').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.grants.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            charity: fd.get('charity'),
            amount: Number(fd.get('amount')) || 0,
        });
        persist();
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0,10);
        showToast(t('view.daf.toast.grant_added'), { level: 'success' });
        render();
    });
    document.getElementById('daf-agi').addEventListener('change', e => { state.agi = Number(e.target.value) || 0; persist(); render(); });
    document.getElementById('daf-std').addEventListener('change', e => { state.standard_deduction = Number(e.target.value) || 0; persist(); render(); });
    document.getElementById('daf-marginal').addEventListener('change', e => { state.marginal_rate = Number(e.target.value) || 0; persist(); render(); });
    document.getElementById('daf-ltcg').addEventListener('change', e => { state.ltcg_rate = Number(e.target.value) || 0; persist(); render(); });
    render();
}

function render() {
    renderSummary();
    renderTables();
}

function renderSummary() {
    const el = document.getElementById('daf-summary');
    if (!el) return;
    const totalContribFMV = state.contributions.reduce((s, c) => s + c.fmv, 0);
    const totalContribBasis = state.contributions.reduce((s, c) => s + c.basis, 0);
    const totalGrants = state.grants.reduce((s, g) => s + g.amount, 0);
    const balance = totalContribFMV - totalGrants;
    const cap = state.agi * 0.30;  // 30% AGI cap for FMV appreciated stock
    const usableY1 = Math.min(totalContribFMV, cap);
    const carryforward = Math.max(0, totalContribFMV - usableY1);
    const taxSavingsItemized = totalContribFMV * state.marginal_rate;
    const stdDedLost = state.standard_deduction;
    const incrementalSavings = Math.max(0, (totalContribFMV - stdDedLost)) * state.marginal_rate;
    const ltcgAvoided = state.contributions
        .filter(c => c.asset_type === 'lt_stock' || c.asset_type === 'real_estate' || c.asset_type === 'crypto')
        .reduce((s, c) => s + Math.max(0, c.fmv - c.basis) * state.ltcg_rate, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.daf.h2.summary">DAF summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.daf.card.contributions">Total contributions (FMV)</div>
                    <div class="value">$${totalContribFMV.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.daf.card.grants">Total granted out</div>
                    <div class="value">$${totalGrants.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.daf.card.balance">Remaining balance</div>
                    <div class="value">$${balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.daf.card.agi_cap">AGI cap (30%)</div>
                    <div class="value">$${cap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.daf.card.usable_y1">Usable year-1</div>
                    <div class="value">$${usableY1.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.daf.card.carryforward">Carryforward (5 yr)</div>
                    <div class="value">$${carryforward.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.daf.card.income_savings">Income tax savings</div>
                    <div class="value">$${taxSavingsItemized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.daf.card.incremental">Incremental (over standard)</div>
                    <div class="value">$${incrementalSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.daf.card.ltcg_avoided">LTCG tax avoided</div>
                    <div class="value">$${ltcgAvoided.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTables() {
    const el = document.getElementById('daf-tables');
    if (!el) return;
    el.innerHTML = `
        <h2 data-i18n="view.daf.h2.contributions">Contributions</h2>
        ${state.contributions.length === 0 ? `<p class="muted" data-i18n="view.daf.empty_contrib">No contributions logged.</p>` : `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.daf.th.date">Date</th>
                    <th data-i18n="view.daf.th.asset">Asset</th>
                    <th data-i18n="view.daf.th.fmv">FMV</th>
                    <th data-i18n="view.daf.th.basis">Basis</th>
                    <th data-i18n="view.daf.th.gain_avoided">LTCG avoided</th>
                    <th data-i18n="view.daf.th.actions">Actions</th>
                </tr></thead>
                <tbody>${[...state.contributions].sort((a, b) => (b.date || '').localeCompare(a.date || '')).map(c => `
                    <tr>
                        <td class="muted">${esc(c.date || '')}</td>
                        <td class="muted">${esc(c.asset_type)}</td>
                        <td>$${c.fmv.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="muted">$${c.basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="pos">$${Math.max(0, c.fmv - c.basis).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td><button class="link neg" data-cdel="${esc(c.id)}" data-i18n="view.daf.btn.delete">delete</button></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `}
        <h2 style="margin-top:16px" data-i18n="view.daf.h2.grants">Grants out</h2>
        ${state.grants.length === 0 ? `<p class="muted" data-i18n="view.daf.empty_grants">No grants logged.</p>` : `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.daf.th.date">Date</th>
                    <th data-i18n="view.daf.th.charity">Charity</th>
                    <th data-i18n="view.daf.th.amount">Amount</th>
                    <th data-i18n="view.daf.th.actions">Actions</th>
                </tr></thead>
                <tbody>${[...state.grants].sort((a, b) => (b.date || '').localeCompare(a.date || '')).map(g => `
                    <tr>
                        <td class="muted">${esc(g.date || '')}</td>
                        <td>${esc(g.charity)}</td>
                        <td>$${g.amount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td><button class="link neg" data-gdel="${esc(g.id)}" data-i18n="view.daf.btn.delete">delete</button></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `}
    `;
    el.querySelectorAll('[data-cdel]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.contributions = state.contributions.filter(c => c.id !== btn.dataset.cdel);
            persist(); render();
        });
    });
    el.querySelectorAll('[data-gdel]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.grants = state.grants.filter(g => g.id !== btn.dataset.gdel);
            persist(); render();
        });
    });
}
