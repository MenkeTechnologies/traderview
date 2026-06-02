// IRC § 32 — Earned Income Tax Credit (EITC).
// Refundable credit for low-income working taxpayers.
// 2024 max credit: $7,830 (3+ kids) / $6,960 (2 kids) / $4,213 (1 kid) / $632 (no kids).
// Computed on Schedule EIC + complex earned income + AGI phase-in/phase-out.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'mfj',
    qualifying_children_count: 0,
    earned_income: 0,
    agi: 0,
    investment_income: 0,
    tax_year: 2024,
    is_eligible_age: false,
    age_no_children: 25,
    has_valid_ssn_self: true,
    has_valid_ssn_spouse: true,
    has_valid_ssn_children: 0,
    is_us_citizen_resident_full_year: true,
    is_not_qualifying_child_other: true,
    is_not_separated_dependent: true,
    nonresident_alien_filing_jointly: false,
    foreign_earned_income_excluded: false,
    is_clergy: false,
    is_military_combat_pay: false,
    combat_pay_election: false,
    s32_d_disqualified_income: 0,
    s32_d_disqualified_threshold: 11600,
    self_employed_se_tax_paid: 0,
    is_separated: false,
    s32_c_3_a_full_year_us: true,
    s32_m_taxpayer_id: false,
    received_advance_eitc: 0,
    expected_credit: 0,
    actc_overlap: 0,
    s24_d_actc_overlap: false,
    is_eitc_eligible: false,
    phase_in_pct: 0,
    phase_out_pct: 0,
};

export async function renderSection32(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s32.h1.title">// § 32 EARNED INCOME TAX CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s32.hint.intro">
            <strong>2024 EITC max:</strong> $7,830 (3+ kids) / $6,960 (2 kids) / $4,213 (1 kid) /
            $632 (no kids). <strong>Refundable</strong> — fully refundable component. <strong>Phase-in:</strong>
            7.65% (no kids) / 34% (1) / 40% (2 or 3+) of earned income. <strong>Phase-out:</strong>
            7.65% (no) / 15.98% (1) / 21.06% (2 or 3+). <strong>2024 phase-out start (MFJ):</strong>
            $17,250 (no kids) / $29,640 (with kids). <strong>2024 max AGI MFJ (3+ kids):</strong>
            $66,819. <strong>§ 32(d) disqualified income test:</strong> $11,600 (2024) — interest +
            dividends + cap gains + passive rental. <strong>Requirements:</strong> valid SSN, US
            citizen/resident, NOT qualifying child of another, NOT separated dependent, age 25-64 if
            no kids, earned income, file Schedule EIC. <strong>IRS Letter 1058</strong> if EITC denied.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s32.h2.inputs">Inputs</h2>
            <form id="s32-form" class="inline-form">
                <label><span data-i18n="view.s32.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HOH</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS (limited)</option>
                        <option value="qw" ${state.filing_status === 'qw' ? 'selected' : ''}>QW</option>
                    </select>
                </label>
                <label><span data-i18n="view.s32.label.kids">Qualifying children count</span>
                    <input type="number" step="1" name="qualifying_children_count" value="${state.qualifying_children_count}"></label>
                <label><span data-i18n="view.s32.label.earned">Earned income ($)</span>
                    <input type="number" step="100" name="earned_income" value="${state.earned_income}"></label>
                <label><span data-i18n="view.s32.label.agi">AGI ($)</span>
                    <input type="number" step="100" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.s32.label.invest">Investment income ($)</span>
                    <input type="number" step="100" name="investment_income" value="${state.investment_income}"></label>
                <label><span data-i18n="view.s32.label.year">Year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s32.label.eligible_age">Eligible age?</span>
                    <input type="checkbox" name="is_eligible_age" ${state.is_eligible_age ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.age_no_kids">Age (no kids)</span>
                    <input type="number" step="1" name="age_no_children" value="${state.age_no_children}"></label>
                <label><span data-i18n="view.s32.label.ssn">Self SSN?</span>
                    <input type="checkbox" name="has_valid_ssn_self" ${state.has_valid_ssn_self ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.spouse_ssn">Spouse SSN?</span>
                    <input type="checkbox" name="has_valid_ssn_spouse" ${state.has_valid_ssn_spouse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.kid_ssn">Children w/ valid SSN</span>
                    <input type="number" step="1" name="has_valid_ssn_children" value="${state.has_valid_ssn_children}"></label>
                <label><span data-i18n="view.s32.label.us_citizen">Full-yr US citizen / resident?</span>
                    <input type="checkbox" name="is_us_citizen_resident_full_year" ${state.is_us_citizen_resident_full_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.not_qc">Not qualifying child of other?</span>
                    <input type="checkbox" name="is_not_qualifying_child_other" ${state.is_not_qualifying_child_other ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.not_sep">Not separated dependent?</span>
                    <input type="checkbox" name="is_not_separated_dependent" ${state.is_not_separated_dependent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.nra_mfj">NRA filing jointly?</span>
                    <input type="checkbox" name="nonresident_alien_filing_jointly" ${state.nonresident_alien_filing_jointly ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.feie">FEIE claimed?</span>
                    <input type="checkbox" name="foreign_earned_income_excluded" ${state.foreign_earned_income_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.clergy">Clergy?</span>
                    <input type="checkbox" name="is_clergy" ${state.is_clergy ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.combat">Military combat pay?</span>
                    <input type="checkbox" name="is_military_combat_pay" ${state.is_military_combat_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.combat_elect">Combat pay election?</span>
                    <input type="checkbox" name="combat_pay_election" ${state.combat_pay_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.s32_d">§ 32(d) disqualified inc ($)</span>
                    <input type="number" step="100" name="s32_d_disqualified_income" value="${state.s32_d_disqualified_income}"></label>
                <label><span data-i18n="view.s32.label.threshold">§ 32(d) threshold ($)</span>
                    <input type="number" step="100" name="s32_d_disqualified_threshold" value="${state.s32_d_disqualified_threshold}"></label>
                <label><span data-i18n="view.s32.label.se">SE tax paid ($)</span>
                    <input type="number" step="100" name="self_employed_se_tax_paid" value="${state.self_employed_se_tax_paid}"></label>
                <label><span data-i18n="view.s32.label.separated">Separated?</span>
                    <input type="checkbox" name="is_separated" ${state.is_separated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.s32_c3a">§ 32(c)(3)(A) full-yr US?</span>
                    <input type="checkbox" name="s32_c_3_a_full_year_us" ${state.s32_c_3_a_full_year_us ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.s32_m">§ 32(m) TIN by due date?</span>
                    <input type="checkbox" name="s32_m_taxpayer_id" ${state.s32_m_taxpayer_id ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.advance">Advance EITC received ($)</span>
                    <input type="number" step="100" name="received_advance_eitc" value="${state.received_advance_eitc}"></label>
                <label><span data-i18n="view.s32.label.expected">Expected credit ($)</span>
                    <input type="number" step="100" name="expected_credit" value="${state.expected_credit}"></label>
                <label><span data-i18n="view.s32.label.actc">ACTC overlap ($)</span>
                    <input type="number" step="100" name="actc_overlap" value="${state.actc_overlap}"></label>
                <label><span data-i18n="view.s32.label.s24d">§ 24(d) ACTC overlap?</span>
                    <input type="checkbox" name="s24_d_actc_overlap" ${state.s24_d_actc_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.eligible">EITC eligible?</span>
                    <input type="checkbox" name="is_eitc_eligible" ${state.is_eitc_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s32.label.phase_in">Phase-in %</span>
                    <input type="number" step="0.01" name="phase_in_pct" value="${state.phase_in_pct}"></label>
                <label><span data-i18n="view.s32.label.phase_out">Phase-out %</span>
                    <input type="number" step="0.01" name="phase_out_pct" value="${state.phase_out_pct}"></label>
                <button class="primary" type="submit" data-i18n="view.s32.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s32-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s32.h2.amounts">2024 EITC amounts</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s32.tbl.kids">Children</th><th data-i18n="view.s32.tbl.max">Max credit</th><th data-i18n="view.s32.tbl.phase_in_pct">Phase-in %</th><th data-i18n="view.s32.tbl.phase_out_pct">Phase-out %</th><th data-i18n="view.s32.tbl.phase_out_start">Phase-out start (MFJ/other)</th><th data-i18n="view.s32.tbl.complete">Complete phase-out (MFJ/other)</th></tr></thead>
                <tbody>
                    <tr><td>None</td><td>$632</td><td>7.65%</td><td>7.65%</td><td>$17,250 / $10,330</td><td>$25,511 / $18,591</td></tr>
                    <tr><td>1</td><td>$4,213</td><td>34%</td><td>15.98%</td><td>$29,640 / $22,720</td><td>$56,004 / $49,084</td></tr>
                    <tr><td>2</td><td>$6,960</td><td>40%</td><td>21.06%</td><td>$29,640 / $22,720</td><td>$62,688 / $55,768</td></tr>
                    <tr><td>3+</td><td>$7,830</td><td>45%</td><td>21.06%</td><td>$29,640 / $22,720</td><td>$66,819 / $59,899</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s32.h2.requirements">Requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s32.req.earned_income">Must have EARNED income (wages, SE, statutory employee, etc.)</li>
                <li data-i18n="view.s32.req.agi_limit">AGI below threshold (varies by # kids + filing status)</li>
                <li data-i18n="view.s32.req.ssn">Valid SSN for self + spouse + each qualifying child</li>
                <li data-i18n="view.s32.req.us_citizen">US citizen / resident alien full year</li>
                <li data-i18n="view.s32.req.not_qc">Cannot be qualifying child of another</li>
                <li data-i18n="view.s32.req.age_no_kids">If no kids: age 25-64</li>
                <li data-i18n="view.s32.req.no_feie">Cannot exclude foreign earned income (§ 911)</li>
                <li data-i18n="view.s32.req.not_dependent">Cannot be claimed as dependent of another</li>
                <li data-i18n="view.s32.req.mfs">MFS: limited eligibility per § 32(d)(2) — must meet separation tests</li>
                <li data-i18n="view.s32.req.s32d">§ 32(d) — investment income ≤ $11,600 (2024 indexed)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s32.h2.disqualified">§ 32(d) disqualified income test</h2>
            <ul class="muted small">
                <li data-i18n="view.s32.disq.threshold">$11,600 limit (2024) — indexed annually</li>
                <li data-i18n="view.s32.disq.interest">Taxable + tax-exempt interest</li>
                <li data-i18n="view.s32.disq.dividends">Dividends</li>
                <li data-i18n="view.s32.disq.cap_gain">Net capital gain (including § 1411 passive)</li>
                <li data-i18n="view.s32.disq.rental">Net passive rental income</li>
                <li data-i18n="view.s32.disq.royalty">Net royalty income</li>
                <li data-i18n="view.s32.disq.cliff">CLIFF — exceed by $1, lose entire credit</li>
                <li data-i18n="view.s32.disq.gross">Gross investment income — losses do NOT offset (per § 32(i)(1))</li>
                <li data-i18n="view.s32.disq.s32_i_2">§ 32(i)(2) — other investment income at IRS discretion</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s32.h2.qualifying_child">Qualifying child test (§ 152(c))</h2>
            <ul class="muted small">
                <li data-i18n="view.s32.qc.relationship">Son, daughter, stepchild, foster child, sibling, descendant</li>
                <li data-i18n="view.s32.qc.age">Under 19 OR under 24 + full-time student OR any age if permanently disabled</li>
                <li data-i18n="view.s32.qc.residency">Lived with taxpayer &gt; 6 months in US</li>
                <li data-i18n="view.s32.qc.joint_return">Cannot file joint return (except for refund only)</li>
                <li data-i18n="view.s32.qc.tiebreaker">§ 32(c)(1)(C) tie-breaker for shared custody</li>
                <li data-i18n="view.s32.qc.ssn_required">Each child MUST have valid SSN (NOT ITIN) for full EITC w/ kids</li>
                <li data-i18n="view.s32.qc.s32_c_1_e">§ 32(c)(1)(E) qualifying child test similar to § 152 but for EITC</li>
                <li data-i18n="view.s32.qc.uniform">Uniform definition of "qualifying child" across § 24, § 32, § 152</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s32.h2.disallowance">EITC ban + disallowance</h2>
            <ul class="muted small">
                <li data-i18n="view.s32.ban.2yr">§ 32(k) 2-year ban for reckless disregard / improper claim</li>
                <li data-i18n="view.s32.ban.10yr">10-year ban for fraud (IRS determination)</li>
                <li data-i18n="view.s32.ban.recertification">Form 8862 required to recertify after disallowance</li>
                <li data-i18n="view.s32.ban.paid_preparer">Paid preparer § 6695(g) due diligence — Form 8867 ($635 per violation 2024)</li>
                <li data-i18n="view.s32.ban.s7430">Wins by taxpayer against IRS: may collect attorney fees § 7430</li>
                <li data-i18n="view.s32.ban.audit_rate">EITC audit rate ~1% — highest of all taxpayer groups (despite low compliance)</li>
                <li data-i18n="view.s32.ban.s6213_b">§ 6213(b) summary assessment authority for EITC disallowance</li>
                <li data-i18n="view.s32.ban.improper_payment">Improper payment rate ~25% — among IRS Top 10 highest</li>
            </ul>
        </div>
    `;
    document.getElementById('s32-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.qualifying_children_count = Number(fd.get('qualifying_children_count')) || 0;
        state.earned_income = Number(fd.get('earned_income')) || 0;
        state.agi = Number(fd.get('agi')) || 0;
        state.investment_income = Number(fd.get('investment_income')) || 0;
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.is_eligible_age = !!fd.get('is_eligible_age');
        state.age_no_children = Number(fd.get('age_no_children')) || 0;
        state.has_valid_ssn_self = !!fd.get('has_valid_ssn_self');
        state.has_valid_ssn_spouse = !!fd.get('has_valid_ssn_spouse');
        state.has_valid_ssn_children = Number(fd.get('has_valid_ssn_children')) || 0;
        state.is_us_citizen_resident_full_year = !!fd.get('is_us_citizen_resident_full_year');
        state.is_not_qualifying_child_other = !!fd.get('is_not_qualifying_child_other');
        state.is_not_separated_dependent = !!fd.get('is_not_separated_dependent');
        state.nonresident_alien_filing_jointly = !!fd.get('nonresident_alien_filing_jointly');
        state.foreign_earned_income_excluded = !!fd.get('foreign_earned_income_excluded');
        state.is_clergy = !!fd.get('is_clergy');
        state.is_military_combat_pay = !!fd.get('is_military_combat_pay');
        state.combat_pay_election = !!fd.get('combat_pay_election');
        state.s32_d_disqualified_income = Number(fd.get('s32_d_disqualified_income')) || 0;
        state.s32_d_disqualified_threshold = Number(fd.get('s32_d_disqualified_threshold')) || 0;
        state.self_employed_se_tax_paid = Number(fd.get('self_employed_se_tax_paid')) || 0;
        state.is_separated = !!fd.get('is_separated');
        state.s32_c_3_a_full_year_us = !!fd.get('s32_c_3_a_full_year_us');
        state.s32_m_taxpayer_id = !!fd.get('s32_m_taxpayer_id');
        state.received_advance_eitc = Number(fd.get('received_advance_eitc')) || 0;
        state.expected_credit = Number(fd.get('expected_credit')) || 0;
        state.actc_overlap = Number(fd.get('actc_overlap')) || 0;
        state.s24_d_actc_overlap = !!fd.get('s24_d_actc_overlap');
        state.is_eitc_eligible = !!fd.get('is_eitc_eligible');
        state.phase_in_pct = Number(fd.get('phase_in_pct')) || 0;
        state.phase_out_pct = Number(fd.get('phase_out_pct')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s32-output');
    if (!el) return;
    const kids = state.qualifying_children_count;
    const max_credit = kids === 0 ? 632 : (kids === 1 ? 4213 : (kids === 2 ? 6960 : 7830));
    const phase_in_pct = kids === 0 ? 0.0765 : (kids === 1 ? 0.34 : (kids === 2 ? 0.40 : 0.45));
    const phase_out_pct = kids === 0 ? 0.0765 : (kids === 1 ? 0.1598 : 0.2106);
    const phase_out_start = state.filing_status === 'mfj' ? (kids === 0 ? 17_250 : 29_640) : (kids === 0 ? 10_330 : 22_720);
    const credit_phase_in = Math.min(state.earned_income * phase_in_pct, max_credit);
    const max_AGI = state.agi > phase_out_start ? credit_phase_in - (state.agi - phase_out_start) * phase_out_pct : credit_phase_in;
    const disq = state.investment_income > state.s32_d_disqualified_threshold;
    const final_credit = disq ? 0 : Math.max(0, max_AGI);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s32.h2.result">§ 32 EITC calculation</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s32.card.max">Max credit</div><div class="value">$${max_credit.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s32.card.phase_in">Phase-in</div><div class="value">$${credit_phase_in.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${state.agi > phase_out_start ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s32.card.after_AGI">After AGI phase-out</div><div class="value">$${Math.max(0, max_AGI).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${disq ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s32.card.s32d">§ 32(d) disqualified?</div><div class="value">${disq ? 'YES (zero credit)' : 'NO'}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s32.card.final">Final EITC</div><div class="value">$${final_credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
            </div>
        </div>
    `;
}
