//! IRC § 709 — Treatment of organization and syndication fees
//! (partnership). Parallel to § 195 (startup expenditures, built
//! iter 270) and § 248 (corporate organizational, built iter 296)
//! with partnership-specific terminology and the distinctive
//! § 709(b)(3) SYNDICATION EXPENSE carve-out — syndication
//! expenses are PERMANENTLY CAPITALIZED with NO amortization.
//!
//! Trader-relevant when forming a partnership for trading
//! operations (LLP, LP, LLC taxed as partnership). § 709 covers
//! ORGANIZATIONAL expenses (incident to creating the partnership);
//! § 709(b)(3) syndication expenses (raising capital from
//! investors) follow a different treatment — never deductible.
//!
//! § 709(a) GENERAL RULE — except as provided in § 709(b), no
//! deduction shall be allowed for amounts paid or incurred to
//! organize a partnership or to promote the sale of, or to sell,
//! an interest in such partnership.
//!
//! § 709(b)(1) ELECTION — at the election of the partnership,
//! organizational expenses may be treated as deferred expenses
//! and allowed as a deduction:
//!   (A) in the taxable year in which the partnership begins
//!       business, in an amount equal to the LESSER OF (i) the
//!       amount of organizational expenses, or (ii) $5,000,
//!       reduced (but NOT BELOW ZERO) by the amount by which such
//!       organizational expenses exceed $50,000; AND
//!   (B) the REMAINDER ratably over the 180-month period
//!       beginning with the month in which the partnership begins
//!       business.
//!
//! § 709(b)(2) ORGANIZATIONAL EXPENSES DEFINED — any expense
//! which is (A) incident to the creation of the partnership;
//! (B) chargeable to capital account; and (C) of a character
//! which, if expended incident to the creation of a partnership
//! having an ascertainable life, would be amortizable over such
//! life.
//!
//! § 709(b)(3) — SYNDICATION EXPENSES NOT AMORTIZABLE — paragraph
//! (1) shall not apply to any amount paid or incurred to PROMOTE
//! THE SALE OF, OR TO SELL, an interest in the partnership.
//! Treas. Reg. § 1.709-2(b) defines syndication expenses to
//! include: brokerage fees, registration fees, legal fees of the
//! issuer, accounting fees of the issuer, and printing costs of
//! the prospectus. Syndication expenses are PERMANENTLY CAPITALIZED
//! to partner basis with NO amortization deduction available —
//! distinct from § 248 corporate organizational which only
//! excludes share-issuance expenses without the syndication
//! treatment.
//!
//! AUTOMATIC ELECTION via T.D. 9542 (September 8, 2011) — same as
//! § 195(d) and § 248(c) — partnership deemed to elect § 709(b)
//! deduction unless it affirmatively elects to capitalize.
//!
//! Citations: IRC § 709(a) (general no-deduction rule); § 709(b)(1)
//! (election + $5K cap + phase-out + 180-month amortization);
//! § 709(b)(2) (organizational expense definition); § 709(b)(3)
//! (syndication expenses not amortizable); Treas. Reg. § 1.709-1
//! (treatment of organization and syndication costs); Treas. Reg.
//! § 1.709-2(a) (organizational expense definition); Treas. Reg.
//! § 1.709-2(b) (syndication expense definition — brokerage / reg /
//! legal / accounting / prospectus); T.D. 9542 (Sept. 8, 2011 —
//! automatic election); AJCA 2004 § 902 (harmonized § 709 with
//! § 195 / § 248); cross-reference § 195 (startup expenditures)
//! and § 248 (corporate organizational expenditures).

use serde::{Deserialize, Serialize};

pub const PHASE_OUT_FLOOR_CENTS: i64 = 5_000_000;
pub const FIRST_YEAR_DEDUCTION_CAP_CENTS: i64 = 500_000;
pub const PHASE_OUT_CEILING_CENTS: i64 = 5_500_000;
pub const AMORTIZATION_MONTHS: i64 = 180;

#[derive(Debug, Clone, Deserialize)]
pub struct Section709Input {
    /// Total aggregate ORGANIZATIONAL expenses in cents. Must
    /// EXCLUDE syndication expenses (which are passed separately).
    pub total_organizational_expenses_cents: i64,
    /// Total syndication expenses in cents (brokerage fees,
    /// registration fees, legal/accounting fees for prospectus,
    /// printing costs). § 709(b)(3) — PERMANENTLY CAPITALIZED with
    /// NO amortization deduction available.
    pub total_syndication_expenses_cents: i64,
    /// Months remaining in the partnership's first taxable year
    /// after the month the partnership began business. 0-12
    /// inclusive.
    pub months_active_in_first_year: u32,
    /// Whether the partnership affirmatively elected to capitalize
    /// under § 709 (rare — automatic election under T.D. 9542 is
    /// the default).
    pub affirmative_capitalization_election: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section709Result {
    pub election_applies: bool,
    pub first_year_immediate_deduction_cents: i64,
    pub amortization_pool_cents: i64,
    pub monthly_amortization_cents: i64,
    pub first_year_amortization_cents: i64,
    pub first_year_total_deduction_cents: i64,
    pub phase_out_reduction_cents: i64,
    pub syndication_expenses_permanently_capitalized_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section709Input) -> Section709Result {
    let mut notes: Vec<String> = Vec::new();
    let organizational = input.total_organizational_expenses_cents.max(0);
    let syndication = input.total_syndication_expenses_cents.max(0);

    if syndication > 0 {
        notes.push(format!(
            "§ 709(b)(3) — syndication expenses {} cents PERMANENTLY CAPITALIZED with NO amortization deduction; capitalized to partner basis instead",
            syndication
        ));
        notes.push(
            "Treas. Reg. § 1.709-2(b) — syndication expenses = brokerage fees + registration fees + legal/accounting fees for prospectus + printing costs of prospectus"
                .to_string(),
        );
    }

    if input.affirmative_capitalization_election {
        notes.push(
            "partnership affirmatively elected to capitalize under § 709 — no first-year deduction, no amortization; capitalized to partner basis"
                .to_string(),
        );
        return Section709Result {
            election_applies: false,
            first_year_immediate_deduction_cents: 0,
            amortization_pool_cents: 0,
            monthly_amortization_cents: 0,
            first_year_amortization_cents: 0,
            first_year_total_deduction_cents: 0,
            phase_out_reduction_cents: 0,
            syndication_expenses_permanently_capitalized_cents: syndication,
            citation: citation(),
            notes,
        };
    }

    let phase_out_reduction = (organizational - PHASE_OUT_FLOOR_CENTS).max(0);
    let raw_cap = (FIRST_YEAR_DEDUCTION_CAP_CENTS - phase_out_reduction).max(0);
    let first_year_immediate = organizational.min(raw_cap);

    if organizational >= PHASE_OUT_CEILING_CENTS {
        notes.push(
            "total organizational expenses at or above $55,000 — first-year deduction fully phased out under § 709(b)(1)(A); 100% flows to 180-month amortization stream"
                .to_string(),
        );
    } else if organizational > PHASE_OUT_FLOOR_CENTS {
        notes.push(format!(
            "phase-out engaged — organizational expenses exceed $50,000 by {} cents; first-year cap reduced from $5,000 to {} cents",
            phase_out_reduction, raw_cap
        ));
    }

    let amortization_pool = organizational - first_year_immediate;
    let monthly_amortization = if amortization_pool > 0 {
        amortization_pool / AMORTIZATION_MONTHS
    } else {
        0
    };

    let months_active = (input.months_active_in_first_year.min(12)) as i64;
    let first_year_amortization = monthly_amortization.saturating_mul(months_active);

    if months_active > 0 && amortization_pool > 0 {
        notes.push(format!(
            "remaining {} cents amortized over 180 months starting month partnership begins business; first-year ratable portion = {} months × {} cents/month = {} cents",
            amortization_pool, months_active, monthly_amortization, first_year_amortization
        ));
    }

    let first_year_total = first_year_immediate.saturating_add(first_year_amortization);

    notes.push(
        "T.D. 9542 (Sept. 8, 2011) — automatic election under § 709(b) (parallel to § 195(d) and § 248(c))"
            .to_string(),
    );
    notes.push(
        "Treas. Reg. § 1.709-2(a) — organizational expense = incident to creation of partnership + chargeable to capital account + amortizable-character"
            .to_string(),
    );

    Section709Result {
        election_applies: true,
        first_year_immediate_deduction_cents: first_year_immediate,
        amortization_pool_cents: amortization_pool,
        monthly_amortization_cents: monthly_amortization,
        first_year_amortization_cents: first_year_amortization,
        first_year_total_deduction_cents: first_year_total,
        phase_out_reduction_cents: phase_out_reduction,
        syndication_expenses_permanently_capitalized_cents: syndication,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 709(a)/(b)(1)/(b)(2)/(b)(3); Treas. Reg. § 1.709-1/§ 1.709-2(a)/(b); T.D. 9542 (Sept. 8, 2011); AJCA 2004 § 902; cross-reference § 195 (startup) + § 248 (corporate organizational)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        organizational_dollars: i64,
        syndication_dollars: i64,
        months_active: u32,
    ) -> Section709Input {
        Section709Input {
            total_organizational_expenses_cents: organizational_dollars * 100,
            total_syndication_expenses_cents: syndication_dollars * 100,
            months_active_in_first_year: months_active,
            affirmative_capitalization_election: false,
        }
    }

    #[test]
    fn small_organizational_under_5k_full_first_year() {
        let r = compute(&input(3_000, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 300_000);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.syndication_expenses_permanently_capitalized_cents, 0);
    }

    #[test]
    fn ten_k_organizational_5k_immediate_plus_180_month_amortization() {
        let r = compute(&input(10_000, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert_eq!(r.amortization_pool_cents, 500_000);
        assert_eq!(r.monthly_amortization_cents, 500_000 / 180);
    }

    #[test]
    fn fifty_k_organizational_at_phase_out_floor_5k_immediate() {
        let r = compute(&input(50_000, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert_eq!(r.amortization_pool_cents, 4_500_000);
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn fifty_three_k_organizational_3k_phase_out_reduces_immediate_to_2k() {
        let r = compute(&input(53_000, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 200_000);
        assert_eq!(r.phase_out_reduction_cents, 300_000);
    }

    #[test]
    fn fifty_five_k_organizational_full_phase_out() {
        let r = compute(&input(55_000, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 5_500_000);
    }

    #[test]
    fn syndication_expenses_permanently_capitalized() {
        let r = compute(&input(10_000, 50_000, 12));
        assert_eq!(
            r.syndication_expenses_permanently_capitalized_cents,
            5_000_000
        );
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 709(b)(3)") && n.contains("PERMANENTLY CAPITALIZED")));
    }

    #[test]
    fn syndication_expenses_treasury_reg_definition_note() {
        let r = compute(&input(10_000, 25_000, 12));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1.709-2(b)") && n.contains("brokerage fees")));
    }

    #[test]
    fn syndication_expenses_separately_capitalized_from_organizational_amortization() {
        let r = compute(&input(10_000, 100_000, 12));
        assert_eq!(r.amortization_pool_cents, 500_000);
        assert_eq!(
            r.syndication_expenses_permanently_capitalized_cents,
            10_000_000
        );
        assert!(r.amortization_pool_cents < r.syndication_expenses_permanently_capitalized_cents);
    }

    #[test]
    fn no_syndication_no_syndication_note() {
        let r = compute(&input(10_000, 0, 12));
        let synd_notes: Vec<_> = r
            .notes
            .iter()
            .filter(|n| n.contains("§ 709(b)(3)"))
            .collect();
        assert!(synd_notes.is_empty());
    }

    #[test]
    fn affirmative_capitalization_election_zeros_organizational_deductions() {
        let mut i = input(10_000, 0, 12);
        i.affirmative_capitalization_election = true;
        let r = compute(&i);
        assert!(!r.election_applies);
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 0);
    }

    #[test]
    fn affirmative_capitalization_election_still_capitalizes_syndication() {
        let mut i = input(10_000, 25_000, 12);
        i.affirmative_capitalization_election = true;
        let r = compute(&i);
        assert!(!r.election_applies);
        assert_eq!(
            r.syndication_expenses_permanently_capitalized_cents,
            2_500_000
        );
    }

    #[test]
    fn months_active_caps_at_12() {
        let r = compute(&input(10_000, 0, 13));
        let expected_monthly = 500_000 / 180;
        assert_eq!(r.first_year_amortization_cents, expected_monthly * 12);
    }

    #[test]
    fn months_active_zero_no_amortization() {
        let r = compute(&input(10_000, 0, 0));
        assert_eq!(r.first_year_amortization_cents, 0);
        assert_eq!(r.first_year_total_deduction_cents, 500_000);
    }

    #[test]
    fn phase_out_boundary_50k_to_51k_one_dollar_reduction() {
        let r50 = compute(&input(50_000, 0, 12));
        let r51 = compute(&input(51_000, 0, 12));
        assert_eq!(r50.phase_out_reduction_cents, 0);
        assert_eq!(r51.phase_out_reduction_cents, 100_000);
        assert_eq!(
            r51.first_year_immediate_deduction_cents,
            r50.first_year_immediate_deduction_cents - 100_000
        );
    }

    #[test]
    fn phase_out_completes_exactly_at_55k() {
        let r55_000 = compute(&input(55_000, 0, 12));
        assert_eq!(r55_000.first_year_immediate_deduction_cents, 0);
    }

    #[test]
    fn citation_pins_all_subsections_and_treasury_regs() {
        let r = compute(&input(10_000, 0, 12));
        assert!(r.citation.contains("§ 709(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(b)(2)"));
        assert!(r.citation.contains("(b)(3)"));
        assert!(r.citation.contains("§ 1.709-1"));
        assert!(r.citation.contains("§ 1.709-2(a)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("T.D. 9542"));
        assert!(r.citation.contains("Sept. 8, 2011"));
        assert!(r.citation.contains("AJCA 2004 § 902"));
        assert!(r.citation.contains("§ 195"));
        assert!(r.citation.contains("§ 248"));
    }

    #[test]
    fn zero_organizational_zero_deduction() {
        let r = compute(&input(0, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 0);
    }

    #[test]
    fn negative_organizational_clamped_to_zero() {
        let i = Section709Input {
            total_organizational_expenses_cents: -100_000,
            total_syndication_expenses_cents: 0,
            months_active_in_first_year: 12,
            affirmative_capitalization_election: false,
        };
        let r = compute(&i);
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
    }

    #[test]
    fn negative_syndication_clamped_to_zero() {
        let i = Section709Input {
            total_organizational_expenses_cents: 10_000_00,
            total_syndication_expenses_cents: -100_000,
            months_active_in_first_year: 12,
            affirmative_capitalization_election: false,
        };
        let r = compute(&i);
        assert_eq!(r.syndication_expenses_permanently_capitalized_cents, 0);
    }

    #[test]
    fn one_million_organizational_full_phase_out_amortizes_over_15_years() {
        let r = compute(&input(1_000_000, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 100_000_000);
        assert_eq!(
            r.monthly_amortization_cents,
            100_000_000 / AMORTIZATION_MONTHS
        );
    }

    #[test]
    fn constants_pin_50k_5k_55k_180() {
        assert_eq!(PHASE_OUT_FLOOR_CENTS, 5_000_000);
        assert_eq!(FIRST_YEAR_DEDUCTION_CAP_CENTS, 500_000);
        assert_eq!(PHASE_OUT_CEILING_CENTS, 5_500_000);
        assert_eq!(AMORTIZATION_MONTHS, 180);
    }

    #[test]
    fn section_709_matches_section_195_section_248_phaseout_invariant() {
        let r = compute(&input(53_000, 0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 200_000);
    }

    #[test]
    fn syndication_distinct_from_section_248_share_issuance_invariant() {
        let r = compute(&input(10_000, 25_000, 12));
        assert_eq!(
            r.syndication_expenses_permanently_capitalized_cents,
            2_500_000
        );
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("brokerage fees + registration fees")));
    }

    #[test]
    fn election_applies_default_path_true() {
        let r = compute(&input(10_000, 0, 12));
        assert!(r.election_applies);
    }

    #[test]
    fn automatic_election_note_present() {
        let r = compute(&input(10_000, 0, 12));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("T.D. 9542") && n.contains("automatic election")));
    }

    #[test]
    fn organizational_expense_definition_note_present() {
        let r = compute(&input(10_000, 0, 12));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1.709-2(a)") && n.contains("incident to creation")));
    }

    #[test]
    fn syndication_capitalized_amount_returned_in_result_field() {
        let r = compute(&input(10_000, 75_000, 12));
        assert_eq!(
            r.syndication_expenses_permanently_capitalized_cents,
            7_500_000
        );
    }

    #[test]
    fn first_year_total_equals_immediate_plus_amortization() {
        let r = compute(&input(10_000, 0, 6));
        assert_eq!(
            r.first_year_total_deduction_cents,
            r.first_year_immediate_deduction_cents + r.first_year_amortization_cents
        );
    }

    #[test]
    fn syndication_amount_independent_of_phase_out_calculation() {
        let r_low_synd = compute(&input(53_000, 1_000, 12));
        let r_high_synd = compute(&input(53_000, 100_000, 12));
        assert_eq!(
            r_low_synd.first_year_immediate_deduction_cents,
            r_high_synd.first_year_immediate_deduction_cents,
            "syndication does not affect organizational phase-out"
        );
        assert_eq!(
            r_low_synd.phase_out_reduction_cents,
            r_high_synd.phase_out_reduction_cents
        );
    }

    #[test]
    fn phase_out_engagement_note_appears_above_50k_below_55k() {
        let r = compute(&input(52_500, 0, 12));
        assert!(r.notes.iter().any(|n| n.contains("phase-out engaged")));
    }

    #[test]
    fn cross_reference_to_section_195_and_248_in_citation() {
        let r = compute(&input(10_000, 0, 12));
        assert!(r.citation.contains("§ 195"));
        assert!(r.citation.contains("§ 248"));
    }
}
