// IRC § 269A — Personal Service Corporations.
// IRS may reallocate income, deductions, credits between PSC and owner-employees if substantially all
// services performed for ONE other entity AND PRINCIPAL PURPOSE = tax avoidance.
// Pairs with § 269 (acquisition of corp for tax avoidance) + § 482 (related-party allocation).

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_personal_service_corp: false,
    services_primary_user: '',
    services_pct_to_one_entity: 0,
    employee_owner_pct: 0,
    s269a_threshold_substantially_all: 95,
    principal_purpose_tax_avoidance: false,
    tax_avoidance_purpose_quantified: 0,
    reallocation_amount: 0,
    psc_gross_income: 0,
    psc_deductions: 0,
    psc_taxable_income: 0,
    psc_qpsc_flat_rate_repealed: true,
    psc_pre_tcja_35_pct_rate: false,
    is_qualified_PSC_s448_d2: false,
    psc_calendar_year_required: false,
    s444_election_year: false,
    has_5_5_5_minimum_distribution: false,
    s269a_substantially_all_test: false,
    s269_acquisition_purpose: false,
    s482_transfer_pricing_related: false,
    has_legitimate_business_purpose: false,
    business_purpose_documented: false,
    employee_independent_status_test: false,
    rev_rul_2002_69_factors: false,
    relevant_industries: 'accounting',
    s269b_corporate_acquisition: false,
    s269c_overall_inc_distortion: false,
    s269_a_2_tax_avoidance: false,
    cra_examples_application: false,
};

export async function renderSection269A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s269a.h1.title">// § 269A PERSONAL SERVICE CORPORATION</span></h1>
        <p class="muted small" data-i18n="view.s269a.hint.intro">
            <strong>§ 269A</strong> empowers IRS to ALLOCATE income, deductions, credits between
            personal service corporation (PSC) + owner-employees IF: (1) "substantially all"
            (typically <strong>≥ 95%</strong>) services performed for ONE other entity, AND
            (2) PRINCIPAL PURPOSE is tax avoidance. <strong>Targets:</strong> "loan-out corp" abuse
            where individual incorporates to delay income, shift to lower-bracket entity, or
            improve deduction timing. <strong>Pairs with:</strong> § 269 (corp acquisition for tax
            avoidance) + § 482 (related-party transfer pricing). <strong>§ 448(d)(2) Qualified PSC:</strong>
            in fields of health, law, engineering, architecture, accounting, actuarial science,
            performing arts, consulting — taxed at <strong>flat 21%</strong> post-TCJA. <strong>§ 444</strong>
            allows fiscal year w/ minimum distribution rule. <strong>Court approach:</strong> facts +
            circumstances + economic substance + business purpose documentation.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s269a.h2.inputs">Inputs</h2>
            <form id="s269a-form" class="inline-form">
                <label><span data-i18n="view.s269a.label.is_psc">Is PSC?</span>
                    <input type="checkbox" name="is_personal_service_corp" ${state.is_personal_service_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.user">Primary user</span>
                    <input type="text" name="services_primary_user" value="${esc(state.services_primary_user)}"></label>
                <label><span data-i18n="view.s269a.label.pct_one">% to one entity</span>
                    <input type="number" step="0.1" name="services_pct_to_one_entity" value="${state.services_pct_to_one_entity}"></label>
                <label><span data-i18n="view.s269a.label.owner_pct">Employee-owner %</span>
                    <input type="number" step="0.1" name="employee_owner_pct" value="${state.employee_owner_pct}"></label>
                <label><span data-i18n="view.s269a.label.threshold">"Substantially all" %</span>
                    <input type="number" step="0.1" name="s269a_threshold_substantially_all" value="${state.s269a_threshold_substantially_all}"></label>
                <label><span data-i18n="view.s269a.label.purpose">Tax-avoidance purpose?</span>
                    <input type="checkbox" name="principal_purpose_tax_avoidance" ${state.principal_purpose_tax_avoidance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.quantified">Tax savings ($)</span>
                    <input type="number" step="10000" name="tax_avoidance_purpose_quantified" value="${state.tax_avoidance_purpose_quantified}"></label>
                <label><span data-i18n="view.s269a.label.realloc">Reallocation amount ($)</span>
                    <input type="number" step="10000" name="reallocation_amount" value="${state.reallocation_amount}"></label>
                <label><span data-i18n="view.s269a.label.gross">PSC gross income ($)</span>
                    <input type="number" step="10000" name="psc_gross_income" value="${state.psc_gross_income}"></label>
                <label><span data-i18n="view.s269a.label.dedn">PSC deductions ($)</span>
                    <input type="number" step="10000" name="psc_deductions" value="${state.psc_deductions}"></label>
                <label><span data-i18n="view.s269a.label.taxable">PSC taxable income ($)</span>
                    <input type="number" step="10000" name="psc_taxable_income" value="${state.psc_taxable_income}"></label>
                <label><span data-i18n="view.s269a.label.repealed">35% pre-TCJA flat rate?</span>
                    <input type="checkbox" name="psc_qpsc_flat_rate_repealed" ${state.psc_qpsc_flat_rate_repealed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.pre_tcja">Pre-TCJA?</span>
                    <input type="checkbox" name="psc_pre_tcja_35_pct_rate" ${state.psc_pre_tcja_35_pct_rate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.qualified">§ 448(d)(2) qualified PSC?</span>
                    <input type="checkbox" name="is_qualified_PSC_s448_d2" ${state.is_qualified_PSC_s448_d2 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.cal_yr">Calendar year required?</span>
                    <input type="checkbox" name="psc_calendar_year_required" ${state.psc_calendar_year_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.s444">§ 444 election?</span>
                    <input type="checkbox" name="s444_election_year" ${state.s444_election_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.minimum">5-5-5 minimum dist?</span>
                    <input type="checkbox" name="has_5_5_5_minimum_distribution" ${state.has_5_5_5_minimum_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.subst_all">Substantially all test?</span>
                    <input type="checkbox" name="s269a_substantially_all_test" ${state.s269a_substantially_all_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.s269_acq">§ 269 acquisition?</span>
                    <input type="checkbox" name="s269_acquisition_purpose" ${state.s269_acquisition_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.s482">§ 482 transfer pricing?</span>
                    <input type="checkbox" name="s482_transfer_pricing_related" ${state.s482_transfer_pricing_related ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.legit">Legitimate biz purpose?</span>
                    <input type="checkbox" name="has_legitimate_business_purpose" ${state.has_legitimate_business_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.doc">Documented?</span>
                    <input type="checkbox" name="business_purpose_documented" ${state.business_purpose_documented ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.indep">Employee indep status?</span>
                    <input type="checkbox" name="employee_independent_status_test" ${state.employee_independent_status_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.rev_rul">Rev Rul 2002-69?</span>
                    <input type="checkbox" name="rev_rul_2002_69_factors" ${state.rev_rul_2002_69_factors ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.industry">Industry</span>
                    <select name="relevant_industries">
                        <option value="accounting" ${state.relevant_industries === 'accounting' ? 'selected' : ''}>Accounting</option>
                        <option value="law" ${state.relevant_industries === 'law' ? 'selected' : ''}>Law</option>
                        <option value="health" ${state.relevant_industries === 'health' ? 'selected' : ''}>Health</option>
                        <option value="engineering" ${state.relevant_industries === 'engineering' ? 'selected' : ''}>Engineering</option>
                        <option value="architecture" ${state.relevant_industries === 'architecture' ? 'selected' : ''}>Architecture</option>
                        <option value="actuarial" ${state.relevant_industries === 'actuarial' ? 'selected' : ''}>Actuarial</option>
                        <option value="performing" ${state.relevant_industries === 'performing' ? 'selected' : ''}>Performing arts</option>
                        <option value="consulting" ${state.relevant_industries === 'consulting' ? 'selected' : ''}>Consulting</option>
                    </select>
                </label>
                <label><span data-i18n="view.s269a.label.s269b">§ 269(b) acquisition?</span>
                    <input type="checkbox" name="s269b_corporate_acquisition" ${state.s269b_corporate_acquisition ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.s269c">§ 269(c) distortion?</span>
                    <input type="checkbox" name="s269c_overall_inc_distortion" ${state.s269c_overall_inc_distortion ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.s269a2">§ 269A(a)(2) tax avoidance?</span>
                    <input type="checkbox" name="s269_a_2_tax_avoidance" ${state.s269_a_2_tax_avoidance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269a.label.cra">CRA examples?</span>
                    <input type="checkbox" name="cra_examples_application" ${state.cra_examples_application ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s269a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s269a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269a.h2.test">§ 269A(a) two-prong test</h2>
            <ol class="muted small">
                <li data-i18n="view.s269a.test.first">"Substantially all" services performed for ONE other entity (~95%)</li>
                <li data-i18n="view.s269a.test.second">Principal purpose: tax avoidance or evasion</li>
                <li data-i18n="view.s269a.test.s269a_b_psc">§ 269A(b) "PSC" definition: corp where employee-owner performs substantially all services</li>
                <li data-i18n="view.s269a.test.employee_owner">"Employee-owner" = owns &gt; 10% of stock (constructive ownership via § 318)</li>
                <li data-i18n="view.s269a.test.related">"Related" to PSC under § 414 / § 482</li>
                <li data-i18n="view.s269a.test.IRS_burden">IRS burden of proof: tax avoidance + substantially all + reallocation reasonable</li>
                <li data-i18n="view.s269a.test.taxpayer_response">Taxpayer defenses: business purpose, economic substance, profit motive</li>
                <li data-i18n="view.s269a.test.allocation_methods">IRS allocation methods: arm's length compensation, distribution timing changes</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269a.h2.qpsc_classification">§ 448(d)(2) qualified PSC fields</h2>
            <ul class="muted small">
                <li data-i18n="view.s269a.qpsc.health">Health (doctors, dentists, vets, nurses)</li>
                <li data-i18n="view.s269a.qpsc.law">Law (lawyers, paralegals)</li>
                <li data-i18n="view.s269a.qpsc.engineering">Engineering</li>
                <li data-i18n="view.s269a.qpsc.architecture">Architecture</li>
                <li data-i18n="view.s269a.qpsc.accounting">Accounting (CPAs, tax preparers)</li>
                <li data-i18n="view.s269a.qpsc.actuarial">Actuarial science</li>
                <li data-i18n="view.s269a.qpsc.performing">Performing arts</li>
                <li data-i18n="view.s269a.qpsc.consulting">Consulting (NOT routine management)</li>
                <li data-i18n="view.s269a.qpsc.pre_tcja">Pre-TCJA: 35% flat rate (vs graduated max 35%)</li>
                <li data-i18n="view.s269a.qpsc.post_tcja">Post-TCJA: 21% flat rate same as all C-corps</li>
                <li data-i18n="view.s269a.qpsc.deferral">Calendar year required UNLESS § 444 election made</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269a.h2.s444">§ 444 fiscal year election</h2>
            <ul class="muted small">
                <li data-i18n="view.s269a.s444.deferral">Allows fiscal year (max 3-month deferral)</li>
                <li data-i18n="view.s269a.s444.required_payment">REQUIRED PAYMENT = approx federal tax that would be owed on deferral period</li>
                <li data-i18n="view.s269a.s444.minimum_dist">5-5-5 minimum distribution rule for PSC</li>
                <li data-i18n="view.s269a.s444.5_5_5">5% of comp + 5% of distributions + 5% of either — must distribute at least amount</li>
                <li data-i18n="view.s269a.s444.s280h">§ 280H: limits PSC deductions if 5-5-5 not met</li>
                <li data-i18n="view.s269a.s444.f8716">Form 8716 elects § 444 fiscal year</li>
                <li data-i18n="view.s269a.s444.no_economic_purpose">"No business purpose" required — § 444 simply opts in to fiscal year</li>
                <li data-i18n="view.s269a.s444.terminates">Terminates if 5-5-5 fails OR ceasing to be PSC</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269a.h2.examples">Common scenarios</h2>
            <ul class="muted small">
                <li data-i18n="view.s269a.ex.loan_out">"Loan-out" corp: actor/athlete with single primary studio/team</li>
                <li data-i18n="view.s269a.ex.doctor">Solo doctor incorporating + working primarily at one hospital</li>
                <li data-i18n="view.s269a.ex.consultant">Solo consultant w/ one primary client</li>
                <li data-i18n="view.s269a.ex.deferral">Tax deferral via PSC fiscal year (now limited by § 280H)</li>
                <li data-i18n="view.s269a.ex.pre_tcja_arb">Pre-TCJA bracket arbitrage (PSC 35% vs individual 39.6%) — repealed</li>
                <li data-i18n="view.s269a.ex.bonus_year_end">Year-end bonus to employee-owner to zero out PSC income</li>
                <li data-i18n="view.s269a.ex.s162_unreasonable_comp">§ 162 unreasonable compensation: separate doctrine — PSC normal context</li>
                <li data-i18n="view.s269a.ex.s269a_minimal">§ 269A in practice: rarely invoked vs other doctrines (substance over form, § 482)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269a.h2.defenses">Defensive planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s269a.def.diversify">Diversify clients/employers (multiple users of services)</li>
                <li data-i18n="view.s269a.def.business_purpose">Document business purpose for incorporation</li>
                <li data-i18n="view.s269a.def.legitimate">Insurance liability, employee benefit, employee retention, succession planning</li>
                <li data-i18n="view.s269a.def.reasonable_comp">Reasonable compensation to employee-owner (no excessive deferral)</li>
                <li data-i18n="view.s269a.def.actual_operations">Actual operations: separate office, equipment, hiring, marketing</li>
                <li data-i18n="view.s269a.def.calendar_default">Use calendar year by default — avoid § 444 election scrutiny</li>
                <li data-i18n="view.s269a.def.economic_substance">Maintain economic substance: bona fide salary, benefits, retirement plan</li>
                <li data-i18n="view.s269a.def.s199a">Consider S-corp + § 199A QBI vs PSC C-corp for tax efficiency</li>
                <li data-i18n="view.s269a.def.s162_m">§ 162(m) excess comp limit for public co — separate</li>
            </ul>
        </div>
    `;
    document.getElementById('s269a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_personal_service_corp = !!fd.get('is_personal_service_corp');
        state.services_primary_user = fd.get('services_primary_user') || '';
        state.services_pct_to_one_entity = Number(fd.get('services_pct_to_one_entity')) || 0;
        state.employee_owner_pct = Number(fd.get('employee_owner_pct')) || 0;
        state.s269a_threshold_substantially_all = Number(fd.get('s269a_threshold_substantially_all')) || 0;
        state.principal_purpose_tax_avoidance = !!fd.get('principal_purpose_tax_avoidance');
        state.tax_avoidance_purpose_quantified = Number(fd.get('tax_avoidance_purpose_quantified')) || 0;
        state.reallocation_amount = Number(fd.get('reallocation_amount')) || 0;
        state.psc_gross_income = Number(fd.get('psc_gross_income')) || 0;
        state.psc_deductions = Number(fd.get('psc_deductions')) || 0;
        state.psc_taxable_income = Number(fd.get('psc_taxable_income')) || 0;
        state.psc_qpsc_flat_rate_repealed = !!fd.get('psc_qpsc_flat_rate_repealed');
        state.psc_pre_tcja_35_pct_rate = !!fd.get('psc_pre_tcja_35_pct_rate');
        state.is_qualified_PSC_s448_d2 = !!fd.get('is_qualified_PSC_s448_d2');
        state.psc_calendar_year_required = !!fd.get('psc_calendar_year_required');
        state.s444_election_year = !!fd.get('s444_election_year');
        state.has_5_5_5_minimum_distribution = !!fd.get('has_5_5_5_minimum_distribution');
        state.s269a_substantially_all_test = !!fd.get('s269a_substantially_all_test');
        state.s269_acquisition_purpose = !!fd.get('s269_acquisition_purpose');
        state.s482_transfer_pricing_related = !!fd.get('s482_transfer_pricing_related');
        state.has_legitimate_business_purpose = !!fd.get('has_legitimate_business_purpose');
        state.business_purpose_documented = !!fd.get('business_purpose_documented');
        state.employee_independent_status_test = !!fd.get('employee_independent_status_test');
        state.rev_rul_2002_69_factors = !!fd.get('rev_rul_2002_69_factors');
        state.relevant_industries = fd.get('relevant_industries');
        state.s269b_corporate_acquisition = !!fd.get('s269b_corporate_acquisition');
        state.s269c_overall_inc_distortion = !!fd.get('s269c_overall_inc_distortion');
        state.s269_a_2_tax_avoidance = !!fd.get('s269_a_2_tax_avoidance');
        state.cra_examples_application = !!fd.get('cra_examples_application');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s269a-output');
    if (!el) return;
    const substantial = state.services_pct_to_one_entity >= state.s269a_threshold_substantially_all;
    const at_risk = substantial && state.principal_purpose_tax_avoidance && !state.has_legitimate_business_purpose;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s269a.h2.result">§ 269A risk assessment</h2>
            <div class="cards">
                <div class="card ${state.is_personal_service_corp ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s269a.card.psc">PSC?</div><div class="value">${state.is_personal_service_corp ? 'YES' : 'NO'}</div></div>
                <div class="card ${substantial ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s269a.card.substantial">Substantially all?</div><div class="value">${substantial ? 'YES' : 'NO'} (${state.services_pct_to_one_entity}%)</div></div>
                <div class="card ${state.principal_purpose_tax_avoidance ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s269a.card.purpose">Tax avoidance?</div><div class="value">${state.principal_purpose_tax_avoidance ? 'YES' : 'NO'}</div></div>
                <div class="card ${at_risk ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s269a.card.risk">§ 269A risk</div><div class="value">${at_risk ? 'HIGH' : 'LOW'}</div></div>
            </div>
        </div>
    `;
}
