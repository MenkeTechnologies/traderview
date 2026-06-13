// Car affordability — the 20/4/10 rule worked back to a max car price, via
// /calc/car-affordability. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export async function renderCarAffordability(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.caraff.h1.title">// CAR AFFORDABILITY</span></h1>
        <p class="muted small" data-i18n="view.caraff.hint.intro">
            The 20/4/10 rule of thumb — put 20% down, finance for no more than 4 years, and keep
            total vehicle spending under 10% of gross income. This works back from the income cap to
            the most car you can afford. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.caraff.h2.inputs">Your budget</h2>
            <form id="caraff-form" class="inline-form">
                <label><span data-i18n="view.caraff.label.income">Annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_income_usd" value="60000" required></label>
                <label><span data-i18n="view.caraff.label.down">Down payment (%)</span>
                    <input type="number" step="1" min="0" max="99" name="down_payment_pct" value="20" required></label>
                <label><span data-i18n="view.caraff.label.term">Loan term (months)</span>
                    <input type="number" step="1" min="1" name="loan_term_months" value="48" required></label>
                <label><span data-i18n="view.caraff.label.apr">APR (%)</span>
                    <input type="number" step="0.01" min="0" name="apr_pct" value="6" required></label>
                <label><span data-i18n="view.caraff.label.pct">Max % of gross income</span>
                    <input type="number" step="0.1" min="0" name="max_payment_pct_of_income" value="10" required></label>
                <label><span data-i18n="view.caraff.label.insfuel">Insurance + fuel / mo ($)</span>
                    <input type="number" step="0.01" min="0" name="insurance_fuel_monthly_usd" value="0"></label>
            </form>
        </div>
        <div id="caraff-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#caraff-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            annual_income_usd: Number(fd.get('annual_income_usd')) || 0,
            down_payment_pct: Number(fd.get('down_payment_pct')) || 0,
            loan_term_months: Number(fd.get('loan_term_months')) || 0,
            apr_pct: Number(fd.get('apr_pct')) || 0,
            max_payment_pct_of_income: Number(fd.get('max_payment_pct_of_income')) || 0,
            insurance_fuel_monthly_usd: Number(fd.get('insurance_fuel_monthly_usd')) || 0,
        };
        try {
            const r = await api.calcCarAffordability(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.caraff.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#caraff-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.caraff.h2.result">What you can afford</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.caraff.card.price">Max car price</div>
                    <div class="value pos">${money(r.max_car_price_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.caraff.card.loan">Max loan</div>
                    <div class="value">${money(r.max_loan_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.caraff.card.payment">Payment budget</div>
                    <div class="value">${money(r.monthly_payment_budget_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.caraff.row.budget">Transport budget / mo</td><td>${money(r.monthly_transport_budget_usd)}</td></tr>
                    <tr><td data-i18n="view.caraff.row.payment">Payment budget / mo</td><td>${money(r.monthly_payment_budget_usd)}</td></tr>
                    <tr><td data-i18n="view.caraff.row.loan">Max loan</td><td>${money(r.max_loan_usd)}</td></tr>
                    <tr><td data-i18n="view.caraff.row.down">Down payment needed</td><td>${money(r.down_payment_needed_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.caraff.row.price">Max car price</td><td>${money(r.max_car_price_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
