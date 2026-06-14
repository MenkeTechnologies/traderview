// True cost of hire — fully-loaded W-2 employee cost vs 1099 contractor,
// with the burden multiplier and effective hourly for each, via
// /calc/cost-of-hire. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const FIELDS = [
    ['base_salary_usd', 'Base salary ($)', 100000],
    ['employer_payroll_tax_pct', 'Employer payroll tax (%)', 7.65],
    ['annual_benefits_usd', 'Annual benefits ($)', 12000],
    ['retirement_match_pct', 'Retirement match (% of salary)', 4],
    ['workers_comp_pct', 'Workers comp (% of salary)', 1],
    ['other_overhead_usd', 'Other overhead ($)', 5000],
    ['pto_days', 'Paid time off (days)', 15],
    ['annual_hours', 'Annual hours', 2080],
    ['contractor_annual_usd', '1099 contractor annual ($)', 150000],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const hourly = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '/hr';
const pct = (n) => Number(n).toFixed(1) + '%';
const VIEW = 'cost-of-hire';
let lastReport = null;
let lastBody = null;

export async function renderCostOfHire(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.coh.h1.title">// TRUE COST OF HIRE</span></h1>
        <p class="muted small" data-i18n="view.coh.hint.intro">
            A salary is only part of what an employee costs. Fully loaded adds the employer's
            payroll taxes, benefits, retirement match, workers' comp, and overhead — commonly
            1.25×–1.4× base. A 1099 contractor bills a higher rate but covers their own taxes,
            benefits, and gear, so their cost is just the contract spend. This compares total
            annual cost and effective hourly (the employee's PTO cuts productive hours, raising
            their true hourly). Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.coh.h2.inputs">The roles</h2>
            <form id="coh-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.coh.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
            <div id="coh-tools" class="ce-toolbar"></div>
            <button type="button" id="coh-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="coh-sens" class="ce-sens"></div>
        </div>
        <div id="coh-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#coh-form');
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
            const r = await api.calcCostOfHire(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.coh.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#coh-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'cost-of-hire.csv' });
    mount.querySelector('#coh-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['total_w2_cost_usd', r.total_w2_cost_usd],
        ['burden_pct', r.burden_pct],
        ['w2_cheaper', r.w2_cheaper],
        ['w2_effective_hourly_usd', r.w2_effective_hourly_usd],
        ['contractor_effective_hourly_usd', r.contractor_effective_hourly_usd],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#coh-result');
    const winnerCls = r.w2_cheaper ? 'pos' : 'neg';
    const winner = r.w2_cheaper ? t('view.coh.winner.w2') : t('view.coh.winner.contractor');
    // Line chart: fully-loaded W-2 cost as base salary sweeps 0.5× → 1.5×.
    const base = body.base_salary_usd || 100000;
    const xs = enh.linspace(base * 0.5, base * 1.5, 13);
    const pts = await Promise.all(xs.map(async (sal) => {
        const rr = await api.calcCostOfHire({ ...body, base_salary_usd: sal });
        return { x: sal / 1000, y: rr ? rr.total_w2_cost_usd / 1000 : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'base $k', ylabel: 'W-2 cost $k' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.coh.h2.result">The comparison</h2>
            <div class="cards">
                <div class="card ${winnerCls}"><div class="label" data-i18n="view.coh.card.cheaper">Cheaper option</div>
                    <div class="value ${winnerCls}">${winner}</div></div>
                <div class="card"><div class="label" data-i18n="view.coh.card.w2">W-2 fully loaded</div>
                    <div class="value">${money(r.total_w2_cost_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.coh.card.burden">Burden over base</div>
                    <div class="value">${pct(r.burden_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.coh.col.line">Line</th>
                    <th data-i18n="view.coh.col.amount">Amount</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.coh.row.payroll">Employer payroll tax</td><td>${money(r.employer_payroll_tax_usd)}</td></tr>
                    <tr><td data-i18n="view.coh.row.match">Retirement match</td><td>${money(r.retirement_match_usd)}</td></tr>
                    <tr><td data-i18n="view.coh.row.wc">Workers comp</td><td>${money(r.workers_comp_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.coh.row.total">Total W-2 cost</td><td>${money(r.total_w2_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.coh.row.w2_hourly">W-2 effective hourly</td><td>${hourly(r.w2_effective_hourly_usd)}</td></tr>
                    <tr><td data-i18n="view.coh.row.contractor_hourly">Contractor effective hourly</td><td>${hourly(r.contractor_effective_hourly_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#coh-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: base salary 0.5× → 1.5×; y: annual benefits 0 → 2×. Output: total W-2 cost.
    const sal = base.base_salary_usd || 100000;
    const ben = base.annual_benefits_usd || 12000;
    const xVals = enh.linspace(sal * 0.5, sal * 1.5, 5);
    const yVals = enh.linspace(0, ben * 2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'base_salary_usd', yKey: 'annual_benefits_usd', xVals, yVals, compute: (b) => api.calcCostOfHire(b), pick: (r) => (r ? r.total_w2_cost_usd : null) });
    if (!viewIsCurrent(tok)) return;
    // Lower total cost is better → negate for shading (green = cheaper) while showing the real value.
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells: cells.map((row) => row.map((v) => (v == null ? null : -v))), fmt: (v) => (v == null ? '—' : '$' + Math.round(-v / 1000) + 'k'), xfmt: (v) => '$' + Math.round(v / 1000) + 'k', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.coh.label.base_salary_usd') || 'Base', yName: t('view.coh.label.annual_benefits_usd') || 'Benefits' });
}
