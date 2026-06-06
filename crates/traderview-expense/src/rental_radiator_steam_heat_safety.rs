//! Rental radiator + steam heat safety framework — covers landlord
//! obligations under NYC Int 1489-2017 / Local Law 79 of 2018 (radiator
//! covers required for households with children twelve or younger upon
//! written tenant request — 90-day installation window) plus NYC Int
//! 0925-2024 "Ben Z's Law" (biennial radiator inspections in apartments
//! and common areas where children under 6 reside) plus state-specific
//! habitability frameworks.
//!
//! Distinct from sibling `rental_chimney_fireplace_inspection_disclosure`
//! (solid-fuel heating framework), `rental_pellet_stove_disclosure`
//! (iter 499 pellet stove framework), `rental_natural_gas_leak_response`
//! (gas-system leak framework), `rental_oil_tank_replacement_disclosure`
//! (oil-heat fuel-storage framework), `heat_requirements` (general
//! heat-temperature habitability).
//!
//! Trader-landlord critical because (1) NYC steam radiators reach 250°F at
//! peak heating per ASHRAE Handbook; severe contact burns occur in less
//! than 2 seconds at this temperature per Mayo Clinic burn classification;
//! (2) **NYC Int 1489-2017 / Local Law 79 of 2018** amending NYC Admin
//! Code § 27-2076 — radiator covers MUST be installed by landlord within
//! 90 days of WRITTEN tenant request when a child age twelve or younger
//! resides in the unit; cover must completely enclose top + sides + front
//! with grill openings sized to prevent child finger insertion; (3) **NYC
//! Int 0925-2024 "Ben Z's Law"** named after infant Bencel "Ben Z" Yancanay
//! who died December 2020 from steam radiator burns — requires biennial
//! inspection (every 2 years) of radiators in apartments and common areas
//! where children under 6 reside; exempts owner-occupied co-ops + condos;
//! (4) HPD civil penalties up to $500 per violation per HPD penalty
//! schedule; (5) post-installation violation = HAZARD classification under
//! NYC Housing Maintenance Code triggering tenant rent withholding plus
//! repair-and-deduct plus constructive-eviction; (6) NYC trader-landlord
//! steam radiator burn injury litigation routinely exceeds $500K-$2M for
//! permanent scarring + disfigurement of pediatric or elderly tenant.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// NYC framework with Int 1489-2017 / LL 79 of 2018 + Ben Z's Law.
    NewYorkCity,
    /// Boston framework via M.G.L. ch. 186 § 14 quiet enjoyment.
    Boston,
    /// Chicago framework via Chicago RLTO § 5-12-110.
    Chicago,
    /// Default — common-law habitability plus state-specific.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatingSystemType {
    /// Steam radiator (reaches 250°F).
    SteamRadiator,
    /// Hot-water radiator (lower max temperature ~180°F).
    HotWaterRadiator,
    /// Electric baseboard heat.
    ElectricBaseboard,
    /// Forced-air central heat — no radiator.
    ForcedAirCentral,
    /// No radiator-based heating system.
    NoRadiator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantComposition {
    /// Household has child age 12 or younger — Int 1489-2017 cover-request
    /// right attaches.
    HouseholdWithChildUnder12,
    /// Household has child under 6 — Ben Z's Law biennial inspection
    /// requirement attaches.
    HouseholdWithChildUnder6,
    /// Household has elderly or cognitively-impaired tenant — heightened
    /// duty under common-law negligence.
    HouseholdWithElderlyOrImpaired,
    /// Standard adult household — base requirements only.
    StandardAdult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantCoverInstalledOrNoRequest,
    LandlordCoverInstallationOverdue90Day,
    BenZLawBiennialInspectionOverdue,
    CoverMissingGrillOpeningChildSafetySpec,
    SteamBurnInjuryHabitabilityBreach,
    UnregulatedRadiatorWithoutThermostatHazard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub heating_system_type: HeatingSystemType,
    pub tenant_composition: TenantComposition,
    pub tenant_written_request_for_cover_submitted: bool,
    pub days_since_tenant_cover_request: u32,
    pub radiator_cover_installed: bool,
    pub cover_meets_grill_opening_child_safety_spec: bool,
    pub months_since_last_biennial_inspection: u32,
    pub steam_burn_injury_occurred: bool,
    pub radiator_thermostat_or_control_present: bool,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub hpd_civil_penalty_max_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const NYC_RADIATOR_COVER_INSTALL_DAYS: u32 = 90;
pub const NYC_BIENNIAL_INSPECTION_INTERVAL_MONTHS: u32 = 24;
pub const NYC_HPD_PENALTY_MAX_CENTS: u64 = 50_000;
pub const STEAM_RADIATOR_PEAK_TEMP_F: u32 = 250;
pub const INT_1489_2017_LOCAL_LAW_YEAR: i32 = 2018;
pub const BEN_Z_DEATH_DATE: &str = "2020-12";

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.heating_system_type, HeatingSystemType::NoRadiator)
        || matches!(
            input.heating_system_type,
            HeatingSystemType::ForcedAirCentral
        )
    {
        notes.push(
            "No radiator-based heating system on premises — framework inapplicable. \
             Forced-air central heating and ductless mini-split systems do NOT pose \
             radiator-contact burn risk. Confirm via building heating diagram before \
             relying on this status."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            hpd_civil_penalty_max_cents: 0,
            citation: "n/a (no radiator)",
            notes,
        };
    }

    if input.steam_burn_injury_occurred {
        severity = Severity::SteamBurnInjuryHabitabilityBreach;
        actions.push(format!(
            "Steam-burn injury to tenant occurred — immediate habitability breach plus \
             premises-liability exposure. NYC steam radiators reach {}°F at peak heating; \
             severe contact burns occur in < 2 seconds per Mayo Clinic burn classification. \
             Trader-landlord settlement exposure routinely $500K-$2M for permanent \
             scarring / disfigurement of pediatric or elderly tenant. Document injury via \
             medical records; preserve scene photos; notify general liability carrier \
             plus umbrella policy carrier within 24 hours; engage premises-liability \
             counsel. Install cover plus biennial inspection compliance immediately to \
             mitigate ongoing exposure.",
            STEAM_RADIATOR_PEAK_TEMP_F
        ));
    } else if matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && matches!(
            input.tenant_composition,
            TenantComposition::HouseholdWithChildUnder12
                | TenantComposition::HouseholdWithChildUnder6
        )
        && input.tenant_written_request_for_cover_submitted
        && !input.radiator_cover_installed
        && input.days_since_tenant_cover_request > NYC_RADIATOR_COVER_INSTALL_DAYS
    {
        severity = Severity::LandlordCoverInstallationOverdue90Day;
        actions.push(format!(
            "NYC Int 1489-2017 / Local Law 79 of {} (amending NYC Admin Code § 27-2076) \
             violation: tenant submitted written cover request {} days ago, exceeding \
             {}-day installation window. Failure to install cover within statutory window \
             classified as HAZARD under NYC Housing Maintenance Code triggering tenant rent \
             withholding plus repair-and-deduct plus constructive eviction. HPD civil \
             penalty up to ${} per violation. INSTALL COVER WITHIN 7 DAYS to mitigate \
             accruing penalty + tenant remedies; document with installation invoice plus \
             tenant acknowledgment.",
            INT_1489_2017_LOCAL_LAW_YEAR,
            input.days_since_tenant_cover_request,
            NYC_RADIATOR_COVER_INSTALL_DAYS,
            NYC_HPD_PENALTY_MAX_CENTS / 100
        ));
    } else if matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && matches!(
            input.tenant_composition,
            TenantComposition::HouseholdWithChildUnder6
        )
        && input.months_since_last_biennial_inspection > NYC_BIENNIAL_INSPECTION_INTERVAL_MONTHS
    {
        severity = Severity::BenZLawBiennialInspectionOverdue;
        actions.push(format!(
            "NYC Int 0925-2024 'Ben Z's Law' violation: biennial radiator inspection \
             overdue ({} months since last inspection; statutory interval {} months). Law \
             named after infant Bencel 'Ben Z' Yancanay who died {} from steam radiator \
             burns. Requires inspection of radiators in apartments AND common areas where \
             children under 6 reside. Engage licensed inspector immediately; document \
             findings; cure any identified deficiencies within 30-day cure window.",
            input.months_since_last_biennial_inspection,
            NYC_BIENNIAL_INSPECTION_INTERVAL_MONTHS,
            BEN_Z_DEATH_DATE
        ));
    } else if input.radiator_cover_installed
        && !input.cover_meets_grill_opening_child_safety_spec
        && matches!(
            input.tenant_composition,
            TenantComposition::HouseholdWithChildUnder12
                | TenantComposition::HouseholdWithChildUnder6
        )
    {
        severity = Severity::CoverMissingGrillOpeningChildSafetySpec;
        actions.push(
            "Radiator cover installed but DOES NOT MEET grill-opening child-safety spec — \
             cover must completely enclose top + sides + front with grill openings sized \
             to prevent child finger insertion per NYC Int 1489-2017. Replace with \
             compliant cover; verify grill openings less than 1/2 inch per ASTM F963-23 \
             toy safety standard (analogous child-finger-trap specification)."
                .to_string(),
        );
    } else if matches!(input.heating_system_type, HeatingSystemType::SteamRadiator)
        && !input.radiator_thermostat_or_control_present
    {
        severity = Severity::UnregulatedRadiatorWithoutThermostatHazard;
        actions.push(
            "Steam radiator operating without thermostat or shut-off valve — unregulated \
             heat output creates burn hazard plus tenant inability to control room \
             temperature. Install thermostatic radiator valve (TRV) such as Honeywell HR92 \
             or Drayton Wiser; cost approximately $50-$150 per radiator. Consider \
             coordinating with [[tenant_smart_thermostat_install_right]] (iter 497) for \
             building-wide thermostatic control upgrade."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantCoverInstalledOrNoRequest;
        actions.push(format!(
            "Compliant: radiator cover installed (or no tenant request submitted), grill-\
             opening child-safety spec met (if cover present), biennial inspection cycle \
             current (last inspection within {} months for households with children under \
             6). Maintain inspection records for {}-year statute-of-limitations window; \
             schedule next biennial inspection plus reminder calendar; train property-\
             management staff on Int 1489-2017 / Ben Z's Law requirements.",
            NYC_BIENNIAL_INSPECTION_INTERVAL_MONTHS, 6
        ));
    }

    match input.jurisdiction {
        Jurisdiction::NewYorkCity => {
            notes.push(format!(
                "NYC Int 1489-2017 / Local Law 79 of {} (amending NYC Admin Code § 27-2076) \
                 requires landlord to install radiator covers within {} days of WRITTEN \
                 tenant request when child age twelve or younger resides in unit; cover \
                 must completely enclose top + sides + front with grill openings preventing \
                 child finger insertion. NYC Int 0925-2024 'Ben Z's Law' (named after \
                 infant Bencel 'Ben Z' Yancanay who died {}) requires biennial inspection \
                 of radiators in apartments AND common areas where children under 6 reside; \
                 exempts owner-occupied co-ops + condos. HPD civil penalty up to ${} per \
                 violation per HPD penalty schedule. Steam radiator burn injury litigation \
                 routinely $500K-$2M settlement exposure for permanent pediatric \
                 disfigurement.",
                INT_1489_2017_LOCAL_LAW_YEAR,
                NYC_RADIATOR_COVER_INSTALL_DAYS,
                BEN_Z_DEATH_DATE,
                NYC_HPD_PENALTY_MAX_CENTS / 100
            ));
        }
        Jurisdiction::Boston => {
            notes.push(
                "Massachusetts M.G.L. ch. 186 § 14 quiet enjoyment plus 105 CMR 410.180 \
                 State Sanitary Code radiator-temperature provisions plus 527 CMR 1.00 \
                 fire prevention plus common-law negligence framework. No state-mandated \
                 cover requirement for households with children but common-law duty of care \
                 attaches via negligence per se theory when steam radiator burn injury \
                 occurs."
                    .to_string(),
            );
        }
        Jurisdiction::Chicago => {
            notes.push(
                "Chicago Municipal Code § 5-12-110 RLTO habitability requires heat without \
                 separately mandating radiator covers; common-law negligence + ordinance § \
                 13-196-300 building maintenance govern. Chicago Department of Public \
                 Health Section 11-4-1700 child-injury reporting requirement applies."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Common-law habitability doctrine plus state-specific child-safety \
                 negligence framework. Steam radiator burn injury litigation governed by \
                 premises-liability and negligence-per-se theories; landlord duty heightened \
                 when (1) child or cognitively-impaired tenant in residence, (2) prior \
                 burn-injury incident on premises, (3) tenant complaint of unsafe radiator \
                 condition. ASTM F2779-19 portable radiator safety standard provides \
                 manufacturer baseline applicable by analogy."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[heat_requirements]] (general heat temperature habitability), \
         [[rental_chimney_fireplace_inspection_disclosure]] (solid-fuel heating framework), \
         [[rental_pellet_stove_disclosure]] (iter 499 pellet stove), [[rental_natural_gas_\
         leak_response]] (gas-system leak), [[rental_oil_tank_replacement_disclosure]] (iter \
         493 oil-heat fuel-storage framework), [[tenant_smart_thermostat_install_right]] \
         (iter 497 — building-wide thermostatic control upgrade pathway), [[mid_tenancy_\
         temporary_relocation]] (when unit unsafe pending cover installation), \
         [[tenant_emotional_distress_damages]] (IIED claim for severe burn injury)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::SteamBurnInjuryHabitabilityBreach => input.annual_rent_cents,
        Severity::LandlordCoverInstallationOverdue90Day
        | Severity::BenZLawBiennialInspectionOverdue
        | Severity::CoverMissingGrillOpeningChildSafetySpec => input.annual_rent_cents,
        Severity::UnregulatedRadiatorWithoutThermostatHazard => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        hpd_civil_penalty_max_cents: NYC_HPD_PENALTY_MAX_CENTS,
        citation: match input.jurisdiction {
            Jurisdiction::NewYorkCity => {
                "NYC Int 1489-2017 + LL 79 of 2018 + Admin Code § 27-2076 + Int 0925-2024 Ben Z's Law"
            }
            Jurisdiction::Boston => "M.G.L. ch. 186 § 14 + 105 CMR 410.180 + 527 CMR 1.00",
            Jurisdiction::Chicago => "Chicago Municipal Code § 5-12-110 + § 13-196-300",
            Jurisdiction::Default => "Common-law habitability + ASTM F2779-19",
        },
        notes,
    }
}

pub type RentalRadiatorSteamHeatSafetyInput = Input;
pub type RentalRadiatorSteamHeatSafetyResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYorkCity,
            heating_system_type: HeatingSystemType::SteamRadiator,
            tenant_composition: TenantComposition::HouseholdWithChildUnder12,
            tenant_written_request_for_cover_submitted: true,
            days_since_tenant_cover_request: 30,
            radiator_cover_installed: true,
            cover_meets_grill_opening_child_safety_spec: true,
            months_since_last_biennial_inspection: 12,
            steam_burn_injury_occurred: false,
            radiator_thermostat_or_control_present: true,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_radiator_not_applicable() {
        let mut i = baseline();
        i.heating_system_type = HeatingSystemType::NoRadiator;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn forced_air_central_not_applicable() {
        let mut i = baseline();
        i.heating_system_type = HeatingSystemType::ForcedAirCentral;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
    }

    #[test]
    fn compliant_cover_installed_baseline() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantCoverInstalledOrNoRequest
        ));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn nyc_cover_overdue_after_90_days_violation_full_rent() {
        let mut i = baseline();
        i.radiator_cover_installed = false;
        i.days_since_tenant_cover_request = 120;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LandlordCoverInstallationOverdue90Day
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Int 1489-2017")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 27-2076")));
    }

    #[test]
    fn nyc_cover_request_within_90_days_compliant() {
        let mut i = baseline();
        i.radiator_cover_installed = false;
        i.days_since_tenant_cover_request = 60;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantCoverInstalledOrNoRequest
        ));
    }

    #[test]
    fn ben_z_law_biennial_inspection_overdue_child_under_6() {
        let mut i = baseline();
        i.tenant_composition = TenantComposition::HouseholdWithChildUnder6;
        i.months_since_last_biennial_inspection = 30;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BenZLawBiennialInspectionOverdue
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Int 0925-2024")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Ben Z")));
    }

    #[test]
    fn ben_z_law_child_under_12_not_under_6_no_inspection_violation() {
        let mut i = baseline();
        i.tenant_composition = TenantComposition::HouseholdWithChildUnder12;
        i.months_since_last_biennial_inspection = 30;
        let r = check(&i);
        assert!(!matches!(
            r.severity,
            Severity::BenZLawBiennialInspectionOverdue
        ));
    }

    #[test]
    fn cover_missing_grill_opening_spec_violation() {
        let mut i = baseline();
        i.cover_meets_grill_opening_child_safety_spec = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CoverMissingGrillOpeningChildSafetySpec
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("ASTM F963-23")));
    }

    #[test]
    fn steam_burn_injury_habitability_breach_full_rent() {
        let mut i = baseline();
        i.steam_burn_injury_occurred = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::SteamBurnInjuryHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("250°F")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Mayo Clinic")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("$500K-$2M")));
    }

    #[test]
    fn steam_radiator_no_thermostat_hazard_half_rent() {
        let mut i = baseline();
        i.radiator_thermostat_or_control_present = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::UnregulatedRadiatorWithoutThermostatHazard
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Honeywell HR92")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("tenant_smart_thermostat_install_right")));
    }

    #[test]
    fn nyc_jurisdiction_pins_int_1489_2017_and_ben_z() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Int 1489-2017")));
        assert!(r.notes.iter().any(|n| n.contains("Local Law 79")));
        assert!(r.notes.iter().any(|n| n.contains("Ben Z")));
        assert!(r.notes.iter().any(|n| n.contains("Bencel")));
        assert!(r.notes.iter().any(|n| n.contains("2020-12")));
        assert!(r.notes.iter().any(|n| n.contains("§ 27-2076")));
    }

    #[test]
    fn boston_jurisdiction_pins_105_cmr_410_180() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Boston;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 186 § 14")));
        assert!(r.notes.iter().any(|n| n.contains("105 CMR 410.180")));
        assert!(r.notes.iter().any(|n| n.contains("527 CMR 1.00")));
    }

    #[test]
    fn chicago_jurisdiction_pins_rlto_5_12_110() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Chicago;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Chicago Municipal Code § 5-12-110")));
        assert!(r.notes.iter().any(|n| n.contains("§ 13-196-300")));
    }

    #[test]
    fn default_jurisdiction_pins_astm_f2779_19() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("ASTM F2779-19")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Common-law habitability")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("heat_requirements")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_chimney_fireplace_inspection_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_pellet_stove_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_smart_thermostat_install_right")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::NewYorkCity,
            Jurisdiction::Boston,
            Jurisdiction::Chicago,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes.iter().any(|n| n.contains("heat_requirements")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn nyc_radiator_cover_install_days_pins_90() {
        assert_eq!(NYC_RADIATOR_COVER_INSTALL_DAYS, 90);
    }

    #[test]
    fn nyc_biennial_inspection_pins_24_months() {
        assert_eq!(NYC_BIENNIAL_INSPECTION_INTERVAL_MONTHS, 24);
    }

    #[test]
    fn nyc_hpd_penalty_pins_500_dollars() {
        assert_eq!(NYC_HPD_PENALTY_MAX_CENTS, 50_000);
    }

    #[test]
    fn steam_radiator_peak_temp_pins_250_f() {
        assert_eq!(STEAM_RADIATOR_PEAK_TEMP_F, 250);
    }

    #[test]
    fn int_1489_local_law_year_pins_2018() {
        assert_eq!(INT_1489_2017_LOCAL_LAW_YEAR, 2018);
    }

    #[test]
    fn ben_z_death_date_pins_2020_12() {
        assert_eq!(BEN_Z_DEATH_DATE, "2020-12");
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let nyc = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewYorkCity;
            i
        });
        let bos = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Boston;
            i
        });
        let chi = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Chicago;
            i
        });
        let de = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Default;
            i
        });
        assert!(nyc.citation.contains("Int 1489-2017"));
        assert!(bos.citation.contains("M.G.L. ch. 186"));
        assert!(chi.citation.contains("§ 5-12-110"));
        assert!(de.citation.contains("ASTM F2779-19"));
    }

    #[test]
    fn severity_priority_burn_injury_overrides_cover_status() {
        let mut i = baseline();
        i.steam_burn_injury_occurred = true;
        i.radiator_cover_installed = false;
        i.days_since_tenant_cover_request = 200;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::SteamBurnInjuryHabitabilityBreach
        ));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.radiator_cover_installed = false;
        i.days_since_tenant_cover_request = 120;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn hot_water_radiator_compliant_no_steam_burn_hazard() {
        let mut i = baseline();
        i.heating_system_type = HeatingSystemType::HotWaterRadiator;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantCoverInstalledOrNoRequest
        ));
    }
}
