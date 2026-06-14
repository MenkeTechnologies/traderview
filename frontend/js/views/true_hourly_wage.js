// True hourly wage — net pay after job costs over all job-related hours, via
// /calc/true-hourly-wage. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';
const hrs = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VIEW = 'true-hourly-wage';
let lastReport = null;
let lastBody = null;

export async function renderTrueHourlyWage(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.truewage.h1.title">// TRUE HOURLY WAGE</span></h1>
        <p class="muted small" data-i18n="view.truewage.hint.intro">
            What a job really pays once job-related costs come out of pay and job-related time —
            commute, prep, decompression — is added to the hours. The nominal hourly (salary ÷ paid
            hours) almost always overstates it. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.truewage.h2.inputs">The job</h2>
            <form id="truewage-form" class="inline-form">
                <label><span data-i18n="view.truewage.label.salary">Gross annual salary ($)</span>
                    <input type="number" step="0.01" min="0" name="gross_annual_salary_usd" value="60000" required></label>
                <label><span data-i18n="view.truewage.label.weeks">Weeks worked / year</span>
                    <input type="number" step="1" min="1" name="weeks_worked" value="50" required></label>
                <label><span data-i18n="view.truewage.label.work">Work hours / week</span>
                    <input type="number" step="0.5" min="0" name="weekly_work_hours" value="40" required></label>
                <label><span data-i18n="view.truewage.label.commute">Commute hours / week</span>
                    <input type="number" step="0.5" min="0" name="weekly_commute_hours" value="5"></label>
                <label><span data-i18n="view.truewage.label.extra">Extra job hours / week</span>
                    <input type="number" step="0.5" min="0" name="weekly_extra_hours" value="0"></label>
                <label><span data-i18n="view.truewage.label.taxes">Annual taxes ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_taxes_usd" value="12000"></label>
                <label><span data-i18n="view.truewage.label.expenses">Annual work expenses ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_work_expenses_usd" value="3000"></label>
            </form>
            <div id="truewage-tools" class="ce-toolbar"></div>
            <button type="button" id="truewage-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="truewage-sens" class="ce-sens"></div>
        </div>
        <div id="truewage-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#truewage-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            gross_annual_salary_usd: Number(fd.get('gross_annual_salary_usd')) || 0,
            weeks_worked: Number(fd.get('weeks_worked')) || 0,
            weekly_work_hours: Number(fd.get('weekly_work_hours')) || 0,
            weekly_commute_hours: Number(fd.get('weekly_commute_hours')) || 0,
            weekly_extra_hours: Number(fd.get('weekly_extra_hours')) || 0,
            annual_taxes_usd: Number(fd.get('annual_taxes_usd')) || 0,
            annual_work_expenses_usd: Number(fd.get('annual_work_expenses_usd')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcTrueHourlyWage(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.truewage.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#truewage-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'true-hourly-wage.csv' });
    mount.querySelector('#truewage-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['true_hourly_usd', r.true_hourly_usd],
        ['nominal_hourly_usd', r.nominal_hourly_usd],
        ['erosion_pct', r.erosion_pct],
        ['net_annual_usd', r.net_annual_usd],
        ['total_annual_hours', r.total_annual_hours],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#truewage-result');
    // Line chart: true hourly as weekly commute hours sweep 0 -> 20 (more unpaid time erodes pay).
    const xs = enh.linspace(0, 20, 13);
    const pts = await Promise.all(xs.map(async (c) => {
        const rr = await api.calcTrueHourlyWage({ ...body, weekly_commute_hours: c });
        return { x: c, y: rr ? rr.true_hourly_usd : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'commute h/wk', ylabel: 'true $/hr' });
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.truewage.h2.result">Your real pay</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.truewage.card.true">True hourly</div>
                    <div class="value pos">${money(r.true_hourly_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.truewage.card.nominal">Nominal hourly</div>
                    <div class="value">${money(r.nominal_hourly_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.truewage.card.erosion">Erosion</div>
                    <div class="value neg">${pct(r.erosion_pct)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.truewage.row.net">Net annual (after job costs)</td><td>${money(r.net_annual_usd)}</td></tr>
                    <tr><td data-i18n="view.truewage.row.nominalhours">Paid hours / year</td><td>${hrs(r.nominal_annual_hours)}</td></tr>
                    <tr><td data-i18n="view.truewage.row.totalhours">Job-related hours / year</td><td>${hrs(r.total_annual_hours)}</td></tr>
                    <tr><td data-i18n="view.truewage.row.nominal">Nominal hourly</td><td>${money(r.nominal_hourly_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.truewage.row.true">True hourly</td><td>${money(r.true_hourly_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#truewage-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: weekly commute hours 0 -> 20; y: annual work expenses 0 -> 2x. Output: true hourly.
    const ex = base.annual_work_expenses_usd || 3000;
    const xVals = enh.linspace(0, 20, 5);
    const yVals = enh.linspace(0, ex * 2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'weekly_commute_hours', yKey: 'annual_work_expenses_usd', xVals, yVals, compute: (b) => api.calcTrueHourlyWage(b), pick: (r) => (r ? r.true_hourly_usd : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : '$' + v.toFixed(2)), xfmt: (v) => v.toFixed(0) + 'h', yfmt: (v) => '$' + Math.round(v / 1000) + 'k', xName: t('view.truewage.label.commute') || 'Commute', yName: t('view.truewage.label.expenses') || 'Expenses' });
}
