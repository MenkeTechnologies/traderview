// True loan APR — all-in APR once upfront fees are folded in, via
// /calc/loan-apr. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%';

export async function renderLoanApr(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.loanapr.h1.title">// LOAN APR</span></h1>
        <p class="muted small" data-i18n="view.loanapr.hint.intro">
            The true APR once upfront fees are folded in — the figure lenders must disclose, and why
            APR sits above the note rate. The payment is set by the note rate on the full loan, but
            you only receive the loan less fees, so the effective rate is higher. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.loanapr.h2.inputs">The loan</h2>
            <form id="loanapr-form" class="inline-form">
                <label><span data-i18n="view.loanapr.label.amount">Loan amount ($)</span>
                    <input type="number" step="0.01" min="0" name="loan_amount_usd" value="200000" required></label>
                <label><span data-i18n="view.loanapr.label.rate">Note rate (%)</span>
                    <input type="number" step="0.001" min="0" name="note_rate_pct" value="6" required></label>
                <label><span data-i18n="view.loanapr.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="360" required></label>
                <label><span data-i18n="view.loanapr.label.fees">Upfront fees ($)</span>
                    <input type="number" step="0.01" min="0" name="fees_usd" value="4000"></label>
            </form>
        </div>
        <div id="loanapr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#loanapr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            loan_amount_usd: Number(fd.get('loan_amount_usd')) || 0,
            note_rate_pct: Number(fd.get('note_rate_pct')) || 0,
            term_months: Number(fd.get('term_months')) || 0,
            fees_usd: Number(fd.get('fees_usd')) || 0,
        };
        try {
            const r = await api.calcLoanApr(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.loanapr.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#loanapr-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.loanapr.h2.result">The true cost</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.loanapr.card.apr">True APR</div>
                    <div class="value pos">${pct(r.true_apr_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.loanapr.card.premium">APR premium</div>
                    <div class="value">${pct(r.apr_premium_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.loanapr.card.payment">Monthly payment</div>
                    <div class="value">${money(r.monthly_payment_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.loanapr.row.payment">Monthly payment</td><td>${money(r.monthly_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.loanapr.row.net">Net proceeds</td><td>${money(r.net_proceeds_usd)}</td></tr>
                    <tr><td data-i18n="view.loanapr.row.premium">APR premium over note</td><td>${pct(r.apr_premium_pct)}</td></tr>
                    <tr><td data-i18n="view.loanapr.row.total">Total of payments</td><td>${money(r.total_of_payments_usd)}</td></tr>
                    <tr><td data-i18n="view.loanapr.row.interest">Total interest</td><td>${money(r.total_interest_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.loanapr.row.apr">True APR</td><td>${pct(r.true_apr_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
