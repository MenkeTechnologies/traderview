//! Tennessee Uniform Residential Landlord and Tenant Act
//! (URLTA) Compliance Module — codified at T.C.A. §§
//! 66-28-101 through 66-28-523. Pure-compute check for
//! trader-landlord compliance with Tennessee's
//! foundational residential tenancy regime.
//!
//! Adopted by the Tennessee General Assembly in **1975**
//! (Public Acts 1975, Chapter 245) based on the Uniform
//! Residential Landlord and Tenant Act (URLTA). Tennessee
//! URLTA applies ONLY to counties with population **>
//! 75,000** (per most recent federal census), covering the
//! 10 most-populous counties — **Davidson (Nashville),
//! Shelby (Memphis), Knox (Knoxville), Hamilton
//! (Chattanooga), Rutherford (Murfreesboro), Williamson
//! (Franklin), Sumner (Hendersonville), Montgomery
//! (Clarksville), Wilson (Lebanon), and Blount
//! (Maryville)**. Rural counties (population ≤ 75,000)
//! follow common-law landlord-tenant rules instead.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Tennessee Uniform Residential Landlord and Tenant Act adopted by Tennessee General Assembly in **1975** (Public Acts 1975, Chapter 245); codified at **T.C.A. §§ 66-28-101 through 66-28-523** (Title 66 Property, Chapter 28 Uniform Residential Landlord and Tenant Act) ([Justia — 2024 Tennessee Code § 66-28-301 Security Deposits](https://law.justia.com/codes/tennessee/title-66/chapter-28/part-3/section-66-28-301/); [Justia — 2024 Tennessee Code § 66-28-403 Access by Landlord](https://law.justia.com/codes/tennessee/title-66/chapter-28/part-4/section-66-28-403/); [Justia — 2024 Tennessee Code § 66-28-512 Termination of Periodic Tenancy Holdover Remedies](https://law.justia.com/codes/tennessee/title-66/chapter-28/part-5/section-66-28-512/); [Justia — 2024 Tennessee Code § 66-28-514 Retaliatory Conduct Prohibited](https://law.justia.com/codes/tennessee/title-66/chapter-28/part-5/section-66-28-514/); [Justia — 2024 Tennessee Code Title 66 Chapter 28 Part 5 Enforcement and Remedies](https://law.justia.com/codes/tennessee/title-66/chapter-28/part-5/); [FindLaw — Tennessee Code Title 66 Property § 66-28-301](https://codes.findlaw.com/tn/title-66-property/tn-code-sect-66-28-301/); [FindLaw — Tennessee Code Title 66 Property § 66-28-403](https://codes.findlaw.com/tn/title-66-property/tn-code-sect-66-28-403/); [FindLaw — Tennessee Code Title 66 Property § 66-28-512](https://codes.findlaw.com/tn/title-66-property/tn-code-sect-66-28-512/); [FindLaw — Tennessee Code Title 66 Property § 66-28-514](https://codes.findlaw.com/tn/title-66-property/tn-code-sect-66-28-514.html); [Blount County Government — Tennessee Code § 66-28-301 PDF](https://www.blounttn.gov/DocumentCenter/View/27670/1171_001); [Blount County Government — Tennessee Code § 66-28-501 PDF](https://www.blounttn.gov/DocumentCenter/View/27672/1173_001); [Paine Bickers — MJK 2018 URLTA Memo on Tennessee Eviction Procedures and Sample Notices PDF](http://www.painebickers.com/wp-content/uploads/2018/12/MJK-2018-URLTA-Memo-on-Tennessee-Eviction-Procedures-and-Sample-Notices.pdf); [LawServer — Tennessee Code 66-28-403 Access by Landlord](https://www.lawserver.com/law/state/tennessee/tn-code/tennessee_code_66-28-403); [LawServer — Tennessee Code 66-28-514 Retaliatory Conduct Prohibited](https://www.lawserver.com/law/state/tennessee/tn-code/tennessee_code_66-28-514); [Tennessee SB 1009 / HB 1760 Public Chapter Bill Text PDF](https://www.capitol.tn.gov/Bills/107/Bill/SB1009.pdf); [American Apartment Owners Association — Tennessee Landlord Tenant Laws](https://american-apartment-owners-association.org/landlord-tenant-laws/tennessee/); [PayRent — Tennessee Landlord Tenant Laws Updated 2023](https://www.payrent.com/articles/tennessee-landlord-tenant-laws/); [Nolo — Tennessee Landlord-Tenant Law Essential Guide](https://www.nolo.com/legal-encyclopedia/overview-landlord-tenant-laws-tennessee.html); [Nolo — The Eviction Process in Tennessee Rules and Procedures](https://www.nolo.com/legal-encyclopedia/the-eviction-process-tennessee-rules-landlords-property-managers.html); [Nolo — Tenant's Right to Break a Rental Lease in Tennessee](https://www.nolo.com/legal-encyclopedia/tenants-right-break-rental-lease-tennessee.html); [Nolo — Defenses Tenants Can Raise to Evictions in Tennessee](https://www.nolo.com/legal-encyclopedia/tenant-defenses-evictions-tennessee.html); [iPropertyManagement — Tennessee Security Deposit Laws 2026 Returns Deductions](https://ipropertymanagement.com/laws/tennessee-security-deposit-returns); [iPropertyManagement — Tennessee 14 Day Notice to Pay or Quit](https://ipropertymanagement.com/templates/tennessee-14-day-notice-to-quit); [iPropertyManagement — Tennessee Eviction Process in 2026](https://ipropertymanagement.com/laws/tennessee-eviction-process); [Hemlane — Tennessee Security Deposit Laws 2026](https://www.hemlane.com/resources/tennessee-security-deposit-laws/); [DoorLoop — Tennessee Security Deposit Laws Deductions Rights](https://www.doorloop.com/laws/tennessee-security-deposit-laws); [Rentable — Tennessee Security Deposit Laws Complete Guide](https://www.rentable.com/blog/tennessee-security-deposit-laws-a-complete-guide-for-landlords-tenants/); [eForms — Free Tennessee 14-Day Notice to Quit Form Non-Payment](https://eforms.com/eviction/tn/tennessee-14-day-notice-to-quit-form-non-payment/); [Landlord Studio — Tennessee Landlord Tenant Laws](https://www.landlordstudio.com/landlord-tenant-laws/tennessee-landlord-tenant-laws/); [LegalTemplates — Free Tennessee Eviction Notice Forms](https://legaltemplates.net/form/eviction-notice/tennessee-tn/); [Landlord Guidance — Tennessee Eviction Notice Rights and Legal Actions](https://www.landlordguidance.com/eviction-notice-forms/tennessee-eviction/)).
//! - **Population Threshold Applicability**: Tennessee URLTA applies ONLY to **COUNTIES WITH POPULATION GREATER THAN 75,000** (per most recent federal census); rural counties (population ≤ 75,000) follow common-law landlord-tenant rules; covered counties include Davidson (Nashville), Shelby (Memphis), Knox (Knoxville), Hamilton (Chattanooga), Rutherford (Murfreesboro), Williamson (Franklin), Sumner (Hendersonville), Montgomery (Clarksville), Wilson (Lebanon), Blount (Maryville).
//! - **T.C.A. § 66-28-301 Security Deposits**: **NO STATUTORY CAP** on security deposit amount; landlord must place security deposit into **SEPARATE, FEDERALLY INSURED BANK ACCOUNT LOCATED WITHIN TENNESSEE**; landlord must return security deposit (or remaining balance after deductions) with written itemized statement within **30 DAYS** after termination of tenancy; if tenant does not claim remaining deposit within **60 DAYS** after landlord's written notice of refund due, landlord may remove deposit from separate account and keep the funds.
//! - **T.C.A. § 66-28-302 Landlord Obligations**: landlord must comply with applicable building and housing codes materially affecting health and safety; make all repairs to put and keep premises in fit and habitable condition; keep common areas safe and sanitary; maintain in good and safe working order all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities and appliances supplied by landlord.
//! - **T.C.A. § 66-28-403 Landlord Access**: tenant shall not unreasonably withhold consent to landlord entry to inspect / repair / decorate / alter / improve / supply services / exhibit to prospective purchasers / mortgagees / workers / contractors; landlord may enter **WITHOUT CONSENT IN CASE OF EMERGENCY** (sudden, generally unexpected occurrence demanding immediate action); within the **FINAL 30 DAYS** of rental agreement termination, landlord may enter for the purpose of showing the premises to prospective tenants with at least **24 HOURS' NOTICE** (provided right of access is set forth in the rental agreement).
//! - **T.C.A. § 66-28-505(a)(2) 14-Day Cure or Pay or Quit Notice**: for nonpayment of rent OR breach that is remediable by payment or repairs, landlord must give tenant **14-DAY NOTICE TO CURE OR QUIT**.
//! - **T.C.A. § 66-28-505(a)(2)(B) 7-Day No-Cure Notice for Second Violation**: if tenant commits a second lease violation or fails to pay rent a second time within **6 MONTHS**, landlord may give tenant a **7-DAY NOTICE TO QUIT** with **NO OPPORTUNITY TO CORRECT** the behavior.
//! - **T.C.A. § 66-28-512 Periodic Tenancy Termination**: landlord must give **30-DAY NOTICE** to end a month-to-month tenancy.
//! - **T.C.A. § 66-28-514 Retaliatory Conduct Prohibited**: landlord may NOT retaliate by increasing rent, decreasing services, or bringing or threatening to bring an action for possession because tenant has (a) complained to landlord of a violation under § 66-28-301 (security deposit) OR § 66-28-302 (landlord obligations); (b) complained to a governmental agency about a building or housing code violation; (c) made use of remedies provided under this chapter.
//! - **14-Day Repair Remedy under § 66-28-502**: if landlord does NOT take action within **14 DAYS** of tenant's written notice of code violation or condition affecting health and safety, tenant may seek court order directing landlord to remedy (injunction), terminate the rental agreement, or receive compensation, plus reasonable attorney's fees.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const TN_URLTA_TITLE_NUMBER: u32 = 66;
pub const TN_URLTA_CHAPTER_NUMBER: u32 = 28;
pub const TN_URLTA_ENACTMENT_YEAR: u32 = 1975;
pub const TN_URLTA_ENABLING_PUBLIC_ACT_CHAPTER: u32 = 245;
pub const TN_URLTA_POPULATION_APPLICABILITY_THRESHOLD: u64 = 75_000;
pub const TN_URLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const TN_URLTA_SECURITY_DEPOSIT_UNCLAIMED_WINDOW_DAYS: u32 = 60;
pub const TN_URLTA_PAY_OR_QUIT_OR_CURE_NOTICE_DAYS: u32 = 14;
pub const TN_URLTA_SECOND_VIOLATION_NO_CURE_NOTICE_DAYS: u32 = 7;
pub const TN_URLTA_SECOND_VIOLATION_LOOKBACK_MONTHS: u32 = 6;
pub const TN_URLTA_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS: u32 = 30;
pub const TN_URLTA_FINAL_30_DAYS_SHOWING_NOTICE_HOURS: u32 = 24;
pub const TN_URLTA_LANDLORD_REPAIR_RESPONSE_DEADLINE_DAYS: u32 = 14;
pub const TN_URLTA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CountyApplicability {
    CountyAboveSeventyFiveThousandPopulationCoveredByUrlta,
    CountyAtOrBelowSeventyFiveThousandPopulationCommonLawApplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BreachCategory {
    NonpaymentOfRent,
    BreachRemediableByPaymentOrRepairs,
    SecondViolationWithinSixMonths,
    NoBreachAlleged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositSeparateAccountAndReturnUnderTca66_28_301,
    LandlordObligationsToMaintainPremisesUnderTca66_28_302,
    LandlordAccessUnderTca66_28_403,
    FourteenDayCureOrQuitNoticeUnderTca66_28_505,
    SevenDayNoCureNoticeForSecondViolationUnderTca66_28_505AB,
    PeriodicTenancyTerminationThirtyDayNoticeUnderTca66_28_512,
    RetaliatoryConductProhibitedUnderTca66_28_514,
    LandlordRepairFourteenDayResponseUnderTca66_28_502,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TnUrltaMode {
    NotApplicableCountyAtOrBelowSeventyFiveThousandPopulation,
    CompliantSecurityDepositInSeparateAccountAndReturnedWithinThirtyDays,
    CompliantLandlordObligationsMet,
    CompliantLandlordEntryReasonableNoticeOrEmergencyOrFinalThirtyDayShowingNotice,
    CompliantFourteenDayCureOrQuitNoticeProvided,
    CompliantSevenDayNoCureNoticeForSecondViolationProvided,
    CompliantPeriodicTenancyThirtyDayTerminationNoticeProvided,
    CompliantNoRetaliatoryConduct,
    CompliantLandlordRepairWithinFourteenDayResponse,
    ViolationSecurityDepositNotInSeparateAccountOrReturnedPastThirtyDayDeadline,
    ViolationLandlordObligationsBreached,
    ViolationLandlordEntryUnreasonableAndNotEmergencyAndNotFinalThirtyDayShowingNotice,
    ViolationCureOrQuitNoticeShorterThanFourteenDays,
    ViolationSecondViolationNoticeShorterThanSevenDays,
    ViolationPeriodicTenancyTerminationNoticeShorterThanThirtyDays,
    ViolationRetaliatoryConduct,
    ViolationLandlordRepairExceededFourteenDayDeadline,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub county_applicability: CountyApplicability,
    pub compliance_aspect: ComplianceAspect,
    pub breach_category: BreachCategory,
    pub deposit_in_separate_federally_insured_account_in_tennessee: bool,
    pub deposit_returned_with_itemized_statement_within_window: bool,
    pub days_since_termination_for_deposit_return: u32,
    pub landlord_obligations_met: bool,
    pub landlord_entry_with_reasonable_notice_or_emergency_or_final_thirty_days_showing: bool,
    pub pay_or_quit_or_cure_notice_days_given: u32,
    pub second_violation_no_cure_notice_days_given: u32,
    pub periodic_tenancy_termination_notice_days_given: u32,
    pub retaliatory_conduct_occurred: bool,
    pub landlord_repair_response_days: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: TnUrltaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalTennesseeUrltaTca66_28_101Input = Input;
pub type RentalTennesseeUrltaTca66_28_101Output = Output;
pub type RentalTennesseeUrltaTca66_28_101Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Tennessee Uniform Residential Landlord and Tenant Act — adopted by Tennessee General Assembly in 1975 (Public Acts 1975, Chapter 245); codified at T.C.A. §§ 66-28-101 through 66-28-523 (Title 66 Property, Chapter 28 Uniform Residential Landlord and Tenant Act)".to_string(),
        "Tennessee URLTA Population Threshold Applicability — Tennessee URLTA applies ONLY to COUNTIES WITH POPULATION GREATER THAN 75,000 (per most recent federal census); rural counties (population ≤ 75,000) follow common-law landlord-tenant rules; covered counties include Davidson (Nashville), Shelby (Memphis), Knox (Knoxville), Hamilton (Chattanooga), Rutherford (Murfreesboro), Williamson (Franklin), Sumner (Hendersonville), Montgomery (Clarksville), Wilson (Lebanon), Blount (Maryville)".to_string(),
        "T.C.A. § 66-28-301 Security Deposits — NO STATUTORY CAP on security deposit amount; landlord must place security deposit into SEPARATE, FEDERALLY INSURED BANK ACCOUNT LOCATED WITHIN TENNESSEE; landlord must return security deposit (or remaining balance after deductions) with written itemized statement within 30 DAYS after termination of tenancy; if tenant does not claim remaining deposit within 60 DAYS after landlord's written notice of refund due, landlord may remove deposit from separate account and keep the funds".to_string(),
        "T.C.A. § 66-28-302 Landlord Obligations — landlord must comply with applicable building and housing codes materially affecting health and safety; make all repairs to put and keep premises in fit and habitable condition; keep common areas safe and sanitary; maintain in good and safe working order all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities and appliances supplied by landlord".to_string(),
        "T.C.A. § 66-28-403 Landlord Access — tenant shall not unreasonably withhold consent to landlord entry to inspect / repair / decorate / alter / improve / supply services / exhibit to prospective purchasers / mortgagees / workers / contractors; landlord may enter WITHOUT CONSENT IN CASE OF EMERGENCY (sudden, generally unexpected occurrence demanding immediate action); within the FINAL 30 DAYS of rental agreement termination, landlord may enter for purpose of showing premises to prospective tenants with at least 24 HOURS' NOTICE (provided right of access is set forth in rental agreement)".to_string(),
        "T.C.A. § 66-28-502 Landlord Repair Response — if landlord does NOT take action within 14 DAYS of tenant's written notice of code violation or condition affecting health and safety, tenant may seek court order directing landlord to remedy (injunction), terminate the rental agreement, or receive compensation, plus reasonable attorney's fees".to_string(),
        "T.C.A. § 66-28-505(a)(2) Cure or Pay or Quit Notice — for nonpayment of rent OR breach that is remediable by payment or repairs, landlord must give tenant 14-DAY NOTICE TO CURE OR QUIT".to_string(),
        "T.C.A. § 66-28-505(a)(2)(B) Second Violation No-Cure Notice — if tenant commits a second lease violation or fails to pay rent a second time within 6 MONTHS, landlord may give tenant a 7-DAY NOTICE TO QUIT with NO OPPORTUNITY TO CORRECT the behavior".to_string(),
        "T.C.A. § 66-28-512 Periodic Tenancy Termination — landlord must give 30-DAY NOTICE to end a month-to-month tenancy".to_string(),
        "T.C.A. § 66-28-514 Retaliatory Conduct Prohibited — landlord may NOT retaliate by increasing rent, decreasing services, or bringing or threatening to bring an action for possession because tenant has (a) complained to landlord of a violation under § 66-28-301 (security deposit) OR § 66-28-302 (landlord obligations); (b) complained to a governmental agency about a building or housing code violation; (c) made use of remedies provided under this chapter".to_string(),
        "Justia + FindLaw + Blount County Government + Paine Bickers + LawServer + American Apartment Owners Association + PayRent + Nolo + iPropertyManagement + Hemlane + DoorLoop + Rentable + eForms + Landlord Studio + LegalTemplates + Landlord Guidance — primary statutory text and practitioner guides".to_string(),
    ];

    if input.county_applicability
        == CountyApplicability::CountyAtOrBelowSeventyFiveThousandPopulationCommonLawApplies
    {
        return Output {
            mode: TnUrltaMode::NotApplicableCountyAtOrBelowSeventyFiveThousandPopulation,
            statutory_basis: "T.C.A. § 66-28-102 — URLTA applies only to counties with population greater than 75,000".to_string(),
            notes: "NOT APPLICABLE: county population is at or below 75,000 (rural Tennessee); URLTA does not apply; common-law landlord-tenant rules govern.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositSeparateAccountAndReturnUnderTca66_28_301 => {
            if input.deposit_in_separate_federally_insured_account_in_tennessee
                && input.deposit_returned_with_itemized_statement_within_window
                && input.days_since_termination_for_deposit_return
                    <= TN_URLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: TnUrltaMode::CompliantSecurityDepositInSeparateAccountAndReturnedWithinThirtyDays,
                    statutory_basis: "T.C.A. § 66-28-301 — security deposit in separate federally insured account AND returned with itemized statement within 30 days".to_string(),
                    notes: "COMPLIANT: landlord held security deposit in separate, federally insured bank account located within Tennessee AND returned security deposit (or remaining balance after deductions) with written itemized statement within 30 days after termination of tenancy under § 66-28-301.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::ViolationSecurityDepositNotInSeparateAccountOrReturnedPastThirtyDayDeadline,
                    statutory_basis: "T.C.A. § 66-28-301 — security deposit not in separate federally insured account OR returned past 30-day deadline".to_string(),
                    notes: "VIOLATION: landlord did not hold security deposit in separate federally insured account within Tennessee OR did not return deposit with itemized statement within 30-day deadline; tenant may seek statutory damages and reasonable attorney's fees.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordObligationsToMaintainPremisesUnderTca66_28_302 => {
            if input.landlord_obligations_met {
                Output {
                    mode: TnUrltaMode::CompliantLandlordObligationsMet,
                    statutory_basis: "T.C.A. § 66-28-302 — landlord obligations met".to_string(),
                    notes: "COMPLIANT: landlord met § 66-28-302 obligations (codes + repairs + common areas + facilities and appliances supplied by landlord).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::ViolationLandlordObligationsBreached,
                    statutory_basis: "T.C.A. § 66-28-302 — landlord obligations breached".to_string(),
                    notes: "VIOLATION: landlord breached § 66-28-302 obligations; tenant remedies under § 66-28-502 (14-day landlord response window then injunction / termination / damages + attorney's fees).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordAccessUnderTca66_28_403 => {
            if input.landlord_entry_with_reasonable_notice_or_emergency_or_final_thirty_days_showing
            {
                Output {
                    mode: TnUrltaMode::CompliantLandlordEntryReasonableNoticeOrEmergencyOrFinalThirtyDayShowingNotice,
                    statutory_basis: "T.C.A. § 66-28-403 — landlord entry with reasonable notice, emergency exception, or final 30-day showing notice".to_string(),
                    notes: "COMPLIANT: landlord entry with reasonable notice for inspection / repairs / showing OR emergency without consent OR final 30-day showing notice with 24 hours under § 66-28-403.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::ViolationLandlordEntryUnreasonableAndNotEmergencyAndNotFinalThirtyDayShowingNotice,
                    statutory_basis: "T.C.A. § 66-28-403 — landlord entry unreasonable and not emergency and not final 30-day showing notice".to_string(),
                    notes: "VIOLATION: landlord entered without reasonable notice AND not under emergency AND not within final 30-day showing window with 24 hours under § 66-28-403; tenant may obtain injunctive relief.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::FourteenDayCureOrQuitNoticeUnderTca66_28_505 => {
            if input.pay_or_quit_or_cure_notice_days_given
                >= TN_URLTA_PAY_OR_QUIT_OR_CURE_NOTICE_DAYS
            {
                Output {
                    mode: TnUrltaMode::CompliantFourteenDayCureOrQuitNoticeProvided,
                    statutory_basis: "T.C.A. § 66-28-505(a)(2) — 14-day cure or quit notice provided for nonpayment or remediable breach".to_string(),
                    notes: "COMPLIANT: landlord provided 14-day written notice to cure or quit under § 66-28-505(a)(2) for nonpayment of rent OR breach remediable by payment or repairs.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::ViolationCureOrQuitNoticeShorterThanFourteenDays,
                    statutory_basis: "T.C.A. § 66-28-505(a)(2) — cure or quit notice period shorter than 14-day statutory minimum".to_string(),
                    notes: "VIOLATION: cure or quit notice period shorter than 14-day statutory minimum under § 66-28-505(a)(2); detainer action subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SevenDayNoCureNoticeForSecondViolationUnderTca66_28_505AB => {
            if input.breach_category == BreachCategory::SecondViolationWithinSixMonths
                && input.second_violation_no_cure_notice_days_given
                    >= TN_URLTA_SECOND_VIOLATION_NO_CURE_NOTICE_DAYS
            {
                Output {
                    mode: TnUrltaMode::CompliantSevenDayNoCureNoticeForSecondViolationProvided,
                    statutory_basis: "T.C.A. § 66-28-505(a)(2)(B) — 7-day no-cure notice provided for second violation within 6 months".to_string(),
                    notes: "COMPLIANT: landlord provided 7-day no-cure written notice under § 66-28-505(a)(2)(B) for second lease violation or second nonpayment within 6 months.".to_string(),
                    citations,
                }
            } else if input.breach_category != BreachCategory::SecondViolationWithinSixMonths {
                Output {
                    mode: TnUrltaMode::ViolationSecondViolationNoticeShorterThanSevenDays,
                    statutory_basis: "T.C.A. § 66-28-505(a)(2)(B) — 7-day no-cure notice available only for second violation within 6 months".to_string(),
                    notes: "NOT TRIGGERED: tenant has not committed second violation within 6 months; 14-day cure-or-quit notice under § 66-28-505(a)(2) applies instead of 7-day no-cure notice.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::ViolationSecondViolationNoticeShorterThanSevenDays,
                    statutory_basis: "T.C.A. § 66-28-505(a)(2)(B) — second-violation no-cure notice shorter than 7-day statutory minimum".to_string(),
                    notes: "VIOLATION: second-violation no-cure notice shorter than 7-day statutory minimum under § 66-28-505(a)(2)(B); detainer action subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::PeriodicTenancyTerminationThirtyDayNoticeUnderTca66_28_512 => {
            if input.periodic_tenancy_termination_notice_days_given
                >= TN_URLTA_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS
            {
                Output {
                    mode: TnUrltaMode::CompliantPeriodicTenancyThirtyDayTerminationNoticeProvided,
                    statutory_basis: "T.C.A. § 66-28-512 — 30-day periodic tenancy termination notice provided".to_string(),
                    notes: "COMPLIANT: landlord provided 30-day written notice to end month-to-month tenancy under § 66-28-512.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::ViolationPeriodicTenancyTerminationNoticeShorterThanThirtyDays,
                    statutory_basis: "T.C.A. § 66-28-512 — periodic tenancy termination notice shorter than 30-day statutory minimum".to_string(),
                    notes: "VIOLATION: periodic tenancy termination notice shorter than 30-day statutory minimum under § 66-28-512; termination invalid.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::RetaliatoryConductProhibitedUnderTca66_28_514 => {
            if input.retaliatory_conduct_occurred {
                Output {
                    mode: TnUrltaMode::ViolationRetaliatoryConduct,
                    statutory_basis: "T.C.A. § 66-28-514 — retaliatory conduct prohibited".to_string(),
                    notes: "VIOLATION: landlord engaged in retaliatory conduct (rent increase / service decrease / possession action / threat thereof) because tenant (a) complained to landlord of § 66-28-301 or § 66-28-302 violation; (b) complained to governmental agency about code violation; (c) made use of URLTA remedies.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::CompliantNoRetaliatoryConduct,
                    statutory_basis: "T.C.A. § 66-28-514 — no retaliatory conduct".to_string(),
                    notes: "COMPLIANT: no retaliatory conduct under § 66-28-514.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordRepairFourteenDayResponseUnderTca66_28_502 => {
            if input.landlord_repair_response_days
                <= TN_URLTA_LANDLORD_REPAIR_RESPONSE_DEADLINE_DAYS
            {
                Output {
                    mode: TnUrltaMode::CompliantLandlordRepairWithinFourteenDayResponse,
                    statutory_basis: "T.C.A. § 66-28-502 — landlord repair within 14-day response window".to_string(),
                    notes: "COMPLIANT: landlord took action within 14 days of tenant's written notice of code violation or condition affecting health and safety under § 66-28-502.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: TnUrltaMode::ViolationLandlordRepairExceededFourteenDayDeadline,
                    statutory_basis: "T.C.A. § 66-28-502 — landlord repair response exceeded 14-day deadline".to_string(),
                    notes: "VIOLATION: landlord did not take action within 14 days of tenant's written notice under § 66-28-502; tenant may seek court order (injunction), terminate rental agreement, or receive compensation plus reasonable attorney's fees.".to_string(),
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
            county_applicability:
                CountyApplicability::CountyAboveSeventyFiveThousandPopulationCoveredByUrlta,
            compliance_aspect:
                ComplianceAspect::SecurityDepositSeparateAccountAndReturnUnderTca66_28_301,
            breach_category: BreachCategory::NonpaymentOfRent,
            deposit_in_separate_federally_insured_account_in_tennessee: true,
            deposit_returned_with_itemized_statement_within_window: true,
            days_since_termination_for_deposit_return: 25,
            landlord_obligations_met: true,
            landlord_entry_with_reasonable_notice_or_emergency_or_final_thirty_days_showing: true,
            pay_or_quit_or_cure_notice_days_given: 14,
            second_violation_no_cure_notice_days_given: 7,
            periodic_tenancy_termination_notice_days_given: 30,
            retaliatory_conduct_occurred: false,
            landlord_repair_response_days: 10,
        }
    }

    #[test]
    fn rural_county_below_75000_not_applicable() {
        let mut input = baseline_input();
        input.county_applicability =
            CountyApplicability::CountyAtOrBelowSeventyFiveThousandPopulationCommonLawApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::NotApplicableCountyAtOrBelowSeventyFiveThousandPopulation
        );
    }

    #[test]
    fn deposit_in_separate_account_and_returned_within_30_days_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantSecurityDepositInSeparateAccountAndReturnedWithinThirtyDays
        );
    }

    #[test]
    fn deposit_at_exactly_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.days_since_termination_for_deposit_return = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantSecurityDepositInSeparateAccountAndReturnedWithinThirtyDays
        );
    }

    #[test]
    fn deposit_at_31_days_violation() {
        let mut input = baseline_input();
        input.days_since_termination_for_deposit_return = 31;
        input.deposit_returned_with_itemized_statement_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationSecurityDepositNotInSeparateAccountOrReturnedPastThirtyDayDeadline
        );
    }

    #[test]
    fn deposit_not_in_separate_account_violation() {
        let mut input = baseline_input();
        input.deposit_in_separate_federally_insured_account_in_tennessee = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationSecurityDepositNotInSeparateAccountOrReturnedPastThirtyDayDeadline
        );
    }

    #[test]
    fn landlord_obligations_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationsToMaintainPremisesUnderTca66_28_302;
        let output = check(&input);
        assert_eq!(output.mode, TnUrltaMode::CompliantLandlordObligationsMet);
    }

    #[test]
    fn landlord_obligations_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationsToMaintainPremisesUnderTca66_28_302;
        input.landlord_obligations_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationLandlordObligationsBreached
        );
    }

    #[test]
    fn landlord_entry_reasonable_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordAccessUnderTca66_28_403;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantLandlordEntryReasonableNoticeOrEmergencyOrFinalThirtyDayShowingNotice
        );
    }

    #[test]
    fn landlord_entry_unreasonable_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordAccessUnderTca66_28_403;
        input.landlord_entry_with_reasonable_notice_or_emergency_or_final_thirty_days_showing =
            false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationLandlordEntryUnreasonableAndNotEmergencyAndNotFinalThirtyDayShowingNotice
        );
    }

    #[test]
    fn fourteen_day_cure_or_quit_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FourteenDayCureOrQuitNoticeUnderTca66_28_505;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantFourteenDayCureOrQuitNoticeProvided
        );
    }

    #[test]
    fn cure_or_quit_notice_under_14_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FourteenDayCureOrQuitNoticeUnderTca66_28_505;
        input.pay_or_quit_or_cure_notice_days_given = 13;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationCureOrQuitNoticeShorterThanFourteenDays
        );
    }

    #[test]
    fn seven_day_no_cure_notice_for_second_violation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SevenDayNoCureNoticeForSecondViolationUnderTca66_28_505AB;
        input.breach_category = BreachCategory::SecondViolationWithinSixMonths;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantSevenDayNoCureNoticeForSecondViolationProvided
        );
    }

    #[test]
    fn seven_day_no_cure_notice_under_7_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SevenDayNoCureNoticeForSecondViolationUnderTca66_28_505AB;
        input.breach_category = BreachCategory::SecondViolationWithinSixMonths;
        input.second_violation_no_cure_notice_days_given = 6;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationSecondViolationNoticeShorterThanSevenDays
        );
    }

    #[test]
    fn periodic_tenancy_30_day_termination_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::PeriodicTenancyTerminationThirtyDayNoticeUnderTca66_28_512;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantPeriodicTenancyThirtyDayTerminationNoticeProvided
        );
    }

    #[test]
    fn periodic_tenancy_termination_under_30_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::PeriodicTenancyTerminationThirtyDayNoticeUnderTca66_28_512;
        input.periodic_tenancy_termination_notice_days_given = 29;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationPeriodicTenancyTerminationNoticeShorterThanThirtyDays
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliatoryConductProhibitedUnderTca66_28_514;
        let output = check(&input);
        assert_eq!(output.mode, TnUrltaMode::CompliantNoRetaliatoryConduct);
    }

    #[test]
    fn retaliatory_conduct_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliatoryConductProhibitedUnderTca66_28_514;
        input.retaliatory_conduct_occurred = true;
        let output = check(&input);
        assert_eq!(output.mode, TnUrltaMode::ViolationRetaliatoryConduct);
    }

    #[test]
    fn landlord_repair_within_14_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordRepairFourteenDayResponseUnderTca66_28_502;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantLandlordRepairWithinFourteenDayResponse
        );
    }

    #[test]
    fn landlord_repair_at_exactly_14_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordRepairFourteenDayResponseUnderTca66_28_502;
        input.landlord_repair_response_days = 14;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::CompliantLandlordRepairWithinFourteenDayResponse
        );
    }

    #[test]
    fn landlord_repair_at_15_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordRepairFourteenDayResponseUnderTca66_28_502;
        input.landlord_repair_response_days = 15;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TnUrltaMode::ViolationLandlordRepairExceededFourteenDayDeadline
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(TN_URLTA_TITLE_NUMBER, 66);
        assert_eq!(TN_URLTA_CHAPTER_NUMBER, 28);
        assert_eq!(TN_URLTA_ENACTMENT_YEAR, 1975);
        assert_eq!(TN_URLTA_ENABLING_PUBLIC_ACT_CHAPTER, 245);
        assert_eq!(TN_URLTA_POPULATION_APPLICABILITY_THRESHOLD, 75_000);
        assert_eq!(TN_URLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(TN_URLTA_SECURITY_DEPOSIT_UNCLAIMED_WINDOW_DAYS, 60);
        assert_eq!(TN_URLTA_PAY_OR_QUIT_OR_CURE_NOTICE_DAYS, 14);
        assert_eq!(TN_URLTA_SECOND_VIOLATION_NO_CURE_NOTICE_DAYS, 7);
        assert_eq!(TN_URLTA_SECOND_VIOLATION_LOOKBACK_MONTHS, 6);
        assert_eq!(TN_URLTA_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS, 30);
        assert_eq!(TN_URLTA_FINAL_30_DAYS_SHOWING_NOTICE_HOURS, 24);
        assert_eq!(TN_URLTA_LANDLORD_REPAIR_RESPONSE_DEADLINE_DAYS, 14);
        assert_eq!(TN_URLTA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Tennessee Uniform Residential Landlord and Tenant Act"));
        assert!(joined.contains("Public Acts 1975, Chapter 245"));
        assert!(joined.contains("T.C.A. §§ 66-28-101 through 66-28-523"));
        assert!(joined.contains("§ 66-28-301"));
        assert!(joined.contains("§ 66-28-302"));
        assert!(joined.contains("§ 66-28-403"));
        assert!(joined.contains("§ 66-28-502"));
        assert!(joined.contains("§ 66-28-505(a)(2)"));
        assert!(joined.contains("§ 66-28-505(a)(2)(B)"));
        assert!(joined.contains("§ 66-28-512"));
        assert!(joined.contains("§ 66-28-514"));
        assert!(joined.contains("75,000"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("60 DAYS"));
        assert!(joined.contains("14-DAY"));
        assert!(joined.contains("7-DAY"));
        assert!(joined.contains("30-DAY"));
        assert!(joined.contains("24 HOURS"));
        assert!(joined.contains("SEPARATE, FEDERALLY INSURED BANK ACCOUNT"));
        assert!(joined.contains("Davidson"));
        assert!(joined.contains("Shelby"));
        assert!(joined.contains("Knox"));
        assert!(joined.contains("Hamilton"));
        assert!(joined.contains("EMERGENCY"));
    }
}
