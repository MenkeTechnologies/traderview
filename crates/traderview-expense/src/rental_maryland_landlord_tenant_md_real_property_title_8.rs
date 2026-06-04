//! Maryland Landlord-Tenant Law — Md. Real Property
//! Code Title 8 (Landlord and Tenant) Compliance Module —
//! pure-compute check for landlord statutory compliance
//! with Maryland's statewide landlord-tenant regime
//! spanning **Md. Real Prop. §§ 8-101 through 8-704**,
//! including **Subtitle 2 Residential Leases**
//! (§§ 8-201 through 8-218) and **Subtitle 4 Landlord's
//! Remedies Other Than Distraint** (§§ 8-401 through 8-405).
//!
//! Major recent reforms: the **Renters' Rights and
//! Stabilization Act of 2024** (effective **October 1,
//! 2024**) reduced the statutory security deposit cap
//! from 2 months' rent to **1 MONTH'S RENT** for new
//! leases entered into on or after October 1, 2024.
//!
//! Web research (verified 2026-06-03):
//! - **Md. Real Property Code Title 8 (Landlord and Tenant)**: Maryland's statewide landlord-tenant regime; codified at Md. Real Prop. §§ 8-101 through 8-704; non-URLTA statutory framework ([Justia — Maryland Real Property Code Title 8 (2025)](https://law.justia.com/codes/maryland/real-property/title-8/); [Justia — Title 8 Subtitle 2 Residential Leases](https://law.justia.com/codes/maryland/real-property/title-8/subtitle-2/); [Maryland General Assembly — Md. Real Prop. § 8-203](https://mgaleg.maryland.gov/mgawebsite/Laws/StatuteText?article=grp&section=8-203); [iPropertyManagement — Maryland Security Deposit Law](https://ipropertymanagement.com/laws/maryland-security-deposit-returns); [FreeNetLaw — Tenant Rights in Maryland](https://freenetlaw.com/tenant-rights/maryland/); [LegalClarity — Maryland Security Deposit Law 8-203 Rules and Penalties](https://legalclarity.org/maryland-security-deposit-laws-landlord-and-tenant-guide/); [Tentunit Help — Maryland Landlord Responsibility Statement](https://help.tentunit.com/landlord-responsibility/maryland/); [Tenant Rights — Maryland Tenant Security Deposit Rules 2024](https://tenant-rights.com/maryland/maryland-tenant-security-deposit-rules-2024); [LeaseLenses — Maryland Landlord-Tenant Law Guide 2026](https://www.leaselenses.com/blog/maryland-landlord-tenant-law-guide/); [Maryland Property Rentals — Ultimate Guide to Maryland Rental Laws](https://www.marylandpropertyrentals.com/tenant/the-ultimate-guide-to-maryland-rental-laws-everything-you-need-to-succeed-as-a-local-landlord/)).
//! - **Md. Real Prop. § 8-203 Security Deposit Cap — Renters' Rights and Stabilization Act of 2024**: for most residential leases entered into on or after **OCTOBER 1, 2024**, landlords may not charge a security deposit exceeding **ONE MONTH'S RENT**; a higher limit of up to **2 MONTHS' RENT** is permitted only in a narrow exception involving certain tenants who receive utility assistance AND agree in writing to the higher amount.
//! - **Md. Real Prop. § 8-203(e)(1) 45-Day Deposit Return + Itemized List**: landlord must return any remaining deposit within **45 DAYS** after the tenancy ends; if deductions apply, landlord must also send an **ITEMIZED LIST OF DAMAGES** within that same 45-day period.
//! - **Md. Real Prop. § 8-203(e)(4) 3x Wrongful Withholding Penalty**: a landlord who **WRONGFULLY WITHHOLDS** any part of the deposit may owe **UP TO 3 TIMES THE AMOUNT WITHHELD**, plus **REASONABLE ATTORNEY'S FEES**.
//! - **Md. Real Prop. § 8-203(d) Interest-Bearing Account Required**: landlord must hold deposit in an **INTEREST-BEARING ACCOUNT** in a **MARYLAND BRANCH OF A FEDERALLY INSURED FINANCIAL INSTITUTION**; landlord must pay tenant interest at the **U.S. TREASURY YIELD CURVE RATE for one year** as of January 1 of each year (a Maryland-unique interest-payment requirement).
//! - **Md. Real Prop. § 8-208 Prohibited Lease Provisions**: certain lease provisions are **VOID** if included in a residential lease, including: (i) **WAIVER OF NOTICE** required by law; (ii) **WAIVER OF RIGHT TO TRIAL BY JURY**; (iii) **CONFESSION OF JUDGMENT** clauses; (iv) provisions which authorize the landlord to take possession of the leased premises without judicial process; (v) **HOLD HARMLESS** or indemnification clauses; (vi) provisions which alter the landlord's duty to mitigate damages; (vii) attorney's fees clauses **EXCEEDING 15 PERCENT** of unpaid rent.
//! - **Md. Real Prop. § 8-211 Rent Escrow Procedure**: tenant may pay rent into a **COURT-ADMINISTERED ESCROW ACCOUNT** when landlord fails to maintain habitable conditions causing a **SUBSTANTIAL AND SERIOUS THREAT** of danger to the life, health, and safety of the tenant; tenant may invoke rent escrow only after providing landlord with **WRITTEN NOTICE** + reasonable time to repair.
//! - **Md. Real Prop. § 8-401 Failure to Pay Rent — 10-Day Pay-or-Quit Notice**: before filing a complaint for nonpayment of rent, landlord must provide tenant with **WRITTEN NOTICE** of the landlord's intent to file a complaint for failure to pay rent, telling the tenant **HOW MUCH RENT IS DUE** and giving the tenant **10 DAYS** to pay the amount due.
//! - **Md. Real Prop. § 8-401 Summary Ejectment Procedure**: when tenant fails to pay rent, landlord may file a **WRITTEN COMPLAINT in the District Court** asking to repossess the property, for the amount of rent due, and court costs; landlord must possess a **CURRENT LICENSE TO OPERATE** if required by the county and/or municipality to use summary ejectment.
//! - **Md. Real Prop. § 8-401(c) Right of Redemption**: in any action of summary ejectment for failure to pay rent where the landlord is awarded a judgment giving the landlord restitution of the leased premises, the tenant has the **RIGHT OF REDEMPTION** of the leased premises by tendering in **CASH, CERTIFIED CHECK, OR MONEY ORDER** to the landlord or landlord's agent **ALL PAST DUE AMOUNTS, PLUS ALL COURT-AWARDED COSTS AND FEES**, **AT ANY TIME BEFORE ACTUAL EXECUTION OF THE EVICTION ORDER**.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MD_REAL_PROPERTY_TITLE_NUMBER: u32 = 8;
pub const MD_SECURITY_DEPOSIT_CAP_REDUCTION_EFFECTIVE_YEAR: u32 = 2024;
pub const MD_SECURITY_DEPOSIT_CAP_REDUCTION_EFFECTIVE_MONTH: u32 = 10;
pub const MD_SECURITY_DEPOSIT_CAP_REDUCTION_EFFECTIVE_DAY: u32 = 1;
pub const MD_SECURITY_DEPOSIT_CAP_NEW_LEASE_MONTHS: u32 = 1;
pub const MD_SECURITY_DEPOSIT_CAP_OLD_LEASE_MONTHS: u32 = 2;
pub const MD_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 45;
pub const MD_WRONGFUL_WITHHOLDING_MAX_MULTIPLIER: u32 = 3;
pub const MD_PAY_OR_QUIT_NOTICE_DAYS: u32 = 10;
pub const MD_ATTORNEY_FEES_CAP_PERCENT_OF_UNPAID_RENT_BPS: u64 = 1_500;
pub const MD_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromTitle8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseExecutionDateStatus {
    LeaseExecutedOnOrAfterOctober1_2024PostRrsa2024OneMonthCap,
    LeaseExecutedBeforeOctober1_2024PreRrsa2024TwoMonthCap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityAssistanceExceptionStatus {
    UtilityAssistanceExceptionApplicableWithWrittenAgreementForTwoMonthCap,
    UtilityAssistanceExceptionNotApplicableOneMonthCapApplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositHoldingArrangement {
    HeldInInterestBearingAccountMarylandBranchFederallyInsured,
    HeldInNonInterestBearingOrNonMarylandAccount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProhibitedLeaseProvisionStatus {
    NoProhibitedProvisionsIncluded,
    WaiverOfNoticeProvisionIncluded,
    JuryWaiverProvisionIncluded,
    ConfessionOfJudgmentProvisionIncluded,
    SelfHelpRepossessionProvisionIncluded,
    HoldHarmlessOrIndemnificationProvisionIncluded,
    AlterDutyToMitigateDamagesProvisionIncluded,
    AttorneyFeesExceedingFifteenPercentOfUnpaidRentProvisionIncluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentEscrowNoticeStatus {
    TenantGaveWrittenNoticeAndReasonableTimeBeforeInvokingEscrow,
    TenantInvokedEscrowWithoutNoticeOrReasonableTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordLicensingStatus {
    LandlordHoldsCurrentRequiredLicenseToOperate,
    LandlordLacksCurrentRequiredLicenseToOperate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapUnderSection8_203,
    FortyFiveDayDepositReturnUnderSection8_203E1,
    WrongfulWithholdingPenaltyUnderSection8_203E4,
    InterestBearingAccountRequirementUnderSection8_203D,
    ProhibitedLeaseProvisionsUnderSection8_208,
    RentEscrowProcedureUnderSection8_211,
    TenDayPayOrQuitNoticeUnderSection8_401,
    SummaryEjectmentLicensingUnderSection8_401,
    RightOfRedemptionUnderSection8_401C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MdLandlordTenantMode {
    NotApplicableTenancyExemptFromTitle8,
    CompliantDepositAtOrBelowOneMonthCapForNewLease,
    CompliantDepositAtOrBelowTwoMonthCapWithUtilityAssistanceException,
    CompliantDepositAtOrBelowTwoMonthCapForPreRrsa2024Lease,
    CompliantDepositReturnedWithItemizedListWithin45Days,
    CompliantWrongfulWithholdingPenaltyAcknowledged,
    CompliantDepositInInterestBearingMarylandAccount,
    CompliantNoProhibitedLeaseProvisionsIncluded,
    CompliantRentEscrowNoticeAndReasonableTimePrerequisitesMet,
    CompliantTenDayPayOrQuitNoticeProperlyServed,
    CompliantSummaryEjectmentLicensingHeld,
    CompliantRightOfRedemptionPreservedUntilEvictionExecution,
    ViolationDepositExceedsOneMonthCapForNewLease,
    ViolationDepositExceedsTwoMonthCap,
    ViolationDepositReturnedPast45DayDeadline,
    ViolationWrongfulWithholdingTriplesDepositLiability,
    ViolationDepositNotHeldInInterestBearingMarylandAccount,
    ViolationProhibitedLeaseProvisionIncludedClauseVoid,
    ViolationRentEscrowInvokedWithoutNoticeOrReasonableTime,
    ViolationPayOrQuitNoticeShorterThan10Days,
    ViolationSummaryEjectmentWithoutRequiredLicense,
    ViolationRightOfRedemptionRejected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub lease_execution_date_status: LeaseExecutionDateStatus,
    pub utility_assistance_exception_status: UtilityAssistanceExceptionStatus,
    pub deposit_holding_arrangement: DepositHoldingArrangement,
    pub prohibited_lease_provision_status: ProhibitedLeaseProvisionStatus,
    pub rent_escrow_notice_status: RentEscrowNoticeStatus,
    pub landlord_licensing_status: LandlordLicensingStatus,
    pub compliance_aspect: ComplianceAspect,
    pub deposit_amount_in_tenths_of_months_rent: u64,
    pub days_to_return_deposit: u32,
    pub pay_or_quit_notice_days_given: u32,
    pub deposit_wrongfully_withheld: bool,
    pub right_of_redemption_offered_before_eviction_execution: bool,
    pub right_of_redemption_accepted_by_landlord: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: MdLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type MdLandlordTenantInput = Input;
pub type MdLandlordTenantOutput = Output;
pub type MdLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Md. Real Property Code Title 8 (Landlord and Tenant) — Maryland's statewide landlord-tenant regime; codified at Md. Real Prop. §§ 8-101 through 8-704; non-URLTA statutory framework".to_string(),
        "Md. Real Prop. § 8-203 Security Deposit Cap — Renters' Rights and Stabilization Act of 2024 — for most residential leases entered into on or after OCTOBER 1, 2024, landlords may not charge a security deposit exceeding ONE MONTH'S RENT; a higher limit of up to 2 MONTHS' RENT is permitted only in a narrow exception involving certain tenants who receive utility assistance AND agree in writing to the higher amount".to_string(),
        "Md. Real Prop. § 8-203(e)(1) 45-Day Deposit Return + Itemized List — landlord must return any remaining deposit within 45 DAYS after the tenancy ends; if deductions apply, landlord must also send an ITEMIZED LIST OF DAMAGES within that same 45-day period".to_string(),
        "Md. Real Prop. § 8-203(e)(4) 3x Wrongful Withholding Penalty — a landlord who WRONGFULLY WITHHOLDS any part of the deposit may owe UP TO 3 TIMES THE AMOUNT WITHHELD, plus REASONABLE ATTORNEY'S FEES".to_string(),
        "Md. Real Prop. § 8-203(d) Interest-Bearing Account Required — landlord must hold deposit in an INTEREST-BEARING ACCOUNT in a MARYLAND BRANCH OF A FEDERALLY INSURED FINANCIAL INSTITUTION; landlord must pay tenant interest at the U.S. TREASURY YIELD CURVE RATE for one year as of January 1 of each year".to_string(),
        "Md. Real Prop. § 8-208 Prohibited Lease Provisions — certain lease provisions are VOID if included in a residential lease, including: (i) WAIVER OF NOTICE required by law; (ii) WAIVER OF RIGHT TO TRIAL BY JURY; (iii) CONFESSION OF JUDGMENT clauses; (iv) provisions which authorize landlord to take possession without judicial process; (v) HOLD HARMLESS or indemnification clauses; (vi) provisions which alter landlord's duty to mitigate damages; (vii) attorney's fees clauses EXCEEDING 15 PERCENT of unpaid rent".to_string(),
        "Md. Real Prop. § 8-211 Rent Escrow Procedure — tenant may pay rent into a COURT-ADMINISTERED ESCROW ACCOUNT when landlord fails to maintain habitable conditions causing a SUBSTANTIAL AND SERIOUS THREAT of danger to the life, health, and safety of the tenant; tenant may invoke rent escrow only after providing landlord with WRITTEN NOTICE + reasonable time to repair".to_string(),
        "Md. Real Prop. § 8-401 Failure to Pay Rent — 10-Day Pay-or-Quit Notice — before filing a complaint for nonpayment of rent, landlord must provide tenant with WRITTEN NOTICE of the landlord's intent to file a complaint for failure to pay rent, telling the tenant HOW MUCH RENT IS DUE and giving the tenant 10 DAYS to pay the amount due".to_string(),
        "Md. Real Prop. § 8-401 Summary Ejectment Procedure — when tenant fails to pay rent, landlord may file a WRITTEN COMPLAINT in the District Court asking to repossess the property, for the amount of rent due, and court costs; landlord must possess a CURRENT LICENSE TO OPERATE if required by the county and/or municipality to use summary ejectment procedures".to_string(),
        "Md. Real Prop. § 8-401(c) Right of Redemption — in any action of summary ejectment for failure to pay rent where the landlord is awarded a judgment giving the landlord restitution of the leased premises, the tenant has the RIGHT OF REDEMPTION of the leased premises by tendering in CASH, CERTIFIED CHECK, OR MONEY ORDER to the landlord or landlord's agent ALL PAST DUE AMOUNTS, PLUS ALL COURT-AWARDED COSTS AND FEES, AT ANY TIME BEFORE ACTUAL EXECUTION OF THE EVICTION ORDER".to_string(),
        "Justia + Maryland General Assembly + iPropertyManagement + FreeNetLaw + LegalClarity + Tentunit Help + Tenant Rights + LeaseLenses + Maryland Property Rentals + Maryland People's Law Library — practitioner overviews of Md. Real Property Code Title 8".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromTitle8 {
        return Output {
            mode: MdLandlordTenantMode::NotApplicableTenancyExemptFromTitle8,
            statutory_basis: "Md. Real Prop. Title 8 jurisdiction — tenancy exempt from Title 8 coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Maryland Real Property Code Title 8; Title 8 landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapUnderSection8_203 => {
            match input.lease_execution_date_status {
                LeaseExecutionDateStatus::LeaseExecutedOnOrAfterOctober1_2024PostRrsa2024OneMonthCap => {
                    if input.utility_assistance_exception_status
                        == UtilityAssistanceExceptionStatus::UtilityAssistanceExceptionApplicableWithWrittenAgreementForTwoMonthCap
                    {
                        if input.deposit_amount_in_tenths_of_months_rent <= 20 {
                            Output {
                                mode: MdLandlordTenantMode::CompliantDepositAtOrBelowTwoMonthCapWithUtilityAssistanceException,
                                statutory_basis: "Md. Real Prop. § 8-203 — 2-month cap permitted under utility assistance exception with written tenant agreement".to_string(),
                                notes: "COMPLIANT: deposit at or below 2-month cap under § 8-203 utility assistance exception with written tenant agreement.".to_string(),
                                citations,
                            }
                        } else {
                            Output {
                                mode: MdLandlordTenantMode::ViolationDepositExceedsTwoMonthCap,
                                statutory_basis: "Md. Real Prop. § 8-203 — deposit exceeds 2-month cap even with utility assistance exception".to_string(),
                                notes: "VIOLATION: deposit exceeds 2-month cap even with § 8-203 utility assistance exception.".to_string(),
                                citations,
                            }
                        }
                    } else if input.deposit_amount_in_tenths_of_months_rent <= 10 {
                        Output {
                            mode: MdLandlordTenantMode::CompliantDepositAtOrBelowOneMonthCapForNewLease,
                            statutory_basis: "Md. Real Prop. § 8-203 — 1-month cap for leases on or after October 1, 2024 under Renters' Rights and Stabilization Act of 2024".to_string(),
                            notes: "COMPLIANT: deposit at or below 1-month cap for lease executed on or after October 1, 2024 under § 8-203 RRSA 2024 reform.".to_string(),
                            citations,
                        }
                    } else {
                        Output {
                            mode: MdLandlordTenantMode::ViolationDepositExceedsOneMonthCapForNewLease,
                            statutory_basis: "Md. Real Prop. § 8-203 — deposit exceeds 1-month cap for lease on or after October 1, 2024".to_string(),
                            notes: "VIOLATION: deposit exceeds 1-month cap for lease executed on or after October 1, 2024 under § 8-203 RRSA 2024 reform.".to_string(),
                            citations,
                        }
                    }
                }
                LeaseExecutionDateStatus::LeaseExecutedBeforeOctober1_2024PreRrsa2024TwoMonthCap => {
                    if input.deposit_amount_in_tenths_of_months_rent <= 20 {
                        Output {
                            mode: MdLandlordTenantMode::CompliantDepositAtOrBelowTwoMonthCapForPreRrsa2024Lease,
                            statutory_basis: "Md. Real Prop. § 8-203 — 2-month cap for leases executed before October 1, 2024 (pre-RRSA 2024)".to_string(),
                            notes: "COMPLIANT: deposit at or below pre-RRSA 2024 2-month cap for lease executed before October 1, 2024 under § 8-203.".to_string(),
                            citations,
                        }
                    } else {
                        Output {
                            mode: MdLandlordTenantMode::ViolationDepositExceedsTwoMonthCap,
                            statutory_basis: "Md. Real Prop. § 8-203 — deposit exceeds pre-RRSA 2024 2-month cap".to_string(),
                            notes: "VIOLATION: deposit exceeds pre-RRSA 2024 2-month cap for lease executed before October 1, 2024 under § 8-203.".to_string(),
                            citations,
                        }
                    }
                }
            }
        }
        ComplianceAspect::FortyFiveDayDepositReturnUnderSection8_203E1 => {
            if input.days_to_return_deposit <= MD_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS {
                Output {
                    mode: MdLandlordTenantMode::CompliantDepositReturnedWithItemizedListWithin45Days,
                    statutory_basis: "Md. Real Prop. § 8-203(e)(1) — deposit returned with itemized list of damages within 45 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized list at day {d} (within 45-day statutory window) under § 8-203(e)(1).",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MdLandlordTenantMode::ViolationDepositReturnedPast45DayDeadline,
                    statutory_basis: "Md. Real Prop. § 8-203(e)(1) — deposit return exceeded 45-day statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} past 45-day statutory window under § 8-203(e)(1).",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::WrongfulWithholdingPenaltyUnderSection8_203E4 => {
            if input.deposit_wrongfully_withheld {
                Output {
                    mode: MdLandlordTenantMode::ViolationWrongfulWithholdingTriplesDepositLiability,
                    statutory_basis: "Md. Real Prop. § 8-203(e)(4) — wrongful withholding triggers up-to-3x deposit penalty + attorney's fees".to_string(),
                    notes: "VIOLATION: deposit wrongfully withheld; court may award UP TO 3 TIMES THE AMOUNT WITHHELD + REASONABLE ATTORNEY'S FEES under § 8-203(e)(4).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: MdLandlordTenantMode::CompliantWrongfulWithholdingPenaltyAcknowledged,
                    statutory_basis: "Md. Real Prop. § 8-203(e)(4) — no wrongful withholding; up-to-3x deposit penalty not triggered".to_string(),
                    notes: "COMPLIANT: no wrongful withholding under § 8-203(e)(4); up-to-3x deposit penalty + attorney's fees exposure not triggered.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::InterestBearingAccountRequirementUnderSection8_203D => {
            if input.deposit_holding_arrangement
                == DepositHoldingArrangement::HeldInInterestBearingAccountMarylandBranchFederallyInsured
            {
                Output {
                    mode: MdLandlordTenantMode::CompliantDepositInInterestBearingMarylandAccount,
                    statutory_basis: "Md. Real Prop. § 8-203(d) — deposit held in interest-bearing Maryland branch of federally insured financial institution".to_string(),
                    notes: "COMPLIANT: deposit held in interest-bearing account in a Maryland branch of a federally insured financial institution under § 8-203(d); landlord must pay tenant interest at U.S. Treasury yield curve rate for one year as of January 1 of each year.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: MdLandlordTenantMode::ViolationDepositNotHeldInInterestBearingMarylandAccount,
                    statutory_basis: "Md. Real Prop. § 8-203(d) — deposit not held in interest-bearing Maryland account".to_string(),
                    notes: "VIOLATION: deposit not held in interest-bearing account in a Maryland branch of a federally insured financial institution under § 8-203(d).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::ProhibitedLeaseProvisionsUnderSection8_208 => match input
            .prohibited_lease_provision_status
        {
            ProhibitedLeaseProvisionStatus::NoProhibitedProvisionsIncluded => Output {
                mode: MdLandlordTenantMode::CompliantNoProhibitedLeaseProvisionsIncluded,
                statutory_basis: "Md. Real Prop. § 8-208 — lease contains no prohibited provisions".to_string(),
                notes: "COMPLIANT: lease contains no prohibited provisions under § 8-208 (waiver of notice / jury waiver / confession of judgment / self-help repossession / hold harmless / alter duty to mitigate / attorney's fees exceeding 15 % of unpaid rent).".to_string(),
                citations,
            },
            _ => Output {
                mode: MdLandlordTenantMode::ViolationProhibitedLeaseProvisionIncludedClauseVoid,
                statutory_basis: "Md. Real Prop. § 8-208 — lease contains prohibited provision; clause is VOID".to_string(),
                notes: "VIOLATION: lease contains prohibited provision under § 8-208 (waiver of notice / jury waiver / confession of judgment / self-help repossession / hold harmless / alter duty to mitigate / attorney's fees exceeding 15 % of unpaid rent); the prohibited clause is VOID.".to_string(),
                citations,
            },
        },
        ComplianceAspect::RentEscrowProcedureUnderSection8_211 => match input.rent_escrow_notice_status {
            RentEscrowNoticeStatus::TenantGaveWrittenNoticeAndReasonableTimeBeforeInvokingEscrow => {
                Output {
                    mode: MdLandlordTenantMode::CompliantRentEscrowNoticeAndReasonableTimePrerequisitesMet,
                    statutory_basis: "Md. Real Prop. § 8-211 — tenant gave written notice + reasonable time before invoking rent escrow".to_string(),
                    notes: "COMPLIANT: tenant satisfied § 8-211 prerequisites (written notice of substantial and serious threat + reasonable time for landlord to repair) before invoking rent escrow.".to_string(),
                    citations,
                }
            }
            RentEscrowNoticeStatus::TenantInvokedEscrowWithoutNoticeOrReasonableTime => Output {
                mode: MdLandlordTenantMode::ViolationRentEscrowInvokedWithoutNoticeOrReasonableTime,
                statutory_basis: "Md. Real Prop. § 8-211 — rent escrow invoked without notice or reasonable time prerequisites".to_string(),
                notes: "VIOLATION: tenant invoked rent escrow without satisfying § 8-211 prerequisites (written notice of substantial and serious threat + reasonable time for landlord to repair).".to_string(),
                citations,
            },
        },
        ComplianceAspect::TenDayPayOrQuitNoticeUnderSection8_401 => {
            if input.pay_or_quit_notice_days_given >= MD_PAY_OR_QUIT_NOTICE_DAYS {
                Output {
                    mode: MdLandlordTenantMode::CompliantTenDayPayOrQuitNoticeProperlyServed,
                    statutory_basis: "Md. Real Prop. § 8-401 — 10-day pay-or-quit notice for nonpayment properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day pay-or-quit notice satisfies 10-day statutory minimum under § 8-401.",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MdLandlordTenantMode::ViolationPayOrQuitNoticeShorterThan10Days,
                    statutory_basis: "Md. Real Prop. § 8-401 — pay-or-quit notice shorter than statutory 10-day minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day pay-or-quit notice shorter than 10-day statutory minimum under § 8-401.",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::SummaryEjectmentLicensingUnderSection8_401 => match input
            .landlord_licensing_status
        {
            LandlordLicensingStatus::LandlordHoldsCurrentRequiredLicenseToOperate => Output {
                mode: MdLandlordTenantMode::CompliantSummaryEjectmentLicensingHeld,
                statutory_basis: "Md. Real Prop. § 8-401 — landlord holds current required license to operate; summary ejectment available".to_string(),
                notes: "COMPLIANT: landlord holds current required license to operate (if required by county or municipality); summary ejectment procedures available under § 8-401.".to_string(),
                citations,
            },
            LandlordLicensingStatus::LandlordLacksCurrentRequiredLicenseToOperate => Output {
                mode: MdLandlordTenantMode::ViolationSummaryEjectmentWithoutRequiredLicense,
                statutory_basis: "Md. Real Prop. § 8-401 — landlord lacks current required license; summary ejectment unavailable".to_string(),
                notes: "VIOLATION: landlord lacks current required license to operate (where required by county or municipality); summary ejectment procedures unavailable under § 8-401.".to_string(),
                citations,
            },
        },
        ComplianceAspect::RightOfRedemptionUnderSection8_401C => {
            if input.right_of_redemption_offered_before_eviction_execution {
                if input.right_of_redemption_accepted_by_landlord {
                    Output {
                        mode: MdLandlordTenantMode::CompliantRightOfRedemptionPreservedUntilEvictionExecution,
                        statutory_basis: "Md. Real Prop. § 8-401(c) — right of redemption preserved by tenant tender before eviction execution; landlord accepted".to_string(),
                        notes: "COMPLIANT: tenant tendered cash / certified check / money order with all past due amounts + court-awarded costs and fees before actual execution of eviction order under § 8-401(c); landlord accepted; tenancy redeemed.".to_string(),
                        citations,
                    }
                } else {
                    Output {
                        mode: MdLandlordTenantMode::ViolationRightOfRedemptionRejected,
                        statutory_basis: "Md. Real Prop. § 8-401(c) — landlord rejected tenant's valid redemption tender".to_string(),
                        notes: "VIOLATION: landlord rejected tenant's valid right-of-redemption tender (cash / certified check / money order with all past due amounts + court-awarded costs and fees before actual execution of eviction order) under § 8-401(c).".to_string(),
                        citations,
                    }
                }
            } else {
                Output {
                    mode: MdLandlordTenantMode::CompliantRightOfRedemptionPreservedUntilEvictionExecution,
                    statutory_basis: "Md. Real Prop. § 8-401(c) — right of redemption preserved until actual execution of eviction order; tenant did not invoke".to_string(),
                    notes: "COMPLIANT: § 8-401(c) right of redemption preserved until actual execution of eviction order; tenant did not invoke right of redemption.".to_string(),
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
            lease_execution_date_status:
                LeaseExecutionDateStatus::LeaseExecutedOnOrAfterOctober1_2024PostRrsa2024OneMonthCap,
            utility_assistance_exception_status:
                UtilityAssistanceExceptionStatus::UtilityAssistanceExceptionNotApplicableOneMonthCapApplies,
            deposit_holding_arrangement:
                DepositHoldingArrangement::HeldInInterestBearingAccountMarylandBranchFederallyInsured,
            prohibited_lease_provision_status:
                ProhibitedLeaseProvisionStatus::NoProhibitedProvisionsIncluded,
            rent_escrow_notice_status:
                RentEscrowNoticeStatus::TenantGaveWrittenNoticeAndReasonableTimeBeforeInvokingEscrow,
            landlord_licensing_status:
                LandlordLicensingStatus::LandlordHoldsCurrentRequiredLicenseToOperate,
            compliance_aspect: ComplianceAspect::SecurityDepositCapUnderSection8_203,
            deposit_amount_in_tenths_of_months_rent: 10,
            days_to_return_deposit: 35,
            pay_or_quit_notice_days_given: 10,
            deposit_wrongfully_withheld: false,
            right_of_redemption_offered_before_eviction_execution: false,
            right_of_redemption_accepted_by_landlord: false,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromTitle8;
        let out = check(&input);
        assert_eq!(out.mode, MdLandlordTenantMode::NotApplicableTenancyExemptFromTitle8);
    }

    #[test]
    fn deposit_at_one_month_cap_new_lease_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderSection8_203;
        input.lease_execution_date_status =
            LeaseExecutionDateStatus::LeaseExecutedOnOrAfterOctober1_2024PostRrsa2024OneMonthCap;
        input.utility_assistance_exception_status =
            UtilityAssistanceExceptionStatus::UtilityAssistanceExceptionNotApplicableOneMonthCapApplies;
        input.deposit_amount_in_tenths_of_months_rent = 10;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantDepositAtOrBelowOneMonthCapForNewLease
        );
    }

    #[test]
    fn deposit_above_one_month_new_lease_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderSection8_203;
        input.lease_execution_date_status =
            LeaseExecutionDateStatus::LeaseExecutedOnOrAfterOctober1_2024PostRrsa2024OneMonthCap;
        input.utility_assistance_exception_status =
            UtilityAssistanceExceptionStatus::UtilityAssistanceExceptionNotApplicableOneMonthCapApplies;
        input.deposit_amount_in_tenths_of_months_rent = 11;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationDepositExceedsOneMonthCapForNewLease
        );
    }

    #[test]
    fn deposit_at_two_month_cap_with_utility_assistance_exception_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderSection8_203;
        input.lease_execution_date_status =
            LeaseExecutionDateStatus::LeaseExecutedOnOrAfterOctober1_2024PostRrsa2024OneMonthCap;
        input.utility_assistance_exception_status =
            UtilityAssistanceExceptionStatus::UtilityAssistanceExceptionApplicableWithWrittenAgreementForTwoMonthCap;
        input.deposit_amount_in_tenths_of_months_rent = 20;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantDepositAtOrBelowTwoMonthCapWithUtilityAssistanceException
        );
    }

    #[test]
    fn deposit_at_two_month_pre_rrsa_2024_lease_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderSection8_203;
        input.lease_execution_date_status =
            LeaseExecutionDateStatus::LeaseExecutedBeforeOctober1_2024PreRrsa2024TwoMonthCap;
        input.deposit_amount_in_tenths_of_months_rent = 20;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantDepositAtOrBelowTwoMonthCapForPreRrsa2024Lease
        );
    }

    #[test]
    fn deposit_above_two_month_pre_rrsa_2024_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderSection8_203;
        input.lease_execution_date_status =
            LeaseExecutionDateStatus::LeaseExecutedBeforeOctober1_2024PreRrsa2024TwoMonthCap;
        input.deposit_amount_in_tenths_of_months_rent = 21;
        let out = check(&input);
        assert_eq!(out.mode, MdLandlordTenantMode::ViolationDepositExceedsTwoMonthCap);
    }

    #[test]
    fn deposit_returned_at_45_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FortyFiveDayDepositReturnUnderSection8_203E1;
        input.days_to_return_deposit = 45;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantDepositReturnedWithItemizedListWithin45Days
        );
    }

    #[test]
    fn deposit_returned_at_46_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FortyFiveDayDepositReturnUnderSection8_203E1;
        input.days_to_return_deposit = 46;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationDepositReturnedPast45DayDeadline
        );
    }

    #[test]
    fn wrongful_withholding_triggers_3x_penalty() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrongfulWithholdingPenaltyUnderSection8_203E4;
        input.deposit_wrongfully_withheld = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationWrongfulWithholdingTriplesDepositLiability
        );
    }

    #[test]
    fn no_wrongful_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrongfulWithholdingPenaltyUnderSection8_203E4;
        input.deposit_wrongfully_withheld = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantWrongfulWithholdingPenaltyAcknowledged
        );
    }

    #[test]
    fn deposit_in_interest_bearing_maryland_account_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::InterestBearingAccountRequirementUnderSection8_203D;
        input.deposit_holding_arrangement =
            DepositHoldingArrangement::HeldInInterestBearingAccountMarylandBranchFederallyInsured;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantDepositInInterestBearingMarylandAccount
        );
    }

    #[test]
    fn deposit_not_in_interest_bearing_maryland_account_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::InterestBearingAccountRequirementUnderSection8_203D;
        input.deposit_holding_arrangement =
            DepositHoldingArrangement::HeldInNonInterestBearingOrNonMarylandAccount;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationDepositNotHeldInInterestBearingMarylandAccount
        );
    }

    #[test]
    fn no_prohibited_lease_provisions_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ProhibitedLeaseProvisionsUnderSection8_208;
        input.prohibited_lease_provision_status =
            ProhibitedLeaseProvisionStatus::NoProhibitedProvisionsIncluded;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantNoProhibitedLeaseProvisionsIncluded
        );
    }

    #[test]
    fn confession_of_judgment_provision_void_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ProhibitedLeaseProvisionsUnderSection8_208;
        input.prohibited_lease_provision_status =
            ProhibitedLeaseProvisionStatus::ConfessionOfJudgmentProvisionIncluded;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationProhibitedLeaseProvisionIncludedClauseVoid
        );
    }

    #[test]
    fn attorney_fees_exceeding_15_percent_void_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ProhibitedLeaseProvisionsUnderSection8_208;
        input.prohibited_lease_provision_status =
            ProhibitedLeaseProvisionStatus::AttorneyFeesExceedingFifteenPercentOfUnpaidRentProvisionIncluded;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationProhibitedLeaseProvisionIncludedClauseVoid
        );
    }

    #[test]
    fn rent_escrow_with_notice_and_reasonable_time_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentEscrowProcedureUnderSection8_211;
        input.rent_escrow_notice_status =
            RentEscrowNoticeStatus::TenantGaveWrittenNoticeAndReasonableTimeBeforeInvokingEscrow;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantRentEscrowNoticeAndReasonableTimePrerequisitesMet
        );
    }

    #[test]
    fn rent_escrow_without_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentEscrowProcedureUnderSection8_211;
        input.rent_escrow_notice_status =
            RentEscrowNoticeStatus::TenantInvokedEscrowWithoutNoticeOrReasonableTime;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationRentEscrowInvokedWithoutNoticeOrReasonableTime
        );
    }

    #[test]
    fn ten_day_pay_or_quit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayPayOrQuitNoticeUnderSection8_401;
        input.pay_or_quit_notice_days_given = 10;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantTenDayPayOrQuitNoticeProperlyServed
        );
    }

    #[test]
    fn nine_day_pay_or_quit_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayPayOrQuitNoticeUnderSection8_401;
        input.pay_or_quit_notice_days_given = 9;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationPayOrQuitNoticeShorterThan10Days
        );
    }

    #[test]
    fn summary_ejectment_landlord_holds_license_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SummaryEjectmentLicensingUnderSection8_401;
        input.landlord_licensing_status =
            LandlordLicensingStatus::LandlordHoldsCurrentRequiredLicenseToOperate;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantSummaryEjectmentLicensingHeld
        );
    }

    #[test]
    fn summary_ejectment_landlord_lacks_license_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SummaryEjectmentLicensingUnderSection8_401;
        input.landlord_licensing_status =
            LandlordLicensingStatus::LandlordLacksCurrentRequiredLicenseToOperate;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationSummaryEjectmentWithoutRequiredLicense
        );
    }

    #[test]
    fn right_of_redemption_offered_and_accepted_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RightOfRedemptionUnderSection8_401C;
        input.right_of_redemption_offered_before_eviction_execution = true;
        input.right_of_redemption_accepted_by_landlord = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantRightOfRedemptionPreservedUntilEvictionExecution
        );
    }

    #[test]
    fn right_of_redemption_offered_but_rejected_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RightOfRedemptionUnderSection8_401C;
        input.right_of_redemption_offered_before_eviction_execution = true;
        input.right_of_redemption_accepted_by_landlord = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::ViolationRightOfRedemptionRejected
        );
    }

    #[test]
    fn right_of_redemption_not_invoked_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RightOfRedemptionUnderSection8_401C;
        input.right_of_redemption_offered_before_eviction_execution = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MdLandlordTenantMode::CompliantRightOfRedemptionPreservedUntilEvictionExecution
        );
    }

    #[test]
    fn constants_pin_maryland_landlord_tenant_statutory_thresholds() {
        assert_eq!(MD_REAL_PROPERTY_TITLE_NUMBER, 8);
        assert_eq!(MD_SECURITY_DEPOSIT_CAP_REDUCTION_EFFECTIVE_YEAR, 2024);
        assert_eq!(MD_SECURITY_DEPOSIT_CAP_REDUCTION_EFFECTIVE_MONTH, 10);
        assert_eq!(MD_SECURITY_DEPOSIT_CAP_REDUCTION_EFFECTIVE_DAY, 1);
        assert_eq!(MD_SECURITY_DEPOSIT_CAP_NEW_LEASE_MONTHS, 1);
        assert_eq!(MD_SECURITY_DEPOSIT_CAP_OLD_LEASE_MONTHS, 2);
        assert_eq!(MD_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 45);
        assert_eq!(MD_WRONGFUL_WITHHOLDING_MAX_MULTIPLIER, 3);
        assert_eq!(MD_PAY_OR_QUIT_NOTICE_DAYS, 10);
        assert_eq!(MD_ATTORNEY_FEES_CAP_PERCENT_OF_UNPAID_RENT_BPS, 1_500);
        assert_eq!(MD_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_maryland_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Md. Real Property Code Title 8"));
        assert!(joined.contains("Md. Real Prop. § 8-203"));
        assert!(joined.contains("Renters' Rights and Stabilization Act of 2024"));
        assert!(joined.contains("OCTOBER 1, 2024"));
        assert!(joined.contains("ONE MONTH'S RENT"));
        assert!(joined.contains("2 MONTHS' RENT"));
        assert!(joined.contains("utility assistance"));
        assert!(joined.contains("Md. Real Prop. § 8-203(e)(1)"));
        assert!(joined.contains("45 DAYS"));
        assert!(joined.contains("ITEMIZED LIST OF DAMAGES"));
        assert!(joined.contains("Md. Real Prop. § 8-203(e)(4)"));
        assert!(joined.contains("UP TO 3 TIMES THE AMOUNT WITHHELD"));
        assert!(joined.contains("REASONABLE ATTORNEY'S FEES"));
        assert!(joined.contains("Md. Real Prop. § 8-203(d)"));
        assert!(joined.contains("INTEREST-BEARING ACCOUNT"));
        assert!(joined.contains("MARYLAND BRANCH OF A FEDERALLY INSURED FINANCIAL INSTITUTION"));
        assert!(joined.contains("U.S. TREASURY YIELD CURVE RATE"));
        assert!(joined.contains("Md. Real Prop. § 8-208"));
        assert!(joined.contains("VOID"));
        assert!(joined.contains("WAIVER OF NOTICE"));
        assert!(joined.contains("WAIVER OF RIGHT TO TRIAL BY JURY"));
        assert!(joined.contains("CONFESSION OF JUDGMENT"));
        assert!(joined.contains("HOLD HARMLESS"));
        assert!(joined.contains("EXCEEDING 15 PERCENT"));
        assert!(joined.contains("Md. Real Prop. § 8-211"));
        assert!(joined.contains("COURT-ADMINISTERED ESCROW ACCOUNT"));
        assert!(joined.contains("SUBSTANTIAL AND SERIOUS THREAT"));
        assert!(joined.contains("WRITTEN NOTICE"));
        assert!(joined.contains("Md. Real Prop. § 8-401"));
        assert!(joined.contains("10 DAYS"));
        assert!(joined.contains("HOW MUCH RENT IS DUE"));
        assert!(joined.contains("WRITTEN COMPLAINT"));
        assert!(joined.contains("CURRENT LICENSE TO OPERATE"));
        assert!(joined.contains("Md. Real Prop. § 8-401(c)"));
        assert!(joined.contains("RIGHT OF REDEMPTION"));
        assert!(joined.contains("CASH, CERTIFIED CHECK, OR MONEY ORDER"));
        assert!(joined.contains("ALL PAST DUE AMOUNTS, PLUS ALL COURT-AWARDED COSTS AND FEES"));
        assert!(joined.contains("AT ANY TIME BEFORE ACTUAL EXECUTION OF THE EVICTION ORDER"));
    }
}
