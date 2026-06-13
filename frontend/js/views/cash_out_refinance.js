// Cash-out refinance — equity you can pull out, the new payment, and remaining
// equity, via /calc/cash-out-refinance. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';

export async function renderCashOutRefinance(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cashout.h1.title">// CASH-OUT REFINANCE</span></h1>
        <p class="muted small" data-i18n="view.cashout.hint.intro">
            How much equity you can pull out of your home. The new loan is capped at the lender's max
            LTV against the home's value; the cash in hand is what's left after paying off the old
            balance and closing costs. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.cashout.h2.inputs">The property</h2>
            <form id="cashout-form" class="inline-form">
                <label><span data-i18n="view.cashout.label.value">Home value ($)</span>
                    <input type="number" step="0.01" min="0" name="home_value_usd" value="500000" required></label>
                <label><span data-i18n="view.cashout.label.balance">Current balance ($)</span>
                    <input type="number" step="0.01" min="0" name="current_balance_usd" value="250000" required></label>
                <label><span data-i18n="view.cashout.label.ltv">Max LTV (%)</span>
                    <input type="number" step="0.1" min="0" max="100" name="max_ltv_pct" value="80" required></label>
                <label><span data-i18n="view.cashout.label.rate">New rate (%)</span>
                    <input type="number" step="0.001" min="0" name="new_rate_pct" value="6.5" required></label>
                <label><span data-i18n="view.cashout.label.term">New term (months)</span>
                    <input type="number" step="1" min="1" name="new_term_months" value="360" required></label>
                <label><span data-i18n="view.cashout.label.costs">Closing costs ($)</span>
                    <input type="number" step="0.01" min="0" name="closing_costs_usd" value="6000"></label>
            </form>
        </div>
        <div id="cashout-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cashout-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            home_value_usd: Number(fd.get('home_value_usd')) || 0,
            current_balance_usd: Number(fd.get('current_balance_usd')) || 0,
            max_ltv_pct: Number(fd.get('max_ltv_pct')) || 0,
            new_rate_pct: Number(fd.get('new_rate_pct')) || 0,
            new_term_months: Number(fd.get('new_term_months')) || 0,
            closing_costs_usd: Number(fd.get('closing_costs_usd')) || 0,
        };
        try {
            const r = await api.calcCashOutRefinance(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.cashout.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#cashout-result');
    const cashCls = r.net_cash_out_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.cashout.h2.result">The cash-out</h2>
            <div class="cards">
                <div class="card ${cashCls}"><div class="label" data-i18n="view.cashout.card.net">Net cash out</div>
                    <div class="value ${cashCls}">${money(r.net_cash_out_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cashout.card.payment">New payment</div>
                    <div class="value">${money(r.new_monthly_payment_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cashout.card.equity">Equity remaining</div>
                    <div class="value">${money(r.equity_remaining_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.cashout.row.maxloan">Max new loan</td><td>${money(r.max_new_loan_usd)}</td></tr>
                    <tr><td data-i18n="view.cashout.row.gross">Gross cash out</td><td>${money(r.gross_cash_out_usd)}</td></tr>
                    <tr><td data-i18n="view.cashout.row.payment">New monthly payment</td><td>${money(r.new_monthly_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.cashout.row.currentltv">Current LTV</td><td>${pct(r.current_ltv_pct)}</td></tr>
                    <tr><td data-i18n="view.cashout.row.newltv">New LTV</td><td>${pct(r.new_ltv_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.cashout.row.net">Net cash out</td><td>${money(r.net_cash_out_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
