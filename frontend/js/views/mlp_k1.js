// MLP / Partnership K-1 Tracker — UBTI in IRA alert + § 469 PAL grouping.
// Master Limited Partnerships (EPD, MPLX, ET, MMP, etc.) pass K-1s, NOT 1099-DIVs.
// In an IRA: > $1000 UBTI triggers Form 990-T filing + tax inside the IRA.
// Outside IRA: § 469 passive activity loss limits + recapture at sale.
// Holders must track: cumulative distributions (return of capital reduces basis),
// suspended losses (carry until disposition), and UBTI/UBIT for tax-deferred accounts.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-mlp-k1-v1';
const UBTI_IRA_THRESHOLD = 1_000;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    holdings: load(),
    year: new Date().getFullYear(),
};

export async function renderMlpK1(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mlp.h1.title">// MLP / K-1 TRACKER</span></h1>
        <p class="muted small" data-i18n="view.mlp.hint.intro">
            MLPs (EPD, MPLX, ET, MMP) issue K-1s, NOT 1099-DIVs. K-1s ship March—April,
            delaying tax returns. In an <strong>IRA</strong>: $1,000 aggregate UBTI triggers
            <strong>Form 990-T</strong> and tax PAID FROM THE IRA. Outside IRA: § 469 passive
            losses suspend until disposition. Distributions are mostly return-of-capital, reducing basis.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.mlp.h2.add">Log K-1 holding</h2>
            <form id="mlp-form" class="inline-form">
                <label><span data-i18n="view.mlp.label.year">Tax year</span>
                    <input type="number" step="1" name="year" value="${state.year}" required></label>
                <label><span data-i18n="view.mlp.label.symbol">Symbol</span>
                    <input type="text" name="symbol" placeholder="EPD" required></label>
                <label><span data-i18n="view.mlp.label.account_type">Account type</span>
                    <select name="account_type">
                        <option value="taxable">Taxable brokerage</option>
                        <option value="ira">Traditional IRA</option>
                        <option value="roth">Roth IRA</option>
                        <option value="hsa">HSA</option>
                        <option value="401k">401(k)</option>
                    </select>
                </label>
                <label><span data-i18n="view.mlp.label.shares">Shares</span>
                    <input type="number" step="0.001" name="shares" required></label>
                <label><span data-i18n="view.mlp.label.cost_basis">Cost basis ($)</span>
                    <input type="number" step="0.01" name="cost_basis" required></label>
                <label><span data-i18n="view.mlp.label.distributions">Cash distributions ($)</span>
                    <input type="number" step="0.01" name="distributions" required></label>
                <label><span data-i18n="view.mlp.label.ubti">UBTI from K-1 box 20V ($)</span>
                    <input type="number" step="0.01" name="ubti"></label>
                <label><span data-i18n="view.mlp.label.ordinary_income">Box 1 ordinary income ($)</span>
                    <input type="number" step="0.01" name="ordinary_income"></label>
                <label><span data-i18n="view.mlp.label.section_199a">Box 20Z § 199A income ($)</span>
                    <input type="number" step="0.01" name="section_199a"></label>
                <label><span data-i18n="view.mlp.label.passive_loss">Box 1 / 2 passive loss ($)</span>
                    <input type="number" step="0.01" name="passive_loss"></label>
                <button class="primary" type="submit" data-i18n="view.mlp.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.mlp.label.view_year">View year</span>
                    <input type="number" id="mlp-view-year" value="${state.year}" min="2010"></label>
            </div>
        </div>
        <div id="mlp-summary"></div>
        <div id="mlp-table" class="chart-panel"></div>
    `;
    document.getElementById('mlp-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const h = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            year: Number(fd.get('year')),
            symbol: String(fd.get('symbol') || '').trim().toUpperCase(),
            account_type: fd.get('account_type'),
            shares: Number(fd.get('shares')) || 0,
            cost_basis: Number(fd.get('cost_basis')) || 0,
            distributions: Number(fd.get('distributions')) || 0,
            ubti: Number(fd.get('ubti')) || 0,
            ordinary_income: Number(fd.get('ordinary_income')) || 0,
            section_199a: Number(fd.get('section_199a')) || 0,
            passive_loss: Number(fd.get('passive_loss')) || 0,
        };
        state.holdings.push(h);
        save(state.holdings);
        e.target.reset();
        e.target.querySelector('[name="year"]').value = state.year;
        showToast(t('view.mlp.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('mlp-view-year').addEventListener('change', e => {
        state.year = Number(e.target.value) || new Date().getFullYear();
        render();
    });
    render();
}

function render() {
    const yearHoldings = state.holdings.filter(h => h.year === state.year);
    renderSummary(yearHoldings);
    renderTable(yearHoldings);
}

function renderSummary(yearHoldings) {
    const el = document.getElementById('mlp-summary');
    if (!el) return;
    const iraHoldings = yearHoldings.filter(h => ['ira', 'roth', 'hsa', '401k'].includes(h.account_type));
    const taxableHoldings = yearHoldings.filter(h => h.account_type === 'taxable');
    const iraUbtiTotal = iraHoldings.reduce((s, h) => s + h.ubti, 0);
    const form990tRequired = iraUbtiTotal >= UBTI_IRA_THRESHOLD;
    const totalDistributions = yearHoldings.reduce((s, h) => s + h.distributions, 0);
    const totalOrdinary = yearHoldings.reduce((s, h) => s + h.ordinary_income, 0);
    const total199a = yearHoldings.reduce((s, h) => s + h.section_199a, 0);
    const totalPassiveLosses = yearHoldings.reduce((s, h) => s + h.passive_loss, 0);
    const qbiDeduction = Math.max(0, total199a * 0.20);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.mlp.h2.summary">${state.year} K-1 summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.mlp.card.holdings">Holdings</div>
                    <div class="value">${yearHoldings.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.mlp.card.taxable_holdings">Taxable holdings</div>
                    <div class="value">${taxableHoldings.length}</div>
                </div>
                <div class="card ${form990tRequired ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.mlp.card.ira_holdings">IRA holdings (UBTI risk)</div>
                    <div class="value">${iraHoldings.length}</div>
                </div>
                <div class="card ${form990tRequired ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.mlp.card.ira_ubti">Total IRA UBTI</div>
                    <div class="value">$${iraUbtiTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${form990tRequired ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.mlp.card.form_990t">Form 990-T required</div>
                    <div class="value">${form990tRequired ? esc(t('view.mlp.status.yes')) : esc(t('view.mlp.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.mlp.card.distributions">Total cash distributions</div>
                    <div class="value">$${totalDistributions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.mlp.card.ordinary_income">Box 1 ordinary income</div>
                    <div class="value">$${totalOrdinary.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.mlp.card.qbi">§ 199A QBI deduction (20%)</div>
                    <div class="value">$${qbiDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalPassiveLosses > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.mlp.card.suspended_losses">Suspended passive losses</div>
                    <div class="value">$${totalPassiveLosses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${form990tRequired ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.mlp.warning.ubti">
                    UBTI ≥ $1,000 in your IRA. Custodian (Fidelity, Schwab, etc.) files Form 990-T on
                    behalf of the IRA. The TAX is paid FROM the IRA. Many investors avoid MLPs in IRAs
                    entirely; consider switching to MLP ETFs (AMLP, MLPX) which pay corporate tax internally.
                </p>
            ` : ''}
        </div>
    `;
}

function renderTable(yearHoldings) {
    const el = document.getElementById('mlp-table');
    if (!el) return;
    if (!yearHoldings.length) {
        el.innerHTML = `<h2 data-i18n="view.mlp.h2.holdings">Holdings</h2>
            <p class="muted" data-i18n="view.mlp.empty">No K-1 holdings recorded for this year.</p>`;
        return;
    }
    const sorted = [...yearHoldings].sort((a, b) => Math.abs(b.ubti) - Math.abs(a.ubti));
    el.innerHTML = `
        <h2 data-i18n="view.mlp.h2.holdings">Holdings</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.mlp.th.symbol">Symbol</th>
                <th data-i18n="view.mlp.th.account">Account</th>
                <th data-i18n="view.mlp.th.shares">Shares</th>
                <th data-i18n="view.mlp.th.cost_basis">Cost basis</th>
                <th data-i18n="view.mlp.th.distributions">Distributions</th>
                <th data-i18n="view.mlp.th.ubti">UBTI</th>
                <th data-i18n="view.mlp.th.ordinary">Ord income</th>
                <th data-i18n="view.mlp.th.199a">§ 199A</th>
                <th data-i18n="view.mlp.th.passive_loss">Pass loss</th>
                <th data-i18n="view.mlp.th.actions">Actions</th>
            </tr></thead>
            <tbody>${sorted.map(h => {
                const isIra = ['ira', 'roth', 'hsa', '401k'].includes(h.account_type);
                const ubtiCls = isIra && h.ubti >= UBTI_IRA_THRESHOLD ? 'neg' : '';
                return `<tr>
                    <td>${esc(h.symbol)}</td>
                    <td class="${isIra ? 'neg' : 'muted'}">${esc(h.account_type)}</td>
                    <td>${h.shares.toFixed(2)}</td>
                    <td class="muted">$${h.cost_basis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td>$${h.distributions.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${ubtiCls}">$${h.ubti.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">$${h.ordinary_income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="muted">$${h.section_199a.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td class="${h.passive_loss > 0 ? 'neg' : 'muted'}">$${h.passive_loss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    <td><button class="link neg" data-del="${esc(h.id)}" data-i18n="view.mlp.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.holdings = state.holdings.filter(h => h.id !== btn.dataset.del);
            save(state.holdings);
            render();
        });
    });
}
