//! Georgia Landlord-Tenant Law (O.C.G.A. § 44-7)
//! Compliance Module — codified at O.C.G.A. §§ 44-7-1
//! through 44-7-119 (Title 44 Property, Chapter 7
//! Landlord and Tenant). Pure-compute check for trader-
//! landlord compliance with Georgia's foundational
//! residential tenancy regime.
//!
//! Georgia is **NOT a URLTA state** — it operates under
//! its own statutory framework rather than adopting the
//! Uniform Residential Landlord and Tenant Act. Atlanta
//! metropolitan area is a top-10 US rental market, with
//! significant rental markets also in Savannah, Augusta,
//! Columbus, Macon, and Athens. Substantially amended by
//! the **Safe at Home Act of 2024** (effective **July 1,
//! 2024**), which codified the implied warranty of
//! habitability and added a statutory 3-business-day
//! pay-or-quit notice requirement before dispossessory
//! action.
//!
//! Web research (verified 2026-06-03):
//! - **Codification**: Georgia Landlord-Tenant Law codified at **O.C.G.A. §§ 44-7-1 through 44-7-119** (Title 44 Property, Chapter 7 Landlord and Tenant). Articles include Article 1 (General), Article 2 (Security Deposits §§ 44-7-30 through 44-7-37), Article 3 (Dispossessory Proceedings §§ 44-7-49 through 44-7-59) ([Justia — 2024 Georgia Code § 44-7-31 Placement of Security Deposit in Escrow Account](https://law.justia.com/codes/georgia/title-44/chapter-7/article-2/section-44-7-31/); [Justia — 2024 Georgia Code § 44-7-34 Return of Security Deposit Grounds for Retention](https://law.justia.com/codes/georgia/title-44/chapter-7/article-2/section-44-7-34/); [Justia — 2024 Georgia Code § 44-7-35 Remedies for Landlord's Noncompliance With Article](https://law.justia.com/codes/georgia/title-44/chapter-7/article-2/section-44-7-35/); [Justia — 2024 Georgia Code § 44-7-13 Landlord's Duties as to Repairs and Improvements](https://law.justia.com/codes/georgia/title-44/chapter-7/article-1/section-44-7-13/); [Justia — 2024 Georgia Code § 44-7-50 Demand for Possession; Procedure Upon Tenant's Refusal; Notice to Vacate or Pay](https://law.justia.com/codes/georgia/title-44/chapter-7/article-3/section-44-7-50/); [Justia — 2024 Georgia Code Title 44 Chapter 7 Article 2 Security Deposits](https://law.justia.com/codes/georgia/title-44/chapter-7/article-2/); [Georgia Appleseed — Safe at Home Act Bench Card PDF](https://gaappleseed.org/wp-content/uploads/SafeAtHome_BenchCard.pdf); [FindLaw — Georgia Code § 44-7-50](https://codes.findlaw.com/ga/title-44-property/ga-code-sect-44-7-50/); [Excalibur Homes — Landlord-Tenant Law in Georgia 44-7-6 & 44-7-13](https://www.excaliburhomes.com/blog/landlord-tenant-law-in-georgia-44-7-6-tenancy-at-will-44-7-13-landlords-duty-to-repair/); [Georgia E-Laws — Section 44-7-13](http://ga.elaws.us/law/section44-7-13); [Landlord Studio — Georgia Security Deposit Laws](https://www.landlordstudio.com/landlord-tenant-laws/georgia-security-deposit-laws); [TurboTenant — Georgia Security Deposit Law](https://www.turbotenant.com/rental-lease-agreement/georgia/security-deposit-law/); [Deposit Forensics — Georgia Security Deposit Laws 2026 30-Day Return Deadline](https://www.depositforensics.com/georgia-security-deposit-laws); [LeaseRunner — Georgia Security Deposit Law Escrow, Charges and Return](https://www.leaserunner.com/lease-agreement/georgia/security-deposits-in-georgia); [Rentable — Georgia Security Deposit Laws Complete Guide](https://www.rentable.com/blog/georgia-security-deposit-laws-a-complete-guide-for-landlords-tenants/); [iPropertyManagement — Georgia Security Deposit Law](https://ipropertymanagement.com/laws/georgia-security-deposit-returns); [American Landlord — Georgia Landlord Security Deposit Escrow Account Requirement](https://americanlandlord.com/georgia-landlord-tenant-law/georgia-landlord-security-deposit-escrow-account-requirement/); [LeaseRunner — Security Deposit Laws Georgia Small Landlords](https://www.leaserunner.com/laws/georgia-security-deposits-for-small-landlords); [Consumer-SOS — Security Deposits Georgia](https://www.consumer-sos.com/Georgia/Landlord_&_Tenant/ga_security.htm)).
//! - **O.C.G.A. § 44-7-31 Security Deposit Escrow Account**: security deposit shall be deposited in an **ESCROW ACCOUNT** established only for that purpose in any **BANK OR LENDING INSTITUTION SUBJECT TO REGULATION** by Georgia or by any agency of the United States government; landlord must inform tenant **IN WRITING** of the location of the escrow account.
//! - **O.C.G.A. § 44-7-34 Security Deposit Return**: landlord must return security deposit within **30 DAYS** after tenant vacates the property, minus lawful deductions; if actual cause exists for retaining any portion, landlord must provide tenant with **WRITTEN STATEMENT identifying the exact reasons for retention**, which must include the comprehensive list of damages prepared as required by § 44-7-33.
//! - **O.C.G.A. § 44-7-35 Bad Faith Retention Treble Damages**: any landlord who fails to return any part of a security deposit which is required to be returned is liable for **THREE TIMES the sum improperly withheld PLUS reasonable attorney's fees**; failure of landlord to provide the lists and written statements within the time periods specified in § 44-7-34 **WORKS A FORFEITURE** of all the landlord's rights to withhold any portion of the security deposit or to bring an action against the tenant for damages to the premises.
//! - **O.C.G.A. § 44-7-13 Landlord's Duties as to Repairs and Improvements**: landlord shall **KEEP THE PREMISES IN REPAIR** and shall be liable for all substantial improvements placed upon the premises by such landlord's consent; any contract for the use or rental of real property as a dwelling place is deemed to include a provision that the premises is **FIT FOR HUMAN HABITATION** (codified by Safe at Home Act of 2024).
//! - **O.C.G.A. § 44-7-7 Notice to Terminate Tenancy at Will**: **60-DAY NOTICE FROM LANDLORD**; **30-DAY NOTICE FROM TENANT** for tenancy at will under § 44-7-6 when no time is specified for termination.
//! - **O.C.G.A. § 44-7-50 Dispossessory Proceedings — Three-Business-Day Pay or Quit Notice** (added by Safe at Home Act of 2024, effective **JULY 1, 2024**): when tenant fails to pay rent, late fees, utilities, or other charges, landlord must provide tenant with a **NOTICE TO VACATE OR PAY** all past due rent, late fees, utilities, and other charges **WITHIN 3 BUSINESS DAYS** before filing dispossessory affidavit; if tenant refuses to pay or fails to deliver possession within 3 business days, landlord may file dispossessory affidavit with court (superior court, state court, magistrate court).
//! - **Safe at Home Act of 2024**: substantially amended Georgia landlord-tenant law effective **July 1, 2024**; codified the implied warranty of habitability under § 44-7-13; added 3-business-day pay-or-quit notice requirement under § 44-7-50; applies to residential lease agreements entered into or renewed on or after July 1, 2024.
//! - **NO Statutory Security Deposit Cap**: Georgia has **NO STATUTORY CAP** on security deposit amount; landlord may charge any amount agreed to in the lease.
//! - **Small-Landlord Exception (10 Units or Fewer)**: landlords who own 10 or fewer rental units (and use no rental management agent) MAY be exempt from the escrow account requirement under O.C.G.A. § 44-7-32; small landlords with 10 or fewer units may hold deposit as personal funds but must still return within 30 days.
//! - **Retaliation Prohibited (Section 8 Voucher Tenants — O.C.G.A. § 44-7-103)**: retaliation against Section 8 Housing Choice Voucher tenants is specifically prohibited; Georgia does not have a general statewide statutory retaliation prohibition for non-voucher tenants but common-law claims and federal Fair Housing Act protections apply.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const GA_LANDLORD_TENANT_TITLE_NUMBER: u32 = 44;
pub const GA_LANDLORD_TENANT_CHAPTER_NUMBER: u32 = 7;
pub const GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_ENACTMENT_YEAR: u32 = 2024;
pub const GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_EFFECTIVE_DATE_YEAR: u32 = 2024;
pub const GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_EFFECTIVE_DATE_MONTH: u32 = 7;
pub const GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_EFFECTIVE_DATE_DAY: u32 = 1;
pub const GA_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const GA_LANDLORD_TENANT_PAY_OR_QUIT_NOTICE_BUSINESS_DAYS: u32 = 3;
pub const GA_LANDLORD_TENANT_LANDLORD_NOTICE_TO_TERMINATE_AT_WILL_DAYS: u32 = 60;
pub const GA_LANDLORD_TENANT_TENANT_NOTICE_TO_TERMINATE_AT_WILL_DAYS: u32 = 30;
pub const GA_LANDLORD_TENANT_TREBLE_DAMAGES_MULTIPLIER: u64 = 3;
pub const GA_LANDLORD_TENANT_SMALL_LANDLORD_UNIT_THRESHOLD: u32 = 10;
pub const GA_LANDLORD_TENANT_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalCoveredByChapter7,
    CommercialRentalExempt,
    HotelMotelTransientLodgingExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticePartyType {
    LandlordTerminatingTenancyAtWill,
    TenantTerminatingTenancyAtWill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositEscrowAccountUnderOcga44_7_31,
    SecurityDepositReturnAndItemizedStatementUnderOcga44_7_34,
    SecurityDepositBadFaithRetentionTrebleDamagesUnderOcga44_7_35,
    LandlordObligationToRepairAndHabitabilityUnderOcga44_7_13,
    NoticeToTerminateTenancyAtWillUnderOcga44_7_7,
    ThreeBusinessDayPayOrQuitNoticeUnderOcga44_7_50,
    SmallLandlordEscrowExceptionUnderOcga44_7_32,
    Section8RetaliationProhibitedUnderOcga44_7_103,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GaLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter7,
    CompliantSecurityDepositInEscrowAccountAndTenantInformedInWriting,
    CompliantSecurityDepositReturnedWithinThirtyDaysWithItemizedStatement,
    CompliantSecurityDepositNoBadFaithRetention,
    CompliantLandlordObligationsToRepairAndHabitabilityMet,
    CompliantNoticeToTerminateTenancyAtWillProvided,
    CompliantThreeBusinessDayPayOrQuitNoticeProvided,
    CompliantSmallLandlordExceptionMetExemptFromEscrow,
    CompliantNoSection8Retaliation,
    ViolationSecurityDepositNotInEscrowAccountOrTenantNotInformed,
    ViolationSecurityDepositReturnedPastThirtyDayDeadlineForfeitsWithholdingRights,
    ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlusAttorneyFees,
    ViolationLandlordRepairObligationsBreached,
    ViolationNoticeToTerminateTenancyAtWillShorterThanStatutoryMinimum,
    ViolationPayOrQuitNoticeShorterThanThreeBusinessDaysPostSafeAtHomeAct,
    ViolationSection8RetaliationProhibited,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub notice_party_type: NoticePartyType,
    pub monthly_rent_dollars: u64,
    pub security_deposit_dollars: u64,
    pub portion_of_deposit_wrongfully_withheld_dollars: u64,
    pub deposit_in_escrow_account_in_regulated_institution: bool,
    pub tenant_informed_in_writing_of_escrow_location: bool,
    pub deposit_returned_within_window_with_itemized_statement: bool,
    pub days_since_tenant_vacated_for_deposit_return: u32,
    pub landlord_repair_and_habitability_obligations_met: bool,
    pub termination_notice_days_given: u32,
    pub pay_or_quit_notice_business_days_given: u32,
    pub landlord_total_unit_count: u32,
    pub uses_rental_management_agent: bool,
    pub section_8_voucher_tenant_retaliation_occurred: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: GaLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub treble_damages_remedy_dollars: u64,
}

pub type RentalGeorgiaLandlordTenantActOcga44_7Input = Input;
pub type RentalGeorgiaLandlordTenantActOcga44_7Output = Output;
pub type RentalGeorgiaLandlordTenantActOcga44_7Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Georgia Landlord-Tenant Law — codified at O.C.G.A. §§ 44-7-1 through 44-7-119 (Title 44 Property, Chapter 7 Landlord and Tenant). Articles include Article 1 (General), Article 2 (Security Deposits §§ 44-7-30 through 44-7-37), Article 3 (Dispossessory Proceedings §§ 44-7-49 through 44-7-59). Georgia is NOT a URLTA state.".to_string(),
        "O.C.G.A. § 44-7-31 Security Deposit Escrow Account — security deposit shall be deposited in an ESCROW ACCOUNT established only for that purpose in any BANK OR LENDING INSTITUTION SUBJECT TO REGULATION by Georgia or by any agency of the United States government; landlord must inform tenant IN WRITING of the location of the escrow account".to_string(),
        "O.C.G.A. § 44-7-32 Small-Landlord Exception — landlords who own 10 OR FEWER rental units (and use no rental management agent) may be exempt from the escrow account requirement; small landlords with 10 or fewer units may hold deposit as personal funds but must still return within 30 days".to_string(),
        "O.C.G.A. § 44-7-33 Inspection and Damages List — landlord must conduct move-in and move-out inspections and provide tenant with a comprehensive list of damages".to_string(),
        "O.C.G.A. § 44-7-34 Security Deposit Return — landlord must return security deposit within 30 DAYS after tenant vacates the property, minus lawful deductions; if actual cause exists for retaining any portion, landlord must provide tenant with WRITTEN STATEMENT identifying the exact reasons for retention, which must include the comprehensive list of damages prepared under § 44-7-33".to_string(),
        "O.C.G.A. § 44-7-35 Remedies for Landlord's Noncompliance — any landlord who fails to return any part of a security deposit which is required to be returned is liable for THREE TIMES the sum improperly withheld PLUS reasonable attorney's fees; failure of landlord to provide the lists and written statements within the time periods specified in § 44-7-34 WORKS A FORFEITURE of all the landlord's rights to withhold any portion of the security deposit or to bring an action against the tenant for damages to the premises".to_string(),
        "O.C.G.A. § 44-7-13 Landlord's Duties as to Repairs and Improvements — landlord shall KEEP THE PREMISES IN REPAIR and shall be liable for all substantial improvements placed upon the premises by such landlord's consent; any contract for the use or rental of real property as a dwelling place is deemed to include a provision that the premises is FIT FOR HUMAN HABITATION (codified by Safe at Home Act of 2024 effective July 1, 2024)".to_string(),
        "O.C.G.A. § 44-7-7 Notice to Terminate Tenancy at Will — 60-DAY NOTICE FROM LANDLORD; 30-DAY NOTICE FROM TENANT for tenancy at will under § 44-7-6 when no time is specified for termination".to_string(),
        "O.C.G.A. § 44-7-50 Dispossessory Proceedings — Three-Business-Day Pay or Quit Notice — added by Safe at Home Act of 2024 effective JULY 1, 2024; when tenant fails to pay rent, late fees, utilities, or other charges, landlord must provide tenant with NOTICE TO VACATE OR PAY all past due rent, late fees, utilities, and other charges WITHIN 3 BUSINESS DAYS before filing dispossessory affidavit; if tenant refuses to pay or fails to deliver possession within 3 business days, landlord may file dispossessory affidavit with court".to_string(),
        "Safe at Home Act of 2024 — substantially amended Georgia landlord-tenant law effective JULY 1, 2024; codified the implied warranty of habitability under § 44-7-13; added 3-business-day pay-or-quit notice requirement under § 44-7-50; applies to residential lease agreements entered into or renewed on or after July 1, 2024".to_string(),
        "O.C.G.A. § 44-7-103 Section 8 Voucher Tenant Retaliation Prohibited — retaliation against Section 8 Housing Choice Voucher tenants specifically prohibited; Georgia does not have a general statewide statutory retaliation prohibition for non-voucher tenants but common-law claims and federal Fair Housing Act protections apply".to_string(),
        "NO Statutory Security Deposit Cap — Georgia has NO STATUTORY CAP on security deposit amount; landlord may charge any amount agreed to in the lease".to_string(),
        "Georgia Appleseed — Safe at Home Act Bench Card — practitioner guide for 2024 reforms".to_string(),
        "Justia + FindLaw + Georgia E-Laws + Excalibur Homes + Landlord Studio + TurboTenant + Deposit Forensics + LeaseRunner + Rentable + iPropertyManagement + American Landlord + Consumer-SOS — primary statutory text and practitioner guides".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalCoveredByChapter7 {
        return Output {
            mode: GaLandlordTenantMode::NotApplicableTenancyExemptFromChapter7,
            statutory_basis: "O.C.G.A. § 44-7-1 — Chapter 7 applies only to residential leaseholds; commercial / hotel-motel transient lodging exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from O.C.G.A. § 44-7 (commercial rental; hotel/motel transient lodging).".to_string(),
            citations,
            treble_damages_remedy_dollars: 0,
        };
    }

    let treble_damages = input
        .portion_of_deposit_wrongfully_withheld_dollars
        .saturating_mul(GA_LANDLORD_TENANT_TREBLE_DAMAGES_MULTIPLIER);

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositEscrowAccountUnderOcga44_7_31 => {
            if input.deposit_in_escrow_account_in_regulated_institution
                && input.tenant_informed_in_writing_of_escrow_location
            {
                Output {
                    mode: GaLandlordTenantMode::CompliantSecurityDepositInEscrowAccountAndTenantInformedInWriting,
                    statutory_basis: "O.C.G.A. § 44-7-31 — security deposit in escrow account at regulated institution AND tenant informed in writing of escrow location".to_string(),
                    notes: "COMPLIANT: landlord deposited security deposit in escrow account at bank or lending institution subject to Georgia or federal regulation AND informed tenant in writing of the escrow account location under § 44-7-31.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::ViolationSecurityDepositNotInEscrowAccountOrTenantNotInformed,
                    statutory_basis: "O.C.G.A. § 44-7-31 — security deposit not in escrow account at regulated institution OR tenant not informed in writing".to_string(),
                    notes: "VIOLATION: landlord did not deposit security deposit in escrow account at regulated institution OR did not inform tenant in writing of escrow location under § 44-7-31; tenant may seek statutory remedies including treble damages.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnAndItemizedStatementUnderOcga44_7_34 => {
            if input.deposit_returned_within_window_with_itemized_statement
                && input.days_since_tenant_vacated_for_deposit_return
                    <= GA_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: GaLandlordTenantMode::CompliantSecurityDepositReturnedWithinThirtyDaysWithItemizedStatement,
                    statutory_basis: "O.C.G.A. § 44-7-34 — security deposit returned with itemized statement within 30-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord returned security deposit with written itemized statement of damages within 30 days after tenant vacated the property under § 44-7-34.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::ViolationSecurityDepositReturnedPastThirtyDayDeadlineForfeitsWithholdingRights,
                    statutory_basis: "O.C.G.A. § 44-7-34 + § 44-7-35 — security deposit not returned with itemized statement within 30-day deadline FORFEITS all withholding rights".to_string(),
                    notes: "VIOLATION: landlord missed 30-day deposit return / itemized statement deadline under § 44-7-34; failure to provide lists and written statements WORKS A FORFEITURE of all landlord's rights to withhold any portion of deposit or bring action against tenant for damages under § 44-7-35.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: treble_damages,
                }
            }
        }
        ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderOcga44_7_35 => {
            if input.portion_of_deposit_wrongfully_withheld_dollars > 0 {
                Output {
                    mode: GaLandlordTenantMode::ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlusAttorneyFees,
                    statutory_basis: "O.C.G.A. § 44-7-35 — bad faith retention triggers treble damages + reasonable attorney's fees".to_string(),
                    notes: "VIOLATION: landlord wrongfully retained portion of security deposit; § 44-7-35 remedy = THREE TIMES the sum improperly withheld PLUS reasonable attorney's fees.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: treble_damages,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::CompliantSecurityDepositNoBadFaithRetention,
                    statutory_basis: "O.C.G.A. § 44-7-35 — no wrongful retention".to_string(),
                    notes: "COMPLIANT: no wrongful retention of security deposit; § 44-7-35 treble-damages remedy does not attach.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::LandlordObligationToRepairAndHabitabilityUnderOcga44_7_13 => {
            if input.landlord_repair_and_habitability_obligations_met {
                Output {
                    mode: GaLandlordTenantMode::CompliantLandlordObligationsToRepairAndHabitabilityMet,
                    statutory_basis: "O.C.G.A. § 44-7-13 — landlord obligations to repair and habitability met".to_string(),
                    notes: "COMPLIANT: landlord keeps the premises in repair AND premises is fit for human habitation under § 44-7-13 (implied warranty of habitability codified by Safe at Home Act of 2024).".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::ViolationLandlordRepairObligationsBreached,
                    statutory_basis: "O.C.G.A. § 44-7-13 — landlord obligations to repair and habitability breached".to_string(),
                    notes: "VIOLATION: landlord breached § 44-7-13 obligations to keep premises in repair AND ensure premises is fit for human habitation; tenant remedies including damages, repair-and-deduct (where applicable), and termination.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::NoticeToTerminateTenancyAtWillUnderOcga44_7_7 => {
            let required_days = match input.notice_party_type {
                NoticePartyType::LandlordTerminatingTenancyAtWill => {
                    GA_LANDLORD_TENANT_LANDLORD_NOTICE_TO_TERMINATE_AT_WILL_DAYS
                }
                NoticePartyType::TenantTerminatingTenancyAtWill => {
                    GA_LANDLORD_TENANT_TENANT_NOTICE_TO_TERMINATE_AT_WILL_DAYS
                }
            };
            if input.termination_notice_days_given >= required_days {
                Output {
                    mode: GaLandlordTenantMode::CompliantNoticeToTerminateTenancyAtWillProvided,
                    statutory_basis: "O.C.G.A. § 44-7-7 — notice to terminate tenancy at will meets statutory minimum".to_string(),
                    notes: "COMPLIANT: notice to terminate tenancy at will meets statutory minimum under § 44-7-7 (60 days for landlord; 30 days for tenant).".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::ViolationNoticeToTerminateTenancyAtWillShorterThanStatutoryMinimum,
                    statutory_basis: "O.C.G.A. § 44-7-7 — notice to terminate tenancy at will shorter than statutory minimum".to_string(),
                    notes: "VIOLATION: notice to terminate tenancy at will shorter than statutory minimum under § 44-7-7 (60 days for landlord; 30 days for tenant); termination invalid.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::ThreeBusinessDayPayOrQuitNoticeUnderOcga44_7_50 => {
            if input.pay_or_quit_notice_business_days_given
                >= GA_LANDLORD_TENANT_PAY_OR_QUIT_NOTICE_BUSINESS_DAYS
            {
                Output {
                    mode: GaLandlordTenantMode::CompliantThreeBusinessDayPayOrQuitNoticeProvided,
                    statutory_basis: "O.C.G.A. § 44-7-50 — 3-business-day pay or quit notice provided (Safe at Home Act 2024)".to_string(),
                    notes: "COMPLIANT: landlord provided 3-business-day pay or quit notice under § 44-7-50 before filing dispossessory affidavit (Safe at Home Act of 2024 requirement effective July 1, 2024).".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::ViolationPayOrQuitNoticeShorterThanThreeBusinessDaysPostSafeAtHomeAct,
                    statutory_basis: "O.C.G.A. § 44-7-50 — pay or quit notice shorter than 3 business days post-Safe at Home Act 2024".to_string(),
                    notes: "VIOLATION: pay or quit notice shorter than 3-business-day statutory minimum under § 44-7-50 (Safe at Home Act of 2024); dispossessory action subject to dismissal.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::SmallLandlordEscrowExceptionUnderOcga44_7_32 => {
            if input.landlord_total_unit_count <= GA_LANDLORD_TENANT_SMALL_LANDLORD_UNIT_THRESHOLD
                && !input.uses_rental_management_agent
            {
                Output {
                    mode: GaLandlordTenantMode::CompliantSmallLandlordExceptionMetExemptFromEscrow,
                    statutory_basis: "O.C.G.A. § 44-7-32 — small landlord exception (10 or fewer units + no management agent) met".to_string(),
                    notes: "COMPLIANT: landlord owns 10 or fewer rental units AND does not use rental management agent; exempt from escrow account requirement under § 44-7-32; deposit may be held as personal funds but must still be returned within 30 days under § 44-7-34.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::ViolationSecurityDepositNotInEscrowAccountOrTenantNotInformed,
                    statutory_basis: "O.C.G.A. § 44-7-32 — small landlord exception not met (more than 10 units OR uses management agent)".to_string(),
                    notes: "VIOLATION: landlord owns MORE than 10 rental units OR uses rental management agent; small landlord exception under § 44-7-32 does not apply; escrow account requirement under § 44-7-31 applies.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::Section8RetaliationProhibitedUnderOcga44_7_103 => {
            if input.section_8_voucher_tenant_retaliation_occurred {
                Output {
                    mode: GaLandlordTenantMode::ViolationSection8RetaliationProhibited,
                    statutory_basis: "O.C.G.A. § 44-7-103 — retaliation against Section 8 voucher tenant prohibited".to_string(),
                    notes: "VIOLATION: landlord retaliated against Section 8 Housing Choice Voucher tenant; § 44-7-103 specifically prohibits such retaliation.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: GaLandlordTenantMode::CompliantNoSection8Retaliation,
                    statutory_basis: "O.C.G.A. § 44-7-103 — no Section 8 voucher tenant retaliation".to_string(),
                    notes: "COMPLIANT: no retaliation against Section 8 Housing Choice Voucher tenant under § 44-7-103.".to_string(),
                    citations,
                    treble_damages_remedy_dollars: 0,
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
            tenancy_type: TenancyType::ResidentialRentalCoveredByChapter7,
            compliance_aspect:
                ComplianceAspect::SecurityDepositReturnAndItemizedStatementUnderOcga44_7_34,
            notice_party_type: NoticePartyType::LandlordTerminatingTenancyAtWill,
            monthly_rent_dollars: 1_500,
            security_deposit_dollars: 1_500,
            portion_of_deposit_wrongfully_withheld_dollars: 0,
            deposit_in_escrow_account_in_regulated_institution: true,
            tenant_informed_in_writing_of_escrow_location: true,
            deposit_returned_within_window_with_itemized_statement: true,
            days_since_tenant_vacated_for_deposit_return: 25,
            landlord_repair_and_habitability_obligations_met: true,
            termination_notice_days_given: 60,
            pay_or_quit_notice_business_days_given: 3,
            landlord_total_unit_count: 50,
            uses_rental_management_agent: true,
            section_8_voucher_tenant_retaliation_occurred: false,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::CommercialRentalExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::NotApplicableTenancyExemptFromChapter7
        );
    }

    #[test]
    fn deposit_in_escrow_and_tenant_informed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositEscrowAccountUnderOcga44_7_31;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantSecurityDepositInEscrowAccountAndTenantInformedInWriting
        );
    }

    #[test]
    fn deposit_not_in_escrow_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositEscrowAccountUnderOcga44_7_31;
        input.deposit_in_escrow_account_in_regulated_institution = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSecurityDepositNotInEscrowAccountOrTenantNotInformed
        );
    }

    #[test]
    fn tenant_not_informed_in_writing_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositEscrowAccountUnderOcga44_7_31;
        input.tenant_informed_in_writing_of_escrow_location = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSecurityDepositNotInEscrowAccountOrTenantNotInformed
        );
    }

    #[test]
    fn deposit_return_within_30_days_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantSecurityDepositReturnedWithinThirtyDaysWithItemizedStatement
        );
    }

    #[test]
    fn deposit_return_at_exactly_30_days_boundary_compliant() {
        let mut input = baseline_input();
        input.days_since_tenant_vacated_for_deposit_return = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantSecurityDepositReturnedWithinThirtyDaysWithItemizedStatement
        );
    }

    #[test]
    fn deposit_return_at_31_days_violation_forfeits_withholding() {
        let mut input = baseline_input();
        input.days_since_tenant_vacated_for_deposit_return = 31;
        input.deposit_returned_within_window_with_itemized_statement = false;
        input.portion_of_deposit_wrongfully_withheld_dollars = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSecurityDepositReturnedPastThirtyDayDeadlineForfeitsWithholdingRights
        );
        assert_eq!(output.treble_damages_remedy_dollars, 500 * 3);
    }

    #[test]
    fn bad_faith_retention_treble_damages_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderOcga44_7_35;
        input.portion_of_deposit_wrongfully_withheld_dollars = 800;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlusAttorneyFees
        );
        assert_eq!(output.treble_damages_remedy_dollars, 800 * 3);
    }

    #[test]
    fn no_bad_faith_retention_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderOcga44_7_35;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantSecurityDepositNoBadFaithRetention
        );
    }

    #[test]
    fn landlord_repair_and_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToRepairAndHabitabilityUnderOcga44_7_13;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantLandlordObligationsToRepairAndHabitabilityMet
        );
    }

    #[test]
    fn landlord_repair_obligations_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToRepairAndHabitabilityUnderOcga44_7_13;
        input.landlord_repair_and_habitability_obligations_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationLandlordRepairObligationsBreached
        );
    }

    #[test]
    fn landlord_60_day_notice_to_terminate_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NoticeToTerminateTenancyAtWillUnderOcga44_7_7;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantNoticeToTerminateTenancyAtWillProvided
        );
    }

    #[test]
    fn landlord_59_day_notice_to_terminate_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NoticeToTerminateTenancyAtWillUnderOcga44_7_7;
        input.termination_notice_days_given = 59;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationNoticeToTerminateTenancyAtWillShorterThanStatutoryMinimum
        );
    }

    #[test]
    fn tenant_30_day_notice_to_terminate_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NoticeToTerminateTenancyAtWillUnderOcga44_7_7;
        input.notice_party_type = NoticePartyType::TenantTerminatingTenancyAtWill;
        input.termination_notice_days_given = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantNoticeToTerminateTenancyAtWillProvided
        );
    }

    #[test]
    fn tenant_29_day_notice_to_terminate_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NoticeToTerminateTenancyAtWillUnderOcga44_7_7;
        input.notice_party_type = NoticePartyType::TenantTerminatingTenancyAtWill;
        input.termination_notice_days_given = 29;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationNoticeToTerminateTenancyAtWillShorterThanStatutoryMinimum
        );
    }

    #[test]
    fn three_business_day_pay_or_quit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeBusinessDayPayOrQuitNoticeUnderOcga44_7_50;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantThreeBusinessDayPayOrQuitNoticeProvided
        );
    }

    #[test]
    fn pay_or_quit_under_3_business_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeBusinessDayPayOrQuitNoticeUnderOcga44_7_50;
        input.pay_or_quit_notice_business_days_given = 2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationPayOrQuitNoticeShorterThanThreeBusinessDaysPostSafeAtHomeAct
        );
    }

    #[test]
    fn small_landlord_exception_at_10_units_no_agent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SmallLandlordEscrowExceptionUnderOcga44_7_32;
        input.landlord_total_unit_count = 10;
        input.uses_rental_management_agent = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantSmallLandlordExceptionMetExemptFromEscrow
        );
    }

    #[test]
    fn small_landlord_at_11_units_violation_escrow_required() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SmallLandlordEscrowExceptionUnderOcga44_7_32;
        input.landlord_total_unit_count = 11;
        input.uses_rental_management_agent = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSecurityDepositNotInEscrowAccountOrTenantNotInformed
        );
    }

    #[test]
    fn small_landlord_uses_management_agent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SmallLandlordEscrowExceptionUnderOcga44_7_32;
        input.landlord_total_unit_count = 5;
        input.uses_rental_management_agent = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSecurityDepositNotInEscrowAccountOrTenantNotInformed
        );
    }

    #[test]
    fn no_section_8_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::Section8RetaliationProhibitedUnderOcga44_7_103;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::CompliantNoSection8Retaliation
        );
    }

    #[test]
    fn section_8_retaliation_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::Section8RetaliationProhibitedUnderOcga44_7_103;
        input.section_8_voucher_tenant_retaliation_occurred = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSection8RetaliationProhibited
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(GA_LANDLORD_TENANT_TITLE_NUMBER, 44);
        assert_eq!(GA_LANDLORD_TENANT_CHAPTER_NUMBER, 7);
        assert_eq!(GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_ENACTMENT_YEAR, 2024);
        assert_eq!(
            GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_EFFECTIVE_DATE_YEAR,
            2024
        );
        assert_eq!(GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_EFFECTIVE_DATE_MONTH, 7);
        assert_eq!(GA_LANDLORD_TENANT_SAFE_AT_HOME_ACT_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(GA_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(GA_LANDLORD_TENANT_PAY_OR_QUIT_NOTICE_BUSINESS_DAYS, 3);
        assert_eq!(
            GA_LANDLORD_TENANT_LANDLORD_NOTICE_TO_TERMINATE_AT_WILL_DAYS,
            60
        );
        assert_eq!(
            GA_LANDLORD_TENANT_TENANT_NOTICE_TO_TERMINATE_AT_WILL_DAYS,
            30
        );
        assert_eq!(GA_LANDLORD_TENANT_TREBLE_DAMAGES_MULTIPLIER, 3);
        assert_eq!(GA_LANDLORD_TENANT_SMALL_LANDLORD_UNIT_THRESHOLD, 10);
        assert_eq!(GA_LANDLORD_TENANT_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Georgia Landlord-Tenant Law"));
        assert!(joined.contains("O.C.G.A. §§ 44-7-1 through 44-7-119"));
        assert!(joined.contains("§ 44-7-7"));
        assert!(joined.contains("§ 44-7-13"));
        assert!(joined.contains("§ 44-7-31"));
        assert!(joined.contains("§ 44-7-32"));
        assert!(joined.contains("§ 44-7-33"));
        assert!(joined.contains("§ 44-7-34"));
        assert!(joined.contains("§ 44-7-35"));
        assert!(joined.contains("§ 44-7-50"));
        assert!(joined.contains("§ 44-7-103"));
        assert!(joined.contains("Safe at Home Act of 2024"));
        assert!(joined.contains("JULY 1, 2024"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("3 BUSINESS DAYS"));
        assert!(joined.contains("60-DAY"));
        assert!(joined.contains("30-DAY"));
        assert!(joined.contains("ESCROW ACCOUNT"));
        assert!(joined.contains("THREE TIMES"));
        assert!(joined.contains("FORFEITURE"));
        assert!(joined.contains("FIT FOR HUMAN HABITATION"));
        assert!(joined.contains("10 OR FEWER"));
        assert!(joined.contains("NOT a URLTA state"));
    }

    #[test]
    fn treble_damages_saturating_overflow_defense() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderOcga44_7_35;
        input.portion_of_deposit_wrongfully_withheld_dollars = u64::MAX;
        let output = check(&input);
        assert_eq!(
            output.mode,
            GaLandlordTenantMode::ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlusAttorneyFees
        );
        assert_eq!(output.treble_damages_remedy_dollars, u64::MAX);
    }
}
