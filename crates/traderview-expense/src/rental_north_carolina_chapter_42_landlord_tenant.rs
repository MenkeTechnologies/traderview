//! North Carolina General Statutes Chapter 42 (Landlord
//! and Tenant) Compliance Module — codified at N.C. Gen.
//! Stat. §§ 42-1 through 42-86. Pure-compute check for
//! trader-landlord compliance with North Carolina's
//! foundational residential tenancy regime.
//!
//! North Carolina Chapter 42 is structured into Article 1
//! (General Provisions), Article 4A (Ejectment of
//! Residential Tenants), Article 5 (Residential Rental
//! Agreements Act §§ 42-38 through 42-46), Article 6
//! (Tenant Security Deposit Act §§ 42-50 through 42-56),
//! Article 7 (Expedited Eviction of Drug Traffickers),
//! Article 8 (Landlord Damages Cure), and Article 9
//! (Vacation Rental Act). Charlotte and Raleigh-Durham are
//! major Southeastern rental markets; North Carolina is the
//! **9th most-populous US state**.
//!
//! Web research (verified 2026-06-03):
//! - **Codification**: North Carolina Landlord-Tenant Law codified at **N.C. Gen. Stat. §§ 42-1 through 42-86** (Chapter 42 Landlord and Tenant) ([NC General Assembly — Chapter 42 Landlord and Tenant](https://www.ncleg.gov/enactedlegislation/statutes/html/bychapter/chapter_42.html); [NC General Assembly — Chapter 42 General Statute Sections](https://www.ncleg.gov/Laws/GeneralStatuteSections/Chapter42); [NC General Assembly — G.S. 42-46](https://www.ncleg.net/enactedlegislation/statutes/html/bysection/chapter_42/gs_42-46.html); [NC General Assembly — G.S. 42-51](https://www.ncleg.net/enactedlegislation/statutes/html/bysection/chapter_42/gs_42-51.html); [NC General Assembly — G.S. 42-50 PDF](https://www.ncleg.gov/EnactedLegislation/Statutes/PDF/BySection/Chapter_42/GS_42-50.pdf); [NC General Assembly — Chapter 42 Article 6 PDF](https://www.ncleg.net/EnactedLegislation/Statutes/PDF/ByArticle/Chapter_42/Article_6.pdf); [NC General Assembly — G.S. 42-46 PDF](https://www.ncleg.gov/EnactedLegislation/Statutes/PDF/BySection/Chapter_42/GS_42-46.pdf); [Justia — 2024 NC Chapter 42 Article 6 Tenant Security Deposit Act](https://law.justia.com/codes/north-carolina/chapter-42/article-6/); [Justia — 2024 NC § 42-51 Permitted Uses of the Deposit](https://law.justia.com/codes/north-carolina/chapter-42/article-6/section-42-51/); [Justia — 2024 NC § 42-50 Deposits from the Tenant](https://law.justia.com/codes/north-carolina/chapter-42/article-6/section-42-50/); [Justia — 2023 NC § 42-52 Landlord's Obligations](https://law.justia.com/codes/north-carolina/chapter-42/article-6/section-42-52/); [Justia — 2025 NC § 42-46 Authorized Fees, Costs, and Expenses](https://law.justia.com/codes/north-carolina/chapter-42/article-5/section-42-46/); [Justia 2005 — NC § 42-51 Permitted Uses of the Deposit](https://law.justia.com/codes/north-carolina/2005/chapter_42/gs_42-51.html); [Justia 2005 — NC § 42-46 Late Fees](https://law.justia.com/codes/north-carolina/2005/chapter_42/gs_42-46.html); [Justia 2005 — NC Chapter 42 Article 6 Tenant Security Deposit Act](https://law.justia.com/codes/north-carolina/2005/chapter_42/article_6.html); [FindLaw — N.C. Gen. Stat. § 42-46](https://codes.findlaw.com/nc/chapter-42-landlord-and-tenant/nc-gen-st-sect-42-46/); [LegalClarity — North Carolina Rent Late Fee Laws Caps and Grace Period](https://legalclarity.org/north-carolina-rent-late-fee-laws-and-compliance-guidelines/); [Hemlane — North Carolina Eviction Laws 2026 Step-by-Step Process](https://www.hemlane.com/resources/north-carolina-eviction-laws/); [RocketRent — N.C. Gen. Stat. § 42-46](https://rocketrent.com/landlord-tenant-laws/north-carolina/statutes/n-c-gen-stat-%C2%A7-42-46/); [NC Legal Aid Manual on Procedures (NCLAMP) — NC Private Landlord/Tenant Law Overview](https://www.nclamp.gov/media/488559/NC-Landlord-Tenant.PDF)).
//! - **N.C. Gen. Stat. § 42-51 Security Deposit Caps**: tiered caps based on tenancy term — **2 WEEKS' RENT** for week-to-week tenancies; **1.5 MONTHS' RENT** for month-to-month tenancies; **2 MONTHS' RENT** for tenancies with term greater than month-to-month.
//! - **N.C. Gen. Stat. § 42-52 Security Deposit Return**: landlord must mail or deliver to tenant a **WRITTEN ITEMIZED STATEMENT** of any damages along with the balance of the security deposit **WITHIN 30 DAYS** after termination of tenancy and delivery of possession by tenant; if extent of landlord's claim cannot be determined within 30 days, landlord must provide **INTERIM ACCOUNTING within 30 days** AND **FINAL ACCOUNTING within 60 DAYS** after termination; if tenant's address is unknown, landlord shall apply the deposit per § 42-51 after 30 days and hold balance for collection by tenant for at least **6 MONTHS**.
//! - **N.C. Gen. Stat. § 42-50 Deposit Held in Trust or Bond**: security deposits shall be deposited in a **SEPARATE TRUST ACCOUNT** maintained for that purpose in a North Carolina insured banking or savings institution OR the landlord may furnish a **BOND** for the amount of the security deposit from an insurance company licensed to do business in North Carolina.
//! - **N.C. Gen. Stat. § 42-46(a) Late Fees**: for rent due in monthly installments, landlord may charge a late fee NOT TO EXCEED **THE GREATER OF $15.00 OR 5 PERCENT of the monthly rent**; late fee may be imposed only **ONE TIME** for each late rental payment.
//! - **N.C. Gen. Stat. § 42-46(b) Administrative Complaint-Filing Fee**: landlord may charge an **ADMINISTRATIVE COMPLAINT-FILING FEE NOT TO EXCEED THE GREATER OF $15.00 OR 5 PERCENT** of the monthly rent ONLY IF: (1) tenant was in default of the lease; (2) landlord filed and served a complaint for summary ejectment and/or money owed; (3) tenant cured the default or claim; AND (4) landlord dismissed the complaint prior to judgment.
//! - **N.C. Gen. Stat. § 42-42 Landlord Obligation to Maintain**: implied warranty of habitability — landlord shall comply with applicable building and housing codes; make all repairs and do whatever is necessary to put and keep premises in fit and habitable condition; keep common areas safe and sanitary; maintain all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities supplied by landlord in good and safe working order.
//! - **N.C. Gen. Stat. § 42-26 Summary Ejectment Notice**: landlord must first **DEMAND PAYMENT** and then **WAIT 10 DAYS** before filing summary ejectment action in small claims court (magistrate court) for nonpayment of rent.
//! - **N.C. Gen. Stat. § 42-25.6 Self-Help Eviction Prohibited**: it is the public policy of North Carolina that lawful eviction shall be effected only by judicial process; landlord may NOT use self-help measures (lockout, utility shut-off, removal of tenant's property) to evict tenant.
//! - **N.C. Gen. Stat. § 42-7.1 Acceptance of Past-Due Rent**: landlord's acceptance of past-due rent does NOT waive the landlord's right to maintain an eviction action.
//! - **N.C. Gen. Stat. § 42-44 Tenant Remedies for Landlord Noncompliance**: tenant may recover actual damages + attorney's fees in some cases; tenant may NOT withhold rent (unlike most other states) without seeking court order.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const NC_LANDLORD_TENANT_CHAPTER_NUMBER: u32 = 42;
pub const NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_WEEKS_WEEK_TO_WEEK: u32 = 2;
pub const NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_TENTHS_OF_MONTHS_MONTH_TO_MONTH: u64 = 15;
pub const NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS: u64 = 10;
pub const NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_MONTHS_TERM_GREATER_THAN_MONTH_TO_MONTH: u32 = 2;
pub const NC_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const NC_LANDLORD_TENANT_SECURITY_DEPOSIT_FINAL_ACCOUNTING_DEADLINE_DAYS: u32 = 60;
pub const NC_LANDLORD_TENANT_UNKNOWN_ADDRESS_HOLD_PERIOD_MONTHS: u32 = 6;
pub const NC_LANDLORD_TENANT_LATE_FEE_FLAT_DOLLAR_CAP: u64 = 15;
pub const NC_LANDLORD_TENANT_LATE_FEE_PERCENT_CAP_BPS: u64 = 500;
pub const NC_LANDLORD_TENANT_PAY_OR_QUIT_DEMAND_DAYS: u32 = 10;
pub const NC_LANDLORD_TENANT_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalCoveredByChapter42,
    CommercialRentalExempt,
    HotelMotelTransientLodgingExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyTermLength {
    WeekToWeek,
    MonthToMonth,
    TermGreaterThanMonthToMonth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositTrustAccountOrBondUnderGs42_50,
    SecurityDepositCapBasedOnTenancyTermUnderGs42_51,
    SecurityDepositReturn30DayWithItemizedStatementUnderGs42_52,
    SecurityDepositInterimAndFinalAccountingUnderGs42_52,
    LandlordObligationToMaintainPremisesUnderGs42_42,
    LateFeeCapGreaterOf15Or5PercentUnderGs42_46A,
    AdministrativeComplaintFilingFeeCapUnderGs42_46B,
    TenDayPayOrQuitDemandUnderGs42_26,
    SelfHelpEvictionProhibitedUnderGs42_25_6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NcLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter42,
    CompliantSecurityDepositInTrustAccountOrBondProvided,
    CompliantSecurityDepositAtOrBelowStatutoryCapForTenancyTerm,
    CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days,
    CompliantSecurityDepositInterimAccountingAt30DaysAndFinalAt60Days,
    CompliantLandlordObligationsToMaintainPremisesMet,
    CompliantLateFeeAtOrBelowGreaterOf15Or5PercentCap,
    CompliantAdministrativeFeeAtOrBelowGreaterOf15Or5PercentWithRequiredConditions,
    CompliantTenDayPayOrQuitDemandProvided,
    CompliantNoSelfHelpEvictionLawfulJudicialProcessUsed,
    ViolationSecurityDepositNotInTrustAccountOrBond,
    ViolationSecurityDepositExceedsStatutoryCapForTenancyTerm,
    ViolationSecurityDepositReturnedPastThirtyDayDeadline,
    ViolationSecurityDepositInterimOrFinalAccountingDeadlineMissed,
    ViolationLandlordObligationsToMaintainPremisesBreached,
    ViolationLateFeeExceedsGreaterOf15Or5PercentCap,
    ViolationLateFeeChargedMoreThanOncePerLatePayment,
    ViolationAdministrativeFeeChargedWithoutMeetingFourPrerequisites,
    ViolationPayOrQuitDemandShorterThanTenDays,
    ViolationSelfHelpEvictionProhibited,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub tenancy_term_length: TenancyTermLength,
    pub monthly_rent_dollars: u64,
    pub weekly_rent_dollars: u64,
    pub security_deposit_dollars: u64,
    pub deposit_in_separate_trust_account_or_bond: bool,
    pub deposit_returned_with_itemized_statement_within_window: bool,
    pub days_since_tenancy_termination_for_deposit_return: u32,
    pub days_since_tenancy_termination_for_final_accounting: u32,
    pub interim_accounting_provided_at_30_days: bool,
    pub final_accounting_provided_at_60_days: bool,
    pub landlord_repair_and_maintenance_obligations_met: bool,
    pub late_fee_charged_dollars: u64,
    pub late_fee_charged_multiple_times_for_same_late_payment: bool,
    pub administrative_fee_charged_dollars: u64,
    pub administrative_fee_four_prerequisites_met: bool,
    pub pay_or_quit_demand_days_given: u32,
    pub used_self_help_eviction: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: NcLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalNorthCarolinaChapter42LandlordTenantInput = Input;
pub type RentalNorthCarolinaChapter42LandlordTenantOutput = Output;
pub type RentalNorthCarolinaChapter42LandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "North Carolina Landlord-Tenant Law — codified at N.C. Gen. Stat. §§ 42-1 through 42-86 (Chapter 42 Landlord and Tenant). Articles include Article 1 (General Provisions), Article 4A (Ejectment of Residential Tenants), Article 5 (Residential Rental Agreements Act §§ 42-38 through 42-46), Article 6 (Tenant Security Deposit Act §§ 42-50 through 42-56), Article 7 (Expedited Eviction of Drug Traffickers), Article 8 (Landlord Damages Cure), Article 9 (Vacation Rental Act).".to_string(),
        "N.C. Gen. Stat. § 42-50 Deposits from the Tenant — security deposits shall be deposited in a SEPARATE TRUST ACCOUNT maintained for that purpose in a North Carolina insured banking or savings institution OR the landlord may furnish a BOND for the amount of the security deposit from an insurance company licensed to do business in North Carolina".to_string(),
        "N.C. Gen. Stat. § 42-51 Permitted Uses of the Deposit — tiered caps based on tenancy term: 2 WEEKS' RENT for week-to-week tenancies; 1.5 MONTHS' RENT for month-to-month tenancies; 2 MONTHS' RENT for tenancies with term greater than month-to-month; permitted uses include nonpayment of rent + damage to premises beyond normal wear and tear + nonfulfillment of rental period + unpaid bills + cost of removal and storage of tenant's property + court costs".to_string(),
        "N.C. Gen. Stat. § 42-52 Landlord's Obligations — landlord must mail or deliver to tenant a WRITTEN ITEMIZED STATEMENT of any damages along with the balance of the security deposit WITHIN 30 DAYS after termination of tenancy and delivery of possession by tenant; if extent of landlord's claim cannot be determined within 30 days, landlord must provide INTERIM ACCOUNTING within 30 days AND FINAL ACCOUNTING within 60 DAYS after termination; if tenant's address is unknown, landlord shall apply the deposit per § 42-51 after 30 days and hold balance for collection by tenant for at least 6 MONTHS".to_string(),
        "N.C. Gen. Stat. § 42-46(a) Late Fees — for rent due in monthly installments, landlord may charge a late fee NOT TO EXCEED THE GREATER OF $15.00 OR 5 PERCENT of the monthly rent; late fee may be imposed only ONE TIME for each late rental payment".to_string(),
        "N.C. Gen. Stat. § 42-46(b) Administrative Complaint-Filing Fee — landlord may charge an ADMINISTRATIVE COMPLAINT-FILING FEE NOT TO EXCEED THE GREATER OF $15.00 OR 5 PERCENT of the monthly rent ONLY IF: (1) tenant was in default of the lease; (2) landlord filed and served a complaint for summary ejectment and/or money owed; (3) tenant cured the default or claim; AND (4) landlord dismissed the complaint prior to judgment".to_string(),
        "N.C. Gen. Stat. § 42-42 Landlord Obligation to Maintain — implied warranty of habitability; landlord shall comply with applicable building and housing codes; make all repairs to put and keep premises in fit and habitable condition; keep common areas safe and sanitary; maintain all electrical / plumbing / sanitary / heating / ventilating / air-conditioning and other facilities supplied by landlord in good and safe working order".to_string(),
        "N.C. Gen. Stat. § 42-26 Summary Ejectment Notice — landlord must first DEMAND PAYMENT and then WAIT 10 DAYS before filing summary ejectment action in small claims court (magistrate court) for nonpayment of rent".to_string(),
        "N.C. Gen. Stat. § 42-25.6 Self-Help Eviction Prohibited — it is the public policy of North Carolina that lawful eviction shall be effected ONLY by judicial process; landlord may NOT use self-help measures (lockout, utility shut-off, removal of tenant's property) to evict tenant".to_string(),
        "N.C. Gen. Stat. § 42-7.1 Acceptance of Past-Due Rent — landlord's acceptance of past-due rent does NOT waive the landlord's right to maintain an eviction action".to_string(),
        "N.C. Gen. Stat. § 42-44 Tenant Remedies for Landlord Noncompliance — tenant may recover actual damages + attorney's fees in some cases; tenant may NOT withhold rent (unlike most other states) without seeking court order".to_string(),
        "NC General Assembly + Justia + FindLaw + Hemlane + LegalClarity + RocketRent + NC Legal Aid Manual on Procedures (NCLAMP) — primary statutory text and practitioner guides".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalCoveredByChapter42 {
        return Output {
            mode: NcLandlordTenantMode::NotApplicableTenancyExemptFromChapter42,
            statutory_basis: "N.C. Gen. Stat. Chapter 42 applies only to residential leaseholds; commercial / hotel-motel transient lodging exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from N.C. Gen. Stat. Chapter 42 (commercial rental; hotel/motel transient lodging).".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositTrustAccountOrBondUnderGs42_50 => {
            if input.deposit_in_separate_trust_account_or_bond {
                Output {
                    mode: NcLandlordTenantMode::CompliantSecurityDepositInTrustAccountOrBondProvided,
                    statutory_basis: "N.C. Gen. Stat. § 42-50 — security deposit held in separate trust account at NC insured institution OR bond provided from licensed NC insurance company".to_string(),
                    notes: "COMPLIANT: landlord deposited security deposit in separate trust account at North Carolina insured banking or savings institution OR furnished bond from NC-licensed insurance company under § 42-50.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationSecurityDepositNotInTrustAccountOrBond,
                    statutory_basis: "N.C. Gen. Stat. § 42-50 — security deposit not in trust account at NC insured institution AND no bond provided".to_string(),
                    notes: "VIOLATION: landlord did not deposit security deposit in separate trust account at North Carolina insured institution AND did not furnish bond under § 42-50; tenant may seek statutory remedies.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositCapBasedOnTenancyTermUnderGs42_51 => {
            let cap = match input.tenancy_term_length {
                TenancyTermLength::WeekToWeek => input.weekly_rent_dollars.saturating_mul(
                    u64::from(NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_WEEKS_WEEK_TO_WEEK),
                ),
                TenancyTermLength::MonthToMonth => {
                    (u128::from(input.monthly_rent_dollars)
                        * u128::from(
                            NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_TENTHS_OF_MONTHS_MONTH_TO_MONTH,
                        )
                        / u128::from(NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS))
                        as u64
                }
                TenancyTermLength::TermGreaterThanMonthToMonth => input
                    .monthly_rent_dollars
                    .saturating_mul(u64::from(
                    NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_MONTHS_TERM_GREATER_THAN_MONTH_TO_MONTH,
                )),
            };
            if input.security_deposit_dollars <= cap {
                Output {
                    mode: NcLandlordTenantMode::CompliantSecurityDepositAtOrBelowStatutoryCapForTenancyTerm,
                    statutory_basis: "N.C. Gen. Stat. § 42-51 — security deposit at or below statutory cap for tenancy term".to_string(),
                    notes: format!("COMPLIANT: security deposit at or below statutory cap of ${cap} under § 42-51 for applicable tenancy term."),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationSecurityDepositExceedsStatutoryCapForTenancyTerm,
                    statutory_basis: "N.C. Gen. Stat. § 42-51 — security deposit exceeds statutory cap for tenancy term".to_string(),
                    notes: format!("VIOLATION: security deposit exceeds ${cap} statutory cap under § 42-51 for applicable tenancy term (2 weeks' rent for week-to-week; 1.5 months for month-to-month; 2 months for greater terms)."),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturn30DayWithItemizedStatementUnderGs42_52 => {
            if input.deposit_returned_with_itemized_statement_within_window
                && input.days_since_tenancy_termination_for_deposit_return
                    <= NC_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: NcLandlordTenantMode::CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days,
                    statutory_basis: "N.C. Gen. Stat. § 42-52 — security deposit returned with written itemized statement within 30-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord mailed or delivered written itemized statement of damages along with balance of security deposit within 30 days after termination of tenancy and delivery of possession by tenant under § 42-52.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationSecurityDepositReturnedPastThirtyDayDeadline,
                    statutory_basis: "N.C. Gen. Stat. § 42-52 — security deposit not returned with itemized statement within 30-day statutory deadline".to_string(),
                    notes: "VIOLATION: landlord missed 30-day deposit return / itemized statement deadline under § 42-52; tenant may seek recovery of deposit + attorney's fees.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositInterimAndFinalAccountingUnderGs42_52 => {
            if input.interim_accounting_provided_at_30_days
                && input.final_accounting_provided_at_60_days
                && input.days_since_tenancy_termination_for_final_accounting
                    <= NC_LANDLORD_TENANT_SECURITY_DEPOSIT_FINAL_ACCOUNTING_DEADLINE_DAYS
            {
                Output {
                    mode: NcLandlordTenantMode::CompliantSecurityDepositInterimAccountingAt30DaysAndFinalAt60Days,
                    statutory_basis: "N.C. Gen. Stat. § 42-52 — interim accounting at 30 days and final accounting at 60 days provided".to_string(),
                    notes: "COMPLIANT: landlord provided interim accounting within 30 days AND final accounting within 60 days after termination under § 42-52 (when extent of claim cannot be determined within 30 days).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationSecurityDepositInterimOrFinalAccountingDeadlineMissed,
                    statutory_basis: "N.C. Gen. Stat. § 42-52 — interim or final accounting deadline missed".to_string(),
                    notes: "VIOLATION: landlord did not provide interim accounting within 30 days OR final accounting within 60 days under § 42-52 (when extent of claim cannot be determined within 30 days).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordObligationToMaintainPremisesUnderGs42_42 => {
            if input.landlord_repair_and_maintenance_obligations_met {
                Output {
                    mode: NcLandlordTenantMode::CompliantLandlordObligationsToMaintainPremisesMet,
                    statutory_basis: "N.C. Gen. Stat. § 42-42 — landlord obligations to maintain premises met".to_string(),
                    notes: "COMPLIANT: landlord complies with applicable building and housing codes; makes all repairs to put and keep premises in fit and habitable condition; maintains facilities and appliances in good and safe working order under § 42-42.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationLandlordObligationsToMaintainPremisesBreached,
                    statutory_basis: "N.C. Gen. Stat. § 42-42 — landlord obligations to maintain premises breached".to_string(),
                    notes: "VIOLATION: landlord breached § 42-42 obligations to maintain premises in fit and habitable condition; tenant remedies under § 42-44 (actual damages + attorney's fees in some cases; tenant may NOT withhold rent without court order).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LateFeeCapGreaterOf15Or5PercentUnderGs42_46A => {
            if input.late_fee_charged_multiple_times_for_same_late_payment {
                return Output {
                    mode: NcLandlordTenantMode::ViolationLateFeeChargedMoreThanOncePerLatePayment,
                    statutory_basis: "N.C. Gen. Stat. § 42-46(a) — late fee imposed more than once per late rental payment".to_string(),
                    notes: "VIOLATION: late fee imposed more than once for same late rental payment; § 42-46(a) limits late fee to ONE TIME per late payment.".to_string(),
                    citations,
                };
            }
            let five_pct_cap = (u128::from(input.monthly_rent_dollars) * 500 / 10_000) as u64;
            let cap = NC_LANDLORD_TENANT_LATE_FEE_FLAT_DOLLAR_CAP.max(five_pct_cap);
            if input.late_fee_charged_dollars <= cap {
                Output {
                    mode: NcLandlordTenantMode::CompliantLateFeeAtOrBelowGreaterOf15Or5PercentCap,
                    statutory_basis: "N.C. Gen. Stat. § 42-46(a) — late fee at or below greater of $15 or 5 percent of monthly rent".to_string(),
                    notes: format!("COMPLIANT: late fee at or below cap of ${cap} (greater of $15 or 5 percent of monthly rent) under § 42-46(a)."),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationLateFeeExceedsGreaterOf15Or5PercentCap,
                    statutory_basis: "N.C. Gen. Stat. § 42-46(a) — late fee exceeds greater of $15 or 5 percent cap".to_string(),
                    notes: format!("VIOLATION: late fee exceeds ${cap} statutory cap (greater of $15 or 5 percent of monthly rent) under § 42-46(a)."),
                    citations,
                }
            }
        }
        ComplianceAspect::AdministrativeComplaintFilingFeeCapUnderGs42_46B => {
            let five_pct_cap = (u128::from(input.monthly_rent_dollars) * 500 / 10_000) as u64;
            let cap = NC_LANDLORD_TENANT_LATE_FEE_FLAT_DOLLAR_CAP.max(five_pct_cap);
            if input.administrative_fee_four_prerequisites_met
                && input.administrative_fee_charged_dollars <= cap
            {
                Output {
                    mode: NcLandlordTenantMode::CompliantAdministrativeFeeAtOrBelowGreaterOf15Or5PercentWithRequiredConditions,
                    statutory_basis: "N.C. Gen. Stat. § 42-46(b) — administrative fee at or below cap with four prerequisites met".to_string(),
                    notes: format!("COMPLIANT: administrative complaint-filing fee at or below cap of ${cap} (greater of $15 or 5 percent of monthly rent) AND four prerequisites met (tenant default + complaint filed + tenant cured + landlord dismissed) under § 42-46(b)."),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationAdministrativeFeeChargedWithoutMeetingFourPrerequisites,
                    statutory_basis: "N.C. Gen. Stat. § 42-46(b) — administrative fee charged without meeting four prerequisites OR exceeds cap".to_string(),
                    notes: format!("VIOLATION: administrative complaint-filing fee charged WITHOUT meeting all four prerequisites under § 42-46(b) OR exceeds ${cap} cap; fee NOT authorized."),
                    citations,
                }
            }
        }
        ComplianceAspect::TenDayPayOrQuitDemandUnderGs42_26 => {
            if input.pay_or_quit_demand_days_given >= NC_LANDLORD_TENANT_PAY_OR_QUIT_DEMAND_DAYS {
                Output {
                    mode: NcLandlordTenantMode::CompliantTenDayPayOrQuitDemandProvided,
                    statutory_basis: "N.C. Gen. Stat. § 42-26 — 10-day pay or quit demand provided before summary ejectment".to_string(),
                    notes: "COMPLIANT: landlord demanded payment and waited 10 days before filing summary ejectment action in small claims court under § 42-26.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::ViolationPayOrQuitDemandShorterThanTenDays,
                    statutory_basis: "N.C. Gen. Stat. § 42-26 — pay or quit demand shorter than 10-day statutory minimum".to_string(),
                    notes: "VIOLATION: pay or quit demand shorter than 10-day statutory minimum under § 42-26; summary ejectment action subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SelfHelpEvictionProhibitedUnderGs42_25_6 => {
            if input.used_self_help_eviction {
                Output {
                    mode: NcLandlordTenantMode::ViolationSelfHelpEvictionProhibited,
                    statutory_basis: "N.C. Gen. Stat. § 42-25.6 — self-help eviction prohibited; lawful eviction requires judicial process".to_string(),
                    notes: "VIOLATION: landlord used self-help eviction (lockout / utility shut-off / removal of tenant's property) prohibited under § 42-25.6; lawful eviction in North Carolina requires judicial process; tenant may seek injunctive relief and damages.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: NcLandlordTenantMode::CompliantNoSelfHelpEvictionLawfulJudicialProcessUsed,
                    statutory_basis: "N.C. Gen. Stat. § 42-25.6 — no self-help eviction".to_string(),
                    notes: "COMPLIANT: no self-help eviction; landlord used lawful judicial process (summary ejectment via small claims court) under § 42-25.6.".to_string(),
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
            tenancy_type: TenancyType::ResidentialRentalCoveredByChapter42,
            compliance_aspect: ComplianceAspect::SecurityDepositCapBasedOnTenancyTermUnderGs42_51,
            tenancy_term_length: TenancyTermLength::MonthToMonth,
            monthly_rent_dollars: 1_200,
            weekly_rent_dollars: 300,
            security_deposit_dollars: 1_800,
            deposit_in_separate_trust_account_or_bond: true,
            deposit_returned_with_itemized_statement_within_window: true,
            days_since_tenancy_termination_for_deposit_return: 25,
            days_since_tenancy_termination_for_final_accounting: 55,
            interim_accounting_provided_at_30_days: true,
            final_accounting_provided_at_60_days: true,
            landlord_repair_and_maintenance_obligations_met: true,
            late_fee_charged_dollars: 60,
            late_fee_charged_multiple_times_for_same_late_payment: false,
            administrative_fee_charged_dollars: 60,
            administrative_fee_four_prerequisites_met: true,
            pay_or_quit_demand_days_given: 10,
            used_self_help_eviction: false,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::CommercialRentalExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::NotApplicableTenancyExemptFromChapter42
        );
    }

    #[test]
    fn deposit_in_trust_account_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositTrustAccountOrBondUnderGs42_50;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantSecurityDepositInTrustAccountOrBondProvided
        );
    }

    #[test]
    fn deposit_not_in_trust_account_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositTrustAccountOrBondUnderGs42_50;
        input.deposit_in_separate_trust_account_or_bond = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationSecurityDepositNotInTrustAccountOrBond
        );
    }

    #[test]
    fn deposit_at_one_and_one_half_months_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantSecurityDepositAtOrBelowStatutoryCapForTenancyTerm
        );
    }

    #[test]
    fn deposit_at_1_5_months_plus_one_violation() {
        let mut input = baseline_input();
        input.security_deposit_dollars = 1_801;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationSecurityDepositExceedsStatutoryCapForTenancyTerm
        );
    }

    #[test]
    fn deposit_week_to_week_at_two_weeks_rent_compliant() {
        let mut input = baseline_input();
        input.tenancy_term_length = TenancyTermLength::WeekToWeek;
        input.weekly_rent_dollars = 300;
        input.security_deposit_dollars = 600;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantSecurityDepositAtOrBelowStatutoryCapForTenancyTerm
        );
    }

    #[test]
    fn deposit_week_to_week_at_two_weeks_plus_one_dollar_violation() {
        let mut input = baseline_input();
        input.tenancy_term_length = TenancyTermLength::WeekToWeek;
        input.weekly_rent_dollars = 300;
        input.security_deposit_dollars = 601;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationSecurityDepositExceedsStatutoryCapForTenancyTerm
        );
    }

    #[test]
    fn deposit_term_greater_than_month_at_two_months_compliant() {
        let mut input = baseline_input();
        input.tenancy_term_length = TenancyTermLength::TermGreaterThanMonthToMonth;
        input.security_deposit_dollars = 2_400;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantSecurityDepositAtOrBelowStatutoryCapForTenancyTerm
        );
    }

    #[test]
    fn deposit_term_greater_than_month_at_two_months_plus_one_dollar_violation() {
        let mut input = baseline_input();
        input.tenancy_term_length = TenancyTermLength::TermGreaterThanMonthToMonth;
        input.security_deposit_dollars = 2_401;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationSecurityDepositExceedsStatutoryCapForTenancyTerm
        );
    }

    #[test]
    fn deposit_return_within_30_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturn30DayWithItemizedStatementUnderGs42_52;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantSecurityDepositReturnedWithItemizedStatementWithin30Days
        );
    }

    #[test]
    fn deposit_return_at_31_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturn30DayWithItemizedStatementUnderGs42_52;
        input.days_since_tenancy_termination_for_deposit_return = 31;
        input.deposit_returned_with_itemized_statement_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationSecurityDepositReturnedPastThirtyDayDeadline
        );
    }

    #[test]
    fn interim_and_final_accounting_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositInterimAndFinalAccountingUnderGs42_52;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantSecurityDepositInterimAccountingAt30DaysAndFinalAt60Days
        );
    }

    #[test]
    fn final_accounting_at_61_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositInterimAndFinalAccountingUnderGs42_52;
        input.days_since_tenancy_termination_for_final_accounting = 61;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationSecurityDepositInterimOrFinalAccountingDeadlineMissed
        );
    }

    #[test]
    fn landlord_obligations_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToMaintainPremisesUnderGs42_42;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantLandlordObligationsToMaintainPremisesMet
        );
    }

    #[test]
    fn landlord_obligations_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToMaintainPremisesUnderGs42_42;
        input.landlord_repair_and_maintenance_obligations_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationLandlordObligationsToMaintainPremisesBreached
        );
    }

    #[test]
    fn late_fee_at_five_percent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LateFeeCapGreaterOf15Or5PercentUnderGs42_46A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantLateFeeAtOrBelowGreaterOf15Or5PercentCap
        );
    }

    #[test]
    fn late_fee_at_15_dollar_floor_when_rent_low_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LateFeeCapGreaterOf15Or5PercentUnderGs42_46A;
        input.monthly_rent_dollars = 100;
        input.late_fee_charged_dollars = 15;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantLateFeeAtOrBelowGreaterOf15Or5PercentCap
        );
    }

    #[test]
    fn late_fee_above_5_percent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LateFeeCapGreaterOf15Or5PercentUnderGs42_46A;
        input.late_fee_charged_dollars = 61;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationLateFeeExceedsGreaterOf15Or5PercentCap
        );
    }

    #[test]
    fn late_fee_charged_multiple_times_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LateFeeCapGreaterOf15Or5PercentUnderGs42_46A;
        input.late_fee_charged_multiple_times_for_same_late_payment = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationLateFeeChargedMoreThanOncePerLatePayment
        );
    }

    #[test]
    fn administrative_fee_with_prerequisites_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::AdministrativeComplaintFilingFeeCapUnderGs42_46B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantAdministrativeFeeAtOrBelowGreaterOf15Or5PercentWithRequiredConditions
        );
    }

    #[test]
    fn administrative_fee_without_prerequisites_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::AdministrativeComplaintFilingFeeCapUnderGs42_46B;
        input.administrative_fee_four_prerequisites_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationAdministrativeFeeChargedWithoutMeetingFourPrerequisites
        );
    }

    #[test]
    fn ten_day_pay_or_quit_demand_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayPayOrQuitDemandUnderGs42_26;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantTenDayPayOrQuitDemandProvided
        );
    }

    #[test]
    fn pay_or_quit_demand_under_10_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayPayOrQuitDemandUnderGs42_26;
        input.pay_or_quit_demand_days_given = 9;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationPayOrQuitDemandShorterThanTenDays
        );
    }

    #[test]
    fn no_self_help_eviction_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SelfHelpEvictionProhibitedUnderGs42_25_6;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::CompliantNoSelfHelpEvictionLawfulJudicialProcessUsed
        );
    }

    #[test]
    fn self_help_eviction_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SelfHelpEvictionProhibitedUnderGs42_25_6;
        input.used_self_help_eviction = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            NcLandlordTenantMode::ViolationSelfHelpEvictionProhibited
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(NC_LANDLORD_TENANT_CHAPTER_NUMBER, 42);
        assert_eq!(
            NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_WEEKS_WEEK_TO_WEEK,
            2
        );
        assert_eq!(
            NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_TENTHS_OF_MONTHS_MONTH_TO_MONTH,
            15
        );
        assert_eq!(
            NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS,
            10
        );
        assert_eq!(
            NC_LANDLORD_TENANT_SECURITY_DEPOSIT_CAP_MONTHS_TERM_GREATER_THAN_MONTH_TO_MONTH,
            2
        );
        assert_eq!(NC_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(
            NC_LANDLORD_TENANT_SECURITY_DEPOSIT_FINAL_ACCOUNTING_DEADLINE_DAYS,
            60
        );
        assert_eq!(NC_LANDLORD_TENANT_UNKNOWN_ADDRESS_HOLD_PERIOD_MONTHS, 6);
        assert_eq!(NC_LANDLORD_TENANT_LATE_FEE_FLAT_DOLLAR_CAP, 15);
        assert_eq!(NC_LANDLORD_TENANT_LATE_FEE_PERCENT_CAP_BPS, 500);
        assert_eq!(NC_LANDLORD_TENANT_PAY_OR_QUIT_DEMAND_DAYS, 10);
        assert_eq!(NC_LANDLORD_TENANT_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("North Carolina Landlord-Tenant Law"));
        assert!(joined.contains("N.C. Gen. Stat. §§ 42-1 through 42-86"));
        assert!(joined.contains("§ 42-50"));
        assert!(joined.contains("§ 42-51"));
        assert!(joined.contains("§ 42-52"));
        assert!(joined.contains("§ 42-46(a)"));
        assert!(joined.contains("§ 42-46(b)"));
        assert!(joined.contains("§ 42-42"));
        assert!(joined.contains("§ 42-26"));
        assert!(joined.contains("§ 42-25.6"));
        assert!(joined.contains("§ 42-7.1"));
        assert!(joined.contains("§ 42-44"));
        assert!(joined.contains("Article 6"));
        assert!(joined.contains("Tenant Security Deposit Act"));
        assert!(joined.contains("SEPARATE TRUST ACCOUNT"));
        assert!(joined.contains("BOND"));
        assert!(joined.contains("2 WEEKS"));
        assert!(joined.contains("1.5 MONTHS"));
        assert!(joined.contains("2 MONTHS"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("60 DAYS"));
        assert!(joined.contains("6 MONTHS"));
        assert!(joined.contains("$15.00"));
        assert!(joined.contains("5 PERCENT"));
        assert!(joined.contains("ONE TIME"));
        assert!(joined.contains("ADMINISTRATIVE"));
        assert!(joined.contains("10 DAYS"));
        assert!(joined.contains("judicial process"));
    }
}
