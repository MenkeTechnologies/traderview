// True hourly wage — net pay after job costs over all job-related hours, via
// /calc/true-hourly-wage. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';
const hrs = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

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
        </div>
        <div id="truewage-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#truewage-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            gross_annual_salary_usd: Number(fd.get('gross_annual_salary_usd')) || 0,
            weeks_worked: Number(fd.get('weeks_worked')) || 0,
            weekly_work_hours: Number(fd.get('weekly_work_hours')) || 0,
            weekly_commute_hours: Number(fd.get('weekly_commute_hours')) || 0,
            weekly_extra_hours: Number(fd.get('weekly_extra_hours')) || 0,
            annual_taxes_usd: Number(fd.get('annual_taxes_usd')) || 0,
            annual_work_expenses_usd: Number(fd.get('annual_work_expenses_usd')) || 0,
        };
        try {
            const r = await api.calcTrueHourlyWage(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.truewage.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#truewage-result');
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
