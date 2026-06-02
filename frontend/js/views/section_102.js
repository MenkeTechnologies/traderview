// IRC § 102 — Gifts, Bequests, Devises, Inheritances Excluded from Gross Income.
// § 102(a) — general exclusion for property received by gift, bequest, devise, inheritance.
// § 102(b) — INCOME from gift property is taxable to recipient.
// § 102(c) — employer-employee transfers presumed compensation (NOT gift).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transfer_type: 'gift',
    transferor_relation: 'family',
    is_donative_intent: true,
    s102_a_exclusion_applies: false,
    s102_b_income_taxable_to_recipient: false,
    s102_c_employer_employee: false,
    duberstein_test_met: false,
    is_compensation_disguised_as_gift: false,
    is_business_gift_dedn_25: false,
    transferor_dedn_basis: 0,
    s1015_carryover_basis_gift: 0,
    s1014_step_up_inheritance: 0,
    fmv_at_transfer: 0,
    donor_adjusted_basis: 0,
    holding_period_donee_tacks: false,
    s1015_d_dual_basis: false,
    appreciated_property: false,
    gift_tax_paid_by_donor: 0,
    s1015_d_basis_adjustment: 0,
    is_part_sale_part_gift: false,
    bargain_sale_basis_allocation: 0,
    is_political_gift: false,
    s102_c_employer_award: 0,
    s274_j_employee_achievement_award: 0,
    s274_j_qualified_plan_award: 0,
    s274_j_non_qualified_award: 0,
    s132_b_gift_de_minimis: 0,
    is_inheritance: false,
    inheritance_amount: 0,
    is_devise: false,
    devise_through_will: false,
    is_bequest: false,
    bequest_specific_amount: 0,
    s101_a_life_insurance: false,
    life_insurance_proceeds_excluded: 0,
    is_step_up_basis_eligible: false,
    s1014_FMV_date_of_death: 0,
    s1014_e_inherited_from_decedent_1y_rule: false,
    s691_a_income_in_respect_of_decedent: 0,
    inherited_retirement_acct: false,
    inherited_appreciated_securities: false,
    estate_tax_on_iir: 0,
};

export async function renderSection102(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s102.h1.title">// § 102 GIFT / INHERITANCE EXCLUSION</span></h1>
        <p class="muted small" data-i18n="view.s102.hint.intro">
            <strong>§ 102(a)</strong> general exclusion for property received by GIFT, BEQUEST,
            DEVISE, INHERITANCE. <strong>§ 102(b):</strong> INCOME from gift property TAXABLE to
            recipient (rent, dividends, interest). <strong>§ 102(c):</strong> EMPLOYER-to-EMPLOYEE
            transfers presumed COMPENSATION (NOT gift). <strong>Duberstein test</strong> (1960):
            "detached and disinterested generosity" — proceeds from constraining force out of "moral
            or legal duty" insufficient. <strong>§ 274(j):</strong> employee achievement awards
            limited ($1,600 qualified plan / $400 non-qualified). <strong>§ 1015 carryover basis</strong>
            on gifts (lesser of donor basis OR FMV at gift for losses + § 1015(d) basis adjustment for
            gift tax paid). <strong>§ 1014 STEPPED-UP BASIS</strong> at death (FMV at date of death).
            <strong>§ 691 income in respect of decedent</strong> exception — NOT stepped up.
            <strong>§ 102(c) exception:</strong> bona fide non-employment transfers (e.g., to
            employee from owner's personal estate).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.inputs">Inputs</h2>
            <form id="s102-form" class="inline-form">
                <label><span data-i18n="view.s102.label.type">Transfer type</span>
                    <select name="transfer_type">
                        <option value="gift" ${state.transfer_type === 'gift' ? 'selected' : ''}>Inter vivos gift</option>
                        <option value="inheritance" ${state.transfer_type === 'inheritance' ? 'selected' : ''}>Inheritance (intestate)</option>
                        <option value="devise" ${state.transfer_type === 'devise' ? 'selected' : ''}>Devise (will - real)</option>
                        <option value="bequest" ${state.transfer_type === 'bequest' ? 'selected' : ''}>Bequest (will - personal)</option>
                        <option value="life_insurance" ${state.transfer_type === 'life_insurance' ? 'selected' : ''}>Life insurance proceeds</option>
                        <option value="employer_to_employee" ${state.transfer_type === 'employer_to_employee' ? 'selected' : ''}>Employer → employee</option>
                        <option value="political" ${state.transfer_type === 'political' ? 'selected' : ''}>Political contribution</option>
                        <option value="prize_award" ${state.transfer_type === 'prize_award' ? 'selected' : ''}>Prize / award (§ 74)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s102.label.relation">Transferor relation</span>
                    <select name="transferor_relation">
                        <option value="family" ${state.transferor_relation === 'family' ? 'selected' : ''}>Family member</option>
                        <option value="friend" ${state.transferor_relation === 'friend' ? 'selected' : ''}>Friend (non-business)</option>
                        <option value="employer" ${state.transferor_relation === 'employer' ? 'selected' : ''}>Employer (business)</option>
                        <option value="customer" ${state.transferor_relation === 'customer' ? 'selected' : ''}>Customer / business contact</option>
                        <option value="stranger" ${state.transferor_relation === 'stranger' ? 'selected' : ''}>Stranger</option>
                    </select>
                </label>
                <label><span data-i18n="view.s102.label.donative">Donative intent?</span>
                    <input type="checkbox" name="is_donative_intent" ${state.is_donative_intent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.s102a">§ 102(a) excluded?</span>
                    <input type="checkbox" name="s102_a_exclusion_applies" ${state.s102_a_exclusion_applies ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.s102b">§ 102(b) income taxable?</span>
                    <input type="checkbox" name="s102_b_income_taxable_to_recipient" ${state.s102_b_income_taxable_to_recipient ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.s102c">§ 102(c) emp-emp?</span>
                    <input type="checkbox" name="s102_c_employer_employee" ${state.s102_c_employer_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.duberstein">Duberstein test?</span>
                    <input type="checkbox" name="duberstein_test_met" ${state.duberstein_test_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.disguised">Disguised comp?</span>
                    <input type="checkbox" name="is_compensation_disguised_as_gift" ${state.is_compensation_disguised_as_gift ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.s274">§ 274 business gift $25?</span>
                    <input type="checkbox" name="is_business_gift_dedn_25" ${state.is_business_gift_dedn_25 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.basis">Transferor basis ($)</span>
                    <input type="number" step="100" name="transferor_dedn_basis" value="${state.transferor_dedn_basis}"></label>
                <label><span data-i18n="view.s102.label.s1015_carry">§ 1015 carryover ($)</span>
                    <input type="number" step="100" name="s1015_carryover_basis_gift" value="${state.s1015_carryover_basis_gift}"></label>
                <label><span data-i18n="view.s102.label.s1014_step">§ 1014 step-up ($)</span>
                    <input type="number" step="100" name="s1014_step_up_inheritance" value="${state.s1014_step_up_inheritance}"></label>
                <label><span data-i18n="view.s102.label.fmv">FMV at transfer ($)</span>
                    <input type="number" step="100" name="fmv_at_transfer" value="${state.fmv_at_transfer}"></label>
                <label><span data-i18n="view.s102.label.donor_basis">Donor adj basis ($)</span>
                    <input type="number" step="100" name="donor_adjusted_basis" value="${state.donor_adjusted_basis}"></label>
                <label><span data-i18n="view.s102.label.tacks">Holding tacks?</span>
                    <input type="checkbox" name="holding_period_donee_tacks" ${state.holding_period_donee_tacks ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.dual">§ 1015(d) dual basis?</span>
                    <input type="checkbox" name="s1015_d_dual_basis" ${state.s1015_d_dual_basis ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.appreciated">Appreciated property?</span>
                    <input type="checkbox" name="appreciated_property" ${state.appreciated_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.gift_tax">Gift tax paid ($)</span>
                    <input type="number" step="100" name="gift_tax_paid_by_donor" value="${state.gift_tax_paid_by_donor}"></label>
                <label><span data-i18n="view.s102.label.gift_tax_adj">§ 1015(d) adj ($)</span>
                    <input type="number" step="100" name="s1015_d_basis_adjustment" value="${state.s1015_d_basis_adjustment}"></label>
                <label><span data-i18n="view.s102.label.bargain">Part sale, part gift?</span>
                    <input type="checkbox" name="is_part_sale_part_gift" ${state.is_part_sale_part_gift ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.bargain_basis">Bargain basis ($)</span>
                    <input type="number" step="100" name="bargain_sale_basis_allocation" value="${state.bargain_sale_basis_allocation}"></label>
                <label><span data-i18n="view.s102.label.political">Political gift?</span>
                    <input type="checkbox" name="is_political_gift" ${state.is_political_gift ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.s102c_award">§ 102(c) emp award ($)</span>
                    <input type="number" step="100" name="s102_c_employer_award" value="${state.s102_c_employer_award}"></label>
                <label><span data-i18n="view.s102.label.s274j">§ 274(j) achiev award ($)</span>
                    <input type="number" step="100" name="s274_j_employee_achievement_award" value="${state.s274_j_employee_achievement_award}"></label>
                <label><span data-i18n="view.s102.label.qualified_plan">Qual plan award ($)</span>
                    <input type="number" step="100" name="s274_j_qualified_plan_award" value="${state.s274_j_qualified_plan_award}"></label>
                <label><span data-i18n="view.s102.label.non_qualified">Non-qual award ($)</span>
                    <input type="number" step="100" name="s274_j_non_qualified_award" value="${state.s274_j_non_qualified_award}"></label>
                <label><span data-i18n="view.s102.label.s132_de">§ 132(b) de minimis ($)</span>
                    <input type="number" step="100" name="s132_b_gift_de_minimis" value="${state.s132_b_gift_de_minimis}"></label>
                <label><span data-i18n="view.s102.label.inheritance">Inheritance?</span>
                    <input type="checkbox" name="is_inheritance" ${state.is_inheritance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.inh_amt">Inheritance ($)</span>
                    <input type="number" step="1000" name="inheritance_amount" value="${state.inheritance_amount}"></label>
                <label><span data-i18n="view.s102.label.devise">Devise?</span>
                    <input type="checkbox" name="is_devise" ${state.is_devise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.via_will">Via will?</span>
                    <input type="checkbox" name="devise_through_will" ${state.devise_through_will ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.bequest">Bequest?</span>
                    <input type="checkbox" name="is_bequest" ${state.is_bequest ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.beq_amt">Bequest amt ($)</span>
                    <input type="number" step="1000" name="bequest_specific_amount" value="${state.bequest_specific_amount}"></label>
                <label><span data-i18n="view.s102.label.s101a">§ 101(a) life ins?</span>
                    <input type="checkbox" name="s101_a_life_insurance" ${state.s101_a_life_insurance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.ins_amt">Life ins excl ($)</span>
                    <input type="number" step="1000" name="life_insurance_proceeds_excluded" value="${state.life_insurance_proceeds_excluded}"></label>
                <label><span data-i18n="view.s102.label.step_up">Step-up eligible?</span>
                    <input type="checkbox" name="is_step_up_basis_eligible" ${state.is_step_up_basis_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.fmv_dod">FMV at DoD ($)</span>
                    <input type="number" step="1000" name="s1014_FMV_date_of_death" value="${state.s1014_FMV_date_of_death}"></label>
                <label><span data-i18n="view.s102.label.s1014e">§ 1014(e) 1-yr?</span>
                    <input type="checkbox" name="s1014_e_inherited_from_decedent_1y_rule" ${state.s1014_e_inherited_from_decedent_1y_rule ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.s691">§ 691 IRD ($)</span>
                    <input type="number" step="1000" name="s691_a_income_in_respect_of_decedent" value="${state.s691_a_income_in_respect_of_decedent}"></label>
                <label><span data-i18n="view.s102.label.retirement">Inherited retirement?</span>
                    <input type="checkbox" name="inherited_retirement_acct" ${state.inherited_retirement_acct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.appreciated_inh">Appreciated sec?</span>
                    <input type="checkbox" name="inherited_appreciated_securities" ${state.inherited_appreciated_securities ? 'checked' : ''}></label>
                <label><span data-i18n="view.s102.label.estate_tax_iir">Estate tax on IRD ($)</span>
                    <input type="number" step="1000" name="estate_tax_on_iir" value="${state.estate_tax_on_iir}"></label>
                <button class="primary" type="submit" data-i18n="view.s102.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s102-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.duberstein">Duberstein "gift" test</h2>
            <ul class="muted small">
                <li data-i18n="view.s102.dub.case">Comm. v. Duberstein, 363 U.S. 278 (1960)</li>
                <li data-i18n="view.s102.dub.test">"Detached and disinterested generosity" — out of love, affection, respect, charity</li>
                <li data-i18n="view.s102.dub.constraining">NOT constraining force / moral duty / business expectation</li>
                <li data-i18n="view.s102.dub.intent">Donor's intent CONTROLS (objective evidence of motivation)</li>
                <li data-i18n="view.s102.dub.context">Context: family vs business; surprise vs anticipated; quid-pro-quo</li>
                <li data-i18n="view.s102.dub.no_legal">Legal obligation defeats gift status</li>
                <li data-i18n="view.s102.dub.consideration">Past or implied consideration defeats gift status</li>
                <li data-i18n="view.s102.dub.tips">Tips: NOT gifts (Olk v. United States) — even if voluntary</li>
                <li data-i18n="view.s102.dub.crowdfunding">Crowdfunding / GoFundMe: typically gift if non-business + no return expected</li>
                <li data-i18n="view.s102.dub.windfall">Windfall + lottery: NOT gift (winner has earned via game)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.s102c">§ 102(c) employer-employee exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s102.s102c.purpose">Anti-abuse: prevents disguising compensation as gift</li>
                <li data-i18n="view.s102.s102c.applies">Transfer from EMPLOYER to EMPLOYEE (current or former) presumed comp</li>
                <li data-i18n="view.s102.s102c.s274_j_exception">§ 274(j) employee achievement awards EXCLUDED from § 102(c)</li>
                <li data-i18n="view.s102.s102c.s132_b">§ 132(b) de minimis fringe benefits — separately excluded</li>
                <li data-i18n="view.s102.s102c.s132_a_4">§ 132(a)(4) other fringe benefits</li>
                <li data-i18n="view.s102.s102c.bona_fide_owner">Owner-employee from personal funds: may rebut presumption (facts &amp; circumstances)</li>
                <li data-i18n="view.s102.s102c.business_purpose">Business purpose evidence: pay-for-performance vs gift on personal occasion</li>
                <li data-i18n="view.s102.s102c.client_referral">Customer-to-employee tips: NOT gift (incentivized service)</li>
                <li data-i18n="view.s102.s102c.bonus_disguise">Year-end "bonus" labeled gift: § 102(c) applies</li>
                <li data-i18n="view.s102.s102c.death_benefit">Employer death benefit to family: § 102(c) presumed but rebuttal possible (§ 101(b))</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.s274j">§ 274(j) employee achievement awards</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s102.tbl.type">Award type</th><th data-i18n="view.s102.tbl.limit">Limit</th><th data-i18n="view.s102.tbl.exclusion">Exclusion</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s102.tbl.qualified">Qualified plan (in established plan)</td><td>$1,600</td><td data-i18n="view.s102.tbl.full_qual">Full to extent of limit</td></tr>
                    <tr><td data-i18n="view.s102.tbl.non_qualified">Non-qualified (informal)</td><td>$400</td><td data-i18n="view.s102.tbl.full_non">Full to extent of limit</td></tr>
                    <tr><td data-i18n="view.s102.tbl.combined">Combined (both)</td><td>$1,600 max</td><td data-i18n="view.s102.tbl.combined_cap">Combined cap</td></tr>
                    <tr><td data-i18n="view.s102.tbl.tangible">Tangible personal property only</td><td>—</td><td data-i18n="view.s102.tbl.no_cash">NO cash, cash equivalent, gift card, stocks</td></tr>
                    <tr><td data-i18n="view.s102.tbl.length_safety">Length of service / safety achievement only</td><td>—</td><td data-i18n="view.s102.tbl.no_recent">No within 5 years (length) / 10% (safety)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.s1015">§ 1015 carryover basis (gifts)</h2>
            <ul class="muted small">
                <li data-i18n="view.s102.s1015.general">Donee's basis = DONOR's adjusted basis (carryover)</li>
                <li data-i18n="view.s102.s1015.s1015_a">§ 1015(a) — carryover basis on gift</li>
                <li data-i18n="view.s102.s1015.dual_basis">§ 1015(a) dual basis: for LOSS purposes = LESSER of donor basis OR FMV at gift</li>
                <li data-i18n="view.s102.s1015.holding_tacks">Holding period TACKS — donee holds since donor's acquisition</li>
                <li data-i18n="view.s102.s1015.gift_tax_adj">§ 1015(d) basis adjustment: + portion of gift tax paid attributable to net appreciation</li>
                <li data-i18n="view.s102.s1015.formula">Adjustment = gift tax × (net appreciation / FMV at gift)</li>
                <li data-i18n="view.s102.s1015.bargain_sale">Bargain sale: basis allocation between sale portion + gift portion</li>
                <li data-i18n="view.s102.s1015.s2701_2704">§ 2701-2704 — special valuation rules for family transfers (estate planning)</li>
                <li data-i18n="view.s102.s1015.f709">Form 709 — gift tax return (annual exclusion + lifetime exemption)</li>
                <li data-i18n="view.s102.s1015.exemption_2024">2024 annual exclusion $18,000 / lifetime exemption $13.61M</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.s1014">§ 1014 stepped-up basis (inheritance)</h2>
            <ul class="muted small">
                <li data-i18n="view.s102.s1014.general">Inherited property: basis = FMV at decedent's DATE OF DEATH</li>
                <li data-i18n="view.s102.s1014.alt">§ 2032 alternate valuation date: 6 months after death (if elected at estate level)</li>
                <li data-i18n="view.s102.s1014.long_term">Holding period: ALWAYS long-term (§ 1223(9))</li>
                <li data-i18n="view.s102.s1014.unrealized_gain_extinguished">Unrealized gain at death: ELIMINATED — major estate planning benefit</li>
                <li data-i18n="view.s102.s1014.community_property">Community property double step-up at first death (§ 1014(b)(6))</li>
                <li data-i18n="view.s102.s1014.s1014_e">§ 1014(e) — gift within 1 year of death + back to donor: NO step-up</li>
                <li data-i18n="view.s102.s1014.IRD_exception">§ 691 income in respect of decedent (IRD): NOT stepped up</li>
                <li data-i18n="view.s102.s1014.IRD_examples">IRD: unrealized salary, accrued interest, traditional IRA / 401(k), installment notes</li>
                <li data-i18n="view.s102.s1014.s691_c">§ 691(c) estate tax deduction on IRD — offsets income tax on IRD</li>
                <li data-i18n="view.s102.s1014.s2056_marital">§ 2056 marital deduction unlimited — no estate tax + step-up preserved</li>
                <li data-i18n="view.s102.s1014.beneficiary_step_up">Inherited Roth IRA: still tax-free + 10-year rule (post-SECURE)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.special_situations">Special situations</h2>
            <ul class="muted small">
                <li data-i18n="view.s102.spec.tips">Tips: § 61 ordinary income (NOT gift) — even when voluntary</li>
                <li data-i18n="view.s102.spec.scholarship">§ 117 scholarship: tax-free for qualified educational expenses</li>
                <li data-i18n="view.s102.spec.crowdfunding">Crowdfunding (GoFundMe etc.): facts &amp; circumstances — typically gift if donative intent</li>
                <li data-i18n="view.s102.spec.support_payments">Support payments to family: typically gift (§ 102(a)) — not income</li>
                <li data-i18n="view.s102.spec.tip_jar">Tip jar collections: § 61 income (not gift)</li>
                <li data-i18n="view.s102.spec.prizes">§ 74 prizes + awards: GENERALLY taxable (vs § 102 gift)</li>
                <li data-i18n="view.s102.spec.s74_b_charitable_exception">§ 74(b) — Nobel Prize/similar: excluded if assigned to charity</li>
                <li data-i18n="view.s102.spec.political">Political gifts to campaigns: tax-free to candidate (§ 527)</li>
                <li data-i18n="view.s102.spec.kickback_disguised">"Kickbacks" disguised as gifts: § 162(c) — not gift, illegal payment</li>
                <li data-i18n="view.s102.spec.bribery">Bribes: § 162(c) NOT deductible — NOT gift to recipient (illegal)</li>
            </ul>
        </div>
    `;
    document.getElementById('s102-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transfer_type = fd.get('transfer_type');
        state.transferor_relation = fd.get('transferor_relation');
        state.is_donative_intent = !!fd.get('is_donative_intent');
        state.s102_a_exclusion_applies = !!fd.get('s102_a_exclusion_applies');
        state.s102_b_income_taxable_to_recipient = !!fd.get('s102_b_income_taxable_to_recipient');
        state.s102_c_employer_employee = !!fd.get('s102_c_employer_employee');
        state.duberstein_test_met = !!fd.get('duberstein_test_met');
        state.is_compensation_disguised_as_gift = !!fd.get('is_compensation_disguised_as_gift');
        state.is_business_gift_dedn_25 = !!fd.get('is_business_gift_dedn_25');
        state.transferor_dedn_basis = Number(fd.get('transferor_dedn_basis')) || 0;
        state.s1015_carryover_basis_gift = Number(fd.get('s1015_carryover_basis_gift')) || 0;
        state.s1014_step_up_inheritance = Number(fd.get('s1014_step_up_inheritance')) || 0;
        state.fmv_at_transfer = Number(fd.get('fmv_at_transfer')) || 0;
        state.donor_adjusted_basis = Number(fd.get('donor_adjusted_basis')) || 0;
        state.holding_period_donee_tacks = !!fd.get('holding_period_donee_tacks');
        state.s1015_d_dual_basis = !!fd.get('s1015_d_dual_basis');
        state.appreciated_property = !!fd.get('appreciated_property');
        state.gift_tax_paid_by_donor = Number(fd.get('gift_tax_paid_by_donor')) || 0;
        state.s1015_d_basis_adjustment = Number(fd.get('s1015_d_basis_adjustment')) || 0;
        state.is_part_sale_part_gift = !!fd.get('is_part_sale_part_gift');
        state.bargain_sale_basis_allocation = Number(fd.get('bargain_sale_basis_allocation')) || 0;
        state.is_political_gift = !!fd.get('is_political_gift');
        state.s102_c_employer_award = Number(fd.get('s102_c_employer_award')) || 0;
        state.s274_j_employee_achievement_award = Number(fd.get('s274_j_employee_achievement_award')) || 0;
        state.s274_j_qualified_plan_award = Number(fd.get('s274_j_qualified_plan_award')) || 0;
        state.s274_j_non_qualified_award = Number(fd.get('s274_j_non_qualified_award')) || 0;
        state.s132_b_gift_de_minimis = Number(fd.get('s132_b_gift_de_minimis')) || 0;
        state.is_inheritance = !!fd.get('is_inheritance');
        state.inheritance_amount = Number(fd.get('inheritance_amount')) || 0;
        state.is_devise = !!fd.get('is_devise');
        state.devise_through_will = !!fd.get('devise_through_will');
        state.is_bequest = !!fd.get('is_bequest');
        state.bequest_specific_amount = Number(fd.get('bequest_specific_amount')) || 0;
        state.s101_a_life_insurance = !!fd.get('s101_a_life_insurance');
        state.life_insurance_proceeds_excluded = Number(fd.get('life_insurance_proceeds_excluded')) || 0;
        state.is_step_up_basis_eligible = !!fd.get('is_step_up_basis_eligible');
        state.s1014_FMV_date_of_death = Number(fd.get('s1014_FMV_date_of_death')) || 0;
        state.s1014_e_inherited_from_decedent_1y_rule = !!fd.get('s1014_e_inherited_from_decedent_1y_rule');
        state.s691_a_income_in_respect_of_decedent = Number(fd.get('s691_a_income_in_respect_of_decedent')) || 0;
        state.inherited_retirement_acct = !!fd.get('inherited_retirement_acct');
        state.inherited_appreciated_securities = !!fd.get('inherited_appreciated_securities');
        state.estate_tax_on_iir = Number(fd.get('estate_tax_on_iir')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s102-output');
    if (!el) return;
    const excluded = state.is_donative_intent && !state.s102_c_employer_employee;
    const basis = state.transfer_type === 'gift' ? state.donor_adjusted_basis : state.s1014_FMV_date_of_death;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s102.h2.result">§ 102 exclusion analysis</h2>
            <div class="cards">
                <div class="card ${excluded ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s102.card.exclude">§ 102(a) excluded?</div><div class="value">${excluded ? 'YES' : 'NO'}</div></div>
                <div class="card ${state.s102_c_employer_employee ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s102.card.s102c">§ 102(c) trigger?</div><div class="value">${state.s102_c_employer_employee ? 'YES (comp)' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s102.card.fmv">FMV</div><div class="value">$${state.fmv_at_transfer.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s102.card.basis">Donee basis</div><div class="value">$${basis.toLocaleString()}</div></div>
                <div class="card ${state.s691_a_income_in_respect_of_decedent > 0 ? 'warn' : ''}"><div class="label" data-i18n="view.s102.card.ird">§ 691 IRD?</div><div class="value">${state.s691_a_income_in_respect_of_decedent > 0 ? '$'+state.s691_a_income_in_respect_of_decedent.toLocaleString()+' (no step-up)' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
