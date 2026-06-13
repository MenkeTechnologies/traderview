// Treasury-bill yields — convert a bill's price or quoted discount rate into
// bank-discount, money-market, coupon-equivalent, and effective annual yields,
// via /calc/tbill-yield. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { minimumFractionDigits: 3, maximumFractionDigits: 4 }) + '%';

export async function renderTbillYield(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tbill.h1.title">// T-BILL YIELD</span></h1>
        <p class="muted small" data-i18n="view.tbill.hint.intro">
            T-bills are quoted on a bank-discount basis, which understates the true return. Enter
            either the price per $100 or the quoted discount rate and the days to maturity to see
            the money-market, coupon-equivalent (Treasury investment rate), and effective annual
            yields. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.tbill.h2.inputs">The bill</h2>
            <form id="tbill-form" class="inline-form">
                <label><span data-i18n="view.tbill.label.mode">Given</span>
                    <select name="mode">
                        <option value="from_discount" data-i18n="view.tbill.mode.discount">Discount rate (%)</option>
                        <option value="from_price" data-i18n="view.tbill.mode.price">Price per $100</option>
                    </select></label>
                <label><span data-i18n="view.tbill.label.value">Value</span>
                    <input type="number" step="0.0001" min="0" name="value" value="5" required></label>
                <label><span data-i18n="view.tbill.label.days">Days to maturity</span>
                    <input type="number" step="1" min="0" name="days_to_maturity" value="91" required></label>
                <label><span data-i18n="view.tbill.label.face">Face value ($)</span>
                    <input type="number" step="0.01" min="0" name="face_value" value="1000" required></label>
                <label><span data-i18n="view.tbill.label.year">Days in year</span>
                    <select name="year_days">
                        <option value="365" data-i18n="view.tbill.year.365">365</option>
                        <option value="366" data-i18n="view.tbill.year.366">366 (leap)</option>
                    </select></label>
            </form>
        </div>
        <div id="tbill-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#tbill-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            mode: fd.get('mode'),
            value: Number(fd.get('value')) || 0,
            days_to_maturity: Number(fd.get('days_to_maturity')) || 0,
            face_value: Number(fd.get('face_value')) || 0,
            year_days: Number(fd.get('year_days')) || 365,
        };
        try {
            const r = await api.calcTbillYield(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.tbill.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#tbill-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tbill.h2.result">The yields</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.tbill.card.price">Price / $100</div>
                    <div class="value">${Number(r.price_per_100).toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 6 })}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.tbill.card.investment">Coupon-equivalent</div>
                    <div class="value pos">${pct(r.investment_rate_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tbill.card.effective">Effective annual</div>
                    <div class="value">${pct(r.effective_annual_yield_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.tbill.row.purchase">Purchase price</td><td>${money(r.purchase_price)}</td></tr>
                    <tr><td data-i18n="view.tbill.row.discount">Discount from face</td><td>${money(r.discount_usd)}</td></tr>
                    <tr><td data-i18n="view.tbill.row.bankdiscount">Bank-discount rate</td><td>${pct(r.bank_discount_rate_pct)}</td></tr>
                    <tr><td data-i18n="view.tbill.row.moneymarket">Money-market yield</td><td>${pct(r.money_market_yield_pct)}</td></tr>
                    <tr><td data-i18n="view.tbill.row.investment">Coupon-equivalent (investment rate)</td><td>${pct(r.investment_rate_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.tbill.row.effective">Effective annual yield</td><td>${pct(r.effective_annual_yield_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
