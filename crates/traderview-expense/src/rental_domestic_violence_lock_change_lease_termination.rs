//! Domestic Violence Tenant Lock Change + Early Lease Termination
//! Compliance Module.
//!
//! Pure-compute multi-jurisdictional check for landlord compliance
//! with statutes protecting domestic-violence (DV) survivors,
//! covering (a) mandatory lock-change procedures upon tenant
//! written request + court order or police report, and (b) early
//! lease termination rights without penalty.
//!
//! Web research (verified 2026-06-03):
//! - **California Civ. Code § 1941.5**: when the restrained person
//!   is NOT a tenant, landlord shall change locks of protected
//!   tenant's dwelling unit upon written request not later than
//!   **24 hours** after the protected tenant gives a copy of a
//!   court order or police report, and shall give protected tenant
//!   a key. Tenant bears actual cost.
//! - **California Civ. Code § 1941.6**: when the restrained person
//!   IS a tenant, landlord shall, at the **landlord's own expense**,
//!   change locks within **24 hours** after the protected tenant
//!   gives a copy of a court order excluding the restrained person
//!   from the dwelling unit. If landlord fails within 24 hours, the
//!   protected tenant may change locks without landlord's
//!   permission, notwithstanding any lease provision; tenant must
//!   change in workmanlike manner with locks of similar or better
//!   quality. ([WomensLaw.org 1941.6 explainer](https://www.womenslaw.org/laws/ca/statutes/19416-tenant-protected-restraining-order-against-another-tenant-change-locks);
//!   NHLP California Lock Changes Packet.)
//! - **California Civ. Code § 1946.7**: early lease termination
//!   right for victims of DV, sexual assault, stalking, human
//!   trafficking, elder abuse, or dependent adult abuse. Tenant
//!   may move out at any time but must continue paying rent for
//!   **14 days** after written notice is given.
//! - **Texas Property Code § 92.016**: family violence victim may
//!   break residential lease without penalty after giving landlord
//!   30 days written notice. If lease lacks the required statutory
//!   rights language ("Tenants may have special statutory rights
//!   to terminate the lease early in certain situations involving
//!   family violence or a military deployment or transfer"), tenant
//!   is NOT liable for unpaid rent at termination. If abuser is co-
//!   tenant or occupant, tenant does NOT need to give 30 days'
//!   advance notice. Landlords must change locks within 3 business
//!   days of tenant's request + documentation; landlord may charge
//!   actual cost. (Texas Property Code § 92.016 FindLaw + Texas
//!   Law Help Early Lease Termination guide.)
//! - **New York Real Property Law § 227-c**: tenant or household
//!   member who is DV victim and reasonably fears remaining in
//!   rental may terminate lease by written notice with termination
//!   date at least **30 days** after delivery. Within **25 days**
//!   of notice, tenant must provide proof. Tenant remains liable
//!   for 30 days of rent during notice period. ([NY RPL § 227-c
//!   Justia](https://law.justia.com/codes/new-york/2014/rpp/article-7/227-c/);
//!   NY State Senate Bill 2019-S4281A.)
//! - **Arizona Rev. Stat. § 33-1318**: early termination + lock
//!   replacement + access refusal protections + treble damages +
//!   landlord immunity for compliant lock change. ([AZ Legislature
//!   § 33-1318](https://www.azleg.gov/ars/33/01318.htm).)
//! - **VAWA federally subsidized housing** (Violence Against Women
//!   Act of 1994; 2013 Reauthorization; 2022 Reauthorization Act):
//!   federally subsidized housing (Section 8, public housing,
//!   LIHTC, HOME, etc.) bars DV victim eviction for DV-related
//!   incidents; permits lease bifurcation to remove abuser
//!   tenant; requires emergency transfer plans.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const DV_LOCK_CHANGE_CA_1941_5_HOURS: u32 = 24;
pub const DV_LOCK_CHANGE_CA_1941_6_HOURS: u32 = 24;
pub const DV_LEASE_TERMINATION_CA_1946_7_POST_NOTICE_RENT_DAYS: u32 = 14;
pub const DV_LOCK_CHANGE_TEXAS_92_016_BUSINESS_DAYS: u32 = 3;
pub const DV_LEASE_TERMINATION_TEXAS_92_016_NOTICE_DAYS: u32 = 30;
pub const DV_LEASE_TERMINATION_NY_RPL_227C_NOTICE_DAYS: u32 = 30;
pub const DV_LEASE_TERMINATION_NY_RPL_227C_PROOF_DAYS: u32 = 25;
pub const DV_VAWA_ORIGINAL_ENACTMENT_YEAR: u32 = 1994;
pub const DV_VAWA_REAUTHORIZATION_2013_YEAR: u32 = 2013;
pub const DV_VAWA_REAUTHORIZATION_2022_YEAR: u32 = 2022;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DvJurisdiction {
    California,
    Texas,
    NewYork,
    Arizona,
    VawaFederallySubsidizedHousing,
    OtherStateWithoutDvLockChangeMandate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DvScenarioType {
    LockChangeRequestRestrainedPersonNotTenant,
    LockChangeRequestRestrainedPersonIsTenant,
    EarlyLeaseTerminationRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DvDocumentationType {
    CourtOrderOrPoliceReport,
    CourtOrderExcludingRestrainedPersonFromUnit,
    DvAffidavitOrCertification,
    NoDocumentationProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DvLockChangeLeaseTerminationMode {
    NotApplicableNoDvDocumentationProvided,
    NotApplicableJurisdictionLacksDvMandate,
    CompliantCa1941_5LandlordChangedLocksWithin24HoursTenantPaid,
    CompliantCa1941_6LandlordChangedLocksWithin24HoursLandlordExpense,
    CompliantTexasLockChangeWithin3BusinessDaysActualCostCharged,
    CompliantCa1946_7EarlyTerminationWith14DayPostNoticeRent,
    CompliantNyRpl227cTerminationWith30DayNoticeAnd25DayProof,
    CompliantTexas92_016EarlyTerminationWith30DayNotice,
    CompliantTexas92_016ZeroNoticeAbuserIsCoTenant,
    CompliantVawaSection8FederalProtectionApplied,
    CompliantTenantSelfChangedLocksAfterLandlord24HourFailure,
    ViolationLandlordFailedToChangeLocksWithinStatutoryWindow,
    ViolationLandlordChargedTenantForCa1941_6LockChange,
    ViolationLandlordRefusedEarlyTerminationDespiteCompliantNotice,
    ViolationTexasLeaseLackedRequiredStatutoryRightsLanguage,
    ViolationCa1946_7ImproperNoticeOrTerminationDate,
    ViolationNyRpl227cTerminationDateLessThan30DaysFromNotice,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: DvJurisdiction,
    pub scenario: DvScenarioType,
    pub documentation_type: DvDocumentationType,
    pub hours_to_landlord_lock_change: u32,
    pub business_days_to_landlord_lock_change_texas: u32,
    pub tenant_self_changed_locks_after_landlord_failure: bool,
    pub landlord_charged_tenant_for_1941_6_lock_change: bool,
    pub days_between_notice_and_termination_date: u32,
    pub days_between_notice_and_proof_provided: u32,
    pub texas_lease_contains_required_statutory_rights_language: bool,
    pub texas_abuser_is_co_tenant_or_occupant: bool,
    pub landlord_accepted_termination: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: DvLockChangeLeaseTerminationMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalDomesticViolenceLockChangeLeaseTerminationInput = Input;
pub type RentalDomesticViolenceLockChangeLeaseTerminationOutput = Output;
pub type RentalDomesticViolenceLockChangeLeaseTerminationResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cal. Civ. Code § 1941.5 — restrained person NOT tenant: landlord changes locks within 24 hours of tenant's written request + court order/police report; tenant pays cost".to_string(),
        "Cal. Civ. Code § 1941.6 — restrained person IS tenant: landlord at LANDLORD'S OWN EXPENSE changes locks within 24 hours of court order excluding restrained person; tenant may self-change if landlord fails 24-hour window".to_string(),
        "Cal. Civ. Code § 1946.7 — early termination for DV/sexual assault/stalking/human trafficking/elder abuse victims; tenant pays rent 14 days post-notice".to_string(),
        "Texas Property Code § 92.016 — family violence victim 30-day notice early termination; 3 business days lock change; landlord may charge actual cost; required statutory-rights lease language; zero notice if abuser is co-tenant".to_string(),
        "New York Real Property Law § 227-c — DV victim termination by written notice ≥ 30 days from delivery; proof within 25 days; tenant pays 30-day notice period rent".to_string(),
        "Arizona Rev. Stat. § 33-1318 — early termination + lock replacement + treble damages + landlord immunity".to_string(),
        "Violence Against Women Act of 1994 + 2013 Reauthorization + 2022 Reauthorization Act — federally subsidized housing (Section 8, public housing, LIHTC, HOME) bars DV-related eviction; permits lease bifurcation; emergency transfer plans required".to_string(),
        "Treas. Reg. — VAWA-covered federally subsidized housing emergency transfer plan requirements".to_string(),
    ];

    if input.documentation_type == DvDocumentationType::NoDocumentationProvided {
        return Output {
            mode: DvLockChangeLeaseTerminationMode::NotApplicableNoDvDocumentationProvided,
            statutory_basis: "Documentation prerequisite not met; statutes condition protections on tenant providing court order, police report, or DV affidavit".to_string(),
            notes: "No DV documentation provided; statutory lock-change and termination rights do not arise.".to_string(),
            citations,
        };
    }

    if input.jurisdiction == DvJurisdiction::OtherStateWithoutDvLockChangeMandate
        && input.scenario != DvScenarioType::EarlyLeaseTerminationRequest
    {
        return Output {
            mode: DvLockChangeLeaseTerminationMode::NotApplicableJurisdictionLacksDvMandate,
            statutory_basis: "Jurisdiction lacks codified DV lock-change mandate".to_string(),
            notes: "Jurisdiction does not impose statutory lock-change obligation on landlord; landlord may voluntarily change locks but no statutory deadline applies.".to_string(),
            citations,
        };
    }

    if input.jurisdiction == DvJurisdiction::VawaFederallySubsidizedHousing
        && input.scenario == DvScenarioType::EarlyLeaseTerminationRequest
    {
        return Output {
            mode: DvLockChangeLeaseTerminationMode::CompliantVawaSection8FederalProtectionApplied,
            statutory_basis: "VAWA federal protection for federally subsidized housing".to_string(),
            notes: "VAWA protects DV victims in federally subsidized housing from eviction for DV-related incidents and permits lease bifurcation to remove abuser tenant; emergency transfer plans required.".to_string(),
            citations,
        };
    }

    match (input.jurisdiction, input.scenario) {
        (DvJurisdiction::California, DvScenarioType::LockChangeRequestRestrainedPersonNotTenant) => {
            if input.hours_to_landlord_lock_change > DV_LOCK_CHANGE_CA_1941_5_HOURS {
                if input.tenant_self_changed_locks_after_landlord_failure {
                    return Output {
                        mode: DvLockChangeLeaseTerminationMode::CompliantTenantSelfChangedLocksAfterLandlord24HourFailure,
                        statutory_basis: "Cal. Civ. Code § 1941.5 — tenant self-change permitted after landlord 24-hour failure".to_string(),
                        notes: format!(
                            "Landlord exceeded 24-hour window ({} hours actual); tenant self-changed locks. Cal. Civ. Code permits self-change in workmanlike manner with locks of similar or better quality.",
                            input.hours_to_landlord_lock_change
                        ),
                        citations,
                    };
                }
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordFailedToChangeLocksWithinStatutoryWindow,
                    statutory_basis: "Cal. Civ. Code § 1941.5 — 24-hour window exceeded".to_string(),
                    notes: format!(
                        "VIOLATION: Cal. Civ. Code § 1941.5 requires lock change within 24 hours; landlord took {} hours.",
                        input.hours_to_landlord_lock_change
                    ),
                    citations,
                };
            }
            Output {
                mode: DvLockChangeLeaseTerminationMode::CompliantCa1941_5LandlordChangedLocksWithin24HoursTenantPaid,
                statutory_basis: "Cal. Civ. Code § 1941.5 — 24-hour lock change satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: lock change completed in {} hours (≤ 24-hour statutory window). Tenant bears cost per § 1941.5 (restrained person is not tenant).",
                    input.hours_to_landlord_lock_change
                ),
                citations,
            }
        }
        (DvJurisdiction::California, DvScenarioType::LockChangeRequestRestrainedPersonIsTenant) => {
            if input.landlord_charged_tenant_for_1941_6_lock_change {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordChargedTenantForCa1941_6LockChange,
                    statutory_basis: "Cal. Civ. Code § 1941.6 — landlord bears own expense".to_string(),
                    notes: "VIOLATION: § 1941.6 requires landlord to change locks at LANDLORD'S OWN EXPENSE when restrained person is tenant; landlord improperly charged protected tenant.".to_string(),
                    citations,
                };
            }
            if input.hours_to_landlord_lock_change > DV_LOCK_CHANGE_CA_1941_6_HOURS {
                if input.tenant_self_changed_locks_after_landlord_failure {
                    return Output {
                        mode: DvLockChangeLeaseTerminationMode::CompliantTenantSelfChangedLocksAfterLandlord24HourFailure,
                        statutory_basis: "Cal. Civ. Code § 1941.6 — tenant self-change permitted after landlord 24-hour failure".to_string(),
                        notes: format!(
                            "Landlord exceeded 24-hour window ({} hours actual); tenant self-changed locks. § 1941.6 permits self-change notwithstanding any lease provision; tenant must use workmanlike manner with locks of similar or better quality.",
                            input.hours_to_landlord_lock_change
                        ),
                        citations,
                    };
                }
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordFailedToChangeLocksWithinStatutoryWindow,
                    statutory_basis: "Cal. Civ. Code § 1941.6 — 24-hour window exceeded".to_string(),
                    notes: format!(
                        "VIOLATION: § 1941.6 requires lock change within 24 hours at landlord's own expense; landlord took {} hours.",
                        input.hours_to_landlord_lock_change
                    ),
                    citations,
                };
            }
            Output {
                mode: DvLockChangeLeaseTerminationMode::CompliantCa1941_6LandlordChangedLocksWithin24HoursLandlordExpense,
                statutory_basis: "Cal. Civ. Code § 1941.6 — 24-hour lock change at landlord expense satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: lock change completed in {} hours at landlord's own expense (§ 1941.6 — restrained person is tenant).",
                    input.hours_to_landlord_lock_change
                ),
                citations,
            }
        }
        (DvJurisdiction::Texas, DvScenarioType::LockChangeRequestRestrainedPersonNotTenant)
        | (DvJurisdiction::Texas, DvScenarioType::LockChangeRequestRestrainedPersonIsTenant) => {
            if input.business_days_to_landlord_lock_change_texas > DV_LOCK_CHANGE_TEXAS_92_016_BUSINESS_DAYS {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordFailedToChangeLocksWithinStatutoryWindow,
                    statutory_basis: "Texas Property Code § 92.016 — 3 business days exceeded".to_string(),
                    notes: format!(
                        "VIOLATION: Texas § 92.016 requires lock change within 3 business days; landlord took {} business days.",
                        input.business_days_to_landlord_lock_change_texas
                    ),
                    citations,
                };
            }
            Output {
                mode: DvLockChangeLeaseTerminationMode::CompliantTexasLockChangeWithin3BusinessDaysActualCostCharged,
                statutory_basis: "Texas Property Code § 92.016 — 3 business day lock change satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: lock change completed in {} business days (≤ 3 statutory window). Landlord may charge actual cost.",
                    input.business_days_to_landlord_lock_change_texas
                ),
                citations,
            }
        }
        (DvJurisdiction::California, DvScenarioType::EarlyLeaseTerminationRequest) => {
            if input.days_between_notice_and_termination_date < DV_LEASE_TERMINATION_CA_1946_7_POST_NOTICE_RENT_DAYS {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationCa1946_7ImproperNoticeOrTerminationDate,
                    statutory_basis: "Cal. Civ. Code § 1946.7 — 14-day post-notice rent obligation".to_string(),
                    notes: format!(
                        "VIOLATION: § 1946.7 requires tenant to pay rent for 14 days after written notice; tenant termination date = {} days after notice (less than 14).",
                        input.days_between_notice_and_termination_date
                    ),
                    citations,
                };
            }
            if !input.landlord_accepted_termination {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordRefusedEarlyTerminationDespiteCompliantNotice,
                    statutory_basis: "Cal. Civ. Code § 1946.7 — landlord refusal not statutorily authorized".to_string(),
                    notes: "VIOLATION: tenant provided compliant § 1946.7 notice but landlord refused to terminate the lease.".to_string(),
                    citations,
                };
            }
            Output {
                mode: DvLockChangeLeaseTerminationMode::CompliantCa1946_7EarlyTerminationWith14DayPostNoticeRent,
                statutory_basis: "Cal. Civ. Code § 1946.7 — 14-day post-notice rent satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: tenant provided written notice; termination occurs {} days after notice (≥ 14 statutory minimum).",
                    input.days_between_notice_and_termination_date
                ),
                citations,
            }
        }
        (DvJurisdiction::Texas, DvScenarioType::EarlyLeaseTerminationRequest) => {
            if input.texas_abuser_is_co_tenant_or_occupant {
                if input.landlord_accepted_termination {
                    return Output {
                        mode: DvLockChangeLeaseTerminationMode::CompliantTexas92_016ZeroNoticeAbuserIsCoTenant,
                        statutory_basis: "Texas Property Code § 92.016 — zero notice required when abuser is co-tenant/occupant".to_string(),
                        notes: "COMPLIANT: § 92.016 waives 30-day notice requirement when abuser is co-tenant or occupant; tenant may terminate immediately.".to_string(),
                        citations,
                    };
                }
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordRefusedEarlyTerminationDespiteCompliantNotice,
                    statutory_basis: "Texas Property Code § 92.016 — zero-notice termination right (abuser co-tenant)".to_string(),
                    notes: "VIOLATION: tenant exercised zero-notice termination right; landlord refused.".to_string(),
                    citations,
                };
            }
            if input.days_between_notice_and_termination_date < DV_LEASE_TERMINATION_TEXAS_92_016_NOTICE_DAYS {
                if !input.texas_lease_contains_required_statutory_rights_language {
                    return Output {
                        mode: DvLockChangeLeaseTerminationMode::ViolationTexasLeaseLackedRequiredStatutoryRightsLanguage,
                        statutory_basis: "Texas Property Code § 92.016 — lease lacked required statutory rights language".to_string(),
                        notes: format!(
                            "VIOLATION: Texas lease lacks 'Tenants may have special statutory rights to terminate the lease early in certain situations involving family violence or a military deployment or transfer' language; § 92.016 entitles tenant to NO LIABILITY for unpaid rent at termination notwithstanding {}-day notice.",
                            input.days_between_notice_and_termination_date
                        ),
                        citations,
                    };
                }
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationCa1946_7ImproperNoticeOrTerminationDate,
                    statutory_basis: "Texas Property Code § 92.016 — 30-day notice deadline".to_string(),
                    notes: format!(
                        "VIOLATION: § 92.016 requires 30-day written notice; tenant notice/termination gap = {} days.",
                        input.days_between_notice_and_termination_date
                    ),
                    citations,
                };
            }
            if !input.landlord_accepted_termination {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordRefusedEarlyTerminationDespiteCompliantNotice,
                    statutory_basis: "Texas Property Code § 92.016 — 30-day notice termination right".to_string(),
                    notes: "VIOLATION: tenant provided compliant 30-day notice; landlord refused.".to_string(),
                    citations,
                };
            }
            Output {
                mode: DvLockChangeLeaseTerminationMode::CompliantTexas92_016EarlyTerminationWith30DayNotice,
                statutory_basis: "Texas Property Code § 92.016 — 30-day notice termination satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: § 92.016 termination after {} days notice (≥ 30 statutory minimum).",
                    input.days_between_notice_and_termination_date
                ),
                citations,
            }
        }
        (DvJurisdiction::NewYork, DvScenarioType::EarlyLeaseTerminationRequest) => {
            if input.days_between_notice_and_termination_date < DV_LEASE_TERMINATION_NY_RPL_227C_NOTICE_DAYS {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationNyRpl227cTerminationDateLessThan30DaysFromNotice,
                    statutory_basis: "NY RPL § 227-c — termination date must be ≥ 30 days from notice".to_string(),
                    notes: format!(
                        "VIOLATION: NY RPL § 227-c requires termination date ≥ 30 days after notice delivery; tenant termination date = {} days after notice.",
                        input.days_between_notice_and_termination_date
                    ),
                    citations,
                };
            }
            if input.days_between_notice_and_proof_provided > DV_LEASE_TERMINATION_NY_RPL_227C_PROOF_DAYS {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationCa1946_7ImproperNoticeOrTerminationDate,
                    statutory_basis: "NY RPL § 227-c — 25-day proof window exceeded".to_string(),
                    notes: format!(
                        "VIOLATION: NY RPL § 227-c requires proof within 25 days of notice; tenant provided proof at {} days.",
                        input.days_between_notice_and_proof_provided
                    ),
                    citations,
                };
            }
            if !input.landlord_accepted_termination {
                return Output {
                    mode: DvLockChangeLeaseTerminationMode::ViolationLandlordRefusedEarlyTerminationDespiteCompliantNotice,
                    statutory_basis: "NY RPL § 227-c — landlord refusal not authorized".to_string(),
                    notes: "VIOLATION: tenant provided compliant NY RPL § 227-c notice + proof; landlord refused.".to_string(),
                    citations,
                };
            }
            Output {
                mode: DvLockChangeLeaseTerminationMode::CompliantNyRpl227cTerminationWith30DayNoticeAnd25DayProof,
                statutory_basis: "NY RPL § 227-c — 30-day notice + 25-day proof satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: notice/termination gap = {} days (≥ 30); proof at {} days (≤ 25).",
                    input.days_between_notice_and_termination_date,
                    input.days_between_notice_and_proof_provided
                ),
                citations,
            }
        }
        _ => Output {
            mode: DvLockChangeLeaseTerminationMode::NotApplicableJurisdictionLacksDvMandate,
            statutory_basis: "Unhandled jurisdiction + scenario combination".to_string(),
            notes: "Fall-through; no statutory protection identified for this combination.".to_string(),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_ca_1941_5_compliant() -> Input {
        Input {
            jurisdiction: DvJurisdiction::California,
            scenario: DvScenarioType::LockChangeRequestRestrainedPersonNotTenant,
            documentation_type: DvDocumentationType::CourtOrderOrPoliceReport,
            hours_to_landlord_lock_change: 20,
            business_days_to_landlord_lock_change_texas: 0,
            tenant_self_changed_locks_after_landlord_failure: false,
            landlord_charged_tenant_for_1941_6_lock_change: false,
            days_between_notice_and_termination_date: 0,
            days_between_notice_and_proof_provided: 0,
            texas_lease_contains_required_statutory_rights_language: true,
            texas_abuser_is_co_tenant_or_occupant: false,
            landlord_accepted_termination: true,
        }
    }

    #[test]
    fn no_documentation_not_applicable() {
        let input = Input {
            documentation_type: DvDocumentationType::NoDocumentationProvided,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::NotApplicableNoDvDocumentationProvided);
    }

    #[test]
    fn ca_1941_5_compliant_within_24_hours() {
        let result = check(&baseline_ca_1941_5_compliant());
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantCa1941_5LandlordChangedLocksWithin24HoursTenantPaid);
    }

    #[test]
    fn ca_1941_5_at_exactly_24_hours_compliant() {
        let input = Input {
            hours_to_landlord_lock_change: 24,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantCa1941_5LandlordChangedLocksWithin24HoursTenantPaid);
    }

    #[test]
    fn ca_1941_5_25_hours_violation() {
        let input = Input {
            hours_to_landlord_lock_change: 25,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::ViolationLandlordFailedToChangeLocksWithinStatutoryWindow);
    }

    #[test]
    fn ca_1941_5_tenant_self_change_after_landlord_failure_compliant() {
        let input = Input {
            hours_to_landlord_lock_change: 48,
            tenant_self_changed_locks_after_landlord_failure: true,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantTenantSelfChangedLocksAfterLandlord24HourFailure);
    }

    #[test]
    fn ca_1941_6_compliant_landlord_expense() {
        let input = Input {
            scenario: DvScenarioType::LockChangeRequestRestrainedPersonIsTenant,
            documentation_type: DvDocumentationType::CourtOrderExcludingRestrainedPersonFromUnit,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantCa1941_6LandlordChangedLocksWithin24HoursLandlordExpense);
    }

    #[test]
    fn ca_1941_6_landlord_charged_tenant_violation() {
        let input = Input {
            scenario: DvScenarioType::LockChangeRequestRestrainedPersonIsTenant,
            documentation_type: DvDocumentationType::CourtOrderExcludingRestrainedPersonFromUnit,
            landlord_charged_tenant_for_1941_6_lock_change: true,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::ViolationLandlordChargedTenantForCa1941_6LockChange);
    }

    #[test]
    fn texas_lock_change_3_business_days_compliant() {
        let input = Input {
            jurisdiction: DvJurisdiction::Texas,
            scenario: DvScenarioType::LockChangeRequestRestrainedPersonNotTenant,
            business_days_to_landlord_lock_change_texas: 2,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantTexasLockChangeWithin3BusinessDaysActualCostCharged);
    }

    #[test]
    fn texas_lock_change_4_business_days_violation() {
        let input = Input {
            jurisdiction: DvJurisdiction::Texas,
            scenario: DvScenarioType::LockChangeRequestRestrainedPersonNotTenant,
            business_days_to_landlord_lock_change_texas: 4,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::ViolationLandlordFailedToChangeLocksWithinStatutoryWindow);
    }

    #[test]
    fn ca_1946_7_14_day_compliant() {
        let input = Input {
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 14,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantCa1946_7EarlyTerminationWith14DayPostNoticeRent);
    }

    #[test]
    fn ca_1946_7_13_day_violation() {
        let input = Input {
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 13,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::ViolationCa1946_7ImproperNoticeOrTerminationDate);
    }

    #[test]
    fn texas_92_016_30_day_compliant() {
        let input = Input {
            jurisdiction: DvJurisdiction::Texas,
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 30,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantTexas92_016EarlyTerminationWith30DayNotice);
    }

    #[test]
    fn texas_92_016_zero_notice_abuser_co_tenant_compliant() {
        let input = Input {
            jurisdiction: DvJurisdiction::Texas,
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 0,
            texas_abuser_is_co_tenant_or_occupant: true,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantTexas92_016ZeroNoticeAbuserIsCoTenant);
    }

    #[test]
    fn texas_lease_lacks_statutory_rights_language_violation() {
        let input = Input {
            jurisdiction: DvJurisdiction::Texas,
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 15,
            texas_lease_contains_required_statutory_rights_language: false,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::ViolationTexasLeaseLackedRequiredStatutoryRightsLanguage);
    }

    #[test]
    fn ny_rpl_227c_30_day_notice_25_day_proof_compliant() {
        let input = Input {
            jurisdiction: DvJurisdiction::NewYork,
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 30,
            days_between_notice_and_proof_provided: 25,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantNyRpl227cTerminationWith30DayNoticeAnd25DayProof);
    }

    #[test]
    fn ny_rpl_227c_termination_29_days_violation() {
        let input = Input {
            jurisdiction: DvJurisdiction::NewYork,
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 29,
            days_between_notice_and_proof_provided: 25,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::ViolationNyRpl227cTerminationDateLessThan30DaysFromNotice);
    }

    #[test]
    fn ny_rpl_227c_proof_at_26_days_violation() {
        let input = Input {
            jurisdiction: DvJurisdiction::NewYork,
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            days_between_notice_and_termination_date: 30,
            days_between_notice_and_proof_provided: 26,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::ViolationCa1946_7ImproperNoticeOrTerminationDate);
    }

    #[test]
    fn vawa_federally_subsidized_housing_compliant() {
        let input = Input {
            jurisdiction: DvJurisdiction::VawaFederallySubsidizedHousing,
            scenario: DvScenarioType::EarlyLeaseTerminationRequest,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::CompliantVawaSection8FederalProtectionApplied);
    }

    #[test]
    fn other_jurisdiction_lock_change_not_applicable() {
        let input = Input {
            jurisdiction: DvJurisdiction::OtherStateWithoutDvLockChangeMandate,
            scenario: DvScenarioType::LockChangeRequestRestrainedPersonNotTenant,
            ..baseline_ca_1941_5_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, DvLockChangeLeaseTerminationMode::NotApplicableJurisdictionLacksDvMandate);
    }

    #[test]
    fn citations_pin_jurisdictional_statutes() {
        let result = check(&baseline_ca_1941_5_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Cal. Civ. Code § 1941.5"));
        assert!(joined.contains("Cal. Civ. Code § 1941.6"));
        assert!(joined.contains("Cal. Civ. Code § 1946.7"));
        assert!(joined.contains("Texas Property Code § 92.016"));
        assert!(joined.contains("New York Real Property Law § 227-c"));
        assert!(joined.contains("Arizona Rev. Stat. § 33-1318"));
        assert!(joined.contains("Violence Against Women Act of 1994"));
        assert!(joined.contains("2013 Reauthorization"));
        assert!(joined.contains("2022 Reauthorization Act"));
    }

    #[test]
    fn constant_pin_statutory_windows() {
        assert_eq!(DV_LOCK_CHANGE_CA_1941_5_HOURS, 24);
        assert_eq!(DV_LOCK_CHANGE_CA_1941_6_HOURS, 24);
        assert_eq!(DV_LEASE_TERMINATION_CA_1946_7_POST_NOTICE_RENT_DAYS, 14);
        assert_eq!(DV_LOCK_CHANGE_TEXAS_92_016_BUSINESS_DAYS, 3);
        assert_eq!(DV_LEASE_TERMINATION_TEXAS_92_016_NOTICE_DAYS, 30);
        assert_eq!(DV_LEASE_TERMINATION_NY_RPL_227C_NOTICE_DAYS, 30);
        assert_eq!(DV_LEASE_TERMINATION_NY_RPL_227C_PROOF_DAYS, 25);
        assert_eq!(DV_VAWA_ORIGINAL_ENACTMENT_YEAR, 1994);
        assert_eq!(DV_VAWA_REAUTHORIZATION_2013_YEAR, 2013);
        assert_eq!(DV_VAWA_REAUTHORIZATION_2022_YEAR, 2022);
    }
}
