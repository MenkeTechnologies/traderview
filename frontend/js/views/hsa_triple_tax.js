// HSA triple-tax advantage — HSA vs a taxable account over a horizon, the
// dollar value of deductible-in / tax-free-growth / tax-free-out, via
// /calc/hsa-triple-tax. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['annual_contribution_usd', 'Annual contribution ($)', 4000],
    ['years', 'Years', 20],
    ['annual_growth_pct', 'Annual growth (%)', 7],
    ['marginal_tax_rate_pct', 'Marginal tax rate (%)', 24],
    ['ltcg_rate_pct', 'Long-term cap-gains rate (%)', 15],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderHsaTripleTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hsatt.h1.title">// HSA TRIPLE-TAX ADVANTAGE</span></h1>
        <p class="muted small" data-i18n="view.hsatt.hint.intro">
            The HSA is the only account taxed favorably three ways: contributions are
            deductible, growth is tax-free, and withdrawals for medical expenses are tax-free.
            This projects an HSA against a taxable account funded with the same gross income —
            the taxable side invests only the after-tax amount and pays cap-gains on its growth.
            The gap is the dollar value of the triple-tax treatment. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.hsatt.h2.inputs">The plan</h2>
            <form id="hsa-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.hsatt.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="hsa-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#hsa-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        body.years = Math.max(0, Math.round(body.years));
        try {
            const r = await api.calcHsaTripleTax(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.hsatt.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#hsa-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.hsatt.h2.result">The advantage</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.hsatt.card.advantage">HSA advantage</div>
                    <div class="value pos">${money(r.hsa_advantage_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.hsatt.card.hsa">HSA ending value</div>
                    <div class="value">${money(r.hsa_ending_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.hsatt.card.taxable">Taxable ending value</div>
                    <div class="value">${money(r.taxable_ending_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.hsatt.col.line">Line</th><th data-i18n="view.hsatt.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.hsatt.row.contributions">Total contributions</td><td>${money(r.total_contributions_usd)}</td></tr>
                    <tr><td data-i18n="view.hsatt.row.upfront">Upfront tax savings (deduction)</td><td>${money(r.upfront_tax_savings_usd)}</td></tr>
                    <tr><td data-i18n="view.hsatt.row.hsa">HSA ending (tax-free)</td><td>${money(r.hsa_ending_usd)}</td></tr>
                    <tr><td data-i18n="view.hsatt.row.taxable">Taxable ending (after cap-gains)</td><td>${money(r.taxable_ending_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.hsatt.row.advantage">Triple-tax advantage</td><td class="pos">${money(r.hsa_advantage_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
