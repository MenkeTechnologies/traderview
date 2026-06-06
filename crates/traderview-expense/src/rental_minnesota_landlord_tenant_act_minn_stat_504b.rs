//! Minnesota Landlord and Tenant Act — Minn. Stat. Ch. 504B
//! Compliance Module — pure-compute check for landlord
//! statutory compliance with Minnesota's statewide
//! landlord-tenant regime spanning **Minn. Stat.
//! §§ 504B.001 through 504B.475**. Covers the **3-week
//! security deposit return** + **1% per annum interest**
//! requirement under § 504B.178, **non-waivable landlord
//! habitability covenants** under § 504B.161, **14-day
//! pay-or-quit notice** under § 504B.135, **24-hour notice
//! to enter** under § 504B.211, **unlawful utility
//! termination prohibition** under § 504B.221, and **rent
//! escrow action** under § 504B.385.
//!
//! **Distinctive Minnesota features**: **1% PER ANNUM
//! INTEREST** on security deposits (Minn. Stat. § 504B.178)
//! — Minnesota is one of only ~12 states requiring landlords
//! to pay interest on security deposits; **NON-WAIVABLE
//! HABITABILITY COVENANTS** under Minn. Stat. § 504B.161 —
//! parties may NOT waive or modify the landlord's covenants
//! of fitness and reasonable repair.
//!
//! Web research (verified 2026-06-03):
//! - **Minn. Stat. Ch. 504B (Landlord and Tenant)**: Minnesota's statewide landlord-tenant regime; codified at Minn. Stat. §§ 504B.001 through 504B.475 ([Minnesota Office of the Revisor of Statutes — Chapter 504B](https://www.revisor.mn.gov/statutes/cite/504B); [Minnesota Office of the Revisor — Minn. Stat. § 504B.178](https://www.revisor.mn.gov/statutes/cite/504b.178); [Minnesota Office of the Revisor — Minn. Stat. § 504B.135](https://www.revisor.mn.gov/statutes/cite/504B.135); [Minnesota Office of the Revisor — Minn. Stat. § 504B.385](https://www.revisor.mn.gov/statutes/cite/504b.385); [Justia — 2025 Minnesota Statutes Chapter 504B](https://law.justia.com/codes/minnesota/chapters-500-515b/chapter-504b/); [Minnesota Attorney General — Landlords and Tenants Rights and Responsibilities PDF](https://www.ag.state.mn.us/brochures/publandlordtenants.pdf); [FindLaw — Minn. Stat. § 504B.178](https://codes.findlaw.com/mn/property-and-property-interests-ch-500-515b/mn-st-sect-504b-178/); [Rentable — Minnesota Security Deposit Laws Complete Guide](https://www.rentable.com/blog/minnesota-security-deposit-laws-a-complete-guide-for-landlords-tenants/); [Innago — Minnesota Landlord Tenant Laws](https://innago.com/minnesota-landlord-tenant-laws/); [LegalClarity — Minnesota Statute 504B Tenant and Landlord Rights and Duties](https://legalclarity.org/minnesota-statute-504b-tenant-and-landlord-rights-and-duties/); [RPM Viking — Minnesota Landlord Tenant Law](https://www.rpmviking.com/minnesota-landlord-tenant-law); [Tenant Rights — Minnesota Tenant's Handbook PDF](https://tenant-rights.com/handbooks/minnesota.pdf); [Innago — Minnesota Eviction Process 2025](https://innago.com/minnesota-eviction-process/); [Hennepin County Law Library — Current State of Landlord-Tenant Law: Landlord's Perspective February 2021 PDF](https://www.hclawlib.org/-/media/law-library/CLEs/2021/the-current-state-of-landlord-tenant-law-the-landlords-perspective-february-2021.pdf); [Minnesota Attorney General — During the Tenancy](https://www.ag.state.mn.us/consumer/handbooks/lt/CH2.asp)).
//! - **Minn. Stat. § 504B.178 3-Week Deposit Return + 1% Interest**: landlord must return security deposit within **THREE WEEKS** (21 days) after termination of the tenancy; security deposits must bear **SIMPLE NONCOMPOUNDED INTEREST at the rate of ONE PERCENT (1%) PER ANNUM**, computed from the first day of the next month following the full payment of the deposit to the last day of the month in which the landlord complies with returning the deposit (or to the date of judgment).
//! - **Minn. Stat. § 504B.178 Burden of Proof on Landlord**: in any action concerning the deposit, the burden of proving by a **FAIR PREPONDERANCE OF THE EVIDENCE** the reason for withholding all or any portion of the deposit shall be on the landlord.
//! - **Minn. Stat. § 504B.178 Wrongful Withholding Penalty**: a landlord who fails to provide a written statement within three weeks of termination, or fails to return the deposit as required, is liable to the tenant for **DAMAGES IN AN AMOUNT EQUAL TO THE PORTION OF THE DEPOSIT WITHHELD by the landlord AND INTEREST THEREON as a PENALTY** (effectively doubling the wrongful withholding exposure).
//! - **Minn. Stat. § 504B.161 Non-Waivable Habitability Covenants**: in every lease or license of residential premises, the landlord or licensor covenants: **(1) THE PREMISES AND ALL COMMON AREAS ARE FIT FOR THE USE INTENDED BY THE PARTIES**; **(2) TO KEEP THE PREMISES IN REASONABLE REPAIR DURING THE TERM OF THE LEASE**; **(3) TO MAKE THE PREMISES REASONABLY ENERGY EFFICIENT** by installing weather stripping, caulking, storm windows, and storm doors; **(4) TO MAINTAIN THE PREMISES IN COMPLIANCE WITH APPLICABLE HEALTH AND SAFETY LAWS**. The parties to a lease or license of residential premises **MAY NOT WAIVE OR MODIFY** the covenants imposed by this section.
//! - **Minn. Stat. § 504B.135 14-Day Pay-or-Quit Notice**: if a tenant neglects or refuses to pay rent due on a tenancy at will, the landlord may terminate the tenancy by giving the tenant **14 DAYS NOTICE TO QUIT IN WRITING**.
//! - **Minn. Stat. § 504B.211 24-Hour Notice to Enter**: a landlord must provide the tenant with **REASONABLE NOTICE** of the landlord's intent to enter the rental unit before entering (typically interpreted as **24 HOURS** advance notice); exceptions apply for emergencies.
//! - **Minn. Stat. § 504B.221 Unlawful Termination of Utilities**: a landlord may **NOT INTENTIONALLY TERMINATE** the supply of water, hot water, heat, fuel, electricity, gas, telephone, or other essential services to a tenant; landlord who unlawfully terminates utilities faces civil damages + statutory penalties + tenant remedies.
//! - **Minn. Stat. § 504B.385 Rent Escrow Action to Remedy Violations**: if a landlord fails to maintain habitable conditions, tenants can file a **RENT ESCROW ACTION** allowing them to deposit rent with the court until repairs are made; tenants can file rent escrow cases for any violations of Chapter 504B or federal, state, county, or city discrimination laws.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MN_CHAPTER_NUMBER: u32 = 504;
pub const MN_SECURITY_DEPOSIT_RETURN_DEADLINE_WEEKS: u32 = 3;
pub const MN_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 21;
pub const MN_SECURITY_DEPOSIT_INTEREST_RATE_BPS_PER_ANNUM: u64 = 100;
pub const MN_PAY_OR_QUIT_NOTICE_DAYS: u32 = 14;
pub const MN_LANDLORD_ENTRY_NOTICE_HOURS: u32 = 24;
pub const MN_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromChapter504B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordCovenantWaiverStatus {
    LandlordCovenantsNotWaivedOrModified,
    LeaseAttemptsToWaiveOrModifyLandlordCovenants,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryNoticeStatus {
    ReasonableNotice24HoursOrMore,
    EntryWithoutReasonableNoticeNotEmergency,
    EmergencyEntryWithoutNotice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityTerminationStatus {
    UtilitiesMaintainedByLandlord,
    LandlordUnlawfullyTerminatedUtilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentEscrowPrerequisiteStatus {
    TenantSatisfiedNoticeAndReasonableTimePrerequisites,
    TenantInvokedEscrowWithoutNoticeOrReasonableTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    ThreeWeekDepositReturnUnderSection504B178,
    OnePercentInterestOnDepositUnderSection504B178,
    WrongfulWithholdingPenaltyUnderSection504B178,
    NonWaivableHabitabilityCovenantsUnderSection504B161,
    FourteenDayPayOrQuitNoticeUnderSection504B135,
    EntryNoticeUnderSection504B211,
    UnlawfulUtilityTerminationProhibitedUnderSection504B221,
    RentEscrowActionUnderSection504B385,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MnLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter504B,
    CompliantDepositReturnedWithinThreeWeeks,
    CompliantInterestPaidAtOnePercentPerAnnum,
    CompliantNoWrongfulWithholding,
    CompliantNonWaivableHabitabilityCovenantsPreserved,
    CompliantFourteenDayPayOrQuitNoticeProperlyServed,
    CompliantReasonableEntryNoticeProvided,
    CompliantEmergencyEntryWithoutNotice,
    CompliantUtilitiesMaintained,
    CompliantRentEscrowNoticeAndReasonableTimePrerequisitesMet,
    ViolationDepositReturnedPastThreeWeekDeadline,
    ViolationInterestNotPaidOrUnderpaid,
    ViolationWrongfulWithholdingDoublesDepositExposure,
    ViolationLeaseAttemptsToWaiveOrModifyNonWaivableCovenants,
    ViolationPayOrQuitNoticeShorterThan14Days,
    ViolationEntryWithoutReasonableNoticeNotEmergency,
    ViolationUnlawfulUtilityTerminationByLandlord,
    ViolationRentEscrowInvokedWithoutNoticeOrReasonableTime,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub landlord_covenant_waiver_status: LandlordCovenantWaiverStatus,
    pub entry_notice_status: EntryNoticeStatus,
    pub utility_termination_status: UtilityTerminationStatus,
    pub rent_escrow_prerequisite_status: RentEscrowPrerequisiteStatus,
    pub compliance_aspect: ComplianceAspect,
    pub days_to_return_deposit: u32,
    pub interest_paid_at_one_percent_per_annum: bool,
    pub deposit_wrongfully_withheld: bool,
    pub pay_or_quit_notice_days_given: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: MnLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type MnLandlordTenantInput = Input;
pub type MnLandlordTenantOutput = Output;
pub type MnLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Minnesota Landlord and Tenant Act — Minn. Stat. Chapter 504B; statewide landlord-tenant regime codified at Minn. Stat. §§ 504B.001 through 504B.475".to_string(),
        "Minn. Stat. § 504B.178 3-Week Deposit Return + 1% Interest — landlord must return security deposit within THREE WEEKS (21 days) after termination of the tenancy; security deposits must bear SIMPLE NONCOMPOUNDED INTEREST at the rate of ONE PERCENT (1%) PER ANNUM, computed from the first day of the next month following the full payment of the deposit to the last day of the month in which the landlord complies with returning the deposit".to_string(),
        "Minn. Stat. § 504B.178 Burden of Proof on Landlord — in any action concerning the deposit, the burden of proving by a FAIR PREPONDERANCE OF THE EVIDENCE the reason for withholding all or any portion of the deposit shall be on the landlord".to_string(),
        "Minn. Stat. § 504B.178 Wrongful Withholding Penalty — a landlord who fails to provide a written statement within three weeks of termination, or fails to return the deposit as required, is liable to the tenant for DAMAGES IN AN AMOUNT EQUAL TO THE PORTION OF THE DEPOSIT WITHHELD by the landlord AND INTEREST THEREON as a PENALTY (effectively doubling the wrongful withholding exposure)".to_string(),
        "Minn. Stat. § 504B.161 Non-Waivable Habitability Covenants — in every lease or license of residential premises, the landlord or licensor covenants: (1) THE PREMISES AND ALL COMMON AREAS ARE FIT FOR THE USE INTENDED BY THE PARTIES; (2) TO KEEP THE PREMISES IN REASONABLE REPAIR DURING THE TERM OF THE LEASE; (3) TO MAKE THE PREMISES REASONABLY ENERGY EFFICIENT (weather stripping, caulking, storm windows, storm doors); (4) TO MAINTAIN THE PREMISES IN COMPLIANCE WITH APPLICABLE HEALTH AND SAFETY LAWS. The parties to a lease or license of residential premises MAY NOT WAIVE OR MODIFY the covenants imposed by this section".to_string(),
        "Minn. Stat. § 504B.135 14-Day Pay-or-Quit Notice — if a tenant neglects or refuses to pay rent due on a tenancy at will, the landlord may terminate the tenancy by giving the tenant 14 DAYS NOTICE TO QUIT IN WRITING".to_string(),
        "Minn. Stat. § 504B.211 24-Hour Notice to Enter — landlord must provide tenant with REASONABLE NOTICE of intent to enter the rental unit before entering (typically interpreted as 24 HOURS advance notice); exceptions apply for emergencies".to_string(),
        "Minn. Stat. § 504B.221 Unlawful Termination of Utilities — landlord may NOT INTENTIONALLY TERMINATE the supply of water, hot water, heat, fuel, electricity, gas, telephone, or other essential services to a tenant; landlord who unlawfully terminates utilities faces civil damages + statutory penalties + tenant remedies".to_string(),
        "Minn. Stat. § 504B.385 Rent Escrow Action to Remedy Violations — if a landlord fails to maintain habitable conditions, tenants can file a RENT ESCROW ACTION allowing them to deposit rent with the court until repairs are made; tenants can file rent escrow cases for any violations of Chapter 504B or federal, state, county, or city discrimination laws".to_string(),
        "Minnesota Office of the Revisor of Statutes + Justia + Minnesota Attorney General + FindLaw + Rentable + Innago + LegalClarity + RPM Viking + Tenant Rights Minnesota Handbook + Hennepin County Law Library — practitioner overviews of Minn. Stat. Chapter 504B".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromChapter504B {
        return Output {
            mode: MnLandlordTenantMode::NotApplicableTenancyExemptFromChapter504B,
            statutory_basis: "Minn. Stat. Chapter 504B jurisdiction — tenancy exempt from Chapter 504B coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Minn. Stat. Chapter 504B; Chapter 504B landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::ThreeWeekDepositReturnUnderSection504B178 => {
            if input.days_to_return_deposit <= MN_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS {
                Output {
                    mode: MnLandlordTenantMode::CompliantDepositReturnedWithinThreeWeeks,
                    statutory_basis: "Minn. Stat. § 504B.178 — deposit returned within 3 weeks (21 days) of termination".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned at day {d} (within 3-week / 21-day statutory window) under § 504B.178.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MnLandlordTenantMode::ViolationDepositReturnedPastThreeWeekDeadline,
                    statutory_basis: "Minn. Stat. § 504B.178 — deposit return exceeded 3-week (21-day) statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} (past 3-week / 21-day statutory window) under § 504B.178.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::OnePercentInterestOnDepositUnderSection504B178 => {
            if input.interest_paid_at_one_percent_per_annum {
                Output {
                    mode: MnLandlordTenantMode::CompliantInterestPaidAtOnePercentPerAnnum,
                    statutory_basis: "Minn. Stat. § 504B.178 — 1 % per annum interest paid on security deposit".to_string(),
                    notes: "COMPLIANT: simple noncompounded interest at 1 % per annum paid on security deposit per § 504B.178; Minnesota is one of only ~12 states requiring landlords to pay interest on security deposits.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: MnLandlordTenantMode::ViolationInterestNotPaidOrUnderpaid,
                    statutory_basis: "Minn. Stat. § 504B.178 — required 1 % per annum interest on security deposit not paid or underpaid".to_string(),
                    notes: "VIOLATION: required 1 % per annum interest on security deposit not paid or underpaid under § 504B.178.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::WrongfulWithholdingPenaltyUnderSection504B178 => {
            if input.deposit_wrongfully_withheld {
                Output {
                    mode: MnLandlordTenantMode::ViolationWrongfulWithholdingDoublesDepositExposure,
                    statutory_basis: "Minn. Stat. § 504B.178 — wrongful withholding triggers damages equal to deposit withheld + interest as penalty".to_string(),
                    notes: "VIOLATION: deposit wrongfully withheld; landlord liable for damages equal to portion withheld + interest as penalty under § 504B.178 (effectively doubles wrongful withholding exposure).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: MnLandlordTenantMode::CompliantNoWrongfulWithholding,
                    statutory_basis: "Minn. Stat. § 504B.178 — no wrongful withholding".to_string(),
                    notes: "COMPLIANT: no wrongful withholding under § 504B.178; double-damages penalty exposure not triggered.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::NonWaivableHabitabilityCovenantsUnderSection504B161 => match input
            .landlord_covenant_waiver_status
        {
            LandlordCovenantWaiverStatus::LandlordCovenantsNotWaivedOrModified => Output {
                mode: MnLandlordTenantMode::CompliantNonWaivableHabitabilityCovenantsPreserved,
                statutory_basis: "Minn. Stat. § 504B.161 — non-waivable habitability covenants preserved".to_string(),
                notes: "COMPLIANT: non-waivable landlord habitability covenants under § 504B.161 (fit for use + reasonable repair + energy efficiency + health and safety code compliance) preserved.".to_string(),
                citations,
            },
            LandlordCovenantWaiverStatus::LeaseAttemptsToWaiveOrModifyLandlordCovenants => Output {
                mode: MnLandlordTenantMode::ViolationLeaseAttemptsToWaiveOrModifyNonWaivableCovenants,
                statutory_basis: "Minn. Stat. § 504B.161 — non-waivable habitability covenants; parties MAY NOT WAIVE OR MODIFY".to_string(),
                notes: "VIOLATION: lease attempts to waive or modify non-waivable landlord habitability covenants under § 504B.161; waiver clause is unenforceable.".to_string(),
                citations,
            },
        },
        ComplianceAspect::FourteenDayPayOrQuitNoticeUnderSection504B135 => {
            if input.pay_or_quit_notice_days_given >= MN_PAY_OR_QUIT_NOTICE_DAYS {
                Output {
                    mode: MnLandlordTenantMode::CompliantFourteenDayPayOrQuitNoticeProperlyServed,
                    statutory_basis: "Minn. Stat. § 504B.135 — 14-day pay-or-quit notice for nonpayment properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day pay-or-quit notice satisfies 14-day statutory minimum under § 504B.135.",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: MnLandlordTenantMode::ViolationPayOrQuitNoticeShorterThan14Days,
                    statutory_basis: "Minn. Stat. § 504B.135 — pay-or-quit notice shorter than statutory 14-day minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day pay-or-quit notice shorter than 14-day statutory minimum under § 504B.135.",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::EntryNoticeUnderSection504B211 => match input.entry_notice_status {
            EntryNoticeStatus::ReasonableNotice24HoursOrMore => Output {
                mode: MnLandlordTenantMode::CompliantReasonableEntryNoticeProvided,
                statutory_basis: "Minn. Stat. § 504B.211 — reasonable notice of intent to enter provided".to_string(),
                notes: "COMPLIANT: landlord provided reasonable notice (typically 24 hours or more) of intent to enter rental unit under § 504B.211.".to_string(),
                citations,
            },
            EntryNoticeStatus::EmergencyEntryWithoutNotice => Output {
                mode: MnLandlordTenantMode::CompliantEmergencyEntryWithoutNotice,
                statutory_basis: "Minn. Stat. § 504B.211 — emergency exception to entry notice requirement".to_string(),
                notes: "COMPLIANT: emergency entry without notice permitted under § 504B.211 emergency exception.".to_string(),
                citations,
            },
            EntryNoticeStatus::EntryWithoutReasonableNoticeNotEmergency => Output {
                mode: MnLandlordTenantMode::ViolationEntryWithoutReasonableNoticeNotEmergency,
                statutory_basis: "Minn. Stat. § 504B.211 — entry without reasonable notice in non-emergency violates tenant privacy".to_string(),
                notes: "VIOLATION: landlord entered rental unit without reasonable notice in non-emergency under § 504B.211; tenant privacy right violated.".to_string(),
                citations,
            },
        },
        ComplianceAspect::UnlawfulUtilityTerminationProhibitedUnderSection504B221 => match input
            .utility_termination_status
        {
            UtilityTerminationStatus::UtilitiesMaintainedByLandlord => Output {
                mode: MnLandlordTenantMode::CompliantUtilitiesMaintained,
                statutory_basis: "Minn. Stat. § 504B.221 — utilities maintained by landlord".to_string(),
                notes: "COMPLIANT: landlord maintained utilities (water / hot water / heat / fuel / electricity / gas / telephone / essential services) under § 504B.221.".to_string(),
                citations,
            },
            UtilityTerminationStatus::LandlordUnlawfullyTerminatedUtilities => Output {
                mode: MnLandlordTenantMode::ViolationUnlawfulUtilityTerminationByLandlord,
                statutory_basis: "Minn. Stat. § 504B.221 — landlord unlawfully terminated utilities".to_string(),
                notes: "VIOLATION: landlord unlawfully terminated utilities under § 504B.221; landlord faces civil damages + statutory penalties + tenant remedies.".to_string(),
                citations,
            },
        },
        ComplianceAspect::RentEscrowActionUnderSection504B385 => match input.rent_escrow_prerequisite_status {
            RentEscrowPrerequisiteStatus::TenantSatisfiedNoticeAndReasonableTimePrerequisites => {
                Output {
                    mode: MnLandlordTenantMode::CompliantRentEscrowNoticeAndReasonableTimePrerequisitesMet,
                    statutory_basis: "Minn. Stat. § 504B.385 — tenant satisfied notice and reasonable time prerequisites before filing rent escrow action".to_string(),
                    notes: "COMPLIANT: tenant satisfied § 504B.385 prerequisites (written notice of habitability violation + reasonable time for landlord to repair) before filing rent escrow action.".to_string(),
                    citations,
                }
            }
            RentEscrowPrerequisiteStatus::TenantInvokedEscrowWithoutNoticeOrReasonableTime => Output {
                mode: MnLandlordTenantMode::ViolationRentEscrowInvokedWithoutNoticeOrReasonableTime,
                statutory_basis: "Minn. Stat. § 504B.385 — rent escrow invoked without notice or reasonable time prerequisites".to_string(),
                notes: "VIOLATION: tenant invoked rent escrow action without satisfying § 504B.385 prerequisites (written notice of habitability violation + reasonable time for landlord to repair).".to_string(),
                citations,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_coverage: TenancyCoverage::CoveredResidentialTenancy,
            landlord_covenant_waiver_status:
                LandlordCovenantWaiverStatus::LandlordCovenantsNotWaivedOrModified,
            entry_notice_status: EntryNoticeStatus::ReasonableNotice24HoursOrMore,
            utility_termination_status: UtilityTerminationStatus::UtilitiesMaintainedByLandlord,
            rent_escrow_prerequisite_status:
                RentEscrowPrerequisiteStatus::TenantSatisfiedNoticeAndReasonableTimePrerequisites,
            compliance_aspect: ComplianceAspect::ThreeWeekDepositReturnUnderSection504B178,
            days_to_return_deposit: 15,
            interest_paid_at_one_percent_per_annum: true,
            deposit_wrongfully_withheld: false,
            pay_or_quit_notice_days_given: 14,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromChapter504B;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::NotApplicableTenancyExemptFromChapter504B
        );
    }

    #[test]
    fn deposit_returned_at_21_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeWeekDepositReturnUnderSection504B178;
        input.days_to_return_deposit = 21;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantDepositReturnedWithinThreeWeeks
        );
    }

    #[test]
    fn deposit_returned_at_22_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeWeekDepositReturnUnderSection504B178;
        input.days_to_return_deposit = 22;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationDepositReturnedPastThreeWeekDeadline
        );
    }

    #[test]
    fn interest_paid_at_one_percent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OnePercentInterestOnDepositUnderSection504B178;
        input.interest_paid_at_one_percent_per_annum = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantInterestPaidAtOnePercentPerAnnum
        );
    }

    #[test]
    fn interest_not_paid_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OnePercentInterestOnDepositUnderSection504B178;
        input.interest_paid_at_one_percent_per_annum = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationInterestNotPaidOrUnderpaid
        );
    }

    #[test]
    fn no_wrongful_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrongfulWithholdingPenaltyUnderSection504B178;
        input.deposit_wrongfully_withheld = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantNoWrongfulWithholding
        );
    }

    #[test]
    fn wrongful_withholding_doubles_deposit_exposure_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrongfulWithholdingPenaltyUnderSection504B178;
        input.deposit_wrongfully_withheld = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationWrongfulWithholdingDoublesDepositExposure
        );
    }

    #[test]
    fn non_waivable_covenants_preserved_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::NonWaivableHabitabilityCovenantsUnderSection504B161;
        input.landlord_covenant_waiver_status =
            LandlordCovenantWaiverStatus::LandlordCovenantsNotWaivedOrModified;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantNonWaivableHabitabilityCovenantsPreserved
        );
    }

    #[test]
    fn lease_attempts_to_waive_covenants_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::NonWaivableHabitabilityCovenantsUnderSection504B161;
        input.landlord_covenant_waiver_status =
            LandlordCovenantWaiverStatus::LeaseAttemptsToWaiveOrModifyLandlordCovenants;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationLeaseAttemptsToWaiveOrModifyNonWaivableCovenants
        );
    }

    #[test]
    fn pay_or_quit_at_14_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FourteenDayPayOrQuitNoticeUnderSection504B135;
        input.pay_or_quit_notice_days_given = 14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantFourteenDayPayOrQuitNoticeProperlyServed
        );
    }

    #[test]
    fn pay_or_quit_at_13_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FourteenDayPayOrQuitNoticeUnderSection504B135;
        input.pay_or_quit_notice_days_given = 13;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationPayOrQuitNoticeShorterThan14Days
        );
    }

    #[test]
    fn reasonable_entry_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EntryNoticeUnderSection504B211;
        input.entry_notice_status = EntryNoticeStatus::ReasonableNotice24HoursOrMore;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantReasonableEntryNoticeProvided
        );
    }

    #[test]
    fn emergency_entry_without_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EntryNoticeUnderSection504B211;
        input.entry_notice_status = EntryNoticeStatus::EmergencyEntryWithoutNotice;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantEmergencyEntryWithoutNotice
        );
    }

    #[test]
    fn entry_without_reasonable_notice_not_emergency_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EntryNoticeUnderSection504B211;
        input.entry_notice_status = EntryNoticeStatus::EntryWithoutReasonableNoticeNotEmergency;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationEntryWithoutReasonableNoticeNotEmergency
        );
    }

    #[test]
    fn utilities_maintained_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::UnlawfulUtilityTerminationProhibitedUnderSection504B221;
        input.utility_termination_status = UtilityTerminationStatus::UtilitiesMaintainedByLandlord;
        let out = check(&input);
        assert_eq!(out.mode, MnLandlordTenantMode::CompliantUtilitiesMaintained);
    }

    #[test]
    fn unlawful_utility_termination_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::UnlawfulUtilityTerminationProhibitedUnderSection504B221;
        input.utility_termination_status =
            UtilityTerminationStatus::LandlordUnlawfullyTerminatedUtilities;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationUnlawfulUtilityTerminationByLandlord
        );
    }

    #[test]
    fn rent_escrow_with_notice_and_reasonable_time_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentEscrowActionUnderSection504B385;
        input.rent_escrow_prerequisite_status =
            RentEscrowPrerequisiteStatus::TenantSatisfiedNoticeAndReasonableTimePrerequisites;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::CompliantRentEscrowNoticeAndReasonableTimePrerequisitesMet
        );
    }

    #[test]
    fn rent_escrow_without_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentEscrowActionUnderSection504B385;
        input.rent_escrow_prerequisite_status =
            RentEscrowPrerequisiteStatus::TenantInvokedEscrowWithoutNoticeOrReasonableTime;
        let out = check(&input);
        assert_eq!(
            out.mode,
            MnLandlordTenantMode::ViolationRentEscrowInvokedWithoutNoticeOrReasonableTime
        );
    }

    #[test]
    fn constants_pin_minnesota_landlord_tenant_statutory_thresholds() {
        assert_eq!(MN_CHAPTER_NUMBER, 504);
        assert_eq!(MN_SECURITY_DEPOSIT_RETURN_DEADLINE_WEEKS, 3);
        assert_eq!(MN_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 21);
        assert_eq!(MN_SECURITY_DEPOSIT_INTEREST_RATE_BPS_PER_ANNUM, 100);
        assert_eq!(MN_PAY_OR_QUIT_NOTICE_DAYS, 14);
        assert_eq!(MN_LANDLORD_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(MN_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_minnesota_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Minnesota Landlord and Tenant Act"));
        assert!(joined.contains("Minn. Stat. Chapter 504B"));
        assert!(joined.contains("Minn. Stat. § 504B.178"));
        assert!(joined.contains("THREE WEEKS"));
        assert!(joined.contains("21 days"));
        assert!(joined.contains("SIMPLE NONCOMPOUNDED INTEREST"));
        assert!(joined.contains("ONE PERCENT (1%) PER ANNUM"));
        assert!(joined.contains("FAIR PREPONDERANCE OF THE EVIDENCE"));
        assert!(
            joined.contains("DAMAGES IN AN AMOUNT EQUAL TO THE PORTION OF THE DEPOSIT WITHHELD")
        );
        assert!(joined.contains("INTEREST THEREON"));
        assert!(joined.contains("PENALTY"));
        assert!(joined.contains("Minn. Stat. § 504B.161"));
        assert!(joined.contains("FIT FOR THE USE INTENDED BY THE PARTIES"));
        assert!(joined.contains("REASONABLE REPAIR"));
        assert!(joined.contains("REASONABLY ENERGY EFFICIENT"));
        assert!(joined.contains("HEALTH AND SAFETY LAWS"));
        assert!(joined.contains("MAY NOT WAIVE OR MODIFY"));
        assert!(joined.contains("Minn. Stat. § 504B.135"));
        assert!(joined.contains("14 DAYS NOTICE TO QUIT IN WRITING"));
        assert!(joined.contains("Minn. Stat. § 504B.211"));
        assert!(joined.contains("REASONABLE NOTICE"));
        assert!(joined.contains("24 HOURS"));
        assert!(joined.contains("Minn. Stat. § 504B.221"));
        assert!(joined.contains("NOT INTENTIONALLY TERMINATE"));
        assert!(joined.contains("Minn. Stat. § 504B.385"));
        assert!(joined.contains("RENT ESCROW ACTION"));
    }
}
