// Federal estate tax — taxable estate above the unified exclusion, via
// /calc/estate-tax. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const VIEW = 'estate-tax';
let lastReport = null;
let lastBody = null;

export async function renderEstateTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.estate.h1.title">// FEDERAL ESTATE TAX</span></h1>
        <p class="muted small" data-i18n="view.estate.hint.intro">
            Gross estate less debts, the unlimited marital deduction, and charitable bequests gives the
            taxable estate; lifetime gifts are added back, and the amount above the exclusion is taxed at
            the top rate. 2026 defaults: $15M per person ($30M for a couple via portability), 40% rate.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.estate.h2.inputs">The estate</h2>
            <form id="estate-form" class="inline-form">
                <label><span data-i18n="view.estate.label.gross">Gross estate ($)</span>
                    <input type="number" step="1000" min="0" name="gross_estate_usd" value="20000000" required></label>
                <label><span data-i18n="view.estate.label.debts">Debts &amp; expenses ($)</span>
                    <input type="number" step="1000" min="0" name="debts_expenses_usd" value="0"></label>
                <label><span data-i18n="view.estate.label.marital">Marital deduction ($)</span>
                    <input type="number" step="1000" min="0" name="marital_deduction_usd" value="0"></label>
                <label><span data-i18n="view.estate.label.charitable">Charitable bequests ($)</span>
                    <input type="number" step="1000" min="0" name="charitable_deduction_usd" value="0"></label>
                <label><span data-i18n="view.estate.label.gifts">Adjusted lifetime gifts ($)</span>
                    <input type="number" step="1000" min="0" name="lifetime_gifts_usd" value="0"></label>
                <label><span data-i18n="view.estate.label.exemption">Basic exclusion ($)</span>
                    <input type="number" step="1000" min="0" name="exemption_usd" value="15000000"></label>
                <label><span data-i18n="view.estate.label.dsue">Ported DSUE from spouse ($)</span>
                    <input type="number" step="1000" min="0" name="dsue_usd" value="0"></label>
                <label><span data-i18n="view.estate.label.rate">Top rate (%)</span>
                    <input type="number" step="0.1" min="0" name="rate_pct" value="40"></label>
            </form>
            <div id="estate-tools" class="ce-toolbar"></div>
            <button type="button" id="estate-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="estate-sens" class="ce-sens"></div>
        </div>
        <div id="estate-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#estate-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            gross_estate_usd: Number(fd.get('gross_estate_usd')) || 0,
            debts_expenses_usd: Number(fd.get('debts_expenses_usd')) || 0,
            marital_deduction_usd: Number(fd.get('marital_deduction_usd')) || 0,
            charitable_deduction_usd: Number(fd.get('charitable_deduction_usd')) || 0,
            lifetime_gifts_usd: Number(fd.get('lifetime_gifts_usd')) || 0,
            exemption_usd: Number(fd.get('exemption_usd')) || 0,
            dsue_usd: Number(fd.get('dsue_usd')) || 0,
            rate_pct: Number(fd.get('rate_pct')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcEstateTax(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.estate.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#estate-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'estate-tax.csv' });
    mount.querySelector('#estate-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['estate_tax_usd', r.estate_tax_usd],
        ['net_to_heirs_usd', r.net_to_heirs_usd],
        ['effective_rate_pct', r.effective_rate_pct],
        ['taxable_estate_usd', r.taxable_estate_usd],
        ['amount_taxed_usd', r.amount_taxed_usd],
        ['total_exemption_usd', r.total_exemption_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#estate-result');
    const taxCls = r.is_taxable ? 'neg' : 'pos';
    // Line chart: estate tax as gross estate sweeps 0 -> $40M (zero below the exclusion, then ramps).
    const xs = enh.linspace(0, 40000000, 16);
    const pts = await Promise.all(xs.map(async (g) => {
        const rr = await api.calcEstateTax({ ...body, gross_estate_usd: g });
        return { x: g / 1000000, y: rr ? rr.estate_tax_usd / 1000000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'estate $M', ylabel: 'tax $M' });
    const statusRow = r.is_taxable
        ? `<tr class="neg"><td data-i18n="view.estate.row.status">Status</td><td data-i18n="view.estate.status.taxable">Estate tax due</td></tr>`
        : `<tr class="pos"><td data-i18n="view.estate.row.status">Status</td><td data-i18n="view.estate.status.exempt">Under exclusion — no tax</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.estate.h2.result">The tax</h2>
            <div class="cards">
                <div class="card ${taxCls}"><div class="label" data-i18n="view.estate.card.tax">Estate tax</div>
                    <div class="value ${taxCls}">${money(r.estate_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.estate.card.heirs">Net to heirs</div>
                    <div class="value">${money(r.net_to_heirs_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.estate.card.effrate">Effective rate</div>
                    <div class="value">${pct(r.effective_rate_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.estate.row.taxable">Taxable estate</td><td>${money(r.taxable_estate_usd)}</td></tr>
                    <tr><td data-i18n="view.estate.row.base">Tax base (+ lifetime gifts)</td><td>${money(r.estate_tax_base_usd)}</td></tr>
                    <tr><td data-i18n="view.estate.row.exemption">Total exclusion</td><td>${money(r.total_exemption_usd)}</td></tr>
                    <tr><td data-i18n="view.estate.row.used">Exclusion used</td><td>${money(r.exemption_used_usd)}</td></tr>
                    <tr><td data-i18n="view.estate.row.remaining">Exclusion remaining</td><td>${money(r.exemption_remaining_usd)}</td></tr>
                    <tr><td data-i18n="view.estate.row.taxed">Amount taxed</td><td>${money(r.amount_taxed_usd)}</td></tr>
                    ${statusRow}
                    <tr class="emph"><td data-i18n="view.estate.row.tax">Estate tax</td><td>${money(r.estate_tax_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#estate-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: gross estate 0 -> $40M; y: top rate 18% -> 45%. Output: estate tax.
    const xVals = enh.linspace(0, 40000000, 5);
    const yVals = enh.linspace(18, 45, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'gross_estate_usd', yKey: 'rate_pct', xVals, yVals, compute: (b) => api.calcEstateTax(b), pick: (r) => (r ? r.estate_tax_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + (v / 1000000).toFixed(1) + 'M'), xfmt: (v) => '$' + (v / 1000000).toFixed(0) + 'M', yfmt: (v) => v.toFixed(0) + '%', xName: t('view.estate.label.gross') || 'Estate', yName: t('view.estate.label.rate') || 'Rate' });
}
