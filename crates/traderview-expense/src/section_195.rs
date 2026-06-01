//! IRC § 195 — Start-up expenditures.
//!
//! Trader-relevant for any taxpayer organizing a trade-or-business
//! entity (TTS-elected LLC, prop-trading firm, family office, S-corp
//! trading vehicle) and incurring pre-operational costs before the
//! "active conduct of trade or business" begins. § 195 provides the
//! ONLY federal vehicle to deduct pre-operational expenses that would
//! otherwise be capitalized under § 263 or non-deductible as not yet
//! engaged in a § 162 trade or business.
//!
//! § 195(a) GENERAL RULE — except as otherwise provided in this
//! section, no deduction shall be allowed for start-up expenditures.
//! The default treatment is capitalization.
//!
//! § 195(b)(1)(A) ELECTION — at the election of the taxpayer, start-up
//! expenditures may be treated as deferred expenses and allowed as a
//! deduction (i) in the taxable year in which the active trade or
//! business begins, in an amount equal to the LESSER OF (I) the amount
//! of start-up expenditures with respect to the active trade or
//! business, or (II) $5,000, reduced (but NOT BELOW ZERO) by the
//! amount by which such start-up expenditures exceed $50,000, AND
//! (ii) the REMAINDER of such start-up expenditures shall be allowed
//! as a deduction RATABLY OVER THE 180-MONTH PERIOD beginning with the
//! month in which the active trade or business begins.
//!
//! § 195(b)(1)(A)(ii)(II) PHASE-OUT — the $5,000 first-year deduction
//! phases out dollar-for-dollar by the amount of startup expenditures
//! exceeding $50,000. At $55,000 total startup expenditures, the
//! first-year deduction is fully phased out and all startup costs must
//! be amortized over 180 months.
//!
//! § 195(c)(1) DEFINITION — "start-up expenditure" means any amount
//! (A) paid or incurred in connection with (i) investigating the
//! creation or acquisition of an active trade or business, (ii)
//! creating an active trade or business, or (iii) any activity engaged
//! in for profit and for the production of income before the day on
//! which the active trade or business begins, in anticipation of such
//! activity becoming an active trade or business, AND (B) which, if
//! paid or incurred in connection with the operation of an existing
//! active trade or business (in the same field as the trade or
//! business referred to in subparagraph (A)), would be allowable as a
//! deduction for the taxable year in which paid or incurred.
//!
//! § 195(c)(1) EXCLUSIONS — start-up expenditures do NOT include
//! amounts deductible under §§ 163(a) (interest), 164 (taxes), or 174
//! (research and experimental). These items follow their own
//! deduction regimes.
//!
//! § 195(d)(1) DEEMED ELECTION — a taxpayer is deemed to have made the
//! § 195(b) election for the taxable year in which the active trade
//! or business begins, unless the taxpayer affirmatively elects to
//! capitalize the expenditures. Treas. Reg. § 1.195-1 (T.D. 9542,
//! September 8, 2011) made the election automatic to reduce taxpayer
//! traps for failure to file an election statement.
//!
//! § 248 INTERACTION — corporate organizational expenditures
//! (incorporation costs, legal fees for charter) are covered by § 248
//! with parallel $5,000 / $50,000 / 180-month mechanics. Partnership
//! organizational expenditures are covered by § 709 with parallel
//! mechanics. § 195 covers PRE-OPERATIONAL OPERATING costs separate
//! from entity formation costs.
//!
//! TRADER APPLICATION: a taxpayer planning to make a § 475(f)
//! mark-to-market trader election for a new entity may incur
//! pre-launch costs (market data subscriptions, brokerage account
//! setup, software, legal opinion on TTS eligibility, accounting
//! services) before the active trade or business begins. These costs
//! qualify as § 195 startup expenditures if they would have been
//! § 162 deductible had the trading business already been active.
//! Once aggregate startup costs reach $50,000, each additional dollar
//! reduces the first-year deduction; at $55,000 first-year deduction
//! is fully phased out and all costs flow to the 180-month
//! amortization stream.
//!
//! Citations: IRC § 195(a) (general capitalization rule); § 195(b)(1)
//! (election + dollar limit + amortization); § 195(b)(1)(A)(ii)
//! (phase-out beginning at $50,000 startup expenditures); § 195(c)(1)
//! (definition + § 163/§ 164/§ 174 exclusions); § 195(d)(1) (deemed
//! election); Treas. Reg. § 1.195-1 (T.D. 9542, September 8, 2011 —
//! automatic election); IRC § 248 (parallel corporate organizational
//! expenditures); IRC § 709 (parallel partnership organizational
//! expenditures); Rev. Rul. 99-23 (when activities cross the line
//! into "active trade or business").

use serde::{Deserialize, Serialize};

pub const PHASE_OUT_FLOOR_CENTS: i64 = 5_000_000;
pub const FIRST_YEAR_DEDUCTION_CAP_CENTS: i64 = 500_000;
pub const PHASE_OUT_CEILING_CENTS: i64 = 5_500_000;
pub const AMORTIZATION_MONTHS: i64 = 180;

#[derive(Debug, Clone, Deserialize)]
pub struct Section195Input {
    /// Total aggregate start-up expenditures in cents. Must EXCLUDE
    /// amounts deductible under §§ 163(a), 164, or 174 (§ 195(c)(1)
    /// last sentence).
    pub total_startup_expenditures_cents: i64,
    /// Months remaining in the taxpayer's first taxable year after
    /// the month the active trade or business begins. Used to compute
    /// the first-year ratable portion of the 180-month amortization
    /// stream. Must be 0-12 inclusive; 12 = active business began in
    /// the first month of the taxable year, 0 = active business began
    /// in the last month.
    pub months_active_in_first_year: u32,
    /// Whether the taxpayer affirmatively elected to capitalize under
    /// § 195(d) (rare — election is now automatic per T.D. 9542).
    pub affirmative_capitalization_election: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section195Result {
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

pub fn compute(input: &Section195Input) -> Section195Result {
    let mut notes: Vec<String> = Vec::new();
    let total = input.total_startup_expenditures_cents.max(0);

    if input.affirmative_capitalization_election {
        notes.push(
            "taxpayer affirmatively elected to capitalize under § 195(d) — no first-year deduction, no amortization"
                .to_string(),
        );
        return Section195Result {
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
            "total startup expenditures at or above $55,000 — first-year deduction fully phased out under § 195(b)(1)(A)(ii)(II)"
                .to_string(),
        );
    } else if total > PHASE_OUT_FLOOR_CENTS {
        notes.push(format!(
            "phase-out engaged — startup costs exceed $50,000 by {} cents; first-year cap reduced from $5,000 to {} cents",
            phase_out_reduction, raw_cap
        ));
    }

    let amortization_pool = total - first_year_immediate;
    let monthly_amortization = if amortization_pool > 0 {
        amortization_pool / AMORTIZATION_MONTHS
    } else {
        0
    };

    let months_active = input.months_active_in_first_year.min(12) as i64;
    let first_year_amortization = monthly_amortization.saturating_mul(months_active);

    if months_active > 0 && amortization_pool > 0 {
        notes.push(format!(
            "remaining {} cents amortized over 180 months starting month active trade or business begins; first-year ratable portion = {} months × {} cents/month = {} cents",
            amortization_pool, months_active, monthly_amortization, first_year_amortization
        ));
    }

    let first_year_total = first_year_immediate.saturating_add(first_year_amortization);

    Section195Result {
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
    "IRC § 195(a)/(b)(1)/(c)(1)/(d)(1); Treas. Reg. § 1.195-1; T.D. 9542 (Sept. 8, 2011); Rev. Rul. 99-23"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(total_dollars: i64, months_active: u32) -> Section195Input {
        Section195Input {
            total_startup_expenditures_cents: total_dollars * 100,
            months_active_in_first_year: months_active,
            affirmative_capitalization_election: false,
        }
    }

    #[test]
    fn small_startup_under_5k_full_first_year_no_amortization() {
        let r = compute(&input(3_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 300_000);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.monthly_amortization_cents, 0);
        assert_eq!(r.first_year_amortization_cents, 0);
        assert_eq!(r.first_year_total_deduction_cents, 300_000);
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn five_k_startup_full_first_year_no_amortization() {
        let r = compute(&input(5_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn ten_k_startup_5k_immediate_plus_180_month_amortization() {
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
    fn fifty_k_startup_at_phase_out_floor_5k_immediate() {
        let r = compute(&input(50_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 500_000);
        assert_eq!(r.amortization_pool_cents, 4500_000);
        assert_eq!(r.phase_out_reduction_cents, 0);
    }

    #[test]
    fn fifty_one_k_startup_1k_phase_out_reduces_immediate_to_4k() {
        let r = compute(&input(51_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 400_000);
        assert_eq!(r.amortization_pool_cents, 4700_000);
        assert_eq!(r.phase_out_reduction_cents, 100_000);
    }

    #[test]
    fn fifty_three_k_startup_3k_phase_out_reduces_immediate_to_2k() {
        let r = compute(&input(53_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 200_000);
        assert_eq!(r.amortization_pool_cents, 5100_000);
        assert_eq!(r.phase_out_reduction_cents, 300_000);
    }

    #[test]
    fn fifty_five_k_startup_full_phase_out_no_immediate_deduction() {
        let r = compute(&input(55_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 5500_000);
        assert_eq!(r.phase_out_reduction_cents, 500_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("fully phased out")));
    }

    #[test]
    fn sixty_k_startup_full_phase_out() {
        let r = compute(&input(60_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 6000_000);
        assert_eq!(r.phase_out_reduction_cents, 1000_000);
    }

    #[test]
    fn one_hundred_k_startup_full_phase_out_large_amortization_pool() {
        let r = compute(&input(100_000, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 10000_000);
        assert_eq!(r.monthly_amortization_cents, 10000_000 / 180);
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
    fn months_active_one_proportional_amortization() {
        let r = compute(&input(10_000, 1));
        let expected_monthly = 500_000 / 180;
        assert_eq!(r.first_year_amortization_cents, expected_monthly);
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
        let r54_999 = compute(&Section195Input {
            total_startup_expenditures_cents: 5499_900,
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
        assert!(r.citation.contains("§ 195(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(c)(1)"));
        assert!(r.citation.contains("(d)(1)"));
        assert!(r.citation.contains("§ 1.195-1"));
        assert!(r.citation.contains("T.D. 9542"));
        assert!(r.citation.contains("Sept. 8, 2011"));
        assert!(r.citation.contains("Rev. Rul. 99-23"));
    }

    #[test]
    fn zero_startup_zero_deduction_no_panic() {
        let r = compute(&input(0, 12));
        assert_eq!(r.first_year_immediate_deduction_cents, 0);
        assert_eq!(r.amortization_pool_cents, 0);
        assert_eq!(r.monthly_amortization_cents, 0);
    }

    #[test]
    fn negative_startup_clamped_to_zero() {
        let i = Section195Input {
            total_startup_expenditures_cents: -100_000,
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
    fn one_million_startup_full_phase_out_amortizes_over_15_years() {
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
        assert_eq!(PHASE_OUT_FLOOR_CENTS, 5000_000);
    }

    #[test]
    fn first_year_cap_constant_pins_5000() {
        assert_eq!(FIRST_YEAR_DEDUCTION_CAP_CENTS, 500_000);
    }

    #[test]
    fn phase_out_ceiling_constant_pins_55000() {
        assert_eq!(PHASE_OUT_CEILING_CENTS, 5500_000);
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
}
