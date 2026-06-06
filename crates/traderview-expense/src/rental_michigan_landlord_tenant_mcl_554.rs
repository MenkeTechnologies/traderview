//! Michigan Landlord-Tenant Law (MCL §§ 554.601-554.641)
//! Compliance Module — pure-compute check for landlord
//! statutory compliance with Michigan's two-act
//! landlord-tenant regime: the **Security Deposit Act**
//! (Public Act 348 of 1972; MCL §§ 554.601-554.616) and
//! the **Truth in Renting Act** (Public Act 454 of 1978;
//! MCL §§ 554.631-554.641). Michigan operates a non-URLTA
//! hybrid regime with these two separately codified acts.
//!
//! **Distinctive Michigan features**: **1.5 MONTHS' RENT
//! STATUTORY CAP** on security deposits under MCL 554.602;
//! **14-DAY landlord notice** of bank name and address +
//! move-in inventory checklist required at tenant move-in;
//! **30-DAY itemized damages list** + return of unwithheld
//! portion under MCL 554.609; **7-DAY tenant response
//! right** to dispute itemized damages under MCL 554.610;
//! **TRUTH IN RENTING ACT prohibited clauses** (MCL
//! 554.633) — confession-of-judgment, habitability-waiver,
//! exculpatory, jury-waiver, and personal-property
//! security-interest clauses are **VOID**.
//!
//! Web research (verified 2026-06-03):
//! - **Security Deposit Act (Public Act 348 of 1972; MCL §§ 554.601-554.616)**: Michigan's foundational deposit regime; deposit governed by separate act distinct from Truth in Renting Act ([Michigan Legislature — MCL § 554.601](https://www.legislature.mi.gov/Laws/MCL?objectName=mcl-554-601); [Justia — 2025 Michigan Compiled Laws Chapter 554 Act 348 of 1972](https://law.justia.com/codes/michigan/chapter-554/statute-act-348-of-1972/); [Justia — MCL § 554.602 Security Deposit Amount](https://law.justia.com/codes/michigan/chapter-554/statute-act-348-of-1972/section-554-602/); [Michigan Courts — Chapter 2: Specific Landlord-Tenant Laws](https://www.courts.michigan.gov/4a4eb6/siteassets/publications/benchbooks/lltbb/lltbbresponsivehtml5.zip/LLTBB/Ch_2_Specific_Acts/Chapter_2__58__Specific_Landlord-Tenant_Laws.htm); [Tilchin & Hall PC — Almost Everything You Need to Know About Residential Security Deposits in Michigan](https://www.tilchinhall.com/blog/2021/03/almost-everything-you-need-to-know-about-residential-security-deposits-in-michigan/); [Landlord Studio — Michigan Security Deposit Laws](https://www.landlordstudio.com/landlord-tenant-laws/michigan-security-deposit-laws); [Rentable — Michigan Security Deposit Laws Complete Guide](https://www.rentable.com/blog/michigan-security-deposit-laws-a-complete-guide-for-landlords-tenants/); [Tenant Rights — Michigan Tenant Security Deposit Rules & Deadlines](https://tenant-rights.com/michigan/michigan-tenant-security-deposit-rules-deadlines); [Kershaw Vititoe & Jedinak PLC — Security Deposits in Michigan An Overview](https://www.monroecountylawyers.com/blog/2018/09/security-deposits-in-michigan-an-overview/); [Michigan Legislature — MCL § 554.607](https://www.legislature.mi.gov/Laws/MCL?objectName=mcl-554-607)).
//! - **MCL § 554.602 Statutory 1.5-Month Security Deposit Cap**: a security deposit shall **NOT EXCEED 1 1/2 MONTHS' RENT** — Michigan is one of approximately 25 states with a statutory cap (cf. IN no cap; FL 2 months; NY/CA/MA 1 month).
//! - **MCL § 554.603 14-Day Landlord Notice of Bank**: within **14 DAYS** of the tenant taking possession, the landlord must provide a **WRITTEN NOTICE OF THE BANK NAME AND ADDRESS** where the deposit is held, along with a **DETAILED MOVE-IN INVENTORY CHECKLIST**.
//! - **MCL § 554.604 Deposit Held in Regulated Financial Institution**: deposits must be held in a **REGULATED FINANCIAL INSTITUTION** and must NOT be commingled with the landlord's personal funds; alternatively, landlord may secure a **LANDLORD TENANT SECURITY DEPOSIT BOND** and comply with annual certification requirements with the **Michigan Department of Attorney General**.
//! - **MCL § 554.609 30-Day Itemized Damages List + Return**: landlord must mail to the tenant an **ITEMIZED LIST OF DAMAGES** within **30 DAYS** of the tenant ending occupancy, together with payment of any portion of the deposit not withheld.
//! - **MCL § 554.610 7-Day Tenant Response to Dispute**: tenant has **7 DAYS** from receipt of the notice of damages to notify the landlord **IN WRITING** of any disagreement with the itemized list; failure of tenant to respond in writing within 7 days **forfeits the tenant's ability to dispute** the deductions in subsequent litigation.
//! - **MCL § 554.612 Landlord Must File Suit to Retain**: if tenant disputes within 7 days, landlord must commence a **SMALL CLAIMS or DISTRICT COURT** action within **45 DAYS** of tenant's response to enforce the damages claim; failure to file within 45 days **WAIVES** the landlord's right to retain.
//! - **Truth in Renting Act (Public Act 454 of 1978; MCL §§ 554.631-554.641)**: separately codified act regulating residential lease form and content ([Michigan Legislature — Truth in Renting Act PDF](https://www.legislature.mi.gov/documents/mcl/pdf/mcl-act-454-of-1978.pdf); [Michigan Legislature — MCL Act 454 of 1978](https://www.legislature.mi.gov/Laws/MCL?objectName=mcl-Act-454-of-1978); [Michigan Legislature — MCL § 554.631](https://www.legislature.mi.gov/Laws/MCL?objectName=mcl-554-631); [Justia — 2025 Michigan Compiled Laws Chapter 554 Act 454 of 1978](https://law.justia.com/codes/michigan/chapter-554/statute-act-454-of-1978/); [Michigan Courts — Truth in Renting Act](https://www.courts.michigan.gov/4a4eb6/siteassets/publications/benchbooks/lltbb/lltbbresponsivehtml5.zip/LLTBB/Ch_2_Specific_Acts/Truth_in_Renting_Act.htm); [Michigan Legislature — MCL 554.633 PDF](https://www.legislature.mi.gov/documents/mcl/pdf/MCL-554-633.pdf); [Michigan Legislature — MCL 554.639 Waiver Prohibited PDF](https://www.legislature.mi.gov/documents/mcl/pdf/MCL-554-639.pdf); [Michigan Legislature — Truth in Renting Act Act 454 of 1978 Archive PDF](https://legislature.mi.gov/documents/mcl/archive/2025/June/mcl-Act-454-of-1978.pdf)).
//! - **MCL § 554.633 Prohibited Lease Clauses — VOID**: certain clauses are **PROHIBITED** from inclusion in written leases, and if such a provision is included in a lease, the provision is **VOID**: (1) **WAIVER OF REMEDY** available for violation of covenants of fitness and habitability; (2) **CONFESSION OF JUDGMENT** clauses; (3) provisions which **EXCULPATE LANDLORDS** from liability imposed by law; (4) clauses that **WAIVE A TENANT'S RIGHT TO TRIAL BY JURY**, to notice, or to any other procedural rights provided by the Summary Proceedings Act, or by the anti-lockout law; (5) clauses which grant the landlord a **SECURITY INTEREST IN ANY PERSONAL PROPERTY OF THE TENANT** to assure payment of rent or other charges.
//! - **MCL § 554.639 Waiver Prohibited**: the requirements of the Truth in Renting Act may **NOT BE WAIVED**; any clause purporting to waive a tenant's rights with regard to summary proceedings for eviction or with regard to illegal detainer, lockout, or interference with possession is **PROHIBITED AND VOID**.
//! - **MCL § 554.634 Disclosure Required**: lessors must **DISCLOSE** to the lessee certain information in the rental agreement, including the address where lessor will accept service of process and notices.
//! - **MCL § 554.635 Sale of Printed Lease Forms**: regulates the **COMMERCIAL SALE** of printed rental agreement forms; commercially sold forms must comply with the Truth in Renting Act.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MI_SECURITY_DEPOSIT_ACT_PUBLIC_ACT_NUMBER: u32 = 348;
pub const MI_SECURITY_DEPOSIT_ACT_ENACTMENT_YEAR: u32 = 1972;
pub const MI_TRUTH_IN_RENTING_ACT_PUBLIC_ACT_NUMBER: u32 = 454;
pub const MI_TRUTH_IN_RENTING_ACT_ENACTMENT_YEAR: u32 = 1978;
pub const MI_SECURITY_DEPOSIT_CAP_TENTHS_OF_MONTHS: u64 = 15;
pub const MI_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS: u64 = 10;
pub const MI_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const MI_TENANT_RESPONSE_DAYS: u32 = 7;
pub const MI_LANDLORD_BANK_NOTICE_DAYS: u32 = 14;
pub const MI_LANDLORD_SUIT_DEADLINE_DAYS_AFTER_TENANT_RESPONSE: u32 = 45;
pub const MI_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromMichiganLandlordTenantActs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositHoldingArrangement {
    HeldInRegulatedFinancialInstitutionSeparateFromLandlordFunds,
    LandlordTenantSecurityDepositBondWithAnnualCertification,
    CommingledWithLandlordPersonalFundsProhibited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraProhibitedClauseStatus {
    NoProhibitedClausesIncluded,
    WaiverOfHabitabilityRemedyIncluded,
    ConfessionOfJudgmentClauseIncluded,
    LandlordExculpationClauseIncluded,
    JuryWaiverOrSummaryProceedingsRightsWaiverIncluded,
    SecurityInterestInTenantPersonalPropertyIncluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapUnderMcl554_602,
    LandlordBankNoticeAndChecklistUnderMcl554_603,
    DepositHoldingArrangementUnderMcl554_604,
    ThirtyDayItemizedDamagesListUnderMcl554_609,
    SevenDayTenantResponseRightUnderMcl554_610,
    FortyFiveDayLandlordSuitDeadlineUnderMcl554_612,
    TraProhibitedClausesUnderMcl554_633,
    TraWaiverProhibitionUnderMcl554_639,
    TraDisclosureRequirementUnderMcl554_634,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MiLandlordTenantMode {
    NotApplicableTenancyExemptFromMichiganLandlordTenantActs,
    CompliantSecurityDepositAtOrBelowOnePointFiveMonthsCap,
    CompliantLandlordBankNoticeAndChecklistDeliveredWithin14Days,
    CompliantDepositInRegulatedFinancialInstitutionOrBonded,
    CompliantThirtyDayItemizedDamagesListMailedWithin30Days,
    CompliantSevenDayTenantResponseRightObserved,
    CompliantLandlordSuitFiledWithin45DaysOfTenantResponse,
    CompliantTraNoProhibitedClausesIncluded,
    CompliantTraWaiverProhibitionObserved,
    CompliantTraDisclosureProvided,
    ViolationSecurityDepositExceedsOnePointFiveMonthsCap,
    ViolationLandlordBankNoticeOrChecklistNotDeliveredWithin14Days,
    ViolationDepositCommingledWithLandlordPersonalFunds,
    ViolationItemizedDamagesListPast30DayDeadline,
    ViolationTenantResponseRightNotObservedOrPastSevenDays,
    ViolationLandlordSuitNotFiledWithin45DaysWaivesRightToRetain,
    ViolationTraProhibitedClauseIncludedClauseVoid,
    ViolationTraWaiverAttempted,
    ViolationTraDisclosureOmitted,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub deposit_holding_arrangement: DepositHoldingArrangement,
    pub tra_prohibited_clause_status: TraProhibitedClauseStatus,
    pub compliance_aspect: ComplianceAspect,
    pub deposit_amount_tenths_of_months_rent: u64,
    pub days_landlord_bank_notice_and_checklist_delivered: u32,
    pub days_to_mail_itemized_damages_list: u32,
    pub days_tenant_responded_after_receiving_damages_list: u32,
    pub days_landlord_filed_suit_after_tenant_response: u32,
    pub tra_waiver_attempted: bool,
    pub tra_disclosure_provided: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: MiLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type MiLandlordTenantInput = Input;
pub type MiLandlordTenantOutput = Output;
pub type MiLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Michigan Landlord-Tenant Law — codified across two separately codified acts: Security Deposit Act (Public Act 348 of 1972, MCL §§ 554.601-554.616) and Truth in Renting Act (Public Act 454 of 1978, MCL §§ 554.631-554.641); Michigan operates a non-URLTA hybrid regime".to_string(),
        "MCL § 554.602 Security Deposit Statutory Cap — a security deposit shall NOT EXCEED 1 1/2 MONTHS' RENT; Michigan is one of approximately 25 states with a statutory cap (cf. IN no cap; FL 2 months; NY/CA/MA 1 month)".to_string(),
        "MCL § 554.603 14-Day Landlord Notice of Bank — within 14 DAYS of the tenant taking possession, the landlord must provide a WRITTEN NOTICE OF THE BANK NAME AND ADDRESS where the deposit is held, along with a DETAILED MOVE-IN INVENTORY CHECKLIST".to_string(),
        "MCL § 554.604 Deposit Held in Regulated Financial Institution — deposits must be held in a REGULATED FINANCIAL INSTITUTION and must NOT be commingled with the landlord's personal funds; alternatively, landlord may secure a LANDLORD TENANT SECURITY DEPOSIT BOND and comply with annual certification requirements with the Michigan Department of Attorney General".to_string(),
        "MCL § 554.609 30-Day Itemized Damages List + Return — landlord must mail to the tenant an ITEMIZED LIST OF DAMAGES within 30 DAYS of the tenant ending occupancy, together with payment of any portion of the deposit not withheld".to_string(),
        "MCL § 554.610 7-Day Tenant Response to Dispute — tenant has 7 DAYS from receipt of the notice of damages to notify the landlord IN WRITING of any disagreement with the itemized list; failure of tenant to respond in writing within 7 days FORFEITS the tenant's ability to dispute the deductions in subsequent litigation".to_string(),
        "MCL § 554.612 Landlord 45-Day Suit Deadline — if tenant disputes within 7 days, landlord must commence a SMALL CLAIMS or DISTRICT COURT action within 45 DAYS of tenant's response to enforce the damages claim; failure to file within 45 days WAIVES the landlord's right to retain the disputed portion".to_string(),
        "Truth in Renting Act — Public Act 454 of 1978, MCL §§ 554.631-554.641 — separately codified act regulating residential lease form and content".to_string(),
        "MCL § 554.633 Prohibited Lease Clauses — VOID — certain clauses are PROHIBITED from inclusion in written leases, and if such a provision is included in a lease, the provision is VOID: (1) WAIVER OF REMEDY available for violation of covenants of fitness and habitability; (2) CONFESSION OF JUDGMENT clauses; (3) provisions which EXCULPATE LANDLORDS from liability imposed by law; (4) clauses that WAIVE A TENANT'S RIGHT TO TRIAL BY JURY, to notice, or to any other procedural rights provided by the Summary Proceedings Act, or by the anti-lockout law; (5) clauses which grant the landlord a SECURITY INTEREST IN ANY PERSONAL PROPERTY OF THE TENANT to assure payment of rent or other charges".to_string(),
        "MCL § 554.639 Waiver Prohibited — the requirements of the Truth in Renting Act may NOT BE WAIVED; any clause purporting to waive a tenant's rights with regard to summary proceedings for eviction or with regard to illegal detainer, lockout, or interference with possession is PROHIBITED AND VOID".to_string(),
        "MCL § 554.634 Disclosure Required — lessors must DISCLOSE to the lessee certain information in the rental agreement, including the address where lessor will accept service of process and notices".to_string(),
        "Michigan Legislature + Justia + Michigan Courts Bench Book + Tilchin & Hall PC + Landlord Studio + Rentable + Tenant Rights + Kershaw Vititoe & Jedinak — practitioner overviews of Michigan landlord-tenant law".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromMichiganLandlordTenantActs {
        return Output {
            mode: MiLandlordTenantMode::NotApplicableTenancyExemptFromMichiganLandlordTenantActs,
            statutory_basis: "Michigan landlord-tenant acts jurisdiction — tenancy exempt from both Security Deposit Act and Truth in Renting Act coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Michigan Security Deposit Act (PA 348 of 1972) and Truth in Renting Act (PA 454 of 1978); Michigan landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapUnderMcl554_602 => {
            if input.deposit_amount_tenths_of_months_rent <= MI_SECURITY_DEPOSIT_CAP_TENTHS_OF_MONTHS {
                Output {
                    mode: MiLandlordTenantMode::CompliantSecurityDepositAtOrBelowOnePointFiveMonthsCap,
                    statutory_basis: "MCL § 554.602 — security deposit at or below 1.5 months' rent cap".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit at {tenths} tenths-of-months rent within 1.5-months (15 tenths) statutory cap under MCL § 554.602.",
                        tenths = input.deposit_amount_tenths_of_months_rent,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MiLandlordTenantMode::ViolationSecurityDepositExceedsOnePointFiveMonthsCap,
                    statutory_basis: "MCL § 554.602 — security deposit exceeds 1.5 months' rent statutory cap".to_string(),
                    notes: format!(
                        "VIOLATION: deposit at {tenths} tenths-of-months rent exceeds 1.5-months (15 tenths) statutory cap under MCL § 554.602.",
                        tenths = input.deposit_amount_tenths_of_months_rent,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordBankNoticeAndChecklistUnderMcl554_603 => {
            if input.days_landlord_bank_notice_and_checklist_delivered
                <= MI_LANDLORD_BANK_NOTICE_DAYS
            {
                Output {
                    mode: MiLandlordTenantMode::CompliantLandlordBankNoticeAndChecklistDeliveredWithin14Days,
                    statutory_basis: "MCL § 554.603 — landlord delivered bank notice and move-in checklist within 14 days".to_string(),
                    notes: format!(
                        "COMPLIANT: bank notice + move-in inventory checklist delivered at day {d} (within 14-day statutory window) under MCL § 554.603.",
                        d = input.days_landlord_bank_notice_and_checklist_delivered,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MiLandlordTenantMode::ViolationLandlordBankNoticeOrChecklistNotDeliveredWithin14Days,
                    statutory_basis: "MCL § 554.603 — landlord failed to deliver bank notice and checklist within 14 days".to_string(),
                    notes: format!(
                        "VIOLATION: bank notice + move-in inventory checklist delivered at day {d} (past 14-day statutory window) under MCL § 554.603.",
                        d = input.days_landlord_bank_notice_and_checklist_delivered,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::DepositHoldingArrangementUnderMcl554_604 => match input
            .deposit_holding_arrangement
        {
            DepositHoldingArrangement::HeldInRegulatedFinancialInstitutionSeparateFromLandlordFunds
            | DepositHoldingArrangement::LandlordTenantSecurityDepositBondWithAnnualCertification => Output {
                mode: MiLandlordTenantMode::CompliantDepositInRegulatedFinancialInstitutionOrBonded,
                statutory_basis: "MCL § 554.604 — deposit held in regulated financial institution separate from landlord funds OR landlord tenant security deposit bond with annual certification".to_string(),
                notes: "COMPLIANT: deposit held in regulated financial institution separate from landlord personal funds OR landlord secured a Landlord Tenant Security Deposit Bond with annual certification under MCL § 554.604.".to_string(),
                citations,
            },
            DepositHoldingArrangement::CommingledWithLandlordPersonalFundsProhibited => Output {
                mode: MiLandlordTenantMode::ViolationDepositCommingledWithLandlordPersonalFunds,
                statutory_basis: "MCL § 554.604 — deposit may not be commingled with landlord personal funds".to_string(),
                notes: "VIOLATION: deposit commingled with landlord personal funds prohibited under MCL § 554.604; landlord must hold deposit in regulated financial institution separate from personal funds OR secure a Landlord Tenant Security Deposit Bond.".to_string(),
                citations,
            },
        },
        ComplianceAspect::ThirtyDayItemizedDamagesListUnderMcl554_609 => {
            if input.days_to_mail_itemized_damages_list
                <= MI_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: MiLandlordTenantMode::CompliantThirtyDayItemizedDamagesListMailedWithin30Days,
                    statutory_basis: "MCL § 554.609 — itemized damages list mailed within 30 days of tenant ending occupancy".to_string(),
                    notes: format!(
                        "COMPLIANT: itemized damages list mailed at day {d} (within 30-day statutory window) under MCL § 554.609.",
                        d = input.days_to_mail_itemized_damages_list,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MiLandlordTenantMode::ViolationItemizedDamagesListPast30DayDeadline,
                    statutory_basis: "MCL § 554.609 — itemized damages list mailed past 30-day statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: itemized damages list mailed at day {d} (past 30-day statutory window) under MCL § 554.609.",
                        d = input.days_to_mail_itemized_damages_list,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::SevenDayTenantResponseRightUnderMcl554_610 => {
            if input.days_tenant_responded_after_receiving_damages_list <= MI_TENANT_RESPONSE_DAYS
            {
                Output {
                    mode: MiLandlordTenantMode::CompliantSevenDayTenantResponseRightObserved,
                    statutory_basis: "MCL § 554.610 — tenant responded in writing within 7 days of receiving itemized damages list".to_string(),
                    notes: format!(
                        "COMPLIANT: tenant responded at day {d} (within 7-day statutory window) preserving right to dispute deductions under MCL § 554.610.",
                        d = input.days_tenant_responded_after_receiving_damages_list,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MiLandlordTenantMode::ViolationTenantResponseRightNotObservedOrPastSevenDays,
                    statutory_basis: "MCL § 554.610 — tenant response past 7 days forfeits right to dispute".to_string(),
                    notes: format!(
                        "VIOLATION: tenant response at day {d} (past 7-day statutory window) forfeits tenant's ability to dispute deductions in subsequent litigation under MCL § 554.610.",
                        d = input.days_tenant_responded_after_receiving_damages_list,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::FortyFiveDayLandlordSuitDeadlineUnderMcl554_612 => {
            if input.days_landlord_filed_suit_after_tenant_response
                <= MI_LANDLORD_SUIT_DEADLINE_DAYS_AFTER_TENANT_RESPONSE
            {
                Output {
                    mode: MiLandlordTenantMode::CompliantLandlordSuitFiledWithin45DaysOfTenantResponse,
                    statutory_basis: "MCL § 554.612 — landlord filed suit within 45 days of tenant response".to_string(),
                    notes: format!(
                        "COMPLIANT: landlord filed suit at day {d} after tenant response (within 45-day statutory window) preserving right to retain disputed portion under MCL § 554.612.",
                        d = input.days_landlord_filed_suit_after_tenant_response,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MiLandlordTenantMode::ViolationLandlordSuitNotFiledWithin45DaysWaivesRightToRetain,
                    statutory_basis: "MCL § 554.612 — landlord failed to file suit within 45 days; waives right to retain disputed portion".to_string(),
                    notes: format!(
                        "VIOLATION: landlord filed suit at day {d} after tenant response (past 45-day statutory window); landlord WAIVES right to retain disputed portion under MCL § 554.612.",
                        d = input.days_landlord_filed_suit_after_tenant_response,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::TraProhibitedClausesUnderMcl554_633 => match input
            .tra_prohibited_clause_status
        {
            TraProhibitedClauseStatus::NoProhibitedClausesIncluded => Output {
                mode: MiLandlordTenantMode::CompliantTraNoProhibitedClausesIncluded,
                statutory_basis: "MCL § 554.633 — lease contains no prohibited clauses".to_string(),
                notes: "COMPLIANT: lease contains no prohibited clauses (habitability-waiver / confession-of-judgment / landlord-exculpation / jury-waiver / personal-property-security-interest) under MCL § 554.633.".to_string(),
                citations,
            },
            _ => Output {
                mode: MiLandlordTenantMode::ViolationTraProhibitedClauseIncludedClauseVoid,
                statutory_basis: "MCL § 554.633 — lease contains prohibited clause; clause is VOID".to_string(),
                notes: "VIOLATION: lease contains prohibited clause under MCL § 554.633 (habitability-waiver / confession-of-judgment / landlord-exculpation / jury-waiver / personal-property-security-interest); the prohibited clause is VOID.".to_string(),
                citations,
            },
        },
        ComplianceAspect::TraWaiverProhibitionUnderMcl554_639 => {
            if input.tra_waiver_attempted {
                Output {
                    mode: MiLandlordTenantMode::ViolationTraWaiverAttempted,
                    statutory_basis: "MCL § 554.639 — waiver of Truth in Renting Act requirements prohibited and void".to_string(),
                    notes: "VIOLATION: lease attempted to waive Truth in Renting Act requirements; any waiver of TRA requirements is PROHIBITED AND VOID under MCL § 554.639.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: MiLandlordTenantMode::CompliantTraWaiverProhibitionObserved,
                    statutory_basis: "MCL § 554.639 — no waiver of Truth in Renting Act requirements attempted".to_string(),
                    notes: "COMPLIANT: lease did not attempt to waive Truth in Renting Act requirements under MCL § 554.639.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::TraDisclosureRequirementUnderMcl554_634 => {
            if input.tra_disclosure_provided {
                Output {
                    mode: MiLandlordTenantMode::CompliantTraDisclosureProvided,
                    statutory_basis: "MCL § 554.634 — Truth in Renting Act disclosures provided to lessee".to_string(),
                    notes: "COMPLIANT: lessor provided Truth in Renting Act disclosures (address for service of process and notices, etc.) to lessee under MCL § 554.634.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: MiLandlordTenantMode::ViolationTraDisclosureOmitted,
                    statutory_basis: "MCL § 554.634 — required Truth in Renting Act disclosures omitted".to_string(),
                    notes: "VIOLATION: lessor omitted required Truth in Renting Act disclosures (address for service of process and notices, etc.) under MCL § 554.634.".to_string(),
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
                DepositHoldingArrangement::HeldInRegulatedFinancialInstitutionSeparateFromLandlordFunds,
            tra_prohibited_clause_status: TraProhibitedClauseStatus::NoProhibitedClausesIncluded,
            compliance_aspect: ComplianceAspect::SecurityDepositCapUnderMcl554_602,
            deposit_amount_tenths_of_months_rent: 10,
            days_landlord_bank_notice_and_checklist_delivered: 14,
            days_to_mail_itemized_damages_list: 25,
            days_tenant_responded_after_receiving_damages_list: 5,
            days_landlord_filed_suit_after_tenant_response: 30,
            tra_waiver_attempted: false,
            tra_disclosure_provided: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromMichiganLandlordTenantActs;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::NotApplicableTenancyExemptFromMichiganLandlordTenantActs
        );
    }

    #[test]
    fn deposit_at_1_5_month_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderMcl554_602;
        input.deposit_amount_tenths_of_months_rent = 15;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantSecurityDepositAtOrBelowOnePointFiveMonthsCap
        );
    }

    #[test]
    fn deposit_above_1_5_month_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositCapUnderMcl554_602;
        input.deposit_amount_tenths_of_months_rent = 16;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationSecurityDepositExceedsOnePointFiveMonthsCap
        );
    }

    #[test]
    fn landlord_bank_notice_at_14_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordBankNoticeAndChecklistUnderMcl554_603;
        input.days_landlord_bank_notice_and_checklist_delivered = 14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantLandlordBankNoticeAndChecklistDeliveredWithin14Days
        );
    }

    #[test]
    fn landlord_bank_notice_at_15_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordBankNoticeAndChecklistUnderMcl554_603;
        input.days_landlord_bank_notice_and_checklist_delivered = 15;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationLandlordBankNoticeOrChecklistNotDeliveredWithin14Days
        );
    }

    #[test]
    fn deposit_in_regulated_financial_institution_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositHoldingArrangementUnderMcl554_604;
        input.deposit_holding_arrangement =
            DepositHoldingArrangement::HeldInRegulatedFinancialInstitutionSeparateFromLandlordFunds;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantDepositInRegulatedFinancialInstitutionOrBonded
        );
    }

    #[test]
    fn deposit_landlord_tenant_security_bond_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositHoldingArrangementUnderMcl554_604;
        input.deposit_holding_arrangement =
            DepositHoldingArrangement::LandlordTenantSecurityDepositBondWithAnnualCertification;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantDepositInRegulatedFinancialInstitutionOrBonded
        );
    }

    #[test]
    fn deposit_commingled_with_landlord_funds_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositHoldingArrangementUnderMcl554_604;
        input.deposit_holding_arrangement =
            DepositHoldingArrangement::CommingledWithLandlordPersonalFundsProhibited;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationDepositCommingledWithLandlordPersonalFunds
        );
    }

    #[test]
    fn itemized_damages_at_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThirtyDayItemizedDamagesListUnderMcl554_609;
        input.days_to_mail_itemized_damages_list = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantThirtyDayItemizedDamagesListMailedWithin30Days
        );
    }

    #[test]
    fn itemized_damages_at_31_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThirtyDayItemizedDamagesListUnderMcl554_609;
        input.days_to_mail_itemized_damages_list = 31;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationItemizedDamagesListPast30DayDeadline
        );
    }

    #[test]
    fn tenant_response_at_7_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SevenDayTenantResponseRightUnderMcl554_610;
        input.days_tenant_responded_after_receiving_damages_list = 7;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantSevenDayTenantResponseRightObserved
        );
    }

    #[test]
    fn tenant_response_at_8_days_forfeits_dispute_right() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SevenDayTenantResponseRightUnderMcl554_610;
        input.days_tenant_responded_after_receiving_damages_list = 8;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationTenantResponseRightNotObservedOrPastSevenDays
        );
    }

    #[test]
    fn landlord_suit_at_45_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FortyFiveDayLandlordSuitDeadlineUnderMcl554_612;
        input.days_landlord_filed_suit_after_tenant_response = 45;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantLandlordSuitFiledWithin45DaysOfTenantResponse
        );
    }

    #[test]
    fn landlord_suit_at_46_days_waives_right_to_retain() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FortyFiveDayLandlordSuitDeadlineUnderMcl554_612;
        input.days_landlord_filed_suit_after_tenant_response = 46;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationLandlordSuitNotFiledWithin45DaysWaivesRightToRetain
        );
    }

    #[test]
    fn tra_no_prohibited_clauses_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraProhibitedClausesUnderMcl554_633;
        input.tra_prohibited_clause_status = TraProhibitedClauseStatus::NoProhibitedClausesIncluded;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantTraNoProhibitedClausesIncluded
        );
    }

    #[test]
    fn tra_confession_of_judgment_clause_void_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraProhibitedClausesUnderMcl554_633;
        input.tra_prohibited_clause_status =
            TraProhibitedClauseStatus::ConfessionOfJudgmentClauseIncluded;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationTraProhibitedClauseIncludedClauseVoid
        );
    }

    #[test]
    fn tra_jury_waiver_clause_void_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraProhibitedClausesUnderMcl554_633;
        input.tra_prohibited_clause_status =
            TraProhibitedClauseStatus::JuryWaiverOrSummaryProceedingsRightsWaiverIncluded;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationTraProhibitedClauseIncludedClauseVoid
        );
    }

    #[test]
    fn tra_security_interest_clause_void_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraProhibitedClausesUnderMcl554_633;
        input.tra_prohibited_clause_status =
            TraProhibitedClauseStatus::SecurityInterestInTenantPersonalPropertyIncluded;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationTraProhibitedClauseIncludedClauseVoid
        );
    }

    #[test]
    fn tra_waiver_not_attempted_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraWaiverProhibitionUnderMcl554_639;
        input.tra_waiver_attempted = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantTraWaiverProhibitionObserved
        );
    }

    #[test]
    fn tra_waiver_attempted_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraWaiverProhibitionUnderMcl554_639;
        input.tra_waiver_attempted = true;
        let out = check(&input);
        assert_eq!(out.mode, MiLandlordTenantMode::ViolationTraWaiverAttempted);
    }

    #[test]
    fn tra_disclosure_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraDisclosureRequirementUnderMcl554_634;
        input.tra_disclosure_provided = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::CompliantTraDisclosureProvided
        );
    }

    #[test]
    fn tra_disclosure_omitted_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TraDisclosureRequirementUnderMcl554_634;
        input.tra_disclosure_provided = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MiLandlordTenantMode::ViolationTraDisclosureOmitted
        );
    }

    #[test]
    fn constants_pin_michigan_landlord_tenant_statutory_thresholds() {
        assert_eq!(MI_SECURITY_DEPOSIT_ACT_PUBLIC_ACT_NUMBER, 348);
        assert_eq!(MI_SECURITY_DEPOSIT_ACT_ENACTMENT_YEAR, 1972);
        assert_eq!(MI_TRUTH_IN_RENTING_ACT_PUBLIC_ACT_NUMBER, 454);
        assert_eq!(MI_TRUTH_IN_RENTING_ACT_ENACTMENT_YEAR, 1978);
        assert_eq!(MI_SECURITY_DEPOSIT_CAP_TENTHS_OF_MONTHS, 15);
        assert_eq!(MI_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS, 10);
        assert_eq!(MI_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(MI_TENANT_RESPONSE_DAYS, 7);
        assert_eq!(MI_LANDLORD_BANK_NOTICE_DAYS, 14);
        assert_eq!(MI_LANDLORD_SUIT_DEADLINE_DAYS_AFTER_TENANT_RESPONSE, 45);
        assert_eq!(MI_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_michigan_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Michigan Landlord-Tenant Law"));
        assert!(joined.contains("Security Deposit Act"));
        assert!(joined.contains("Public Act 348 of 1972"));
        assert!(joined.contains("MCL §§ 554.601-554.616"));
        assert!(joined.contains("Truth in Renting Act"));
        assert!(joined.contains("Public Act 454 of 1978"));
        assert!(joined.contains("MCL §§ 554.631-554.641"));
        assert!(joined.contains("MCL § 554.602"));
        assert!(joined.contains("1 1/2 MONTHS' RENT"));
        assert!(joined.contains("MCL § 554.603"));
        assert!(joined.contains("14 DAYS"));
        assert!(joined.contains("WRITTEN NOTICE OF THE BANK NAME AND ADDRESS"));
        assert!(joined.contains("DETAILED MOVE-IN INVENTORY CHECKLIST"));
        assert!(joined.contains("MCL § 554.604"));
        assert!(joined.contains("REGULATED FINANCIAL INSTITUTION"));
        assert!(joined.contains("LANDLORD TENANT SECURITY DEPOSIT BOND"));
        assert!(joined.contains("MCL § 554.609"));
        assert!(joined.contains("ITEMIZED LIST OF DAMAGES"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("MCL § 554.610"));
        assert!(joined.contains("7 DAYS"));
        assert!(joined.contains("FORFEITS"));
        assert!(joined.contains("MCL § 554.612"));
        assert!(joined.contains("45 DAYS"));
        assert!(joined.contains("WAIVES"));
        assert!(joined.contains("MCL § 554.633"));
        assert!(joined.contains("PROHIBITED"));
        assert!(joined.contains("VOID"));
        assert!(joined.contains("WAIVER OF REMEDY"));
        assert!(joined.contains("CONFESSION OF JUDGMENT"));
        assert!(joined.contains("EXCULPATE LANDLORDS"));
        assert!(joined.contains("WAIVE A TENANT'S RIGHT TO TRIAL BY JURY"));
        assert!(joined.contains("SECURITY INTEREST IN ANY PERSONAL PROPERTY OF THE TENANT"));
        assert!(joined.contains("MCL § 554.639"));
        assert!(joined.contains("NOT BE WAIVED"));
        assert!(joined.contains("MCL § 554.634"));
        assert!(joined.contains("DISCLOSE"));
    }
}
