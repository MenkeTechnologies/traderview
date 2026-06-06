//! NYC GBL § 352-eeee Cooperative / Condominium Conversion
//! Eviction Protection Compliance Module.
//!
//! Pure-compute check for landlord-sponsor compliance with the
//! NYC cooperative/condominium conversion regime under New York
//! General Business Law § 352-eeee (Martin Act Article 23-A).
//! Trader-landlord critical because conversion plans involve
//! per-unit purchase agreements, tenant protections, senior/
//! disabled tenant permanent eviction shields, and 3-year grace
//! period before non-purchasing tenants can be evicted.
//!
//! Web research (verified 2026-06-03):
//! - **GBL § 352-eeee** (Conversions to Cooperative or
//!   Condominium Ownership in the City of New York; Martin Act
//!   Article 23-A; codified at General Business Law § 352-eeee):
//!   defines eviction plan and non-eviction plan procedures and
//!   tenant protections. ([NY State Senate § 352-EEEE](https://www.nysenate.gov/legislation/laws/GBS/352-EEEE);
//!   FindLaw GBS § 352-eeee.)
//! - **Eviction Plan**: cannot be declared effective until at
//!   least **51 % of bona fide tenants in occupancy** have
//!   executed written agreements to purchase. After plan is
//!   declared effective, non-purchasing tenants can be evicted
//!   at the **LATER of** (a) expiration of their lease OR (b)
//!   **3 years after the plan is declared effective**.
//! - **Non-Eviction Plan**: cannot be declared effective until
//!   written purchase agreements for at least **51 %** of all
//!   dwelling units are executed. For **buildings containing
//!   five or fewer units** where the sponsor offers a unit they
//!   or their immediate family member has occupied for at least
//!   two years, the plan may not be effective until **15 %** of
//!   dwelling units are subscribed for by bona fide tenants in
//!   occupancy or bona fide purchasers.
//! - **Senior Citizen and Disabled Person Permanent Protection**
//!   (GBL § 352-e(2-a) + § 352-eee + § 352-eeee): senior citizens
//!   aged **62+** and disabled persons who meet eligibility
//!   requirements may NOT be evicted by holders of unsold shares
//!   or any subsequent purchaser because the building is
//!   converted to cooperative ownership or under owner-occupancy
//!   provisions of rent codes.
//! - **Tenant Exclusive Purchase Rights**: tenants in occupancy
//!   have an **exclusive 90-day right** to purchase their
//!   dwelling units after the plan is accepted for filing, and a
//!   subsequent **6-month right of first refusal**.
//! - **Non-purchasing tenant eviction permitted ONLY for**:
//!   non-payment of rent, illegal use of premises, or similar
//!   breaches of obligations; NOT for failure to purchase or
//!   expiration of tenancy.
//! - **NY S3758 (2025-2026 session)**: would amend GBL § 352-e,
//!   § 352-eee, and § 352-eeee to extend eviction protections to
//!   senior citizens and disabled persons under eviction plans
//!   (currently § 352-eee covers this but § 352-eeee amendment
//!   is pending) ([NY Senate Bill S3758](https://www.nysenate.gov/legislation/bills/2025/S3758)).
//! - **NY S4910 (2025-2026 session)**: would permit conversion
//!   at 25 % purchase + 51 % written consent (lower threshold).
//! - **GBL § 352-eeeee**: Conversions to condominium in
//!   Westchester, Rockland, Nassau Counties (separate statute).
//! - **13 NYCRR Part 18**: Attorney General's Real Estate Finance
//!   Bureau regulations implementing Martin Act.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const GBL_352_EEEE_EVICTION_PLAN_THRESHOLD_BASIS_POINTS: u64 = 5_100;
pub const GBL_352_EEEE_NON_EVICTION_PLAN_THRESHOLD_BASIS_POINTS: u64 = 5_100;
pub const GBL_352_EEEE_SMALL_BUILDING_NON_EVICTION_THRESHOLD_BASIS_POINTS: u64 = 1_500;
pub const GBL_352_EEEE_3_YEAR_GRACE_PERIOD_YEARS: u32 = 3;
pub const GBL_352_EEEE_90_DAY_EXCLUSIVE_PURCHASE_DAYS: u32 = 90;
pub const GBL_352_EEEE_6_MONTH_RIGHT_FIRST_REFUSAL_MONTHS: u32 = 6;
pub const GBL_352_EEEE_SENIOR_AGE_THRESHOLD: u32 = 62;
pub const GBL_352_EEEE_SMALL_BUILDING_MAX_UNITS: u32 = 5;
pub const GBL_352_EEEE_SPONSOR_FAMILY_OCCUPANCY_MIN_YEARS: u32 = 2;
pub const GBL_352_EEEE_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const NY_S3758_PENDING_YEAR: u32 = 2025;
pub const NY_S4910_PENDING_YEAR: u32 = 2025;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionPlanType {
    NonEvictionPlan51PctLargeBuilding,
    NonEvictionPlanSmallBuilding15Pct,
    EvictionPlan51Pct,
    NoPlanFiledYet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantCategory {
    TenantOfRecordInOccupancyUnregulated,
    TenantOfRecordInOccupancyRentRegulated,
    SeniorCitizen62Plus,
    DisabledPersonEligible,
    NonPurchasingTenant,
    BonaFidePurchaserOutsideTenant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingScenario {
    NycResidentialMultipleDwellingLargeBuilding,
    NycSmallBuilding5OrFewerUnits,
    WestchesterRocklandNassauSeparateStatute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NycCoopConversionMode {
    NotApplicableNoConversionInProgress,
    NotApplicableNotInNycSubjectToSeparateStatute,
    CompliantNonEvictionPlanWith51PctSubscription,
    CompliantNonEvictionPlanWith15PctSmallBuildingException,
    CompliantEvictionPlanWith51PctSubscription,
    CompliantSeniorOrDisabledTenantPermanentlyProtected,
    CompliantNonPurchasingTenant3YearGracePeriod,
    Compliant90DayExclusivePurchaseAnd6MonthRightFirstRefusal,
    ViolationConversionPlanDeclaredEffectiveBelowSubscriptionThreshold,
    ViolationSeniorOrDisabledTenantEvictionAttempted,
    ViolationNonPurchasingTenantEvictionBefore3YearPeriod,
    ViolationTenant90DayExclusivePurchaseRightDenied,
    ViolationTenantRightOfFirstRefusalDenied,
    ViolationEvictionForReasonOtherThanRentOrIllegalUse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub plan_type: ConversionPlanType,
    pub building_scenario: BuildingScenario,
    pub tenant_category: TenantCategory,
    pub bona_fide_tenant_subscription_basis_points: u64,
    pub total_units_in_building: u32,
    pub years_since_plan_declared_effective: u32,
    pub eviction_attempted_against_senior_or_disabled: bool,
    pub eviction_attempted_against_non_purchasing_tenant: bool,
    pub tenant_90_day_exclusive_purchase_right_honored: bool,
    pub tenant_6_month_right_of_first_refusal_honored: bool,
    pub eviction_reason_non_payment_or_illegal_use: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: NycCoopConversionMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalNycCoopConversionEvictionProtectionInput = Input;
pub type RentalNycCoopConversionEvictionProtectionOutput = Output;
pub type RentalNycCoopConversionEvictionProtectionResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "NY GBL § 352-eeee (Martin Act Article 23-A; Conversions to Cooperative or Condominium Ownership in the City of New York) — eviction plan + non-eviction plan procedures + tenant protections".to_string(),
        "GBL § 352-eeee eviction plan — 51 % of bona fide tenants in occupancy must execute purchase agreements; non-purchasing tenants evicted at LATER of lease expiration OR 3 years after plan declared effective".to_string(),
        "GBL § 352-eeee non-eviction plan — 51 % of all dwelling units; 15 % for buildings ≤ 5 units with sponsor/family 2-year occupancy".to_string(),
        "GBL § 352-e(2-a) + § 352-eee + § 352-eeee senior/disabled permanent eviction protection — age 62+ or disabled may NOT be evicted by holders of unsold shares or subsequent purchasers; owner-occupancy provisions cannot be used either".to_string(),
        "GBL § 352-eeee tenant exclusive purchase rights — 90-day exclusive purchase right after plan accepted for filing + subsequent 6-month right of first refusal".to_string(),
        "GBL § 352-eeee non-purchasing tenant eviction PERMITTED ONLY for non-payment of rent, illegal use of premises, or similar breaches; NOT failure to purchase or expiration of tenancy".to_string(),
        "GBL § 352-eeeee — Conversions to condominium in Westchester, Rockland, Nassau Counties (separate statute)".to_string(),
        "13 NYCRR Part 18 — Attorney General Real Estate Finance Bureau regulations implementing Martin Act".to_string(),
        "NY Senate S3758 (2025-2026 session) — pending amendment to extend GBL § 352-eeee eviction protection to senior citizens and disabled persons under eviction plans".to_string(),
        "NY Senate S4910 (2025-2026 session) — pending amendment to permit conversion at 25 % purchase + 51 % written consent (lower threshold)".to_string(),
        "HSTPA 2019 (Housing Stability and Tenant Protection Act) — amended GBL § 352-eeee to raise non-eviction plan threshold from 15 % to 51 % for most buildings".to_string(),
    ];

    if input.plan_type == ConversionPlanType::NoPlanFiledYet {
        return Output {
            mode: NycCoopConversionMode::NotApplicableNoConversionInProgress,
            statutory_basis: "No conversion plan filed; GBL § 352-eeee not invoked".to_string(),
            notes: "No conversion plan has been filed; GBL § 352-eeee procedures not invoked."
                .to_string(),
            citations,
        };
    }

    if input.building_scenario == BuildingScenario::WestchesterRocklandNassauSeparateStatute {
        return Output {
            mode: NycCoopConversionMode::NotApplicableNotInNycSubjectToSeparateStatute,
            statutory_basis: "GBL § 352-eeeee governs Westchester/Rockland/Nassau, not § 352-eeee".to_string(),
            notes: "Building outside NYC in Westchester/Rockland/Nassau; subject to GBL § 352-eeeee separate statute.".to_string(),
            citations,
        };
    }

    if matches!(
        input.tenant_category,
        TenantCategory::SeniorCitizen62Plus | TenantCategory::DisabledPersonEligible
    ) {
        if input.eviction_attempted_against_senior_or_disabled {
            return Output {
                mode: NycCoopConversionMode::ViolationSeniorOrDisabledTenantEvictionAttempted,
                statutory_basis: "GBL § 352-e(2-a) + § 352-eee + § 352-eeee — senior 62+/disabled PERMANENT eviction protection".to_string(),
                notes: "VIOLATION: senior 62+ or disabled tenant facing eviction under conversion plan; GBL § 352-e(2-a) provides PERMANENT protection; no eviction by holders of unsold shares or subsequent purchasers permitted.".to_string(),
                citations,
            };
        }
        return Output {
            mode: NycCoopConversionMode::CompliantSeniorOrDisabledTenantPermanentlyProtected,
            statutory_basis: "GBL § 352-e(2-a) + § 352-eee — senior/disabled permanently protected".to_string(),
            notes: "COMPLIANT: senior 62+ or disabled tenant entitled to PERMANENT eviction protection; landlord respected statutory shield.".to_string(),
            citations,
        };
    }

    let required_threshold = match input.plan_type {
        ConversionPlanType::EvictionPlan51Pct => GBL_352_EEEE_EVICTION_PLAN_THRESHOLD_BASIS_POINTS,
        ConversionPlanType::NonEvictionPlan51PctLargeBuilding => {
            GBL_352_EEEE_NON_EVICTION_PLAN_THRESHOLD_BASIS_POINTS
        }
        ConversionPlanType::NonEvictionPlanSmallBuilding15Pct => {
            if input.total_units_in_building > GBL_352_EEEE_SMALL_BUILDING_MAX_UNITS {
                return Output {
                    mode: NycCoopConversionMode::ViolationConversionPlanDeclaredEffectiveBelowSubscriptionThreshold,
                    statutory_basis: "GBL § 352-eeee small-building 15 % threshold requires ≤ 5 units".to_string(),
                    notes: format!(
                        "VIOLATION: claimed small-building 15 % threshold but building has {} units (> 5 statutory maximum).",
                        input.total_units_in_building
                    ),
                    citations,
                };
            }
            GBL_352_EEEE_SMALL_BUILDING_NON_EVICTION_THRESHOLD_BASIS_POINTS
        }
        ConversionPlanType::NoPlanFiledYet => unreachable!(),
    };

    if input.bona_fide_tenant_subscription_basis_points < required_threshold {
        return Output {
            mode: NycCoopConversionMode::ViolationConversionPlanDeclaredEffectiveBelowSubscriptionThreshold,
            statutory_basis: format!(
                "GBL § 352-eeee — subscription threshold {} basis points",
                required_threshold
            ),
            notes: format!(
                "VIOLATION: plan declared effective with {} basis points of subscriptions, below required {} basis points threshold for plan type {:?}.",
                input.bona_fide_tenant_subscription_basis_points, required_threshold, input.plan_type
            ),
            citations,
        };
    }

    if !input.tenant_90_day_exclusive_purchase_right_honored {
        return Output {
            mode: NycCoopConversionMode::ViolationTenant90DayExclusivePurchaseRightDenied,
            statutory_basis: "GBL § 352-eeee — 90-day exclusive purchase right".to_string(),
            notes: "VIOLATION: tenant in occupancy denied 90-day exclusive right to purchase dwelling unit after plan accepted for filing.".to_string(),
            citations,
        };
    }

    if !input.tenant_6_month_right_of_first_refusal_honored {
        return Output {
            mode: NycCoopConversionMode::ViolationTenantRightOfFirstRefusalDenied,
            statutory_basis: "GBL § 352-eeee — 6-month right of first refusal".to_string(),
            notes: "VIOLATION: tenant in occupancy denied 6-month right of first refusal after expiration of 90-day exclusive period.".to_string(),
            citations,
        };
    }

    if input.eviction_attempted_against_non_purchasing_tenant {
        if input.eviction_reason_non_payment_or_illegal_use {
            return Output {
                mode: NycCoopConversionMode::CompliantNonPurchasingTenant3YearGracePeriod,
                statutory_basis: "GBL § 352-eeee — eviction for rent non-payment or illegal use permitted".to_string(),
                notes: format!(
                    "COMPLIANT: eviction of non-purchasing tenant for non-payment of rent or illegal use of premises; {} years since plan declared effective.",
                    input.years_since_plan_declared_effective
                ),
                citations,
            };
        }
        if input.years_since_plan_declared_effective < GBL_352_EEEE_3_YEAR_GRACE_PERIOD_YEARS {
            return Output {
                mode: NycCoopConversionMode::ViolationNonPurchasingTenantEvictionBefore3YearPeriod,
                statutory_basis: "GBL § 352-eeee — 3-year grace period for non-purchasing tenants under eviction plan".to_string(),
                notes: format!(
                    "VIOLATION: eviction of non-purchasing tenant attempted only {} years after plan declared effective; minimum 3-year grace period required (or lease expiration, whichever later).",
                    input.years_since_plan_declared_effective
                ),
                citations,
            };
        }
        if !input.eviction_reason_non_payment_or_illegal_use
            && input.plan_type != ConversionPlanType::EvictionPlan51Pct
        {
            return Output {
                mode: NycCoopConversionMode::ViolationEvictionForReasonOtherThanRentOrIllegalUse,
                statutory_basis: "GBL § 352-eeee — non-eviction plan tenants cannot be evicted for failure to purchase".to_string(),
                notes: "VIOLATION: non-eviction plan tenants may only be evicted for non-payment of rent, illegal use, or similar breaches; not for failure to purchase or expiration of tenancy.".to_string(),
                citations,
            };
        }
    }

    match input.plan_type {
        ConversionPlanType::EvictionPlan51Pct => Output {
            mode: NycCoopConversionMode::CompliantEvictionPlanWith51PctSubscription,
            statutory_basis: "GBL § 352-eeee eviction plan — 51 % subscription threshold satisfied".to_string(),
            notes: format!(
                "COMPLIANT: eviction plan with {} basis points subscriptions (≥ 5,100 required); tenant exclusive purchase rights honored.",
                input.bona_fide_tenant_subscription_basis_points
            ),
            citations,
        },
        ConversionPlanType::NonEvictionPlan51PctLargeBuilding => Output {
            mode: NycCoopConversionMode::CompliantNonEvictionPlanWith51PctSubscription,
            statutory_basis: "GBL § 352-eeee non-eviction plan — 51 % subscription threshold satisfied".to_string(),
            notes: format!(
                "COMPLIANT: non-eviction plan with {} basis points subscriptions (≥ 5,100 required); tenant exclusive purchase rights honored.",
                input.bona_fide_tenant_subscription_basis_points
            ),
            citations,
        },
        ConversionPlanType::NonEvictionPlanSmallBuilding15Pct => Output {
            mode: NycCoopConversionMode::CompliantNonEvictionPlanWith15PctSmallBuildingException,
            statutory_basis: "GBL § 352-eeee non-eviction plan — 15 % small-building exception satisfied".to_string(),
            notes: format!(
                "COMPLIANT: non-eviction plan with {} basis points subscriptions (≥ 1,500 required for ≤ 5 unit building with sponsor/family 2-year occupancy).",
                input.bona_fide_tenant_subscription_basis_points
            ),
            citations,
        },
        ConversionPlanType::NoPlanFiledYet => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_non_eviction_plan_compliant() -> Input {
        Input {
            plan_type: ConversionPlanType::NonEvictionPlan51PctLargeBuilding,
            building_scenario: BuildingScenario::NycResidentialMultipleDwellingLargeBuilding,
            tenant_category: TenantCategory::TenantOfRecordInOccupancyUnregulated,
            bona_fide_tenant_subscription_basis_points: 5_500,
            total_units_in_building: 50,
            years_since_plan_declared_effective: 0,
            eviction_attempted_against_senior_or_disabled: false,
            eviction_attempted_against_non_purchasing_tenant: false,
            tenant_90_day_exclusive_purchase_right_honored: true,
            tenant_6_month_right_of_first_refusal_honored: true,
            eviction_reason_non_payment_or_illegal_use: false,
        }
    }

    #[test]
    fn no_plan_filed_not_applicable() {
        let input = Input {
            plan_type: ConversionPlanType::NoPlanFiledYet,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::NotApplicableNoConversionInProgress
        );
    }

    #[test]
    fn westchester_separate_statute_not_applicable() {
        let input = Input {
            building_scenario: BuildingScenario::WestchesterRocklandNassauSeparateStatute,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::NotApplicableNotInNycSubjectToSeparateStatute
        );
    }

    #[test]
    fn non_eviction_plan_51_pct_compliant() {
        let result = check(&baseline_non_eviction_plan_compliant());
        assert_eq!(
            result.mode,
            NycCoopConversionMode::CompliantNonEvictionPlanWith51PctSubscription
        );
    }

    #[test]
    fn non_eviction_plan_below_51_pct_violation() {
        let input = Input {
            bona_fide_tenant_subscription_basis_points: 5_000,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, NycCoopConversionMode::ViolationConversionPlanDeclaredEffectiveBelowSubscriptionThreshold);
    }

    #[test]
    fn non_eviction_plan_at_exactly_51_pct_compliant() {
        let input = Input {
            bona_fide_tenant_subscription_basis_points: 5_100,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::CompliantNonEvictionPlanWith51PctSubscription
        );
    }

    #[test]
    fn eviction_plan_51_pct_compliant() {
        let input = Input {
            plan_type: ConversionPlanType::EvictionPlan51Pct,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::CompliantEvictionPlanWith51PctSubscription
        );
    }

    #[test]
    fn small_building_15_pct_non_eviction_compliant() {
        let input = Input {
            plan_type: ConversionPlanType::NonEvictionPlanSmallBuilding15Pct,
            building_scenario: BuildingScenario::NycSmallBuilding5OrFewerUnits,
            total_units_in_building: 4,
            bona_fide_tenant_subscription_basis_points: 1_500,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::CompliantNonEvictionPlanWith15PctSmallBuildingException
        );
    }

    #[test]
    fn small_building_15_pct_with_6_units_violation() {
        let input = Input {
            plan_type: ConversionPlanType::NonEvictionPlanSmallBuilding15Pct,
            total_units_in_building: 6,
            bona_fide_tenant_subscription_basis_points: 1_500,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, NycCoopConversionMode::ViolationConversionPlanDeclaredEffectiveBelowSubscriptionThreshold);
    }

    #[test]
    fn senior_62_plus_permanently_protected_compliant() {
        let input = Input {
            tenant_category: TenantCategory::SeniorCitizen62Plus,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::CompliantSeniorOrDisabledTenantPermanentlyProtected
        );
    }

    #[test]
    fn senior_eviction_attempt_violation() {
        let input = Input {
            tenant_category: TenantCategory::SeniorCitizen62Plus,
            eviction_attempted_against_senior_or_disabled: true,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::ViolationSeniorOrDisabledTenantEvictionAttempted
        );
    }

    #[test]
    fn disabled_eviction_attempt_violation() {
        let input = Input {
            tenant_category: TenantCategory::DisabledPersonEligible,
            eviction_attempted_against_senior_or_disabled: true,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::ViolationSeniorOrDisabledTenantEvictionAttempted
        );
    }

    #[test]
    fn ninety_day_exclusive_purchase_right_denied_violation() {
        let input = Input {
            tenant_90_day_exclusive_purchase_right_honored: false,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::ViolationTenant90DayExclusivePurchaseRightDenied
        );
    }

    #[test]
    fn six_month_right_of_first_refusal_denied_violation() {
        let input = Input {
            tenant_6_month_right_of_first_refusal_honored: false,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::ViolationTenantRightOfFirstRefusalDenied
        );
    }

    #[test]
    fn non_purchasing_tenant_eviction_before_3_years_violation() {
        let input = Input {
            plan_type: ConversionPlanType::EvictionPlan51Pct,
            eviction_attempted_against_non_purchasing_tenant: true,
            years_since_plan_declared_effective: 2,
            eviction_reason_non_payment_or_illegal_use: false,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::ViolationNonPurchasingTenantEvictionBefore3YearPeriod
        );
    }

    #[test]
    fn non_purchasing_tenant_eviction_after_3_years_compliant() {
        let input = Input {
            plan_type: ConversionPlanType::EvictionPlan51Pct,
            eviction_attempted_against_non_purchasing_tenant: true,
            years_since_plan_declared_effective: 4,
            eviction_reason_non_payment_or_illegal_use: false,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::CompliantEvictionPlanWith51PctSubscription
        );
    }

    #[test]
    fn non_payment_eviction_compliant_anytime() {
        let input = Input {
            eviction_attempted_against_non_purchasing_tenant: true,
            years_since_plan_declared_effective: 1,
            eviction_reason_non_payment_or_illegal_use: true,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::CompliantNonPurchasingTenant3YearGracePeriod
        );
    }

    #[test]
    fn non_eviction_plan_tenant_eviction_for_non_breach_violation() {
        let input = Input {
            plan_type: ConversionPlanType::NonEvictionPlan51PctLargeBuilding,
            eviction_attempted_against_non_purchasing_tenant: true,
            years_since_plan_declared_effective: 5,
            eviction_reason_non_payment_or_illegal_use: false,
            ..baseline_non_eviction_plan_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycCoopConversionMode::ViolationEvictionForReasonOtherThanRentOrIllegalUse
        );
    }

    #[test]
    fn citations_pin_gbl_352_eeee_subsections_and_pending_amendments() {
        let result = check(&baseline_non_eviction_plan_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("NY GBL § 352-eeee"));
        assert!(joined.contains("Martin Act Article 23-A"));
        assert!(joined.contains("51 % of bona fide tenants"));
        assert!(joined.contains("3 years after plan declared effective"));
        assert!(joined.contains("15 % for buildings ≤ 5 units"));
        assert!(joined.contains("GBL § 352-e(2-a)"));
        assert!(joined.contains("GBL § 352-eee"));
        assert!(joined.contains("62+"));
        assert!(joined.contains("disabled"));
        assert!(joined.contains("90-day exclusive purchase"));
        assert!(joined.contains("6-month right of first refusal"));
        assert!(joined.contains("GBL § 352-eeeee"));
        assert!(joined.contains("13 NYCRR Part 18"));
        assert!(joined.contains("NY Senate S3758"));
        assert!(joined.contains("NY Senate S4910"));
        assert!(joined.contains("HSTPA 2019"));
    }

    #[test]
    fn constant_pin_thresholds_and_grace_period() {
        assert_eq!(GBL_352_EEEE_EVICTION_PLAN_THRESHOLD_BASIS_POINTS, 5_100);
        assert_eq!(GBL_352_EEEE_NON_EVICTION_PLAN_THRESHOLD_BASIS_POINTS, 5_100);
        assert_eq!(
            GBL_352_EEEE_SMALL_BUILDING_NON_EVICTION_THRESHOLD_BASIS_POINTS,
            1_500
        );
        assert_eq!(GBL_352_EEEE_3_YEAR_GRACE_PERIOD_YEARS, 3);
        assert_eq!(GBL_352_EEEE_90_DAY_EXCLUSIVE_PURCHASE_DAYS, 90);
        assert_eq!(GBL_352_EEEE_6_MONTH_RIGHT_FIRST_REFUSAL_MONTHS, 6);
        assert_eq!(GBL_352_EEEE_SENIOR_AGE_THRESHOLD, 62);
        assert_eq!(GBL_352_EEEE_SMALL_BUILDING_MAX_UNITS, 5);
        assert_eq!(GBL_352_EEEE_SPONSOR_FAMILY_OCCUPANCY_MIN_YEARS, 2);
        assert_eq!(NY_S3758_PENDING_YEAR, 2025);
        assert_eq!(NY_S4910_PENDING_YEAR, 2025);
    }
}
