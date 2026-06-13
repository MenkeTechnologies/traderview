// Overtime pay — weekly/annual gross from regular, overtime, and double-time
// hours, via /calc/overtime-pay. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const hrs = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 });

export async function renderOvertimePay(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ot.h1.title">// OVERTIME PAY</span></h1>
        <p class="muted small" data-i18n="view.ot.hint.intro">
            Weekly gross from regular, overtime, and double-time hours. Under the FLSA, hours over 40
            a week pay 1.5×; some states or contracts add 2× double-time. Shows the blended effective
            hourly, the premium over straight time, and the annualized gross. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ot.h2.inputs">The week</h2>
            <form id="ot-form" class="inline-form">
                <label><span data-i18n="view.ot.label.rate">Hourly rate ($)</span>
                    <input type="number" step="0.01" min="0" name="hourly_rate_usd" value="20" required></label>
                <label><span data-i18n="view.ot.label.regular">Regular hours</span>
                    <input type="number" step="0.25" min="0" name="regular_hours" value="40" required></label>
                <label><span data-i18n="view.ot.label.ot">Overtime hours</span>
                    <input type="number" step="0.25" min="0" name="overtime_hours" value="10"></label>
                <label><span data-i18n="view.ot.label.dt">Double-time hours</span>
                    <input type="number" step="0.25" min="0" name="double_time_hours" value="0"></label>
                <label><span data-i18n="view.ot.label.otmult">OT multiplier</span>
                    <input type="number" step="0.1" min="1" name="overtime_multiplier" value="1.5"></label>
                <label><span data-i18n="view.ot.label.dtmult">DT multiplier</span>
                    <input type="number" step="0.1" min="1" name="double_time_multiplier" value="2"></label>
                <label><span data-i18n="view.ot.label.weeks">Weeks / year</span>
                    <input type="number" step="1" min="1" name="weeks_per_year" value="52"></label>
            </form>
        </div>
        <div id="ot-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ot-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            hourly_rate_usd: Number(fd.get('hourly_rate_usd')) || 0,
            regular_hours: Number(fd.get('regular_hours')) || 0,
            overtime_hours: Number(fd.get('overtime_hours')) || 0,
            double_time_hours: Number(fd.get('double_time_hours')) || 0,
            overtime_multiplier: Number(fd.get('overtime_multiplier')) || 0,
            double_time_multiplier: Number(fd.get('double_time_multiplier')) || 0,
            weeks_per_year: Number(fd.get('weeks_per_year')) || 0,
        };
        try {
            const r = await api.calcOvertimePay(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.ot.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#ot-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ot.h2.result">The paycheck</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ot.card.weekly">Weekly gross</div>
                    <div class="value pos">${money(r.weekly_gross_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ot.card.effective">Effective hourly</div>
                    <div class="value">${money(r.effective_hourly_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ot.card.premium">OT premium</div>
                    <div class="value">${money(r.premium_pay_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.ot.row.regular">Regular pay</td><td>${money(r.regular_pay_usd)}</td></tr>
                    <tr><td data-i18n="view.ot.row.ot">Overtime pay</td><td>${money(r.overtime_pay_usd)}</td></tr>
                    <tr><td data-i18n="view.ot.row.dt">Double-time pay</td><td>${money(r.double_time_pay_usd)}</td></tr>
                    <tr><td data-i18n="view.ot.row.hours">Total hours</td><td>${hrs(r.total_hours)}</td></tr>
                    <tr><td data-i18n="view.ot.row.annual">Annual gross</td><td>${money(r.annual_gross_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.ot.row.weekly">Weekly gross</td><td>${money(r.weekly_gross_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
