// IRC § 21 — Child & Dependent Care Credit.
// 2024: 20-35% of $3k (1 child) / $6k (2+) qualifying expenses. Non-refundable.
// AGI > $43k: minimum 20% rate. Care to ENABLE work / job search.
// ARPA 2021 boost (refundable, up to $8k) expired. Form 2441.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EXPENSES_CAP_ONE = 3_000;
const EXPENSES_CAP_MULTIPLE = 6_000;
const MAX_RATE = 0.35;
const MIN_RATE = 0.20;
const RATE_FLOOR_AGI = 43_000;

let state = {
    agi: 0,
    qualifying_individuals: 0,
    qualified_care_expenses: 0,
    earned_income_lower_spouse: 0,
    earned_income_higher_spouse: 0,
    employer_dependent_care_assistance: 0,
    filing_status: 'mfj',
    fed_tax_liability_before_credits: 0,
};

export async function renderSection21Cdcc(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cdcc.h1.title">// § 21 CHILD & DEPENDENT CARE CREDIT</span></h1>
        <p class="muted small" data-i18n="view.cdcc.hint.intro">
            2024: <strong>20-35% of $3k (1 child) / $6k (2+) qualifying expenses</strong>.
            <strong>Non-refundable</strong>. AGI &gt; $43k: minimum 20% rate. Care must ENABLE
            taxpayer (+ spouse if MFJ) to work or job-search. <strong>ARPA 2021 boost expired</strong>
            (was refundable, up to $8k). § 129 Dependent Care FSA $5,000 alternative + can use both
            (offset rule). <strong>Form 2441</strong>.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.cdcc.h2.inputs">Inputs</h2>
            <form id="cdcc-form" class="inline-form">
                <label><span data-i18n="view.cdcc.label.agi">AGI ($)</span>
                    <input type="number" step="1000" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.cdcc.label.qual_count">Qualifying individuals</span>
                    <input type="number" step="1" name="qualifying_individuals" value="${state.qualifying_individuals}"></label>
                <label><span data-i18n="view.cdcc.label.care_expenses">Qualified care expenses ($)</span>
                    <input type="number" step="100" name="qualified_care_expenses" value="${state.qualified_care_expenses}"></label>
                <label><span data-i18n="view.cdcc.label.lower_earned">Lower-earned spouse income ($)</span>
                    <input type="number" step="1000" name="earned_income_lower_spouse" value="${state.earned_income_lower_spouse}"></label>
                <label><span data-i18n="view.cdcc.label.higher_earned">Higher-earned spouse income ($)</span>
                    <input type="number" step="1000" name="earned_income_higher_spouse" value="${state.earned_income_higher_spouse}"></label>
                <label><span data-i18n="view.cdcc.label.employer">Employer-provided dep care ($)</span>
                    <input type="number" step="100" name="employer_dependent_care_assistance" value="${state.employer_dependent_care_assistance}"></label>
                <label><span data-i18n="view.cdcc.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.cdcc.label.tax_liability">Tax liability before credits ($)</span>
                    <input type="number" step="100" name="fed_tax_liability_before_credits" value="${state.fed_tax_liability_before_credits}"></label>
                <button class="primary" type="submit" data-i18n="view.cdcc.btn.compute">Compute</button>
            </form>
        </div>
        <div id="cdcc-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.cdcc.h2.qualifying_individuals">Qualifying individuals</h2>
            <ul class="muted small">
                <li data-i18n="view.cdcc.qi.child">Dependent child under age 13 when care provided</li>
                <li data-i18n="view.cdcc.qi.spouse">Spouse mentally / physically unable to care for self + lives with you &gt; half year</li>
                <li data-i18n="view.cdcc.qi.dependent_disabled">Dependent mentally / physically unable to care for self + lives with you &gt; half year</li>
                <li data-i18n="view.cdcc.qi.identifying">Identification # (SSN / ITIN / EIN) of care provider required</li>
                <li data-i18n="view.cdcc.qi.qualifying_provider">Provider must NOT be your spouse / parent of qualifying child / your other dependent</li>
                <li data-i18n="view.cdcc.qi.location">Care at provider's home OR yours; outside home OK</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.cdcc.h2.qualifying_expenses">Qualifying expenses</h2>
            <ul class="muted small">
                <li data-i18n="view.cdcc.qe.day_care">Day care center / preschool</li>
                <li data-i18n="view.cdcc.qe.au_pair">Au pair, nanny (employer-employee, you must pay SS + Medicare)</li>
                <li data-i18n="view.cdcc.qe.summer_camp">Day summer camp (not overnight)</li>
                <li data-i18n="view.cdcc.qe.transport">Transportation TO daycare (provider-to-provider)</li>
                <li data-i18n="view.cdcc.qe.qualifying_household">Household services (housekeeping that includes care)</li>
                <li data-i18n="view.cdcc.qe.before_after">Before / after school programs</li>
                <li data-i18n="view.cdcc.qe.special_needs">Special-needs care</li>
                <li data-i18n="view.cdcc.qe.not_school">NOT tuition for kindergarten + above (schoolwork)</li>
                <li data-i18n="view.cdcc.qe.not_food">NOT food / clothing / entertainment</li>
                <li data-i18n="view.cdcc.qe.not_overnight">NOT overnight camps</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.cdcc.h2.dependent_care_fsa">§ 129 Dependent Care FSA coordination</h2>
            <p class="muted small" data-i18n="view.cdcc.dcfsa.body">
                Employer Dependent Care FSA: pre-tax up to <strong>$5,000 single/MFJ
                ($2,500 MFS)</strong>. Reduces salary subject to FICA + income tax.
                <strong>Coordination:</strong> § 21 expense cap REDUCED by FSA used (no double-dip).
                Strategy: if high earners + 2+ kids, max FSA first; remaining $1k+ expenses ($6k - $5k)
                for credit. Lower earners benefit more from § 21 (35% rate).
            </p>
        </div>
    `;
    document.getElementById('cdcc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.agi = Number(fd.get('agi')) || 0;
        state.qualifying_individuals = Number(fd.get('qualifying_individuals')) || 0;
        state.qualified_care_expenses = Number(fd.get('qualified_care_expenses')) || 0;
        state.earned_income_lower_spouse = Number(fd.get('earned_income_lower_spouse')) || 0;
        state.earned_income_higher_spouse = Number(fd.get('earned_income_higher_spouse')) || 0;
        state.employer_dependent_care_assistance = Number(fd.get('employer_dependent_care_assistance')) || 0;
        state.filing_status = fd.get('filing_status');
        state.fed_tax_liability_before_credits = Number(fd.get('fed_tax_liability_before_credits')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('cdcc-output');
    if (!el) return;
    const expenseCap = state.qualifying_individuals === 1 ? EXPENSES_CAP_ONE : EXPENSES_CAP_MULTIPLE;
    const reducedCap = Math.max(0, expenseCap - state.employer_dependent_care_assistance);
    const lowerEarned = state.filing_status === 'mfj'
        ? Math.min(state.earned_income_lower_spouse, state.earned_income_higher_spouse)
        : state.earned_income_higher_spouse;
    const effectiveExpenses = Math.min(state.qualified_care_expenses, reducedCap, lowerEarned);
    const rate = state.agi <= 15_000
        ? 0.35
        : Math.max(MIN_RATE, MAX_RATE - Math.floor((state.agi - 15_000) / 2_000) * 0.01);
    const credit = effectiveExpenses * rate;
    const finalCredit = Math.min(credit, state.fed_tax_liability_before_credits);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.cdcc.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.cdcc.card.cap">Expense cap</div>
                    <div class="value">$${expenseCap.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.cdcc.card.fsa_reduction">FSA offset</div>
                    <div class="value">$${state.employer_dependent_care_assistance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.cdcc.card.expenses_used">Effective expenses</div>
                    <div class="value">$${effectiveExpenses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.cdcc.card.rate">Applicable rate</div>
                    <div class="value">${(rate * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.cdcc.card.credit">Credit (non-refundable)</div>
                    <div class="value">$${finalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
