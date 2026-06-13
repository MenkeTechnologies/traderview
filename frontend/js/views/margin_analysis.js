// Income-statement margin waterfall — gross / operating / pre-tax / net profit
// and margins, via /calc/margin-analysis. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');

export async function renderMarginAnalysis(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.margin.h1.title">// MARGIN ANALYSIS</span></h1>
        <p class="muted small" data-i18n="view.margin.hint.intro">
            The income-statement waterfall: revenue down to net income, with the margin (percent of
            revenue) at each level. Gross strips out COGS, operating subtracts overhead, pre-tax
            removes interest, and net is after tax. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.margin.h2.inputs">The income statement</h2>
            <form id="margin-form" class="inline-form">
                <label><span data-i18n="view.margin.label.revenue">Revenue ($)</span>
                    <input type="number" step="0.01" min="0" name="revenue_usd" value="1000" required></label>
                <label><span data-i18n="view.margin.label.cogs">COGS ($)</span>
                    <input type="number" step="0.01" min="0" name="cogs_usd" value="600" required></label>
                <label><span data-i18n="view.margin.label.opex">Operating expenses ($)</span>
                    <input type="number" step="0.01" min="0" name="operating_expenses_usd" value="200"></label>
                <label><span data-i18n="view.margin.label.interest">Interest expense ($)</span>
                    <input type="number" step="0.01" min="0" name="interest_expense_usd" value="50"></label>
                <label><span data-i18n="view.margin.label.tax">Tax rate (%)</span>
                    <input type="number" step="0.1" min="0" name="tax_rate_pct" value="21"></label>
            </form>
        </div>
        <div id="margin-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#margin-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
            cogs_usd: Number(fd.get('cogs_usd')) || 0,
            operating_expenses_usd: Number(fd.get('operating_expenses_usd')) || 0,
            interest_expense_usd: Number(fd.get('interest_expense_usd')) || 0,
            tax_rate_pct: Number(fd.get('tax_rate_pct')) || 0,
        };
        try {
            const r = await api.calcMarginAnalysis(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.margin.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#margin-result');
    const netCls = r.net_income_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.margin.h2.result">The margins</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.margin.card.gross">Gross margin</div>
                    <div class="value">${pct(r.gross_margin_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.margin.card.operating">Operating margin</div>
                    <div class="value">${pct(r.operating_margin_pct)}</div></div>
                <div class="card ${netCls}"><div class="label" data-i18n="view.margin.card.net">Net margin</div>
                    <div class="value ${netCls}">${pct(r.net_margin_pct)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.margin.col.line">Line</th><th data-i18n="view.margin.col.amount">Amount</th><th data-i18n="view.margin.col.margin">Margin</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.margin.row.gross">Gross profit</td><td>${money(r.gross_profit_usd)}</td><td>${pct(r.gross_margin_pct)}</td></tr>
                    <tr><td data-i18n="view.margin.row.operating">Operating income</td><td>${money(r.operating_income_usd)}</td><td>${pct(r.operating_margin_pct)}</td></tr>
                    <tr><td data-i18n="view.margin.row.pretax">Pre-tax income</td><td>${money(r.pretax_income_usd)}</td><td>${pct(r.pretax_margin_pct)}</td></tr>
                    <tr><td data-i18n="view.margin.row.tax">Tax</td><td>${money(r.tax_usd)}</td><td>—</td></tr>
                    <tr class="emph ${netCls}"><td data-i18n="view.margin.row.net">Net income</td><td>${money(r.net_income_usd)}</td><td>${pct(r.net_margin_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
