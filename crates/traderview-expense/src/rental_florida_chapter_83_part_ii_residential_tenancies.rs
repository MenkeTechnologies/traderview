//! Florida Statute Chapter 83 Part II — Residential
//! Tenancies Compliance Module. Pure-compute check for
//! trader-landlord compliance with the foundational Florida
//! statewide residential tenancy regime codified at **Fla.
//! Stat. §§ 83.40 through 83.683** (Title VI Civil Practice
//! and Procedure, Chapter 83 Landlord and Tenant, Part II
//! Residential Tenancies).
//!
//! Florida is the **third-largest residential rental market**
//! in the United States; Chapter 83 Part II (also known as
//! the **Florida Residential Landlord and Tenant Act**) is
//! the statewide-uniform floor for security deposits +
//! habitability / maintenance + landlord access + notice
//! requirements + retaliation prohibition + servicemember
//! protections. Companion to the existing
//! `rental_florida_hb_1417_state_preemption` module — HB
//! 1417 of 2023 preempts local ordinances that exceed Part
//! II, making Chapter 83 Part II both the statewide floor
//! AND ceiling for Florida residential tenancies.
//!
//! Web research (verified 2026-06-03):
//! - **Florida Residential Landlord and Tenant Act**: Florida statewide residential tenancy regime codified at **Fla. Stat. §§ 83.40 through 83.683** (Title VI Civil Practice and Procedure, Chapter 83 Landlord and Tenant, Part II Residential Tenancies) ([Florida Senate — Chapter 83 Part II Index (2025)](https://www.leg.state.fl.us/statutes/index.cfm?App_mode=Display_Statute&URL=0000-0099%2F0083%2F0083PARTIIContentsIndex.html); [Florida Senate — Chapter 83 (2025) All](https://www.flsenate.gov/Laws/Statutes/2025/Chapter83/All); [Florida Senate — Chapter 83 Part II (2023)](https://www.flsenate.gov/Laws/Statutes/2023/Chapter83/Part_II); [Florida Senate — § 83.49 (2023)](https://www.flsenate.gov/laws/statutes/2023/83.49); [Florida Suncoast PM — Florida Landlord and Tenant Act 2024 PDF](https://floridasuncoastpropertymanagement.com/wp-content/uploads/2025/03/Florida-Landlord-and-Tenant-Act-2024.pdf); [Florida Bar — Rights and Duties of Tenants and Landlords](https://www.floridabar.org/public/consumer/tip014/); [Justia — 2025 Florida Statutes § 83.49 Deposit Money or Advance Rent](https://law.justia.com/codes/florida/title-vi/chapter-83/part-ii/section-83-49/); [American Apartment Owners Association — Florida Landlord Tenant Law Chapter 83 Statutes & Rental Rights](https://american-apartment-owners-association.org/landlord-tenant-laws/florida/); [Lowenhaupt Sawyers & Spinale — Chapter 83 Part II](https://fl-landlord.com/statutes/chapter-83-part-ii/); [Castro Potts Law — Requirements for Deductions and the Return of Security Deposits for Residential Tenants in Florida](https://castropottslaw.com/requirements-for-security-deposit-deductions-and-the-return-of-security-deposits-for-residential-tenancies-in-florida/); [Kelley Grant & Tanis — West Palm Beach Landlord's Guide to Florida Statute 83.56](https://kelleygrantlaw.com/west-palm-beach-landlords-guide-to-florida-statute-83-56-handling-security-deposits-legally/); [True Patriot PM — Florida Landlord Tenant Law: 2026 Cheat Sheet](https://truepatriotpropertymanagement.com/florida-landlord-tenant-law-cheat-sheet/); [AMG Rents — Florida Statute Chapter 83 Tenant and Landlord Guide](https://www.amgrents.com/kissimmee-property-management-blog/understanding-florida-statute-83-a-guide); [FL ELaws — Chapter 83 Title VI](https://fl.elaws.us/law/titlevi_chapter83); [Florida Courts — Landlord and Tenant Forms Instructions PDF](https://flcourts-media.flcourts.gov/content/download/241621/file/92023a3.pdf)).
//! - **§ 83.49 Security Deposit / Advance Rent**: landlord must return full deposit within **15 DAYS** after tenant vacates if no claim; OR send written notice of intent to impose claim within **30 DAYS** after tenant vacates; tenant has **15 DAYS** after receipt of landlord's notice to object to the claim; failure of tenant to object within 15-day window means landlord may collect claim and mail remaining deposit (if any).
//! - **§ 83.51 Landlord Obligation to Maintain Premises**: landlord shall at all times during the tenancy comply with applicable building, housing, and health codes; absent applicable codes, maintain roofs + windows + doors + floors + steps + porches + exterior walls + foundations + structural components in good repair AND plumbing in reasonable working condition; functioning hot water + heating + safe wiring + smoke detectors + extermination required.
//! - **§ 83.53 Landlord Access**: landlord may enter dwelling unit to inspect + repair + decorate + alter + improve + supply agreed services + exhibit to prospective purchasers / mortgagees / tenants / workers / contractors; **REASONABLE NOTICE for repair = at least 24 HOURS prior to entry**; **REASONABLE TIME = between 7:30 AM and 8:00 PM**.
//! - **§ 83.56 Termination of Rental Agreement**: **§ 83.56(3) — 3-day pay or quit notice** for nonpayment of rent (excluding Saturdays, Sundays, and legal holidays); **§ 83.56(2)(a) — 7-day notice to cure or vacate** for material noncompliance other than nonpayment; **§ 83.57 — 15-day notice for month-to-month no-fault non-renewal** by either party.
//! - **§ 83.55 Wrongful Retention Remedies**: tenant may file action for damages + injunctive relief if landlord wrongfully retains deposit.
//! - **§ 83.595 Wrongful Termination Damages**: landlord wrongful breach of rental agreement OR wrongful termination = tenant entitled to damages + 1 month's rent in some cases.
//! - **§ 83.64 Retaliatory Conduct Prohibited**: landlord may not retaliate against tenant who has (a) complained to governmental agency about suspected building / housing / health code violation; (b) organized tenants' organization; (c) complained to landlord; (d) is servicemember terminating rental agreement; (e) paid rent to condo or HOA; OR (f) exercised rights under local / state / federal fair housing laws; **PRESUMPTION of retaliation arises within 1 YEAR** of protected activity.
//! - **§ 83.682 Termination by Servicemember**: servicemember may terminate rental agreement without penalty for (a) permanent change of station orders to relocate more than 35 miles from rental premises; (b) prematurely or involuntarily discharged or released from active duty; (c) released from active duty after receiving orders to relocate more than 35 miles; (d) ordered to military housing; (e) receipt of military orders requiring move-out before rental agreement begins; (f) being temporarily but indefinitely assigned to a new duty station with no return to original duty station; (g) dies during active duty (estate may terminate).
//! - **HB 1417 of 2023 Statewide Preemption** (companion module — built as `rental_florida_hb_1417_state_preemption`): Florida localities may NOT enact landlord-tenant regulations more stringent than Chapter 83 Part II; Chapter 83 Part II is therefore BOTH the statewide floor AND the ceiling for Florida residential tenancies.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const FLA_STAT_CHAPTER_NUMBER: u32 = 83;
pub const FLA_STAT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS_NO_CLAIM: u32 = 15;
pub const FLA_STAT_SECURITY_DEPOSIT_CLAIM_NOTICE_DEADLINE_DAYS: u32 = 30;
pub const FLA_STAT_TENANT_OBJECTION_WINDOW_DAYS: u32 = 15;
pub const FLA_STAT_PAY_OR_QUIT_NOTICE_DAYS: u32 = 3;
pub const FLA_STAT_CURE_OR_VACATE_NOTICE_DAYS: u32 = 7;
pub const FLA_STAT_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS: u32 = 15;
pub const FLA_STAT_LANDLORD_ENTRY_NOTICE_HOURS: u32 = 24;
pub const FLA_STAT_REASONABLE_TIME_START_HOUR_24H: u32 = 7;
pub const FLA_STAT_REASONABLE_TIME_END_HOUR_24H: u32 = 20;
pub const FLA_STAT_RETALIATION_PRESUMPTION_WINDOW_MONTHS: u32 = 12;
pub const FLA_STAT_SERVICEMEMBER_TERMINATION_RELOCATION_MILES_THRESHOLD: u32 = 35;
pub const FLA_STAT_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialDwellingUnitCoveredByChapter83PartII,
    TransientLodgingOccupancyUnder6MonthsExempt,
    CommercialTenancyUnderChapter83PartIExempt,
    InstitutionalMedicalOrJailExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositReturnFifteenDayNoClaimUnderSection83_49,
    SecurityDepositClaimNoticeThirtyDayUnderSection83_49,
    LandlordObligationToMaintainPremisesUnderSection83_51,
    LandlordEntryTwentyFourHourNoticeAndReasonableTimeUnderSection83_53,
    ThreeDayPayOrQuitNoticeUnderSection83_56_3,
    SevenDayCureOrVacateNoticeUnderSection83_56_2A,
    FifteenDayMonthToMonthTerminationUnderSection83_57,
    RetaliatoryConductProhibitedOneYearPresumptionUnderSection83_64,
    ServicemamberTerminationUnderSection83_682,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlaStat83PartIIMode {
    NotApplicableTenancyExemptFromChapter83PartII,
    CompliantSecurityDepositReturnedWithinFifteenDays,
    CompliantSecurityDepositClaimNoticeWithinThirtyDays,
    CompliantPremisesMaintainedToCodeUnderSection83_51,
    CompliantLandlordEntryTwentyFourHourNoticeAndReasonableTime,
    CompliantThreeDayPayOrQuitNoticeProvided,
    CompliantSevenDayCureOrVacateNoticeProvided,
    CompliantFifteenDayMonthToMonthTerminationNoticeProvided,
    CompliantNoRetaliationWithinOneYearOfProtectedActivity,
    CompliantServicemamberTerminationGrantedWithoutPenalty,
    ViolationSecurityDepositReturnedPastFifteenDayDeadline,
    ViolationSecurityDepositClaimNoticeProvidedPastThirtyDayDeadline,
    ViolationPremisesNotMaintainedToApplicableCode,
    ViolationLandlordEntryWithoutTwentyFourHourNoticeOrOutsideReasonableTime,
    ViolationPayOrQuitNoticePeriodShorterThanThreeDays,
    ViolationCureOrVacateNoticePeriodShorterThanSevenDays,
    ViolationMonthToMonthTerminationNoticeShorterThanFifteenDays,
    ViolationRetaliatoryConductWithinOneYearOfProtectedActivity,
    ViolationServicemamberTerminationDeniedDespiteQualifyingMilitaryCircumstance,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub days_since_tenant_vacated_for_deposit_return: u32,
    pub deposit_refunded_or_claim_notice_provided_within_window: bool,
    pub days_since_tenant_vacated_for_claim_notice: u32,
    pub premises_maintained_to_applicable_code: bool,
    pub landlord_entry_notice_hours_given: u32,
    pub entry_within_reasonable_time_window: bool,
    pub pay_or_quit_notice_days_given: u32,
    pub cure_or_vacate_notice_days_given: u32,
    pub month_to_month_termination_notice_days_given: u32,
    pub protected_activity_within_one_year: bool,
    pub adverse_action_taken: bool,
    pub servicemamber_qualifying_circumstance_present: bool,
    pub servicemamber_termination_granted_without_penalty: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: FlaStat83PartIIMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalFloridaChapter83PartIIResidentialTenanciesInput = Input;
pub type RentalFloridaChapter83PartIIResidentialTenanciesOutput = Output;
pub type RentalFloridaChapter83PartIIResidentialTenanciesResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Florida Residential Landlord and Tenant Act — Fla. Stat. §§ 83.40 through 83.683 (Title VI Civil Practice and Procedure, Chapter 83 Landlord and Tenant, Part II Residential Tenancies). Florida statewide residential tenancy regime — third-largest US residential rental market.".to_string(),
        "Fla. Stat. § 83.49 Deposit Money or Advance Rent — landlord must return full deposit within 15 DAYS after tenant vacates if no claim; OR send written notice of intent to impose claim within 30 DAYS after tenant vacates; tenant has 15 DAYS after receipt of landlord's notice to object to the claim; failure of tenant to object within 15-day window means landlord may collect claim and mail remaining deposit (if any)".to_string(),
        "Fla. Stat. § 83.51 Landlord Obligation to Maintain Premises — landlord shall at all times during the tenancy comply with applicable building, housing, and health codes; absent applicable codes, maintain roofs + windows + doors + floors + steps + porches + exterior walls + foundations + structural components in good repair AND plumbing in reasonable working condition; functioning hot water + heating + safe wiring + smoke detectors + extermination required".to_string(),
        "Fla. Stat. § 83.53 Landlord Access to Dwelling Unit — landlord may enter to inspect + repair + decorate + alter + improve + supply agreed services + exhibit to prospective purchasers / mortgagees / tenants / workers / contractors; REASONABLE NOTICE for repair = at least 24 HOURS prior to entry; REASONABLE TIME = between 7:30 AM and 8:00 PM".to_string(),
        "Fla. Stat. § 83.56(3) Termination for Nonpayment — 3-DAY pay or quit notice (excluding Saturdays, Sundays, and legal holidays)".to_string(),
        "Fla. Stat. § 83.56(2)(a) Termination for Material Noncompliance Other Than Nonpayment — 7-DAY notice to cure or vacate".to_string(),
        "Fla. Stat. § 83.57 Month-to-Month No-Fault Termination — 15-DAY notice by either party for non-renewal of month-to-month tenancy".to_string(),
        "Fla. Stat. § 83.55 Wrongful Retention Remedies — tenant may file action for damages + injunctive relief if landlord wrongfully retains deposit".to_string(),
        "Fla. Stat. § 83.595 Wrongful Termination Damages — landlord wrongful breach of rental agreement OR wrongful termination = tenant entitled to damages + 1 month's rent in some cases".to_string(),
        "Fla. Stat. § 83.64 Retaliatory Conduct — landlord may not retaliate against tenant who has (a) complained to governmental agency about suspected building / housing / health code violation; (b) organized tenants' organization; (c) complained to landlord; (d) is servicemember terminating rental agreement; (e) paid rent to condo or HOA; OR (f) exercised rights under local / state / federal fair housing laws; PRESUMPTION of retaliation arises within 1 YEAR of protected activity".to_string(),
        "Fla. Stat. § 83.682 Termination by Servicemember — servicemember may terminate rental agreement without penalty for (a) permanent change of station orders to relocate more than 35 miles; (b) prematurely or involuntarily discharged or released from active duty; (c) released from active duty after orders to relocate more than 35 miles; (d) ordered to military housing; (e) receipt of military orders requiring move-out before rental agreement begins; (f) temporarily but indefinitely assigned to a new duty station with no return; (g) dies during active duty (estate may terminate)".to_string(),
        "Florida HB 1417 of 2023 Statewide Preemption (built as rental_florida_hb_1417_state_preemption companion module) — Florida localities may NOT enact landlord-tenant regulations more stringent than Chapter 83 Part II; Chapter 83 Part II is therefore BOTH the statewide floor AND the ceiling for Florida residential tenancies".to_string(),
        "The Florida Bar — Rights and Duties of Tenants and Landlords — official enforcement guide for Chapter 83".to_string(),
        "Florida Senate — Chapter 83 Part II Index (2025) + Florida Statutes Online — primary statutory text".to_string(),
        "Florida Suncoast Property Management — Florida Landlord and Tenant Act 2024 PDF — practitioner-distributed annotated text".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialDwellingUnitCoveredByChapter83PartII {
        return Output {
            mode: FlaStat83PartIIMode::NotApplicableTenancyExemptFromChapter83PartII,
            statutory_basis: "Fla. Stat. § 83.42 — Chapter 83 Part II applies only to residential tenancies; transient lodging under 6 months / commercial / institutional medical or jail exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from Chapter 83 Part II (transient lodging under 6 months; commercial tenancy under Chapter 83 Part I; institutional medical or correctional facility occupancy).".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositReturnFifteenDayNoClaimUnderSection83_49 => {
            if input.deposit_refunded_or_claim_notice_provided_within_window
                && input.days_since_tenant_vacated_for_deposit_return
                    <= FLA_STAT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS_NO_CLAIM
            {
                Output {
                    mode: FlaStat83PartIIMode::CompliantSecurityDepositReturnedWithinFifteenDays,
                    statutory_basis: "Fla. Stat. § 83.49 — security deposit returned within 15-day statutory deadline (no claim asserted)".to_string(),
                    notes: "COMPLIANT: landlord returned full security deposit within 15-day window after tenant vacated; no claim against deposit asserted.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::ViolationSecurityDepositReturnedPastFifteenDayDeadline,
                    statutory_basis: "Fla. Stat. § 83.49 — security deposit not returned within 15-day statutory deadline".to_string(),
                    notes: "VIOLATION: landlord missed 15-day deposit return deadline under § 83.49; § 83.55 wrongful retention remedies attach (tenant may file action for damages + injunctive relief).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositClaimNoticeThirtyDayUnderSection83_49 => {
            if input.deposit_refunded_or_claim_notice_provided_within_window
                && input.days_since_tenant_vacated_for_claim_notice
                    <= FLA_STAT_SECURITY_DEPOSIT_CLAIM_NOTICE_DEADLINE_DAYS
            {
                Output {
                    mode: FlaStat83PartIIMode::CompliantSecurityDepositClaimNoticeWithinThirtyDays,
                    statutory_basis: "Fla. Stat. § 83.49 — written notice of intent to impose claim provided within 30-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord provided written notice of intent to impose claim against security deposit within 30-day window after tenant vacated; tenant has 15 days after receipt to object.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::ViolationSecurityDepositClaimNoticeProvidedPastThirtyDayDeadline,
                    statutory_basis: "Fla. Stat. § 83.49 — claim notice not provided within 30-day statutory deadline".to_string(),
                    notes: "VIOLATION: landlord missed 30-day claim notice deadline under § 83.49; § 83.55 wrongful retention remedies attach.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordObligationToMaintainPremisesUnderSection83_51 => {
            if input.premises_maintained_to_applicable_code {
                Output {
                    mode: FlaStat83PartIIMode::CompliantPremisesMaintainedToCodeUnderSection83_51,
                    statutory_basis: "Fla. Stat. § 83.51 — premises maintained to applicable building / housing / health codes".to_string(),
                    notes: "COMPLIANT: landlord maintains premises in good repair and in compliance with applicable building / housing / health codes; structural components and plumbing in reasonable working condition.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::ViolationPremisesNotMaintainedToApplicableCode,
                    statutory_basis: "Fla. Stat. § 83.51 — premises not maintained to applicable code".to_string(),
                    notes: "VIOLATION: landlord failed to maintain premises to applicable building / housing / health codes; § 83.56(1) tenant remedies trigger (7-day notice + termination right + damages + injunctive relief).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordEntryTwentyFourHourNoticeAndReasonableTimeUnderSection83_53 => {
            if input.landlord_entry_notice_hours_given >= FLA_STAT_LANDLORD_ENTRY_NOTICE_HOURS
                && input.entry_within_reasonable_time_window
            {
                Output {
                    mode: FlaStat83PartIIMode::CompliantLandlordEntryTwentyFourHourNoticeAndReasonableTime,
                    statutory_basis: "Fla. Stat. § 83.53 — landlord entry with at least 24-hour notice during reasonable time (7:30 AM-8:00 PM)".to_string(),
                    notes: "COMPLIANT: landlord provided at least 24-hour notice prior to entry AND entered within reasonable time window of 7:30 AM to 8:00 PM under § 83.53.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::ViolationLandlordEntryWithoutTwentyFourHourNoticeOrOutsideReasonableTime,
                    statutory_basis: "Fla. Stat. § 83.53 — landlord entry without 24-hour notice or outside reasonable time window".to_string(),
                    notes: "VIOLATION: landlord entered without 24-hour notice OR outside reasonable time window of 7:30 AM to 8:00 PM under § 83.53; tenant may seek injunctive relief.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::ThreeDayPayOrQuitNoticeUnderSection83_56_3 => {
            if input.pay_or_quit_notice_days_given >= FLA_STAT_PAY_OR_QUIT_NOTICE_DAYS {
                Output {
                    mode: FlaStat83PartIIMode::CompliantThreeDayPayOrQuitNoticeProvided,
                    statutory_basis: "Fla. Stat. § 83.56(3) — 3-day pay or quit notice provided for nonpayment".to_string(),
                    notes: "COMPLIANT: landlord provided 3-day pay or quit written notice under § 83.56(3) for nonpayment of rent; 3-day window excludes Saturdays, Sundays, and legal holidays.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::ViolationPayOrQuitNoticePeriodShorterThanThreeDays,
                    statutory_basis: "Fla. Stat. § 83.56(3) — pay or quit notice period shorter than 3-day statutory minimum".to_string(),
                    notes: "VIOLATION: pay or quit notice period shorter than 3-day statutory minimum under § 83.56(3); eviction action subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SevenDayCureOrVacateNoticeUnderSection83_56_2A => {
            if input.cure_or_vacate_notice_days_given >= FLA_STAT_CURE_OR_VACATE_NOTICE_DAYS {
                Output {
                    mode: FlaStat83PartIIMode::CompliantSevenDayCureOrVacateNoticeProvided,
                    statutory_basis: "Fla. Stat. § 83.56(2)(a) — 7-day cure or vacate notice provided for material noncompliance".to_string(),
                    notes: "COMPLIANT: landlord provided 7-day cure or vacate written notice under § 83.56(2)(a) for material noncompliance other than nonpayment.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::ViolationCureOrVacateNoticePeriodShorterThanSevenDays,
                    statutory_basis: "Fla. Stat. § 83.56(2)(a) — cure or vacate notice period shorter than 7-day statutory minimum".to_string(),
                    notes: "VIOLATION: cure or vacate notice period shorter than 7-day statutory minimum under § 83.56(2)(a); eviction action subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::FifteenDayMonthToMonthTerminationUnderSection83_57 => {
            if input.month_to_month_termination_notice_days_given
                >= FLA_STAT_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS
            {
                Output {
                    mode: FlaStat83PartIIMode::CompliantFifteenDayMonthToMonthTerminationNoticeProvided,
                    statutory_basis: "Fla. Stat. § 83.57 — 15-day notice provided for month-to-month no-fault termination".to_string(),
                    notes: "COMPLIANT: party provided 15-day written notice under § 83.57 for month-to-month no-fault non-renewal.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::ViolationMonthToMonthTerminationNoticeShorterThanFifteenDays,
                    statutory_basis: "Fla. Stat. § 83.57 — month-to-month termination notice shorter than 15-day statutory minimum".to_string(),
                    notes: "VIOLATION: month-to-month termination notice shorter than 15-day statutory minimum under § 83.57; termination invalid.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::RetaliatoryConductProhibitedOneYearPresumptionUnderSection83_64 => {
            if input.protected_activity_within_one_year && input.adverse_action_taken {
                Output {
                    mode: FlaStat83PartIIMode::ViolationRetaliatoryConductWithinOneYearOfProtectedActivity,
                    statutory_basis: "Fla. Stat. § 83.64 — retaliatory conduct within 1-year presumption window".to_string(),
                    notes: "VIOLATION: landlord engaged in adverse action (rent raise / service reduction / eviction / non-renewal) within 1-year retaliation presumption window after tenant's protected activity (governmental agency complaint / tenants' organization / landlord complaint / servicemember termination / condo or HOA rent payment / fair housing rights assertion).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::CompliantNoRetaliationWithinOneYearOfProtectedActivity,
                    statutory_basis: "Fla. Stat. § 83.64 — no retaliatory conduct presumption arises".to_string(),
                    notes: "COMPLIANT: no adverse action within 1-year retaliation window OR no protected tenant activity to trigger § 83.64 presumption.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::ServicemamberTerminationUnderSection83_682 => {
            if input.servicemamber_qualifying_circumstance_present
                && input.servicemamber_termination_granted_without_penalty
            {
                Output {
                    mode: FlaStat83PartIIMode::CompliantServicemamberTerminationGrantedWithoutPenalty,
                    statutory_basis: "Fla. Stat. § 83.682 — servicemember termination granted without penalty for qualifying military circumstance".to_string(),
                    notes: "COMPLIANT: landlord granted servicemember termination without penalty under § 83.682 for qualifying military circumstance (PCS orders ≥ 35 miles / discharge / military housing / pre-move-in orders / temporary indefinite reassignment / death during active duty).".to_string(),
                    citations,
                }
            } else if input.servicemamber_qualifying_circumstance_present {
                Output {
                    mode: FlaStat83PartIIMode::ViolationServicemamberTerminationDeniedDespiteQualifyingMilitaryCircumstance,
                    statutory_basis: "Fla. Stat. § 83.682 — servicemember termination denied despite qualifying military circumstance".to_string(),
                    notes: "VIOLATION: landlord refused servicemember termination despite qualifying military circumstance under § 83.682; servicemember may seek damages + injunctive relief + SCRA cross-protection (Servicemembers Civil Relief Act, 50 USC § 3955).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: FlaStat83PartIIMode::CompliantServicemamberTerminationGrantedWithoutPenalty,
                    statutory_basis: "Fla. Stat. § 83.682 — no qualifying military circumstance present".to_string(),
                    notes: "NOT TRIGGERED: no qualifying military circumstance present under § 83.682; servicemember termination right does not apply.".to_string(),
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
            tenancy_type: TenancyType::ResidentialDwellingUnitCoveredByChapter83PartII,
            compliance_aspect:
                ComplianceAspect::SecurityDepositReturnFifteenDayNoClaimUnderSection83_49,
            days_since_tenant_vacated_for_deposit_return: 14,
            deposit_refunded_or_claim_notice_provided_within_window: true,
            days_since_tenant_vacated_for_claim_notice: 28,
            premises_maintained_to_applicable_code: true,
            landlord_entry_notice_hours_given: 24,
            entry_within_reasonable_time_window: true,
            pay_or_quit_notice_days_given: 3,
            cure_or_vacate_notice_days_given: 7,
            month_to_month_termination_notice_days_given: 15,
            protected_activity_within_one_year: false,
            adverse_action_taken: false,
            servicemamber_qualifying_circumstance_present: true,
            servicemamber_termination_granted_without_penalty: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::TransientLodgingOccupancyUnder6MonthsExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::NotApplicableTenancyExemptFromChapter83PartII
        );
    }

    #[test]
    fn security_deposit_returned_within_fifteen_days_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantSecurityDepositReturnedWithinFifteenDays
        );
    }

    #[test]
    fn security_deposit_at_exactly_fifteen_day_boundary_compliant() {
        let mut input = baseline_input();
        input.days_since_tenant_vacated_for_deposit_return = 15;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantSecurityDepositReturnedWithinFifteenDays
        );
    }

    #[test]
    fn security_deposit_at_sixteen_days_violation() {
        let mut input = baseline_input();
        input.days_since_tenant_vacated_for_deposit_return = 16;
        input.deposit_refunded_or_claim_notice_provided_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationSecurityDepositReturnedPastFifteenDayDeadline
        );
    }

    #[test]
    fn claim_notice_within_thirty_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositClaimNoticeThirtyDayUnderSection83_49;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantSecurityDepositClaimNoticeWithinThirtyDays
        );
    }

    #[test]
    fn claim_notice_at_exactly_thirty_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositClaimNoticeThirtyDayUnderSection83_49;
        input.days_since_tenant_vacated_for_claim_notice = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantSecurityDepositClaimNoticeWithinThirtyDays
        );
    }

    #[test]
    fn claim_notice_at_thirty_one_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositClaimNoticeThirtyDayUnderSection83_49;
        input.days_since_tenant_vacated_for_claim_notice = 31;
        input.deposit_refunded_or_claim_notice_provided_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationSecurityDepositClaimNoticeProvidedPastThirtyDayDeadline
        );
    }

    #[test]
    fn premises_maintained_to_code_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToMaintainPremisesUnderSection83_51;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantPremisesMaintainedToCodeUnderSection83_51
        );
    }

    #[test]
    fn premises_not_maintained_to_code_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToMaintainPremisesUnderSection83_51;
        input.premises_maintained_to_applicable_code = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationPremisesNotMaintainedToApplicableCode
        );
    }

    #[test]
    fn landlord_entry_with_twenty_four_hour_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordEntryTwentyFourHourNoticeAndReasonableTimeUnderSection83_53;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantLandlordEntryTwentyFourHourNoticeAndReasonableTime
        );
    }

    #[test]
    fn landlord_entry_under_twenty_four_hours_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordEntryTwentyFourHourNoticeAndReasonableTimeUnderSection83_53;
        input.landlord_entry_notice_hours_given = 12;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationLandlordEntryWithoutTwentyFourHourNoticeOrOutsideReasonableTime
        );
    }

    #[test]
    fn landlord_entry_outside_reasonable_time_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordEntryTwentyFourHourNoticeAndReasonableTimeUnderSection83_53;
        input.entry_within_reasonable_time_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationLandlordEntryWithoutTwentyFourHourNoticeOrOutsideReasonableTime
        );
    }

    #[test]
    fn three_day_pay_or_quit_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeDayPayOrQuitNoticeUnderSection83_56_3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantThreeDayPayOrQuitNoticeProvided
        );
    }

    #[test]
    fn pay_or_quit_under_three_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeDayPayOrQuitNoticeUnderSection83_56_3;
        input.pay_or_quit_notice_days_given = 2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationPayOrQuitNoticePeriodShorterThanThreeDays
        );
    }

    #[test]
    fn seven_day_cure_or_vacate_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SevenDayCureOrVacateNoticeUnderSection83_56_2A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantSevenDayCureOrVacateNoticeProvided
        );
    }

    #[test]
    fn cure_or_vacate_under_seven_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SevenDayCureOrVacateNoticeUnderSection83_56_2A;
        input.cure_or_vacate_notice_days_given = 6;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationCureOrVacateNoticePeriodShorterThanSevenDays
        );
    }

    #[test]
    fn fifteen_day_month_to_month_termination_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::FifteenDayMonthToMonthTerminationUnderSection83_57;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantFifteenDayMonthToMonthTerminationNoticeProvided
        );
    }

    #[test]
    fn month_to_month_termination_under_fifteen_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::FifteenDayMonthToMonthTerminationUnderSection83_57;
        input.month_to_month_termination_notice_days_given = 14;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationMonthToMonthTerminationNoticeShorterThanFifteenDays
        );
    }

    #[test]
    fn retaliation_within_one_year_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::RetaliatoryConductProhibitedOneYearPresumptionUnderSection83_64;
        input.protected_activity_within_one_year = true;
        input.adverse_action_taken = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationRetaliatoryConductWithinOneYearOfProtectedActivity
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::RetaliatoryConductProhibitedOneYearPresumptionUnderSection83_64;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantNoRetaliationWithinOneYearOfProtectedActivity
        );
    }

    #[test]
    fn servicemamber_termination_granted_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ServicemamberTerminationUnderSection83_682;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantServicemamberTerminationGrantedWithoutPenalty
        );
    }

    #[test]
    fn servicemamber_termination_denied_despite_qualifying_circumstance_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ServicemamberTerminationUnderSection83_682;
        input.servicemamber_termination_granted_without_penalty = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::ViolationServicemamberTerminationDeniedDespiteQualifyingMilitaryCircumstance
        );
    }

    #[test]
    fn servicemamber_no_qualifying_circumstance_not_triggered() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ServicemamberTerminationUnderSection83_682;
        input.servicemamber_qualifying_circumstance_present = false;
        input.servicemamber_termination_granted_without_penalty = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FlaStat83PartIIMode::CompliantServicemamberTerminationGrantedWithoutPenalty
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(FLA_STAT_CHAPTER_NUMBER, 83);
        assert_eq!(FLA_STAT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS_NO_CLAIM, 15);
        assert_eq!(FLA_STAT_SECURITY_DEPOSIT_CLAIM_NOTICE_DEADLINE_DAYS, 30);
        assert_eq!(FLA_STAT_TENANT_OBJECTION_WINDOW_DAYS, 15);
        assert_eq!(FLA_STAT_PAY_OR_QUIT_NOTICE_DAYS, 3);
        assert_eq!(FLA_STAT_CURE_OR_VACATE_NOTICE_DAYS, 7);
        assert_eq!(FLA_STAT_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS, 15);
        assert_eq!(FLA_STAT_LANDLORD_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(FLA_STAT_REASONABLE_TIME_START_HOUR_24H, 7);
        assert_eq!(FLA_STAT_REASONABLE_TIME_END_HOUR_24H, 20);
        assert_eq!(FLA_STAT_RETALIATION_PRESUMPTION_WINDOW_MONTHS, 12);
        assert_eq!(
            FLA_STAT_SERVICEMEMBER_TERMINATION_RELOCATION_MILES_THRESHOLD,
            35
        );
        assert_eq!(FLA_STAT_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Florida Residential Landlord and Tenant Act"));
        assert!(joined.contains("Fla. Stat. §§ 83.40 through 83.683"));
        assert!(joined.contains("§ 83.49"));
        assert!(joined.contains("§ 83.51"));
        assert!(joined.contains("§ 83.53"));
        assert!(joined.contains("§ 83.55"));
        assert!(joined.contains("§ 83.56(3)"));
        assert!(joined.contains("§ 83.56(2)(a)"));
        assert!(joined.contains("§ 83.57"));
        assert!(joined.contains("§ 83.595"));
        assert!(joined.contains("§ 83.64"));
        assert!(joined.contains("§ 83.682"));
        assert!(joined.contains("15 DAYS"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("3-DAY"));
        assert!(joined.contains("7-DAY"));
        assert!(joined.contains("15-DAY"));
        assert!(joined.contains("24 HOURS"));
        assert!(joined.contains("7:30 AM and 8:00 PM"));
        assert!(joined.contains("1 YEAR"));
        assert!(joined.contains("35 miles"));
        assert!(joined.contains("HB 1417"));
    }
}
