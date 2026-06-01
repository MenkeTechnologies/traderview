//! Landlord vehicle towing from rental property — compliance check
//! for unauthorized-vehicle removal from rental-property parking
//! facilities. Trader-landlord operational concern when residential
//! parking is mismanaged, abandoned vehicles accumulate, or tenants
//! exceed allotted spaces.
//!
//! Distinct from `landlord_lien_prohibition` (statutory prohibition on
//! landlord lien over tenant personalty), `tenant_abandonment`
//! (procedures for treating possessions as abandoned after tenant
//! departure), and `abandoned_property_handling` (general personal-
//! property abandonment). This module addresses the SPECIFIC
//! VEHICLE-REMOVAL pathway under state vehicle/transportation codes.
//!
//! Four regimes:
//!
//! **California — Cal. Veh. Code § 22658**. Strict signage and timing
//! framework. Property owner or person in lawful possession may
//! remove a vehicle parked without permission ONLY if (i) a sign not
//! less than 17 inches by 22 inches with lettering not less than one
//! inch in height is posted in plain view at all entrances to the
//! property, prohibiting public parking, indicating vehicles will be
//! removed at owner's expense, and containing the telephone number of
//! the local traffic law enforcement agency plus the name and
//! telephone number of each towing company under a written general
//! towing authorization, AND (ii) at least 96 hours have elapsed
//! since a notice of parking violation was issued. Non-compliance
//! makes the property owner liable for DOUBLE the storage or towing
//! charges. § 22658(l)(1) liquidated damages.
//!
//! **Texas — Tex. Occ. Code Ch. 2308 (Vehicle Towing and Booting)**.
//! § 2308.252 prescribes signage requirements at parking facility
//! entrances. § 2308.253 contains the apartment-complex-specific rule
//! that a vehicle parked at an apartment complex may NOT be towed for
//! lacking current registration or license plate without 10 days of
//! advance written notice to the vehicle's last known registered
//! owner. § 2308.255 requires written verification from the parking
//! facility owner to the towing company that signs are posted before
//! the towing company may tow.
//!
//! **Florida — Fla. Stat. § 715.07**. Signage required before towing
//! from private property EXCEPT for single-family residences where
//! notice may be given personally to the vehicle's owner.
//! § 715.07(2)(a)(3) prescribes storage within a 10-mile radius of
//! the point of removal in any county of 500,000+ population, and
//! within 15 miles in counties under 500,000. Storage site must be
//! open 8:00 a.m. to 6:00 p.m. for redemption. § 715.07(2)(a)(4)
//! requires the towing company to notify the municipal police
//! department or sheriff WITHIN 30 MINUTES after completion of the
//! tow. § 715.07(4) requires stopping mid-tow when the vehicle owner
//! seeks return and accepting half the posted towing rate.
//!
//! **Default — common-law trespass to chattel + reasonable
//! self-help**. Most states require posted signage and reasonable
//! advance notice before removing a vehicle from private property.
//! Failure to comply may expose the landlord to conversion claims and
//! statutory penalties under state-specific vehicle codes.
//!
//! Citations: Cal. Veh. Code § 22658(a) (signage); § 22658(a)(1)
//! (17-by-22 inch sign at all entrances); § 22658(a)(2) (telephone
//! numbers); § 22658(l)(1) (96-hour parking violation notice rule);
//! § 22658(l)(1) (double charges liability for non-compliance); Tex.
//! Occ. Code § 2308.252 (signage); § 2308.253 (apartment 10-day
//! registration notice); § 2308.255 (written verification of signs);
//! Fla. Stat. § 715.07 (private property towing general);
//! § 715.07(2)(a)(3) (storage distance); § 715.07(2)(a)(4) (30-minute
//! police notification); § 715.07(4) (stop-during-tow + half-fee
//! redemption); § 715.07(2)(c) (single-family residence personal
//! notice exception).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    Florida,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TowReason {
    UnauthorizedParking,
    NoRegistrationOrLicensePlate,
    AbandonedVehicle,
    BlockingAccess,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TowingInput {
    pub regime: Regime,
    pub tow_reason: TowReason,
    /// Whether the property has CA-spec signage (17×22, 1" lettering,
    /// at all entrances, with required phone numbers). For TX/FL/
    /// Default, indicates whether minimum signage requirements per
    /// regime are met.
    pub signage_meets_requirements: bool,
    /// Whether the property is a single-family residence (FL-only
    /// exception under § 715.07(2)(c) permitting personal notice in
    /// lieu of signage).
    pub single_family_residence: bool,
    /// For CA: whether a parking-violation notice was issued and at
    /// least 96 hours have elapsed since issuance.
    pub ca_ninety_six_hours_elapsed_since_parking_notice: bool,
    /// For TX apartment complexes: 10-day advance written notice
    /// given to last known registered owner before towing for
    /// registration/license-plate reason.
    pub tx_ten_day_notice_given_for_registration_tow: bool,
    /// For TX: written verification from parking facility owner to
    /// towing company under § 2308.255.
    pub tx_written_signage_verification_provided: bool,
    /// For FL: storage location distance from point of removal in
    /// miles.
    pub fl_storage_distance_miles: u32,
    /// For FL: county population 500K+ (drives 10-mile vs 15-mile
    /// storage radius).
    pub fl_county_population_500k_plus: bool,
    /// For FL: minutes elapsed between completion of tow and
    /// police/sheriff notification.
    pub fl_minutes_to_law_enforcement_notification: u32,
    /// For FL: whether the towing company permitted the vehicle
    /// owner to recover the vehicle for half the posted rate when
    /// the owner appeared mid-tow under § 715.07(4).
    pub fl_stop_during_tow_half_fee_offered: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TowingResult {
    pub lawful_tow: bool,
    pub double_charges_liability: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TowingInput) -> TowingResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut double_liability = false;

    match input.regime {
        Regime::California => {
            if !input.signage_meets_requirements {
                violations.push(
                    "Cal. Veh. Code § 22658(a) signage missing or non-compliant — 17-by-22 inch sign with 1-inch lettering at all entrances required"
                        .to_string(),
                );
                double_liability = true;
            }
            if matches!(input.tow_reason, TowReason::UnauthorizedParking)
                && !input.ca_ninety_six_hours_elapsed_since_parking_notice
            {
                violations.push(
                    "Cal. Veh. Code § 22658 — 96 hours must elapse since parking-violation notice before removal"
                        .to_string(),
                );
                double_liability = true;
            }
            if double_liability {
                notes.push(
                    "§ 22658(l)(1) — property owner liable for DOUBLE storage or towing charges on non-compliance"
                        .to_string(),
                );
            }
        }
        Regime::Texas => {
            if !input.signage_meets_requirements {
                violations.push(
                    "Tex. Occ. Code § 2308.252 — required parking-facility signage not posted"
                        .to_string(),
                );
            }
            if !input.tx_written_signage_verification_provided {
                violations.push(
                    "Tex. Occ. Code § 2308.255 — written verification of signage from parking-facility owner to towing company required before towing"
                        .to_string(),
                );
            }
            if matches!(input.tow_reason, TowReason::NoRegistrationOrLicensePlate)
                && !input.tx_ten_day_notice_given_for_registration_tow
            {
                violations.push(
                    "Tex. Occ. Code § 2308.253 — apartment-complex vehicle may not be towed for missing registration or license plate without 10-day advance written notice"
                        .to_string(),
                );
            }
        }
        Regime::Florida => {
            if !input.signage_meets_requirements && !input.single_family_residence {
                violations.push(
                    "Fla. Stat. § 715.07 — signage required before private-property towing except single-family residence personal-notice exception"
                        .to_string(),
                );
            } else if input.single_family_residence {
                notes.push(
                    "§ 715.07(2)(c) single-family residence — personal notice in lieu of signage permitted"
                        .to_string(),
                );
            }
            let storage_limit = if input.fl_county_population_500k_plus { 10 } else { 15 };
            if input.fl_storage_distance_miles > storage_limit {
                violations.push(format!(
                    "Fla. Stat. § 715.07(2)(a)(3) — storage location {} miles exceeds {}-mile radius for {} county",
                    input.fl_storage_distance_miles,
                    storage_limit,
                    if input.fl_county_population_500k_plus { "500K+ population" } else { "under-500K population" }
                ));
            }
            if input.fl_minutes_to_law_enforcement_notification > 30 {
                violations.push(format!(
                    "Fla. Stat. § 715.07(2)(a)(4) — {} minutes to law-enforcement notification exceeds 30-minute deadline",
                    input.fl_minutes_to_law_enforcement_notification
                ));
            }
            if !input.fl_stop_during_tow_half_fee_offered {
                violations.push(
                    "Fla. Stat. § 715.07(4) — towing company must stop mid-tow on owner appearance and accept half the posted rate"
                        .to_string(),
                );
            }
        }
        Regime::Default => {
            if !input.signage_meets_requirements {
                violations.push(
                    "default common-law rule — posted signage and reasonable advance notice required before private-property vehicle removal"
                        .to_string(),
                );
                notes.push(
                    "non-compliance may expose landlord to common-law conversion and trespass-to-chattel claims plus state-specific vehicle code statutory penalties"
                        .to_string(),
                );
            }
        }
    }

    TowingResult {
        lawful_tow: violations.is_empty(),
        double_charges_liability: double_liability,
        violations,
        citation: citation_for(input.regime),
        notes,
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Veh. Code § 22658(a)/(l)(1)",
        Regime::Texas => "Tex. Occ. Code §§ 2308.252, 2308.253, 2308.255",
        Regime::Florida => "Fla. Stat. § 715.07(2)(a)(3)/(a)(4)/(c) and § 715.07(4)",
        Regime::Default => "common-law trespass to chattel + state-specific vehicle code",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> TowingInput {
        TowingInput {
            regime: Regime::California,
            tow_reason: TowReason::UnauthorizedParking,
            signage_meets_requirements: true,
            single_family_residence: false,
            ca_ninety_six_hours_elapsed_since_parking_notice: true,
            tx_ten_day_notice_given_for_registration_tow: false,
            tx_written_signage_verification_provided: false,
            fl_storage_distance_miles: 0,
            fl_county_population_500k_plus: false,
            fl_minutes_to_law_enforcement_notification: 0,
            fl_stop_during_tow_half_fee_offered: false,
        }
    }

    fn tx_base() -> TowingInput {
        TowingInput {
            regime: Regime::Texas,
            tow_reason: TowReason::UnauthorizedParking,
            signage_meets_requirements: true,
            single_family_residence: false,
            ca_ninety_six_hours_elapsed_since_parking_notice: false,
            tx_ten_day_notice_given_for_registration_tow: false,
            tx_written_signage_verification_provided: true,
            fl_storage_distance_miles: 0,
            fl_county_population_500k_plus: false,
            fl_minutes_to_law_enforcement_notification: 0,
            fl_stop_during_tow_half_fee_offered: false,
        }
    }

    fn fl_base() -> TowingInput {
        TowingInput {
            regime: Regime::Florida,
            tow_reason: TowReason::UnauthorizedParking,
            signage_meets_requirements: true,
            single_family_residence: false,
            ca_ninety_six_hours_elapsed_since_parking_notice: false,
            tx_ten_day_notice_given_for_registration_tow: false,
            tx_written_signage_verification_provided: false,
            fl_storage_distance_miles: 5,
            fl_county_population_500k_plus: true,
            fl_minutes_to_law_enforcement_notification: 15,
            fl_stop_during_tow_half_fee_offered: true,
        }
    }

    fn default_base() -> TowingInput {
        TowingInput {
            regime: Regime::Default,
            tow_reason: TowReason::UnauthorizedParking,
            signage_meets_requirements: true,
            single_family_residence: false,
            ca_ninety_six_hours_elapsed_since_parking_notice: false,
            tx_ten_day_notice_given_for_registration_tow: false,
            tx_written_signage_verification_provided: false,
            fl_storage_distance_miles: 0,
            fl_county_population_500k_plus: false,
            fl_minutes_to_law_enforcement_notification: 0,
            fl_stop_during_tow_half_fee_offered: false,
        }
    }

    #[test]
    fn ca_compliant_signage_and_ninety_six_hours_lawful() {
        let r = check(&ca_base());
        assert!(r.lawful_tow);
        assert!(!r.double_charges_liability);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_missing_signage_double_liability_engages() {
        let mut i = ca_base();
        i.signage_meets_requirements = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r.double_charges_liability);
        assert!(r.violations.iter().any(|v| v.contains("17-by-22 inch")));
        assert!(r.notes.iter().any(|n| n.contains("DOUBLE storage")));
    }

    #[test]
    fn ca_signage_compliant_but_under_96_hours_violation_and_double_liability() {
        let mut i = ca_base();
        i.ca_ninety_six_hours_elapsed_since_parking_notice = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r.double_charges_liability);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("96 hours must elapse")));
    }

    #[test]
    fn ca_abandoned_vehicle_no_96_hour_requirement_with_signage_lawful() {
        let mut i = ca_base();
        i.tow_reason = TowReason::AbandonedVehicle;
        i.ca_ninety_six_hours_elapsed_since_parking_notice = false;
        let r = check(&i);
        assert!(r.lawful_tow, "96-hour rule applies only to UnauthorizedParking");
    }

    #[test]
    fn ca_blocking_access_no_96_hour_requirement_with_signage_lawful() {
        let mut i = ca_base();
        i.tow_reason = TowReason::BlockingAccess;
        i.ca_ninety_six_hours_elapsed_since_parking_notice = false;
        let r = check(&i);
        assert!(r.lawful_tow);
    }

    #[test]
    fn tx_compliant_signage_and_verification_lawful() {
        let r = check(&tx_base());
        assert!(r.lawful_tow);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn tx_missing_signage_violation() {
        let mut i = tx_base();
        i.signage_meets_requirements = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r.violations.iter().any(|v| v.contains("§ 2308.252")));
    }

    #[test]
    fn tx_missing_written_verification_violation() {
        let mut i = tx_base();
        i.tx_written_signage_verification_provided = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r.violations.iter().any(|v| v.contains("§ 2308.255")));
    }

    #[test]
    fn tx_apartment_registration_tow_requires_10_day_notice_violation_when_absent() {
        let mut i = tx_base();
        i.tow_reason = TowReason::NoRegistrationOrLicensePlate;
        i.tx_ten_day_notice_given_for_registration_tow = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r.violations.iter().any(|v| v.contains("§ 2308.253")));
        assert!(r.violations.iter().any(|v| v.contains("10-day")));
    }

    #[test]
    fn tx_apartment_registration_tow_lawful_with_10_day_notice() {
        let mut i = tx_base();
        i.tow_reason = TowReason::NoRegistrationOrLicensePlate;
        i.tx_ten_day_notice_given_for_registration_tow = true;
        let r = check(&i);
        assert!(r.lawful_tow);
    }

    #[test]
    fn fl_compliant_signage_storage_within_10_miles_500k_county_lawful() {
        let r = check(&fl_base());
        assert!(r.lawful_tow);
    }

    #[test]
    fn fl_storage_exceeds_10_mile_radius_500k_county_violation() {
        let mut i = fl_base();
        i.fl_storage_distance_miles = 11;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 715.07(2)(a)(3)")));
        assert!(r.violations.iter().any(|v| v.contains("11 miles")));
    }

    #[test]
    fn fl_storage_within_15_mile_radius_under_500k_county_lawful() {
        let mut i = fl_base();
        i.fl_county_population_500k_plus = false;
        i.fl_storage_distance_miles = 13;
        let r = check(&i);
        assert!(r.lawful_tow);
    }

    #[test]
    fn fl_storage_exceeds_15_mile_radius_under_500k_county_violation() {
        let mut i = fl_base();
        i.fl_county_population_500k_plus = false;
        i.fl_storage_distance_miles = 16;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("16 miles") && v.contains("15-mile")));
    }

    #[test]
    fn fl_law_enforcement_notification_within_30_minutes_lawful() {
        let mut i = fl_base();
        i.fl_minutes_to_law_enforcement_notification = 30;
        let r = check(&i);
        assert!(r.lawful_tow);
    }

    #[test]
    fn fl_law_enforcement_notification_at_31_minutes_violation() {
        let mut i = fl_base();
        i.fl_minutes_to_law_enforcement_notification = 31;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 715.07(2)(a)(4)")));
    }

    #[test]
    fn fl_half_fee_redemption_not_offered_violation() {
        let mut i = fl_base();
        i.fl_stop_during_tow_half_fee_offered = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r.violations.iter().any(|v| v.contains("§ 715.07(4)")));
        assert!(r.violations.iter().any(|v| v.contains("half the posted rate")));
    }

    #[test]
    fn fl_single_family_residence_personal_notice_exception_signage_not_required() {
        let mut i = fl_base();
        i.signage_meets_requirements = false;
        i.single_family_residence = true;
        let r = check(&i);
        assert!(r.lawful_tow);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 715.07(2)(c)")));
    }

    #[test]
    fn fl_no_signage_no_single_family_violation() {
        let mut i = fl_base();
        i.signage_meets_requirements = false;
        i.single_family_residence = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r.violations.iter().any(|v| v.contains("§ 715.07")));
    }

    #[test]
    fn default_compliant_signage_lawful() {
        let r = check(&default_base());
        assert!(r.lawful_tow);
    }

    #[test]
    fn default_missing_signage_violation_with_conversion_warning() {
        let mut i = default_base();
        i.signage_meets_requirements = false;
        let r = check(&i);
        assert!(!r.lawful_tow);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("default common-law")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("conversion") && n.contains("trespass-to-chattel")));
    }

    #[test]
    fn citation_california_pins_22658_subsections() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§ 22658(a)"));
        assert!(r.citation.contains("(l)(1)"));
    }

    #[test]
    fn citation_texas_pins_all_three_sections() {
        let r = check(&tx_base());
        assert!(r.citation.contains("§§ 2308.252, 2308.253, 2308.255"));
    }

    #[test]
    fn citation_florida_pins_715_07_subsections() {
        let r = check(&fl_base());
        assert!(r.citation.contains("§ 715.07(2)(a)(3)"));
        assert!(r.citation.contains("(a)(4)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("§ 715.07(4)"));
    }

    #[test]
    fn double_liability_only_engages_in_california_invariant() {
        for regime in [Regime::Texas, Regime::Florida, Regime::Default] {
            let mut i = ca_base();
            i.regime = regime;
            i.signage_meets_requirements = false;
            i.tx_written_signage_verification_provided = true;
            i.fl_storage_distance_miles = 5;
            i.fl_minutes_to_law_enforcement_notification = 15;
            i.fl_stop_during_tow_half_fee_offered = true;
            let r = check(&i);
            assert!(!r.double_charges_liability, "regime {:?}", regime);
        }
    }

    #[test]
    fn ca_ten_mile_storage_irrelevant_to_california_lawful() {
        let mut i = ca_base();
        i.fl_storage_distance_miles = 999;
        let r = check(&i);
        assert!(r.lawful_tow, "FL storage rule does not apply to CA");
    }
}
