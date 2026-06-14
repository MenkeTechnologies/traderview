// Car affordability — the 20/4/10 rule worked back to a max car price, via
// /calc/car-affordability. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const VIEW = 'car-affordability';
let lastReport = null;
let lastBody = null;

export async function renderCarAffordability(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.caraff.h1.title">// CAR AFFORDABILITY</span></h1>
        <p class="muted small" data-i18n="view.caraff.hint.intro">
            The 20/4/10 rule of thumb — put 20% down, finance for no more than 4 years, and keep
            total vehicle spending under 10% of gross income. This works back from the income cap to
            the most car you can afford. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.caraff.h2.inputs">Your budget</h2>
            <form id="caraff-form" class="inline-form">
                <label><span data-i18n="view.caraff.label.income">Annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_income_usd" value="60000" required></label>
                <label><span data-i18n="view.caraff.label.down">Down payment (%)</span>
                    <input type="number" step="1" min="0" max="99" name="down_payment_pct" value="20" required></label>
                <label><span data-i18n="view.caraff.label.term">Loan term (months)</span>
                    <input type="number" step="1" min="1" name="loan_term_months" value="48" required></label>
                <label><span data-i18n="view.caraff.label.apr">APR (%)</span>
                    <input type="number" step="0.01" min="0" name="apr_pct" value="6" required></label>
                <label><span data-i18n="view.caraff.label.pct">Max % of gross income</span>
                    <input type="number" step="0.1" min="0" name="max_payment_pct_of_income" value="10" required></label>
                <label><span data-i18n="view.caraff.label.insfuel">Insurance + fuel / mo ($)</span>
                    <input type="number" step="0.01" min="0" name="insurance_fuel_monthly_usd" value="0"></label>
            </form>
            <div id="caraff-tools" class="ce-toolbar"></div>
            <button type="button" id="caraff-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="caraff-sens" class="ce-sens"></div>
        </div>
        <div id="caraff-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#caraff-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            annual_income_usd: Number(fd.get('annual_income_usd')) || 0,
            down_payment_pct: Number(fd.get('down_payment_pct')) || 0,
            loan_term_months: Number(fd.get('loan_term_months')) || 0,
            apr_pct: Number(fd.get('apr_pct')) || 0,
            max_payment_pct_of_income: Number(fd.get('max_payment_pct_of_income')) || 0,
            insurance_fuel_monthly_usd: Number(fd.get('insurance_fuel_monthly_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcCarAffordability(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.caraff.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#caraff-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'car-affordability.csv' });
    mount.querySelector('#caraff-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['max_car_price_usd', r.max_car_price_usd],
        ['max_loan_usd', r.max_loan_usd],
        ['monthly_payment_budget_usd', r.monthly_payment_budget_usd],
        ['monthly_transport_budget_usd', r.monthly_transport_budget_usd],
        ['down_payment_needed_usd', r.down_payment_needed_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#caraff-result');
    // Line chart: max car price as APR sweeps 3% -> 12% (higher rate buys less car).
    const xs = enh.linspace(3, 12, 13);
    const pts = await Promise.all(xs.map(async (apr) => {
        const rr = await api.calcCarAffordability({ ...body, apr_pct: apr });
        return { x: apr, y: rr ? rr.max_car_price_usd / 1000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'APR %', ylabel: 'max price $k' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.caraff.h2.result">What you can afford</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.caraff.card.price">Max car price</div>
                    <div class="value pos">${money(r.max_car_price_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.caraff.card.loan">Max loan</div>
                    <div class="value">${money(r.max_loan_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.caraff.card.payment">Payment budget</div>
                    <div class="value">${money(r.monthly_payment_budget_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.caraff.row.budget">Transport budget / mo</td><td>${money(r.monthly_transport_budget_usd)}</td></tr>
                    <tr><td data-i18n="view.caraff.row.payment">Payment budget / mo</td><td>${money(r.monthly_payment_budget_usd)}</td></tr>
                    <tr><td data-i18n="view.caraff.row.loan">Max loan</td><td>${money(r.max_loan_usd)}</td></tr>
                    <tr><td data-i18n="view.caraff.row.down">Down payment needed</td><td>${money(r.down_payment_needed_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.caraff.row.price">Max car price</td><td>${money(r.max_car_price_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#caraff-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: APR 3% -> 12%; y: annual income 0.6x -> 1.4x. Output: max car price.
    const inc = base.annual_income_usd || 60000;
    const xVals = enh.linspace(3, 12, 5);
    const yVals = enh.linspace(inc * 0.6, inc * 1.4, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'apr_pct', yKey: 'annual_income_usd', xVals, yVals, compute: (b) => api.calcCarAffordability(b), pick: (r) => (r ? r.max_car_price_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v / 1000) + 'k'), xfmt: (v) => v.toFixed(1) + '%', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.caraff.label.apr') || 'APR', yName: t('view.caraff.label.income') || 'Income' });
}
