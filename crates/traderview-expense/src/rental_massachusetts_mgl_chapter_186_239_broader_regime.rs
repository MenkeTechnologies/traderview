//! Massachusetts Landlord-Tenant Law — MGL Chapter 186
//! (Landlord and Tenant) + Chapter 239 (Summary Process)
//! Broader Regime Compliance Module — pure-compute check
//! for the broader Massachusetts statewide landlord-tenant
//! regime spanning **MGL Chapter 186 (Estates for Years
//! and at Will)** including § 14 covenant of quiet
//! enjoyment + warranty of habitability + § 15F last
//! month's rent + interest, and **MGL Chapter 239**
//! (Summary Process / Eviction) including § 1 14-day
//! notice to quit and § 10 right to cure.
//!
//! Companion to two existing Massachusetts modules:
//! `rental_massachusetts_security_deposit_statute`
//! (covers MGL c. 186 § 15B security deposits
//! specifically) and `rental_massachusetts_homes_act_
//! eviction_sealing` (covers eviction sealing).
//!
//! **Distinctive Massachusetts features**: **TREBLE
//! DAMAGES** (3x) for covenant of quiet enjoyment
//! violations (greater of actual damages or 3x rent);
//! **non-discretionary treble damages** for § 15B strict
//! violations; **right to cure until judgment is entered**
//! under MGL c. 239 § 10.
//!
//! Web research (verified 2026-06-03):
//! - **MGL Chapter 186 (Estates for Years and at Will)**: Massachusetts statewide landlord-tenant regime governing leases and tenancies at will ([Mass.gov — Mass. General Laws c. 186](https://www.mass.gov/lists/mass-general-laws-c186); [Massachusetts Legislature — Chapter 186 General Laws](https://malegislature.gov/laws/generallaws/partii/titlei/chapter186); [Massachusetts Legislature — MGL c. 186 § 14](https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section14); [Massachusetts Legislature — MGL c. 186 § 15B](https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section15b); [Mass.gov — Mass. General Laws c. 186 § 15B](https://www.mass.gov/info-details/mass-general-laws-c186-ss-15b); [Justia — Massachusetts General Laws Chapter 186 Section 15B (2023)](https://law.justia.com/codes/massachusetts/part-ii/title-i/chapter-186/section-15b/); [Justia — 2025 Massachusetts General Laws Chapter 186](https://law.justia.com/codes/massachusetts/part-ii/title-i/chapter-186/); [FindLaw — Massachusetts General Laws c. 186 § 14](https://codes.findlaw.com/ma/part-ii-real-and-personal-property-and-domestic-relations-ch-183-210/ma-gen-laws-ch-186-sect-14/); [DRI Law — Understanding the Implied Covenant of Quiet Enjoyment](https://www.drilaw.com/blog/2025/08/understanding-the-important-of-the-implied-covenant-of-quiet-enjoyment/); [Mass.gov — Massachusetts Law About Tenants' Security Deposits](https://www.mass.gov/info-details/massachusetts-law-about-tenants-security-deposits); [Massachusetts Legal Help — Chapter 3 Security Deposits & Last Month's Rent PDF](https://www.masslegalhelp.org/sites/default/files/2025-10/03%20Security%20Deposits%202025-%20Updated%208-13-25.pdf); [Massachusetts Legal Help — Your Landlord's Responsibilities](https://www.masslegalhelp.org/housing-apartments-shelter/security-deposits/your-landlords-responsibilities); [Green Ocean Property Management — Massachusetts Security Deposit Laws for Landlords](https://greenoceanpropertymanagement.com/massachusetts-security-deposit-laws-for-landlords/); [OCM Law — Security Deposits in Massachusetts Rules That Every Landlord Needs to Know](https://www.ocmlaw.net/security-deposits-in-massachusetts-rules-that-every-landlord-needs-to-know/); [Rentto — Massachusetts Security Deposit Laws Complete Guide](https://www.renttohq.com/resources/laws/massachusetts/massachusetts-security-deposits); [Massachusetts Legislature — MGL Chapter 239](https://malegislature.gov/Laws/GeneralLaws/Partiii/Titleiii/Chapter239); [Mass.gov — Mass. General Laws c. 239](https://www.mass.gov/lists/mass-general-laws-c239); [Mass.gov — Section 15 Monthly Reports Chapter 239](https://www.mass.gov/lists/monthly-reports-section-15-of-chapter-239-of-the-general-laws-permanent-rental-protections); [Tellus Resource Guide — Massachusetts Eviction Protections for Tenants](https://resources.tellusapp.com/tenants/massachusetts/eviction-process/massachusetts-eviction-protections); [Tenant Rights — Massachusetts Eviction Process Step-by-Step Timeline](https://tenant-rights.com/massachusetts/massachusetts-eviction-process-step-by-step-timeline); [Mass.gov — Mass. General Laws c. 239 § 17](https://www.mass.gov/info-details/mass-general-laws-c239-ss-17); [iPropertyManagement — Massachusetts Warranty of Habitability 2026](https://ipropertymanagement.com/laws/warranty-of-habitability-massachusetts); [Massachusetts Legal Help — Your Right to a Decent Place to Live](https://www.masslegalhelp.org/housing-apartments-shelter/repairs-bad-conditions/your-right-decent-place-live); [LegalClarity — Massachusetts Habitability Warranty Tenant Rights & Legal Remedies](https://legalclarity.org/massachusetts-habitability-warranty-tenant-rights-legal-remedies/); [Generis Online — Understanding Tenant Rights in Habitability Disputes in Massachusetts](https://generisonline.com/understanding-tenant-rights-in-habitability-disputes-in-massachusetts/); [Gaudet Law Office — Landlord-Tenant Matters: The Warranty of Habitability in Massachusetts](https://gaudetlawoffice.com/landlord-tenant-matters-the-warranty-of-habitability-in-massachusetts/); [JD Molleur Law — The Warranty of Habitability](https://www.jdmolleurlaw.com/the-of-warranty-of-habitability/); [Boston College Law Review — Landlord Liability in Massachusetts PDF](https://bclawreview.bc.edu/articles/1702/files/63ca32b10499d.pdf); [iPropertyManagement — Massachusetts Renter's Rights for Repairs 2026](https://ipropertymanagement.com/laws/massachusetts-renters-rights-for-repairs); [Hoozzee — Massachusetts Landlord-Tenant Rights](https://www.hoozzee.com/state/massachusetts); [Boston University Law — Gatekeeping a Tenant's Right to 100% Habitable Housing](https://scholarship.law.bu.edu/cgi/viewcontent.cgi?article=4773&context=faculty_scholarship)).
//! - **MGL c. 186 § 14 Covenant of Quiet Enjoyment**: tenants are entitled to **QUIET ENJOYMENT** of their rented property and are guaranteed the right to be free from **SERIOUS INTERFERENCES** with their tenancies; this right is **IMPLICIT IN EVERY CONTRACT TO LEASE**, whether or not expressly written, and **CANNOT BE INVALIDATED REGARDLESS OF THE LEASE LANGUAGE**.
//! - **MGL c. 186 § 14 Treble Damages for Quiet Enjoyment Violations**: for interferences with the right to quiet enjoyment, tenants may sue the landlord for money damages, which are their **ACTUAL DAMAGES OR 3 TIMES THEIR RENT, WHICHEVER IS GREATER** (a 3x multiplier penalty).
//! - **MGL c. 186 § 14 Implied Warranty of Habitability**: MGL Chapter 186 § 14 outlines landlord obligations including **MAINTAINING STRUCTURAL INTEGRITY**, **PROVIDING ADEQUATE HEATING**, and **ENSURING ESSENTIAL UTILITIES** (water and electricity) are available; rental properties must be **FIT FOR HUMAN HABITATION** and **SAFE AND SUITABLE FOR OCCUPANCY**.
//! - **MGL c. 186 § 14 Tenant Remedies for Habitability Breach**: tenants may **(1) WITHHOLD RENT** OR deduct the cost of repairs from rent; **(2) GO TO COURT** and ask a judge to order repairs and reduce rent; OR **(3) CANCEL THE LEASE** or rental agreement and move out.
//! - **MGL c. 186 § 15B Security Deposit — 1-Month Cap + 30-Day Interest-Bearing Account**: security deposit **CANNOT EXCEED ONE MONTH'S RENT**; within **30 DAYS of receipt**, landlord must deposit the security deposit into a **SEPARATE, INTEREST-BEARING BANK ACCOUNT**; annual interest paid to tenant; within 30 days of tenancy end, written itemized statement required.
//! - **MGL c. 186 § 15B Non-Discretionary Treble Damages**: tenant is entitled to **TREBLE DAMAGE REMEDY** whenever the landlord fails to **STRICTLY COMPLY** with the terms of the statute; the penalty is **NOT DISCRETIONARY**, and the tenant does **NOT NEED TO PROVE** that the landlord acted in bad faith or that the tenant lost money because of the landlord's actions.
//! - **MGL c. 186 § 15F Last Month's Rent + Interest**: landlord must pay **ANNUAL INTEREST** on last month's rent at the same rate as security deposit interest; tenant has the right to apply last month's rent toward the final rental period.
//! - **MGL c. 239 § 1 Summary Process for Nonpayment — 14-Day Notice to Quit**: for nonpayment of rent, the landlord must serve a **14-DAY NOTICE TO QUIT** before filing a summary process action.
//! - **MGL c. 239 § 1A Tenancy at Will — 30-Day Notice for Other Violations**: for lease violations OR terminating a tenancy at will, the notice period depends on the lease but is typically **30 DAYS** for tenancies at will.
//! - **MGL c. 239 § 10 Right to Cure Until Judgment Entered**: under § 10, the court must **DISMISS THE CASE** if the tenant makes **FULL PAYMENT** of rent + interest + court costs at any point **UNTIL JUDGMENT IS ENTERED**; this is one of the most tenant-protective right-to-cure provisions in the United States.
//! - **MGL c. 239 Summary Process Court**: summary process action filed in **HOUSING COURT** (where available) **OR DISTRICT COURT**; tenant has a statutorily-prescribed time to file an answer.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MA_CHAPTER_186_NUMBER: u32 = 186;
pub const MA_CHAPTER_239_NUMBER: u32 = 239;
pub const MA_SECURITY_DEPOSIT_CAP_MONTHS: u32 = 1;
pub const MA_INTEREST_BEARING_ACCOUNT_DEADLINE_DAYS: u32 = 30;
pub const MA_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const MA_TREBLE_DAMAGES_MULTIPLIER: u32 = 3;
pub const MA_NONPAYMENT_NOTICE_DAYS: u32 = 14;
pub const MA_TENANCY_AT_WILL_NOTICE_DAYS: u32 = 30;
pub const MA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromMassachusettsLandlordTenantLaws,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuietEnjoymentInterferenceStatus {
    NoSeriousInterference,
    SeriousInterferenceWithTenantQuietEnjoyment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitabilityStatus {
    LandlordMaintainsHabitabilityUnderSection14,
    LandlordFailedHabitabilityUnderSection14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LastMonthRentInterestStatus {
    LandlordPaidAnnualInterestOnLastMonthRent,
    LandlordOmittedAnnualInterestOnLastMonthRent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RightToCureStatus {
    TenantTenderedFullPaymentBeforeJudgment,
    TenantDidNotTenderFullPayment,
    JudgmentAlreadyEntered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    QuietEnjoymentUnderChapter186Section14,
    WarrantyOfHabitabilityUnderChapter186Section14,
    LastMonthRentInterestUnderChapter186Section15F,
    SummaryProcessNonpaymentNoticeUnderChapter239Section1,
    TenancyAtWillTerminationNoticeUnderChapter239Section1A,
    RightToCureUnderChapter239Section10,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaLandlordTenantMode {
    NotApplicableTenancyExemptFromMassachusettsLandlordTenantLaws,
    CompliantNoSeriousInterferenceWithQuietEnjoyment,
    CompliantLandlordMaintainsHabitability,
    CompliantLandlordPaidAnnualInterestOnLastMonthRent,
    CompliantFourteenDayNonpaymentNoticeProperlyServed,
    CompliantThirtyDayTenancyAtWillNoticeProperlyServed,
    CompliantTenantCuredBeforeJudgmentDismissalRequired,
    ViolationSeriousInterferenceWithQuietEnjoymentTrebleDamages,
    ViolationLandlordFailedHabitabilityTenantRemediesAvailable,
    ViolationLandlordOmittedAnnualInterestOnLastMonthRent,
    ViolationNonpaymentNoticeShorterThan14Days,
    ViolationTenancyAtWillNoticeShorterThan30Days,
    ViolationCureRefusedAfterTenantTenderedFullPaymentBeforeJudgment,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub quiet_enjoyment_interference_status: QuietEnjoymentInterferenceStatus,
    pub habitability_status: HabitabilityStatus,
    pub last_month_rent_interest_status: LastMonthRentInterestStatus,
    pub right_to_cure_status: RightToCureStatus,
    pub compliance_aspect: ComplianceAspect,
    pub monthly_rent_dollars: u64,
    pub actual_damages_dollars: u64,
    pub nonpayment_notice_days_given: u32,
    pub tenancy_at_will_notice_days_given: u32,
    pub cure_accepted_by_landlord: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: MaLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_damages_dollars: u64,
}

pub type MaLandlordTenantInput = Input;
pub type MaLandlordTenantOutput = Output;
pub type MaLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Massachusetts Landlord-Tenant Law — MGL Chapter 186 (Estates for Years and at Will) governs leases and tenancies at will; MGL Chapter 239 (Summary Process) governs evictions".to_string(),
        "MGL c. 186 § 14 Covenant of Quiet Enjoyment — tenants are entitled to QUIET ENJOYMENT of their rented property and are guaranteed the right to be free from SERIOUS INTERFERENCES with their tenancies; this right is IMPLICIT IN EVERY CONTRACT TO LEASE, whether or not expressly written, and CANNOT BE INVALIDATED REGARDLESS OF THE LEASE LANGUAGE".to_string(),
        "MGL c. 186 § 14 Treble Damages for Quiet Enjoyment Violations — for interferences with the right to quiet enjoyment, tenants may sue the landlord for money damages, which are their ACTUAL DAMAGES OR 3 TIMES THEIR RENT, WHICHEVER IS GREATER".to_string(),
        "MGL c. 186 § 14 Implied Warranty of Habitability — landlord obligations include MAINTAINING STRUCTURAL INTEGRITY, PROVIDING ADEQUATE HEATING, and ENSURING ESSENTIAL UTILITIES (water and electricity) are available; rental properties must be FIT FOR HUMAN HABITATION and SAFE AND SUITABLE FOR OCCUPANCY".to_string(),
        "MGL c. 186 § 14 Tenant Remedies for Habitability Breach — tenants may (1) WITHHOLD RENT OR deduct the cost of repairs from rent; (2) GO TO COURT and ask a judge to order repairs and reduce rent; OR (3) CANCEL THE LEASE or rental agreement and move out".to_string(),
        "MGL c. 186 § 15B Security Deposit — covered by separate rental_massachusetts_security_deposit_statute module".to_string(),
        "MGL c. 186 § 15F Last Month's Rent + Interest — landlord must pay ANNUAL INTEREST on last month's rent at the same rate as security deposit interest; tenant has the right to apply last month's rent toward the final rental period".to_string(),
        "MGL c. 239 § 1 Summary Process for Nonpayment — 14-Day Notice to Quit — for nonpayment of rent, the landlord must serve a 14-DAY NOTICE TO QUIT before filing a summary process action".to_string(),
        "MGL c. 239 § 1A Tenancy at Will — 30-Day Notice for Other Violations — for lease violations OR terminating a tenancy at will, the notice period depends on the lease but is typically 30 DAYS for tenancies at will".to_string(),
        "MGL c. 239 § 10 Right to Cure Until Judgment Entered — under § 10, the court must DISMISS THE CASE if the tenant makes FULL PAYMENT of rent + interest + court costs at any point UNTIL JUDGMENT IS ENTERED; one of the most tenant-protective right-to-cure provisions in the United States".to_string(),
        "MGL c. 239 Summary Process Court — summary process action filed in HOUSING COURT (where available) OR DISTRICT COURT".to_string(),
        "Mass.gov + Massachusetts Legislature + Justia + FindLaw + DRI Law + Massachusetts Legal Help + Green Ocean Property Management + OCM Law + Rentto + iPropertyManagement + LegalClarity + Generis Online + Gaudet Law Office + JD Molleur Law + Boston College Law Review + Boston University Law + Hoozzee + Tellus Resource Guide + Tenant Rights — practitioner overviews of MGL Chapter 186 + Chapter 239".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromMassachusettsLandlordTenantLaws {
        return Output {
            mode: MaLandlordTenantMode::NotApplicableTenancyExemptFromMassachusettsLandlordTenantLaws,
            statutory_basis: "Massachusetts landlord-tenant statutes jurisdiction — tenancy exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Massachusetts landlord-tenant statutes (MGL Chapter 186 + Chapter 239); Massachusetts landlord-tenant obligations unavailable.".to_string(),
            citations,
            computed_damages_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::QuietEnjoymentUnderChapter186Section14 => match input
            .quiet_enjoyment_interference_status
        {
            QuietEnjoymentInterferenceStatus::NoSeriousInterference => Output {
                mode: MaLandlordTenantMode::CompliantNoSeriousInterferenceWithQuietEnjoyment,
                statutory_basis: "MGL c. 186 § 14 — no serious interference with tenant's quiet enjoyment".to_string(),
                notes: "COMPLIANT: no serious interference with tenant's covenant of quiet enjoyment under MGL c. 186 § 14.".to_string(),
                citations,
                computed_damages_dollars: 0,
            },
            QuietEnjoymentInterferenceStatus::SeriousInterferenceWithTenantQuietEnjoyment => {
                let treble_rent =
                    input.monthly_rent_dollars * u64::from(MA_TREBLE_DAMAGES_MULTIPLIER);
                let damages = input.actual_damages_dollars.max(treble_rent);
                Output {
                    mode: MaLandlordTenantMode::ViolationSeriousInterferenceWithQuietEnjoymentTrebleDamages,
                    statutory_basis: "MGL c. 186 § 14 — serious interference triggers actual damages or 3x rent, whichever is greater".to_string(),
                    notes: format!(
                        "VIOLATION: serious interference with tenant's quiet enjoyment under MGL c. 186 § 14; damages = greater of ${actual} actual damages or ${treble} (3x ${rent} rent) = ${damages}.",
                        actual = input.actual_damages_dollars,
                        treble = treble_rent,
                        rent = input.monthly_rent_dollars,
                    ),
                    citations,
                    computed_damages_dollars: damages,
                }
            }
        },
        ComplianceAspect::WarrantyOfHabitabilityUnderChapter186Section14 => match input
            .habitability_status
        {
            HabitabilityStatus::LandlordMaintainsHabitabilityUnderSection14 => Output {
                mode: MaLandlordTenantMode::CompliantLandlordMaintainsHabitability,
                statutory_basis: "MGL c. 186 § 14 — landlord maintains habitability obligations".to_string(),
                notes: "COMPLIANT: landlord maintains structural integrity + adequate heating + essential utilities (water and electricity) under MGL c. 186 § 14 implied warranty of habitability.".to_string(),
                citations,
                computed_damages_dollars: 0,
            },
            HabitabilityStatus::LandlordFailedHabitabilityUnderSection14 => Output {
                mode: MaLandlordTenantMode::ViolationLandlordFailedHabitabilityTenantRemediesAvailable,
                statutory_basis: "MGL c. 186 § 14 — landlord failed habitability; tenant remedies available".to_string(),
                notes: "VIOLATION: landlord failed implied warranty of habitability under MGL c. 186 § 14; tenant remedies include withholding rent, court-ordered repairs + rent reduction, or canceling the lease and moving out.".to_string(),
                citations,
                computed_damages_dollars: 0,
            },
        },
        ComplianceAspect::LastMonthRentInterestUnderChapter186Section15F => match input
            .last_month_rent_interest_status
        {
            LastMonthRentInterestStatus::LandlordPaidAnnualInterestOnLastMonthRent => Output {
                mode: MaLandlordTenantMode::CompliantLandlordPaidAnnualInterestOnLastMonthRent,
                statutory_basis: "MGL c. 186 § 15F — landlord paid annual interest on last month's rent".to_string(),
                notes: "COMPLIANT: landlord paid annual interest on last month's rent at the same rate as security deposit interest under MGL c. 186 § 15F.".to_string(),
                citations,
                computed_damages_dollars: 0,
            },
            LastMonthRentInterestStatus::LandlordOmittedAnnualInterestOnLastMonthRent => Output {
                mode: MaLandlordTenantMode::ViolationLandlordOmittedAnnualInterestOnLastMonthRent,
                statutory_basis: "MGL c. 186 § 15F — landlord omitted annual interest on last month's rent".to_string(),
                notes: "VIOLATION: landlord omitted required annual interest on last month's rent under MGL c. 186 § 15F.".to_string(),
                citations,
                computed_damages_dollars: 0,
            },
        },
        ComplianceAspect::SummaryProcessNonpaymentNoticeUnderChapter239Section1 => {
            if input.nonpayment_notice_days_given >= MA_NONPAYMENT_NOTICE_DAYS {
                Output {
                    mode: MaLandlordTenantMode::CompliantFourteenDayNonpaymentNoticeProperlyServed,
                    statutory_basis: "MGL c. 239 § 1 — 14-day nonpayment notice to quit properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day nonpayment notice satisfies 14-day statutory minimum under MGL c. 239 § 1.",
                        d = input.nonpayment_notice_days_given,
                    ),
                    citations,
                    computed_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: MaLandlordTenantMode::ViolationNonpaymentNoticeShorterThan14Days,
                    statutory_basis: "MGL c. 239 § 1 — nonpayment notice shorter than 14-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day nonpayment notice shorter than 14-day statutory minimum under MGL c. 239 § 1.",
                        d = input.nonpayment_notice_days_given,
                    ),
                    citations,
                    computed_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::TenancyAtWillTerminationNoticeUnderChapter239Section1A => {
            if input.tenancy_at_will_notice_days_given >= MA_TENANCY_AT_WILL_NOTICE_DAYS {
                Output {
                    mode: MaLandlordTenantMode::CompliantThirtyDayTenancyAtWillNoticeProperlyServed,
                    statutory_basis: "MGL c. 239 § 1A — 30-day tenancy at will notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day tenancy at will termination notice satisfies 30-day statutory minimum under MGL c. 239 § 1A.",
                        d = input.tenancy_at_will_notice_days_given,
                    ),
                    citations,
                    computed_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: MaLandlordTenantMode::ViolationTenancyAtWillNoticeShorterThan30Days,
                    statutory_basis: "MGL c. 239 § 1A — tenancy at will notice shorter than 30-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day tenancy at will termination notice shorter than 30-day statutory minimum under MGL c. 239 § 1A.",
                        d = input.tenancy_at_will_notice_days_given,
                    ),
                    citations,
                    computed_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::RightToCureUnderChapter239Section10 => match input.right_to_cure_status {
            RightToCureStatus::TenantTenderedFullPaymentBeforeJudgment => {
                if input.cure_accepted_by_landlord {
                    Output {
                        mode: MaLandlordTenantMode::CompliantTenantCuredBeforeJudgmentDismissalRequired,
                        statutory_basis: "MGL c. 239 § 10 — tenant cured before judgment; court must dismiss the case".to_string(),
                        notes: "COMPLIANT: tenant tendered full payment of rent + interest + court costs before judgment was entered under MGL c. 239 § 10; court must DISMISS THE CASE.".to_string(),
                        citations,
                        computed_damages_dollars: 0,
                    }
                } else {
                    Output {
                        mode: MaLandlordTenantMode::ViolationCureRefusedAfterTenantTenderedFullPaymentBeforeJudgment,
                        statutory_basis: "MGL c. 239 § 10 — landlord refused cure after tenant tendered full payment before judgment".to_string(),
                        notes: "VIOLATION: landlord refused tenant's valid right-to-cure tender of full payment of rent + interest + court costs before judgment was entered under MGL c. 239 § 10; court must still dismiss the case.".to_string(),
                        citations,
                        computed_damages_dollars: 0,
                    }
                }
            }
            RightToCureStatus::TenantDidNotTenderFullPayment | RightToCureStatus::JudgmentAlreadyEntered => {
                Output {
                    mode: MaLandlordTenantMode::CompliantTenantCuredBeforeJudgmentDismissalRequired,
                    statutory_basis: "MGL c. 239 § 10 — right to cure not invoked or no longer available".to_string(),
                    notes: "COMPLIANT: tenant did not tender full payment or judgment was already entered; § 10 right to cure not invoked or no longer available.".to_string(),
                    citations,
                    computed_damages_dollars: 0,
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_coverage: TenancyCoverage::CoveredResidentialTenancy,
            quiet_enjoyment_interference_status:
                QuietEnjoymentInterferenceStatus::NoSeriousInterference,
            habitability_status: HabitabilityStatus::LandlordMaintainsHabitabilityUnderSection14,
            last_month_rent_interest_status:
                LastMonthRentInterestStatus::LandlordPaidAnnualInterestOnLastMonthRent,
            right_to_cure_status: RightToCureStatus::TenantDidNotTenderFullPayment,
            compliance_aspect: ComplianceAspect::QuietEnjoymentUnderChapter186Section14,
            monthly_rent_dollars: 2_000,
            actual_damages_dollars: 1_000,
            nonpayment_notice_days_given: 14,
            tenancy_at_will_notice_days_given: 30,
            cure_accepted_by_landlord: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromMassachusettsLandlordTenantLaws;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::NotApplicableTenancyExemptFromMassachusettsLandlordTenantLaws
        );
    }

    #[test]
    fn no_serious_interference_with_quiet_enjoyment_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QuietEnjoymentUnderChapter186Section14;
        input.quiet_enjoyment_interference_status =
            QuietEnjoymentInterferenceStatus::NoSeriousInterference;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::CompliantNoSeriousInterferenceWithQuietEnjoyment
        );
    }

    #[test]
    fn quiet_enjoyment_treble_rent_damages_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QuietEnjoymentUnderChapter186Section14;
        input.quiet_enjoyment_interference_status =
            QuietEnjoymentInterferenceStatus::SeriousInterferenceWithTenantQuietEnjoyment;
        input.monthly_rent_dollars = 2_000;
        input.actual_damages_dollars = 1_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::ViolationSeriousInterferenceWithQuietEnjoymentTrebleDamages
        );
        assert_eq!(out.computed_damages_dollars, 6_000);
    }

    #[test]
    fn quiet_enjoyment_actual_damages_greater_than_treble_rent() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QuietEnjoymentUnderChapter186Section14;
        input.quiet_enjoyment_interference_status =
            QuietEnjoymentInterferenceStatus::SeriousInterferenceWithTenantQuietEnjoyment;
        input.monthly_rent_dollars = 1_000;
        input.actual_damages_dollars = 5_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::ViolationSeriousInterferenceWithQuietEnjoymentTrebleDamages
        );
        assert_eq!(out.computed_damages_dollars, 5_000);
    }

    #[test]
    fn habitability_maintained_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WarrantyOfHabitabilityUnderChapter186Section14;
        input.habitability_status = HabitabilityStatus::LandlordMaintainsHabitabilityUnderSection14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::CompliantLandlordMaintainsHabitability
        );
    }

    #[test]
    fn habitability_failed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WarrantyOfHabitabilityUnderChapter186Section14;
        input.habitability_status = HabitabilityStatus::LandlordFailedHabitabilityUnderSection14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::ViolationLandlordFailedHabitabilityTenantRemediesAvailable
        );
    }

    #[test]
    fn last_month_rent_interest_paid_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LastMonthRentInterestUnderChapter186Section15F;
        input.last_month_rent_interest_status =
            LastMonthRentInterestStatus::LandlordPaidAnnualInterestOnLastMonthRent;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::CompliantLandlordPaidAnnualInterestOnLastMonthRent
        );
    }

    #[test]
    fn last_month_rent_interest_omitted_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LastMonthRentInterestUnderChapter186Section15F;
        input.last_month_rent_interest_status =
            LastMonthRentInterestStatus::LandlordOmittedAnnualInterestOnLastMonthRent;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::ViolationLandlordOmittedAnnualInterestOnLastMonthRent
        );
    }

    #[test]
    fn nonpayment_14_day_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SummaryProcessNonpaymentNoticeUnderChapter239Section1;
        input.nonpayment_notice_days_given = 14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::CompliantFourteenDayNonpaymentNoticeProperlyServed
        );
    }

    #[test]
    fn nonpayment_13_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SummaryProcessNonpaymentNoticeUnderChapter239Section1;
        input.nonpayment_notice_days_given = 13;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::ViolationNonpaymentNoticeShorterThan14Days
        );
    }

    #[test]
    fn tenancy_at_will_30_day_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenancyAtWillTerminationNoticeUnderChapter239Section1A;
        input.tenancy_at_will_notice_days_given = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::CompliantThirtyDayTenancyAtWillNoticeProperlyServed
        );
    }

    #[test]
    fn tenancy_at_will_29_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenancyAtWillTerminationNoticeUnderChapter239Section1A;
        input.tenancy_at_will_notice_days_given = 29;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::ViolationTenancyAtWillNoticeShorterThan30Days
        );
    }

    #[test]
    fn tenant_cured_before_judgment_dismissal_required_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RightToCureUnderChapter239Section10;
        input.right_to_cure_status = RightToCureStatus::TenantTenderedFullPaymentBeforeJudgment;
        input.cure_accepted_by_landlord = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::CompliantTenantCuredBeforeJudgmentDismissalRequired
        );
    }

    #[test]
    fn cure_refused_by_landlord_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RightToCureUnderChapter239Section10;
        input.right_to_cure_status = RightToCureStatus::TenantTenderedFullPaymentBeforeJudgment;
        input.cure_accepted_by_landlord = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::ViolationCureRefusedAfterTenantTenderedFullPaymentBeforeJudgment
        );
    }

    #[test]
    fn right_to_cure_not_invoked_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RightToCureUnderChapter239Section10;
        input.right_to_cure_status = RightToCureStatus::TenantDidNotTenderFullPayment;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MaLandlordTenantMode::CompliantTenantCuredBeforeJudgmentDismissalRequired
        );
    }

    #[test]
    fn constants_pin_massachusetts_landlord_tenant_statutory_thresholds() {
        assert_eq!(MA_CHAPTER_186_NUMBER, 186);
        assert_eq!(MA_CHAPTER_239_NUMBER, 239);
        assert_eq!(MA_SECURITY_DEPOSIT_CAP_MONTHS, 1);
        assert_eq!(MA_INTEREST_BEARING_ACCOUNT_DEADLINE_DAYS, 30);
        assert_eq!(MA_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(MA_TREBLE_DAMAGES_MULTIPLIER, 3);
        assert_eq!(MA_NONPAYMENT_NOTICE_DAYS, 14);
        assert_eq!(MA_TENANCY_AT_WILL_NOTICE_DAYS, 30);
        assert_eq!(MA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_massachusetts_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Massachusetts Landlord-Tenant Law"));
        assert!(joined.contains("MGL Chapter 186"));
        assert!(joined.contains("MGL Chapter 239"));
        assert!(joined.contains("MGL c. 186 § 14"));
        assert!(joined.contains("QUIET ENJOYMENT"));
        assert!(joined.contains("SERIOUS INTERFERENCES"));
        assert!(joined.contains("IMPLICIT IN EVERY CONTRACT TO LEASE"));
        assert!(joined.contains("CANNOT BE INVALIDATED REGARDLESS OF THE LEASE LANGUAGE"));
        assert!(joined.contains("ACTUAL DAMAGES OR 3 TIMES THEIR RENT, WHICHEVER IS GREATER"));
        assert!(joined.contains("MAINTAINING STRUCTURAL INTEGRITY"));
        assert!(joined.contains("PROVIDING ADEQUATE HEATING"));
        assert!(joined.contains("ENSURING ESSENTIAL UTILITIES"));
        assert!(joined.contains("FIT FOR HUMAN HABITATION"));
        assert!(joined.contains("WITHHOLD RENT"));
        assert!(joined.contains("CANCEL THE LEASE"));
        assert!(joined.contains("MGL c. 186 § 15F"));
        assert!(joined.contains("ANNUAL INTEREST"));
        assert!(joined.contains("MGL c. 239 § 1"));
        assert!(joined.contains("14-DAY NOTICE TO QUIT"));
        assert!(joined.contains("MGL c. 239 § 1A"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("MGL c. 239 § 10"));
        assert!(joined.contains("DISMISS THE CASE"));
        assert!(joined.contains("FULL PAYMENT"));
        assert!(joined.contains("UNTIL JUDGMENT IS ENTERED"));
        assert!(joined.contains("rental_massachusetts_security_deposit_statute"));
    }
}
