// Life-insurance needs (DIME) — coverage gap to replace income and clear
// obligations net of existing coverage, via /calc/life-insurance-needs. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });

export async function renderLifeInsuranceNeeds(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lifeins.h1.title">// LIFE INSURANCE NEEDS</span></h1>
        <p class="muted small" data-i18n="view.lifeins.hint.intro">
            How much coverage replaces lost income and clears the family's obligations — the DIME
            method (Debt, Income, Mortgage, Education) plus final expenses, net of existing coverage
            and liquid savings. The gap is what you'd need to buy. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.lifeins.h2.inputs">Your situation</h2>
            <form id="lifeins-form" class="inline-form">
                <label><span data-i18n="view.lifeins.label.income">Annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_income_usd" value="80000" required></label>
                <label><span data-i18n="view.lifeins.label.years">Years to replace</span>
                    <input type="number" step="1" min="0" name="years_to_replace" value="10" required></label>
                <label><span data-i18n="view.lifeins.label.mortgage">Mortgage balance ($)</span>
                    <input type="number" step="0.01" min="0" name="mortgage_balance_usd" value="250000"></label>
                <label><span data-i18n="view.lifeins.label.debts">Other debts ($)</span>
                    <input type="number" step="0.01" min="0" name="other_debts_usd" value="20000"></label>
                <label><span data-i18n="view.lifeins.label.education">Education costs ($)</span>
                    <input type="number" step="0.01" min="0" name="education_costs_usd" value="100000"></label>
                <label><span data-i18n="view.lifeins.label.final">Final expenses ($)</span>
                    <input type="number" step="0.01" min="0" name="final_expenses_usd" value="15000"></label>
                <label><span data-i18n="view.lifeins.label.coverage">Existing coverage ($)</span>
                    <input type="number" step="0.01" min="0" name="existing_coverage_usd" value="200000"></label>
                <label><span data-i18n="view.lifeins.label.savings">Liquid savings ($)</span>
                    <input type="number" step="0.01" min="0" name="existing_savings_usd" value="50000"></label>
            </form>
        </div>
        <div id="lifeins-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#lifeins-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            annual_income_usd: Number(fd.get('annual_income_usd')) || 0,
            years_to_replace: Number(fd.get('years_to_replace')) || 0,
            mortgage_balance_usd: Number(fd.get('mortgage_balance_usd')) || 0,
            other_debts_usd: Number(fd.get('other_debts_usd')) || 0,
            education_costs_usd: Number(fd.get('education_costs_usd')) || 0,
            final_expenses_usd: Number(fd.get('final_expenses_usd')) || 0,
            existing_coverage_usd: Number(fd.get('existing_coverage_usd')) || 0,
            existing_savings_usd: Number(fd.get('existing_savings_usd')) || 0,
        };
        try {
            const r = await api.calcLifeInsuranceNeeds(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.lifeins.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#lifeins-result');
    const gapCls = r.coverage_gap_usd > 0 ? 'neg' : 'pos';
    const surplusRow = r.surplus_usd > 0
        ? `<tr class="pos"><td data-i18n="view.lifeins.row.surplus">Surplus coverage</td><td>${money(r.surplus_usd)}</td></tr>`
        : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.lifeins.h2.result">The coverage gap</h2>
            <div class="cards">
                <div class="card ${gapCls}"><div class="label" data-i18n="view.lifeins.card.gap">Coverage gap</div>
                    <div class="value ${gapCls}">${money(r.coverage_gap_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lifeins.card.need">Total need</div>
                    <div class="value">${money(r.total_need_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.lifeins.card.income">Income replacement</div>
                    <div class="value">${money(r.income_replacement_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.lifeins.row.income">Income replacement</td><td>${money(r.income_replacement_usd)}</td></tr>
                    <tr><td data-i18n="view.lifeins.row.need">Total need</td><td>${money(r.total_need_usd)}</td></tr>
                    <tr><td data-i18n="view.lifeins.row.offsets">Existing coverage + savings</td><td>${money(r.total_offsets_usd)}</td></tr>
                    ${surplusRow}
                    <tr class="emph ${gapCls}"><td data-i18n="view.lifeins.row.gap">Coverage gap</td><td>${money(r.coverage_gap_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
