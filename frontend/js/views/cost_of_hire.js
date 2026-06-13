// True cost of hire — fully-loaded W-2 employee cost vs 1099 contractor,
// with the burden multiplier and effective hourly for each, via
// /calc/cost-of-hire. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

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
        </div>
        <div id="coh-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#coh-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcCostOfHire(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.coh.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#coh-result');
    const winnerCls = r.w2_cheaper ? 'pos' : 'neg';
    const winner = r.w2_cheaper ? t('view.coh.winner.w2') : t('view.coh.winner.contractor');
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
