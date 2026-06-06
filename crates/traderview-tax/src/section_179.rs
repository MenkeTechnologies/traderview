//! § 179 expensing + § 168(k) bonus depreciation for 2025.
//!
//! Lets a small business write off the cost of qualifying property
//! (equipment, off-the-shelf software, qualified improvement property)
//! in the year placed in service, instead of depreciating it over the
//! MACRS recovery period.
//!
//! ### § 179 — Election to Expense (2025 per Rev. Proc. 2024-40 § 3.18)
//!
//! * **Maximum deduction**: $1,250,000
//! * **Phaseout threshold**: $3,130,000
//! * Above-threshold reduction: $1 of § 179 cap lost for each $1 of
//!   qualifying-property cost above the threshold (fully phases out at
//!   $4,380,000 = $3,130,000 + $1,250,000).
//! * Cannot create a loss — limited to aggregate trade-or-business
//!   taxable income (§ 179(b)(3)). Disallowed amount carries forward.
//! * Vehicles ≤ 6,000 lb GVWR are subject to luxury-auto limits
//!   (§ 280F) — caller passes a separate `auto_cap` to enforce.
//!
//! ### § 168(k) — Bonus depreciation (TCJA phasedown)
//!
//! * 2025 rate: **40%** (was 60% in 2024, down to 20% in 2026, 0% in 2027).
//! * Applied to the remainder *after* § 179 is taken.
//! * No income limit — bonus can create a loss.
//! * Default-applied; taxpayer can elect out per class of property.
//!
//! ### Algorithm
//!
//! 1. Apply § 179 cap (with phaseout) → `s179_allowed`.
//! 2. Apply income limit → `s179_income_limited`. Excess carries
//!    forward (returned separately).
//! 3. Compute remainder = total_cost - s179_income_limited.
//! 4. Apply 40% bonus to remainder.
//! 5. Final remainder = MACRS depreciation base for future years.
//!
//! Sources:
//!   * IRC § 179
//!   * IRC § 168(k) (TCJA phasedown schedule)
//!   * Rev. Proc. 2024-40 § 3.18 (2025 § 179 limits)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// § 179 maximum deduction for 2025.
pub const S179_MAX_2025: i64 = 1_250_000;
/// § 179 phaseout threshold for 2025.
pub const S179_PHASEOUT_THRESHOLD_2025: i64 = 3_130_000;
/// Bonus depreciation rate for property placed in service in 2025.
pub const BONUS_RATE_2025: &str = "0.40";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Section179Input {
    /// Total cost of § 179 qualifying property placed in service in 2025.
    pub total_qualifying_cost: Decimal,
    /// Aggregate trade-or-business taxable income before § 179 (§ 179(b)(3) cap).
    pub trade_or_business_income: Decimal,
    /// Optional per-vehicle luxury-auto cap (IRC § 280F). Pass `None`
    /// when no § 280F-affected vehicles are in the basket; pass `Some(x)`
    /// to clamp § 179 at `x` instead of the statutory max.
    pub auto_cap: Option<Decimal>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Section179Result {
    /// § 179 max for 2025 *after* phaseout reduction (before income limit).
    pub s179_max_after_phaseout: Decimal,
    /// § 179 actually allowed this year (after both phaseout AND income).
    pub s179_deduction: Decimal,
    /// Portion of § 179 election disallowed by income limit, carried
    /// forward to next year.
    pub s179_carryover: Decimal,
    /// 40% bonus applied to (total_cost - s179_deduction).
    pub bonus_depreciation: Decimal,
    /// MACRS depreciation base remaining after § 179 + bonus.
    pub remaining_macrs_basis: Decimal,
    /// First-year total write-off = § 179 + bonus.
    pub first_year_write_off: Decimal,
}

pub fn compute(input: Section179Input) -> Section179Result {
    let total_cost = input.total_qualifying_cost.max(Decimal::ZERO);

    // § 179 max with phaseout: dollar-for-dollar reduction above threshold.
    let s179_statutory_max = Decimal::from(S179_MAX_2025);
    let phaseout_threshold = Decimal::from(S179_PHASEOUT_THRESHOLD_2025);
    let excess_over_phaseout = (total_cost - phaseout_threshold).max(Decimal::ZERO);
    let mut s179_max_after_phaseout =
        (s179_statutory_max - excess_over_phaseout).max(Decimal::ZERO);

    // § 280F luxury-auto clamp.
    if let Some(cap) = input.auto_cap {
        s179_max_after_phaseout = s179_max_after_phaseout.min(cap);
    }

    // § 179 election capped by qualifying cost itself (can't elect more
    // than was actually purchased).
    let s179_elected = s179_max_after_phaseout.min(total_cost);

    // Income limit (§ 179(b)(3)): § 179 can't create a loss.
    let income_cap = input.trade_or_business_income.max(Decimal::ZERO);
    let s179_deduction = s179_elected.min(income_cap);
    let s179_carryover = s179_elected - s179_deduction;

    // Bonus on remainder (no income limit).
    let remainder_after_s179 = (total_cost - s179_deduction).max(Decimal::ZERO);
    let bonus_rate: Decimal = BONUS_RATE_2025.parse().unwrap();
    let bonus_depreciation = (remainder_after_s179 * bonus_rate).round_dp(2);

    let remaining_macrs_basis = (remainder_after_s179 - bonus_depreciation).max(Decimal::ZERO);
    let first_year_write_off = s179_deduction + bonus_depreciation;

    Section179Result {
        s179_max_after_phaseout,
        s179_deduction,
        s179_carryover,
        bonus_depreciation,
        remaining_macrs_basis,
        first_year_write_off,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }
    fn dc(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn base() -> Section179Input {
        Section179Input {
            total_qualifying_cost: Decimal::ZERO,
            trade_or_business_income: d(10_000_000),
            auto_cap: None,
        }
    }

    #[test]
    fn small_purchase_fully_section_179() {
        // $50k equipment, plenty of income → all $50k as § 179.
        let r = compute(Section179Input {
            total_qualifying_cost: d(50_000),
            ..base()
        });
        assert_eq!(r.s179_deduction, d(50_000));
        assert_eq!(r.bonus_depreciation, Decimal::ZERO);
        assert_eq!(r.first_year_write_off, d(50_000));
    }

    #[test]
    fn purchase_above_s179_cap_uses_bonus_on_remainder() {
        // $2M purchase. § 179 cap = $1.25M (no phaseout, $2M < $3.13M).
        // Remainder = $750k × 40% bonus = $300k.
        // Total first-year = $1.25M + $300k = $1.55M.
        let r = compute(Section179Input {
            total_qualifying_cost: d(2_000_000),
            ..base()
        });
        assert_eq!(r.s179_deduction, d(1_250_000));
        assert_eq!(r.bonus_depreciation, d(300_000));
        assert_eq!(r.first_year_write_off, d(1_550_000));
        assert_eq!(r.remaining_macrs_basis, d(450_000));
    }

    #[test]
    fn phaseout_dollar_for_dollar_above_threshold() {
        // $3.5M cost. Excess over threshold = $370k. § 179 max reduced
        // by $370k → $880k. § 179 deducted = $880k.
        // Remainder = $3.5M - $880k = $2.62M × 40% = $1.048M.
        let r = compute(Section179Input {
            total_qualifying_cost: d(3_500_000),
            ..base()
        });
        assert_eq!(r.s179_max_after_phaseout, d(880_000));
        assert_eq!(r.s179_deduction, d(880_000));
        assert_eq!(r.bonus_depreciation, d(1_048_000));
    }

    #[test]
    fn fully_phased_out_at_4_38m() {
        // At $4.38M: excess = $1.25M → § 179 max = $0.
        let r = compute(Section179Input {
            total_qualifying_cost: d(4_380_000),
            ..base()
        });
        assert_eq!(r.s179_max_after_phaseout, Decimal::ZERO);
        assert_eq!(r.s179_deduction, Decimal::ZERO);
        // All goes to bonus: $4.38M × 40% = $1.752M.
        assert_eq!(r.bonus_depreciation, dc("1752000"));
    }

    #[test]
    fn income_limit_caps_section_179_with_carryover() {
        // $100k purchase, only $30k business income.
        // § 179 election = $100k; income-limited to $30k.
        // Carryover = $70k. Remainder = $70k × 40% = $28k bonus.
        let r = compute(Section179Input {
            total_qualifying_cost: d(100_000),
            trade_or_business_income: d(30_000),
            auto_cap: None,
        });
        assert_eq!(r.s179_deduction, d(30_000));
        assert_eq!(r.s179_carryover, d(70_000));
        assert_eq!(r.bonus_depreciation, d(28_000));
    }

    #[test]
    fn negative_income_yields_zero_section_179() {
        let r = compute(Section179Input {
            total_qualifying_cost: d(100_000),
            trade_or_business_income: d(-5_000),
            auto_cap: None,
        });
        assert_eq!(r.s179_deduction, Decimal::ZERO);
        assert_eq!(r.s179_carryover, d(100_000));
        // Bonus still applies (no income limit on bonus).
        assert_eq!(r.bonus_depreciation, d(40_000));
    }

    #[test]
    fn auto_cap_constrains_s179() {
        // Heavy SUV with § 280F cap of $30,500. $80k purchase → § 179 capped at $30,500.
        let r = compute(Section179Input {
            total_qualifying_cost: d(80_000),
            trade_or_business_income: d(500_000),
            auto_cap: Some(d(30_500)),
        });
        assert_eq!(r.s179_deduction, d(30_500));
        // Remainder $49,500 × 40% = $19,800.
        assert_eq!(r.bonus_depreciation, dc("19800"));
    }

    #[test]
    fn zero_purchase_zero_everything() {
        let r = compute(Section179Input {
            total_qualifying_cost: Decimal::ZERO,
            ..base()
        });
        assert_eq!(r.s179_deduction, Decimal::ZERO);
        assert_eq!(r.bonus_depreciation, Decimal::ZERO);
        assert_eq!(r.first_year_write_off, Decimal::ZERO);
    }

    #[test]
    fn write_off_plus_basis_equals_total_cost() {
        // Invariant: write_off + remaining_macrs = total_cost.
        for cost in [50_000i64, 200_000, 1_500_000, 3_500_000] {
            let r = compute(Section179Input {
                total_qualifying_cost: d(cost),
                ..base()
            });
            assert_eq!(
                r.first_year_write_off + r.remaining_macrs_basis,
                d(cost),
                "invariant broken for cost {}",
                cost
            );
        }
    }

    #[test]
    fn bonus_rate_is_40pct_for_2025() {
        // Sanity check on the constant — guards against accidental update.
        let r = compute(Section179Input {
            total_qualifying_cost: d(10_000_000),
            ..base()
        });
        // With $10M cost, § 179 fully phased out at $4.38M+, so all goes to bonus.
        // Bonus = $10M × 40% = $4M.
        assert_eq!(r.bonus_depreciation, d(4_000_000));
    }
}
