// Low Income Housing Credit § 42 (LIHTC) Tracker.
// Federal tax credit for affordable housing investors. 10-year stream.
// 9% credit: new construction / substantial rehab, competitive allocation.
// 4% credit: acquisition / bond-financed deals, non-competitive.
// 15-year compliance period + 15-year extended-use period = 30-year total.
// Subject to passive activity rules unless real-estate professional.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-lihtc-v1';
const CREDIT_PERIOD = 10;
const COMPLIANCE_PERIOD = 15;
const EXTENDED_USE_PERIOD = 15;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = {
    investments: load(),
    your_marginal_rate: 0.37,
};

export async function renderLihtc(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lihtc.h1.title">// LIHTC § 42 TRACKER</span></h1>
        <p class="muted small" data-i18n="view.lihtc.hint.intro">
            Federal credit for affordable housing investors. 10-year stream of credits.
            <strong>9% credit:</strong> new construction / substantial rehab (competitive
            state allocation). <strong>4% credit:</strong> acquisition / bond-financed
            (non-competitive). 15-year compliance + 15-year extended-use. Subject to
            passive activity rules unless REP. Sold by syndicators to corporate / passive
            investors at 80-95¢ on the dollar.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.lihtc.h2.add">Log investment</h2>
            <form id="lihtc-form" class="inline-form">
                <label><span data-i18n="view.lihtc.label.project_name">Project / property name</span>
                    <input type="text" name="project_name" required></label>
                <label><span data-i18n="view.lihtc.label.investment">Your investment ($)</span>
                    <input type="number" step="1000" name="investment" required></label>
                <label><span data-i18n="view.lihtc.label.annual_credit">Annual credit ($)</span>
                    <input type="number" step="100" name="annual_credit" required></label>
                <label><span data-i18n="view.lihtc.label.credit_type">Credit type</span>
                    <select name="credit_type">
                        <option value="9_percent">9% (new / substantial rehab)</option>
                        <option value="4_percent">4% (acquisition / bond)</option>
                    </select>
                </label>
                <label><span data-i18n="view.lihtc.label.placed_year">Placed in service year</span>
                    <input type="number" step="1" name="placed_year" value="${new Date().getFullYear()}" required></label>
                <label><span data-i18n="view.lihtc.label.first_credit_year">First credit year</span>
                    <input type="number" step="1" name="first_credit_year" value="${new Date().getFullYear()}" required></label>
                <button class="primary" type="submit" data-i18n="view.lihtc.btn.add">Add</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.lihtc.h2.context">Tax context</h2>
            <form id="lihtc-tax" class="inline-form">
                <label><span data-i18n="view.lihtc.label.marginal_rate">Marginal federal %</span>
                    <input type="number" step="0.5" name="your_marginal_rate" value="${(state.your_marginal_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.lihtc.btn.update">Update</button>
            </form>
        </div>
        <div id="lihtc-summary"></div>
        <div id="lihtc-table" class="chart-panel"></div>
    `;
    document.getElementById('lihtc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const inv = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            project_name: fd.get('project_name'),
            investment: Number(fd.get('investment')),
            annual_credit: Number(fd.get('annual_credit')),
            credit_type: fd.get('credit_type'),
            placed_year: Number(fd.get('placed_year')),
            first_credit_year: Number(fd.get('first_credit_year')),
        };
        state.investments.push(inv);
        save(state.investments);
        e.target.reset();
        showToast(t('view.lihtc.toast.added'), { level: 'success' });
        render();
    });
    document.getElementById('lihtc-tax').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.your_marginal_rate = (Number(fd.get('your_marginal_rate')) || 37) / 100;
        render();
    });
    render();
}

function analyzeInvestment(inv) {
    const totalCredits = inv.annual_credit * CREDIT_PERIOD;
    const cents_on_dollar = totalCredits > 0 ? inv.investment / totalCredits : 0;
    const irr_approx = totalCredits > inv.investment
        ? Math.pow(totalCredits / inv.investment, 1 / CREDIT_PERIOD) - 1
        : -1;
    const today_year = new Date().getFullYear();
    const yearsRemaining = Math.max(0, inv.first_credit_year + CREDIT_PERIOD - today_year);
    const yearsClaimed = Math.min(CREDIT_PERIOD, Math.max(0, today_year - inv.first_credit_year));
    const claimedToDate = inv.annual_credit * yearsClaimed;
    const remainingCredits = inv.annual_credit * yearsRemaining;
    const compliancePeriodEnd = inv.placed_year + COMPLIANCE_PERIOD;
    const extendedUsePeriodEnd = inv.placed_year + COMPLIANCE_PERIOD + EXTENDED_USE_PERIOD;
    return {
        totalCredits, cents_on_dollar, irr_approx,
        yearsRemaining, claimedToDate, remainingCredits,
        compliancePeriodEnd, extendedUsePeriodEnd,
    };
}

function render() {
    const stats = state.investments.map(analyzeInvestment);
    renderSummary(stats);
    renderTable(stats);
}

function renderSummary(stats) {
    const el = document.getElementById('lihtc-summary');
    if (!el) return;
    const totalInvested = state.investments.reduce((s, i) => s + i.investment, 0);
    const totalCredits = stats.reduce((s, x) => s + x.totalCredits, 0);
    const remainingCredits = stats.reduce((s, x) => s + x.remainingCredits, 0);
    const taxSavings = totalCredits;  // dollar-for-dollar offset to federal tax
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.lihtc.h2.summary">Portfolio summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.lihtc.card.investments">Investments</div>
                    <div class="value">${state.investments.length}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.lihtc.card.total_invested">Total invested</div>
                    <div class="value">$${totalInvested.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.lihtc.card.total_credits">Total credits (10-yr)</div>
                    <div class="value">$${totalCredits.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.lihtc.card.remaining">Remaining credits</div>
                    <div class="value">$${remainingCredits.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.lihtc.card.tax_savings">Federal tax offset</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(stats) {
    const el = document.getElementById('lihtc-table');
    if (!el) return;
    if (!state.investments.length) {
        el.innerHTML = `<h2 data-i18n="view.lihtc.h2.investments">Investments</h2>
            <p class="muted" data-i18n="view.lihtc.empty">No LIHTC investments tracked yet.</p>`;
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.lihtc.h2.investments">Investments</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.lihtc.th.project">Project</th>
                <th data-i18n="view.lihtc.th.type">Type</th>
                <th data-i18n="view.lihtc.th.investment">Investment</th>
                <th data-i18n="view.lihtc.th.annual_credit">Annual credit</th>
                <th data-i18n="view.lihtc.th.total_credit">10-yr total</th>
                <th data-i18n="view.lihtc.th.cents">Cents/$</th>
                <th data-i18n="view.lihtc.th.irr">IRR</th>
                <th data-i18n="view.lihtc.th.compliance_end">Compliance ends</th>
                <th data-i18n="view.lihtc.th.extended_end">Extended use ends</th>
                <th data-i18n="view.lihtc.th.actions">Actions</th>
            </tr></thead>
            <tbody>${state.investments.map((inv, i) => {
                const s = stats[i];
                return `<tr>
                    <td><strong>${esc(inv.project_name)}</strong></td>
                    <td class="muted">${inv.credit_type === '9_percent' ? '9%' : '4%'}</td>
                    <td>$${inv.investment.toLocaleString()}</td>
                    <td>$${inv.annual_credit.toLocaleString()}</td>
                    <td class="pos">$${s.totalCredits.toLocaleString()}</td>
                    <td>${s.cents_on_dollar.toFixed(2)}</td>
                    <td>${s.irr_approx >= 0 ? (s.irr_approx * 100).toFixed(1) + '%' : '—'}</td>
                    <td class="muted">${s.compliancePeriodEnd}</td>
                    <td class="muted">${s.extendedUsePeriodEnd}</td>
                    <td><button class="link neg" data-del="${esc(inv.id)}" data-i18n="view.lihtc.btn.delete">delete</button></td>
                </tr>`;
            }).join('')}</tbody>
        </table>
        <p class="muted small" style="margin-top:10px" data-i18n="view.lihtc.note">
            Recapture risk: if compliance fails in years 1-15, credits CLAW BACK pro-rata.
            Most syndicators absorb this risk via "yield guarantee". Investors typically
            buy at 80-95¢ per credit dollar, earning 4-7% after-tax IRR + losses to
            shelter against passive income.
        </p>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.investments = state.investments.filter(inv => inv.id !== btn.dataset.del);
            save(state.investments);
            render();
        });
    });
}
