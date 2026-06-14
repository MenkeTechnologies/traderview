// Depreciation recapture — splits a rental sale's gain into unrecaptured
// § 1250 gain (max 25%) and LTCG, with the tax on each, via
// /calc/depreciation-recapture. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const VIEW = 'depreciation-recapture';
let lastReport = null;
let lastBody = null;

const FIELDS = [
    ['purchase_price_usd', 'Purchase price ($)', 200000],
    ['improvements_usd', 'Improvements ($)', 0],
    ['accumulated_depreciation_usd', 'Accumulated depreciation ($)', 50000],
    ['sale_price_usd', 'Sale price ($)', 300000],
    ['selling_costs_usd', 'Selling costs ($)', 0],
    ['ltcg_rate_pct', 'LTCG rate (%)', 15],
    ['recapture_rate_pct', 'Recapture rate (max 25%)', 25],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => Number(n).toFixed(2) + '%';

export async function renderDepreciationRecapture(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dr.h1.title">// DEPRECIATION RECAPTURE</span></h1>
        <p class="muted small" data-i18n="view.dr.hint.intro">
            Depreciation lowers a rental's basis while you hold it; selling claws part of that
            back. For real property, the depreciation portion of the gain is unrecaptured
            § 1250 gain, taxed at a maximum 25% rate; the appreciation above your original
            basis is regular long-term capital gain. Recapture is limited to the lesser of
            depreciation taken or the total gain — a sale at a loss recaptures nothing. NIIT
            (3.8%) may apply on top. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.dr.h2.inputs">The sale</h2>
            <form id="dr-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.dr.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="dr-tools" class="ce-toolbar"></div>
        </div>
        <div id="dr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#dr-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        return body;
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcDepreciationRecapture(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.dr.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#dr-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'depreciation-recapture.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['total_gain_usd', r.total_gain_usd],
        ['unrecaptured_1250_gain_usd', r.unrecaptured_1250_gain_usd],
        ['recapture_tax_usd', r.recapture_tax_usd],
        ['ltcg_gain_usd', r.ltcg_gain_usd],
        ['ltcg_tax_usd', r.ltcg_tax_usd],
        ['total_tax_usd', r.total_tax_usd],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#dr-result');
    if (r.is_loss) {
        el.innerHTML = `<div class="chart-panel"><h2 data-i18n="view.dr.h2.result">The tax</h2>
            <div class="cards"><div class="card"><div class="label" data-i18n="view.dr.card.gain">Total gain</div>
                <div class="value neg">${money(r.total_gain_usd)}</div></div></div>
            <p class="muted small" data-i18n="view.dr.loss">Sale is at a loss — no gain and no depreciation recapture.</p></div>`;
        applyUiI18n(el);
        return;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.dr.h2.result">The tax</h2>
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.dr.card.total_tax">Total tax</div>
                    <div class="value neg">${money(r.total_tax_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dr.card.effective">Effective rate</div>
                    <div class="value">${pct(r.effective_rate_pct)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.dr.card.after_tax">After-tax gain</div>
                    <div class="value pos">${money(r.after_tax_gain_usd)}</div></div>
            </div>
            ${enh.svgBarChart([
                { label: '§1250 tax', value: -r.recapture_tax_usd },
                { label: 'LTCG tax', value: -r.ltcg_tax_usd },
                { label: 'After-tax', value: r.after_tax_gain_usd },
            ])}
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.dr.col.line">Line</th>
                    <th data-i18n="view.dr.col.gain">Gain</th>
                    <th data-i18n="view.dr.col.tax">Tax</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.dr.row.recapture">Unrecaptured § 1250 (≤25%)</td>
                        <td>${money(r.unrecaptured_1250_gain_usd)}</td><td>${money(r.recapture_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.dr.row.ltcg">LTCG (appreciation)</td>
                        <td>${money(r.ltcg_gain_usd)}</td><td>${money(r.ltcg_tax_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.dr.row.total">Total gain</td>
                        <td>${money(r.total_gain_usd)}</td><td>${money(r.total_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.dr.row.basis">Adjusted basis</td>
                        <td colspan="2">${money(r.adjusted_basis_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
