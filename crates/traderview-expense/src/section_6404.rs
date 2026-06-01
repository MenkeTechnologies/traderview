//! IRC § 6404 — Abatement of interest, tax, additions to tax, and
//! penalties. Trader-tax cleanup procedure after audit cycle or
//! IRS administrative error. Distinct from `section_6664`
//! (reasonable cause defense to penalties), `section_6601`
//! (interest accrual on underpayments), and `section_6611`
//! (interest on overpayments).
//!
//! **§ 6404(a) — General abatement authority**. IRS may abate
//! unpaid portion of any assessment that is (1) excessive, (2)
//! assessed after expiration of applicable statute of
//! limitations, OR (3) erroneously or illegally assessed.
//! Discretionary; not a taxpayer right.
//!
//! **§ 6404(b) — No claim for abatement for interest, additions
//! to tax, additional amounts, or assessable penalties**.
//! Taxpayer has NO statutory right to file abatement claim for
//! these items (only for taxes themselves) — must rely on
//! discretionary IRS authority OR specific abatement provisions
//! (§ 6404(e)/(f)/(g)).
//!
//! **§ 6404(c) — Small tax balances**. IRS may abate
//! assessments of $5 or less.
//!
//! **§ 6404(e)(1) — Interest abatement for unreasonable error
//! or delay by IRS employee**. Most important interest-relief
//! pathway. IRS may abate interest on underpayment attributable
//! to MINISTERIAL or MANAGERIAL act of an IRS employee where:
//! - Error/delay occurred AFTER IRS contacted taxpayer in
//!   writing about examination, underpayment, or payment
//! - Taxpayer (or representative) did NOT significantly
//!   contribute to error or delay
//! - Error/delay was UNREASONABLE
//!
//! **Ministerial act** = procedural or mechanical act, NO
//! judgment or discretion, occurs after all prerequisites done
//! (conferences, supervisory reviews). **Managerial act** =
//! administrative act involving temporary/permanent loss of
//! records OR exercise of judgment/discretion relating to
//! personnel management. **Decisions concerning proper
//! application of federal tax law are NEITHER ministerial NOR
//! managerial** — interest from legal-judgment delay is NOT
//! abatable.
//!
//! **§ 6404(e)(2) — Erroneous refund check**. IRS may abate
//! interest assessed on an erroneous refund check up to $50,000
//! unless taxpayer caused the erroneous refund.
//!
//! **§ 6404(f) — Penalty abatement for erroneous written
//! advice**. ANY penalty attributable to erroneous written
//! advice from IRS may be abated if:
//! - Taxpayer requested the advice in writing
//! - Taxpayer provided accurate and adequate facts
//! - Taxpayer reasonably relied on the written advice
//!
//! **§ 6404(g) — Interest suspension for IRS delay in notice**.
//! If IRS fails to provide notice of additional tax liability
//! within 36 MONTHS of timely return filing, interest SUSPENDS
//! between 36-month mark and date of notice (21-day grace
//! period). Limited to individuals and certain returns.
//!
//! **§ 6404(h) — Tax Court review**. Tax Court may review IRS
//! refusal to abate interest under § 6404(e) for abuse of
//! discretion. Petition deadline 180 days after final
//! determination.
//!
//! **Claim mechanics**: File Form 843 within (a) 3 years of
//! original return filing OR (b) 2 years from payment of tax,
//! whichever is later (§ 6511 lookback applies).
//!
//! Citations: IRC § 6404(a) general abatement; § 6404(b) no
//! claim for interest/penalties; § 6404(c) small tax balance;
//! § 6404(e)(1) unreasonable error/delay interest abatement;
//! § 6404(e)(2) erroneous refund interest abatement ($50K cap);
//! § 6404(f) erroneous written advice penalty abatement;
//! § 6404(g) 36-month interest suspension; § 6404(h) Tax Court
//! review; Treas. Reg. § 301.6404-2 implementing regulation;
//! IRM 20.2.7 Abatement and Suspension of Underpayment Interest;
//! § 6511 lookback for Form 843 filing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AbatementPathway {
    /// § 6404(e)(1) — unreasonable error/delay by IRS employee.
    UnreasonableErrorOrDelay,
    /// § 6404(e)(2) — erroneous refund check ($50K cap).
    ErroneousRefundCheck,
    /// § 6404(f) — erroneous written advice from IRS.
    ErroneousWrittenAdvice,
    /// § 6404(g) — 36-month interest suspension for late IRS
    /// notification.
    ThirtySixMonthInterestSuspension,
    /// § 6404(c) — small tax balance ($5 or less).
    SmallTaxBalance,
    /// § 6404(a) — general abatement authority.
    GeneralAbatement,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6404Input {
    pub claim_pathway: AbatementPathway,
    /// Whether the Form 843 claim was filed within (a) 3 years of
    /// original return filing OR (b) 2 years from payment,
    /// whichever is later.
    pub claim_filed_within_lookback_window: bool,
    // § 6404(e)(1) fields:
    /// Whether the error/delay occurred AFTER IRS contacted the
    /// taxpayer in writing about examination or underpayment.
    pub error_after_written_irs_contact: bool,
    /// Whether the taxpayer or representative contributed to the
    /// error or delay (defeats § 6404(e)(1)).
    pub taxpayer_contributed_to_error_or_delay: bool,
    /// Whether the act was MINISTERIAL or MANAGERIAL (vs a
    /// legal-judgment-based decision, which is NOT abatable).
    pub act_was_ministerial_or_managerial: bool,
    // § 6404(e)(2) fields:
    /// Erroneous refund check amount in cents (subject to $50K
    /// cap under § 6404(e)(2)).
    pub erroneous_refund_amount_cents: i64,
    /// Whether the taxpayer caused the erroneous refund (defeats
    /// § 6404(e)(2)).
    pub taxpayer_caused_erroneous_refund: bool,
    // § 6404(f) fields:
    /// Whether the taxpayer requested the written advice from
    /// IRS in writing.
    pub written_advice_requested_in_writing: bool,
    /// Whether the taxpayer provided accurate and adequate facts
    /// to the IRS.
    pub taxpayer_provided_accurate_adequate_facts: bool,
    /// Whether the taxpayer reasonably relied on the written
    /// advice.
    pub reasonable_reliance_on_written_advice: bool,
    // § 6404(g) fields:
    /// Whether the return was timely filed (§ 6404(g) only
    /// applies to timely returns).
    pub return_timely_filed: bool,
    /// Days between timely return filing and IRS notification of
    /// additional liability. § 6404(g) suspends interest after
    /// 36 months (1,096 days approx).
    pub days_from_filing_to_irs_notification: u32,
    // § 6404(c) fields:
    /// Tax balance in cents — § 6404(c) abatement available
    /// for balances of $5 or less.
    pub small_tax_balance_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6404Result {
    pub abatement_authorized: bool,
    pub abatement_pathway: AbatementPathway,
    pub claim_within_filing_window: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6404Input) -> Section6404Result {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "Form 843 — IRS abatement claim; § 6511 lookback applies (3 years from return filing OR 2 years from payment, whichever is later)"
            .to_string(),
    );

    let claim_within_window = input.claim_filed_within_lookback_window;

    if !claim_within_window {
        notes.push(
            "§ 6511 — claim filed OUTSIDE 3-year-from-return / 2-year-from-payment lookback window; abatement BARRED regardless of substantive merit"
                .to_string(),
        );
        return Section6404Result {
            abatement_authorized: false,
            abatement_pathway: input.claim_pathway,
            claim_within_filing_window: false,
            citation: citation_for(input.claim_pathway),
            notes,
        };
    }

    let authorized = match input.claim_pathway {
        AbatementPathway::UnreasonableErrorOrDelay => {
            check_e_1(input, &mut notes)
        }
        AbatementPathway::ErroneousRefundCheck => {
            check_e_2(input, &mut notes)
        }
        AbatementPathway::ErroneousWrittenAdvice => {
            check_f(input, &mut notes)
        }
        AbatementPathway::ThirtySixMonthInterestSuspension => {
            check_g(input, &mut notes)
        }
        AbatementPathway::SmallTaxBalance => {
            check_c(input, &mut notes)
        }
        AbatementPathway::GeneralAbatement => {
            notes.push(
                "§ 6404(a) — IRS DISCRETIONARY authority to abate (1) excessive assessment, (2) post-SOL assessment, OR (3) erroneously / illegally assessed; not a taxpayer right"
                    .to_string(),
            );
            true
        }
    };

    notes.push(
        "§ 6404(h) — Tax Court may review IRS refusal to abate interest under § 6404(e) for abuse of discretion; petition deadline 180 days after final determination"
            .to_string(),
    );

    notes.push(
        "§ 6404(b) — taxpayer has NO statutory RIGHT to file abatement claim for interest / additions to tax / additional amounts / assessable penalties; relies on discretionary IRS authority OR specific § 6404(e)/(f)/(g) pathways"
            .to_string(),
    );

    Section6404Result {
        abatement_authorized: authorized,
        abatement_pathway: input.claim_pathway,
        claim_within_filing_window: true,
        citation: citation_for(input.claim_pathway),
        notes,
    }
}

fn check_e_1(input: &Section6404Input, notes: &mut Vec<String>) -> bool {
    if !input.error_after_written_irs_contact {
        notes.push(
            "§ 6404(e)(1) — error/delay must have occurred AFTER IRS contacted taxpayer in writing about examination, underpayment, or payment"
                .to_string(),
        );
        return false;
    }

    if input.taxpayer_contributed_to_error_or_delay {
        notes.push(
            "§ 6404(e)(1) — abatement BARRED when taxpayer or representative significantly contributed to error or delay"
                .to_string(),
        );
        return false;
    }

    if !input.act_was_ministerial_or_managerial {
        notes.push(
            "§ 6404(e)(1) + Treas. Reg. § 301.6404-2 — abatement BARRED for legal-judgment-based delays; act must be MINISTERIAL (procedural/mechanical, no judgment) OR MANAGERIAL (administrative, loss of records or personnel discretion)"
                .to_string(),
        );
        return false;
    }

    notes.push(
        "§ 6404(e)(1) — unreasonable error/delay abatement AUTHORIZED; ministerial act = procedural/mechanical with no judgment; managerial act = administrative (loss of records OR personnel discretion)"
            .to_string(),
    );
    true
}

fn check_e_2(input: &Section6404Input, notes: &mut Vec<String>) -> bool {
    notes.push(
        "§ 6404(e)(2) — interest abatement on erroneous refund check; $50,000 ($5,000,000 cents) cap"
            .to_string(),
    );

    if input.taxpayer_caused_erroneous_refund {
        notes.push(
            "§ 6404(e)(2) — abatement BARRED when taxpayer caused the erroneous refund"
                .to_string(),
        );
        return false;
    }

    let cap = 5_000_000i64;
    if input.erroneous_refund_amount_cents > cap {
        notes.push(format!(
            "§ 6404(e)(2) — erroneous refund (${}) exceeds $50,000 cap; abatement limited to first $50,000 of interest only",
            input.erroneous_refund_amount_cents / 100
        ));
        return true;
    }

    true
}

fn check_f(input: &Section6404Input, notes: &mut Vec<String>) -> bool {
    let three_elements_met = input.written_advice_requested_in_writing
        && input.taxpayer_provided_accurate_adequate_facts
        && input.reasonable_reliance_on_written_advice;

    if three_elements_met {
        notes.push(
            "§ 6404(f) — penalty abatement for erroneous written advice from IRS AUTHORIZED; three-element test satisfied: (1) advice requested in writing, (2) accurate + adequate facts provided, (3) reasonable reliance on advice"
                .to_string(),
        );
        true
    } else {
        notes.push(
            "§ 6404(f) — penalty abatement BARRED; three-element test requires ALL of: (1) advice requested in writing, (2) accurate + adequate facts provided by taxpayer, (3) reasonable reliance"
                .to_string(),
        );
        false
    }
}

fn check_g(input: &Section6404Input, notes: &mut Vec<String>) -> bool {
    if !input.return_timely_filed {
        notes.push(
            "§ 6404(g) — 36-month interest suspension applies ONLY to timely-filed returns"
                .to_string(),
        );
        return false;
    }

    let thirty_six_months_days = 1095u32;
    if input.days_from_filing_to_irs_notification > thirty_six_months_days {
        notes.push(format!(
            "§ 6404(g) — interest SUSPENDS for individuals when IRS fails to notify within 36 months ({} days) of timely return filing; days elapsed = {}; interest suspended from 36-month mark to IRS notification (21-day grace period)",
            thirty_six_months_days, input.days_from_filing_to_irs_notification
        ));
        true
    } else {
        notes.push(format!(
            "§ 6404(g) — IRS notification within 36-month window ({} days elapsed); interest suspension NOT engaged",
            input.days_from_filing_to_irs_notification
        ));
        false
    }
}

fn check_c(input: &Section6404Input, notes: &mut Vec<String>) -> bool {
    let five_dollars_cents = 500i64;
    if input.small_tax_balance_cents <= five_dollars_cents
        && input.small_tax_balance_cents > 0
    {
        notes.push(
            "§ 6404(c) — small tax balance abatement available for balances of $5 or less"
                .to_string(),
        );
        true
    } else {
        notes.push(format!(
            "§ 6404(c) — balance (${:.2}) exceeds $5 threshold; small-balance abatement not engaged",
            input.small_tax_balance_cents as f64 / 100.0
        ));
        false
    }
}

fn citation_for(pathway: AbatementPathway) -> &'static str {
    match pathway {
        AbatementPathway::UnreasonableErrorOrDelay => {
            "IRC §§ 6404(b), 6404(e)(1), 6404(h); Treas. Reg. § 301.6404-2; IRM 20.2.7; § 6511 lookback"
        }
        AbatementPathway::ErroneousRefundCheck => {
            "IRC §§ 6404(b), 6404(e)(2); § 6511 lookback"
        }
        AbatementPathway::ErroneousWrittenAdvice => {
            "IRC §§ 6404(b), 6404(f); § 6511 lookback"
        }
        AbatementPathway::ThirtySixMonthInterestSuspension => {
            "IRC § 6404(g); § 6511 lookback"
        }
        AbatementPathway::SmallTaxBalance => "IRC § 6404(c); § 6511 lookback",
        AbatementPathway::GeneralAbatement => "IRC § 6404(a); § 6511 lookback",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e_1_base() -> Section6404Input {
        Section6404Input {
            claim_pathway: AbatementPathway::UnreasonableErrorOrDelay,
            claim_filed_within_lookback_window: true,
            error_after_written_irs_contact: true,
            taxpayer_contributed_to_error_or_delay: false,
            act_was_ministerial_or_managerial: true,
            erroneous_refund_amount_cents: 0,
            taxpayer_caused_erroneous_refund: false,
            written_advice_requested_in_writing: false,
            taxpayer_provided_accurate_adequate_facts: false,
            reasonable_reliance_on_written_advice: false,
            return_timely_filed: false,
            days_from_filing_to_irs_notification: 0,
            small_tax_balance_cents: 0,
        }
    }

    fn f_base() -> Section6404Input {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::ErroneousWrittenAdvice;
        i.written_advice_requested_in_writing = true;
        i.taxpayer_provided_accurate_adequate_facts = true;
        i.reasonable_reliance_on_written_advice = true;
        i
    }

    fn g_base() -> Section6404Input {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::ThirtySixMonthInterestSuspension;
        i.return_timely_filed = true;
        i.days_from_filing_to_irs_notification = 1200;
        i
    }

    #[test]
    fn e_1_clean_path_authorized() {
        let r = check(&e_1_base());
        assert!(r.abatement_authorized);
        assert!(r.claim_within_filing_window);
    }

    #[test]
    fn e_1_no_written_irs_contact_bars() {
        let mut i = e_1_base();
        i.error_after_written_irs_contact = false;
        let r = check(&i);
        assert!(!r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(e)(1)") && n.contains("in writing")));
    }

    #[test]
    fn e_1_taxpayer_contribution_bars() {
        let mut i = e_1_base();
        i.taxpayer_contributed_to_error_or_delay = true;
        let r = check(&i);
        assert!(!r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(e)(1)") && n.contains("significantly contributed")));
    }

    #[test]
    fn e_1_legal_judgment_act_bars() {
        let mut i = e_1_base();
        i.act_was_ministerial_or_managerial = false;
        let r = check(&i);
        assert!(!r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 301.6404-2") && n.contains("legal-judgment-based")));
    }

    #[test]
    fn e_1_authorized_note_describes_ministerial_managerial() {
        let r = check(&e_1_base());
        assert!(r.notes.iter().any(|n| n.contains("ministerial act = procedural/mechanical") && n.contains("managerial act = administrative")));
    }

    #[test]
    fn outside_lookback_window_bars_all_pathways() {
        for pathway in [
            AbatementPathway::UnreasonableErrorOrDelay,
            AbatementPathway::ErroneousRefundCheck,
            AbatementPathway::ErroneousWrittenAdvice,
            AbatementPathway::ThirtySixMonthInterestSuspension,
            AbatementPathway::SmallTaxBalance,
            AbatementPathway::GeneralAbatement,
        ] {
            let mut i = e_1_base();
            i.claim_pathway = pathway;
            i.claim_filed_within_lookback_window = false;
            let r = check(&i);
            assert!(!r.abatement_authorized);
            assert!(!r.claim_within_filing_window);
        }
    }

    #[test]
    fn e_2_clean_refund_authorized() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::ErroneousRefundCheck;
        i.erroneous_refund_amount_cents = 1_000_000;
        let r = check(&i);
        assert!(r.abatement_authorized);
    }

    #[test]
    fn e_2_taxpayer_caused_refund_bars() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::ErroneousRefundCheck;
        i.erroneous_refund_amount_cents = 1_000_000;
        i.taxpayer_caused_erroneous_refund = true;
        let r = check(&i);
        assert!(!r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(e)(2)") && n.contains("taxpayer caused")));
    }

    #[test]
    fn e_2_refund_above_50k_cap_note() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::ErroneousRefundCheck;
        i.erroneous_refund_amount_cents = 6_000_000;
        let r = check(&i);
        assert!(r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("$50,000 cap")));
    }

    #[test]
    fn f_three_element_test_satisfied_authorized() {
        let r = check(&f_base());
        assert!(r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(f)") && n.contains("three-element test satisfied")));
    }

    #[test]
    fn f_missing_written_request_bars() {
        let mut i = f_base();
        i.written_advice_requested_in_writing = false;
        let r = check(&i);
        assert!(!r.abatement_authorized);
    }

    #[test]
    fn f_missing_accurate_facts_bars() {
        let mut i = f_base();
        i.taxpayer_provided_accurate_adequate_facts = false;
        let r = check(&i);
        assert!(!r.abatement_authorized);
    }

    #[test]
    fn f_missing_reasonable_reliance_bars() {
        let mut i = f_base();
        i.reasonable_reliance_on_written_advice = false;
        let r = check(&i);
        assert!(!r.abatement_authorized);
    }

    #[test]
    fn f_three_element_truth_table() {
        for w in [false, true] {
            for f in [false, true] {
                for r in [false, true] {
                    let mut i = f_base();
                    i.written_advice_requested_in_writing = w;
                    i.taxpayer_provided_accurate_adequate_facts = f;
                    i.reasonable_reliance_on_written_advice = r;
                    let result = check(&i);
                    assert_eq!(result.abatement_authorized, w && f && r);
                }
            }
        }
    }

    #[test]
    fn g_36_month_suspension_engages_above_window() {
        let mut i = g_base();
        i.days_from_filing_to_irs_notification = 1200;
        let r = check(&i);
        assert!(r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(g)") && n.contains("interest SUSPENDS")));
    }

    #[test]
    fn g_36_month_suspension_not_engaged_within_window() {
        let mut i = g_base();
        i.days_from_filing_to_irs_notification = 900;
        let r = check(&i);
        assert!(!r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(g)") && n.contains("NOT engaged")));
    }

    #[test]
    fn g_not_timely_filed_bars_suspension() {
        let mut i = g_base();
        i.return_timely_filed = false;
        let r = check(&i);
        assert!(!r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(g)") && n.contains("timely-filed returns")));
    }

    #[test]
    fn g_boundary_1095_days_no_suspension() {
        let mut i = g_base();
        i.days_from_filing_to_irs_notification = 1095;
        let r = check(&i);
        assert!(!r.abatement_authorized);
    }

    #[test]
    fn g_boundary_1096_days_engages_suspension() {
        let mut i = g_base();
        i.days_from_filing_to_irs_notification = 1096;
        let r = check(&i);
        assert!(r.abatement_authorized);
    }

    #[test]
    fn c_5_dollar_balance_authorized() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::SmallTaxBalance;
        i.small_tax_balance_cents = 500;
        let r = check(&i);
        assert!(r.abatement_authorized);
    }

    #[test]
    fn c_above_5_dollar_balance_not_authorized() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::SmallTaxBalance;
        i.small_tax_balance_cents = 501;
        let r = check(&i);
        assert!(!r.abatement_authorized);
    }

    #[test]
    fn c_zero_balance_not_authorized() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::SmallTaxBalance;
        i.small_tax_balance_cents = 0;
        let r = check(&i);
        assert!(!r.abatement_authorized);
    }

    #[test]
    fn general_abatement_authority_authorized() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::GeneralAbatement;
        let r = check(&i);
        assert!(r.abatement_authorized);
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(a)") && n.contains("DISCRETIONARY")));
    }

    #[test]
    fn form_843_lookback_note_always_present() {
        let r = check(&e_1_base());
        assert!(r.notes.iter().any(|n| n.contains("Form 843") && n.contains("§ 6511")));
    }

    #[test]
    fn section_6404_b_no_claim_right_note_present() {
        let r = check(&e_1_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(b)") && n.contains("NO statutory RIGHT")));
    }

    #[test]
    fn section_6404_h_tax_court_review_note() {
        let r = check(&e_1_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6404(h)") && n.contains("Tax Court") && n.contains("180 days")));
    }

    #[test]
    fn citation_e_1_pins_subsections_and_treas_reg() {
        let r = check(&e_1_base());
        assert!(r.citation.contains("§§ 6404(b), 6404(e)(1), 6404(h)"));
        assert!(r.citation.contains("§ 301.6404-2"));
        assert!(r.citation.contains("IRM 20.2.7"));
    }

    #[test]
    fn citation_e_2_pins_subsection_e_2() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::ErroneousRefundCheck;
        let r = check(&i);
        assert!(r.citation.contains("§§ 6404(b), 6404(e)(2)"));
    }

    #[test]
    fn citation_f_pins_subsection_f() {
        let r = check(&f_base());
        assert!(r.citation.contains("§§ 6404(b), 6404(f)"));
    }

    #[test]
    fn citation_g_pins_subsection_g() {
        let r = check(&g_base());
        assert!(r.citation.contains("§ 6404(g)"));
    }

    #[test]
    fn citation_c_pins_subsection_c() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::SmallTaxBalance;
        let r = check(&i);
        assert!(r.citation.contains("§ 6404(c)"));
    }

    #[test]
    fn citation_a_pins_subsection_a() {
        let mut i = e_1_base();
        i.claim_pathway = AbatementPathway::GeneralAbatement;
        let r = check(&i);
        assert!(r.citation.contains("§ 6404(a)"));
    }

    #[test]
    fn six_pathways_routed_correctly() {
        for pathway in [
            AbatementPathway::UnreasonableErrorOrDelay,
            AbatementPathway::ErroneousRefundCheck,
            AbatementPathway::ErroneousWrittenAdvice,
            AbatementPathway::ThirtySixMonthInterestSuspension,
            AbatementPathway::SmallTaxBalance,
            AbatementPathway::GeneralAbatement,
        ] {
            let mut i = f_base();
            i.claim_pathway = pathway;
            i.return_timely_filed = true;
            i.days_from_filing_to_irs_notification = 1200;
            i.small_tax_balance_cents = 500;
            i.erroneous_refund_amount_cents = 1_000_000;
            i.error_after_written_irs_contact = true;
            i.act_was_ministerial_or_managerial = true;
            let r = check(&i);
            let _ = r.abatement_authorized;
            assert_eq!(r.abatement_pathway, pathway);
        }
    }
}
