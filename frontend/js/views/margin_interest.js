// Margin interest — carry cost of a margin loan and the break-even return
// needed to cover it, via /calc/margin-interest. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');
const mult = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '×');

export async function renderMarginInterest(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.margint.h1.title">// MARGIN INTEREST</span></h1>
        <p class="muted small" data-i18n="view.margint.hint.intro">
            Borrowing on margin accrues interest daily on a 360-day basis. See the daily, period,
            and annual carry, your leverage, and the total-position return you need just to cover
            the interest. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.margint.h2.inputs">The loan</h2>
            <form id="margint-form" class="inline-form">
                <label><span data-i18n="view.margint.label.own">Your cash ($)</span>
                    <input type="number" step="0.01" min="0" name="own_cash_usd" value="10000" required></label>
                <label><span data-i18n="view.margint.label.borrowed">Borrowed ($)</span>
                    <input type="number" step="0.01" min="0" name="borrowed_amount_usd" value="10000" required></label>
                <label><span data-i18n="view.margint.label.rate">Margin rate (%)</span>
                    <input type="number" step="0.01" min="0" name="margin_rate_pct" value="8" required></label>
                <label><span data-i18n="view.margint.label.days">Days held</span>
                    <input type="number" step="1" min="0" name="days_held" value="30" required></label>
                <label><span data-i18n="view.margint.label.daycount">Day-count basis</span>
                    <input type="number" step="1" min="1" name="day_count" value="360"></label>
            </form>
        </div>
        <div id="margint-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#margint-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            own_cash_usd: Number(fd.get('own_cash_usd')) || 0,
            borrowed_amount_usd: Number(fd.get('borrowed_amount_usd')) || 0,
            margin_rate_pct: Number(fd.get('margin_rate_pct')) || 0,
            days_held: Number(fd.get('days_held')) || 0,
            day_count: Number(fd.get('day_count')) || 360,
        };
        try {
            const r = await api.calcMarginInterest(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.margint.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#margint-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.margint.h2.result">The carry</h2>
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.margint.card.period">Interest this period</div>
                    <div class="value neg">${money(r.interest_cost_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.margint.card.leverage">Leverage</div>
                    <div class="value">${mult(r.leverage)}</div></div>
                <div class="card"><div class="label" data-i18n="view.margint.card.breakeven">Break-even return</div>
                    <div class="value">${pct(r.breakeven_return_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.margint.row.position">Position value</td><td>${money(r.position_value_usd)}</td></tr>
                    <tr><td data-i18n="view.margint.row.leverage">Leverage</td><td>${mult(r.leverage)}</td></tr>
                    <tr><td data-i18n="view.margint.row.daily">Daily interest</td><td>${money(r.daily_interest_usd)}</td></tr>
                    <tr><td data-i18n="view.margint.row.period">Interest this period</td><td>${money(r.interest_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.margint.row.annual">Annual interest</td><td>${money(r.annual_interest_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.margint.row.breakeven">Break-even return</td><td>${pct(r.breakeven_return_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
