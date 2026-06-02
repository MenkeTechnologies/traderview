//! Short-term rental (STR) conversion restriction compliance
//! — when may a trader-landlord lawfully convert a long-term
//! residential rental unit to a short-term rental (Airbnb /
//! VRBO / Booking.com listing under 30 nights) without
//! violating local registration ordinances? Trader-landlord
//! critical for any rental owner in NYC + SF + LA
//! considering STR conversion: unregistered hosting exposes
//! owner to per-violation penalties + booking-platform
//! verification refusal + ordinance enforcement.
//!
//! Distinct from siblings `rental_property_registration`
//! (general landlord registration), `condominium_conversion_
//! protection` (condo conversion), `tenant_relocation_
//! assistance` (relocation), and `just_cause_eviction`
//! (eviction).
//!
//! **Four regimes**:
//!
//! **New York City — Local Law 18 (effective September 5,
//! 2023)**:
//! - All hosts must REGISTER with Office of Special
//!   Enforcement (OSE) before listing.
//! - Applies to stays under **30 nights**.
//! - Booking platforms must verify valid registration before
//!   processing reservations.
//! - Penalties up to **$5,000 per violation** for hosts AND
//!   platforms.
//! - Host must be permanent occupant of the unit AND
//!   present during stay.
//! - OSE has denied 4,300+ non-compliant applications;
//!   approved 3,000+ host registrations.
//!
//! **San Francisco — Office of Short-Term Rentals
//! Ordinance 218-14 (Chapter 41A SF Admin. Code)**:
//! - Host must REGISTER and obtain Business Registration
//!   Certificate.
//! - Primary residence requirement: **270 nights/year
//!   occupancy** (down from 275 days).
//! - Cap of **90 unhosted nights per year**.
//! - Penalties up to **$1,000 per day** for unregistered
//!   hosting.
//!
//! **Los Angeles — Home-Sharing Ordinance (effective
//! November 1, 2019)**:
//! - Host must REGISTER with LADBS Home-Sharing Program.
//! - Primary residence requirement.
//! - Cap of **120 days/year unhosted** by default; extended
//!   to **240 days/year** with Extended Home-Sharing permit.
//! - Penalties + recordation under LAMC § 12.22 A.32.
//!
//! **Default — locality-controlled at state level**. Most
//! states do not regulate STR conversion at state level;
//! some states have preempted local STR regulation (FL, AZ,
//! TX partial); some have permitted it (NY, CA, CO).
//!
//! Citations: NYC Local Law 18 of 2022 (eff. September 5,
//! 2023); NYC Admin. Code § 26-3001 et seq.; SF Admin. Code
//! Chapter 41A (Ordinance 218-14); LAMC § 12.22 A.32 (LA
//! Home-Sharing Ordinance, eff. November 1, 2019).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYorkCity,
    SanFrancisco,
    LosAngeles,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShortTermRentalConversionInput {
    pub regime: Regime,
    /// Whether host has registered with the local authority.
    pub host_registered: bool,
    /// Whether the unit is the host's primary residence (SF
    /// + LA requirement).
    pub primary_residence: bool,
    /// Whether the host is the permanent occupant AND
    /// present during the stay (NYC Local Law 18
    /// requirement).
    pub host_present_during_stay: bool,
    /// Number of nights per year unit is rented unhosted (no
    /// host on premises).
    pub unhosted_nights_per_year: u32,
    /// Number of nights per year host occupies the unit (for
    /// SF primary residence requirement).
    pub host_occupancy_nights_per_year: u32,
    /// Whether stay is shorter than 30 nights (NYC trigger).
    pub stay_under_30_nights: bool,
    /// Whether host has LA Extended Home-Sharing permit
    /// (allows 240 vs 120 unhosted days).
    pub la_extended_permit: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ShortTermRentalConversionResult {
    pub conversion_lawful: bool,
    pub registration_required: bool,
    pub max_unhosted_nights: u32,
    pub max_penalty_per_violation_cents: i64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &ShortTermRentalConversionInput) -> ShortTermRentalConversionResult {
    match input.regime {
        Regime::NewYorkCity => check_nyc(input),
        Regime::SanFrancisco => check_sf(input),
        Regime::LosAngeles => check_la(input),
        Regime::Default => check_default(input),
    }
}

fn check_nyc(input: &ShortTermRentalConversionInput) -> ShortTermRentalConversionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC Local Law 18 of 2022 (effective September 5, 2023) — all hosts must REGISTER with Office of Special Enforcement (OSE) before listing; applies to stays under 30 nights"
            .to_string(),
        "NYC Local Law 18 — booking platforms must verify valid registration before processing reservations; penalties up to $5,000 per violation for hosts AND platforms"
            .to_string(),
        "NYC Local Law 18 — host must be permanent occupant of the unit AND PRESENT during the stay; OSE has denied 4,300+ non-compliant applications; approved 3,000+"
            .to_string(),
    ];

    if input.stay_under_30_nights && !input.host_registered {
        violations.push(
            "NYC Local Law 18 — host has not registered with Office of Special Enforcement (OSE) before listing stay under 30 nights".to_string(),
        );
    }

    if input.stay_under_30_nights && !input.host_present_during_stay {
        violations.push(
            "NYC Local Law 18 — NYC requires host to be PRESENT during stay (host-and-guest model only); unhosted stays under 30 nights are unlawful".to_string(),
        );
    }

    ShortTermRentalConversionResult {
        conversion_lawful: violations.is_empty(),
        registration_required: input.stay_under_30_nights,
        max_unhosted_nights: 0,
        max_penalty_per_violation_cents: 500_000,
        violations,
        citation: "NYC Local Law 18 of 2022; NYC Admin. Code § 26-3001 et seq.",
        notes,
    }
}

fn check_sf(input: &ShortTermRentalConversionInput) -> ShortTermRentalConversionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "SF Admin. Code Chapter 41A (Ordinance 218-14) — host must REGISTER and obtain Business Registration Certificate before STR hosting"
            .to_string(),
        "SF Admin. Code Chapter 41A — primary residence requirement: 270 nights/year occupancy; cap of 90 unhosted nights per year; penalties up to $1,000 per day for unregistered hosting"
            .to_string(),
    ];

    if !input.host_registered {
        violations.push(
            "SF Admin. Code Chapter 41A — host has not obtained required Business Registration Certificate before STR hosting".to_string(),
        );
    }

    if !input.primary_residence || input.host_occupancy_nights_per_year < 270 {
        violations.push(format!(
            "SF Admin. Code Chapter 41A — primary residence requirement not satisfied: 270 nights/year occupancy required; host occupied {} nights",
            input.host_occupancy_nights_per_year
        ));
    }

    if input.unhosted_nights_per_year > 90 {
        violations.push(format!(
            "SF Admin. Code Chapter 41A — unhosted nights cap of 90 per year exceeded ({} nights)",
            input.unhosted_nights_per_year
        ));
    }

    ShortTermRentalConversionResult {
        conversion_lawful: violations.is_empty(),
        registration_required: true,
        max_unhosted_nights: 90,
        max_penalty_per_violation_cents: 100_000,
        violations,
        citation: "SF Admin. Code Chapter 41A (Ordinance 218-14)",
        notes,
    }
}

fn check_la(input: &ShortTermRentalConversionInput) -> ShortTermRentalConversionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "LAMC § 12.22 A.32 (LA Home-Sharing Ordinance, effective November 1, 2019) — host must REGISTER with LADBS Home-Sharing Program before STR hosting"
            .to_string(),
        "LAMC § 12.22 A.32 — primary residence requirement; cap of 120 days/year unhosted by default; cap extended to 240 days/year with Extended Home-Sharing permit"
            .to_string(),
    ];

    let max_unhosted_la = if input.la_extended_permit { 240 } else { 120 };

    if !input.host_registered {
        violations.push(
            "LAMC § 12.22 A.32 — host has not registered with LADBS Home-Sharing Program".to_string(),
        );
    }

    if !input.primary_residence {
        violations.push(
            "LAMC § 12.22 A.32 — primary residence requirement not satisfied; unit must be host's primary residence".to_string(),
        );
    }

    if input.unhosted_nights_per_year > max_unhosted_la {
        violations.push(format!(
            "LAMC § 12.22 A.32 — unhosted nights cap of {} per year exceeded ({} nights; extended permit: {})",
            max_unhosted_la, input.unhosted_nights_per_year, input.la_extended_permit
        ));
    }

    ShortTermRentalConversionResult {
        conversion_lawful: violations.is_empty(),
        registration_required: true,
        max_unhosted_nights: max_unhosted_la,
        max_penalty_per_violation_cents: 0,
        violations,
        citation: "LAMC § 12.22 A.32 (LA Home-Sharing Ordinance)",
        notes,
    }
}

fn check_default(_input: &ShortTermRentalConversionInput) -> ShortTermRentalConversionResult {
    let notes: Vec<String> = vec![
        "default rule — locality-controlled at state level; most states do not regulate STR conversion at state level"
            .to_string(),
        "default rule — some states have preempted local STR regulation (FL, AZ, partial TX); some have permitted it (NY, CA, CO); verify state preemption status before relying on default"
            .to_string(),
    ];

    ShortTermRentalConversionResult {
        conversion_lawful: true,
        registration_required: false,
        max_unhosted_nights: 0,
        max_penalty_per_violation_cents: 0,
        violations: Vec::new(),
        citation: "no statewide statute; locality-controlled; FL/AZ preempt local; NY/CA/CO permit local",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nyc_compliant() -> ShortTermRentalConversionInput {
        ShortTermRentalConversionInput {
            regime: Regime::NewYorkCity,
            host_registered: true,
            primary_residence: true,
            host_present_during_stay: true,
            unhosted_nights_per_year: 0,
            host_occupancy_nights_per_year: 365,
            stay_under_30_nights: true,
            la_extended_permit: false,
        }
    }

    fn sf_compliant() -> ShortTermRentalConversionInput {
        ShortTermRentalConversionInput {
            regime: Regime::SanFrancisco,
            host_registered: true,
            primary_residence: true,
            host_present_during_stay: false,
            unhosted_nights_per_year: 60,
            host_occupancy_nights_per_year: 280,
            stay_under_30_nights: true,
            la_extended_permit: false,
        }
    }

    fn la_compliant() -> ShortTermRentalConversionInput {
        ShortTermRentalConversionInput {
            regime: Regime::LosAngeles,
            host_registered: true,
            primary_residence: true,
            host_present_during_stay: false,
            unhosted_nights_per_year: 100,
            host_occupancy_nights_per_year: 365,
            stay_under_30_nights: true,
            la_extended_permit: false,
        }
    }

    fn default_base() -> ShortTermRentalConversionInput {
        let mut i = nyc_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn nyc_registered_host_present_lawful() {
        let r = check(&nyc_compliant());
        assert!(r.conversion_lawful);
        assert!(r.registration_required);
    }

    #[test]
    fn nyc_unregistered_unlawful() {
        let mut i = nyc_compliant();
        i.host_registered = false;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Local Law 18") && v.contains("OSE")));
    }

    #[test]
    fn nyc_host_not_present_unlawful() {
        let mut i = nyc_compliant();
        i.host_present_during_stay = false;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("PRESENT")));
    }

    #[test]
    fn nyc_stay_over_30_nights_no_registration_required() {
        let mut i = nyc_compliant();
        i.stay_under_30_nights = false;
        i.host_registered = false;
        let r = check(&i);
        assert!(r.conversion_lawful);
        assert!(!r.registration_required);
    }

    #[test]
    fn nyc_max_penalty_5000_per_violation() {
        let r = check(&nyc_compliant());
        assert_eq!(r.max_penalty_per_violation_cents, 500_000);
    }

    #[test]
    fn sf_registered_within_caps_lawful() {
        let r = check(&sf_compliant());
        assert!(r.conversion_lawful);
        assert_eq!(r.max_unhosted_nights, 90);
    }

    #[test]
    fn sf_unregistered_unlawful() {
        let mut i = sf_compliant();
        i.host_registered = false;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Chapter 41A") && v.contains("Business Registration")));
    }

    #[test]
    fn sf_below_270_night_primary_residence_violates() {
        let mut i = sf_compliant();
        i.host_occupancy_nights_per_year = 269;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("270 nights/year") && v.contains("269")));
    }

    #[test]
    fn sf_at_270_night_boundary_compliant() {
        let mut i = sf_compliant();
        i.host_occupancy_nights_per_year = 270;
        let r = check(&i);
        assert!(r.conversion_lawful);
    }

    #[test]
    fn sf_over_90_unhosted_nights_violates() {
        let mut i = sf_compliant();
        i.unhosted_nights_per_year = 91;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("90 per year") && v.contains("91")));
    }

    #[test]
    fn sf_at_90_unhosted_nights_boundary_compliant() {
        let mut i = sf_compliant();
        i.unhosted_nights_per_year = 90;
        let r = check(&i);
        assert!(r.conversion_lawful);
    }

    #[test]
    fn sf_max_penalty_1000_per_day() {
        let r = check(&sf_compliant());
        assert_eq!(r.max_penalty_per_violation_cents, 100_000);
    }

    #[test]
    fn la_registered_within_default_cap_lawful() {
        let r = check(&la_compliant());
        assert!(r.conversion_lawful);
        assert_eq!(r.max_unhosted_nights, 120);
    }

    #[test]
    fn la_unregistered_unlawful() {
        let mut i = la_compliant();
        i.host_registered = false;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 12.22 A.32") && v.contains("LADBS")));
    }

    #[test]
    fn la_at_120_default_boundary_compliant() {
        let mut i = la_compliant();
        i.unhosted_nights_per_year = 120;
        let r = check(&i);
        assert!(r.conversion_lawful);
    }

    #[test]
    fn la_121_default_violates() {
        let mut i = la_compliant();
        i.unhosted_nights_per_year = 121;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("120 per year") && v.contains("121")));
    }

    #[test]
    fn la_extended_permit_240_day_cap() {
        let mut i = la_compliant();
        i.la_extended_permit = true;
        i.unhosted_nights_per_year = 240;
        let r = check(&i);
        assert!(r.conversion_lawful);
        assert_eq!(r.max_unhosted_nights, 240);
    }

    #[test]
    fn la_extended_permit_241_violates() {
        let mut i = la_compliant();
        i.la_extended_permit = true;
        i.unhosted_nights_per_year = 241;
        let r = check(&i);
        assert!(!r.conversion_lawful);
    }

    #[test]
    fn la_not_primary_residence_violates() {
        let mut i = la_compliant();
        i.primary_residence = false;
        let r = check(&i);
        assert!(!r.conversion_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("primary residence")));
    }

    #[test]
    fn default_no_regulation_lawful() {
        let r = check(&default_base());
        assert!(r.conversion_lawful);
        assert!(!r.registration_required);
    }

    #[test]
    fn default_unregistered_still_lawful() {
        let mut i = default_base();
        i.host_registered = false;
        i.primary_residence = false;
        i.unhosted_nights_per_year = 365;
        let r = check(&i);
        assert!(r.conversion_lawful);
    }

    #[test]
    fn nyc_citation_pins_local_law_18() {
        let r = check(&nyc_compliant());
        assert!(r.citation.contains("Local Law 18 of 2022"));
        assert!(r.citation.contains("§ 26-3001"));
    }

    #[test]
    fn sf_citation_pins_chapter_41a() {
        let r = check(&sf_compliant());
        assert!(r.citation.contains("Chapter 41A"));
        assert!(r.citation.contains("Ordinance 218-14"));
    }

    #[test]
    fn la_citation_pins_lamc_12_22_a_32() {
        let r = check(&la_compliant());
        assert!(r.citation.contains("§ 12.22 A.32"));
        assert!(r.citation.contains("Home-Sharing Ordinance"));
    }

    #[test]
    fn default_citation_pins_locality_preemption() {
        let r = check(&default_base());
        assert!(r.citation.contains("no statewide statute"));
        assert!(r.citation.contains("FL/AZ preempt"));
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [
            Regime::NewYorkCity,
            Regime::SanFrancisco,
            Regime::LosAngeles,
            Regime::Default,
        ] {
            let mut i = nyc_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nyc_uniquely_requires_host_present_invariant() {
        let mut i_nyc = nyc_compliant();
        i_nyc.host_present_during_stay = false;
        let r_nyc = check(&i_nyc);
        assert!(!r_nyc.conversion_lawful);

        let mut i_sf = sf_compliant();
        i_sf.host_present_during_stay = false;
        let r_sf = check(&i_sf);
        assert!(r_sf.conversion_lawful);
    }

    #[test]
    fn sf_uniquely_requires_270_night_occupancy_invariant() {
        let mut i_sf = sf_compliant();
        i_sf.host_occupancy_nights_per_year = 200;
        let r_sf = check(&i_sf);
        assert!(!r_sf.conversion_lawful);

        let mut i_la = la_compliant();
        i_la.host_occupancy_nights_per_year = 200;
        let r_la = check(&i_la);
        assert!(r_la.conversion_lawful);
    }

    #[test]
    fn la_extended_permit_doubles_unhosted_cap_invariant() {
        let mut i_default = la_compliant();
        i_default.la_extended_permit = false;
        let r_default = check(&i_default);
        assert_eq!(r_default.max_unhosted_nights, 120);

        let mut i_extended = la_compliant();
        i_extended.la_extended_permit = true;
        let r_extended = check(&i_extended);
        assert_eq!(r_extended.max_unhosted_nights, 240);
    }

    #[test]
    fn regime_unhosted_cap_truth_table() {
        let r_nyc = check(&nyc_compliant());
        assert_eq!(r_nyc.max_unhosted_nights, 0);

        let r_sf = check(&sf_compliant());
        assert_eq!(r_sf.max_unhosted_nights, 90);

        let r_la = check(&la_compliant());
        assert_eq!(r_la.max_unhosted_nights, 120);

        let r_default = check(&default_base());
        assert_eq!(r_default.max_unhosted_nights, 0);
    }
}
