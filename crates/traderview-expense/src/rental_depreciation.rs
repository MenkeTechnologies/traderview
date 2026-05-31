//! MACRS depreciation for rental real property (Schedule E line 18).
//!
//! Two recovery periods cover the field:
//!   * Residential rental property — 27.5 years straight-line, mid-month
//!     convention. IRS Pub 946 Table A-6.
//!   * Non-residential (commercial) — 39 years straight-line, mid-month
//!     convention. IRS Pub 946 Table A-7a.
//!
//! "Mid-month convention" means the placed-in-service month counts as
//! half a month, so year-1 depreciation depends on which month the
//! property went into service. After year 1 a full year's depreciation
//! is taken until the final year, when only the remaining basis (also
//! a partial month) is recovered.
//!
//! Depreciable basis = purchase_price - land_value - prior depreciation.
//! Land is never depreciable (IRS Pub 527).
//!
//! Pure compute, no IO. Caller is responsible for: (a) subtracting land
//! value from the cost basis, (b) tracking prior-year deductions so we
//! don't over-depreciate past the basis.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealPropertyClass {
    /// Residential rental property — 27.5y SL, Pub 946 Table A-6.
    Residential27_5,
    /// Non-residential real property — 39y SL, Pub 946 Table A-7a.
    Commercial39,
}

impl RealPropertyClass {
    pub fn recovery_period_years(self) -> Decimal {
        match self {
            RealPropertyClass::Residential27_5 => Decimal::from_str("27.5").unwrap(),
            RealPropertyClass::Commercial39   => Decimal::from(39),
        }
    }
}

/// IRS Pub 946 Table A-6 — Residential rental property, 27.5-year SL,
/// mid-month convention. Year-1 percentages indexed by placed-in-service
/// month (1=Jan .. 12=Dec). Values from the IRS table (e.g. 3.485% = 0.03485).
const TABLE_A6_RESIDENTIAL_YEAR_1: [&str; 12] = [
    "0.03485", "0.03182", "0.02879", "0.02576", "0.02273", "0.01970",
    "0.01667", "0.01364", "0.01061", "0.00758", "0.00455", "0.00152",
];

/// IRS Pub 946 Table A-7a — Non-residential real property, 39-year SL,
/// mid-month convention. Year-1 percentages indexed by month.
const TABLE_A7A_COMMERCIAL_YEAR_1: [&str; 12] = [
    "0.02461", "0.02247", "0.02033", "0.01819", "0.01605", "0.01391",
    "0.01177", "0.00963", "0.00749", "0.00535", "0.00321", "0.00107",
];

/// Year-2-through-last full-recovery-year percentage for 27.5y residential.
/// 1 / 27.5 = 0.03636 (rounded to IRS table precision).
const MID_LIFE_RESIDENTIAL: &str = "0.03636";

/// Year-2-through-last full-recovery-year percentage for 39y commercial.
/// 1 / 39 = 0.02564 (rounded to IRS table precision).
const MID_LIFE_COMMERCIAL: &str = "0.02564";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentalPropertyDepreciation {
    /// Depreciable basis: usually `purchase_price - land_value`. Land is
    /// never depreciable so the caller subtracts it out before passing in.
    pub depreciable_basis: Decimal,
    pub class: RealPropertyClass,
    pub placed_in_service_year: i32,
    pub placed_in_service_month: u32, // 1..=12
    /// The tax year we're computing depreciation for.
    pub current_tax_year: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DepreciationForYear {
    /// Year-of-life this calculation is for (1 for first calendar year
    /// of service, 2 for second, etc.). Returned for transparency.
    pub year_of_life: u32,
    pub deduction: Decimal,
    /// Cumulative depreciation taken from year-of-service through the
    /// requested tax year.
    pub cumulative_through_year: Decimal,
    /// Basis remaining after the requested tax year.
    pub basis_remaining: Decimal,
    /// Why the deduction is zero — placed-in-service after tax year,
    /// fully depreciated, etc. Empty when a real deduction was taken.
    pub note: String,
}

fn pct(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn year_1_pct(class: RealPropertyClass, month: u32) -> Decimal {
    let m = month.clamp(1, 12) as usize - 1;
    pct(match class {
        RealPropertyClass::Residential27_5 => TABLE_A6_RESIDENTIAL_YEAR_1[m],
        RealPropertyClass::Commercial39   => TABLE_A7A_COMMERCIAL_YEAR_1[m],
    })
}

fn mid_life_pct(class: RealPropertyClass) -> Decimal {
    pct(match class {
        RealPropertyClass::Residential27_5 => MID_LIFE_RESIDENTIAL,
        RealPropertyClass::Commercial39   => MID_LIFE_COMMERCIAL,
    })
}

fn life_in_calendar_years(class: RealPropertyClass) -> u32 {
    // Mid-month convention means recovery actually stretches across
    // ⌈life⌉ + 1 calendar years (a sliver in year 1 + full years +
    // the leftover sliver in the final year).
    match class {
        RealPropertyClass::Residential27_5 => 28 + 1, // 29 calendar years touched
        RealPropertyClass::Commercial39    => 40 + 1, // 41 calendar years touched
    }
}

/// Compute the depreciation deduction for a single tax year.
///
/// Handles three cases:
///   1. Tax year < placed-in-service year → 0.
///   2. Tax year = year-1, year-2, ..., year-N → table rate × basis.
///   3. Tax year > recovery exhausted → 0, note explains.
///
/// Cumulative-through-year tracks the running total so the caller can
/// check basis-remaining without re-running every prior year.
pub fn macrs_rental_year(p: &RentalPropertyDepreciation) -> DepreciationForYear {
    let mut out = DepreciationForYear::default();
    if p.depreciable_basis <= Decimal::ZERO {
        out.note = "no depreciable basis (cost = land?)".into();
        return out;
    }
    if p.current_tax_year < p.placed_in_service_year {
        out.note = "not yet placed in service".into();
        out.basis_remaining = p.depreciable_basis;
        return out;
    }
    let year_of_life = (p.current_tax_year - p.placed_in_service_year) as u32 + 1;
    out.year_of_life = year_of_life;
    let max_calendar_year = life_in_calendar_years(p.class);
    if year_of_life > max_calendar_year {
        out.note = "fully depreciated".into();
        out.cumulative_through_year = p.depreciable_basis;
        return out;
    }

    // Sum up table rates for each year from 1 through the requested year.
    let mut cumulative = Decimal::ZERO;
    let mid = mid_life_pct(p.class);
    for yol in 1..=year_of_life {
        if yol == 1 {
            cumulative += year_1_pct(p.class, p.placed_in_service_month);
        } else if yol == max_calendar_year {
            // Final year — only whatever fraction is left after the
            // year-1 partial + (max_calendar_year - 2) full mid-life
            // years. Stay non-negative.
            let total_so_far = year_1_pct(p.class, p.placed_in_service_month)
                + mid * Decimal::from(max_calendar_year - 2);
            let leftover = (Decimal::ONE - total_so_far).max(Decimal::ZERO);
            cumulative += leftover;
        } else {
            cumulative += mid;
        }
    }
    // Cap cumulative at 100% so rounding can't over-deduct.
    if cumulative > Decimal::ONE {
        cumulative = Decimal::ONE;
    }

    let cum_dollars = (p.depreciable_basis * cumulative).round_dp(2);

    // Year's deduction = this year's cumulative minus prior year's cumulative.
    let prior_cum_pct = if year_of_life == 1 {
        Decimal::ZERO
    } else {
        // Re-derive the running pct excluding the requested year.
        let mut c = Decimal::ZERO;
        for yol in 1..year_of_life {
            if yol == 1 {
                c += year_1_pct(p.class, p.placed_in_service_month);
            } else if yol == max_calendar_year {
                let total_so_far = year_1_pct(p.class, p.placed_in_service_month)
                    + mid * Decimal::from(max_calendar_year - 2);
                c += (Decimal::ONE - total_so_far).max(Decimal::ZERO);
            } else {
                c += mid;
            }
        }
        if c > Decimal::ONE { Decimal::ONE } else { c }
    };
    let prior_dollars = (p.depreciable_basis * prior_cum_pct).round_dp(2);

    out.deduction = (cum_dollars - prior_dollars).max(Decimal::ZERO);
    out.cumulative_through_year = cum_dollars;
    out.basis_remaining = (p.depreciable_basis - cum_dollars).max(Decimal::ZERO);
    out
}

/// Convenience: just the year's deduction. Used by the Schedule E API
/// roll-up to fill `depreciation_for_year` per property.
pub fn macrs_rental_year_1_deduction(
    depreciable_basis: Decimal,
    class: RealPropertyClass,
    placed_in_service_year: i32,
    placed_in_service_month: u32,
    current_tax_year: i32,
) -> Decimal {
    macrs_rental_year(&RentalPropertyDepreciation {
        depreciable_basis,
        class,
        placed_in_service_year,
        placed_in_service_month,
        current_tax_year,
    })
    .deduction
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn residential_year_1_january_matches_pub946_table_a6() {
        // $275,000 basis × 3.485% = $9,583.75
        let r = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: dec!(275000),
            class: RealPropertyClass::Residential27_5,
            placed_in_service_year: 2024,
            placed_in_service_month: 1,
            current_tax_year: 2024,
        });
        assert_eq!(r.deduction, dec!(9583.75));
        assert_eq!(r.year_of_life, 1);
    }

    #[test]
    fn residential_year_1_december_matches_pub946_table_a6() {
        // $275,000 basis × 0.152% = $418.00
        let r = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: dec!(275000),
            class: RealPropertyClass::Residential27_5,
            placed_in_service_year: 2024,
            placed_in_service_month: 12,
            current_tax_year: 2024,
        });
        assert_eq!(r.deduction, dec!(418.00));
    }

    #[test]
    fn residential_year_2_full_year_at_mid_life_pct() {
        // Year 2 onwards: $275,000 × 3.636% = $10,000 (rounded to dollar).
        let r = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: dec!(275000),
            class: RealPropertyClass::Residential27_5,
            placed_in_service_year: 2024,
            placed_in_service_month: 1,
            current_tax_year: 2025,
        });
        assert_eq!(r.deduction, dec!(9999.00));
        assert_eq!(r.year_of_life, 2);
    }

    #[test]
    fn commercial_year_1_july_matches_pub946_table_a7a() {
        // $390,000 × 1.177% = $4,590.30
        let r = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: dec!(390000),
            class: RealPropertyClass::Commercial39,
            placed_in_service_year: 2024,
            placed_in_service_month: 7,
            current_tax_year: 2024,
        });
        assert_eq!(r.deduction, dec!(4590.30));
    }

    #[test]
    fn pre_service_year_returns_zero() {
        let r = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: dec!(275000),
            class: RealPropertyClass::Residential27_5,
            placed_in_service_year: 2026,
            placed_in_service_month: 1,
            current_tax_year: 2024,
        });
        assert_eq!(r.deduction, Decimal::ZERO);
        assert_eq!(r.basis_remaining, dec!(275000));
        assert!(r.note.contains("not yet"));
    }

    #[test]
    fn post_recovery_year_returns_zero() {
        // Residential touches 29 calendar years (yol 1..=29). Year 30 is zero.
        let r = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: dec!(275000),
            class: RealPropertyClass::Residential27_5,
            placed_in_service_year: 2000,
            placed_in_service_month: 1,
            current_tax_year: 2030, // year-of-life 31
        });
        assert_eq!(r.deduction, Decimal::ZERO);
        assert!(r.note.contains("fully depreciated"));
    }

    #[test]
    fn cumulative_recovery_does_not_exceed_basis() {
        // Walk a $275k residential property through years 1..=29 and
        // verify cumulative depreciation never exceeds basis. This is
        // the load-bearing invariant — rounding bugs here over-deduct.
        for yol in 1..=29 {
            let tax_year = 2024 + yol - 1;
            let r = macrs_rental_year(&RentalPropertyDepreciation {
                depreciable_basis: dec!(275000),
                class: RealPropertyClass::Residential27_5,
                placed_in_service_year: 2024,
                placed_in_service_month: 1,
                current_tax_year: tax_year,
            });
            assert!(
                r.cumulative_through_year <= dec!(275000),
                "cumulative {} exceeds basis at year of life {}",
                r.cumulative_through_year, yol
            );
            assert!(r.basis_remaining >= Decimal::ZERO);
        }
    }

    #[test]
    fn convenience_deduction_helper_matches_full_compute() {
        let basis = dec!(275000);
        let helper = macrs_rental_year_1_deduction(
            basis,
            RealPropertyClass::Residential27_5,
            2024, 6, 2024,
        );
        let full = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: basis,
            class: RealPropertyClass::Residential27_5,
            placed_in_service_year: 2024,
            placed_in_service_month: 6,
            current_tax_year: 2024,
        }).deduction;
        assert_eq!(helper, full);
    }

    #[test]
    fn zero_basis_returns_zero_no_panic() {
        let r = macrs_rental_year(&RentalPropertyDepreciation {
            depreciable_basis: Decimal::ZERO,
            class: RealPropertyClass::Residential27_5,
            placed_in_service_year: 2024,
            placed_in_service_month: 1,
            current_tax_year: 2024,
        });
        assert_eq!(r.deduction, Decimal::ZERO);
    }

    #[test]
    fn recovery_period_years_match_27_5_and_39() {
        assert_eq!(
            RealPropertyClass::Residential27_5.recovery_period_years(),
            Decimal::from_str("27.5").unwrap()
        );
        assert_eq!(
            RealPropertyClass::Commercial39.recovery_period_years(),
            Decimal::from(39)
        );
    }
}
