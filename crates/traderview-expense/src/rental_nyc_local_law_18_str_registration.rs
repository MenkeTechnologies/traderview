//! NYC Local Law 18 of 2022 Short-Term Rental Registration
//! Law Compliance Module.
//!
//! Pure-compute check for landlord / host / booking-platform
//! compliance with NYC Local Law 18 of 2022 — the Short-Term
//! Rental Registration Law administered by the NYC Office of
//! Special Enforcement (OSE) under the Mayor's Office of
//! Criminal Justice. Signed January 9, 2022; enforcement
//! began September 5, 2023; codified at NYC Admin Code
//! § 26-3001 through § 26-3007 with 31 RCNY Part 12
//! implementing rules. Reduced NYC active STR listings from
//! over 38,000 in early 2023 to approximately 3,000
//! registered listings by mid-2025 — a ~92 % reduction in
//! active STR supply.
//!
//! Web research (verified 2026-06-03):
//! - **NYC Local Law 18 of 2022** (Short-Term Rental
//!   Registration Law) — signed **January 9, 2022**;
//!   enforcement began **September 5, 2023**; codified at
//!   **NYC Admin Code § 26-3001 through § 26-3007**;
//!   administered by NYC **Office of Special Enforcement
//!   (OSE)** under Mayor's Office of Criminal Justice ([NYC
//!   OSE — Registration Law](https://www.nyc.gov/site/specialenforcement/registration-law/registration.page);
//!   [Local Law 18 of 2022 — Wikipedia](https://en.wikipedia.org/wiki/Local_Law_18_of_2022);
//!   [Mayor's Office of Criminal Justice — LL18 Report Sheds
//!   Light on Eliminated Illegal Rentals](https://criminaljustice.cityofnewyork.us/press-release/ll18-report-sheds-light-on-eliminated-illegal-rentals-in-nyc/)).
//! - **Short-Term Rental Definition**: rental of a dwelling
//!   unit or part of a dwelling for fewer than **30
//!   CONSECUTIVE DAYS**; rentals at or above 30 days are
//!   long-term rentals not subject to LL 18.
//! - **Registration Requirement**: hosts must register with
//!   OSE **BEFORE listing** a property for short-term
//!   rental; booking platforms (Airbnb, VRBO, Booking.com,
//!   etc.) prohibited from processing transactions for
//!   unregistered STRs.
//! - **Host Present Requirement (NYC Admin Code § 26-3001)**:
//!   the **permanent occupant must be present** during the
//!   guest stay, sharing the dwelling as a **"common
//!   household"**; maximum **2 PAYING GUESTS** at a time;
//!   interior doors cannot be configured to deny guest
//!   access to household areas.
//! - **Class B Multiple Dwelling Exemption**: short-term
//!   rental listings for units in **Class B multiple
//!   dwellings** approved by the City of New York for legal
//!   short-term occupancies (licensed hotels, motels,
//!   hostels, bed-and-breakfasts, officially permitted
//!   rooming houses) are EXEMPT from registration ([NY
//!   Multiple Dwelling Law § 4(8)(a)](https://www.nysenate.gov/legislation/laws/MDW/4)).
//! - **Penalties** (NYC Admin Code § 26-3006): violations
//!   subject to civil penalties of **$100 to $5,000 per
//!   violation** against host; up to **$1,500 per infraction**
//!   against booking services; **3 TIMES the illegal revenue
//!   collected** as financial penalty; revocation of
//!   registration for non-compliant hosts.
//! - **Prohibited Buildings List (PBL)**: building owners
//!   (including cooperative and condominium boards) may apply
//!   under 31 RCNY Part 12 to have their building added to
//!   the PBL, preventing all STR registrations within the
//!   building ([Belkin Burden Goldman — City Adopts Rules
//!   Allowing Owners to Apply For Short-Term Rental Prohibited
//!   Buildings List](https://bbgllp.com/new/city-adopts-rules-allowing-owners-to-apply-for-short-term-rental-prohibited-buildings-list/)).
//! - **Enforcement Impact**: from over **38,000 active
//!   listings** in early 2023 to approximately **3,000
//!   registered listings** by mid-2025 — a **~92 % reduction**
//!   in active STR supply ([NYC OSE — One Year Later Report](https://www.nyc.gov/site/specialenforcement/news/new-report-sheds-fresh-light-on-how-local-law-18.page);
//!   [Lodgify — One Year Later Report](https://www.lodgify.com/blog/local-law-18-one-year-report/)).
//! - **First LL18 Lawsuit**: NYC filed first civil lawsuit
//!   under LL18 against operator alleging operation of
//!   illegal short-term rentals in violation of registration
//!   and host-present requirements.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const NYC_LL_18_SIGNED_DATE_YEAR: u32 = 2022;
pub const NYC_LL_18_SIGNED_DATE_MONTH: u32 = 1;
pub const NYC_LL_18_SIGNED_DATE_DAY: u32 = 9;
pub const NYC_LL_18_ENFORCEMENT_START_YEAR: u32 = 2023;
pub const NYC_LL_18_ENFORCEMENT_START_MONTH: u32 = 9;
pub const NYC_LL_18_ENFORCEMENT_START_DAY: u32 = 5;
pub const NYC_LL_18_SHORT_TERM_RENTAL_DAYS_THRESHOLD: u32 = 30;
pub const NYC_LL_18_MAX_PAYING_GUESTS: u32 = 2;
pub const NYC_LL_18_HOST_VIOLATION_MIN_PENALTY_DOLLARS: u64 = 100;
pub const NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS: u64 = 5_000;
pub const NYC_LL_18_BOOKING_SERVICE_MAX_PENALTY_DOLLARS: u64 = 1_500;
pub const NYC_LL_18_REVENUE_MULTIPLIER_PENALTY: u64 = 3;
pub const NYC_LL_18_PRE_LL18_ACTIVE_LISTINGS_COUNT: u32 = 38_000;
pub const NYC_LL_18_POST_LL18_REGISTERED_LISTINGS_COUNT: u32 = 3_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DwellingClassification {
    StandardResidentialDwellingUnit,
    ClassBMultipleDwellingExempt,
    NotInNyc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RentalDuration {
    UnderThirtyConsecutiveDaysSubjectToLl18,
    AtLeastThirtyConsecutiveDaysLongTermNotSubject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostPresenceStatus {
    PermanentOccupantPresentSharedCommonHousehold,
    PermanentOccupantNotPresentEntireUnitRented,
    InteriorDoorsConfiguredToDenyGuestAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    RegisteredWithOseBeforeFirstListing,
    NotRegisteredWithOse,
    RegistrationRevokedForNonCompliance,
    InProhibitedBuildingsListPbl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NycLocalLaw18Mode {
    NotApplicableNotInNyc,
    NotApplicableClassBMultipleDwellingExempt,
    NotApplicableRentalAtLeast30DaysNotShortTerm,
    CompliantRegisteredHostPresent2GuestsMaxNonViolating,
    ViolationUnregisteredShortTermRental,
    ViolationProhibitedBuildingsListPbl,
    ViolationHostNotPresentEntireUnitRented,
    ViolationMoreThan2PayingGuests,
    ViolationInteriorDoorsConfiguredToDenyGuestAccess,
    ViolationBookingServiceProcessedTransactionForUnregisteredStr,
    ViolationRevenueMultiplierPenalty3xIllegalRevenue,
    ViolationRegistrationRevokedContinuingListings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub dwelling_classification: DwellingClassification,
    pub rental_duration: RentalDuration,
    pub host_presence_status: HostPresenceStatus,
    pub registration_status: RegistrationStatus,
    pub number_of_paying_guests: u32,
    pub booking_service_processed_unregistered_listing: bool,
    pub illegal_revenue_collected_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: NycLocalLaw18Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub host_civil_penalty_dollars: u64,
    pub booking_service_civil_penalty_dollars: u64,
    pub revenue_multiplier_penalty_dollars: u64,
}

pub type RentalNycLocalLaw18StrRegistrationInput = Input;
pub type RentalNycLocalLaw18StrRegistrationOutput = Output;
pub type RentalNycLocalLaw18StrRegistrationResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "NYC Local Law 18 of 2022 — Short-Term Rental Registration Law; signed January 9, 2022; enforcement began September 5, 2023; codified at NYC Admin Code § 26-3001 through § 26-3007; administered by NYC Office of Special Enforcement (OSE) under Mayor's Office of Criminal Justice".to_string(),
        "31 RCNY Part 12 — implementing rules for OSE STR registration, Prohibited Buildings List, and enforcement procedures".to_string(),
        "Short-Term Rental Definition — rental of a dwelling unit or part of a dwelling for fewer than 30 CONSECUTIVE DAYS; rentals at or above 30 days are long-term rentals not subject to LL 18".to_string(),
        "Registration Requirement — hosts must register with OSE BEFORE listing; booking platforms (Airbnb, VRBO, Booking.com, etc.) prohibited from processing transactions for unregistered STRs; verification mechanism with platforms".to_string(),
        "Host Present Requirement (NYC Admin Code § 26-3001) — permanent occupant must be present during guest stay, sharing dwelling as 'common household'; maximum 2 PAYING GUESTS at a time; interior doors cannot be configured to deny guest access to household areas".to_string(),
        "Class B Multiple Dwelling Exemption — STR listings for units in Class B multiple dwellings approved by City of New York for legal short-term occupancies (licensed hotels, motels, hostels, bed-and-breakfasts, officially permitted rooming houses) EXEMPT from registration; controlled by NY Multiple Dwelling Law § 4(8)(a)".to_string(),
        "Penalties (NYC Admin Code § 26-3006) — civil penalties of $100 to $5,000 per violation against host; up to $1,500 per infraction against booking services; 3 TIMES illegal revenue collected as financial penalty; revocation of registration for non-compliant hosts".to_string(),
        "Prohibited Buildings List (PBL) — building owners (including cooperative and condominium boards) may apply under 31 RCNY Part 12 to have their building added to PBL, preventing all STR registrations within the building".to_string(),
        "Enforcement Impact — from over 38,000 active listings in early 2023 to approximately 3,000 registered listings by mid-2025 — a ~92 % reduction in active STR supply".to_string(),
        "First LL18 Lawsuit — NYC filed first civil lawsuit under LL18 against operator alleging operation of illegal short-term rentals in violation of registration and host-present requirements".to_string(),
        "NYC OSE Registration Law primary landing page".to_string(),
        "Mayor's Office of Criminal Justice LL18 Report Sheds Light on Eliminated Illegal Rentals".to_string(),
        "Belkin Burden Goldman — City Adopts Rules Allowing Owners to Apply For Short-Term Rental Prohibited Buildings List".to_string(),
        "Lodgify — One Year Later Local Law 18 Report".to_string(),
    ];

    if input.dwelling_classification == DwellingClassification::NotInNyc {
        return Output {
            mode: NycLocalLaw18Mode::NotApplicableNotInNyc,
            statutory_basis: "Property outside NYC; LL 18 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside NYC; NYC Local Law 18 of 2022 inapplicable.".to_string(),
            citations,
            host_civil_penalty_dollars: 0,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars: 0,
        };
    }

    if input.dwelling_classification == DwellingClassification::ClassBMultipleDwellingExempt {
        return Output {
            mode: NycLocalLaw18Mode::NotApplicableClassBMultipleDwellingExempt,
            statutory_basis: "Class B Multiple Dwelling exemption — LL 18 registration not required".to_string(),
            notes: "NOT APPLICABLE: Class B Multiple Dwelling approved by City of New York for legal short-term occupancies (licensed hotels, motels, hostels, B&Bs, permitted rooming houses); LL 18 registration exempt.".to_string(),
            citations,
            host_civil_penalty_dollars: 0,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars: 0,
        };
    }

    if input.rental_duration == RentalDuration::AtLeastThirtyConsecutiveDaysLongTermNotSubject {
        return Output {
            mode: NycLocalLaw18Mode::NotApplicableRentalAtLeast30DaysNotShortTerm,
            statutory_basis: "Rental ≥ 30 consecutive days — long-term rental not subject to LL 18".to_string(),
            notes: "NOT APPLICABLE: rental at or above 30 consecutive days is long-term rental; LL 18 STR registration requirements do not apply.".to_string(),
            citations,
            host_civil_penalty_dollars: 0,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars: 0,
        };
    }

    let revenue_multiplier_penalty_dollars = input
        .illegal_revenue_collected_dollars
        .saturating_mul(NYC_LL_18_REVENUE_MULTIPLIER_PENALTY);

    if input.registration_status == RegistrationStatus::InProhibitedBuildingsListPbl {
        return Output {
            mode: NycLocalLaw18Mode::ViolationProhibitedBuildingsListPbl,
            statutory_basis: "31 RCNY Part 12 — building on Prohibited Buildings List (PBL); no STR registrations permitted".to_string(),
            notes: "VIOLATION: building added to NYC OSE Prohibited Buildings List (PBL) by owner/board under 31 RCNY Part 12; no STR registrations permitted within building.".to_string(),
            citations,
            host_civil_penalty_dollars: NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars,
        };
    }

    if input.registration_status == RegistrationStatus::RegistrationRevokedForNonCompliance {
        return Output {
            mode: NycLocalLaw18Mode::ViolationRegistrationRevokedContinuingListings,
            statutory_basis: "NYC Admin Code § 26-3006 — registration revocation for non-compliance".to_string(),
            notes: "VIOLATION: OSE registration revoked for non-compliance; continued listings prohibited; further civil penalties accrue.".to_string(),
            citations,
            host_civil_penalty_dollars: NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars,
        };
    }

    if input.registration_status == RegistrationStatus::NotRegisteredWithOse {
        let booking_penalty = if input.booking_service_processed_unregistered_listing {
            NYC_LL_18_BOOKING_SERVICE_MAX_PENALTY_DOLLARS
        } else {
            0
        };
        return Output {
            mode: NycLocalLaw18Mode::ViolationUnregisteredShortTermRental,
            statutory_basis: "NYC Local Law 18 of 2022 — host registration with OSE required before listing".to_string(),
            notes: format!(
                "VIOLATION: short-term rental listed without OSE registration; civil penalty up to ${} against host; revenue multiplier penalty 3 × ${} = ${}.",
                NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS,
                input.illegal_revenue_collected_dollars,
                revenue_multiplier_penalty_dollars
            ),
            citations,
            host_civil_penalty_dollars: NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS,
            booking_service_civil_penalty_dollars: booking_penalty,
            revenue_multiplier_penalty_dollars,
        };
    }

    if input.booking_service_processed_unregistered_listing {
        return Output {
            mode: NycLocalLaw18Mode::ViolationBookingServiceProcessedTransactionForUnregisteredStr,
            statutory_basis: "NYC Local Law 18 of 2022 — booking platform prohibited from processing transactions for unregistered STRs".to_string(),
            notes: format!(
                "VIOLATION: booking platform processed transaction for unregistered STR; civil penalty up to ${} per infraction against booking service.",
                NYC_LL_18_BOOKING_SERVICE_MAX_PENALTY_DOLLARS
            ),
            citations,
            host_civil_penalty_dollars: 0,
            booking_service_civil_penalty_dollars: NYC_LL_18_BOOKING_SERVICE_MAX_PENALTY_DOLLARS,
            revenue_multiplier_penalty_dollars: 0,
        };
    }

    if input.host_presence_status == HostPresenceStatus::PermanentOccupantNotPresentEntireUnitRented
    {
        return Output {
            mode: NycLocalLaw18Mode::ViolationHostNotPresentEntireUnitRented,
            statutory_basis: "NYC Admin Code § 26-3001 — permanent occupant must be present during STR stay".to_string(),
            notes: "VIOLATION: permanent occupant not present during STR stay; entire unit rented to guests; host-present requirement under § 26-3001 violated.".to_string(),
            citations,
            host_civil_penalty_dollars: NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars,
        };
    }

    if input.host_presence_status == HostPresenceStatus::InteriorDoorsConfiguredToDenyGuestAccess {
        return Output {
            mode: NycLocalLaw18Mode::ViolationInteriorDoorsConfiguredToDenyGuestAccess,
            statutory_basis: "NYC Admin Code § 26-3001 — interior doors cannot be configured to deny guest access".to_string(),
            notes: "VIOLATION: interior doors configured to deny guest access to household areas; common-household requirement under § 26-3001 violated.".to_string(),
            citations,
            host_civil_penalty_dollars: NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars,
        };
    }

    if input.number_of_paying_guests > NYC_LL_18_MAX_PAYING_GUESTS {
        return Output {
            mode: NycLocalLaw18Mode::ViolationMoreThan2PayingGuests,
            statutory_basis: "NYC Admin Code § 26-3001 — maximum 2 paying guests at a time".to_string(),
            notes: format!(
                "VIOLATION: {} paying guests exceeds 2-guest maximum under § 26-3001.",
                input.number_of_paying_guests
            ),
            citations,
            host_civil_penalty_dollars: NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS,
            booking_service_civil_penalty_dollars: 0,
            revenue_multiplier_penalty_dollars,
        };
    }

    Output {
        mode: NycLocalLaw18Mode::CompliantRegisteredHostPresent2GuestsMaxNonViolating,
        statutory_basis: "NYC Local Law 18 of 2022 — registered, host present, ≤ 2 guests, non-violating".to_string(),
        notes: format!(
            "COMPLIANT: short-term rental registered with OSE; permanent occupant present sharing common household; {} paying guests (≤ 2 maximum); interior doors do not deny guest access.",
            input.number_of_paying_guests
        ),
        citations,
        host_civil_penalty_dollars: 0,
        booking_service_civil_penalty_dollars: 0,
        revenue_multiplier_penalty_dollars: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_registered() -> Input {
        Input {
            dwelling_classification: DwellingClassification::StandardResidentialDwellingUnit,
            rental_duration: RentalDuration::UnderThirtyConsecutiveDaysSubjectToLl18,
            host_presence_status: HostPresenceStatus::PermanentOccupantPresentSharedCommonHousehold,
            registration_status: RegistrationStatus::RegisteredWithOseBeforeFirstListing,
            number_of_paying_guests: 2,
            booking_service_processed_unregistered_listing: false,
            illegal_revenue_collected_dollars: 0,
        }
    }

    #[test]
    fn property_not_in_nyc_not_applicable() {
        let input = Input {
            dwelling_classification: DwellingClassification::NotInNyc,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(result.mode, NycLocalLaw18Mode::NotApplicableNotInNyc);
    }

    #[test]
    fn class_b_multiple_dwelling_exempt() {
        let input = Input {
            dwelling_classification: DwellingClassification::ClassBMultipleDwellingExempt,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::NotApplicableClassBMultipleDwellingExempt
        );
    }

    #[test]
    fn long_term_rental_30_days_not_subject() {
        let input = Input {
            rental_duration: RentalDuration::AtLeastThirtyConsecutiveDaysLongTermNotSubject,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::NotApplicableRentalAtLeast30DaysNotShortTerm
        );
    }

    #[test]
    fn registered_compliant_baseline() {
        let result = check(&baseline_compliant_registered());
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::CompliantRegisteredHostPresent2GuestsMaxNonViolating
        );
    }

    #[test]
    fn at_exactly_2_guests_compliant() {
        let result = check(&baseline_compliant_registered());
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::CompliantRegisteredHostPresent2GuestsMaxNonViolating
        );
    }

    #[test]
    fn one_guest_compliant() {
        let input = Input {
            number_of_paying_guests: 1,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::CompliantRegisteredHostPresent2GuestsMaxNonViolating
        );
    }

    #[test]
    fn unregistered_str_violation() {
        let input = Input {
            registration_status: RegistrationStatus::NotRegisteredWithOse,
            illegal_revenue_collected_dollars: 5_000,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationUnregisteredShortTermRental
        );
        assert_eq!(result.host_civil_penalty_dollars, 5_000);
        assert_eq!(result.revenue_multiplier_penalty_dollars, 15_000);
    }

    #[test]
    fn unregistered_with_booking_service_violation() {
        let input = Input {
            registration_status: RegistrationStatus::NotRegisteredWithOse,
            booking_service_processed_unregistered_listing: true,
            illegal_revenue_collected_dollars: 10_000,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationUnregisteredShortTermRental
        );
        assert_eq!(result.host_civil_penalty_dollars, 5_000);
        assert_eq!(result.booking_service_civil_penalty_dollars, 1_500);
        assert_eq!(result.revenue_multiplier_penalty_dollars, 30_000);
    }

    #[test]
    fn prohibited_buildings_list_violation() {
        let input = Input {
            registration_status: RegistrationStatus::InProhibitedBuildingsListPbl,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(result.mode, NycLocalLaw18Mode::ViolationProhibitedBuildingsListPbl);
    }

    #[test]
    fn registration_revoked_violation() {
        let input = Input {
            registration_status: RegistrationStatus::RegistrationRevokedForNonCompliance,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationRegistrationRevokedContinuingListings
        );
    }

    #[test]
    fn host_not_present_violation() {
        let input = Input {
            host_presence_status: HostPresenceStatus::PermanentOccupantNotPresentEntireUnitRented,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationHostNotPresentEntireUnitRented
        );
    }

    #[test]
    fn interior_doors_deny_access_violation() {
        let input = Input {
            host_presence_status: HostPresenceStatus::InteriorDoorsConfiguredToDenyGuestAccess,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationInteriorDoorsConfiguredToDenyGuestAccess
        );
    }

    #[test]
    fn three_paying_guests_violation() {
        let input = Input {
            number_of_paying_guests: 3,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationMoreThan2PayingGuests
        );
    }

    #[test]
    fn booking_service_processed_unregistered_violation() {
        let input = Input {
            booking_service_processed_unregistered_listing: true,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationBookingServiceProcessedTransactionForUnregisteredStr
        );
        assert_eq!(result.booking_service_civil_penalty_dollars, 1_500);
    }

    #[test]
    fn revenue_multiplier_3x_illegal_revenue_calculation() {
        let input = Input {
            registration_status: RegistrationStatus::NotRegisteredWithOse,
            illegal_revenue_collected_dollars: 25_000,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(result.revenue_multiplier_penalty_dollars, 75_000);
    }

    #[test]
    fn citations_pin_ll18_admin_code_and_enforcement() {
        let result = check(&baseline_compliant_registered());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("NYC Local Law 18 of 2022"));
        assert!(joined.contains("January 9, 2022"));
        assert!(joined.contains("September 5, 2023"));
        assert!(joined.contains("NYC Admin Code § 26-3001"));
        assert!(joined.contains("NYC Admin Code § 26-3006"));
        assert!(joined.contains("Office of Special Enforcement"));
        assert!(joined.contains("OSE"));
        assert!(joined.contains("Mayor's Office of Criminal Justice"));
        assert!(joined.contains("31 RCNY Part 12"));
        assert!(joined.contains("30 CONSECUTIVE DAYS"));
        assert!(joined.contains("2 PAYING GUESTS"));
        assert!(joined.contains("Class B Multiple Dwelling"));
        assert!(joined.contains("NY Multiple Dwelling Law § 4(8)(a)"));
        assert!(joined.contains("$100 to $5,000"));
        assert!(joined.contains("$1,500"));
        assert!(joined.contains("3 TIMES"));
        assert!(joined.contains("Prohibited Buildings List"));
        assert!(joined.contains("PBL"));
        assert!(joined.contains("38,000"));
        assert!(joined.contains("3,000"));
        assert!(joined.contains("92 %"));
    }

    #[test]
    fn constant_pin_dates_thresholds_and_penalties() {
        assert_eq!(NYC_LL_18_SIGNED_DATE_YEAR, 2022);
        assert_eq!(NYC_LL_18_SIGNED_DATE_MONTH, 1);
        assert_eq!(NYC_LL_18_SIGNED_DATE_DAY, 9);
        assert_eq!(NYC_LL_18_ENFORCEMENT_START_YEAR, 2023);
        assert_eq!(NYC_LL_18_ENFORCEMENT_START_MONTH, 9);
        assert_eq!(NYC_LL_18_ENFORCEMENT_START_DAY, 5);
        assert_eq!(NYC_LL_18_SHORT_TERM_RENTAL_DAYS_THRESHOLD, 30);
        assert_eq!(NYC_LL_18_MAX_PAYING_GUESTS, 2);
        assert_eq!(NYC_LL_18_HOST_VIOLATION_MIN_PENALTY_DOLLARS, 100);
        assert_eq!(NYC_LL_18_HOST_VIOLATION_MAX_PENALTY_DOLLARS, 5_000);
        assert_eq!(NYC_LL_18_BOOKING_SERVICE_MAX_PENALTY_DOLLARS, 1_500);
        assert_eq!(NYC_LL_18_REVENUE_MULTIPLIER_PENALTY, 3);
        assert_eq!(NYC_LL_18_PRE_LL18_ACTIVE_LISTINGS_COUNT, 38_000);
        assert_eq!(NYC_LL_18_POST_LL18_REGISTERED_LISTINGS_COUNT, 3_000);
    }

    #[test]
    fn saturating_overflow_defense_extreme_revenue() {
        let input = Input {
            registration_status: RegistrationStatus::NotRegisteredWithOse,
            illegal_revenue_collected_dollars: u64::MAX,
            ..baseline_compliant_registered()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            NycLocalLaw18Mode::ViolationUnregisteredShortTermRental
        );
        assert_eq!(result.revenue_multiplier_penalty_dollars, u64::MAX);
    }
}
