// Holding-period return — price + income return on a position, annualized over
// the days held, via /calc/holding-period-return. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const yrs = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });

export async function renderHoldingPeriodReturn(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hpr.h1.title">// HOLDING-PERIOD RETURN</span></h1>
        <p class="muted small" data-i18n="view.hpr.hint.intro">
            Total return on a position, split into the price change and the income (dividends,
            interest, coupons) collected while held, then annualized over the days held. A 25% gain
            over two years annualizes to ~11.8%; the same gain over six months annualizes to ~56%.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.hpr.h2.inputs">The position</h2>
            <form id="hpr-form" class="inline-form">
                <label><span data-i18n="view.hpr.label.buy">Buy price ($)</span>
                    <input type="number" step="0.01" min="0" name="buy_price" value="100" required></label>
                <label><span data-i18n="view.hpr.label.sell">Sell price ($)</span>
                    <input type="number" step="0.01" min="0" name="sell_price" value="120" required></label>
                <label><span data-i18n="view.hpr.label.income">Income / share ($)</span>
                    <input type="number" step="0.01" min="0" name="income_per_share" value="5"></label>
                <label><span data-i18n="view.hpr.label.days">Holding days</span>
                    <input type="number" step="1" min="0" name="holding_days" value="365" required></label>
                <label><span data-i18n="view.hpr.label.shares">Shares (optional)</span>
                    <input type="number" step="0.0001" min="0" name="shares" value="0"></label>
            </form>
        </div>
        <div id="hpr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#hpr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            buy_price: Number(fd.get('buy_price')) || 0,
            sell_price: Number(fd.get('sell_price')) || 0,
            income_per_share: Number(fd.get('income_per_share')) || 0,
            holding_days: Number(fd.get('holding_days')) || 0,
            shares: Number(fd.get('shares')) || 0,
        };
        try {
            const r = await api.calcHoldingPeriodReturn(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.hpr.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#hpr-result');
    const totalClass = r.total_return_pct >= 0 ? 'pos' : 'neg';
    const annClass = r.annualized_return_pct == null ? '' : (r.annualized_return_pct >= 0 ? 'pos' : 'neg');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.hpr.h2.result">The return</h2>
            <div class="cards">
                <div class="card ${totalClass}"><div class="label" data-i18n="view.hpr.card.total">Total return</div>
                    <div class="value ${totalClass}">${pct(r.total_return_pct)}</div></div>
                <div class="card ${annClass}"><div class="label" data-i18n="view.hpr.card.annualized">Annualized</div>
                    <div class="value ${annClass}">${pct(r.annualized_return_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.hpr.card.years">Years held</div>
                    <div class="value">${yrs(r.holding_years)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.hpr.row.price">Price return</td><td>${pct(r.price_return_pct)}</td></tr>
                    <tr><td data-i18n="view.hpr.row.income">Income return</td><td>${pct(r.income_return_pct)}</td></tr>
                    <tr><td data-i18n="view.hpr.row.pl">Total P&L</td><td>${money(r.total_pl_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.hpr.row.total">Total return</td><td>${pct(r.total_return_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.hpr.row.annualized">Annualized return</td><td>${pct(r.annualized_return_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
