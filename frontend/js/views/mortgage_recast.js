// Mortgage recast — re-amortize the balance after a lump-sum payment, same
// term, lower payment, via /calc/mortgage-recast. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export async function renderMortgageRecast(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.recast.h1.title">// MORTGAGE RECAST</span></h1>
        <p class="muted small" data-i18n="view.recast.hint.intro">
            A recast re-amortizes your loan after a lump-sum principal payment, keeping the same
            rate and remaining term — so the monthly payment drops (unlike extra payments, which
            shorten the term instead). Enter the balance, rate, months left, and the lump sum.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.recast.h2.inputs">The loan</h2>
            <form id="recast-form" class="inline-form">
                <label><span data-i18n="view.recast.label.balance">Current balance ($)</span>
                    <input type="number" step="0.01" min="0" name="current_balance_usd" value="300000" required></label>
                <label><span data-i18n="view.recast.label.rate">Annual rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6" required></label>
                <label><span data-i18n="view.recast.label.term">Remaining term (months)</span>
                    <input type="number" step="1" min="1" name="remaining_term_months" value="360" required></label>
                <label><span data-i18n="view.recast.label.lump">Lump-sum payment ($)</span>
                    <input type="number" step="0.01" min="0" name="lump_sum_usd" value="50000" required></label>
                <label><span data-i18n="view.recast.label.fee">Recast fee ($)</span>
                    <input type="number" step="0.01" min="0" name="recast_fee_usd" value="250"></label>
            </form>
        </div>
        <div id="recast-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#recast-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            current_balance_usd: Number(fd.get('current_balance_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            remaining_term_months: Number(fd.get('remaining_term_months')) || 0,
            lump_sum_usd: Number(fd.get('lump_sum_usd')) || 0,
            recast_fee_usd: Number(fd.get('recast_fee_usd')) || 0,
        };
        try {
            const r = await api.calcMortgageRecast(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.recast.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#recast-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.recast.h2.result">After the recast</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.recast.card.newpayment">New payment</div>
                    <div class="value pos">${money(r.new_payment_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.recast.card.savings">Monthly savings</div>
                    <div class="value pos">${money(r.monthly_savings_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.recast.card.intsaved">Interest saved</div>
                    <div class="value">${money(r.interest_saved_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.recast.row.oldpayment">Old payment</td><td>${money(r.old_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.newbalance">New balance</td><td>${money(r.new_balance_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.newpayment">New payment</td><td>${money(r.new_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.oldint">Old total interest</td><td>${money(r.old_total_interest_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.newint">New total interest</td><td>${money(r.new_total_interest_usd)}</td></tr>
                    <tr><td data-i18n="view.recast.row.netsaved">Net interest saved (after fee)</td><td>${money(r.net_interest_saved_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.recast.row.intsaved">Interest saved</td><td>${money(r.interest_saved_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
