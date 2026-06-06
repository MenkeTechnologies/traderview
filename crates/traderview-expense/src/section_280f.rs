//! IRC §280F — Luxury auto depreciation cap for listed property.
//!
//! §280F(a)(1) caps the annual depreciation deduction on a "passenger
//! automobile" used in a trade or business. Without the cap, MACRS
//! 5-year on a $60k vehicle would generate ~$12,000 of year-1
//! depreciation; §280F caps it at the published Rev. Proc. amount
//! for the placed-in-service year (e.g. $12,400 in 2024 without
//! §168(k) bonus, $20,400 WITH bonus — barely above the uncapped
//! number for a $60k vehicle, but a strict cliff for anything more
//! expensive).
//!
//! Dollar caps are annually inflation-adjusted by IRS revenue
//! procedure. The table here reflects the values cited in
//! Rev. Proc. 2020-37 through 2024-13 (or the values the caller
//! provides for years not yet in the static table — `caller_override`
//! exists for that purpose).
//!
//! Three structural elements modeled:
//!
//!   * **Year-by-year cap table** — passenger autos (sedans, vehicles
//!     under 6,000 lbs GVWR). Different caps apply to (a) year 1 with
//!     §168(k) bonus depreciation elected, (b) year 1 without bonus,
//!     (c) year 2, (d) year 3, and (e) year 4+ (the depreciation
//!     "tail" until basis is fully recovered).
//!
//!   * **§280F(d)(5) heavy-vehicle carve-out** — passenger automobile
//!     is defined to EXCLUDE vehicles with gross vehicle weight rating
//!     (GVWR) over 6,000 lbs. So light trucks, large SUVs, and
//!     commercial vans escape the §280F cap entirely. Caller asserts
//!     this via the `over_6000_lb_gvwr` flag; we skip cap computation.
//!
//!   * **Business-use percentage scaling** — §280F(b)(1): caps apply
//!     PROPORTIONALLY to business-use percentage. A 60% business-use
//!     auto in 2024 has a year-1 cap of $20,400 × 0.60 = $12,240.
//!     §280F(b)(2): if business use drops to ≤ 50% in a subsequent
//!     year, excess depreciation is RECAPTURED into ordinary income
//!     (recapture math out of scope here — caller models).
//!
//! Pure compute. Caller asserts placed-in-service year + cost basis + business-use percentage + bonus election + heavy-vehicle status; we return the year's depreciation cap and the actual capped deduction.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Published §280F(a)(1) caps per Rev. Proc. by placed-in-service year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassengerAutoCaps {
    pub year_1_with_bonus: Decimal,
    pub year_1_no_bonus: Decimal,
    pub year_2: Decimal,
    pub year_3: Decimal,
    /// Year 4 through end of recovery period.
    pub year_4_plus: Decimal,
    pub rev_proc_citation: String,
}

fn dollar(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// Static table for years with confirmed Rev. Proc. published caps.
/// Returns `None` for years not yet in the table — caller must
/// provide `caller_override`.
pub fn published_caps(placed_in_service_year: i32) -> Option<PassengerAutoCaps> {
    match placed_in_service_year {
        2020 => Some(PassengerAutoCaps {
            year_1_with_bonus: dollar("18100"),
            year_1_no_bonus: dollar("10100"),
            year_2: dollar("16100"),
            year_3: dollar("9700"),
            year_4_plus: dollar("5760"),
            rev_proc_citation: "Rev. Proc. 2020-37".into(),
        }),
        2021 => Some(PassengerAutoCaps {
            year_1_with_bonus: dollar("18200"),
            year_1_no_bonus: dollar("10200"),
            year_2: dollar("16400"),
            year_3: dollar("9800"),
            year_4_plus: dollar("5860"),
            rev_proc_citation: "Rev. Proc. 2021-31".into(),
        }),
        2022 => Some(PassengerAutoCaps {
            year_1_with_bonus: dollar("19200"),
            year_1_no_bonus: dollar("11200"),
            year_2: dollar("18000"),
            year_3: dollar("10800"),
            year_4_plus: dollar("6460"),
            rev_proc_citation: "Rev. Proc. 2022-17".into(),
        }),
        2023 => Some(PassengerAutoCaps {
            year_1_with_bonus: dollar("20200"),
            year_1_no_bonus: dollar("12200"),
            year_2: dollar("19500"),
            year_3: dollar("11700"),
            year_4_plus: dollar("6960"),
            rev_proc_citation: "Rev. Proc. 2023-14".into(),
        }),
        2024 => Some(PassengerAutoCaps {
            year_1_with_bonus: dollar("20400"),
            year_1_no_bonus: dollar("12400"),
            year_2: dollar("19800"),
            year_3: dollar("11900"),
            year_4_plus: dollar("7160"),
            rev_proc_citation: "Rev. Proc. 2024-13".into(),
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section280FInput {
    /// Cost basis of the vehicle (allocable to business per §280F(b)).
    pub cost_basis: Decimal,
    pub placed_in_service_year: i32,
    /// Tax year being computed (drives which cap row applies).
    pub current_tax_year: i32,
    /// Business-use percentage 0..1. Below 50% disables §168(k) bonus
    /// and triggers ADS depreciation per §280F(b)(1).
    pub business_use_pct: Decimal,
    /// True if §168(k) bonus elected on placement in service.
    pub elect_bonus_depreciation: bool,
    /// True if vehicle exceeds 6,000 lbs GVWR — §280F(d)(5) carve-out
    /// from the passenger-auto definition entirely.
    pub over_6000_lb_gvwr: bool,
    /// Caller-supplied caps for years not in the static published_caps
    /// table (2025+ at time of writing). When present, takes precedence.
    pub caller_override: Option<PassengerAutoCaps>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section280FResult {
    pub year_of_life: u32,
    pub heavy_vehicle_carve_out_applied: bool,
    pub applicable_cap_before_business_use: Decimal,
    pub applicable_cap_after_business_use: Decimal,
    /// What MACRS 5-year would have produced this year before the cap.
    pub uncapped_macrs_estimate: Decimal,
    pub allowed_deduction: Decimal,
    pub capped_amount_lost: Decimal,
    pub rev_proc_citation: String,
    pub note: String,
}

/// MACRS 5-year passenger-auto rates (200% DB switching to SL, half-
/// year convention). IRS Pub 946 Table A-1. Six published rates:
fn macrs_5yr_rate(year_of_life: u32) -> Decimal {
    match year_of_life {
        1 => dollar("0.2000"),
        2 => dollar("0.3200"),
        3 => dollar("0.1920"),
        4 => dollar("0.1152"),
        5 => dollar("0.1152"),
        6 => dollar("0.0576"),
        _ => Decimal::ZERO,
    }
}

pub fn compute(input: &Section280FInput) -> Section280FResult {
    let mut r = Section280FResult {
        year_of_life: (input.current_tax_year - input.placed_in_service_year + 1).max(0) as u32,
        ..Section280FResult::default()
    };

    let biz_pct = input.business_use_pct.clamp(Decimal::ZERO, Decimal::ONE);

    // §280F(d)(5) carve-out: > 6,000 lb GVWR escapes the passenger-auto
    // cap entirely. MACRS proceeds at the unmodified rate.
    if input.over_6000_lb_gvwr {
        r.heavy_vehicle_carve_out_applied = true;
        let macrs_rate = macrs_5yr_rate(r.year_of_life);
        let uncapped = (input.cost_basis * biz_pct * macrs_rate).round_dp(2);
        r.uncapped_macrs_estimate = uncapped;
        r.allowed_deduction = uncapped;
        r.note = format!(
            "§280F(d)(5) heavy-vehicle carve-out: > 6,000 lb GVWR exempt from passenger-auto cap. MACRS 5-yr {} produces ${} at {} business use.",
            r.year_of_life, r.allowed_deduction, biz_pct
        );
        return r;
    }

    // Pre-service year — nothing to deduct.
    if input.current_tax_year < input.placed_in_service_year {
        r.note = "current_tax_year before placed_in_service_year — no deduction".into();
        return r;
    }

    // Resolve cap table.
    let caps = match input
        .caller_override
        .clone()
        .or_else(|| published_caps(input.placed_in_service_year))
    {
        Some(c) => c,
        None => {
            r.note = format!(
                "no published §280F caps on file for placed-in-service year {} — pass caller_override",
                input.placed_in_service_year
            );
            return r;
        }
    };
    r.rev_proc_citation = caps.rev_proc_citation.clone();

    let cap = match r.year_of_life {
        1 if input.elect_bonus_depreciation => caps.year_1_with_bonus,
        1 => caps.year_1_no_bonus,
        2 => caps.year_2,
        3 => caps.year_3,
        4..=6 => caps.year_4_plus,
        _ => Decimal::ZERO,
    };
    r.applicable_cap_before_business_use = cap;
    r.applicable_cap_after_business_use = (cap * biz_pct).round_dp(2);

    let macrs_rate = macrs_5yr_rate(r.year_of_life);
    r.uncapped_macrs_estimate = (input.cost_basis * biz_pct * macrs_rate).round_dp(2);

    r.allowed_deduction = r
        .uncapped_macrs_estimate
        .min(r.applicable_cap_after_business_use);
    r.capped_amount_lost = (r.uncapped_macrs_estimate - r.allowed_deduction).max(Decimal::ZERO);

    r.note = if r.capped_amount_lost > Decimal::ZERO {
        format!(
            "§280F cap year {}: MACRS would have produced ${}, capped at ${} ({} business use). ${} deferred to tail years per Reg. §1.280F-2T. {}",
            r.year_of_life, r.uncapped_macrs_estimate, r.allowed_deduction, biz_pct, r.capped_amount_lost, r.rev_proc_citation,
        )
    } else {
        format!(
            "§280F cap year {}: ${} deduction under ${} cap ({} business use). {}",
            r.year_of_life,
            r.allowed_deduction,
            r.applicable_cap_after_business_use,
            biz_pct,
            r.rev_proc_citation,
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section280FInput {
        Section280FInput {
            cost_basis: dec!(60000),
            placed_in_service_year: 2024,
            current_tax_year: 2024,
            business_use_pct: Decimal::ONE,
            elect_bonus_depreciation: false,
            over_6000_lb_gvwr: false,
            caller_override: None,
        }
    }

    #[test]
    fn year_1_no_bonus_2024_caps_at_12400() {
        let r = compute(&base());
        // MACRS 20% × $60k = $12,000 uncapped. Cap = $12,400. No cap.
        assert_eq!(r.uncapped_macrs_estimate, dec!(12000));
        assert_eq!(r.applicable_cap_before_business_use, dec!(12400));
        assert_eq!(r.allowed_deduction, dec!(12000));
        assert_eq!(r.capped_amount_lost, Decimal::ZERO);
    }

    #[test]
    fn year_1_with_bonus_2024_caps_at_20400() {
        // Bonus would otherwise deduct most of the $60k basis. Cap
        // limits to $20,400 in 2024.
        let mut i = base();
        i.elect_bonus_depreciation = true;
        // Note: our MACRS rate for year 1 is 20% — bonus depreciation
        // sits on top, which we don't separately model here. For the
        // cap test, we just verify the cap value resolved correctly.
        let r = compute(&i);
        assert_eq!(r.applicable_cap_before_business_use, dec!(20400));
    }

    #[test]
    fn year_2_cap_2024_caps_at_19800() {
        let mut i = base();
        i.current_tax_year = 2025;
        let r = compute(&i);
        assert_eq!(r.year_of_life, 2);
        assert_eq!(r.applicable_cap_before_business_use, dec!(19800));
        // MACRS 32% × $60k = $19,200 uncapped. Under cap.
        assert_eq!(r.uncapped_macrs_estimate, dec!(19200));
        assert_eq!(r.allowed_deduction, dec!(19200));
    }

    #[test]
    fn year_3_cap_2024_caps_at_11900() {
        let mut i = base();
        i.current_tax_year = 2026;
        let r = compute(&i);
        assert_eq!(r.year_of_life, 3);
        assert_eq!(r.applicable_cap_before_business_use, dec!(11900));
        // MACRS 19.20% × $60k = $11,520. Under cap.
        assert_eq!(r.uncapped_macrs_estimate, dec!(11520));
    }

    #[test]
    fn year_4_cap_2024_caps_at_7160() {
        let mut i = base();
        i.current_tax_year = 2027;
        let r = compute(&i);
        assert_eq!(r.year_of_life, 4);
        assert_eq!(r.applicable_cap_before_business_use, dec!(7160));
        // MACRS 11.52% × $60k = $6,912. Under cap.
        assert_eq!(r.uncapped_macrs_estimate, dec!(6912));
    }

    #[test]
    fn expensive_vehicle_capped_at_published_amount() {
        let mut i = base();
        i.cost_basis = dec!(150000); // luxury sedan
        let r = compute(&i);
        // MACRS 20% × $150k = $30,000 uncapped. Cap = $12,400. Big loss.
        assert_eq!(r.uncapped_macrs_estimate, dec!(30000));
        assert_eq!(r.allowed_deduction, dec!(12400));
        assert_eq!(r.capped_amount_lost, dec!(17600));
    }

    #[test]
    fn heavy_vehicle_over_6000_lb_skips_cap() {
        let mut i = base();
        i.cost_basis = dec!(100000); // large SUV
        i.over_6000_lb_gvwr = true;
        let r = compute(&i);
        assert!(r.heavy_vehicle_carve_out_applied);
        // MACRS 20% × $100k = $20,000 — uncapped.
        assert_eq!(r.allowed_deduction, dec!(20000));
        assert_eq!(r.capped_amount_lost, Decimal::ZERO);
    }

    #[test]
    fn business_use_60pct_scales_cap_proportionally() {
        let mut i = base();
        i.business_use_pct = dec!(0.6);
        let r = compute(&i);
        // Cap = $12,400 × 0.6 = $7,440.
        assert_eq!(r.applicable_cap_after_business_use, dec!(7440));
        // MACRS basis × 0.6 × 20% = $60k × 0.6 × 0.20 = $7,200. Under
        // adjusted cap.
        assert_eq!(r.uncapped_macrs_estimate, dec!(7200));
    }

    #[test]
    fn business_use_zero_no_deduction() {
        let mut i = base();
        i.business_use_pct = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.allowed_deduction, Decimal::ZERO);
    }

    #[test]
    fn business_use_above_one_clamps_to_one() {
        let mut i = base();
        i.business_use_pct = dec!(1.5);
        let r = compute(&i);
        // Should treat as 100% — same as base case.
        assert_eq!(r.allowed_deduction, dec!(12000));
    }

    #[test]
    fn published_caps_table_2020_through_2024() {
        for (year, year_1_no_bonus) in [
            (2020, dec!(10100)),
            (2021, dec!(10200)),
            (2022, dec!(11200)),
            (2023, dec!(12200)),
            (2024, dec!(12400)),
        ] {
            let caps =
                published_caps(year).unwrap_or_else(|| panic!("year {year} should have caps"));
            assert_eq!(caps.year_1_no_bonus, year_1_no_bonus, "year {}", year);
        }
    }

    #[test]
    fn unknown_year_returns_none_and_caller_override_path() {
        assert!(published_caps(2099).is_none());

        let mut i = base();
        i.placed_in_service_year = 2099;
        i.current_tax_year = 2099;
        let r_no_override = compute(&i);
        assert!(r_no_override.note.contains("no published §280F caps"));

        i.caller_override = Some(PassengerAutoCaps {
            year_1_with_bonus: dec!(25000),
            year_1_no_bonus: dec!(15000),
            year_2: dec!(22000),
            year_3: dec!(13500),
            year_4_plus: dec!(8000),
            rev_proc_citation: "Rev. Proc. 2099-XX".into(),
        });
        let r_with_override = compute(&i);
        assert_eq!(
            r_with_override.applicable_cap_before_business_use,
            dec!(15000)
        );
        assert_eq!(r_with_override.rev_proc_citation, "Rev. Proc. 2099-XX");
    }

    #[test]
    fn caller_override_takes_precedence_over_published_table() {
        let mut i = base();
        i.caller_override = Some(PassengerAutoCaps {
            year_1_with_bonus: dec!(99999),
            year_1_no_bonus: dec!(99999),
            year_2: dec!(99999),
            year_3: dec!(99999),
            year_4_plus: dec!(99999),
            rev_proc_citation: "USER_OVERRIDE".into(),
        });
        let r = compute(&i);
        assert_eq!(r.applicable_cap_before_business_use, dec!(99999));
        assert_eq!(r.rev_proc_citation, "USER_OVERRIDE");
    }

    #[test]
    fn pre_service_year_no_deduction() {
        let mut i = base();
        i.current_tax_year = 2023; // before 2024 placed-in-service
        let r = compute(&i);
        assert_eq!(r.allowed_deduction, Decimal::ZERO);
        assert!(r.note.contains("before"));
    }

    #[test]
    fn macrs_rates_match_pub_946_table_a1() {
        assert_eq!(macrs_5yr_rate(1), dec!(0.2000));
        assert_eq!(macrs_5yr_rate(2), dec!(0.3200));
        assert_eq!(macrs_5yr_rate(3), dec!(0.1920));
        assert_eq!(macrs_5yr_rate(4), dec!(0.1152));
        assert_eq!(macrs_5yr_rate(5), dec!(0.1152));
        assert_eq!(macrs_5yr_rate(6), dec!(0.0576));
        assert_eq!(macrs_5yr_rate(7), Decimal::ZERO);
    }

    #[test]
    fn year_5_and_6_use_year_4_plus_cap() {
        let mut i = base();
        i.current_tax_year = 2028; // year 5
        let r5 = compute(&i);
        assert_eq!(r5.applicable_cap_before_business_use, dec!(7160));

        i.current_tax_year = 2029; // year 6 — tail
        let r6 = compute(&i);
        assert_eq!(r6.applicable_cap_before_business_use, dec!(7160));
    }

    #[test]
    fn capped_amount_lost_calculated_correctly() {
        let mut i = base();
        i.cost_basis = dec!(100000);
        let r = compute(&i);
        // Uncapped = $20,000. Cap = $12,400. Lost = $7,600.
        assert_eq!(r.capped_amount_lost, dec!(7600));
    }
}
