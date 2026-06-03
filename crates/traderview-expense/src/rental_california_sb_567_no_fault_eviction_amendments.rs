//! California SB 567 of 2023 No-Fault Eviction Amendments
//! Compliance Module — strengthens the AB 1482 (Tenant
//! Protection Act / TPA) no-fault eviction grounds for
//! owner / family-member move-in (OMI) and substantial
//! remodel, with new 12-consecutive-month occupancy
//! requirement, mandatory permit / signed-contractor-
//! agreement documentation, and 3× actual-damages civil
//! liability for willful violations.
//!
//! Pure-compute check for landlord compliance with
//! California Senate Bill 567 (2023-2024 Regular Session;
//! Author: Senator Maria Elena Durazo, D-Los Angeles),
//! signed by Governor Gavin Newsom on **September 30, 2023**
//! and effective **April 1, 2024**. SB 567 amends California
//! Civil Code § 1946.2 (the Tenant Protection Act of 2019
//! enacted by AB 1482) to tighten the OMI and substantial-
//! remodel no-fault eviction grounds. The AB 1482 baseline
//! (5 % + CPI rent cap and 15 just causes for termination)
//! remains in place; SB 567 ratchets the no-fault subset.
//!
//! Web research (verified 2026-06-03):
//! - **SB 567 Enactment**: Signed by Governor Gavin Newsom on
//!   **September 30, 2023**; effective **April 1, 2024**
//!   ([California Legislative Information SB 567](https://leginfo.legislature.ca.gov/faces/billNavClient.xhtml?bill_id=202320240SB567);
//!   [CalMatters Digital Democracy — SB 567 Termination of
//!   tenancy](https://calmatters.digitaldemocracy.org/bills/ca_202320240sb567);
//!   [Public Advocates — Renters' Rights under the Tenant
//!   Protection Act (2024)](https://publicadvocates.org/resources/library/sb-567/);
//!   [Apartment Association of Greater Los Angeles — SB 567
//!   Signed by Governor: New No-Fault Eviction Rules Effective
//!   April 1](https://members.aagla.org/news/senate-bill-567-is-signed-by-governor-new-no-fault-eviction-rules-effective-april-1st)).
//! - **Owner / Family-Member Move-In Requirements**: the owner
//!   or eligible family member must (1) **MOVE IN within 90
//!   DAYS** of the tenant vacating, AND (2) **OCCUPY the
//!   property as their PRIMARY RESIDENCE for AT LEAST 12
//!   CONSECUTIVE MONTHS**. **Permitted family members**:
//!   **spouse, domestic partner, children, grandchildren,
//!   parents, grandparents** (six enumerated categories
//!   under amended Civ. Code § 1946.2(b)(2)(A)(ii)).
//!   **Cause UNAVAILABLE** if the intended occupant
//!   ALREADY RENTS a unit on the property OR if a SIMILAR
//!   VACANT UNIT EXISTS on the property ([Kahana Feld —
//!   Navigating the New Landscape: How AB 12 and SB 567 Impact
//!   Landlords and Tenants](https://kahanafeld.com/2024/02/26/navigating-the-new-landscape-how-ab-12-and-sb-567-impact-landlords-and-tenants-in-california/);
//!   [SoCal Rental Housing Association — SB 567 Guidance](https://www.socalrha.org/sb-567-guidance);
//!   [AOA USA — How Will SB 567 Affect You?](https://aoausa.com/how-will-sb-567-affect-you-by-aoa/)).
//! - **OMI Failure-to-Comply Remedy**: if the owner fails to
//!   meet the 90-day move-in OR 12-month continuous-residency
//!   requirements, the owner MUST OFFER THE UNIT TO THE
//!   TENANT WHO VACATED at the **SAME RENT AND LEASE TERMS**
//!   in effect at vacate AND **REIMBURSE REASONABLE MOVING
//!   EXPENSES**.
//! - **Substantial Remodel / Demolition Requirements**:
//!   (1) the work must require the tenant to **VACATE for
//!   AT LEAST 30 CONSECUTIVE DAYS** AND must not reasonably
//!   be accomplished in a safe manner that would allow the
//!   tenant to remain in the unit; (2) written notice must
//!   include **(a) a description of the substantial remodel,
//!   (b) the expected duration of the work, AND (c) either
//!   the required permit(s) OR a SIGNED CONTRACTOR
//!   AGREEMENT** detailing the work to be performed; (3) if
//!   the substantial remodel is not completed, the tenant
//!   may RECLAIM the unit at the previous rental rate.
//! - **Civil Penalties**: owner who violates the TPA by
//!   improperly terminating tenancy or raising rent beyond
//!   the AB 1482 cap is liable for (1) **ACTUAL DAMAGES**;
//!   (2) **UP TO 3 TIMES actual damages** upon a showing of
//!   **WILLFUL OR MALICIOUS** conduct; (3) **REASONABLE
//!   ATTORNEY'S FEES AND COSTS** (at the court's discretion);
//!   AND (4) **PUNITIVE DAMAGES**.
//! - **Notice Voidability**: an owner's failure to comply
//!   with any of the just-cause provisions renders a written
//!   termination notice **VOID**.
//! - **Underlying AB 1482 Just-Cause Framework**: SB 567
//!   amends the AB 1482 / TPA no-fault subset; the remaining
//!   AB 1482 baseline (5 % + CPI rent cap; 15 just causes;
//!   coverage rules excluding single-family homes not owned
//!   by REIT/LLC/corporation, owner-occupied properties with
//!   ≤ 2 units, and buildings issued certificate of occupancy
//!   within last 15 years) continues to apply ([Martinez Law
//!   Center — California Just Cause Eviction Law SB 567
//!   Requirements](https://martinezlawcenter.com/california-just-cause-eviction-law-sb-567-requirements/);
//!   [Attorney David — SB 567: New Eviction Rules Coming to
//!   California in 2024](https://www.attorneydavid.com/blog/sb-567-new-eviction-rules-coming-to-california-in-2024/)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CA_SB_567_ENACTMENT_YEAR: u32 = 2023;
pub const CA_SB_567_ENACTMENT_MONTH: u32 = 9;
pub const CA_SB_567_ENACTMENT_DAY: u32 = 30;
pub const CA_SB_567_EFFECTIVE_DATE_YEAR: u32 = 2024;
pub const CA_SB_567_EFFECTIVE_DATE_MONTH: u32 = 4;
pub const CA_SB_567_EFFECTIVE_DATE_DAY: u32 = 1;
pub const CA_SB_567_OMI_MOVE_IN_DEADLINE_DAYS: u32 = 90;
pub const CA_SB_567_OMI_CONTINUOUS_OCCUPANCY_MONTHS: u32 = 12;
pub const CA_SB_567_SUBSTANTIAL_REMODEL_TENANT_VACATE_DAYS: u32 = 30;
pub const CA_SB_567_PERMITTED_FAMILY_MEMBER_CATEGORIES_COUNT: u32 = 6;
pub const CA_SB_567_WILLFUL_VIOLATION_DAMAGES_MULTIPLIER: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinCalifornia,
    OutsideCalifornia,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationNoticeDateStatus {
    NoticeServedAtOrAfterApril1_2024PostSb567Effective,
    NoticeServedBeforeApril1_2024PreSb567Effective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitCoverage {
    CoveredByAb1482Tpa,
    ExemptFromAb1482Tpa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoFaultCauseAsserted {
    OwnerOrFamilyMemberMoveIn,
    SubstantialRemodelOrDemolition,
    OtherNoFaultCauseUnderAb1482,
    NotANoFaultCauseAtFaultGround,
    NoCauseAsserted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OmiActualOccupancyStatus {
    OwnerMovedInWithin90DaysAndOccupiedFor12ContinuousMonths,
    OwnerDidNotMoveInWithin90DaysOfTenantVacate,
    OwnerMovedInButDidNotOccupyFor12ContinuousMonths,
    NotApplicableNonOmiCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OmiIntendedOccupantStatus {
    OwnerOrPermittedFamilyMemberPerStatute,
    NotPermittedFamilyMemberOutsideEnumeratedSixCategories,
    IntendedOccupantAlreadyOccupiesAnotherUnitOnProperty,
    SimilarVacantUnitAvailableOnPropertyMustUseThatInstead,
    NotApplicableNonOmiCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstantialRemodelComplianceStatus {
    PermitObtainedAnd30DayVacateRequiredAndNoticeContentComplete,
    PermitOrSignedContractorAgreementNotProvidedWithNotice,
    TenantVacateRequirementUnder30ConsecutiveDays,
    WrittenNoticeMissingRequiredContent,
    NotApplicableNonRemodelCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReoccupancyOfferStatus {
    TenantOfferedReoccupancyAtSameRentAndLeaseTermsPlusMovingExpenses,
    NoReoccupancyOfferAfterOmiFailureToComply,
    NotApplicableNoOmiFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WillfulnessStatus {
    WillfulOrMaliciousViolation3xActualDamagesApplies,
    NotWillfulSimpleActualDamagesOnly,
    NotApplicableNoViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaSb567Mode {
    NotApplicablePropertyOutsideCalifornia,
    NotApplicableTerminationNoticeBeforeApril1_2024PreSb567Effective,
    NotApplicableUnitNotCoveredByAb1482TpaJustCauseRegime,
    NotApplicableAtFaultCauseGroundOutsideSb567NoFaultScope,
    NotApplicableNoCauseAssertedRedirectToAb1482BaselineViolation,
    CompliantOwnerOrFamilyMemberMoveIn90DayAnd12MonthOccupancySatisfied,
    CompliantSubstantialRemodelWithPermitAnd30DayVacateAndCompleteNotice,
    CompliantOtherNoFaultCauseUnderAb1482BaselineProvisions,
    CompliantTenantOfferedReoccupancyAtSameRentAfterOmiFailure,
    ViolationOwnerMoveInFailureToMoveInWithin90DaysOfVacate,
    ViolationOwnerMoveInFailureToOccupyFor12ContinuousMonths,
    ViolationOwnerMoveInIntendedOccupantNotPermittedFamilyMember,
    ViolationOwnerMoveInIntendedOccupantAlreadyOccupiesAnotherUnitOnProperty,
    ViolationOwnerMoveInSimilarVacantUnitAvailableMustUseThatInstead,
    ViolationSubstantialRemodelPermitOrSignedContractorAgreementNotProvided,
    ViolationSubstantialRemodelTenantVacateRequirementUnder30Days,
    ViolationSubstantialRemodelWrittenNoticeMissingRequiredContent,
    ViolationFailureToOfferReoccupancyAtOriginalRentAfterOmiNonCompliance,
    ViolationWillfulOrMalicious3xActualDamagesAndPunitiveDamagesApply,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub termination_notice_date_status: TerminationNoticeDateStatus,
    pub unit_coverage: UnitCoverage,
    pub no_fault_cause_asserted: NoFaultCauseAsserted,
    pub omi_actual_occupancy_status: OmiActualOccupancyStatus,
    pub omi_intended_occupant_status: OmiIntendedOccupantStatus,
    pub substantial_remodel_compliance_status: SubstantialRemodelComplianceStatus,
    pub reoccupancy_offer_status: ReoccupancyOfferStatus,
    pub willfulness_status: WillfulnessStatus,
    pub monthly_rent_cents: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: CaSb567Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub actual_damages_due_cents: u64,
    pub treble_damages_due_cents: u64,
    pub mandatory_reoccupancy_at_original_rent_required: bool,
}

pub type RentalCaliforniaSb567NoFaultEvictionAmendmentsInput = Input;
pub type RentalCaliforniaSb567NoFaultEvictionAmendmentsOutput = Output;
pub type RentalCaliforniaSb567NoFaultEvictionAmendmentsResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "California SB 567 (2023-2024 Regular Session) — Senator Maria Elena Durazo (D-Los Angeles); signed by Governor Gavin Newsom on September 30, 2023; effective April 1, 2024; amends California Civil Code § 1946.2 (Tenant Protection Act of 2019 enacted by AB 1482)".to_string(),
        "Underlying AB 1482 / Tenant Protection Act framework — 5 % + CPI annual rent cap and 15 just causes for termination remain in place; SB 567 tightens the no-fault subset (owner / family-member move-in and substantial remodel)".to_string(),
        "Civ. Code § 1946.2(b)(2)(A)(ii) Owner Move-In Requirements — owner or eligible family member must (1) MOVE IN within 90 DAYS of tenant vacating AND (2) OCCUPY as PRIMARY RESIDENCE for AT LEAST 12 CONSECUTIVE MONTHS".to_string(),
        "Permitted Family Members (six enumerated categories) — spouse, domestic partner, children, grandchildren, parents, grandparents".to_string(),
        "OMI Cause Unavailability — cause UNAVAILABLE if intended occupant ALREADY RENTS a unit on the property OR if SIMILAR VACANT UNIT EXISTS on the property".to_string(),
        "OMI Failure-to-Comply Remedy — owner MUST OFFER unit to vacated tenant at SAME RENT AND LEASE TERMS AND REIMBURSE REASONABLE MOVING EXPENSES if 90-day move-in or 12-month occupancy requirement not met".to_string(),
        "Substantial Remodel Requirements — work must require tenant to VACATE for AT LEAST 30 CONSECUTIVE DAYS and not reasonably accomplishable in safe manner allowing tenant to remain; written notice must include (a) description of remodel + (b) expected duration + (c) required permit(s) OR signed contractor agreement detailing work".to_string(),
        "Substantial Remodel Tenant Re-Claim Right — if substantial remodel not completed, tenant may RECLAIM unit at PREVIOUS RENTAL RATE".to_string(),
        "Civil Penalties — owner violating TPA liable for (1) ACTUAL DAMAGES + (2) UP TO 3 TIMES actual damages for WILLFUL OR MALICIOUS conduct + (3) REASONABLE ATTORNEY'S FEES AND COSTS (court discretion) + (4) PUNITIVE DAMAGES".to_string(),
        "Notice Voidability — written termination notice rendered VOID by owner's failure to comply with any just-cause provisions".to_string(),
        "Effective Date — SB 567 provisions apply to termination notices served on or after April 1, 2024".to_string(),
        "California Legislative Information SB 567 — primary bill text and history".to_string(),
        "CalMatters Digital Democracy — SB 567 Termination of tenancy: no-fault just causes".to_string(),
        "Public Advocates — Renters' Rights under the Tenant Protection Act (2024)".to_string(),
        "Apartment Association of Greater Los Angeles — Senate Bill 567 Signed by Governor: New No-Fault Eviction Rules Effective April 1st".to_string(),
        "Kahana Feld — Navigating the New Landscape: How AB 12 and SB 567 Impact Landlords and Tenants in California".to_string(),
        "Southern California Rental Housing Association — SB 567 Guidance".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideCalifornia {
        return Output {
            mode: CaSb567Mode::NotApplicablePropertyOutsideCalifornia,
            statutory_basis: "Property outside California; SB 567 / AB 1482 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside California; California SB 567 (2023) and underlying AB 1482 Tenant Protection Act framework inapplicable.".to_string(),
            citations,
            actual_damages_due_cents: 0,
            treble_damages_due_cents: 0,
            mandatory_reoccupancy_at_original_rent_required: false,
        };
    }

    if input.termination_notice_date_status
        == TerminationNoticeDateStatus::NoticeServedBeforeApril1_2024PreSb567Effective
    {
        return Output {
            mode: CaSb567Mode::NotApplicableTerminationNoticeBeforeApril1_2024PreSb567Effective,
            statutory_basis: "California SB 567 effective date April 1, 2024 — applies to termination notices served on or after that date".to_string(),
            notes: "NOT APPLICABLE: termination notice served before April 1, 2024 effective date of SB 567; pre-SB-567 AB 1482 baseline applies without SB 567 OMI / substantial-remodel tightening.".to_string(),
            citations,
            actual_damages_due_cents: 0,
            treble_damages_due_cents: 0,
            mandatory_reoccupancy_at_original_rent_required: false,
        };
    }

    if input.unit_coverage == UnitCoverage::ExemptFromAb1482Tpa {
        return Output {
            mode: CaSb567Mode::NotApplicableUnitNotCoveredByAb1482TpaJustCauseRegime,
            statutory_basis: "California Civ. Code § 1946.2 — unit exempt from AB 1482 TPA just-cause regime".to_string(),
            notes: "NOT APPLICABLE: unit is exempt from AB 1482 Tenant Protection Act just-cause regime (e.g., single-family home not owned by REIT/LLC/corporation; owner-occupied property with ≤ 2 units; certificate of occupancy issued within last 15 years); SB 567 amendments to no-fault grounds also do not apply.".to_string(),
            citations,
            actual_damages_due_cents: 0,
            treble_damages_due_cents: 0,
            mandatory_reoccupancy_at_original_rent_required: false,
        };
    }

    match input.no_fault_cause_asserted {
        NoFaultCauseAsserted::NotANoFaultCauseAtFaultGround => {
            return Output {
                mode: CaSb567Mode::NotApplicableAtFaultCauseGroundOutsideSb567NoFaultScope,
                statutory_basis: "California Civ. Code § 1946.2 — SB 567 amendments apply only to no-fault just causes (OMI + substantial remodel)".to_string(),
                notes: "NOT APPLICABLE: cause asserted is an at-fault ground (e.g., non-payment of rent, lease violation, criminal activity); SB 567 amendments target the no-fault just-cause subset only.".to_string(),
                citations,
                actual_damages_due_cents: 0,
                treble_damages_due_cents: 0,
                mandatory_reoccupancy_at_original_rent_required: false,
            };
        }
        NoFaultCauseAsserted::NoCauseAsserted => {
            return Output {
                mode: CaSb567Mode::NotApplicableNoCauseAssertedRedirectToAb1482BaselineViolation,
                statutory_basis: "California Civ. Code § 1946.2 — AB 1482 just-cause requirement; SB 567 amendments add to but do not displace the baseline rule that termination requires a qualifying cause".to_string(),
                notes: "NOT APPLICABLE TO SB 567: no cause asserted; baseline AB 1482 just-cause violation applies (covered by rental_just_cause_eviction module); SB 567 amendments redirect to the AB 1482 baseline.".to_string(),
                citations,
                actual_damages_due_cents: 0,
                treble_damages_due_cents: 0,
                mandatory_reoccupancy_at_original_rent_required: false,
            };
        }
        NoFaultCauseAsserted::OtherNoFaultCauseUnderAb1482 => {
            return Output {
                mode: CaSb567Mode::CompliantOtherNoFaultCauseUnderAb1482BaselineProvisions,
                statutory_basis: "California Civ. Code § 1946.2 — other no-fault cause under AB 1482 baseline (e.g., compliance with government order to vacate); not modified by SB 567".to_string(),
                notes: "COMPLIANT WITH SB 567 SCOPE: other AB 1482 no-fault cause asserted (not OMI or substantial remodel); SB 567 tightening does not apply; baseline AB 1482 no-fault procedural requirements continue (relocation assistance under Civ. Code § 1946.2(d)).".to_string(),
                citations,
                actual_damages_due_cents: 0,
                treble_damages_due_cents: 0,
                mandatory_reoccupancy_at_original_rent_required: false,
            };
        }
        NoFaultCauseAsserted::OwnerOrFamilyMemberMoveIn => {}
        NoFaultCauseAsserted::SubstantialRemodelOrDemolition => {}
    }

    if input.no_fault_cause_asserted == NoFaultCauseAsserted::OwnerOrFamilyMemberMoveIn {
        match input.omi_intended_occupant_status {
            OmiIntendedOccupantStatus::NotPermittedFamilyMemberOutsideEnumeratedSixCategories => {
                return Output {
                    mode: CaSb567Mode::ViolationOwnerMoveInIntendedOccupantNotPermittedFamilyMember,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(A)(ii) as amended by SB 567 — permitted family members limited to spouse, domestic partner, children, grandchildren, parents, grandparents".to_string(),
                    notes: "VIOLATION: intended occupant is not within the six enumerated permitted-family-member categories (spouse, domestic partner, children, grandchildren, parents, grandparents); SB 567 narrows OMI to this closed list; termination notice is VOID.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            OmiIntendedOccupantStatus::IntendedOccupantAlreadyOccupiesAnotherUnitOnProperty => {
                return Output {
                    mode: CaSb567Mode::ViolationOwnerMoveInIntendedOccupantAlreadyOccupiesAnotherUnitOnProperty,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(A)(ii) as amended by SB 567 — OMI cause unavailable if intended occupant already rents another unit on the property".to_string(),
                    notes: "VIOLATION: intended occupant already rents (or otherwise occupies) another unit on the property; SB 567 makes OMI cause UNAVAILABLE in this situation; termination notice is VOID.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            OmiIntendedOccupantStatus::SimilarVacantUnitAvailableOnPropertyMustUseThatInstead => {
                return Output {
                    mode: CaSb567Mode::ViolationOwnerMoveInSimilarVacantUnitAvailableMustUseThatInstead,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(A)(ii) as amended by SB 567 — OMI cause unavailable if similar vacant unit exists on property; intended occupant must use vacant unit instead".to_string(),
                    notes: "VIOLATION: similar vacant unit exists on the property; SB 567 makes OMI cause UNAVAILABLE — owner must use the vacant unit instead of evicting the tenant; termination notice is VOID.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            OmiIntendedOccupantStatus::NotApplicableNonOmiCause => {}
            OmiIntendedOccupantStatus::OwnerOrPermittedFamilyMemberPerStatute => {}
        }

        match input.omi_actual_occupancy_status {
            OmiActualOccupancyStatus::OwnerDidNotMoveInWithin90DaysOfTenantVacate => {
                let actual_damages = input.monthly_rent_cents.saturating_mul(3);
                let treble = if input.willfulness_status
                    == WillfulnessStatus::WillfulOrMaliciousViolation3xActualDamagesApplies
                {
                    actual_damages.saturating_mul(u64::from(CA_SB_567_WILLFUL_VIOLATION_DAMAGES_MULTIPLIER))
                } else {
                    0
                };
                let needs_reoccupancy =
                    input.reoccupancy_offer_status != ReoccupancyOfferStatus::TenantOfferedReoccupancyAtSameRentAndLeaseTermsPlusMovingExpenses;
                let mode = if needs_reoccupancy {
                    CaSb567Mode::ViolationFailureToOfferReoccupancyAtOriginalRentAfterOmiNonCompliance
                } else {
                    CaSb567Mode::ViolationOwnerMoveInFailureToMoveInWithin90DaysOfVacate
                };
                return Output {
                    mode,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(A)(ii) as amended by SB 567 — owner / family member must move in within 90 days of tenant vacate".to_string(),
                    notes: format!(
                        "VIOLATION: owner / family member did NOT move in within 90 days of tenant vacating; SB 567 OMI requirement breached; owner must OFFER tenant reoccupancy at same rent and lease terms plus reasonable moving expenses; willfulness = {:?}; estimated actual damages = {} cents (3 × monthly rent baseline); treble damages = {} cents.",
                        input.willfulness_status, actual_damages, treble
                    ),
                    citations,
                    actual_damages_due_cents: actual_damages,
                    treble_damages_due_cents: treble,
                    mandatory_reoccupancy_at_original_rent_required: needs_reoccupancy,
                };
            }
            OmiActualOccupancyStatus::OwnerMovedInButDidNotOccupyFor12ContinuousMonths => {
                let actual_damages = input.monthly_rent_cents.saturating_mul(3);
                let treble = if input.willfulness_status
                    == WillfulnessStatus::WillfulOrMaliciousViolation3xActualDamagesApplies
                {
                    actual_damages.saturating_mul(u64::from(CA_SB_567_WILLFUL_VIOLATION_DAMAGES_MULTIPLIER))
                } else {
                    0
                };
                let needs_reoccupancy =
                    input.reoccupancy_offer_status != ReoccupancyOfferStatus::TenantOfferedReoccupancyAtSameRentAndLeaseTermsPlusMovingExpenses;
                let mode = if needs_reoccupancy {
                    CaSb567Mode::ViolationFailureToOfferReoccupancyAtOriginalRentAfterOmiNonCompliance
                } else {
                    CaSb567Mode::ViolationOwnerMoveInFailureToOccupyFor12ContinuousMonths
                };
                return Output {
                    mode,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(A)(ii) as amended by SB 567 — owner / family member must occupy as primary residence for at least 12 continuous months".to_string(),
                    notes: format!(
                        "VIOLATION: owner / family member moved in but did NOT occupy property for 12 continuous months; SB 567 OMI 12-month residency requirement breached; owner must OFFER tenant reoccupancy at same rent and lease terms plus reasonable moving expenses; willfulness = {:?}; estimated actual damages = {} cents (3 × monthly rent baseline); treble damages = {} cents.",
                        input.willfulness_status, actual_damages, treble
                    ),
                    citations,
                    actual_damages_due_cents: actual_damages,
                    treble_damages_due_cents: treble,
                    mandatory_reoccupancy_at_original_rent_required: needs_reoccupancy,
                };
            }
            OmiActualOccupancyStatus::OwnerMovedInWithin90DaysAndOccupiedFor12ContinuousMonths => {
                if input.willfulness_status
                    == WillfulnessStatus::WillfulOrMaliciousViolation3xActualDamagesApplies
                {
                    let actual_damages = input.monthly_rent_cents.saturating_mul(3);
                    let treble = actual_damages.saturating_mul(u64::from(CA_SB_567_WILLFUL_VIOLATION_DAMAGES_MULTIPLIER));
                    return Output {
                        mode: CaSb567Mode::ViolationWillfulOrMalicious3xActualDamagesAndPunitiveDamagesApply,
                        statutory_basis: "Civ. Code § 1946.2(h)(3) as amended by SB 567 — willful or malicious TPA violation triggers up to 3x actual damages + attorney fees + punitive damages".to_string(),
                        notes: format!(
                            "VIOLATION: willful or malicious TPA violation (even though OMI 90-day / 12-month thresholds met); SB 567 imposes UP TO 3x actual damages = {} cents + attorney fees + punitive damages; statutory treble multiplier = {}.",
                            treble, CA_SB_567_WILLFUL_VIOLATION_DAMAGES_MULTIPLIER
                        ),
                        citations,
                        actual_damages_due_cents: actual_damages,
                        treble_damages_due_cents: treble,
                        mandatory_reoccupancy_at_original_rent_required: false,
                    };
                }
                return Output {
                    mode: CaSb567Mode::CompliantOwnerOrFamilyMemberMoveIn90DayAnd12MonthOccupancySatisfied,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(A)(ii) as amended by SB 567 — OMI 90-day move-in and 12-month continuous occupancy requirements satisfied".to_string(),
                    notes: "COMPLIANT: owner / family member moved in within 90 days of tenant vacating AND occupied property as primary residence for at least 12 continuous months; SB 567 OMI requirements satisfied.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            OmiActualOccupancyStatus::NotApplicableNonOmiCause => {}
        }

        if input.reoccupancy_offer_status
            == ReoccupancyOfferStatus::TenantOfferedReoccupancyAtSameRentAndLeaseTermsPlusMovingExpenses
        {
            return Output {
                mode: CaSb567Mode::CompliantTenantOfferedReoccupancyAtSameRentAfterOmiFailure,
                statutory_basis: "Civ. Code § 1946.2(b)(2)(A)(ii) as amended by SB 567 — failure-to-comply remedy: offer reoccupancy at same rent + reimburse moving expenses".to_string(),
                notes: "COMPLIANT REMEDY: although OMI requirements were not fully met, owner offered tenant reoccupancy at same rent and lease terms plus reasonable moving expenses; SB 567 failure-to-comply remedy satisfied; no further civil damages exposure for OMI failure.".to_string(),
                citations,
                actual_damages_due_cents: 0,
                treble_damages_due_cents: 0,
                mandatory_reoccupancy_at_original_rent_required: false,
            };
        }
    }

    if input.no_fault_cause_asserted == NoFaultCauseAsserted::SubstantialRemodelOrDemolition {
        match input.substantial_remodel_compliance_status {
            SubstantialRemodelComplianceStatus::PermitOrSignedContractorAgreementNotProvidedWithNotice => {
                return Output {
                    mode: CaSb567Mode::ViolationSubstantialRemodelPermitOrSignedContractorAgreementNotProvided,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(D) as amended by SB 567 — written notice must include permit OR signed contractor agreement detailing work".to_string(),
                    notes: "VIOLATION: written substantial-remodel termination notice did NOT include the required permit OR signed contractor agreement detailing the work; SB 567 documentation requirement breached; termination notice is VOID.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            SubstantialRemodelComplianceStatus::TenantVacateRequirementUnder30ConsecutiveDays => {
                return Output {
                    mode: CaSb567Mode::ViolationSubstantialRemodelTenantVacateRequirementUnder30Days,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(D) as amended by SB 567 — substantial remodel must require tenant to vacate for at least 30 consecutive days".to_string(),
                    notes: "VIOLATION: substantial remodel does NOT require tenant to vacate for at least 30 consecutive days; SB 567 threshold not met; remodel could reasonably proceed with tenant in place; termination notice is VOID.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            SubstantialRemodelComplianceStatus::WrittenNoticeMissingRequiredContent => {
                return Output {
                    mode: CaSb567Mode::ViolationSubstantialRemodelWrittenNoticeMissingRequiredContent,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(D) as amended by SB 567 — written notice must include description of remodel + expected duration + permits OR signed contractor agreement".to_string(),
                    notes: "VIOLATION: written substantial-remodel termination notice is missing required content (description of remodel and/or expected duration); SB 567 notice-content requirement breached; termination notice is VOID.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            SubstantialRemodelComplianceStatus::PermitObtainedAnd30DayVacateRequiredAndNoticeContentComplete => {
                return Output {
                    mode: CaSb567Mode::CompliantSubstantialRemodelWithPermitAnd30DayVacateAndCompleteNotice,
                    statutory_basis: "Civ. Code § 1946.2(b)(2)(D) as amended by SB 567 — substantial remodel with permit + 30-day vacate + complete written notice".to_string(),
                    notes: "COMPLIANT: substantial remodel termination notice includes required permits or signed contractor agreement, requires tenant to vacate for at least 30 consecutive days, and contains complete required content; SB 567 substantial-remodel requirements satisfied.".to_string(),
                    citations,
                    actual_damages_due_cents: 0,
                    treble_damages_due_cents: 0,
                    mandatory_reoccupancy_at_original_rent_required: false,
                };
            }
            SubstantialRemodelComplianceStatus::NotApplicableNonRemodelCause => {}
        }
    }

    Output {
        mode: CaSb567Mode::NotApplicableAtFaultCauseGroundOutsideSb567NoFaultScope,
        statutory_basis: "Civ. Code § 1946.2 — input combination did not match any SB 567 enforcement pathway".to_string(),
        notes: "NOT APPLICABLE: SB 567 enforcement pathway not triggered by input combination; fall-through indicates input fields inconsistent with the asserted no-fault cause.".to_string(),
        citations,
        actual_damages_due_cents: 0,
        treble_damages_due_cents: 0,
        mandatory_reoccupancy_at_original_rent_required: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_omi_compliant_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinCalifornia,
            termination_notice_date_status:
                TerminationNoticeDateStatus::NoticeServedAtOrAfterApril1_2024PostSb567Effective,
            unit_coverage: UnitCoverage::CoveredByAb1482Tpa,
            no_fault_cause_asserted: NoFaultCauseAsserted::OwnerOrFamilyMemberMoveIn,
            omi_actual_occupancy_status:
                OmiActualOccupancyStatus::OwnerMovedInWithin90DaysAndOccupiedFor12ContinuousMonths,
            omi_intended_occupant_status:
                OmiIntendedOccupantStatus::OwnerOrPermittedFamilyMemberPerStatute,
            substantial_remodel_compliance_status:
                SubstantialRemodelComplianceStatus::NotApplicableNonRemodelCause,
            reoccupancy_offer_status: ReoccupancyOfferStatus::NotApplicableNoOmiFailure,
            willfulness_status: WillfulnessStatus::NotApplicableNoViolation,
            monthly_rent_cents: 300_000,
        }
    }

    fn baseline_remodel_compliant_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinCalifornia,
            termination_notice_date_status:
                TerminationNoticeDateStatus::NoticeServedAtOrAfterApril1_2024PostSb567Effective,
            unit_coverage: UnitCoverage::CoveredByAb1482Tpa,
            no_fault_cause_asserted: NoFaultCauseAsserted::SubstantialRemodelOrDemolition,
            omi_actual_occupancy_status: OmiActualOccupancyStatus::NotApplicableNonOmiCause,
            omi_intended_occupant_status: OmiIntendedOccupantStatus::NotApplicableNonOmiCause,
            substantial_remodel_compliance_status:
                SubstantialRemodelComplianceStatus::PermitObtainedAnd30DayVacateRequiredAndNoticeContentComplete,
            reoccupancy_offer_status: ReoccupancyOfferStatus::NotApplicableNoOmiFailure,
            willfulness_status: WillfulnessStatus::NotApplicableNoViolation,
            monthly_rent_cents: 300_000,
        }
    }

    #[test]
    fn property_outside_california_not_applicable() {
        let mut input = baseline_omi_compliant_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideCalifornia;
        let output = check(&input);
        assert_eq!(output.mode, CaSb567Mode::NotApplicablePropertyOutsideCalifornia);
    }

    #[test]
    fn notice_before_april_1_2024_pre_sb567_not_applicable() {
        let mut input = baseline_omi_compliant_input();
        input.termination_notice_date_status =
            TerminationNoticeDateStatus::NoticeServedBeforeApril1_2024PreSb567Effective;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::NotApplicableTerminationNoticeBeforeApril1_2024PreSb567Effective
        );
    }

    #[test]
    fn unit_exempt_from_ab_1482_not_applicable() {
        let mut input = baseline_omi_compliant_input();
        input.unit_coverage = UnitCoverage::ExemptFromAb1482Tpa;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::NotApplicableUnitNotCoveredByAb1482TpaJustCauseRegime
        );
    }

    #[test]
    fn at_fault_cause_outside_sb567_scope_not_applicable() {
        let mut input = baseline_omi_compliant_input();
        input.no_fault_cause_asserted = NoFaultCauseAsserted::NotANoFaultCauseAtFaultGround;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::NotApplicableAtFaultCauseGroundOutsideSb567NoFaultScope
        );
    }

    #[test]
    fn no_cause_asserted_redirects_to_ab1482_baseline() {
        let mut input = baseline_omi_compliant_input();
        input.no_fault_cause_asserted = NoFaultCauseAsserted::NoCauseAsserted;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::NotApplicableNoCauseAssertedRedirectToAb1482BaselineViolation
        );
    }

    #[test]
    fn other_no_fault_cause_under_ab1482_baseline_compliant() {
        let mut input = baseline_omi_compliant_input();
        input.no_fault_cause_asserted = NoFaultCauseAsserted::OtherNoFaultCauseUnderAb1482;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::CompliantOtherNoFaultCauseUnderAb1482BaselineProvisions
        );
    }

    #[test]
    fn omi_90_day_and_12_month_compliant() {
        let output = check(&baseline_omi_compliant_input());
        assert_eq!(
            output.mode,
            CaSb567Mode::CompliantOwnerOrFamilyMemberMoveIn90DayAnd12MonthOccupancySatisfied
        );
        assert_eq!(output.actual_damages_due_cents, 0);
        assert_eq!(output.treble_damages_due_cents, 0);
    }

    #[test]
    fn omi_intended_occupant_not_permitted_family_member_violation() {
        let mut input = baseline_omi_compliant_input();
        input.omi_intended_occupant_status =
            OmiIntendedOccupantStatus::NotPermittedFamilyMemberOutsideEnumeratedSixCategories;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationOwnerMoveInIntendedOccupantNotPermittedFamilyMember
        );
    }

    #[test]
    fn omi_intended_occupant_already_occupies_another_unit_violation() {
        let mut input = baseline_omi_compliant_input();
        input.omi_intended_occupant_status =
            OmiIntendedOccupantStatus::IntendedOccupantAlreadyOccupiesAnotherUnitOnProperty;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationOwnerMoveInIntendedOccupantAlreadyOccupiesAnotherUnitOnProperty
        );
    }

    #[test]
    fn omi_similar_vacant_unit_available_violation() {
        let mut input = baseline_omi_compliant_input();
        input.omi_intended_occupant_status =
            OmiIntendedOccupantStatus::SimilarVacantUnitAvailableOnPropertyMustUseThatInstead;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationOwnerMoveInSimilarVacantUnitAvailableMustUseThatInstead
        );
    }

    #[test]
    fn omi_did_not_move_in_within_90_days_violation() {
        let mut input = baseline_omi_compliant_input();
        input.omi_actual_occupancy_status =
            OmiActualOccupancyStatus::OwnerDidNotMoveInWithin90DaysOfTenantVacate;
        input.reoccupancy_offer_status = ReoccupancyOfferStatus::NoReoccupancyOfferAfterOmiFailureToComply;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationFailureToOfferReoccupancyAtOriginalRentAfterOmiNonCompliance
        );
        // 3 × 300_000 = 900_000 baseline; non-willful → treble = 0
        assert_eq!(output.actual_damages_due_cents, 900_000);
        assert_eq!(output.treble_damages_due_cents, 0);
        assert!(output.mandatory_reoccupancy_at_original_rent_required);
    }

    #[test]
    fn omi_did_not_occupy_12_months_violation_with_willful_treble_damages() {
        let mut input = baseline_omi_compliant_input();
        input.omi_actual_occupancy_status =
            OmiActualOccupancyStatus::OwnerMovedInButDidNotOccupyFor12ContinuousMonths;
        input.reoccupancy_offer_status = ReoccupancyOfferStatus::NoReoccupancyOfferAfterOmiFailureToComply;
        input.willfulness_status =
            WillfulnessStatus::WillfulOrMaliciousViolation3xActualDamagesApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationFailureToOfferReoccupancyAtOriginalRentAfterOmiNonCompliance
        );
        // 3 × 300_000 = 900_000 baseline; willful × 3 = 2_700_000
        assert_eq!(output.actual_damages_due_cents, 900_000);
        assert_eq!(output.treble_damages_due_cents, 2_700_000);
        assert!(output.mandatory_reoccupancy_at_original_rent_required);
    }

    #[test]
    fn omi_failure_with_reoccupancy_offer_remedy_compliant() {
        let mut input = baseline_omi_compliant_input();
        input.omi_actual_occupancy_status =
            OmiActualOccupancyStatus::OwnerDidNotMoveInWithin90DaysOfTenantVacate;
        input.reoccupancy_offer_status =
            ReoccupancyOfferStatus::TenantOfferedReoccupancyAtSameRentAndLeaseTermsPlusMovingExpenses;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationOwnerMoveInFailureToMoveInWithin90DaysOfVacate
        );
        // 3 × 300_000 = 900_000 baseline (still computed as the OMI breach occurred)
        assert_eq!(output.actual_damages_due_cents, 900_000);
        assert!(!output.mandatory_reoccupancy_at_original_rent_required);
    }

    #[test]
    fn omi_compliant_but_willful_violation_triggers_treble_damages() {
        let mut input = baseline_omi_compliant_input();
        input.willfulness_status =
            WillfulnessStatus::WillfulOrMaliciousViolation3xActualDamagesApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationWillfulOrMalicious3xActualDamagesAndPunitiveDamagesApply
        );
        // 3 × 300_000 = 900_000 baseline; treble = 900_000 × 3 = 2_700_000
        assert_eq!(output.actual_damages_due_cents, 900_000);
        assert_eq!(output.treble_damages_due_cents, 2_700_000);
    }

    #[test]
    fn substantial_remodel_compliant() {
        let output = check(&baseline_remodel_compliant_input());
        assert_eq!(
            output.mode,
            CaSb567Mode::CompliantSubstantialRemodelWithPermitAnd30DayVacateAndCompleteNotice
        );
    }

    #[test]
    fn substantial_remodel_permit_not_provided_violation() {
        let mut input = baseline_remodel_compliant_input();
        input.substantial_remodel_compliance_status =
            SubstantialRemodelComplianceStatus::PermitOrSignedContractorAgreementNotProvidedWithNotice;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationSubstantialRemodelPermitOrSignedContractorAgreementNotProvided
        );
    }

    #[test]
    fn substantial_remodel_vacate_under_30_days_violation() {
        let mut input = baseline_remodel_compliant_input();
        input.substantial_remodel_compliance_status =
            SubstantialRemodelComplianceStatus::TenantVacateRequirementUnder30ConsecutiveDays;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationSubstantialRemodelTenantVacateRequirementUnder30Days
        );
    }

    #[test]
    fn substantial_remodel_notice_missing_content_violation() {
        let mut input = baseline_remodel_compliant_input();
        input.substantial_remodel_compliance_status =
            SubstantialRemodelComplianceStatus::WrittenNoticeMissingRequiredContent;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CaSb567Mode::ViolationSubstantialRemodelWrittenNoticeMissingRequiredContent
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(CA_SB_567_ENACTMENT_YEAR, 2023);
        assert_eq!(CA_SB_567_ENACTMENT_MONTH, 9);
        assert_eq!(CA_SB_567_ENACTMENT_DAY, 30);
        assert_eq!(CA_SB_567_EFFECTIVE_DATE_YEAR, 2024);
        assert_eq!(CA_SB_567_EFFECTIVE_DATE_MONTH, 4);
        assert_eq!(CA_SB_567_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(CA_SB_567_OMI_MOVE_IN_DEADLINE_DAYS, 90);
        assert_eq!(CA_SB_567_OMI_CONTINUOUS_OCCUPANCY_MONTHS, 12);
        assert_eq!(CA_SB_567_SUBSTANTIAL_REMODEL_TENANT_VACATE_DAYS, 30);
        assert_eq!(CA_SB_567_PERMITTED_FAMILY_MEMBER_CATEGORIES_COUNT, 6);
        assert_eq!(CA_SB_567_WILLFUL_VIOLATION_DAMAGES_MULTIPLIER, 3);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_omi_compliant_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("California SB 567"));
        assert!(joined.contains("September 30, 2023"));
        assert!(joined.contains("April 1, 2024"));
        assert!(joined.contains("Maria Elena Durazo"));
        assert!(joined.contains("Civil Code § 1946.2"));
        assert!(joined.contains("AB 1482"));
        assert!(joined.contains("90 DAYS"));
        assert!(joined.contains("12 CONSECUTIVE MONTHS"));
        assert!(joined.contains("30 CONSECUTIVE DAYS"));
        assert!(joined.contains("3 TIMES"));
    }

    #[test]
    fn damages_saturating_overflow_defense() {
        let mut input = baseline_omi_compliant_input();
        input.monthly_rent_cents = u64::MAX;
        input.omi_actual_occupancy_status =
            OmiActualOccupancyStatus::OwnerDidNotMoveInWithin90DaysOfTenantVacate;
        input.willfulness_status =
            WillfulnessStatus::WillfulOrMaliciousViolation3xActualDamagesApplies;
        let output = check(&input);
        // Saturating multiplication clamps without panic
        assert_eq!(output.actual_damages_due_cents, u64::MAX);
        assert_eq!(output.treble_damages_due_cents, u64::MAX);
    }
}
