//! Texas Property Code Chapter 92 — Residential Tenancies
//! Compliance Module. Pure-compute check for trader-landlord
//! compliance with the foundational Texas statewide
//! residential tenancy regime, covering Subchapters A
//! through H of Texas Property Code Title 8 Chapter 92.
//!
//! Texas is the **second-largest residential rental market**
//! in the United States; Chapter 92 is the statewide-uniform
//! floor for security deposits, habitability / repair duty,
//! tenant repair-and-deduct remedies, smoke alarm / fire
//! extinguisher requirements, and landlord retaliation
//! prohibition. Companion to the existing
//! `rental_texas_hb_2127_state_preemption` module (which
//! prohibits localities from imposing landlord requirements
//! beyond state law) — Chapter 92 IS the Texas statewide
//! floor that HB 2127 enforces as the ceiling.
//!
//! Web research (verified 2026-06-03):
//! - **Texas Property Code Chapter 92 — Residential Tenancies**: Texas statewide residential tenancy regime under Title 8 (Landlord and Tenant), Chapter 92. Codified at **Tex. Prop. Code §§ 92.001 through 92.355** ([Texas Statutes — Property Code Chapter 92 Residential Tenancies](https://statutes.capitol.texas.gov/Docs/PR/htm/PR.92.htm); [Texas Statutes — Property Code Chapter 92 PDF](https://statutes.capitol.texas.gov/docs/pr/pdf/pr.92.pdf); [Public.Law — Texas Property Code Chapter 92 Residential Tenancies](https://texas.public.law/statutes/tex._prop._code_title_8_chapter_92); [Texas Attorney General — Renter's Rights](https://www.texasattorneygeneral.gov/consumer-protection/home-real-estate-and-travel/renters-rights); [Texas State Law Library — Security Deposits](https://guides.sll.texas.gov/landlord-tenant-law/security-deposits); [Texas State Law Library — Security Deposit Refunds](https://guides.sll.texas.gov/landlord-tenant-law/security-deposit-refunds); [Texas State Law Library — Does my landlord have to make repairs?](https://www.sll.texas.gov/faqs/landlord-duty-to-repair/); [Justia 2024 — § 92.103 Obligation to Refund](https://law.justia.com/codes/texas/property-code/title-8/chapter-92/subchapter-c/section-92-103/); [Justia 2025 — § 92.056 Landlord Liability and Tenant Remedies; Notice and Time for Repair](https://law.justia.com/codes/texas/property-code/title-8/chapter-92/subchapter-b/section-92-056/); [Justia 2024 — Subchapter F Smoke Alarms and Fire Extinguishers](https://law.justia.com/codes/texas/property-code/title-8/chapter-92/subchapter-f/); [FindLaw — Texas Property Code § 92.056](https://codes.findlaw.com/tx/property-code/prop-sect-92-056/); [FindLaw — Texas Property Code § 92.0561 Tenant's Repair and Deduct Remedies](https://codes.findlaw.com/tx/property-code/prop-sect-92-0561/); [LoneStarLandLaw — Security Deposits in Texas Residential Leases](https://lonestarlandlaw.com/security-deposits-in-texas-residential-leases/); [Public.Law — § 92.056 Landlord Liability and Tenant Remedies; Notice and Time for Repair](https://texas.public.law/statutes/tex._prop._code_section_92.056); [Texas Tenant Advisor — Smoke Detectors](https://www.texastenant.org/while-you-are-renting/smoke-detectors); [Azibo — Understanding Texas Landlord Repair Laws](https://www.azibo.com/blog/texas-landlord-repair-laws); [Flat Fee Landlord — Texas Property Code Chapter 92 Landlord Guide](https://flatfeelandlord.com/blog/texas-property-code-chapter-92-landlord-guide)).
//! - **§ 92.052 Landlord's Duty to Repair**: landlord must make a diligent effort to repair or remedy a condition if the tenant specifies the condition in a notice and the condition **MATERIALLY AFFECTS THE PHYSICAL HEALTH OR SAFETY OF AN ORDINARY TENANT**; landlord NOT obligated to repair conditions caused by tenant, lawful occupant, household member, or guest; landlord NOT obligated to repair conditions arising from normal wear and tear.
//! - **§ 92.056 Landlord Liability and Tenant Remedies**: tenant must give landlord written notice + reasonable time to repair (**presumption of 7 days** unless circumstances make different period reasonable); tenant remedies if landlord fails: (a) terminate lease; (b) sue for actual damages + attorney fees; (c) repair-and-deduct under § 92.0561; (d) one month's rent + $500 + actual damages + attorney fees.
//! - **§ 92.0561 Tenant's Repair and Deduct Remedy**: tenant may have the condition repaired and **DEDUCT FROM RENT** up to the GREATER OF (a) **ONE MONTH'S RENT** OR (b) **$500**, in any one month; repairs and deductions may be made as often as necessary so long as the monthly cap is respected.
//! - **§ 92.103 Security Deposit Return Deadline**: landlord must refund security deposit on or before the **30TH DAY** after the date the tenant surrenders the premises; this is 30 calendar days, NOT 30 business days.
//! - **§ 92.104 Itemized Deductions**: if the landlord makes deductions from the security deposit, landlord must provide tenant with a **WRITTEN ITEMIZED LIST OF DEDUCTIONS** (when tenant has paid all rent and there is no controversy over rent).
//! - **§ 92.109 Bad Faith Retention of Security Deposit**: a landlord who in bad faith retains all or part of a security deposit is liable for the amount wrongfully withheld + **$100** + **THREE TIMES the portion of the deposit wrongfully withheld** + reasonable attorney fees + court costs.
//! - **§ 92.156 Smoke Alarm Installation, Inspection, Repair**: landlord must install + inspect + repair smoke alarms; tenant request → landlord must install / inspect / repair within **7 DAYS**; tenant may NOT waive these provisions; tenant may NOT disconnect or disable a smoke detector.
//! - **§ 92.260 Tenant Remedies for Smoke Alarm Failure**: tenant may file lawsuit OR **TERMINATE LEASE WITHOUT COURT PROCEEDINGS** if landlord does not install / inspect / repair within 7 days of tenant request.
//! - **§ 92.331 Retaliation Prohibited**: landlord may NOT retaliate against tenant within **6 MONTHS** after a tenant's protected activity — good-faith complaint to landlord or government agency about necessary repairs + good-faith assertion of tenant rights under Chapter 92 + good-faith establishment / participation in tenant organization.
//! - **§ 92.335 Retaliation Remedies**: $500 statutory damages + one month's rent + actual damages + attorney fees + injunctive relief.
//! - **HB 2127 Statewide Preemption (companion module)**: localities may NOT impose residential tenancy requirements MORE STRINGENT than Chapter 92; Chapter 92 is therefore both the floor AND the ceiling for Texas residential tenancies. Built as `rental_texas_hb_2127_state_preemption`.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const TX_PROP_CODE_CHAPTER_NUMBER: u32 = 92;
pub const TX_PROP_CODE_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const TX_PROP_CODE_REPAIR_AND_DEDUCT_DOLLAR_FLOOR: u64 = 500;
pub const TX_PROP_CODE_REPAIR_NOTICE_REASONABLE_TIME_DAYS_PRESUMPTION: u32 = 7;
pub const TX_PROP_CODE_SMOKE_ALARM_REPAIR_DEADLINE_DAYS: u32 = 7;
pub const TX_PROP_CODE_RETALIATION_PRESUMPTION_WINDOW_MONTHS: u32 = 6;
pub const TX_PROP_CODE_BAD_FAITH_RETENTION_STATUTORY_FLAT_PENALTY_DOLLARS: u64 = 100;
pub const TX_PROP_CODE_BAD_FAITH_RETENTION_TREBLE_DAMAGES_MULTIPLIER: u64 = 3;
pub const TX_PROP_CODE_RETALIATION_STATUTORY_DAMAGES_DOLLARS: u64 = 500;
pub const TX_PROP_CODE_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalCoveredByChapter92,
    HotelMotelOrShortTermTransientLodgingExempt,
    CommercialRentalExempt,
    InstitutionalCareFacilityExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositReturnThirtyDayDeadlineUnderSection92_103,
    SecurityDepositItemizedDeductionsUnderSection92_104,
    SecurityDepositBadFaithRetentionTrebleDamagesUnderSection92_109,
    LandlordDutyToRepairMaterialHealthSafetyUnderSection92_052,
    TenantRepairAndDeductCapUnderSection92_0561,
    LandlordRepairNoticeReasonableTimeUnderSection92_056,
    SmokeAlarmInstallationAndSevenDayRepairUnderSection92_156,
    SmokeAlarmFailureTenantRemediesUnderSection92_260,
    RetaliationProhibitedSixMonthWindowUnderSection92_331,
    RetaliationStatutoryRemediesUnderSection92_335,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TexPropCode92Mode {
    NotApplicableTenancyExemptFromChapter92,
    CompliantSecurityDepositReturnedWithinThirtyDays,
    CompliantSecurityDepositItemizedDeductionsProvided,
    CompliantSecurityDepositNoBadFaithRetention,
    CompliantLandlordRepairDutySatisfiedForHealthSafetyCondition,
    CompliantTenantRepairAndDeductWithinGreaterOfMonthRentOr500Cap,
    CompliantLandlordRepairWithinReasonableTimeSevenDayPresumption,
    CompliantSmokeAlarmInstalledAndInspectedAndRepairedWithinSevenDays,
    CompliantSmokeAlarmInPlaceTenantNoTerminationGrounds,
    CompliantNoRetaliationWithinSixMonthsOfProtectedActivity,
    ViolationSecurityDepositReturnedPastThirtyDayDeadline,
    ViolationSecurityDepositNoItemizedDeductionsWhenRequired,
    ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlus100,
    ViolationLandlordFailedToRepairMaterialHealthSafetyCondition,
    ViolationTenantRepairAndDeductExceedsGreaterOfMonthRentOr500Cap,
    ViolationLandlordRepairNoticeBelowReasonableTimeSevenDayPresumption,
    ViolationSmokeAlarmNotInstalledOrNotRepairedWithinSevenDays,
    ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub monthly_rent_dollars: u64,
    pub security_deposit_dollars: u64,
    pub days_since_tenant_surrendered_for_deposit_return: u32,
    pub deposit_refunded_on_or_before_thirty_days: bool,
    pub itemized_deductions_provided_when_required: bool,
    pub bad_faith_retention_of_deposit: bool,
    pub portion_of_deposit_wrongfully_withheld_dollars: u64,
    pub condition_materially_affects_physical_health_or_safety: bool,
    pub landlord_repaired_within_reasonable_time: bool,
    pub days_since_tenant_repair_notice: u32,
    pub repair_and_deduct_amount_dollars: u64,
    pub smoke_alarm_installed_and_repaired_within_seven_days: bool,
    pub days_since_smoke_alarm_repair_request: u32,
    pub protected_tenant_activity_occurred: bool,
    pub adverse_action_within_six_months_of_protected_activity: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: TexPropCode92Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub bad_faith_retention_treble_plus_100_dollars: u64,
    pub retaliation_statutory_damages_plus_one_month_rent_dollars: u64,
}

pub type RentalTexasPropertyCodeChapter92ResidentialTenanciesInput = Input;
pub type RentalTexasPropertyCodeChapter92ResidentialTenanciesOutput = Output;
pub type RentalTexasPropertyCodeChapter92ResidentialTenanciesResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Tex. Prop. Code Chapter 92 — Residential Tenancies. Title 8 (Landlord and Tenant), Chapter 92. Codified at Tex. Prop. Code §§ 92.001 through 92.355. Texas statewide residential tenancy regime providing floor for security deposit + habitability + tenant remedies + smoke alarm + retaliation. Companion: HB 2127 (statewide preemption — Chapter 92 is BOTH floor AND ceiling).".to_string(),
        "Tex. Prop. Code § 92.052 Landlord's Duty to Repair — landlord must make diligent effort to repair a condition that MATERIALLY AFFECTS THE PHYSICAL HEALTH OR SAFETY OF AN ORDINARY TENANT after written tenant notice; landlord NOT obligated to repair conditions caused by tenant / occupant / household member / guest; landlord NOT obligated to repair conditions arising from normal wear and tear".to_string(),
        "Tex. Prop. Code § 92.056 Landlord Liability and Tenant Remedies; Notice and Time for Repair — tenant must give written notice + reasonable time to repair; PRESUMPTION OF 7 DAYS as reasonable absent circumstances; tenant remedies: terminate lease + sue for actual damages + attorney fees + § 92.0561 repair-and-deduct + one month's rent + $500 + actual damages + attorney fees".to_string(),
        "Tex. Prop. Code § 92.0561 Tenant's Repair and Deduct Remedy — tenant may have condition repaired and deduct from rent up to GREATER OF (a) ONE MONTH'S RENT OR (b) $500 in any one month; repairs and deductions may be made as often as necessary so long as the monthly cap is respected".to_string(),
        "Tex. Prop. Code § 92.103 Security Deposit Obligation to Refund — landlord must refund security deposit on or before the 30TH DAY after the date the tenant surrenders the premises (30 CALENDAR days, not 30 business days)".to_string(),
        "Tex. Prop. Code § 92.104 Security Deposit Itemized List of Deductions — landlord must provide tenant with WRITTEN ITEMIZED LIST OF DEDUCTIONS when tenant has paid all rent and there is no controversy over rent".to_string(),
        "Tex. Prop. Code § 92.109 Liability for Withholding Last Month's Rent or Security Deposit in Bad Faith — landlord acting in BAD FAITH liable for: (a) $100; (b) THREE TIMES the portion of the deposit wrongfully withheld; (c) reasonable attorney's fees; (d) court costs".to_string(),
        "Tex. Prop. Code § 92.156 Smoke Alarms — landlord must INSTALL + INSPECT + REPAIR smoke alarms; tenant may NOT waive these provisions; tenant may NOT disconnect or disable a smoke detector; landlord must install / inspect / repair within 7 DAYS of tenant request".to_string(),
        "Tex. Prop. Code § 92.260 Tenant Remedies for Smoke Alarm Failure — if landlord does not install / inspect / repair within 7 days of tenant request, tenant may file lawsuit OR TERMINATE LEASE WITHOUT COURT PROCEEDINGS".to_string(),
        "Tex. Prop. Code § 92.331 Retaliation by Landlord Prohibited — landlord may NOT retaliate by raising rent / decreasing services / commencing eviction / terminating tenancy within 6 MONTHS of tenant's good-faith complaint about necessary repairs / good-faith assertion of Chapter 92 rights / good-faith establishment or participation in tenant organization".to_string(),
        "Tex. Prop. Code § 92.335 Retaliation Remedies — $500 statutory damages + ONE MONTH'S RENT + actual damages + attorney fees + injunctive relief".to_string(),
        "HB 2127 Texas Statewide Preemption (built as rental_texas_hb_2127_state_preemption companion module) — localities may NOT impose residential tenancy requirements MORE STRINGENT than Chapter 92; Chapter 92 is therefore both FLOOR and CEILING for Texas residential tenancies".to_string(),
        "Texas Attorney General — Renter's Rights — official enforcement guide for Chapter 92".to_string(),
        "Texas State Law Library — Security Deposits + Security Deposit Refunds + Landlord Duty to Repair FAQs — practitioner guides".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalCoveredByChapter92 {
        return Output {
            mode: TexPropCode92Mode::NotApplicableTenancyExemptFromChapter92,
            statutory_basis: "Tex. Prop. Code § 92.001 et seq. — Chapter 92 applies only to residential leaseholds; transient lodging / commercial / institutional care exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from Chapter 92 (hotel/motel/short-term transient lodging; commercial rental; institutional care facility).".to_string(),
            citations,
            bad_faith_retention_treble_plus_100_dollars: 0,
            retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
        };
    }

    let repair_and_deduct_cap = input
        .monthly_rent_dollars
        .max(TX_PROP_CODE_REPAIR_AND_DEDUCT_DOLLAR_FLOOR);

    let bad_faith_remedy = input
        .portion_of_deposit_wrongfully_withheld_dollars
        .saturating_mul(TX_PROP_CODE_BAD_FAITH_RETENTION_TREBLE_DAMAGES_MULTIPLIER)
        .saturating_add(TX_PROP_CODE_BAD_FAITH_RETENTION_STATUTORY_FLAT_PENALTY_DOLLARS);

    let retaliation_remedy = TX_PROP_CODE_RETALIATION_STATUTORY_DAMAGES_DOLLARS
        .saturating_add(input.monthly_rent_dollars);

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositReturnThirtyDayDeadlineUnderSection92_103 => {
            if input.deposit_refunded_on_or_before_thirty_days
                && input.days_since_tenant_surrendered_for_deposit_return
                    <= TX_PROP_CODE_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: TexPropCode92Mode::CompliantSecurityDepositReturnedWithinThirtyDays,
                    statutory_basis: "Tex. Prop. Code § 92.103 — security deposit refunded within 30-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord refunded security deposit on or before the 30th day after tenant surrendered the premises (30 calendar days, not 30 business days).".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::ViolationSecurityDepositReturnedPastThirtyDayDeadline,
                    statutory_basis: "Tex. Prop. Code § 92.103 — security deposit not refunded within 30-day statutory deadline".to_string(),
                    notes: "VIOLATION: landlord missed 30-day security deposit refund deadline under § 92.103; failure to refund creates presumption of bad faith retention under § 92.108 if landlord did not provide written description and itemized list of deductions before 30-day deadline.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: bad_faith_remedy,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositItemizedDeductionsUnderSection92_104 => {
            if input.itemized_deductions_provided_when_required {
                Output {
                    mode: TexPropCode92Mode::CompliantSecurityDepositItemizedDeductionsProvided,
                    statutory_basis: "Tex. Prop. Code § 92.104 — landlord provided written itemized list of deductions".to_string(),
                    notes: "COMPLIANT: landlord provided tenant with written itemized list of deductions from security deposit as required by § 92.104 (when tenant has paid all rent and there is no controversy over rent).".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::ViolationSecurityDepositNoItemizedDeductionsWhenRequired,
                    statutory_basis: "Tex. Prop. Code § 92.104 — landlord failed to provide written itemized list of deductions".to_string(),
                    notes: "VIOLATION: landlord failed to provide written itemized list of deductions as required by § 92.104; presumption of bad faith retention under § 92.109 triggers $100 + 3× wrongfully withheld + attorney fees exposure.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: bad_faith_remedy,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderSection92_109 => {
            if input.bad_faith_retention_of_deposit {
                Output {
                    mode: TexPropCode92Mode::ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlus100,
                    statutory_basis: "Tex. Prop. Code § 92.109 — bad faith retention of security deposit triggers $100 + 3× wrongfully withheld + attorney fees".to_string(),
                    notes: "VIOLATION: landlord acted in BAD FAITH in retaining all or part of security deposit; § 92.109 remedies = $100 + 3× portion wrongfully withheld + reasonable attorney's fees + court costs.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: bad_faith_remedy,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::CompliantSecurityDepositNoBadFaithRetention,
                    statutory_basis: "Tex. Prop. Code § 92.109 — no bad faith retention".to_string(),
                    notes: "COMPLIANT: landlord did not engage in bad faith retention of security deposit; § 92.109 treble damages remedy does not attach.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::LandlordDutyToRepairMaterialHealthSafetyUnderSection92_052 => {
            if !input.condition_materially_affects_physical_health_or_safety {
                Output {
                    mode: TexPropCode92Mode::CompliantLandlordRepairDutySatisfiedForHealthSafetyCondition,
                    statutory_basis: "Tex. Prop. Code § 92.052 — repair duty triggered only by conditions materially affecting health or safety".to_string(),
                    notes: "COMPLIANT: condition does NOT materially affect physical health or safety of ordinary tenant; § 92.052 repair duty not triggered.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else if input.landlord_repaired_within_reasonable_time {
                Output {
                    mode: TexPropCode92Mode::CompliantLandlordRepairDutySatisfiedForHealthSafetyCondition,
                    statutory_basis: "Tex. Prop. Code § 92.052 — landlord made diligent effort to repair material health/safety condition".to_string(),
                    notes: "COMPLIANT: condition materially affects physical health or safety of ordinary tenant AND landlord made diligent effort to repair within reasonable time under § 92.052.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::ViolationLandlordFailedToRepairMaterialHealthSafetyCondition,
                    statutory_basis: "Tex. Prop. Code § 92.052 + § 92.056 — landlord failed to repair material health/safety condition; tenant remedies trigger".to_string(),
                    notes: "VIOLATION: condition materially affects physical health or safety of ordinary tenant AND landlord did not make diligent effort to repair within reasonable time; tenant remedies under § 92.056 (lease termination + damages + § 92.0561 repair-and-deduct + one month's rent + $500 + attorney fees).".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::TenantRepairAndDeductCapUnderSection92_0561 => {
            if input.repair_and_deduct_amount_dollars <= repair_and_deduct_cap {
                Output {
                    mode: TexPropCode92Mode::CompliantTenantRepairAndDeductWithinGreaterOfMonthRentOr500Cap,
                    statutory_basis: "Tex. Prop. Code § 92.0561 — tenant repair-and-deduct within greater of one month's rent or $500 monthly cap".to_string(),
                    notes: "COMPLIANT: tenant repair-and-deduct amount within the statutory monthly cap of GREATER OF one month's rent OR $500.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::ViolationTenantRepairAndDeductExceedsGreaterOfMonthRentOr500Cap,
                    statutory_basis: "Tex. Prop. Code § 92.0561 — tenant repair-and-deduct exceeds greater-of-month-rent-or-$500 monthly cap".to_string(),
                    notes: "VIOLATION: tenant repair-and-deduct amount exceeds the statutory monthly cap of GREATER OF one month's rent OR $500; landlord may pursue rent collection for excess.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::LandlordRepairNoticeReasonableTimeUnderSection92_056 => {
            if input.days_since_tenant_repair_notice
                <= TX_PROP_CODE_REPAIR_NOTICE_REASONABLE_TIME_DAYS_PRESUMPTION
                && input.landlord_repaired_within_reasonable_time
            {
                Output {
                    mode: TexPropCode92Mode::CompliantLandlordRepairWithinReasonableTimeSevenDayPresumption,
                    statutory_basis: "Tex. Prop. Code § 92.056 — landlord repaired within 7-day reasonable-time presumption".to_string(),
                    notes: "COMPLIANT: landlord repaired condition within the 7-day reasonable-time presumption under § 92.056.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::ViolationLandlordRepairNoticeBelowReasonableTimeSevenDayPresumption,
                    statutory_basis: "Tex. Prop. Code § 92.056 — landlord did not repair within 7-day reasonable-time presumption; tenant remedies trigger".to_string(),
                    notes: "VIOLATION: landlord did not repair within the 7-day reasonable-time presumption under § 92.056; tenant remedies including lease termination, repair-and-deduct, $500 + one month's rent + actual damages + attorney fees attach.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::SmokeAlarmInstallationAndSevenDayRepairUnderSection92_156 => {
            if input.smoke_alarm_installed_and_repaired_within_seven_days
                && input.days_since_smoke_alarm_repair_request
                    <= TX_PROP_CODE_SMOKE_ALARM_REPAIR_DEADLINE_DAYS
            {
                Output {
                    mode: TexPropCode92Mode::CompliantSmokeAlarmInstalledAndInspectedAndRepairedWithinSevenDays,
                    statutory_basis: "Tex. Prop. Code § 92.156 — smoke alarm installed and repaired within 7-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord installed / inspected / repaired smoke alarm within the 7-day deadline after tenant request under § 92.156; provisions not waivable by tenant; tenant may not disconnect or disable smoke detector.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::ViolationSmokeAlarmNotInstalledOrNotRepairedWithinSevenDays,
                    statutory_basis: "Tex. Prop. Code § 92.156 + § 92.260 — smoke alarm not installed / repaired within 7-day deadline; tenant termination right".to_string(),
                    notes: "VIOLATION: landlord did not install / inspect / repair smoke alarm within the 7-day deadline; § 92.260 tenant remedies = file lawsuit OR terminate lease without court proceedings.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::SmokeAlarmFailureTenantRemediesUnderSection92_260 => Output {
            mode: TexPropCode92Mode::CompliantSmokeAlarmInPlaceTenantNoTerminationGrounds,
            statutory_basis: "Tex. Prop. Code § 92.260 — smoke alarm in place; tenant has no extrajudicial termination grounds".to_string(),
            notes: "COMPLIANT: smoke alarm is in place and landlord has not failed § 92.156 obligations; tenant has no § 92.260 extrajudicial termination grounds.".to_string(),
            citations,
            bad_faith_retention_treble_plus_100_dollars: 0,
            retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
        },
        ComplianceAspect::RetaliationProhibitedSixMonthWindowUnderSection92_331 => {
            if input.protected_tenant_activity_occurred
                && input.adverse_action_within_six_months_of_protected_activity
            {
                Output {
                    mode: TexPropCode92Mode::ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity,
                    statutory_basis: "Tex. Prop. Code § 92.331 + § 92.335 — retaliatory conduct within 6 months of protected tenant activity".to_string(),
                    notes: "VIOLATION: landlord engaged in adverse action (rent raise / service reduction / eviction commencement / tenancy termination) within 6 months of tenant's good-faith protected activity (repair complaint / Chapter 92 rights assertion / tenant organization participation); § 92.335 remedies = $500 statutory damages + ONE MONTH'S RENT + actual damages + attorney fees + injunctive relief.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: retaliation_remedy,
                }
            } else {
                Output {
                    mode: TexPropCode92Mode::CompliantNoRetaliationWithinSixMonthsOfProtectedActivity,
                    statutory_basis: "Tex. Prop. Code § 92.331 — no retaliatory conduct presumption arises".to_string(),
                    notes: "COMPLIANT: no adverse action within 6-month retaliation window OR no protected tenant activity to trigger § 92.331 presumption.".to_string(),
                    citations,
                    bad_faith_retention_treble_plus_100_dollars: 0,
                    retaliation_statutory_damages_plus_one_month_rent_dollars: 0,
                }
            }
        }
        ComplianceAspect::RetaliationStatutoryRemediesUnderSection92_335 => Output {
            mode: TexPropCode92Mode::CompliantNoRetaliationWithinSixMonthsOfProtectedActivity,
            statutory_basis: "Tex. Prop. Code § 92.335 — retaliation remedies framework".to_string(),
            notes: "INFORMATIONAL: § 92.335 remedies framework if landlord violates § 92.331 = $500 statutory damages + ONE MONTH'S RENT + actual damages + attorney fees + injunctive relief.".to_string(),
            citations,
            bad_faith_retention_treble_plus_100_dollars: 0,
            retaliation_statutory_damages_plus_one_month_rent_dollars: retaliation_remedy,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_type: TenancyType::ResidentialRentalCoveredByChapter92,
            compliance_aspect:
                ComplianceAspect::SecurityDepositReturnThirtyDayDeadlineUnderSection92_103,
            monthly_rent_dollars: 2_000,
            security_deposit_dollars: 2_000,
            days_since_tenant_surrendered_for_deposit_return: 30,
            deposit_refunded_on_or_before_thirty_days: true,
            itemized_deductions_provided_when_required: true,
            bad_faith_retention_of_deposit: false,
            portion_of_deposit_wrongfully_withheld_dollars: 0,
            condition_materially_affects_physical_health_or_safety: true,
            landlord_repaired_within_reasonable_time: true,
            days_since_tenant_repair_notice: 7,
            repair_and_deduct_amount_dollars: 2_000,
            smoke_alarm_installed_and_repaired_within_seven_days: true,
            days_since_smoke_alarm_repair_request: 7,
            protected_tenant_activity_occurred: false,
            adverse_action_within_six_months_of_protected_activity: false,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::HotelMotelOrShortTermTransientLodgingExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::NotApplicableTenancyExemptFromChapter92
        );
    }

    #[test]
    fn security_deposit_returned_within_thirty_days_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantSecurityDepositReturnedWithinThirtyDays
        );
    }

    #[test]
    fn security_deposit_returned_at_exactly_thirty_day_boundary_compliant() {
        let mut input = baseline_input();
        input.days_since_tenant_surrendered_for_deposit_return = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantSecurityDepositReturnedWithinThirtyDays
        );
    }

    #[test]
    fn security_deposit_returned_past_thirty_day_violation() {
        let mut input = baseline_input();
        input.days_since_tenant_surrendered_for_deposit_return = 35;
        input.deposit_refunded_on_or_before_thirty_days = false;
        input.portion_of_deposit_wrongfully_withheld_dollars = 1_500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationSecurityDepositReturnedPastThirtyDayDeadline
        );
        assert_eq!(
            output.bad_faith_retention_treble_plus_100_dollars,
            1_500 * 3 + 100
        );
    }

    #[test]
    fn itemized_deductions_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositItemizedDeductionsUnderSection92_104;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantSecurityDepositItemizedDeductionsProvided
        );
    }

    #[test]
    fn no_itemized_deductions_when_required_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositItemizedDeductionsUnderSection92_104;
        input.itemized_deductions_provided_when_required = false;
        input.portion_of_deposit_wrongfully_withheld_dollars = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationSecurityDepositNoItemizedDeductionsWhenRequired
        );
        assert_eq!(
            output.bad_faith_retention_treble_plus_100_dollars,
            500 * 3 + 100
        );
    }

    #[test]
    fn bad_faith_retention_treble_damages_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderSection92_109;
        input.bad_faith_retention_of_deposit = true;
        input.portion_of_deposit_wrongfully_withheld_dollars = 1_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlus100
        );
        assert_eq!(
            output.bad_faith_retention_treble_plus_100_dollars,
            1_000 * 3 + 100
        );
    }

    #[test]
    fn no_bad_faith_retention_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderSection92_109;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantSecurityDepositNoBadFaithRetention
        );
    }

    #[test]
    fn landlord_duty_to_repair_health_safety_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordDutyToRepairMaterialHealthSafetyUnderSection92_052;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantLandlordRepairDutySatisfiedForHealthSafetyCondition
        );
    }

    #[test]
    fn landlord_failed_to_repair_health_safety_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordDutyToRepairMaterialHealthSafetyUnderSection92_052;
        input.landlord_repaired_within_reasonable_time = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationLandlordFailedToRepairMaterialHealthSafetyCondition
        );
    }

    #[test]
    fn non_material_condition_no_duty_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordDutyToRepairMaterialHealthSafetyUnderSection92_052;
        input.condition_materially_affects_physical_health_or_safety = false;
        input.landlord_repaired_within_reasonable_time = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantLandlordRepairDutySatisfiedForHealthSafetyCondition
        );
    }

    #[test]
    fn repair_and_deduct_within_one_months_rent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantRepairAndDeductCapUnderSection92_0561;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantTenantRepairAndDeductWithinGreaterOfMonthRentOr500Cap
        );
    }

    #[test]
    fn repair_and_deduct_uses_500_floor_when_rent_below_500() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantRepairAndDeductCapUnderSection92_0561;
        input.monthly_rent_dollars = 300;
        input.repair_and_deduct_amount_dollars = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantTenantRepairAndDeductWithinGreaterOfMonthRentOr500Cap
        );
    }

    #[test]
    fn repair_and_deduct_at_501_when_rent_below_500_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantRepairAndDeductCapUnderSection92_0561;
        input.monthly_rent_dollars = 300;
        input.repair_and_deduct_amount_dollars = 501;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationTenantRepairAndDeductExceedsGreaterOfMonthRentOr500Cap
        );
    }

    #[test]
    fn repair_and_deduct_exceeds_one_months_rent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantRepairAndDeductCapUnderSection92_0561;
        input.repair_and_deduct_amount_dollars = 3_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationTenantRepairAndDeductExceedsGreaterOfMonthRentOr500Cap
        );
    }

    #[test]
    fn landlord_repair_within_seven_day_presumption_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordRepairNoticeReasonableTimeUnderSection92_056;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantLandlordRepairWithinReasonableTimeSevenDayPresumption
        );
    }

    #[test]
    fn landlord_repair_at_exactly_seven_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordRepairNoticeReasonableTimeUnderSection92_056;
        input.days_since_tenant_repair_notice = 7;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantLandlordRepairWithinReasonableTimeSevenDayPresumption
        );
    }

    #[test]
    fn landlord_repair_at_eight_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordRepairNoticeReasonableTimeUnderSection92_056;
        input.days_since_tenant_repair_notice = 8;
        input.landlord_repaired_within_reasonable_time = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationLandlordRepairNoticeBelowReasonableTimeSevenDayPresumption
        );
    }

    #[test]
    fn smoke_alarm_within_seven_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SmokeAlarmInstallationAndSevenDayRepairUnderSection92_156;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantSmokeAlarmInstalledAndInspectedAndRepairedWithinSevenDays
        );
    }

    #[test]
    fn smoke_alarm_not_within_seven_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SmokeAlarmInstallationAndSevenDayRepairUnderSection92_156;
        input.smoke_alarm_installed_and_repaired_within_seven_days = false;
        input.days_since_smoke_alarm_repair_request = 8;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationSmokeAlarmNotInstalledOrNotRepairedWithinSevenDays
        );
    }

    #[test]
    fn retaliation_within_six_months_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::RetaliationProhibitedSixMonthWindowUnderSection92_331;
        input.protected_tenant_activity_occurred = true;
        input.adverse_action_within_six_months_of_protected_activity = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity
        );
        assert_eq!(
            output.retaliation_statutory_damages_plus_one_month_rent_dollars,
            500 + 2_000
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::RetaliationProhibitedSixMonthWindowUnderSection92_331;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::CompliantNoRetaliationWithinSixMonthsOfProtectedActivity
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(TX_PROP_CODE_CHAPTER_NUMBER, 92);
        assert_eq!(TX_PROP_CODE_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(TX_PROP_CODE_REPAIR_AND_DEDUCT_DOLLAR_FLOOR, 500);
        assert_eq!(
            TX_PROP_CODE_REPAIR_NOTICE_REASONABLE_TIME_DAYS_PRESUMPTION,
            7
        );
        assert_eq!(TX_PROP_CODE_SMOKE_ALARM_REPAIR_DEADLINE_DAYS, 7);
        assert_eq!(TX_PROP_CODE_RETALIATION_PRESUMPTION_WINDOW_MONTHS, 6);
        assert_eq!(
            TX_PROP_CODE_BAD_FAITH_RETENTION_STATUTORY_FLAT_PENALTY_DOLLARS,
            100
        );
        assert_eq!(
            TX_PROP_CODE_BAD_FAITH_RETENTION_TREBLE_DAMAGES_MULTIPLIER,
            3
        );
        assert_eq!(TX_PROP_CODE_RETALIATION_STATUTORY_DAMAGES_DOLLARS, 500);
        assert_eq!(TX_PROP_CODE_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Tex. Prop. Code Chapter 92"));
        assert!(joined.contains("§§ 92.001 through 92.355"));
        assert!(joined.contains("§ 92.052"));
        assert!(joined.contains("§ 92.056"));
        assert!(joined.contains("§ 92.0561"));
        assert!(joined.contains("§ 92.103"));
        assert!(joined.contains("§ 92.104"));
        assert!(joined.contains("§ 92.109"));
        assert!(joined.contains("§ 92.156"));
        assert!(joined.contains("§ 92.260"));
        assert!(joined.contains("§ 92.331"));
        assert!(joined.contains("§ 92.335"));
        assert!(joined.contains("30TH DAY"));
        assert!(joined.contains("$500"));
        assert!(joined.contains("$100"));
        assert!(joined.contains("THREE TIMES"));
        assert!(joined.contains("7 DAYS"));
        assert!(joined.contains("6 MONTHS"));
        assert!(joined.contains("ONE MONTH"));
        assert!(joined.contains("HB 2127"));
    }

    #[test]
    fn bad_faith_remedy_saturating_overflow_defense() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositBadFaithRetentionTrebleDamagesUnderSection92_109;
        input.bad_faith_retention_of_deposit = true;
        input.portion_of_deposit_wrongfully_withheld_dollars = u64::MAX;
        let output = check(&input);
        assert_eq!(
            output.mode,
            TexPropCode92Mode::ViolationSecurityDepositBadFaithRetentionTriggersTrebleDamagesPlus100
        );
        assert_eq!(output.bad_faith_retention_treble_plus_100_dollars, u64::MAX);
    }
}
