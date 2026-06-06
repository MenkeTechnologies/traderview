//! IRC § 170 — Charitable contribution deduction (individual).
//!
//! § 170(a) allows an itemized deduction for charitable contributions
//! to qualified organizations. The deduction is subject to (i) a per-
//! category PERCENTAGE-OF-AGI ceiling under § 170(b)(1) and (ii) the
//! 0.5%-of-AGI FLOOR introduced by OBBBA § 70425 for tax years
//! beginning after 2025-12-31.
//!
//! Sibling to `section_170e` (built-in-gain ordinary-income reduction
//! for non-LTCG property donations). This module addresses the broader
//! § 170(b) percentage limits + the new OBBBA floor.
//!
//! Percentage-of-AGI ceilings (§ 170(b)(1)):
//!
//! 60% AGI — cash contributions to public charities (§ 170(b)(1)(G)) —
//! made PERMANENT by OBBBA (was scheduled to revert to 50%).
//!
//! 50% AGI — non-cash contributions to public charities (§ 170(b)(1)(A)).
//!
//! 30% AGI — capital-gain property to public charities, OR cash to
//! 30%-limit organizations (§ 170(b)(1)(B)/(C)).
//!
//! 20% AGI — capital-gain property to private foundations
//! (§ 170(b)(1)(D)).
//!
//! 0.5%-of-AGI floor (§ 170(b)(1)(I), OBBBA § 70425 eff. 2026):
//!
//! Itemizers must reduce their aggregate charitable deduction by 0.5%
//! of AGI. Amounts blocked by the floor carry forward for 5 succeeding
//! tax years.
//!
//! Non-itemizer above-the-line deduction (§ 170(p), OBBBA eff. 2026):
//!
//! Up to $1,000 for individuals / $2,000 for MFJ. Available only to
//! NON-ITEMIZERS. Limited to cash contributions to public charities.
//!
//! Citations: 26 U.S.C. § 170; § 170(a) (general deduction); § 170(b)(1)
//! (per-category AGI ceilings); § 170(b)(1)(G) (60% cash-to-public-
//! charity made permanent by OBBBA); § 170(b)(1)(I) (0.5% AGI floor,
//! OBBBA § 70425); § 170(p) (non-itemizer above-the-line deduction,
//! OBBBA eff. 2026); § 170(d)(1) (5-year carryforward).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    QualifyingWidow,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section170Input {
    pub year: u32,
    pub filing_status: FilingStatus,
    pub itemizes: bool,
    pub agi_cents: i64,
    /// Cash contributions to public charities (60% AGI ceiling).
    pub cash_to_public_charity_cents: i64,
    /// Capital-gain property to public charities (30% AGI ceiling).
    pub capital_gain_property_to_public_charity_cents: i64,
    /// Cash contributions to private foundations / 30%-limit orgs.
    pub cash_to_private_foundation_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section170Result {
    pub aggregate_contributions_cents: i64,
    pub agi_floor_threshold_cents: i64,
    pub agi_floor_applies: bool,
    pub amount_blocked_by_floor_cents: i64,
    pub sixty_pct_cap_cents: i64,
    pub thirty_pct_cap_cents: i64,
    pub amount_above_ceiling_cents: i64,
    pub allowed_itemized_deduction_cents: i64,
    pub non_itemizer_above_line_deduction_cents: i64,
    pub non_itemizer_max_cents: i64,
    pub carryforward_to_next_year_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section170Input) -> Section170Result {
    let agi = input.agi_cents.max(0);
    let cash_public = input.cash_to_public_charity_cents.max(0);
    let cap_gain_public = input.capital_gain_property_to_public_charity_cents.max(0);
    let cash_private = input.cash_to_private_foundation_cents.max(0);
    let aggregate = cash_public + cap_gain_public + cash_private;

    // § 170(b)(1) percentage-of-AGI ceilings.
    let sixty_cap = (agi as i128 * 60 / 100) as i64;
    let thirty_cap = (agi as i128 * 30 / 100) as i64;

    // OBBBA § 170(p) non-itemizer above-the-line deduction.
    let non_itemizer_max = match input.filing_status {
        FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow => 2_00000,
        _ => 1_00000,
    };

    if !input.itemizes {
        // Non-itemizer path: § 170(p) eff. 2026. Pre-2026 = no deduction.
        let non_itemizer_deduction = if input.year >= 2026 {
            cash_public.min(non_itemizer_max)
        } else {
            0
        };
        return Section170Result {
            aggregate_contributions_cents: aggregate,
            agi_floor_threshold_cents: 0,
            agi_floor_applies: false,
            amount_blocked_by_floor_cents: 0,
            sixty_pct_cap_cents: sixty_cap,
            thirty_pct_cap_cents: thirty_cap,
            amount_above_ceiling_cents: 0,
            allowed_itemized_deduction_cents: 0,
            non_itemizer_above_line_deduction_cents: non_itemizer_deduction,
            non_itemizer_max_cents: non_itemizer_max,
            carryforward_to_next_year_cents: 0,
            citation: if input.year >= 2026 {
                "26 U.S.C. § 170(p) (OBBBA eff. 2026) — non-itemizer above-the-line deduction up to $1,000 single / $2,000 MFJ, cash to public charity only"
            } else {
                "26 U.S.C. § 170 — pre-2026 non-itemizers cannot deduct charitable contributions; § 170(p) above-the-line option starts in 2026"
            },
            note: format!(
                "Non-itemizer for {}. Cash to public charity {} cents. § 170(p) deduction = min(cash, {} cents max) = {} cents.",
                input.year, cash_public, non_itemizer_max, non_itemizer_deduction
            ),
        };
    }

    // Itemizer path. Apply per-category ceilings first (in ordering rule),
    // then apply 0.5% AGI floor (if 2026+) reducing the aggregate.
    let after_ceilings = aggregate.min(
        cash_public.min(sixty_cap) + cap_gain_public.min(thirty_cap) + cash_private.min(thirty_cap),
    );
    let amount_above_ceiling = aggregate - after_ceilings;

    let floor_applies = input.year >= 2026;
    let floor_threshold = if floor_applies {
        (agi as i128 * 5 / 1000) as i64 // 0.5% = 5/1000
    } else {
        0
    };
    let amount_blocked_by_floor = if floor_applies {
        after_ceilings.min(floor_threshold)
    } else {
        0
    };
    let allowed = (after_ceilings - amount_blocked_by_floor).max(0);
    // Both ceiling-blocked and floor-blocked amounts carry forward 5 years
    // under § 170(d)(1) + § 170(b)(1)(I) carryforward rule.
    let carryforward = amount_above_ceiling + amount_blocked_by_floor;

    let note = format!(
        "Itemizer for {}. Aggregate = {} cents (cash-public {} + cap-gain-public {} + cash-private {}). Per-category ceilings (60% AGI cash {} / 30% AGI property {}). After ceilings = {} cents. {}AGI floor 0.5% × {} = {} cents → blocked {} cents. Allowed itemized deduction = {} cents. § 170(d)(1) carryforward to next 5 years = {} cents.",
        input.year,
        aggregate,
        cash_public,
        cap_gain_public,
        cash_private,
        sixty_cap,
        thirty_cap,
        after_ceilings,
        if floor_applies { "OBBBA § 70425 0.5% " } else { "(pre-OBBBA — no floor) " },
        agi,
        floor_threshold,
        amount_blocked_by_floor,
        allowed,
        carryforward,
    );

    Section170Result {
        aggregate_contributions_cents: aggregate,
        agi_floor_threshold_cents: floor_threshold,
        agi_floor_applies: floor_applies,
        amount_blocked_by_floor_cents: amount_blocked_by_floor,
        sixty_pct_cap_cents: sixty_cap,
        thirty_pct_cap_cents: thirty_cap,
        amount_above_ceiling_cents: amount_above_ceiling,
        allowed_itemized_deduction_cents: allowed,
        non_itemizer_above_line_deduction_cents: 0,
        non_itemizer_max_cents: non_itemizer_max,
        carryforward_to_next_year_cents: carryforward,
        citation:
            "26 U.S.C. § 170(b)(1) per-category AGI ceilings (60% cash-to-public made permanent by OBBBA); § 170(b)(1)(I) (OBBBA § 70425 0.5% AGI floor eff. 2026); § 170(d)(1) 5-year carryforward; § 170(p) non-itemizer above-the-line",
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        fs: FilingStatus,
        itemizes: bool,
        agi: i64,
        cash_public: i64,
        cap_gain_public: i64,
        cash_private: i64,
    ) -> Section170Input {
        Section170Input {
            year,
            filing_status: fs,
            itemizes,
            agi_cents: agi,
            cash_to_public_charity_cents: cash_public,
            capital_gain_property_to_public_charity_cents: cap_gain_public,
            cash_to_private_foundation_cents: cash_private,
        }
    }

    #[test]
    fn itemizer_2026_cash_under_60pct_cap_under_floor_no_deduction() {
        // AGI $100K, cash $400 → under 0.5% × $100K = $500 floor → all blocked.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            100_000_00,
            400_00,
            0,
            0,
        ));
        assert_eq!(r.allowed_itemized_deduction_cents, 0);
        assert_eq!(r.amount_blocked_by_floor_cents, 400_00);
        assert_eq!(r.carryforward_to_next_year_cents, 400_00);
        assert!(r.agi_floor_applies);
    }

    #[test]
    fn itemizer_2026_cash_above_floor_partial_deduction() {
        // AGI $100K, cash $2,000 → 0.5% floor = $500. Allowed = $2,000 - $500 = $1,500.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            100_000_00,
            2_000_00,
            0,
            0,
        ));
        assert_eq!(r.agi_floor_threshold_cents, 500_00);
        assert_eq!(r.allowed_itemized_deduction_cents, 1_500_00);
        assert_eq!(r.carryforward_to_next_year_cents, 500_00);
    }

    #[test]
    fn itemizer_2025_no_floor_full_deduction() {
        // 2025 → no 0.5% floor (OBBBA not yet effective).
        let r = compute(&input(
            2025,
            FilingStatus::Single,
            true,
            100_000_00,
            2_000_00,
            0,
            0,
        ));
        assert!(!r.agi_floor_applies);
        assert_eq!(r.amount_blocked_by_floor_cents, 0);
        assert_eq!(r.allowed_itemized_deduction_cents, 2_000_00);
    }

    #[test]
    fn itemizer_2026_above_60pct_cash_cap_carries_forward() {
        // AGI $50K, cash $40K to public charity. 60% cap = $30K.
        // $10K above ceiling carries forward; floor $250 → $30K - $250 = $29,750 allowed.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            50_000_00,
            40_000_00,
            0,
            0,
        ));
        assert_eq!(r.sixty_pct_cap_cents, 30_000_00);
        assert_eq!(r.amount_above_ceiling_cents, 10_000_00);
        assert_eq!(r.agi_floor_threshold_cents, 250_00);
        assert_eq!(r.allowed_itemized_deduction_cents, 29_750_00);
        // Carryforward = $10K above ceiling + $250 below floor.
        assert_eq!(r.carryforward_to_next_year_cents, 10_250_00);
    }

    #[test]
    fn itemizer_2026_capital_gain_property_30pct_cap() {
        // AGI $100K, capital-gain property $50K → 30% cap = $30K.
        // $20K above ceiling. Floor $500 → $30K - $500 = $29,500 allowed.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            100_000_00,
            0,
            50_000_00,
            0,
        ));
        assert_eq!(r.thirty_pct_cap_cents, 30_000_00);
        assert_eq!(r.allowed_itemized_deduction_cents, 29_500_00);
    }

    #[test]
    fn non_itemizer_2026_above_line_1000_single() {
        // § 170(p) eff. 2026: $1,000 single / $2,000 MFJ above-the-line.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            false,
            100_000_00,
            500_00,
            0,
            0,
        ));
        assert_eq!(r.non_itemizer_above_line_deduction_cents, 500_00);
        assert_eq!(r.non_itemizer_max_cents, 100000);
        assert_eq!(r.allowed_itemized_deduction_cents, 0);
        assert!(r.citation.contains("§ 170(p)"));
    }

    #[test]
    fn non_itemizer_2026_max_caps_at_1000_single() {
        // $5,000 cash but $1,000 max.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            false,
            100_000_00,
            5_000_00,
            0,
            0,
        ));
        assert_eq!(r.non_itemizer_above_line_deduction_cents, 1_000_00);
    }

    #[test]
    fn non_itemizer_2026_mfj_2000_max() {
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingJointly,
            false,
            100_000_00,
            5_000_00,
            0,
            0,
        ));
        assert_eq!(r.non_itemizer_above_line_deduction_cents, 2_000_00);
        assert_eq!(r.non_itemizer_max_cents, 200000);
    }

    #[test]
    fn non_itemizer_2025_no_deduction() {
        // Pre-2026: non-itemizer cannot deduct.
        let r = compute(&input(
            2025,
            FilingStatus::Single,
            false,
            100_000_00,
            500_00,
            0,
            0,
        ));
        assert_eq!(r.non_itemizer_above_line_deduction_cents, 0);
        assert!(r.citation.contains("pre-2026"));
    }

    #[test]
    fn itemizer_2026_at_0_5_pct_boundary_no_blockage() {
        // AGI $100K, cash exactly $500 (= 0.5% floor). All blocked.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            100_000_00,
            500_00,
            0,
            0,
        ));
        assert_eq!(r.amount_blocked_by_floor_cents, 500_00);
        assert_eq!(r.allowed_itemized_deduction_cents, 0);
    }

    #[test]
    fn itemizer_2026_just_above_floor() {
        // AGI $100K, cash $501. Allowed = $501 - $500 = $1.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            100_000_00,
            501_00,
            0,
            0,
        ));
        assert_eq!(r.allowed_itemized_deduction_cents, 1_00);
    }

    #[test]
    fn worked_example_500k_agi_itemizer_2026() {
        // Blue J worked example: $500K AGI, $10K cash to public charity.
        // 60% cap = $300K (not binding). Floor = $2,500. Allowed = $7,500.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            500_000_00,
            10_000_00,
            0,
            0,
        ));
        assert_eq!(r.agi_floor_threshold_cents, 2_500_00);
        assert_eq!(r.allowed_itemized_deduction_cents, 7_500_00);
        assert_eq!(r.carryforward_to_next_year_cents, 2_500_00);
    }

    #[test]
    fn worked_example_500k_agi_itemizer_2025_no_floor() {
        let r = compute(&input(
            2025,
            FilingStatus::Single,
            true,
            500_000_00,
            10_000_00,
            0,
            0,
        ));
        assert_eq!(r.allowed_itemized_deduction_cents, 10_000_00);
    }

    #[test]
    fn mfj_filing_status_uses_joint_2000_max() {
        let r = compute(&input(
            2026,
            FilingStatus::QualifyingWidow,
            false,
            100_000_00,
            5_000_00,
            0,
            0,
        ));
        assert_eq!(r.non_itemizer_max_cents, 200000);
    }

    #[test]
    fn hoh_uses_single_1000_max() {
        let r = compute(&input(
            2026,
            FilingStatus::HeadOfHousehold,
            false,
            100_000_00,
            5_000_00,
            0,
            0,
        ));
        assert_eq!(r.non_itemizer_max_cents, 100000);
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(2026, FilingStatus::Single, true, -1, -1, -1, -1));
        assert_eq!(r.allowed_itemized_deduction_cents, 0);
    }

    #[test]
    fn zero_agi_zero_floor() {
        let r = compute(&input(2026, FilingStatus::Single, true, 0, 500_00, 0, 0));
        assert_eq!(r.agi_floor_threshold_cents, 0);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_itemizer = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            100_000_00,
            2_000_00,
            0,
            0,
        ));
        assert!(r_itemizer.citation.contains("§ 170(b)(1)"));
        assert!(r_itemizer.citation.contains("OBBBA § 70425"));
        assert!(r_itemizer.citation.contains("§ 170(b)(1)(I)"));
        assert!(r_itemizer.citation.contains("§ 170(d)(1)"));
        assert!(r_itemizer.citation.contains("§ 170(p)"));

        let r_non_itemizer_2026 = compute(&input(
            2026,
            FilingStatus::Single,
            false,
            100_000_00,
            500_00,
            0,
            0,
        ));
        assert!(r_non_itemizer_2026.citation.contains("§ 170(p)"));

        let r_non_itemizer_2025 = compute(&input(
            2025,
            FilingStatus::Single,
            false,
            100_000_00,
            500_00,
            0,
            0,
        ));
        assert!(r_non_itemizer_2025.citation.contains("pre-2026"));
    }

    #[test]
    fn ceiling_then_floor_both_carry_forward() {
        // AGI $50K, cash $40K. Ceiling blocks $10K. Then floor blocks
        // another $250 of the $30K cap. Total carryforward = $10,250.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            true,
            50_000_00,
            40_000_00,
            0,
            0,
        ));
        assert_eq!(r.amount_above_ceiling_cents, 10_000_00);
        assert_eq!(r.amount_blocked_by_floor_cents, 250_00);
        assert_eq!(r.carryforward_to_next_year_cents, 10_250_00);
    }

    #[test]
    fn obbba_floor_effective_only_2026_plus() {
        for year in [2024, 2025] {
            let r = compute(&input(
                year,
                FilingStatus::Single,
                true,
                100_000_00,
                500_00,
                0,
                0,
            ));
            assert!(!r.agi_floor_applies);
            assert_eq!(r.amount_blocked_by_floor_cents, 0);
        }
        for year in [2026, 2027, 2030] {
            let r = compute(&input(
                year,
                FilingStatus::Single,
                true,
                100_000_00,
                500_00,
                0,
                0,
            ));
            assert!(r.agi_floor_applies);
            assert_eq!(r.amount_blocked_by_floor_cents, 500_00);
        }
    }
}
