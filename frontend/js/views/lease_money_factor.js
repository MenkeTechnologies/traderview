// Car lease payment — depreciation + finance fee from cap cost, residual,
// and money factor, with the equivalent APR, via /calc/lease-payment. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['cap_cost_usd', 'Capitalized cost ($)', 35000],
    ['residual_value_usd', 'Residual value ($)', 21000],
    ['term_months', 'Term (months)', 36],
    ['money_factor', 'Money factor', 0.00125],
    ['sales_tax_pct', 'Sales tax (%)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const money0 = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderLeasePayment(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lmf.h1.title">// CAR LEASE PAYMENT</span></h1>
        <p class="muted small" data-i18n="view.lmf.hint.intro">
            A lease payment is depreciation + finance fee + tax. Depreciation = (cap cost −
            residual) ÷ term — paying for the value lost while you drive. The finance fee =
            (cap cost + residual) × money factor — the interest; the money factor is a disguised
            rate (APR = money factor × 2400). Tax is on the payment in most states. Updates as
            you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lmf.h2.inputs">The lease</h2>
            <form id="lmf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.lmf.label.${key}">${label}</span>
                        <input type="number" step="${key === 'money_factor' ? '0.00001' : '0.01'}" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="lmf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lmf-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcLeasePayment(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.lmf.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#lmf-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.lmf.h2.result">The payment</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.lmf.card.payment">Monthly payment</div>
                    <div class="value pos">${money(r.monthly_payment_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lmf.card.apr">Equivalent APR</div>
                    <div class="value">${Number(r.equivalent_apr_pct).toFixed(2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.lmf.card.total">Total lease cost</div>
                    <div class="value">${money0(r.total_lease_cost_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.lmf.col.line">Line</th><th data-i18n="view.lmf.col.amount">Monthly</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.lmf.row.dep">Depreciation fee</td><td>${money(r.monthly_depreciation_usd)}</td></tr>
                    <tr><td data-i18n="view.lmf.row.fin">Finance (rent) fee</td><td>${money(r.monthly_finance_usd)}</td></tr>
                    <tr><td data-i18n="view.lmf.row.tax">Tax</td><td>${money(r.monthly_tax_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.lmf.row.total">Monthly payment</td><td class="pos">${money(r.monthly_payment_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
