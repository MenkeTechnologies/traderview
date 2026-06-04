//! Washington Residential Landlord-Tenant Act (RLTA)
//! RCW Chapter 59.18 — Compliance Module — pure-compute
//! check for landlord statutory compliance with
//! Washington's statewide RLTA covering security deposit
//! return + itemized statement + written checklist
//! prerequisite, landlord habitability obligations under
//! RCW 59.18.060, 14-day pay-or-quit notice under
//! RCW 59.18.057, tenant repair-and-deduct remedy under
//! RCW 59.18.100, and rent receipt requirement under
//! RCW 59.18.063.
//!
//! Originally enacted by **Laws of 1973**, 1st Ex.
//! Session, Chapter 207, **§§ 1-46**; codified at
//! Title 59, Chapter 59.18 of the Revised Code of
//! Washington (RCW); substantially amended over five
//! decades with the most consequential reforms in
//! **2019 (SB 5600 — extended pay-or-quit from 3 to 14
//! days)** and **2021 (SB 5160 — landmark just-cause /
//! eviction reform)**.
//!
//! Web research (verified 2026-06-03):
//! - **RCW Chapter 59.18 Residential Landlord-Tenant Act**: governs all residential tenancies in Washington State; broader than the rent-stabilization-specific **HB 1217 (Engrossed Substitute House Bill 1217)** module (iter 643) which addresses statewide rent stabilization specifically ([Washington State Legislature — Chapter 59.18 RCW Full Text](https://app.leg.wa.gov/rcw/default.aspx?cite=59.18&full=true); [Justia — Revised Code of Washington Title 59 Chapter 59.18 (2025)](https://law.justia.com/codes/washington/title-59/chapter-59-18/); [WA Law — 59.18 Residential Landlord-Tenant Act](https://wa-law.org/bill/2021-22/hb/1236/1/rcw/59_landlord_and_tenant/59.18_residential_landlord-tenant_act.html); [Rentec Direct — Washington State Landlord-Tenant Laws Resource Guide](https://www.rentecdirect.com/blog/washington-landlord-tenant-laws/); [Hemlane — Washington Security Deposit Laws in 2026](https://www.hemlane.com/resources/washington-security-deposit-laws/); [Tenants Union — Eviction Process](https://tenantsunion.org/rights/eviction-process); [Washington Law Help — Eviction Notices](https://www.washingtonlawhelp.org/en/eviction-notices); [Aberdeen WA Tenants' Rights Manual PDF July 2023](https://www.aberdeenwa.gov/DocumentCenter/View/1879/Tenants-Rights-Manual-PDF); [LawFiles WA — Chapter 59.18 RCW Sections PDF](https://lawfilesext.leg.wa.gov/law/RCWArchive/2024/pdf/RCW%20%2059%20%20TITLE/RCW%20%2059%20.%2018%20%20CHAPTER/RCW%20%2059%20.%2018%20%20CHAPTER.pdf)).
//! - **RCW 59.18.060 Landlord — Duties (Habitability)**: landlord must (i) maintain the premises to substantially comply with **APPLICABLE STATE AND LOCAL CODES**; (ii) provide a reasonable program for the **CONTROL OF INFESTATION** by insects, rodents, and other pests at the inception of the tenancy; (iii) maintain **STRUCTURAL COMPONENTS** including roofs, walls, foundations, and floors in a **REASONABLY WEATHERTIGHT CONDITION**; (iv) maintain all **ELECTRICAL, PLUMBING, HEATING, and other facilities and appliances** supplied or required to be supplied; (v) provide **HOT AND COLD WATER**; (vi) provide adequate **HEAT** and the appropriate facilities to maintain reasonable temperatures.
//! - **RCW 59.18.063 Landlord — Written Receipts for Payments Made by Tenant**: a landlord shall provide, **UPON THE REQUEST OF A TENANT**, a **WRITTEN RECEIPT** for any payments made by the tenant; broader than mandatory-receipt regimes (e.g., NY) — Washington's regime is request-based.
//! - **RCW 59.18.260 Security Deposit — Written Rental Agreement + Written Checklist Required**: if moneys are paid to the landlord by the tenant as a deposit or as security, the rental agreement **MUST BE IN WRITING** and must specify the **TERMS AND CONDITIONS UNDER WHICH THE DEPOSIT MAY BE WITHHELD**; landlord must provide a **WRITTEN CHECKLIST** documenting the condition of the rental unit **BEFORE COLLECTING A SECURITY DEPOSIT** and signed by both parties; failure to comply with the written agreement + written checklist requirements means the deposit is **NOT ENFORCEABLE** against the tenant.
//! - **RCW 59.18.280 Security Deposit — 30-Day Itemized Statement + Documentation**: within **30 DAYS** after termination of the rental agreement and vacation of the premises (or within 30 days after landlord learns of abandonment), landlord must give a **FULL AND SPECIFIC STATEMENT OF THE BASIS FOR RETAINING** any of the deposit, **TOGETHER WITH ANY DOCUMENTATION REQUIRED**, together with payment of any refund due the tenant.
//! - **RCW 59.18.280 Intentional Refusal Penalty — UP TO 2x Deposit**: court may award **UP TO TWO TIMES THE AMOUNT OF THE DEPOSIT** for the **INTENTIONAL REFUSAL** of the landlord to give the statement, documentation, or refund due, unless the landlord shows that **CIRCUMSTANCES BEYOND THE LANDLORD'S CONTROL** prevented compliance within 30 days.
//! - **RCW 59.18.057 14-Day Pay-or-Quit Notice for Nonpayment**: Washington requires **14 DAYS** written notice before eviction can be filed for nonpayment of rent; original 3-day notice extended to 14 days by **SB 5600 (Laws of 2019, Chapter 23)**, **EFFECTIVE JULY 28, 2019**.
//! - **RCW 59.18.057 Rent Only — No Late Fees / Attorney Fees / Damages**: the 14-day pay-or-quit notice may demand **ONLY RENT**; **LATE FEES**, attorney fees, damages, or other **NON-RENT CHARGES** are NOT included and **CANNOT BE THE BASIS** for an unlawful detainer action for nonpayment — Washington practitioner trap.
//! - **RCW 59.18.100 Tenant Repair-and-Deduct Remedy**: tenant may make repairs themselves and **DEDUCT THE COST FROM RENT** under specified conditions when landlord fails to complete necessary repairs after **WRITTEN NOTICE + REASONABLE TIME**; statutory cap on deductible amount depends on whether tenant used a licensed contractor.
//! - **RCW 59.18.650 Just-Cause Eviction — SB 5160 (2021)**: Washington enacted **LANDMARK JUST-CAUSE / EVICTION REFORM** by **Substitute Senate Bill 5160 (Laws of 2021, Chapter 115)**, effective May 10, 2021; ended at-will month-to-month terminations and required landlords to assert a **STATUTORILY ENUMERATED CAUSE** for any eviction or non-renewal; lists **16+ ENUMERATED CAUSES** including nonpayment, lease violations after notice to cure, owner move-in, substantial rehab, demolition, condo conversion, withdrawal from rental market.
//! - **RCW 59.18.270 Moneys Paid for Deposit — Statement of Amount and Specified Account**: landlord must give the tenant a **WRITTEN RECEIPT** for any deposit + must specify the **TRUST ACCOUNT or BANK** where the deposit is held; failure to comply means the deposit must be **REFUNDED IN FULL**.
//! - **Two-Times Damages Cap**: practitioners note that the RCW 59.18.280 intentional-refusal penalty is **UP TO 2x deposit damages** — discretionary court award, not automatic doubling.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const WA_RLTA_CHAPTER_NUMBER: u32 = 59;
pub const WA_RLTA_SECTION_NUMBER: u32 = 18;
pub const WA_RLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const WA_RLTA_PAY_OR_QUIT_DEMAND_DAYS: u32 = 14;
pub const WA_RLTA_INTENTIONAL_REFUSAL_MAX_MULTIPLIER: u32 = 2;
pub const WA_RLTA_SB_5600_ENACTMENT_YEAR: u32 = 2019;
pub const WA_RLTA_SB_5160_ENACTMENT_YEAR: u32 = 2021;
pub const WA_RLTA_ORIGINAL_ENACTMENT_YEAR: u32 = 1973;
pub const WA_RLTA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromRcw5918,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WrittenAgreementAndChecklistStatus {
    WrittenAgreementAndChecklistProvidedAndSigned,
    WrittenAgreementOrChecklistMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositReturnRefusalStatus {
    GoodFaithDelayBeyondLandlordsControl,
    IntentionalRefusalWithoutJustification,
    LandlordCompliedOnTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayOrQuitNoticeContent {
    OnlyRentDemanded,
    LateFeesOrNonRentChargesDemandedInPayOrQuitNotice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    LandlordHabitabilityDutiesUnderRcw5918060,
    WrittenReceiptForPaymentsUnderRcw5918063,
    WrittenAgreementAndChecklistUnderRcw5918260,
    SecurityDepositReturnUnderRcw5918280,
    PayOrQuitNoticeUnderRcw5918057,
    PayOrQuitNoticeContentRentOnlyUnderRcw5918057,
    TenantRepairAndDeductRemedyUnderRcw5918100,
    JustCauseEvictionUnderRcw5918650,
    DepositTrustAccountReceiptUnderRcw5918270,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WaRltaMode {
    NotApplicableTenancyExemptFromRcw5918,
    CompliantLandlordMaintainsHabitabilityUnderRcw5918060,
    CompliantWrittenReceiptProvidedUponRequest,
    CompliantWrittenAgreementAndChecklistProvidedAndSigned,
    CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days,
    CompliantFourteenDayPayOrQuitNoticeProperlyServed,
    CompliantPayOrQuitNoticeDemandsOnlyRent,
    CompliantTenantRepairAndDeductRemedyExecuted,
    CompliantJustCauseEvictionGroundAsserted,
    CompliantDepositTrustAccountReceiptProvided,
    ViolationLandlordFailedHabitabilityDuties,
    ViolationWrittenReceiptNotProvidedDespiteRequest,
    ViolationWrittenAgreementOrChecklistMissingDepositUnenforceable,
    ViolationSecurityDepositReturnedPast30DayDeadlineWithIntentionalRefusalPenalty,
    ViolationSecurityDepositReturnedPast30DayDeadlineWithGoodFaithDelay,
    ViolationPayOrQuitNoticeShorterThan14Days,
    ViolationPayOrQuitNoticeDemandsLateFeesOrNonRentCharges,
    ViolationTenantRepairAndDeductWithoutWrittenNoticeOrReasonableTime,
    ViolationEvictionWithoutStatutoryJustCauseGround,
    ViolationDepositCollectedWithoutTrustAccountReceiptOrSpecifiedAccount,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub written_agreement_and_checklist_status: WrittenAgreementAndChecklistStatus,
    pub deposit_return_refusal_status: DepositReturnRefusalStatus,
    pub pay_or_quit_notice_content: PayOrQuitNoticeContent,
    pub compliance_aspect: ComplianceAspect,
    pub landlord_maintains_habitability_per_rcw_5918060: bool,
    pub written_receipt_provided_upon_request: bool,
    pub days_to_return_deposit: u32,
    pub pay_or_quit_notice_days_given: u32,
    pub tenant_gave_written_notice_and_reasonable_time_before_repair_and_deduct: bool,
    pub eviction_asserts_statutory_just_cause_ground: bool,
    pub deposit_trust_account_receipt_provided: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: WaRltaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type WaRltaInput = Input;
pub type WaRltaOutput = Output;
pub type WaRltaResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Washington Residential Landlord-Tenant Act (RLTA) — RCW Chapter 59.18; originally enacted by Laws of 1973, 1st Ex. Session, Chapter 207; substantially amended over five decades, most consequentially by SB 5600 (2019 — 14-day pay-or-quit) and SB 5160 (2021 — just-cause eviction reform)".to_string(),
        "RCW 59.18.060 Landlord Duties (Habitability) — landlord must (i) maintain premises to substantially comply with APPLICABLE STATE AND LOCAL CODES; (ii) provide a reasonable program for the CONTROL OF INFESTATION; (iii) maintain STRUCTURAL COMPONENTS in a REASONABLY WEATHERTIGHT CONDITION; (iv) maintain all ELECTRICAL, PLUMBING, HEATING, and other facilities and appliances; (v) provide HOT AND COLD WATER; (vi) provide adequate HEAT and the appropriate facilities to maintain reasonable temperatures".to_string(),
        "RCW 59.18.063 Landlord — Written Receipts for Payments — a landlord shall provide, UPON THE REQUEST OF A TENANT, a WRITTEN RECEIPT for any payments made by the tenant".to_string(),
        "RCW 59.18.260 Security Deposit — Written Rental Agreement + Written Checklist Required — if moneys are paid as a deposit, the rental agreement MUST BE IN WRITING and must specify the TERMS AND CONDITIONS UNDER WHICH THE DEPOSIT MAY BE WITHHELD; landlord must provide a WRITTEN CHECKLIST documenting the condition of the rental unit BEFORE COLLECTING A SECURITY DEPOSIT and signed by both parties; failure to comply means the deposit is NOT ENFORCEABLE".to_string(),
        "RCW 59.18.270 Moneys Paid for Deposit — Statement of Amount and Specified Account — landlord must give the tenant a WRITTEN RECEIPT for any deposit + must specify the TRUST ACCOUNT or BANK where the deposit is held; failure to comply means the deposit must be REFUNDED IN FULL".to_string(),
        "RCW 59.18.280 Security Deposit — 30-Day Itemized Statement + Documentation — within 30 DAYS after termination of the rental agreement and vacation of the premises (or within 30 days after landlord learns of abandonment), landlord must give a FULL AND SPECIFIC STATEMENT OF THE BASIS FOR RETAINING any of the deposit, TOGETHER WITH ANY DOCUMENTATION REQUIRED, together with payment of any refund due".to_string(),
        "RCW 59.18.280 Intentional Refusal Penalty — UP TO TWO TIMES THE AMOUNT OF THE DEPOSIT — court may award up to 2x deposit for INTENTIONAL REFUSAL of the landlord to give the statement, documentation, or refund, unless landlord shows CIRCUMSTANCES BEYOND THE LANDLORD'S CONTROL prevented compliance within 30 days".to_string(),
        "RCW 59.18.057 14-Day Pay-or-Quit Notice for Nonpayment — Washington requires 14 DAYS written notice before eviction can be filed for nonpayment of rent; original 3-day notice extended to 14 days by SB 5600 (Laws of 2019, Chapter 23), EFFECTIVE JULY 28, 2019".to_string(),
        "RCW 59.18.057 Rent Only — No Late Fees / Attorney Fees / Damages — the 14-day pay-or-quit notice may demand ONLY RENT; LATE FEES, attorney fees, damages, or other NON-RENT CHARGES are NOT included and CANNOT BE THE BASIS for an unlawful detainer action for nonpayment".to_string(),
        "RCW 59.18.100 Tenant Repair-and-Deduct Remedy — tenant may make repairs themselves and DEDUCT THE COST FROM RENT under specified conditions when landlord fails to complete necessary repairs after WRITTEN NOTICE + REASONABLE TIME; statutory cap on deductible amount depends on whether tenant used a licensed contractor".to_string(),
        "RCW 59.18.650 Just-Cause Eviction (SB 5160 of 2021) — Washington enacted LANDMARK JUST-CAUSE / EVICTION REFORM by Substitute Senate Bill 5160 (Laws of 2021, Chapter 115), effective May 10, 2021; ended at-will month-to-month terminations; required landlords to assert a STATUTORILY ENUMERATED CAUSE for any eviction or non-renewal; lists 16+ ENUMERATED CAUSES including nonpayment, lease violations after notice to cure, owner move-in, substantial rehab, demolition, condo conversion, withdrawal from rental market".to_string(),
        "Washington State Legislature + Justia + WA Law + Rentec Direct + Hemlane + Tenants Union + Washington Law Help + Aberdeen WA Tenants Rights Manual + LawFiles WA — practitioner overviews of RCW 59.18".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromRcw5918 {
        return Output {
            mode: WaRltaMode::NotApplicableTenancyExemptFromRcw5918,
            statutory_basis: "RCW 59.18.040 — tenancy exempt from RCW Chapter 59.18 coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Washington Residential Landlord-Tenant Act under RCW 59.18.040; RLTA landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::LandlordHabitabilityDutiesUnderRcw5918060 => {
            if input.landlord_maintains_habitability_per_rcw_5918060 {
                Output {
                    mode: WaRltaMode::CompliantLandlordMaintainsHabitabilityUnderRcw5918060,
                    statutory_basis: "RCW 59.18.060 — landlord maintains habitability per six statutory duty categories".to_string(),
                    notes: "COMPLIANT: landlord maintains state/local code compliance + pest control program + weathertight structural components + electrical/plumbing/heating maintenance + hot and cold water + adequate heat under RCW 59.18.060.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: WaRltaMode::ViolationLandlordFailedHabitabilityDuties,
                    statutory_basis: "RCW 59.18.060 — landlord failed habitability duty categories".to_string(),
                    notes: "VIOLATION: landlord failed one or more of the six RCW 59.18.060 habitability duty categories.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::WrittenReceiptForPaymentsUnderRcw5918063 => {
            if input.written_receipt_provided_upon_request {
                Output {
                    mode: WaRltaMode::CompliantWrittenReceiptProvidedUponRequest,
                    statutory_basis: "RCW 59.18.063 — landlord provided written receipt upon tenant request".to_string(),
                    notes: "COMPLIANT: landlord provided written receipt for tenant payments upon request under RCW 59.18.063.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: WaRltaMode::ViolationWrittenReceiptNotProvidedDespiteRequest,
                    statutory_basis: "RCW 59.18.063 — landlord failed to provide written receipt despite tenant request".to_string(),
                    notes: "VIOLATION: landlord failed to provide written receipt for tenant payments despite tenant request under RCW 59.18.063.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::WrittenAgreementAndChecklistUnderRcw5918260 => {
            if input.written_agreement_and_checklist_status
                == WrittenAgreementAndChecklistStatus::WrittenAgreementAndChecklistProvidedAndSigned
            {
                Output {
                    mode: WaRltaMode::CompliantWrittenAgreementAndChecklistProvidedAndSigned,
                    statutory_basis: "RCW 59.18.260 — written rental agreement and written checklist provided before collecting security deposit".to_string(),
                    notes: "COMPLIANT: landlord provided written rental agreement specifying deposit retention terms AND written move-in checklist documenting unit condition, both signed by tenant before collecting security deposit, under RCW 59.18.260.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: WaRltaMode::ViolationWrittenAgreementOrChecklistMissingDepositUnenforceable,
                    statutory_basis: "RCW 59.18.260 — deposit unenforceable without written agreement and written checklist".to_string(),
                    notes: "VIOLATION: written rental agreement and/or written move-in checklist missing under RCW 59.18.260; security deposit NOT ENFORCEABLE against the tenant.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnUnderRcw5918280 => {
            if input.days_to_return_deposit <= WA_RLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS {
                Output {
                    mode: WaRltaMode::CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days,
                    statutory_basis: "RCW 59.18.280 — security deposit returned with full and specific statement and documentation within 30 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized statement + documentation within {d} days (statutory deadline is 30 days under RCW 59.18.280).",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                match input.deposit_return_refusal_status {
                    DepositReturnRefusalStatus::IntentionalRefusalWithoutJustification => Output {
                        mode: WaRltaMode::ViolationSecurityDepositReturnedPast30DayDeadlineWithIntentionalRefusalPenalty,
                        statutory_basis: "RCW 59.18.280 — intentional refusal to provide statement/documentation/refund triggers up to 2x deposit penalty".to_string(),
                        notes: format!(
                            "VIOLATION: deposit returned at {d} days exceeds 30-day statutory deadline under RCW 59.18.280; intentional refusal without justification — court may award UP TO 2x deposit amount as damages.",
                            d = input.days_to_return_deposit,
                        ),
                        citations,
                    },
                    DepositReturnRefusalStatus::GoodFaithDelayBeyondLandlordsControl => Output {
                        mode: WaRltaMode::ViolationSecurityDepositReturnedPast30DayDeadlineWithGoodFaithDelay,
                        statutory_basis: "RCW 59.18.280 — good-faith delay beyond landlord's control mitigates penalty exposure".to_string(),
                        notes: format!(
                            "VIOLATION: deposit returned at {d} days exceeds 30-day statutory deadline; landlord may show circumstances beyond landlord's control to avoid up-to-2x penalty under RCW 59.18.280.",
                            d = input.days_to_return_deposit,
                        ),
                        citations,
                    },
                    DepositReturnRefusalStatus::LandlordCompliedOnTime => Output {
                        mode: WaRltaMode::CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days,
                        statutory_basis: "RCW 59.18.280 — landlord complied on time despite reported elapsed days".to_string(),
                        notes: "COMPLIANT: landlord complied on time under RCW 59.18.280.".to_string(),
                        citations,
                    },
                }
            }
        }
        ComplianceAspect::PayOrQuitNoticeUnderRcw5918057 => {
            if input.pay_or_quit_notice_days_given >= WA_RLTA_PAY_OR_QUIT_DEMAND_DAYS {
                Output {
                    mode: WaRltaMode::CompliantFourteenDayPayOrQuitNoticeProperlyServed,
                    statutory_basis: "RCW 59.18.057 — 14-day pay-or-quit notice for nonpayment properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day pay-or-quit notice satisfies 14-day statutory minimum under RCW 59.18.057 (SB 5600 of 2019 extended from prior 3-day notice).",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: WaRltaMode::ViolationPayOrQuitNoticeShorterThan14Days,
                    statutory_basis: "RCW 59.18.057 — pay-or-quit notice shorter than statutory 14-day minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day pay-or-quit notice shorter than 14-day statutory minimum under RCW 59.18.057.",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::PayOrQuitNoticeContentRentOnlyUnderRcw5918057 => {
            match input.pay_or_quit_notice_content {
                PayOrQuitNoticeContent::OnlyRentDemanded => Output {
                    mode: WaRltaMode::CompliantPayOrQuitNoticeDemandsOnlyRent,
                    statutory_basis: "RCW 59.18.057 — pay-or-quit notice demands only rent (no late fees / attorney fees / non-rent charges)".to_string(),
                    notes: "COMPLIANT: pay-or-quit notice demands only rent under RCW 59.18.057; no late fees, attorney fees, damages, or non-rent charges included.".to_string(),
                    citations,
                },
                PayOrQuitNoticeContent::LateFeesOrNonRentChargesDemandedInPayOrQuitNotice => {
                    Output {
                        mode: WaRltaMode::ViolationPayOrQuitNoticeDemandsLateFeesOrNonRentCharges,
                        statutory_basis: "RCW 59.18.057 — pay-or-quit notice may not demand late fees / attorney fees / damages or non-rent charges".to_string(),
                        notes: "VIOLATION: pay-or-quit notice demanded late fees or non-rent charges in addition to rent under RCW 59.18.057; non-rent charges CANNOT be the basis for an unlawful detainer action for nonpayment.".to_string(),
                        citations,
                    }
                }
            }
        }
        ComplianceAspect::TenantRepairAndDeductRemedyUnderRcw5918100 => {
            if input.tenant_gave_written_notice_and_reasonable_time_before_repair_and_deduct {
                Output {
                    mode: WaRltaMode::CompliantTenantRepairAndDeductRemedyExecuted,
                    statutory_basis: "RCW 59.18.100 — tenant gave landlord written notice and reasonable time before repair-and-deduct remedy".to_string(),
                    notes: "COMPLIANT: tenant satisfied RCW 59.18.100 prerequisites (written notice + reasonable time for landlord to repair) before executing repair-and-deduct remedy.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: WaRltaMode::ViolationTenantRepairAndDeductWithoutWrittenNoticeOrReasonableTime,
                    statutory_basis: "RCW 59.18.100 — repair-and-deduct remedy requires written notice + reasonable time prerequisites".to_string(),
                    notes: "VIOLATION: tenant executed repair-and-deduct remedy without satisfying RCW 59.18.100 prerequisites (written notice of repair need + reasonable time for landlord to repair).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::JustCauseEvictionUnderRcw5918650 => {
            if input.eviction_asserts_statutory_just_cause_ground {
                Output {
                    mode: WaRltaMode::CompliantJustCauseEvictionGroundAsserted,
                    statutory_basis: "RCW 59.18.650 — eviction asserts statutorily enumerated just-cause ground under SB 5160 of 2021".to_string(),
                    notes: "COMPLIANT: landlord asserts one of the 16+ statutorily enumerated just-cause grounds under RCW 59.18.650 (SB 5160 of 2021).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: WaRltaMode::ViolationEvictionWithoutStatutoryJustCauseGround,
                    statutory_basis: "RCW 59.18.650 — eviction without statutory just-cause ground prohibited under SB 5160 of 2021".to_string(),
                    notes: "VIOLATION: landlord attempted eviction or non-renewal without asserting one of the 16+ statutorily enumerated just-cause grounds under RCW 59.18.650 (SB 5160 of 2021); at-will month-to-month termination prohibited.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::DepositTrustAccountReceiptUnderRcw5918270 => {
            if input.deposit_trust_account_receipt_provided {
                Output {
                    mode: WaRltaMode::CompliantDepositTrustAccountReceiptProvided,
                    statutory_basis: "RCW 59.18.270 — written receipt for deposit + specified trust account or bank provided".to_string(),
                    notes: "COMPLIANT: landlord provided written receipt for deposit + specified trust account or bank where deposit is held under RCW 59.18.270.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: WaRltaMode::ViolationDepositCollectedWithoutTrustAccountReceiptOrSpecifiedAccount,
                    statutory_basis: "RCW 59.18.270 — deposit collected without required written receipt and specified trust account or bank".to_string(),
                    notes: "VIOLATION: landlord collected deposit without providing the required written receipt + specifying the trust account or bank under RCW 59.18.270; deposit must be REFUNDED IN FULL.".to_string(),
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
            written_agreement_and_checklist_status:
                WrittenAgreementAndChecklistStatus::WrittenAgreementAndChecklistProvidedAndSigned,
            deposit_return_refusal_status: DepositReturnRefusalStatus::LandlordCompliedOnTime,
            pay_or_quit_notice_content: PayOrQuitNoticeContent::OnlyRentDemanded,
            compliance_aspect: ComplianceAspect::LandlordHabitabilityDutiesUnderRcw5918060,
            landlord_maintains_habitability_per_rcw_5918060: true,
            written_receipt_provided_upon_request: true,
            days_to_return_deposit: 25,
            pay_or_quit_notice_days_given: 14,
            tenant_gave_written_notice_and_reasonable_time_before_repair_and_deduct: true,
            eviction_asserts_statutory_just_cause_ground: true,
            deposit_trust_account_receipt_provided: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromRcw5918;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::NotApplicableTenancyExemptFromRcw5918);
    }

    #[test]
    fn landlord_maintains_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityDutiesUnderRcw5918060;
        input.landlord_maintains_habitability_per_rcw_5918060 = true;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::CompliantLandlordMaintainsHabitabilityUnderRcw5918060);
    }

    #[test]
    fn landlord_failed_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityDutiesUnderRcw5918060;
        input.landlord_maintains_habitability_per_rcw_5918060 = false;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::ViolationLandlordFailedHabitabilityDuties);
    }

    #[test]
    fn written_receipt_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrittenReceiptForPaymentsUnderRcw5918063;
        input.written_receipt_provided_upon_request = true;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::CompliantWrittenReceiptProvidedUponRequest);
    }

    #[test]
    fn written_receipt_not_provided_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrittenReceiptForPaymentsUnderRcw5918063;
        input.written_receipt_provided_upon_request = false;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::ViolationWrittenReceiptNotProvidedDespiteRequest);
    }

    #[test]
    fn written_agreement_and_checklist_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrittenAgreementAndChecklistUnderRcw5918260;
        input.written_agreement_and_checklist_status =
            WrittenAgreementAndChecklistStatus::WrittenAgreementAndChecklistProvidedAndSigned;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::CompliantWrittenAgreementAndChecklistProvidedAndSigned
        );
    }

    #[test]
    fn written_agreement_or_checklist_missing_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrittenAgreementAndChecklistUnderRcw5918260;
        input.written_agreement_and_checklist_status =
            WrittenAgreementAndChecklistStatus::WrittenAgreementOrChecklistMissing;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::ViolationWrittenAgreementOrChecklistMissingDepositUnenforceable
        );
    }

    #[test]
    fn deposit_returned_within_30_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderRcw5918280;
        input.days_to_return_deposit = 25;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days
        );
    }

    #[test]
    fn deposit_returned_at_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderRcw5918280;
        input.days_to_return_deposit = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days
        );
    }

    #[test]
    fn deposit_returned_at_31_days_with_intentional_refusal_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderRcw5918280;
        input.days_to_return_deposit = 31;
        input.deposit_return_refusal_status =
            DepositReturnRefusalStatus::IntentionalRefusalWithoutJustification;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::ViolationSecurityDepositReturnedPast30DayDeadlineWithIntentionalRefusalPenalty
        );
    }

    #[test]
    fn deposit_returned_at_31_days_with_good_faith_delay_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderRcw5918280;
        input.days_to_return_deposit = 31;
        input.deposit_return_refusal_status =
            DepositReturnRefusalStatus::GoodFaithDelayBeyondLandlordsControl;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::ViolationSecurityDepositReturnedPast30DayDeadlineWithGoodFaithDelay
        );
    }

    #[test]
    fn pay_or_quit_at_14_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PayOrQuitNoticeUnderRcw5918057;
        input.pay_or_quit_notice_days_given = 14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::CompliantFourteenDayPayOrQuitNoticeProperlyServed
        );
    }

    #[test]
    fn pay_or_quit_at_13_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PayOrQuitNoticeUnderRcw5918057;
        input.pay_or_quit_notice_days_given = 13;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::ViolationPayOrQuitNoticeShorterThan14Days);
    }

    #[test]
    fn pay_or_quit_notice_demands_only_rent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PayOrQuitNoticeContentRentOnlyUnderRcw5918057;
        input.pay_or_quit_notice_content = PayOrQuitNoticeContent::OnlyRentDemanded;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::CompliantPayOrQuitNoticeDemandsOnlyRent);
    }

    #[test]
    fn pay_or_quit_notice_demands_late_fees_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PayOrQuitNoticeContentRentOnlyUnderRcw5918057;
        input.pay_or_quit_notice_content =
            PayOrQuitNoticeContent::LateFeesOrNonRentChargesDemandedInPayOrQuitNotice;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::ViolationPayOrQuitNoticeDemandsLateFeesOrNonRentCharges
        );
    }

    #[test]
    fn tenant_repair_and_deduct_with_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantRepairAndDeductRemedyUnderRcw5918100;
        input.tenant_gave_written_notice_and_reasonable_time_before_repair_and_deduct = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::CompliantTenantRepairAndDeductRemedyExecuted
        );
    }

    #[test]
    fn tenant_repair_and_deduct_without_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantRepairAndDeductRemedyUnderRcw5918100;
        input.tenant_gave_written_notice_and_reasonable_time_before_repair_and_deduct = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::ViolationTenantRepairAndDeductWithoutWrittenNoticeOrReasonableTime
        );
    }

    #[test]
    fn just_cause_eviction_ground_asserted_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEvictionUnderRcw5918650;
        input.eviction_asserts_statutory_just_cause_ground = true;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::CompliantJustCauseEvictionGroundAsserted);
    }

    #[test]
    fn eviction_without_just_cause_ground_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEvictionUnderRcw5918650;
        input.eviction_asserts_statutory_just_cause_ground = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::ViolationEvictionWithoutStatutoryJustCauseGround
        );
    }

    #[test]
    fn deposit_trust_account_receipt_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositTrustAccountReceiptUnderRcw5918270;
        input.deposit_trust_account_receipt_provided = true;
        let out = check(&input);
        assert_eq!(out.mode, WaRltaMode::CompliantDepositTrustAccountReceiptProvided);
    }

    #[test]
    fn deposit_without_trust_account_receipt_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositTrustAccountReceiptUnderRcw5918270;
        input.deposit_trust_account_receipt_provided = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WaRltaMode::ViolationDepositCollectedWithoutTrustAccountReceiptOrSpecifiedAccount
        );
    }

    #[test]
    fn constants_pin_washington_rlta_statutory_thresholds() {
        assert_eq!(WA_RLTA_CHAPTER_NUMBER, 59);
        assert_eq!(WA_RLTA_SECTION_NUMBER, 18);
        assert_eq!(WA_RLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(WA_RLTA_PAY_OR_QUIT_DEMAND_DAYS, 14);
        assert_eq!(WA_RLTA_INTENTIONAL_REFUSAL_MAX_MULTIPLIER, 2);
        assert_eq!(WA_RLTA_SB_5600_ENACTMENT_YEAR, 2019);
        assert_eq!(WA_RLTA_SB_5160_ENACTMENT_YEAR, 2021);
        assert_eq!(WA_RLTA_ORIGINAL_ENACTMENT_YEAR, 1973);
        assert_eq!(WA_RLTA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_washington_rlta_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Washington Residential Landlord-Tenant Act"));
        assert!(joined.contains("RCW Chapter 59.18"));
        assert!(joined.contains("Laws of 1973"));
        assert!(joined.contains("SB 5600"));
        assert!(joined.contains("SB 5160"));
        assert!(joined.contains("RCW 59.18.060"));
        assert!(joined.contains("APPLICABLE STATE AND LOCAL CODES"));
        assert!(joined.contains("CONTROL OF INFESTATION"));
        assert!(joined.contains("REASONABLY WEATHERTIGHT CONDITION"));
        assert!(joined.contains("HOT AND COLD WATER"));
        assert!(joined.contains("RCW 59.18.063"));
        assert!(joined.contains("WRITTEN RECEIPT"));
        assert!(joined.contains("UPON THE REQUEST OF A TENANT"));
        assert!(joined.contains("RCW 59.18.260"));
        assert!(joined.contains("WRITTEN CHECKLIST"));
        assert!(joined.contains("BEFORE COLLECTING A SECURITY DEPOSIT"));
        assert!(joined.contains("NOT ENFORCEABLE"));
        assert!(joined.contains("RCW 59.18.270"));
        assert!(joined.contains("TRUST ACCOUNT"));
        assert!(joined.contains("REFUNDED IN FULL"));
        assert!(joined.contains("RCW 59.18.280"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("FULL AND SPECIFIC STATEMENT OF THE BASIS FOR RETAINING"));
        assert!(joined.contains("TWO TIMES THE AMOUNT OF THE DEPOSIT"));
        assert!(joined.contains("INTENTIONAL REFUSAL"));
        assert!(joined.contains("CIRCUMSTANCES BEYOND THE LANDLORD'S CONTROL"));
        assert!(joined.contains("RCW 59.18.057"));
        assert!(joined.contains("14 DAYS"));
        assert!(joined.contains("ONLY RENT"));
        assert!(joined.contains("LATE FEES"));
        assert!(joined.contains("CANNOT BE THE BASIS"));
        assert!(joined.contains("JULY 28, 2019"));
        assert!(joined.contains("RCW 59.18.100"));
        assert!(joined.contains("DEDUCT THE COST FROM RENT"));
        assert!(joined.contains("WRITTEN NOTICE + REASONABLE TIME"));
        assert!(joined.contains("RCW 59.18.650"));
        assert!(joined.contains("LANDMARK JUST-CAUSE / EVICTION REFORM"));
        assert!(joined.contains("Substitute Senate Bill 5160"));
        assert!(joined.contains("STATUTORILY ENUMERATED CAUSE"));
        assert!(joined.contains("16+ ENUMERATED CAUSES"));
    }
}
