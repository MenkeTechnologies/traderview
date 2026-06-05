// IRC § 4980H — ACA Employer Mandate (Play-or-Pay).
// Applicable Large Employer (ALE) = avg 50+ FTE in prior calendar year.
// (a) Failure to offer = $2,970 (2024) × ALL FTEs minus 30, if ANY employee gets PTC.
// (b) Failure to offer AFFORDABLE / minimum value = $4,460 (2024) per non-covered FTE that gets PTC.
// Affordability: 8.39% of household income (2024). Form 1095-C required.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const A_PENALTY_2024 = 2_970;
const B_PENALTY_2024 = 4_460;
const ALE_THRESHOLD = 50;
const AFFORDABILITY_PCT_2024 = 0.0839;
const EXCLUDED_FTES = 30;

let state = {
    avg_fte_prior_year: 0,
    offers_coverage: false,
    offers_minimum_value: false,
    offers_affordable: false,
    coverage_offered_to_fte_count: 0,
    employees_getting_ptc: 0,
    lowest_cost_plan_monthly: 0,
    avg_employee_household_income_monthly: 0,
};

export async function renderSection4980h(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4980h.h1.title">// § 4980H ACA EMPLOYER MANDATE</span></h1>
        <p class="muted small" data-i18n="view.s4980h.hint.intro">
            <strong>Applicable Large Employer (ALE)</strong> = 50+ FTE in prior calendar year.
            <strong>(a) Penalty:</strong> $2,970 × ALL FTEs minus 30 if ANY employee gets PTC and
            you offer to &lt; 95% of FTEs. <strong>(b) Penalty:</strong> $4,460 per non-covered FTE
            that gets PTC when offered NOT minimum value / NOT affordable. <strong>Affordability:</strong>
            employee share ≤ 8.39% of household income (W-2 safe harbor, FPL safe harbor available).
            Form 1095-C / 1094-C required for each employee + transmittal.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4980h.h2.inputs">Inputs</h2>
            <form id="s4980h-form" class="inline-form">
                <label><span data-i18n="view.s4980h.label.fte">Avg FTE prior year</span>
                    <input type="number" step="1" name="avg_fte_prior_year" value="${state.avg_fte_prior_year}"></label>
                <label><span data-i18n="view.s4980h.label.offers">Offers coverage to ≥ 95% FTE?</span>
                    <input type="checkbox" name="offers_coverage" ${state.offers_coverage ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980h.label.mv">Minimum value (60% actuarial)?</span>
                    <input type="checkbox" name="offers_minimum_value" ${state.offers_minimum_value ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980h.label.affordable">Affordable?</span>
                    <input type="checkbox" name="offers_affordable" ${state.offers_affordable ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4980h.label.covered_count">Covered FTE count</span>
                    <input type="number" step="1" name="coverage_offered_to_fte_count" value="${state.coverage_offered_to_fte_count}"></label>
                <label><span data-i18n="view.s4980h.label.ptc_count">Employees getting PTC</span>
                    <input type="number" step="1" name="employees_getting_ptc" value="${state.employees_getting_ptc}"></label>
                <label><span data-i18n="view.s4980h.label.lowest_monthly">Lowest cost plan monthly ($)</span>
                    <input type="number" step="0.01" name="lowest_cost_plan_monthly" value="${state.lowest_cost_plan_monthly}"></label>
                <label><span data-i18n="view.s4980h.label.income_monthly">Avg employee monthly income ($)</span>
                    <input type="number" step="0.01" name="avg_employee_household_income_monthly" value="${state.avg_employee_household_income_monthly}"></label>
                <button class="primary" type="submit" data-i18n="view.s4980h.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4980h-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4980h.h2.safe_harbors">Affordability safe harbors</h2>
            <ul class="muted small">
                <li data-i18n="view.s4980h.sh.w2">W-2 safe harbor: lowest plan ≤ 8.39% of Box 1 W-2 wages</li>
                <li data-i18n="view.s4980h.sh.rop">Rate-of-pay: lowest plan ≤ 8.39% of hourly × 130 (monthly)</li>
                <li data-i18n="view.s4980h.sh.fpl">FPL: lowest plan ≤ 8.39% × FPL (single household)</li>
                <li data-i18n="view.s4980h.sh.choose">Choose safest based on workforce composition</li>
                <li data-i18n="view.s4980h.sh.combine">Different safe harbor per "reasonable category" of employees allowed</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4980h.h2.fte_calc">FTE calculation</h2>
            <ul class="muted small">
                <li data-i18n="view.s4980h.fte.30_hour">FTE = avg ≥ 30 hours/week (or 130 hours/month)</li>
                <li data-i18n="view.s4980h.fte.pt_calc">Part-time: total hours ÷ 120 = FTE equivalent</li>
                <li data-i18n="view.s4980h.fte.seasonal">Seasonal employees ≤ 120 days: excluded</li>
                <li data-i18n="view.s4980h.fte.measurement">Use look-back measurement (3-12 mo) + stability (6-12 mo) for variable hour</li>
                <li data-i18n="view.s4980h.fte.controlled">Controlled group / affiliated service group: aggregate FTEs across all entities</li>
                <li data-i18n="view.s4980h.fte.foreign">Foreign workers: excluded (no US source income)</li>
            </ul>
        </div>
    `;
    document.getElementById('s4980h-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.avg_fte_prior_year = Number(fd.get('avg_fte_prior_year')) || 0;
        state.offers_coverage = !!fd.get('offers_coverage');
        state.offers_minimum_value = !!fd.get('offers_minimum_value');
        state.offers_affordable = !!fd.get('offers_affordable');
        state.coverage_offered_to_fte_count = Number(fd.get('coverage_offered_to_fte_count')) || 0;
        state.employees_getting_ptc = Number(fd.get('employees_getting_ptc')) || 0;
        state.lowest_cost_plan_monthly = Number(fd.get('lowest_cost_plan_monthly')) || 0;
        state.avg_employee_household_income_monthly = Number(fd.get('avg_employee_household_income_monthly')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4980h-output');
    if (!el) return;
    const isAle = state.avg_fte_prior_year >= ALE_THRESHOLD;
    let aPenalty = 0;
    let bPenalty = 0;
    if (isAle) {
        const offeredPct = state.avg_fte_prior_year > 0
            ? state.coverage_offered_to_fte_count / state.avg_fte_prior_year
            : 0;
        const offers95pct = offeredPct >= 0.95;
        const anyPtc = state.employees_getting_ptc > 0;
        if (!offers95pct && anyPtc) {
            aPenalty = A_PENALTY_2024 * Math.max(0, state.avg_fte_prior_year - EXCLUDED_FTES);
        } else if (offers95pct && anyPtc) {
            const isAffordable = state.avg_employee_household_income_monthly > 0
                ? state.lowest_cost_plan_monthly / state.avg_employee_household_income_monthly <= AFFORDABILITY_PCT_2024
                : true;
            const meetsMV = state.offers_minimum_value;
            const triggers_b = !isAffordable || !meetsMV;
            if (triggers_b) bPenalty = B_PENALTY_2024 * state.employees_getting_ptc;
        }
    }
    const totalPenalty = aPenalty + bPenalty;
    const affordabilityRatio = state.avg_employee_household_income_monthly > 0
        ? state.lowest_cost_plan_monthly / state.avg_employee_household_income_monthly
        : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4980h.h2.result">Mandate analysis</h2>
            <div class="cards">
                <div class="card ${isAle ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4980h.card.is_ale">Is ALE?</div>
                    <div class="value">${isAle ? esc(t('view.s4980h.status.yes')) : esc(t('view.s4980h.status.no'))}</div>
                </div>
                <div class="card ${affordabilityRatio > AFFORDABILITY_PCT_2024 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4980h.card.affordability">Affordability %</div>
                    <div class="value">${(affordabilityRatio * 100).toFixed(2)}%</div>
                </div>
                <div class="card ${aPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4980h.card.a_penalty">(a) Penalty</div>
                    <div class="value">$${aPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${bPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4980h.card.b_penalty">(b) Penalty</div>
                    <div class="value">$${bPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${totalPenalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4980h.card.total">Total annual penalty</div>
                    <div class="value">$${totalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
