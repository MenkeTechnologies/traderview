// IRC § 152 — Definition of Dependents.
// Two categories: (1) Qualifying Child (QC) — relationship + age + residency + support + JR,
// (2) Qualifying Relative (QR) — relationship/HOH-member + gross income $5,050 (2024) + support.
// Multiple support agreement (Form 2120) if no one provides > 50% support.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const GROSS_INCOME_LIMIT_2024 = 5_050;

let state = {
    person_category: 'child',
    relationship_to_you: 'child',
    age: 0,
    is_full_time_student: false,
    is_permanently_disabled: false,
    months_lived_with_you: 12,
    your_support_provided: 0,
    their_total_support: 0,
    their_gross_income: 0,
    is_us_citizen_or_resident: true,
    filing_joint_return: false,
    is_qualifying_child_of_another: false,
};

export async function renderSection152(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s152.h1.title">// § 152 DEPENDENT QUALIFICATION</span></h1>
        <p class="muted small" data-i18n="view.s152.hint.intro">
            Two categories: <strong>(1) Qualifying Child (QC)</strong> — relationship + age (&lt; 19
            or &lt; 24 student or any age disabled) + residency &gt; ½ yr + did not provide &gt; ½
            own support, <strong>(2) Qualifying Relative (QR)</strong> — relationship OR
            household member full year + gross income &lt; $5,050 (2024) + support &gt; ½.
            <strong>Multiple Support Agreement (Form 2120)</strong> if no one provides &gt; 50%.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s152.h2.inputs">Inputs</h2>
            <form id="s152-form" class="inline-form">
                <label><span data-i18n="view.s152.label.category">Category</span>
                    <select name="person_category">
                        <option value="child" ${state.person_category === 'child' ? 'selected' : ''}>Qualifying Child</option>
                        <option value="relative" ${state.person_category === 'relative' ? 'selected' : ''}>Qualifying Relative</option>
                    </select>
                </label>
                <label><span data-i18n="view.s152.label.relationship">Relationship</span>
                    <select name="relationship_to_you">
                        <option value="child" ${state.relationship_to_you === 'child' ? 'selected' : ''}>Child / Step / Foster</option>
                        <option value="sibling" ${state.relationship_to_you === 'sibling' ? 'selected' : ''}>Sibling / Half / Step</option>
                        <option value="parent" ${state.relationship_to_you === 'parent' ? 'selected' : ''}>Parent</option>
                        <option value="other_relative" ${state.relationship_to_you === 'other_relative' ? 'selected' : ''}>Aunt / Uncle / Niece / Nephew / In-law</option>
                        <option value="household_member" ${state.relationship_to_you === 'household_member' ? 'selected' : ''}>Unrelated household member (full year)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s152.label.age">Age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.s152.label.student">Full-time student?</span>
                    <input type="checkbox" name="is_full_time_student" ${state.is_full_time_student ? 'checked' : ''}></label>
                <label><span data-i18n="view.s152.label.disabled">Permanently disabled?</span>
                    <input type="checkbox" name="is_permanently_disabled" ${state.is_permanently_disabled ? 'checked' : ''}></label>
                <label><span data-i18n="view.s152.label.months">Months lived with you</span>
                    <input type="number" step="1" min="0" max="12" name="months_lived_with_you" value="${state.months_lived_with_you}"></label>
                <label><span data-i18n="view.s152.label.your_support">Your support provided ($)</span>
                    <input type="number" step="0.01" name="your_support_provided" value="${state.your_support_provided}"></label>
                <label><span data-i18n="view.s152.label.total_support">Their total support ($)</span>
                    <input type="number" step="0.01" name="their_total_support" value="${state.their_total_support}"></label>
                <label><span data-i18n="view.s152.label.gross_income">Their gross income ($)</span>
                    <input type="number" step="0.01" name="their_gross_income" value="${state.their_gross_income}"></label>
                <label><span data-i18n="view.s152.label.citizen">US citizen / resident?</span>
                    <input type="checkbox" name="is_us_citizen_or_resident" ${state.is_us_citizen_or_resident ? 'checked' : ''}></label>
                <label><span data-i18n="view.s152.label.joint">Filing joint return?</span>
                    <input type="checkbox" name="filing_joint_return" ${state.filing_joint_return ? 'checked' : ''}></label>
                <label><span data-i18n="view.s152.label.qc_other">Qualifying child of another taxpayer?</span>
                    <input type="checkbox" name="is_qualifying_child_of_another" ${state.is_qualifying_child_of_another ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s152.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s152-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s152.h2.tiebreaker">Tiebreaker rules</h2>
            <ol class="muted small">
                <li data-i18n="view.s152.tb.parents">Parent claims child OVER non-parent</li>
                <li data-i18n="view.s152.tb.most_time">If both parents — parent with MORE residency time</li>
                <li data-i18n="view.s152.tb.highest_agi">Equal time — parent with higher AGI</li>
                <li data-i18n="view.s152.tb.no_parents">No parent claims — highest AGI taxpayer can claim</li>
                <li data-i18n="view.s152.tb.agreement">Multiple support agreement (Form 2120): &gt; 10% support each, sign-off</li>
                <li data-i18n="view.s152.tb.divorced">Custodial parent default unless Form 8332 release signed</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s152.h2.relationship_qr">Qualifying relative — relationship list</h2>
            <ul class="muted small">
                <li data-i18n="view.s152.qr.child">Child, stepchild, foster child + descendants</li>
                <li data-i18n="view.s152.qr.siblings">Brother, sister, half / step + descendants</li>
                <li data-i18n="view.s152.qr.parents">Father, mother + ancestors + step</li>
                <li data-i18n="view.s152.qr.aunts_uncles">Aunt, uncle, niece, nephew (by blood ONLY, not in-law)</li>
                <li data-i18n="view.s152.qr.in_laws">Son-in-law, daughter-in-law, parent-in-law, brother/sister-in-law</li>
                <li data-i18n="view.s152.qr.member">Any unrelated person who lives with you ENTIRE year</li>
                <li data-i18n="view.s152.qr.not_violation">Relationship not violation of local law (e.g., not bigamy)</li>
            </ul>
        </div>
    `;
    document.getElementById('s152-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.person_category = fd.get('person_category');
        state.relationship_to_you = fd.get('relationship_to_you');
        state.age = Number(fd.get('age')) || 0;
        state.is_full_time_student = !!fd.get('is_full_time_student');
        state.is_permanently_disabled = !!fd.get('is_permanently_disabled');
        state.months_lived_with_you = Number(fd.get('months_lived_with_you')) || 12;
        state.your_support_provided = Number(fd.get('your_support_provided')) || 0;
        state.their_total_support = Number(fd.get('their_total_support')) || 0;
        state.their_gross_income = Number(fd.get('their_gross_income')) || 0;
        state.is_us_citizen_or_resident = !!fd.get('is_us_citizen_or_resident');
        state.filing_joint_return = !!fd.get('filing_joint_return');
        state.is_qualifying_child_of_another = !!fd.get('is_qualifying_child_of_another');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s152-output');
    if (!el) return;
    const supportPct = state.their_total_support > 0 ? state.your_support_provided / state.their_total_support : 0;
    const meetsAgeQC = state.age < 19
        || (state.is_full_time_student && state.age < 24)
        || state.is_permanently_disabled;
    const meetsResidencyQC = state.months_lived_with_you >= 7;
    const meetsSupportQC = supportPct > 0.50;
    const meetsRelationshipQC = ['child', 'sibling'].includes(state.relationship_to_you);
    const qcQualifies = state.person_category === 'child'
        && meetsAgeQC && meetsResidencyQC && meetsSupportQC && meetsRelationshipQC
        && state.is_us_citizen_or_resident && !state.filing_joint_return && !state.is_qualifying_child_of_another;
    const meetsRelationshipQR = state.relationship_to_you !== 'household_member' || state.months_lived_with_you === 12;
    const meetsGrossIncomeQR = state.their_gross_income < GROSS_INCOME_LIMIT_2024;
    const meetsSupportQR = supportPct > 0.50;
    const qrQualifies = state.person_category === 'relative'
        && meetsRelationshipQR && meetsGrossIncomeQR && meetsSupportQR
        && state.is_us_citizen_or_resident && !state.filing_joint_return;
    const qualifies = state.person_category === 'child' ? qcQualifies : qrQualifies;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s152.h2.result">Dependent qualification</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s152.card.qualifies">Qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s152.status.yes')) : esc(t('view.s152.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s152.card.support_pct">Your support %</div>
                    <div class="value">${(supportPct * 100).toFixed(0)}%</div>
                </div>
                <div class="card ${state.person_category === 'child' && meetsAgeQC ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s152.card.age_qc">QC age met</div>
                    <div class="value">${state.person_category === 'child' ? (meetsAgeQC ? esc(t('view.s152.status.yes')) : esc(t('view.s152.status.no'))) : '—'}</div>
                </div>
                <div class="card ${state.person_category === 'relative' && meetsGrossIncomeQR ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s152.card.gross_qr">QR gross income met</div>
                    <div class="value">${state.person_category === 'relative' ? (meetsGrossIncomeQR ? esc(t('view.s152.status.yes')) : esc(t('view.s152.status.no'))) : '—'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s152.card.us_citizen">Citizenship met</div>
                    <div class="value">${state.is_us_citizen_or_resident ? esc(t('view.s152.status.yes')) : esc(t('view.s152.status.no'))}</div>
                </div>
            </div>
        </div>
    `;
}
