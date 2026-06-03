// IRC § 134 — Certain Military Benefits.
// Combat zone pay exclusion + qualified military benefits exclusion from gross income.
// Coordinates with § 112 (combat pay) + § 121(d)(9) (home sale 5-yr → 10-yr extension for service).

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_active_duty: false,
    is_reservist: false,
    is_national_guard: false,
    service_branch: 'army',
    rank: 'enlisted',
    is_combat_zone_service: false,
    s112_combat_pay_total: 0,
    enlisted_combat_pay_excluded: 0,
    officer_combat_pay_capped: 0,
    officer_pay_cap_2024: 11636,
    combat_zone_start_date: '',
    combat_zone_end_date: '',
    is_hospitalized_due_to_combat: false,
    months_qualifying: 0,
    s134_qualified_benefit: 0,
    benefits_dependents_education: 0,
    benefits_dependents_dependent_care: 0,
    benefits_housing_in_kind: 0,
    benefits_housing_allowance_bah: 0,
    bah_amount: 0,
    bas_amount: 0,
    benefits_moving_expenses_pcs: 0,
    benefits_uniform_allowance: 0,
    benefits_death_gratuity: 0,
    s134_b_2_a_qualified_dep_care: false,
    is_dod_authorized_program: true,
    s121_d_9_home_sale_extension: false,
    months_extended_for_qualified_service: 0,
    extension_period_max_120_months: 0,
    s32_combat_pay_election: false,
    s32_combat_pay_election_amount: 0,
    s24_combat_pay_inclusion: 0,
    s1402_h_clergy_overlap: false,
    s7508_filing_payment_extension: false,
    s7508a_disaster_zone_overlap: false,
    is_special_extended_active_duty: false,
    is_armed_forces_disability: false,
    s104_a_4_disability_excluded: 0,
    s104_a_5_armed_forces_excluded: 0,
};

export async function renderSection134(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s134.h1.title">// § 134 MILITARY BENEFITS EXCLUSION</span></h1>
        <p class="muted small" data-i18n="view.s134.hint.intro">
            <strong>§ 134</strong> excludes from gross income "qualified military benefits" —
            non-cash benefits provided by reason of military service authorized by DoD. <strong>§ 112
            combat zone exclusion:</strong> ENLISTED + WARRANT officers: FULL combat pay excluded.
            <strong>Officers</strong> (commissioned, O-1 and above): capped at highest enlisted basic
            pay + hostile fire / imminent danger pay (~$11,636/month 2024). <strong>§ 121(d)(9)
            home sale 5-yr rule extension:</strong> qualified armed forces service overseas extends
            5-yr ownership/use period UP TO 10 YEARS. <strong>§ 7508 SOL + filing/payment
            extensions:</strong> 180 days + days hospitalized + days in combat zone.
            <strong>§ 32(c)(2)(B)(vi) EITC election:</strong> include / exclude combat pay to maximize.
            <strong>§ 24 CTC:</strong> combat pay treated as earned income. <strong>Form W-2 Box 12
            Code Q:</strong> reports nontaxable combat pay separately. <strong>SCRA</strong>
            (Servicemembers Civil Relief Act) provides separate state-tax + interest-rate protections.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s134.h2.inputs">Inputs</h2>
            <form id="s134-form" class="inline-form">
                <label><span data-i18n="view.s134.label.active">Active duty?</span>
                    <input type="checkbox" name="is_active_duty" ${state.is_active_duty ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.reservist">Reservist?</span>
                    <input type="checkbox" name="is_reservist" ${state.is_reservist ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.guard">National Guard?</span>
                    <input type="checkbox" name="is_national_guard" ${state.is_national_guard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.branch">Service</span>
                    <select name="service_branch">
                        <option value="army" ${state.service_branch === 'army' ? 'selected' : ''}>Army</option>
                        <option value="navy" ${state.service_branch === 'navy' ? 'selected' : ''}>Navy</option>
                        <option value="air_force" ${state.service_branch === 'air_force' ? 'selected' : ''}>Air Force</option>
                        <option value="marines" ${state.service_branch === 'marines' ? 'selected' : ''}>Marines</option>
                        <option value="coast_guard" ${state.service_branch === 'coast_guard' ? 'selected' : ''}>Coast Guard</option>
                        <option value="space_force" ${state.service_branch === 'space_force' ? 'selected' : ''}>Space Force</option>
                    </select>
                </label>
                <label><span data-i18n="view.s134.label.rank">Rank</span>
                    <select name="rank">
                        <option value="enlisted" ${state.rank === 'enlisted' ? 'selected' : ''}>Enlisted (E-1 to E-9)</option>
                        <option value="warrant" ${state.rank === 'warrant' ? 'selected' : ''}>Warrant officer</option>
                        <option value="officer" ${state.rank === 'officer' ? 'selected' : ''}>Officer (O-1 to O-10)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s134.label.combat">Combat zone?</span>
                    <input type="checkbox" name="is_combat_zone_service" ${state.is_combat_zone_service ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.s112_combat">§ 112 combat pay ($)</span>
                    <input type="number" step="100" name="s112_combat_pay_total" value="${state.s112_combat_pay_total}"></label>
                <label><span data-i18n="view.s134.label.enlisted">Enlisted excluded ($)</span>
                    <input type="number" step="100" name="enlisted_combat_pay_excluded" value="${state.enlisted_combat_pay_excluded}"></label>
                <label><span data-i18n="view.s134.label.officer_cap">Officer capped ($)</span>
                    <input type="number" step="100" name="officer_combat_pay_capped" value="${state.officer_combat_pay_capped}"></label>
                <label><span data-i18n="view.s134.label.officer_2024">2024 officer cap ($)</span>
                    <input type="number" step="100" name="officer_pay_cap_2024" value="${state.officer_pay_cap_2024}"></label>
                <label><span data-i18n="view.s134.label.cz_start">CZ start</span>
                    <input type="date" name="combat_zone_start_date" value="${state.combat_zone_start_date}"></label>
                <label><span data-i18n="view.s134.label.cz_end">CZ end</span>
                    <input type="date" name="combat_zone_end_date" value="${state.combat_zone_end_date}"></label>
                <label><span data-i18n="view.s134.label.hospitalized">Hospitalized?</span>
                    <input type="checkbox" name="is_hospitalized_due_to_combat" ${state.is_hospitalized_due_to_combat ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.months">Months qualifying</span>
                    <input type="number" step="1" name="months_qualifying" value="${state.months_qualifying}"></label>
                <label><span data-i18n="view.s134.label.qual_ben">§ 134 qualified benefit ($)</span>
                    <input type="number" step="100" name="s134_qualified_benefit" value="${state.s134_qualified_benefit}"></label>
                <label><span data-i18n="view.s134.label.education">Education benefits ($)</span>
                    <input type="number" step="100" name="benefits_dependents_education" value="${state.benefits_dependents_education}"></label>
                <label><span data-i18n="view.s134.label.dep_care">Dep care benefits ($)</span>
                    <input type="number" step="100" name="benefits_dependents_dependent_care" value="${state.benefits_dependents_dependent_care}"></label>
                <label><span data-i18n="view.s134.label.housing_in_kind">Housing in-kind ($)</span>
                    <input type="number" step="100" name="benefits_housing_in_kind" value="${state.benefits_housing_in_kind}"></label>
                <label><span data-i18n="view.s134.label.housing_bah">BAH ($)</span>
                    <input type="number" step="100" name="benefits_housing_allowance_bah" value="${state.benefits_housing_allowance_bah}"></label>
                <label><span data-i18n="view.s134.label.bah_amt">BAH amount ($)</span>
                    <input type="number" step="100" name="bah_amount" value="${state.bah_amount}"></label>
                <label><span data-i18n="view.s134.label.bas">BAS ($)</span>
                    <input type="number" step="100" name="bas_amount" value="${state.bas_amount}"></label>
                <label><span data-i18n="view.s134.label.moving">PCS moving ($)</span>
                    <input type="number" step="100" name="benefits_moving_expenses_pcs" value="${state.benefits_moving_expenses_pcs}"></label>
                <label><span data-i18n="view.s134.label.uniform">Uniform allow ($)</span>
                    <input type="number" step="100" name="benefits_uniform_allowance" value="${state.benefits_uniform_allowance}"></label>
                <label><span data-i18n="view.s134.label.death">Death gratuity ($)</span>
                    <input type="number" step="100" name="benefits_death_gratuity" value="${state.benefits_death_gratuity}"></label>
                <label><span data-i18n="view.s134.label.s134_b2a">§ 134(b)(2)(A) dep care?</span>
                    <input type="checkbox" name="s134_b_2_a_qualified_dep_care" ${state.s134_b_2_a_qualified_dep_care ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.dod">DoD authorized?</span>
                    <input type="checkbox" name="is_dod_authorized_program" ${state.is_dod_authorized_program ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.s121_d9">§ 121(d)(9) ext?</span>
                    <input type="checkbox" name="s121_d_9_home_sale_extension" ${state.s121_d_9_home_sale_extension ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.months_ext">Months extended</span>
                    <input type="number" step="1" name="months_extended_for_qualified_service" value="${state.months_extended_for_qualified_service}"></label>
                <label><span data-i18n="view.s134.label.max_120">Max 120 mo</span>
                    <input type="number" step="1" name="extension_period_max_120_months" value="${state.extension_period_max_120_months}"></label>
                <label><span data-i18n="view.s134.label.s32_elect">§ 32 combat elect?</span>
                    <input type="checkbox" name="s32_combat_pay_election" ${state.s32_combat_pay_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.s32_elect_amt">§ 32 elect amount ($)</span>
                    <input type="number" step="100" name="s32_combat_pay_election_amount" value="${state.s32_combat_pay_election_amount}"></label>
                <label><span data-i18n="view.s134.label.s24">§ 24 CTC combat ($)</span>
                    <input type="number" step="100" name="s24_combat_pay_inclusion" value="${state.s24_combat_pay_inclusion}"></label>
                <label><span data-i18n="view.s134.label.s1402h">§ 1402(h) clergy?</span>
                    <input type="checkbox" name="s1402_h_clergy_overlap" ${state.s1402_h_clergy_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.s7508">§ 7508 extension?</span>
                    <input type="checkbox" name="s7508_filing_payment_extension" ${state.s7508_filing_payment_extension ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.s7508a">§ 7508A disaster?</span>
                    <input type="checkbox" name="s7508a_disaster_zone_overlap" ${state.s7508a_disaster_zone_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.special_active">Special extended active?</span>
                    <input type="checkbox" name="is_special_extended_active_duty" ${state.is_special_extended_active_duty ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.disability">AF disability?</span>
                    <input type="checkbox" name="is_armed_forces_disability" ${state.is_armed_forces_disability ? 'checked' : ''}></label>
                <label><span data-i18n="view.s134.label.s104a4">§ 104(a)(4) excl ($)</span>
                    <input type="number" step="100" name="s104_a_4_disability_excluded" value="${state.s104_a_4_disability_excluded}"></label>
                <label><span data-i18n="view.s134.label.s104a5">§ 104(a)(5) excl ($)</span>
                    <input type="number" step="100" name="s104_a_5_armed_forces_excluded" value="${state.s104_a_5_armed_forces_excluded}"></label>
                <button class="primary" type="submit" data-i18n="view.s134.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s134-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s134.h2.combat_pay">§ 112 Combat zone pay exclusion</h2>
            <ul class="muted small">
                <li data-i18n="view.s134.cp.enlisted">ENLISTED + WARRANT officers: FULL combat pay excluded</li>
                <li data-i18n="view.s134.cp.officer_cap">Officers: capped at highest enlisted basic pay + Hostile Fire / Imminent Danger pay (HFP/IDP)</li>
                <li data-i18n="view.s134.cp.2024_cap">2024 officer cap: ~$11,636/month</li>
                <li data-i18n="view.s134.cp.month">Any DAY in combat zone counts as ENTIRE MONTH</li>
                <li data-i18n="view.s134.cp.hospitalized">Hospitalized due to wounds/injury sustained in combat zone: continues exclusion</li>
                <li data-i18n="view.s134.cp.duration">Up to 24 months hospitalization eligible (Reg § 1.112-1(c))</li>
                <li data-i18n="view.s134.cp.fica">Combat pay EXEMPT from FICA (SS + Medicare)</li>
                <li data-i18n="view.s134.cp.w2_box_12">Reported W-2 Box 12 Code Q (separately from Box 1)</li>
                <li data-i18n="view.s134.cp.s32_eitc">§ 32(c)(2)(B)(vi) — taxpayer may ELECT to include combat pay for EITC computation</li>
                <li data-i18n="view.s134.cp.s24_ctc">§ 24(d)(1)(B)(ii) — combat pay included for refundable CTC computation</li>
                <li data-i18n="view.s134.cp.s219_g">§ 219(g)(3)(B) — combat pay treated as earned income for IRA</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s134.h2.qualified_benefits">§ 134 Qualified military benefits</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s134.tbl.benefit">Benefit</th><th data-i18n="view.s134.tbl.excludable">Excludable?</th><th data-i18n="view.s134.tbl.citation">Citation</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s134.tbl.bah">Basic Allowance for Housing (BAH)</td><td>YES (full)</td><td>§ 134(b)(3)(A)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.bas">Basic Allowance for Subsistence (BAS)</td><td>YES (full)</td><td>§ 134(b)(3)(B)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.cola">CONUS / OCONUS COLA</td><td>YES</td><td>§ 134(b)(2)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.uniform">Uniform allowance (enlisted)</td><td>YES</td><td>§ 134(b)(3)(C)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.in_kind_quarters">In-kind on-base quarters</td><td>YES</td><td>§ 134(b)(2)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.in_kind_subsistence">In-kind mess subsistence</td><td>YES</td><td>§ 134(b)(2)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.moving_pcs">PCS moving (post-TCJA: members only)</td><td>YES (members only)</td><td>§ 132(g)(2)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.death_gratuity">$100K death gratuity</td><td>YES</td><td>§ 101(a) + § 134</td></tr>
                    <tr><td data-i18n="view.s134.tbl.bonus">Reenlistment / bonus</td><td>YES (if in combat zone)</td><td>§ 112</td></tr>
                    <tr><td data-i18n="view.s134.tbl.disability">VA disability comp</td><td>YES</td><td>§ 104(a)(4)</td></tr>
                    <tr><td data-i18n="view.s134.tbl.dependent_care">Qualified dep care</td><td>YES</td><td>§ 134(b)(2)(A)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s134.h2.combat_zones">Currently designated combat zones</h2>
            <ul class="muted small">
                <li data-i18n="view.s134.cz.afghanistan">Afghanistan (since 2001 — Operation Enduring Freedom)</li>
                <li data-i18n="view.s134.cz.iraq">Iraq + adjacent waters (since 2003 — Operation Iraqi Freedom)</li>
                <li data-i18n="view.s134.cz.syria">Syria (since 2014 — Operation Inherent Resolve)</li>
                <li data-i18n="view.s134.cz.kosovo">Kosovo (designated, currently inactive)</li>
                <li data-i18n="view.s134.cz.balkan">Federal Republic of Yugoslavia + Adriatic Sea (designated)</li>
                <li data-i18n="view.s134.cz.qhda">"Qualified Hazardous Duty Areas": Bosnia + Herzegovina + Croatia (similar treatment per § 112(c)(3))</li>
                <li data-i18n="view.s134.cz.update">List updated via EO + Public Law</li>
                <li data-i18n="view.s134.cz.adjacent">Adjacent waters / airspace + countries where direct combat support operations</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s134.h2.s121_extension">§ 121(d)(9) home sale 5-yr extension</h2>
            <ul class="muted small">
                <li data-i18n="view.s134.s121.standard">§ 121 standard: lived in home as principal residence 2 of last 5 years</li>
                <li data-i18n="view.s134.s121.qualified_service">"Qualified official extended duty" - active duty 50+ miles from principal residence OR in govt quarters</li>
                <li data-i18n="view.s134.s121.suspension">Suspends 5-year period during qualified service</li>
                <li data-i18n="view.s134.s121.max_120">Maximum suspension: 120 MONTHS (10 years)</li>
                <li data-i18n="view.s134.s121.electing">Taxpayer ELECTS suspension on Form 5405 + Schedule D</li>
                <li data-i18n="view.s134.s121.intelligence">Intelligence community personnel: similar rule under § 121(d)(9)</li>
                <li data-i18n="view.s134.s121.peace_corps">Peace Corps + foreign service: similar election</li>
                <li data-i18n="view.s134.s121.s121_b_4_one_year">§ 121(b)(4) — once-per-year rule waived for armed forces during qualified service</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s134.h2.s7508">§ 7508 filing + payment extensions</h2>
            <ul class="muted small">
                <li data-i18n="view.s134.s7508.scope">Active duty + service in combat zone OR contingency operation</li>
                <li data-i18n="view.s134.s7508.deadline_extension">180 DAYS after last day in combat zone</li>
                <li data-i18n="view.s134.s7508.plus_qualifying">+ days in qualifying status</li>
                <li data-i18n="view.s134.s7508.hospitalization">+ days hospitalized due to combat zone injury</li>
                <li data-i18n="view.s134.s7508.no_interest">NO interest + penalties during extension</li>
                <li data-i18n="view.s134.s7508.applies_to">Applies to: filing, payment, collection, refund SOL</li>
                <li data-i18n="view.s134.s7508.s7508a_overlap">§ 7508A disaster zone — separate but may overlap</li>
                <li data-i18n="view.s134.s7508.scra_state">SCRA state tax + interest rate cap protections separately</li>
                <li data-i18n="view.s134.s7508.notice_2024">IRS Notice 2024-xx updates list of combat zones each year</li>
            </ul>
        </div>
    `;
    document.getElementById('s134-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_active_duty = !!fd.get('is_active_duty');
        state.is_reservist = !!fd.get('is_reservist');
        state.is_national_guard = !!fd.get('is_national_guard');
        state.service_branch = fd.get('service_branch');
        state.rank = fd.get('rank');
        state.is_combat_zone_service = !!fd.get('is_combat_zone_service');
        state.s112_combat_pay_total = Number(fd.get('s112_combat_pay_total')) || 0;
        state.enlisted_combat_pay_excluded = Number(fd.get('enlisted_combat_pay_excluded')) || 0;
        state.officer_combat_pay_capped = Number(fd.get('officer_combat_pay_capped')) || 0;
        state.officer_pay_cap_2024 = Number(fd.get('officer_pay_cap_2024')) || 0;
        state.combat_zone_start_date = fd.get('combat_zone_start_date') || '';
        state.combat_zone_end_date = fd.get('combat_zone_end_date') || '';
        state.is_hospitalized_due_to_combat = !!fd.get('is_hospitalized_due_to_combat');
        state.months_qualifying = Number(fd.get('months_qualifying')) || 0;
        state.s134_qualified_benefit = Number(fd.get('s134_qualified_benefit')) || 0;
        state.benefits_dependents_education = Number(fd.get('benefits_dependents_education')) || 0;
        state.benefits_dependents_dependent_care = Number(fd.get('benefits_dependents_dependent_care')) || 0;
        state.benefits_housing_in_kind = Number(fd.get('benefits_housing_in_kind')) || 0;
        state.benefits_housing_allowance_bah = Number(fd.get('benefits_housing_allowance_bah')) || 0;
        state.bah_amount = Number(fd.get('bah_amount')) || 0;
        state.bas_amount = Number(fd.get('bas_amount')) || 0;
        state.benefits_moving_expenses_pcs = Number(fd.get('benefits_moving_expenses_pcs')) || 0;
        state.benefits_uniform_allowance = Number(fd.get('benefits_uniform_allowance')) || 0;
        state.benefits_death_gratuity = Number(fd.get('benefits_death_gratuity')) || 0;
        state.s134_b_2_a_qualified_dep_care = !!fd.get('s134_b_2_a_qualified_dep_care');
        state.is_dod_authorized_program = !!fd.get('is_dod_authorized_program');
        state.s121_d_9_home_sale_extension = !!fd.get('s121_d_9_home_sale_extension');
        state.months_extended_for_qualified_service = Number(fd.get('months_extended_for_qualified_service')) || 0;
        state.extension_period_max_120_months = Number(fd.get('extension_period_max_120_months')) || 0;
        state.s32_combat_pay_election = !!fd.get('s32_combat_pay_election');
        state.s32_combat_pay_election_amount = Number(fd.get('s32_combat_pay_election_amount')) || 0;
        state.s24_combat_pay_inclusion = Number(fd.get('s24_combat_pay_inclusion')) || 0;
        state.s1402_h_clergy_overlap = !!fd.get('s1402_h_clergy_overlap');
        state.s7508_filing_payment_extension = !!fd.get('s7508_filing_payment_extension');
        state.s7508a_disaster_zone_overlap = !!fd.get('s7508a_disaster_zone_overlap');
        state.is_special_extended_active_duty = !!fd.get('is_special_extended_active_duty');
        state.is_armed_forces_disability = !!fd.get('is_armed_forces_disability');
        state.s104_a_4_disability_excluded = Number(fd.get('s104_a_4_disability_excluded')) || 0;
        state.s104_a_5_armed_forces_excluded = Number(fd.get('s104_a_5_armed_forces_excluded')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s134-output');
    if (!el) return;
    const is_enlisted = state.rank === 'enlisted' || state.rank === 'warrant';
    const combat_excluded = is_enlisted ? state.s112_combat_pay_total : Math.min(state.s112_combat_pay_total, state.officer_pay_cap_2024 * state.months_qualifying);
    const total_excluded = combat_excluded + state.bah_amount + state.bas_amount + state.s134_qualified_benefit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s134.h2.result">§ 134 + § 112 exclusion</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s134.card.rank">Rank</div><div class="value">${esc(state.rank)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s134.card.combat">§ 112 combat excl</div><div class="value">$${combat_excluded.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s134.card.bah">BAH excl</div><div class="value">$${state.bah_amount.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s134.card.bas">BAS excl</div><div class="value">$${state.bas_amount.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s134.card.total">Total excluded</div><div class="value">$${total_excluded.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
