// IRC § 221 Student Loan Interest Deduction.
// Above-the-line deduction (Schedule 1) up to $2,500/year per RETURN.
// MAGI phase-out 2024: $80k-$95k single / $165k-$195k MFJ. MFS not allowed.
// Loan must be qualified — for taxpayer / spouse / dependent's qualified education
// at qualified institution. Voluntary payments + capitalized interest both eligible.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const CAP_2024 = 2_500;
const SINGLE_LOW = 80_000;
const SINGLE_HIGH = 95_000;
const MFJ_LOW = 165_000;
const MFJ_HIGH = 195_000;

let state = {
    filing_status: 'single',
    magi: 0,
    interest_paid: 0,
    is_dependent_of_someone: false,
    marginal_rate: 0.22,
};

export async function renderSection221(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s221.h1.title">// § 221 STUDENT LOAN INTEREST</span></h1>
        <p class="muted small" data-i18n="view.s221.hint.intro">
            Above-the-line deduction (Schedule 1) up to <strong>$2,500/year per RETURN</strong>.
            MAGI phase-out 2024: <strong>$80k-$95k single / $165k-$195k MFJ</strong>. MFS NOT
            allowed. Cannot be claimed if you're a DEPENDENT on someone else's return.
            Loan must be qualified — for taxpayer / spouse / dependent's education. Voluntary
            and capitalized interest both count.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s221.h2.inputs">Inputs</h2>
            <form id="s221-form" class="inline-form">
                <label><span data-i18n="view.s221.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS (not eligible)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s221.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s221.label.interest_paid">Interest paid this year ($)</span>
                    <input type="number" step="10" name="interest_paid" value="${state.interest_paid}"></label>
                <label><span data-i18n="view.s221.label.dependent">Claimed as dependent on another return?</span>
                    <input type="checkbox" name="is_dependent_of_someone" ${state.is_dependent_of_someone ? 'checked' : ''}></label>
                <label><span data-i18n="view.s221.label.marginal">Marginal tax %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s221.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s221-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s221.h2.qualified">Qualified student loan</h2>
            <ul class="muted small">
                <li data-i18n="view.s221.qual.education">Money used SOLELY to pay qualified education expenses</li>
                <li data-i18n="view.s221.qual.who">For taxpayer / spouse / dependent at time loan taken out</li>
                <li data-i18n="view.s221.qual.eligible_institution">Eligible institution (post-secondary, accredited)</li>
                <li data-i18n="view.s221.qual.half_time">Enrolled at least half-time</li>
                <li data-i18n="view.s221.qual.related">NOT from related person (parent), employer benefit, or qualified employer plan</li>
                <li data-i18n="view.s221.qual.timing">Loan must be primarily liable on (cosigners may share)</li>
                <li data-i18n="view.s221.qual.refinanced">Refinanced student loans: maintain qualified character if proceeds used only to pay original</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s221.h2.secure_2_0">SECURE 2.0 student-loan match</h2>
            <p class="muted" data-i18n="view.s221.note.secure">
                Effective 2024: employers can MATCH employee student loan payments to 401(k) /
                403(b) plan. Treated like deferral for match purposes — even if employee makes
                no plan contribution. Helps young workers debt-paydown + retirement save simultaneously.
            </p>
        </div>
    `;
    document.getElementById('s221-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.magi = Number(fd.get('magi')) || 0;
        state.interest_paid = Number(fd.get('interest_paid')) || 0;
        state.is_dependent_of_someone = !!fd.get('is_dependent_of_someone');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.22;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s221-output');
    if (!el) return;
    if (state.filing_status === 'mfs' || state.is_dependent_of_someone) {
        el.innerHTML = `
            <div class="chart-panel">
                <p class="muted small neg" data-i18n="view.s221.warning.ineligible">
                    Not eligible: MFS filers + dependents on another return are disqualified.
                </p>
            </div>
        `;
        return;
    }
    const low = state.filing_status === 'mfj' ? MFJ_LOW : SINGLE_LOW;
    const high = state.filing_status === 'mfj' ? MFJ_HIGH : SINGLE_HIGH;
    let factor;
    if (state.magi <= low) factor = 1;
    else if (state.magi >= high) factor = 0;
    else factor = (high - state.magi) / (high - low);
    const cappedInterest = Math.min(state.interest_paid, CAP_2024);
    const deduction = cappedInterest * factor;
    const taxSavings = deduction * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s221.h2.result">Deduction</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s221.card.interest">Interest paid</div>
                    <div class="value">$${state.interest_paid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s221.card.cap">$2,500 cap applies</div>
                    <div class="value">$${cappedInterest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s221.card.factor">Phase-out factor</div>
                    <div class="value">${(factor * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s221.card.deduction">Deduction</div>
                    <div class="value">$${deduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s221.card.savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
