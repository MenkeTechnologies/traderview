// Mortgage discount points — break-even on buying down the rate: the
// bought-down rate, points cost, payment savings, and months to recoup, via
// /calc/mortgage-points. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['loan_amount_usd', 'Loan amount ($)', 400000],
    ['term_years', 'Term (years)', 30],
    ['base_rate_pct', 'Base rate (%)', 7],
    ['points', 'Discount points', 2],
    ['rate_reduction_per_point_pct', 'Rate cut per point (%)', 0.25],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const months = (n) => Number(n).toFixed(1);

export async function renderMortgagePoints(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mpt.h1.title">// MORTGAGE POINTS BREAK-EVEN</span></h1>
        <p class="muted small" data-i18n="view.mpt.hint.intro">
            Paying discount points up front (1 point = 1% of the loan) lowers the rate, and so
            the monthly payment. Whether it pays off comes down to how long you keep the loan:
            the break-even is the months for the payment savings to recoup the points cost.
            Keep the loan past it and the points win; sell or refinance before it and you lose.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.mpt.h2.inputs">The loan</h2>
            <form id="mpt-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.mpt.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="mpt-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#mpt-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcMortgagePoints(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.mpt.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#mpt-result');
    const be = r.breakeven_months == null
        ? '—'
        : `${months(r.breakeven_months)} ${t('view.mpt.mo')} (${(r.breakeven_months / 12).toFixed(1)} ${t('view.mpt.yr')})`;
    const netCls = r.lifetime_net_savings_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.mpt.h2.result">The break-even</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.mpt.card.breakeven">Break-even</div>
                    <div class="value pos">${be}</div></div>
                <div class="card"><div class="label" data-i18n="view.mpt.card.savings">Monthly savings</div>
                    <div class="value">${money(r.monthly_savings_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.mpt.card.cost">Points cost</div>
                    <div class="value neg">${money(r.points_cost_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mpt.card.rate">Bought-down rate</div>
                    <div class="value">${Number(r.bought_down_rate_pct).toFixed(3)}%</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.mpt.col.line">Line</th><th data-i18n="view.mpt.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.mpt.row.base">Base payment</td><td>${money(r.base_monthly_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.mpt.row.new">Payment with points</td><td>${money(r.new_monthly_payment_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.mpt.row.lifetime">Lifetime net savings (full term)</td>
                        <td class="${netCls}">${money(r.lifetime_net_savings_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
