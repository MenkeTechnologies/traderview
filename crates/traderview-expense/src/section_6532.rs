//! IRC § 6532 — Periods of limitation on suits. Procedural
//! companion to § 7422 (taxpayer-initiated refund suits),
//! § 7426 (third-party wrongful levy suits), § 7405 (IRS-
//! initiated erroneous refund recovery), § 6511 (refund
//! claim filing SOL), and § 6343 (release of levy +
//! administrative wrongful levy claim).
//!
//! Trader-critical: every refund-suit-eligible scenario
//! (NOL § 172/§ 475(f) carryback claim, § 1256 60/40 mark-
//! to-market amended return, wash-sale § 1091 recomputation,
//! § 988 currency loss restatement, § 988(a)(1)(B) qualified
//! business unit elections) pushes through § 6532(a)
//! 2-year-from-disallowance clock. Every third-party
//! wrongful levy claim (broker-held trading account, prime-
//! broker margin, custodial securities) pushes through
//! § 6532(c) 2-year-from-levy clock — **extended from 9
//! months to 2 years by Tax Cuts and Jobs Act of 2017
//! § 11071 (Pub. L. 115-97, effective for levies made
//! after December 22, 2017)**.
//!
//! **§ 6532(a) Taxpayer refund suits — six-month floor +
//! two-year ceiling**:
//! 1. § 6532(a)(1) — no suit under § 7422(a) shall be
//!    begun BEFORE expiration of 6 MONTHS from claim filing
//!    UNLESS Secretary renders decision within that time;
//!    NOR AFTER expiration of 2 YEARS from date of mailing
//!    by certified or registered mail of notice of
//!    disallowance.
//! 2. § 6532(a)(2) — 2-year period may be EXTENDED by
//!    written agreement between taxpayer and Secretary.
//! 3. § 6532(a)(3) — any consideration, reconsideration,
//!    or action by Secretary with respect to claim
//!    following mailing of disallowance shall NOT extend
//!    the period within which suit may be begun.
//! 4. § 6532(a)(4) — if taxpayer files written waiver of
//!    requirement that disallowance notice be mailed by
//!    certified mail, the 2-year period runs from FILING
//!    OF WAIVER.
//!
//! **§ 6532(b) US suits for erroneous refunds (§ 7405)** —
//! recovery of erroneous refund by suit pursuant to § 7405
//! shall be allowed only if begun WITHIN:
//! 1. **2 YEARS** after making of refund (standard); OR
//! 2. **5 YEARS** after making of refund if any part of
//!    refund induced by **FRAUD OR MISREPRESENTATION OF A
//!    MATERIAL FACT**.
//!
//! **§ 6532(c) Wrongful levy suits (§ 7426)** —
//! 1. § 6532(c)(1) — no suit or proceeding under § 7426
//!    shall be begun after expiration of **2 YEARS from
//!    DATE OF LEVY** or agreement giving rise to such
//!    action. **Tax Cuts and Jobs Act of 2017 § 11071
//!    EXTENDED the prior 9-month period to 2 years**,
//!    effective for levies made after December 22, 2017,
//!    AND levies on or before that date if the 9-month
//!    period had NOT expired under § 6343(b).
//! 2. § 6532(c)(2) — if § 6343(b) administrative request
//!    for return of wrongfully levied property is made by
//!    third party, the 2-year period may be EXTENDED until
//!    the SOONER of:
//!    - **12 MONTHS** from date third party filed § 6343(b)
//!      administrative request; OR
//!    - **6 MONTHS** from date IRS mailed notice of
//!      disallowance by certified or registered mail.
//!
//! Citations: 26 USC § 6532(a)-(c); Tax Cuts and Jobs Act of
//! 2017 § 11071 (Pub. L. 115-97, December 22, 2017); § 7422
//! refund suits; § 7426 wrongful levy; § 7405 erroneous
//! refund recovery; § 6511 refund claim filing; § 6343(b)
//! administrative wrongful levy claim; IRM 34.5.3 (Suits
//! Brought Against United States); IRS Pub. 4528
//! (Administrative Wrongful Levy Claim).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuitType {
    /// § 6532(a) — taxpayer refund suit under § 7422.
    TaxpayerRefundSuit,
    /// § 6532(b) — US erroneous refund recovery under
    /// § 7405.
    UsErroneousRefundSuit,
    /// § 6532(c) — third-party wrongful levy under § 7426.
    ThirdPartyWrongfulLevySuit,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6532Input {
    pub suit_type: SuitType,
    /// Days since claim filing (relevant for § 6532(a)(1)
    /// 6-month floor).
    pub days_since_claim_filing: u32,
    /// Whether IRS rendered decision on claim within 6
    /// months (lifts § 6532(a)(1) floor).
    pub irs_rendered_decision_within_6_months: bool,
    /// Days since notice of disallowance mailed by
    /// certified or registered mail (§ 6532(a) 2-year
    /// ceiling).
    pub days_since_disallowance_notice: u32,
    /// Days since refund was made (§ 6532(b) US suit clock).
    pub days_since_refund_made: u32,
    /// Whether refund induced by fraud or misrepresentation
    /// of material fact (extends § 6532(b) clock to 5 years).
    pub refund_induced_by_fraud_or_misrepresentation: bool,
    /// Days since date of levy (§ 6532(c)(1) 2-year clock).
    pub days_since_levy: u32,
    /// Whether levy made after December 22, 2017 (TCJA
    /// 2-year rule) vs prior 9-month rule.
    pub levy_after_tcja_effective_date: bool,
    /// Days since § 6343(b) administrative wrongful levy
    /// request filed (§ 6532(c)(2) extension trigger).
    pub days_since_section_6343b_request: u32,
    /// Days since IRS disallowance of § 6343(b)
    /// administrative request (§ 6532(c)(2) extension
    /// trigger).
    pub days_since_section_6343b_disallowance: u32,
    /// Whether § 6532(a)(2) written extension agreement
    /// between taxpayer and Secretary executed.
    pub written_extension_agreement_executed: bool,
    /// Days extended by § 6532(a)(2) written agreement
    /// (added to base 2-year period).
    pub written_extension_days_added: u32,
    /// Whether taxpayer filed § 6532(a)(4) written waiver
    /// of certified-mail disallowance notice requirement.
    pub waiver_of_certified_mail_filed: bool,
    /// Days since § 6532(a)(4) waiver was filed.
    pub days_since_waiver_filing: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6532Result {
    pub suit_type: SuitType,
    pub six_month_floor_satisfied: bool,
    pub two_year_ceiling_satisfied: bool,
    pub five_year_fraud_ceiling_engaged: bool,
    pub effective_sol_days: u32,
    pub suit_timely: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6532Input) -> Section6532Result {
    let mut failure_reasons: Vec<String> = Vec::new();
    let mut six_month_floor_satisfied = true;
    let two_year_ceiling_satisfied: bool;
    let mut five_year_fraud_ceiling_engaged = false;
    let effective_sol_days: u32;

    match input.suit_type {
        SuitType::TaxpayerRefundSuit => {
            let floor_lifted =
                input.irs_rendered_decision_within_6_months || input.days_since_claim_filing >= 180;
            six_month_floor_satisfied = floor_lifted;
            if !floor_lifted {
                failure_reasons.push(format!(
                    "26 USC § 6532(a)(1) — refund suit cannot be begun before expiration of 6 MONTHS from filing of refund claim unless Secretary renders decision within that time; only {} days since claim filing without IRS decision",
                    input.days_since_claim_filing
                ));
            }

            let base_two_year_days: u32 = 730;
            let extension_days = if input.written_extension_agreement_executed {
                input.written_extension_days_added
            } else {
                0
            };

            let ceiling_clock_days = if input.waiver_of_certified_mail_filed {
                input.days_since_waiver_filing
            } else {
                input.days_since_disallowance_notice
            };

            effective_sol_days = base_two_year_days.saturating_add(extension_days);
            two_year_ceiling_satisfied = ceiling_clock_days <= effective_sol_days;
            if !two_year_ceiling_satisfied {
                failure_reasons.push(format!(
                    "26 USC § 6532(a)(1) — refund suit cannot be begun after expiration of 2 YEARS (730 days) from date of mailing certified/registered mail notice of disallowance; effective SOL {} days, current {} days post-trigger",
                    effective_sol_days, ceiling_clock_days
                ));
            }
        }
        SuitType::UsErroneousRefundSuit => {
            five_year_fraud_ceiling_engaged = input.refund_induced_by_fraud_or_misrepresentation;
            effective_sol_days = if five_year_fraud_ceiling_engaged {
                1825
            } else {
                730
            };
            two_year_ceiling_satisfied = input.days_since_refund_made <= effective_sol_days;
            if !two_year_ceiling_satisfied {
                failure_reasons.push(format!(
                    "26 USC § 6532(b) — US erroneous refund suit must be begun within 2 YEARS (730 days) of making of refund, OR 5 YEARS (1825 days) if refund induced by fraud or misrepresentation; effective SOL {} days, current {} days post-refund",
                    effective_sol_days, input.days_since_refund_made
                ));
            }
        }
        SuitType::ThirdPartyWrongfulLevySuit => {
            let base_sol_days: u32 = if input.levy_after_tcja_effective_date {
                730
            } else {
                270
            };

            let mut effective_levy_sol = base_sol_days;
            if input.days_since_section_6343b_request > 0 {
                let twelve_month_extension = input.days_since_section_6343b_request;
                let six_month_disallowance_clock =
                    if input.days_since_section_6343b_disallowance > 0 {
                        Some(input.days_since_section_6343b_disallowance)
                    } else {
                        None
                    };

                let twelve_month_cap: u32 = 365;
                let six_month_cap: u32 = 180;

                let extension_option_1 = base_sol_days
                    .saturating_add(twelve_month_cap.saturating_sub(twelve_month_extension));
                let extension_option_2 = six_month_disallowance_clock.map(|d_disallowance| {
                    base_sol_days.saturating_add(six_month_cap.saturating_sub(d_disallowance))
                });

                effective_levy_sol = match extension_option_2 {
                    Some(opt2) => extension_option_1.min(opt2),
                    None => extension_option_1,
                };
            }

            effective_sol_days = effective_levy_sol;
            two_year_ceiling_satisfied = input.days_since_levy <= effective_levy_sol;
            if !two_year_ceiling_satisfied {
                failure_reasons.push(format!(
                    "26 USC § 6532(c)(1) — third-party wrongful levy suit under § 7426 must be begun within {} days of levy (TCJA 2017 extended 9 months to 2 YEARS for post-2017-12-22 levies); current {} days post-levy",
                    effective_levy_sol, input.days_since_levy
                ));
            }
        }
    }

    let suit_timely =
        six_month_floor_satisfied && two_year_ceiling_satisfied && failure_reasons.is_empty();

    let notes: Vec<String> = vec![
        "26 USC § 6532(a)(1) — refund suit under § 7422(a) cannot be begun BEFORE 6 MONTHS from claim filing UNLESS Secretary renders decision within that time, NOR AFTER 2 YEARS from date of mailing certified or registered mail notice of disallowance".to_string(),
        "26 USC § 6532(a)(2) — 2-year period may be EXTENDED by written agreement between taxpayer and Secretary".to_string(),
        "26 USC § 6532(a)(3) — any consideration, reconsideration, or action by Secretary with respect to claim following mailing of disallowance shall NOT extend period within which suit may be begun".to_string(),
        "26 USC § 6532(a)(4) — if taxpayer files written waiver of requirement that disallowance notice be mailed by certified mail, 2-year period runs from FILING OF WAIVER".to_string(),
        "26 USC § 6532(b) — US erroneous refund recovery under § 7405 must be begun within 2 YEARS (730 days) of making of refund OR 5 YEARS (1825 days) if refund induced by FRAUD OR MISREPRESENTATION OF A MATERIAL FACT".to_string(),
        "26 USC § 6532(c)(1) — third-party wrongful levy suit under § 7426 must be begun within 2 YEARS of date of levy (TCJA 2017 § 11071 EXTENDED prior 9-month period to 2 years for levies made after December 22, 2017; also applies to pre-2017-12-22 levies if 9-month period had not yet expired under § 6343(b))".to_string(),
        "26 USC § 6532(c)(2) — if § 6343(b) administrative request for return of wrongfully levied property is made by third party, 2-year period may be EXTENDED to SOONER of (A) 12 months from date third party filed § 6343(b) administrative request OR (B) 6 months from date IRS mailed disallowance notice by certified/registered mail".to_string(),
        "Tax Cuts and Jobs Act of 2017 § 11071 (Pub. L. 115-97, December 22, 2017) — extended § 6532(c)(1) and § 6343(b) wrongful-levy SOL from 9 months to 2 years; applies to levies made after December 22, 2017, AND levies on or before that date if 9-month period had NOT expired".to_string(),
        "IRM 34.5.3 (Suits Brought Against the United States) — internal IRS guidance on § 6532 SOL administration; IRS Pub. 4528 (Administrative Wrongful Levy Claim) — third-party § 6343(b) procedural guidance".to_string(),
    ];

    Section6532Result {
        suit_type: input.suit_type,
        six_month_floor_satisfied,
        two_year_ceiling_satisfied,
        five_year_fraud_ceiling_engaged,
        effective_sol_days,
        suit_timely,
        failure_reasons,
        citation: "26 USC § 6532(a) + § 6532(b) + § 6532(c); Tax Cuts and Jobs Act of 2017 § 11071 (Pub. L. 115-97, December 22, 2017); § 7422; § 7426; § 7405; § 6511; § 6343(b); IRM 34.5.3; IRS Pub. 4528",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taxpayer_refund_base() -> Section6532Input {
        Section6532Input {
            suit_type: SuitType::TaxpayerRefundSuit,
            days_since_claim_filing: 200,
            irs_rendered_decision_within_6_months: false,
            days_since_disallowance_notice: 365,
            days_since_refund_made: 0,
            refund_induced_by_fraud_or_misrepresentation: false,
            days_since_levy: 0,
            levy_after_tcja_effective_date: true,
            days_since_section_6343b_request: 0,
            days_since_section_6343b_disallowance: 0,
            written_extension_agreement_executed: false,
            written_extension_days_added: 0,
            waiver_of_certified_mail_filed: false,
            days_since_waiver_filing: 0,
        }
    }

    #[test]
    fn taxpayer_refund_baseline_timely() {
        let r = check(&taxpayer_refund_base());
        assert!(r.suit_timely);
        assert!(r.six_month_floor_satisfied);
        assert!(r.two_year_ceiling_satisfied);
    }

    #[test]
    fn taxpayer_refund_under_6_months_no_decision_floor_violated() {
        let mut i = taxpayer_refund_base();
        i.days_since_claim_filing = 179;
        i.irs_rendered_decision_within_6_months = false;
        let r = check(&i);
        assert!(!r.six_month_floor_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6532(a)(1)") && f.contains("6 MONTHS")));
    }

    #[test]
    fn taxpayer_refund_at_180_days_floor_lifted() {
        let mut i = taxpayer_refund_base();
        i.days_since_claim_filing = 180;
        let r = check(&i);
        assert!(r.six_month_floor_satisfied);
    }

    #[test]
    fn taxpayer_refund_irs_early_decision_lifts_floor() {
        let mut i = taxpayer_refund_base();
        i.days_since_claim_filing = 30;
        i.irs_rendered_decision_within_6_months = true;
        let r = check(&i);
        assert!(r.six_month_floor_satisfied);
    }

    #[test]
    fn taxpayer_refund_at_730_days_post_disallowance_compliant() {
        let mut i = taxpayer_refund_base();
        i.days_since_disallowance_notice = 730;
        let r = check(&i);
        assert!(r.two_year_ceiling_satisfied);
    }

    #[test]
    fn taxpayer_refund_at_731_days_post_disallowance_expired() {
        let mut i = taxpayer_refund_base();
        i.days_since_disallowance_notice = 731;
        let r = check(&i);
        assert!(!r.two_year_ceiling_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6532(a)(1)") && f.contains("2 YEARS")));
    }

    #[test]
    fn taxpayer_refund_section_a2_written_extension_adds_days() {
        let mut i = taxpayer_refund_base();
        i.days_since_disallowance_notice = 1000;
        i.written_extension_agreement_executed = true;
        i.written_extension_days_added = 365;
        let r = check(&i);
        assert!(r.two_year_ceiling_satisfied);
        assert_eq!(r.effective_sol_days, 1095);
    }

    #[test]
    fn taxpayer_refund_section_a4_waiver_runs_from_waiver_filing() {
        let mut i = taxpayer_refund_base();
        i.days_since_disallowance_notice = 9999;
        i.waiver_of_certified_mail_filed = true;
        i.days_since_waiver_filing = 100;
        let r = check(&i);
        assert!(r.two_year_ceiling_satisfied);
    }

    #[test]
    fn us_erroneous_refund_at_730_days_compliant() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::UsErroneousRefundSuit;
        i.days_since_refund_made = 730;
        let r = check(&i);
        assert_eq!(r.effective_sol_days, 730);
        assert!(r.two_year_ceiling_satisfied);
    }

    #[test]
    fn us_erroneous_refund_at_731_days_expired() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::UsErroneousRefundSuit;
        i.days_since_refund_made = 731;
        let r = check(&i);
        assert!(!r.two_year_ceiling_satisfied);
    }

    #[test]
    fn us_erroneous_refund_fraud_extends_to_1825() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::UsErroneousRefundSuit;
        i.refund_induced_by_fraud_or_misrepresentation = true;
        i.days_since_refund_made = 1800;
        let r = check(&i);
        assert!(r.five_year_fraud_ceiling_engaged);
        assert_eq!(r.effective_sol_days, 1825);
        assert!(r.two_year_ceiling_satisfied);
    }

    #[test]
    fn us_erroneous_refund_fraud_at_1826_days_expired() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::UsErroneousRefundSuit;
        i.refund_induced_by_fraud_or_misrepresentation = true;
        i.days_since_refund_made = 1826;
        let r = check(&i);
        assert!(!r.two_year_ceiling_satisfied);
    }

    #[test]
    fn wrongful_levy_post_tcja_730_days_compliant() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::ThirdPartyWrongfulLevySuit;
        i.days_since_levy = 730;
        i.levy_after_tcja_effective_date = true;
        let r = check(&i);
        assert_eq!(r.effective_sol_days, 730);
        assert!(r.two_year_ceiling_satisfied);
    }

    #[test]
    fn wrongful_levy_post_tcja_731_days_expired() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::ThirdPartyWrongfulLevySuit;
        i.days_since_levy = 731;
        i.levy_after_tcja_effective_date = true;
        let r = check(&i);
        assert!(!r.two_year_ceiling_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6532(c)(1)")
            && f.contains("TCJA 2017")
            && f.contains("9 months to 2 YEARS")));
    }

    #[test]
    fn wrongful_levy_pre_tcja_270_days_compliant() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::ThirdPartyWrongfulLevySuit;
        i.days_since_levy = 270;
        i.levy_after_tcja_effective_date = false;
        let r = check(&i);
        assert_eq!(r.effective_sol_days, 270);
        assert!(r.two_year_ceiling_satisfied);
    }

    #[test]
    fn wrongful_levy_pre_tcja_271_days_expired() {
        let mut i = taxpayer_refund_base();
        i.suit_type = SuitType::ThirdPartyWrongfulLevySuit;
        i.days_since_levy = 271;
        i.levy_after_tcja_effective_date = false;
        let r = check(&i);
        assert!(!r.two_year_ceiling_satisfied);
    }

    #[test]
    fn wrongful_levy_tcja_extension_unique_invariant() {
        let mut pre = taxpayer_refund_base();
        pre.suit_type = SuitType::ThirdPartyWrongfulLevySuit;
        pre.days_since_levy = 0;
        pre.levy_after_tcja_effective_date = false;
        let r_pre = check(&pre);

        let mut post = taxpayer_refund_base();
        post.suit_type = SuitType::ThirdPartyWrongfulLevySuit;
        post.days_since_levy = 0;
        post.levy_after_tcja_effective_date = true;
        let r_post = check(&post);

        assert!(r_post.effective_sol_days > r_pre.effective_sol_days);
        assert_eq!(r_pre.effective_sol_days, 270);
        assert_eq!(r_post.effective_sol_days, 730);
    }

    #[test]
    fn citation_pins_all_subsections_and_tcja() {
        let r = check(&taxpayer_refund_base());
        assert!(r.citation.contains("§ 6532(a)"));
        assert!(r.citation.contains("§ 6532(b)"));
        assert!(r.citation.contains("§ 6532(c)"));
        assert!(r.citation.contains("Tax Cuts and Jobs Act of 2017 § 11071"));
        assert!(r.citation.contains("Pub. L. 115-97"));
        assert!(r.citation.contains("December 22, 2017"));
        assert!(r.citation.contains("§ 7422"));
        assert!(r.citation.contains("§ 7426"));
        assert!(r.citation.contains("§ 7405"));
        assert!(r.citation.contains("§ 6511"));
        assert!(r.citation.contains("§ 6343(b)"));
        assert!(r.citation.contains("IRM 34.5.3"));
        assert!(r.citation.contains("IRS Pub. 4528"));
    }

    #[test]
    fn note_pins_subsection_a1_six_month_floor_two_year_ceiling() {
        let r = check(&taxpayer_refund_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6532(a)(1)")
            && n.contains("6 MONTHS")
            && n.contains("2 YEARS")
            && n.contains("certified or registered mail")));
    }

    #[test]
    fn note_pins_subsection_a2_written_extension() {
        let r = check(&taxpayer_refund_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6532(a)(2)") && n.contains("EXTENDED by written agreement")));
    }

    #[test]
    fn note_pins_subsection_a3_reconsideration_does_not_extend() {
        let r = check(&taxpayer_refund_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6532(a)(3)")
            && n.contains("reconsideration")
            && n.contains("NOT extend")));
    }

    #[test]
    fn note_pins_subsection_a4_waiver_runs_from_waiver_filing() {
        let r = check(&taxpayer_refund_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6532(a)(4)")
            && n.contains("written waiver")
            && n.contains("FILING OF WAIVER")));
    }

    #[test]
    fn note_pins_subsection_b_2_year_5_year_fraud() {
        let r = check(&taxpayer_refund_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6532(b)")
            && n.contains("2 YEARS (730 days)")
            && n.contains("5 YEARS (1825 days)")
            && n.contains("FRAUD OR MISREPRESENTATION OF A MATERIAL FACT")));
    }

    #[test]
    fn note_pins_subsection_c1_tcja_2_year() {
        let r = check(&taxpayer_refund_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6532(c)(1)")
            && n.contains("2 YEARS")
            && n.contains("TCJA 2017 § 11071")
            && n.contains("December 22, 2017")));
    }

    #[test]
    fn note_pins_subsection_c2_six_343b_12_month_6_month() {
        let r = check(&taxpayer_refund_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6532(c)(2)")
            && n.contains("12 months")
            && n.contains("6 months")
            && n.contains("SOONER")));
    }

    #[test]
    fn note_pins_tcja_2017_section_11071() {
        let r = check(&taxpayer_refund_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Tax Cuts and Jobs Act of 2017 § 11071")
                && n.contains("Pub. L. 115-97")
                && n.contains("9 months to 2 years")));
    }

    #[test]
    fn note_pins_irm_and_pub_4528() {
        let r = check(&taxpayer_refund_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("IRM 34.5.3") && n.contains("IRS Pub. 4528")));
    }

    #[test]
    fn suit_type_truth_table_three_cells() {
        for st in [
            SuitType::TaxpayerRefundSuit,
            SuitType::UsErroneousRefundSuit,
            SuitType::ThirdPartyWrongfulLevySuit,
        ] {
            let mut i = taxpayer_refund_base();
            i.suit_type = st;
            let r = check(&i);
            assert_eq!(r.suit_type, st);
        }
    }

    #[test]
    fn defensive_zero_days_all_suits_timely() {
        for st in [
            SuitType::UsErroneousRefundSuit,
            SuitType::ThirdPartyWrongfulLevySuit,
        ] {
            let mut i = taxpayer_refund_base();
            i.suit_type = st;
            i.days_since_refund_made = 0;
            i.days_since_levy = 0;
            let r = check(&i);
            assert!(r.two_year_ceiling_satisfied);
        }
    }

    #[test]
    fn effective_sol_truth_table_us_suits() {
        let mut std_input = taxpayer_refund_base();
        std_input.suit_type = SuitType::UsErroneousRefundSuit;
        std_input.refund_induced_by_fraud_or_misrepresentation = false;
        let r_std = check(&std_input);
        assert_eq!(r_std.effective_sol_days, 730);

        let mut fraud_input = taxpayer_refund_base();
        fraud_input.suit_type = SuitType::UsErroneousRefundSuit;
        fraud_input.refund_induced_by_fraud_or_misrepresentation = true;
        let r_fraud = check(&fraud_input);
        assert_eq!(r_fraud.effective_sol_days, 1825);
    }

    #[test]
    fn taxpayer_floor_lifted_at_exactly_180_days() {
        let mut i = taxpayer_refund_base();
        i.days_since_claim_filing = 180;
        i.irs_rendered_decision_within_6_months = false;
        let r = check(&i);
        assert!(r.six_month_floor_satisfied);
    }
}
