// Portfolio longevity — years a nest egg lasts under inflation-adjusted
// withdrawals, via /calc/portfolio-longevity. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';

export async function renderPortfolioLongevity(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.longev.h1.title">// PORTFOLIO LONGEVITY</span></h1>
        <p class="muted small" data-i18n="view.longev.hint.intro">
            How long a nest egg lasts under inflation-adjusted withdrawals. Each year the balance
            grows at the return, then the (inflation-grown) withdrawal comes out. If it survives the
            horizon, the draw is sustainable. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.longev.h2.inputs">The plan</h2>
            <form id="longev-form" class="inline-form">
                <label><span data-i18n="view.longev.label.balance">Starting balance ($)</span>
                    <input type="number" step="0.01" min="0" name="starting_balance_usd" value="1000000" required></label>
                <label><span data-i18n="view.longev.label.withdrawal">Annual withdrawal ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_withdrawal_usd" value="80000" required></label>
                <label><span data-i18n="view.longev.label.return">Annual return (%)</span>
                    <input type="number" step="0.1" name="annual_return_pct" value="4" required></label>
                <label><span data-i18n="view.longev.label.inflation">Inflation (%)</span>
                    <input type="number" step="0.1" min="0" name="inflation_pct" value="3"></label>
                <label><span data-i18n="view.longev.label.cap">Horizon cap (years)</span>
                    <input type="number" step="1" min="1" name="max_years" value="100"></label>
            </form>
        </div>
        <div id="longev-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#longev-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            starting_balance_usd: Number(fd.get('starting_balance_usd')) || 0,
            annual_withdrawal_usd: Number(fd.get('annual_withdrawal_usd')) || 0,
            annual_return_pct: Number(fd.get('annual_return_pct')) || 0,
            inflation_pct: Number(fd.get('inflation_pct')) || 0,
            max_years: Number(fd.get('max_years')) || 0,
        };
        try {
            const r = await api.calcPortfolioLongevity(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.longev.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#longev-result');
    const cls = r.sustainable ? 'pos' : 'neg';
    const yearsText = r.sustainable
        ? t('view.longev.sustainable')
        : `${r.years_lasted} ${t('view.longev.years')}`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.longev.h2.result">How long it lasts</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.longev.card.years">Years it lasts</div>
                    <div class="value ${cls}">${yearsText}</div></div>
                <div class="card"><div class="label" data-i18n="view.longev.card.rate">Withdrawal rate</div>
                    <div class="value">${pct(r.withdrawal_rate_pct)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.longev.card.final">Final balance</div>
                    <div class="value ${cls}">${money(r.final_balance_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.longev.row.rate">Withdrawal rate</td><td>${pct(r.withdrawal_rate_pct)}</td></tr>
                    <tr><td data-i18n="view.longev.row.years">Full withdrawal years</td><td>${r.years_lasted}</td></tr>
                    <tr><td data-i18n="view.longev.row.sustainable">Sustainable?</td><td>${r.sustainable ? t('view.longev.yes') : t('view.longev.no')}</td></tr>
                    <tr class="emph ${cls}"><td data-i18n="view.longev.row.final">Final balance</td><td>${money(r.final_balance_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
