//! Multi-jurisdictional tenant noise nuisance enforcement
//! framework. Trader-landlord critical because (1) noise
//! complaints are among the most common multifamily
//! tenant grievances; (2) failure to address noise
//! nuisance breaches the warranty of QUIET ENJOYMENT and
//! exposes landlord to rent abatement, constructive
//! eviction claims, lease termination, and damages;
//! (3) NYC + Chicago + Boston impose absolute decibel and
//! quiet-hour standards regardless of tenant tolerance;
//! (4) landlord has DUTY TO ABATE known noise nuisances
//! created by other tenants.
//!
//! Companion to landlord_self_help_eviction_prohibition,
//! tenant_rent_escrow_withholding (habitability),
//! landlord_emergency_entry_notice,
//! rental_window_guard_installation.
//!
//! **California Civ. Code § 1941.4 + Common-Law Warranty
//! of Quiet Enjoyment**:
//! 1. § 1941.4 — landlord has obligation to maintain
//!    premises in fit condition (interaction with warranty
//!    of habitability).
//! 2. Andrews v. Mobile Aire Estates, 125 Cal. App. 4th
//!    578 (2005) — covenant of quiet enjoyment IMPLIED in
//!    every lease.
//! 3. Tenant remedies: constructive eviction + lease
//!    termination + rent withholding + damages + damages
//!    against landlord and offending tenant.
//!
//! **New York City Admin. Code § 24-218 + NYC Noise
//! Code**:
//! 1. § 24-218 — no person shall make unreasonable noise.
//! 2. Nighttime quiet hours: **10:00 p.m. to 7:00 a.m.**;
//!    sound 7 dB(A) or more above ambient prohibited
//!    (measured 15+ feet from source on public right-of-
//!    way OR at receiving property).
//! 3. Daytime standard: sound 10 dB(A) or more above
//!    ambient prohibited.
//! 4. Residential apartment-to-apartment standard:
//!    AMPLIFIED SOUND audible inside adjacent dwelling
//!    unit with windows closed = VIOLATION at any hour.
//! 5. Residential nighttime ambient typically 35-45 dB.
//! 6. Enforcement: NYC DEP (most categories) + NYPD
//!    (residential); 311 complaint pathway.
//!
//! NYC also incorporates noise into RPL § 235-b warranty
//! of habitability (Park West Management Corp. v.
//! Mitchell, 47 N.Y.2d 316 (1979)) — chronic noise can
//! be a habitability breach.
//!
//! **Chicago Municipal Code § 8-32**:
//! 1. Quiet hours: **10:00 p.m. to 8:00 a.m.** weekdays;
//!    10:00 p.m. to 10:00 a.m. weekends.
//! 2. Plainly audible sound 75 feet from source =
//!    presumptive violation.
//! 3. Amplified music in residential area prohibited
//!    between 9:00 p.m. and 8:00 a.m.
//! 4. Enforcement: Chicago Police Department + Department
//!    of Buildings; municipal fines.
//!
//! **Massachusetts G.L. + Common-Law Quiet Enjoyment**:
//! 1. Mass. G.L. c. 186 § 14 — willful interference with
//!    tenant's quiet enjoyment is a CRIME (small fine +
//!    treble damages + attorney fees + costs).
//! 2. Berman & Sons v. Jefferson, 379 Mass. 196 (1979) —
//!    constructive eviction available for substantial
//!    breach of quiet enjoyment.
//!
//! **Default — Common-Law Nuisance + Implied Covenant of
//! Quiet Enjoyment**:
//! 1. Common-law nuisance (private + public);
//! 2. Implied covenant of quiet enjoyment in every lease;
//! 3. Local quiet-hour ordinances vary by jurisdiction.
//!
//! Citations: Cal. Civ. Code § 1941.4; Andrews v. Mobile
//! Aire Estates, 125 Cal. App. 4th 578 (2005); NYC Admin.
//! Code § 24-218; NYC Noise Code (Chapter 2 of Title 24);
//! N.Y. Real Prop. Law § 235-b; Park West Management
//! Corp. v. Mitchell, 47 N.Y.2d 316 (1979); Chicago
//! Municipal Code § 8-32; Mass. G.L. c. 186 § 14; Berman &
//! Sons v. Jefferson, 379 Mass. 196 (1979).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYorkCity,
    Chicago,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoiseTimeWindow {
    /// Daytime hours.
    Daytime,
    /// Nighttime hours (varies by jurisdiction).
    Nighttime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantNoiseNuisanceEnforcementInput {
    pub jurisdiction: Jurisdiction,
    pub time_window: NoiseTimeWindow,
    /// Hour of measurement (0-23 in 24-hour format).
    pub hour_of_day: u32,
    /// Measured noise level in decibels (dB(A)).
    pub measured_db: u32,
    /// Ambient noise level in decibels (dB(A)).
    pub ambient_db: u32,
    /// Whether noise is amplified (music, TV, speakers).
    pub amplified_noise: bool,
    /// Whether amplified sound audible in adjacent
    /// dwelling unit with windows closed (NYC apartment-
    /// to-apartment violation regardless of hour).
    pub audible_in_adjacent_unit_windows_closed: bool,
    /// Whether plainly audible 75 feet from source
    /// (Chicago presumptive violation).
    pub plainly_audible_75_feet: bool,
    /// Whether landlord received tenant complaint.
    pub landlord_received_complaint: bool,
    /// Whether landlord took reasonable abatement action.
    pub landlord_abated_nuisance: bool,
    /// Days since complaint without landlord action.
    pub days_since_complaint_without_action: u32,
    /// Whether weekend (Chicago has different quiet hours).
    pub is_weekend: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantNoiseNuisanceEnforcementResult {
    pub jurisdiction: Jurisdiction,
    pub noise_violation_exists: bool,
    pub during_quiet_hours: bool,
    pub landlord_liability_engaged: bool,
    pub tenant_remedy_available: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &TenantNoiseNuisanceEnforcementInput,
) -> TenantNoiseNuisanceEnforcementResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let db_above_ambient = input.measured_db.saturating_sub(input.ambient_db);

    let (during_quiet_hours, noise_violation_exists) = match input.jurisdiction {
        Jurisdiction::NewYorkCity => {
            let quiet = input.hour_of_day >= 22 || input.hour_of_day < 7;
            let threshold = if quiet { 7 } else { 10 };
            let db_violation = db_above_ambient >= threshold;
            let apartment_violation = input.amplified_noise
                && input.audible_in_adjacent_unit_windows_closed;
            let violation = db_violation || apartment_violation;

            if db_violation {
                failure_reasons.push(format!(
                    "NYC Admin. Code § 24-218 — noise of {} dB(A) above ambient violates the {}-dB threshold for {} hours (10:00 p.m. - 7:00 a.m. nighttime requires 7 dB; daytime requires 10 dB above ambient measured 15+ feet from source on public right-of-way OR at receiving property)",
                    db_above_ambient,
                    threshold,
                    if quiet { "nighttime" } else { "daytime" }
                ));
            }
            if apartment_violation {
                failure_reasons.push(
                    "NYC Admin. Code § 24-218 — AMPLIFIED SOUND audible inside adjacent dwelling unit with windows closed = VIOLATION AT ANY HOUR (no time-window restriction for apartment-to-apartment amplified-source violations)".to_string(),
                );
            }
            (quiet, violation)
        }
        Jurisdiction::Chicago => {
            let quiet = if input.is_weekend {
                input.hour_of_day >= 22 || input.hour_of_day < 10
            } else {
                input.hour_of_day >= 22 || input.hour_of_day < 8
            };
            let amplified_violation = input.amplified_noise
                && (input.hour_of_day >= 21 || input.hour_of_day < 8);
            let presumptive_violation = input.plainly_audible_75_feet;
            let violation = quiet && (amplified_violation || presumptive_violation);

            if presumptive_violation {
                failure_reasons.push(
                    "Chicago Municipal Code § 8-32 — sound PLAINLY AUDIBLE 75 FEET from source is presumptive violation; enforcement by Chicago Police Department + Department of Buildings".to_string(),
                );
            }
            if amplified_violation {
                failure_reasons.push(
                    "Chicago Municipal Code § 8-32 — AMPLIFIED MUSIC in residential area prohibited between 9:00 p.m. and 8:00 a.m.".to_string(),
                );
            }
            (quiet, violation)
        }
        Jurisdiction::California => {
            let quiet = input.hour_of_day >= 22 || input.hour_of_day < 7;
            let violation = db_above_ambient >= 10 || (input.amplified_noise && quiet);

            if violation {
                failure_reasons.push(
                    "Cal. Civ. Code § 1941.4 + Andrews v. Mobile Aire Estates, 125 Cal. App. 4th 578 (2005) — covenant of quiet enjoyment IMPLIED in every lease; tenant remedies include constructive eviction + lease termination + rent withholding + damages against landlord and offending tenant".to_string(),
                );
            }
            (quiet, violation)
        }
        Jurisdiction::Massachusetts => {
            let quiet = input.hour_of_day >= 22 || input.hour_of_day < 7;
            let violation = (input.amplified_noise && quiet) || db_above_ambient >= 15;

            if violation && input.days_since_complaint_without_action > 30 {
                failure_reasons.push(
                    "Mass. G.L. c. 186 § 14 — WILLFUL INTERFERENCE with tenant's quiet enjoyment is a CRIME (small fine + TREBLE DAMAGES + attorney fees + costs); Berman & Sons v. Jefferson, 379 Mass. 196 (1979) — constructive eviction available for substantial breach".to_string(),
                );
            }
            (quiet, violation)
        }
        Jurisdiction::Default => {
            let quiet = input.hour_of_day >= 22 || input.hour_of_day < 7;
            let violation = (input.amplified_noise && quiet) || db_above_ambient >= 15;
            if violation {
                failure_reasons.push(
                    "Default — common-law NUISANCE (private + public) + IMPLIED COVENANT of quiet enjoyment in every lease; local quiet-hour ordinances vary by jurisdiction".to_string(),
                );
            }
            (quiet, violation)
        }
    };

    let landlord_liability_engaged = noise_violation_exists
        && input.landlord_received_complaint
        && !input.landlord_abated_nuisance;

    let tenant_remedy_available = landlord_liability_engaged
        || (noise_violation_exists && input.days_since_complaint_without_action > 14);

    if landlord_liability_engaged {
        failure_reasons.push(format!(
            "LANDLORD DUTY TO ABATE — landlord received complaint of noise nuisance but FAILED to take reasonable abatement action; tenant remedies escalate: rent abatement + warranty of habitability claim + lease termination + constructive eviction + damages; {} days since complaint without action",
            input.days_since_complaint_without_action
        ));
    }

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1941.4 + Andrews v. Mobile Aire Estates, 125 Cal. App. 4th 578 (2005) — covenant of quiet enjoyment IMPLIED in every lease; tenant remedies: constructive eviction + lease termination + rent withholding + damages against landlord AND offending tenant".to_string(),
        "NYC Admin. Code § 24-218 — no person shall make UNREASONABLE NOISE; nighttime (10pm-7am) threshold = 7 dB(A) above ambient; daytime threshold = 10 dB(A) above ambient; residential ambient typically 35-45 dB".to_string(),
        "NYC Admin. Code § 24-218 (apartment-to-apartment) — AMPLIFIED SOUND audible inside adjacent dwelling unit with windows closed = VIOLATION AT ANY HOUR (no time-window restriction)".to_string(),
        "NYC noise enforcement — DEP (most categories) + NYPD (residential); 311 complaint pathway; also incorporates noise into RPL § 235-b warranty of habitability per Park West Management Corp. v. Mitchell, 47 N.Y.2d 316 (1979) — chronic noise can be habitability breach".to_string(),
        "Chicago Municipal Code § 8-32 — quiet hours: 10:00 p.m. to 8:00 a.m. weekdays / 10:00 p.m. to 10:00 a.m. weekends; sound PLAINLY AUDIBLE 75 FEET from source is presumptive violation; AMPLIFIED MUSIC in residential prohibited 9pm-8am".to_string(),
        "Chicago noise enforcement — Chicago Police Department + Department of Buildings; municipal fines + repeat-violator surcharges".to_string(),
        "Mass. G.L. c. 186 § 14 — WILLFUL INTERFERENCE with tenant's quiet enjoyment is a CRIME with small fine + TREBLE DAMAGES + attorney fees + costs; Berman & Sons v. Jefferson, 379 Mass. 196 (1979) — constructive eviction available for substantial breach".to_string(),
        "Default — common-law NUISANCE (private + public) + IMPLIED COVENANT of quiet enjoyment in every lease; local quiet-hour ordinances vary by jurisdiction (Los Angeles 10pm-7am; San Francisco 10pm-7am; Seattle 10pm-7am; Portland 10pm-7am; Denver 10pm-7am)".to_string(),
        "Landlord duty to abate — landlord has DUTY to investigate noise complaints + take reasonable abatement action against offending tenant (lease violation notice + cure-or-quit + eviction action); failure exposes landlord to (1) tenant rent abatement; (2) warranty of habitability claim; (3) lease termination; (4) constructive eviction; (5) damages action".to_string(),
        "Trader-landlord critical because (1) noise complaints among most common multifamily grievances; (2) failure to address breaches warranty of QUIET ENJOYMENT and exposes landlord to rent abatement + constructive eviction + lease termination + damages; (3) NYC + Chicago + Boston impose absolute decibel + quiet-hour standards regardless of tenant tolerance; (4) landlord DUTY TO ABATE known nuisances created by other tenants".to_string(),
        "Cross-jurisdictional architecture: California uses § 1941.4 + Andrews IMPLIED COVENANT; NYC uses § 24-218 DECIBEL + AMBIENT-DELTA + APARTMENT-AUDIBILITY; Chicago uses § 8-32 QUIET HOURS + 75-FOOT-AUDIBLE; Massachusetts uses § 14 CRIMINAL + TREBLE DAMAGES + Berman & Sons; Default uses common-law nuisance + implied covenant".to_string(),
    ];

    TenantNoiseNuisanceEnforcementResult {
        jurisdiction: input.jurisdiction,
        noise_violation_exists,
        during_quiet_hours,
        landlord_liability_engaged,
        tenant_remedy_available,
        failure_reasons,
        citation: "Cal. Civ. Code § 1941.4; Andrews v. Mobile Aire Estates, 125 Cal. App. 4th 578 (2005); NYC Admin. Code § 24-218; NYC Noise Code (Chapter 2 of Title 24); N.Y. Real Prop. Law § 235-b; Park West Management Corp. v. Mitchell, 47 N.Y.2d 316 (1979); Chicago Municipal Code § 8-32; Mass. G.L. c. 186 § 14; Berman & Sons v. Jefferson, 379 Mass. 196 (1979)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nyc_baseline() -> TenantNoiseNuisanceEnforcementInput {
        TenantNoiseNuisanceEnforcementInput {
            jurisdiction: Jurisdiction::NewYorkCity,
            time_window: NoiseTimeWindow::Nighttime,
            hour_of_day: 23,
            measured_db: 50,
            ambient_db: 40,
            amplified_noise: false,
            audible_in_adjacent_unit_windows_closed: false,
            plainly_audible_75_feet: false,
            landlord_received_complaint: false,
            landlord_abated_nuisance: false,
            days_since_complaint_without_action: 0,
            is_weekend: false,
        }
    }

    #[test]
    fn nyc_10_db_above_ambient_nighttime_violation() {
        let r = check(&nyc_baseline());
        assert!(r.noise_violation_exists);
        assert!(r.during_quiet_hours);
    }

    #[test]
    fn nyc_5_db_above_ambient_nighttime_no_violation() {
        let mut i = nyc_baseline();
        i.measured_db = 45;
        let r = check(&i);
        assert!(!r.noise_violation_exists);
    }

    #[test]
    fn nyc_amplified_adjacent_unit_any_hour_violation() {
        let mut i = nyc_baseline();
        i.hour_of_day = 14;
        i.measured_db = 40;
        i.amplified_noise = true;
        i.audible_in_adjacent_unit_windows_closed = true;
        let r = check(&i);
        assert!(r.noise_violation_exists);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 24-218")
            && f.contains("AMPLIFIED SOUND audible inside adjacent dwelling")
            && f.contains("VIOLATION AT ANY HOUR")));
    }

    #[test]
    fn nyc_daytime_10_db_threshold() {
        let mut i = nyc_baseline();
        i.hour_of_day = 14;
        i.measured_db = 50;
        i.ambient_db = 40;
        let r = check(&i);
        assert!(r.noise_violation_exists);
        assert!(!r.during_quiet_hours);
    }

    #[test]
    fn nyc_daytime_9_db_no_violation() {
        let mut i = nyc_baseline();
        i.hour_of_day = 14;
        i.measured_db = 49;
        i.ambient_db = 40;
        let r = check(&i);
        assert!(!r.noise_violation_exists);
    }

    #[test]
    fn nyc_quiet_hours_7am_boundary() {
        let mut i = nyc_baseline();
        i.hour_of_day = 7;
        let r = check(&i);
        assert!(!r.during_quiet_hours);
    }

    #[test]
    fn nyc_quiet_hours_6am_engages() {
        let mut i = nyc_baseline();
        i.hour_of_day = 6;
        let r = check(&i);
        assert!(r.during_quiet_hours);
    }

    #[test]
    fn chicago_plainly_audible_75_feet_violation() {
        let mut i = nyc_baseline();
        i.jurisdiction = Jurisdiction::Chicago;
        i.hour_of_day = 23;
        i.plainly_audible_75_feet = true;
        let r = check(&i);
        assert!(r.noise_violation_exists);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 8-32")
            && f.contains("PLAINLY AUDIBLE 75 FEET")));
    }

    #[test]
    fn chicago_weekday_quiet_hours_10pm_8am() {
        let mut i = nyc_baseline();
        i.jurisdiction = Jurisdiction::Chicago;
        i.hour_of_day = 7;
        i.is_weekend = false;
        let r = check(&i);
        assert!(r.during_quiet_hours);
    }

    #[test]
    fn chicago_weekend_quiet_hours_10pm_10am() {
        let mut i = nyc_baseline();
        i.jurisdiction = Jurisdiction::Chicago;
        i.hour_of_day = 9;
        i.is_weekend = true;
        let r = check(&i);
        assert!(r.during_quiet_hours);
    }

    #[test]
    fn chicago_amplified_music_9pm_8am_violation() {
        let mut i = nyc_baseline();
        i.jurisdiction = Jurisdiction::Chicago;
        i.hour_of_day = 22;
        i.amplified_noise = true;
        let r = check(&i);
        assert!(r.noise_violation_exists);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 8-32")
            && f.contains("AMPLIFIED MUSIC")
            && f.contains("9:00 p.m. and 8:00 a.m.")));
    }

    #[test]
    fn california_implied_covenant_engagement() {
        let mut i = nyc_baseline();
        i.jurisdiction = Jurisdiction::California;
        let r = check(&i);
        assert!(r.noise_violation_exists);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1941.4")
            && f.contains("Andrews v. Mobile Aire Estates")
            && f.contains("125 Cal. App. 4th 578 (2005)")));
    }

    #[test]
    fn massachusetts_treble_damages_after_30_days() {
        let mut i = nyc_baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.measured_db = 60;
        i.ambient_db = 40;
        i.days_since_complaint_without_action = 31;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("c. 186 § 14")
            && f.contains("CRIME")
            && f.contains("TREBLE DAMAGES")
            && f.contains("Berman & Sons v. Jefferson")));
    }

    #[test]
    fn default_jurisdiction_common_law_nuisance() {
        let mut i = nyc_baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.measured_db = 60;
        i.ambient_db = 40;
        let r = check(&i);
        assert!(r.noise_violation_exists);
        assert!(r.failure_reasons.iter().any(|f| f.contains("Default")
            && f.contains("common-law NUISANCE")
            && f.contains("IMPLIED COVENANT")));
    }

    #[test]
    fn landlord_failed_to_abate_complaint_liability_engaged() {
        let mut i = nyc_baseline();
        i.landlord_received_complaint = true;
        i.landlord_abated_nuisance = false;
        i.days_since_complaint_without_action = 20;
        let r = check(&i);
        assert!(r.landlord_liability_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("LANDLORD DUTY TO ABATE")
            && f.contains("FAILED to take reasonable abatement")));
    }

    #[test]
    fn landlord_abated_no_liability() {
        let mut i = nyc_baseline();
        i.landlord_received_complaint = true;
        i.landlord_abated_nuisance = true;
        let r = check(&i);
        assert!(!r.landlord_liability_engaged);
    }

    #[test]
    fn no_complaint_no_landlord_liability() {
        let mut i = nyc_baseline();
        i.landlord_received_complaint = false;
        let r = check(&i);
        assert!(!r.landlord_liability_engaged);
    }

    #[test]
    fn tenant_remedy_available_after_14_days_without_action() {
        let mut i = nyc_baseline();
        i.landlord_received_complaint = true;
        i.days_since_complaint_without_action = 15;
        let r = check(&i);
        assert!(r.tenant_remedy_available);
    }

    #[test]
    fn jurisdiction_truth_table_five_cells() {
        for jur in [
            Jurisdiction::California,
            Jurisdiction::NewYorkCity,
            Jurisdiction::Chicago,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = nyc_baseline();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn chicago_uniquely_75_foot_audible_invariant() {
        let mut chi = nyc_baseline();
        chi.jurisdiction = Jurisdiction::Chicago;
        chi.plainly_audible_75_feet = true;
        chi.hour_of_day = 23;
        let r_chi = check(&chi);
        assert!(r_chi.failure_reasons.iter().any(|f| f.contains("§ 8-32")
            && f.contains("PLAINLY AUDIBLE 75 FEET")));

        for jur in [
            Jurisdiction::California,
            Jurisdiction::NewYorkCity,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = nyc_baseline();
            i.jurisdiction = jur;
            i.plainly_audible_75_feet = true;
            let r = check(&i);
            assert!(
                !r.failure_reasons.iter().any(|f| f.contains("75 FEET")),
                "jur={:?}",
                jur
            );
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&nyc_baseline());
        assert!(r.citation.contains("Cal. Civ. Code § 1941.4"));
        assert!(r.citation.contains("Andrews v. Mobile Aire Estates, 125 Cal. App. 4th 578 (2005)"));
        assert!(r.citation.contains("NYC Admin. Code § 24-218"));
        assert!(r.citation.contains("NYC Noise Code"));
        assert!(r.citation.contains("N.Y. Real Prop. Law § 235-b"));
        assert!(r.citation.contains("Park West Management Corp. v. Mitchell, 47 N.Y.2d 316 (1979)"));
        assert!(r.citation.contains("Chicago Municipal Code § 8-32"));
        assert!(r.citation.contains("Mass. G.L. c. 186 § 14"));
        assert!(r.citation.contains("Berman & Sons v. Jefferson, 379 Mass. 196 (1979)"));
    }

    #[test]
    fn note_pins_ca_andrews_implied_covenant() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 1941.4")
            && n.contains("Andrews v. Mobile Aire Estates")
            && n.contains("125 Cal. App. 4th 578 (2005)")
            && n.contains("IMPLIED")));
    }

    #[test]
    fn note_pins_nyc_general_db_ambient_thresholds() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 24-218")
            && n.contains("nighttime (10pm-7am)")
            && n.contains("7 dB(A)")
            && n.contains("10 dB(A)")
            && n.contains("ambient typically 35-45 dB")));
    }

    #[test]
    fn note_pins_nyc_apartment_to_apartment_amplified() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 24-218 (apartment-to-apartment)")
            && n.contains("AMPLIFIED SOUND")
            && n.contains("VIOLATION AT ANY HOUR")));
    }

    #[test]
    fn note_pins_nyc_dep_311_enforcement() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("DEP")
            && n.contains("NYPD")
            && n.contains("311 complaint pathway")
            && n.contains("Park West Management Corp. v. Mitchell")));
    }

    #[test]
    fn note_pins_chicago_quiet_hours_dual_schedule() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("§ 8-32")
            && n.contains("10:00 p.m. to 8:00 a.m. weekdays")
            && n.contains("10:00 p.m. to 10:00 a.m. weekends")
            && n.contains("PLAINLY AUDIBLE 75 FEET")
            && n.contains("AMPLIFIED MUSIC")));
    }

    #[test]
    fn note_pins_chicago_enforcement() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("Chicago Police Department")
            && n.contains("Department of Buildings")));
    }

    #[test]
    fn note_pins_ma_c186_14_criminal_treble() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("c. 186 § 14")
            && n.contains("WILLFUL INTERFERENCE")
            && n.contains("CRIME")
            && n.contains("TREBLE DAMAGES")
            && n.contains("Berman & Sons v. Jefferson")));
    }

    #[test]
    fn note_pins_default_common_law_nuisance() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("Default")
            && n.contains("common-law NUISANCE")
            && n.contains("IMPLIED COVENANT")
            && n.contains("Los Angeles")
            && n.contains("San Francisco")));
    }

    #[test]
    fn note_pins_landlord_duty_to_abate() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("Landlord duty to abate")
            && n.contains("DUTY")
            && n.contains("cure-or-quit")
            && n.contains("constructive eviction")));
    }

    #[test]
    fn note_pins_trader_landlord_most_common_grievances() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("Trader-landlord critical")
            && n.contains("most common multifamily grievances")
            && n.contains("QUIET ENJOYMENT")
            && n.contains("DUTY TO ABATE")));
    }

    #[test]
    fn note_pins_cross_jurisdictional_architecture() {
        let r = check(&nyc_baseline());
        assert!(r.notes.iter().any(|n| n.contains("Cross-jurisdictional architecture")
            && n.contains("IMPLIED COVENANT")
            && n.contains("DECIBEL + AMBIENT-DELTA")
            && n.contains("75-FOOT-AUDIBLE")
            && n.contains("CRIMINAL + TREBLE DAMAGES")));
    }

    #[test]
    fn multiple_failures_stack_landlord_failed_to_abate() {
        let mut i = nyc_baseline();
        i.landlord_received_complaint = true;
        i.days_since_complaint_without_action = 30;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 2);
    }
}
