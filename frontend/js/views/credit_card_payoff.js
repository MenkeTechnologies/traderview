// Credit-card minimum-payment trap — months and interest paying only the
// declining minimum vs a fixed payment, via /calc/credit-card-payoff. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const months = (n) => {
    if (n == null) return '—';
    const y = Math.floor(n / 12);
    const rem = n % 12;
    return `${n} (${y}y ${rem}m)`;
};

export async function renderCreditCardPayoff(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ccpayoff.h1.title">// CREDIT CARD PAYOFF</span></h1>
        <p class="muted small" data-i18n="view.ccpayoff.hint.intro">
            The minimum-payment trap. The minimum due is the greater of a floor and a percent of the
            balance, so it shrinks as you pay down — most of each payment goes to interest, stretching
            a small balance into decades. Compare it to a fixed monthly payment. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ccpayoff.h2.inputs">The card</h2>
            <form id="ccpayoff-form" class="inline-form">
                <label><span data-i18n="view.ccpayoff.label.balance">Balance ($)</span>
                    <input type="number" step="0.01" min="0" name="balance_usd" value="5000" required></label>
                <label><span data-i18n="view.ccpayoff.label.apr">APR (%)</span>
                    <input type="number" step="0.01" min="0" name="apr_pct" value="22" required></label>
                <label><span data-i18n="view.ccpayoff.label.minpct">Minimum payment (% of balance)</span>
                    <input type="number" step="0.1" min="0" name="min_payment_pct" value="2" required></label>
                <label><span data-i18n="view.ccpayoff.label.floor">Minimum payment floor ($)</span>
                    <input type="number" step="1" min="0" name="min_payment_floor_usd" value="25"></label>
                <label><span data-i18n="view.ccpayoff.label.fixed">Fixed payment to compare ($)</span>
                    <input type="number" step="0.01" min="0" name="fixed_payment_usd" value="200"></label>
            </form>
        </div>
        <div id="ccpayoff-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ccpayoff-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            balance_usd: Number(fd.get('balance_usd')) || 0,
            apr_pct: Number(fd.get('apr_pct')) || 0,
            min_payment_pct: Number(fd.get('min_payment_pct')) || 0,
            min_payment_floor_usd: Number(fd.get('min_payment_floor_usd')) || 0,
            fixed_payment_usd: Number(fd.get('fixed_payment_usd')) || 0,
        };
        try {
            const r = await api.calcCreditCardPayoff(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.ccpayoff.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#ccpayoff-result');
    const minMonths = r.never_pays_off ? t('view.ccpayoff.never') : months(r.months_minimum);
    const fixedRows = r.months_fixed == null ? '' : `
        <tr><td data-i18n="view.ccpayoff.row.fixedmonths">Months (fixed)</td><td>${months(r.months_fixed)}</td></tr>
        <tr><td data-i18n="view.ccpayoff.row.fixedint">Interest (fixed)</td><td>${money(r.total_interest_fixed_usd)}</td></tr>
        ${r.interest_saved_usd == null ? '' : `<tr class="pos"><td data-i18n="view.ccpayoff.row.saved">Interest saved</td><td>${money(r.interest_saved_usd)}</td></tr>`}`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ccpayoff.h2.result">The cost</h2>
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.ccpayoff.card.minmonths">Minimum-only payoff</div>
                    <div class="value neg">${minMonths}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.ccpayoff.card.minint">Interest (minimum)</div>
                    <div class="value neg">${money(r.total_interest_minimum_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccpayoff.card.firstmin">First minimum</div>
                    <div class="value">${money(r.first_minimum_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.ccpayoff.row.minmonths">Months (minimum only)</td><td>${minMonths}</td></tr>
                    <tr><td data-i18n="view.ccpayoff.row.minint">Total interest (minimum)</td><td>${money(r.total_interest_minimum_usd)}</td></tr>
                    <tr><td data-i18n="view.ccpayoff.row.minpaid">Total paid (minimum)</td><td>${money(r.total_paid_minimum_usd)}</td></tr>
                    ${fixedRows}
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
