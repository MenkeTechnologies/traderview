//! Iowa Uniform Residential Landlord and
//! Tenant Law — Iowa Code Chapter 562A
//! (§§ 562A.1-562A.37)
//! Compliance Module — pure-compute check for landlord
//! statutory compliance with Iowa's URLTA regime.
//!
//! **Distinctive Iowa features**: **3-DAY NONPAYMENT
//! NOTICE** under § 562A.27 (TIED FOR SHORTEST AMONG URLTA
//! STATES — only OH 3-day equals it; cf. SC 5-day, WI
//! 5-day, AZ 5-day, KY 7-day, WA 14-day, TN 14-day);
//! **NO CURE RIGHT** on 3-day nonpayment notice — pay or
//! terminate (one of the most landlord-favorable
//! nonpayment notice schemes in URLTA states); **2-MONTH
//! RENT DEPOSIT CAP** under § 562A.12; **FEDERALLY
//! INSURED HOLDING** required (bank/savings/credit union);
//! **BAD-FAITH PUNITIVE DAMAGES** under § 562A.12 — not
//! to exceed **2x MONTHLY RENTAL PAYMENT** (not 2x
//! deposit; capped differently than other URLTA states)
//! plus actual damages.
//!
//! Web research (verified 2026-06-03):
//! - **Iowa Uniform Residential Landlord and Tenant Law** — codified at Iowa Code §§ 562A.1-562A.37; classic URLTA adoption ([Iowa Legislature — Iowa Code Chapter 562A PDF](https://www.legis.iowa.gov/docs/ico/chapter/562a.pdf); [Iowa Legislature — Iowa Code § 562A.12 Rental Deposits PDF](https://www.legis.iowa.gov/docs/code/562A.12.pdf); [Iowa Legislature — Iowa Code § 562A.15 Landlord to Maintain Fit Premises PDF](https://www.legis.iowa.gov/docs/code/562A.15.pdf); [Iowa Legislature — Iowa Code § 562A.19 Access PDF](https://www.legis.iowa.gov/docs/code/562A.19.pdf); [Iowa Legislature — Iowa Code § 562A.27 Noncompliance PDF](https://www.legis.iowa.gov/docs/code/562a.27.pdf); [Justia — 2024 Iowa Code § 562A.12 Rental Deposits](https://law.justia.com/codes/iowa/title-xiv/chapter-562a/section-562a-12/); [Justia — 2025 Iowa Code § 562A.27 Noncompliance with Rental Agreement](https://law.justia.com/codes/iowa/title-xiv/chapter-562a/section-562a-27/); [Justia — 2016 Iowa Code § 562A.15 Landlord to Maintain Fit Premises](https://law.justia.com/codes/iowa/2016/title-xiv/chapter-562a/section-562a.15/); [Justia — 2025 Iowa Code Title XIV Chapter 562A](https://law.justia.com/codes/iowa/title-xiv/chapter-562a/); [Iowa People's Law Library — Security Deposits](https://www.peopleslawiowa.org/index.php/research-topics/landlordtenant-law/leases/security-deposits); [Iowa People's Law Library — Landlord Rights Duties and Remedies](https://www.peopleslawiowa.org/index.php/research-topics/landlordtenant-law/introduction/landlord-rights-duties-and-remedies); [Iowa People's Law Library — Repairs Your Landlord Must Make](https://www.peopleslawiowa.org/index.php/research-topics/landlordtenant-law/property-condition-and-repairs/repairs-your-landlord-must-make); [FlagMyLease — Iowa Tenant Rights: The URLTA — Iowa's Take](https://www.flagmylease.com/tenant-rights/iowa); [iPropertyManagement — Iowa Security Deposit Law](https://ipropertymanagement.com/laws/iowa-security-deposit-returns); [iPropertyManagement — Iowa Landlord Tenant Laws 2026](https://ipropertymanagement.com/laws/iowa-landlord-tenant-rights); [Landlord-Tenant-Law — Iowa Landlord Tenant Law in Plain English](https://www.landlord-tenant-law.com/iowa-landlord-tenant-law.html); [Innago — Iowa Landlord Tenant Rental Laws & Rights 2025](https://innago.com/iowa-landlord-tenant-laws/); [American Apartment Owners — Iowa Landlord Tenant Law](https://american-apartment-owners-association.org/landlord-tenant-laws/iowa/); [Landlord Studio — Iowa Landlord Tenant Laws](https://www.landlordstudio.com/landlord-tenant-laws/iowa-landlord-tenant-laws); [Tenant Rights — Iowa Security Deposit Retaliation](https://tenant-rights.com/iowa/iowa-security-deposit-retaliation-your-rights-explained); [NCHH — Iowa URLTA Summary PDF](https://nchh.org/resource-library/HH_Codes_IA_URLTA_5-15-08.pdf); [LawServer — Iowa Code § 562A.27](https://www.lawserver.com/law/state/iowa/ia-code/iowa_code_562a-27); [Winneshiek County — Iowa Code 562A Eviction Laws PDF](https://winneshiekcounty.iowa.gov/wp-content/uploads/2017/07/562A-Eviction-Laws.pdf)).
//! - **Iowa Code § 562A.12(1) Rental Deposit Cap — 2 Months' Rent**: a landlord shall not demand or receive as a security deposit an amount or value in **EXCESS OF TWO MONTHS' RENT**.
//! - **Iowa Code § 562A.12(2) Federally Insured Holding**: all rental deposits shall be held by the landlord for the tenant in a **BANK OR SAVINGS AND LOAN ASSOCIATION OR CREDIT UNION** which is **INSURED BY AN AGENCY OF THE FEDERAL GOVERNMENT**; rental deposits shall **NOT BE COMMINGLED** with the personal funds of the landlord.
//! - **Iowa Code § 562A.12(3) Security Deposit Return — 30 Days**: a landlord shall, within **30 DAYS** from the date of termination of the tenancy and receipt of the tenant's mailing address or delivery instructions, return the rental deposit to the tenant or furnish to the tenant a **WRITTEN STATEMENT** showing the specific reason for withholding of the rental deposit or any portion thereof.
//! - **Iowa Code § 562A.12(3) Permissible Withholding Reasons**: the landlord may withhold from the rental deposit only such amounts as are reasonably necessary for: **(a)** to remedy a tenant's default in the payment of **RENT** or of other funds due to the landlord; and **(b)** to restore the dwelling unit to its condition at the commencement of the tenancy, **ORDINARY WEAR AND TEAR EXCEPTED**.
//! - **Iowa Code § 562A.12(7) Bad-Faith Retention Penalty**: the **BAD-FAITH RETENTION** of a deposit by a landlord, or any portion of the rental deposit, in violation of this section shall subject the landlord to **PUNITIVE DAMAGES NOT TO EXCEED 2x MONTHLY RENTAL PAYMENT** in addition to **ACTUAL DAMAGES** — UNUSUAL CAPS PUNITIVE TO 2x MONTHLY RENT rather than 2x deposit amount.
//! - **Iowa Code § 562A.27(2) Nonpayment Notice — 3 Days, No Cure Right**: if rent is unpaid when due and the tenant fails to pay rent within **THREE DAYS** after written notice by the landlord of nonpayment and the landlord's intention to terminate the rental agreement if the rent is not paid within that period of time, the landlord may terminate the rental agreement — **NO STATUTORY CURE RIGHT** beyond the 3-day window.
//! - **Iowa Code § 562A.27(1) Material Noncompliance Notice — 7 Days with Cure Right**: if there is a material noncompliance by the tenant with the rental agreement or a noncompliance with section 562A.17 materially affecting health and safety, the landlord may deliver a written notice to the tenant specifying the acts and omissions constituting the breach and that the rental agreement will terminate upon a date **NOT LESS THAN 7 DAYS** after receipt of the notice if the breach is not remedied in 7 days, and the rental agreement shall terminate as provided in the notice.
//! - **Iowa Code § 562A.19(2) Entry Notice — 24 Hours**: except in case of emergency or if it is impracticable to do so, the landlord shall give the tenant at least **24 HOURS' NOTICE** of the landlord's intent to enter and **ENTER ONLY AT REASONABLE TIMES**; the landlord shall **NOT ABUSE THE RIGHT OF ACCESS** or use it to **HARASS THE TENANT**.
//! - **Iowa Code § 562A.15 Habitability — Four Obligations**: the landlord shall: **(1)** comply with the requirements of applicable **BUILDING AND HOUSING CODES** materially affecting health and safety; **(2)** make all repairs and do whatever is necessary to put and keep the premises in a **FIT AND HABITABLE CONDITION**; **(3)** keep all **COMMON AREAS** of the premises in a **CLEAN AND SAFE CONDITION**; **(4)** maintain in good and safe working order and condition all **ELECTRICAL, PLUMBING, SANITARY, HEATING, VENTILATING, AIR-CONDITIONING**, and other facilities and appliances, including **ELEVATORS**, supplied or required to be supplied by the landlord.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IA_CHAPTER_NUMBER: u32 = 562;
pub const IA_DEPOSIT_CAP_MONTHS_RENT: u32 = 2;
pub const IA_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const IA_BAD_FAITH_PUNITIVE_MULTIPLIER_MONTHS_RENT: u32 = 2;
pub const IA_NONPAYMENT_NOTICE_DAYS: u32 = 3;
pub const IA_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS: u32 = 7;
pub const IA_MATERIAL_NONCOMPLIANCE_CURE_DAYS: u32 = 7;
pub const IA_ENTRY_NOTICE_HOURS: u32 = 24;
pub const IA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromChapter562A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositHoldingStatus {
    DepositHeldInFederallyInsuredInstitutionAndNotCommingled,
    DepositCommingledOrNotFederallyInsured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WithholdingReasonStatus {
    PermissibleWithholdingRentDefaultOrRestorationLessOrdinaryWearAndTear,
    ImpermissibleWithholdingNormalWearAndTearOrOtherImpermissibleReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BadFaithStatus {
    BadFaithRetention,
    NoBadFaithGoodFaithDispute,
    NoBadFaithProperReturn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialNoncomplianceStatus {
    TenantCorrectedNoncomplianceWithin7Days,
    TenantDidNotCorrectNoncomplianceWithin7Days,
    NoMaterialNoncompliance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    DepositCapTwoMonthsRentUnderSection562A121,
    FederallyInsuredHoldingUnderSection562A122,
    SecurityDepositReturnUnderSection562A123,
    PermissibleWithholdingReasonsUnderSection562A123,
    BadFaithRetention2xMonthlyRentPenaltyUnderSection562A127,
    LandlordHabitabilityObligationUnderSection562A15,
    NonpaymentNoticeUnderSection562A272,
    MaterialNoncomplianceNoticeUnderSection562A271,
    LandlordEntryNoticeUnderSection562A192,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IaLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter562A,
    CompliantDepositCapWithinTwoMonthsRent,
    CompliantFederallyInsuredHoldingAndNotCommingled,
    CompliantDepositReturnedWithWrittenStatementWithin30Days,
    CompliantPermissibleWithholdingReasons,
    CompliantNoBadFaithGoodFaithDispute,
    CompliantNoBadFaithProperReturn,
    CompliantLandlordMaintainsHabitabilityUnderSection562A15,
    CompliantThreeDayNonpaymentNoticeProperlyServed,
    CompliantMaterialNoncompliance7DayCureWithTenantCorrection,
    CompliantMaterialNoncompliance7DayNoticeWithoutCure,
    CompliantNoMaterialNoncompliance,
    Compliant24HourEntryNoticeProperlyServed,
    CompliantEmergencyOrImpracticableEntryWithoutNotice,
    ViolationDepositExceedsTwoMonthsRent,
    ViolationDepositCommingledOrNotFederallyInsured,
    ViolationDepositReturnedPast30DayDeadline,
    ViolationImpermissibleWithholdingReasons,
    ViolationBadFaithRetention2xMonthlyRentPenaltyTriggered,
    ViolationLandlordFailedHabitabilityObligation,
    ViolationNonpaymentNoticeShorterThan3Days,
    ViolationMaterialNoncomplianceNoticeShorterThan7Days,
    ViolationEntryNoticeShorterThan24Hours,
    ViolationLandlordAbusedRightOfAccessOrHarassedTenant,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub deposit_holding_status: DepositHoldingStatus,
    pub withholding_reason_status: WithholdingReasonStatus,
    pub bad_faith_status: BadFaithStatus,
    pub material_noncompliance_status: MaterialNoncomplianceStatus,
    pub compliance_aspect: ComplianceAspect,
    pub deposit_amount_in_months_rent: u32,
    pub days_to_return_deposit: u32,
    pub landlord_maintains_habitability: bool,
    pub nonpayment_notice_days_given: u32,
    pub material_noncompliance_notice_days_given: u32,
    pub entry_notice_hours_given: u32,
    pub entry_was_emergency_or_impracticable: bool,
    pub landlord_abused_right_of_access_or_harassed_tenant: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: IaLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type IaLandlordTenantInput = Input;
pub type IaLandlordTenantOutput = Output;
pub type IaLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Iowa Uniform Residential Landlord and Tenant Law — codified at Iowa Code §§ 562A.1-562A.37; classic URLTA adoption".to_string(),
        "Iowa Code § 562A.12(1) Rental Deposit Cap — 2 Months' Rent — a landlord shall not demand or receive as a security deposit an amount or value in EXCESS OF TWO MONTHS' RENT".to_string(),
        "Iowa Code § 562A.12(2) Federally Insured Holding — all rental deposits shall be held by the landlord for the tenant in a BANK OR SAVINGS AND LOAN ASSOCIATION OR CREDIT UNION which is INSURED BY AN AGENCY OF THE FEDERAL GOVERNMENT; rental deposits shall NOT BE COMMINGLED with the personal funds of the landlord".to_string(),
        "Iowa Code § 562A.12(3) Security Deposit Return — 30 Days — a landlord shall, within 30 DAYS from the date of termination of the tenancy and receipt of the tenant's mailing address or delivery instructions, return the rental deposit to the tenant or furnish to the tenant a WRITTEN STATEMENT showing the specific reason for withholding".to_string(),
        "Iowa Code § 562A.12(3) Permissible Withholding Reasons — the landlord may withhold from the rental deposit only such amounts as are reasonably necessary for: (a) to remedy a tenant's default in the payment of RENT or of other funds due to the landlord; and (b) to restore the dwelling unit to its condition at the commencement of the tenancy, ORDINARY WEAR AND TEAR EXCEPTED".to_string(),
        "Iowa Code § 562A.12(7) Bad-Faith Retention Penalty — the BAD-FAITH RETENTION of a deposit by a landlord, or any portion of the rental deposit, in violation of this section shall subject the landlord to PUNITIVE DAMAGES NOT TO EXCEED 2x MONTHLY RENTAL PAYMENT in addition to ACTUAL DAMAGES — UNUSUAL CAP TO 2x MONTHLY RENT rather than 2x deposit amount".to_string(),
        "Iowa Code § 562A.27(2) Nonpayment Notice — 3 Days, No Cure Right — if rent is unpaid when due and the tenant fails to pay rent within THREE DAYS after written notice by the landlord of nonpayment and the landlord's intention to terminate the rental agreement if the rent is not paid within that period of time, the landlord may terminate the rental agreement — NO STATUTORY CURE RIGHT beyond the 3-day window".to_string(),
        "Iowa Code § 562A.27(1) Material Noncompliance Notice — 7 Days with Cure Right — if there is a material noncompliance by the tenant with the rental agreement or a noncompliance with section 562A.17 materially affecting health and safety, the landlord may deliver a written notice to the tenant specifying the acts and omissions constituting the breach and that the rental agreement will terminate upon a date NOT LESS THAN 7 DAYS after receipt of the notice if the breach is not remedied in 7 days".to_string(),
        "Iowa Code § 562A.19(2) Entry Notice — 24 Hours — except in case of emergency or if it is impracticable to do so, the landlord shall give the tenant at least 24 HOURS' NOTICE of the landlord's intent to enter and ENTER ONLY AT REASONABLE TIMES; the landlord shall NOT ABUSE THE RIGHT OF ACCESS or use it to HARASS THE TENANT".to_string(),
        "Iowa Code § 562A.15 Habitability — Four Obligations — the landlord shall: (1) comply with BUILDING AND HOUSING CODES; (2) make all repairs and put premises in a FIT AND HABITABLE CONDITION; (3) keep COMMON AREAS clean and safe; (4) maintain ELECTRICAL, PLUMBING, SANITARY, HEATING, VENTILATING, AIR-CONDITIONING facilities including ELEVATORS".to_string(),
        "Iowa Legislature + Justia + Iowa People's Law Library + FlagMyLease + iPropertyManagement + Landlord-Tenant-Law + Innago + American Apartment Owners + Landlord Studio + Tenant Rights + NCHH + LawServer + Winneshiek County — practitioner overviews of Iowa URLTA".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromChapter562A {
        return Output {
            mode: IaLandlordTenantMode::NotApplicableTenancyExemptFromChapter562A,
            statutory_basis: "Iowa Code Chapter 562A jurisdiction — tenancy exempt from URLTA coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Iowa URLTA; URLTA landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::DepositCapTwoMonthsRentUnderSection562A121 => {
            if input.deposit_amount_in_months_rent <= IA_DEPOSIT_CAP_MONTHS_RENT {
                Output {
                    mode: IaLandlordTenantMode::CompliantDepositCapWithinTwoMonthsRent,
                    statutory_basis: "Iowa Code § 562A.12(1) — deposit within 2-month rent cap".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit equal to {m} month(s) of rent satisfies 2-month statutory cap under Iowa Code § 562A.12(1).",
                        m = input.deposit_amount_in_months_rent,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: IaLandlordTenantMode::ViolationDepositExceedsTwoMonthsRent,
                    statutory_basis: "Iowa Code § 562A.12(1) — deposit exceeds 2-month rent cap".to_string(),
                    notes: format!(
                        "VIOLATION: deposit equal to {m} month(s) of rent exceeds 2-month statutory cap under Iowa Code § 562A.12(1).",
                        m = input.deposit_amount_in_months_rent,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::FederallyInsuredHoldingUnderSection562A122 => {
            match input.deposit_holding_status {
                DepositHoldingStatus::DepositHeldInFederallyInsuredInstitutionAndNotCommingled => Output {
                    mode: IaLandlordTenantMode::CompliantFederallyInsuredHoldingAndNotCommingled,
                    statutory_basis: "Iowa Code § 562A.12(2) — deposit held in federally insured institution and not commingled".to_string(),
                    notes: "COMPLIANT: rental deposit held in BANK OR SAVINGS AND LOAN ASSOCIATION OR CREDIT UNION INSURED BY AN AGENCY OF THE FEDERAL GOVERNMENT and NOT COMMINGLED with landlord personal funds under Iowa Code § 562A.12(2).".to_string(),
                    citations,
                },
                DepositHoldingStatus::DepositCommingledOrNotFederallyInsured => Output {
                    mode: IaLandlordTenantMode::ViolationDepositCommingledOrNotFederallyInsured,
                    statutory_basis: "Iowa Code § 562A.12(2) — deposit commingled or not federally insured".to_string(),
                    notes: "VIOLATION: rental deposit commingled with landlord personal funds OR not held in federally insured institution under Iowa Code § 562A.12(2).".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::SecurityDepositReturnUnderSection562A123 => {
            if input.days_to_return_deposit <= IA_DEPOSIT_RETURN_DEADLINE_DAYS {
                Output {
                    mode: IaLandlordTenantMode::CompliantDepositReturnedWithWrittenStatementWithin30Days,
                    statutory_basis: "Iowa Code § 562A.12(3) — deposit returned with written statement within 30 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with written statement at day {d} (within 30-day statutory window) under Iowa Code § 562A.12(3).",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: IaLandlordTenantMode::ViolationDepositReturnedPast30DayDeadline,
                    statutory_basis: "Iowa Code § 562A.12(3) — deposit return exceeded 30-day statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} past 30-day statutory window under Iowa Code § 562A.12(3).",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::PermissibleWithholdingReasonsUnderSection562A123 => {
            match input.withholding_reason_status {
                WithholdingReasonStatus::PermissibleWithholdingRentDefaultOrRestorationLessOrdinaryWearAndTear => Output {
                    mode: IaLandlordTenantMode::CompliantPermissibleWithholdingReasons,
                    statutory_basis: "Iowa Code § 562A.12(3) — permissible withholding reasons".to_string(),
                    notes: "COMPLIANT: withholding for permissible reasons under Iowa Code § 562A.12(3) — RENT default and/or restoration less ORDINARY WEAR AND TEAR.".to_string(),
                    citations,
                },
                WithholdingReasonStatus::ImpermissibleWithholdingNormalWearAndTearOrOtherImpermissibleReason => Output {
                    mode: IaLandlordTenantMode::ViolationImpermissibleWithholdingReasons,
                    statutory_basis: "Iowa Code § 562A.12(3) — impermissible withholding reasons".to_string(),
                    notes: "VIOLATION: withholding for impermissible reasons under Iowa Code § 562A.12(3) — NORMAL WEAR AND TEAR or other impermissible reason; only rent default and restoration less ordinary wear and tear permitted.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::BadFaithRetention2xMonthlyRentPenaltyUnderSection562A127 => {
            match input.bad_faith_status {
                BadFaithStatus::BadFaithRetention => Output {
                    mode: IaLandlordTenantMode::ViolationBadFaithRetention2xMonthlyRentPenaltyTriggered,
                    statutory_basis: "Iowa Code § 562A.12(7) — bad-faith retention triggers 2x monthly rent + actual damages".to_string(),
                    notes: "VIOLATION: bad-faith retention under Iowa Code § 562A.12(7); tenant may recover PUNITIVE DAMAGES NOT TO EXCEED 2x MONTHLY RENTAL PAYMENT plus ACTUAL DAMAGES.".to_string(),
                    citations,
                },
                BadFaithStatus::NoBadFaithGoodFaithDispute => Output {
                    mode: IaLandlordTenantMode::CompliantNoBadFaithGoodFaithDispute,
                    statutory_basis: "Iowa Code § 562A.12(7) — no bad-faith; good-faith dispute".to_string(),
                    notes: "COMPLIANT: landlord retained deposit in good-faith dispute over deductions under Iowa Code § 562A.12(7); 2x monthly rent punitive penalty exposure not triggered.".to_string(),
                    citations,
                },
                BadFaithStatus::NoBadFaithProperReturn => Output {
                    mode: IaLandlordTenantMode::CompliantNoBadFaithProperReturn,
                    statutory_basis: "Iowa Code § 562A.12(7) — no bad-faith; proper return".to_string(),
                    notes: "COMPLIANT: landlord properly returned deposit under Iowa Code § 562A.12(7); 2x monthly rent punitive penalty exposure not triggered.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::LandlordHabitabilityObligationUnderSection562A15 => {
            if input.landlord_maintains_habitability {
                Output {
                    mode: IaLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection562A15,
                    statutory_basis: "Iowa Code § 562A.15 — landlord maintains all four habitability obligations".to_string(),
                    notes: "COMPLIANT: landlord maintains all four habitability obligations under Iowa Code § 562A.15 (building/housing codes + fit and habitable condition + common areas clean and safe + electrical/plumbing/sanitary/heating/ventilating/AC facilities including elevators).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: IaLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation,
                    statutory_basis: "Iowa Code § 562A.15 — landlord failed habitability obligation".to_string(),
                    notes: "VIOLATION: landlord failed one or more of the four habitability obligations under Iowa Code § 562A.15.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::NonpaymentNoticeUnderSection562A272 => {
            if input.nonpayment_notice_days_given >= IA_NONPAYMENT_NOTICE_DAYS {
                Output {
                    mode: IaLandlordTenantMode::CompliantThreeDayNonpaymentNoticeProperlyServed,
                    statutory_basis: "Iowa Code § 562A.27(2) — 3-day nonpayment notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day nonpayment notice satisfies 3-day statutory minimum under Iowa Code § 562A.27(2); NO STATUTORY CURE RIGHT beyond the 3-day window.",
                        d = input.nonpayment_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: IaLandlordTenantMode::ViolationNonpaymentNoticeShorterThan3Days,
                    statutory_basis: "Iowa Code § 562A.27(2) — nonpayment notice shorter than 3-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day nonpayment notice shorter than 3-day statutory minimum under Iowa Code § 562A.27(2).",
                        d = input.nonpayment_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::MaterialNoncomplianceNoticeUnderSection562A271 => {
            if input.material_noncompliance_notice_days_given < IA_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS
            {
                return Output {
                    mode: IaLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan7Days,
                    statutory_basis: "Iowa Code § 562A.27(1) — material noncompliance notice shorter than 7-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day material noncompliance notice shorter than 7-day statutory minimum under Iowa Code § 562A.27(1).",
                        d = input.material_noncompliance_notice_days_given,
                    ),
                    citations,
                };
            }
            match input.material_noncompliance_status {
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin7Days => Output {
                    mode: IaLandlordTenantMode::CompliantMaterialNoncompliance7DayCureWithTenantCorrection,
                    statutory_basis: "Iowa Code § 562A.27(1) — tenant cured material noncompliance within 7 days".to_string(),
                    notes: "COMPLIANT: tenant corrected material noncompliance within 7-day cure window under Iowa Code § 562A.27(1); tenancy continues.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin7Days => Output {
                    mode: IaLandlordTenantMode::CompliantMaterialNoncompliance7DayNoticeWithoutCure,
                    statutory_basis: "Iowa Code § 562A.27(1) — 7-day notice served + tenant did not cure".to_string(),
                    notes: "COMPLIANT: 7-day material noncompliance notice properly served under Iowa Code § 562A.27(1); tenant did not correct within cure window; landlord may proceed with eviction.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::NoMaterialNoncompliance => Output {
                    mode: IaLandlordTenantMode::CompliantNoMaterialNoncompliance,
                    statutory_basis: "Iowa Code § 562A.27(1) — no material noncompliance".to_string(),
                    notes: "COMPLIANT: no material noncompliance condition present under Iowa Code § 562A.27(1); eviction notice not triggered.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::LandlordEntryNoticeUnderSection562A192 => {
            if input.landlord_abused_right_of_access_or_harassed_tenant {
                return Output {
                    mode: IaLandlordTenantMode::ViolationLandlordAbusedRightOfAccessOrHarassedTenant,
                    statutory_basis: "Iowa Code § 562A.19(2) — landlord abused right of access or harassed tenant".to_string(),
                    notes: "VIOLATION: landlord ABUSED THE RIGHT OF ACCESS or used it to HARASS THE TENANT under Iowa Code § 562A.19(2); statutory limitation on access regardless of notice given.".to_string(),
                    citations,
                };
            }
            if input.entry_was_emergency_or_impracticable {
                return Output {
                    mode: IaLandlordTenantMode::CompliantEmergencyOrImpracticableEntryWithoutNotice,
                    statutory_basis: "Iowa Code § 562A.19(2) — emergency or impracticable entry without 24-hour notice".to_string(),
                    notes: "COMPLIANT: entry was emergency or impracticable to give notice under Iowa Code § 562A.19(2); 24-hour notice requirement excused; entry must still occur at reasonable times.".to_string(),
                    citations,
                };
            }
            if input.entry_notice_hours_given >= IA_ENTRY_NOTICE_HOURS {
                Output {
                    mode: IaLandlordTenantMode::Compliant24HourEntryNoticeProperlyServed,
                    statutory_basis: "Iowa Code § 562A.19(2) — 24-hour entry notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {h}-hour entry notice satisfies 24-hour statutory minimum under Iowa Code § 562A.19(2).",
                        h = input.entry_notice_hours_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: IaLandlordTenantMode::ViolationEntryNoticeShorterThan24Hours,
                    statutory_basis: "Iowa Code § 562A.19(2) — entry notice shorter than 24-hour statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {h}-hour entry notice shorter than 24-hour statutory minimum under Iowa Code § 562A.19(2).",
                        h = input.entry_notice_hours_given,
                    ),
                    citations,
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
            tenancy_coverage: TenancyCoverage::CoveredResidentialTenancy,
            deposit_holding_status:
                DepositHoldingStatus::DepositHeldInFederallyInsuredInstitutionAndNotCommingled,
            withholding_reason_status:
                WithholdingReasonStatus::PermissibleWithholdingRentDefaultOrRestorationLessOrdinaryWearAndTear,
            bad_faith_status: BadFaithStatus::NoBadFaithProperReturn,
            material_noncompliance_status:
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin7Days,
            compliance_aspect: ComplianceAspect::DepositCapTwoMonthsRentUnderSection562A121,
            deposit_amount_in_months_rent: 2,
            days_to_return_deposit: 25,
            landlord_maintains_habitability: true,
            nonpayment_notice_days_given: 3,
            material_noncompliance_notice_days_given: 7,
            entry_notice_hours_given: 24,
            entry_was_emergency_or_impracticable: false,
            landlord_abused_right_of_access_or_harassed_tenant: false,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromChapter562A;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::NotApplicableTenancyExemptFromChapter562A);
    }

    #[test]
    fn deposit_within_two_months_rent_cap_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositCapTwoMonthsRentUnderSection562A121;
        input.deposit_amount_in_months_rent = 2;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::CompliantDepositCapWithinTwoMonthsRent);
    }

    #[test]
    fn deposit_exceeds_two_months_rent_cap_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositCapTwoMonthsRentUnderSection562A121;
        input.deposit_amount_in_months_rent = 3;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::ViolationDepositExceedsTwoMonthsRent);
    }

    #[test]
    fn federally_insured_holding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FederallyInsuredHoldingUnderSection562A122;
        input.deposit_holding_status =
            DepositHoldingStatus::DepositHeldInFederallyInsuredInstitutionAndNotCommingled;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::CompliantFederallyInsuredHoldingAndNotCommingled
        );
    }

    #[test]
    fn commingled_deposit_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FederallyInsuredHoldingUnderSection562A122;
        input.deposit_holding_status =
            DepositHoldingStatus::DepositCommingledOrNotFederallyInsured;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::ViolationDepositCommingledOrNotFederallyInsured
        );
    }

    #[test]
    fn deposit_returned_at_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection562A123;
        input.days_to_return_deposit = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::CompliantDepositReturnedWithWrittenStatementWithin30Days
        );
    }

    #[test]
    fn deposit_returned_at_31_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection562A123;
        input.days_to_return_deposit = 31;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::ViolationDepositReturnedPast30DayDeadline);
    }

    #[test]
    fn permissible_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleWithholdingReasonsUnderSection562A123;
        input.withholding_reason_status =
            WithholdingReasonStatus::PermissibleWithholdingRentDefaultOrRestorationLessOrdinaryWearAndTear;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::CompliantPermissibleWithholdingReasons);
    }

    #[test]
    fn impermissible_withholding_normal_wear_and_tear_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleWithholdingReasonsUnderSection562A123;
        input.withholding_reason_status =
            WithholdingReasonStatus::ImpermissibleWithholdingNormalWearAndTearOrOtherImpermissibleReason;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::ViolationImpermissibleWithholdingReasons);
    }

    #[test]
    fn no_bad_faith_proper_return_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BadFaithRetention2xMonthlyRentPenaltyUnderSection562A127;
        input.bad_faith_status = BadFaithStatus::NoBadFaithProperReturn;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::CompliantNoBadFaithProperReturn);
    }

    #[test]
    fn no_bad_faith_good_faith_dispute_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BadFaithRetention2xMonthlyRentPenaltyUnderSection562A127;
        input.bad_faith_status = BadFaithStatus::NoBadFaithGoodFaithDispute;
        let out = check(&input);
        assert_eq!(out.mode, IaLandlordTenantMode::CompliantNoBadFaithGoodFaithDispute);
    }

    #[test]
    fn bad_faith_retention_2x_monthly_rent_penalty_triggered_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BadFaithRetention2xMonthlyRentPenaltyUnderSection562A127;
        input.bad_faith_status = BadFaithStatus::BadFaithRetention;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::ViolationBadFaithRetention2xMonthlyRentPenaltyTriggered
        );
    }

    #[test]
    fn landlord_maintains_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderSection562A15;
        input.landlord_maintains_habitability = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection562A15
        );
    }

    #[test]
    fn landlord_failed_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderSection562A15;
        input.landlord_maintains_habitability = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation
        );
    }

    #[test]
    fn three_day_nonpayment_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection562A272;
        input.nonpayment_notice_days_given = 3;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::CompliantThreeDayNonpaymentNoticeProperlyServed
        );
    }

    #[test]
    fn two_day_nonpayment_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection562A272;
        input.nonpayment_notice_days_given = 2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::ViolationNonpaymentNoticeShorterThan3Days
        );
    }

    #[test]
    fn material_noncompliance_7_day_cure_with_correction_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection562A271;
        input.material_noncompliance_notice_days_given = 7;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin7Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::CompliantMaterialNoncompliance7DayCureWithTenantCorrection
        );
    }

    #[test]
    fn material_noncompliance_7_day_notice_without_cure_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection562A271;
        input.material_noncompliance_notice_days_given = 7;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin7Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::CompliantMaterialNoncompliance7DayNoticeWithoutCure
        );
    }

    #[test]
    fn material_noncompliance_6_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection562A271;
        input.material_noncompliance_notice_days_given = 6;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan7Days
        );
    }

    #[test]
    fn twenty_four_hour_entry_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection562A192;
        input.entry_notice_hours_given = 24;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::Compliant24HourEntryNoticeProperlyServed
        );
    }

    #[test]
    fn twenty_three_hour_entry_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection562A192;
        input.entry_notice_hours_given = 23;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::ViolationEntryNoticeShorterThan24Hours
        );
    }

    #[test]
    fn emergency_entry_without_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection562A192;
        input.entry_was_emergency_or_impracticable = true;
        input.entry_notice_hours_given = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::CompliantEmergencyOrImpracticableEntryWithoutNotice
        );
    }

    #[test]
    fn landlord_abused_right_of_access_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection562A192;
        input.landlord_abused_right_of_access_or_harassed_tenant = true;
        input.entry_notice_hours_given = 24;
        let out = check(&input);
        assert_eq!(
            out.mode,
            IaLandlordTenantMode::ViolationLandlordAbusedRightOfAccessOrHarassedTenant
        );
    }

    #[test]
    fn constants_pin_iowa_landlord_tenant_statutory_thresholds() {
        assert_eq!(IA_CHAPTER_NUMBER, 562);
        assert_eq!(IA_DEPOSIT_CAP_MONTHS_RENT, 2);
        assert_eq!(IA_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(IA_BAD_FAITH_PUNITIVE_MULTIPLIER_MONTHS_RENT, 2);
        assert_eq!(IA_NONPAYMENT_NOTICE_DAYS, 3);
        assert_eq!(IA_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS, 7);
        assert_eq!(IA_MATERIAL_NONCOMPLIANCE_CURE_DAYS, 7);
        assert_eq!(IA_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(IA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_iowa_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Iowa Uniform Residential Landlord and Tenant Law"));
        assert!(joined.contains("Iowa Code §§ 562A.1-562A.37"));
        assert!(joined.contains("Iowa Code § 562A.12(1)"));
        assert!(joined.contains("EXCESS OF TWO MONTHS' RENT"));
        assert!(joined.contains("Iowa Code § 562A.12(2)"));
        assert!(joined.contains("BANK OR SAVINGS AND LOAN ASSOCIATION OR CREDIT UNION"));
        assert!(joined.contains("INSURED BY AN AGENCY OF THE FEDERAL GOVERNMENT"));
        assert!(joined.contains("NOT BE COMMINGLED"));
        assert!(joined.contains("Iowa Code § 562A.12(3)"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("WRITTEN STATEMENT"));
        assert!(joined.contains("ORDINARY WEAR AND TEAR EXCEPTED"));
        assert!(joined.contains("Iowa Code § 562A.12(7)"));
        assert!(joined.contains("BAD-FAITH RETENTION"));
        assert!(joined.contains("PUNITIVE DAMAGES NOT TO EXCEED 2x MONTHLY RENTAL PAYMENT"));
        assert!(joined.contains("ACTUAL DAMAGES"));
        assert!(joined.contains("Iowa Code § 562A.27(2)"));
        assert!(joined.contains("THREE DAYS"));
        assert!(joined.contains("NO STATUTORY CURE RIGHT"));
        assert!(joined.contains("Iowa Code § 562A.27(1)"));
        assert!(joined.contains("NOT LESS THAN 7 DAYS"));
        assert!(joined.contains("Iowa Code § 562A.19(2)"));
        assert!(joined.contains("24 HOURS' NOTICE"));
        assert!(joined.contains("ENTER ONLY AT REASONABLE TIMES"));
        assert!(joined.contains("NOT ABUSE THE RIGHT OF ACCESS"));
        assert!(joined.contains("HARASS THE TENANT"));
        assert!(joined.contains("Iowa Code § 562A.15"));
        assert!(joined.contains("BUILDING AND HOUSING CODES"));
        assert!(joined.contains("FIT AND HABITABLE CONDITION"));
        assert!(joined.contains("COMMON AREAS"));
        assert!(joined.contains("ELECTRICAL, PLUMBING, SANITARY, HEATING, VENTILATING, AIR-CONDITIONING"));
        assert!(joined.contains("ELEVATORS"));
    }
}
