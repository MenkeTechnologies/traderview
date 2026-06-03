//! Florida HB 1417 of 2023 State Preemption of Local
//! Landlord-Tenant Ordinances Compliance Module (Fla. Stat.
//! § 83.425).
//!
//! Pure-compute check for landlord compliance with Florida's
//! HB 1417 statewide preemption regime, which renders all
//! local government residential-tenancy ordinances null and
//! void in favor of statewide uniform Chapter 83 Part II
//! rules. Signed by Governor Ron DeSantis; effective July 1,
//! 2023; codified at Fla. Stat. § 83.425.
//!
//! Web research (verified 2026-06-03):
//! - **Florida HB 1417 of 2023** (CS/HB 1417 — Residential
//!   Tenancies) signed by Governor **Ron DeSantis**; effective
//!   **July 1, 2023**; created Fla. Stat. § 83.425; amended
//!   Chapter 83 Part II (Florida Residential Landlord and
//!   Tenant Act) ([Florida Senate HB 1417 (2023)](https://www.flsenate.gov/Session/Bill/2023/1417);
//!   [CS/HB 1417 Bill Analysis](https://www.flsenate.gov/Session/Bill/2023/1417/Analyses/h1417z1.CJS.PDF);
//!   [NLIHC — Florida Governor Signs Preemption Legislation](https://nlihc.org/resource/florida-governor-signs-preemption-legislation-impacting-tenant-protections-across-state)).
//! - **Fla. Stat. § 83.425 Preemption**: preempts to the STATE
//!   the regulation of residential tenancies, the landlord-
//!   tenant relationship, and ALL OTHER MATTERS covered under
//!   Chapter 83 Part II, F.S.; expressly **supersedes any
//!   local government regulations** on those matters; renders
//!   all existing local government ordinances throughout the
//!   state that purport to regulate residential tenancies,
//!   the landlord-tenant relationship, or any other matters
//!   covered under Chapter 83 Part II **null and void**.
//! - **Impacted Local Ordinances**: housing advocates estimate
//!   **46 tenant protection ordinances** were preempted
//!   spanning **35 cities and counties**, including
//!   jurisdictions like **Miami-Dade, Broward, Orange,
//!   Hillsborough, and Pinellas counties** ([Florida Realtors
//!   — State Law Preempts Many Local Rental Rules](https://www.floridarealtors.org/news-media/news-articles/2023/08/state-law-preempts-many-local-rental-rules)).
//! - **Local Rules No Longer Permissible**: rent notices,
//!   Section 8 housing voucher acceptance mandates,
//!   source-of-income protections, tenants' bill of rights
//!   ordinances, rent stabilization measures, eviction
//!   sealing, late fee caps, summons-process modifications,
//!   and any other matter covered under Chapter 83 Part II.
//! - **Month-to-Month Tenancy Termination Notice (Fla. Stat.
//!   § 83.57(3))**: increased from **15 days to 30 days**
//!   prior to end of monthly period.
//! - **End-of-Term Termination Notice (Fla. Stat. § 83.575)**:
//!   revised range to **not less than 30 days' notice or more
//!   than 60 days' notice** (from prior "not more than 60
//!   days") — applies to landlord OR tenant termination.
//! - **Chapter 83 Part II — Florida Residential Landlord and
//!   Tenant Act**: governs all residential tenancies in
//!   Florida post-HB 1417 with statewide uniform rules.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const FL_HB_1417_SIGNED_DATE_YEAR: u32 = 2023;
pub const FL_HB_1417_EFFECTIVE_DATE_YEAR: u32 = 2023;
pub const FL_HB_1417_EFFECTIVE_DATE_MONTH: u32 = 7;
pub const FL_HB_1417_EFFECTIVE_DATE_DAY: u32 = 1;
pub const FL_PREEMPTED_ORDINANCES_COUNT: u32 = 46;
pub const FL_PREEMPTED_LOCALITIES_COUNT: u32 = 35;
pub const FL_MONTH_TO_MONTH_NOTICE_DAYS_PRE_HB1417: u32 = 15;
pub const FL_MONTH_TO_MONTH_NOTICE_DAYS_POST_HB1417: u32 = 30;
pub const FL_END_OF_TERM_NOTICE_DAYS_MINIMUM: u32 = 30;
pub const FL_END_OF_TERM_NOTICE_DAYS_MAXIMUM: u32 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    FloridaSubjectToHb1417Preemption,
    NotInFlorida,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalOrdinanceCategory {
    NoLocalOrdinanceAtIssue,
    TenantsBillOfRightsOrdinance,
    RentStabilizationMeasure,
    SourceOfIncomeProtection,
    Section8VoucherMandatoryAcceptance,
    RentIncreaseNoticeExtensionLocal,
    EvictionRecordSealingLocal,
    LateFeeCapLocal,
    SummonsProcessModificationLocal,
    OtherLocalLandlordTenantOrdinance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyAction {
    MonthToMonthTermination,
    EndOfTermTerminationByLandlord,
    EndOfTermTerminationByTenant,
    ApplyingPreemptedLocalOrdinance,
    NoTerminationAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FloridaHb1417Mode {
    NotApplicableNotInFlorida,
    NotApplicableNoTerminationActionAndNoOrdinanceAtIssue,
    CompliantMonthToMonth30DayNoticeUnderRevisedSection83_57_3,
    CompliantEndOfTermNoticeWithinStatutoryRange30to60Days,
    CompliantStatewidePreemptionRespectedNoLocalOrdinanceApplied,
    ViolationLocalOrdinanceAppliedPreemptedUnderSection83_425,
    ViolationMonthToMonthNoticeBelow30Days,
    ViolationEndOfTermNoticeBelow30Days,
    ViolationEndOfTermNoticeAbove60Days,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub tenancy_action: TenancyAction,
    pub local_ordinance_category: LocalOrdinanceCategory,
    pub notice_days_provided: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: FloridaHb1417Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalFloridaHb1417StatePreemptionInput = Input;
pub type RentalFloridaHb1417StatePreemptionOutput = Output;
pub type RentalFloridaHb1417StatePreemptionResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Florida HB 1417 of 2023 (CS/HB 1417 — Residential Tenancies); signed by Governor Ron DeSantis; effective July 1, 2023; created Fla. Stat. § 83.425; amended Chapter 83 Part II (Florida Residential Landlord and Tenant Act)".to_string(),
        "Fla. Stat. § 83.425 — preempts to the STATE the regulation of residential tenancies, landlord-tenant relationship, and ALL OTHER MATTERS covered under Chapter 83 Part II; expressly supersedes any local government regulations; renders all existing local government ordinances null and void".to_string(),
        "Impacted Local Ordinances — 46 tenant protection ordinances preempted spanning 35 cities and counties, including Miami-Dade, Broward, Orange, Hillsborough, and Pinellas counties".to_string(),
        "Local Rules No Longer Permissible — rent notices, Section 8 housing voucher acceptance mandates, source-of-income protections, tenants' bill of rights ordinances, rent stabilization measures, eviction sealing, late fee caps, summons-process modifications, and any other matter covered under Chapter 83 Part II".to_string(),
        "Month-to-Month Tenancy Termination Notice (Fla. Stat. § 83.57(3)) — increased from 15 days to 30 days prior to end of monthly period".to_string(),
        "End-of-Term Termination Notice (Fla. Stat. § 83.575) — revised range to not less than 30 days' notice or more than 60 days' notice (from prior 'not more than 60 days'); applies to landlord OR tenant termination".to_string(),
        "Chapter 83 Part II — Florida Residential Landlord and Tenant Act governs all residential tenancies in Florida post-HB 1417 with statewide uniform rules".to_string(),
        "Florida Senate HB 1417 (2023) — official bill page with complete legislative history".to_string(),
        "CS/HB 1417 Bill Analysis — Civil Justice Subcommittee staff analysis".to_string(),
        "NLIHC — Florida Governor Signs Preemption Legislation Impacting Tenant Protections — practitioner analysis".to_string(),
        "Florida Realtors — State Law Preempts Many Local Rental Rules — industry summary".to_string(),
        "Tampa Bay Property Management — Shaping the Future of Florida's Rental Landscape — operational guide".to_string(),
        "Orlando Weekly — Industry-Backed Landlord Tenant Law analysis".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::NotInFlorida {
        return Output {
            mode: FloridaHb1417Mode::NotApplicableNotInFlorida,
            statutory_basis: "Property outside Florida; Fla. Stat. § 83.425 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Florida; Florida HB 1417 statewide preemption regime inapplicable.".to_string(),
            citations,
        };
    }

    if input.tenancy_action == TenancyAction::ApplyingPreemptedLocalOrdinance
        || matches!(
            input.local_ordinance_category,
            LocalOrdinanceCategory::TenantsBillOfRightsOrdinance
                | LocalOrdinanceCategory::RentStabilizationMeasure
                | LocalOrdinanceCategory::SourceOfIncomeProtection
                | LocalOrdinanceCategory::Section8VoucherMandatoryAcceptance
                | LocalOrdinanceCategory::RentIncreaseNoticeExtensionLocal
                | LocalOrdinanceCategory::EvictionRecordSealingLocal
                | LocalOrdinanceCategory::LateFeeCapLocal
                | LocalOrdinanceCategory::SummonsProcessModificationLocal
                | LocalOrdinanceCategory::OtherLocalLandlordTenantOrdinance
        )
    {
        return Output {
            mode: FloridaHb1417Mode::ViolationLocalOrdinanceAppliedPreemptedUnderSection83_425,
            statutory_basis: "Fla. Stat. § 83.425 — local landlord-tenant ordinances null and void".to_string(),
            notes: format!(
                "VIOLATION: local ordinance ({:?}) applied to residential tenancy; Fla. Stat. § 83.425 renders all such local ordinances null and void; only statewide Chapter 83 Part II rules apply.",
                input.local_ordinance_category
            ),
            citations,
        };
    }

    if input.tenancy_action == TenancyAction::MonthToMonthTermination {
        if input.notice_days_provided < FL_MONTH_TO_MONTH_NOTICE_DAYS_POST_HB1417 {
            return Output {
                mode: FloridaHb1417Mode::ViolationMonthToMonthNoticeBelow30Days,
                statutory_basis: "Fla. Stat. § 83.57(3) — month-to-month termination requires 30-day notice".to_string(),
                notes: format!(
                    "VIOLATION: month-to-month termination notice of {} days below 30-day statutory minimum under Fla. Stat. § 83.57(3) as amended by HB 1417 (increased from prior 15 days).",
                    input.notice_days_provided
                ),
                citations,
            };
        }
        return Output {
            mode: FloridaHb1417Mode::CompliantMonthToMonth30DayNoticeUnderRevisedSection83_57_3,
            statutory_basis: "Fla. Stat. § 83.57(3) — month-to-month termination 30-day notice satisfied".to_string(),
            notes: format!(
                "COMPLIANT: month-to-month termination notice of {} days satisfies 30-day statutory minimum under Fla. Stat. § 83.57(3) as amended by HB 1417.",
                input.notice_days_provided
            ),
            citations,
        };
    }

    if matches!(
        input.tenancy_action,
        TenancyAction::EndOfTermTerminationByLandlord | TenancyAction::EndOfTermTerminationByTenant
    ) {
        if input.notice_days_provided < FL_END_OF_TERM_NOTICE_DAYS_MINIMUM {
            return Output {
                mode: FloridaHb1417Mode::ViolationEndOfTermNoticeBelow30Days,
                statutory_basis: "Fla. Stat. § 83.575 — end-of-term termination notice not less than 30 days".to_string(),
                notes: format!(
                    "VIOLATION: end-of-term termination notice of {} days below 30-day statutory minimum under Fla. Stat. § 83.575 as amended by HB 1417.",
                    input.notice_days_provided
                ),
                citations,
            };
        }
        if input.notice_days_provided > FL_END_OF_TERM_NOTICE_DAYS_MAXIMUM {
            return Output {
                mode: FloridaHb1417Mode::ViolationEndOfTermNoticeAbove60Days,
                statutory_basis: "Fla. Stat. § 83.575 — end-of-term termination notice not more than 60 days".to_string(),
                notes: format!(
                    "VIOLATION: end-of-term termination notice of {} days above 60-day statutory maximum under Fla. Stat. § 83.575 as amended by HB 1417.",
                    input.notice_days_provided
                ),
                citations,
            };
        }
        return Output {
            mode: FloridaHb1417Mode::CompliantEndOfTermNoticeWithinStatutoryRange30to60Days,
            statutory_basis: "Fla. Stat. § 83.575 — end-of-term termination notice within 30-60 day range".to_string(),
            notes: format!(
                "COMPLIANT: end-of-term termination notice of {} days within statutory range of 30-60 days under Fla. Stat. § 83.575 as amended by HB 1417.",
                input.notice_days_provided
            ),
            citations,
        };
    }

    if input.tenancy_action == TenancyAction::NoTerminationAction
        && input.local_ordinance_category == LocalOrdinanceCategory::NoLocalOrdinanceAtIssue
    {
        return Output {
            mode: FloridaHb1417Mode::NotApplicableNoTerminationActionAndNoOrdinanceAtIssue,
            statutory_basis: "Florida HB 1417 / Fla. Stat. § 83.425 — no triggering action or ordinance".to_string(),
            notes: "NOT APPLICABLE: no termination action and no local ordinance at issue; Florida HB 1417 framework not triggered for this scenario.".to_string(),
            citations,
        };
    }

    Output {
        mode: FloridaHb1417Mode::CompliantStatewidePreemptionRespectedNoLocalOrdinanceApplied,
        statutory_basis: "Fla. Stat. § 83.425 — statewide preemption respected".to_string(),
        notes: "COMPLIANT: statewide preemption respected; only Chapter 83 Part II rules applied; no local ordinance imposed; no termination notice issue.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_florida() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::FloridaSubjectToHb1417Preemption,
            tenancy_action: TenancyAction::NoTerminationAction,
            local_ordinance_category: LocalOrdinanceCategory::NoLocalOrdinanceAtIssue,
            notice_days_provided: 0,
        }
    }

    #[test]
    fn property_outside_florida_not_applicable() {
        let input = Input {
            property_jurisdiction: PropertyJurisdiction::NotInFlorida,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(result.mode, FloridaHb1417Mode::NotApplicableNotInFlorida);
    }

    #[test]
    fn no_termination_no_ordinance_not_applicable() {
        let result = check(&baseline_compliant_florida());
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::NotApplicableNoTerminationActionAndNoOrdinanceAtIssue
        );
    }

    #[test]
    fn applying_preempted_local_ordinance_violation() {
        let input = Input {
            tenancy_action: TenancyAction::ApplyingPreemptedLocalOrdinance,
            local_ordinance_category: LocalOrdinanceCategory::TenantsBillOfRightsOrdinance,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationLocalOrdinanceAppliedPreemptedUnderSection83_425
        );
    }

    #[test]
    fn source_of_income_protection_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::SourceOfIncomeProtection,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationLocalOrdinanceAppliedPreemptedUnderSection83_425
        );
    }

    #[test]
    fn section_8_voucher_mandate_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::Section8VoucherMandatoryAcceptance,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationLocalOrdinanceAppliedPreemptedUnderSection83_425
        );
    }

    #[test]
    fn rent_stabilization_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::RentStabilizationMeasure,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationLocalOrdinanceAppliedPreemptedUnderSection83_425
        );
    }

    #[test]
    fn late_fee_cap_local_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::LateFeeCapLocal,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationLocalOrdinanceAppliedPreemptedUnderSection83_425
        );
    }

    #[test]
    fn month_to_month_30_day_notice_compliant() {
        let input = Input {
            tenancy_action: TenancyAction::MonthToMonthTermination,
            notice_days_provided: 30,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::CompliantMonthToMonth30DayNoticeUnderRevisedSection83_57_3
        );
    }

    #[test]
    fn month_to_month_29_day_notice_violation() {
        let input = Input {
            tenancy_action: TenancyAction::MonthToMonthTermination,
            notice_days_provided: 29,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationMonthToMonthNoticeBelow30Days
        );
    }

    #[test]
    fn month_to_month_15_day_pre_hb1417_violation() {
        let input = Input {
            tenancy_action: TenancyAction::MonthToMonthTermination,
            notice_days_provided: 15,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationMonthToMonthNoticeBelow30Days
        );
    }

    #[test]
    fn end_of_term_landlord_45_day_notice_compliant() {
        let input = Input {
            tenancy_action: TenancyAction::EndOfTermTerminationByLandlord,
            notice_days_provided: 45,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::CompliantEndOfTermNoticeWithinStatutoryRange30to60Days
        );
    }

    #[test]
    fn end_of_term_tenant_30_day_notice_compliant() {
        let input = Input {
            tenancy_action: TenancyAction::EndOfTermTerminationByTenant,
            notice_days_provided: 30,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::CompliantEndOfTermNoticeWithinStatutoryRange30to60Days
        );
    }

    #[test]
    fn end_of_term_at_exactly_60_days_compliant() {
        let input = Input {
            tenancy_action: TenancyAction::EndOfTermTerminationByLandlord,
            notice_days_provided: 60,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::CompliantEndOfTermNoticeWithinStatutoryRange30to60Days
        );
    }

    #[test]
    fn end_of_term_below_30_violation() {
        let input = Input {
            tenancy_action: TenancyAction::EndOfTermTerminationByLandlord,
            notice_days_provided: 25,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationEndOfTermNoticeBelow30Days
        );
    }

    #[test]
    fn end_of_term_above_60_violation() {
        let input = Input {
            tenancy_action: TenancyAction::EndOfTermTerminationByLandlord,
            notice_days_provided: 61,
            ..baseline_compliant_florida()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            FloridaHb1417Mode::ViolationEndOfTermNoticeAbove60Days
        );
    }

    #[test]
    fn citations_pin_hb_1417_section_83_425_and_local_preemption() {
        let result = check(&baseline_compliant_florida());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Florida HB 1417 of 2023"));
        assert!(joined.contains("Governor Ron DeSantis"));
        assert!(joined.contains("July 1, 2023"));
        assert!(joined.contains("Fla. Stat. § 83.425"));
        assert!(joined.contains("Chapter 83 Part II"));
        assert!(joined.contains("Florida Residential Landlord and Tenant Act"));
        assert!(joined.contains("null and void"));
        assert!(joined.contains("46 tenant protection ordinances"));
        assert!(joined.contains("35 cities and counties"));
        assert!(joined.contains("Miami-Dade"));
        assert!(joined.contains("Broward"));
        assert!(joined.contains("Orange"));
        assert!(joined.contains("Hillsborough"));
        assert!(joined.contains("Pinellas"));
        assert!(joined.contains("source-of-income"));
        assert!(joined.contains("Section 8"));
        assert!(joined.contains("rent stabilization"));
        assert!(joined.contains("eviction sealing"));
        assert!(joined.contains("late fee caps"));
        assert!(joined.contains("Fla. Stat. § 83.57(3)"));
        assert!(joined.contains("30 days"));
        assert!(joined.contains("Fla. Stat. § 83.575"));
        assert!(joined.contains("60 days"));
        assert!(joined.contains("NLIHC"));
        assert!(joined.contains("Florida Realtors"));
    }

    #[test]
    fn constant_pin_dates_thresholds_and_counts() {
        assert_eq!(FL_HB_1417_SIGNED_DATE_YEAR, 2023);
        assert_eq!(FL_HB_1417_EFFECTIVE_DATE_YEAR, 2023);
        assert_eq!(FL_HB_1417_EFFECTIVE_DATE_MONTH, 7);
        assert_eq!(FL_HB_1417_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(FL_PREEMPTED_ORDINANCES_COUNT, 46);
        assert_eq!(FL_PREEMPTED_LOCALITIES_COUNT, 35);
        assert_eq!(FL_MONTH_TO_MONTH_NOTICE_DAYS_PRE_HB1417, 15);
        assert_eq!(FL_MONTH_TO_MONTH_NOTICE_DAYS_POST_HB1417, 30);
        assert_eq!(FL_END_OF_TERM_NOTICE_DAYS_MINIMUM, 30);
        assert_eq!(FL_END_OF_TERM_NOTICE_DAYS_MAXIMUM, 60);
    }
}
