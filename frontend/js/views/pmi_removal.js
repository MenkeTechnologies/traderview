// PMI removal timeline — months until the loan amortizes to 80% / 78% of the
// original home value, via /calc/pmi-removal. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';
const months = (n) => {
    if (n == null) return '—';
    const m = Math.round(n);
    const y = Math.floor(m / 12);
    const rem = m % 12;
    return `${m} (${y}y ${rem}m)`;
};

export async function renderPmiRemoval(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pmi.h1.title">// PMI REMOVAL</span></h1>
        <p class="muted small" data-i18n="view.pmi.hint.intro">
            When private mortgage insurance drops off a conventional loan. PMI is tied to
            loan-to-value against the original home value — you can request cancellation at 80% LTV,
            and the servicer must cancel automatically at 78% (Homeowners Protection Act). This shows
            the month scheduled payments reach each threshold. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pmi.h2.inputs">The loan</h2>
            <form id="pmi-form" class="inline-form">
                <label><span data-i18n="view.pmi.label.value">Original home value ($)</span>
                    <input type="number" step="0.01" min="0" name="original_home_value_usd" value="400000" required></label>
                <label><span data-i18n="view.pmi.label.loan">Original loan ($)</span>
                    <input type="number" step="0.01" min="0" name="original_loan_usd" value="360000" required></label>
                <label><span data-i18n="view.pmi.label.rate">Annual rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6" required></label>
                <label><span data-i18n="view.pmi.label.term">Loan term (months)</span>
                    <input type="number" step="1" min="1" name="loan_term_months" value="360" required></label>
            </form>
        </div>
        <div id="pmi-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pmi-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            original_home_value_usd: Number(fd.get('original_home_value_usd')) || 0,
            original_loan_usd: Number(fd.get('original_loan_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            loan_term_months: Number(fd.get('loan_term_months')) || 0,
        };
        try {
            const r = await api.calcPmiRemoval(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.pmi.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#pmi-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pmi.h2.result">When PMI drops</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.pmi.card.ltv">Original LTV</div>
                    <div class="value">${pct(r.original_ltv_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pmi.card.request">Request at 80%</div>
                    <div class="value">${months(r.months_to_80)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.pmi.card.auto">Auto at 78%</div>
                    <div class="value pos">${months(r.months_to_78)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.pmi.row.payment">Monthly payment</td><td>${money(r.monthly_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.pmi.row.t80">80% balance threshold</td><td>${money(r.target_80_balance_usd)}</td></tr>
                    <tr><td data-i18n="view.pmi.row.t78">78% balance threshold</td><td>${money(r.target_78_balance_usd)}</td></tr>
                    <tr><td data-i18n="view.pmi.row.request">Months to request (80%)</td><td>${months(r.months_to_80)}</td></tr>
                    <tr class="emph"><td data-i18n="view.pmi.row.auto">Months to automatic (78%)</td><td>${months(r.months_to_78)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
