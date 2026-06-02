// IRC § 127 — Educational Assistance Programs.
// Employer can provide up to $5,250/year tax-free for qualifying education to employee.
// Through 12/31/2025: includes employer payments toward student loans (SECURE 2.0 / CARES Act).
// Excludes courses involving sports, games, hobbies (unless job-related).
// Non-discrimination + written plan required.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EXCLUSION_LIMIT = 5_250;

let state = {
    educational_assistance_received: 0,
    student_loan_payments_received: 0,
    total_other_employees_received: 0,
    is_owner_5_pct: false,
    employer_has_written_plan: true,
    has_education_assistance_program_qualifies: true,
    course_related_to_job: true,
    is_undergraduate_grad: true,
    fed_marginal_rate: 0.32,
    state_marginal_rate: 0.06,
    fica_rate: 0.0765,
};

export async function renderSection127(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s127.h1.title">// § 127 EDUCATIONAL ASSISTANCE</span></h1>
        <p class="muted small" data-i18n="view.s127.hint.intro">
            Employer can provide up to <strong>$5,250/year tax-free</strong> for qualifying education.
            <strong>Through 12/31/2025</strong>: includes employer payments toward student loans
            (SECURE 2.0 / CARES Act). Excludes sports, games, hobbies (unless job-related).
            <strong>Non-discrimination + written plan required</strong>. Above $5,250 generally
            taxable unless qualifies as "working condition fringe" § 132(d).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s127.h2.inputs">Inputs</h2>
            <form id="s127-form" class="inline-form">
                <label><span data-i18n="view.s127.label.assistance">Educational assistance received ($)</span>
                    <input type="number" step="100" name="educational_assistance_received" value="${state.educational_assistance_received}"></label>
                <label><span data-i18n="view.s127.label.loans">Student loan payments by employer ($)</span>
                    <input type="number" step="100" name="student_loan_payments_received" value="${state.student_loan_payments_received}"></label>
                <label><span data-i18n="view.s127.label.other_received">Other employees received total ($)</span>
                    <input type="number" step="1000" name="total_other_employees_received" value="${state.total_other_employees_received}"></label>
                <label><span data-i18n="view.s127.label.owner">Own &gt; 5% of business?</span>
                    <input type="checkbox" name="is_owner_5_pct" ${state.is_owner_5_pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s127.label.written">Employer has written plan?</span>
                    <input type="checkbox" name="employer_has_written_plan" ${state.employer_has_written_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s127.label.qualifies">Plan qualifies under § 127?</span>
                    <input type="checkbox" name="has_education_assistance_program_qualifies" ${state.has_education_assistance_program_qualifies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s127.label.related">Course related to job?</span>
                    <input type="checkbox" name="course_related_to_job" ${state.course_related_to_job ? 'checked' : ''}></label>
                <label><span data-i18n="view.s127.label.undergrad">Undergraduate or graduate?</span>
                    <input type="checkbox" name="is_undergraduate_grad" ${state.is_undergraduate_grad ? 'checked' : ''}></label>
                <label><span data-i18n="view.s127.label.fed_rate">Federal marginal %</span>
                    <input type="number" step="0.01" name="fed_marginal_rate" value="${state.fed_marginal_rate}"></label>
                <label><span data-i18n="view.s127.label.state_rate">State marginal %</span>
                    <input type="number" step="0.01" name="state_marginal_rate" value="${state.state_marginal_rate}"></label>
                <label><span data-i18n="view.s127.label.fica">FICA rate (employee + employer split)</span>
                    <input type="number" step="0.0001" name="fica_rate" value="${state.fica_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s127.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s127-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s127.h2.qualifying">Qualifying education</h2>
            <ul class="muted small">
                <li data-i18n="view.s127.qual.tuition">Tuition + fees at any educational institution</li>
                <li data-i18n="view.s127.qual.books">Books + supplies + equipment</li>
                <li data-i18n="view.s127.qual.undergrad_grad">Both undergraduate + graduate courses qualify</li>
                <li data-i18n="view.s127.qual.related_no_required">Job-related NOT required (broader than § 132(d) working condition)</li>
                <li data-i18n="view.s127.qual.student_loan_2026">Student loan payments through 12/31/2025 (subject to congressional extension)</li>
                <li data-i18n="view.s127.qual.sports_no">NOT sports / games / hobbies (unless job-related)</li>
                <li data-i18n="view.s127.qual.room_board_no">NOT room + board / transportation / personal use property</li>
                <li data-i18n="view.s127.qual.meals_no">NOT meals + insurance + housing</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s127.h2.nondiscrim">Non-discrimination rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s127.nd.no_more_5pct">No more than 5% can go to &gt; 5% owners + relatives</li>
                <li data-i18n="view.s127.nd.eligibility">Eligibility must not favor highly compensated</li>
                <li data-i18n="view.s127.nd.written_plan">Written plan required: must be reasonable + reduce taxes</li>
                <li data-i18n="view.s127.nd.notice">Reasonable notice to employees about plan benefits</li>
                <li data-i18n="view.s127.nd.no_choice">No choice between cash + benefit (or it becomes salary)</li>
                <li data-i18n="view.s127.nd.section_132_overlap">Above $5,250 can be excluded under § 132(d) working condition if job-related</li>
                <li data-i18n="view.s127.nd.section_117">§ 117 qualified scholarship overlap allowed (tuition reduction for university employees)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s127.h2.related">Related education benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.s127.rel.117">§ 117 Qualified Scholarship: tax-free if degree candidate + tuition portion</li>
                <li data-i18n="view.s127.rel.132d">§ 132(d) Working Condition: job-related education (no $ limit, but must be necessary)</li>
                <li data-i18n="view.s127.rel.221">§ 221 Student Loan Interest deduction: $2,500/yr cap, MAGI phase-out</li>
                <li data-i18n="view.s127.rel.25A">§ 25A AOTC / LLC: education credits for taxpayer's own education</li>
                <li data-i18n="view.s127.rel.529">§ 529 Plan distributions for qualified expenses</li>
                <li data-i18n="view.s127.rel.530">§ 530 Coverdell ESA</li>
                <li data-i18n="view.s127.rel.qsts">§ 222 Qualified Tuition + Related Expenses deduction (suspended 2021+)</li>
            </ul>
        </div>
    `;
    document.getElementById('s127-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.educational_assistance_received = Number(fd.get('educational_assistance_received')) || 0;
        state.student_loan_payments_received = Number(fd.get('student_loan_payments_received')) || 0;
        state.total_other_employees_received = Number(fd.get('total_other_employees_received')) || 0;
        state.is_owner_5_pct = !!fd.get('is_owner_5_pct');
        state.employer_has_written_plan = !!fd.get('employer_has_written_plan');
        state.has_education_assistance_program_qualifies = !!fd.get('has_education_assistance_program_qualifies');
        state.course_related_to_job = !!fd.get('course_related_to_job');
        state.is_undergraduate_grad = !!fd.get('is_undergraduate_grad');
        state.fed_marginal_rate = Number(fd.get('fed_marginal_rate')) || 0.32;
        state.state_marginal_rate = Number(fd.get('state_marginal_rate')) || 0.06;
        state.fica_rate = Number(fd.get('fica_rate')) || 0.0765;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s127-output');
    if (!el) return;
    const totalReceived = state.educational_assistance_received + state.student_loan_payments_received;
    const programQualifies = state.employer_has_written_plan && state.has_education_assistance_program_qualifies;
    const excludedUnder127 = programQualifies ? Math.min(totalReceived, EXCLUSION_LIMIT) : 0;
    const excessUnder127 = Math.max(0, totalReceived - EXCLUSION_LIMIT);
    const excludedUnder132d = state.course_related_to_job ? excessUnder127 : 0;
    const taxableRemaining = excessUnder127 - excludedUnder132d;
    const totalExcluded = excludedUnder127 + excludedUnder132d;
    const totalRate = state.fed_marginal_rate + state.state_marginal_rate + state.fica_rate;
    const taxSavings = totalExcluded * totalRate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s127.h2.result">Exclusion + tax savings</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s127.card.total_received">Total received</div>
                    <div class="value">$${totalReceived.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${programQualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s127.card.qualifies">Plan qualifies</div>
                    <div class="value">${programQualifies ? esc(t('view.s127.status.yes')) : esc(t('view.s127.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s127.card.excluded_127">§ 127 excluded</div>
                    <div class="value">$${excludedUnder127.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s127.card.excluded_132d">§ 132(d) excluded (above $5,250)</div>
                    <div class="value">$${excludedUnder132d.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${taxableRemaining > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s127.card.taxable">Remaining taxable</div>
                    <div class="value">$${taxableRemaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s127.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
