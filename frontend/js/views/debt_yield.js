// Debt yield & loan sizing — commercial-RE lender ratios (debt yield, LTV,
// LTC) and the max loan each allows, with the binding constraint, via
// /calc/debt-yield. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['noi_usd', 'Net operating income ($)', 100000],
    ['property_value_usd', 'Appraised value ($)', 1400000],
    ['total_project_cost_usd', 'Total project cost ($)', 1350000],
    ['loan_amount_usd', 'Proposed loan ($)', 1000000],
    ['min_debt_yield_pct', 'Min debt yield (%)', 10],
    ['max_ltv_pct', 'Max LTV (%)', 75],
    ['max_ltc_pct', 'Max LTC (%)', 80],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => Number(n).toFixed(2) + '%';
const BINDING = { debt_yield: 'Debt yield', ltv: 'LTV', ltc: 'LTC' };
const VIEW = 'debt-yield';
let lastReport = null;
let lastBody = null;

export async function renderDebtYield(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dy.h1.title">// DEBT YIELD & LOAN SIZING</span></h1>
        <p class="muted small" data-i18n="view.dy.hint.intro">
            A commercial lender sizes a loan against three ceilings, and the smallest wins:
            debt yield (NOI ÷ loan, a rate-independent risk measure, floored at ~8–10%), LTV
            (loan ÷ value), and LTC (loan ÷ total cost). This shows each ratio for your proposed
            loan, the max loan each ceiling allows, and which one binds. Complements cap rate
            and DSCR. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.dy.h2.inputs">The deal</h2>
            <form id="dy-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.dy.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="dy-tools" class="ce-toolbar"></div>
            <button type="button" id="dy-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="dy-sens" class="ce-sens"></div>
        </div>
        <div id="dy-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#dy-form');
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
            const r = await api.calcDebtYield(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.dy.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#dy-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'debt-yield.csv' });
    mount.querySelector('#dy-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['max_loan_usd', r.max_loan_usd],
        ['binding_constraint', r.binding_constraint],
        ['loan_fits', r.loan_fits],
        ['debt_yield_pct', r.debt_yield_pct],
        ['ltv_pct', r.ltv_pct],
        ['ltc_pct', r.ltc_pct],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#dy-result');
    const fitCls = r.loan_fits ? 'pos' : 'neg';
    const binding = BINDING[r.binding_constraint] || r.binding_constraint;
    // Line chart: max loan as min debt yield sweeps 8% → 14% (loan ceiling falls as the floor rises).
    const xs = enh.linspace(8, 14, 13);
    const pts = await Promise.all(xs.map(async (dy) => {
        const rr = await api.calcDebtYield({ ...body, min_debt_yield_pct: dy });
        return { x: dy, y: rr ? rr.max_loan_usd / 1000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'min DY %', ylabel: 'max loan $k' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.dy.h2.result">Loan sizing</h2>
            <div class="cards">
                <div class="card ${fitCls}"><div class="label" data-i18n="view.dy.card.maxloan">Max loan</div>
                    <div class="value ${fitCls}">${money(r.max_loan_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dy.card.binding">Binding constraint</div>
                    <div class="value">${binding}</div></div>
                <div class="card ${fitCls}"><div class="label" data-i18n="view.dy.card.fits">Proposed loan fits?</div>
                    <div class="value ${fitCls}">${r.loan_fits ? t('view.dy.yes') : t('view.dy.no')}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.dy.col.metric">Metric</th>
                    <th data-i18n="view.dy.col.current">Current</th>
                    <th data-i18n="view.dy.col.maxloan">Max loan</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.dy.row.dy">Debt yield</td><td>${pct(r.debt_yield_pct)}</td><td>${money(r.max_loan_by_debt_yield_usd)}</td></tr>
                    <tr><td data-i18n="view.dy.row.ltv">LTV</td><td>${pct(r.ltv_pct)}</td><td>${money(r.max_loan_by_ltv_usd)}</td></tr>
                    <tr><td data-i18n="view.dy.row.ltc">LTC</td><td>${pct(r.ltc_pct)}</td><td>${money(r.max_loan_by_ltc_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#dy-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: min debt yield 8% → 14%; y: max LTV 60% → 85%. Output: max loan.
    const xVals = enh.linspace(8, 14, 5);
    const yVals = enh.linspace(60, 85, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'min_debt_yield_pct', yKey: 'max_ltv_pct', xVals, yVals, compute: (b) => api.calcDebtYield(b), pick: (r) => (r ? r.max_loan_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v / 1000) + 'k'), xfmt: (v) => v.toFixed(0) + '%', yfmt: (v) => v.toFixed(0) + '%', xName: t('view.dy.label.min_debt_yield_pct') || 'Min DY', yName: t('view.dy.label.max_ltv_pct') || 'Max LTV' });
}
