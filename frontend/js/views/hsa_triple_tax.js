// HSA triple-tax advantage — HSA vs a taxable account over a horizon, the
// dollar value of deductible-in / tax-free-growth / tax-free-out, via
// /calc/hsa-triple-tax. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['annual_contribution_usd', 'Annual contribution ($)', 4000],
    ['years', 'Years', 20],
    ['annual_growth_pct', 'Annual growth (%)', 7],
    ['marginal_tax_rate_pct', 'Marginal tax rate (%)', 24],
    ['ltcg_rate_pct', 'Long-term cap-gains rate (%)', 15],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'hsa-triple-tax';
let lastReport = null;
let lastBody = null;

export async function renderHsaTripleTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.hsatt.h1.title">// HSA TRIPLE-TAX ADVANTAGE</span></h1>
        <p class="muted small" data-i18n="view.hsatt.hint.intro">
            The HSA is the only account taxed favorably three ways: contributions are
            deductible, growth is tax-free, and withdrawals for medical expenses are tax-free.
            This projects an HSA against a taxable account funded with the same gross income —
            the taxable side invests only the after-tax amount and pays cap-gains on its growth.
            The gap is the dollar value of the triple-tax treatment. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.hsatt.h2.inputs">The plan</h2>
            <form id="hsa-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.hsatt.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="hsa-tools" class="ce-toolbar"></div>
            <button type="button" id="hsa-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="hsa-sens" class="ce-sens"></div>
        </div>
        <div id="hsa-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#hsa-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        body.years = Math.max(0, Math.round(body.years));
        return body;
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcHsaTripleTax(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.hsatt.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#hsa-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'hsa-triple-tax.csv' });
    mount.querySelector('#hsa-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['hsa_advantage_usd', r.hsa_advantage_usd],
        ['hsa_ending_usd', r.hsa_ending_usd],
        ['taxable_ending_usd', r.taxable_ending_usd],
        ['total_contributions_usd', r.total_contributions_usd],
        ['upfront_tax_savings_usd', r.upfront_tax_savings_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#hsa-result');
    // Line chart: HSA advantage as the horizon sweeps 0 -> 40 years (compounding widens the gap).
    const xs = enh.linspace(0, 40, 13);
    const pts = await Promise.all(xs.map(async (yr) => {
        const rr = await api.calcHsaTripleTax({ ...body, years: Math.round(yr) });
        return { x: yr, y: rr ? rr.hsa_advantage_usd / 1000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'years', ylabel: 'advantage $k' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.hsatt.h2.result">The advantage</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.hsatt.card.advantage">HSA advantage</div>
                    <div class="value pos">${money(r.hsa_advantage_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.hsatt.card.hsa">HSA ending value</div>
                    <div class="value">${money(r.hsa_ending_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.hsatt.card.taxable">Taxable ending value</div>
                    <div class="value">${money(r.taxable_ending_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr><th data-i18n="view.hsatt.col.line">Line</th><th data-i18n="view.hsatt.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.hsatt.row.contributions">Total contributions</td><td>${money(r.total_contributions_usd)}</td></tr>
                    <tr><td data-i18n="view.hsatt.row.upfront">Upfront tax savings (deduction)</td><td>${money(r.upfront_tax_savings_usd)}</td></tr>
                    <tr><td data-i18n="view.hsatt.row.hsa">HSA ending (tax-free)</td><td>${money(r.hsa_ending_usd)}</td></tr>
                    <tr><td data-i18n="view.hsatt.row.taxable">Taxable ending (after cap-gains)</td><td>${money(r.taxable_ending_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.hsatt.row.advantage">Triple-tax advantage</td><td class="pos">${money(r.hsa_advantage_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#hsa-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: years 0 -> 40; y: annual growth 2% -> 12%. Output: HSA advantage.
    const xVals = enh.linspace(0, 40, 5);
    const yVals = enh.linspace(2, 12, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'years', yKey: 'annual_growth_pct', xVals: xVals.map(Math.round), yVals, compute: (b) => api.calcHsaTripleTax(b), pick: (r) => (r ? r.hsa_advantage_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals: xVals.map(Math.round), yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v / 1000) + 'k'), xfmt: (v) => v.toFixed(0) + 'y', yfmt: (v) => v.toFixed(0) + '%', xName: t('view.hsatt.label.years') || 'Years', yName: t('view.hsatt.label.annual_growth_pct') || 'Growth' });
}
