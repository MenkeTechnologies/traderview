// IRC § 24 — Child Tax Credit (CTC) + Other Dependent Credit (ODC).
// 2024: $2,000 per qualifying child under 17 / $1,700 refundable as ACTC.
// $500 ODC for other dependents (qualifying relatives, dependents 17+).
// MAGI phase-out: $200K single / $400K MFJ — $50 reduction per $1,000 over.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'mfj',
    qualifying_children_count: 0,
    other_dependents_count: 0,
    magi: 0,
    earned_income: 0,
    tax_year: 2024,
    refundable_actc_taken: 0,
    children_ages: '',
    children_with_ssn: 0,
    children_without_ssn: 0,
    is_arpa_year: false,
    arpa_age_under_6: 0,
    arpa_age_6_17: 0,
    arpa_advance_payment: 0,
    is_excessive_actc_clawback: false,
    s24_h_ssn_required: true,
    s24_d_15pct_calculation: 0,
    earned_income_2500_threshold: 0,
    s24_d_max_refundable: 1700,
    s24_d_full_refundable: false,
    s24_phaseout_complete: false,
    excess_advance_2021: 0,
    safe_harbor_2021: false,
    repayment_protection_income: 0,
};

export async function renderSection24(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s24.h1.title">// § 24 CHILD TAX CREDIT</span></h1>
        <p class="muted small" data-i18n="view.s24.hint.intro">
            <strong>2024 CTC: $2,000</strong> per qualifying child under 17. <strong>$1,700 refundable</strong>
            via § 24(d) Additional Child Tax Credit (ACTC). <strong>$500 ODC</strong> for other dependents
            (qualifying relatives, dependents 17+ such as college students). <strong>MAGI phase-out:</strong>
            $200K single / $400K MFJ — $50 reduction per $1,000 above. <strong>Requirements:</strong>
            valid SSN (§ 24(h)(7)) for child, US citizen/national/resident alien, &lt; 17 at year end,
            lived with taxpayer &gt; 6 months, did NOT provide more than ½ own support.
            <strong>§ 24(d) ACTC formula:</strong> 15% × (earned income - $2,500). <strong>ARPA 2021
            expansion expired:</strong> was $3,600 under 6 / $3,000 ages 6-17 + fully refundable.
            <strong>TCJA sunset Dec 31, 2025</strong> — reverts to pre-TCJA $1,000 + $3K SSN requirement
            absent extension.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s24.h2.inputs">Inputs</h2>
            <form id="s24-form" class="inline-form">
                <label><span data-i18n="view.s24.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HOH</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s24.label.qc">Qualifying children (&lt; 17)</span>
                    <input type="number" step="1" name="qualifying_children_count" value="${state.qualifying_children_count}"></label>
                <label><span data-i18n="view.s24.label.od">Other dependents (ODC)</span>
                    <input type="number" step="1" name="other_dependents_count" value="${state.other_dependents_count}"></label>
                <label><span data-i18n="view.s24.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s24.label.earned">Earned income ($)</span>
                    <input type="number" step="1000" name="earned_income" value="${state.earned_income}"></label>
                <label><span data-i18n="view.s24.label.year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s24.label.actc_taken">ACTC refundable taken ($)</span>
                    <input type="number" step="100" name="refundable_actc_taken" value="${state.refundable_actc_taken}"></label>
                <label><span data-i18n="view.s24.label.ages">Children ages (csv)</span>
                    <input type="text" name="children_ages" value="${esc(state.children_ages)}"></label>
                <label><span data-i18n="view.s24.label.ssn">Children w/ SSN</span>
                    <input type="number" step="1" name="children_with_ssn" value="${state.children_with_ssn}"></label>
                <label><span data-i18n="view.s24.label.no_ssn">Children w/o SSN</span>
                    <input type="number" step="1" name="children_without_ssn" value="${state.children_without_ssn}"></label>
                <label><span data-i18n="view.s24.label.arpa">ARPA year (2021)?</span>
                    <input type="checkbox" name="is_arpa_year" ${state.is_arpa_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s24.label.arpa_u6">ARPA age &lt; 6 count</span>
                    <input type="number" step="1" name="arpa_age_under_6" value="${state.arpa_age_under_6}"></label>
                <label><span data-i18n="view.s24.label.arpa_6_17">ARPA age 6-17 count</span>
                    <input type="number" step="1" name="arpa_age_6_17" value="${state.arpa_age_6_17}"></label>
                <label><span data-i18n="view.s24.label.arpa_adv">ARPA advance payment ($)</span>
                    <input type="number" step="100" name="arpa_advance_payment" value="${state.arpa_advance_payment}"></label>
                <label><span data-i18n="view.s24.label.clawback">Excessive ACTC clawback?</span>
                    <input type="checkbox" name="is_excessive_actc_clawback" ${state.is_excessive_actc_clawback ? 'checked' : ''}></label>
                <label><span data-i18n="view.s24.label.s24h">SSN required?</span>
                    <input type="checkbox" name="s24_h_ssn_required" ${state.s24_h_ssn_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s24.label.s24d15">§ 24(d) 15% calc ($)</span>
                    <input type="number" step="100" name="s24_d_15pct_calculation" value="${state.s24_d_15pct_calculation}"></label>
                <label><span data-i18n="view.s24.label.threshold">$2,500 threshold ($)</span>
                    <input type="number" step="100" name="earned_income_2500_threshold" value="${state.earned_income_2500_threshold}"></label>
                <label><span data-i18n="view.s24.label.max_refund">Max refundable ($)</span>
                    <input type="number" step="100" name="s24_d_max_refundable" value="${state.s24_d_max_refundable}"></label>
                <label><span data-i18n="view.s24.label.full_refund">Fully refundable?</span>
                    <input type="checkbox" name="s24_d_full_refundable" ${state.s24_d_full_refundable ? 'checked' : ''}></label>
                <label><span data-i18n="view.s24.label.complete">Phase-out complete?</span>
                    <input type="checkbox" name="s24_phaseout_complete" ${state.s24_phaseout_complete ? 'checked' : ''}></label>
                <label><span data-i18n="view.s24.label.excess">Excess advance 2021 ($)</span>
                    <input type="number" step="100" name="excess_advance_2021" value="${state.excess_advance_2021}"></label>
                <label><span data-i18n="view.s24.label.safe_harbor">Safe harbor 2021?</span>
                    <input type="checkbox" name="safe_harbor_2021" ${state.safe_harbor_2021 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s24.label.repay_protect">Repayment protection income ($)</span>
                    <input type="number" step="1000" name="repayment_protection_income" value="${state.repayment_protection_income}"></label>
                <button class="primary" type="submit" data-i18n="view.s24.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s24-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s24.h2.requirements">Qualifying child (§ 152(c) + § 24)</h2>
            <ol class="muted small">
                <li data-i18n="view.s24.req.age">Under age 17 at year end</li>
                <li data-i18n="view.s24.req.relationship">Son, daughter, stepchild, foster child, sibling, half-sibling, stepsibling, descendant</li>
                <li data-i18n="view.s24.req.residency">Lived with taxpayer &gt; 6 months (calendar year)</li>
                <li data-i18n="view.s24.req.support">Did NOT provide more than ½ of own support</li>
                <li data-i18n="view.s24.req.citizenship">US citizen / US national / US resident alien</li>
                <li data-i18n="view.s24.req.dependent">Claimed as dependent on taxpayer's return</li>
                <li data-i18n="view.s24.req.joint">Did NOT file joint return (except for refund only)</li>
                <li data-i18n="view.s24.req.ssn">§ 24(h)(7) valid SSN by due date of return — post-PATH Act</li>
                <li data-i18n="view.s24.req.itin">ITIN children: get ODC $500 only, NOT $2,000 CTC</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s24.h2.historic">Historical evolution</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s24.tbl.year">Year</th><th data-i18n="view.s24.tbl.amount">Amount</th><th data-i18n="view.s24.tbl.refundable">Refundable</th><th data-i18n="view.s24.tbl.notes">Notes</th></tr></thead>
                <tbody>
                    <tr><td>1997</td><td>$400</td><td>NO</td><td data-i18n="view.s24.tbl.original">Original (TPRA 1997)</td></tr>
                    <tr><td>2001-2017</td><td>$1,000</td><td>$1,000</td><td data-i18n="view.s24.tbl.egtrra">EGTRRA</td></tr>
                    <tr><td>2018-2025</td><td>$2,000</td><td>$1,700 (2024)</td><td data-i18n="view.s24.tbl.tcja">TCJA + SSN required</td></tr>
                    <tr><td>2021 (ARPA)</td><td>$3,600 (under 6) / $3,000 (6-17)</td><td data-i18n="view.s24.tbl.full">Fully refundable</td><td data-i18n="view.s24.tbl.arpa">ARPA — expired 2021</td></tr>
                    <tr><td>Post-2025</td><td>$1,000</td><td>$1,000</td><td data-i18n="view.s24.tbl.sunset">Reverts to pre-TCJA absent extension</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s24.h2.phaseout">Phase-out + refundability</h2>
            <ul class="muted small">
                <li data-i18n="view.s24.po.threshold">$200K single / $400K MFJ — $50 reduction per $1,000 above</li>
                <li data-i18n="view.s24.po.complete_calc">CTC fully phased out at: $240K single (1-child) / $440K MFJ (1-child) — depends on # children</li>
                <li data-i18n="view.s24.po.actc_refundable">ACTC § 24(d): 15% × max(0, earned income - $2,500)</li>
                <li data-i18n="view.s24.po.actc_cap">2024 ACTC max: $1,700 per child</li>
                <li data-i18n="view.s24.po.s24_h_2">§ 24(h)(2) earned income threshold $2,500 (since TCJA, was $3,000)</li>
                <li data-i18n="view.s24.po.s32_eitc_no_double">No double-dipping with § 32 EITC for same earned income</li>
                <li data-i18n="view.s24.po.s24_g_3child_more">§ 24(g) alternative formula for 3+ children — incl. uncollected SS/Medicare tax</li>
                <li data-i18n="view.s24.po.s8812">Schedule 8812 reports + computes CTC + ODC + ACTC</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s24.h2.odc">Other Dependent Credit (§ 24(h)(4))</h2>
            <ul class="muted small">
                <li data-i18n="view.s24.odc.amount">$500 nonrefundable credit per other dependent</li>
                <li data-i18n="view.s24.odc.eligible">Qualifying relative (§ 152(d)) — child age 17+, parents, other relatives</li>
                <li data-i18n="view.s24.odc.college">College students up to age 24 (full-time + 5+ months)</li>
                <li data-i18n="view.s24.odc.parents">Parents living separately if claimed as dependent</li>
                <li data-i18n="view.s24.odc.itin">ITIN dependents eligible (unlike CTC)</li>
                <li data-i18n="view.s24.odc.phase_out">Same $200K/$400K phase-out</li>
                <li data-i18n="view.s24.odc.support_test">Support test: taxpayer provides &gt; ½ support</li>
                <li data-i18n="view.s24.odc.gross_income">Qualifying relative income test: &lt; $5,050 (2024 indexed)</li>
                <li data-i18n="view.s24.odc.no_residency">No residency requirement — qualifying relative need NOT live with taxpayer</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s24.h2.arpa">ARPA 2021 enhancements (expired)</h2>
            <ul class="muted small">
                <li data-i18n="view.s24.arpa.amounts">$3,600 under 6 / $3,000 ages 6-17</li>
                <li data-i18n="view.s24.arpa.fully">Fully refundable — no earned income required</li>
                <li data-i18n="view.s24.arpa.advance">50% paid as advance July-Dec 2021 (IRS Letter 6419)</li>
                <li data-i18n="view.s24.arpa.phaseouts">Two phase-outs: $75K/$150K (enhanced) then $200K/$400K (base)</li>
                <li data-i18n="view.s24.arpa.age_17">Age 17 included (not under 17)</li>
                <li data-i18n="view.s24.arpa.expansion_expired">Expired Dec 31, 2021 — NOT extended by IRA 2022</li>
                <li data-i18n="view.s24.arpa.repayment">Safe harbor protection for excess advance payments based on income</li>
                <li data-i18n="view.s24.arpa.s24i">§ 24(i) ARPA-specific provisions</li>
                <li data-i18n="view.s24.arpa.proposals">Reinstatement proposed in BBB / Inflation Reduction Act — not enacted</li>
            </ul>
        </div>
    `;
    document.getElementById('s24-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.qualifying_children_count = Number(fd.get('qualifying_children_count')) || 0;
        state.other_dependents_count = Number(fd.get('other_dependents_count')) || 0;
        state.magi = Number(fd.get('magi')) || 0;
        state.earned_income = Number(fd.get('earned_income')) || 0;
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.refundable_actc_taken = Number(fd.get('refundable_actc_taken')) || 0;
        state.children_ages = fd.get('children_ages') || '';
        state.children_with_ssn = Number(fd.get('children_with_ssn')) || 0;
        state.children_without_ssn = Number(fd.get('children_without_ssn')) || 0;
        state.is_arpa_year = !!fd.get('is_arpa_year');
        state.arpa_age_under_6 = Number(fd.get('arpa_age_under_6')) || 0;
        state.arpa_age_6_17 = Number(fd.get('arpa_age_6_17')) || 0;
        state.arpa_advance_payment = Number(fd.get('arpa_advance_payment')) || 0;
        state.is_excessive_actc_clawback = !!fd.get('is_excessive_actc_clawback');
        state.s24_h_ssn_required = !!fd.get('s24_h_ssn_required');
        state.s24_d_15pct_calculation = Number(fd.get('s24_d_15pct_calculation')) || 0;
        state.earned_income_2500_threshold = Number(fd.get('earned_income_2500_threshold')) || 0;
        state.s24_d_max_refundable = Number(fd.get('s24_d_max_refundable')) || 0;
        state.s24_d_full_refundable = !!fd.get('s24_d_full_refundable');
        state.s24_phaseout_complete = !!fd.get('s24_phaseout_complete');
        state.excess_advance_2021 = Number(fd.get('excess_advance_2021')) || 0;
        state.safe_harbor_2021 = !!fd.get('safe_harbor_2021');
        state.repayment_protection_income = Number(fd.get('repayment_protection_income')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s24-output');
    if (!el) return;
    const ctc_base = state.qualifying_children_count * 2000;
    const odc_base = state.other_dependents_count * 500;
    const threshold = state.filing_status === 'mfj' ? 400_000 : 200_000;
    const phase_out = Math.max(0, Math.ceil((state.magi - threshold) / 1000)) * 50;
    const ctc_after_phase = Math.max(0, ctc_base - phase_out);
    const odc_after_phase = Math.max(0, odc_base - Math.max(0, phase_out - ctc_base));
    const actc = Math.min(state.qualifying_children_count * 1700, Math.max(0, (state.earned_income - 2500)) * 0.15);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s24.h2.result">§ 24 CTC + ODC calculation</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s24.card.ctc">CTC base ($2K × kids)</div><div class="value">$${ctc_base.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s24.card.odc">ODC ($500 × others)</div><div class="value">$${odc_base.toLocaleString()}</div></div>
                <div class="card ${phase_out > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s24.card.phase">Phase-out</div><div class="value">−$${phase_out.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s24.card.ctc_after">CTC after phase</div><div class="value">$${ctc_after_phase.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s24.card.odc_after">ODC after phase</div><div class="value">$${odc_after_phase.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s24.card.actc">ACTC refundable</div><div class="value">$${actc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
            </div>
        </div>
    `;
}
