//! IRC § 168 Modified Accelerated Cost Recovery System (MACRS) —
//! the general depreciation provision for property placed in
//! service after December 31, 1986.
//!
//! MACRS replaced the prior ACRS (Accelerated Cost Recovery System)
//! under the Tax Reform Act of 1986. § 168 establishes three
//! determinative variables that combine into the annual
//! depreciation deduction: (1) applicable depreciation method,
//! (2) applicable recovery period, (3) applicable convention.
//!
//! **§ 168(a) general rule**: depreciation deduction equals the
//! product of (i) applicable depreciation method × (ii) applicable
//! recovery period × (iii) applicable convention. Practical
//! computation uses IRS Pub. 946 percentage tables that combine
//! all three into ready-made annual percentages.
//!
//! **§ 168(b) applicable depreciation method**:
//!
//! - **§ 168(b)(1) 200% declining balance** (default for personal
//!   property): 3-year, 5-year, 7-year, and 10-year property
//!   classes. Switches to straight-line when SL yields larger
//!   allowance.
//! - **§ 168(b)(2)(B) 150% declining balance**: 15-year and 20-year
//!   property, qualified smart electric meters, qualified smart
//!   electric grid systems, and elective application.
//! - **§ 168(b)(3) straight-line method (required)**: residential
//!   rental (27.5 years), nonresidential real (39 years), and any
//!   property where taxpayer elects straight-line under § 168(b)(5).
//!
//! **§ 168(c) applicable recovery periods**:
//!
//! - **3-year**: tractors, racehorses 2 years or older, certain
//!   property used in connection with research.
//! - **5-year**: computers, autos, light trucks, qualified
//!   technological equipment.
//! - **7-year**: office furniture and equipment.
//! - **10-year**: water transportation equipment.
//! - **15-year**: qualified improvement property (QIP — TCJA
//!   technical correction applied 2018+).
//! - **20-year**: farm buildings.
//! - **25-year**: water utility property.
//! - **27.5-year**: residential rental property.
//! - **39-year**: nonresidential real property.
//!
//! **§ 168(d) applicable convention**:
//!
//! - **§ 168(d)(1) half-year convention** (default for personal
//!   property): half-year deduction in first and last years.
//! - **§ 168(d)(3) mid-quarter convention**: triggered when more
//!   than **40%** of personal property is placed in service in the
//!   last quarter of the taxable year.
//! - **§ 168(d)(2) mid-month convention** (real property): mid-
//!   month placement assumed; first-year deduction depends on
//!   month placed in service.
//!
//! **§ 168(e) classification of property**:
//!
//! - **§ 168(e)(2)(A) residential rental**: any building or
//!   structure from which 80% or more of the gross rental income
//!   for the tax year is from dwelling units.
//! - **§ 168(e)(2)(B) nonresidential real**: § 1250 property that
//!   is not residential rental and has class life of at least 27.5
//!   years.
//!
//! **§ 168(g) Alternative Depreciation System (ADS)**: straight-line
//! over class life. Applies to tax-exempt use property, listed
//! property used less than 50% in trade or business, foreign-use
//! property, and electively for any property class.
//!
//! **§ 168(k) bonus depreciation** (separate iter 168k module):
//! additional first-year depreciation for qualified property —
//! 100% under TCJA 2017 through 2022, phased down 80%/60%/40%/20%
//! through 2026, restored to 100% permanent by OBBBA 2025.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const RESIDENTIAL_RENTAL_RECOVERY_PERIOD_X_10: u32 = 275;
#[allow(dead_code)]
pub const NONRESIDENTIAL_REAL_RECOVERY_PERIOD_YEARS: u32 = 39;
#[allow(dead_code)]
pub const METHOD_200_PERCENT_DB: u32 = 200;
#[allow(dead_code)]
pub const METHOD_150_PERCENT_DB: u32 = 150;
#[allow(dead_code)]
pub const MID_QUARTER_CONVENTION_THRESHOLD_PERCENT: u32 = 40;
#[allow(dead_code)]
pub const RESIDENTIAL_RENTAL_GROSS_RENT_FROM_DWELLING_THRESHOLD_PERCENT: u32 = 80;
#[allow(dead_code)]
pub const MACRS_EFFECTIVE_DATE_YEAR: u32 = 1986;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyClass {
    ThreeYear,
    FiveYear,
    SevenYear,
    TenYear,
    FifteenYear,
    TwentyYear,
    TwentyFiveYearWaterUtility,
    ResidentialRental27_5,
    NonresidentialReal39,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    DecliningBalance200,
    DecliningBalance150,
    StraightLine,
    AdsStraightLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Convention {
    HalfYear,
    MidQuarter,
    MidMonth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Method200DBPersonalProperty3To10Years,
    Method150DBPersonalProperty15To20Years,
    MethodStraightLineResidentialRental27_5,
    MethodStraightLineNonresidentialReal39,
    MethodElectiveStraightLineOnPersonalProperty,
    MethodAdsStraightLineOverClassLife,
    ConventionMidQuarterTriggeredBy40PctLastQuarter,
    ViolationResidentialRentalClassifiedWithUnder80PctDwelling,
    ViolationMissingMidQuarterConventionWhen40PctTriggered,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub property_class: PropertyClass,
    pub last_quarter_placement_aggregate_pct: u32,
    pub gross_rental_income_pct_from_dwelling_units: u32,
    pub taxpayer_elected_straight_line: bool,
    pub taxpayer_elected_ads: bool,
    pub adjusted_basis_cents: u64,
    pub taxpayer_applied_mid_quarter_when_triggered: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub applicable_method: Method,
    pub applicable_recovery_period_years_x_10: u32,
    pub applicable_convention: Convention,
    pub first_year_annual_rate_pct_x_10: u32,
    pub first_year_deduction_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section168Input = Input;
pub type Section168Output = Output;
pub type Section168Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 168(a) (general rule — MACRS deduction formula)".to_string(),
        "IRC § 168(b)(1) (200% declining balance default for personal property)".to_string(),
        "IRC § 168(b)(2)(B) (150% declining balance for 15/20-year property)".to_string(),
        "IRC § 168(b)(3) (straight-line required for real property)".to_string(),
        "IRC § 168(b)(5) (elective straight-line)".to_string(),
        "IRC § 168(c) (applicable recovery periods)".to_string(),
        "IRC § 168(d)(1) (half-year convention default for personal property)".to_string(),
        "IRC § 168(d)(2) (mid-month convention for real property)".to_string(),
        "IRC § 168(d)(3) (mid-quarter convention — 40% last-quarter trigger)".to_string(),
        "IRC § 168(e)(2)(A) (residential rental 80% dwelling-income threshold)".to_string(),
        "IRC § 168(e)(2)(B) (nonresidential real property)".to_string(),
        "IRC § 168(g) (Alternative Depreciation System — straight-line over class life)"
            .to_string(),
        "IRC § 168(k) (bonus depreciation — separate provision)".to_string(),
        "Tax Reform Act of 1986 (Pub. L. 99-514) — MACRS enactment".to_string(),
        "IRS Publication 946 (MACRS percentage tables)".to_string(),
    ];

    if matches!(input.property_class, PropertyClass::ResidentialRental27_5)
        && input.gross_rental_income_pct_from_dwelling_units
            < RESIDENTIAL_RENTAL_GROSS_RENT_FROM_DWELLING_THRESHOLD_PERCENT
    {
        notes.push(format!(
            "Property classified residential rental but only {}% of gross rental income is from dwelling units (< {}% threshold) — § 168(e)(2)(A) violation.",
            input.gross_rental_income_pct_from_dwelling_units,
            RESIDENTIAL_RENTAL_GROSS_RENT_FROM_DWELLING_THRESHOLD_PERCENT
        ));
        return Output {
            severity: Severity::ViolationResidentialRentalClassifiedWithUnder80PctDwelling,
            applicable_method: Method::StraightLine,
            applicable_recovery_period_years_x_10: RESIDENTIAL_RENTAL_RECOVERY_PERIOD_X_10,
            applicable_convention: Convention::MidMonth,
            first_year_annual_rate_pct_x_10: 0,
            first_year_deduction_cents: 0,
            notes,
            citations,
        };
    }

    if input.taxpayer_elected_ads {
        let recovery_period_x_10 = recovery_period_x_10(input.property_class);
        notes.push(format!(
            "§ 168(g) ADS elected: straight-line over {}.{} year class life.",
            recovery_period_x_10 / 10,
            recovery_period_x_10 % 10
        ));
        return Output {
            severity: Severity::MethodAdsStraightLineOverClassLife,
            applicable_method: Method::AdsStraightLine,
            applicable_recovery_period_years_x_10: recovery_period_x_10,
            applicable_convention: match input.property_class {
                PropertyClass::ResidentialRental27_5 | PropertyClass::NonresidentialReal39 => {
                    Convention::MidMonth
                }
                _ => Convention::HalfYear,
            },
            first_year_annual_rate_pct_x_10: 0,
            first_year_deduction_cents: 0,
            notes,
            citations,
        };
    }

    match input.property_class {
        PropertyClass::ResidentialRental27_5 => {
            let annual_rate_pct_x_10 = 36; // ~3.636% annual after mid-month convention
            let first_year_deduction = input
                .adjusted_basis_cents
                .saturating_mul(annual_rate_pct_x_10 as u64)
                / 1_000;
            notes.push(format!(
                "Residential rental 27.5-year straight-line + mid-month convention. First-year approximate rate {}.{}%.",
                annual_rate_pct_x_10 / 10,
                annual_rate_pct_x_10 % 10
            ));
            Output {
                severity: Severity::MethodStraightLineResidentialRental27_5,
                applicable_method: Method::StraightLine,
                applicable_recovery_period_years_x_10: RESIDENTIAL_RENTAL_RECOVERY_PERIOD_X_10,
                applicable_convention: Convention::MidMonth,
                first_year_annual_rate_pct_x_10: annual_rate_pct_x_10,
                first_year_deduction_cents: first_year_deduction,
                notes,
                citations,
            }
        }
        PropertyClass::NonresidentialReal39 => {
            let annual_rate_pct_x_10 = 26; // ~2.564% annual
            let first_year_deduction = input
                .adjusted_basis_cents
                .saturating_mul(annual_rate_pct_x_10 as u64)
                / 1_000;
            notes.push(format!(
                "Nonresidential real 39-year straight-line + mid-month convention. First-year approximate rate {}.{}%.",
                annual_rate_pct_x_10 / 10,
                annual_rate_pct_x_10 % 10
            ));
            Output {
                severity: Severity::MethodStraightLineNonresidentialReal39,
                applicable_method: Method::StraightLine,
                applicable_recovery_period_years_x_10: NONRESIDENTIAL_REAL_RECOVERY_PERIOD_YEARS
                    * 10,
                applicable_convention: Convention::MidMonth,
                first_year_annual_rate_pct_x_10: annual_rate_pct_x_10,
                first_year_deduction_cents: first_year_deduction,
                notes,
                citations,
            }
        }
        _ => {
            let mid_quarter_triggered = input.last_quarter_placement_aggregate_pct
                > MID_QUARTER_CONVENTION_THRESHOLD_PERCENT;
            if mid_quarter_triggered && !input.taxpayer_applied_mid_quarter_when_triggered {
                notes.push(format!(
                    "{}% of personal property placed in last quarter (> {}% threshold) — § 168(d)(3) mid-quarter convention required; taxpayer applied half-year.",
                    input.last_quarter_placement_aggregate_pct,
                    MID_QUARTER_CONVENTION_THRESHOLD_PERCENT
                ));
                return Output {
                    severity: Severity::ViolationMissingMidQuarterConventionWhen40PctTriggered,
                    applicable_method: Method::DecliningBalance200,
                    applicable_recovery_period_years_x_10: recovery_period_x_10(
                        input.property_class,
                    ),
                    applicable_convention: Convention::MidQuarter,
                    first_year_annual_rate_pct_x_10: 0,
                    first_year_deduction_cents: 0,
                    notes,
                    citations,
                };
            }
            let convention = if mid_quarter_triggered {
                Convention::MidQuarter
            } else {
                Convention::HalfYear
            };
            if input.taxpayer_elected_straight_line {
                notes.push(format!(
                    "§ 168(b)(5) elective straight-line over recovery period {}.{} years.",
                    recovery_period_x_10(input.property_class) / 10,
                    recovery_period_x_10(input.property_class) % 10
                ));
                return Output {
                    severity: Severity::MethodElectiveStraightLineOnPersonalProperty,
                    applicable_method: Method::StraightLine,
                    applicable_recovery_period_years_x_10: recovery_period_x_10(
                        input.property_class,
                    ),
                    applicable_convention: convention,
                    first_year_annual_rate_pct_x_10: 0,
                    first_year_deduction_cents: 0,
                    notes,
                    citations,
                };
            }
            let is_15_or_20_year = matches!(
                input.property_class,
                PropertyClass::FifteenYear | PropertyClass::TwentyYear
            );
            let method = if is_15_or_20_year {
                Method::DecliningBalance150
            } else {
                Method::DecliningBalance200
            };
            if mid_quarter_triggered {
                notes.push(format!(
                    "Mid-quarter convention triggered ({}% > {}% threshold).",
                    input.last_quarter_placement_aggregate_pct,
                    MID_QUARTER_CONVENTION_THRESHOLD_PERCENT
                ));
                return Output {
                    severity: Severity::ConventionMidQuarterTriggeredBy40PctLastQuarter,
                    applicable_method: method,
                    applicable_recovery_period_years_x_10: recovery_period_x_10(
                        input.property_class,
                    ),
                    applicable_convention: convention,
                    first_year_annual_rate_pct_x_10: 0,
                    first_year_deduction_cents: 0,
                    notes,
                    citations,
                };
            }
            let severity = if is_15_or_20_year {
                notes.push("§ 168(b)(2)(B) 150% declining balance + half-year convention for 15/20-year property.".to_string());
                Severity::Method150DBPersonalProperty15To20Years
            } else {
                notes.push("§ 168(b)(1) 200% declining balance + half-year convention for 3/5/7/10-year property.".to_string());
                Severity::Method200DBPersonalProperty3To10Years
            };
            Output {
                severity,
                applicable_method: method,
                applicable_recovery_period_years_x_10: recovery_period_x_10(input.property_class),
                applicable_convention: convention,
                first_year_annual_rate_pct_x_10: 0,
                first_year_deduction_cents: 0,
                notes,
                citations,
            }
        }
    }
}

fn recovery_period_x_10(p: PropertyClass) -> u32 {
    match p {
        PropertyClass::ThreeYear => 30,
        PropertyClass::FiveYear => 50,
        PropertyClass::SevenYear => 70,
        PropertyClass::TenYear => 100,
        PropertyClass::FifteenYear => 150,
        PropertyClass::TwentyYear => 200,
        PropertyClass::TwentyFiveYearWaterUtility => 250,
        PropertyClass::ResidentialRental27_5 => RESIDENTIAL_RENTAL_RECOVERY_PERIOD_X_10,
        PropertyClass::NonresidentialReal39 => NONRESIDENTIAL_REAL_RECOVERY_PERIOD_YEARS * 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_residential() -> Input {
        Input {
            property_class: PropertyClass::ResidentialRental27_5,
            last_quarter_placement_aggregate_pct: 0,
            gross_rental_income_pct_from_dwelling_units: 100,
            taxpayer_elected_straight_line: false,
            taxpayer_elected_ads: false,
            adjusted_basis_cents: 50_000_000,
            taxpayer_applied_mid_quarter_when_triggered: false,
        }
    }

    #[test]
    fn residential_rental_27_5_straight_line_mid_month() {
        let out = check(&base_residential());
        assert_eq!(
            out.severity,
            Severity::MethodStraightLineResidentialRental27_5
        );
        assert_eq!(out.applicable_method, Method::StraightLine);
        assert_eq!(out.applicable_convention, Convention::MidMonth);
        assert_eq!(out.applicable_recovery_period_years_x_10, 275);
    }

    #[test]
    fn residential_rental_below_80_pct_dwelling_violation() {
        let mut i = base_residential();
        i.gross_rental_income_pct_from_dwelling_units = 70;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationResidentialRentalClassifiedWithUnder80PctDwelling
        );
    }

    #[test]
    fn residential_at_exactly_80_pct_dwelling_compliant() {
        let mut i = base_residential();
        i.gross_rental_income_pct_from_dwelling_units = 80;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::MethodStraightLineResidentialRental27_5
        );
    }

    #[test]
    fn nonresidential_real_39_straight_line_mid_month() {
        let mut i = base_residential();
        i.property_class = PropertyClass::NonresidentialReal39;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::MethodStraightLineNonresidentialReal39
        );
        assert_eq!(out.applicable_recovery_period_years_x_10, 390);
        assert_eq!(out.applicable_convention, Convention::MidMonth);
    }

    #[test]
    fn five_year_property_200_db_half_year() {
        let mut i = base_residential();
        i.property_class = PropertyClass::FiveYear;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Method200DBPersonalProperty3To10Years
        );
        assert_eq!(out.applicable_method, Method::DecliningBalance200);
        assert_eq!(out.applicable_convention, Convention::HalfYear);
    }

    #[test]
    fn seven_year_property_200_db() {
        let mut i = base_residential();
        i.property_class = PropertyClass::SevenYear;
        let out = check(&i);
        assert_eq!(out.applicable_method, Method::DecliningBalance200);
    }

    #[test]
    fn fifteen_year_property_150_db_half_year() {
        let mut i = base_residential();
        i.property_class = PropertyClass::FifteenYear;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Method150DBPersonalProperty15To20Years
        );
        assert_eq!(out.applicable_method, Method::DecliningBalance150);
    }

    #[test]
    fn twenty_year_property_150_db() {
        let mut i = base_residential();
        i.property_class = PropertyClass::TwentyYear;
        let out = check(&i);
        assert_eq!(out.applicable_method, Method::DecliningBalance150);
    }

    #[test]
    fn mid_quarter_convention_triggered_at_41_pct() {
        let mut i = base_residential();
        i.property_class = PropertyClass::FiveYear;
        i.last_quarter_placement_aggregate_pct = 41;
        i.taxpayer_applied_mid_quarter_when_triggered = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ConventionMidQuarterTriggeredBy40PctLastQuarter
        );
        assert_eq!(out.applicable_convention, Convention::MidQuarter);
    }

    #[test]
    fn mid_quarter_at_exactly_40_pct_not_triggered() {
        let mut i = base_residential();
        i.property_class = PropertyClass::FiveYear;
        i.last_quarter_placement_aggregate_pct = 40;
        let out = check(&i);
        assert_eq!(out.applicable_convention, Convention::HalfYear);
    }

    #[test]
    fn mid_quarter_missing_violation_when_taxpayer_used_half_year() {
        let mut i = base_residential();
        i.property_class = PropertyClass::FiveYear;
        i.last_quarter_placement_aggregate_pct = 50;
        i.taxpayer_applied_mid_quarter_when_triggered = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationMissingMidQuarterConventionWhen40PctTriggered
        );
    }

    #[test]
    fn elective_straight_line_on_personal_property() {
        let mut i = base_residential();
        i.property_class = PropertyClass::FiveYear;
        i.taxpayer_elected_straight_line = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::MethodElectiveStraightLineOnPersonalProperty
        );
        assert_eq!(out.applicable_method, Method::StraightLine);
    }

    #[test]
    fn ads_elected_uses_straight_line_over_class_life() {
        let mut i = base_residential();
        i.taxpayer_elected_ads = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::MethodAdsStraightLineOverClassLife);
        assert_eq!(out.applicable_method, Method::AdsStraightLine);
    }

    #[test]
    fn residential_first_year_deduction_approximates_3_6_pct() {
        let out = check(&base_residential());
        assert_eq!(out.first_year_deduction_cents, 1_800_000);
    }

    #[test]
    fn nonresidential_first_year_deduction_approximates_2_6_pct() {
        let mut i = base_residential();
        i.property_class = PropertyClass::NonresidentialReal39;
        let out = check(&i);
        assert_eq!(out.first_year_deduction_cents, 1_300_000);
    }

    #[test]
    fn citations_pin_168_subsections_a_b_c_d_e() {
        let out = check(&base_residential());
        assert!(out.citations.iter().any(|c| c.contains("§ 168(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(b)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(b)(2)(B)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(b)(3)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(d)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(d)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(d)(3)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(e)(2)(A)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(e)(2)(B)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(g)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168(k)")));
    }

    #[test]
    fn citations_pin_tax_reform_act_1986_and_pub_946() {
        let out = check(&base_residential());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Tax Reform Act of 1986")));
        assert!(out.citations.iter().any(|c| c.contains("Publication 946")));
    }

    #[test]
    fn constant_pin_residential_27_5_x_10() {
        assert_eq!(RESIDENTIAL_RENTAL_RECOVERY_PERIOD_X_10, 275);
    }

    #[test]
    fn constant_pin_nonresidential_39_years() {
        assert_eq!(NONRESIDENTIAL_REAL_RECOVERY_PERIOD_YEARS, 39);
    }

    #[test]
    fn constant_pin_200_db_method() {
        assert_eq!(METHOD_200_PERCENT_DB, 200);
    }

    #[test]
    fn constant_pin_150_db_method() {
        assert_eq!(METHOD_150_PERCENT_DB, 150);
    }

    #[test]
    fn constant_pin_40_pct_mid_quarter_threshold() {
        assert_eq!(MID_QUARTER_CONVENTION_THRESHOLD_PERCENT, 40);
    }

    #[test]
    fn constant_pin_80_pct_residential_dwelling_threshold() {
        assert_eq!(
            RESIDENTIAL_RENTAL_GROSS_RENT_FROM_DWELLING_THRESHOLD_PERCENT,
            80
        );
    }

    #[test]
    fn very_large_basis_no_overflow() {
        let mut i = base_residential();
        i.adjusted_basis_cents = u64::MAX;
        let out = check(&i);
        assert!(out.first_year_deduction_cents > 0);
    }
}
