// IRC § 129 — Dependent Care Assistance Programs (DCAP).
// Employer-sponsored DCAP up to $5,000 ($2,500 MFS) excluded from wages.
// Used for child care, after-school care, dependent adult care.
// Coordination with § 21 Child & Dependent Care Credit (no double dipping).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'mfj',
    dcap_contribution: 0,
    dcap_annual_limit_2024: 5000,
    dcap_mfs_limit: 2500,
    qualifying_children_count: 0,
    qualifying_adults_count: 0,
    earned_income_taxpayer: 0,
    earned_income_spouse: 0,
    actual_qualifying_expenses: 0,
    total_dcap_used: 0,
    taxable_excess: 0,
    is_qualified_dcap_employer_plan: false,
    plan_meets_s129_d_eligibility: true,
    benefits_limited_55pct_5pct_owners: false,
    s129_d_3_concentration_test: false,
    s129_d_5_average_test: false,
    s125_cafeteria_election: false,
    is_dependent_care_fsa: true,
    fsa_use_or_lose: true,
    grace_period_2_5_months: false,
    carryover_550_allowed: false,
    is_higher_compensated_employee: false,
    s125_compensation_threshold: 0,
    is_5pct_owner: false,
    s129_d_4_reasonable_communication: true,
    s129_d_6_safe_harbor: false,
    s125_50pct_employee_eligible: false,
    is_qualified_individual: false,
    qualifying_individual_under_13: false,
    qualifying_individual_disabled: false,
    qualifying_individual_lives_with_taxpayer: true,
    is_employee_benefit_qualified: false,
    aggregate_count: 0,
    s21_credit_coordination: 0,
    s21_credit_remaining_eligible: 0,
    s129_no_double_dipping: true,
    arpa_2021_10500_temporary: false,
    employer_match: 0,
    employer_employer_contribution: 0,
    s129_e_form_2441_required: false,
    is_household_employee_paid: false,
    s129_e_dependent_care_provider_tin: '',
    care_provided_for_work_purpose: true,
};

export async function renderSection129(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s129.h1.title">// § 129 DEPENDENT CARE ASSISTANCE PROGRAM</span></h1>
        <p class="muted small" data-i18n="view.s129.hint.intro">
            <strong>§ 129</strong> — employer-sponsored Dependent Care Assistance Program (DCAP)
            excludes up to <strong>$5,000</strong> ($2,500 MFS) annually from employee wages — used
            for qualifying dependent care expenses. <strong>ARPA 2021 ONE-YEAR ONLY:</strong> $10,500
            limit (expired Dec 31, 2021). <strong>Qualifying individuals:</strong> child UNDER 13 OR
            disabled spouse/dependent of any age. <strong>Employer plan tests:</strong> § 129(d)
            written plan + nondiscrimination + concentration test (no &gt; 25% to 5%-owners).
            <strong>Use-or-lose:</strong> traditional DCAP-FSA — funds forfeit at year-end.
            <strong>§ 125 cafeteria plan</strong> typical delivery mechanism with annual election.
            <strong>§ 21 Child &amp; Dependent Care Credit coordination:</strong> NO DOUBLE DIPPING —
            credit-eligible expenses REDUCED by DCAP exclusion amount. <strong>Form 2441</strong>
            reports both DCAP + § 21 credit + provider TIN.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s129.h2.inputs">Inputs</h2>
            <form id="s129-form" class="inline-form">
                <label><span data-i18n="view.s129.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HOH</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s129.label.contribution">DCAP contribution ($)</span>
                    <input type="number" step="100" name="dcap_contribution" value="${state.dcap_contribution}"></label>
                <label><span data-i18n="view.s129.label.limit">2024 limit ($)</span>
                    <input type="number" step="100" name="dcap_annual_limit_2024" value="${state.dcap_annual_limit_2024}"></label>
                <label><span data-i18n="view.s129.label.mfs">MFS limit ($)</span>
                    <input type="number" step="100" name="dcap_mfs_limit" value="${state.dcap_mfs_limit}"></label>
                <label><span data-i18n="view.s129.label.children">Qualifying children &lt; 13</span>
                    <input type="number" step="1" name="qualifying_children_count" value="${state.qualifying_children_count}"></label>
                <label><span data-i18n="view.s129.label.adults">Qualifying adults</span>
                    <input type="number" step="1" name="qualifying_adults_count" value="${state.qualifying_adults_count}"></label>
                <label><span data-i18n="view.s129.label.income_tp">Earned income TP ($)</span>
                    <input type="number" step="1000" name="earned_income_taxpayer" value="${state.earned_income_taxpayer}"></label>
                <label><span data-i18n="view.s129.label.income_sp">Earned income spouse ($)</span>
                    <input type="number" step="1000" name="earned_income_spouse" value="${state.earned_income_spouse}"></label>
                <label><span data-i18n="view.s129.label.actual">Actual expenses ($)</span>
                    <input type="number" step="100" name="actual_qualifying_expenses" value="${state.actual_qualifying_expenses}"></label>
                <label><span data-i18n="view.s129.label.used">Total DCAP used ($)</span>
                    <input type="number" step="100" name="total_dcap_used" value="${state.total_dcap_used}"></label>
                <label><span data-i18n="view.s129.label.excess">Taxable excess ($)</span>
                    <input type="number" step="100" name="taxable_excess" value="${state.taxable_excess}"></label>
                <label><span data-i18n="view.s129.label.qual_plan">Qualified employer plan?</span>
                    <input type="checkbox" name="is_qualified_dcap_employer_plan" ${state.is_qualified_dcap_employer_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.elig">§ 129(d) eligibility?</span>
                    <input type="checkbox" name="plan_meets_s129_d_eligibility" ${state.plan_meets_s129_d_eligibility ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.55_5">55%/5% owners limit?</span>
                    <input type="checkbox" name="benefits_limited_55pct_5pct_owners" ${state.benefits_limited_55pct_5pct_owners ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.s129d3">§ 129(d)(3) concentration?</span>
                    <input type="checkbox" name="s129_d_3_concentration_test" ${state.s129_d_3_concentration_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.s129d5">§ 129(d)(5) avg?</span>
                    <input type="checkbox" name="s129_d_5_average_test" ${state.s129_d_5_average_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.s125">§ 125 elect?</span>
                    <input type="checkbox" name="s125_cafeteria_election" ${state.s125_cafeteria_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.fsa">Dep care FSA?</span>
                    <input type="checkbox" name="is_dependent_care_fsa" ${state.is_dependent_care_fsa ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.use_lose">Use or lose?</span>
                    <input type="checkbox" name="fsa_use_or_lose" ${state.fsa_use_or_lose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.grace">2.5 mo grace?</span>
                    <input type="checkbox" name="grace_period_2_5_months" ${state.grace_period_2_5_months ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.carryover">$550 carryover?</span>
                    <input type="checkbox" name="carryover_550_allowed" ${state.carryover_550_allowed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.hce">HCE?</span>
                    <input type="checkbox" name="is_higher_compensated_employee" ${state.is_higher_compensated_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.s125_thresh">§ 125 comp threshold ($)</span>
                    <input type="number" step="1000" name="s125_compensation_threshold" value="${state.s125_compensation_threshold}"></label>
                <label><span data-i18n="view.s129.label.5pct_owner">5% owner?</span>
                    <input type="checkbox" name="is_5pct_owner" ${state.is_5pct_owner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.s129d4">§ 129(d)(4) communication?</span>
                    <input type="checkbox" name="s129_d_4_reasonable_communication" ${state.s129_d_4_reasonable_communication ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.s129d6">§ 129(d)(6) safe harbor?</span>
                    <input type="checkbox" name="s129_d_6_safe_harbor" ${state.s129_d_6_safe_harbor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.s125_50">§ 125 50% eligible?</span>
                    <input type="checkbox" name="s125_50pct_employee_eligible" ${state.s125_50pct_employee_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.qual_ind">Qualified individual?</span>
                    <input type="checkbox" name="is_qualified_individual" ${state.is_qualified_individual ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.under_13">Under 13?</span>
                    <input type="checkbox" name="qualifying_individual_under_13" ${state.qualifying_individual_under_13 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.disabled">Disabled?</span>
                    <input type="checkbox" name="qualifying_individual_disabled" ${state.qualifying_individual_disabled ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.lives_with">Lives with TP?</span>
                    <input type="checkbox" name="qualifying_individual_lives_with_taxpayer" ${state.qualifying_individual_lives_with_taxpayer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.emp_benefit">Qualified benefit?</span>
                    <input type="checkbox" name="is_employee_benefit_qualified" ${state.is_employee_benefit_qualified ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.agg_count">Aggregate count</span>
                    <input type="number" step="1" name="aggregate_count" value="${state.aggregate_count}"></label>
                <label><span data-i18n="view.s129.label.s21">§ 21 credit ($)</span>
                    <input type="number" step="100" name="s21_credit_coordination" value="${state.s21_credit_coordination}"></label>
                <label><span data-i18n="view.s129.label.s21_remaining">§ 21 remaining ($)</span>
                    <input type="number" step="100" name="s21_credit_remaining_eligible" value="${state.s21_credit_remaining_eligible}"></label>
                <label><span data-i18n="view.s129.label.no_double">No double dip?</span>
                    <input type="checkbox" name="s129_no_double_dipping" ${state.s129_no_double_dipping ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.arpa">ARPA $10,500 (2021)?</span>
                    <input type="checkbox" name="arpa_2021_10500_temporary" ${state.arpa_2021_10500_temporary ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.match">Employer match ($)</span>
                    <input type="number" step="100" name="employer_match" value="${state.employer_match}"></label>
                <label><span data-i18n="view.s129.label.employer">Employer contrib ($)</span>
                    <input type="number" step="100" name="employer_employer_contribution" value="${state.employer_employer_contribution}"></label>
                <label><span data-i18n="view.s129.label.f2441">Form 2441 required?</span>
                    <input type="checkbox" name="s129_e_form_2441_required" ${state.s129_e_form_2441_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.household">Household employee paid?</span>
                    <input type="checkbox" name="is_household_employee_paid" ${state.is_household_employee_paid ? 'checked' : ''}></label>
                <label><span data-i18n="view.s129.label.provider_tin">Provider TIN</span>
                    <input type="text" name="s129_e_dependent_care_provider_tin" value="${esc(state.s129_e_dependent_care_provider_tin)}"></label>
                <label><span data-i18n="view.s129.label.work_purpose">For work purpose?</span>
                    <input type="checkbox" name="care_provided_for_work_purpose" ${state.care_provided_for_work_purpose ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s129.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s129-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s129.h2.limits">Annual limits + history</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s129.tbl.year">Year</th><th data-i18n="view.s129.tbl.limit">Limit</th><th data-i18n="view.s129.tbl.mfs">MFS</th><th data-i18n="view.s129.tbl.note">Note</th></tr></thead>
                <tbody>
                    <tr><td>Pre-1986</td><td>$5,000</td><td>$2,500</td><td data-i18n="view.s129.tbl.original">Original</td></tr>
                    <tr><td>1986-2020</td><td>$5,000</td><td>$2,500</td><td data-i18n="view.s129.tbl.unchanged">Unchanged (no indexing)</td></tr>
                    <tr><td>2021 ARPA</td><td>$10,500</td><td>$5,250</td><td data-i18n="view.s129.tbl.arpa">ARPA temporary</td></tr>
                    <tr><td>2022-Now</td><td>$5,000</td><td>$2,500</td><td data-i18n="view.s129.tbl.reverted">Reverted</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s129.h2.qualifying_expenses">Qualifying dependent care expenses</h2>
            <ul class="muted small">
                <li data-i18n="view.s129.q.work_related">Must be WORK-RELATED (enable taxpayer + spouse to work)</li>
                <li data-i18n="view.s129.q.under_13">Child under 13 at time of care</li>
                <li data-i18n="view.s129.q.disabled">Disabled spouse or dependent (any age)</li>
                <li data-i18n="view.s129.q.live_together">Qualifying individual must live with taxpayer &gt; ½ year</li>
                <li data-i18n="view.s129.q.in_home">In-home care: babysitter, nanny, day care provider</li>
                <li data-i18n="view.s129.q.day_camp">Day camp: yes (residential overnight camp: no)</li>
                <li data-i18n="view.s129.q.before_after">Before-school + after-school care</li>
                <li data-i18n="view.s129.q.preschool">Preschool / nursery school (educational excluded if kindergarten or higher)</li>
                <li data-i18n="view.s129.q.school_tuition">School tuition for kindergarten + above: NOT qualifying</li>
                <li data-i18n="view.s129.q.dependent_care_center">Dependent care center serving 6+ people</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s129.h2.s21_coordination">§ 21 Child &amp; Dependent Care Credit coordination</h2>
            <ul class="muted small">
                <li data-i18n="view.s129.s21.no_double_dip">NO double-dipping: § 21 credit-eligible expenses REDUCED by DCAP exclusion</li>
                <li data-i18n="view.s129.s21.limits">§ 21 limits: $3,000 (1 dep) / $6,000 (2+ deps) — much lower than § 129 $5K</li>
                <li data-i18n="view.s129.s21.rates">§ 21 credit rate: 20-35% of expenses (sliding with AGI)</li>
                <li data-i18n="view.s129.s21.arpa">ARPA 2021 expanded § 21 to refundable + larger — expired</li>
                <li data-i18n="view.s129.s21.dcap_first">Strategy: § 129 DCAP exclusion first (saves 22-37% income + FICA ~7.65%), then § 21 credit on remainder</li>
                <li data-i18n="view.s129.s21.high_income">High-income: § 129 better (full exclusion); § 21 credit only 20%</li>
                <li data-i18n="view.s129.s21.low_income">Low-income: § 21 credit better (35%); but DCAP saves FICA too</li>
                <li data-i18n="view.s129.s21.split_optimization">Split optimization: DCAP up to limit + § 21 credit on excess</li>
                <li data-i18n="view.s129.s21.f2441">Form 2441 computes both + coordinates limit</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s129.h2.nondiscrimination">Nondiscrimination tests (§ 129(d))</h2>
            <ul class="muted small">
                <li data-i18n="view.s129.nd.eligibility">§ 129(d)(2) eligibility: not in favor of HCEs / 5% owners</li>
                <li data-i18n="view.s129.nd.benefits_55_5">§ 129(d)(3): benefits to 5% owners cannot exceed 25% of total</li>
                <li data-i18n="view.s129.nd.utilization">§ 129(d)(5): average benefits to non-HCEs ≥ 55% of HCE average</li>
                <li data-i18n="view.s129.nd.reasonable">§ 129(d)(4): reasonable communication to all eligible employees</li>
                <li data-i18n="view.s129.nd.safe_harbor">§ 129(d)(6): safe harbor for plans satisfying ratio + average benefits</li>
                <li data-i18n="view.s129.nd.failure">Failure: HCE benefits become TAXABLE (non-HCEs preserved exclusion)</li>
                <li data-i18n="view.s129.nd.s125_separate">§ 125 cafeteria plan nondiscrimination separate (key employee + average benefit)</li>
                <li data-i18n="view.s129.nd.s414_q_hce">HCE = § 414(q): &gt; 5% owner OR &gt; $155K compensation (2024 indexed)</li>
                <li data-i18n="view.s129.nd.testing_year">Annual testing — failure assessed at YEAR END</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s129.h2.fsa">Dependent Care FSA mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s129.fsa.s125">§ 125 cafeteria plan election irrevocable for year</li>
                <li data-i18n="view.s129.fsa.use_or_lose">Use-or-lose: funds forfeit Dec 31 (unspent)</li>
                <li data-i18n="view.s129.fsa.grace_period">Up to 2.5-month grace period (employer choice)</li>
                <li data-i18n="view.s129.fsa.carryover_FSA_health_only">$550+ carryover ONLY for health FSA — NOT dep care</li>
                <li data-i18n="view.s129.fsa.reimbursement">Reimbursement only after expense incurred + receipts submitted</li>
                <li data-i18n="view.s129.fsa.qualifying_event">Mid-year change: only with qualifying life event (birth, marriage, divorce)</li>
                <li data-i18n="view.s129.fsa.cares_act_2020">CARES Act 2020 + ARPA 2021 temporary flexibility (carryover allowed for 2020/2021)</li>
                <li data-i18n="view.s129.fsa.no_run_out">No "run-out" period in following year for dep care (vs health FSA)</li>
                <li data-i18n="view.s129.fsa.employer_match">Some employers contribute additional dep care benefit on top</li>
            </ul>
        </div>
    `;
    document.getElementById('s129-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.dcap_contribution = Number(fd.get('dcap_contribution')) || 0;
        state.dcap_annual_limit_2024 = Number(fd.get('dcap_annual_limit_2024')) || 0;
        state.dcap_mfs_limit = Number(fd.get('dcap_mfs_limit')) || 0;
        state.qualifying_children_count = Number(fd.get('qualifying_children_count')) || 0;
        state.qualifying_adults_count = Number(fd.get('qualifying_adults_count')) || 0;
        state.earned_income_taxpayer = Number(fd.get('earned_income_taxpayer')) || 0;
        state.earned_income_spouse = Number(fd.get('earned_income_spouse')) || 0;
        state.actual_qualifying_expenses = Number(fd.get('actual_qualifying_expenses')) || 0;
        state.total_dcap_used = Number(fd.get('total_dcap_used')) || 0;
        state.taxable_excess = Number(fd.get('taxable_excess')) || 0;
        state.is_qualified_dcap_employer_plan = !!fd.get('is_qualified_dcap_employer_plan');
        state.plan_meets_s129_d_eligibility = !!fd.get('plan_meets_s129_d_eligibility');
        state.benefits_limited_55pct_5pct_owners = !!fd.get('benefits_limited_55pct_5pct_owners');
        state.s129_d_3_concentration_test = !!fd.get('s129_d_3_concentration_test');
        state.s129_d_5_average_test = !!fd.get('s129_d_5_average_test');
        state.s125_cafeteria_election = !!fd.get('s125_cafeteria_election');
        state.is_dependent_care_fsa = !!fd.get('is_dependent_care_fsa');
        state.fsa_use_or_lose = !!fd.get('fsa_use_or_lose');
        state.grace_period_2_5_months = !!fd.get('grace_period_2_5_months');
        state.carryover_550_allowed = !!fd.get('carryover_550_allowed');
        state.is_higher_compensated_employee = !!fd.get('is_higher_compensated_employee');
        state.s125_compensation_threshold = Number(fd.get('s125_compensation_threshold')) || 0;
        state.is_5pct_owner = !!fd.get('is_5pct_owner');
        state.s129_d_4_reasonable_communication = !!fd.get('s129_d_4_reasonable_communication');
        state.s129_d_6_safe_harbor = !!fd.get('s129_d_6_safe_harbor');
        state.s125_50pct_employee_eligible = !!fd.get('s125_50pct_employee_eligible');
        state.is_qualified_individual = !!fd.get('is_qualified_individual');
        state.qualifying_individual_under_13 = !!fd.get('qualifying_individual_under_13');
        state.qualifying_individual_disabled = !!fd.get('qualifying_individual_disabled');
        state.qualifying_individual_lives_with_taxpayer = !!fd.get('qualifying_individual_lives_with_taxpayer');
        state.is_employee_benefit_qualified = !!fd.get('is_employee_benefit_qualified');
        state.aggregate_count = Number(fd.get('aggregate_count')) || 0;
        state.s21_credit_coordination = Number(fd.get('s21_credit_coordination')) || 0;
        state.s21_credit_remaining_eligible = Number(fd.get('s21_credit_remaining_eligible')) || 0;
        state.s129_no_double_dipping = !!fd.get('s129_no_double_dipping');
        state.arpa_2021_10500_temporary = !!fd.get('arpa_2021_10500_temporary');
        state.employer_match = Number(fd.get('employer_match')) || 0;
        state.employer_employer_contribution = Number(fd.get('employer_employer_contribution')) || 0;
        state.s129_e_form_2441_required = !!fd.get('s129_e_form_2441_required');
        state.is_household_employee_paid = !!fd.get('is_household_employee_paid');
        state.s129_e_dependent_care_provider_tin = fd.get('s129_e_dependent_care_provider_tin') || '';
        state.care_provided_for_work_purpose = !!fd.get('care_provided_for_work_purpose');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s129-output');
    if (!el) return;
    const limit = state.filing_status === 'mfs' ? state.dcap_mfs_limit : state.dcap_annual_limit_2024;
    const earned_floor = Math.min(state.earned_income_taxpayer, state.earned_income_spouse || state.earned_income_taxpayer);
    const max_exclude = Math.min(state.dcap_contribution, limit, earned_floor, state.actual_qualifying_expenses);
    const excess = Math.max(0, state.dcap_contribution - max_exclude);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s129.h2.result">§ 129 DCAP exclusion</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s129.card.contrib">Contribution</div><div class="value">$${state.dcap_contribution.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s129.card.limit">Limit</div><div class="value">$${limit.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s129.card.earned">Earned floor</div><div class="value">$${earned_floor.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s129.card.excluded">Excluded from wages</div><div class="value">$${max_exclude.toLocaleString()}</div></div>
                <div class="card ${excess > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s129.card.excess">Taxable excess</div><div class="value">$${excess.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
