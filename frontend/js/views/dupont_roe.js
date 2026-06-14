// DuPont ROE decomposition (5-step) — splits return on equity into the
// operating, efficiency, and leverage levers, via /calc/dupont-roe. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const VIEW = 'dupont-roe';
let lastReport = null;
let lastBody = null;

export async function renderDupontRoe(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dupont.h1.title">// DUPONT ROE</span></h1>
        <p class="muted small" data-i18n="view.dupont.hint.intro">
            Decomposes return on equity into the levers that drive it: tax burden × interest burden ×
            operating margin × asset turnover × equity multiplier. The first three collapse to net
            profit margin, so the picture is margin × turnover × leverage — showing whether ROE comes
            from profitability, efficiency, or debt. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.dupont.h2.inputs">The financials</h2>
            <form id="dupont-form" class="inline-form">
                <label><span data-i18n="view.dupont.label.ni">Net income ($)</span>
                    <input type="number" step="0.01" name="net_income_usd" value="120" required></label>
                <label><span data-i18n="view.dupont.label.ebt">Pre-tax income / EBT ($)</span>
                    <input type="number" step="0.01" name="pretax_income_usd" value="150" required></label>
                <label><span data-i18n="view.dupont.label.ebit">EBIT ($)</span>
                    <input type="number" step="0.01" name="ebit_usd" value="200" required></label>
                <label><span data-i18n="view.dupont.label.rev">Revenue ($)</span>
                    <input type="number" step="0.01" name="revenue_usd" value="1000" required></label>
                <label><span data-i18n="view.dupont.label.assets">Total assets ($)</span>
                    <input type="number" step="0.01" name="total_assets_usd" value="800" required></label>
                <label><span data-i18n="view.dupont.label.equity">Total equity ($)</span>
                    <input type="number" step="0.01" name="total_equity_usd" value="400" required></label>
            </form>
            <div id="dupont-tools" class="ce-toolbar"></div>
        </div>
        <div id="dupont-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#dupont-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            net_income_usd: Number(fd.get('net_income_usd')) || 0,
            pretax_income_usd: Number(fd.get('pretax_income_usd')) || 0,
            ebit_usd: Number(fd.get('ebit_usd')) || 0,
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
            total_assets_usd: Number(fd.get('total_assets_usd')) || 0,
            total_equity_usd: Number(fd.get('total_equity_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcDupontRoe(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.dupont.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#dupont-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'dupont-roe.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['factor', 'value'],
        ['tax_burden', r.tax_burden],
        ['interest_burden', r.interest_burden],
        ['operating_margin_pct', r.operating_margin_pct],
        ['asset_turnover', r.asset_turnover],
        ['equity_multiplier', r.equity_multiplier],
        ['net_profit_margin_pct', r.net_profit_margin_pct],
        ['roe_pct', r.roe_pct],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#dupont-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.dupont.h2.result">The decomposition</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.dupont.card.roe">ROE</div>
                    <div class="value pos">${pct(r.roe_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dupont.card.margin">Net margin</div>
                    <div class="value">${pct(r.net_profit_margin_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dupont.card.leverage">Equity multiplier</div>
                    <div class="value">${num(r.equity_multiplier)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.dupont.col.factor">Factor</th><th data-i18n="view.dupont.col.value">Value</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.dupont.row.taxburden">Tax burden (NI / EBT)</td><td>${num(r.tax_burden)}</td></tr>
                    <tr><td data-i18n="view.dupont.row.intburden">Interest burden (EBT / EBIT)</td><td>${num(r.interest_burden)}</td></tr>
                    <tr><td data-i18n="view.dupont.row.opmargin">Operating margin (EBIT / rev)</td><td>${pct(r.operating_margin_pct)}</td></tr>
                    <tr><td data-i18n="view.dupont.row.turnover">Asset turnover (rev / assets)</td><td>${num(r.asset_turnover)}</td></tr>
                    <tr><td data-i18n="view.dupont.row.multiplier">Equity multiplier (assets / equity)</td><td>${num(r.equity_multiplier)}</td></tr>
                    <tr class="emph"><td data-i18n="view.dupont.row.roe">ROE</td><td>${pct(r.roe_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
