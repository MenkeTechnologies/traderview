//! IRC § 7430 — Awarding of costs and certain fees. Trader-relevant
//! when a taxpayer prevails in administrative or judicial
//! proceedings against the IRS — under what conditions may the
//! taxpayer recover litigation costs and attorney fees from the
//! government?
//!
//! Distinct from `section_6402` (refund offsets), `section_6611`
//! (interest on overpayments), and `section_6213` (Tax Court
//! petition timing). This module addresses ONLY the fee-recovery
//! pathway following a successful taxpayer challenge.
//!
//! § 7430(a) GENERAL RULE — in any administrative or court
//! proceeding which is brought by or against the United States in
//! connection with the determination, collection, or refund of any
//! tax, interest, or penalty, the prevailing party may be awarded
//! a judgment or settlement for (1) reasonable administrative
//! costs incurred in connection with such administrative
//! proceeding within the IRS, and (2) reasonable litigation costs
//! incurred in connection with such court proceeding.
//!
//! § 7430(b)(1) EXHAUSTION OF ADMINISTRATIVE REMEDIES — judgment
//! for reasonable litigation costs shall not be awarded unless
//! the court determines that the prevailing party has EXHAUSTED
//! THE ADMINISTRATIVE REMEDIES available within the IRS.
//!
//! § 7430(c)(4) PREVAILING PARTY — taxpayer must (A) have
//! substantially prevailed with respect to the amount in
//! controversy OR the most significant issue or set of issues
//! presented; AND (C) meet net worth and size limitations. Per
//! § 7430(c)(4)(B) — taxpayer is NOT treated as prevailing party
//! if the United States establishes that the position of the
//! United States was SUBSTANTIALLY JUSTIFIED.
//!
//! § 7430(c)(4)(D) NET WORTH AND SIZE LIMITS (cross-reference to
//! 28 U.S.C. § 2412(d)(2)(B)):
//!   - Individual: NET WORTH ≤ $2,000,000 at time case filed
//!   - Unincorporated business / corporation / partnership: NET
//!     WORTH ≤ $7,000,000 at time case filed
//!   - Maximum 500 EMPLOYEES at filing (regardless of net worth)
//!
//! § 7430(c)(7) QUALIFIED OFFER (QO) RULE — taxpayer is treated as
//! prevailing party if the taxpayer's liability under the LAST
//! QUALIFIED OFFER would equal or EXCEED the liability under the
//! judgment entered by the court. The QO is a settlement offer
//! made during the administrative period that the IRS rejects;
//! if the eventual judgment matches or beats the QO from the
//! taxpayer's perspective, the IRS pays the post-QO fees.
//!
//! § 7430(c)(1)(B)(iii) HOURLY RATE CAP — attorney fees limited
//! to $125 per hour (1996 base) plus cost-of-living adjustment.
//! 2026 cap (Rev. Proc. 2025-32): $260 per hour.
//!
//! § 7430(c)(1)(A) REASONABLE COSTS — includes administrative
//! fees imposed by IRS, expert witness fees, costs of studies/
//! tests/analyses necessary for case preparation, and reasonable
//! attorney fees.
//!
//! Citations: IRC § 7430(a) (general rule); § 7430(b)(1)
//! (exhaustion of administrative remedies); § 7430(c)(1)(A)
//! (reasonable costs definition); § 7430(c)(1)(B)(iii) (hourly
//! rate cap); § 7430(c)(4)(A)/(B)/(C)/(D) (prevailing party
//! definition + substantial justification defense + net worth /
//! size limits); § 7430(c)(7) (qualified offer rule); 28 U.S.C.
//! § 2412(d)(2)(B) (net worth cross-reference); Rev. Proc.
//! 2025-32 (2026 hourly cap $260); 26 CFR § 301.7430-2 (regs);
//! IRS IRM 35.10.1 (Awards of Litigation and Administration
//! Costs).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Individual taxpayer — net worth ceiling $2M.
    Individual,
    /// Unincorporated business / corporation / partnership — net
    /// worth ceiling $7M.
    BusinessEntity,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrevailingPartyStatus {
    /// Taxpayer is prevailing party under § 7430(c)(4)(A) (amount-
    /// in-controversy / most significant issue).
    SubstantivelyPrevailed,
    /// Taxpayer is prevailing party under § 7430(c)(7) qualified
    /// offer rule (taxpayer's QO ≥ eventual judgment).
    QualifiedOfferRule,
    /// Taxpayer is NOT prevailing party.
    NotPrevailing,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7430Input {
    /// Whether the taxpayer substantially prevailed on the amount
    /// in controversy OR most significant issue (§ 7430(c)(4)(A)).
    pub substantially_prevailed_on_amount_or_issue: bool,
    /// Whether the IRS position was substantially justified
    /// (§ 7430(c)(4)(B) defense — defeats prevailing party
    /// status).
    pub irs_position_substantially_justified: bool,
    /// Whether the taxpayer exhausted available administrative
    /// remedies before petitioning Tax Court (§ 7430(b)(1)
    /// prerequisite).
    pub administrative_remedies_exhausted: bool,
    /// Taxpayer entity type for net worth ceiling.
    pub entity_type: EntityType,
    /// Net worth at time case was filed.
    pub net_worth_cents: i64,
    /// Employee count at time case was filed (§ 7430(c)(4)(D)
    /// 500-employee ceiling).
    pub employee_count_at_filing: u32,
    /// Whether the taxpayer made a § 7430(c)(7) qualified offer
    /// during the administrative period.
    pub qualified_offer_made: bool,
    /// Taxpayer's liability under the qualified offer (cents).
    pub qualified_offer_liability_cents: i64,
    /// Final judgment liability against the taxpayer (cents).
    pub judgment_liability_cents: i64,
    /// Total attorney hours billed.
    pub attorney_hours_billed: u32,
    /// Year fees were incurred (drives hourly rate cap).
    pub year_fees_incurred: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7430Result {
    pub prevailing_party_status: PrevailingPartyStatus,
    pub net_worth_test_satisfied: bool,
    pub employee_count_test_satisfied: bool,
    pub administrative_exhaustion_satisfied: bool,
    pub hourly_rate_cap_cents: i64,
    pub max_attorney_fees_awardable_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section7430Input) -> Section7430Result {
    let mut notes: Vec<String> = Vec::new();

    let net_worth_ceiling = match input.entity_type {
        EntityType::Individual => 200_000_000i64,
        EntityType::BusinessEntity => 700_000_000i64,
    };
    let net_worth_test_satisfied = input.net_worth_cents <= net_worth_ceiling;
    let employee_count_test_satisfied = input.employee_count_at_filing <= 500;

    if !net_worth_test_satisfied {
        notes.push(format!(
            "§ 7430(c)(4)(D) + 28 U.S.C. § 2412(d)(2)(B) — net worth {} cents exceeds ceiling of {} cents for {:?}; prevailing party status unavailable",
            input.net_worth_cents, net_worth_ceiling, input.entity_type
        ));
    }
    if !employee_count_test_satisfied {
        notes.push(format!(
            "§ 7430(c)(4)(D) — employee count {} exceeds 500-employee ceiling; prevailing party status unavailable regardless of net worth",
            input.employee_count_at_filing
        ));
    }

    let administrative_exhaustion_satisfied = input.administrative_remedies_exhausted;
    if !administrative_exhaustion_satisfied {
        notes.push(
            "§ 7430(b)(1) — administrative remedies within the IRS must be EXHAUSTED before litigation costs may be awarded"
                .to_string(),
        );
    }

    let qualified_offer_engaged = input.qualified_offer_made
        && input.qualified_offer_liability_cents >= input.judgment_liability_cents;
    if input.qualified_offer_made {
        if qualified_offer_engaged {
            notes.push(format!(
                "§ 7430(c)(7) qualified offer rule — taxpayer's QO liability {} cents ≥ judgment {} cents; taxpayer treated as prevailing party",
                input.qualified_offer_liability_cents, input.judgment_liability_cents
            ));
        } else {
            notes.push(format!(
                "§ 7430(c)(7) qualified offer rule — taxpayer's QO liability {} cents < judgment {} cents; QO does NOT engage prevailing party status",
                input.qualified_offer_liability_cents, input.judgment_liability_cents
            ));
        }
    }

    let prevailing_party_status = if !net_worth_test_satisfied || !employee_count_test_satisfied {
        PrevailingPartyStatus::NotPrevailing
    } else if qualified_offer_engaged {
        PrevailingPartyStatus::QualifiedOfferRule
    } else if input.substantially_prevailed_on_amount_or_issue
        && !input.irs_position_substantially_justified
    {
        PrevailingPartyStatus::SubstantivelyPrevailed
    } else if input.substantially_prevailed_on_amount_or_issue
        && input.irs_position_substantially_justified
    {
        notes.push(
            "§ 7430(c)(4)(B) — IRS position substantially justified; taxpayer NOT treated as prevailing party despite substantively prevailing"
                .to_string(),
        );
        PrevailingPartyStatus::NotPrevailing
    } else {
        PrevailingPartyStatus::NotPrevailing
    };

    let hourly_rate_cap_cents = hourly_rate_cap_for_year(input.year_fees_incurred);
    notes.push(format!(
        "§ 7430(c)(1)(B)(iii) — attorney fee hourly cap for {} = {} cents (${:.2}/hour) per Rev. Proc. 2025-32",
        input.year_fees_incurred,
        hourly_rate_cap_cents,
        hourly_rate_cap_cents as f64 / 100.0
    ));

    let max_fees_awardable_cents = if matches!(prevailing_party_status, PrevailingPartyStatus::NotPrevailing)
        || !administrative_exhaustion_satisfied
    {
        0
    } else {
        hourly_rate_cap_cents
            .saturating_mul(input.attorney_hours_billed as i64)
    };

    Section7430Result {
        prevailing_party_status,
        net_worth_test_satisfied,
        employee_count_test_satisfied,
        administrative_exhaustion_satisfied,
        hourly_rate_cap_cents,
        max_attorney_fees_awardable_cents: max_fees_awardable_cents,
        citation: citation(),
        notes,
    }
}

fn hourly_rate_cap_for_year(year: u32) -> i64 {
    match year {
        2026 => 26_000,
        2025 => 25_000,
        2024 => 23_000,
        2023 => 23_000,
        y if y < 1996 => 12_500,
        _ => 26_000,
    }
}

fn citation() -> &'static str {
    "IRC § 7430(a)/(b)(1)/(c)(1)(A)/(c)(1)(B)(iii)/(c)(4)(A)/(B)/(C)/(D)/(c)(7); 28 U.S.C. § 2412(d)(2)(B); 26 CFR § 301.7430-2; Rev. Proc. 2025-32 (2026 hourly cap); IRS IRM 35.10.1"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section7430Input {
        Section7430Input {
            substantially_prevailed_on_amount_or_issue: true,
            irs_position_substantially_justified: false,
            administrative_remedies_exhausted: true,
            entity_type: EntityType::Individual,
            net_worth_cents: 50_000_000,
            employee_count_at_filing: 0,
            qualified_offer_made: false,
            qualified_offer_liability_cents: 0,
            judgment_liability_cents: 0,
            attorney_hours_billed: 100,
            year_fees_incurred: 2026,
        }
    }

    #[test]
    fn substantively_prevailed_individual_within_net_worth_awards_fees() {
        let r = compute(&base());
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::SubstantivelyPrevailed);
        assert!(r.net_worth_test_satisfied);
        assert!(r.employee_count_test_satisfied);
        assert!(r.administrative_exhaustion_satisfied);
        assert_eq!(r.hourly_rate_cap_cents, 26_000);
        assert_eq!(r.max_attorney_fees_awardable_cents, 26_000 * 100);
    }

    #[test]
    fn irs_position_substantially_justified_defeats_prevailing_party() {
        let mut i = base();
        i.irs_position_substantially_justified = true;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::NotPrevailing);
        assert_eq!(r.max_attorney_fees_awardable_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("§ 7430(c)(4)(B)") && n.contains("substantially justified")));
    }

    #[test]
    fn individual_net_worth_above_2m_disqualifies() {
        let mut i = base();
        i.net_worth_cents = 2_000_000_01;
        let r = compute(&i);
        assert!(!r.net_worth_test_satisfied);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::NotPrevailing);
    }

    #[test]
    fn individual_net_worth_at_2m_boundary_satisfied() {
        let mut i = base();
        i.net_worth_cents = 200_000_000;
        let r = compute(&i);
        assert!(r.net_worth_test_satisfied);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::SubstantivelyPrevailed);
    }

    #[test]
    fn business_entity_net_worth_7m_ceiling_uniquely_higher() {
        let mut i = base();
        i.entity_type = EntityType::BusinessEntity;
        i.net_worth_cents = 500_000_000;
        let r = compute(&i);
        assert!(r.net_worth_test_satisfied);
    }

    #[test]
    fn business_entity_net_worth_above_7m_disqualifies() {
        let mut i = base();
        i.entity_type = EntityType::BusinessEntity;
        i.net_worth_cents = 7_000_000_01;
        let r = compute(&i);
        assert!(!r.net_worth_test_satisfied);
    }

    #[test]
    fn employee_count_above_500_disqualifies_regardless_of_net_worth() {
        let mut i = base();
        i.entity_type = EntityType::BusinessEntity;
        i.net_worth_cents = 10_000_000;
        i.employee_count_at_filing = 501;
        let r = compute(&i);
        assert!(!r.employee_count_test_satisfied);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::NotPrevailing);
    }

    #[test]
    fn employee_count_at_500_satisfied() {
        let mut i = base();
        i.employee_count_at_filing = 500;
        let r = compute(&i);
        assert!(r.employee_count_test_satisfied);
    }

    #[test]
    fn administrative_remedies_not_exhausted_zeros_fee_award() {
        let mut i = base();
        i.administrative_remedies_exhausted = false;
        let r = compute(&i);
        assert!(!r.administrative_exhaustion_satisfied);
        assert_eq!(r.max_attorney_fees_awardable_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("§ 7430(b)(1)") && n.contains("EXHAUSTED")));
    }

    #[test]
    fn qualified_offer_engages_when_qo_liability_geq_judgment() {
        let mut i = base();
        i.qualified_offer_made = true;
        i.qualified_offer_liability_cents = 10_000_000;
        i.judgment_liability_cents = 10_000_000;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::QualifiedOfferRule);
        assert!(r.notes.iter().any(|n| n.contains("§ 7430(c)(7)") && n.contains("≥")));
    }

    #[test]
    fn qualified_offer_does_not_engage_when_qo_liability_less_than_judgment() {
        let mut i = base();
        i.substantially_prevailed_on_amount_or_issue = false;
        i.qualified_offer_made = true;
        i.qualified_offer_liability_cents = 5_000_000;
        i.judgment_liability_cents = 10_000_000;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::NotPrevailing);
        assert!(r.notes.iter().any(|n| n.contains("§ 7430(c)(7)") && n.contains("does NOT engage")));
    }

    #[test]
    fn qualified_offer_rule_bypasses_substantial_justification_defense() {
        let mut i = base();
        i.irs_position_substantially_justified = true;
        i.qualified_offer_made = true;
        i.qualified_offer_liability_cents = 10_000_000;
        i.judgment_liability_cents = 10_000_000;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::QualifiedOfferRule);
    }

    #[test]
    fn hourly_cap_2026_is_260_dollars() {
        let r = compute(&base());
        assert_eq!(r.hourly_rate_cap_cents, 26_000);
    }

    #[test]
    fn hourly_cap_2025_is_250_dollars() {
        let mut i = base();
        i.year_fees_incurred = 2025;
        let r = compute(&i);
        assert_eq!(r.hourly_rate_cap_cents, 25_000);
    }

    #[test]
    fn hourly_cap_pre_1996_falls_back_to_125_base() {
        let mut i = base();
        i.year_fees_incurred = 1995;
        let r = compute(&i);
        assert_eq!(r.hourly_rate_cap_cents, 12_500);
    }

    #[test]
    fn max_fees_calculation_hourly_cap_times_hours() {
        let mut i = base();
        i.attorney_hours_billed = 50;
        let r = compute(&i);
        assert_eq!(r.max_attorney_fees_awardable_cents, 26_000 * 50);
    }

    #[test]
    fn citation_pins_all_subsections_and_2025_32() {
        let r = compute(&base());
        assert!(r.citation.contains("§ 7430(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(c)(1)(A)"));
        assert!(r.citation.contains("(c)(1)(B)(iii)"));
        assert!(r.citation.contains("(c)(4)(A)/(B)/(C)/(D)"));
        assert!(r.citation.contains("(c)(7)"));
        assert!(r.citation.contains("28 U.S.C. § 2412(d)(2)(B)"));
        assert!(r.citation.contains("§ 301.7430-2"));
        assert!(r.citation.contains("Rev. Proc. 2025-32"));
        assert!(r.citation.contains("IRM 35.10.1"));
    }

    #[test]
    fn business_entity_7m_uniquely_higher_than_individual_2m_invariant() {
        let mut i_ind = base();
        i_ind.entity_type = EntityType::Individual;
        i_ind.net_worth_cents = 300_000_000;

        let mut i_biz = base();
        i_biz.entity_type = EntityType::BusinessEntity;
        i_biz.net_worth_cents = 300_000_000;

        let r_ind = compute(&i_ind);
        let r_biz = compute(&i_biz);

        assert!(!r_ind.net_worth_test_satisfied, "individual $3M exceeds $2M");
        assert!(r_biz.net_worth_test_satisfied, "business $3M within $7M");
    }

    #[test]
    fn note_describes_2026_hourly_cap() {
        let r = compute(&base());
        assert!(r.notes.iter().any(|n| n.contains("2026") && n.contains("$260.00/hour")));
    }

    #[test]
    fn substantively_prevailed_but_not_exhausted_zeros_fees() {
        let mut i = base();
        i.administrative_remedies_exhausted = false;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::SubstantivelyPrevailed);
        assert_eq!(r.max_attorney_fees_awardable_cents, 0);
    }

    #[test]
    fn qualified_offer_rule_only_engaged_when_offer_made() {
        let mut i = base();
        i.qualified_offer_made = false;
        i.qualified_offer_liability_cents = 10_000_000;
        i.judgment_liability_cents = 10_000_000;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::SubstantivelyPrevailed);
    }

    #[test]
    fn employee_count_at_501_uniquely_disqualifies() {
        let mut i = base();
        i.employee_count_at_filing = 501;
        let r = compute(&i);
        assert!(!r.employee_count_test_satisfied);
    }

    #[test]
    fn negative_net_worth_satisfies_test() {
        let mut i = base();
        i.net_worth_cents = -100_000_000;
        let r = compute(&i);
        assert!(r.net_worth_test_satisfied);
    }

    #[test]
    fn qualified_offer_exactly_at_judgment_engages_rule() {
        let mut i = base();
        i.substantially_prevailed_on_amount_or_issue = false;
        i.qualified_offer_made = true;
        i.qualified_offer_liability_cents = 7_500_000;
        i.judgment_liability_cents = 7_500_000;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::QualifiedOfferRule);
    }

    #[test]
    fn qualified_offer_one_cent_below_judgment_does_not_engage() {
        let mut i = base();
        i.substantially_prevailed_on_amount_or_issue = false;
        i.qualified_offer_made = true;
        i.qualified_offer_liability_cents = 7_500_000 - 1;
        i.judgment_liability_cents = 7_500_000;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::NotPrevailing);
    }

    #[test]
    fn not_prevailing_zeros_fee_award() {
        let mut i = base();
        i.substantially_prevailed_on_amount_or_issue = false;
        let r = compute(&i);
        assert_eq!(r.prevailing_party_status, PrevailingPartyStatus::NotPrevailing);
        assert_eq!(r.max_attorney_fees_awardable_cents, 0);
    }

    #[test]
    fn three_gating_tests_all_must_pass_for_fee_award_invariant() {
        let base_i = base();
        let mut i_net = base_i.clone();
        i_net.net_worth_cents = 300_000_000;
        let r_net = compute(&i_net);
        assert_eq!(r_net.max_attorney_fees_awardable_cents, 0);

        let mut i_emp = base_i.clone();
        i_emp.employee_count_at_filing = 501;
        let r_emp = compute(&i_emp);
        assert_eq!(r_emp.max_attorney_fees_awardable_cents, 0);

        let mut i_exh = base_i.clone();
        i_exh.administrative_remedies_exhausted = false;
        let r_exh = compute(&i_exh);
        assert_eq!(r_exh.max_attorney_fees_awardable_cents, 0);

        let r_base = compute(&base_i);
        assert!(r_base.max_attorney_fees_awardable_cents > 0);
    }
}
