// Enterprise value + EV/EBITDA — capital-structure-neutral valuation multiple,
// with EV/Sales and EBITDA margin, via /calc/ev-ebitda. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const mult = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '×');
const pctv = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%');

export async function renderEvEbitda(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.evebitda.h1.title">// EV / EBITDA</span></h1>
        <p class="muted small" data-i18n="view.evebitda.hint.intro">
            Enterprise value is what it costs to buy the whole business — equity plus the debt you
            assume, less the cash you get. EV/EBITDA is the capital-structure-neutral valuation
            multiple: unlike P/E it ignores how the firm is financed. Add revenue for EV/Sales and
            the EBITDA margin. All figures in the same units (millions or dollars). Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.evebitda.h2.inputs">The company</h2>
            <form id="evebitda-form" class="inline-form">
                <label><span data-i18n="view.evebitda.label.mcap">Market cap</span>
                    <input type="number" step="0.01" min="0" name="market_cap_usd" value="1000" required></label>
                <label><span data-i18n="view.evebitda.label.debt">Total debt</span>
                    <input type="number" step="0.01" min="0" name="total_debt_usd" value="300" required></label>
                <label><span data-i18n="view.evebitda.label.cash">Cash & equivalents</span>
                    <input type="number" step="0.01" min="0" name="cash_usd" value="100" required></label>
                <label><span data-i18n="view.evebitda.label.ebitda">EBITDA</span>
                    <input type="number" step="0.01" name="ebitda_usd" value="150" required></label>
                <fieldset class="inline-fieldset">
                    <legend data-i18n="view.evebitda.legend.optional">Optional</legend>
                    <label><span data-i18n="view.evebitda.label.preferred">Preferred equity</span>
                        <input type="number" step="0.01" min="0" name="preferred_equity_usd" value="0"></label>
                    <label><span data-i18n="view.evebitda.label.minority">Minority interest</span>
                        <input type="number" step="0.01" min="0" name="minority_interest_usd" value="0"></label>
                    <label><span data-i18n="view.evebitda.label.revenue">Revenue</span>
                        <input type="number" step="0.01" min="0" name="revenue_usd" value="0"></label>
                </fieldset>
            </form>
        </div>
        <div id="evebitda-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#evebitda-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            market_cap_usd: Number(fd.get('market_cap_usd')) || 0,
            total_debt_usd: Number(fd.get('total_debt_usd')) || 0,
            cash_usd: Number(fd.get('cash_usd')) || 0,
            ebitda_usd: Number(fd.get('ebitda_usd')) || 0,
            preferred_equity_usd: Number(fd.get('preferred_equity_usd')) || 0,
            minority_interest_usd: Number(fd.get('minority_interest_usd')) || 0,
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
        };
        try {
            const r = await api.calcEvEbitda(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.evebitda.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#evebitda-result');
    const salesRows = r.ev_sales == null ? '' : `
        <tr><td data-i18n="view.evebitda.row.evsales">EV / Sales</td><td>${mult(r.ev_sales)}</td></tr>
        <tr><td data-i18n="view.evebitda.row.margin">EBITDA margin</td><td>${pctv(r.ebitda_margin_pct)}</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.evebitda.h2.result">The multiple</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.evebitda.card.ev">Enterprise value</div>
                    <div class="value pos">${money(r.enterprise_value_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.evebitda.card.multiple">EV / EBITDA</div>
                    <div class="value">${mult(r.ev_ebitda)}</div></div>
                <div class="card"><div class="label" data-i18n="view.evebitda.card.netdebt">Net debt</div>
                    <div class="value">${money(r.net_debt_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.evebitda.row.ev">Enterprise value</td><td>${money(r.enterprise_value_usd)}</td></tr>
                    <tr><td data-i18n="view.evebitda.row.netdebt">Net debt (debt − cash)</td><td>${money(r.net_debt_usd)}</td></tr>
                    ${salesRows}
                    <tr class="emph"><td data-i18n="view.evebitda.row.multiple">EV / EBITDA</td><td>${mult(r.ev_ebitda)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
