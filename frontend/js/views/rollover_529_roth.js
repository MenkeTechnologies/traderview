// 529 → Roth IRA Rollover Planner — SECURE 2.0 § 126 (2024+).
// $35,000 LIFETIME per beneficiary, annual Roth IRA contribution limits
// each year, 529 must be 15+ years old, beneficiary must have earned income
// equal to or exceeding the rollover amount that year.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const LIFETIME_CAP = 35_000;
const ROTH_LIMITS = {
    2024: { regular: 7_000, catchup_50: 1_000 },
    2025: { regular: 7_000, catchup_50: 1_000 },
    2026: { regular: 7_500, catchup_50: 1_000 },
};
const MIN_529_AGE_YEARS = 15;

let state = {
    beneficiary_age: 22,
    plan_open_date: '2010-06-15',
    current_529_balance: 50_000,
    beneficiary_earned_income: 30_000,
    rollovers_to_date: 0,
    target_start_year: new Date().getFullYear(),
};

export async function renderRollover529Roth(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.529roth.h1.title">// 529 → ROTH ROLLOVER</span></h1>
        <p class="muted small" data-i18n="view.529roth.hint.intro">
            <strong>SECURE 2.0 (2024+):</strong> roll up to $35,000 lifetime from a 529 plan
            to a Roth IRA in the BENEFICIARY's name. 529 must be 15+ years old. Annual
            cap = Roth limit ($7,000 in 2024). Beneficiary must have earned income equal
            to or exceeding the rollover that year. Solves the "what if my kid doesn't
            need 529" problem.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.529roth.h2.inputs">Inputs</h2>
            <form id="rr-form" class="inline-form">
                <label><span data-i18n="view.529roth.label.beneficiary_age">Beneficiary age</span>
                    <input type="number" step="1" name="beneficiary_age" value="${state.beneficiary_age}" min="0" max="100"></label>
                <label><span data-i18n="view.529roth.label.plan_open_date">529 plan open date</span>
                    <input type="date" name="plan_open_date" value="${state.plan_open_date}"></label>
                <label><span data-i18n="view.529roth.label.current_529_balance">Current 529 balance ($)</span>
                    <input type="number" step="1000" name="current_529_balance" value="${state.current_529_balance}"></label>
                <label><span data-i18n="view.529roth.label.earned_income">Beneficiary annual earned income ($)</span>
                    <input type="number" step="1000" name="beneficiary_earned_income" value="${state.beneficiary_earned_income}"></label>
                <label><span data-i18n="view.529roth.label.rollovers_to_date">Rollovers already done ($)</span>
                    <input type="number" step="100" name="rollovers_to_date" value="${state.rollovers_to_date}"></label>
                <label><span data-i18n="view.529roth.label.target_year">Target start year</span>
                    <input type="number" step="1" name="target_start_year" value="${state.target_start_year}"></label>
                <button class="primary" type="submit" data-i18n="view.529roth.btn.plan">Plan</button>
            </form>
        </div>
        <div id="rr-output"></div>
    `;
    document.getElementById('rr-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.beneficiary_age = Number(fd.get('beneficiary_age'));
        state.plan_open_date = fd.get('plan_open_date');
        state.current_529_balance = Number(fd.get('current_529_balance')) || 0;
        state.beneficiary_earned_income = Number(fd.get('beneficiary_earned_income')) || 0;
        state.rollovers_to_date = Number(fd.get('rollovers_to_date')) || 0;
        state.target_start_year = Number(fd.get('target_start_year'));
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('rr-output');
    if (!el) return;
    const planOpenDate = new Date(state.plan_open_date);
    const fifteenYearDate = new Date(planOpenDate);
    fifteenYearDate.setFullYear(fifteenYearDate.getFullYear() + MIN_529_AGE_YEARS);
    const today = new Date();
    const planAgeYrs = (today - planOpenDate) / (365.25 * 86_400_000);
    const eligible = planAgeYrs >= MIN_529_AGE_YEARS;
    const lifetimeRemaining = Math.max(0, LIFETIME_CAP - state.rollovers_to_date);

    // Project a multi-year ladder
    const ladder = [];
    let cumulative = state.rollovers_to_date;
    for (let y = 0; y < 8; y++) {
        const year = state.target_start_year + y;
        const limits = ROTH_LIMITS[year] || ROTH_LIMITS[2024];
        const annualCap = limits.regular + (state.beneficiary_age + y >= 50 ? limits.catchup_50 : 0);
        const remainingLifetime = Math.max(0, LIFETIME_CAP - cumulative);
        const annualRolloverPossible = Math.min(annualCap, remainingLifetime, state.beneficiary_earned_income);
        cumulative += annualRolloverPossible;
        ladder.push({
            year,
            annual_cap: annualCap,
            remaining_lifetime: remainingLifetime,
            rollover_amount: annualRolloverPossible,
            cumulative,
        });
        if (cumulative >= LIFETIME_CAP) break;
    }
    const totalRollover = ladder.reduce((s, r) => s + r.rollover_amount, 0);
    el.innerHTML = `
        <div class="chart-panel ${eligible ? 'pos' : 'neg'}">
            <h2 data-i18n="view.529roth.h2.eligibility">Eligibility</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.529roth.card.plan_age">529 plan age</div>
                    <div class="value">${planAgeYrs.toFixed(1)} ${esc(t('view.529roth.years'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.529roth.card.eligible_date">Eligible after</div>
                    <div class="value">${esc(fifteenYearDate.toISOString().slice(0, 10))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.529roth.card.lifetime_remaining">Lifetime remaining</div>
                    <div class="value">$${lifetimeRemaining.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.529roth.card.total_rollover">Total projected rollover</div>
                    <div class="value">$${totalRollover.toLocaleString()}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.529roth.h2.ladder">Rollover ladder</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.529roth.th.year">Year</th>
                    <th data-i18n="view.529roth.th.annual_cap">Annual cap</th>
                    <th data-i18n="view.529roth.th.remaining">Lifetime remaining</th>
                    <th data-i18n="view.529roth.th.rollover_amount">Rollover</th>
                    <th data-i18n="view.529roth.th.cumulative">Cumulative</th>
                </tr></thead>
                <tbody>${ladder.map(r => `
                    <tr>
                        <td>${r.year}</td>
                        <td>$${r.annual_cap.toLocaleString()}</td>
                        <td>$${r.remaining_lifetime.toLocaleString()}</td>
                        <td class="pos">$${r.rollover_amount.toLocaleString()}</td>
                        <td>$${r.cumulative.toLocaleString()}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.529roth.h2.requirements">Requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.529roth.req.15_year">529 plan must be 15+ years old (oldest version of the plan, NOT the beneficiary)</li>
                <li data-i18n="view.529roth.req.beneficiary">Beneficiary of 529 = owner of receiving Roth IRA (same person)</li>
                <li data-i18n="view.529roth.req.earned_income">Beneficiary must have earned income ≥ rollover amount that year</li>
                <li data-i18n="view.529roth.req.5_year_seasoning">Contributions in last 5 years CANNOT be rolled (5-year seasoning per contribution)</li>
                <li data-i18n="view.529roth.req.no_income_cap">Roth income phase-out does NOT apply to this rollover (huge for high earners!)</li>
                <li data-i18n="view.529roth.req.no_change_beneficiary">Changing beneficiary may reset the 15-year clock (IRS guidance pending)</li>
            </ol>
        </div>
    `;
}
