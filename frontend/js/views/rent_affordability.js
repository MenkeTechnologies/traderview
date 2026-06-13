// Rent affordability — max rent under the 30% and debt-adjusted rules, via
// /calc/rent-affordability. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export async function renderRentAffordability(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rentaff.h1.title">// RENT AFFORDABILITY</span></h1>
        <p class="muted small" data-i18n="view.rentaff.hint.intro">
            The most rent you can comfortably afford. The 30% rule caps rent at 30% of gross monthly
            income (the same as the landlord 40× annual rule); a debt-adjusted cap keeps rent plus
            other debts within a back-end limit. The lower of the two is the recommended ceiling.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rentaff.h2.inputs">Your finances</h2>
            <form id="rentaff-form" class="inline-form">
                <label><span data-i18n="view.rentaff.label.income">Annual income ($)</span>
                    <input type="number" step="0.01" min="0" name="annual_income_usd" value="60000" required></label>
                <label><span data-i18n="view.rentaff.label.debts">Other monthly debts ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_debts_usd" value="300"></label>
                <label><span data-i18n="view.rentaff.label.rentpct">Rent cap (% of income)</span>
                    <input type="number" step="0.1" min="0" name="rent_pct" value="30" required></label>
                <label><span data-i18n="view.rentaff.label.backend">Back-end cap (rent + debts %)</span>
                    <input type="number" step="0.1" min="0" name="back_end_pct" value="40"></label>
            </form>
        </div>
        <div id="rentaff-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rentaff-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            annual_income_usd: Number(fd.get('annual_income_usd')) || 0,
            monthly_debts_usd: Number(fd.get('monthly_debts_usd')) || 0,
            rent_pct: Number(fd.get('rent_pct')) || 0,
            back_end_pct: Number(fd.get('back_end_pct')) || 0,
        };
        try {
            const r = await api.calcRentAffordability(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.rentaff.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#rentaff-result');
    const bindKey = r.binding_constraint === 'income' ? 'view.rentaff.bind.income' : 'view.rentaff.bind.debts';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rentaff.h2.result">What you can afford</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.rentaff.card.recommended">Recommended max rent</div>
                    <div class="value pos">${money(r.recommended_max_rent_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rentaff.card.income">30% rule</div>
                    <div class="value">${money(r.max_rent_income_rule_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rentaff.card.debts">Debt-adjusted</div>
                    <div class="value">${money(r.max_rent_debt_adjusted_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.rentaff.row.income">Monthly income</td><td>${money(r.monthly_income_usd)}</td></tr>
                    <tr><td data-i18n="view.rentaff.row.rule30">30% rule (= 40× annual)</td><td>${money(r.max_rent_income_rule_usd)}</td></tr>
                    <tr><td data-i18n="view.rentaff.row.debtadj">Debt-adjusted cap</td><td>${money(r.max_rent_debt_adjusted_usd)}</td></tr>
                    <tr><td data-i18n="view.rentaff.row.binding">Binding constraint</td><td data-i18n="${bindKey}">—</td></tr>
                    <tr class="emph"><td data-i18n="view.rentaff.row.recommended">Recommended max rent</td><td>${money(r.recommended_max_rent_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
