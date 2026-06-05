// Kiddie Tax § 1(g) Calculator.
// Unearned income above $2,600 (2024 / $2,700 2025) taxed at PARENT'S
// marginal rate. Applies to: under-18, OR full-time student 19-23 whose
// earned income < 50% of support. Defeats the "give appreciated stock
// to kid to sell at 0% LT cap-gains rate" strategy unless kid is grown.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const STANDARD_DEDUCTION_DEPENDENT_2024 = 1_300;  // greater of $1,300 or earned + $450
const UNEARNED_TIER_2024 = 2_600;  // first $1,300 0%, next $1,300 at kid's rate, above at parent's
const STANDARD_DEDUCTION_DEPENDENT_2025 = 1_350;
const UNEARNED_TIER_2025 = 2_700;

const KID_BRACKETS_SINGLE = [
    [11_600, 0.10],
    [47_150, 0.12],
    [100_525, 0.22],
    [191_950, 0.24],
    [Infinity, 0.37],
];

let state = {
    year: new Date().getFullYear(),
    kid_age: 12,
    kid_earned_income: 0,
    kid_unearned_income: 8_000,
    parent_marginal_rate: 0.32,
    parent_lt_cap_gains_rate: 0.20,
};

export async function renderKiddieTax(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.kiddie.h1.title">// KIDDIE TAX § 1(g)</span></h1>
        <p class="muted small" data-i18n="view.kiddie.hint.intro">
            Unearned income &gt; $2,600 (2024) taxed at <strong>parent's marginal rate</strong>.
            Applies to: under-18, OR full-time student 19-23 whose earned income &lt; 50%
            of support. Defeats the classic "gift appreciated stock to kid" strategy
            unless kid is past age threshold or has substantial earned income.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.kiddie.h2.inputs">Inputs</h2>
            <form id="kt-form" class="inline-form">
                <label><span data-i18n="view.kiddie.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.kiddie.label.kid_age">Kid age</span>
                    <input type="number" step="1" name="kid_age" value="${state.kid_age}" min="0" max="25"></label>
                <label><span data-i18n="view.kiddie.label.kid_earned">Kid earned income ($)</span>
                    <input type="number" step="0.01" name="kid_earned_income" value="${state.kid_earned_income}"></label>
                <label><span data-i18n="view.kiddie.label.kid_unearned">Kid unearned income ($)</span>
                    <input type="number" step="0.01" name="kid_unearned_income" value="${state.kid_unearned_income}"></label>
                <label><span data-i18n="view.kiddie.label.parent_rate">Parent's marginal rate %</span>
                    <input type="number" step="0.5" name="parent_marginal_rate" value="${(state.parent_marginal_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.kiddie.label.parent_lt_rate">Parent's LT cap-gains rate %</span>
                    <input type="number" step="0.5" name="parent_lt_cap_gains_rate" value="${(state.parent_lt_cap_gains_rate * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.kiddie.btn.compute">Compute</button>
            </form>
        </div>
        <div id="kt-output"></div>
    `;
    document.getElementById('kt-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.year = Number(fd.get('year'));
        state.kid_age = Number(fd.get('kid_age'));
        state.kid_earned_income = Number(fd.get('kid_earned_income')) || 0;
        state.kid_unearned_income = Number(fd.get('kid_unearned_income')) || 0;
        state.parent_marginal_rate = (Number(fd.get('parent_marginal_rate')) || 32) / 100;
        state.parent_lt_cap_gains_rate = (Number(fd.get('parent_lt_cap_gains_rate')) || 20) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('kt-output');
    if (!el) return;
    const stdDed = state.year >= 2025 ? STANDARD_DEDUCTION_DEPENDENT_2025 : STANDARD_DEDUCTION_DEPENDENT_2024;
    const tier = state.year >= 2025 ? UNEARNED_TIER_2025 : UNEARNED_TIER_2024;
    const half_tier = tier / 2;  // = $1,300 / $1,350

    const kiddieTaxApplies = state.kid_age < 18 ||
        (state.kid_age >= 19 && state.kid_age <= 23);

    // Standard deduction for dependent: greater of $1,300 or earned + $450
    const effStdDed = Math.max(stdDed, state.kid_earned_income + 450);
    const taxableIncome = Math.max(0, state.kid_earned_income + state.kid_unearned_income - effStdDed);

    // First half_tier ($1,300) of unearned offset by std deduction
    // Next half_tier ($1,300) taxed at kid's rate (10%)
    // Above tier ($2,600) of unearned taxed at parent's rate
    const subjectToParentRate = Math.max(0, state.kid_unearned_income - tier);
    const kidsOwnRateAmount = Math.max(0, Math.min(state.kid_unearned_income, tier) - half_tier);
    const offsetByStdDed = Math.min(state.kid_unearned_income, half_tier);
    const kidsTaxOnEarned = computeKidTax(Math.max(0, state.kid_earned_income - effStdDed));
    const kidsTaxOnOwnUnearned = kidsOwnRateAmount * 0.10;
    const parentRateTax = subjectToParentRate * state.parent_marginal_rate;

    const totalKidTax = kiddieTaxApplies
        ? kidsTaxOnEarned + kidsTaxOnOwnUnearned + parentRateTax
        : kidsTaxOnEarned + computeKidTax(state.kid_unearned_income);

    // Compare to parent owning the asset directly (LT cap gains)
    const parentDirectTax = state.kid_unearned_income * state.parent_lt_cap_gains_rate;
    const savingsVsParent = parentDirectTax - totalKidTax;

    el.innerHTML = `
        <div class="chart-panel ${kiddieTaxApplies ? 'neg' : 'pos'}">
            <h2 data-i18n="view.kiddie.h2.applies">Kiddie tax applies?</h2>
            <p style="font-size:1.4em">
                <strong>${kiddieTaxApplies ? esc(t('view.kiddie.status.yes')) : esc(t('view.kiddie.status.no'))}</strong>
            </p>
            <p class="muted small">
                ${kiddieTaxApplies
                    ? esc(t('view.kiddie.status.applies_reason'))
                    : esc(t('view.kiddie.status.exempt_reason'))}
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.kiddie.h2.breakdown">Tax breakdown</h2>
            <table class="trades">
                <tbody>
                    <tr><td data-i18n="view.kiddie.row.std_ded">Standard deduction (dependent)</td>
                        <td>$${effStdDed.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.kiddie.row.offset_by_std_ded">Unearned offset by std deduction</td>
                        <td>$${offsetByStdDed.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.kiddie.row.kids_rate">Unearned at kid's rate (10%)</td>
                        <td>$${kidsOwnRateAmount.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.kiddie.row.parent_rate_amt">Unearned at parent's rate</td>
                        <td>$${subjectToParentRate.toLocaleString()}</td></tr>
                    <tr><td data-i18n="view.kiddie.row.tax_earned">Tax on earned</td>
                        <td class="neg">$${kidsTaxOnEarned.toFixed(2)}</td></tr>
                    <tr><td data-i18n="view.kiddie.row.tax_own">Tax at kid's rate (10%)</td>
                        <td class="neg">$${kidsTaxOnOwnUnearned.toFixed(2)}</td></tr>
                    <tr><td data-i18n="view.kiddie.row.tax_parent">Tax at parent's rate</td>
                        <td class="neg">$${parentRateTax.toFixed(2)}</td></tr>
                    <tr><td><strong data-i18n="view.kiddie.row.total">Total tax</strong></td>
                        <td><strong class="neg">$${totalKidTax.toFixed(2)}</strong></td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.kiddie.h2.comparison">Vs. parent holding the asset</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.kiddie.card.parent_direct">Parent's tax @ LT cap-gains</div>
                    <div class="value">$${parentDirectTax.toFixed(2)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.kiddie.card.kids_total">Kid's total tax</div>
                    <div class="value">$${totalKidTax.toFixed(2)}</div>
                </div>
                <div class="card ${savingsVsParent > 0 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.kiddie.card.savings">Gift-to-kid savings</div>
                    <div class="value">$${savingsVsParent.toFixed(2)}</div>
                </div>
            </div>
            <p class="muted small" data-i18n="view.kiddie.strategy">
                Strategy: gift basis-stepped stock to kids ONLY if they're 18+ AND have
                low income (use 0% LT cap-gains bracket up to $47,025 in 2024). Under 18:
                limit gifts to $2,600/yr of unearned income to stay within kid-rate tier.
                529 plans bypass kiddie tax entirely (growth is tax-free).
            </p>
        </div>
    `;
}

function computeKidTax(taxable) {
    let owe = 0;
    let lastCap = 0;
    for (const [cap, rate] of KID_BRACKETS_SINGLE) {
        const slice = Math.max(0, Math.min(taxable, cap) - lastCap);
        owe += slice * rate;
        if (taxable <= cap) break;
        lastCap = cap;
    }
    return owe;
}
