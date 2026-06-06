//! Oakland Just Cause for Eviction Ordinance (Measure EE,
//! Oakland Municipal Code § 8.22 Article II) Compliance
//! Module — voter-enacted 2002 just-cause regime that
//! complements the rent-control rules at OMC § 8.22 Article
//! I (Residential Rent Adjustment Program).
//!
//! Pure-compute check for landlord compliance with Oakland's
//! Just Cause for Eviction Ordinance, adopted by voters of
//! Oakland in November 2002 as **MEASURE EE** and codified
//! at Oakland Municipal Code (OMC) Title 8 (Health and
//! Safety), Chapter 8.22 (Residential Rent Adjustments and
//! Evictions), Article II (Just Cause for Eviction
//! Ordinance / Measure EE). The ordinance was substantially
//! amended in November 2022 by voter approval of
//! **MEASURE V**, which broadened protected-tenant
//! categories and increased landlord penalties for
//! violations.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Oakland voters approved **Measure EE** in November 2002; codified at OMC Title 8 Chapter 8.22 Article II §§ 8.22.300-8.22.470 ([Oakland Municipal Code § 8.22.300 Just Cause for Eviction Introductory Clauses](http://oakland-ca.elaws.us/code/coor_title8_ch8.22_artii_sec8.22.300); [Oakland Municipal Code § 8.22.350 Applicability and Exemptions](http://oakland-ca.elaws.us/code/coor_title8_ch8.22_artii_sec8.22.350); [Oakland Municipal Code § 8.22.360 Good cause required for eviction](http://oakland-ca.elaws.us/code/coor_title8_ch8.22_artii_sec8.22.360); [Oakland Municipal Code § 8.22.370 Remedies](http://oakland-ca.elaws.us/code/coor_title8_ch8.22_artii_sec8.22.370); [Municode Library — Oakland Article II](https://library.municode.com/ca/oakland/codes/code_of_ordinances/283553?nodeId=TIT8HESA_CH8.22REREADEV_ARTIIJUCAEVORMEEE_8.22.350AP); [Hooshmand Law Group — Oakland Just Cause for Eviction Ordinance](https://hooshmandlawgroup.com/oakland-just-cause-for-eviction-ordinance-specifies-reasons-a-landlord-may-legally-evict-a-tenant/); [SPUR — Oakland Measure V Just Cause Amendment 2022](https://www.spur.org/voter-guide/2022-11/oak-measure-v-just-cause-amendment); [Bracamontes & Vlasak — Oakland Just Cause Eviction Lawyers](https://bvlawsf.com/landlord-tenant-law/oakland-just-cause-eviction/)).
//! - **Applicability and Exemptions (OMC § 8.22.350)**: applies to buildings with **2 OR MORE UNITS** with certificate of occupancy issued **BEFORE JANUARY 1, 1983**. Specific exemptions include hospitals; nonprofit-operated housing; owner-occupied buildings (depending on configuration); units with first certificate of occupancy after January 1, 1983 (or after 2003 for certain new-construction carve-outs); units governed by other state or federal law that preempts local just-cause regulation; transient hotel occupancy.
//! - **11 Just Cause Grounds under OMC § 8.22.360**: (1) **non-payment of rent** after written notice requiring payment within not less than 3 days; (2) **breach of lease terms** or failure to sign a lease extension/renewal with materially the same terms; (3) **willful damage** to the property; (4) **disorderly conduct** affecting other tenants' peace and quiet; (5) **illegal use** of the rental unit; (6) **denial of landlord access** for lawful purposes after written notice; (7) **substantial repairs** requiring temporary relocation (typically 3-month displacement); (8) **owner or relative move-in** as principal residence (owner, spouse, domestic partner, child, parent, or grandparent); (9) **Ellis Act withdrawal** from rental housing market under California Government Code § 7060 et seq.; (10) **demolition** with valid permits; (11) **end of a temporary tenancy** or other narrow specified circumstance.
//! - **Owner Move-In Restrictions Against Protected Tenants**: tenants who are **AGE 60 OR OLDER WITH AT LEAST 5 YEARS' TENURE** in the unit OR **TENANTS WITH A DISABILITY** are PROTECTED from owner / relative move-in evictions; the protected-tenant exemption is the most-litigated provision of the ordinance. Measure V (2022) expanded the protected-tenant categories to include tenants with **CATASTROPHIC ILLNESS**.
//! - **Ellis Act Withdrawal Procedures**: California Government Code § 7060 et seq. (the Ellis Act) requires withdrawal-from-rental-housing-market notice; standard notice period **120 DAYS** for most tenants; **EXTENDED 1-YEAR (365-DAY) notice** for **SENIOR (62+) OR DISABLED** tenants who have resided in the unit for at least 1 year before the date of delivery of the notice of intent to withdraw.
//! - **Costa-Hawkins Rental Housing Act of 1995** (CA state law): vacancy decontrol overlay exempts single-family homes, condominiums, and post-Feb-1-1995 certificate-of-occupancy buildings from local rent-price control; **JUST-CAUSE EVICTION PROTECTION CONTINUES TO APPLY** during an ongoing tenancy under OMC § 8.22.360 even for Costa-Hawkins-exempt units.
//! - **Notice of Termination Filing Requirement**: landlord must file copy of termination notice with the Oakland Rent Adjustment Program (RAP) within statutory window of service on the tenant; notice must state just cause AND reference the Rent Adjustment Program counseling services.
//! - **Remedies under OMC § 8.22.370**: tenant remedy for violation includes statutory damages, actual damages, treble damages for willful violations, reasonable attorney's fees, and injunctive relief (Measure V (2022) materially increased the penalty structure).
//! - **Measure V (2022) Amendments**: voter-approved amendment that (1) expanded protected-tenant categories to include tenants with catastrophic illness; (2) increased civil penalties for landlord violations; (3) added stronger anti-harassment protections; (4) extended just-cause eviction protection to certain previously-exempt single-family homes and duplexes ([SPUR — Oakland Measure V Just Cause Amendment](https://www.spur.org/voter-guide/2022-11/oak-measure-v-just-cause-amendment)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const OAKLAND_MEASURE_EE_ENACTMENT_YEAR: u32 = 2002;
pub const OAKLAND_MEASURE_V_AMENDMENT_YEAR: u32 = 2022;
pub const OAKLAND_CERTIFICATE_OF_OCCUPANCY_CUTOFF_YEAR: u32 = 1983;
pub const OAKLAND_CERTIFICATE_OF_OCCUPANCY_CUTOFF_MONTH: u32 = 1;
pub const OAKLAND_CERTIFICATE_OF_OCCUPANCY_CUTOFF_DAY: u32 = 1;
pub const OAKLAND_NEW_CONSTRUCTION_EXEMPTION_CUTOFF_YEAR: u32 = 2003;
pub const OAKLAND_NUMBER_OF_JUST_CAUSE_GROUNDS: u32 = 11;
pub const OAKLAND_MIN_UNITS_FOR_COVERAGE: u32 = 2;
pub const OAKLAND_PROTECTED_TENANT_AGE_THRESHOLD_YEARS: u32 = 60;
pub const OAKLAND_PROTECTED_TENANT_TENURE_YEARS: u32 = 5;
pub const OAKLAND_NON_PAYMENT_NOTICE_DAYS: u32 = 3;
pub const OAKLAND_SUBSTANTIAL_REPAIRS_TEMPORARY_RELOCATION_MONTHS: u32 = 3;
pub const OAKLAND_ELLIS_ACT_SENIOR_DISABLED_NOTICE_DAYS: u32 = 365;
pub const OAKLAND_ELLIS_ACT_STANDARD_NOTICE_DAYS: u32 = 120;
pub const OAKLAND_ELLIS_ACT_SENIOR_AGE_THRESHOLD: u32 = 62;
pub const OAKLAND_ELLIS_ACT_SENIOR_TENURE_YEARS: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinOaklandCityLimits,
    OutsideOaklandCityLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    StandardCoveredApartment,
    SingleFamilyHomeCostaHawkinsApplies,
    CondominiumUnitCostaHawkinsApplies,
    NonResidentialUnitExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExemptionStatus {
    NotExemptFullyCovered,
    OwnerOccupiedBuildingExemption,
    HospitalOrNonprofitOperatedHousingExempt,
    NewlyConstructedAfter2003Exempt,
    PreemptedByStateOrFederalLawExempt,
    TransientHotelOccupancyExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateOfOccupancyDateStatus {
    IssuedBeforeJanuary1_1983CoveredByOmc822360,
    IssuedOnOrAfterJanuary1_1983ExemptFromMeasureEe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JustCauseGroundAsserted {
    NonPaymentOfRentAfter3DayNoticeOmc822360A,
    BreachOfLeaseTermsOrLeaseRenewalRefusalOmc822360B,
    WillfulDamageToPropertyOmc822360C,
    DisorderlyConductAffectingPeaceOmc822360D,
    IllegalUseOfRentalUnitOmc822360E,
    DenialOfLandlordAccessAfterWrittenNoticeOmc822360F,
    SubstantialRepairsRequiringTemporaryRelocationOmc822360G,
    OwnerOrRelativeMoveInPrincipalResidenceOmc822360H,
    EllisActWithdrawalUnderCaliforniaGovCode7060Omc822360I,
    DemolitionWithValidPermitsOmc822360J,
    EndOfTemporaryTenancyOrOtherNarrowGroundOmc822360K,
    NoJustCauseAsserted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedTenantStatus {
    NotProtected,
    AgeAtOrAbove60WithAtLeast5YearTenureProtectedFromOwnerMoveIn,
    DisabledTenantProtectedFromOwnerMoveIn,
    CatastrophicIllnessProtectedFromOwnerMoveInUnderMeasureV2022,
    NotApplicableNonOwnerMoveInCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    JustCauseEvictionScopeCheck,
    OwnerMoveInProtectedTenantCheck,
    EllisActWithdrawalNoticeRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OaklandMeasureEeMode {
    NotApplicablePropertyOutsideOakland,
    NotApplicableNonResidentialUnit,
    NotApplicableSingleUnitPropertyBelow2UnitMinimum,
    NotApplicableCertificateOfOccupancyOnOrAfterJanuary1_1983,
    NotApplicableNewlyConstructedAfter2003Exempt,
    NotApplicableOwnerOccupiedBuildingExempt,
    NotApplicableHospitalOrNonprofitExempt,
    NotApplicablePreemptedByStateOrFederalLawExempt,
    NotApplicableTransientHotelOccupancyExempt,
    NotApplicableCostaHawkinsSingleFamilyOrCondoVacancyDecontrolApplies,
    CompliantJustCauseEvictionUnderOneOfElevenSection822360Grounds,
    CompliantOwnerMoveInAgainstNonProtectedTenant,
    CompliantEllisActWithdrawalStandard120DayNotice,
    CompliantEllisActWithdrawalExtended365DayNoticeForSeniorOrDisabled,
    ViolationEvictionWithoutOneOfElevenSection822360JustCauseGrounds,
    ViolationOwnerMoveInAgainstProtectedTenant60PlusWith5YearTenureOrDisabledOrCatastrophicIllness,
    ViolationEllisActWithdrawalInsufficientNoticeForSeniorOrDisabledTenant,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub unit_count_at_property: u32,
    pub unit_type: UnitType,
    pub exemption_status: ExemptionStatus,
    pub certificate_of_occupancy_date_status: CertificateOfOccupancyDateStatus,
    pub compliance_aspect: ComplianceAspect,
    pub just_cause_ground_asserted: JustCauseGroundAsserted,
    pub protected_tenant_status: ProtectedTenantStatus,
    pub ellis_act_notice_days_provided: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: OaklandMeasureEeMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub ellis_act_notice_days_required: u32,
}

pub type RentalOaklandMeasureEeJustCauseOmc822Input = Input;
pub type RentalOaklandMeasureEeJustCauseOmc822Output = Output;
pub type RentalOaklandMeasureEeJustCauseOmc822Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Oakland Just Cause for Eviction Ordinance — Measure EE adopted by voters of Oakland in November 2002; codified at Oakland Municipal Code Title 8 Chapter 8.22 Article II (§§ 8.22.300 through 8.22.470); administered by the Oakland Rent Adjustment Program (RAP)".to_string(),
        "OMC § 8.22.350 Applicability and Exemptions — applies to buildings with 2 OR MORE UNITS with certificate of occupancy issued BEFORE JANUARY 1, 1983; exemptions include hospitals; nonprofit-operated housing; owner-occupied buildings (depending on configuration); units with first certificate of occupancy after January 1, 1983 (or after 2003 for certain new-construction carve-outs); units governed by state/federal law preempting local just-cause; transient hotel occupancy".to_string(),
        "OMC § 8.22.360 11 Just Cause Grounds — (1) non-payment of rent after 3-day notice; (2) breach of lease or refusal to sign renewal with materially identical terms; (3) willful damage; (4) disorderly conduct affecting peace; (5) illegal use; (6) denial of landlord access after written notice; (7) substantial repairs requiring temporary relocation; (8) owner / relative move-in as principal residence (owner, spouse, domestic partner, child, parent, grandparent); (9) Ellis Act withdrawal under California Government Code § 7060 et seq.; (10) demolition with valid permits; (11) end of temporary tenancy or other narrow specified ground".to_string(),
        "Owner Move-In Protected Tenants — tenants 60 OR OLDER WITH AT LEAST 5 YEARS' TENURE OR tenants with a DISABILITY are PROTECTED from owner / relative move-in evictions; Measure V (2022) expanded protected categories to include tenants with CATASTROPHIC ILLNESS".to_string(),
        "Ellis Act Withdrawal Procedures (California Government Code § 7060 et seq.) — standard withdrawal notice period 120 DAYS for most tenants; EXTENDED 1-YEAR (365-DAY) notice for SENIOR (62+) OR DISABLED tenants who have resided in unit for at least 1 year before date of delivery of notice of intent to withdraw".to_string(),
        "Costa-Hawkins Rental Housing Act of 1995 (CA state law) — vacancy decontrol overlay exempts single-family homes, condominiums, and post-Feb-1-1995 certificate-of-occupancy buildings from local rent-price control; JUST-CAUSE EVICTION PROTECTION CONTINUES TO APPLY during ongoing tenancy under OMC § 8.22.360 even for Costa-Hawkins-exempt units".to_string(),
        "Notice of Termination Filing — landlord must file copy of termination notice with Oakland Rent Adjustment Program within statutory window of service on tenant; notice must state just cause AND reference RAP counseling services".to_string(),
        "OMC § 8.22.370 Remedies — statutory damages + actual damages + treble damages for willful violations + reasonable attorney's fees + injunctive relief (Measure V (2022) materially increased penalty structure)".to_string(),
        "Measure V (2022) Amendments — voter-approved amendment that (1) expanded protected-tenant categories to include tenants with catastrophic illness; (2) increased civil penalties for landlord violations; (3) added stronger anti-harassment protections; (4) extended just-cause eviction protection to certain previously-exempt single-family homes and duplexes".to_string(),
        "Oakland Municipal Code § 8.22.300 — Just Cause for Eviction Introductory Clauses".to_string(),
        "Oakland Municipal Code § 8.22.350 — Applicability and Exemptions".to_string(),
        "Oakland Municipal Code § 8.22.360 — Good cause required for eviction".to_string(),
        "Oakland Municipal Code § 8.22.370 — Remedies".to_string(),
        "Municode Library — Oakland Article II Just Cause for Eviction Ordinance Measure EE".to_string(),
        "Hooshmand Law Group — Oakland Just Cause for Eviction Ordinance practitioner guide".to_string(),
        "SPUR — Oakland Measure V Just Cause Amendment 2022 voter guide".to_string(),
        "Bracamontes & Vlasak — Oakland Just Cause Eviction Lawyers practitioner guide".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideOaklandCityLimits {
        return Output {
            mode: OaklandMeasureEeMode::NotApplicablePropertyOutsideOakland,
            statutory_basis: "Property outside Oakland city limits; OMC Chapter 8.22 Article II inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Oakland city limits; Oakland Just Cause for Eviction Ordinance (Measure EE, OMC § 8.22 Article II) inapplicable.".to_string(),
            citations,
            ellis_act_notice_days_required: 0,
        };
    }

    if input.unit_type == UnitType::NonResidentialUnitExempt {
        return Output {
            mode: OaklandMeasureEeMode::NotApplicableNonResidentialUnit,
            statutory_basis: "OMC § 8.22 Article II applies only to residential rental units".to_string(),
            notes: "NOT APPLICABLE: unit is non-residential; OMC § 8.22 Article II applies only to residential rental units.".to_string(),
            citations,
            ellis_act_notice_days_required: 0,
        };
    }

    if input.unit_count_at_property < OAKLAND_MIN_UNITS_FOR_COVERAGE {
        return Output {
            mode: OaklandMeasureEeMode::NotApplicableSingleUnitPropertyBelow2UnitMinimum,
            statutory_basis: "OMC § 8.22.350 — applies to buildings with 2 or more units; single-unit properties exempt".to_string(),
            notes: format!(
                "NOT APPLICABLE: property has {} unit(s) (< 2-unit statutory minimum under OMC § 8.22.350); Measure EE just-cause eviction protection does not apply.",
                input.unit_count_at_property
            ),
            citations,
            ellis_act_notice_days_required: 0,
        };
    }

    if input.certificate_of_occupancy_date_status
        == CertificateOfOccupancyDateStatus::IssuedOnOrAfterJanuary1_1983ExemptFromMeasureEe
    {
        return Output {
            mode: OaklandMeasureEeMode::NotApplicableCertificateOfOccupancyOnOrAfterJanuary1_1983,
            statutory_basis: "OMC § 8.22.350 — certificate of occupancy issued on or after January 1, 1983 exempt from Measure EE".to_string(),
            notes: "NOT APPLICABLE: certificate of occupancy issued on or after January 1, 1983; building exempt from Measure EE just-cause eviction protection.".to_string(),
            citations,
            ellis_act_notice_days_required: 0,
        };
    }

    match input.exemption_status {
        ExemptionStatus::NewlyConstructedAfter2003Exempt => {
            return Output {
                mode: OaklandMeasureEeMode::NotApplicableNewlyConstructedAfter2003Exempt,
                statutory_basis: "OMC § 8.22.350 — newly constructed after 2003 exemption".to_string(),
                notes: "NOT APPLICABLE: newly constructed after 2003; OMC § 8.22.350 new-construction exemption applies.".to_string(),
                citations,
                ellis_act_notice_days_required: 0,
            };
        }
        ExemptionStatus::OwnerOccupiedBuildingExemption => {
            return Output {
                mode: OaklandMeasureEeMode::NotApplicableOwnerOccupiedBuildingExempt,
                statutory_basis: "OMC § 8.22.350 — owner-occupied building exemption".to_string(),
                notes: "NOT APPLICABLE: owner-occupied building (depending on configuration); OMC § 8.22.350 owner-occupied exemption applies.".to_string(),
                citations,
                ellis_act_notice_days_required: 0,
            };
        }
        ExemptionStatus::HospitalOrNonprofitOperatedHousingExempt => {
            return Output {
                mode: OaklandMeasureEeMode::NotApplicableHospitalOrNonprofitExempt,
                statutory_basis: "OMC § 8.22.350 — hospital or nonprofit-operated housing exemption".to_string(),
                notes: "NOT APPLICABLE: hospital or nonprofit-operated housing; OMC § 8.22.350 hospital/nonprofit exemption applies.".to_string(),
                citations,
                ellis_act_notice_days_required: 0,
            };
        }
        ExemptionStatus::PreemptedByStateOrFederalLawExempt => {
            return Output {
                mode: OaklandMeasureEeMode::NotApplicablePreemptedByStateOrFederalLawExempt,
                statutory_basis: "OMC § 8.22.350 — preemption by state or federal law exemption".to_string(),
                notes: "NOT APPLICABLE: unit governed by state or federal law that preempts local just-cause regulation; OMC § 8.22.350 preemption exemption applies.".to_string(),
                citations,
                ellis_act_notice_days_required: 0,
            };
        }
        ExemptionStatus::TransientHotelOccupancyExempt => {
            return Output {
                mode: OaklandMeasureEeMode::NotApplicableTransientHotelOccupancyExempt,
                statutory_basis: "OMC § 8.22.350 — transient hotel occupancy exemption".to_string(),
                notes: "NOT APPLICABLE: transient hotel occupancy; OMC § 8.22.350 hotel-occupancy exemption applies.".to_string(),
                citations,
                ellis_act_notice_days_required: 0,
            };
        }
        ExemptionStatus::NotExemptFullyCovered => {}
    }

    let costa_hawkins_applies = matches!(
        input.unit_type,
        UnitType::SingleFamilyHomeCostaHawkinsApplies
            | UnitType::CondominiumUnitCostaHawkinsApplies
    );

    if costa_hawkins_applies
        && input.compliance_aspect == ComplianceAspect::JustCauseEvictionScopeCheck
    {
        // Costa-Hawkins exempts the unit from local rent-price control but NOT from just-cause eviction protection during ongoing tenancy.
        // Continue with just-cause analysis below.
    }

    match input.compliance_aspect {
        ComplianceAspect::JustCauseEvictionScopeCheck => match input.just_cause_ground_asserted {
            JustCauseGroundAsserted::NoJustCauseAsserted => Output {
                mode: OaklandMeasureEeMode::ViolationEvictionWithoutOneOfElevenSection822360JustCauseGrounds,
                statutory_basis: "OMC § 8.22.360 — eviction prohibited without one of the 11 enumerated just-cause grounds".to_string(),
                notes: "VIOLATION: landlord served termination notice without asserting any of the 11 enumerated just-cause grounds under OMC § 8.22.360; termination notice unenforceable; tenant may assert as affirmative defense in unlawful detainer.".to_string(),
                citations,
                ellis_act_notice_days_required: 0,
            },
            _ => Output {
                mode: OaklandMeasureEeMode::CompliantJustCauseEvictionUnderOneOfElevenSection822360Grounds,
                statutory_basis: "OMC § 8.22.360 — eviction under one of the 11 enumerated just-cause grounds".to_string(),
                notes: format!(
                    "COMPLIANT: just-cause eviction asserted under one of the 11 enumerated grounds in OMC § 8.22.360 ({:?}); separate filing with Rent Adjustment Program required.",
                    input.just_cause_ground_asserted
                ),
                citations,
                ellis_act_notice_days_required: 0,
            },
        },
        ComplianceAspect::OwnerMoveInProtectedTenantCheck => match input.protected_tenant_status {
            ProtectedTenantStatus::AgeAtOrAbove60WithAtLeast5YearTenureProtectedFromOwnerMoveIn
            | ProtectedTenantStatus::DisabledTenantProtectedFromOwnerMoveIn
            | ProtectedTenantStatus::CatastrophicIllnessProtectedFromOwnerMoveInUnderMeasureV2022 => Output {
                mode: OaklandMeasureEeMode::ViolationOwnerMoveInAgainstProtectedTenant60PlusWith5YearTenureOrDisabledOrCatastrophicIllness,
                statutory_basis: "OMC § 8.22.360(8) — owner / relative move-in PROHIBITED against protected tenants".to_string(),
                notes: format!(
                    "VIOLATION: owner / relative move-in asserted against protected tenant ({:?}); OMC § 8.22.360 protected-tenant exemption blocks the eviction; Measure V (2022) expanded protected categories to include catastrophic illness; termination notice unenforceable.",
                    input.protected_tenant_status
                ),
                citations,
                ellis_act_notice_days_required: 0,
            },
            ProtectedTenantStatus::NotProtected | ProtectedTenantStatus::NotApplicableNonOwnerMoveInCase => Output {
                mode: OaklandMeasureEeMode::CompliantOwnerMoveInAgainstNonProtectedTenant,
                statutory_basis: "OMC § 8.22.360(8) — owner / relative move-in against non-protected tenant".to_string(),
                notes: "COMPLIANT: owner / relative move-in asserted against non-protected tenant (tenant is not 60+ with 5-year tenure, not disabled, not catastrophically ill); OMC § 8.22.360(8) does not block; additional substantive OMI requirements (good-faith intent, principal residence, continuous occupancy) apply separately.".to_string(),
                citations,
                ellis_act_notice_days_required: 0,
            },
        },
        ComplianceAspect::EllisActWithdrawalNoticeRequirement => {
            let senior_or_disabled = matches!(
                input.protected_tenant_status,
                ProtectedTenantStatus::AgeAtOrAbove60WithAtLeast5YearTenureProtectedFromOwnerMoveIn
                    | ProtectedTenantStatus::DisabledTenantProtectedFromOwnerMoveIn
                    | ProtectedTenantStatus::CatastrophicIllnessProtectedFromOwnerMoveInUnderMeasureV2022
            );
            if senior_or_disabled {
                if input.ellis_act_notice_days_provided < OAKLAND_ELLIS_ACT_SENIOR_DISABLED_NOTICE_DAYS {
                    Output {
                        mode: OaklandMeasureEeMode::ViolationEllisActWithdrawalInsufficientNoticeForSeniorOrDisabledTenant,
                        statutory_basis: "California Government Code § 7060.4(b) (Ellis Act) — 1-year (365-day) notice required for senior (62+) or disabled tenants with at least 1 year tenure".to_string(),
                        notes: format!(
                            "VIOLATION: Ellis Act withdrawal notice provided was {} days but senior (62+) or disabled tenant requires 365-day extended notice under California Government Code § 7060.4(b); withdrawal notice unenforceable.",
                            input.ellis_act_notice_days_provided
                        ),
                        citations,
                        ellis_act_notice_days_required: OAKLAND_ELLIS_ACT_SENIOR_DISABLED_NOTICE_DAYS,
                    }
                } else {
                    Output {
                        mode: OaklandMeasureEeMode::CompliantEllisActWithdrawalExtended365DayNoticeForSeniorOrDisabled,
                        statutory_basis: "California Government Code § 7060.4(b) — 1-year extended notice for senior or disabled tenants".to_string(),
                        notes: format!(
                            "COMPLIANT: Ellis Act withdrawal notice of {} days satisfies the 365-day extended-notice requirement for senior (62+) or disabled tenants under California Government Code § 7060.4(b).",
                            input.ellis_act_notice_days_provided
                        ),
                        citations,
                        ellis_act_notice_days_required: OAKLAND_ELLIS_ACT_SENIOR_DISABLED_NOTICE_DAYS,
                    }
                }
            } else if input.ellis_act_notice_days_provided < OAKLAND_ELLIS_ACT_STANDARD_NOTICE_DAYS {
                Output {
                    mode: OaklandMeasureEeMode::ViolationEllisActWithdrawalInsufficientNoticeForSeniorOrDisabledTenant,
                    statutory_basis: "California Government Code § 7060.4(a) — 120-day standard Ellis Act withdrawal notice".to_string(),
                    notes: format!(
                        "VIOLATION: standard Ellis Act withdrawal notice of {} days is below the 120-day minimum under California Government Code § 7060.4(a).",
                        input.ellis_act_notice_days_provided
                    ),
                    citations,
                    ellis_act_notice_days_required: OAKLAND_ELLIS_ACT_STANDARD_NOTICE_DAYS,
                }
            } else {
                Output {
                    mode: OaklandMeasureEeMode::CompliantEllisActWithdrawalStandard120DayNotice,
                    statutory_basis: "California Government Code § 7060.4(a) — 120-day standard Ellis Act withdrawal notice".to_string(),
                    notes: format!(
                        "COMPLIANT: standard 120-day Ellis Act withdrawal notice satisfied ({} days provided).",
                        input.ellis_act_notice_days_provided
                    ),
                    citations,
                    ellis_act_notice_days_required: OAKLAND_ELLIS_ACT_STANDARD_NOTICE_DAYS,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinOaklandCityLimits,
            unit_count_at_property: 10,
            unit_type: UnitType::StandardCoveredApartment,
            exemption_status: ExemptionStatus::NotExemptFullyCovered,
            certificate_of_occupancy_date_status:
                CertificateOfOccupancyDateStatus::IssuedBeforeJanuary1_1983CoveredByOmc822360,
            compliance_aspect: ComplianceAspect::JustCauseEvictionScopeCheck,
            just_cause_ground_asserted:
                JustCauseGroundAsserted::NonPaymentOfRentAfter3DayNoticeOmc822360A,
            protected_tenant_status: ProtectedTenantStatus::NotApplicableNonOwnerMoveInCase,
            ellis_act_notice_days_provided: 0,
        }
    }

    #[test]
    fn property_outside_oakland_not_applicable() {
        let mut input = baseline_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideOaklandCityLimits;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::NotApplicablePropertyOutsideOakland
        );
    }

    #[test]
    fn non_residential_not_applicable() {
        let mut input = baseline_input();
        input.unit_type = UnitType::NonResidentialUnitExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::NotApplicableNonResidentialUnit
        );
    }

    #[test]
    fn single_unit_property_below_2_unit_minimum_not_applicable() {
        let mut input = baseline_input();
        input.unit_count_at_property = 1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::NotApplicableSingleUnitPropertyBelow2UnitMinimum
        );
    }

    #[test]
    fn two_unit_property_at_minimum_covered() {
        let mut input = baseline_input();
        input.unit_count_at_property = 2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::CompliantJustCauseEvictionUnderOneOfElevenSection822360Grounds
        );
    }

    #[test]
    fn certificate_of_occupancy_after_1983_not_applicable() {
        let mut input = baseline_input();
        input.certificate_of_occupancy_date_status =
            CertificateOfOccupancyDateStatus::IssuedOnOrAfterJanuary1_1983ExemptFromMeasureEe;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::NotApplicableCertificateOfOccupancyOnOrAfterJanuary1_1983
        );
    }

    #[test]
    fn newly_constructed_after_2003_exempt() {
        let mut input = baseline_input();
        input.exemption_status = ExemptionStatus::NewlyConstructedAfter2003Exempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::NotApplicableNewlyConstructedAfter2003Exempt
        );
    }

    #[test]
    fn owner_occupied_exempt() {
        let mut input = baseline_input();
        input.exemption_status = ExemptionStatus::OwnerOccupiedBuildingExemption;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::NotApplicableOwnerOccupiedBuildingExempt
        );
    }

    #[test]
    fn hospital_or_nonprofit_exempt() {
        let mut input = baseline_input();
        input.exemption_status = ExemptionStatus::HospitalOrNonprofitOperatedHousingExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::NotApplicableHospitalOrNonprofitExempt
        );
    }

    #[test]
    fn just_cause_non_payment_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::CompliantJustCauseEvictionUnderOneOfElevenSection822360Grounds
        );
    }

    #[test]
    fn just_cause_ellis_act_withdrawal_compliant() {
        let mut input = baseline_input();
        input.just_cause_ground_asserted =
            JustCauseGroundAsserted::EllisActWithdrawalUnderCaliforniaGovCode7060Omc822360I;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::CompliantJustCauseEvictionUnderOneOfElevenSection822360Grounds
        );
    }

    #[test]
    fn eviction_without_just_cause_violation() {
        let mut input = baseline_input();
        input.just_cause_ground_asserted = JustCauseGroundAsserted::NoJustCauseAsserted;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::ViolationEvictionWithoutOneOfElevenSection822360JustCauseGrounds
        );
    }

    #[test]
    fn owner_move_in_against_60_plus_5_year_protected_tenant_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInProtectedTenantCheck;
        input.protected_tenant_status =
            ProtectedTenantStatus::AgeAtOrAbove60WithAtLeast5YearTenureProtectedFromOwnerMoveIn;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::ViolationOwnerMoveInAgainstProtectedTenant60PlusWith5YearTenureOrDisabledOrCatastrophicIllness
        );
    }

    #[test]
    fn owner_move_in_against_disabled_tenant_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInProtectedTenantCheck;
        input.protected_tenant_status =
            ProtectedTenantStatus::DisabledTenantProtectedFromOwnerMoveIn;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::ViolationOwnerMoveInAgainstProtectedTenant60PlusWith5YearTenureOrDisabledOrCatastrophicIllness
        );
    }

    #[test]
    fn owner_move_in_against_catastrophic_illness_tenant_measure_v_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInProtectedTenantCheck;
        input.protected_tenant_status =
            ProtectedTenantStatus::CatastrophicIllnessProtectedFromOwnerMoveInUnderMeasureV2022;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::ViolationOwnerMoveInAgainstProtectedTenant60PlusWith5YearTenureOrDisabledOrCatastrophicIllness
        );
    }

    #[test]
    fn owner_move_in_against_non_protected_tenant_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInProtectedTenantCheck;
        input.protected_tenant_status = ProtectedTenantStatus::NotProtected;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::CompliantOwnerMoveInAgainstNonProtectedTenant
        );
    }

    #[test]
    fn ellis_act_standard_120_day_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EllisActWithdrawalNoticeRequirement;
        input.protected_tenant_status = ProtectedTenantStatus::NotProtected;
        input.ellis_act_notice_days_provided = 120;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::CompliantEllisActWithdrawalStandard120DayNotice
        );
        assert_eq!(output.ellis_act_notice_days_required, 120);
    }

    #[test]
    fn ellis_act_senior_disabled_365_day_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EllisActWithdrawalNoticeRequirement;
        input.protected_tenant_status =
            ProtectedTenantStatus::DisabledTenantProtectedFromOwnerMoveIn;
        input.ellis_act_notice_days_provided = 365;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::CompliantEllisActWithdrawalExtended365DayNoticeForSeniorOrDisabled
        );
        assert_eq!(output.ellis_act_notice_days_required, 365);
    }

    #[test]
    fn ellis_act_senior_disabled_under_365_day_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EllisActWithdrawalNoticeRequirement;
        input.protected_tenant_status =
            ProtectedTenantStatus::DisabledTenantProtectedFromOwnerMoveIn;
        input.ellis_act_notice_days_provided = 200;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::ViolationEllisActWithdrawalInsufficientNoticeForSeniorOrDisabledTenant
        );
        assert_eq!(output.ellis_act_notice_days_required, 365);
    }

    #[test]
    fn ellis_act_standard_under_120_day_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EllisActWithdrawalNoticeRequirement;
        input.protected_tenant_status = ProtectedTenantStatus::NotProtected;
        input.ellis_act_notice_days_provided = 90;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OaklandMeasureEeMode::ViolationEllisActWithdrawalInsufficientNoticeForSeniorOrDisabledTenant
        );
        assert_eq!(output.ellis_act_notice_days_required, 120);
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(OAKLAND_MEASURE_EE_ENACTMENT_YEAR, 2002);
        assert_eq!(OAKLAND_MEASURE_V_AMENDMENT_YEAR, 2022);
        assert_eq!(OAKLAND_CERTIFICATE_OF_OCCUPANCY_CUTOFF_YEAR, 1983);
        assert_eq!(OAKLAND_CERTIFICATE_OF_OCCUPANCY_CUTOFF_MONTH, 1);
        assert_eq!(OAKLAND_CERTIFICATE_OF_OCCUPANCY_CUTOFF_DAY, 1);
        assert_eq!(OAKLAND_NEW_CONSTRUCTION_EXEMPTION_CUTOFF_YEAR, 2003);
        assert_eq!(OAKLAND_NUMBER_OF_JUST_CAUSE_GROUNDS, 11);
        assert_eq!(OAKLAND_MIN_UNITS_FOR_COVERAGE, 2);
        assert_eq!(OAKLAND_PROTECTED_TENANT_AGE_THRESHOLD_YEARS, 60);
        assert_eq!(OAKLAND_PROTECTED_TENANT_TENURE_YEARS, 5);
        assert_eq!(OAKLAND_NON_PAYMENT_NOTICE_DAYS, 3);
        assert_eq!(OAKLAND_SUBSTANTIAL_REPAIRS_TEMPORARY_RELOCATION_MONTHS, 3);
        assert_eq!(OAKLAND_ELLIS_ACT_SENIOR_DISABLED_NOTICE_DAYS, 365);
        assert_eq!(OAKLAND_ELLIS_ACT_STANDARD_NOTICE_DAYS, 120);
        assert_eq!(OAKLAND_ELLIS_ACT_SENIOR_AGE_THRESHOLD, 62);
        assert_eq!(OAKLAND_ELLIS_ACT_SENIOR_TENURE_YEARS, 1);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Measure EE"));
        assert!(joined.contains("2002"));
        assert!(joined.contains("OMC § 8.22.360"));
        assert!(joined.contains("OMC § 8.22.350"));
        assert!(joined.contains("OMC § 8.22.370"));
        assert!(joined.contains("JANUARY 1, 1983"));
        assert!(joined.contains("Ellis Act"));
        assert!(joined.contains("60 OR OLDER"));
        assert!(joined.contains("5 YEARS"));
        assert!(joined.contains("DISABILITY"));
        assert!(joined.contains("CATASTROPHIC ILLNESS"));
        assert!(joined.contains("Measure V (2022)"));
        assert!(joined.contains("120 DAYS"));
        assert!(joined.contains("365-DAY"));
    }
}
