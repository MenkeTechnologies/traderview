// IRC § 6601 — Interest on Underpayments + § 6621 Federal Underpayment Rate.
// Daily compound interest from due date until paid.
// Rate = federal short-term rate + 3 percentage points (most underpayments).
// "Hot interest" § 6621(c): large corporate underpayments ≥ $100K → +2pp = +5pp total.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    underpayment_amount: 0,
    quarter: 'Q4_2024',
    short_term_rate: 5.5,
    s6621_a_2_underpayment_rate: 8.5,
    s6621_c_hot_interest_rate: 10.5,
    is_large_corp_underpayment: false,
    corporate_threshold: 100000,
    s6621_a_1_overpayment_rate: 7.5,
    days_outstanding: 0,
    is_compound_daily: true,
    is_compound_monthly: false,
    s6601_a_due_date: '',
    payment_date: '',
    is_extension_filed: false,
    extension_to_date: '',
    grace_period_extension: false,
    s6601_b_3_payments_credited: 0,
    is_interest_paid: false,
    interest_paid: 0,
    s6601_e_1_loss_carryback: false,
    nol_carryback_year: 0,
    s6601_e_2_credit_carryback: false,
    credit_carryback_year: 0,
    s6601_e_3_foreign_tax_credit: false,
    interest_suspension_irs_failure_to_act: false,
    s6404_e_18_months_interest_abatement: false,
    irs_notice_demand_date: '',
    s6601_h_partnership_items: false,
    s6601_j_estate_tax_installment: false,
    s6166_estate_tax_installment: false,
    s6166_2pct_special_rate: 0,
    individual_failure_to_pay_penalty: 0,
    s6651_a_2_penalty: 0,
    s6651_a_3_penalty: 0,
    s6654_estimated_tax_penalty: 0,
    s6655_corp_estimated_penalty: 0,
    fraud_failure_to_pay: false,
    s6651_f_fraud_penalty: 0,
    is_subject_to_compounding: true,
    federal_short_term_rate_history: '',
};

export async function renderSection6601(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6601.h1.title">// § 6601 INTEREST ON UNDERPAYMENTS</span></h1>
        <p class="muted small" data-i18n="view.s6601.hint.intro">
            <strong>§ 6601</strong> imposes interest on underpayments from DUE DATE OF RETURN
            (NOT date of notice) until PAID. <strong>§ 6621(a)(2):</strong> underpayment rate =
            federal short-term rate + 3 percentage points (typical individuals + small corps).
            <strong>§ 6621(c) "hot interest":</strong> LARGE CORPORATE UNDERPAYMENT (&gt; $100K) +
            increase rate by 2pp = federal ST + 5pp. <strong>§ 6621(d) interest netting:</strong>
            simultaneous underpayment + overpayment for same period: net at lower (overpayment) rate.
            <strong>Daily compound interest</strong> (§ 6622). <strong>2024 Q4 rate:</strong> 8.0%
            (overpayment for individuals + ST + 3) / 8.0% (underpayment, both). Hot interest: 10.0%.
            <strong>§ 6404(e) abatement:</strong> for IRS delays of 18+ months (limited grounds).
            <strong>NOL/credit carrybacks:</strong> § 6601(e) — special timing rules.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6601.h2.inputs">Inputs</h2>
            <form id="s6601-form" class="inline-form">
                <label><span data-i18n="view.s6601.label.amount">Underpayment ($)</span>
                    <input type="number" step="0.01" name="underpayment_amount" value="${state.underpayment_amount}"></label>
                <label><span data-i18n="view.s6601.label.quarter">Quarter</span>
                    <select name="quarter">
                        <option value="Q1_2024" ${state.quarter === 'Q1_2024' ? 'selected' : ''}>Q1 2024 (8%)</option>
                        <option value="Q2_2024" ${state.quarter === 'Q2_2024' ? 'selected' : ''}>Q2 2024 (8%)</option>
                        <option value="Q3_2024" ${state.quarter === 'Q3_2024' ? 'selected' : ''}>Q3 2024 (8%)</option>
                        <option value="Q4_2024" ${state.quarter === 'Q4_2024' ? 'selected' : ''}>Q4 2024 (8%)</option>
                        <option value="Q1_2025" ${state.quarter === 'Q1_2025' ? 'selected' : ''}>Q1 2025 (7%)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6601.label.short_rate">Federal ST rate %</span>
                    <input type="number" step="0.01" name="short_term_rate" value="${state.short_term_rate}"></label>
                <label><span data-i18n="view.s6601.label.under_rate">§ 6621(a)(2) under %</span>
                    <input type="number" step="0.01" name="s6621_a_2_underpayment_rate" value="${state.s6621_a_2_underpayment_rate}"></label>
                <label><span data-i18n="view.s6601.label.hot">§ 6621(c) hot rate %</span>
                    <input type="number" step="0.01" name="s6621_c_hot_interest_rate" value="${state.s6621_c_hot_interest_rate}"></label>
                <label><span data-i18n="view.s6601.label.is_large">Large corp underpayment?</span>
                    <input type="checkbox" name="is_large_corp_underpayment" ${state.is_large_corp_underpayment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.threshold">Corp threshold ($)</span>
                    <input type="number" step="0.01" name="corporate_threshold" value="${state.corporate_threshold}"></label>
                <label><span data-i18n="view.s6601.label.over_rate">§ 6621(a)(1) over %</span>
                    <input type="number" step="0.01" name="s6621_a_1_overpayment_rate" value="${state.s6621_a_1_overpayment_rate}"></label>
                <label><span data-i18n="view.s6601.label.days">Days outstanding</span>
                    <input type="number" step="1" name="days_outstanding" value="${state.days_outstanding}"></label>
                <label><span data-i18n="view.s6601.label.daily">Compound daily?</span>
                    <input type="checkbox" name="is_compound_daily" ${state.is_compound_daily ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.monthly">Compound monthly?</span>
                    <input type="checkbox" name="is_compound_monthly" ${state.is_compound_monthly ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.due_date">Due date</span>
                    <input type="date" name="s6601_a_due_date" value="${state.s6601_a_due_date}"></label>
                <label><span data-i18n="view.s6601.label.pay_date">Payment date</span>
                    <input type="date" name="payment_date" value="${state.payment_date}"></label>
                <label><span data-i18n="view.s6601.label.ext">Extension filed?</span>
                    <input type="checkbox" name="is_extension_filed" ${state.is_extension_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.ext_to">Extension to</span>
                    <input type="date" name="extension_to_date" value="${state.extension_to_date}"></label>
                <label><span data-i18n="view.s6601.label.grace">Grace period?</span>
                    <input type="checkbox" name="grace_period_extension" ${state.grace_period_extension ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.credited">Payments credited ($)</span>
                    <input type="number" step="0.01" name="s6601_b_3_payments_credited" value="${state.s6601_b_3_payments_credited}"></label>
                <label><span data-i18n="view.s6601.label.int_paid">Interest paid?</span>
                    <input type="checkbox" name="is_interest_paid" ${state.is_interest_paid ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.int_amt">Interest amount ($)</span>
                    <input type="number" step="0.01" name="interest_paid" value="${state.interest_paid}"></label>
                <label><span data-i18n="view.s6601.label.nol_carryback">§ 6601(e)(1) NOL carryback?</span>
                    <input type="checkbox" name="s6601_e_1_loss_carryback" ${state.s6601_e_1_loss_carryback ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.nol_yr">NOL carryback yr</span>
                    <input type="number" step="1" name="nol_carryback_year" value="${state.nol_carryback_year}"></label>
                <label><span data-i18n="view.s6601.label.credit_carryback">§ 6601(e)(2) credit carryback?</span>
                    <input type="checkbox" name="s6601_e_2_credit_carryback" ${state.s6601_e_2_credit_carryback ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.credit_yr">Credit yr</span>
                    <input type="number" step="1" name="credit_carryback_year" value="${state.credit_carryback_year}"></label>
                <label><span data-i18n="view.s6601.label.ftc">§ 6601(e)(3) FTC?</span>
                    <input type="checkbox" name="s6601_e_3_foreign_tax_credit" ${state.s6601_e_3_foreign_tax_credit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.suspension">IRS failure-to-act suspension?</span>
                    <input type="checkbox" name="interest_suspension_irs_failure_to_act" ${state.interest_suspension_irs_failure_to_act ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.s6404_e">§ 6404(e) 18-mo abatement?</span>
                    <input type="checkbox" name="s6404_e_18_months_interest_abatement" ${state.s6404_e_18_months_interest_abatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.notice_date">IRS notice/demand date</span>
                    <input type="date" name="irs_notice_demand_date" value="${state.irs_notice_demand_date}"></label>
                <label><span data-i18n="view.s6601.label.ps">§ 6601(h) PS items?</span>
                    <input type="checkbox" name="s6601_h_partnership_items" ${state.s6601_h_partnership_items ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.estate_install">§ 6601(j) estate install?</span>
                    <input type="checkbox" name="s6601_j_estate_tax_installment" ${state.s6601_j_estate_tax_installment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.s6166">§ 6166 estate install?</span>
                    <input type="checkbox" name="s6166_estate_tax_installment" ${state.s6166_estate_tax_installment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.s6166_2pct">§ 6166 2% rate ($)</span>
                    <input type="number" step="0.01" name="s6166_2pct_special_rate" value="${state.s6166_2pct_special_rate}"></label>
                <label><span data-i18n="view.s6601.label.indiv_penalty">Indiv FTP penalty ($)</span>
                    <input type="number" step="0.01" name="individual_failure_to_pay_penalty" value="${state.individual_failure_to_pay_penalty}"></label>
                <label><span data-i18n="view.s6601.label.s6651a2">§ 6651(a)(2) ($)</span>
                    <input type="number" step="0.01" name="s6651_a_2_penalty" value="${state.s6651_a_2_penalty}"></label>
                <label><span data-i18n="view.s6601.label.s6651a3">§ 6651(a)(3) ($)</span>
                    <input type="number" step="0.01" name="s6651_a_3_penalty" value="${state.s6651_a_3_penalty}"></label>
                <label><span data-i18n="view.s6601.label.s6654">§ 6654 est tax ($)</span>
                    <input type="number" step="0.01" name="s6654_estimated_tax_penalty" value="${state.s6654_estimated_tax_penalty}"></label>
                <label><span data-i18n="view.s6601.label.s6655">§ 6655 corp est ($)</span>
                    <input type="number" step="0.01" name="s6655_corp_estimated_penalty" value="${state.s6655_corp_estimated_penalty}"></label>
                <label><span data-i18n="view.s6601.label.fraud">Fraud failure to pay?</span>
                    <input type="checkbox" name="fraud_failure_to_pay" ${state.fraud_failure_to_pay ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.s6651f">§ 6651(f) fraud ($)</span>
                    <input type="number" step="0.01" name="s6651_f_fraud_penalty" value="${state.s6651_f_fraud_penalty}"></label>
                <label><span data-i18n="view.s6601.label.compound">Subject to compound?</span>
                    <input type="checkbox" name="is_subject_to_compounding" ${state.is_subject_to_compounding ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6601.label.history">Federal ST rate history</span>
                    <input type="text" name="federal_short_term_rate_history" value="${esc(state.federal_short_term_rate_history)}"></label>
                <button class="primary" type="submit" data-i18n="view.s6601.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6601-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6601.h2.rates">2024 quarterly rates</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6601.tbl.quarter">Quarter</th><th data-i18n="view.s6601.tbl.under_indiv">Under (indiv)</th><th data-i18n="view.s6601.tbl.over_indiv">Over (indiv)</th><th data-i18n="view.s6601.tbl.under_corp">Under (corp)</th><th data-i18n="view.s6601.tbl.over_corp">Over (corp)</th><th data-i18n="view.s6601.tbl.hot">Hot interest</th></tr></thead>
                <tbody>
                    <tr><td>Q1 2024</td><td>8%</td><td>8%</td><td>8%</td><td>7%</td><td>10%</td></tr>
                    <tr><td>Q2 2024</td><td>8%</td><td>8%</td><td>8%</td><td>7%</td><td>10%</td></tr>
                    <tr><td>Q3 2024</td><td>8%</td><td>8%</td><td>8%</td><td>7%</td><td>10%</td></tr>
                    <tr><td>Q4 2024</td><td>8%</td><td>8%</td><td>8%</td><td>7%</td><td>10%</td></tr>
                    <tr><td>Q1 2025</td><td>7%</td><td>7%</td><td>7%</td><td>6%</td><td>9%</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6601.h2.calculation">Interest calculation</h2>
            <ul class="muted small">
                <li data-i18n="view.s6601.calc.daily">DAILY COMPOUND from due date until paid</li>
                <li data-i18n="view.s6601.calc.s6622">§ 6622(a) — daily compounding required</li>
                <li data-i18n="view.s6601.calc.s6621_rate">Rate = federal short-term rate + 3pp (most underpayments)</li>
                <li data-i18n="view.s6601.calc.corp_over">Corporate overpayments: rate = ST + 2pp (lower)</li>
                <li data-i18n="view.s6601.calc.large_corp_over">Large corp overpayment ≥ $10K: rate = ST + 0.5pp</li>
                <li data-i18n="view.s6601.calc.hot_corp_under">Hot corporate underpayment ≥ $100K: rate = ST + 5pp</li>
                <li data-i18n="view.s6601.calc.formula">Compound interest = Principal × (1 + r/365)^days - Principal</li>
                <li data-i18n="view.s6601.calc.rate_change">Rate changes quarterly — must use applicable rate per period</li>
                <li data-i18n="view.s6601.calc.s6621_d_netting">§ 6621(d) interest netting: simultaneous over + under → use overpayment rate</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6601.h2.special_timing">Special timing rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s6601.timing.due_date">§ 6601(a) interest from "last date prescribed for payment"</li>
                <li data-i18n="view.s6601.timing.extension">Extension to file ≠ extension to pay (interest from original due)</li>
                <li data-i18n="view.s6601.timing.nol_carryback">§ 6601(e)(1) NOL carryback: interest only AFTER loss year filing</li>
                <li data-i18n="view.s6601.timing.credit_carryback">§ 6601(e)(2) credit carryback: same as NOL</li>
                <li data-i18n="view.s6601.timing.ftc">§ 6601(e)(3) FTC: interest accrues for portion attributable to FTC limitation</li>
                <li data-i18n="view.s6601.timing.amended">Amended return: interest from original due date</li>
                <li data-i18n="view.s6601.timing.estimated">Estimated tax payments: applied chronologically</li>
                <li data-i18n="view.s6601.timing.transcript">Account transcript may show daily interest accruals</li>
                <li data-i18n="view.s6601.timing.s6601_b_3">§ 6601(b)(3) — payments applied as of received date</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6601.h2.abatement">§ 6404(e) interest abatement</h2>
            <ul class="muted small">
                <li data-i18n="view.s6601.abatement.scope">IRS delays of 18+ months in audit + collection (limited)</li>
                <li data-i18n="view.s6601.abatement.factors">Factors: tax type, year, IRS delay attributable</li>
                <li data-i18n="view.s6601.abatement.s6404_e_1">§ 6404(e)(1) — discretionary, IRS may abate "any portion" attributable to delay</li>
                <li data-i18n="view.s6601.abatement.s6404_e_2">§ 6404(e)(2) — mandatory for certain disasters</li>
                <li data-i18n="view.s6601.abatement.unilateral">Court review of IRS denial limited (abuse of discretion standard)</li>
                <li data-i18n="view.s6601.abatement.combat">§ 7508 — military combat zone suspension</li>
                <li data-i18n="view.s6601.abatement.s7508_a">§ 7508A — Presidentially-declared disaster suspension</li>
                <li data-i18n="view.s6601.abatement.s6404_h">§ 6404(h) — Tax Court jurisdiction over abatement denial</li>
                <li data-i18n="view.s6601.abatement.f843">Form 843 — claim for abatement of interest/penalties</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6601.h2.netting">§ 6621(d) interest netting</h2>
            <ul class="muted small">
                <li data-i18n="view.s6601.net.purpose">Eliminates rate differential when taxpayer has simultaneous over + under</li>
                <li data-i18n="view.s6601.net.scope">Applies when: same period + both balances exist on books</li>
                <li data-i18n="view.s6601.net.amount">Lesser of (overpayment balance) AND (underpayment balance) → net at OVERPAYMENT rate</li>
                <li data-i18n="view.s6601.net.benefit">Reduces interest exposure by ~1-3pp × balance × years</li>
                <li data-i18n="view.s6601.net.application_request">Must claim netting via formal request — not automatic</li>
                <li data-i18n="view.s6601.net.s6611">§ 6611 governs overpayment interest</li>
                <li data-i18n="view.s6601.net.refund_claim">Often claimed via refund claim + Form 843</li>
                <li data-i18n="view.s6601.net.notice_2004_19">Notice 2004-19 — § 6621(d) implementation</li>
            </ul>
        </div>
    `;
    document.getElementById('s6601-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.underpayment_amount = Number(fd.get('underpayment_amount')) || 0;
        state.quarter = fd.get('quarter');
        state.short_term_rate = Number(fd.get('short_term_rate')) || 0;
        state.s6621_a_2_underpayment_rate = Number(fd.get('s6621_a_2_underpayment_rate')) || 0;
        state.s6621_c_hot_interest_rate = Number(fd.get('s6621_c_hot_interest_rate')) || 0;
        state.is_large_corp_underpayment = !!fd.get('is_large_corp_underpayment');
        state.corporate_threshold = Number(fd.get('corporate_threshold')) || 0;
        state.s6621_a_1_overpayment_rate = Number(fd.get('s6621_a_1_overpayment_rate')) || 0;
        state.days_outstanding = Number(fd.get('days_outstanding')) || 0;
        state.is_compound_daily = !!fd.get('is_compound_daily');
        state.is_compound_monthly = !!fd.get('is_compound_monthly');
        state.s6601_a_due_date = fd.get('s6601_a_due_date') || '';
        state.payment_date = fd.get('payment_date') || '';
        state.is_extension_filed = !!fd.get('is_extension_filed');
        state.extension_to_date = fd.get('extension_to_date') || '';
        state.grace_period_extension = !!fd.get('grace_period_extension');
        state.s6601_b_3_payments_credited = Number(fd.get('s6601_b_3_payments_credited')) || 0;
        state.is_interest_paid = !!fd.get('is_interest_paid');
        state.interest_paid = Number(fd.get('interest_paid')) || 0;
        state.s6601_e_1_loss_carryback = !!fd.get('s6601_e_1_loss_carryback');
        state.nol_carryback_year = Number(fd.get('nol_carryback_year')) || 0;
        state.s6601_e_2_credit_carryback = !!fd.get('s6601_e_2_credit_carryback');
        state.credit_carryback_year = Number(fd.get('credit_carryback_year')) || 0;
        state.s6601_e_3_foreign_tax_credit = !!fd.get('s6601_e_3_foreign_tax_credit');
        state.interest_suspension_irs_failure_to_act = !!fd.get('interest_suspension_irs_failure_to_act');
        state.s6404_e_18_months_interest_abatement = !!fd.get('s6404_e_18_months_interest_abatement');
        state.irs_notice_demand_date = fd.get('irs_notice_demand_date') || '';
        state.s6601_h_partnership_items = !!fd.get('s6601_h_partnership_items');
        state.s6601_j_estate_tax_installment = !!fd.get('s6601_j_estate_tax_installment');
        state.s6166_estate_tax_installment = !!fd.get('s6166_estate_tax_installment');
        state.s6166_2pct_special_rate = Number(fd.get('s6166_2pct_special_rate')) || 0;
        state.individual_failure_to_pay_penalty = Number(fd.get('individual_failure_to_pay_penalty')) || 0;
        state.s6651_a_2_penalty = Number(fd.get('s6651_a_2_penalty')) || 0;
        state.s6651_a_3_penalty = Number(fd.get('s6651_a_3_penalty')) || 0;
        state.s6654_estimated_tax_penalty = Number(fd.get('s6654_estimated_tax_penalty')) || 0;
        state.s6655_corp_estimated_penalty = Number(fd.get('s6655_corp_estimated_penalty')) || 0;
        state.fraud_failure_to_pay = !!fd.get('fraud_failure_to_pay');
        state.s6651_f_fraud_penalty = Number(fd.get('s6651_f_fraud_penalty')) || 0;
        state.is_subject_to_compounding = !!fd.get('is_subject_to_compounding');
        state.federal_short_term_rate_history = fd.get('federal_short_term_rate_history') || '';
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6601-output');
    if (!el) return;
    const rate = state.is_large_corp_underpayment ? state.s6621_c_hot_interest_rate : state.s6621_a_2_underpayment_rate;
    const daily_rate = rate / 100 / 365;
    const compound_factor = Math.pow(1 + daily_rate, state.days_outstanding);
    const interest = state.underpayment_amount * (compound_factor - 1);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6601.h2.result">§ 6601 interest calculation</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s6601.card.under">Underpayment</div><div class="value">$${state.underpayment_amount.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6601.card.rate">Effective rate</div><div class="value">${rate.toFixed(2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.s6601.card.days">Days</div><div class="value">${state.days_outstanding}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s6601.card.interest">Compound interest</div><div class="value">$${interest.toLocaleString(undefined, { maximumFractionDigits: 2 })}</div></div>
                <div class="card ${state.is_large_corp_underpayment ? 'warn' : ''}"><div class="label" data-i18n="view.s6601.card.hot">Hot interest?</div><div class="value">${state.is_large_corp_underpayment ? 'YES (+2pp)' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
