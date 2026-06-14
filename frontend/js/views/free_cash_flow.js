// Free cash flow — FCF, FCF margin, FCF yield, and cash-conversion quality,
// via /calc/free-cash-flow. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const ratio = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '×');
const VIEW = 'free-cash-flow';
let lastReport = null;
let lastBody = null;

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
            <div id="fcf-tools" class="ce-toolbar"></div>
            <button type="button" id="fcf-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="fcf-sens" class="ce-sens"></div>
        </div>
        <div id="fcf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#fcf-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            operating_cash_flow_usd: Number(fd.get('operating_cash_flow_usd')) || 0,
            capital_expenditures_usd: Number(fd.get('capital_expenditures_usd')) || 0,
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
            market_cap_usd: Number(fd.get('market_cap_usd')) || 0,
            net_income_usd: Number(fd.get('net_income_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcFreeCashFlow(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.fcf.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#fcf-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'free-cash-flow.csv' });
    mount.querySelector('#fcf-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['free_cash_flow_usd', r.free_cash_flow_usd],
        ['fcf_margin_pct', r.fcf_margin_pct],
        ['fcf_yield_pct', r.fcf_yield_pct],
        ['fcf_to_net_income', r.fcf_to_net_income],
    ];
}

function renderResult(mount, r, body) {
    const el = mount.querySelector('#fcf-result');
    const fcfClass = r.free_cash_flow_usd >= 0 ? 'pos' : 'neg';
    const b = body || lastBody || {};
    // FCF = operating cash flow minus capex.
    const chart = enh.svgBarChart([
        { label: 'OCF', value: b.operating_cash_flow_usd || 0 },
        { label: 'Capex', value: -(b.capital_expenditures_usd || 0) },
        { label: 'FCF', value: r.free_cash_flow_usd },
    ]);
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
            ${chart}
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

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#fcf-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: capex 0 -> 2x OCF; y: operating cash flow 0.5x -> 1.5x. Output: free cash flow.
    const ocf = base.operating_cash_flow_usd || 500;
    const xVals = enh.linspace(0, ocf * 2, 5);
    const yVals = enh.linspace(ocf * 0.5, ocf * 1.5, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'capital_expenditures_usd', yKey: 'operating_cash_flow_usd', xVals, yVals, compute: (b) => api.calcFreeCashFlow(b), pick: (r) => (r ? r.free_cash_flow_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v)), xfmt: (v) => '$' + Math.round(v), yfmt: (v) => '$' + Math.round(v), xName: t('view.fcf.label.capex') || 'Capex', yName: t('view.fcf.label.ocf') || 'OCF' });
}
