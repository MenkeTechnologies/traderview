// Disability-insurance needs — monthly benefit gap net of group LTD, vs
// expenses, via /calc/disability-insurance-needs. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const x = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '×');

export async function renderDisabilityInsuranceNeeds(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.disab.h1.title">// DISABILITY INSURANCE NEEDS</span></h1>
        <p class="muted small" data-i18n="view.disab.hint.intro">
            The monthly benefit that replaces lost income if you can't work, net of any employer
            long-term-disability coverage. Policies typically replace ~60% of income (benefits are
            tax-free on premiums you pay). The gap is the extra benefit to buy, checked against your
            essential expenses. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.disab.h2.inputs">Your situation</h2>
            <form id="disab-form" class="inline-form">
                <label><span data-i18n="view.disab.label.income">Annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_income_usd" value="90000" required></label>
                <label><span data-i18n="view.disab.label.replacement">Replacement (% of income)</span>
                    <input type="number" step="1" min="0" max="100" name="replacement_pct" value="60" required></label>
                <label><span data-i18n="view.disab.label.existing">Existing LTD / mo ($)</span>
                    <input type="number" step="0.01" min="0" name="existing_coverage_monthly_usd" value="2000"></label>
                <label><span data-i18n="view.disab.label.expenses">Monthly expenses ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_expenses_usd" value="4000"></label>
            </form>
        </div>
        <div id="disab-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#disab-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            annual_income_usd: Number(fd.get('annual_income_usd')) || 0,
            replacement_pct: Number(fd.get('replacement_pct')) || 0,
            existing_coverage_monthly_usd: Number(fd.get('existing_coverage_monthly_usd')) || 0,
            monthly_expenses_usd: Number(fd.get('monthly_expenses_usd')) || 0,
        };
        try {
            const r = await api.calcDisabilityInsuranceNeeds(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.disab.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#disab-result');
    const gapCls = r.monthly_gap_usd > 0 ? 'neg' : 'pos';
    const coverCls = r.covers_expenses ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.disab.h2.result">The coverage gap</h2>
            <div class="cards">
                <div class="card ${gapCls}"><div class="label" data-i18n="view.disab.card.gap">Monthly gap</div>
                    <div class="value ${gapCls}">${money(r.monthly_gap_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.disab.card.target">Target benefit / mo</div>
                    <div class="value">${money(r.target_monthly_benefit_usd)}</div></div>
                <div class="card ${coverCls}"><div class="label" data-i18n="view.disab.card.cover">Covers expenses</div>
                    <div class="value ${coverCls}">${x(r.expense_coverage_ratio)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.disab.row.income">Monthly income</td><td>${money(r.monthly_income_usd)}</td></tr>
                    <tr><td data-i18n="view.disab.row.target">Target monthly benefit</td><td>${money(r.target_monthly_benefit_usd)}</td></tr>
                    <tr><td data-i18n="view.disab.row.annual">Annual benefit gap</td><td>${money(r.annual_gap_usd)}</td></tr>
                    <tr class="emph ${gapCls}"><td data-i18n="view.disab.row.gap">Monthly benefit gap</td><td>${money(r.monthly_gap_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
