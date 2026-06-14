// Mortgage affordability — max home price under the 28/36 rule, via
// /calc/mortgage-affordability. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const VIEW = 'mortgage-affordability';
let lastReport = null;
let lastBody = null;

export async function renderMortgageAffordability(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.afford.h1.title">// MORTGAGE AFFORDABILITY</span></h1>
        <p class="muted small" data-i18n="view.afford.hint.intro">
            The most house you can buy under the 28/36 rule — lenders cap housing cost (PITI) at 28%
            of gross monthly income and total debt at 36%. The tighter cap sets the budget; since
            taxes and the loan payment both scale with price, it solves for the max home price.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.afford.h2.inputs">Your finances</h2>
            <form id="afford-form" class="inline-form">
                <label><span data-i18n="view.afford.label.income">Annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_income_usd" value="100000" required></label>
                <label><span data-i18n="view.afford.label.debts">Other monthly debts ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_debts_usd" value="500"></label>
                <label><span data-i18n="view.afford.label.down">Down payment ($)</span>
                    <input type="number" step="0.01" min="0" name="down_payment_usd" value="50000" required></label>
                <label><span data-i18n="view.afford.label.rate">Mortgage rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6.5" required></label>
                <label><span data-i18n="view.afford.label.term">Term (months)</span>
                    <input type="number" step="1" min="1" name="term_months" value="360" required></label>
                <label><span data-i18n="view.afford.label.tax">Property tax rate (% / yr)</span>
                    <input type="number" step="0.01" min="0" name="property_tax_rate_pct" value="1.2"></label>
                <label><span data-i18n="view.afford.label.insurance">Annual insurance ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_insurance_usd" value="1500"></label>
                <label><span data-i18n="view.afford.label.front">Front-end (%)</span>
                    <input type="number" step="0.1" min="0" name="front_end_pct" value="28"></label>
                <label><span data-i18n="view.afford.label.back">Back-end (%)</span>
                    <input type="number" step="0.1" min="0" name="back_end_pct" value="36"></label>
            </form>
            <div id="afford-tools" class="ce-toolbar"></div>
            <button type="button" id="afford-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="afford-sens" class="ce-sens"></div>
        </div>
        <div id="afford-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#afford-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            annual_income_usd: Number(fd.get('annual_income_usd')) || 0,
            monthly_debts_usd: Number(fd.get('monthly_debts_usd')) || 0,
            down_payment_usd: Number(fd.get('down_payment_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            term_months: Number(fd.get('term_months')) || 0,
            property_tax_rate_pct: Number(fd.get('property_tax_rate_pct')) || 0,
            annual_insurance_usd: Number(fd.get('annual_insurance_usd')) || 0,
            front_end_pct: Number(fd.get('front_end_pct')) || 0,
            back_end_pct: Number(fd.get('back_end_pct')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcMortgageAffordability(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.afford.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#afford-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'mortgage-affordability.csv' });
    mount.querySelector('#afford-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['max_home_price_usd', r.max_home_price_usd],
        ['max_loan_usd', r.max_loan_usd],
        ['max_piti_usd', r.max_piti_usd],
        ['binding_constraint', r.binding_constraint],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#afford-result');
    const bindKey = r.binding_constraint === 'front' ? 'view.afford.bind.front' : 'view.afford.bind.back';
    // Line chart: max home price as the mortgage rate sweeps 3% -> 10% (affordability falls).
    const xs = enh.linspace(3, 10, 13);
    const pts = await Promise.all(xs.map(async (rate) => {
        const rr = await api.calcMortgageAffordability({ ...body, annual_rate_pct: rate });
        return { x: rate, y: rr ? rr.max_home_price_usd / 1000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'rate %', ylabel: 'max price $k' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.afford.h2.result">What you can afford</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.afford.card.price">Max home price</div>
                    <div class="value pos">${money(r.max_home_price_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.afford.card.loan">Max loan</div>
                    <div class="value">${money(r.max_loan_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.afford.card.piti">Max PITI / mo</div>
                    <div class="value">${money(r.max_piti_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.afford.row.income">Monthly income</td><td>${money(r.monthly_income_usd)}</td></tr>
                    <tr><td data-i18n="view.afford.row.front">Front-end cap (28%)</td><td>${money(r.front_end_max_usd)}</td></tr>
                    <tr><td data-i18n="view.afford.row.back">Back-end room (36% − debts)</td><td>${money(r.back_end_max_usd)}</td></tr>
                    <tr><td data-i18n="view.afford.row.binding">Binding constraint</td><td data-i18n="${bindKey}">—</td></tr>
                    <tr><td data-i18n="view.afford.row.loan">Max loan</td><td>${money(r.max_loan_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.afford.row.price">Max home price</td><td>${money(r.max_home_price_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#afford-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: mortgage rate 3% -> 10%; y: annual income 0.6x -> 1.4x. Output: max home price.
    const inc = base.annual_income_usd || 100000;
    const xVals = enh.linspace(3, 10, 5);
    const yVals = enh.linspace(inc * 0.6, inc * 1.4, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'annual_rate_pct', yKey: 'annual_income_usd', xVals, yVals, compute: (b) => api.calcMortgageAffordability(b), pick: (r) => (r ? r.max_home_price_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v / 1000) + 'k'), xfmt: (v) => v.toFixed(1) + '%', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.afford.label.rate') || 'Rate', yName: t('view.afford.label.income') || 'Income' });
}
