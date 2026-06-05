// IRC § 421 — General Rules for Qualified Stock Options.
// Parent provision for § 422 ISO + § 423 ESPP statutory stock options.
// § 421(a) — no income inclusion at exercise OR vesting (until sale).
// § 421(b) — disqualifying disposition (sale before holding period) → § 83 ordinary income.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    option_type: 'iso',
    is_qualified_iso: false,
    is_qualified_espp: false,
    is_nso_nonqualified: false,
    grant_date: '',
    exercise_date: '',
    sale_date: '',
    fmv_at_grant: 0,
    fmv_at_exercise: 0,
    exercise_price: 0,
    sale_price: 0,
    shares: 0,
    s421_a_no_recognition_at_exercise: true,
    s421_b_disqualifying_disposition: false,
    s422_iso_2_year_grant: false,
    s422_iso_1_year_exercise: false,
    s422_qualifying_holding_met: false,
    s423_espp_15pct_discount_max: false,
    s423_espp_2_year_grant: false,
    s423_espp_1_year_purchase: false,
    s423_lookback_provision: false,
    s422_d_100k_annual_limit_iso: 0,
    s422_b_2_employment_requirement: false,
    s422_b_3_5pct_owner_limit: false,
    is_5pct_owner_at_grant: false,
    s422_c_3_terms_of_option: false,
    s56_b_3_amt_spread: 0,
    s53_amt_credit_recovery: 0,
    s421_b_w2_box_12_v_code: false,
    s421_b_w2_box_12_q_code: false,
    s421_b_ordinary_income: 0,
    s421_b_capital_gain: 0,
    s421_b_capital_loss: 0,
    s6039_form_3921_filed: false,
    s6039_form_3922_filed: false,
    is_cashless_exercise: false,
    is_same_day_sale: false,
    is_sell_to_cover: false,
    s83_b_election_made: false,
    s83_b_60_day_deadline_met: false,
    is_employee_at_grant: true,
    is_employee_at_exercise: true,
    months_post_termination_3_year: 0,
    s422_a_2_employment_3_months: false,
    death_disability_extension: false,
    s422_a_3_no_disposition_2_years: false,
};

export async function renderSection421(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s421.h1.title">// § 421 QUALIFIED STOCK OPTIONS (ISO + ESPP)</span></h1>
        <p class="muted small" data-i18n="view.s421.hint.intro">
            <strong>§ 421</strong> umbrella rules for qualified stock options. <strong>§ 422 ISO</strong>
            (Incentive Stock Options): NO ordinary income at exercise; LTCG on sale IF held 2 years
            from grant + 1 year from exercise. <strong>AMT TRAP</strong> — § 56(b)(3) — spread at
            exercise IS AMT preference. <strong>§ 423 ESPP</strong>: 15% max discount + lookback +
            2 years grant + 1 year purchase. <strong>§ 421(b) disqualifying disposition</strong> —
            ordinary income on spread (FMV at exercise - exercise price) + § 421(a) protection
            forfeited. <strong>§ 422(d) $100K annual limit:</strong> first $100K worth of ISO grants
            per year eligible (excess = NSO). <strong>§ 6039:</strong> Form 3921 (ISO exercise) +
            Form 3922 (ESPP transfer) — mandatory reporting. <strong>§ 422(a)(2):</strong> exercise
            within 3 months of employment termination required (longer for death/disability).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.inputs">Inputs</h2>
            <form id="s421-form" class="inline-form">
                <label><span data-i18n="view.s421.label.type">Option type</span>
                    <select name="option_type">
                        <option value="iso" ${state.option_type === 'iso' ? 'selected' : ''}>§ 422 ISO</option>
                        <option value="espp" ${state.option_type === 'espp' ? 'selected' : ''}>§ 423 ESPP</option>
                        <option value="nso" ${state.option_type === 'nso' ? 'selected' : ''}>NSO (non-qualified)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s421.label.qiso">Qualified ISO?</span>
                    <input type="checkbox" name="is_qualified_iso" ${state.is_qualified_iso ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.qespp">Qualified ESPP?</span>
                    <input type="checkbox" name="is_qualified_espp" ${state.is_qualified_espp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.nso">NSO?</span>
                    <input type="checkbox" name="is_nso_nonqualified" ${state.is_nso_nonqualified ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.grant">Grant date</span>
                    <input type="date" name="grant_date" value="${state.grant_date}"></label>
                <label><span data-i18n="view.s421.label.exercise">Exercise date</span>
                    <input type="date" name="exercise_date" value="${state.exercise_date}"></label>
                <label><span data-i18n="view.s421.label.sale">Sale date</span>
                    <input type="date" name="sale_date" value="${state.sale_date}"></label>
                <label><span data-i18n="view.s421.label.fmv_grant">FMV grant ($)</span>
                    <input type="number" step="0.01" name="fmv_at_grant" value="${state.fmv_at_grant}"></label>
                <label><span data-i18n="view.s421.label.fmv_exercise">FMV exercise ($)</span>
                    <input type="number" step="0.01" name="fmv_at_exercise" value="${state.fmv_at_exercise}"></label>
                <label><span data-i18n="view.s421.label.price">Exercise price ($)</span>
                    <input type="number" step="0.01" name="exercise_price" value="${state.exercise_price}"></label>
                <label><span data-i18n="view.s421.label.sale_price">Sale price ($)</span>
                    <input type="number" step="0.01" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s421.label.shares">Shares</span>
                    <input type="number" step="1" name="shares" value="${state.shares}"></label>
                <label><span data-i18n="view.s421.label.s421a">§ 421(a) no exercise inc?</span>
                    <input type="checkbox" name="s421_a_no_recognition_at_exercise" ${state.s421_a_no_recognition_at_exercise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.s421b">§ 421(b) disqual disp?</span>
                    <input type="checkbox" name="s421_b_disqualifying_disposition" ${state.s421_b_disqualifying_disposition ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.iso_2y">ISO 2-yr grant?</span>
                    <input type="checkbox" name="s422_iso_2_year_grant" ${state.s422_iso_2_year_grant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.iso_1y">ISO 1-yr exercise?</span>
                    <input type="checkbox" name="s422_iso_1_year_exercise" ${state.s422_iso_1_year_exercise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.qualifying">Qualifying holding?</span>
                    <input type="checkbox" name="s422_qualifying_holding_met" ${state.s422_qualifying_holding_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.espp_15pct">ESPP 15% max?</span>
                    <input type="checkbox" name="s423_espp_15pct_discount_max" ${state.s423_espp_15pct_discount_max ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.espp_2y">ESPP 2-yr grant?</span>
                    <input type="checkbox" name="s423_espp_2_year_grant" ${state.s423_espp_2_year_grant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.espp_1y">ESPP 1-yr purchase?</span>
                    <input type="checkbox" name="s423_espp_1_year_purchase" ${state.s423_espp_1_year_purchase ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.lookback">ESPP lookback?</span>
                    <input type="checkbox" name="s423_lookback_provision" ${state.s423_lookback_provision ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.s422d">§ 422(d) $100K limit ($)</span>
                    <input type="number" step="0.01" name="s422_d_100k_annual_limit_iso" value="${state.s422_d_100k_annual_limit_iso}"></label>
                <label><span data-i18n="view.s421.label.employment">§ 422(b)(2) empl?</span>
                    <input type="checkbox" name="s422_b_2_employment_requirement" ${state.s422_b_2_employment_requirement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.5pct">5%+ owner?</span>
                    <input type="checkbox" name="s422_b_3_5pct_owner_limit" ${state.s422_b_3_5pct_owner_limit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.5pct_grant">5% at grant?</span>
                    <input type="checkbox" name="is_5pct_owner_at_grant" ${state.is_5pct_owner_at_grant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.terms">§ 422(c)(3) terms?</span>
                    <input type="checkbox" name="s422_c_3_terms_of_option" ${state.s422_c_3_terms_of_option ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.amt">§ 56(b)(3) AMT spread ($)</span>
                    <input type="number" step="0.01" name="s56_b_3_amt_spread" value="${state.s56_b_3_amt_spread}"></label>
                <label><span data-i18n="view.s421.label.amt_credit">§ 53 AMT credit ($)</span>
                    <input type="number" step="0.01" name="s53_amt_credit_recovery" value="${state.s53_amt_credit_recovery}"></label>
                <label><span data-i18n="view.s421.label.box12v">W-2 Box 12 V?</span>
                    <input type="checkbox" name="s421_b_w2_box_12_v_code" ${state.s421_b_w2_box_12_v_code ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.box12q">W-2 Box 12 Q?</span>
                    <input type="checkbox" name="s421_b_w2_box_12_q_code" ${state.s421_b_w2_box_12_q_code ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.ord">Ordinary income ($)</span>
                    <input type="number" step="0.01" name="s421_b_ordinary_income" value="${state.s421_b_ordinary_income}"></label>
                <label><span data-i18n="view.s421.label.cap_gain">Cap gain ($)</span>
                    <input type="number" step="0.01" name="s421_b_capital_gain" value="${state.s421_b_capital_gain}"></label>
                <label><span data-i18n="view.s421.label.cap_loss">Cap loss ($)</span>
                    <input type="number" step="0.01" name="s421_b_capital_loss" value="${state.s421_b_capital_loss}"></label>
                <label><span data-i18n="view.s421.label.f3921">Form 3921?</span>
                    <input type="checkbox" name="s6039_form_3921_filed" ${state.s6039_form_3921_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.f3922">Form 3922?</span>
                    <input type="checkbox" name="s6039_form_3922_filed" ${state.s6039_form_3922_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.cashless">Cashless exercise?</span>
                    <input type="checkbox" name="is_cashless_exercise" ${state.is_cashless_exercise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.same_day">Same-day sale?</span>
                    <input type="checkbox" name="is_same_day_sale" ${state.is_same_day_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.sell_cover">Sell-to-cover?</span>
                    <input type="checkbox" name="is_sell_to_cover" ${state.is_sell_to_cover ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.s83b">§ 83(b) election?</span>
                    <input type="checkbox" name="s83_b_election_made" ${state.s83_b_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.s83b_60">§ 83(b) 60 days met?</span>
                    <input type="checkbox" name="s83_b_60_day_deadline_met" ${state.s83_b_60_day_deadline_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.emp_grant">Employee at grant?</span>
                    <input type="checkbox" name="is_employee_at_grant" ${state.is_employee_at_grant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.emp_ex">Employee at exercise?</span>
                    <input type="checkbox" name="is_employee_at_exercise" ${state.is_employee_at_exercise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.months_post">Months post-term</span>
                    <input type="number" step="1" name="months_post_termination_3_year" value="${state.months_post_termination_3_year}"></label>
                <label><span data-i18n="view.s421.label.s422_a2">§ 422(a)(2) 3-mo?</span>
                    <input type="checkbox" name="s422_a_2_employment_3_months" ${state.s422_a_2_employment_3_months ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.death">Death/disab ext?</span>
                    <input type="checkbox" name="death_disability_extension" ${state.death_disability_extension ? 'checked' : ''}></label>
                <label><span data-i18n="view.s421.label.s422_a3">§ 422(a)(3) 2-yr?</span>
                    <input type="checkbox" name="s422_a_3_no_disposition_2_years" ${state.s422_a_3_no_disposition_2_years ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s421.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s421-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.iso_vs_nso">ISO vs NSO comparison</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s421.tbl.attr">Attribute</th><th>§ 422 ISO</th><th>NSO (non-qualified)</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s421.tbl.exercise">At exercise</td><td data-i18n="view.s421.tbl.no_ord">No ordinary income (§ 421(a))</td><td data-i18n="view.s421.tbl.spread_ord">Spread = ordinary income (§ 83)</td></tr>
                    <tr><td data-i18n="view.s421.tbl.amt">AMT</td><td data-i18n="view.s421.tbl.spread_amt">Spread = AMT preference (§ 56(b)(3))</td><td>No AMT (already ordinary)</td></tr>
                    <tr><td data-i18n="view.s421.tbl.qualifying">Qualifying sale</td><td data-i18n="view.s421.tbl.ltcg">100% LTCG (2-yr grant + 1-yr exercise)</td><td>LTCG / STCG on subsequent appreciation</td></tr>
                    <tr><td data-i18n="view.s421.tbl.disqualifying">Disqualifying sale</td><td data-i18n="view.s421.tbl.disq_treatment">Spread = ordinary up to actual gain (§ 421(b))</td><td>N/A</td></tr>
                    <tr><td data-i18n="view.s421.tbl.fica">FICA</td><td>NO (§ 3121(a)(22))</td><td>YES (compensation)</td></tr>
                    <tr><td data-i18n="view.s421.tbl.s422_d">$100K annual limit</td><td>YES (§ 422(d))</td><td>NO</td></tr>
                    <tr><td data-i18n="view.s421.tbl.employer_dedn">Employer deduction</td><td data-i18n="view.s421.tbl.disq_only">Only on disqualifying disposition</td><td>YES on exercise</td></tr>
                    <tr><td data-i18n="view.s421.tbl.holding">Holding 2yr/1yr</td><td>Required for ISO benefits</td><td>N/A — start at exercise</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.iso_requirements">§ 422 ISO requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s421.iso.written_plan">Written plan approved by shareholders within 12 months of adoption</li>
                <li data-i18n="view.s421.iso.10_year_grant">Granted within 10 years of plan adoption or approval</li>
                <li data-i18n="view.s421.iso.10_year_exercise">Exercised within 10 years of grant</li>
                <li data-i18n="view.s421.iso.fmv_grant">Exercise price ≥ FMV at grant</li>
                <li data-i18n="view.s421.iso.5pct_110pct">5%+ owners: exercise price ≥ 110% FMV + 5-year exercise period</li>
                <li data-i18n="view.s421.iso.non_transferable">Non-transferable except by will/intestacy</li>
                <li data-i18n="view.s421.iso.employee">Granted to employee at grant + employed within 3 months pre-exercise</li>
                <li data-i18n="view.s421.iso.s422_d_100k">§ 422(d) $100K annual limit: first $100K of FMV per calendar year</li>
                <li data-i18n="view.s421.iso.excess_NSO">Excess over $100K = NSO (separately tracked + reported)</li>
                <li data-i18n="view.s421.iso.disqualifying_2_1">Disqualifying: sale before 2-yr grant OR 1-yr exercise</li>
                <li data-i18n="view.s421.iso.qualifying_LTCG">Qualifying disposition: 100% LTCG on (sale price − strike)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.espp_requirements">§ 423 ESPP requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s421.espp.written_plan">Written plan approved by shareholders within 12 months</li>
                <li data-i18n="view.s421.espp.15pct">Maximum 15% discount on LOWER of FMV at grant or FMV at purchase</li>
                <li data-i18n="view.s421.espp.lookback">Lookback feature: discount applied to LOWER of grant or purchase FMV</li>
                <li data-i18n="view.s421.espp.broad_based">Broad-based: all employees (limited exclusions: HCEs, &lt; 2 yrs service, &lt; 20 hrs/week)</li>
                <li data-i18n="view.s421.espp.no_5pct">5%+ owners EXCLUDED</li>
                <li data-i18n="view.s421.espp.same_rights">Same rights + privileges across employees (capped per amount)</li>
                <li data-i18n="view.s421.espp.25k_limit">$25,000 annual purchase limit (FMV at grant — not purchase)</li>
                <li data-i18n="view.s421.espp.27_month">Maximum 27-month offering period</li>
                <li data-i18n="view.s421.espp.qualifying_2_1">Qualifying: 2 yrs from BEGINNING of offering + 1 yr from purchase</li>
                <li data-i18n="view.s421.espp.tax_qualifying">Qualifying: ordinary income = lesser of (discount) OR (gain); rest = LTCG</li>
                <li data-i18n="view.s421.espp.disqualifying">Disqualifying: full spread (FMV at purchase − purchase price) = ordinary</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.amt_trap">§ 56(b)(3) AMT trap on ISOs</h2>
            <ul class="muted small">
                <li data-i18n="view.s421.amt.preference">Spread at exercise = AMT preference item (§ 56(b)(3))</li>
                <li data-i18n="view.s421.amt.regular_no">Regular tax: NO ordinary income at exercise (§ 421(a))</li>
                <li data-i18n="view.s421.amt.amt_yes">AMT: ADD spread to AMTI — may trigger AMT liability</li>
                <li data-i18n="view.s421.amt.amt_basis">AMT basis in stock = exercise price + spread</li>
                <li data-i18n="view.s421.amt.regular_basis">Regular basis = exercise price only</li>
                <li data-i18n="view.s421.amt.sale_year_recovery">Subsequent sale: AMT basis reduces AMT gain (vs regular)</li>
                <li data-i18n="view.s421.amt.s53_credit">§ 53 minimum tax credit recovers AMT paid in future years</li>
                <li data-i18n="view.s421.amt.unrefundable">Pre-TCJA: § 53 credit also refundable; post-TCJA non-refundable</li>
                <li data-i18n="view.s421.amt.early_exercise">Early exercise + § 83(b) election: starts holding clock + minimizes spread</li>
                <li data-i18n="view.s421.amt.disqualifying_amt">Disqualifying disposition same year: AVOIDS AMT (ordinary income for both)</li>
                <li data-i18n="view.s421.amt.amt_phase_out">2024 AMT exemption: $85,700 (single) / $133,300 (MFJ)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.disqualifying">§ 421(b) disqualifying disposition</h2>
            <ul class="muted small">
                <li data-i18n="view.s421.dis.trigger">Sale before 2-yr grant OR 1-yr exercise holding</li>
                <li data-i18n="view.s421.dis.ordinary">ORDINARY income = LESSER of (FMV at exercise − strike) OR (sale price − strike)</li>
                <li data-i18n="view.s421.dis.no_amt">NO AMT preference (already ordinary on disqualifying)</li>
                <li data-i18n="view.s421.dis.w2_v">W-2 Box 12 Code V (NSO) or no separate code for ISO disqualifying</li>
                <li data-i18n="view.s421.dis.basis">Basis becomes exercise price + ordinary income</li>
                <li data-i18n="view.s421.dis.capital">Remaining gain = capital (LTCG if held &gt; 1 yr from exercise)</li>
                <li data-i18n="view.s421.dis.employer_dedn">EMPLOYER gets compensation deduction (§ 421(b) + Reg § 1.421-2)</li>
                <li data-i18n="view.s421.dis.same_year_avoid_amt">Disqualifying same calendar year: AVOIDS AMT trap</li>
                <li data-i18n="view.s421.dis.death">Death: NO disqualifying disposition (§ 421(c)(1))</li>
                <li data-i18n="view.s421.dis.s424_attribution">§ 424 attribution rules for related-party dispositions</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.reporting">§ 6039 information reporting</h2>
            <ul class="muted small">
                <li data-i18n="view.s421.rep.f3921">Form 3921 — ISO exercise. Employer files + employee receives Jan 31</li>
                <li data-i18n="view.s421.rep.f3922">Form 3922 — ESPP transfer of legal title. Employer files + employee receives Jan 31</li>
                <li data-i18n="view.s421.rep.s6721">§ 6721 penalty: $310/form failure to file (2024)</li>
                <li data-i18n="view.s421.rep.s6722">§ 6722 penalty: $310/form failure to furnish (2024)</li>
                <li data-i18n="view.s421.rep.due_date">IRS due Feb 28 (paper) / Mar 31 (e-file)</li>
                <li data-i18n="view.s421.rep.basis_tracking">3921/3922 critical for tracking basis + holding period</li>
                <li data-i18n="view.s421.rep.w2_v">W-2 Box 12 Code V — NSO exercise spread (ordinary income)</li>
                <li data-i18n="view.s421.rep.no_iso_code">ISO qualifying disposition: NO W-2 code (capital gain only)</li>
                <li data-i18n="view.s421.rep.disqualifying_box1">ISO disqualifying same year: included in Box 1 wages</li>
                <li data-i18n="view.s421.rep.1099B">1099-B: broker-reported basis often INCORRECT for ISO/ESPP (only purchase price)</li>
            </ul>
        </div>
    `;
    document.getElementById('s421-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.option_type = fd.get('option_type');
        state.is_qualified_iso = !!fd.get('is_qualified_iso');
        state.is_qualified_espp = !!fd.get('is_qualified_espp');
        state.is_nso_nonqualified = !!fd.get('is_nso_nonqualified');
        state.grant_date = fd.get('grant_date') || '';
        state.exercise_date = fd.get('exercise_date') || '';
        state.sale_date = fd.get('sale_date') || '';
        state.fmv_at_grant = Number(fd.get('fmv_at_grant')) || 0;
        state.fmv_at_exercise = Number(fd.get('fmv_at_exercise')) || 0;
        state.exercise_price = Number(fd.get('exercise_price')) || 0;
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.shares = Number(fd.get('shares')) || 0;
        state.s421_a_no_recognition_at_exercise = !!fd.get('s421_a_no_recognition_at_exercise');
        state.s421_b_disqualifying_disposition = !!fd.get('s421_b_disqualifying_disposition');
        state.s422_iso_2_year_grant = !!fd.get('s422_iso_2_year_grant');
        state.s422_iso_1_year_exercise = !!fd.get('s422_iso_1_year_exercise');
        state.s422_qualifying_holding_met = !!fd.get('s422_qualifying_holding_met');
        state.s423_espp_15pct_discount_max = !!fd.get('s423_espp_15pct_discount_max');
        state.s423_espp_2_year_grant = !!fd.get('s423_espp_2_year_grant');
        state.s423_espp_1_year_purchase = !!fd.get('s423_espp_1_year_purchase');
        state.s423_lookback_provision = !!fd.get('s423_lookback_provision');
        state.s422_d_100k_annual_limit_iso = Number(fd.get('s422_d_100k_annual_limit_iso')) || 0;
        state.s422_b_2_employment_requirement = !!fd.get('s422_b_2_employment_requirement');
        state.s422_b_3_5pct_owner_limit = !!fd.get('s422_b_3_5pct_owner_limit');
        state.is_5pct_owner_at_grant = !!fd.get('is_5pct_owner_at_grant');
        state.s422_c_3_terms_of_option = !!fd.get('s422_c_3_terms_of_option');
        state.s56_b_3_amt_spread = Number(fd.get('s56_b_3_amt_spread')) || 0;
        state.s53_amt_credit_recovery = Number(fd.get('s53_amt_credit_recovery')) || 0;
        state.s421_b_w2_box_12_v_code = !!fd.get('s421_b_w2_box_12_v_code');
        state.s421_b_w2_box_12_q_code = !!fd.get('s421_b_w2_box_12_q_code');
        state.s421_b_ordinary_income = Number(fd.get('s421_b_ordinary_income')) || 0;
        state.s421_b_capital_gain = Number(fd.get('s421_b_capital_gain')) || 0;
        state.s421_b_capital_loss = Number(fd.get('s421_b_capital_loss')) || 0;
        state.s6039_form_3921_filed = !!fd.get('s6039_form_3921_filed');
        state.s6039_form_3922_filed = !!fd.get('s6039_form_3922_filed');
        state.is_cashless_exercise = !!fd.get('is_cashless_exercise');
        state.is_same_day_sale = !!fd.get('is_same_day_sale');
        state.is_sell_to_cover = !!fd.get('is_sell_to_cover');
        state.s83_b_election_made = !!fd.get('s83_b_election_made');
        state.s83_b_60_day_deadline_met = !!fd.get('s83_b_60_day_deadline_met');
        state.is_employee_at_grant = !!fd.get('is_employee_at_grant');
        state.is_employee_at_exercise = !!fd.get('is_employee_at_exercise');
        state.months_post_termination_3_year = Number(fd.get('months_post_termination_3_year')) || 0;
        state.s422_a_2_employment_3_months = !!fd.get('s422_a_2_employment_3_months');
        state.death_disability_extension = !!fd.get('death_disability_extension');
        state.s422_a_3_no_disposition_2_years = !!fd.get('s422_a_3_no_disposition_2_years');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s421-output');
    if (!el) return;
    const spread = (state.fmv_at_exercise - state.exercise_price) * state.shares;
    const gain = (state.sale_price - state.exercise_price) * state.shares;
    const qualifying = state.s422_qualifying_holding_met;
    let ord_income = 0, cap_gain = 0;
    if (state.option_type === 'iso' && qualifying) {
        cap_gain = gain;
    } else if (state.option_type === 'iso' && !qualifying) {
        ord_income = Math.min(spread, gain);
        cap_gain = gain - ord_income;
    } else if (state.option_type === 'nso') {
        ord_income = spread;
        cap_gain = (state.sale_price - state.fmv_at_exercise) * state.shares;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s421.h2.result">§ 421 option result</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s421.card.spread">Spread at exercise</div><div class="value">$${spread.toLocaleString()}</div></div>
                <div class="card ${state.option_type === 'iso' && qualifying ? 'pos' : ''}"><div class="label" data-i18n="view.s421.card.qualifying">Qualifying?</div><div class="value">${qualifying ? 'YES' : 'NO'}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s421.card.ord">Ordinary income</div><div class="value">$${ord_income.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s421.card.cap">Capital gain</div><div class="value">$${cap_gain.toLocaleString()}</div></div>
                <div class="card ${state.option_type === 'iso' ? 'warn' : ''}"><div class="label" data-i18n="view.s421.card.amt">AMT preference</div><div class="value">${state.option_type === 'iso' && !qualifying ? '$0 (avoided)' : (state.option_type === 'iso' ? '$'+spread.toLocaleString() : 'N/A')}</div></div>
            </div>
        </div>
    `;
}
