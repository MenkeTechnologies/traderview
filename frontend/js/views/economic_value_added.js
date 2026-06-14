// Economic Value Added (EVA) — economic profit above the capital charge, via
// /calc/economic-value-added. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => (n == null ? '—' : (n < 0 ? '−$' : '$') + Math.abs(Number(n)).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const VIEW = 'economic-value-added';
let lastReport = null;
let lastBody = null;

export async function renderEconomicValueAdded(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.eva.h1.title">// ECONOMIC VALUE ADDED</span></h1>
        <p class="muted small" data-i18n="view.eva.hint.intro">
            EVA is the after-tax operating profit a business earns above the cost of the capital it
            employs: NOPAT (EBIT after tax) minus invested capital times WACC. Positive EVA creates
            value; negative destroys it even when accounting profit is positive. Equivalently,
            (ROIC − WACC) × invested capital. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.eva.h2.inputs">The business</h2>
            <form id="eva-form" class="inline-form">
                <label><span data-i18n="view.eva.label.ebit">EBIT ($)</span>
                    <input type="number" step="1000" name="ebit_usd" value="1000000" required></label>
                <label><span data-i18n="view.eva.label.tax">Tax rate (%)</span>
                    <input type="number" step="0.1" min="0" name="tax_rate_pct" value="25" required></label>
                <label><span data-i18n="view.eva.label.capital">Invested capital ($)</span>
                    <input type="number" step="1000" min="0" name="invested_capital_usd" value="5000000" required></label>
                <label><span data-i18n="view.eva.label.wacc">WACC (%)</span>
                    <input type="number" step="0.1" min="0" name="wacc_pct" value="8" required></label>
                <label><span data-i18n="view.eva.label.revenue">Revenue ($, optional)</span>
                    <input type="number" step="1000" min="0" name="revenue_usd" value="4000000"></label>
            </form>
        </div>
        <div id="eva-tools" class="ce-toolbar"></div>
        <div id="eva-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#eva-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            ebit_usd: Number(fd.get('ebit_usd')) || 0,
            tax_rate_pct: Number(fd.get('tax_rate_pct')) || 0,
            invested_capital_usd: Number(fd.get('invested_capital_usd')) || 0,
            wacc_pct: Number(fd.get('wacc_pct')) || 0,
            revenue_usd: Number(fd.get('revenue_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcEconomicValueAdded(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.eva.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#eva-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'economic-value-added.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['eva_usd', r.eva_usd],
        ['nopat_usd', r.nopat_usd],
        ['capital_charge_usd', r.capital_charge_usd],
        ['roic_pct', r.roic_pct],
        ['eva_spread_pct', r.eva_spread_pct],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#eva-result');
    const cls = r.creates_value ? 'pos' : 'neg';
    // EVA decomposition: NOPAT minus capital charge equals economic value added.
    const chart = enh.svgBarChart([
        { label: 'NOPAT', value: r.nopat_usd },
        { label: 'Cap charge', value: -r.capital_charge_usd },
        { label: 'EVA', value: r.eva_usd },
    ]);
    const verdictKey = r.creates_value ? 'view.eva.verdict.creates' : 'view.eva.verdict.destroys';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.eva.h2.result">The economic profit</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="${verdictKey}">—</div>
                    <div class="value ${cls}">${money(r.eva_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.eva.card.roic">ROIC</div>
                    <div class="value">${pct(r.roic_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.eva.card.spread">EVA spread</div>
                    <div class="value">${pct(r.eva_spread_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.eva.row.nopat">NOPAT (EBIT after tax)</td><td>${money(r.nopat_usd)}</td></tr>
                    <tr><td data-i18n="view.eva.row.charge">Capital charge (capital × WACC)</td><td>${money(r.capital_charge_usd)}</td></tr>
                    <tr><td data-i18n="view.eva.row.roic">ROIC (NOPAT / capital)</td><td>${pct(r.roic_pct)}</td></tr>
                    <tr><td data-i18n="view.eva.row.spread">EVA spread (ROIC − WACC)</td><td>${pct(r.eva_spread_pct)}</td></tr>
                    <tr><td data-i18n="view.eva.row.margin">EVA margin</td><td>${pct(r.eva_margin_pct)}</td></tr>
                    <tr class="emph ${cls}"><td data-i18n="view.eva.row.eva">Economic Value Added</td><td>${money(r.eva_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
