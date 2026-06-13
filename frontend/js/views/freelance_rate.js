// Freelance rate — the hourly rate a 1099 contractor must charge to net a
// target after SE tax, income tax, expenses, and benefits, via
// /calc/freelance-rate. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';

export async function renderFreelanceRate(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.freelance.h1.title">// FREELANCE RATE</span></h1>
        <p class="muted small" data-i18n="view.freelance.hint.intro">
            The hourly rate a contractor must charge to actually take home a target — working back
            through self-employment tax, income tax, business expenses, and self-funded benefits,
            spread over billable hours (well under a full year). That's why a freelance rate runs far
            above an employee wage. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.freelance.h2.inputs">Your target</h2>
            <form id="freelance-form" class="inline-form">
                <label><span data-i18n="view.freelance.label.takehome">Desired take-home / yr ($)</span>
                    <input type="number" step="0.01" min="0" name="desired_annual_take_home_usd" value="80000" required></label>
                <label><span data-i18n="view.freelance.label.hours">Billable hours / yr</span>
                    <input type="number" step="1" min="1" name="billable_hours_per_year" value="1500" required></label>
                <label><span data-i18n="view.freelance.label.expenses">Business expenses / yr ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_business_expenses_usd" value="10000"></label>
                <label><span data-i18n="view.freelance.label.benefits">Self-funded benefits / yr ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_benefits_usd" value="12000"></label>
                <label><span data-i18n="view.freelance.label.se">SE tax rate (%)</span>
                    <input type="number" step="0.1" min="0" name="self_employment_tax_rate_pct" value="15.3"></label>
                <label><span data-i18n="view.freelance.label.income">Income tax rate (%)</span>
                    <input type="number" step="0.1" min="0" name="income_tax_rate_pct" value="22"></label>
            </form>
        </div>
        <div id="freelance-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#freelance-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            desired_annual_take_home_usd: Number(fd.get('desired_annual_take_home_usd')) || 0,
            billable_hours_per_year: Number(fd.get('billable_hours_per_year')) || 0,
            annual_business_expenses_usd: Number(fd.get('annual_business_expenses_usd')) || 0,
            annual_benefits_usd: Number(fd.get('annual_benefits_usd')) || 0,
            self_employment_tax_rate_pct: Number(fd.get('self_employment_tax_rate_pct')) || 0,
            income_tax_rate_pct: Number(fd.get('income_tax_rate_pct')) || 0,
        };
        try {
            const r = await api.calcFreelanceRate(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.freelance.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#freelance-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.freelance.h2.result">Your rate</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.freelance.card.hourly">Required hourly</div>
                    <div class="value pos">${money(r.required_hourly_rate_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.freelance.card.revenue">Revenue needed</div>
                    <div class="value">${money(r.revenue_needed_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.freelance.card.rate">Combined tax rate</div>
                    <div class="value">${pct(r.combined_tax_rate_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.freelance.row.pretax">Pre-tax profit needed</td><td>${money(r.pretax_profit_needed_usd)}</td></tr>
                    <tr><td data-i18n="view.freelance.row.tax">Total tax</td><td>${money(r.total_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.freelance.row.revenue">Revenue needed</td><td>${money(r.revenue_needed_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.freelance.row.hourly">Required hourly rate</td><td>${money(r.required_hourly_rate_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
