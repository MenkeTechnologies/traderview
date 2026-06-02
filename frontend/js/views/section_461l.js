// IRC § 461(l) — Excess Business Loss Limitation.
// TCJA + IRA + IRA-extended limits net business losses for non-corporate taxpayers.
// 2024 thresholds: $305,000 single / $610,000 MFJ.
// Excess business loss carries forward as NOL under § 172.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'mfj',
    gross_business_income: 0,
    business_deductions: 0,
    net_business_income_or_loss: 0,
    other_taxable_income: 0,
    wage_income: 0,
    is_corporate_taxpayer: false,
    is_real_estate_professional: false,
    is_farmer: false,
    has_passive_loss_carryover: false,
    passive_loss_carryover: 0,
    has_nol_carryover: false,
    nol_carryover_pre_2018: 0,
    nol_carryover_post_2017: 0,
    year: 2024,
    s199a_deduction_taken: 0,
    is_self_employed: false,
    se_income_or_loss: 0,
    aggregated_business_count: 1,
    is_pass_through: true,
};

export async function renderSection461L(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s461l.h1.title">// § 461(l) EXCESS BUSINESS LOSS</span></h1>
        <p class="muted small" data-i18n="view.s461l.hint.intro">
            <strong>Non-corporate</strong> taxpayers may NOT deduct business losses in excess of
            (gross business income + gain) + threshold amount. <strong>2024:</strong> $305,000
            single / $610,000 MFJ (indexed). <strong>Originally TCJA</strong> (sunsets 2025) — extended
            by <strong>IRA 2022</strong> through 2026 and now <strong>OBBBA</strong> through 2028.
            <strong>Excess business loss = NOL</strong> carried forward under § 172. <strong>Aggregate
            all trades/businesses</strong> — both sole prop + S-corp + partnership + farm. Wages NOT
            included in gross business income for non-employees (Reg § 1.461-1). <strong>Exception:</strong>
            farmer can elect 5-year carryback under § 172(b)(1)(B).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s461l.h2.inputs">Inputs</h2>
            <form id="s461l-form" class="inline-form">
                <label><span data-i18n="view.s461l.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HOH</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s461l.label.gross">Gross business income ($)</span>
                    <input type="number" step="10000" name="gross_business_income" value="${state.gross_business_income}"></label>
                <label><span data-i18n="view.s461l.label.deductions">Business deductions ($)</span>
                    <input type="number" step="10000" name="business_deductions" value="${state.business_deductions}"></label>
                <label><span data-i18n="view.s461l.label.net">Net business income/loss ($)</span>
                    <input type="number" step="10000" name="net_business_income_or_loss" value="${state.net_business_income_or_loss}"></label>
                <label><span data-i18n="view.s461l.label.other">Other taxable income ($)</span>
                    <input type="number" step="10000" name="other_taxable_income" value="${state.other_taxable_income}"></label>
                <label><span data-i18n="view.s461l.label.wages">Wage income ($)</span>
                    <input type="number" step="10000" name="wage_income" value="${state.wage_income}"></label>
                <label><span data-i18n="view.s461l.label.corp">Corporate taxpayer?</span>
                    <input type="checkbox" name="is_corporate_taxpayer" ${state.is_corporate_taxpayer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461l.label.repro">Real estate professional?</span>
                    <input type="checkbox" name="is_real_estate_professional" ${state.is_real_estate_professional ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461l.label.farmer">Farmer?</span>
                    <input type="checkbox" name="is_farmer" ${state.is_farmer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461l.label.passive_carry">Passive loss carryover?</span>
                    <input type="checkbox" name="has_passive_loss_carryover" ${state.has_passive_loss_carryover ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461l.label.passive_amount">Passive loss carryover ($)</span>
                    <input type="number" step="10000" name="passive_loss_carryover" value="${state.passive_loss_carryover}"></label>
                <label><span data-i18n="view.s461l.label.nol_carry">NOL carryover?</span>
                    <input type="checkbox" name="has_nol_carryover" ${state.has_nol_carryover ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461l.label.nol_pre">NOL pre-2018 ($)</span>
                    <input type="number" step="10000" name="nol_carryover_pre_2018" value="${state.nol_carryover_pre_2018}"></label>
                <label><span data-i18n="view.s461l.label.nol_post">NOL post-2017 ($)</span>
                    <input type="number" step="10000" name="nol_carryover_post_2017" value="${state.nol_carryover_post_2017}"></label>
                <label><span data-i18n="view.s461l.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.s461l.label.s199a">§ 199A deduction taken ($)</span>
                    <input type="number" step="1000" name="s199a_deduction_taken" value="${state.s199a_deduction_taken}"></label>
                <label><span data-i18n="view.s461l.label.se">Self-employed?</span>
                    <input type="checkbox" name="is_self_employed" ${state.is_self_employed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461l.label.se_income">SE income/loss ($)</span>
                    <input type="number" step="10000" name="se_income_or_loss" value="${state.se_income_or_loss}"></label>
                <label><span data-i18n="view.s461l.label.agg_count">Aggregated business count</span>
                    <input type="number" step="1" name="aggregated_business_count" value="${state.aggregated_business_count}"></label>
                <label><span data-i18n="view.s461l.label.pass_through">Pass-through?</span>
                    <input type="checkbox" name="is_pass_through" ${state.is_pass_through ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s461l.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s461l-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s461l.h2.thresholds">2024 thresholds + historical</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s461l.tbl.year">Year</th><th data-i18n="view.s461l.tbl.single">Single</th><th data-i18n="view.s461l.tbl.mfj">MFJ</th><th data-i18n="view.s461l.tbl.note">Note</th></tr></thead>
                <tbody>
                    <tr><td>2018-2020</td><td>$250,000</td><td>$500,000</td><td data-i18n="view.s461l.tbl.tcja">TCJA original</td></tr>
                    <tr><td>2021</td><td>$262,000</td><td>$524,000</td><td data-i18n="view.s461l.tbl.indexed">First indexing</td></tr>
                    <tr><td>2022</td><td>$270,000</td><td>$540,000</td><td>—</td></tr>
                    <tr><td>2023</td><td>$289,000</td><td>$578,000</td><td>—</td></tr>
                    <tr><td>2024</td><td>$305,000</td><td>$610,000</td><td>—</td></tr>
                    <tr><td>2025</td><td>$313,000</td><td>$626,000</td><td>—</td></tr>
                    <tr><td>2026-2028</td><td>TBD indexed</td><td>TBD indexed</td><td data-i18n="view.s461l.tbl.obbba">OBBBA extension</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s461l.h2.stacking">Loss limitation stacking order</h2>
            <ol class="muted small">
                <li data-i18n="view.s461l.stack.basis">§ 704(d) outside basis limitation (partnership) or § 1366(d) basis (S-corp)</li>
                <li data-i18n="view.s461l.stack.amount">§ 465 at-risk limitation</li>
                <li data-i18n="view.s461l.stack.passive">§ 469 passive activity loss limitation</li>
                <li data-i18n="view.s461l.stack.s461">§ 461(l) excess business loss limitation</li>
                <li data-i18n="view.s461l.stack.nol">§ 172 NOL deduction (after § 461(l))</li>
                <li data-i18n="view.s461l.stack.cumulative">Cumulative: must apply ALL hurdles before claiming loss</li>
                <li data-i18n="view.s461l.stack.carry_forward">Suspended losses carry to next year + apply same stack</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s461l.h2.mechanics">§ 461(l) mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s461l.mech.aggregate">Aggregate ALL trades/businesses (sole prop + S-corp + partnership + farm)</li>
                <li data-i18n="view.s461l.mech.threshold_aboveline">Compute below threshold first, then apply</li>
                <li data-i18n="view.s461l.mech.gross_plus">Threshold = gross business income + business gains + threshold amount</li>
                <li data-i18n="view.s461l.mech.wages_excluded">Wages NOT included in gross business income for non-employees</li>
                <li data-i18n="view.s461l.mech.exclude_passive">Exclude § 469 passive activity income/loss</li>
                <li data-i18n="view.s461l.mech.excess_nol">Excess business loss = NOL under § 172 — carries forward 100% indefinite</li>
                <li data-i18n="view.s461l.mech.fpr_excluded">Foreign pass-through entities included (§ 875 trade or business)</li>
                <li data-i18n="view.s461l.mech.s199a_separate">§ 199A QBI deduction computed separately (not netted at § 461(l))</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s461l.h2.exceptions">Exceptions + special rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s461l.exc.corporate">C-corporations: § 461(l) does NOT apply</li>
                <li data-i18n="view.s461l.exc.trust">Trusts and estates: § 461(l) applies (per § 642(g) NOL coordination)</li>
                <li data-i18n="view.s461l.exc.real_estate">Real estate professionals: passive vs active matters under § 469 first</li>
                <li data-i18n="view.s461l.exc.farmer_5yr">Farmers: 5-year NOL carryback election under § 172(b)(1)(B)(ii) for farming loss</li>
                <li data-i18n="view.s461l.exc.s108_3">Cancellation of debt income (§ 108): NOT business income for § 461(l)</li>
                <li data-i18n="view.s461l.exc.partial_year">First year as taxpayer: full threshold available (no prorating)</li>
                <li data-i18n="view.s461l.exc.mfs">MFS: each spouse gets ½ MFJ threshold ($305K each)</li>
                <li data-i18n="view.s461l.exc.both_spouses">MFJ: both spouses' businesses aggregated to single $610K</li>
            </ul>
        </div>
    `;
    document.getElementById('s461l-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.gross_business_income = Number(fd.get('gross_business_income')) || 0;
        state.business_deductions = Number(fd.get('business_deductions')) || 0;
        state.net_business_income_or_loss = Number(fd.get('net_business_income_or_loss')) || 0;
        state.other_taxable_income = Number(fd.get('other_taxable_income')) || 0;
        state.wage_income = Number(fd.get('wage_income')) || 0;
        state.is_corporate_taxpayer = !!fd.get('is_corporate_taxpayer');
        state.is_real_estate_professional = !!fd.get('is_real_estate_professional');
        state.is_farmer = !!fd.get('is_farmer');
        state.has_passive_loss_carryover = !!fd.get('has_passive_loss_carryover');
        state.passive_loss_carryover = Number(fd.get('passive_loss_carryover')) || 0;
        state.has_nol_carryover = !!fd.get('has_nol_carryover');
        state.nol_carryover_pre_2018 = Number(fd.get('nol_carryover_pre_2018')) || 0;
        state.nol_carryover_post_2017 = Number(fd.get('nol_carryover_post_2017')) || 0;
        state.year = Number(fd.get('year')) || 0;
        state.s199a_deduction_taken = Number(fd.get('s199a_deduction_taken')) || 0;
        state.is_self_employed = !!fd.get('is_self_employed');
        state.se_income_or_loss = Number(fd.get('se_income_or_loss')) || 0;
        state.aggregated_business_count = Number(fd.get('aggregated_business_count')) || 0;
        state.is_pass_through = !!fd.get('is_pass_through');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s461l-output');
    if (!el) return;
    const threshold = state.filing_status === 'mfj' ? 610_000 : 305_000;
    const net = state.net_business_income_or_loss;
    const cap = state.gross_business_income + threshold;
    const excess_loss = net < 0 ? Math.max(0, Math.abs(net) - cap) : 0;
    const allowed_loss = net < 0 ? Math.abs(net) - excess_loss : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s461l.h2.result">§ 461(l) excess business loss</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s461l.card.net">Net business loss</div><div class="value">$${Math.abs(net).toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s461l.card.threshold">Threshold (gross + cap)</div><div class="value">$${cap.toLocaleString()}</div></div>
                <div class="card ${excess_loss > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s461l.card.excess">Excess business loss → NOL</div><div class="value">$${excess_loss.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s461l.card.allowed">Allowed current loss</div><div class="value">$${allowed_loss.toLocaleString()}</div></div>
            </div>
            ${excess_loss > 0 ? `<p class="muted small neg" style="margin-top:10px" data-i18n="view.s461l.excess_note">Excess of $${excess_loss.toLocaleString()} carries forward as § 172 NOL indefinitely + 80% income limit (post-TCJA).</p>` : ''}
        </div>
    `;
}
