// IRC § 691 — Income in Respect of a Decedent (IRD).
// Income earned but not received before death. Beneficiary reports as ordinary income
// when received. NO step-up in basis under § 1014(c). § 691(c) deduction: recipient
// gets income tax deduction for estate tax attributable to IRD (avoids double taxation).
// Common items: traditional IRA / 401(k), accrued interest, accounts receivable, NQ deferred comp.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-ird-v1';

function load() { try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); } catch { return []; } }
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    items: load(),
    estate_total: 0,
    estate_lifetime_used: 0,
    fed_estate_rate: 0.40,
    beneficiary_marginal_rate: 0.32,
};

export async function renderSection691(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s691.h1.title">// § 691 IRD INCOME IN RESPECT OF DECEDENT</span></h1>
        <p class="muted small" data-i18n="view.s691.hint.intro">
            Income earned but not received before death. Beneficiary reports as ORDINARY income
            when received. <strong>NO § 1014 step-up</strong> in basis. <strong>§ 691(c) deduction:</strong>
            recipient takes income tax deduction for estate tax attributable to IRD — prevents
            double taxation. Most common: <strong>traditional IRA / 401(k) balances</strong>, accrued
            interest, accounts receivable, NQ deferred comp, deferred IRC § 453 installment sale.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s691.h2.add">Log IRD item</h2>
            <form id="s691-form" class="inline-form">
                <label><span data-i18n="view.s691.label.description">Description</span>
                    <input type="text" name="description" placeholder="Decedent's Traditional IRA" required></label>
                <label><span data-i18n="view.s691.label.kind">Kind</span>
                    <select name="kind">
                        <option value="trad_ira">Traditional IRA</option>
                        <option value="401k">401(k) / 403(b)</option>
                        <option value="nq_deferred">NQ deferred comp</option>
                        <option value="accrued_interest">Accrued interest / bond OID</option>
                        <option value="accounts_receivable">Accounts receivable</option>
                        <option value="installment_sale">§ 453 installment sale gain</option>
                        <option value="stock_options">Vested but unexercised options</option>
                        <option value="rsu_vested">Vested RSUs not yet released</option>
                        <option value="annuity">Annuity payments</option>
                        <option value="renewal_comm">Insurance renewal commissions</option>
                    </select>
                </label>
                <label><span data-i18n="view.s691.label.value">Value ($)</span>
                    <input type="number" step="100" name="value" required></label>
                <label><span data-i18n="view.s691.label.basis">Decedent's basis ($)</span>
                    <input type="number" step="100" name="basis" value="0"></label>
                <label><span data-i18n="view.s691.label.beneficiary">Beneficiary</span>
                    <input type="text" name="beneficiary" placeholder="Surviving spouse / child"></label>
                <button class="primary" type="submit" data-i18n="view.s691.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.s691.label.estate_total">Total taxable estate ($)</span>
                    <input type="number" step="100000" id="s691-estate" value="${state.estate_total}"></label>
                <label><span data-i18n="view.s691.label.lifetime_used">Decedent lifetime used ($)</span>
                    <input type="number" step="100000" id="s691-lifetime" value="${state.estate_lifetime_used}"></label>
                <label><span data-i18n="view.s691.label.estate_rate">Estate marginal rate</span>
                    <input type="number" step="0.01" id="s691-est-rate" value="${state.fed_estate_rate}"></label>
                <label><span data-i18n="view.s691.label.beneficiary_rate">Beneficiary marginal rate</span>
                    <input type="number" step="0.01" id="s691-ben-rate" value="${state.beneficiary_marginal_rate}"></label>
            </div>
        </div>
        <div id="s691-summary"></div>
        <div id="s691-table" class="chart-panel"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s691.h2.planning">IRD planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s691.plan.roth_convert">Pre-death Roth conversion: removes IRD treatment, gets future-tax-free</li>
                <li data-i18n="view.s691.plan.charity">Name charity as IRA beneficiary — charity avoids income tax</li>
                <li data-i18n="view.s691.plan.crt">Charitable Remainder Trust as IRA beneficiary — stretches income</li>
                <li data-i18n="view.s691.plan.stretch_secure">SECURE Act 2020 killed lifetime stretch — 10-yr cap for non-spouse</li>
                <li data-i18n="view.s691.plan.qpb">"Eligible designated beneficiary" exceptions: spouse, minor child, disabled, &lt; 10 yr younger</li>
                <li data-i18n="view.s691.plan.691c">§ 691(c) deduction is a MISC itemized — suspended by TCJA through 2025?</li>
                <li data-i18n="view.s691.plan.nimcrut">NIMCRUT funded with IRA for stretch-like outcome</li>
            </ul>
        </div>
    `;
    document.getElementById('s691-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const i = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            description: fd.get('description'),
            kind: fd.get('kind'),
            value: Number(fd.get('value')) || 0,
            basis: Number(fd.get('basis')) || 0,
            beneficiary: fd.get('beneficiary') || '',
        };
        state.items.push(i);
        save(state.items);
        e.target.reset();
        showToast(t('view.s691.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('s691-estate').addEventListener('change', e => { state.estate_total = Number(e.target.value) || 0; render(); });
    document.getElementById('s691-lifetime').addEventListener('change', e => { state.estate_lifetime_used = Number(e.target.value) || 0; render(); });
    document.getElementById('s691-est-rate').addEventListener('change', e => { state.fed_estate_rate = Number(e.target.value) || 0; render(); });
    document.getElementById('s691-ben-rate').addEventListener('change', e => { state.beneficiary_marginal_rate = Number(e.target.value) || 0; render(); });
    render();
}

function render() {
    renderSummary();
    renderTable();
}

function renderSummary() {
    const el = document.getElementById('s691-summary');
    if (!el) return;
    const totalIrd = state.items.reduce((s, i) => s + i.value, 0);
    const ratioInEstate = state.estate_total > 0 ? totalIrd / state.estate_total : 0;
    const estateTaxOnIrd = totalIrd * state.fed_estate_rate * (state.estate_total > state.estate_lifetime_used ? 1 : 0);
    const incomeTaxBeforeDed = totalIrd * state.beneficiary_marginal_rate;
    const irc691cDeduction = estateTaxOnIrd;
    const incomeTaxAfterDed = Math.max(0, (totalIrd - irc691cDeduction) * state.beneficiary_marginal_rate);
    const totalCombinedTax = estateTaxOnIrd + incomeTaxAfterDed;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s691.h2.summary">IRD summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s691.card.count">Items</div>
                    <div class="value">${state.items.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s691.card.total">Total IRD</div>
                    <div class="value">$${totalIrd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s691.card.estate_ratio">% of estate</div>
                    <div class="value">${(ratioInEstate * 100).toFixed(0)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s691.card.estate_tax_on">Estate tax on IRD</div>
                    <div class="value">$${estateTaxOnIrd.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s691.card.income_tax_before">Income tax (before § 691(c))</div>
                    <div class="value">$${incomeTaxBeforeDed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s691.card.income_tax_after">Income tax (after § 691(c) ded)</div>
                    <div class="value">$${incomeTaxAfterDed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s691.card.combined">Combined estate + income</div>
                    <div class="value">$${totalCombinedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable() {
    const el = document.getElementById('s691-table');
    if (!el) return;
    if (!state.items.length) {
        el.innerHTML = `<h2 data-i18n="view.s691.h2.items">Items</h2>
            <p class="muted" data-i18n="view.s691.empty">No IRD items logged.</p>`;
        return;
    }
    const sorted = [...state.items].sort((a, b) => b.value - a.value);
    el.innerHTML = `
        <h2 data-i18n="view.s691.h2.items">Items</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.s691.th.description">Description</th>
                <th data-i18n="view.s691.th.kind">Kind</th>
                <th data-i18n="view.s691.th.value">Value</th>
                <th data-i18n="view.s691.th.basis">Basis</th>
                <th data-i18n="view.s691.th.beneficiary">Beneficiary</th>
                <th data-i18n="view.s691.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(i => `
                <tr>
                    <td>${esc(i.description)}</td>
                    <td class="muted">${esc(i.kind)}</td>
                    <td>$${i.value.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">$${i.basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">${esc(i.beneficiary || '—')}</td>
                    <td><button class="link neg" data-del="${esc(i.id)}" data-i18n="view.s691.btn.delete">delete</button></td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.items = state.items.filter(i => i.id !== btn.dataset.del);
            save(state.items);
            render();
        });
    });
}
