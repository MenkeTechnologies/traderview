// Preferred stock valuation — fair value (perpetuity) and current yield, via
// /calc/preferred-stock. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');

const VERDICT = { undervalued: 'view.pref.verdict.under', overvalued: 'view.pref.verdict.over', fair: 'view.pref.verdict.fair' };

export async function renderPreferredStock(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pref.h1.title">// PREFERRED STOCK</span></h1>
        <p class="muted small" data-i18n="view.pref.hint.intro">
            A preferred share pays a fixed dividend forever, so it's valued as a perpetuity — fair
            value is the dividend divided by your required yield. Enter a market price to see the
            current yield and whether it's cheap. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pref.h2.inputs">The share</h2>
            <form id="pref-form" class="inline-form">
                <label><span data-i18n="view.pref.label.par">Par value ($)</span>
                    <input type="number" step="0.01" min="0" name="par_value_usd" value="100" required></label>
                <label><span data-i18n="view.pref.label.rate">Dividend rate (%)</span>
                    <input type="number" step="0.01" min="0" name="dividend_rate_pct" value="6" required></label>
                <label><span data-i18n="view.pref.label.yield">Required yield (%)</span>
                    <input type="number" step="0.01" min="0" name="required_yield_pct" value="5" required></label>
                <label><span data-i18n="view.pref.label.price">Market price ($)</span>
                    <input type="number" step="0.01" min="0" name="market_price_usd" value="110"></label>
            </form>
        </div>
        <div id="pref-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pref-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            par_value_usd: Number(fd.get('par_value_usd')) || 0,
            dividend_rate_pct: Number(fd.get('dividend_rate_pct')) || 0,
            required_yield_pct: Number(fd.get('required_yield_pct')) || 0,
            market_price_usd: Number(fd.get('market_price_usd')) || 0,
        };
        try {
            const r = await api.calcPreferredStock(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.pref.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#pref-result');
    const vCls = r.verdict === 'undervalued' ? 'pos' : (r.verdict === 'overvalued' ? 'neg' : '');
    const verdictRow = r.verdict
        ? `<tr class="${vCls}"><td data-i18n="view.pref.row.verdict">Verdict</td><td data-i18n="${VERDICT[r.verdict]}">—</td></tr>`
        : '';
    const yieldRow = r.current_yield_pct == null ? '' :
        `<tr><td data-i18n="view.pref.row.cy">Current yield</td><td>${pct(r.current_yield_pct)}</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pref.h2.result">The valuation</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.pref.card.fair">Fair value</div>
                    <div class="value pos">${money(r.fair_value_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pref.card.dividend">Annual dividend</div>
                    <div class="value">${money(r.annual_dividend_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pref.card.cy">Current yield</div>
                    <div class="value">${pct(r.current_yield_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.pref.row.dividend">Annual dividend</td><td>${money(r.annual_dividend_usd)}</td></tr>
                    ${yieldRow}
                    ${verdictRow}
                    <tr class="emph"><td data-i18n="view.pref.row.fair">Fair value</td><td>${money(r.fair_value_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
