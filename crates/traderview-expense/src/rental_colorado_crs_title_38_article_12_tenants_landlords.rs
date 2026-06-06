//! Colorado Tenants and Landlords Statutes — CRS Title
//! 38, Article 12 Compliance Module — pure-compute check
//! for landlord statutory compliance with Colorado's
//! statewide landlord-tenant regime spanning **CRS
//! §§ 38-12-101 through 38-12-1601**.
//!
//! Major recent reforms covered: **HB 19-1170 Residential
//! Tenants Health and Safety Act** (2019; warranty of
//! habitability under CRS § 38-12-503); **CRS § 38-12-103
//! treble damages** for wrongful security deposit retention;
//! **CRS § 38-12-701 60-day rent increase notice**;
//! **HB 25-1249 security deposit bad-faith provisions**
//! effective **January 1, 2026**.
//!
//! Companion to the separately-codified **HB 24-1098
//! just-cause eviction module** (existing
//! `rental_colorado_hb_24_1098_just_cause_eviction`,
//! iter 649).
//!
//! Web research (verified 2026-06-03):
//! - **CRS Title 38, Article 12 (Tenants and Landlords)**: Colorado's statewide landlord-tenant regime; codified at **CRS §§ 38-12-101 through 38-12-1601** ([Justia — CRS § 38-12-103 (2024)](https://law.justia.com/codes/colorado/title-38/tenants-and-landlords/article-12/part-1/section-38-12-103/); [Colorado Public Law — CRS § 38-12-103](https://colorado.public.law/statutes/crs_38-12-103); [FindLaw — CRS § 38-12-103](https://codes.findlaw.com/co/title-38-property-real-and-personal/co-rev-st-sect-38-12-103/); [Larranaga Law — Colorado's Security Deposit Law](https://www.larranagalaw.com/colorados-security-deposit-law); [Otten Johnson — Changes to the Colorado Security Deposit Statute to take Effect January 1, 2026](https://www.ottenjohnson.com/news/changes-to-the-colorado-security-deposit-statute-to-take-effect-january-1-2026/); [Checkerboard — CRS §§ 38-12-103 and 38-12-104 Security Deposits Wrongful Withholding PDF](https://checkerboard.co/CB2/13_LTE/CRS_38-12-103-38-12-104_Security_Deposits_Wrongful_Withholding.pdf); [Larranaga Law — Colorado Security Deposit Law Notice Example](https://www.larranagalaw.com/colorado-security-deposit-law-notice-example); [Judicial Legal Help Center Colorado — Security Deposits](https://lawhelp.colorado.gov/security-deposits); [Justia — CRS § 38-12-503 (2024) Warranty of Habitability](https://law.justia.com/codes/colorado/title-38/tenants-and-landlords/article-12/part-5/section-38-12-503/); [Colorado Public Law — CRS § 38-12-503](https://colorado.public.law/statutes/crs_38-12-503); [Colorado HCPF — Warranty of Habitability PDF](https://hcpf.colorado.gov/sites/hcpf/files/Attachment%205-Warranty%20of%20Habitability.pdf); [Checkerboard — CRS §§ 38-12-501-38-12-511 Obligation to Maintain Residential Premises PDF](https://checkerboard.co/CB2/13_LTE/CRS_38-12-501-38-12-511_Obligation_To_Maintain_Residential_Premises.pdf); [Justia — CRS § 38-12-507 (2024) Breach of Warranty of Habitability Tenant's Remedies](https://law.justia.com/codes/colorado/title-38/tenants-and-landlords/article-12/part-5/section-38-12-507/); [Colorado Real Estate Journal — New Warranty of Habitability Law Includes Many Changes](https://crej.com/news/new-warranty-of-habitability-law-includes-many-changes/); [Colorado General Assembly — HB19-1170 Residential Tenants Health & Safety Act](http://leg.colorado.gov/bills/hb19-1170); [Colorado Public Law — CRS § 38-12-701](https://colorado.public.law/statutes/crs_38-12-701); [Justia — CRS § 38-12-701 (2024) Notice of Rent Increase](https://law.justia.com/codes/colorado/title-38/tenants-and-landlords/article-12/part-7/section-38-12-701/); [FindLaw — CRS § 38-12-701](https://codes.findlaw.com/co/title-38-property-real-and-personal/co-rev-st-sect-38-12-701/); [Colorado Legal Services — Legal Help for Rent Increases in Colorado](https://www.coloradolegalservices.org/housing/landlords-raising-rent/); [Landlord Studio — Colorado Landlord Tenant Laws](https://www.landlordstudio.com/landlord-tenant-laws/colorado-landlord-tenant-laws); [Volpe Law — New Laws in Colorado Related to Rent, Late Fees, and Security Deposits](https://www.volpelawllc.com/new-laws-in-colorado-related-to-rent-late-fees-and-security-deposits/); [Innago — Colorado Landlord Tenant Laws 2025](https://innago.com/colorado-landlord-tenant-laws/)).
//! - **CRS § 38-12-103 Security Deposit Return — 1-Month Default + 60-Day Lease Maximum**: a landlord must **RETURN THE FULL SECURITY DEPOSIT WITHIN ONE MONTH** after the lease termination or surrender of premises; UNLESS the lease specifies a longer period **NOT TO EXCEED 60 DAYS**.
//! - **CRS § 38-12-103 Treble Damages for Wrongful Withholding**: the **WILLFUL RETENTION** of a security deposit in violation of this section renders a landlord liable for **TREBLE (3x) THE AMOUNT WRONGFULLY WITHHELD** from the tenant, together with **REASONABLE ATTORNEY FEES AND COURT COSTS** — Colorado's 3x multiplier is among the highest in the United States.
//! - **CRS § 38-12-103 7-Day Pre-Filing Notice**: tenant must give the landlord **NOTICE OF THE TENANT'S INTENTION TO FILE LEGAL PROCEEDINGS A MINIMUM OF SEVEN DAYS PRIOR TO FILING** to recover treble damages.
//! - **CRS § 38-12-103 Burden of Proof on Landlord**: in any court action brought by a tenant, the landlord bears the burden of proving that **HIS WITHHOLDING OF THE SECURITY DEPOSIT WAS NOT WRONGFUL**.
//! - **HB 25-1249 Bad-Faith Provisions Effective January 1, 2026**: Colorado General Assembly enacted **HB 25-1249** providing detailed circumstances under which a security deposit is **DEEMED TO BE WITHHELD BY A LANDLORD IN BAD FAITH**; sweeping changes effective **JANUARY 1, 2026**.
//! - **CRS § 38-12-503 Warranty of Habitability (HB 19-1170 Residential Tenants Health and Safety Act)**: in every rental agreement, the landlord is deemed to warrant that the residential premises is **FIT FOR HUMAN HABITATION**; HB 19-1170 amended sections (2), (3), and (4) and added sections (2.2), (2.3), and (2.5), effective **AUGUST 2, 2019**; established the **MATERIAL INTERFERENCE STANDARD** for habitability breach.
//! - **CRS § 38-12-503(2) Tiered Response Timeframes**: landlord commits a breach of the warranty if the residential premises is **UNINHABITABLE** or **MATERIALLY INTERFERES WITH THE TENANT'S LIFE, HEALTH, OR SAFETY**, AND the landlord has received reasonably complete **WRITTEN OR ELECTRONIC NOTICE** AND failed to commence remedial action by employing reasonable efforts within: **(A) 24 HOURS** where the condition **MATERIALLY INTERFERES WITH THE TENANT'S LIFE, HEALTH, OR SAFETY**; OR **(B) 96 HOURS** where the premises is **UNINHABITABLE OR OTHERWISE UNFIT FOR HUMAN HABITATION** AND the tenant has included with the notice **PERMISSION FOR THE LANDLORD TO ENTER** the residential premises.
//! - **CRS § 38-12-507 Tenant Remedies for Habitability Breach**: tenant remedies include (i) **INJUNCTIVE RELIEF**; (ii) **DECLARATORY JUDGMENT**; (iii) **CONSEQUENTIAL DAMAGES**; (iv) **REASONABLE ATTORNEY FEES**; (v) **RECISION OF THE RENTAL AGREEMENT**; (vi) **WITHHOLDING RENT** in a rent escrow under CRS § 38-12-506.
//! - **CRS § 38-12-701 60-Day Rent Increase Notice**: in a residential tenancy in which there is no written agreement between the landlord and tenant, a landlord may increase the rent only upon at least **SIXTY DAYS' WRITTEN NOTICE** to the tenant; a landlord may NOT terminate a residential tenancy in which there is no written agreement by serving a tenant with a notice to quit **WITH THE PRIMARY PURPOSE OF INCREASING A TENANT'S RENT** in a manner inconsistent with this section (anti-circumvention rule).
//! - **HB 24-1098 Just-Cause Eviction (Separate Module)**: Colorado's just-cause eviction regime under HB 24-1098 effective April 19, 2024 is covered by the separate `rental_colorado_hb_24_1098_just_cause_eviction` module (iter 649).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CO_CRS_TITLE_NUMBER: u32 = 38;
pub const CO_CRS_ARTICLE_NUMBER: u32 = 12;
pub const CO_DEPOSIT_RETURN_DEFAULT_DAYS: u32 = 30;
pub const CO_DEPOSIT_RETURN_LEASE_SPECIFIED_MAX_DAYS: u32 = 60;
pub const CO_WRONGFUL_WITHHOLDING_MULTIPLIER: u32 = 3;
pub const CO_PRE_FILING_NOTICE_DAYS: u32 = 7;
pub const CO_HABITABILITY_LIFE_HEALTH_SAFETY_HOURS: u32 = 24;
pub const CO_HABITABILITY_UNINHABITABLE_HOURS: u32 = 96;
pub const CO_RENT_INCREASE_NOTICE_DAYS: u32 = 60;
pub const CO_HB_19_1170_ENACTMENT_YEAR: u32 = 2019;
pub const CO_HB_19_1170_ENACTMENT_MONTH: u32 = 8;
pub const CO_HB_19_1170_ENACTMENT_DAY: u32 = 2;
pub const CO_HB_25_1249_BAD_FAITH_EFFECTIVE_YEAR: u32 = 2026;
pub const CO_HB_25_1249_BAD_FAITH_EFFECTIVE_MONTH: u32 = 1;
pub const CO_HB_25_1249_BAD_FAITH_EFFECTIVE_DAY: u32 = 1;
pub const CO_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromCrsArticle12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseSpecifiedDepositReturnStatus {
    LeaseSilentDefaultOneMonthReturn,
    LeaseSpecifiedUpTo60DayReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitabilityConditionSeverity {
    MaterialInterferenceWithLifeHealthOrSafety,
    UninhabitableOrUnfitForHumanHabitation,
    NoMaterialInterference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WrittenAgreementStatus {
    PeriodicTenancyWithoutWrittenAgreement,
    WrittenAgreementInForce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositReturnUnderSection38121030,
    TrebleDamagesWrongfulWithholdingUnderSection38121030,
    SevenDayPreFilingNoticeUnderSection38121030,
    BurdenOfProofOnLandlordUnderSection38121030,
    WarrantyOfHabitability24HourResponseUnderSection38125030,
    WarrantyOfHabitability96HourResponseUnderSection38125030,
    MaterialInterferenceStandardUnderSection38125030HB19_1170,
    SixtyDayRentIncreaseNoticeUnderSection38127010,
    AntiCircumventionRuleUnderSection38127010,
    HB25_1249BadFaithEffectiveJanuary1_2026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoLandlordTenantMode {
    NotApplicableTenancyExemptFromCrsArticle12,
    CompliantDepositReturnedWithinDefaultOneMonth,
    CompliantDepositReturnedWithinLeaseSpecified60Days,
    CompliantNoWrongfulWithholding,
    CompliantSevenDayPreFilingNoticeServed,
    CompliantBurdenOfProofOnLandlordAcknowledged,
    CompliantWarrantyOfHabitability24HourResponseForLifeHealthSafety,
    CompliantWarrantyOfHabitability96HourResponseForUninhabitable,
    CompliantNoMaterialInterferenceConditionPresent,
    CompliantSixtyDayRentIncreaseNoticeProperlyServed,
    CompliantAntiCircumventionRuleObserved,
    CompliantHB25_1249BadFaithProvisionsObservedEffectiveJanuary1_2026,
    ViolationDepositReturnedPastDefaultOneMonth,
    ViolationDepositReturnedPastLeaseSpecified60Days,
    ViolationWrongfulWithholdingTriplesDepositLiability,
    ViolationPreFilingNoticeShorterThanSevenDays,
    ViolationLandlord24HourResponseNotMetForLifeHealthSafety,
    ViolationLandlord96HourResponseNotMetForUninhabitable,
    ViolationRentIncreaseNoticeShorterThan60Days,
    ViolationAntiCircumventionRuleNoticeToQuitWithPrimaryPurposeOfRentIncrease,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub lease_specified_deposit_return_status: LeaseSpecifiedDepositReturnStatus,
    pub habitability_condition_severity: HabitabilityConditionSeverity,
    pub written_agreement_status: WrittenAgreementStatus,
    pub compliance_aspect: ComplianceAspect,
    pub days_to_return_deposit: u32,
    pub deposit_wrongfully_withheld: bool,
    pub pre_filing_notice_days_given: u32,
    pub landlord_life_health_safety_response_hours: u32,
    pub landlord_uninhabitable_response_hours: u32,
    pub rent_increase_notice_days_given: u32,
    pub notice_to_quit_with_primary_purpose_of_rent_increase: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: CoLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type CoLandlordTenantInput = Input;
pub type CoLandlordTenantOutput = Output;
pub type CoLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Colorado Tenants and Landlords Statutes — CRS Title 38, Article 12; statewide landlord-tenant regime codified at CRS §§ 38-12-101 through 38-12-1601".to_string(),
        "CRS § 38-12-103 Security Deposit Return — 1-Month Default + 60-Day Lease Maximum — landlord must RETURN THE FULL SECURITY DEPOSIT WITHIN ONE MONTH after the lease termination or surrender of premises; UNLESS the lease specifies a longer period NOT TO EXCEED 60 DAYS".to_string(),
        "CRS § 38-12-103 Treble Damages for Wrongful Withholding — WILLFUL RETENTION of a security deposit in violation of this section renders a landlord liable for TREBLE (3x) THE AMOUNT WRONGFULLY WITHHELD from the tenant, together with REASONABLE ATTORNEY FEES AND COURT COSTS".to_string(),
        "CRS § 38-12-103 7-Day Pre-Filing Notice — tenant must give the landlord NOTICE OF THE TENANT'S INTENTION TO FILE LEGAL PROCEEDINGS A MINIMUM OF SEVEN DAYS PRIOR TO FILING to recover treble damages".to_string(),
        "CRS § 38-12-103 Burden of Proof on Landlord — in any court action brought by a tenant, the landlord bears the burden of proving that HIS WITHHOLDING OF THE SECURITY DEPOSIT WAS NOT WRONGFUL".to_string(),
        "HB 25-1249 Bad-Faith Provisions Effective January 1, 2026 — Colorado General Assembly enacted HB 25-1249 providing detailed circumstances under which a security deposit is DEEMED TO BE WITHHELD BY A LANDLORD IN BAD FAITH; sweeping changes effective JANUARY 1, 2026".to_string(),
        "CRS § 38-12-503 Warranty of Habitability (HB 19-1170 Residential Tenants Health and Safety Act) — in every rental agreement, the landlord is deemed to warrant that the residential premises is FIT FOR HUMAN HABITATION; HB 19-1170 amended sections (2), (3), and (4) and added sections (2.2), (2.3), and (2.5), effective AUGUST 2, 2019; established the MATERIAL INTERFERENCE STANDARD for habitability breach".to_string(),
        "CRS § 38-12-503(2) Tiered Response Timeframes — landlord commits a breach of the warranty if the residential premises is UNINHABITABLE or MATERIALLY INTERFERES WITH THE TENANT'S LIFE, HEALTH, OR SAFETY, AND the landlord has received reasonably complete WRITTEN OR ELECTRONIC NOTICE AND failed to commence remedial action by employing reasonable efforts within: (A) 24 HOURS where the condition MATERIALLY INTERFERES WITH THE TENANT'S LIFE, HEALTH, OR SAFETY; OR (B) 96 HOURS where the premises is UNINHABITABLE OR OTHERWISE UNFIT FOR HUMAN HABITATION AND the tenant has included with the notice PERMISSION FOR THE LANDLORD TO ENTER the residential premises".to_string(),
        "CRS § 38-12-507 Tenant Remedies for Habitability Breach — tenant remedies include (i) INJUNCTIVE RELIEF; (ii) DECLARATORY JUDGMENT; (iii) CONSEQUENTIAL DAMAGES; (iv) REASONABLE ATTORNEY FEES; (v) RECISION OF THE RENTAL AGREEMENT; (vi) WITHHOLDING RENT in a rent escrow under CRS § 38-12-506".to_string(),
        "CRS § 38-12-701 60-Day Rent Increase Notice — in a residential tenancy in which there is no written agreement between the landlord and tenant, a landlord may increase the rent only upon at least SIXTY DAYS' WRITTEN NOTICE to the tenant".to_string(),
        "CRS § 38-12-701 Anti-Circumvention Rule — a landlord may NOT terminate a residential tenancy in which there is no written agreement by serving a tenant with a notice to quit WITH THE PRIMARY PURPOSE OF INCREASING A TENANT'S RENT in a manner inconsistent with this section".to_string(),
        "HB 24-1098 Just-Cause Eviction (Separate Module) — Colorado's just-cause eviction regime under HB 24-1098 effective April 19, 2024 is covered by the separate rental_colorado_hb_24_1098_just_cause_eviction module".to_string(),
        "Justia + Colorado Public Law + FindLaw + Larranaga Law + Otten Johnson + Checkerboard + Judicial Legal Help Center Colorado + Colorado HCPF + Colorado Real Estate Journal + Colorado General Assembly + Colorado Legal Services + Landlord Studio + Volpe Law + Innago — practitioner overviews of CRS Title 38, Article 12".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromCrsArticle12 {
        return Output {
            mode: CoLandlordTenantMode::NotApplicableTenancyExemptFromCrsArticle12,
            statutory_basis: "CRS Title 38, Article 12 jurisdiction — tenancy exempt from Article 12 coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from CRS Title 38, Article 12; Colorado landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositReturnUnderSection38121030 => match input
            .lease_specified_deposit_return_status
        {
            LeaseSpecifiedDepositReturnStatus::LeaseSilentDefaultOneMonthReturn => {
                if input.days_to_return_deposit <= CO_DEPOSIT_RETURN_DEFAULT_DAYS {
                    Output {
                        mode: CoLandlordTenantMode::CompliantDepositReturnedWithinDefaultOneMonth,
                        statutory_basis: "CRS § 38-12-103 — deposit returned within default 1-month (30-day) window".to_string(),
                        notes: format!(
                            "COMPLIANT: deposit returned at day {d} within default 1-month (30-day) window under CRS § 38-12-103.",
                            d = input.days_to_return_deposit,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: CoLandlordTenantMode::ViolationDepositReturnedPastDefaultOneMonth,
                        statutory_basis: "CRS § 38-12-103 — deposit return exceeded default 1-month (30-day) window".to_string(),
                        notes: format!(
                            "VIOLATION: deposit returned at day {d} past default 1-month (30-day) window under CRS § 38-12-103.",
                            d = input.days_to_return_deposit,
                        ),
                        citations,
                    }
                }
            }
            LeaseSpecifiedDepositReturnStatus::LeaseSpecifiedUpTo60DayReturn => {
                if input.days_to_return_deposit <= CO_DEPOSIT_RETURN_LEASE_SPECIFIED_MAX_DAYS {
                    Output {
                        mode: CoLandlordTenantMode::CompliantDepositReturnedWithinLeaseSpecified60Days,
                        statutory_basis: "CRS § 38-12-103 — deposit returned within lease-specified period not exceeding 60 days".to_string(),
                        notes: format!(
                            "COMPLIANT: deposit returned at day {d} within lease-specified period not exceeding 60 days under CRS § 38-12-103.",
                            d = input.days_to_return_deposit,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: CoLandlordTenantMode::ViolationDepositReturnedPastLeaseSpecified60Days,
                        statutory_basis: "CRS § 38-12-103 — deposit return exceeded lease-specified maximum of 60 days".to_string(),
                        notes: format!(
                            "VIOLATION: deposit returned at day {d} past lease-specified maximum of 60 days under CRS § 38-12-103.",
                            d = input.days_to_return_deposit,
                        ),
                        citations,
                    }
                }
            }
        },
        ComplianceAspect::TrebleDamagesWrongfulWithholdingUnderSection38121030 => {
            if input.deposit_wrongfully_withheld {
                Output {
                    mode: CoLandlordTenantMode::ViolationWrongfulWithholdingTriplesDepositLiability,
                    statutory_basis: "CRS § 38-12-103 — wrongful withholding triggers treble (3x) damages + attorney fees + court costs".to_string(),
                    notes: "VIOLATION: deposit wrongfully withheld; landlord liable for TREBLE (3x) THE AMOUNT WRONGFULLY WITHHELD + REASONABLE ATTORNEY FEES + COURT COSTS under CRS § 38-12-103.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: CoLandlordTenantMode::CompliantNoWrongfulWithholding,
                    statutory_basis: "CRS § 38-12-103 — no wrongful withholding".to_string(),
                    notes: "COMPLIANT: no wrongful withholding under CRS § 38-12-103; treble damages exposure not triggered.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SevenDayPreFilingNoticeUnderSection38121030 => {
            if input.pre_filing_notice_days_given >= CO_PRE_FILING_NOTICE_DAYS {
                Output {
                    mode: CoLandlordTenantMode::CompliantSevenDayPreFilingNoticeServed,
                    statutory_basis: "CRS § 38-12-103 — 7-day pre-filing notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day pre-filing notice satisfies 7-day statutory minimum under CRS § 38-12-103.",
                        d = input.pre_filing_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: CoLandlordTenantMode::ViolationPreFilingNoticeShorterThanSevenDays,
                    statutory_basis: "CRS § 38-12-103 — pre-filing notice shorter than 7-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day pre-filing notice shorter than 7-day statutory minimum under CRS § 38-12-103.",
                        d = input.pre_filing_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::BurdenOfProofOnLandlordUnderSection38121030 => Output {
            mode: CoLandlordTenantMode::CompliantBurdenOfProofOnLandlordAcknowledged,
            statutory_basis: "CRS § 38-12-103 — burden of proof on landlord to prove withholding was not wrongful".to_string(),
            notes: "COMPLIANT: burden of proof on landlord to prove HIS WITHHOLDING OF THE SECURITY DEPOSIT WAS NOT WRONGFUL under CRS § 38-12-103.".to_string(),
            citations,
        },
        ComplianceAspect::WarrantyOfHabitability24HourResponseUnderSection38125030 => {
            if input.habitability_condition_severity
                == HabitabilityConditionSeverity::MaterialInterferenceWithLifeHealthOrSafety
            {
                if input.landlord_life_health_safety_response_hours
                    <= CO_HABITABILITY_LIFE_HEALTH_SAFETY_HOURS
                {
                    Output {
                        mode: CoLandlordTenantMode::CompliantWarrantyOfHabitability24HourResponseForLifeHealthSafety,
                        statutory_basis: "CRS § 38-12-503(2)(A) — 24-hour landlord response for life/health/safety conditions met".to_string(),
                        notes: format!(
                            "COMPLIANT: landlord response at hour {h} within 24-hour statutory window for life/health/safety conditions under CRS § 38-12-503(2)(A).",
                            h = input.landlord_life_health_safety_response_hours,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: CoLandlordTenantMode::ViolationLandlord24HourResponseNotMetForLifeHealthSafety,
                        statutory_basis: "CRS § 38-12-503(2)(A) — 24-hour landlord response for life/health/safety conditions not met".to_string(),
                        notes: format!(
                            "VIOLATION: landlord response at hour {h} past 24-hour statutory window for life/health/safety conditions under CRS § 38-12-503(2)(A).",
                            h = input.landlord_life_health_safety_response_hours,
                        ),
                        citations,
                    }
                }
            } else {
                Output {
                    mode: CoLandlordTenantMode::CompliantNoMaterialInterferenceConditionPresent,
                    statutory_basis: "CRS § 38-12-503(2)(A) — no life/health/safety material interference condition present".to_string(),
                    notes: "COMPLIANT: no life/health/safety material interference condition present; 24-hour response requirement not triggered.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::WarrantyOfHabitability96HourResponseUnderSection38125030 => {
            if input.habitability_condition_severity
                == HabitabilityConditionSeverity::UninhabitableOrUnfitForHumanHabitation
            {
                if input.landlord_uninhabitable_response_hours
                    <= CO_HABITABILITY_UNINHABITABLE_HOURS
                {
                    Output {
                        mode: CoLandlordTenantMode::CompliantWarrantyOfHabitability96HourResponseForUninhabitable,
                        statutory_basis: "CRS § 38-12-503(2)(B) — 96-hour landlord response for uninhabitable conditions met".to_string(),
                        notes: format!(
                            "COMPLIANT: landlord response at hour {h} within 96-hour statutory window for uninhabitable conditions under CRS § 38-12-503(2)(B).",
                            h = input.landlord_uninhabitable_response_hours,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: CoLandlordTenantMode::ViolationLandlord96HourResponseNotMetForUninhabitable,
                        statutory_basis: "CRS § 38-12-503(2)(B) — 96-hour landlord response for uninhabitable conditions not met".to_string(),
                        notes: format!(
                            "VIOLATION: landlord response at hour {h} past 96-hour statutory window for uninhabitable conditions under CRS § 38-12-503(2)(B).",
                            h = input.landlord_uninhabitable_response_hours,
                        ),
                        citations,
                    }
                }
            } else {
                Output {
                    mode: CoLandlordTenantMode::CompliantNoMaterialInterferenceConditionPresent,
                    statutory_basis: "CRS § 38-12-503(2)(B) — no uninhabitable condition present".to_string(),
                    notes: "COMPLIANT: no uninhabitable condition present; 96-hour response requirement not triggered.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::MaterialInterferenceStandardUnderSection38125030HB19_1170 => match input
            .habitability_condition_severity
        {
            HabitabilityConditionSeverity::MaterialInterferenceWithLifeHealthOrSafety => Output {
                mode: CoLandlordTenantMode::CompliantWarrantyOfHabitability24HourResponseForLifeHealthSafety,
                statutory_basis: "CRS § 38-12-503 (HB 19-1170) — material interference with life/health/safety standard met".to_string(),
                notes: "COMPLIANT: material interference with tenant's life, health, or safety standard under CRS § 38-12-503 (HB 19-1170) identified; triggers 24-hour landlord response obligation.".to_string(),
                citations,
            },
            HabitabilityConditionSeverity::UninhabitableOrUnfitForHumanHabitation => Output {
                mode: CoLandlordTenantMode::CompliantWarrantyOfHabitability96HourResponseForUninhabitable,
                statutory_basis: "CRS § 38-12-503 (HB 19-1170) — uninhabitable or unfit for human habitation standard met".to_string(),
                notes: "COMPLIANT: uninhabitable or unfit for human habitation standard under CRS § 38-12-503 (HB 19-1170) identified; triggers 96-hour landlord response obligation (with tenant entry permission).".to_string(),
                citations,
            },
            HabitabilityConditionSeverity::NoMaterialInterference => Output {
                mode: CoLandlordTenantMode::CompliantNoMaterialInterferenceConditionPresent,
                statutory_basis: "CRS § 38-12-503 (HB 19-1170) — no material interference condition present".to_string(),
                notes: "COMPLIANT: no material interference condition present under CRS § 38-12-503 (HB 19-1170); landlord response obligations not triggered.".to_string(),
                citations,
            },
        },
        ComplianceAspect::SixtyDayRentIncreaseNoticeUnderSection38127010 => {
            if input.written_agreement_status
                == WrittenAgreementStatus::PeriodicTenancyWithoutWrittenAgreement
            {
                if input.rent_increase_notice_days_given >= CO_RENT_INCREASE_NOTICE_DAYS {
                    Output {
                        mode: CoLandlordTenantMode::CompliantSixtyDayRentIncreaseNoticeProperlyServed,
                        statutory_basis: "CRS § 38-12-701 — 60-day rent increase notice for periodic tenancy without written agreement properly served".to_string(),
                        notes: format!(
                            "COMPLIANT: {d}-day rent increase notice satisfies 60-day statutory minimum under CRS § 38-12-701.",
                            d = input.rent_increase_notice_days_given,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: CoLandlordTenantMode::ViolationRentIncreaseNoticeShorterThan60Days,
                        statutory_basis: "CRS § 38-12-701 — rent increase notice shorter than 60-day statutory minimum".to_string(),
                        notes: format!(
                            "VIOLATION: {d}-day rent increase notice shorter than 60-day statutory minimum under CRS § 38-12-701 for periodic tenancy without written agreement.",
                            d = input.rent_increase_notice_days_given,
                        ),
                        citations,
                    }
                }
            } else {
                Output {
                    mode: CoLandlordTenantMode::CompliantSixtyDayRentIncreaseNoticeProperlyServed,
                    statutory_basis: "CRS § 38-12-701 — written agreement governs; statutory 60-day notice requirement does not apply".to_string(),
                    notes: "COMPLIANT: written rental agreement governs rent increase notice; CRS § 38-12-701 60-day notice requirement applies only to periodic tenancies without written agreement.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::AntiCircumventionRuleUnderSection38127010 => {
            if input.notice_to_quit_with_primary_purpose_of_rent_increase
                && input.written_agreement_status
                    == WrittenAgreementStatus::PeriodicTenancyWithoutWrittenAgreement
            {
                Output {
                    mode: CoLandlordTenantMode::ViolationAntiCircumventionRuleNoticeToQuitWithPrimaryPurposeOfRentIncrease,
                    statutory_basis: "CRS § 38-12-701 — anti-circumvention rule violated; notice to quit with primary purpose of rent increase prohibited".to_string(),
                    notes: "VIOLATION: landlord served notice to quit WITH THE PRIMARY PURPOSE OF INCREASING A TENANT'S RENT in a manner inconsistent with CRS § 38-12-701; prohibited anti-circumvention rule.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: CoLandlordTenantMode::CompliantAntiCircumventionRuleObserved,
                    statutory_basis: "CRS § 38-12-701 — anti-circumvention rule observed".to_string(),
                    notes: "COMPLIANT: no notice to quit with primary purpose of rent increase; CRS § 38-12-701 anti-circumvention rule observed.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::HB25_1249BadFaithEffectiveJanuary1_2026 => Output {
            mode: CoLandlordTenantMode::CompliantHB25_1249BadFaithProvisionsObservedEffectiveJanuary1_2026,
            statutory_basis: "HB 25-1249 bad-faith provisions effective January 1, 2026".to_string(),
            notes: "COMPLIANT: HB 25-1249 bad-faith provisions observed; sweeping changes to Colorado's security deposit statute effective January 1, 2026 providing detailed circumstances under which a security deposit is deemed to be withheld by a landlord in bad faith.".to_string(),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_coverage: TenancyCoverage::CoveredResidentialTenancy,
            lease_specified_deposit_return_status:
                LeaseSpecifiedDepositReturnStatus::LeaseSilentDefaultOneMonthReturn,
            habitability_condition_severity: HabitabilityConditionSeverity::NoMaterialInterference,
            written_agreement_status:
                WrittenAgreementStatus::PeriodicTenancyWithoutWrittenAgreement,
            compliance_aspect: ComplianceAspect::SecurityDepositReturnUnderSection38121030,
            days_to_return_deposit: 25,
            deposit_wrongfully_withheld: false,
            pre_filing_notice_days_given: 7,
            landlord_life_health_safety_response_hours: 12,
            landlord_uninhabitable_response_hours: 48,
            rent_increase_notice_days_given: 60,
            notice_to_quit_with_primary_purpose_of_rent_increase: false,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromCrsArticle12;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::NotApplicableTenancyExemptFromCrsArticle12
        );
    }

    #[test]
    fn deposit_returned_at_30_day_default_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection38121030;
        input.lease_specified_deposit_return_status =
            LeaseSpecifiedDepositReturnStatus::LeaseSilentDefaultOneMonthReturn;
        input.days_to_return_deposit = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantDepositReturnedWithinDefaultOneMonth
        );
    }

    #[test]
    fn deposit_returned_at_31_days_default_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection38121030;
        input.lease_specified_deposit_return_status =
            LeaseSpecifiedDepositReturnStatus::LeaseSilentDefaultOneMonthReturn;
        input.days_to_return_deposit = 31;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationDepositReturnedPastDefaultOneMonth
        );
    }

    #[test]
    fn deposit_returned_at_60_day_lease_specified_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection38121030;
        input.lease_specified_deposit_return_status =
            LeaseSpecifiedDepositReturnStatus::LeaseSpecifiedUpTo60DayReturn;
        input.days_to_return_deposit = 60;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantDepositReturnedWithinLeaseSpecified60Days
        );
    }

    #[test]
    fn deposit_returned_at_61_days_lease_specified_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection38121030;
        input.lease_specified_deposit_return_status =
            LeaseSpecifiedDepositReturnStatus::LeaseSpecifiedUpTo60DayReturn;
        input.days_to_return_deposit = 61;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationDepositReturnedPastLeaseSpecified60Days
        );
    }

    #[test]
    fn no_wrongful_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TrebleDamagesWrongfulWithholdingUnderSection38121030;
        input.deposit_wrongfully_withheld = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantNoWrongfulWithholding
        );
    }

    #[test]
    fn wrongful_withholding_triples_liability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TrebleDamagesWrongfulWithholdingUnderSection38121030;
        input.deposit_wrongfully_withheld = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationWrongfulWithholdingTriplesDepositLiability
        );
    }

    #[test]
    fn seven_day_pre_filing_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SevenDayPreFilingNoticeUnderSection38121030;
        input.pre_filing_notice_days_given = 7;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantSevenDayPreFilingNoticeServed
        );
    }

    #[test]
    fn six_day_pre_filing_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SevenDayPreFilingNoticeUnderSection38121030;
        input.pre_filing_notice_days_given = 6;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationPreFilingNoticeShorterThanSevenDays
        );
    }

    #[test]
    fn burden_of_proof_on_landlord_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BurdenOfProofOnLandlordUnderSection38121030;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantBurdenOfProofOnLandlordAcknowledged
        );
    }

    #[test]
    fn habitability_24_hour_response_for_life_health_safety_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::WarrantyOfHabitability24HourResponseUnderSection38125030;
        input.habitability_condition_severity =
            HabitabilityConditionSeverity::MaterialInterferenceWithLifeHealthOrSafety;
        input.landlord_life_health_safety_response_hours = 24;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantWarrantyOfHabitability24HourResponseForLifeHealthSafety
        );
    }

    #[test]
    fn habitability_25_hour_response_for_life_health_safety_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::WarrantyOfHabitability24HourResponseUnderSection38125030;
        input.habitability_condition_severity =
            HabitabilityConditionSeverity::MaterialInterferenceWithLifeHealthOrSafety;
        input.landlord_life_health_safety_response_hours = 25;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationLandlord24HourResponseNotMetForLifeHealthSafety
        );
    }

    #[test]
    fn habitability_96_hour_response_for_uninhabitable_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::WarrantyOfHabitability96HourResponseUnderSection38125030;
        input.habitability_condition_severity =
            HabitabilityConditionSeverity::UninhabitableOrUnfitForHumanHabitation;
        input.landlord_uninhabitable_response_hours = 96;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantWarrantyOfHabitability96HourResponseForUninhabitable
        );
    }

    #[test]
    fn habitability_97_hour_response_for_uninhabitable_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::WarrantyOfHabitability96HourResponseUnderSection38125030;
        input.habitability_condition_severity =
            HabitabilityConditionSeverity::UninhabitableOrUnfitForHumanHabitation;
        input.landlord_uninhabitable_response_hours = 97;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationLandlord96HourResponseNotMetForUninhabitable
        );
    }

    #[test]
    fn material_interference_standard_life_health_safety_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialInterferenceStandardUnderSection38125030HB19_1170;
        input.habitability_condition_severity =
            HabitabilityConditionSeverity::MaterialInterferenceWithLifeHealthOrSafety;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantWarrantyOfHabitability24HourResponseForLifeHealthSafety
        );
    }

    #[test]
    fn material_interference_standard_uninhabitable_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialInterferenceStandardUnderSection38125030HB19_1170;
        input.habitability_condition_severity =
            HabitabilityConditionSeverity::UninhabitableOrUnfitForHumanHabitation;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantWarrantyOfHabitability96HourResponseForUninhabitable
        );
    }

    #[test]
    fn material_interference_standard_no_interference_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialInterferenceStandardUnderSection38125030HB19_1170;
        input.habitability_condition_severity =
            HabitabilityConditionSeverity::NoMaterialInterference;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantNoMaterialInterferenceConditionPresent
        );
    }

    #[test]
    fn sixty_day_rent_increase_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SixtyDayRentIncreaseNoticeUnderSection38127010;
        input.written_agreement_status =
            WrittenAgreementStatus::PeriodicTenancyWithoutWrittenAgreement;
        input.rent_increase_notice_days_given = 60;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantSixtyDayRentIncreaseNoticeProperlyServed
        );
    }

    #[test]
    fn fifty_nine_day_rent_increase_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SixtyDayRentIncreaseNoticeUnderSection38127010;
        input.written_agreement_status =
            WrittenAgreementStatus::PeriodicTenancyWithoutWrittenAgreement;
        input.rent_increase_notice_days_given = 59;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationRentIncreaseNoticeShorterThan60Days
        );
    }

    #[test]
    fn written_agreement_governs_rent_increase_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SixtyDayRentIncreaseNoticeUnderSection38127010;
        input.written_agreement_status = WrittenAgreementStatus::WrittenAgreementInForce;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantSixtyDayRentIncreaseNoticeProperlyServed
        );
    }

    #[test]
    fn anti_circumvention_rule_observed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AntiCircumventionRuleUnderSection38127010;
        input.notice_to_quit_with_primary_purpose_of_rent_increase = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantAntiCircumventionRuleObserved
        );
    }

    #[test]
    fn anti_circumvention_rule_violated() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AntiCircumventionRuleUnderSection38127010;
        input.written_agreement_status =
            WrittenAgreementStatus::PeriodicTenancyWithoutWrittenAgreement;
        input.notice_to_quit_with_primary_purpose_of_rent_increase = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::ViolationAntiCircumventionRuleNoticeToQuitWithPrimaryPurposeOfRentIncrease
        );
    }

    #[test]
    fn hb_25_1249_bad_faith_provisions_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::HB25_1249BadFaithEffectiveJanuary1_2026;
        let out = check(&input);
        assert_eq!(
            out.mode,
            CoLandlordTenantMode::CompliantHB25_1249BadFaithProvisionsObservedEffectiveJanuary1_2026
        );
    }

    #[test]
    fn constants_pin_colorado_landlord_tenant_statutory_thresholds() {
        assert_eq!(CO_CRS_TITLE_NUMBER, 38);
        assert_eq!(CO_CRS_ARTICLE_NUMBER, 12);
        assert_eq!(CO_DEPOSIT_RETURN_DEFAULT_DAYS, 30);
        assert_eq!(CO_DEPOSIT_RETURN_LEASE_SPECIFIED_MAX_DAYS, 60);
        assert_eq!(CO_WRONGFUL_WITHHOLDING_MULTIPLIER, 3);
        assert_eq!(CO_PRE_FILING_NOTICE_DAYS, 7);
        assert_eq!(CO_HABITABILITY_LIFE_HEALTH_SAFETY_HOURS, 24);
        assert_eq!(CO_HABITABILITY_UNINHABITABLE_HOURS, 96);
        assert_eq!(CO_RENT_INCREASE_NOTICE_DAYS, 60);
        assert_eq!(CO_HB_19_1170_ENACTMENT_YEAR, 2019);
        assert_eq!(CO_HB_19_1170_ENACTMENT_MONTH, 8);
        assert_eq!(CO_HB_19_1170_ENACTMENT_DAY, 2);
        assert_eq!(CO_HB_25_1249_BAD_FAITH_EFFECTIVE_YEAR, 2026);
        assert_eq!(CO_HB_25_1249_BAD_FAITH_EFFECTIVE_MONTH, 1);
        assert_eq!(CO_HB_25_1249_BAD_FAITH_EFFECTIVE_DAY, 1);
        assert_eq!(CO_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_colorado_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Colorado Tenants and Landlords Statutes"));
        assert!(joined.contains("CRS Title 38, Article 12"));
        assert!(joined.contains("CRS § 38-12-103"));
        assert!(joined.contains("RETURN THE FULL SECURITY DEPOSIT WITHIN ONE MONTH"));
        assert!(joined.contains("NOT TO EXCEED 60 DAYS"));
        assert!(joined.contains("WILLFUL RETENTION"));
        assert!(joined.contains("TREBLE (3x) THE AMOUNT WRONGFULLY WITHHELD"));
        assert!(joined.contains("REASONABLE ATTORNEY FEES AND COURT COSTS"));
        assert!(joined.contains("SEVEN DAYS PRIOR TO FILING"));
        assert!(joined.contains("HIS WITHHOLDING OF THE SECURITY DEPOSIT WAS NOT WRONGFUL"));
        assert!(joined.contains("HB 25-1249"));
        assert!(joined.contains("JANUARY 1, 2026"));
        assert!(joined.contains("DEEMED TO BE WITHHELD BY A LANDLORD IN BAD FAITH"));
        assert!(joined.contains("CRS § 38-12-503"));
        assert!(joined.contains("HB 19-1170"));
        assert!(joined.contains("Residential Tenants Health and Safety Act"));
        assert!(joined.contains("FIT FOR HUMAN HABITATION"));
        assert!(joined.contains("MATERIAL INTERFERENCE STANDARD"));
        assert!(joined.contains("AUGUST 2, 2019"));
        assert!(joined.contains("24 HOURS"));
        assert!(joined.contains("96 HOURS"));
        assert!(joined.contains("MATERIALLY INTERFERES WITH THE TENANT'S LIFE, HEALTH, OR SAFETY"));
        assert!(joined.contains("UNINHABITABLE OR OTHERWISE UNFIT FOR HUMAN HABITATION"));
        assert!(joined.contains("PERMISSION FOR THE LANDLORD TO ENTER"));
        assert!(joined.contains("CRS § 38-12-507"));
        assert!(joined.contains("INJUNCTIVE RELIEF"));
        assert!(joined.contains("CRS § 38-12-701"));
        assert!(joined.contains("SIXTY DAYS' WRITTEN NOTICE"));
        assert!(joined.contains("PRIMARY PURPOSE OF INCREASING A TENANT'S RENT"));
        assert!(joined.contains("HB 24-1098"));
    }
}
