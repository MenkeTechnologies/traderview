// Altman Z-Score — five-ratio bankruptcy-distress model with safe/grey/distress
// zone, via /calc/altman-z-score. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
const num = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 });
const VIEW = 'altman-z-score';
let lastReport = null;
let lastBody = null;

const ZONE = {
    safe: { key: 'view.altman.zone.safe', cls: 'pos' },
    grey: { key: 'view.altman.zone.grey', cls: '' },
    distress: { key: 'view.altman.zone.distress', cls: 'neg' },
};

export async function renderAltmanZScore(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.altman.h1.title">// ALTMAN Z-SCORE</span></h1>
        <p class="muted small" data-i18n="view.altman.hint.intro">
            Edward Altman's bankruptcy-distress model for public manufacturers — five balance-sheet
            and income ratios weighted into one score. Above 2.99 is the safe zone, 1.81–2.99 is
            grey, and below 1.81 signals elevated distress risk. All figures in the same units.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.altman.h2.inputs">The financials</h2>
            <form id="altman-form" class="inline-form">
                <label><span data-i18n="view.altman.label.ca">Current assets</span>
                    <input type="number" step="0.01" name="current_assets_usd" value="500" required></label>
                <label><span data-i18n="view.altman.label.cl">Current liabilities</span>
                    <input type="number" step="0.01" name="current_liabilities_usd" value="200" required></label>
                <label><span data-i18n="view.altman.label.re">Retained earnings</span>
                    <input type="number" step="0.01" name="retained_earnings_usd" value="400" required></label>
                <label><span data-i18n="view.altman.label.ebit">EBIT</span>
                    <input type="number" step="0.01" name="ebit_usd" value="150" required></label>
                <label><span data-i18n="view.altman.label.mve">Market value of equity</span>
                    <input type="number" step="0.01" name="market_value_equity_usd" value="600" required></label>
                <label><span data-i18n="view.altman.label.tl">Total liabilities</span>
                    <input type="number" step="0.01" name="total_liabilities_usd" value="400" required></label>
                <label><span data-i18n="view.altman.label.sales">Sales</span>
                    <input type="number" step="0.01" name="sales_usd" value="1200" required></label>
                <label><span data-i18n="view.altman.label.ta">Total assets</span>
                    <input type="number" step="0.01" name="total_assets_usd" value="1000" required></label>
            </form>
            <div id="altman-tools" class="ce-toolbar"></div>
        </div>
        <div id="altman-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#altman-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            current_assets_usd: Number(fd.get('current_assets_usd')) || 0,
            current_liabilities_usd: Number(fd.get('current_liabilities_usd')) || 0,
            retained_earnings_usd: Number(fd.get('retained_earnings_usd')) || 0,
            ebit_usd: Number(fd.get('ebit_usd')) || 0,
            market_value_equity_usd: Number(fd.get('market_value_equity_usd')) || 0,
            total_liabilities_usd: Number(fd.get('total_liabilities_usd')) || 0,
            sales_usd: Number(fd.get('sales_usd')) || 0,
            total_assets_usd: Number(fd.get('total_assets_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcAltmanZScore(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.altman.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#altman-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'altman-z-score.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['z_score', r.z_score],
        ['zone', r.zone],
        ['x1_working_capital', r.x1_working_capital],
        ['x2_retained_earnings', r.x2_retained_earnings],
        ['x3_ebit', r.x3_ebit],
        ['x4_equity_to_liabilities', r.x4_equity_to_liabilities],
        ['x5_sales', r.x5_sales],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#altman-result');
    const z = ZONE[r.zone] || ZONE.distress;
    // The five ratios that drive the Z-score.
    const chart = enh.svgBarChart([
        { label: 'X1 WC', value: r.x1_working_capital },
        { label: 'X2 RE', value: r.x2_retained_earnings },
        { label: 'X3 EBIT', value: r.x3_ebit },
        { label: 'X4 Eq/L', value: r.x4_equity_to_liabilities },
        { label: 'X5 Sales', value: r.x5_sales },
    ]);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.altman.h2.result">The score</h2>
            <div class="cards">
                <div class="card ${z.cls}"><div class="label" data-i18n="view.altman.card.z">Z-Score</div>
                    <div class="value ${z.cls}">${num(r.z_score)}</div></div>
                <div class="card ${z.cls}"><div class="label" data-i18n="view.altman.card.zone">Zone</div>
                    <div class="value ${z.cls}" data-i18n="${z.key}">—</div></div>
                <div class="card"><div class="label" data-i18n="view.altman.card.wc">Working capital</div>
                    <div class="value">${money(r.working_capital_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr><th data-i18n="view.altman.col.ratio">Ratio</th><th data-i18n="view.altman.col.value">Value</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.altman.row.x1">X1 — WC / total assets</td><td>${num(r.x1_working_capital)}</td></tr>
                    <tr><td data-i18n="view.altman.row.x2">X2 — retained earnings / TA</td><td>${num(r.x2_retained_earnings)}</td></tr>
                    <tr><td data-i18n="view.altman.row.x3">X3 — EBIT / TA</td><td>${num(r.x3_ebit)}</td></tr>
                    <tr><td data-i18n="view.altman.row.x4">X4 — equity / liabilities</td><td>${num(r.x4_equity_to_liabilities)}</td></tr>
                    <tr><td data-i18n="view.altman.row.x5">X5 — sales / TA</td><td>${num(r.x5_sales)}</td></tr>
                    <tr class="emph"><td data-i18n="view.altman.row.z">Z-Score</td><td>${num(r.z_score)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
