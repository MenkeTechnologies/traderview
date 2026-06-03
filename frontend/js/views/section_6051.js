// IRC § 6051 — Employer W-2 Wage Statements.
// Employers furnish W-2 Wage + Tax Statement to each employee + Social Security Administration.
// Box 1: federal wages (taxable). Box 3 + 5: Social Security + Medicare wages. Boxes 12: codes for retirement, HSA, dep care, group life > $50K, etc.
// Form W-3 transmittal. Due January 31 (employee + SSA, regardless of paper/electronic).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    gross_wages: 0,
    pretax_401k: 0,
    pretax_health_insurance: 0,
    pretax_hsa: 0,
    pretax_dependent_care: 0,
    pretax_transit: 0,
    bonus_amount: 0,
    federal_withholding: 0,
    social_security_withholding: 0,
    medicare_withholding: 0,
    state_withholding: 0,
    box_12_code: 'D',
    box_12_amount: 0,
    fica_wage_base_2025: 176_100,
    additional_medicare_withholding: 0,
    is_2pct_s_corp_owner: false,
    s_corp_health_premium_box1: 0,
    third_party_sick_pay: false,
    statutory_employee: false,
};

export async function renderSection6051(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6051.h1.title">// § 6051 W-2 WAGE STATEMENTS</span></h1>
        <p class="muted small" data-i18n="view.s6051.hint.intro">
            Employers furnish <strong>W-2 Wage + Tax Statement</strong> to each employee + Social Security
            Administration. <strong>Box 1:</strong> federal wages (taxable). <strong>Box 3 + 5:</strong>
            Social Security + Medicare wages. <strong>Box 12:</strong> codes for retirement (D, E, G),
            HSA (W), dep care (DD), group life > $50K (C), etc. <strong>Form W-3</strong> transmittal.
            <strong>Due January 31</strong> (employee + SSA, regardless of paper/electronic).
            <strong>SSA matching:</strong> wages cross-check vs SSA records.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6051.h2.inputs">Inputs</h2>
            <form id="s6051-form" class="inline-form">
                <label><span data-i18n="view.s6051.label.gross">Gross wages ($)</span>
                    <input type="number" step="1000" name="gross_wages" value="${state.gross_wages}"></label>
                <label><span data-i18n="view.s6051.label.401k">Pre-tax 401(k) ($)</span>
                    <input type="number" step="100" name="pretax_401k" value="${state.pretax_401k}"></label>
                <label><span data-i18n="view.s6051.label.health">Pre-tax health insurance ($)</span>
                    <input type="number" step="100" name="pretax_health_insurance" value="${state.pretax_health_insurance}"></label>
                <label><span data-i18n="view.s6051.label.hsa">Pre-tax HSA ($)</span>
                    <input type="number" step="100" name="pretax_hsa" value="${state.pretax_hsa}"></label>
                <label><span data-i18n="view.s6051.label.dep_care">Pre-tax dep care ($)</span>
                    <input type="number" step="100" name="pretax_dependent_care" value="${state.pretax_dependent_care}"></label>
                <label><span data-i18n="view.s6051.label.transit">Pre-tax transit ($)</span>
                    <input type="number" step="100" name="pretax_transit" value="${state.pretax_transit}"></label>
                <label><span data-i18n="view.s6051.label.bonus">Bonus amount ($)</span>
                    <input type="number" step="1000" name="bonus_amount" value="${state.bonus_amount}"></label>
                <label><span data-i18n="view.s6051.label.fed">Federal withholding ($)</span>
                    <input type="number" step="100" name="federal_withholding" value="${state.federal_withholding}"></label>
                <label><span data-i18n="view.s6051.label.ss">Social Security withholding ($)</span>
                    <input type="number" step="100" name="social_security_withholding" value="${state.social_security_withholding}"></label>
                <label><span data-i18n="view.s6051.label.medicare">Medicare withholding ($)</span>
                    <input type="number" step="100" name="medicare_withholding" value="${state.medicare_withholding}"></label>
                <label><span data-i18n="view.s6051.label.state">State withholding ($)</span>
                    <input type="number" step="100" name="state_withholding" value="${state.state_withholding}"></label>
                <label><span data-i18n="view.s6051.label.box12_code">Box 12 code</span>
                    <select name="box_12_code">
                        <option value="D" ${state.box_12_code === 'D' ? 'selected' : ''}>D — 401(k) elective deferral</option>
                        <option value="E" ${state.box_12_code === 'E' ? 'selected' : ''}>E — 403(b) elective deferral</option>
                        <option value="G" ${state.box_12_code === 'G' ? 'selected' : ''}>G — 457(b) elective deferral</option>
                        <option value="W" ${state.box_12_code === 'W' ? 'selected' : ''}>W — HSA contribution (employer + employee)</option>
                        <option value="DD" ${state.box_12_code === 'DD' ? 'selected' : ''}>DD — health coverage employer-sponsored</option>
                        <option value="C" ${state.box_12_code === 'C' ? 'selected' : ''}>C — Group term life > $50K (imputed)</option>
                        <option value="AA" ${state.box_12_code === 'AA' ? 'selected' : ''}>AA — Roth 401(k)</option>
                        <option value="BB" ${state.box_12_code === 'BB' ? 'selected' : ''}>BB — Roth 403(b)</option>
                        <option value="EE" ${state.box_12_code === 'EE' ? 'selected' : ''}>EE — Roth 457(b)</option>
                        <option value="P" ${state.box_12_code === 'P' ? 'selected' : ''}>P — Excluded moving expenses</option>
                        <option value="Y" ${state.box_12_code === 'Y' ? 'selected' : ''}>Y — § 409A deferral</option>
                        <option value="Z" ${state.box_12_code === 'Z' ? 'selected' : ''}>Z — § 409A failure</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6051.label.box12_amount">Box 12 amount ($)</span>
                    <input type="number" step="100" name="box_12_amount" value="${state.box_12_amount}"></label>
                <label><span data-i18n="view.s6051.label.fica_base">FICA wage base 2025 ($)</span>
                    <input type="number" step="100" name="fica_wage_base_2025" value="${state.fica_wage_base_2025}"></label>
                <label><span data-i18n="view.s6051.label.add_medicare">Additional Medicare 0.9% withholding ($)</span>
                    <input type="number" step="10" name="additional_medicare_withholding" value="${state.additional_medicare_withholding}"></label>
                <label><span data-i18n="view.s6051.label.s_corp">2%+ S-corp owner?</span>
                    <input type="checkbox" name="is_2pct_s_corp_owner" ${state.is_2pct_s_corp_owner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6051.label.s_corp_premium">S-corp health premium added to Box 1 ($)</span>
                    <input type="number" step="100" name="s_corp_health_premium_box1" value="${state.s_corp_health_premium_box1}"></label>
                <label><span data-i18n="view.s6051.label.sick_pay">Third-party sick pay?</span>
                    <input type="checkbox" name="third_party_sick_pay" ${state.third_party_sick_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6051.label.statutory">Statutory employee?</span>
                    <input type="checkbox" name="statutory_employee" ${state.statutory_employee ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6051.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6051-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6051.h2.box_definitions">W-2 Box definitions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6051.box.1">Box 1: Wages, tips, other compensation (subject to federal income tax)</li>
                <li data-i18n="view.s6051.box.2">Box 2: Federal income tax withheld</li>
                <li data-i18n="view.s6051.box.3">Box 3: Social Security wages (up to wage base $176,100 for 2025)</li>
                <li data-i18n="view.s6051.box.4">Box 4: Social Security tax withheld (6.2%)</li>
                <li data-i18n="view.s6051.box.5">Box 5: Medicare wages (no cap)</li>
                <li data-i18n="view.s6051.box.6">Box 6: Medicare tax withheld (1.45% + 0.9% additional)</li>
                <li data-i18n="view.s6051.box.7">Box 7: Social Security tips</li>
                <li data-i18n="view.s6051.box.8">Box 8: Allocated tips</li>
                <li data-i18n="view.s6051.box.10">Box 10: Dependent care benefits</li>
                <li data-i18n="view.s6051.box.11">Box 11: Nonqualified deferred comp § 457(f) / § 409A failed</li>
                <li data-i18n="view.s6051.box.12">Box 12: Various codes (see list — most common)</li>
                <li data-i18n="view.s6051.box.13">Box 13: Statutory employee / retirement plan / third-party sick pay</li>
                <li data-i18n="view.s6051.box.14">Box 14: Other (state disability, etc.)</li>
                <li data-i18n="view.s6051.box.15_20">Boxes 15-20: State + local wages + tax</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6051.h2.pretax_reductions">Pre-tax reductions to Box 1</h2>
            <ul class="muted small">
                <li data-i18n="view.s6051.pre.401k">401(k) / 403(b) elective deferrals: reduce Box 1 but NOT Box 3/5 (FICA still applies)</li>
                <li data-i18n="view.s6051.pre.health">§ 125 health insurance premiums: reduce Box 1 + Box 3 + Box 5 (full pre-tax)</li>
                <li data-i18n="view.s6051.pre.hsa">HSA contributions: reduce Box 1 + Box 3 + Box 5 (full pre-tax)</li>
                <li data-i18n="view.s6051.pre.dep_care">Dependent care FSA: reduces Box 1 + Box 3 + Box 5 ($5K limit 2025)</li>
                <li data-i18n="view.s6051.pre.transit">Transit / parking: reduces Box 1 + Box 3 + Box 5 ($325/mo each 2025)</li>
                <li data-i18n="view.s6051.pre.roth">Roth 401(k): does NOT reduce Box 1 (post-tax)</li>
                <li data-i18n="view.s6051.pre.health_pre_tax_vs_post">After-tax health insurance: does NOT reduce — paid from net pay</li>
                <li data-i18n="view.s6051.pre.imputed_income">Imputed income (group life > $50K): ADDS to Box 1 + Box 3 + Box 5</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6051.h2.s_corp_owners">2%+ S-corp owner W-2 quirks</h2>
            <ul class="muted small">
                <li data-i18n="view.s6051.scorp.health_taxable">Health insurance premiums INCLUDED in Box 1 (TAXABLE wages)</li>
                <li data-i18n="view.s6051.scorp.no_fica">Health premiums NOT in Box 3/5 (NO FICA on this portion)</li>
                <li data-i18n="view.s6051.scorp.deduction">Owner takes § 162(l) above-the-line deduction on personal return</li>
                <li data-i18n="view.s6051.scorp.box_14">Optionally disclosed in Box 14 with code 'S CORP HEALTH'</li>
                <li data-i18n="view.s6051.scorp.k1_split">Net effect: tax-free for FICA but income tax-free via § 162(l)</li>
                <li data-i18n="view.s6051.scorp.hsa">HSA: similar rule — included in Box 1, but excluded from Box 3/5 if HDHP</li>
                <li data-i18n="view.s6051.scorp.disability">Disability premiums: same rule (Box 1 included)</li>
                <li data-i18n="view.s6051.scorp.life_insurance">Group term life: similar inclusion rules</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6051.h2.deadlines_penalties">Deadlines + penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s6051.dp.deadline">Due to EMPLOYEE: January 31</li>
                <li data-i18n="view.s6051.dp.ssa">Due to SSA: January 31 (paper + electronic) since 2017 PATH Act</li>
                <li data-i18n="view.s6051.dp.efile_threshold">E-file mandatory: 10+ returns (post-2024 regs)</li>
                <li data-i18n="view.s6051.dp.w3">Form W-3 transmittal: submit with paper W-2; not required for SSA electronic</li>
                <li data-i18n="view.s6051.dp.s6721">§ 6721 penalty: same tiers as 1099 ($60 / $120 / $330 / $660+)</li>
                <li data-i18n="view.s6051.dp.s6722">§ 6722 penalty: failure to furnish to employee — same tiers</li>
                <li data-i18n="view.s6051.dp.corrected_w2c">Corrections: Form W-2c (corrected) + W-3c (transmittal)</li>
                <li data-i18n="view.s6051.dp.extension">Extension: Form 8809 (30-day automatic; only available if hardship)</li>
            </ul>
        </div>
    `;
    document.getElementById('s6051-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_wages = Number(fd.get('gross_wages')) || 0;
        state.pretax_401k = Number(fd.get('pretax_401k')) || 0;
        state.pretax_health_insurance = Number(fd.get('pretax_health_insurance')) || 0;
        state.pretax_hsa = Number(fd.get('pretax_hsa')) || 0;
        state.pretax_dependent_care = Number(fd.get('pretax_dependent_care')) || 0;
        state.pretax_transit = Number(fd.get('pretax_transit')) || 0;
        state.bonus_amount = Number(fd.get('bonus_amount')) || 0;
        state.federal_withholding = Number(fd.get('federal_withholding')) || 0;
        state.social_security_withholding = Number(fd.get('social_security_withholding')) || 0;
        state.medicare_withholding = Number(fd.get('medicare_withholding')) || 0;
        state.state_withholding = Number(fd.get('state_withholding')) || 0;
        state.box_12_code = fd.get('box_12_code');
        state.box_12_amount = Number(fd.get('box_12_amount')) || 0;
        state.fica_wage_base_2025 = Number(fd.get('fica_wage_base_2025')) || 0;
        state.additional_medicare_withholding = Number(fd.get('additional_medicare_withholding')) || 0;
        state.is_2pct_s_corp_owner = !!fd.get('is_2pct_s_corp_owner');
        state.s_corp_health_premium_box1 = Number(fd.get('s_corp_health_premium_box1')) || 0;
        state.third_party_sick_pay = !!fd.get('third_party_sick_pay');
        state.statutory_employee = !!fd.get('statutory_employee');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6051-output');
    if (!el) return;
    const box1 = state.gross_wages + state.bonus_amount + state.s_corp_health_premium_box1
                - state.pretax_401k - state.pretax_health_insurance - state.pretax_hsa - state.pretax_dependent_care - state.pretax_transit;
    const box3_pre_cap = state.gross_wages + state.bonus_amount
                - state.pretax_health_insurance - state.pretax_hsa - state.pretax_dependent_care - state.pretax_transit;
    const box3 = Math.min(box3_pre_cap, state.fica_wage_base_2025);
    const box5 = box3_pre_cap;
    const ss_tax = box3 * 0.062;
    const medicare_tax = box5 * 0.0145;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6051.h2.result">W-2 box computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6051.card.box1">Box 1 (federal wages)</div>
                    <div class="value">$${box1.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6051.card.box3">Box 3 (SS wages, capped)</div>
                    <div class="value">$${box3.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6051.card.box5">Box 5 (Medicare wages)</div>
                    <div class="value">$${box5.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6051.card.ss_tax">SS tax (Box 4)</div>
                    <div class="value">$${ss_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6051.card.medicare_tax">Medicare tax (Box 6)</div>
                    <div class="value">$${medicare_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6051.card.fed_wh">Federal withholding (Box 2)</div>
                    <div class="value">$${state.federal_withholding.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6051.card.state_wh">State withholding</div>
                    <div class="value">$${state.state_withholding.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_2pct_s_corp_owner ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s6051.scorp_note">
                    2%+ S-corp owner: health insurance premiums INCLUDED in Box 1 wages. NO FICA on health
                    portion (Box 3/5 reduced). Owner takes § 162(l) above-the-line deduction on Form 1040.
                    Net result: full income deductibility WITHOUT FICA. Box 14 disclosure recommended:
                    "S CORP HEALTH $X" for clarity.
                </p>
            ` : ''}
        </div>
    `;
}
