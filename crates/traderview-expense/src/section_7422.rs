//! IRC § 7422 — Civil actions for refund. Procedurally
//! completes the refund-procedure constellation. When a
//! trader has paid tax and wants to sue for refund (rather
//! than file a Tax Court petition pre-payment), § 7422
//! governs the procedural gates. Trader-relevant because
//! refund suits are the default pathway when:
//! - Trader-taxpayer missed § 6213 90-day Tax Court window,
//!   OR
//! - Trader-taxpayer affirmatively chose to pay first to
//!   stop interest accrual under § 6601, OR
//! - Issue not amenable to Tax Court (e.g., refund of
//!   penalty already paid, recovery of erroneous offset).
//!
//! Procedural-companion to § 7421 (Anti-Injunction Act + §
//! 7426 wrongful-levy exception — refund-after-payment is
//! AIA-exception pathway), § 7508A (disaster postponement of
//! refund-claim deadlines), § 7430 (litigation costs against
//! IRS), § 6511 (refund-claim limitations), § 6532
//! (limitations on refund suits), § 7429 (jeopardy review),
//! and § 7433 (civil damages for unauthorized collection).
//!
//! **Four pre-suit requirements**:
//!
//! **1. Flora full-payment rule** — Flora v. United States,
//! 362 U.S. 145 (1960). Taxpayer must have FULLY PAID the
//! assessment before suing for refund in district court or
//! Court of Federal Claims (limited exceptions). Distinguishes
//! refund jurisdiction from Tax Court pre-payment
//! jurisdiction.
//!
//! **2. Administrative claim filed within § 6511 deadline**
//! — taxpayer must file Form 843 / 1040-X within the LATER
//! of:
//! - 3 years after filing the original return, OR
//! - 2 years after payment of the tax.
//!
//! **3. Six-month wait period under § 6532(a)** — taxpayer
//! must wait at least 6 months after filing administrative
//! claim before bringing suit, UNLESS IRS issues a notice of
//! disallowance sooner.
//!
//! **4. Two-year filing window under § 6532(a)** — once IRS
//! mails notice of disallowance, taxpayer has 2 years from
//! mailing date to file refund suit.
//!
//! **§ 7422(e) concurrent jurisdiction limitation** — if
//! Secretary mails notice of deficiency BEFORE hearing in
//! district court / Court of Federal Claims, proceedings
//! must be STAYED during Tax Court petition window + 60
//! days. If taxpayer files Tax Court petition, district
//! court / Court of Federal Claims LOSES jurisdiction to
//! extent acquired by Tax Court.
//!
//! **Jurisdiction**: district court (concurrent with United
//! States Court of Federal Claims).
//!
//! Citations: 26 USC § 7422(a)-(e); § 6511 (refund-claim
//! limitations); § 6532(a) (refund suit limitations); Flora
//! v. United States, 362 U.S. 145 (1960); IRS Pub. 556; IRM
//! 25.6.2; 28 USC § 1346(a)(1) (district-court refund
//! jurisdiction); 28 USC § 1491 (Court of Federal Claims).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RefundCourt {
    /// District court (28 USC § 1346(a)(1)).
    DistrictCourt,
    /// United States Court of Federal Claims (28 USC § 1491).
    CourtOfFederalClaims,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7422Input {
    pub refund_court: RefundCourt,
    /// Whether the taxpayer has FULLY paid the assessment
    /// (Flora rule).
    pub full_payment_made: bool,
    /// Whether taxpayer has filed administrative claim with
    /// IRS under § 6511.
    pub administrative_claim_filed: bool,
    /// Days since taxpayer filed administrative claim (for §
    /// 6532(a) 6-month wait).
    pub days_since_administrative_claim: u32,
    /// Whether IRS issued notice of disallowance.
    pub irs_disallowance_issued: bool,
    /// Days since IRS mailed notice of disallowance (for §
    /// 6532(a) 2-year filing window).
    pub days_since_disallowance: u32,
    /// Whether Secretary mailed notice of deficiency BEFORE
    /// suit hearing (triggers § 7422(e) stay).
    pub deficiency_notice_before_hearing: bool,
    /// Whether taxpayer filed Tax Court petition after
    /// receiving deficiency notice (triggers § 7422(e)
    /// jurisdiction transfer).
    pub tax_court_petition_filed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7422Result {
    pub refund_suit_maintainable: bool,
    pub flora_full_payment_satisfied: bool,
    pub administrative_claim_satisfied: bool,
    pub six_month_wait_satisfied: bool,
    pub two_year_window_satisfied: bool,
    pub jurisdiction_stayed_under_7422e: bool,
    pub jurisdiction_transferred_to_tax_court: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7422Input) -> Section7422Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    if !input.full_payment_made {
        failure_reasons.push(
            "Flora v. United States, 362 U.S. 145 (1960) full-payment rule — taxpayer must have FULLY PAID assessment before bringing refund suit in district court or Court of Federal Claims (limited exceptions only)"
                .to_string(),
        );
    }

    if !input.administrative_claim_filed {
        failure_reasons.push(
            "26 USC § 7422(a) + § 6511 — administrative claim must be filed with IRS before bringing refund suit (within later of 3 years from return filing or 2 years from payment)"
                .to_string(),
        );
    }

    let six_month_satisfied = input.administrative_claim_filed
        && (input.days_since_administrative_claim >= 180 || input.irs_disallowance_issued);

    if input.administrative_claim_filed && !six_month_satisfied {
        failure_reasons.push(format!(
            "26 USC § 6532(a) — taxpayer must wait at least 6 months (180 days) after filing administrative claim before bringing suit ({} days elapsed; no disallowance issued)",
            input.days_since_administrative_claim
        ));
    }

    let two_year_satisfied = !input.irs_disallowance_issued || input.days_since_disallowance <= 730;

    if input.irs_disallowance_issued && !two_year_satisfied {
        failure_reasons.push(format!(
            "26 USC § 6532(a) — refund suit filed more than 2 years (730 days) after IRS mailed notice of disallowance ({} days elapsed)",
            input.days_since_disallowance
        ));
    }

    let jurisdiction_stayed =
        input.deficiency_notice_before_hearing && !input.tax_court_petition_filed;
    let jurisdiction_transferred =
        input.deficiency_notice_before_hearing && input.tax_court_petition_filed;

    if jurisdiction_stayed {
        failure_reasons.push(
            "26 USC § 7422(e) — Secretary mailed notice of deficiency BEFORE hearing; proceedings STAYED during 90-day Tax Court petition window + 60 days additional".to_string(),
        );
    }

    if jurisdiction_transferred {
        failure_reasons.push(
            "26 USC § 7422(e) — taxpayer filed Tax Court petition; district court / Court of Federal Claims LOSES jurisdiction to extent acquired by Tax Court".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 7422(a) + § 6511 — administrative claim filed within later of 3 years after return filing or 2 years after payment; Form 843 / 1040-X"
            .to_string(),
        "Flora v. United States, 362 U.S. 145 (1960) full-payment rule — taxpayer must fully pay assessment before suing in district court or Court of Federal Claims (limited exceptions); distinguishes refund jurisdiction from Tax Court pre-payment jurisdiction"
            .to_string(),
        "26 USC § 6532(a) — refund suit limitations: (1) wait at least 6 months after filing administrative claim unless IRS disallows sooner; (2) file suit within 2 years after IRS mailing of notice of disallowance"
            .to_string(),
        "26 USC § 7422(e) — concurrent jurisdiction limitation: if Secretary mails notice of deficiency before suit hearing, proceedings stayed during Tax Court petition window + 60 days; if taxpayer files Tax Court petition, district court / Court of Federal Claims loses jurisdiction to extent acquired by Tax Court"
            .to_string(),
        "28 USC § 1346(a)(1) district-court refund jurisdiction; 28 USC § 1491 Court of Federal Claims; pair with § 7421 Anti-Injunction Act (refund-after-payment is AIA-exception pathway) + § 7508A (disaster postponement of § 6511 deadlines) + § 7430 (litigation costs)"
            .to_string(),
    ];

    Section7422Result {
        refund_suit_maintainable: failure_reasons.is_empty(),
        flora_full_payment_satisfied: input.full_payment_made,
        administrative_claim_satisfied: input.administrative_claim_filed,
        six_month_wait_satisfied: six_month_satisfied,
        two_year_window_satisfied: two_year_satisfied,
        jurisdiction_stayed_under_7422e: jurisdiction_stayed,
        jurisdiction_transferred_to_tax_court: jurisdiction_transferred,
        failure_reasons,
        citation: "26 USC § 7422(a)-(e); § 6511; § 6532(a); Flora v. United States, 362 U.S. 145 (1960); 28 USC § 1346(a)(1); 28 USC § 1491; IRS Pub. 556; IRM 25.6.2",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn maintainable_base() -> Section7422Input {
        Section7422Input {
            refund_court: RefundCourt::DistrictCourt,
            full_payment_made: true,
            administrative_claim_filed: true,
            days_since_administrative_claim: 365,
            irs_disallowance_issued: false,
            days_since_disallowance: 0,
            deficiency_notice_before_hearing: false,
            tax_court_petition_filed: false,
        }
    }

    #[test]
    fn fully_compliant_suit_maintainable() {
        let r = check(&maintainable_base());
        assert!(r.refund_suit_maintainable);
        assert!(r.flora_full_payment_satisfied);
        assert!(r.administrative_claim_satisfied);
        assert!(r.six_month_wait_satisfied);
        assert!(r.two_year_window_satisfied);
        assert!(!r.jurisdiction_stayed_under_7422e);
        assert!(!r.jurisdiction_transferred_to_tax_court);
    }

    #[test]
    fn flora_partial_payment_violates() {
        let mut i = maintainable_base();
        i.full_payment_made = false;
        let r = check(&i);
        assert!(!r.refund_suit_maintainable);
        assert!(!r.flora_full_payment_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Flora") && f.contains("FULLY PAID")));
    }

    #[test]
    fn no_administrative_claim_violates() {
        let mut i = maintainable_base();
        i.administrative_claim_filed = false;
        let r = check(&i);
        assert!(!r.refund_suit_maintainable);
        assert!(!r.administrative_claim_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7422(a)") && f.contains("§ 6511")));
    }

    #[test]
    fn under_6_month_wait_violates() {
        let mut i = maintainable_base();
        i.days_since_administrative_claim = 179;
        let r = check(&i);
        assert!(!r.refund_suit_maintainable);
        assert!(!r.six_month_wait_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6532(a)") && f.contains("180 days") && f.contains("179")));
    }

    #[test]
    fn at_180_day_boundary_compliant() {
        let mut i = maintainable_base();
        i.days_since_administrative_claim = 180;
        let r = check(&i);
        assert!(r.refund_suit_maintainable);
        assert!(r.six_month_wait_satisfied);
    }

    #[test]
    fn irs_disallowance_short_circuits_6_month_wait() {
        let mut i = maintainable_base();
        i.days_since_administrative_claim = 30;
        i.irs_disallowance_issued = true;
        i.days_since_disallowance = 10;
        let r = check(&i);
        assert!(r.refund_suit_maintainable);
        assert!(r.six_month_wait_satisfied);
    }

    #[test]
    fn at_730_day_two_year_boundary_compliant() {
        let mut i = maintainable_base();
        i.irs_disallowance_issued = true;
        i.days_since_disallowance = 730;
        let r = check(&i);
        assert!(r.refund_suit_maintainable);
        assert!(r.two_year_window_satisfied);
    }

    #[test]
    fn at_731_days_two_year_window_violates() {
        let mut i = maintainable_base();
        i.irs_disallowance_issued = true;
        i.days_since_disallowance = 731;
        let r = check(&i);
        assert!(!r.refund_suit_maintainable);
        assert!(!r.two_year_window_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6532(a)") && f.contains("2 years") && f.contains("731")));
    }

    #[test]
    fn deficiency_notice_before_hearing_stays_proceedings() {
        let mut i = maintainable_base();
        i.deficiency_notice_before_hearing = true;
        i.tax_court_petition_filed = false;
        let r = check(&i);
        assert!(!r.refund_suit_maintainable);
        assert!(r.jurisdiction_stayed_under_7422e);
        assert!(!r.jurisdiction_transferred_to_tax_court);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7422(e)") && f.contains("STAYED")));
    }

    #[test]
    fn tax_court_petition_filed_transfers_jurisdiction() {
        let mut i = maintainable_base();
        i.deficiency_notice_before_hearing = true;
        i.tax_court_petition_filed = true;
        let r = check(&i);
        assert!(!r.refund_suit_maintainable);
        assert!(!r.jurisdiction_stayed_under_7422e);
        assert!(r.jurisdiction_transferred_to_tax_court);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7422(e)") && f.contains("LOSES jurisdiction")));
    }

    #[test]
    fn no_deficiency_notice_no_7422e_engagement() {
        let r = check(&maintainable_base());
        assert!(!r.jurisdiction_stayed_under_7422e);
        assert!(!r.jurisdiction_transferred_to_tax_court);
    }

    #[test]
    fn court_of_federal_claims_routing() {
        let mut i = maintainable_base();
        i.refund_court = RefundCourt::CourtOfFederalClaims;
        let r = check(&i);
        assert!(r.refund_suit_maintainable);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&maintainable_base());
        assert!(r.citation.contains("§ 7422(a)-(e)"));
        assert!(r.citation.contains("§ 6511"));
        assert!(r.citation.contains("§ 6532(a)"));
        assert!(r.citation.contains("Flora v. United States"));
        assert!(r.citation.contains("362 U.S. 145"));
        assert!(r.citation.contains("§ 1346(a)(1)"));
        assert!(r.citation.contains("§ 1491"));
        assert!(r.citation.contains("Pub. 556"));
        assert!(r.citation.contains("IRM 25.6.2"));
    }

    #[test]
    fn note_pins_administrative_claim_requirement() {
        let r = check(&maintainable_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7422(a)")
            && n.contains("§ 6511")
            && n.contains("3 years")
            && n.contains("2 years")
            && n.contains("Form 843")));
    }

    #[test]
    fn note_pins_flora_full_payment_rule() {
        let r = check(&maintainable_base());
        assert!(r.notes.iter().any(|n| n.contains("Flora")
            && n.contains("362 U.S. 145 (1960)")
            && n.contains("Tax Court pre-payment")));
    }

    #[test]
    fn note_pins_6_month_2_year_section_6532a() {
        let r = check(&maintainable_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6532(a)") && n.contains("6 months") && n.contains("2 years")));
    }

    #[test]
    fn note_pins_section_7422e_jurisdiction_transfer() {
        let r = check(&maintainable_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7422(e)")
            && n.contains("60 days")
            && n.contains("loses jurisdiction")));
    }

    #[test]
    fn note_pins_district_court_court_of_federal_claims_concurrent_jurisdiction() {
        let r = check(&maintainable_base());
        assert!(r.notes.iter().any(|n| n.contains("28 USC § 1346(a)(1)")
            && n.contains("28 USC § 1491")
            && n.contains("AIA-exception pathway")));
    }

    #[test]
    fn four_failures_stack_for_bare_suit() {
        let i = Section7422Input {
            refund_court: RefundCourt::DistrictCourt,
            full_payment_made: false,
            administrative_claim_filed: false,
            days_since_administrative_claim: 0,
            irs_disallowance_issued: true,
            days_since_disallowance: 800,
            deficiency_notice_before_hearing: true,
            tax_court_petition_filed: false,
        };
        let r = check(&i);
        assert!(!r.refund_suit_maintainable);
        assert!(r.failure_reasons.len() >= 4);
    }

    #[test]
    fn six_month_wait_truth_table() {
        for (days, disallowance, exp_satisfied) in [
            (0u32, false, false),
            (90, false, false),
            (179, false, false),
            (180, false, true),
            (365, false, true),
            (10, true, true),
            (0, true, true),
        ] {
            let mut i = maintainable_base();
            i.days_since_administrative_claim = days;
            i.irs_disallowance_issued = disallowance;
            i.days_since_disallowance = if disallowance { 5 } else { 0 };
            let r = check(&i);
            assert_eq!(r.six_month_wait_satisfied, exp_satisfied);
        }
    }

    #[test]
    fn two_year_window_truth_table() {
        for (days, disallowance, exp_satisfied) in [
            (0u32, false, true),
            (1000, false, true),
            (0, true, true),
            (365, true, true),
            (730, true, true),
            (731, true, false),
            (1000, true, false),
        ] {
            let mut i = maintainable_base();
            i.irs_disallowance_issued = disallowance;
            i.days_since_disallowance = days;
            let r = check(&i);
            assert_eq!(r.two_year_window_satisfied, exp_satisfied);
        }
    }

    #[test]
    fn section_7422e_truth_table() {
        for (deficiency, tax_court, exp_stayed, exp_transferred) in [
            (false, false, false, false),
            (false, true, false, false),
            (true, false, true, false),
            (true, true, false, true),
        ] {
            let mut i = maintainable_base();
            i.deficiency_notice_before_hearing = deficiency;
            i.tax_court_petition_filed = tax_court;
            let r = check(&i);
            assert_eq!(r.jurisdiction_stayed_under_7422e, exp_stayed);
            assert_eq!(r.jurisdiction_transferred_to_tax_court, exp_transferred);
        }
    }

    #[test]
    fn flora_rule_uniquely_blocks_district_court_invariant() {
        let mut i_district = maintainable_base();
        i_district.refund_court = RefundCourt::DistrictCourt;
        i_district.full_payment_made = false;
        let r_district = check(&i_district);
        assert!(!r_district.refund_suit_maintainable);

        let mut i_claims = maintainable_base();
        i_claims.refund_court = RefundCourt::CourtOfFederalClaims;
        i_claims.full_payment_made = false;
        let r_claims = check(&i_claims);
        assert!(!r_claims.refund_suit_maintainable);
    }

    #[test]
    fn at_179_days_violates_at_180_compliant_precision() {
        let mut i_179 = maintainable_base();
        i_179.days_since_administrative_claim = 179;
        let r_179 = check(&i_179);
        assert!(!r_179.six_month_wait_satisfied);

        let mut i_180 = maintainable_base();
        i_180.days_since_administrative_claim = 180;
        let r_180 = check(&i_180);
        assert!(r_180.six_month_wait_satisfied);
    }

    #[test]
    fn at_730_compliant_at_731_violates_precision() {
        let mut i_730 = maintainable_base();
        i_730.irs_disallowance_issued = true;
        i_730.days_since_disallowance = 730;
        let r_730 = check(&i_730);
        assert!(r_730.two_year_window_satisfied);

        let mut i_731 = maintainable_base();
        i_731.irs_disallowance_issued = true;
        i_731.days_since_disallowance = 731;
        let r_731 = check(&i_731);
        assert!(!r_731.two_year_window_satisfied);
    }

    #[test]
    fn jurisdiction_transfer_uniquely_engages_when_both_conditions_invariant() {
        let mut i = maintainable_base();
        i.deficiency_notice_before_hearing = true;
        i.tax_court_petition_filed = true;
        let r = check(&i);
        assert!(r.jurisdiction_transferred_to_tax_court);
        assert!(!r.jurisdiction_stayed_under_7422e);

        let mut i_only_deficiency = maintainable_base();
        i_only_deficiency.deficiency_notice_before_hearing = true;
        let r_stay = check(&i_only_deficiency);
        assert!(r_stay.jurisdiction_stayed_under_7422e);
        assert!(!r_stay.jurisdiction_transferred_to_tax_court);
    }
}
