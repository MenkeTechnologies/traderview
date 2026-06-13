// Second-income worth-it — net household benefit after taxes, childcare, and
// work costs, via /calc/second-income. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';

export async function renderSecondIncome(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.secondinc.h1.title">// SECOND INCOME</span></h1>
        <p class="muted small" data-i18n="view.secondinc.hint.intro">
            What a household actually keeps from a second earner's income. It's taxed at the
            household's marginal rate (it stacks on top of the first), then childcare and
            work-related costs come out — what's left is the real benefit of working. Updates as
            you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.secondinc.h2.inputs">The second job</h2>
            <form id="secondinc-form" class="inline-form">
                <label><span data-i18n="view.secondinc.label.income">Second annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="second_annual_income_usd" value="50000" required></label>
                <label><span data-i18n="view.secondinc.label.tax">Marginal tax rate (%)</span>
                    <input type="number" step="0.1" min="0" name="marginal_tax_rate_pct" value="30" required></label>
                <label><span data-i18n="view.secondinc.label.childcare">Annual childcare ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_childcare_usd" value="18000"></label>
                <label><span data-i18n="view.secondinc.label.commute">Annual commute ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_commute_usd" value="3000"></label>
                <label><span data-i18n="view.secondinc.label.other">Other work expenses ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_other_work_expenses_usd" value="2000"></label>
            </form>
        </div>
        <div id="secondinc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#secondinc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            second_annual_income_usd: Number(fd.get('second_annual_income_usd')) || 0,
            marginal_tax_rate_pct: Number(fd.get('marginal_tax_rate_pct')) || 0,
            annual_childcare_usd: Number(fd.get('annual_childcare_usd')) || 0,
            annual_commute_usd: Number(fd.get('annual_commute_usd')) || 0,
            annual_other_work_expenses_usd: Number(fd.get('annual_other_work_expenses_usd')) || 0,
        };
        try {
            const r = await api.calcSecondIncome(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.secondinc.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#secondinc-result');
    const cls = r.worth_it ? 'pos' : 'neg';
    const verdictKey = r.worth_it ? 'view.secondinc.verdict.worth' : 'view.secondinc.verdict.not';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.secondinc.h2.result">The real benefit</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.secondinc.card.net">Net benefit</div>
                    <div class="value ${cls}">${money(r.net_benefit_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.secondinc.card.keep">Keep rate</div>
                    <div class="value">${pct(r.keep_rate_pct)}</div></div>
                <div class="card ${cls}"><div class="label" data-i18n="view.secondinc.card.verdict">Verdict</div>
                    <div class="value ${cls}" data-i18n="${verdictKey}">—</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.secondinc.row.taxes">Taxes</td><td>${money(r.taxes_usd)}</td></tr>
                    <tr><td data-i18n="view.secondinc.row.aftertax">After-tax income</td><td>${money(r.after_tax_income_usd)}</td></tr>
                    <tr><td data-i18n="view.secondinc.row.costs">Total work costs</td><td>${money(r.total_costs_usd)}</td></tr>
                    <tr><td data-i18n="view.secondinc.row.keep">Keep rate</td><td>${pct(r.keep_rate_pct)}</td></tr>
                    <tr class="emph ${cls}"><td data-i18n="view.secondinc.row.net">Net benefit</td><td>${money(r.net_benefit_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
