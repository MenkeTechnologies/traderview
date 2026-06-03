// IRC § 1058 — Securities Loan Nonrecognition.
// Transferor's lending of securities = no gain/loss recognition (under specific conditions).
// "Securities loan" requires: identical securities returned + lender retains economic position + agreement at FMV.
// Pairs with § 871(m) (dividend equivalents) + § 263(g) carrying charges.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    lender_basis: 0,
    securities_fmv: 0,
    borrower_collateral: 0,
    is_identical_returned: false,
    loan_agreement_meets_s1058: false,
    is_securities_loan: false,
    qualified_securities_loan: false,
    s1058_a_no_recognition: false,
    s1058_b_1_collateral_required: false,
    collateral_threshold_pct: 100,
    s1058_b_2_fees_received: 0,
    s1058_b_3_at_will_termination: true,
    is_substitute_payment: false,
    substitute_payment_received: 0,
    is_dividend_substitute: false,
    is_interest_substitute: false,
    nra_lender: false,
    s871_m_substitute: false,
    s263_g_carrying_charges: 0,
    s246_a_holding_period: 0,
    s246_holding_period_satisfied: true,
    is_qualified_lender: false,
    is_DPP_dealer: false,
    fail_to_borrow_buy_in: false,
    short_sale_against_box: false,
    s1259_constructive_sale: false,
    rev_rul_70_598_safe: false,
    pre_1976_no_section: false,
    fully_paid_account: true,
};

export async function renderSection1058(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1058.h1.title">// § 1058 SECURITIES LOAN NONRECOGNITION</span></h1>
        <p class="muted small" data-i18n="view.s1058.hint.intro">
            <strong>§ 1058</strong> — TRANSFEROR's loan of securities to broker for short sale (or
            similar) is NONRECOGNITION event if agreement satisfies: (1) IDENTICAL securities returned,
            (2) Pays SUBSTITUTE dividends + interest equal to actual amounts, (3) Lender retains
            risks of OWNERSHIP, (4) Agreement terminable on demand (at will). <strong>Substitute
            payments</strong> received by lender: ordinary income (NOT dividend — no § 243 DRD,
            no qualified dividend rate). <strong>§ 871(m) impact:</strong> nonresident lender's
            substitute payments treated as US-source dividend equivalents — 30% withholding.
            <strong>§ 263(g)</strong> requires capitalization of substitute payments + fees as carrying
            charges (with offsetting position). <strong>§ 246</strong> dividend-received deduction
            holding period interruption — STARTS OVER when securities returned. <strong>Rev. Rul.
            70-598:</strong> standard form agreement satisfies § 1058 if meets statutory tests.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1058.h2.inputs">Inputs</h2>
            <form id="s1058-form" class="inline-form">
                <label><span data-i18n="view.s1058.label.basis">Lender basis ($)</span>
                    <input type="number" step="1000" name="lender_basis" value="${state.lender_basis}"></label>
                <label><span data-i18n="view.s1058.label.fmv">Securities FMV ($)</span>
                    <input type="number" step="1000" name="securities_fmv" value="${state.securities_fmv}"></label>
                <label><span data-i18n="view.s1058.label.collateral">Borrower collateral ($)</span>
                    <input type="number" step="1000" name="borrower_collateral" value="${state.borrower_collateral}"></label>
                <label><span data-i18n="view.s1058.label.identical">Identical returned?</span>
                    <input type="checkbox" name="is_identical_returned" ${state.is_identical_returned ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.agreement">Agreement meets § 1058?</span>
                    <input type="checkbox" name="loan_agreement_meets_s1058" ${state.loan_agreement_meets_s1058 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.is_loan">Is securities loan?</span>
                    <input type="checkbox" name="is_securities_loan" ${state.is_securities_loan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.qualified">Qualified loan?</span>
                    <input type="checkbox" name="qualified_securities_loan" ${state.qualified_securities_loan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.s1058a">§ 1058(a) nonrecognition?</span>
                    <input type="checkbox" name="s1058_a_no_recognition" ${state.s1058_a_no_recognition ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.collateral_required">Collateral required?</span>
                    <input type="checkbox" name="s1058_b_1_collateral_required" ${state.s1058_b_1_collateral_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.coll_pct">Collateral threshold %</span>
                    <input type="number" step="1" name="collateral_threshold_pct" value="${state.collateral_threshold_pct}"></label>
                <label><span data-i18n="view.s1058.label.fees">Loan fees received ($)</span>
                    <input type="number" step="100" name="s1058_b_2_fees_received" value="${state.s1058_b_2_fees_received}"></label>
                <label><span data-i18n="view.s1058.label.at_will">At-will termination?</span>
                    <input type="checkbox" name="s1058_b_3_at_will_termination" ${state.s1058_b_3_at_will_termination ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.substitute">Substitute payment?</span>
                    <input type="checkbox" name="is_substitute_payment" ${state.is_substitute_payment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.sub_amt">Substitute payment ($)</span>
                    <input type="number" step="100" name="substitute_payment_received" value="${state.substitute_payment_received}"></label>
                <label><span data-i18n="view.s1058.label.div_sub">Dividend substitute?</span>
                    <input type="checkbox" name="is_dividend_substitute" ${state.is_dividend_substitute ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.int_sub">Interest substitute?</span>
                    <input type="checkbox" name="is_interest_substitute" ${state.is_interest_substitute ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.nra">NRA lender?</span>
                    <input type="checkbox" name="nra_lender" ${state.nra_lender ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.871m">§ 871(m) substitute?</span>
                    <input type="checkbox" name="s871_m_substitute" ${state.s871_m_substitute ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.s263g">§ 263(g) carrying ($)</span>
                    <input type="number" step="100" name="s263_g_carrying_charges" value="${state.s263_g_carrying_charges}"></label>
                <label><span data-i18n="view.s1058.label.s246_a">§ 246 holding days</span>
                    <input type="number" step="1" name="s246_a_holding_period" value="${state.s246_a_holding_period}"></label>
                <label><span data-i18n="view.s1058.label.s246_sat">§ 246 holding satisfied?</span>
                    <input type="checkbox" name="s246_holding_period_satisfied" ${state.s246_holding_period_satisfied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.qualified_lender">Qualified lender?</span>
                    <input type="checkbox" name="is_qualified_lender" ${state.is_qualified_lender ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.dealer">DPP dealer?</span>
                    <input type="checkbox" name="is_DPP_dealer" ${state.is_DPP_dealer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.fail">Fail-to-borrow buy-in?</span>
                    <input type="checkbox" name="fail_to_borrow_buy_in" ${state.fail_to_borrow_buy_in ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.short_box">Short against box?</span>
                    <input type="checkbox" name="short_sale_against_box" ${state.short_sale_against_box ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.s1259">§ 1259 constructive?</span>
                    <input type="checkbox" name="s1259_constructive_sale" ${state.s1259_constructive_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.rr_70_598">Rev Rul 70-598 safe?</span>
                    <input type="checkbox" name="rev_rul_70_598_safe" ${state.rev_rul_70_598_safe ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.pre_1976">Pre-1976 (no § 1058)?</span>
                    <input type="checkbox" name="pre_1976_no_section" ${state.pre_1976_no_section ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1058.label.fully_paid">Fully paid account?</span>
                    <input type="checkbox" name="fully_paid_account" ${state.fully_paid_account ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1058.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1058-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1058.h2.requirements">§ 1058 statutory requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s1058.req.identical">IDENTICAL securities (same class, same issuer) must be returned</li>
                <li data-i18n="view.s1058.req.substitute_dividends">Substitute dividend / interest payments equal to amounts paid on borrowed securities</li>
                <li data-i18n="view.s1058.req.economic_position">Lender retains risk of loss + opportunity for gain (i.e., economic ownership)</li>
                <li data-i18n="view.s1058.req.at_will">Agreement terminable on DEMAND (at-will) by lender</li>
                <li data-i18n="view.s1058.req.s1058_b">§ 1058(b) regs add: collateral requirement, fee documentation, separate account</li>
                <li data-i18n="view.s1058.req.qualified_borrower">Borrower typically registered broker-dealer or qualifying institution</li>
                <li data-i18n="view.s1058.req.documentation">Written master securities lending agreement (e.g., GMSLA, MSLA)</li>
                <li data-i18n="view.s1058.req.collateral_typical">Typical: 102-105% of FMV cash collateral, daily mark-to-market</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1058.h2.substitute_pmts">Substitute payments — character</h2>
            <ul class="muted small">
                <li data-i18n="view.s1058.sub.ordinary">ORDINARY income to lender (Reg § 1.1058-2)</li>
                <li data-i18n="view.s1058.sub.no_drd">NOT eligible for § 243 dividends-received deduction (DRD)</li>
                <li data-i18n="view.s1058.sub.no_qualified">NOT qualified dividend income (no LTCG rate)</li>
                <li data-i18n="view.s1058.sub.s301_c">Compare § 301(c) actual dividend: capital + cost basis treatment</li>
                <li data-i18n="view.s1058.sub.s871m_30pct">NRA: § 871(m) 30% withholding (or treaty rate) on dividend substitutes</li>
                <li data-i18n="view.s1058.sub.fdap">Treated as US-source FDAP for NRA withholding</li>
                <li data-i18n="view.s1058.sub.s1059">§ 1059 extraordinary dividend rules do NOT apply to substitutes</li>
                <li data-i18n="view.s1058.sub.s246_a_3">§ 246(a)(3) holding period interruption — STARTS over when securities returned</li>
                <li data-i18n="view.s1058.sub.s263_g">§ 263(g) carrying charges: capitalize if offsetting position exists</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1058.h2.borrower_side">Borrower side — short sale treatment</h2>
            <ul class="muted small">
                <li data-i18n="view.s1058.borrow.s1233">§ 1233 short sale rules apply to borrower</li>
                <li data-i18n="view.s1058.borrow.s1259">§ 1259 constructive sale if borrower has appreciated position</li>
                <li data-i18n="view.s1058.borrow.s263_g">§ 263(g) capitalization of fees + substitute payments to carrying charges</li>
                <li data-i18n="view.s1058.borrow.s1058_c">§ 1058(c) — substitute payments paid by borrower = deductible (subject to § 263(g))</li>
                <li data-i18n="view.s1058.borrow.s871m_paymt">§ 871(m) dividend equivalent — borrower pays + withholds 30% on NRA payee</li>
                <li data-i18n="view.s1058.borrow.s1058_d">§ 1058(d) treats certain transfers as sales (anti-abuse)</li>
                <li data-i18n="view.s1058.borrow.s1259_d">§ 1259(c) constructive sale exceptions: closed within 30 days + maintained</li>
                <li data-i18n="view.s1058.borrow.exchange_traded">Exchange-traded fund rebalancing — special rules</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1058.h2.s263g">§ 263(g) carrying charges</h2>
            <ul class="muted small">
                <li data-i18n="view.s1058.s263.scope">Applies to taxpayer holding offsetting position (long stock + securities loan)</li>
                <li data-i18n="view.s1058.s263.capitalize">Substitute payments paid + interest + fees CAPITALIZED to basis</li>
                <li data-i18n="view.s1058.s263.exempt">Net cost basis when offsetting position disposed</li>
                <li data-i18n="view.s1058.s263.applies_borrower">Borrower with appreciated stock long: capitalize substitute payments paid</li>
                <li data-i18n="view.s1058.s263.no_offsetting">If no offsetting position: deductible as ordinary expense (not capitalized)</li>
                <li data-i18n="view.s1058.s263.s1092">Coordinates with § 1092 straddle + § 263A capitalization</li>
                <li data-i18n="view.s1058.s263.fnma">§ 1.263(g)-1 regs detail mechanics</li>
                <li data-i18n="view.s1058.s263.investment_interest">Net carrying charges may convert to § 163(d) investment interest later</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1058.h2.failed_loan">Failed § 1058 loan consequences</h2>
            <ul class="muted small">
                <li data-i18n="view.s1058.fail.recognition">Recognition event — treated as SALE at FMV (gain to lender)</li>
                <li data-i18n="view.s1058.fail.basis_collateral">Collateral becomes lender's basis in cash received</li>
                <li data-i18n="view.s1058.fail.s453">§ 453 installment method possibly available</li>
                <li data-i18n="view.s1058.fail.s263g">§ 263(g) interaction continues for offsetting position</li>
                <li data-i18n="view.s1058.fail.fail_to_borrow">Fail-to-borrow buy-in: collateral converted to cash, securities buy-back required</li>
                <li data-i18n="view.s1058.fail.identical_returned">If non-identical securities returned: gain/loss on difference</li>
                <li data-i18n="view.s1058.fail.holding_resets">Holding period of returned securities starts new (Reg § 1.1058-2(b)(3))</li>
                <li data-i18n="view.s1058.fail.constructive_sale">§ 1259 may apply to lender's position if borrower-disposal triggers</li>
            </ul>
        </div>
    `;
    document.getElementById('s1058-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.lender_basis = Number(fd.get('lender_basis')) || 0;
        state.securities_fmv = Number(fd.get('securities_fmv')) || 0;
        state.borrower_collateral = Number(fd.get('borrower_collateral')) || 0;
        state.is_identical_returned = !!fd.get('is_identical_returned');
        state.loan_agreement_meets_s1058 = !!fd.get('loan_agreement_meets_s1058');
        state.is_securities_loan = !!fd.get('is_securities_loan');
        state.qualified_securities_loan = !!fd.get('qualified_securities_loan');
        state.s1058_a_no_recognition = !!fd.get('s1058_a_no_recognition');
        state.s1058_b_1_collateral_required = !!fd.get('s1058_b_1_collateral_required');
        state.collateral_threshold_pct = Number(fd.get('collateral_threshold_pct')) || 0;
        state.s1058_b_2_fees_received = Number(fd.get('s1058_b_2_fees_received')) || 0;
        state.s1058_b_3_at_will_termination = !!fd.get('s1058_b_3_at_will_termination');
        state.is_substitute_payment = !!fd.get('is_substitute_payment');
        state.substitute_payment_received = Number(fd.get('substitute_payment_received')) || 0;
        state.is_dividend_substitute = !!fd.get('is_dividend_substitute');
        state.is_interest_substitute = !!fd.get('is_interest_substitute');
        state.nra_lender = !!fd.get('nra_lender');
        state.s871_m_substitute = !!fd.get('s871_m_substitute');
        state.s263_g_carrying_charges = Number(fd.get('s263_g_carrying_charges')) || 0;
        state.s246_a_holding_period = Number(fd.get('s246_a_holding_period')) || 0;
        state.s246_holding_period_satisfied = !!fd.get('s246_holding_period_satisfied');
        state.is_qualified_lender = !!fd.get('is_qualified_lender');
        state.is_DPP_dealer = !!fd.get('is_DPP_dealer');
        state.fail_to_borrow_buy_in = !!fd.get('fail_to_borrow_buy_in');
        state.short_sale_against_box = !!fd.get('short_sale_against_box');
        state.s1259_constructive_sale = !!fd.get('s1259_constructive_sale');
        state.rev_rul_70_598_safe = !!fd.get('rev_rul_70_598_safe');
        state.pre_1976_no_section = !!fd.get('pre_1976_no_section');
        state.fully_paid_account = !!fd.get('fully_paid_account');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1058-output');
    if (!el) return;
    const conditions_met = state.is_identical_returned && state.loan_agreement_meets_s1058 && state.s1058_b_3_at_will_termination;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1058.h2.result">§ 1058 nonrecognition</h2>
            <div class="cards">
                <div class="card ${conditions_met ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s1058.card.qualifies">Qualifies for § 1058?</div><div class="value">${conditions_met ? 'YES (nonrecognition)' : 'NO (taxable sale)'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1058.card.basis">Lender basis</div><div class="value">$${state.lender_basis.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1058.card.fmv">Securities FMV</div><div class="value">$${state.securities_fmv.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s1058.card.substitute">Substitute pmt (ordinary)</div><div class="value">$${state.substitute_payment_received.toLocaleString()}</div></div>
                <div class="card ${state.nra_lender && state.s871_m_substitute ? 'warn' : ''}"><div class="label" data-i18n="view.s1058.card.871m">§ 871(m) withholding</div><div class="value">${state.nra_lender && state.s871_m_substitute ? '30%' : 'N/A'}</div></div>
            </div>
        </div>
    `;
}
