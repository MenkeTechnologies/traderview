// IRC § 414 — Aggregated Employer Rules + Controlled Groups.
// § 414(b) controlled group of corporations: parent-subsidiary OR brother-sister.
// § 414(c) trades or businesses under common control (LLCs, partnerships).
// § 414(m) affiliated service groups: shared services entities.
// Aggregation required for benefit plan testing, retirement plan limits, ACA, deferred comp.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    structure_type: 'parent_sub',
    ownership_test_pct: 80,
    has_common_owner_5_or_fewer: false,
    common_ownership_pct: 0,
    affiliated_service_group: false,
    medical_practice_combined: false,
    management_company_links: false,
    is_first_service_org: false,
    purpose_aggregation: 'benefits',
    entity_a_employees: 0,
    entity_b_employees: 0,
    benefits_plan_subject_aggregation: false,
    affordable_care_act_aggregation: false,
    pension_funding_aggregation: false,
};

export async function renderSection414(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s414.h1.title">// § 414 AGGREGATED EMPLOYER</span></h1>
        <p class="muted small" data-i18n="view.s414.hint.intro">
            <strong>§ 414(b) controlled group:</strong> parent-subsidiary (80%+ ownership) OR brother-sister
            (5 or fewer common owners; 80%+ common + 50%+ identical). <strong>§ 414(c):</strong> non-corporate
            trades / businesses under common control (LLCs, partnerships, sole props). <strong>§ 414(m)
            affiliated service group:</strong> shared services + first-service org rules. <strong>Purpose:</strong>
            mandatory aggregation for benefit plan testing, retirement limits, ACA, deferred comp.
            <strong>Form 5500 + benefit coverage tests (§ 410(b), § 401(a)(4), § 416 top-heavy).</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s414.h2.inputs">Inputs</h2>
            <form id="s414-form" class="inline-form">
                <label><span data-i18n="view.s414.label.type">Structure type</span>
                    <select name="structure_type">
                        <option value="parent_sub" ${state.structure_type === 'parent_sub' ? 'selected' : ''}>Parent-Subsidiary</option>
                        <option value="brother_sister" ${state.structure_type === 'brother_sister' ? 'selected' : ''}>Brother-Sister</option>
                        <option value="combined" ${state.structure_type === 'combined' ? 'selected' : ''}>Combined (both)</option>
                        <option value="asg" ${state.structure_type === 'asg' ? 'selected' : ''}>Affiliated Service Group</option>
                        <option value="management" ${state.structure_type === 'management' ? 'selected' : ''}>Management company</option>
                        <option value="none" ${state.structure_type === 'none' ? 'selected' : ''}>None (separate)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s414.label.ownership">Ownership test %</span>
                    <input type="number" step="0.1" name="ownership_test_pct" value="${state.ownership_test_pct}"></label>
                <label><span data-i18n="view.s414.label.five_or_fewer">5 or fewer common owners?</span>
                    <input type="checkbox" name="has_common_owner_5_or_fewer" ${state.has_common_owner_5_or_fewer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414.label.common_pct">Common ownership %</span>
                    <input type="number" step="0.1" name="common_ownership_pct" value="${state.common_ownership_pct}"></label>
                <label><span data-i18n="view.s414.label.asg">Affiliated Service Group?</span>
                    <input type="checkbox" name="affiliated_service_group" ${state.affiliated_service_group ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414.label.medical">Medical practice combined?</span>
                    <input type="checkbox" name="medical_practice_combined" ${state.medical_practice_combined ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414.label.management">Management company links?</span>
                    <input type="checkbox" name="management_company_links" ${state.management_company_links ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414.label.first_service">First Service Org (FSO)?</span>
                    <input type="checkbox" name="is_first_service_org" ${state.is_first_service_org ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414.label.purpose">Purpose of aggregation</span>
                    <select name="purpose_aggregation">
                        <option value="benefits" ${state.purpose_aggregation === 'benefits' ? 'selected' : ''}>Benefit plan testing</option>
                        <option value="retirement" ${state.purpose_aggregation === 'retirement' ? 'selected' : ''}>Retirement limits</option>
                        <option value="aca" ${state.purpose_aggregation === 'aca' ? 'selected' : ''}>ACA employer mandate</option>
                        <option value="deferred_comp" ${state.purpose_aggregation === 'deferred_comp' ? 'selected' : ''}>§ 409A deferred comp</option>
                        <option value="pension_funding" ${state.purpose_aggregation === 'pension_funding' ? 'selected' : ''}>Pension funding</option>
                    </select>
                </label>
                <label><span data-i18n="view.s414.label.a_employees">Entity A employees</span>
                    <input type="number" step="1" name="entity_a_employees" value="${state.entity_a_employees}"></label>
                <label><span data-i18n="view.s414.label.b_employees">Entity B employees</span>
                    <input type="number" step="1" name="entity_b_employees" value="${state.entity_b_employees}"></label>
                <label><span data-i18n="view.s414.label.bps">Benefit plan subject?</span>
                    <input type="checkbox" name="benefits_plan_subject_aggregation" ${state.benefits_plan_subject_aggregation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414.label.aca">ACA aggregation?</span>
                    <input type="checkbox" name="affordable_care_act_aggregation" ${state.affordable_care_act_aggregation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s414.label.pension">Pension funding aggregation?</span>
                    <input type="checkbox" name="pension_funding_aggregation" ${state.pension_funding_aggregation ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s414.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s414-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s414.h2.parent_sub">Parent-subsidiary controlled group (§ 414(b)(1))</h2>
            <ul class="muted small">
                <li data-i18n="view.s414.ps.test">Test: at least one corp owns ≥ 80% of another → controlled group</li>
                <li data-i18n="view.s414.ps.chain">Chain: A owns 80% of B owns 80% of C → A, B, C aggregated</li>
                <li data-i18n="view.s414.ps.intermediary">Intermediary: stock of subs aggregated to identify common owner</li>
                <li data-i18n="view.s414.ps.measurement">Measurement: voting power AND value (both required)</li>
                <li data-i18n="view.s414.ps.attribution">§ 1563 constructive ownership rules apply</li>
                <li data-i18n="view.s414.ps.foreign">Foreign corps: attribute downward (§ 318)</li>
                <li data-i18n="view.s414.ps.exempt">Excluded: certain regulated investment companies, REITs, others</li>
                <li data-i18n="view.s414.ps.ss_employee">Direct ownership counts; constructive ownership often controlling</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s414.h2.brother_sister">Brother-sister controlled group (§ 414(b)(2))</h2>
            <ul class="muted small">
                <li data-i18n="view.s414.bs.test_5">Test: 5 or fewer common shareholders own ≥ 80% of each corp</li>
                <li data-i18n="view.s414.bs.test_50">AND those 5 or fewer common shareholders have ≥ 50% IDENTICAL ownership</li>
                <li data-i18n="view.s414.bs.identical">Identical: smallest interest each owner has in any corp</li>
                <li data-i18n="view.s414.bs.example">Example: A owns 60% Corp 1 + 30% Corp 2; B owns 40% Corp 1 + 70% Corp 2 → not controlled (only 30% identical for A, 40% for B)</li>
                <li data-i18n="view.s414.bs.5_taken_with">"Taken together w/ each other" 5 or fewer + 80% + 50% identical</li>
                <li data-i18n="view.s414.bs.individuals_estates_trusts">Individuals, estates, trusts only (not corporate owners)</li>
                <li data-i18n="view.s414.bs.spouse">Spouses: constructive ownership unless legally separated</li>
                <li data-i18n="view.s414.bs.attribution">§ 1563 attribution rules apply</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s414.h2.asg">Affiliated Service Group (§ 414(m))</h2>
            <ul class="muted small">
                <li data-i18n="view.s414.asg.purpose">Captures shared service arrangements not caught by controlled group</li>
                <li data-i18n="view.s414.asg.fso">First Service Org (FSO): performs services for / with B-org</li>
                <li data-i18n="view.s414.asg.a_org">A-Org: B-org provides services FOR FSO; B-org owners involved</li>
                <li data-i18n="view.s414.asg.b_org">B-Org: ≥ 10% owned by FSO owners; provides services TO FSO</li>
                <li data-i18n="view.s414.asg.management">Management company test: B-org primarily manages FSO</li>
                <li data-i18n="view.s414.asg.medical">Medical practices: separate professional corps + management LLC = ASG</li>
                <li data-i18n="view.s414.asg.law_firms">Law firms / accountants: similar professional service organization rules</li>
                <li data-i18n="view.s414.asg.complex">Complex test — IRS Form 5300 PLR for confirmation</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s414.h2.aggregation_purposes">Aggregation purposes (when § 414 applies)</h2>
            <ul class="muted small">
                <li data-i18n="view.s414.ap.coverage">§ 410(b) plan coverage test (ratio percentage / average benefits)</li>
                <li data-i18n="view.s414.ap.nondiscrimination">§ 401(a)(4) plan nondiscrimination</li>
                <li data-i18n="view.s414.ap.top_heavy">§ 416 top-heavy plan test</li>
                <li data-i18n="view.s414.ap.compensation">§ 401(a)(17) comp limit ($345K 2024) — per group not per entity</li>
                <li data-i18n="view.s414.ap.s415">§ 415 retirement plan limit ($69K 2024 DC, $275K DB) — per group</li>
                <li data-i18n="view.s414.ap.s401_k">§ 401(k) deferral limit ($23,500 2025) — per individual across all employers in group</li>
                <li data-i18n="view.s414.ap.aca_50">ACA: aggregate 50+ FTE employer mandate trigger</li>
                <li data-i18n="view.s414.ap.s409a">§ 409A deferred comp: timing rules + 6-month delay for specified employees</li>
            </ul>
        </div>
    `;
    document.getElementById('s414-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.structure_type = fd.get('structure_type');
        state.ownership_test_pct = Number(fd.get('ownership_test_pct')) || 0;
        state.has_common_owner_5_or_fewer = !!fd.get('has_common_owner_5_or_fewer');
        state.common_ownership_pct = Number(fd.get('common_ownership_pct')) || 0;
        state.affiliated_service_group = !!fd.get('affiliated_service_group');
        state.medical_practice_combined = !!fd.get('medical_practice_combined');
        state.management_company_links = !!fd.get('management_company_links');
        state.is_first_service_org = !!fd.get('is_first_service_org');
        state.purpose_aggregation = fd.get('purpose_aggregation');
        state.entity_a_employees = Number(fd.get('entity_a_employees')) || 0;
        state.entity_b_employees = Number(fd.get('entity_b_employees')) || 0;
        state.benefits_plan_subject_aggregation = !!fd.get('benefits_plan_subject_aggregation');
        state.affordable_care_act_aggregation = !!fd.get('affordable_care_act_aggregation');
        state.pension_funding_aggregation = !!fd.get('pension_funding_aggregation');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s414-output');
    if (!el) return;
    const parent_sub_qualifies = state.structure_type === 'parent_sub' && state.ownership_test_pct >= 80;
    const brother_sister_qualifies = state.structure_type === 'brother_sister' && state.has_common_owner_5_or_fewer && state.ownership_test_pct >= 80 && state.common_ownership_pct >= 50;
    const asg_qualifies = state.affiliated_service_group;
    const aggregated = parent_sub_qualifies || brother_sister_qualifies || asg_qualifies;
    const combined_employees = aggregated ? state.entity_a_employees + state.entity_b_employees : state.entity_a_employees;
    const aca_mandate_triggered = aggregated && combined_employees >= 50;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s414.h2.result">§ 414 aggregation outcome</h2>
            <div class="cards">
                <div class="card ${parent_sub_qualifies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s414.card.parent_sub">Parent-sub controlled?</div>
                    <div class="value">${parent_sub_qualifies ? esc(t('view.s414.status.yes')) : esc(t('view.s414.status.no'))}</div>
                </div>
                <div class="card ${brother_sister_qualifies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s414.card.brother_sister">Brother-sister controlled?</div>
                    <div class="value">${brother_sister_qualifies ? esc(t('view.s414.status.yes')) : esc(t('view.s414.status.no'))}</div>
                </div>
                <div class="card ${asg_qualifies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s414.card.asg">Affiliated Service Group?</div>
                    <div class="value">${asg_qualifies ? esc(t('view.s414.status.yes')) : esc(t('view.s414.status.no'))}</div>
                </div>
                <div class="card ${aggregated ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s414.card.aggregated">AGGREGATED?</div>
                    <div class="value">${aggregated ? esc(t('view.s414.status.yes')) : esc(t('view.s414.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s414.card.combined">Combined employees</div>
                    <div class="value">${combined_employees}</div>
                </div>
                <div class="card ${aca_mandate_triggered ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s414.card.aca">ACA mandate (50+ FTE)?</div>
                    <div class="value">${aca_mandate_triggered ? esc(t('view.s414.status.yes')) : esc(t('view.s414.status.no'))}</div>
                </div>
            </div>
            ${aggregated ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s414.aggregated_note">
                    AGGREGATED for § 414: benefit plan coverage testing must include ALL group employees.
                    § 401(k) deferral limit shared across employers. § 415 retirement plan limit per group.
                    Top-heavy testing across group. ACA 50+ FTE: aggregated. § 409A specified employee
                    determined at group level. Form 5500 may need single filing or coordinated.
                </p>
            ` : ''}
        </div>
    `;
}
