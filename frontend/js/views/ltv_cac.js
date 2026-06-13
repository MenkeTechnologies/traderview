// LTV:CAC — customer lifetime value vs acquisition cost, the ratio (3:1
// rule), and CAC payback months, via /calc/ltv-cac. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['avg_monthly_revenue_usd', 'Avg monthly revenue / customer ($)', 100],
    ['gross_margin_pct', 'Gross margin (%)', 80],
    ['monthly_churn_rate_pct', 'Monthly churn (%)', 5],
    ['cac_usd', 'Customer acquisition cost ($)', 400],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const RATING = {
    healthy: ['Healthy (≥3:1)', 'pos'],
    marginal: ['Marginal (1–3:1)', ''],
    unprofitable: ['Unprofitable (<1:1)', 'neg'],
    'n/a': ['—', ''],
};

export async function renderLtvCac(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ltvcac.h1.title">// LTV : CAC</span></h1>
        <p class="muted small" data-i18n="view.ltvcac.hint.intro">
            Does acquiring customers pay off? LTV (lifetime value) = monthly gross profit per
            customer × average lifetime (100 ÷ monthly churn %) — gross profit, since you keep
            only the margin. CAC is the sales + marketing cost per new customer. The LTV:CAC
            ratio rule of thumb is 3:1; below ~1:1 you lose money on each customer, far above
            3:1 you may be under-investing in growth. CAC payback is the months of gross profit
            to recoup the cost. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ltvcac.h2.inputs">Unit economics</h2>
            <form id="ltvcac-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.ltvcac.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="ltvcac-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ltvcac-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcLtvCac(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.ltvcac.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#ltvcac-result');
    const [ratingLabel, ratingCls] = RATING[r.rating] || [r.rating, ''];
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ltvcac.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card ${ratingCls}"><div class="label" data-i18n="view.ltvcac.card.ratio">LTV : CAC ratio</div>
                    <div class="value ${ratingCls}">${Number(r.ltv_cac_ratio).toFixed(2)} : 1</div></div>
                <div class="card ${ratingCls}"><div class="label" data-i18n="view.ltvcac.card.rating">Rating</div>
                    <div class="value ${ratingCls}">${ratingLabel}</div></div>
                <div class="card"><div class="label" data-i18n="view.ltvcac.card.payback">CAC payback</div>
                    <div class="value">${Number(r.cac_payback_months).toFixed(1)} <span data-i18n="view.ltvcac.months">mo</span></div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.ltvcac.col.line">Line</th><th data-i18n="view.ltvcac.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.ltvcac.row.ltv">Lifetime value (LTV)</td><td>${money(r.ltv_usd)}</td></tr>
                    <tr><td data-i18n="view.ltvcac.row.lifetime">Avg lifetime</td><td>${Number(r.avg_lifetime_months).toFixed(1)} mo</td></tr>
                    <tr><td data-i18n="view.ltvcac.row.gp">Monthly gross profit / customer</td><td>${money(r.monthly_gross_profit_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
