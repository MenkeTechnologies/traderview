// Free cash flow — FCF, FCF margin, FCF yield, and cash-conversion quality,
// via /calc/free-cash-flow. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const ratio = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '×');

export async function renderFreeCashFlow(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fcf.h1.title">// FREE CASH FLOW</span></h1>
        <p class="muted small" data-i18n="view.fcf.hint.intro">
            The cash a business throws off after maintaining and growing its asset base — what's
            actually available for dividends, buybacks, and debt paydown. FCF = operating cash flow −
            capex. The margin, yield, and FCF-to-net-income (cash-conversion quality, above 1 means
            earnings are backed by cash) round it out. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.fcf.h2.inputs">The company</h2>
            <form id="fcf-form" class="inline-form">
                <label><span data-i18n="view.fcf.label.ocf">Operating cash flow ($)</span>
                    <input type="number" step="0.01" name="operating_cash_flow_usd" value="500" required></label>
                <label><span data-i18n="view.fcf.label.capex">Capital expenditures ($)</span>
                    <input type="number" step="0.01" min="0" name="capital_expenditures_usd" value="150" required></label>
                <label><span data-i18n="view.fcf.label.revenue">Revenue ($)</span>
                    <input type="number" step="0.01" min="0" name="revenue_usd" value="2000"></label>
                <label><span data-i18n="view.fcf.label.mcap">Market cap ($)</span>
                    <input type="number" step="0.01" min="0" name="market_cap_usd" value="5000"></label>
                <label><span data-i18n="view.fcf.label.ni">Net income ($)</span>
                    <input type="number" step="0.01" name="net_income_usd" value="400"></label>
            </form>
        </div>
        <div id="fcf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#fcf-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            operating_cash_flow_usd: Number(fd.get('operating_cash_flow_usd')) || 0,
            capital_expenditures_usd: Number(fd.get('capital_expenditures_usd')) || 0,
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
            market_cap_usd: Number(fd.get('market_cap_usd')) || 0,
            net_income_usd: Number(fd.get('net_income_usd')) || 0,
        };
        try {
            const r = await api.calcFreeCashFlow(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.fcf.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#fcf-result');
    const fcfClass = r.free_cash_flow_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.fcf.h2.result">The cash flow</h2>
            <div class="cards">
                <div class="card ${fcfClass}"><div class="label" data-i18n="view.fcf.card.fcf">Free cash flow</div>
                    <div class="value ${fcfClass}">${money(r.free_cash_flow_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.fcf.card.yield">FCF yield</div>
                    <div class="value">${pct(r.fcf_yield_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.fcf.card.margin">FCF margin</div>
                    <div class="value">${pct(r.fcf_margin_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.fcf.row.fcf">Free cash flow</td><td>${money(r.free_cash_flow_usd)}</td></tr>
                    <tr><td data-i18n="view.fcf.row.margin">FCF margin</td><td>${pct(r.fcf_margin_pct)}</td></tr>
                    <tr><td data-i18n="view.fcf.row.yield">FCF yield</td><td>${pct(r.fcf_yield_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.fcf.row.quality">FCF / net income</td><td>${ratio(r.fcf_to_net_income)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
