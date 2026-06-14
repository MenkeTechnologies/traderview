// PMI removal timeline — months until the loan amortizes to 80% / 78% of the
// original home value, via /calc/pmi-removal. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%';
const VIEW = 'pmi-removal';
let lastReport = null;
let lastBody = null;
const months = (n) => {
    if (n == null) return '—';
    const m = Math.round(n);
    const y = Math.floor(m / 12);
    const rem = m % 12;
    return `${m} (${y}y ${rem}m)`;
};

export async function renderPmiRemoval(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pmi.h1.title">// PMI REMOVAL</span></h1>
        <p class="muted small" data-i18n="view.pmi.hint.intro">
            When private mortgage insurance drops off a conventional loan. PMI is tied to
            loan-to-value against the original home value — you can request cancellation at 80% LTV,
            and the servicer must cancel automatically at 78% (Homeowners Protection Act). This shows
            the month scheduled payments reach each threshold. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pmi.h2.inputs">The loan</h2>
            <form id="pmi-form" class="inline-form">
                <label><span data-i18n="view.pmi.label.value">Original home value ($)</span>
                    <input type="number" step="0.01" min="0" name="original_home_value_usd" value="400000" required></label>
                <label><span data-i18n="view.pmi.label.loan">Original loan ($)</span>
                    <input type="number" step="0.01" min="0" name="original_loan_usd" value="360000" required></label>
                <label><span data-i18n="view.pmi.label.rate">Annual rate (%)</span>
                    <input type="number" step="0.001" min="0" name="annual_rate_pct" value="6" required></label>
                <label><span data-i18n="view.pmi.label.term">Loan term (months)</span>
                    <input type="number" step="1" min="1" name="loan_term_months" value="360" required></label>
            </form>
            <div id="pmi-tools" class="ce-toolbar"></div>
            <button type="button" id="pmi-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="pmi-sens" class="ce-sens"></div>
        </div>
        <div id="pmi-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pmi-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            original_home_value_usd: Number(fd.get('original_home_value_usd')) || 0,
            original_loan_usd: Number(fd.get('original_loan_usd')) || 0,
            annual_rate_pct: Number(fd.get('annual_rate_pct')) || 0,
            loan_term_months: Number(fd.get('loan_term_months')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcPmiRemoval(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.pmi.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#pmi-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'pmi-removal.csv' });
    mount.querySelector('#pmi-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['original_ltv_pct', r.original_ltv_pct],
        ['monthly_payment_usd', r.monthly_payment_usd],
        ['months_to_80', r.months_to_80 == null ? '' : r.months_to_80],
        ['months_to_78', r.months_to_78 == null ? '' : r.months_to_78],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#pmi-result');
    // Line chart: months to the 78% auto-cancel threshold as the rate sweeps 3% -> 9%
    // (a higher rate slows principal paydown, pushing PMI removal later).
    const xs = enh.linspace(3, 9, 13);
    const pts = await Promise.all(xs.map(async (rate) => {
        const rr = await api.calcPmiRemoval({ ...body, annual_rate_pct: rate });
        return { x: rate, y: rr && rr.months_to_78 != null ? rr.months_to_78 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'rate %', ylabel: 'months to 78%' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pmi.h2.result">When PMI drops</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.pmi.card.ltv">Original LTV</div>
                    <div class="value">${pct(r.original_ltv_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pmi.card.request">Request at 80%</div>
                    <div class="value">${months(r.months_to_80)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.pmi.card.auto">Auto at 78%</div>
                    <div class="value pos">${months(r.months_to_78)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.pmi.row.payment">Monthly payment</td><td>${money(r.monthly_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.pmi.row.t80">80% balance threshold</td><td>${money(r.target_80_balance_usd)}</td></tr>
                    <tr><td data-i18n="view.pmi.row.t78">78% balance threshold</td><td>${money(r.target_78_balance_usd)}</td></tr>
                    <tr><td data-i18n="view.pmi.row.request">Months to request (80%)</td><td>${months(r.months_to_80)}</td></tr>
                    <tr class="emph"><td data-i18n="view.pmi.row.auto">Months to automatic (78%)</td><td>${months(r.months_to_78)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#pmi-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: annual rate 3% -> 9%; y: original loan 0.8x -> 1.2x. Output: months to 78% (lower better -> negate).
    const loan = base.original_loan_usd || 360000;
    const xVals = enh.linspace(3, 9, 5);
    const yVals = enh.linspace(loan * 0.8, loan * 1.2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'annual_rate_pct', yKey: 'original_loan_usd', xVals, yVals, compute: (b) => api.calcPmiRemoval(b), pick: (r) => (r && r.months_to_78 != null ? r.months_to_78 : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : String(-Math.round(v)) + 'mo'), xfmt: (v) => v.toFixed(1) + '%', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.pmi.label.rate') || 'Rate', yName: t('view.pmi.label.loan') || 'Loan' });
}
