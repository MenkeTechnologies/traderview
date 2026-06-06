//! Residential Rent Increase Notice Requirement Multi-State
//! Compliance Module.
//!
//! Pure-compute multi-jurisdictional check for whether a landlord
//! has served a tenant the statutorily required rent-increase
//! notice in time, with appropriate accounting for the tenancy's
//! length and the size of the proposed increase.
//!
//! Web research (verified 2026-06-03):
//! - **California Civ. Code § 827** (amended by AB 1110 (2019),
//!   effective January 1, 2020): for residential periodic tenancies,
//!   landlord must serve **30 days** written notice for rent
//!   increases of 10 % or less, and **90 days** written notice for
//!   increases greater than 10 %. (Wallace, Richardson, Sontag & Le
//!   LLP — "90 Days' Notice Required to Increase Rent by More Than
//!   10%"; CA Legislative Information AB 1110 Bill Text.)
//! - **Washington RCW 59.18.140** (amended effective May 7, 2025):
//!   landlord must provide written notice a minimum of **60 days**
//!   before the effective date of an increase in rent for
//!   residential periodic tenancies. (RCW 59.18.140 official.)
//! - **Oregon ORS 90.323** (also referenced as § 90.600 for some
//!   tenancies; SB 608 of 2019): landlord must provide **90 days**
//!   written notice for any rent increase under SB 608.
//! - **New York Real Property Law § 226-c** (added by HSTPA 2019):
//!   if landlord intends to raise rent by 5 % or more OR intends
//!   not to renew the lease, the landlord must provide advance
//!   written notice keyed to tenancy length: tenancy < 1 year (or
//!   lease term < 1 year) = 30 days notice; tenancy 1-2 years (or
//!   lease term 1-2 years) = 60 days notice; tenancy ≥ 2 years (or
//!   lease term ≥ 2 years) = 90 days notice. Failure consequence:
//!   tenant's lawful tenancy continues under existing terms until
//!   notice period has expired (NY State Senate § 226-c; tenant-
//!   rights.com NY 30/60/90-day guide).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CA_827_AB_1110_EFFECTIVE_YEAR: u32 = 2020;
pub const CA_827_AB_1110_EFFECTIVE_MONTH: u32 = 1;
pub const CA_827_AB_1110_EFFECTIVE_DAY: u32 = 1;
pub const CA_827_SMALL_INCREASE_NOTICE_DAYS: u32 = 30;
pub const CA_827_LARGE_INCREASE_NOTICE_DAYS: u32 = 90;
pub const CA_827_SMALL_INCREASE_CEILING_BASIS_POINTS: u32 = 1_000;
pub const WA_RCW_59_18_140_NOTICE_DAYS: u32 = 60;
pub const WA_RCW_59_18_140_AMENDED_EFFECTIVE_YEAR: u32 = 2025;
pub const WA_RCW_59_18_140_AMENDED_EFFECTIVE_MONTH: u32 = 5;
pub const WA_RCW_59_18_140_AMENDED_EFFECTIVE_DAY: u32 = 7;
pub const OR_ORS_90_323_NOTICE_DAYS: u32 = 90;
pub const OR_SB_608_EFFECTIVE_YEAR: u32 = 2019;
pub const NY_RPL_226C_TRIGGER_THRESHOLD_BASIS_POINTS: u32 = 500;
pub const NY_RPL_226C_SHORT_OCCUPANCY_NOTICE_DAYS: u32 = 30;
pub const NY_RPL_226C_MEDIUM_OCCUPANCY_NOTICE_DAYS: u32 = 60;
pub const NY_RPL_226C_LONG_OCCUPANCY_NOTICE_DAYS: u32 = 90;
pub const NY_HSTPA_EFFECTIVE_YEAR: u32 = 2019;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentIncreaseJurisdiction {
    California,
    Washington,
    Oregon,
    NewYork,
    OtherStateWithoutStatutoryMandate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyOrLeaseLength {
    LessThan1Year,
    Between1And2Years,
    TwoYearsOrMore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentIncreaseNoticeMode {
    NotApplicableNoIncrease,
    NotApplicableJurisdictionLacksStatutoryMandate,
    NotApplicableNyIncreaseBelow5PctThreshold,
    CompliantNoticeServedWithinStatutoryWindow,
    ViolationNoNoticeServed,
    ViolationCaliforniaUnder30DaysShortIncrease,
    ViolationCaliforniaUnder90DaysLargeIncrease,
    ViolationWashingtonUnder60Days,
    ViolationOregonUnder90Days,
    ViolationNyUnder30DayShortOccupancy,
    ViolationNyUnder60DayMediumOccupancy,
    ViolationNyUnder90DayLongOccupancy,
    ViolationOccupancyMisclassifiedToReduceNoticeWindow,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: RentIncreaseJurisdiction,
    pub increase_size_basis_points: u32,
    pub occupancy_or_lease_length: OccupancyOrLeaseLength,
    pub days_notice_served_before_increase_effective_date: u32,
    pub notice_actually_served: bool,
    pub landlord_claimed_shorter_occupancy_than_actual: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: RentIncreaseNoticeMode,
    pub required_notice_days: u32,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalRentIncreaseNoticeRequirementInput = Input;
pub type RentalRentIncreaseNoticeRequirementOutput = Output;
pub type RentalRentIncreaseNoticeRequirementResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

fn required_notice_days_for(input: &Input) -> u32 {
    match input.jurisdiction {
        RentIncreaseJurisdiction::California => {
            if input.increase_size_basis_points > CA_827_SMALL_INCREASE_CEILING_BASIS_POINTS {
                CA_827_LARGE_INCREASE_NOTICE_DAYS
            } else {
                CA_827_SMALL_INCREASE_NOTICE_DAYS
            }
        }
        RentIncreaseJurisdiction::Washington => WA_RCW_59_18_140_NOTICE_DAYS,
        RentIncreaseJurisdiction::Oregon => OR_ORS_90_323_NOTICE_DAYS,
        RentIncreaseJurisdiction::NewYork => match input.occupancy_or_lease_length {
            OccupancyOrLeaseLength::LessThan1Year => NY_RPL_226C_SHORT_OCCUPANCY_NOTICE_DAYS,
            OccupancyOrLeaseLength::Between1And2Years => NY_RPL_226C_MEDIUM_OCCUPANCY_NOTICE_DAYS,
            OccupancyOrLeaseLength::TwoYearsOrMore => NY_RPL_226C_LONG_OCCUPANCY_NOTICE_DAYS,
        },
        RentIncreaseJurisdiction::OtherStateWithoutStatutoryMandate => 0,
    }
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cal. Civ. Code § 827 (AB 1110 of 2019, effective Jan 1, 2020) — 30 days for ≤ 10 % increase; 90 days for > 10 % increase".to_string(),
        "Wash. RCW 59.18.140 (amended effective May 7, 2025) — 60 days written notice before rent increase effective date".to_string(),
        "Or. ORS 90.323 / 90.600 (SB 608 of 2019) — 90 days written notice for any rent increase".to_string(),
        "N.Y. Real Property Law § 226-c (HSTPA of 2019) — notice required if rent increase ≥ 5 % OR non-renewal: 30 days (< 1 yr), 60 days (1-2 yr), 90 days (≥ 2 yr)".to_string(),
        "N.Y. RPL § 226-c failure consequence: tenant's lawful tenancy continues under existing terms until notice period has expired".to_string(),
        "Cal. Civ. Code § 827 small-increase 30-day notice applies to increase ≤ 10 % of lowest gross rental rate in the preceding 12 months".to_string(),
    ];

    if input.increase_size_basis_points == 0 {
        return Output {
            mode: RentIncreaseNoticeMode::NotApplicableNoIncrease,
            required_notice_days: 0,
            statutory_basis: "No rent increase proposed".to_string(),
            notes: "Landlord is not proposing any rent increase; notice statutes do not apply."
                .to_string(),
            citations,
        };
    }

    if input.jurisdiction == RentIncreaseJurisdiction::OtherStateWithoutStatutoryMandate {
        return Output {
            mode: RentIncreaseNoticeMode::NotApplicableJurisdictionLacksStatutoryMandate,
            required_notice_days: 0,
            statutory_basis: "None — jurisdiction lacks statutory rent-increase notice mandate".to_string(),
            notes: "Jurisdiction does not impose statutory notice timing; default to lease and common-law rules.".to_string(),
            citations,
        };
    }

    if input.jurisdiction == RentIncreaseJurisdiction::NewYork
        && input.increase_size_basis_points < NY_RPL_226C_TRIGGER_THRESHOLD_BASIS_POINTS
    {
        return Output {
            mode: RentIncreaseNoticeMode::NotApplicableNyIncreaseBelow5PctThreshold,
            required_notice_days: 0,
            statutory_basis: "N.Y. RPL § 226-c — trigger threshold is 5 % increase".to_string(),
            notes: format!(
                "Increase of {} basis points is below the 5 % (500 basis points) threshold for § 226-c notice. § 226-c does not require statutory notice for sub-5 % increases.",
                input.increase_size_basis_points
            ),
            citations,
        };
    }

    let required = required_notice_days_for(input);

    if input.landlord_claimed_shorter_occupancy_than_actual
        && input.jurisdiction == RentIncreaseJurisdiction::NewYork
    {
        return Output {
            mode: RentIncreaseNoticeMode::ViolationOccupancyMisclassifiedToReduceNoticeWindow,
            required_notice_days: required,
            statutory_basis: "N.Y. RPL § 226-c — occupancy length determines notice window".to_string(),
            notes: format!(
                "VIOLATION: landlord misclassified tenant's occupancy length to shorten the notice window. Required notice based on actual occupancy = {} days.",
                required
            ),
            citations,
        };
    }

    if !input.notice_actually_served {
        return Output {
            mode: RentIncreaseNoticeMode::ViolationNoNoticeServed,
            required_notice_days: required,
            statutory_basis: format!("{} day written notice required but not served", required),
            notes: format!(
                "VIOLATION: jurisdiction requires {} days written notice for rent increase of {} basis points; no notice was served.",
                required, input.increase_size_basis_points
            ),
            citations,
        };
    }

    if input.days_notice_served_before_increase_effective_date >= required {
        return Output {
            mode: RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow,
            required_notice_days: required,
            statutory_basis: format!(
                "Statutory window of {} days satisfied (notice served {} days in advance)",
                required, input.days_notice_served_before_increase_effective_date
            ),
            notes: format!(
                "COMPLIANT: notice served {} days before effective date (≥ {} statutory minimum) for {} basis point increase.",
                input.days_notice_served_before_increase_effective_date, required, input.increase_size_basis_points
            ),
            citations,
        };
    }

    let mode = match input.jurisdiction {
        RentIncreaseJurisdiction::California => {
            if input.increase_size_basis_points > CA_827_SMALL_INCREASE_CEILING_BASIS_POINTS {
                RentIncreaseNoticeMode::ViolationCaliforniaUnder90DaysLargeIncrease
            } else {
                RentIncreaseNoticeMode::ViolationCaliforniaUnder30DaysShortIncrease
            }
        }
        RentIncreaseJurisdiction::Washington => {
            RentIncreaseNoticeMode::ViolationWashingtonUnder60Days
        }
        RentIncreaseJurisdiction::Oregon => RentIncreaseNoticeMode::ViolationOregonUnder90Days,
        RentIncreaseJurisdiction::NewYork => match input.occupancy_or_lease_length {
            OccupancyOrLeaseLength::LessThan1Year => {
                RentIncreaseNoticeMode::ViolationNyUnder30DayShortOccupancy
            }
            OccupancyOrLeaseLength::Between1And2Years => {
                RentIncreaseNoticeMode::ViolationNyUnder60DayMediumOccupancy
            }
            OccupancyOrLeaseLength::TwoYearsOrMore => {
                RentIncreaseNoticeMode::ViolationNyUnder90DayLongOccupancy
            }
        },
        RentIncreaseJurisdiction::OtherStateWithoutStatutoryMandate => {
            RentIncreaseNoticeMode::NotApplicableJurisdictionLacksStatutoryMandate
        }
    };

    Output {
        mode,
        required_notice_days: required,
        statutory_basis: format!(
            "{} day statutory minimum; landlord served {} days",
            required, input.days_notice_served_before_increase_effective_date
        ),
        notes: format!(
            "VIOLATION: jurisdiction requires {} days notice; landlord served {} days (shortfall {} days).",
            required,
            input.days_notice_served_before_increase_effective_date,
            required.saturating_sub(input.days_notice_served_before_increase_effective_date)
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_ca_small_increase_compliant() -> Input {
        Input {
            jurisdiction: RentIncreaseJurisdiction::California,
            increase_size_basis_points: 500,
            occupancy_or_lease_length: OccupancyOrLeaseLength::Between1And2Years,
            days_notice_served_before_increase_effective_date: 35,
            notice_actually_served: true,
            landlord_claimed_shorter_occupancy_than_actual: false,
        }
    }

    #[test]
    fn no_increase_not_applicable() {
        let input = Input {
            increase_size_basis_points: 0,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, RentIncreaseNoticeMode::NotApplicableNoIncrease);
    }

    #[test]
    fn other_jurisdiction_not_applicable() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::OtherStateWithoutStatutoryMandate,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::NotApplicableJurisdictionLacksStatutoryMandate
        );
    }

    #[test]
    fn california_small_increase_30_days_compliant() {
        let result = check(&baseline_ca_small_increase_compliant());
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
        assert_eq!(result.required_notice_days, 30);
    }

    #[test]
    fn california_at_exactly_30_days_compliant() {
        let input = Input {
            days_notice_served_before_increase_effective_date: 30,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
    }

    #[test]
    fn california_29_days_short_increase_violation() {
        let input = Input {
            days_notice_served_before_increase_effective_date: 29,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationCaliforniaUnder30DaysShortIncrease
        );
    }

    #[test]
    fn california_at_exactly_10_pct_small_increase_30_day_window() {
        let input = Input {
            increase_size_basis_points: 1_000,
            days_notice_served_before_increase_effective_date: 30,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
        assert_eq!(result.required_notice_days, 30);
    }

    #[test]
    fn california_10_01_pct_large_increase_90_day_required() {
        let input = Input {
            increase_size_basis_points: 1_001,
            days_notice_served_before_increase_effective_date: 89,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationCaliforniaUnder90DaysLargeIncrease
        );
    }

    #[test]
    fn california_15_pct_increase_90_day_compliant() {
        let input = Input {
            increase_size_basis_points: 1_500,
            days_notice_served_before_increase_effective_date: 90,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
    }

    #[test]
    fn washington_60_days_compliant() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::Washington,
            increase_size_basis_points: 800,
            days_notice_served_before_increase_effective_date: 60,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
        assert_eq!(result.required_notice_days, 60);
    }

    #[test]
    fn washington_59_days_violation() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::Washington,
            increase_size_basis_points: 800,
            days_notice_served_before_increase_effective_date: 59,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationWashingtonUnder60Days
        );
    }

    #[test]
    fn oregon_90_days_compliant() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::Oregon,
            increase_size_basis_points: 600,
            days_notice_served_before_increase_effective_date: 90,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
        assert_eq!(result.required_notice_days, 90);
    }

    #[test]
    fn oregon_89_days_violation() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::Oregon,
            increase_size_basis_points: 600,
            days_notice_served_before_increase_effective_date: 89,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationOregonUnder90Days
        );
    }

    #[test]
    fn new_york_under_5_pct_not_applicable() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 400,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::NotApplicableNyIncreaseBelow5PctThreshold
        );
    }

    #[test]
    fn new_york_at_exactly_5_pct_triggers_226c() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 500,
            occupancy_or_lease_length: OccupancyOrLeaseLength::LessThan1Year,
            days_notice_served_before_increase_effective_date: 30,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
        assert_eq!(result.required_notice_days, 30);
    }

    #[test]
    fn new_york_short_occupancy_29_days_violation() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 700,
            occupancy_or_lease_length: OccupancyOrLeaseLength::LessThan1Year,
            days_notice_served_before_increase_effective_date: 29,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationNyUnder30DayShortOccupancy
        );
    }

    #[test]
    fn new_york_medium_occupancy_60_days_compliant() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 700,
            occupancy_or_lease_length: OccupancyOrLeaseLength::Between1And2Years,
            days_notice_served_before_increase_effective_date: 60,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
        assert_eq!(result.required_notice_days, 60);
    }

    #[test]
    fn new_york_medium_occupancy_59_days_violation() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 700,
            occupancy_or_lease_length: OccupancyOrLeaseLength::Between1And2Years,
            days_notice_served_before_increase_effective_date: 59,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationNyUnder60DayMediumOccupancy
        );
    }

    #[test]
    fn new_york_long_occupancy_90_days_compliant() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 700,
            occupancy_or_lease_length: OccupancyOrLeaseLength::TwoYearsOrMore,
            days_notice_served_before_increase_effective_date: 90,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::CompliantNoticeServedWithinStatutoryWindow
        );
        assert_eq!(result.required_notice_days, 90);
    }

    #[test]
    fn new_york_long_occupancy_89_days_violation() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 700,
            occupancy_or_lease_length: OccupancyOrLeaseLength::TwoYearsOrMore,
            days_notice_served_before_increase_effective_date: 89,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationNyUnder90DayLongOccupancy
        );
    }

    #[test]
    fn no_notice_served_violation() {
        let input = Input {
            notice_actually_served: false,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, RentIncreaseNoticeMode::ViolationNoNoticeServed);
    }

    #[test]
    fn landlord_misclassified_occupancy_violation_ny() {
        let input = Input {
            jurisdiction: RentIncreaseJurisdiction::NewYork,
            increase_size_basis_points: 800,
            landlord_claimed_shorter_occupancy_than_actual: true,
            ..baseline_ca_small_increase_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            RentIncreaseNoticeMode::ViolationOccupancyMisclassifiedToReduceNoticeWindow
        );
    }

    #[test]
    fn citations_pin_statutes_across_jurisdictions() {
        let result = check(&baseline_ca_small_increase_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Cal. Civ. Code § 827"));
        assert!(joined.contains("AB 1110 of 2019"));
        assert!(joined.contains("Wash. RCW 59.18.140"));
        assert!(joined.contains("Or. ORS 90.323"));
        assert!(joined.contains("SB 608 of 2019"));
        assert!(joined.contains("N.Y. Real Property Law § 226-c"));
        assert!(joined.contains("HSTPA of 2019"));
    }

    #[test]
    fn constant_pin_notice_days_and_thresholds() {
        assert_eq!(CA_827_AB_1110_EFFECTIVE_YEAR, 2020);
        assert_eq!(CA_827_SMALL_INCREASE_NOTICE_DAYS, 30);
        assert_eq!(CA_827_LARGE_INCREASE_NOTICE_DAYS, 90);
        assert_eq!(CA_827_SMALL_INCREASE_CEILING_BASIS_POINTS, 1_000);
        assert_eq!(WA_RCW_59_18_140_NOTICE_DAYS, 60);
        assert_eq!(WA_RCW_59_18_140_AMENDED_EFFECTIVE_YEAR, 2025);
        assert_eq!(OR_ORS_90_323_NOTICE_DAYS, 90);
        assert_eq!(OR_SB_608_EFFECTIVE_YEAR, 2019);
        assert_eq!(NY_RPL_226C_TRIGGER_THRESHOLD_BASIS_POINTS, 500);
        assert_eq!(NY_RPL_226C_SHORT_OCCUPANCY_NOTICE_DAYS, 30);
        assert_eq!(NY_RPL_226C_MEDIUM_OCCUPANCY_NOTICE_DAYS, 60);
        assert_eq!(NY_RPL_226C_LONG_OCCUPANCY_NOTICE_DAYS, 90);
        assert_eq!(NY_HSTPA_EFFECTIVE_YEAR, 2019);
    }
}
