// IRC § 529 — Qualified Tuition Programs.
// Tax-free growth + tax-free withdrawals for qualified education expenses.
// 5-year super-funding: $90k single / $180k MFJ at once (counted ratably). State deductions vary.
// Post-2018: $10k/yr K-12 tuition. Post-2024: Roth IRA rollover up to $35k lifetime (15-yr account).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SUPER_FUNDING_SINGLE = 90_000;
const SUPER_FUNDING_MFJ = 180_000;
const ANNUAL_EXCLUSION_2024 = 18_000;
const K12_LIMIT_2018 = 10_000;
const ROTH_ROLLOVER_LIMIT = 35_000;

const STATE_DEDUCTIONS = {
    NY: { max_single: 5_000, max_mfj: 10_000, contributions: ['ny_only'] },
    PA: { max_single: 18_000, max_mfj: 36_000, contributions: ['any'] },
    AZ: { max_single: 2_000, max_mfj: 4_000, contributions: ['any'] },
    KS: { max_single: 3_000, max_mfj: 6_000, contributions: ['any'] },
    MN: { max_single: 1_500, max_mfj: 3_000, contributions: ['any'] },
    MO: { max_single: 8_000, max_mfj: 16_000, contributions: ['any'] },
    GA: { max_single: 4_000, max_mfj: 8_000, contributions: ['ga_only'] },
    IL: { max_single: 10_000, max_mfj: 20_000, contributions: ['il_only'] },
    OH: { max_single: 4_000, max_mfj: 4_000, contributions: ['oh_only'] },
};

let state = {
    state_of_residence: 'NY',
    contribution_amount: 0,
    super_fund: false,
    is_mfj: false,
    annual_distributions: 0,
    qualified_expenses: 0,
    k12_distributions: 0,
    is_in_state_plan: true,
    account_age_years: 0,
    intend_roth_rollover: false,
    fed_marginal_rate: 0.32,
    state_marginal_rate: 0.06,
};

export async function renderSection529(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s529.h1.title">// § 529 QUALIFIED TUITION PLAN</span></h1>
        <p class="muted small" data-i18n="view.s529.hint.intro">
            Tax-free growth + tax-free withdrawals for qualified education expenses.
            <strong>5-year super-funding:</strong> $90k single / $180k MFJ at once. State deductions vary —
            check residence-only vs any-plan. <strong>$10k/yr K-12 tuition</strong> qualified
            (since 2018). <strong>Post-2024 Roth IRA rollover</strong> up to $35k lifetime
            for accounts ≥ 15 yrs old.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s529.h2.inputs">Inputs</h2>
            <form id="s529-form" class="inline-form">
                <label><span data-i18n="view.s529.label.state">State of residence</span>
                    <select name="state_of_residence">
                        ${Object.keys(STATE_DEDUCTIONS).map(s => `<option value="${s}" ${state.state_of_residence === s ? 'selected' : ''}>${s}</option>`).join('')}
                        <option value="other" ${state.state_of_residence === 'other' ? 'selected' : ''}>Other / no deduction</option>
                    </select>
                </label>
                <label><span data-i18n="view.s529.label.contribution">Annual contribution ($)</span>
                    <input type="number" step="1000" name="contribution_amount" value="${state.contribution_amount}"></label>
                <label><span data-i18n="view.s529.label.super">5-year super-fund?</span>
                    <input type="checkbox" name="super_fund" ${state.super_fund ? 'checked' : ''}></label>
                <label><span data-i18n="view.s529.label.mfj">MFJ?</span>
                    <input type="checkbox" name="is_mfj" ${state.is_mfj ? 'checked' : ''}></label>
                <label><span data-i18n="view.s529.label.dist">Total distributions ($)</span>
                    <input type="number" step="100" name="annual_distributions" value="${state.annual_distributions}"></label>
                <label><span data-i18n="view.s529.label.qualified">Qualified higher-ed expenses ($)</span>
                    <input type="number" step="100" name="qualified_expenses" value="${state.qualified_expenses}"></label>
                <label><span data-i18n="view.s529.label.k12">K-12 distributions ($)</span>
                    <input type="number" step="100" name="k12_distributions" value="${state.k12_distributions}"></label>
                <label><span data-i18n="view.s529.label.in_state">In-state plan?</span>
                    <input type="checkbox" name="is_in_state_plan" ${state.is_in_state_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s529.label.age">Account age (years)</span>
                    <input type="number" step="1" name="account_age_years" value="${state.account_age_years}"></label>
                <label><span data-i18n="view.s529.label.roth_rollover">Plan Roth IRA rollover?</span>
                    <input type="checkbox" name="intend_roth_rollover" ${state.intend_roth_rollover ? 'checked' : ''}></label>
                <label><span data-i18n="view.s529.label.fed_rate">Federal marginal %</span>
                    <input type="number" step="0.01" name="fed_marginal_rate" value="${state.fed_marginal_rate}"></label>
                <label><span data-i18n="view.s529.label.state_rate">State marginal %</span>
                    <input type="number" step="0.01" name="state_marginal_rate" value="${state.state_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s529.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s529-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s529.h2.qualified">Qualified expenses</h2>
            <ul class="muted small">
                <li data-i18n="view.s529.qual.tuition">Tuition + required fees at eligible institution</li>
                <li data-i18n="view.s529.qual.books">Required books + supplies + equipment</li>
                <li data-i18n="view.s529.qual.computers">Computers + peripheral equipment + internet (since 2015)</li>
                <li data-i18n="view.s529.qual.room_board">Room + board (if half-time student, not exceeding school allowance)</li>
                <li data-i18n="view.s529.qual.special_needs">Special-needs services for special-needs beneficiary</li>
                <li data-i18n="view.s529.qual.apprenticeship">Apprenticeship program expenses (since SECURE Act)</li>
                <li data-i18n="view.s529.qual.student_loan">Up to $10k lifetime to student loan repayment (since 2019)</li>
                <li data-i18n="view.s529.qual.k12">$10k/yr K-12 tuition (since 2018) — federal qualified</li>
                <li data-i18n="view.s529.qual.elementary">NOTE: K-12 may NOT be state-qualified (only 30 states recognize)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s529.h2.secure_2_0">SECURE 2.0 Roth rollover</h2>
            <p class="muted small" data-i18n="view.s529.secure.body">
                Effective 2024: 529 account → Roth IRA rollover allowed when (1) account ≥ 15 yrs old,
                (2) within annual Roth contribution limit, (3) recipient must be 529 beneficiary,
                (4) cumulative $35,000 lifetime cap, (5) rolling over CONTRIBUTIONS (not earnings) made within 5 yrs prior NOT eligible.
                Eliminates over-funding concerns. Federal-only — state may not conform.
            </p>
        </div>
    `;
    document.getElementById('s529-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.state_of_residence = fd.get('state_of_residence');
        state.contribution_amount = Number(fd.get('contribution_amount')) || 0;
        state.super_fund = !!fd.get('super_fund');
        state.is_mfj = !!fd.get('is_mfj');
        state.annual_distributions = Number(fd.get('annual_distributions')) || 0;
        state.qualified_expenses = Number(fd.get('qualified_expenses')) || 0;
        state.k12_distributions = Number(fd.get('k12_distributions')) || 0;
        state.is_in_state_plan = !!fd.get('is_in_state_plan');
        state.account_age_years = Number(fd.get('account_age_years')) || 0;
        state.intend_roth_rollover = !!fd.get('intend_roth_rollover');
        state.fed_marginal_rate = Number(fd.get('fed_marginal_rate')) || 0.32;
        state.state_marginal_rate = Number(fd.get('state_marginal_rate')) || 0.06;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s529-output');
    if (!el) return;
    const annualExc = state.is_mfj ? ANNUAL_EXCLUSION_2024 * 2 : ANNUAL_EXCLUSION_2024;
    const superCap = state.is_mfj ? SUPER_FUNDING_MFJ : SUPER_FUNDING_SINGLE;
    const effContribLimit = state.super_fund ? superCap : annualExc;
    const overAnnual = Math.max(0, state.contribution_amount - annualExc);
    const stateRules = STATE_DEDUCTIONS[state.state_of_residence] || { max_single: 0, max_mfj: 0, contributions: ['none'] };
    const stateMax = state.is_mfj ? stateRules.max_mfj : stateRules.max_single;
    const stateDeductibleContribution = state.is_in_state_plan || stateRules.contributions.includes('any')
        ? Math.min(state.contribution_amount, stateMax)
        : 0;
    const fedSavings = 0;  // 529 is federally not deductible
    const stateSavings = stateDeductibleContribution * state.state_marginal_rate;
    const totalDist = state.annual_distributions;
    const k12Cap = Math.min(state.k12_distributions, K12_LIMIT_2018);
    const totalQualified = state.qualified_expenses + k12Cap;
    const nonQualifiedDist = Math.max(0, totalDist - totalQualified);
    const earningsRatio = totalDist > 0 ? 0.30 : 0;
    const taxableEarnings = nonQualifiedDist * earningsRatio;
    const fedTaxOnNonQ = taxableEarnings * state.fed_marginal_rate;
    const tenPctPenalty = taxableEarnings * 0.10;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s529.h2.result">§ 529 outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s529.card.limit">Contribution limit</div>
                    <div class="value">$${effContribLimit.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s529.card.state_max">State deduction max</div>
                    <div class="value">$${stateMax.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s529.card.state_savings">State tax savings</div>
                    <div class="value">$${stateSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${overAnnual > 0 && !state.super_fund ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s529.card.gift_over">Gift over $18k (Form 709)</div>
                        <div class="value">$${overAnnual.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card">
                    <div class="label" data-i18n="view.s529.card.qualified_dist">Qualified distributions</div>
                    <div class="value">$${totalQualified.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${nonQualifiedDist > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s529.card.non_qualified">Non-qualified distributions</div>
                    <div class="value">$${nonQualifiedDist.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s529.card.fed_tax">Fed tax on earnings</div>
                    <div class="value">$${fedTaxOnNonQ.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s529.card.penalty">10% penalty on earnings</div>
                    <div class="value">$${tenPctPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.intend_roth_rollover && state.account_age_years >= 15 ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.s529.card.roth_rollover">Roth IRA rollover available</div>
                        <div class="value">$${ROTH_ROLLOVER_LIMIT.toLocaleString()}</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
