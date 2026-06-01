//! IRC § 248 — Organizational expenditures (corporate). Parallel
//! to § 195 (startup expenditures, built iter 270). Trader-relevant
//! when forming a C-corporation for trading operations (incidental
//! to incorporation: drafting charter, bylaws, minutes; legal /
//! accounting services for organization; expenses of organizational
//! meetings; fees paid to state of incorporation).
//!
//! Distinct from `section_195` (operating startup expenditures —
//! pre-launch operating costs of an active trade or business) and
//! § 709 (parallel partnership organizational expenditures). § 248
//! covers ONLY CORPORATE FORMATION costs incident to creating the
//! corporation.
//!
//! § 248(a) ELECTION — if a corporation elects the application of
//! this subsection with respect to organizational expenditures, the
//! corporation shall be allowed a deduction for the taxable year in
//! which the corporation begins business in an amount equal to the
//! LESSER OF (i) the amount of organizational expenditures, OR (ii)
//! $5,000, reduced (but NOT BELOW ZERO) by the amount by which such
//! organizational expenditures exceed $50,000.
//!
//! § 248(a) AMORTIZATION OF REMAINDER — the remainder of such
//! organizational expenditures shall be allowed as a deduction
//! RATABLY OVER THE 180-MONTH PERIOD beginning with the month in
//! which the corporation begins business.
//!
//! § 248(b) ORGANIZATIONAL EXPENDITURE DEFINED — any expenditure
//! which (1) is incident to the creation of the corporation, (2) is
//! chargeable to capital account, and (3) is of a character which,
//! if expended incident to the creation of a corporation having a
//! limited life, would be amortizable over such life. Excludes
//! expenses for issuing or selling shares of stock (§ 248(b)
//! exclusion via Treas. Reg. § 1.248-1(b)).
//!
//! Typical § 248 organizational expenditures: legal services for
//! drafting charter and bylaws; necessary accounting services
//! incident to organization; expenses of temporary directors and
//! organizational meetings; fees paid to the state of incorporation.
//!
//! § 248 EXCLUSIONS (Treas. Reg. § 1.248-1(b)): expenses for issuing
//! or selling shares of stock (capitalized to stock basis); expenses
//! connected with the transfer of assets to a corporation (§ 351
//! organization); expenses connected with reorganization (§ 368).
//!
//! AUTOMATIC ELECTION via T.D. 9542 (September 8, 2011) — corporation
//! is DEEMED to have elected § 248(a) unless it affirmatively elects
//! to capitalize. Same automatic-election mechanic as § 195(d) and
//! § 709(b). 26 CFR § 1.248-1(c) implementing rule.
//!
//! AJCA 2004 § 902 amendment harmonized § 248 with § 195 and § 709 —
//! prior § 248 permitted only 60-month amortization election; current
//! framework matches § 195 / § 709.
//!
//! Citations: IRC § 248(a) (election + $5K cap + phase-out + 180-
//! month amortization); § 248(b) (organizational expenditure
//! definition); Treas. Reg. § 1.248-1(b) (definition and exclusions);
//! 26 CFR § 1.248-1(c) (automatic election); T.D. 9542 (Sept. 8,
//! 2011 — automatic election regulation); AJCA 2004 § 902 (harmonized
//! with § 195 / § 709); cross-reference § 195 (startup expenditures)
//! and § 709 (partnership organizational expenditures).

use serde::{Deserialize, Serialize};

pub const PHASE_OUT_FLOOR_CENTS: i64 = 5_000_000;
pub const FIRST_YEAR_DEDUCTION_CAP_CENTS: i64 = 500_000;
pub const PHASE_OUT_CEILING_CENTS: i64 = 5_500_000;
pub const AMORTIZATION_MONTHS: i64 = 180;

#[derive(Debug, Clone, Deserialize)]
pub struct Section248Input {
    /// Total aggregate organizational expenditures in cents.
    /// Must EXCLUDE expenses for issuing or selling shares of
    /// stock (Treas. Reg. § 1.248-1(b)) and § 351/§ 368 transfer
    /// expenses.
    pub total_organizational_expenditures_cents: i64,
    /// Months remaining in the corporation's first taxable year
    /// after the month the corporation began business. 0-12
    /// inclusive; 12 = began business in first month of taxable
    /// year, 0 = began business in last month.
    pub months_active_in_first_year: u32,
    /// Whether the corporation affirmatively elected to capitalize
    /// under § 248(c) (rare — automatic election is the default
    /// per T.D. 9542).
    pub affirmative_capitalization_election: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section248Result {
    pub election_applies: bool,
    pub first_year_immediate_deduction_cents: i64,
    pub amortization_pool_cents: i64,
    pub monthly_amortization_cents: i64,
    pub first_year_amortization_cents: i64,
    pub first_year_total_deduction_cents: i64,
    pub phase_out_reduction_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section248Input) -> Section248Result {
    let mut notes: Vec<String> = Vec::new();
    let total = input.total_organizational_expenditures_cents.max(0);

    if input.affirmative_capitalization_election {
        notes.push(
            "corporation affirmatively elected to capitalize under § 248(c) — no first-year deduction, no amortization; basis in stock instead"
                .to_string(),
        );
        return Section248Result {
            election_applies: false,
            first_year_immediate_deduction_cents: 0,
            amortization_pool_cents: 0,
            monthly_amortization_cents: 0,
            first_year_amortization_cents: 0,
            first_year_total_deduction_cents: 0,
            phase_out_reduction_cents: 0,
            citation: citation(),
            notes,
        };
    }

    let phase_out_reduction = (total - PHASE_OUT_FLOOR_CENTS).max(0);
    let raw_cap = (FIRST_YEAR_DEDUCTION_CAP_CENTS - phase_out_reduction).max(0);
    let first_year_immediate = total.min(raw_cap);

    if total >= PHASE_OUT_CEILING_CENTS {
        notes.push(
            "total organizational expenditures at or above $55,000 — first-year deduction fully phased out under § 248(a); 100% flows to 180-month amortization stream"
                .to_string(),
        );
    } else if total > PHASE_OUT_FLOOR_CENTS {
        notes.push(format!(
            "phase-out engaged — organizational costs exceed $50,000 by {} cents; first-year cap reduced from $5,000 to {} cents",
            phase_out_reduction, raw_cap
        ));
    }

    let amortization_pool = total - first_year_immediate;
    let monthly_amortization = if amortization_pool > 0 {
        amortization_pool / AMORTIZATION_MONTHS
    } else {
        0
    };

    let months_active = (input.months_active_in_first_year.min(12)) as i64;
    let first_year_amortization = monthly_amortization.saturating_mul(months_active);

    if months_active > 0 && amortization_pool > 0 {
        notes.push(format!(
            "remaining {} cents amortized over 180 months starting month corporation begins business; first-year ratable portion = {} months × {} cents/month = {} cents",
            amortization_pool, months_active, monthly_amortization, first_year_amortization
        ));
    }

    let first_year_total = first_year_immediate.saturating_add(first_year_amortization);

    notes.push(
        "Treas. Reg. § 1.248-1(b) — § 248 EXCLUDES expenses for issuing or selling shares of stock + § 351 transfer expenses + § 368 reorganization expenses"
            .to_string(),
    );
    notes.push(
        "T.D. 9542 (Sept. 8, 2011) — automatic election under § 248(a); to opt OUT and capitalize, corporation must affirmatively elect to capitalize"
            .to_string(),
    );

    Section248Result {
        election_applies: true,
        first_year_immediate_deduction_cents: first_year_immediate,
        amortization_pool_cents: amortization_pool,
        monthly_amortization_cents: monthly_amortization,
        first_year_amortization_cents: first_year_amortization,
        first_year_total_deduction_cents: first_year_total,
        phase_out_reduction_cents: phase_out_reduction,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 248(a)/(b); Treas. Reg. § 1.248-1(b)/(c); T.D. 9542 (Sept. 8, 2011); AJCA 2004 § 902; cross-reference § 195 (startup expenditures) + § 709 (partnership organizational expenditures)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(total_dollars: i64, months_active: u32) -> Section248Input {
        Section248Input {
            total_organizational_expenditures_cents: total_dollars * 100,
            months_active_in_first_year: months_active,
            affirmative_capitalization_election: false,
        }
    }

    #[test]
    fn small_organizational_under_5k_full_first_year_no_amortization() {
        let r = compute(&input(3_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 3_000 * 100);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.monthly_amortization_cents, 0);
        assert_eq!(r.first_year_amortization_cents, 0);
        assert_eq!(r.first_year_total_deduction_cents, 3_000 * 100);
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn five_k_organizational_full_first_year_no_amortization() {
        let r = compute(&input(5_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn ten_k_organizational_5k_immediate_plus_180_month_amortization() {
        let r = compute(&input(10_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert_eq!(r.amortization_pool_cents, 500_000);
        assert_eq!(r.monthly_amortization_cents, 500_000 / 180);
        assert_eq!(
            r.first_year_amortization_cents,
            (500_000 / 180) * 12
        );
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn fifty_k_organizational_at_phase_out_floor_5k_immediate() {
        let r = compute(&input(50_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert_eq!(r.amortization_pool_cents, 4_500_000);
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn fifty_one_k_organizational_1k_phase_out_reduces_immediate_to_4k() {
        let r = compute(&input(51_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 400_000);
        assert_eq!(r.amortization_pool_cents, 4_700_000);
        assert_eq!(r.phase_out_reduction_cents, 100_000);
    }

    #[test]
    fn fifty_three_k_organizational_3k_phase_out_reduces_immediate_to_2k() {
        let r = compute(&input(53_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 200_000);
        assert_eq!(r.amortization_pool_cents, 5_100_000);
        assert_eq!(r.phase_out_reduction_cents, 300_000);
    }

    #[test]
    fn fifty_five_k_organizational_full_phase_out_no_immediate_deduction() {
        let r = compute(&input(55_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 5_500_000);
        assert_eq!(r.phase_out_reduction_cents, 500_000);
        assert!(r.notes.iter().any(|n| n.contains("fully phased out")));
    }

    #[test]
    fn sixty_k_organizational_full_phase_out() {
        let r = compute(&input(60_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 6_000_000);
        assert_eq!(r.phase_out_reduction_cents, 1_000_000);
    }

    #[test]
    fn one_hundred_k_organizational_full_phase_out() {
        let r = compute(&input(100_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 10_000_000);
        assert_eq!(r.monthly_amortization_cents, 10_000_000 / 180);
    }

    #[test]
    fn months_active_in_first_year_caps_at_twelve() {
        let mut i = input(10_000, 13);
        i.months_active_in_first_year = 13;
        let r = compute(&i);
        let expected_monthly = 500_000 / 180;
        assert_eq!(r.first_year_amortization_cents, expected_monthly * 12);
    }

    #[test]
    fn months_active_six_proportional_amortization() {
        let r = compute(&input(10_000, 6));
        let expected_monthly = 500_000 / 180;
        assert_eq!(r.first_year_amortization_cents, expected_monthly * 6);
    }

    #[test]
    fn months_active_zero_no_amortization() {
        let r = compute(&input(10_000, 0));
        assert_eq!(r.first_year_amortization_cents, 0);
        assert_eq!(r.first_year_total_deduction_cents, 500_000);
    }

    #[test]
    fn affirmative_capitalization_election_zeros_all_deductions() {
        let mut i = input(10_000, 12);
        i.affirmative_capitalization_election = true;
        let r = compute(&i);
        assert!(!r.election_applies);
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.first_year_total_deduction_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("affirmatively elected to capitalize")));
    }

    #[test]
    fn phase_out_threshold_boundary_50k_zero_dollar_one_above_floor() {
        let r50 = compute(&input(50_000, 12));
        let r51 = compute(&input(51_000, 12));
        assert_eq!(r50.phase_out_reduction_cents, 0);
        assert_eq!(r51.phase_out_reduction_cents, 100_000);
        assert_eq!(
            r51.first_year_immediate_deduction_cents,
            r50.first_year_immediate_deduction_cents - 100_000
        );
    }

    #[test]
    fn phase_out_completes_exactly_at_55k() {
        let r54_999 = compute(&Section248Input {
            total_organizational_expenditures_cents: 5_499_900,
            months_active_in_first_year: 12,
            affirmative_capitalization_election: false,
        });
        let r55_000 = compute(&input(55_000, 12));
        assert_eq!(r54_999.first_year_immediate_deduction_cents, 100);
        assert_eq!(r55_000.first_year_immediate_deduction_cents, 0);
    }

    #[test]
    fn first_year_total_equals_immediate_plus_amortization() {
        let r = compute(&input(10_000, 6));
        assert_eq!(
            r.first_year_total_deduction_cents,
            r.first_year_immediate_deduction_cents + r.first_year_amortization_cents
        );
    }

    #[test]
    fn citation_pins_subsections_and_treasury_regs() {
        let r = compute(&input(10_000, 12));
        assert!(r.citation.contains("§ 248(a)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("§ 1.248-1(b)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("T.D. 9542"));
        assert!(r.citation.contains("Sept. 8, 2011"));
        assert!(r.citation.contains("AJCA 2004 § 902"));
        assert!(r.citation.contains("§ 195"));
        assert!(r.citation.contains("§ 709"));
    }

    #[test]
    fn zero_organizational_zero_deduction_no_panic() {
        let r = compute(&input(0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.monthly_amortization_cents, 0);
    }

    #[test]
    fn negative_organizational_clamped_to_zero() {
        let i = Section248Input {
            total_organizational_expenditures_cents: -100_000,
            months_active_in_first_year: 12,
            affirmative_capitalization_election: false,
        };
        let r = compute(&i);
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 0);
    }

    #[test]
    fn monthly_amortization_integer_division_truncates() {
        let r = compute(&input(10_000, 12));
        let expected = 500_000i64 / 180;
        assert_eq!(r.monthly_amortization_cents, expected);
        assert_eq!(expected, 2777);
    }

    #[test]
    fn one_million_organizational_full_phase_out_amortizes_over_15_years() {
        let r = compute(&input(1_000_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 100_000_000);
        assert_eq!(
            r.monthly_amortization_cents,
            100_000_000 / AMORTIZATION_MONTHS
        );
    }

    #[test]
    fn phase_out_floor_constant_pins_50000() {
        assert_eq!(PHASE_OUT_FLOOR_CENTS, 5_000_000);
    }

    #[test]
    fn first_year_cap_constant_pins_5000() {
        assert_eq!(FIRST_YEAR_DEDUCTION_CAP_CENTS, 500_000);
    }

    #[test]
    fn phase_out_ceiling_constant_pins_55000() {
        assert_eq!(PHASE_OUT_CEILING_CENTS, 5_500_000);
    }

    #[test]
    fn amortization_months_constant_pins_180() {
        assert_eq!(AMORTIZATION_MONTHS, 180);
    }

    #[test]
    fn election_applies_default_path_true() {
        let r = compute(&input(10_000, 12));
        assert!(r.election_applies);
    }

    #[test]
    fn phase_out_engagement_note_appears_when_above_50k_below_55k() {
        let r = compute(&input(52_500, 12));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("phase-out engaged")));
    }

    #[test]
    fn exclusions_note_always_present() {
        let r = compute(&input(10_000, 12));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1.248-1(b)") && n.contains("EXCLUDES expenses for issuing or selling shares")));
    }

    #[test]
    fn automatic_election_note_present() {
        let r = compute(&input(10_000, 12));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("T.D. 9542") && n.contains("automatic election")));
    }

    #[test]
    fn section_248_phaseout_matches_section_195_invariant() {
        // § 195 and § 248 share the same $5K / $50K / $55K /
        // 180-month framework post-AJCA 2004 § 902.
        let r_248 = compute(&input(53_000, 12));
        // First-year deduction should be $5K - $3K phase-out = $2K
        assert_eq!(r_248.first_year_immediate_deduction_cents, 200_000);
    }
}
