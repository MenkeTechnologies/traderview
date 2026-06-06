//! New Jersey Anti-Eviction Act (N.J.S.A. 2A:18-61.1)
//! Compliance Module — OLDEST statewide just-cause eviction
//! regime in the United States (1974).
//!
//! Pure-compute check for landlord compliance with the New
//! Jersey Anti-Eviction Act, codified at N.J.S.A. 2A:18-61.1,
//! originally enacted as **P.L. 1974, c. 49** in response to
//! the 1970s New Jersey housing shortage. The Act enumerates
//! **18 statutory grounds** for eviction, requires a tiered
//! Notice to Cease + Notice to Quit procedure for most cause
//! categories, and contains the famous "owner-occupied 3-or-
//! fewer-units" exemption that exempts small owner-occupants
//! from the just-cause requirement entirely.
//!
//! Web research (verified 2026-06-03):
//! - **New Jersey Anti-Eviction Act** (P.L. 1974, c. 49) —
//!   originally enacted **1974**; OLDEST statewide just-cause
//!   eviction regime in the United States, predating California
//!   AB 1482 of 2019 by 45 years ([NJ DCA — N.J. Stat. § 2A:18-
//!   61.1](https://www.nj.gov/dca/codes/codreg/pdf_regs/2A_18_61.pdf);
//!   [NJ DCA — Grounds for Eviction Bulletin](https://www.nj.gov/dca/codes/publications/pdf_lti/grnds_for_evicti_bulltin.pdf)).
//! - **18 Statutory Grounds** (N.J.S.A. 2A:18-61.1(a) through
//!   (r)): (a) failure to pay rent; (b) disorderly conduct
//!   after notice to cease; (c) willful or grossly negligent
//!   destruction; (d) substantial lease violation after notice
//!   to cease; (e) continued violation of landlord's rules or
//!   lease covenants after notice to cease; (f) habitual late
//!   payment after notice to cease; (g) refusal to accept
//!   reasonable changes of lease terms at renewal; (h) owner
//!   permanently retires premises from rental business; (i)
//!   conversion to non-residential use; (j) conversion to
//!   condominium/cooperative ownership (overlaps with NJ
//!   condo/coop conversion regime — N.J.S.A. 2A:18-61.7 et
//!   seq.); (k) owner permanently moves into unit; (l) owner
//!   needs unit for parent, child, or step-child occupancy;
//!   (m) refusing to accept reasonable lease changes at
//!   tenancy end; (n) habitual nonpayment; (o) drug-related
//!   criminal activity on premises; (p) assault, threats,
//!   weapons use against landlord or tenants; (q) theft from
//!   premises; (r) other specified criminal activity.
//! - **Owner-Occupied 3-Or-Fewer Unit Exemption**: the Act does
//!   NOT apply to tenants residing in buildings or houses with
//!   **3 OR FEWER apartments where the owner LIVES in one of
//!   the apartments**. Such owner-occupants may evict at end
//!   of lease term for any reason with a month's notice ([Law
//!   Office of Robert J. Wittmann — Notice to Cease & Notice
//!   to Quit](https://rjwnjlaw.com/notice-to-cease-notice-to-quit)).
//! - **Notice to Cease**: required for grounds (b) disorderly
//!   conduct, (d) substantial lease violation, (e) rules
//!   violations, and (f) habitual late payment. Notice to
//!   Cease is a written warning instructing tenant to stop
//!   wrongful conduct; if conduct continues, landlord may then
//!   serve Notice to Quit.
//! - **No Notice Required for Nonpayment**: ground (a) failure
//!   to pay rent allows the landlord to file an eviction
//!   lawsuit IMMEDIATELY without notice to quit.
//! - **Notice to Quit Periods Vary by Ground**: typically 1
//!   month for most grounds; 3 days for certain criminal-
//!   activity grounds; 18 months for owner-occupier conversion
//!   grounds (h), (i), (k), (l); 3 years for conversion to
//!   condominium/coop ownership under (j).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const NJ_ANTI_EVICTION_ACT_ENACTMENT_YEAR: u32 = 1974;
pub const NJ_ANTI_EVICTION_ACT_PL_CHAPTER: u32 = 49;
pub const NJ_ANTI_EVICTION_OWNER_OCCUPIED_MAX_UNITS_EXEMPT: u32 = 3;
pub const NJ_ANTI_EVICTION_STATUTORY_GROUNDS_COUNT: u32 = 18;
pub const NJ_ANTI_EVICTION_NOTICE_TO_QUIT_STANDARD_MONTHS: u32 = 1;
pub const NJ_ANTI_EVICTION_OWNER_OCCUPIER_NOTICE_MONTHS: u32 = 18;
pub const NJ_ANTI_EVICTION_CONDOMINIUM_CONVERSION_NOTICE_YEARS: u32 = 3;
pub const NJ_ANTI_EVICTION_CRIMINAL_ACTIVITY_NOTICE_DAYS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionGround {
    SubsectionAFailureToPayRent,
    SubsectionBDisorderlyConductAfterNoticeToCease,
    SubsectionCWillfulOrGrosslyNegligentDestruction,
    SubsectionDSubstantialLeaseViolationAfterNoticeToCease,
    SubsectionEContinuedViolationOfLandlordRulesAfterNoticeToCease,
    SubsectionFHabitualLatePaymentAfterNoticeToCease,
    SubsectionGRefusalOfReasonableLeaseChangesAtRenewal,
    SubsectionHOwnerRetiresFromRentalBusiness,
    SubsectionIConversionToNonResidentialUse,
    SubsectionJConversionToCondominiumOrCooperative,
    SubsectionKOwnerPermanentlyMovesIntoUnit,
    SubsectionLOwnerNeedsUnitForFamilyOccupancy,
    SubsectionMRefusingReasonableLeaseChangesAtTenancyEnd,
    SubsectionNHabitualNonpayment,
    SubsectionODrugRelatedCriminalActivity,
    SubsectionPAssaultThreatsWeaponsUse,
    SubsectionQTheftFromPremises,
    SubsectionROtherSpecifiedCriminalActivity,
    NoGroundAsserted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyClassification {
    StandardRentalCoveredByAntiEvictionAct,
    OwnerOccupiedBuildingWith3OrFewerApartmentsExempt,
    PropertyOutsideNewJersey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeToCeaseStatus {
    NoticeToCeaseProperlyServed,
    NoticeToCeaseNotServed,
    NoticeToCeaseNotRequiredForThisGround,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeToQuitDuration {
    OneMonthOrMore,
    BetweenThreeDaysAndOneMonth,
    EighteenMonthsOrMore,
    ThreeYearsOrMore,
    LessThanThreeDays,
    NoNoticeRequiredForNonpayment,
    NoNoticeProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NjAntiEvictionMode {
    NotApplicablePropertyOutsideNewJersey,
    NotApplicableOwnerOccupiedBuildingWith3OrFewerApartmentsExempt,
    CompliantSubsectionANonpaymentNoNoticeRequiredEvictionProperlyFiled,
    CompliantSubsectionBDisorderlyConductWithNoticeToCeaseAnd1MonthNoticeToQuit,
    CompliantSubsectionCWillfulOrGrosslyNegligentDestructionEvictionProper,
    CompliantSubsectionDSubstantialLeaseViolationWithNoticeToCeaseAnd1MonthNoticeToQuit,
    CompliantSubsectionEContinuedRulesViolationWithNoticeToCeaseAnd1MonthNoticeToQuit,
    CompliantSubsectionFHabitualLatePaymentWithNoticeToCeaseAnd1MonthNoticeToQuit,
    CompliantSubsectionGRefusalReasonableLeaseChangesAtRenewal1MonthNotice,
    CompliantSubsectionHOwnerRetiresFromRentalBusinessWith18MonthsNotice,
    CompliantSubsectionIConversionToNonResidentialUseWith18MonthsNotice,
    CompliantSubsectionJConversionToCondoOrCoopWith3YearsNotice,
    CompliantSubsectionKOwnerPermanentlyMovesInWith18MonthsNotice,
    CompliantSubsectionLOwnerFamilyOccupancyWith18MonthsNotice,
    CompliantCriminalActivitySubsectionsOPQRWith3DayNotice,
    ViolationNoStatutoryGroundAsserted,
    ViolationNoticeToCeaseNotServedForGroundRequiringIt,
    ViolationNoticeToQuitDurationInsufficientForAssertedGround,
    ViolationOwnerOccupiedExemptionClaimedButAboveThreeUnitsThreshold,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_classification: PropertyClassification,
    pub eviction_ground: EvictionGround,
    pub notice_to_cease_status: NoticeToCeaseStatus,
    pub notice_to_quit_duration: NoticeToQuitDuration,
    pub building_total_apartments: u32,
    pub owner_occupies_one_apartment: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: NjAntiEvictionMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalNewJerseyAntiEvictionActInput = Input;
pub type RentalNewJerseyAntiEvictionActOutput = Output;
pub type RentalNewJerseyAntiEvictionActResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "New Jersey Anti-Eviction Act — P.L. 1974, c. 49; codified at N.J.S.A. 2A:18-61.1; enacted 1974; OLDEST statewide just-cause eviction regime in United States history".to_string(),
        "N.J.S.A. 2A:18-61.1 enumerates 18 statutory grounds for eviction: (a) failure to pay rent; (b) disorderly conduct after notice to cease; (c) willful or grossly negligent destruction; (d) substantial lease violation after notice to cease; (e) continued violation of landlord rules; (f) habitual late payment; (g) refusal of reasonable lease changes at renewal; (h) owner retires from rental business; (i) conversion to non-residential use; (j) conversion to condominium/cooperative; (k) owner permanently moves into unit; (l) owner needs unit for family; (m) refusing reasonable lease changes at tenancy end; (n) habitual nonpayment; (o) drug-related criminal activity; (p) assault/threats/weapons use; (q) theft from premises; (r) other specified criminal activity".to_string(),
        "Owner-Occupied 3-or-Fewer Apartments Exemption — Act does NOT apply where building has 3 or fewer apartments AND owner lives in one of them; such owner-occupants may evict at lease end for any reason with month's notice".to_string(),
        "Notice to Cease — required for grounds (b) disorderly conduct, (d) substantial lease violation, (e) rules violations, and (f) habitual late payment; written warning instructing tenant to stop wrongful conduct".to_string(),
        "No Notice Required for Nonpayment — ground (a) failure to pay rent allows landlord to file eviction lawsuit IMMEDIATELY without notice to quit".to_string(),
        "Notice to Quit Periods — typically 1 month for most grounds; 3 days for criminal-activity grounds (o)/(p)/(q)/(r); 18 months for owner-occupier conversion grounds (h)/(i)/(k)/(l); 3 years for condominium/coop conversion under (j)".to_string(),
        "N.J.S.A. 2A:18-61.7 et seq. — companion New Jersey Condominium / Cooperative Conversion Tenant Protection Act; works with subsection (j) ground".to_string(),
        "Notice to Quit and Demand for Possession — required step under N.J.S.A. 2A:18-56 prerequisite to summary dispossess action in NJ Special Civil Part".to_string(),
        "NJ DCA Codes and Standards Division — N.J.S.A. 2A:18-61.1 official publication".to_string(),
        "NJ DCA Grounds for Eviction Bulletin — comprehensive practitioner guide".to_string(),
        "Law Office of Robert J. Wittmann — Notice to Cease & Notice to Quit practitioner analysis".to_string(),
        "Legal Services of New Jersey — Tenant's Right to Court Process".to_string(),
    ];

    if input.property_classification == PropertyClassification::PropertyOutsideNewJersey {
        return Output {
            mode: NjAntiEvictionMode::NotApplicablePropertyOutsideNewJersey,
            statutory_basis: "Property outside New Jersey; N.J.S.A. 2A:18-61.1 inapplicable"
                .to_string(),
            notes: "Property outside New Jersey; New Jersey Anti-Eviction Act inapplicable."
                .to_string(),
            citations,
        };
    }

    if input.property_classification
        == PropertyClassification::OwnerOccupiedBuildingWith3OrFewerApartmentsExempt
    {
        if !input.owner_occupies_one_apartment {
            return Output {
                mode: NjAntiEvictionMode::ViolationOwnerOccupiedExemptionClaimedButAboveThreeUnitsThreshold,
                statutory_basis: "N.J.S.A. 2A:18-61.1 owner-occupied exemption requires owner to live in one of the apartments".to_string(),
                notes: "VIOLATION: owner-occupied exemption claimed but owner does not actually live in one of the apartments; exemption inapplicable; Anti-Eviction Act fully covers tenants.".to_string(),
                citations,
            };
        }
        if input.building_total_apartments > NJ_ANTI_EVICTION_OWNER_OCCUPIED_MAX_UNITS_EXEMPT {
            return Output {
                mode: NjAntiEvictionMode::ViolationOwnerOccupiedExemptionClaimedButAboveThreeUnitsThreshold,
                statutory_basis: "N.J.S.A. 2A:18-61.1 owner-occupied exemption requires 3 or fewer apartments".to_string(),
                notes: format!(
                    "VIOLATION: owner-occupied exemption claimed but building has {} apartments (exceeds 3-apartment statutory threshold); exemption inapplicable; Anti-Eviction Act fully covers tenants.",
                    input.building_total_apartments
                ),
                citations,
            };
        }
        return Output {
            mode: NjAntiEvictionMode::NotApplicableOwnerOccupiedBuildingWith3OrFewerApartmentsExempt,
            statutory_basis: "N.J.S.A. 2A:18-61.1 owner-occupied 3-or-fewer apartments exemption".to_string(),
            notes: format!(
                "NOT APPLICABLE: owner lives in building with {} apartments (≤ 3 statutory threshold); Anti-Eviction Act does not apply; landlord may evict at lease end for any reason with 1-month notice.",
                input.building_total_apartments
            ),
            citations,
        };
    }

    if input.eviction_ground == EvictionGround::NoGroundAsserted {
        return Output {
            mode: NjAntiEvictionMode::ViolationNoStatutoryGroundAsserted,
            statutory_basis: "N.J.S.A. 2A:18-61.1 — eviction requires assertion of one of 18 statutory grounds".to_string(),
            notes: "VIOLATION: eviction attempted without asserting any of the 18 statutory grounds enumerated in N.J.S.A. 2A:18-61.1; New Jersey Anti-Eviction Act prohibits no-cause evictions of covered tenants.".to_string(),
            citations,
        };
    }

    let notice_to_cease_required = matches!(
        input.eviction_ground,
        EvictionGround::SubsectionBDisorderlyConductAfterNoticeToCease
            | EvictionGround::SubsectionDSubstantialLeaseViolationAfterNoticeToCease
            | EvictionGround::SubsectionEContinuedViolationOfLandlordRulesAfterNoticeToCease
            | EvictionGround::SubsectionFHabitualLatePaymentAfterNoticeToCease
    );

    if notice_to_cease_required
        && input.notice_to_cease_status == NoticeToCeaseStatus::NoticeToCeaseNotServed
    {
        return Output {
            mode: NjAntiEvictionMode::ViolationNoticeToCeaseNotServedForGroundRequiringIt,
            statutory_basis: "N.J.S.A. 2A:18-61.1 — Notice to Cease required for asserted ground".to_string(),
            notes: format!(
                "VIOLATION: Notice to Cease required for ground {:?} but was not served; landlord must first serve written Notice to Cease instructing tenant to stop wrongful conduct before serving Notice to Quit.",
                input.eviction_ground
            ),
            citations,
        };
    }

    match input.eviction_ground {
        EvictionGround::SubsectionAFailureToPayRent => Output {
            mode: NjAntiEvictionMode::CompliantSubsectionANonpaymentNoNoticeRequiredEvictionProperlyFiled,
            statutory_basis: "N.J.S.A. 2A:18-61.1(a) — failure to pay rent; no notice required".to_string(),
            notes: "COMPLIANT: subsection (a) failure to pay rent ground asserted; no notice to quit required; landlord may file eviction lawsuit immediately.".to_string(),
            citations,
        },
        EvictionGround::SubsectionCWillfulOrGrosslyNegligentDestruction => Output {
            mode: NjAntiEvictionMode::CompliantSubsectionCWillfulOrGrosslyNegligentDestructionEvictionProper,
            statutory_basis: "N.J.S.A. 2A:18-61.1(c) — willful or grossly negligent destruction".to_string(),
            notes: "COMPLIANT: subsection (c) willful or grossly negligent destruction ground asserted.".to_string(),
            citations,
        },
        EvictionGround::SubsectionBDisorderlyConductAfterNoticeToCease => {
            require_at_least_one_month(input, NjAntiEvictionMode::CompliantSubsectionBDisorderlyConductWithNoticeToCeaseAnd1MonthNoticeToQuit, "N.J.S.A. 2A:18-61.1(b) — disorderly conduct after notice to cease; 1-month notice to quit", citations)
        }
        EvictionGround::SubsectionDSubstantialLeaseViolationAfterNoticeToCease => {
            require_at_least_one_month(input, NjAntiEvictionMode::CompliantSubsectionDSubstantialLeaseViolationWithNoticeToCeaseAnd1MonthNoticeToQuit, "N.J.S.A. 2A:18-61.1(d) — substantial lease violation after notice to cease; 1-month notice to quit", citations)
        }
        EvictionGround::SubsectionEContinuedViolationOfLandlordRulesAfterNoticeToCease => {
            require_at_least_one_month(input, NjAntiEvictionMode::CompliantSubsectionEContinuedRulesViolationWithNoticeToCeaseAnd1MonthNoticeToQuit, "N.J.S.A. 2A:18-61.1(e) — continued rules violation after notice to cease; 1-month notice to quit", citations)
        }
        EvictionGround::SubsectionFHabitualLatePaymentAfterNoticeToCease => {
            require_at_least_one_month(input, NjAntiEvictionMode::CompliantSubsectionFHabitualLatePaymentWithNoticeToCeaseAnd1MonthNoticeToQuit, "N.J.S.A. 2A:18-61.1(f) — habitual late payment after notice to cease; 1-month notice to quit", citations)
        }
        EvictionGround::SubsectionGRefusalOfReasonableLeaseChangesAtRenewal => {
            require_at_least_one_month(input, NjAntiEvictionMode::CompliantSubsectionGRefusalReasonableLeaseChangesAtRenewal1MonthNotice, "N.J.S.A. 2A:18-61.1(g) — refusal of reasonable lease changes at renewal; 1-month notice to quit", citations)
        }
        EvictionGround::SubsectionHOwnerRetiresFromRentalBusiness => {
            require_at_least_18_months(input, NjAntiEvictionMode::CompliantSubsectionHOwnerRetiresFromRentalBusinessWith18MonthsNotice, "N.J.S.A. 2A:18-61.1(h) — owner retires from rental business; 18-month notice to quit", citations)
        }
        EvictionGround::SubsectionIConversionToNonResidentialUse => {
            require_at_least_18_months(input, NjAntiEvictionMode::CompliantSubsectionIConversionToNonResidentialUseWith18MonthsNotice, "N.J.S.A. 2A:18-61.1(i) — conversion to non-residential use; 18-month notice to quit", citations)
        }
        EvictionGround::SubsectionJConversionToCondominiumOrCooperative => {
            if input.notice_to_quit_duration == NoticeToQuitDuration::ThreeYearsOrMore {
                return Output {
                    mode: NjAntiEvictionMode::CompliantSubsectionJConversionToCondoOrCoopWith3YearsNotice,
                    statutory_basis: "N.J.S.A. 2A:18-61.1(j) — conversion to condominium/cooperative; 3-year notice to quit".to_string(),
                    notes: "COMPLIANT: subsection (j) conversion to condominium or cooperative ownership with required 3-year notice to quit; works alongside N.J.S.A. 2A:18-61.7 et seq. tenant protection regime.".to_string(),
                    citations,
                };
            }
            Output {
                mode: NjAntiEvictionMode::ViolationNoticeToQuitDurationInsufficientForAssertedGround,
                statutory_basis: "N.J.S.A. 2A:18-61.1(j) — 3-year notice to quit required".to_string(),
                notes: format!(
                    "VIOLATION: subsection (j) condominium/cooperative conversion requires 3-year notice to quit; provided duration {:?} insufficient.",
                    input.notice_to_quit_duration
                ),
                citations,
            }
        }
        EvictionGround::SubsectionKOwnerPermanentlyMovesIntoUnit => {
            require_at_least_18_months(input, NjAntiEvictionMode::CompliantSubsectionKOwnerPermanentlyMovesInWith18MonthsNotice, "N.J.S.A. 2A:18-61.1(k) — owner permanently moves into unit; 18-month notice to quit", citations)
        }
        EvictionGround::SubsectionLOwnerNeedsUnitForFamilyOccupancy => {
            require_at_least_18_months(input, NjAntiEvictionMode::CompliantSubsectionLOwnerFamilyOccupancyWith18MonthsNotice, "N.J.S.A. 2A:18-61.1(l) — owner needs unit for parent, child, or step-child; 18-month notice to quit", citations)
        }
        EvictionGround::SubsectionMRefusingReasonableLeaseChangesAtTenancyEnd
        | EvictionGround::SubsectionNHabitualNonpayment => {
            require_at_least_one_month(input, NjAntiEvictionMode::CompliantSubsectionGRefusalReasonableLeaseChangesAtRenewal1MonthNotice, "N.J.S.A. 2A:18-61.1(m) or (n) — 1-month notice to quit", citations)
        }
        EvictionGround::SubsectionODrugRelatedCriminalActivity
        | EvictionGround::SubsectionPAssaultThreatsWeaponsUse
        | EvictionGround::SubsectionQTheftFromPremises
        | EvictionGround::SubsectionROtherSpecifiedCriminalActivity => {
            if matches!(
                input.notice_to_quit_duration,
                NoticeToQuitDuration::BetweenThreeDaysAndOneMonth
                    | NoticeToQuitDuration::OneMonthOrMore
                    | NoticeToQuitDuration::EighteenMonthsOrMore
                    | NoticeToQuitDuration::ThreeYearsOrMore
            ) {
                return Output {
                    mode: NjAntiEvictionMode::CompliantCriminalActivitySubsectionsOPQRWith3DayNotice,
                    statutory_basis: format!("N.J.S.A. 2A:18-61.1 {:?} — 3-day notice to quit for criminal activity", input.eviction_ground),
                    notes: format!(
                        "COMPLIANT: criminal activity ground {:?} asserted with at least 3-day notice to quit.",
                        input.eviction_ground
                    ),
                    citations,
                };
            }
            Output {
                mode: NjAntiEvictionMode::ViolationNoticeToQuitDurationInsufficientForAssertedGround,
                statutory_basis: "N.J.S.A. 2A:18-61.1 criminal activity ground — 3-day notice required".to_string(),
                notes: format!(
                    "VIOLATION: criminal activity ground {:?} requires at least 3-day notice to quit; provided duration {:?} insufficient.",
                    input.eviction_ground, input.notice_to_quit_duration
                ),
                citations,
            }
        }
        EvictionGround::NoGroundAsserted => unreachable!(),
    }
}

fn require_at_least_one_month(
    input: &Input,
    compliant_mode: NjAntiEvictionMode,
    statutory_basis: &str,
    citations: Vec<String>,
) -> Output {
    if matches!(
        input.notice_to_quit_duration,
        NoticeToQuitDuration::OneMonthOrMore
            | NoticeToQuitDuration::EighteenMonthsOrMore
            | NoticeToQuitDuration::ThreeYearsOrMore
    ) {
        return Output {
            mode: compliant_mode,
            statutory_basis: statutory_basis.to_string(),
            notes: format!(
                "COMPLIANT: ground asserted with at least 1-month notice to quit; {}.",
                statutory_basis
            ),
            citations,
        };
    }
    Output {
        mode: NjAntiEvictionMode::ViolationNoticeToQuitDurationInsufficientForAssertedGround,
        statutory_basis: statutory_basis.to_string(),
        notes: format!(
            "VIOLATION: ground requires at least 1-month notice to quit; provided duration {:?} insufficient.",
            input.notice_to_quit_duration
        ),
        citations,
    }
}

fn require_at_least_18_months(
    input: &Input,
    compliant_mode: NjAntiEvictionMode,
    statutory_basis: &str,
    citations: Vec<String>,
) -> Output {
    if matches!(
        input.notice_to_quit_duration,
        NoticeToQuitDuration::EighteenMonthsOrMore | NoticeToQuitDuration::ThreeYearsOrMore
    ) {
        return Output {
            mode: compliant_mode,
            statutory_basis: statutory_basis.to_string(),
            notes: format!("COMPLIANT: owner-occupier or conversion ground asserted with at least 18-month notice to quit; {}.", statutory_basis),
            citations,
        };
    }
    Output {
        mode: NjAntiEvictionMode::ViolationNoticeToQuitDurationInsufficientForAssertedGround,
        statutory_basis: statutory_basis.to_string(),
        notes: format!(
            "VIOLATION: owner-occupier or conversion ground requires 18-month notice to quit; provided duration {:?} insufficient.",
            input.notice_to_quit_duration
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_nonpayment() -> Input {
        Input {
            property_classification: PropertyClassification::StandardRentalCoveredByAntiEvictionAct,
            eviction_ground: EvictionGround::SubsectionAFailureToPayRent,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::NoNoticeRequiredForNonpayment,
            building_total_apartments: 20,
            owner_occupies_one_apartment: false,
        }
    }

    #[test]
    fn property_outside_nj_not_applicable() {
        let input = Input {
            property_classification: PropertyClassification::PropertyOutsideNewJersey,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::NotApplicablePropertyOutsideNewJersey
        );
    }

    #[test]
    fn owner_occupied_3_apartments_exempt() {
        let input = Input {
            property_classification:
                PropertyClassification::OwnerOccupiedBuildingWith3OrFewerApartmentsExempt,
            building_total_apartments: 3,
            owner_occupies_one_apartment: true,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::NotApplicableOwnerOccupiedBuildingWith3OrFewerApartmentsExempt
        );
    }

    #[test]
    fn owner_occupied_4_apartments_exception_inapplicable() {
        let input = Input {
            property_classification:
                PropertyClassification::OwnerOccupiedBuildingWith3OrFewerApartmentsExempt,
            building_total_apartments: 4,
            owner_occupies_one_apartment: true,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::ViolationOwnerOccupiedExemptionClaimedButAboveThreeUnitsThreshold
        );
    }

    #[test]
    fn owner_occupied_exemption_but_owner_does_not_live_there_violation() {
        let input = Input {
            property_classification:
                PropertyClassification::OwnerOccupiedBuildingWith3OrFewerApartmentsExempt,
            building_total_apartments: 2,
            owner_occupies_one_apartment: false,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::ViolationOwnerOccupiedExemptionClaimedButAboveThreeUnitsThreshold
        );
    }

    #[test]
    fn no_ground_asserted_violation() {
        let input = Input {
            eviction_ground: EvictionGround::NoGroundAsserted,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::ViolationNoStatutoryGroundAsserted
        );
    }

    #[test]
    fn subsection_a_nonpayment_no_notice_compliant() {
        let result = check(&baseline_compliant_nonpayment());
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionANonpaymentNoNoticeRequiredEvictionProperlyFiled
        );
    }

    #[test]
    fn subsection_b_disorderly_with_notice_to_cease_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionBDisorderlyConductAfterNoticeToCease,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseProperlyServed,
            notice_to_quit_duration: NoticeToQuitDuration::OneMonthOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionBDisorderlyConductWithNoticeToCeaseAnd1MonthNoticeToQuit
        );
    }

    #[test]
    fn subsection_b_disorderly_no_notice_to_cease_violation() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionBDisorderlyConductAfterNoticeToCease,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotServed,
            notice_to_quit_duration: NoticeToQuitDuration::OneMonthOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::ViolationNoticeToCeaseNotServedForGroundRequiringIt
        );
    }

    #[test]
    fn subsection_d_substantial_lease_violation_with_notice_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionDSubstantialLeaseViolationAfterNoticeToCease,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseProperlyServed,
            notice_to_quit_duration: NoticeToQuitDuration::OneMonthOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionDSubstantialLeaseViolationWithNoticeToCeaseAnd1MonthNoticeToQuit
        );
    }

    #[test]
    fn subsection_f_habitual_late_payment_with_notice_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionFHabitualLatePaymentAfterNoticeToCease,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseProperlyServed,
            notice_to_quit_duration: NoticeToQuitDuration::OneMonthOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionFHabitualLatePaymentWithNoticeToCeaseAnd1MonthNoticeToQuit
        );
    }

    #[test]
    fn subsection_h_owner_retires_with_18_months_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionHOwnerRetiresFromRentalBusiness,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::EighteenMonthsOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionHOwnerRetiresFromRentalBusinessWith18MonthsNotice
        );
    }

    #[test]
    fn subsection_h_owner_retires_with_1_month_violation() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionHOwnerRetiresFromRentalBusiness,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::OneMonthOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::ViolationNoticeToQuitDurationInsufficientForAssertedGround
        );
    }

    #[test]
    fn subsection_j_condo_conversion_with_3_years_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionJConversionToCondominiumOrCooperative,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::ThreeYearsOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionJConversionToCondoOrCoopWith3YearsNotice
        );
    }

    #[test]
    fn subsection_j_condo_conversion_with_18_months_violation() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionJConversionToCondominiumOrCooperative,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::EighteenMonthsOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::ViolationNoticeToQuitDurationInsufficientForAssertedGround
        );
    }

    #[test]
    fn subsection_k_owner_moves_in_with_18_months_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionKOwnerPermanentlyMovesIntoUnit,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::EighteenMonthsOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionKOwnerPermanentlyMovesInWith18MonthsNotice
        );
    }

    #[test]
    fn subsection_o_drug_activity_with_3_day_notice_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionODrugRelatedCriminalActivity,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::BetweenThreeDaysAndOneMonth,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantCriminalActivitySubsectionsOPQRWith3DayNotice
        );
    }

    #[test]
    fn subsection_p_assault_with_less_than_3_days_violation() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionPAssaultThreatsWeaponsUse,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::LessThanThreeDays,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::ViolationNoticeToQuitDurationInsufficientForAssertedGround
        );
    }

    #[test]
    fn subsection_c_willful_destruction_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::SubsectionCWillfulOrGrosslyNegligentDestruction,
            notice_to_cease_status: NoticeToCeaseStatus::NoticeToCeaseNotRequiredForThisGround,
            notice_to_quit_duration: NoticeToQuitDuration::OneMonthOrMore,
            ..baseline_compliant_nonpayment()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NjAntiEvictionMode::CompliantSubsectionCWillfulOrGrosslyNegligentDestructionEvictionProper
        );
    }

    #[test]
    fn citations_pin_nj_anti_eviction_act_and_grounds() {
        let result = check(&baseline_compliant_nonpayment());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("New Jersey Anti-Eviction Act"));
        assert!(joined.contains("P.L. 1974, c. 49"));
        assert!(joined.contains("N.J.S.A. 2A:18-61.1"));
        assert!(joined.contains("OLDEST statewide just-cause eviction regime"));
        assert!(joined.contains("18 statutory grounds"));
        assert!(joined.contains("failure to pay rent"));
        assert!(joined.contains("disorderly conduct"));
        assert!(joined.contains("habitual late payment"));
        assert!(joined.contains("owner retires"));
        assert!(joined.contains("conversion to condominium/cooperative"));
        assert!(joined.contains("owner permanently moves into unit"));
        assert!(joined.contains("owner needs unit for family"));
        assert!(joined.contains("drug-related criminal activity"));
        assert!(joined.contains("3 or fewer apartments"));
        assert!(joined.contains("Notice to Cease"));
        assert!(joined.contains("No Notice Required for Nonpayment"));
        assert!(joined.contains("3 days"));
        assert!(joined.contains("18 months"));
        assert!(joined.contains("3 years"));
        assert!(joined.contains("N.J.S.A. 2A:18-61.7"));
        assert!(joined.contains("N.J.S.A. 2A:18-56"));
    }

    #[test]
    fn constant_pin_enactment_thresholds_and_notice_periods() {
        assert_eq!(NJ_ANTI_EVICTION_ACT_ENACTMENT_YEAR, 1974);
        assert_eq!(NJ_ANTI_EVICTION_ACT_PL_CHAPTER, 49);
        assert_eq!(NJ_ANTI_EVICTION_OWNER_OCCUPIED_MAX_UNITS_EXEMPT, 3);
        assert_eq!(NJ_ANTI_EVICTION_STATUTORY_GROUNDS_COUNT, 18);
        assert_eq!(NJ_ANTI_EVICTION_NOTICE_TO_QUIT_STANDARD_MONTHS, 1);
        assert_eq!(NJ_ANTI_EVICTION_OWNER_OCCUPIER_NOTICE_MONTHS, 18);
        assert_eq!(NJ_ANTI_EVICTION_CONDOMINIUM_CONVERSION_NOTICE_YEARS, 3);
        assert_eq!(NJ_ANTI_EVICTION_CRIMINAL_ACTIVITY_NOTICE_DAYS, 3);
    }
}
