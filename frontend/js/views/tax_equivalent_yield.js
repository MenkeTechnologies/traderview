// Tax-equivalent yield — muni vs taxable bond on an after-tax basis, via
// /calc/tax-equivalent-yield. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');

const VERDICT = { muni: 'view.tey.verdict.muni', taxable: 'view.tey.verdict.taxable', equal: 'view.tey.verdict.equal' };
const VIEW = 'tax-equivalent-yield';
let lastReport = null;
let lastBody = null;

export async function renderTaxEquivalentYield(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tey.h1.title">// TAX-EQUIVALENT YIELD</span></h1>
        <p class="muted small" data-i18n="view.tey.hint.intro">
            Compares a tax-free municipal bond to a taxable bond after tax. Muni interest is free of
            federal tax (and the 3.8% NIIT), and state tax too when issued in your state. The
            tax-equivalent yield is what a taxable bond would have to yield to match the muni after
            tax. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.tey.h2.inputs">The bonds</h2>
            <form id="tey-form" class="inline-form">
                <label><span data-i18n="view.tey.label.muni">Muni yield (%)</span>
                    <input type="number" step="0.01" min="0" name="muni_yield_pct" value="3.5" required></label>
                <label><span data-i18n="view.tey.label.taxable">Taxable yield to compare (%)</span>
                    <input type="number" step="0.01" min="0" name="taxable_yield_pct" value="5"></label>
                <label><span data-i18n="view.tey.label.federal">Federal rate (%)</span>
                    <input type="number" step="0.1" min="0" name="federal_rate_pct" value="32" required></label>
                <label><span data-i18n="view.tey.label.state">State rate (%)</span>
                    <input type="number" step="0.1" min="0" name="state_rate_pct" value="0"></label>
                <label><span data-i18n="view.tey.label.instate">Muni issued in your state</span>
                    <input type="checkbox" name="in_state"></label>
                <label><span data-i18n="view.tey.label.niit">NIIT (3.8%) applies</span>
                    <input type="checkbox" name="niit_applies"></label>
            </form>
            <div id="tey-tools" class="ce-toolbar"></div>
            <button type="button" id="tey-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="tey-sens" class="ce-sens"></div>
        </div>
        <div id="tey-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#tey-form');
    const hashIn = enh.readHashInputs();
    enh.prefillForm(form, hashIn);
    // prefillForm sets .value; checkboxes need .checked restored explicitly.
    if ('in_state' in hashIn) form.querySelector('[name=in_state]').checked = hashIn.in_state === 'true';
    if ('niit_applies' in hashIn) form.querySelector('[name=niit_applies]').checked = hashIn.niit_applies === 'true';
    const readBody = () => {
        const fd = new FormData(form);
        return {
            muni_yield_pct: Number(fd.get('muni_yield_pct')) || 0,
            taxable_yield_pct: Number(fd.get('taxable_yield_pct')) || 0,
            federal_rate_pct: Number(fd.get('federal_rate_pct')) || 0,
            state_rate_pct: Number(fd.get('state_rate_pct')) || 0,
            in_state: form.querySelector('[name=in_state]').checked,
            niit_applies: form.querySelector('[name=niit_applies]').checked,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcTaxEquivalentYield(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.tey.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#tey-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'tax-equivalent-yield.csv' });
    mount.querySelector('#tey-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['tax_equivalent_yield_pct', r.tax_equivalent_yield_pct],
        ['muni_after_tax_pct', r.muni_after_tax_pct],
        ['combined_taxable_rate_pct', r.combined_taxable_rate_pct],
        ['taxable_after_tax_pct', r.taxable_after_tax_pct == null ? '' : r.taxable_after_tax_pct],
        ['verdict', r.verdict || ''],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#tey-result');
    const winCls = r.verdict === 'muni' ? 'pos' : (r.verdict === 'taxable' ? 'neg' : '');
    // Line chart: tax-equivalent yield as the federal rate sweeps 10% → 40% (rises with the bracket).
    const xs = enh.linspace(10, 40, 13);
    const pts = await Promise.all(xs.map(async (f) => {
        const rr = await api.calcTaxEquivalentYield({ ...body, federal_rate_pct: f });
        return { x: f, y: rr && rr.tax_equivalent_yield_pct != null ? rr.tax_equivalent_yield_pct : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'fed %', ylabel: 'TEY %' });
    const verdictRow = r.verdict
        ? `<tr class="${winCls}"><td data-i18n="view.tey.row.verdict">Better after tax</td><td data-i18n="${VERDICT[r.verdict]}">—</td></tr>`
        : '';
    const taxableRow = r.taxable_after_tax_pct == null ? '' :
        `<tr><td data-i18n="view.tey.row.taxableat">Taxable after-tax</td><td>${pct(r.taxable_after_tax_pct)}</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tey.h2.result">After tax</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.tey.card.tey">Tax-equivalent yield</div>
                    <div class="value pos">${pct(r.tax_equivalent_yield_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tey.card.muniat">Muni after-tax</div>
                    <div class="value">${pct(r.muni_after_tax_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tey.card.rate">Combined tax rate</div>
                    <div class="value">${pct(r.combined_taxable_rate_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.tey.row.muniat">Muni after-tax</td><td>${pct(r.muni_after_tax_pct)}</td></tr>
                    ${taxableRow}
                    <tr><td data-i18n="view.tey.row.rate">Combined taxable rate</td><td>${pct(r.combined_taxable_rate_pct)}</td></tr>
                    ${verdictRow}
                    <tr class="emph"><td data-i18n="view.tey.row.tey">Tax-equivalent yield</td><td>${pct(r.tax_equivalent_yield_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#tey-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: federal rate 10% → 40%; y: muni yield 1% → 6%. Output: tax-equivalent yield.
    const xVals = enh.linspace(10, 40, 5);
    const yVals = enh.linspace(1, 6, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'federal_rate_pct', yKey: 'muni_yield_pct', xVals, yVals, compute: (b) => api.calcTaxEquivalentYield(b), pick: (r) => (r ? r.tax_equivalent_yield_pct : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : v.toFixed(2) + '%'), xfmt: (v) => v.toFixed(0) + '%', yfmt: (v) => v.toFixed(1) + '%', xName: t('view.tey.label.federal') || 'Fed', yName: t('view.tey.label.muni') || 'Muni' });
}
