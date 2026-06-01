//! IRC §382 — Limitation on NOL carryforwards and built-in losses
//! following ownership change.
//!
//! Load-bearing rule for any M&A transaction involving a loss
//! corporation. Pairs with `section_172` (NOL deduction): §172
//! determines whether NOLs CAN be deducted in principle; §382
//! determines HOW MUCH per year after an ownership change.
//!
//! **§382(g) ownership change**: triggered when the percentage of
//! stock owned by 5%+ shareholders (or shareholder groups) has
//! increased by more than 50 percentage points within a 3-year
//! testing period, measured against the lowest percentage owned during
//! the period. Trivial-percentage shareholders are aggregated into a
//! single public group.
//!
//! **§382(b)(1) annual limitation** = corporation FMV × applicable
//! long-term tax-exempt rate. The rate is the highest of the federal
//! long-term tax-exempt rates published in the 3 months preceding the
//! ownership change. As an example data point, the February 2026
//! long-term tax-exempt rate was 3.56%.
//!
//! **§382(l)(5) bankruptcy exception**: ownership change occurring in
//! a Title 11 (bankruptcy) case may elect to WAIVE the annual
//! limitation entirely, BUT must reduce pre-change NOLs by the
//! mandatory interest haircut (interest expense deducted on debt
//! converted to stock during the 3 years preceding the bankruptcy
//! petition). Requires the "50% continuity test" — historic
//! shareholders + qualified creditors must own ≥ 50% of reorganized
//! entity by both vote AND value.
//!
//! **§382(h) built-in gains/losses** adjustments: post-ownership-change
//! net unrealized built-in gain (NUBIG) recognized within the 5-year
//! recognition period INCREASES the annual limitation. Conversely,
//! net unrealized built-in losses (NUBIL) recognized within the
//! 5-year recognition period are SUBJECT to the limitation as if
//! pre-change losses. This module surfaces a built-in gain adjustment
//! flag for caller-side handling.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section382Input {
    /// Fair market value of the loss corporation's stock immediately
    /// before the ownership change.
    pub corporation_fmv_at_change: Decimal,
    /// Applicable federal long-term tax-exempt rate in basis points
    /// (e.g., 356 = 3.56%). Highest of the 3 months preceding the
    /// ownership change.
    pub long_term_tax_exempt_rate_basis_points: u32,
    pub pre_change_nol_carryover: Decimal,
    /// True if the §382(l)(5) bankruptcy exception is elected (Title 11
    /// case + 50% continuity test satisfied).
    pub bankruptcy_exception_elected: bool,
    /// Mandatory interest haircut required under §382(l)(5) — interest
    /// expense deducted on debt converted to stock during the 3 years
    /// preceding the bankruptcy petition.
    pub mandatory_interest_haircut_l5: Decimal,
    /// True if §382(h) net unrealized built-in gain (NUBIG) increases
    /// the annual limitation through the 5-year recognition period.
    pub nubig_recognition_increases_limit: bool,
    /// Amount of recognized built-in gain in the relevant year (used to
    /// increase annual limitation when `nubig_recognition_increases_limit`).
    pub recognized_built_in_gain_this_year: Decimal,
    pub taxable_income_before_nol_this_year: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section382Result {
    pub annual_section_382_limitation: Decimal,
    /// Annual limitation as adjusted by §382(h) recognized built-in gain
    /// (when applicable). Equal to the base limitation otherwise.
    pub adjusted_annual_limitation: Decimal,
    /// True if the §382(l)(5) bankruptcy exception waives the annual
    /// limitation. When true, `adjusted_annual_limitation` is effectively
    /// unlimited (returned as the full pre-change NOL adjusted for the
    /// mandatory interest haircut).
    pub bankruptcy_l5_waives_limitation: bool,
    /// Pre-change NOL after the §382(l)(5) mandatory interest haircut
    /// (when bankruptcy exception elected). Equal to input pre-change
    /// NOL otherwise.
    pub usable_nol_after_l5_haircut: Decimal,
    /// NOL allowed to be used in the current year — min(adjusted limit,
    /// usable NOL, taxable income before NOL).
    pub nol_used_this_year: Decimal,
    pub nol_carryforward_after_this_year: Decimal,
    pub note: String,
}

pub fn compute(input: &Section382Input) -> Section382Result {
    // §382(b)(1) annual limitation = FMV × LT tax-exempt rate.
    let base_limit = input.corporation_fmv_at_change
        * Decimal::from(input.long_term_tax_exempt_rate_basis_points)
        / Decimal::from(10_000);

    // §382(h) built-in gain adjustment.
    let adjusted_limit = if input.nubig_recognition_increases_limit {
        base_limit + input.recognized_built_in_gain_this_year
    } else {
        base_limit
    };

    // §382(l)(5) bankruptcy exception.
    let (usable_nol, waives_limit, effective_limit) = if input.bankruptcy_exception_elected {
        let after_haircut = (input.pre_change_nol_carryover - input.mandatory_interest_haircut_l5)
            .max(Decimal::ZERO);
        // Limitation waived entirely; effective limit is the (reduced) NOL itself.
        (after_haircut, true, after_haircut)
    } else {
        (input.pre_change_nol_carryover, false, adjusted_limit)
    };

    // NOL used this year = min(effective limit, usable NOL, TI before NOL).
    let nol_used = effective_limit
        .min(usable_nol)
        .min(input.taxable_income_before_nol_this_year)
        .max(Decimal::ZERO);
    let carryforward = (usable_nol - nol_used).max(Decimal::ZERO);

    let note = if waives_limit {
        format!(
            "§382(l)(5) bankruptcy exception ELECTED — annual limitation WAIVED; pre-change NOL reduced by ${} interest haircut to ${}; ${} NOL used against ${} taxable income; ${} carries forward",
            input.mandatory_interest_haircut_l5.round_dp(2),
            usable_nol.round_dp(2),
            nol_used.round_dp(2),
            input.taxable_income_before_nol_this_year.round_dp(2),
            carryforward.round_dp(2),
        )
    } else {
        format!(
            "§382(b)(1) annual limitation = ${} (FMV ${} × {}.{}% LT tax-exempt rate){}; ${} NOL used against ${} taxable income; ${} pre-change NOL remains",
            base_limit.round_dp(2),
            input.corporation_fmv_at_change.round_dp(2),
            input.long_term_tax_exempt_rate_basis_points / 100,
            input.long_term_tax_exempt_rate_basis_points % 100,
            if input.nubig_recognition_increases_limit {
                format!(
                    " + ${} §382(h) NUBIG adjustment = ${}",
                    input.recognized_built_in_gain_this_year.round_dp(2),
                    adjusted_limit.round_dp(2)
                )
            } else {
                String::new()
            },
            nol_used.round_dp(2),
            input.taxable_income_before_nol_this_year.round_dp(2),
            carryforward.round_dp(2),
        )
    };

    Section382Result {
        annual_section_382_limitation: base_limit,
        adjusted_annual_limitation: adjusted_limit,
        bankruptcy_l5_waives_limitation: waives_limit,
        usable_nol_after_l5_haircut: usable_nol,
        nol_used_this_year: nol_used,
        nol_carryforward_after_this_year: carryforward,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section382Input {
        Section382Input {
            corporation_fmv_at_change: dec!(10_000_000),
            long_term_tax_exempt_rate_basis_points: 356, // Feb 2026 rate
            pre_change_nol_carryover: dec!(5_000_000),
            bankruptcy_exception_elected: false,
            mandatory_interest_haircut_l5: Decimal::ZERO,
            nubig_recognition_increases_limit: false,
            recognized_built_in_gain_this_year: Decimal::ZERO,
            taxable_income_before_nol_this_year: dec!(1_000_000),
        }
    }

    #[test]
    fn basic_382b1_annual_limit_calculation() {
        // FMV $10M × 3.56% = $356,000 annual limit.
        let r = compute(&base());
        assert_eq!(r.annual_section_382_limitation, dec!(356_000));
        assert_eq!(r.adjusted_annual_limitation, dec!(356_000));
    }

    #[test]
    fn nol_use_limited_by_annual_382_limitation() {
        // Annual limit $356k. Taxable income $1M. NOL pool $5M.
        // → NOL used = $356k (limit binds); carryforward = $4.644M.
        let r = compute(&base());
        assert_eq!(r.nol_used_this_year, dec!(356_000));
        assert_eq!(r.nol_carryforward_after_this_year, dec!(4_644_000));
    }

    #[test]
    fn nol_use_limited_by_taxable_income() {
        // TI lower than the limit and NOL pool.
        let mut i = base();
        i.taxable_income_before_nol_this_year = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.nol_used_this_year, dec!(100_000));
    }

    #[test]
    fn nol_use_limited_by_remaining_nol_pool() {
        let mut i = base();
        i.pre_change_nol_carryover = dec!(200_000);
        let r = compute(&i);
        assert_eq!(r.nol_used_this_year, dec!(200_000));
        assert_eq!(r.nol_carryforward_after_this_year, Decimal::ZERO);
    }

    #[test]
    fn nubig_built_in_gain_increases_annual_limit() {
        // §382(h): recognized built-in gain $200k INCREASES the limit.
        // Base $356k + $200k = $556k adjusted limit.
        let mut i = base();
        i.nubig_recognition_increases_limit = true;
        i.recognized_built_in_gain_this_year = dec!(200_000);
        i.taxable_income_before_nol_this_year = dec!(1_000_000);
        let r = compute(&i);
        assert_eq!(r.adjusted_annual_limitation, dec!(556_000));
        assert_eq!(r.nol_used_this_year, dec!(556_000));
    }

    #[test]
    fn bankruptcy_l5_waives_annual_limit_but_haircuts_nol() {
        // §382(l)(5): waive annual limit; reduce pre-change NOL by
        // mandatory interest haircut.
        let mut i = base();
        i.bankruptcy_exception_elected = true;
        i.mandatory_interest_haircut_l5 = dec!(1_000_000);
        let r = compute(&i);
        assert!(r.bankruptcy_l5_waives_limitation);
        // NOL reduced from $5M to $4M.
        assert_eq!(r.usable_nol_after_l5_haircut, dec!(4_000_000));
        // NOL use limited by TI $1M (annual limit waived).
        assert_eq!(r.nol_used_this_year, dec!(1_000_000));
        assert_eq!(r.nol_carryforward_after_this_year, dec!(3_000_000));
    }

    #[test]
    fn bankruptcy_l5_haircut_exceeds_nol_clamps_to_zero() {
        let mut i = base();
        i.bankruptcy_exception_elected = true;
        i.pre_change_nol_carryover = dec!(500_000);
        i.mandatory_interest_haircut_l5 = dec!(1_000_000);
        let r = compute(&i);
        assert_eq!(r.usable_nol_after_l5_haircut, Decimal::ZERO);
        assert_eq!(r.nol_used_this_year, Decimal::ZERO);
    }

    #[test]
    fn zero_fmv_zero_limit_kills_pre_change_nol_effectively() {
        // FMV $0 (insolvent corp without bankruptcy election) → §382(b)
        // limit = $0 × rate = $0. Pre-change NOL effectively dead.
        let mut i = base();
        i.corporation_fmv_at_change = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.annual_section_382_limitation, Decimal::ZERO);
        assert_eq!(r.nol_used_this_year, Decimal::ZERO);
        // Pre-change NOL still carries forward but can never be used.
        assert_eq!(r.nol_carryforward_after_this_year, dec!(5_000_000));
    }

    #[test]
    fn rate_2025_higher_than_2026_higher_limit() {
        // Higher LT tax-exempt rate yields higher annual limit. Pin
        // that the basis-points multiplier is wired correctly.
        let mut i = base();
        i.long_term_tax_exempt_rate_basis_points = 500; // 5.00%
        let r = compute(&i);
        assert_eq!(r.annual_section_382_limitation, dec!(500_000));
    }

    #[test]
    fn nubig_only_increases_when_flag_set() {
        // Without the NUBIG flag, recognized built-in gain alone doesn't
        // affect the limit. Pinned because the flag is the gatekeeper.
        let mut i = base();
        i.nubig_recognition_increases_limit = false;
        i.recognized_built_in_gain_this_year = dec!(200_000);
        let r = compute(&i);
        assert_eq!(r.adjusted_annual_limitation, dec!(356_000));
    }

    #[test]
    fn taxable_income_zero_no_nol_used() {
        let mut i = base();
        i.taxable_income_before_nol_this_year = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.nol_used_this_year, Decimal::ZERO);
    }

    #[test]
    fn very_large_fmv_no_precision_loss() {
        // $1B FMV × 3.56% = $35.6M annual limit.
        let mut i = base();
        i.corporation_fmv_at_change = dec!(1_000_000_000);
        i.taxable_income_before_nol_this_year = dec!(100_000_000);
        i.pre_change_nol_carryover = dec!(500_000_000);
        let r = compute(&i);
        assert_eq!(r.annual_section_382_limitation, dec!(35_600_000));
        assert_eq!(r.nol_used_this_year, dec!(35_600_000));
    }

    #[test]
    fn l5_election_with_zero_haircut_preserves_full_nol() {
        // §382(l)(5) elected with $0 haircut (no debt-for-stock interest
        // to claw back) — annual limit still waived, full NOL preserved.
        let mut i = base();
        i.bankruptcy_exception_elected = true;
        i.mandatory_interest_haircut_l5 = Decimal::ZERO;
        let r = compute(&i);
        assert!(r.bankruptcy_l5_waives_limitation);
        assert_eq!(r.usable_nol_after_l5_haircut, dec!(5_000_000));
    }

    #[test]
    fn note_describes_382b1_annual_limit_path() {
        let r = compute(&base());
        assert!(r.note.contains("§382(b)(1)"));
        assert!(r.note.contains("3.56%"));
    }

    #[test]
    fn note_describes_l5_bankruptcy_path() {
        let mut i = base();
        i.bankruptcy_exception_elected = true;
        i.mandatory_interest_haircut_l5 = dec!(500_000);
        let r = compute(&i);
        assert!(r.note.contains("§382(l)(5)"));
        assert!(r.note.contains("WAIVED"));
        assert!(r.note.contains("interest haircut"));
    }

    #[test]
    fn note_describes_nubig_adjustment_when_applicable() {
        let mut i = base();
        i.nubig_recognition_increases_limit = true;
        i.recognized_built_in_gain_this_year = dec!(200_000);
        let r = compute(&i);
        assert!(r.note.contains("§382(h)"));
        assert!(r.note.contains("NUBIG"));
    }

    #[test]
    fn limit_exact_boundary_uses_full_limit() {
        // Taxable income exactly equals the §382 limit. NOL used = limit
        // exactly; no fractional issue.
        let mut i = base();
        i.taxable_income_before_nol_this_year = dec!(356_000);
        let r = compute(&i);
        assert_eq!(r.nol_used_this_year, dec!(356_000));
    }

    #[test]
    fn rate_zero_yields_zero_limit() {
        let mut i = base();
        i.long_term_tax_exempt_rate_basis_points = 0;
        let r = compute(&i);
        assert_eq!(r.annual_section_382_limitation, Decimal::ZERO);
        assert_eq!(r.nol_used_this_year, Decimal::ZERO);
    }

    #[test]
    fn carryforward_never_negative() {
        // Pathological inputs shouldn't produce negative carryforward.
        let mut i = base();
        i.pre_change_nol_carryover = dec!(100);
        i.taxable_income_before_nol_this_year = dec!(1_000_000_000);
        let r = compute(&i);
        assert!(r.nol_carryforward_after_this_year >= Decimal::ZERO);
        // NOL used = min(limit $356k, NOL $100, TI $1B) = $100.
        assert_eq!(r.nol_used_this_year, dec!(100));
        assert_eq!(r.nol_carryforward_after_this_year, Decimal::ZERO);
    }
}
