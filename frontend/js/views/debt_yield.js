// Debt yield & loan sizing — commercial-RE lender ratios (debt yield, LTV,
// LTC) and the max loan each allows, with the binding constraint, via
// /calc/debt-yield. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['noi_usd', 'Net operating income ($)', 100000],
    ['property_value_usd', 'Appraised value ($)', 1400000],
    ['total_project_cost_usd', 'Total project cost ($)', 1350000],
    ['loan_amount_usd', 'Proposed loan ($)', 1000000],
    ['min_debt_yield_pct', 'Min debt yield (%)', 10],
    ['max_ltv_pct', 'Max LTV (%)', 75],
    ['max_ltc_pct', 'Max LTC (%)', 80],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => Number(n).toFixed(2) + '%';
const BINDING = { debt_yield: 'Debt yield', ltv: 'LTV', ltc: 'LTC' };

export async function renderDebtYield(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dy.h1.title">// DEBT YIELD & LOAN SIZING</span></h1>
        <p class="muted small" data-i18n="view.dy.hint.intro">
            A commercial lender sizes a loan against three ceilings, and the smallest wins:
            debt yield (NOI ÷ loan, a rate-independent risk measure, floored at ~8–10%), LTV
            (loan ÷ value), and LTC (loan ÷ total cost). This shows each ratio for your proposed
            loan, the max loan each ceiling allows, and which one binds. Complements cap rate
            and DSCR. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.dy.h2.inputs">The deal</h2>
            <form id="dy-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.dy.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="dy-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#dy-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcDebtYield(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.dy.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#dy-result');
    const fitCls = r.loan_fits ? 'pos' : 'neg';
    const binding = BINDING[r.binding_constraint] || r.binding_constraint;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.dy.h2.result">Loan sizing</h2>
            <div class="cards">
                <div class="card ${fitCls}"><div class="label" data-i18n="view.dy.card.maxloan">Max loan</div>
                    <div class="value ${fitCls}">${money(r.max_loan_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dy.card.binding">Binding constraint</div>
                    <div class="value">${binding}</div></div>
                <div class="card ${fitCls}"><div class="label" data-i18n="view.dy.card.fits">Proposed loan fits?</div>
                    <div class="value ${fitCls}">${r.loan_fits ? t('view.dy.yes') : t('view.dy.no')}</div></div>
            </div>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.dy.col.metric">Metric</th>
                    <th data-i18n="view.dy.col.current">Current</th>
                    <th data-i18n="view.dy.col.maxloan">Max loan</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.dy.row.dy">Debt yield</td><td>${pct(r.debt_yield_pct)}</td><td>${money(r.max_loan_by_debt_yield_usd)}</td></tr>
                    <tr><td data-i18n="view.dy.row.ltv">LTV</td><td>${pct(r.ltv_pct)}</td><td>${money(r.max_loan_by_ltv_usd)}</td></tr>
                    <tr><td data-i18n="view.dy.row.ltc">LTC</td><td>${pct(r.ltc_pct)}</td><td>${money(r.max_loan_by_ltc_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
