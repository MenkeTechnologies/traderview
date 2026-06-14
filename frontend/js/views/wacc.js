// WACC — weighted average cost of capital, with optional CAPM cost of equity,
// via /calc/wacc. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%';
const VIEW = 'wacc';
let lastReport = null;
let lastBody = null;

export async function renderWacc(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.wacc.h1.title">// WACC</span></h1>
        <p class="muted small" data-i18n="view.wacc.hint.intro">
            Weighted average cost of capital — the blended after-tax rate a firm pays its capital
            providers, and the standard discount rate for a DCF. Debt's interest is tax-deductible,
            so its cost is taken after tax. The cost of equity can be entered directly or derived
            from CAPM (risk-free + beta × equity risk premium). Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.wacc.h2.inputs">The capital structure</h2>
            <form id="wacc-form" class="inline-form">
                <label><span data-i18n="view.wacc.label.equity">Market value of equity ($)</span>
                    <input type="number" step="0.01" min="0" name="market_value_equity_usd" value="600" required></label>
                <label><span data-i18n="view.wacc.label.debt">Market value of debt ($)</span>
                    <input type="number" step="0.01" min="0" name="market_value_debt_usd" value="400" required></label>
                <label><span data-i18n="view.wacc.label.rd">Cost of debt, pre-tax (%)</span>
                    <input type="number" step="0.01" min="0" name="cost_of_debt_pct" value="5" required></label>
                <label><span data-i18n="view.wacc.label.tax">Tax rate (%)</span>
                    <input type="number" step="0.1" min="0" name="tax_rate_pct" value="21" required></label>
                <label><span data-i18n="view.wacc.label.usecapm">Derive cost of equity from CAPM</span>
                    <input type="checkbox" name="use_capm"></label>
                <label><span data-i18n="view.wacc.label.re">Cost of equity (%)</span>
                    <input type="number" step="0.01" name="cost_of_equity_pct" value="10"></label>
                <fieldset class="inline-fieldset">
                    <legend data-i18n="view.wacc.legend.capm">CAPM (when enabled)</legend>
                    <label><span data-i18n="view.wacc.label.rf">Risk-free rate (%)</span>
                        <input type="number" step="0.01" name="risk_free_pct" value="4"></label>
                    <label><span data-i18n="view.wacc.label.beta">Beta</span>
                        <input type="number" step="0.01" name="beta" value="1.2"></label>
                    <label><span data-i18n="view.wacc.label.rm">Market return (%)</span>
                        <input type="number" step="0.01" name="market_return_pct" value="10"></label>
                </fieldset>
            </form>
            <div id="wacc-tools" class="ce-toolbar"></div>
        </div>
        <div id="wacc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#wacc-form');
    const hashIn = enh.readHashInputs();
    enh.prefillForm(form, hashIn);
    if ('use_capm' in hashIn) form.querySelector('[name=use_capm]').checked = hashIn.use_capm === 'true';
    const readBody = () => {
        const fd = new FormData(form);
        return {
            market_value_equity_usd: Number(fd.get('market_value_equity_usd')) || 0,
            market_value_debt_usd: Number(fd.get('market_value_debt_usd')) || 0,
            cost_of_equity_pct: Number(fd.get('cost_of_equity_pct')) || 0,
            cost_of_debt_pct: Number(fd.get('cost_of_debt_pct')) || 0,
            tax_rate_pct: Number(fd.get('tax_rate_pct')) || 0,
            use_capm: form.querySelector('[name=use_capm]').checked,
            risk_free_pct: Number(fd.get('risk_free_pct')) || 0,
            beta: Number(fd.get('beta')) || 0,
            market_return_pct: Number(fd.get('market_return_pct')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcWacc(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.wacc.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#wacc-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'wacc.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['wacc_pct', r.wacc_pct],
        ['cost_of_equity_used_pct', r.cost_of_equity_used_pct],
        ['after_tax_cost_of_debt_pct', r.after_tax_cost_of_debt_pct],
        ['weight_equity_pct', r.weight_equity_pct],
        ['weight_debt_pct', r.weight_debt_pct],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#wacc-result');
    // Cost-of-capital components: equity cost, after-tax debt cost, and the blended WACC.
    const chart = enh.svgBarChart([
        { label: 'Equity', value: r.cost_of_equity_used_pct },
        { label: 'Debt', value: r.after_tax_cost_of_debt_pct },
        { label: 'WACC', value: r.wacc_pct },
    ]);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.wacc.h2.result">The cost of capital</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.wacc.card.wacc">WACC</div>
                    <div class="value pos">${pct(r.wacc_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.wacc.card.re">Cost of equity</div>
                    <div class="value">${pct(r.cost_of_equity_used_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.wacc.card.rd">After-tax cost of debt</div>
                    <div class="value">${pct(r.after_tax_cost_of_debt_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.wacc.row.we">Equity weight</td><td>${pct(r.weight_equity_pct)}</td></tr>
                    <tr><td data-i18n="view.wacc.row.wd">Debt weight</td><td>${pct(r.weight_debt_pct)}</td></tr>
                    <tr><td data-i18n="view.wacc.row.re">Cost of equity</td><td>${pct(r.cost_of_equity_used_pct)}</td></tr>
                    <tr><td data-i18n="view.wacc.row.rd">After-tax cost of debt</td><td>${pct(r.after_tax_cost_of_debt_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.wacc.row.wacc">WACC</td><td>${pct(r.wacc_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
