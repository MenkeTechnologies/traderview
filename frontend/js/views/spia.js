// SPIA — single-premium immediate annuity: monthly income from a lump sum,
// the payout rate, and total received, via /calc/spia. Updates live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['premium_usd', 'Premium (lump sum, $)', 100000],
    ['payout_years', 'Payout period (years)', 20],
    ['annual_rate_pct', 'Assumed rate (%)', 5],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
const money0 = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'spia';
let lastReport = null;
let lastBody = null;

export async function renderSpia(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.spia.h1.title">// SPIA INCOME</span></h1>
        <p class="muted small" data-i18n="view.spia.hint.intro">
            A single-premium immediate annuity turns a lump sum into guaranteed income starting
            now. The monthly payment is the annuity that exhausts the premium over the payout
            period at the insurer's assumed rate — the same math as a loan payment, in reverse.
            The payout rate is the annual income as a percent of the premium. (Period-certain
            approximation; a true life annuity also prices in mortality.) Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.spia.h2.inputs">The annuity</h2>
            <form id="spia-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.spia.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="spia-tools" class="ce-toolbar"></div>
            <button type="button" id="spia-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="spia-sens" class="ce-sens"></div>
        </div>
        <div id="spia-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#spia-form');
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
            const r = await api.calcSpia(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.spia.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#spia-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'spia.csv' });
    mount.querySelector('#spia-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['monthly_income_usd', r.monthly_income_usd],
        ['annual_income_usd', r.annual_income_usd],
        ['payout_rate_pct', r.payout_rate_pct],
        ['total_received_usd', r.total_received_usd],
        ['interest_earned_usd', r.interest_earned_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#spia-result');
    // Line chart: monthly income as the assumed rate sweeps 0 -> 10%.
    const xs = enh.linspace(0, 10, 13);
    const pts = await Promise.all(xs.map(async (rate) => {
        const rr = await api.calcSpia({ ...body, annual_rate_pct: rate });
        return { x: rate, y: rr ? rr.monthly_income_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'rate %', ylabel: 'monthly $' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.spia.h2.result">The income</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.spia.card.monthly">Monthly income</div>
                    <div class="value pos">${money(r.monthly_income_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.spia.card.annual">Annual income</div>
                    <div class="value">${money0(r.annual_income_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.spia.card.payout">Payout rate</div>
                    <div class="value">${Number(r.payout_rate_pct).toFixed(2)}%</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr><th data-i18n="view.spia.col.line">Line</th><th data-i18n="view.spia.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.spia.row.total">Total received (period)</td><td>${money0(r.total_received_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.spia.row.interest">Interest credited</td>
                        <td class="${r.interest_earned_usd >= 0 ? 'pos' : 'neg'}">${money0(r.interest_earned_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#spia-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: assumed rate 0 -> 10%; y: payout years 5 -> 35. Output: monthly income.
    const xVals = enh.linspace(0, 10, 5);
    const yVals = enh.linspace(5, 35, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'annual_rate_pct', yKey: 'payout_years', xVals, yVals, compute: (b) => api.calcSpia(b), pick: (r) => (r ? r.monthly_income_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + Math.round(v)), xfmt: (v) => v.toFixed(1) + '%', yfmt: (v) => v.toFixed(0) + 'y', xName: t('view.spia.label.annual_rate_pct') || 'Rate', yName: t('view.spia.label.payout_years') || 'Years' });
}
