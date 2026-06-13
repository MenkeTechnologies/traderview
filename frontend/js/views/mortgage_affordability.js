// Mortgage affordability — max home price under the 28/36 rule, via
// /calc/mortgage-affordability. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export async function renderMortgageAffordability(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.afford.h1.title">// MORTGAGE AFFORDABILITY</span></h1>
        <p class="muted small" data-i18n="view.afford.hint.intro">
            The most house you can buy under the 28/36 rule — lenders cap housing cost (PITI) at 28%
            of gross monthly income and total debt at 36%. The tighter cap sets the budget; since
            taxes and the loan payment both scale with price, it solves for the max home price.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.afford.h2.inputs">Your finances</h2>
            <form id="afford-form" class="inline-form">
                <label><span data-i18n="view.afford.label.income">Annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_income_usd" value="100000" required></label>
                <label><span data-i18n="view.afford.label.debts">Other monthly debts ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_debts_usd" value="500"></label>
                <label><span data-i18n="view.afford.label.down">Down payment ($)</span>
                    <input type="number" step="0.01" min="0" name="down_payment_usd" value="50000" required></label>
                <label><span data-i18n="view.afford.label.rate">Mortgage rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6.5" required></label>
                <label><span data-i18n="view.afford.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="360" required></label>
                <label><span data-i18n="view.afford.label.tax">Property tax rate (% / yr)</span>
                    <input type="number" step="0.01" min="0" name="property_tax_rate_pct" value="1.2"></label>
                <label><span data-i18n="view.afford.label.insurance">Annual insurance ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_insurance_usd" value="1500"></label>
                <label><span data-i18n="view.afford.label.front">Front-end (%)</span>
                    <input type="number" step="0.1" min="0" name="front_end_pct" value="28"></label>
                <label><span data-i18n="view.afford.label.back">Back-end (%)</span>
                    <input type="number" step="0.1" min="0" name="back_end_pct" value="36"></label>
            </form>
        </div>
        <div id="afford-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#afford-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            annual_income_usd: Number(fd.get('annual_income_usd')) || 0,
            monthly_debts_usd: Number(fd.get('monthly_debts_usd')) || 0,
            down_payment_usd: Number(fd.get('down_payment_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            term_months: Number(fd.get('term_months')) || 0,
            property_tax_rate_pct: Number(fd.get('property_tax_rate_pct')) || 0,
            annual_insurance_usd: Number(fd.get('annual_insurance_usd')) || 0,
            front_end_pct: Number(fd.get('front_end_pct')) || 0,
            back_end_pct: Number(fd.get('back_end_pct')) || 0,
        };
        try {
            const r = await api.calcMortgageAffordability(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.afford.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#afford-result');
    const bindKey = r.binding_constraint === 'front' ? 'view.afford.bind.front' : 'view.afford.bind.back';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.afford.h2.result">What you can afford</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.afford.card.price">Max home price</div>
                    <div class="value pos">${money(r.max_home_price_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.afford.card.loan">Max loan</div>
                    <div class="value">${money(r.max_loan_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.afford.card.piti">Max PITI / mo</div>
                    <div class="value">${money(r.max_piti_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.afford.row.income">Monthly income</td><td>${money(r.monthly_income_usd)}</td></tr>
                    <tr><td data-i18n="view.afford.row.front">Front-end cap (28%)</td><td>${money(r.front_end_max_usd)}</td></tr>
                    <tr><td data-i18n="view.afford.row.back">Back-end room (36% − debts)</td><td>${money(r.back_end_max_usd)}</td></tr>
                    <tr><td data-i18n="view.afford.row.binding">Binding constraint</td><td data-i18n="${bindKey}">—</td></tr>
                    <tr><td data-i18n="view.afford.row.loan">Max loan</td><td>${money(r.max_loan_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.afford.row.price">Max home price</td><td>${money(r.max_home_price_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
