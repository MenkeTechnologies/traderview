// SPIA — single-premium immediate annuity: monthly income from a lump sum,
// the payout rate, and total received, via /calc/spia. Updates live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['premium_usd', 'Premium (lump sum, $)', 100000],
    ['payout_years', 'Payout period (years)', 20],
    ['annual_rate_pct', 'Assumed rate (%)', 5],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const money0 = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderSpia(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.spia.h1.title">// SPIA INCOME</span></h1>
        <p class="muted small" data-i18n="view.spia.hint.intro">
            A single-premium immediate annuity turns a lump sum into guaranteed income starting
            now. The monthly payment is the annuity that exhausts the premium over the payout
            period at the insurer's assumed rate — the same math as a loan payment, in reverse.
            The payout rate is the annual income as a percent of the premium. (Period-certain
            approximation; a true life annuity also prices in mortality.) Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.spia.h2.inputs">The annuity</h2>
            <form id="spia-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.spia.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="spia-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#spia-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcSpia(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.spia.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#spia-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.spia.h2.result">The income</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.spia.card.monthly">Monthly income</div>
                    <div class="value pos">${money(r.monthly_income_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.spia.card.annual">Annual income</div>
                    <div class="value">${money0(r.annual_income_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.spia.card.payout">Payout rate</div>
                    <div class="value">${Number(r.payout_rate_pct).toFixed(2)}%</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.spia.col.line">Line</th><th data-i18n="view.spia.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.spia.row.total">Total received (period)</td><td>${money0(r.total_received_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.spia.row.interest">Interest credited</td>
                        <td class="${r.interest_earned_usd >= 0 ? 'pos' : 'neg'}">${money0(r.interest_earned_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
