// IRC § 4958 — Intermediate Sanctions on Excess Benefit Transactions (§ 501(c)(3), (4), (29) tax-exempt orgs).
// Disqualified person (DP): substantial influence + 25% excise tax on excess benefit.
// Organization manager: 10% excise tax (max $20K) if knew + participated.
// Sister-tier penalty: 200% if not corrected within timely period.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    organization_type: '501c3',
    is_applicable_tax_exempt: false,
    transaction_amount: 0,
    fair_market_value_provided: 0,
    consideration_paid_by_org: 0,
    excess_benefit_amount: 0,
    is_disqualified_person: false,
    dp_substantial_influence_test: false,
    dp_relationship_to_org: 'officer',
    dp_family_member: false,
    dp_35pct_controlled_entity: false,
    s4958_a_1_25pct_excise: 0,
    s4958_b_200pct_correction_failure: 0,
    has_been_corrected: false,
    timely_correction_window: false,
    organization_manager_penalty_10pct: 0,
    om_max_20k_per_transaction: 20000,
    om_knowing_participation: false,
    s4958_c_3_grants_advisor: false,
    is_donor_advised_fund: false,
    s4958_c_1_a_excess_benefit_test: false,
    s4958_a_2_compensation_for_services: false,
    rebuttable_presumption_satisfied: false,
    independent_board_approval: false,
    comparability_data_used: false,
    contemporaneous_documentation: false,
    s4960_executive_comp_excess_1m: 0,
    s4960_21pct_excise: 0,
    excess_comp_over_1m: 0,
    excess_parachute_payment: 0,
    organization_revenue: 0,
    employee_top_5_highest_paid: 0,
    s501_c_3_public_charity: false,
    s501_c_3_private_foundation: false,
    s501_c_4_social_welfare: false,
    s501_c_29_qualified_nonprofit_health: false,
    fiscal_year_end: '',
    excess_benefit_quarter: 'Q4',
    self_dealing_potential_s4941: false,
    revenue_proc_2007_69: false,
};

export async function renderSection4958(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4958.h1.title">// § 4958 INTERMEDIATE SANCTIONS</span></h1>
        <p class="muted small" data-i18n="view.s4958.hint.intro">
            <strong>§ 4958</strong> imposes EXCISE TAX on EXCESS BENEFIT TRANSACTIONS between
            "applicable tax-exempt organization" (§ 501(c)(3) public charity, § 501(c)(4) social
            welfare, § 501(c)(29) qualified nonprofit health) + "disqualified person" (DP).
            <strong>1st-tier tax:</strong> 25% of excess benefit on DP. <strong>Organization
            manager:</strong> 10% excise (max $20,000 per transaction). <strong>2nd-tier tax:</strong>
            200% on DP if NOT TIMELY CORRECTED. <strong>"Disqualified person":</strong> substantial
            influence over org (current/former officer, director, key employee), family members,
            35%-controlled entity. <strong>Rebuttable presumption</strong> of reasonableness:
            (1) independent board approval + (2) comparability data + (3) contemporaneous documentation.
            <strong>§ 4960 EXECUTIVE COMP EXCISE</strong> (TCJA): 21% on comp &gt; $1M to top-5
            highest-paid + excess parachute payments. <strong>Does NOT supplant § 501(c)(3)
            revocation</strong> for major abuses.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.inputs">Inputs</h2>
            <form id="s4958-form" class="inline-form">
                <label><span data-i18n="view.s4958.label.org_type">Org type</span>
                    <select name="organization_type">
                        <option value="501c3" ${state.organization_type === '501c3' ? 'selected' : ''}>§ 501(c)(3)</option>
                        <option value="501c4" ${state.organization_type === '501c4' ? 'selected' : ''}>§ 501(c)(4)</option>
                        <option value="501c29" ${state.organization_type === '501c29' ? 'selected' : ''}>§ 501(c)(29)</option>
                        <option value="private_foundation" ${state.organization_type === 'private_foundation' ? 'selected' : ''}>Private foundation (§ 4941)</option>
                        <option value="daf" ${state.organization_type === 'daf' ? 'selected' : ''}>Donor-advised fund</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4958.label.applicable">Applicable exempt?</span>
                    <input type="checkbox" name="is_applicable_tax_exempt" ${state.is_applicable_tax_exempt ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.amount">Transaction amount ($)</span>
                    <input type="number" step="1000" name="transaction_amount" value="${state.transaction_amount}"></label>
                <label><span data-i18n="view.s4958.label.fmv">FMV provided to org ($)</span>
                    <input type="number" step="1000" name="fair_market_value_provided" value="${state.fair_market_value_provided}"></label>
                <label><span data-i18n="view.s4958.label.consideration">Consideration paid ($)</span>
                    <input type="number" step="1000" name="consideration_paid_by_org" value="${state.consideration_paid_by_org}"></label>
                <label><span data-i18n="view.s4958.label.excess">Excess benefit ($)</span>
                    <input type="number" step="1000" name="excess_benefit_amount" value="${state.excess_benefit_amount}"></label>
                <label><span data-i18n="view.s4958.label.dp">Disqualified person?</span>
                    <input type="checkbox" name="is_disqualified_person" ${state.is_disqualified_person ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.influence">Substantial influence?</span>
                    <input type="checkbox" name="dp_substantial_influence_test" ${state.dp_substantial_influence_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.rel">DP relationship</span>
                    <select name="dp_relationship_to_org">
                        <option value="officer" ${state.dp_relationship_to_org === 'officer' ? 'selected' : ''}>Officer</option>
                        <option value="director" ${state.dp_relationship_to_org === 'director' ? 'selected' : ''}>Director / trustee</option>
                        <option value="key_employee" ${state.dp_relationship_to_org === 'key_employee' ? 'selected' : ''}>Key employee</option>
                        <option value="former_dp" ${state.dp_relationship_to_org === 'former_dp' ? 'selected' : ''}>Former DP (5-yr)</option>
                        <option value="family_member" ${state.dp_relationship_to_org === 'family_member' ? 'selected' : ''}>Family member</option>
                        <option value="35pct_entity" ${state.dp_relationship_to_org === '35pct_entity' ? 'selected' : ''}>35%-controlled entity</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4958.label.family">Family member?</span>
                    <input type="checkbox" name="dp_family_member" ${state.dp_family_member ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.entity_35">35% entity?</span>
                    <input type="checkbox" name="dp_35pct_controlled_entity" ${state.dp_35pct_controlled_entity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.25pct">§ 4958(a)(1) 25% ($)</span>
                    <input type="number" step="1000" name="s4958_a_1_25pct_excise" value="${state.s4958_a_1_25pct_excise}"></label>
                <label><span data-i18n="view.s4958.label.200pct">§ 4958(b) 200% ($)</span>
                    <input type="number" step="1000" name="s4958_b_200pct_correction_failure" value="${state.s4958_b_200pct_correction_failure}"></label>
                <label><span data-i18n="view.s4958.label.corrected">Corrected?</span>
                    <input type="checkbox" name="has_been_corrected" ${state.has_been_corrected ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.timely">Timely correction?</span>
                    <input type="checkbox" name="timely_correction_window" ${state.timely_correction_window ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.om_10">OM 10% ($)</span>
                    <input type="number" step="1000" name="organization_manager_penalty_10pct" value="${state.organization_manager_penalty_10pct}"></label>
                <label><span data-i18n="view.s4958.label.om_max">OM max $20K</span>
                    <input type="number" step="1000" name="om_max_20k_per_transaction" value="${state.om_max_20k_per_transaction}"></label>
                <label><span data-i18n="view.s4958.label.om_knowing">OM knowing?</span>
                    <input type="checkbox" name="om_knowing_participation" ${state.om_knowing_participation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.daf_advisor">DAF advisor?</span>
                    <input type="checkbox" name="s4958_c_3_grants_advisor" ${state.s4958_c_3_grants_advisor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.daf">Donor-advised fund?</span>
                    <input type="checkbox" name="is_donor_advised_fund" ${state.is_donor_advised_fund ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.excess_test">Excess benefit test?</span>
                    <input type="checkbox" name="s4958_c_1_a_excess_benefit_test" ${state.s4958_c_1_a_excess_benefit_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.svc">Comp for svc?</span>
                    <input type="checkbox" name="s4958_a_2_compensation_for_services" ${state.s4958_a_2_compensation_for_services ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.rebut">Rebuttable presumption?</span>
                    <input type="checkbox" name="rebuttable_presumption_satisfied" ${state.rebuttable_presumption_satisfied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.indep">Indep board?</span>
                    <input type="checkbox" name="independent_board_approval" ${state.independent_board_approval ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.comparable">Comparability data?</span>
                    <input type="checkbox" name="comparability_data_used" ${state.comparability_data_used ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.doc">Contemp docs?</span>
                    <input type="checkbox" name="contemporaneous_documentation" ${state.contemporaneous_documentation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.s4960_1m">§ 4960 over $1M ($)</span>
                    <input type="number" step="10000" name="s4960_executive_comp_excess_1m" value="${state.s4960_executive_comp_excess_1m}"></label>
                <label><span data-i18n="view.s4958.label.s4960_21">§ 4960 21% ($)</span>
                    <input type="number" step="1000" name="s4960_21pct_excise" value="${state.s4960_21pct_excise}"></label>
                <label><span data-i18n="view.s4958.label.excess_1m">Excess over $1M ($)</span>
                    <input type="number" step="10000" name="excess_comp_over_1m" value="${state.excess_comp_over_1m}"></label>
                <label><span data-i18n="view.s4958.label.parachute">Excess parachute ($)</span>
                    <input type="number" step="10000" name="excess_parachute_payment" value="${state.excess_parachute_payment}"></label>
                <label><span data-i18n="view.s4958.label.rev">Org revenue ($)</span>
                    <input type="number" step="100000" name="organization_revenue" value="${state.organization_revenue}"></label>
                <label><span data-i18n="view.s4958.label.top5">Top 5 highest paid?</span>
                    <input type="number" step="1" name="employee_top_5_highest_paid" value="${state.employee_top_5_highest_paid}"></label>
                <label><span data-i18n="view.s4958.label.pc">Public charity?</span>
                    <input type="checkbox" name="s501_c_3_public_charity" ${state.s501_c_3_public_charity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.pf">Private foundation?</span>
                    <input type="checkbox" name="s501_c_3_private_foundation" ${state.s501_c_3_private_foundation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.sw">Social welfare?</span>
                    <input type="checkbox" name="s501_c_4_social_welfare" ${state.s501_c_4_social_welfare ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.health">Qualified health?</span>
                    <input type="checkbox" name="s501_c_29_qualified_nonprofit_health" ${state.s501_c_29_qualified_nonprofit_health ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.fye">FYE</span>
                    <input type="date" name="fiscal_year_end" value="${state.fiscal_year_end}"></label>
                <label><span data-i18n="view.s4958.label.quarter">Excess benefit quarter</span>
                    <input type="text" name="excess_benefit_quarter" value="${esc(state.excess_benefit_quarter)}"></label>
                <label><span data-i18n="view.s4958.label.s4941">§ 4941 self-dealing?</span>
                    <input type="checkbox" name="self_dealing_potential_s4941" ${state.self_dealing_potential_s4941 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4958.label.rp">Rev Proc 2007-69?</span>
                    <input type="checkbox" name="revenue_proc_2007_69" ${state.revenue_proc_2007_69 ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s4958.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4958-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.dp_test">"Disqualified person" test (§ 4958(f)(1))</h2>
            <ol class="muted small">
                <li data-i18n="view.s4958.dp.substantial">PERSON WITH SUBSTANTIAL INFLUENCE: officer, director, key employee, etc.</li>
                <li data-i18n="view.s4958.dp.former">FORMER DP for 5 years after substantial influence ends</li>
                <li data-i18n="view.s4958.dp.family">FAMILY MEMBERS: spouse, ancestors, descendants, siblings + spouses</li>
                <li data-i18n="view.s4958.dp.35pct">35%-CONTROLLED ENTITIES (vote or value)</li>
                <li data-i18n="view.s4958.dp.factors">Factors: voting power, founding role, financial contributions, structure of authority</li>
                <li data-i18n="view.s4958.dp.daf_donor">DAF: donor + family members are DPs to fund</li>
                <li data-i18n="view.s4958.dp.501c3_safe_harbor">Pres/treasurer/CEO/CFO: per-se substantial influence</li>
                <li data-i18n="view.s4958.dp.501c3_excluded">EXCLUDED: § 501(c)(3) public charity (org itself), employees making &lt; $130K (2024 indexed), volunteers</li>
                <li data-i18n="view.s4958.dp.regs">Reg § 53.4958-3 factors test for substantial influence</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.excess_benefit">"Excess benefit" definition</h2>
            <ul class="muted small">
                <li data-i18n="view.s4958.ebt.formula">Excess benefit = VALUE PROVIDED TO DP minus CONSIDERATION DP gave to org</li>
                <li data-i18n="view.s4958.ebt.compensation">Compensation: salary + bonuses + perks + retirement + § 79 life insurance</li>
                <li data-i18n="view.s4958.ebt.indirect">Indirect benefit through controlled entity</li>
                <li data-i18n="view.s4958.ebt.s4958_c_2">§ 4958(c)(2) automatic excess benefit — DAF grants to DP — ENTIRE grant excess</li>
                <li data-i18n="view.s4958.ebt.section_67_c">§ 4958(c)(3) — DAF taxable distribution: 100% excise</li>
                <li data-i18n="view.s4958.ebt.economic_benefit">"Economic benefit" includes loans, rentals, sales (FMV vs paid)</li>
                <li data-i18n="view.s4958.ebt.scholarship">Scholarship to DP: typically excess benefit</li>
                <li data-i18n="view.s4958.ebt.partial_release">Forgiveness of debt, payment of personal expenses</li>
                <li data-i18n="view.s4958.ebt.timing">Timing: when economic benefit becomes ascertainable + non-refundable</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.rebuttable">Rebuttable presumption of reasonableness</h2>
            <ol class="muted small">
                <li data-i18n="view.s4958.rebut.independent">Independent board members approved transaction (no conflicts)</li>
                <li data-i18n="view.s4958.rebut.comparable">Comparability data: comparable orgs + arms-length salary surveys</li>
                <li data-i18n="view.s4958.rebut.contemporaneous">Contemporaneous documentation: minutes, comp committee, salary survey data</li>
                <li data-i18n="view.s4958.rebut.satisfaction">Satisfying all 3 → presumption of reasonableness (burden shifts to IRS)</li>
                <li data-i18n="view.s4958.rebut.s4958_c_1_a">§ 4958(c)(1)(A) — reasonable compensation safe harbor</li>
                <li data-i18n="view.s4958.rebut.compensation_committee">Compensation committee independence requirements</li>
                <li data-i18n="view.s4958.rebut.benchmark">Benchmark data: 3+ similar orgs + similar geographic + sector</li>
                <li data-i18n="view.s4958.rebut.regs">Reg § 53.4958-6 detailed safe-harbor procedures</li>
                <li data-i18n="view.s4958.rebut.no_safe_harbor">Without safe harbor: facts &amp; circumstances burden on DP/org</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.penalty_tiers">2-tier penalty structure</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s4958.tbl.tier">Tier</th><th data-i18n="view.s4958.tbl.rate">Rate</th><th data-i18n="view.s4958.tbl.target">Target</th><th data-i18n="view.s4958.tbl.timing">Timing</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s4958.tbl.t1_dp">1st tier (DP)</td><td>25%</td><td data-i18n="view.s4958.tbl.dp_target">Disqualified person</td><td data-i18n="view.s4958.tbl.year">Year of transaction</td></tr>
                    <tr><td data-i18n="view.s4958.tbl.t1_om">1st tier (OM)</td><td>10% (cap $20K/tx)</td><td data-i18n="view.s4958.tbl.om_target">Organization manager (if knowing)</td><td data-i18n="view.s4958.tbl.year">Year of transaction</td></tr>
                    <tr><td data-i18n="view.s4958.tbl.t2">2nd tier</td><td>200%</td><td data-i18n="view.s4958.tbl.dp_target_2">DP only</td><td data-i18n="view.s4958.tbl.no_correct">If NOT timely corrected</td></tr>
                    <tr><td data-i18n="view.s4958.tbl.s4960">§ 4960 (TCJA)</td><td>21%</td><td data-i18n="view.s4958.tbl.org_4960">ORG (paid by org)</td><td data-i18n="view.s4958.tbl.year_paid">Year paid</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.correction">"Correction" procedure</h2>
            <ul class="muted small">
                <li data-i18n="view.s4958.corr.repay">DP repays excess to organization (with interest)</li>
                <li data-i18n="view.s4958.corr.timing">Within taxable period = ending 90 days after FIRST notice of deficiency</li>
                <li data-i18n="view.s4958.corr.full_restoration">Full restoration: cash, property, both</li>
                <li data-i18n="view.s4958.corr.interest">Interest at § 6621 underpayment rate from date of excess</li>
                <li data-i18n="view.s4958.corr.s4961_abatement">§ 4961 — IRS may abate § 4958(b) 200% on timely correction</li>
                <li data-i18n="view.s4958.corr.s4962">§ 4962 — abatement for reasonable cause + correction</li>
                <li data-i18n="view.s4958.corr.f4720">Form 4720 reports + computes</li>
                <li data-i18n="view.s4958.corr.s4958_d">§ 4958(d) — extends correction period during deficiency review</li>
                <li data-i18n="view.s4958.corr.no_double_recovery">DP cannot deduct repayment under § 165 (loss) — adjusts compensation</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.s4960">§ 4960 — Executive Comp Excise (TCJA 2017)</h2>
            <ul class="muted small">
                <li data-i18n="view.s4958.s4960.tcja">TCJA-added: 21% excise on excess executive comp by tax-exempt orgs</li>
                <li data-i18n="view.s4958.s4960.scope">Applies to: § 501(a) exempt orgs, applicable § 521 cooperatives, etc.</li>
                <li data-i18n="view.s4958.s4960.over_1m">Compensation &gt; $1,000,000 to top-5 highest-paid → 21% on excess</li>
                <li data-i18n="view.s4958.s4960.parachute">Excess parachute payment: 3× base + 21% excise on excess</li>
                <li data-i18n="view.s4958.s4960.paid_by_org">Paid by ORGANIZATION (not executive)</li>
                <li data-i18n="view.s4958.s4960.deductibility">NOT deductible (no impact — org is exempt)</li>
                <li data-i18n="view.s4958.s4960.f4720">Form 4720 Schedule N reports</li>
                <li data-i18n="view.s4958.s4960.regs_2020">Final regs T.D. 9938 (2020): scope, computation, related party rules</li>
                <li data-i18n="view.s4958.s4960.medical_exception">Medical services exception: doctors, vets, etc. — not counted</li>
                <li data-i18n="view.s4958.s4960.top_5">Top 5: aggregated current + 1 prior year HCEs</li>
            </ul>
        </div>
    `;
    document.getElementById('s4958-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.organization_type = fd.get('organization_type');
        state.is_applicable_tax_exempt = !!fd.get('is_applicable_tax_exempt');
        state.transaction_amount = Number(fd.get('transaction_amount')) || 0;
        state.fair_market_value_provided = Number(fd.get('fair_market_value_provided')) || 0;
        state.consideration_paid_by_org = Number(fd.get('consideration_paid_by_org')) || 0;
        state.excess_benefit_amount = Number(fd.get('excess_benefit_amount')) || 0;
        state.is_disqualified_person = !!fd.get('is_disqualified_person');
        state.dp_substantial_influence_test = !!fd.get('dp_substantial_influence_test');
        state.dp_relationship_to_org = fd.get('dp_relationship_to_org');
        state.dp_family_member = !!fd.get('dp_family_member');
        state.dp_35pct_controlled_entity = !!fd.get('dp_35pct_controlled_entity');
        state.s4958_a_1_25pct_excise = Number(fd.get('s4958_a_1_25pct_excise')) || 0;
        state.s4958_b_200pct_correction_failure = Number(fd.get('s4958_b_200pct_correction_failure')) || 0;
        state.has_been_corrected = !!fd.get('has_been_corrected');
        state.timely_correction_window = !!fd.get('timely_correction_window');
        state.organization_manager_penalty_10pct = Number(fd.get('organization_manager_penalty_10pct')) || 0;
        state.om_max_20k_per_transaction = Number(fd.get('om_max_20k_per_transaction')) || 0;
        state.om_knowing_participation = !!fd.get('om_knowing_participation');
        state.s4958_c_3_grants_advisor = !!fd.get('s4958_c_3_grants_advisor');
        state.is_donor_advised_fund = !!fd.get('is_donor_advised_fund');
        state.s4958_c_1_a_excess_benefit_test = !!fd.get('s4958_c_1_a_excess_benefit_test');
        state.s4958_a_2_compensation_for_services = !!fd.get('s4958_a_2_compensation_for_services');
        state.rebuttable_presumption_satisfied = !!fd.get('rebuttable_presumption_satisfied');
        state.independent_board_approval = !!fd.get('independent_board_approval');
        state.comparability_data_used = !!fd.get('comparability_data_used');
        state.contemporaneous_documentation = !!fd.get('contemporaneous_documentation');
        state.s4960_executive_comp_excess_1m = Number(fd.get('s4960_executive_comp_excess_1m')) || 0;
        state.s4960_21pct_excise = Number(fd.get('s4960_21pct_excise')) || 0;
        state.excess_comp_over_1m = Number(fd.get('excess_comp_over_1m')) || 0;
        state.excess_parachute_payment = Number(fd.get('excess_parachute_payment')) || 0;
        state.organization_revenue = Number(fd.get('organization_revenue')) || 0;
        state.employee_top_5_highest_paid = Number(fd.get('employee_top_5_highest_paid')) || 0;
        state.s501_c_3_public_charity = !!fd.get('s501_c_3_public_charity');
        state.s501_c_3_private_foundation = !!fd.get('s501_c_3_private_foundation');
        state.s501_c_4_social_welfare = !!fd.get('s501_c_4_social_welfare');
        state.s501_c_29_qualified_nonprofit_health = !!fd.get('s501_c_29_qualified_nonprofit_health');
        state.fiscal_year_end = fd.get('fiscal_year_end') || '';
        state.excess_benefit_quarter = fd.get('excess_benefit_quarter') || '';
        state.self_dealing_potential_s4941 = !!fd.get('self_dealing_potential_s4941');
        state.revenue_proc_2007_69 = !!fd.get('revenue_proc_2007_69');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4958-output');
    if (!el) return;
    const excess = Math.max(0, state.fair_market_value_provided - state.consideration_paid_by_org);
    const dp_25 = excess * 0.25;
    const dp_200 = !state.has_been_corrected ? excess * 2.00 : 0;
    const om_10 = Math.min(excess * 0.10, state.om_max_20k_per_transaction);
    const s4960 = state.excess_comp_over_1m * 0.21;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4958.h2.result">§ 4958 / § 4960 excise tax assessment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s4958.card.excess">Excess benefit</div><div class="value">$${excess.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s4958.card.dp25">DP 25% excise</div><div class="value">$${dp_25.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s4958.card.om">OM 10% (capped)</div><div class="value">$${om_10.toLocaleString()}</div></div>
                <div class="card ${dp_200 > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s4958.card.t2">2nd tier 200%</div><div class="value">$${dp_200.toLocaleString()}</div></div>
                <div class="card ${s4960 > 0 ? 'neg' : ''}"><div class="label" data-i18n="view.s4958.card.s4960">§ 4960 21% comp</div><div class="value">$${s4960.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
