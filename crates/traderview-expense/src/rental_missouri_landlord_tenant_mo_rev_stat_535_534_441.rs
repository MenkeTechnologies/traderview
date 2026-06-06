//! Missouri Landlord-Tenant Law Compliance Module —
//! pure-compute check for landlord statutory compliance
//! with Missouri's three-chapter landlord-tenant regime
//! spanning **Mo. Rev. Stat. Chapter 535 (Landlord-Tenant
//! Actions)** (§§ 535.010 through 535.300), **Chapter 534
//! (Forcible Entry and Unlawful Detainer)** (§§ 534.010
//! through 534.560), and **Chapter 441 (Landlord and
//! Tenant)** (§§ 441.005 through 441.880).
//!
//! Web research (verified 2026-06-03):
//! - **Mo. Rev. Stat. Ch. 535 (Landlord-Tenant Actions)**: governs landlord-side actions including rent and possession + security deposit administration; codified at Mo. Rev. Stat. §§ 535.010 through 535.300 ([Justia — Missouri Revised Statutes Chapter 535 (2025)](https://law.justia.com/codes/missouri/title-xxxvi/chapter-535/); [Missouri Revisor of Statutes — Mo. Rev. Stat. § 535.300](https://revisor.mo.gov/main/OneSection.aspx?section=535.300); [Missouri Revisor of Statutes — Mo. Rev. Stat. § 535.020](https://revisor.mo.gov/main/OneSection.aspx?section=535.020); [Missouri Revisor of Statutes — Mo. Rev. Stat. § 535.030](https://revisor.mo.gov/main/OneSection.aspx?section=535.030); [Justia — Missouri Revised Statutes Chapter 534 (2025)](https://law.justia.com/codes/missouri/title-xxxvi/chapter-534/); [Missouri Revisor of Statutes — Mo. Rev. Stat. § 534.030](https://revisor.mo.gov/main/OneSection.aspx?section=534.030); [Missouri Revisor of Statutes — Mo. Rev. Stat. § 534.330](https://revisor.mo.gov/main/OneSection.aspx?section=534.330); [Justia — Missouri Revised Statutes Chapter 441 (2025)](https://law.justia.com/codes/missouri/title-xxix/chapter-441/); [Hemlane — Missouri Security Deposit Laws in 2026](https://www.hemlane.com/resources/missouri-security-deposit-laws/); [Whale — Missouri Security Deposit Laws](https://www.gowhale.com/security-deposit-laws/missouri); [Gabris Law — Missouri Evictions in a Nutshell](https://stlconstructionlawyer.com/missouri-evictions-in-a-nutshell/); [Gabris Law — St. Louis Eviction Lawyer Discusses Missouri Landlord-Tenant Law](https://stlconstructionlawyer.com/st-louis-eviction-lawyer-landlord-tenant-laws/); [iPropertyManagement — Missouri Landlord Responsibilities for Habitability](https://ipropertymanagement.com/laws/missouri-landlord-responsibilities); [iPropertyManagement — Missouri Warranty of Habitability 2025](https://ipropertymanagement.com/laws/warranty-of-habitability-missouri); [Tenant Rights — Implied Warranty of Habitability Missouri](https://tenant-rights.com/missouri/implied-warranty-of-habitability-missouri); [Missouri Bar — Missouri's Implied Warranty of Habitability](https://news.mobar.org/missouris-implied-warranty-of-habitability/); [DHH Law Firm — The Implied Warranty of Habitability in Missouri](https://www.dhhlawfirm.com/landlords-tenants-implied-warranty-habitability-missouri/); [iPropertyManagement — Missouri Landlord Tenant Laws 2026](https://ipropertymanagement.com/laws/missouri-landlord-tenant-rights); [LeaseRunner — Eviction Process in Missouri: A Guide for Landlords 2025](https://www.leaserunner.com/laws/eviction-process-in-missouri); [Hemlane — Missouri Tenant-Landlord Rental Laws & Rights for 2026](https://www.hemlane.com/resources/missouri-tenant-landlord-law/); [Nolo — Tenant Defenses to Evictions in Missouri](https://www.nolo.com/legal-encyclopedia/tenant-defenses-evictions-missouri.html); [Landlord-Tenant-Law — Missouri Landlord Tenant Law and Statutes in Plain English](https://www.landlord-tenant-law.com/missouri-landlord-tenant-law.html)).
//! - **Mo. Rev. Stat. § 535.300 Security Deposit Cap**: a landlord may **NOT DEMAND OR RECEIVE A SECURITY DEPOSIT IN EXCESS OF TWO MONTHS' RENT** — Missouri's statutory cap is 2 months (cf. MD/CA 1 month under recent reforms; FL 2 months; NC tiered).
//! - **Mo. Rev. Stat. § 535.300 Federally Insured Holding Institution Required**: all security deposits shall be held by the landlord for the tenant **IN A BANK, CREDIT UNION, OR DEPOSITORY INSTITUTION WHICH IS INSURED BY AN AGENCY OF THE FEDERAL GOVERNMENT** — Missouri specifies federally insured institution requirement.
//! - **Mo. Rev. Stat. § 535.300 30-Day Deposit Return + Itemized List**: within **30 DAYS** after the date of termination of the tenancy, the landlord shall: (1) **RETURN THE FULL AMOUNT** of the security deposit; OR (2) **FURNISH TO THE TENANT A WRITTEN ITEMIZED LIST OF THE DAMAGES** for which the security deposit or any portion thereof is withheld, along with the balance of the security deposit.
//! - **Mo. Rev. Stat. § 535.300 Permissible Withholding Reasons**: the landlord may withhold from the security deposit only such amounts as are reasonably necessary for: **(1) RENT DEFAULT** under the rental agreement; AND **(2) RESTORATION** to the dwelling unit's condition at the commencement of the tenancy, **ORDINARY WEAR AND TEAR EXCEPTED**.
//! - **Mo. Rev. Stat. § 535.300 Move-Out Inspection Notice + Tenant Right to Attend**: the landlord shall give the tenant or his representative **REASONABLE NOTICE IN WRITING** at his last known address or in person of the **DATE AND TIME OF MOVE-OUT INSPECTION**; tenant has the **RIGHT TO BE PRESENT** at the inspection — Missouri statutorily preserves a tenant's right to attend the move-out inspection, an unusual practitioner constraint.
//! - **Mo. Rev. Stat. § 535.300 2x Wrongful Withholding Penalty**: if the landlord **WRONGFULLY WITHHOLDS** all or any portion of the security deposit in violation of § 535.300, the tenant shall recover as damages **TWICE THE AMOUNT WRONGFULLY WITHHELD** (a strict 2x multiplier — not "up to 2x" like Maryland's "up to 3x" or Washington's "up to 2x" intentional-refusal penalty).
//! - **Mo. Rev. Stat. § 535.020 Rent and Possession Action**: when rent has become due and payable and payment has been demanded, a landlord or agent may file a **VERIFIED STATEMENT WITH THE CIRCUIT COURT** setting forth the terms of the rental, the amount due, that the rent has been demanded and not paid, and describing the property; giving notice under § 441.060 (1-month notice for periodic tenancy termination) is **NOT REQUIRED** prior to filing a statement under Chapter 535 — Missouri rent-and-possession action requires only demand + nonpayment, no advance notice period.
//! - **Mo. Rev. Stat. § 534.030 Unlawful Detainer**: when any person willfully and without force holds over any lands, tenements or other possessions after the termination of the time for which they were demised, or after foreclosure with written notice, or when premises are occupied incident to employment and employee holds over after termination — such person is guilty of unlawful detainer; Missouri's unlawful detainer action is distinct from rent and possession action under § 535.020.
//! - **Mo. Rev. Stat. § 441.234 14-Day Habitability Repair Notice**: if a tenant provides **WRITTEN NOTICE** of a condition that violates **LOCAL HOUSING OR BUILDING CODES**, the landlord has **14 DAYS** to make repairs — Missouri statutorily allocates a 14-day window for landlords to respond to written tenant habitability complaints.
//! - **Mo. Rev. Stat. § 441.060 1-Month Notice for Periodic Tenancy Termination**: 1-month notice required to terminate a month-to-month tenancy at will under § 441.060; this notice is **NOT REQUIRED** for the rent and possession action under § 535.020.
//! - **10-Day Lease Violation Notice (Practitioner Standard)**: if a tenant violates any portion of the lease agreement, the landlord must first give the tenant a **10-DAY NOTICE** that states the tenant has 10 days to move out of the rental property or the tenant will be evicted (practitioner standard for lease violation evictions distinct from nonpayment rent and possession action).
//! - **Implied Warranty of Habitability (Common Law)**: Missouri courts apply a common-law implied warranty of habitability that the property is habitable at the beginning of the lease and will remain so throughout the tenancy; addresses conditions materially affecting tenant health and safety (hazardous mold, water leaks, inadequate ventilation, structural problems leading to moisture intrusion).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MO_CHAPTER_535_NUMBER: u32 = 535;
pub const MO_CHAPTER_534_NUMBER: u32 = 534;
pub const MO_CHAPTER_441_NUMBER: u32 = 441;
pub const MO_SECURITY_DEPOSIT_CAP_MONTHS: u32 = 2;
pub const MO_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const MO_WRONGFUL_WITHHOLDING_MULTIPLIER: u32 = 2;
pub const MO_HABITABILITY_REPAIR_DEADLINE_DAYS: u32 = 14;
pub const MO_PERIODIC_TENANCY_TERMINATION_NOTICE_DAYS: u32 = 30;
pub const MO_LEASE_VIOLATION_NOTICE_DAYS: u32 = 10;
pub const MO_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromMissouriLandlordTenantStatutes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositHoldingArrangement {
    HeldInFederallyInsuredBankCreditUnionOrDepositoryInstitution,
    HeldInNonFederallyInsuredOrUninsuredAccount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveOutInspectionStatus {
    LandlordGaveReasonableWrittenNoticeAndTenantHadRightToAttend,
    LandlordDidNotGiveReasonableWrittenNoticeOrDeniedTenantRightToAttend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentAndPossessionPrerequisiteStatus {
    RentDueDemandedAndUnpaid,
    RentNotDueOrNotDemandedOrPaid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WithholdingReasonStatus {
    WithheldForRentDefaultOrRestorationOnly,
    WithheldForOrdinaryWearAndTearOrOtherImpermissibleReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapUnderSection535300,
    FederallyInsuredHoldingInstitutionUnderSection535300,
    ThirtyDayDepositReturnUnderSection535300,
    PermissibleWithholdingReasonsUnderSection535300,
    MoveOutInspectionNoticeUnderSection535300,
    TwoxWrongfulWithholdingPenaltyUnderSection535300,
    RentAndPossessionActionUnderSection535020,
    FourteenDayHabitabilityRepairNoticeUnderSection441234,
    PeriodicTenancyTerminationNoticeUnderSection441060,
    LeaseViolationTenDayNoticePractitionerStandard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MoLandlordTenantMode {
    NotApplicableTenancyExemptFromMissouriLandlordTenantStatutes,
    CompliantSecurityDepositAtOrBelowTwoMonthsCap,
    CompliantDepositInFederallyInsuredInstitution,
    CompliantDepositReturnedWithItemizedListWithin30Days,
    CompliantWithholdingForPermissibleReasonsOnly,
    CompliantMoveOutInspectionNoticeWithTenantAttendanceRight,
    CompliantNoWrongfulWithholding,
    CompliantRentAndPossessionActionPrerequisitesMet,
    CompliantFourteenDayHabitabilityRepairWindowObserved,
    CompliantPeriodicTenancyTerminationNoticeMet,
    CompliantTenDayLeaseViolationNoticeServed,
    ViolationSecurityDepositExceedsTwoMonthsCap,
    ViolationDepositHeldInNonFederallyInsuredInstitution,
    ViolationDepositReturnedPast30DayDeadline,
    ViolationWithheldForOrdinaryWearAndTearOrImpermissibleReason,
    ViolationLandlordDeniedMoveOutInspectionNoticeOrTenantAttendance,
    ViolationWrongfulWithholdingDoublesDepositLiability,
    ViolationRentAndPossessionActionWithoutDemandedNonpayment,
    ViolationHabitabilityRepairNotMadeWithin14Days,
    ViolationPeriodicTenancyTerminationNoticeShorterThan30Days,
    ViolationLeaseViolationNoticeShorterThan10Days,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub deposit_holding_arrangement: DepositHoldingArrangement,
    pub move_out_inspection_status: MoveOutInspectionStatus,
    pub rent_and_possession_prerequisite_status: RentAndPossessionPrerequisiteStatus,
    pub withholding_reason_status: WithholdingReasonStatus,
    pub compliance_aspect: ComplianceAspect,
    pub deposit_amount_tenths_of_months_rent: u64,
    pub days_to_return_deposit: u32,
    pub deposit_wrongfully_withheld: bool,
    pub days_landlord_made_habitability_repair: u32,
    pub periodic_tenancy_termination_notice_days_given: u32,
    pub lease_violation_notice_days_given: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: MoLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type MoLandlordTenantInput = Input;
pub type MoLandlordTenantOutput = Output;
pub type MoLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Missouri Landlord-Tenant Law — three-chapter regime spanning Mo. Rev. Stat. Chapter 535 (Landlord-Tenant Actions, §§ 535.010-535.300), Chapter 534 (Forcible Entry and Unlawful Detainer, §§ 534.010-534.560), Chapter 441 (Landlord and Tenant, §§ 441.005-441.880)".to_string(),
        "Mo. Rev. Stat. § 535.300 Security Deposit Cap — a landlord may NOT DEMAND OR RECEIVE A SECURITY DEPOSIT IN EXCESS OF TWO MONTHS' RENT".to_string(),
        "Mo. Rev. Stat. § 535.300 Federally Insured Holding Institution Required — all security deposits shall be held by the landlord for the tenant IN A BANK, CREDIT UNION, OR DEPOSITORY INSTITUTION WHICH IS INSURED BY AN AGENCY OF THE FEDERAL GOVERNMENT".to_string(),
        "Mo. Rev. Stat. § 535.300 30-Day Deposit Return + Itemized List — within 30 DAYS after the date of termination of the tenancy, the landlord shall: (1) RETURN THE FULL AMOUNT of the security deposit; OR (2) FURNISH TO THE TENANT A WRITTEN ITEMIZED LIST OF THE DAMAGES, along with the balance of the security deposit".to_string(),
        "Mo. Rev. Stat. § 535.300 Permissible Withholding Reasons — landlord may withhold from the security deposit only such amounts as are reasonably necessary for (1) RENT DEFAULT under the rental agreement; AND (2) RESTORATION to the dwelling unit's condition at the commencement of the tenancy, ORDINARY WEAR AND TEAR EXCEPTED".to_string(),
        "Mo. Rev. Stat. § 535.300 Move-Out Inspection Notice + Tenant Right to Attend — landlord shall give the tenant or his representative REASONABLE NOTICE IN WRITING at his last known address or in person of the DATE AND TIME OF MOVE-OUT INSPECTION; tenant has the RIGHT TO BE PRESENT at the inspection".to_string(),
        "Mo. Rev. Stat. § 535.300 2x Wrongful Withholding Penalty — if the landlord WRONGFULLY WITHHOLDS all or any portion of the security deposit in violation of § 535.300, the tenant shall recover as damages TWICE THE AMOUNT WRONGFULLY WITHHELD".to_string(),
        "Mo. Rev. Stat. § 535.020 Rent and Possession Action — when rent has become due and payable and payment has been demanded, a landlord or agent may file a VERIFIED STATEMENT WITH THE CIRCUIT COURT; giving notice under § 441.060 (1-month notice for periodic tenancy termination) is NOT REQUIRED prior to filing a statement under Chapter 535".to_string(),
        "Mo. Rev. Stat. § 534.030 Unlawful Detainer — when any person willfully and without force holds over any lands, tenements or other possessions after the termination of the time for which they were demised, or after foreclosure with written notice, or when premises are occupied incident to employment and employee holds over after termination — such person is guilty of unlawful detainer".to_string(),
        "Mo. Rev. Stat. § 441.234 14-Day Habitability Repair Notice — if a tenant provides WRITTEN NOTICE of a condition that violates LOCAL HOUSING OR BUILDING CODES, the landlord has 14 DAYS to make repairs".to_string(),
        "Mo. Rev. Stat. § 441.060 1-Month Notice for Periodic Tenancy Termination — 1-month notice required to terminate a month-to-month tenancy at will; this notice is NOT REQUIRED for the rent and possession action under § 535.020".to_string(),
        "10-Day Lease Violation Notice (Practitioner Standard) — if a tenant violates any portion of the lease agreement, the landlord must first give the tenant a 10-DAY NOTICE that states the tenant has 10 days to move out".to_string(),
        "Implied Warranty of Habitability (Common Law) — Missouri courts apply a common-law implied warranty of habitability that the property is habitable at the beginning of the lease and will remain so throughout the tenancy".to_string(),
        "Justia + Missouri Revisor of Statutes + Hemlane + Whale + Gabris Law + iPropertyManagement + Tenant Rights + Missouri Bar + DHH Law Firm + LeaseRunner + Nolo + Landlord-Tenant-Law — practitioner overviews of Missouri landlord-tenant law".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromMissouriLandlordTenantStatutes {
        return Output {
            mode: MoLandlordTenantMode::NotApplicableTenancyExemptFromMissouriLandlordTenantStatutes,
            statutory_basis: "Missouri landlord-tenant statutes jurisdiction — tenancy exempt from Chapters 535 / 534 / 441 coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Missouri landlord-tenant statutes (Chapters 535 / 534 / 441); Missouri landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapUnderSection535300 => {
            if input.deposit_amount_tenths_of_months_rent <= 20 {
                Output {
                    mode: MoLandlordTenantMode::CompliantSecurityDepositAtOrBelowTwoMonthsCap,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — security deposit at or below 2-month cap".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit at {tenths} tenths-of-months rent within 2-month (20 tenths) statutory cap under § 535.300.",
                        tenths = input.deposit_amount_tenths_of_months_rent,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MoLandlordTenantMode::ViolationSecurityDepositExceedsTwoMonthsCap,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — security deposit exceeds 2-month statutory cap".to_string(),
                    notes: format!(
                        "VIOLATION: deposit at {tenths} tenths-of-months rent exceeds 2-month (20 tenths) statutory cap under § 535.300.",
                        tenths = input.deposit_amount_tenths_of_months_rent,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::FederallyInsuredHoldingInstitutionUnderSection535300 => match input
            .deposit_holding_arrangement
        {
            DepositHoldingArrangement::HeldInFederallyInsuredBankCreditUnionOrDepositoryInstitution => Output {
                mode: MoLandlordTenantMode::CompliantDepositInFederallyInsuredInstitution,
                statutory_basis: "Mo. Rev. Stat. § 535.300 — deposit held in federally insured bank/credit union/depository institution".to_string(),
                notes: "COMPLIANT: deposit held in a bank, credit union, or depository institution insured by an agency of the federal government under § 535.300.".to_string(),
                citations,
            },
            DepositHoldingArrangement::HeldInNonFederallyInsuredOrUninsuredAccount => Output {
                mode: MoLandlordTenantMode::ViolationDepositHeldInNonFederallyInsuredInstitution,
                statutory_basis: "Mo. Rev. Stat. § 535.300 — deposit not held in federally insured bank/credit union/depository institution".to_string(),
                notes: "VIOLATION: deposit held in non-federally-insured or uninsured account under § 535.300; landlord must hold deposit in a federally insured institution.".to_string(),
                citations,
            },
        },
        ComplianceAspect::ThirtyDayDepositReturnUnderSection535300 => {
            if input.days_to_return_deposit <= MO_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS {
                Output {
                    mode: MoLandlordTenantMode::CompliantDepositReturnedWithItemizedListWithin30Days,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — deposit returned with itemized list within 30 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized list at day {d} (within 30-day statutory window) under § 535.300.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MoLandlordTenantMode::ViolationDepositReturnedPast30DayDeadline,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — deposit return exceeded 30-day statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} (past 30-day statutory window) under § 535.300.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::PermissibleWithholdingReasonsUnderSection535300 => match input
            .withholding_reason_status
        {
            WithholdingReasonStatus::WithheldForRentDefaultOrRestorationOnly => Output {
                mode: MoLandlordTenantMode::CompliantWithholdingForPermissibleReasonsOnly,
                statutory_basis: "Mo. Rev. Stat. § 535.300 — withholding limited to rent default and restoration only (ordinary wear and tear excepted)".to_string(),
                notes: "COMPLIANT: withholding limited to permissible reasons under § 535.300 (rent default + restoration to commencement condition with ordinary wear and tear excepted).".to_string(),
                citations,
            },
            WithholdingReasonStatus::WithheldForOrdinaryWearAndTearOrOtherImpermissibleReason => {
                Output {
                    mode: MoLandlordTenantMode::ViolationWithheldForOrdinaryWearAndTearOrImpermissibleReason,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — withholding for ordinary wear and tear or other impermissible reason prohibited".to_string(),
                    notes: "VIOLATION: withholding for ordinary wear and tear or other impermissible reason under § 535.300; only rent default and restoration (ordinary wear and tear excepted) permitted.".to_string(),
                    citations,
                }
            }
        },
        ComplianceAspect::MoveOutInspectionNoticeUnderSection535300 => match input
            .move_out_inspection_status
        {
            MoveOutInspectionStatus::LandlordGaveReasonableWrittenNoticeAndTenantHadRightToAttend => {
                Output {
                    mode: MoLandlordTenantMode::CompliantMoveOutInspectionNoticeWithTenantAttendanceRight,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — landlord gave reasonable written notice of move-out inspection + preserved tenant right to attend".to_string(),
                    notes: "COMPLIANT: landlord gave reasonable written notice of move-out inspection date and time + preserved tenant right to be present under § 535.300.".to_string(),
                    citations,
                }
            }
            MoveOutInspectionStatus::LandlordDidNotGiveReasonableWrittenNoticeOrDeniedTenantRightToAttend => {
                Output {
                    mode: MoLandlordTenantMode::ViolationLandlordDeniedMoveOutInspectionNoticeOrTenantAttendance,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — landlord failed to give reasonable written notice or denied tenant right to attend move-out inspection".to_string(),
                    notes: "VIOLATION: landlord did not give reasonable written notice of move-out inspection OR denied tenant right to be present under § 535.300.".to_string(),
                    citations,
                }
            }
        },
        ComplianceAspect::TwoxWrongfulWithholdingPenaltyUnderSection535300 => {
            if input.deposit_wrongfully_withheld {
                Output {
                    mode: MoLandlordTenantMode::ViolationWrongfulWithholdingDoublesDepositLiability,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — wrongful withholding triggers 2x deposit penalty".to_string(),
                    notes: "VIOLATION: deposit wrongfully withheld; tenant recovers as damages TWICE THE AMOUNT WRONGFULLY WITHHELD under § 535.300 (strict 2x multiplier).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: MoLandlordTenantMode::CompliantNoWrongfulWithholding,
                    statutory_basis: "Mo. Rev. Stat. § 535.300 — no wrongful withholding".to_string(),
                    notes: "COMPLIANT: no wrongful withholding under § 535.300; 2x penalty exposure not triggered.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::RentAndPossessionActionUnderSection535020 => match input
            .rent_and_possession_prerequisite_status
        {
            RentAndPossessionPrerequisiteStatus::RentDueDemandedAndUnpaid => Output {
                mode: MoLandlordTenantMode::CompliantRentAndPossessionActionPrerequisitesMet,
                statutory_basis: "Mo. Rev. Stat. § 535.020 — rent and possession action prerequisites met (rent due + demanded + unpaid)".to_string(),
                notes: "COMPLIANT: rent due + demanded + unpaid prerequisites met under § 535.020; landlord may file verified statement with the circuit court; no § 441.060 1-month notice required.".to_string(),
                citations,
            },
            RentAndPossessionPrerequisiteStatus::RentNotDueOrNotDemandedOrPaid => Output {
                mode: MoLandlordTenantMode::ViolationRentAndPossessionActionWithoutDemandedNonpayment,
                statutory_basis: "Mo. Rev. Stat. § 535.020 — rent and possession action requires rent due + demanded + unpaid".to_string(),
                notes: "VIOLATION: rent and possession action filed without meeting § 535.020 prerequisites (rent due + demanded + unpaid).".to_string(),
                citations,
            },
        },
        ComplianceAspect::FourteenDayHabitabilityRepairNoticeUnderSection441234 => {
            if input.days_landlord_made_habitability_repair <= MO_HABITABILITY_REPAIR_DEADLINE_DAYS
            {
                Output {
                    mode: MoLandlordTenantMode::CompliantFourteenDayHabitabilityRepairWindowObserved,
                    statutory_basis: "Mo. Rev. Stat. § 441.234 — habitability repair completed within 14 days of written tenant notice".to_string(),
                    notes: format!(
                        "COMPLIANT: habitability repair completed at day {d} (within 14-day statutory window) under § 441.234.",
                        d = input.days_landlord_made_habitability_repair,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MoLandlordTenantMode::ViolationHabitabilityRepairNotMadeWithin14Days,
                    statutory_basis: "Mo. Rev. Stat. § 441.234 — habitability repair not made within 14-day statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: habitability repair completed at day {d} (past 14-day statutory window) under § 441.234.",
                        d = input.days_landlord_made_habitability_repair,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::PeriodicTenancyTerminationNoticeUnderSection441060 => {
            if input.periodic_tenancy_termination_notice_days_given
                >= MO_PERIODIC_TENANCY_TERMINATION_NOTICE_DAYS
            {
                Output {
                    mode: MoLandlordTenantMode::CompliantPeriodicTenancyTerminationNoticeMet,
                    statutory_basis: "Mo. Rev. Stat. § 441.060 — 1-month (30-day) notice for periodic tenancy termination properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day periodic tenancy termination notice satisfies 1-month (30-day) statutory minimum under § 441.060.",
                        d = input.periodic_tenancy_termination_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MoLandlordTenantMode::ViolationPeriodicTenancyTerminationNoticeShorterThan30Days,
                    statutory_basis: "Mo. Rev. Stat. § 441.060 — periodic tenancy termination notice shorter than statutory 1-month (30-day) minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day periodic tenancy termination notice shorter than 1-month (30-day) statutory minimum under § 441.060.",
                        d = input.periodic_tenancy_termination_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::LeaseViolationTenDayNoticePractitionerStandard => {
            if input.lease_violation_notice_days_given >= MO_LEASE_VIOLATION_NOTICE_DAYS {
                Output {
                    mode: MoLandlordTenantMode::CompliantTenDayLeaseViolationNoticeServed,
                    statutory_basis: "10-day lease violation notice (practitioner standard) properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day lease violation notice satisfies 10-day practitioner standard for lease violation evictions in Missouri.",
                        d = input.lease_violation_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MoLandlordTenantMode::ViolationLeaseViolationNoticeShorterThan10Days,
                    statutory_basis: "10-day lease violation notice (practitioner standard) shorter than 10-day standard".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day lease violation notice shorter than 10-day practitioner standard for lease violation evictions in Missouri.",
                        d = input.lease_violation_notice_days_given,
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
            deposit_holding_arrangement:
                DepositHoldingArrangement::HeldInFederallyInsuredBankCreditUnionOrDepositoryInstitution,
            move_out_inspection_status:
                MoveOutInspectionStatus::LandlordGaveReasonableWrittenNoticeAndTenantHadRightToAttend,
            rent_and_possession_prerequisite_status:
                RentAndPossessionPrerequisiteStatus::RentDueDemandedAndUnpaid,
            withholding_reason_status:
                WithholdingReasonStatus::WithheldForRentDefaultOrRestorationOnly,
            compliance_aspect: ComplianceAspect::SecurityDepositCapUnderSection535300,
            deposit_amount_tenths_of_months_rent: 20,
            days_to_return_deposit: 25,
            deposit_wrongfully_withheld: false,
            days_landlord_made_habitability_repair: 10,
            periodic_tenancy_termination_notice_days_given: 30,
            lease_violation_notice_days_given: 10,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromMissouriLandlordTenantStatutes;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::NotApplicableTenancyExemptFromMissouriLandlordTenantStatutes
        );
    }

    #[test]
    fn deposit_at_two_month_cap_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderSection535300;
        input.deposit_amount_tenths_of_months_rent = 20;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantSecurityDepositAtOrBelowTwoMonthsCap
        );
    }

    #[test]
    fn deposit_above_two_month_cap_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderSection535300;
        input.deposit_amount_tenths_of_months_rent = 21;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationSecurityDepositExceedsTwoMonthsCap
        );
    }

    #[test]
    fn deposit_in_federally_insured_institution_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::FederallyInsuredHoldingInstitutionUnderSection535300;
        input.deposit_holding_arrangement =
            DepositHoldingArrangement::HeldInFederallyInsuredBankCreditUnionOrDepositoryInstitution;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantDepositInFederallyInsuredInstitution
        );
    }

    #[test]
    fn deposit_in_non_federally_insured_institution_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::FederallyInsuredHoldingInstitutionUnderSection535300;
        input.deposit_holding_arrangement =
            DepositHoldingArrangement::HeldInNonFederallyInsuredOrUninsuredAccount;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationDepositHeldInNonFederallyInsuredInstitution
        );
    }

    #[test]
    fn deposit_returned_at_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThirtyDayDepositReturnUnderSection535300;
        input.days_to_return_deposit = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantDepositReturnedWithItemizedListWithin30Days
        );
    }

    #[test]
    fn deposit_returned_at_31_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThirtyDayDepositReturnUnderSection535300;
        input.days_to_return_deposit = 31;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationDepositReturnedPast30DayDeadline
        );
    }

    #[test]
    fn permissible_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleWithholdingReasonsUnderSection535300;
        input.withholding_reason_status =
            WithholdingReasonStatus::WithheldForRentDefaultOrRestorationOnly;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantWithholdingForPermissibleReasonsOnly
        );
    }

    #[test]
    fn withheld_for_ordinary_wear_and_tear_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleWithholdingReasonsUnderSection535300;
        input.withholding_reason_status =
            WithholdingReasonStatus::WithheldForOrdinaryWearAndTearOrOtherImpermissibleReason;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationWithheldForOrdinaryWearAndTearOrImpermissibleReason
        );
    }

    #[test]
    fn move_out_inspection_notice_with_attendance_right_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MoveOutInspectionNoticeUnderSection535300;
        input.move_out_inspection_status =
            MoveOutInspectionStatus::LandlordGaveReasonableWrittenNoticeAndTenantHadRightToAttend;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantMoveOutInspectionNoticeWithTenantAttendanceRight
        );
    }

    #[test]
    fn move_out_inspection_no_notice_or_denied_attendance_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MoveOutInspectionNoticeUnderSection535300;
        input.move_out_inspection_status =
            MoveOutInspectionStatus::LandlordDidNotGiveReasonableWrittenNoticeOrDeniedTenantRightToAttend;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationLandlordDeniedMoveOutInspectionNoticeOrTenantAttendance
        );
    }

    #[test]
    fn no_wrongful_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TwoxWrongfulWithholdingPenaltyUnderSection535300;
        input.deposit_wrongfully_withheld = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantNoWrongfulWithholding
        );
    }

    #[test]
    fn wrongful_withholding_doubles_liability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TwoxWrongfulWithholdingPenaltyUnderSection535300;
        input.deposit_wrongfully_withheld = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationWrongfulWithholdingDoublesDepositLiability
        );
    }

    #[test]
    fn rent_and_possession_with_demand_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentAndPossessionActionUnderSection535020;
        input.rent_and_possession_prerequisite_status =
            RentAndPossessionPrerequisiteStatus::RentDueDemandedAndUnpaid;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantRentAndPossessionActionPrerequisitesMet
        );
    }

    #[test]
    fn rent_and_possession_without_demand_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentAndPossessionActionUnderSection535020;
        input.rent_and_possession_prerequisite_status =
            RentAndPossessionPrerequisiteStatus::RentNotDueOrNotDemandedOrPaid;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationRentAndPossessionActionWithoutDemandedNonpayment
        );
    }

    #[test]
    fn habitability_repair_at_14_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::FourteenDayHabitabilityRepairNoticeUnderSection441234;
        input.days_landlord_made_habitability_repair = 14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantFourteenDayHabitabilityRepairWindowObserved
        );
    }

    #[test]
    fn habitability_repair_at_15_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::FourteenDayHabitabilityRepairNoticeUnderSection441234;
        input.days_landlord_made_habitability_repair = 15;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationHabitabilityRepairNotMadeWithin14Days
        );
    }

    #[test]
    fn periodic_tenancy_termination_30_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::PeriodicTenancyTerminationNoticeUnderSection441060;
        input.periodic_tenancy_termination_notice_days_given = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantPeriodicTenancyTerminationNoticeMet
        );
    }

    #[test]
    fn periodic_tenancy_termination_29_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::PeriodicTenancyTerminationNoticeUnderSection441060;
        input.periodic_tenancy_termination_notice_days_given = 29;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationPeriodicTenancyTerminationNoticeShorterThan30Days
        );
    }

    #[test]
    fn lease_violation_10_day_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LeaseViolationTenDayNoticePractitionerStandard;
        input.lease_violation_notice_days_given = 10;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::CompliantTenDayLeaseViolationNoticeServed
        );
    }

    #[test]
    fn lease_violation_9_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LeaseViolationTenDayNoticePractitionerStandard;
        input.lease_violation_notice_days_given = 9;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MoLandlordTenantMode::ViolationLeaseViolationNoticeShorterThan10Days
        );
    }

    #[test]
    fn constants_pin_missouri_landlord_tenant_statutory_thresholds() {
        assert_eq!(MO_CHAPTER_535_NUMBER, 535);
        assert_eq!(MO_CHAPTER_534_NUMBER, 534);
        assert_eq!(MO_CHAPTER_441_NUMBER, 441);
        assert_eq!(MO_SECURITY_DEPOSIT_CAP_MONTHS, 2);
        assert_eq!(MO_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(MO_WRONGFUL_WITHHOLDING_MULTIPLIER, 2);
        assert_eq!(MO_HABITABILITY_REPAIR_DEADLINE_DAYS, 14);
        assert_eq!(MO_PERIODIC_TENANCY_TERMINATION_NOTICE_DAYS, 30);
        assert_eq!(MO_LEASE_VIOLATION_NOTICE_DAYS, 10);
        assert_eq!(MO_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_missouri_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Missouri Landlord-Tenant Law"));
        assert!(joined.contains("Mo. Rev. Stat. Chapter 535"));
        assert!(joined.contains("Chapter 534"));
        assert!(joined.contains("Chapter 441"));
        assert!(joined.contains("Mo. Rev. Stat. § 535.300"));
        assert!(joined.contains("TWO MONTHS' RENT"));
        assert!(joined.contains("BANK, CREDIT UNION, OR DEPOSITORY INSTITUTION"));
        assert!(joined.contains("INSURED BY AN AGENCY OF THE FEDERAL GOVERNMENT"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("RETURN THE FULL AMOUNT"));
        assert!(joined.contains("FURNISH TO THE TENANT A WRITTEN ITEMIZED LIST"));
        assert!(joined.contains("RENT DEFAULT"));
        assert!(joined.contains("RESTORATION"));
        assert!(joined.contains("ORDINARY WEAR AND TEAR EXCEPTED"));
        assert!(joined.contains("REASONABLE NOTICE IN WRITING"));
        assert!(joined.contains("DATE AND TIME OF MOVE-OUT INSPECTION"));
        assert!(joined.contains("RIGHT TO BE PRESENT"));
        assert!(joined.contains("TWICE THE AMOUNT WRONGFULLY WITHHELD"));
        assert!(joined.contains("Mo. Rev. Stat. § 535.020"));
        assert!(joined.contains("VERIFIED STATEMENT WITH THE CIRCUIT COURT"));
        assert!(joined.contains("NOT REQUIRED"));
        assert!(joined.contains("Mo. Rev. Stat. § 534.030"));
        assert!(joined.contains("Mo. Rev. Stat. § 441.234"));
        assert!(joined.contains("WRITTEN NOTICE"));
        assert!(joined.contains("LOCAL HOUSING OR BUILDING CODES"));
        assert!(joined.contains("14 DAYS"));
        assert!(joined.contains("Mo. Rev. Stat. § 441.060"));
        assert!(joined.contains("1-month"));
        assert!(joined.contains("10-DAY NOTICE"));
        assert!(joined.contains("Implied Warranty of Habitability"));
    }
}
