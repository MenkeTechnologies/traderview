// Degree of leverage — operating (DOL), financial (DFL), and combined (DCL),
// via /calc/leverage. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const x = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '×');

export async function renderLeverage(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.leverage.h1.title">// DEGREE OF LEVERAGE</span></h1>
        <p class="muted small" data-i18n="view.leverage.hint.intro">
            How a change in sales is amplified down the income statement. Fixed operating costs
            amplify it into EBIT (operating leverage); fixed interest amplifies it again into EPS
            (financial leverage). A DCL of 3 means a 1% change in sales swings EPS 3%. Updates as
            you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.leverage.h2.inputs">The income statement</h2>
            <form id="leverage-form" class="inline-form">
                <label><span data-i18n="view.leverage.label.sales">Sales ($)</span>
                    <input type="number" step="0.01" name="sales_usd" value="1000" required></label>
                <label><span data-i18n="view.leverage.label.variable">Variable costs ($)</span>
                    <input type="number" step="0.01" min="0" name="variable_costs_usd" value="400" required></label>
                <label><span data-i18n="view.leverage.label.fixed">Fixed costs ($)</span>
                    <input type="number" step="0.01" min="0" name="fixed_costs_usd" value="300" required></label>
                <label><span data-i18n="view.leverage.label.interest">Interest expense ($)</span>
                    <input type="number" step="0.01" min="0" name="interest_expense_usd" value="100"></label>
            </form>
        </div>
        <div id="leverage-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#leverage-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            sales_usd: Number(fd.get('sales_usd')) || 0,
            variable_costs_usd: Number(fd.get('variable_costs_usd')) || 0,
            fixed_costs_usd: Number(fd.get('fixed_costs_usd')) || 0,
            interest_expense_usd: Number(fd.get('interest_expense_usd')) || 0,
        };
        try {
            const r = await api.calcLeverage(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.leverage.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#leverage-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.leverage.h2.result">The amplification</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.leverage.card.dol">Operating (DOL)</div>
                    <div class="value">${x(r.dol)}</div></div>
                <div class="card"><div class="label" data-i18n="view.leverage.card.dfl">Financial (DFL)</div>
                    <div class="value">${x(r.dfl)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.leverage.card.dcl">Combined (DCL)</div>
                    <div class="value pos">${x(r.dcl)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.leverage.row.cm">Contribution margin</td><td>${money(r.contribution_margin_usd)}</td></tr>
                    <tr><td data-i18n="view.leverage.row.ebit">EBIT</td><td>${money(r.ebit_usd)}</td></tr>
                    <tr><td data-i18n="view.leverage.row.pretax">Pre-tax income</td><td>${money(r.pretax_income_usd)}</td></tr>
                    <tr><td data-i18n="view.leverage.row.dol">Operating leverage (DOL)</td><td>${x(r.dol)}</td></tr>
                    <tr><td data-i18n="view.leverage.row.dfl">Financial leverage (DFL)</td><td>${x(r.dfl)}</td></tr>
                    <tr class="emph"><td data-i18n="view.leverage.row.dcl">Combined leverage (DCL)</td><td>${x(r.dcl)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
