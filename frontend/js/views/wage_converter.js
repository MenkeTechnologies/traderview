// Wage converter — hourly ↔ salary across week / two weeks / month / year,
// via /calc/wage-converter. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
let lastReport = null;

export async function renderWageConverter(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.wage.h1.title">// WAGE CONVERTER</span></h1>
        <p class="muted small" data-i18n="view.wage.hint.intro">
            Convert an hourly rate to an annual salary (or back) and see it per week, two weeks,
            month, and year. Set the hours worked per week and weeks worked per year — drop weeks
            to account for unpaid time off. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.wage.h2.inputs">The wage</h2>
            <form id="wage-form" class="inline-form">
                <label><span data-i18n="view.wage.label.mode">Convert</span>
                    <select name="mode">
                        <option value="hourly_to_salary" data-i18n="view.wage.mode.h2s">Hourly → salary</option>
                        <option value="salary_to_hourly" data-i18n="view.wage.mode.s2h">Salary → hourly</option>
                    </select></label>
                <label><span data-i18n="view.wage.label.amount">Amount ($)</span>
                    <input type="number" step="0.01" min="0" name="amount_usd" value="30" required></label>
                <label><span data-i18n="view.wage.label.hours">Hours / week</span>
                    <input type="number" step="0.01" min="0" name="hours_per_week" value="40" required></label>
                <label><span data-i18n="view.wage.label.weeks">Weeks / year</span>
                    <input type="number" step="0.01" min="0" name="weeks_per_year" value="52" required></label>
            </form>
            <div id="wage-tools" class="ce-toolbar"></div>
        </div>
        <div id="wage-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#wage-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            mode: fd.get('mode'),
            amount_usd: Number(fd.get('amount_usd')) || 0,
            hours_per_week: Number(fd.get('hours_per_week')) || 0,
            weeks_per_year: Number(fd.get('weeks_per_year')) || 0,
        };
        try {
            const r = await api.calcWageConverter(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.wage.toast.error'), { level: 'error' });
        }
    };
    // Export only — the periods (hourly … annual) span too many orders of
    // magnitude for one bar chart to be meaningful.
    enh.mountToolbar(mount.querySelector('#wage-tools'), {
        viewId: 'wage-converter', link: false, filename: 'wage-converter.csv',
        getRows: () => {
            const r = lastReport;
            if (!r) return [];
            return [['period', 'amount_usd'],
                ['hourly', r.hourly_usd],
                ['weekly', r.weekly_usd],
                ['biweekly', r.biweekly_usd],
                ['monthly', r.monthly_usd],
                ['annual', r.annual_usd]];
        },
    });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#wage-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.wage.h2.result">Every period</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.wage.card.annual">Annual</div>
                    <div class="value pos">${money(r.annual_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.wage.card.hourly">Hourly</div>
                    <div class="value">${money(r.hourly_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.wage.card.monthly">Monthly</div>
                    <div class="value">${money(r.monthly_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.wage.col.period">Period</th><th data-i18n="view.wage.col.pay">Pay</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.wage.row.hourly">Hourly</td><td>${money(r.hourly_usd)}</td></tr>
                    <tr><td data-i18n="view.wage.row.weekly">Weekly</td><td>${money(r.weekly_usd)}</td></tr>
                    <tr><td data-i18n="view.wage.row.biweekly">Biweekly</td><td>${money(r.biweekly_usd)}</td></tr>
                    <tr><td data-i18n="view.wage.row.monthly">Monthly</td><td>${money(r.monthly_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.wage.row.annual">Annual</td><td>${money(r.annual_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
