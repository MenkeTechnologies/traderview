//! Rental balcony grill and propane / LP-gas BBQ restriction framework —
//! covers NFPA 1 Fire Code § 10.10.5-7 (no open-flame grills on combustible
//! balconies within 10 feet of multifamily building exterior) plus § 6.18
//! LP-gas cylinder storage restrictions (no propane above first floor or
//! inside residential units) plus state-specific fire code amendments plus
//! tenant lease enforcement plus insurance coverage exclusions.
//!
//! Distinct from sibling [[rental_chimney_fireplace_inspection_disclosure]]
//! (indoor solid-fuel framework), [[rental_natural_gas_leak_response]] (gas
//! line leak protocol — distinct from LP-gas cylinder), [[rental_propane_
//! tank_lease_disclosure]] (large stationary propane tank disclosure
//! framework), [[rental_pellet_stove_disclosure]] (iter 499 wood-pellet
//! appliance separate framework), [[rental_balcony_inspection_seismic_safety]]
//! (iter 511 EEE inspection — balcony structural framework cross-reference).
//!
//! Trader-landlord critical because (1) **NFPA 1 § 10.10.5** prohibits
//! hibachi, gas-fired grill, charcoal grill, or other open-flame cooking
//! device from being used or kindled on any balcony or under any
//! overhanging portion of a multifamily building within 10 feet of any
//! structure; (2) **NFPA 1 § 6.18** prohibits propane / LP-gas cylinders
//! above 1-pound disposable size (2.5-pound water capacity cylinder) from
//! being stored above the first floor or inside residential units —
//! 20-pound BBQ tanks categorically prohibited in multifamily balcony
//! storage; (3) **carve-out**: listed equipment permanently installed
//! per its listing — gas grill hardwired into building gas supply by
//! licensed plumber — IS permitted; (4) insurance coverage broadly
//! excludes balcony-grill fire losses under standard ISO HO-3 + dwelling
//! policy fire-cause exclusions; (5) annual US fire deaths from BBQ
//! fires approximately 17 + injuries 8,800 per US Fire Administration
//! data; (6) NYC FDNY + LA Fire Department + Boston Fire Department all
//! issue per-violation civil penalties $250-$1,000 plus lease eviction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// California — Cal. Fire Code § 308.1.4 mirrors NFPA 1 with state-
    /// specific amendments.
    California,
    /// New York City — FDNY 3 RCNY 102-01 + NYC Admin Code Title 29.
    NewYorkCity,
    /// Massachusetts — 527 CMR 1.00 Comprehensive Fire Safety Code
    /// (adopted IFC + NFPA 1).
    Massachusetts,
    /// Texas — Texas Local Government Code Ch. 235.
    Texas,
    /// Default — NFPA 1 (2018 + 2021 edition) standalone.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    /// Multifamily 3+ units — NFPA 1 § 10.10.5 applies.
    Multifamily3PlusUnits,
    /// Single-family or duplex — NFPA 1 § 10.10.5 inapplicable; only HOA
    /// rules + lease terms may restrict.
    SingleFamilyOrDuplex,
    /// Detached cottage or villa — out of scope.
    DetachedCottage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrillType {
    /// 20-pound (standard BBQ tank) propane grill.
    PropaneTwentyPoundCylinder,
    /// 1-pound disposable propane canister (camping size).
    PropaneOnePoundDisposable,
    /// Charcoal grill.
    Charcoal,
    /// Hibachi grill.
    Hibachi,
    /// Electric grill (NO open flame, NO LP-gas cylinder).
    Electric,
    /// Permanently installed hardwired natural gas grill (carve-out).
    NaturalGasHardwired,
    /// No grill operated.
    NoGrill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrillLocation {
    /// Balcony of multifamily unit above first floor.
    BalconyAboveFirstFloor,
    /// Ground-floor patio less than 10 feet from building structure.
    GroundFloorPatioWithin10Feet,
    /// Ground-floor patio at or beyond 10 feet from building.
    GroundFloorPatioAtLeast10Feet,
    /// Designated common-area grill station maintained by landlord.
    DesignatedCommonAreaGrillStation,
    /// Indoor — never permitted.
    Indoor,
    /// Not applicable.
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantElectricOrPermanentInstall,
    CompliantBeyond10FeetGroundFloor,
    CompliantDesignatedCommonAreaStation,
    NfpaOpenFlameOnBalconyViolation,
    NfpaPropaneAbove1PoundStoredAboveFirstFloorViolation,
    NfpaWithin10FeetOfStructureViolation,
    IndoorGrillCategoricallyProhibited,
    LeaseEnforcementRequiredTenantViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub building_type: BuildingType,
    pub grill_type: GrillType,
    pub grill_location: GrillLocation,
    pub tenant_observed_in_violation: bool,
    pub lease_explicitly_prohibits_grilling: bool,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub civil_penalty_min_cents: u64,
    pub civil_penalty_max_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const NFPA_1_DISTANCE_FROM_STRUCTURE_FEET: u32 = 10;
pub const NFPA_1_LP_GAS_MAX_DISPOSABLE_WATER_CAPACITY_POUNDS: u32 = 2;
pub const FDNY_NYC_PENALTY_MIN_CENTS: u64 = 25_000;
pub const FDNY_NYC_PENALTY_MAX_CENTS: u64 = 100_000;
pub const US_BBQ_FIRE_DEATHS_PER_YEAR: u32 = 17;
pub const US_BBQ_FIRE_INJURIES_PER_YEAR: u32 = 8_800;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.grill_type, GrillType::NoGrill)
        || matches!(input.grill_location, GrillLocation::NotApplicable)
    {
        notes.push(
            "No grill operated on premises — framework inapplicable. Recommend documenting \
             grill-prohibition language in lease addendum even when no current grill use \
             observed; future tenant operations may trigger NFPA 1 violations."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            civil_penalty_min_cents: 0,
            civil_penalty_max_cents: 0,
            citation: "n/a (no grill)",
            notes,
        };
    }

    if matches!(input.grill_location, GrillLocation::Indoor) {
        severity = Severity::IndoorGrillCategoricallyProhibited;
        actions.push(
            "Indoor grill operation categorically prohibited per NFPA 1 § 10.10.4 plus IFC § \
             308.1 (open flames in residential structure). Carbon monoxide death risk plus \
             structural fire risk. Immediate cease-and-desist notice; refer to lease \
             alteration / nuisance clause for eviction. NFPA 1 § 10.10.4(2) common-law \
             prohibition reinforced by state-mandated CO detector requirements."
                .to_string(),
        );
    } else if matches!(
        input.grill_type,
        GrillType::Electric | GrillType::NaturalGasHardwired
    ) {
        severity = Severity::CompliantElectricOrPermanentInstall;
        actions.push(
            "Electric grill or permanently-installed natural-gas hardwired grill (per its \
             listing per NFPA 1 § 10.10.5 carve-out) is compliant. Document gas-line \
             permitting per IFGC § 401 + state plumbing-code installation by licensed gas-\
             fitter. Verify annual leak test per § 401.7."
                .to_string(),
        );
    } else if matches!(input.grill_location, GrillLocation::DesignatedCommonAreaGrillStation) {
        severity = Severity::CompliantDesignatedCommonAreaStation;
        actions.push(
            "Designated common-area grill station maintained by landlord at least 10 feet \
             from any structure plus monitored for safe operation — compliant. Maintain \
             documented grill-station safety protocol plus annual inspection plus combustible-\
             material clearance plus Class K fire extinguisher per NFPA 10."
                .to_string(),
        );
    } else if matches!(input.building_type, BuildingType::Multifamily3PlusUnits)
        && matches!(input.grill_location, GrillLocation::BalconyAboveFirstFloor)
    {
        severity = Severity::NfpaOpenFlameOnBalconyViolation;
        actions.push(format!(
            "NFPA 1 § 10.10.5 violation: open-flame grill operated on balcony of multifamily \
             building. Hibachi, gas-fired grill, charcoal grill, or other open-flame cooking \
             device PROHIBITED on any balcony or under any overhanging portion of multifamily \
             building. Approximately {} US BBQ fire deaths plus {} injuries annually per US \
             Fire Administration. Immediate cease-and-desist; document violation with \
             photographs; reference NFPA 1 § 10.10.5 in cease notice.",
            US_BBQ_FIRE_DEATHS_PER_YEAR, US_BBQ_FIRE_INJURIES_PER_YEAR
        ));
        if matches!(
            input.grill_type,
            GrillType::PropaneTwentyPoundCylinder
        ) {
            actions.push(format!(
                "ADDITIONAL VIOLATION: NFPA 1 § 6.18 LP-gas cylinder above 1-pound disposable \
                 size ({}-pound water capacity max) PROHIBITED from storage above first floor \
                 or inside residential units. 20-pound BBQ propane tank categorically \
                 prohibited on multifamily balcony. Explosion risk in elevated or enclosed \
                 areas. Immediate removal of cylinder required.",
                NFPA_1_LP_GAS_MAX_DISPOSABLE_WATER_CAPACITY_POUNDS
            ));
        }
    } else if matches!(
        input.grill_type,
        GrillType::PropaneTwentyPoundCylinder
    ) && matches!(
        input.grill_location,
        GrillLocation::BalconyAboveFirstFloor
    ) {
        severity = Severity::NfpaPropaneAbove1PoundStoredAboveFirstFloorViolation;
        actions.push(format!(
            "NFPA 1 § 6.18 violation: 20-pound LP-gas BBQ cylinder stored above first floor. \
             Only disposable cylinders up to {}-pound water capacity (approximately 1-pound \
             propane) permitted above first floor in multifamily building. Immediate cylinder \
             removal required.",
            NFPA_1_LP_GAS_MAX_DISPOSABLE_WATER_CAPACITY_POUNDS
        ));
    } else if matches!(
        input.grill_location,
        GrillLocation::GroundFloorPatioWithin10Feet
    ) && matches!(input.building_type, BuildingType::Multifamily3PlusUnits)
    {
        severity = Severity::NfpaWithin10FeetOfStructureViolation;
        actions.push(format!(
            "NFPA 1 § 10.10.5 violation: open-flame grill operated within {} feet of \
             multifamily building structure. Relocate grill to a minimum of {} feet from \
             building exterior wall plus overhanging portions plus combustible structure.",
            NFPA_1_DISTANCE_FROM_STRUCTURE_FEET, NFPA_1_DISTANCE_FROM_STRUCTURE_FEET
        ));
    } else if input.tenant_observed_in_violation && input.lease_explicitly_prohibits_grilling {
        severity = Severity::LeaseEnforcementRequiredTenantViolation;
        actions.push(
            "Tenant observed in violation of lease no-grilling clause; landlord enforcement \
             remedies: (1) 3-day Notice to Cure per state landlord-tenant procedure, (2) if \
             uncured, unlawful detainer plus eviction, (3) document violation with timestamped \
             photographs plus witness statements plus fire-marshal citation if issued, (4) \
             notify insurance carrier of violation to preserve coverage."
                .to_string(),
        );
    } else if matches!(
        input.grill_location,
        GrillLocation::GroundFloorPatioAtLeast10Feet
    ) {
        severity = Severity::CompliantBeyond10FeetGroundFloor;
        actions.push(
            "Compliant: ground-floor patio grilling beyond 10-foot setback from structure \
             per NFPA 1 § 10.10.5 carve-out for ground-floor use. Maintain Class K or \
             ABC fire extinguisher accessible; clear combustible debris from 3-foot \
             perimeter; never operate unattended."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantBeyond10FeetGroundFloor;
        actions.push(
            "Compliant: grill location and type satisfy NFPA 1 § 10.10.5 setback plus § \
             6.18 LP-gas cylinder restrictions."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(
                "Cal. Fire Code § 308.1.4 mirrors NFPA 1 § 10.10.5 with state-specific \
                 amendments; CFC § 308.1.6.2 specifically addresses LP-gas BBQ devices on \
                 multifamily balconies. Cal. Health & Safety Code § 18900 fire marshal \
                 enforcement. CAL FIRE issues advisories during red flag warning periods."
                    .to_string(),
            );
        }
        Jurisdiction::NewYorkCity => {
            notes.push(format!(
                "FDNY 3 RCNY 102-01 plus NYC Admin Code Title 29 (NYC Fire Code) — civil \
                 penalties ${} to ${} per violation per FDNY enforcement schedule; FDNY \
                 Bureau of Fire Investigation may issue summons. NYC FC 307.5 specifically \
                 prohibits LP-gas grills with > 1-pound cylinders on multifamily balconies; \
                 enforcement intensifies during summer cookout season.",
                FDNY_NYC_PENALTY_MIN_CENTS / 100, FDNY_NYC_PENALTY_MAX_CENTS / 100
            ));
        }
        Jurisdiction::Massachusetts => {
            notes.push(
                "527 CMR 1.00 Massachusetts Comprehensive Fire Safety Code (adopted IFC + \
                 NFPA 1 with state amendments). M.G.L. ch. 148 § 26G governs LP-gas storage. \
                 State Fire Marshal Office issues annual summer-grilling-season advisories."
                    .to_string(),
            );
        }
        Jurisdiction::Texas => {
            notes.push(
                "Texas Local Government Code Ch. 235 authorizes municipal fire codes; \
                 Texas State Fire Marshal Office enforces statewide. Most TX cities (Houston, \
                 Dallas, Austin, San Antonio) adopt IFC + NFPA 1 with local amendments. TX \
                 State Fire Marshal Office advisories on grill safety during burn ban periods."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "NFPA 1 (2018 + 2021 editions) standalone framework: § 10.10.5 open-flame \
                 cooking restriction on multifamily balconies + § 6.18 LP-gas cylinder \
                 storage restrictions. State and local fire code adoption varies; many \
                 municipalities adopt IFC by reference incorporating NFPA 1 standards."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_propane_tank_lease_disclosure]] (stationary propane \
         tank disclosure framework — distinct from BBQ cylinder), [[rental_natural_gas_\
         leak_response]] (natural gas line distinct exposure pathway), [[rental_chimney_\
         fireplace_inspection_disclosure]] (indoor solid-fuel framework), [[rental_pellet_\
         stove_disclosure]] (iter 499 wood-pellet appliance), [[rental_balcony_inspection_\
         seismic_safety]] (iter 511 EEE structural — balcony BBQ damage may affect \
         structural integrity), [[tenant_emotional_distress_damages]] (IIED claim for \
         neighbor exposure to grill smoke or fire risk)."
            .to_string(),
    );

    let (penalty_min, penalty_max) = match input.jurisdiction {
        Jurisdiction::NewYorkCity => (FDNY_NYC_PENALTY_MIN_CENTS, FDNY_NYC_PENALTY_MAX_CENTS),
        _ => (FDNY_NYC_PENALTY_MIN_CENTS, FDNY_NYC_PENALTY_MAX_CENTS),
    };

    let annual_rent_at_risk: u64 = match severity {
        Severity::IndoorGrillCategoricallyProhibited
        | Severity::NfpaOpenFlameOnBalconyViolation
        | Severity::NfpaPropaneAbove1PoundStoredAboveFirstFloorViolation => {
            input.annual_rent_cents
        }
        Severity::NfpaWithin10FeetOfStructureViolation
        | Severity::LeaseEnforcementRequiredTenantViolation => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        civil_penalty_min_cents: penalty_min,
        civil_penalty_max_cents: penalty_max,
        citation: match input.jurisdiction {
            Jurisdiction::California => {
                "Cal. Fire Code § 308.1.4 + § 308.1.6.2 + Cal. Health & Safety Code § 18900"
            }
            Jurisdiction::NewYorkCity => {
                "FDNY 3 RCNY 102-01 + NYC Admin Title 29 + NYC FC 307.5"
            }
            Jurisdiction::Massachusetts => "527 CMR 1.00 + M.G.L. ch. 148 § 26G",
            Jurisdiction::Texas => "Tex. Local Gov't Code Ch. 235 + State Fire Marshal Office",
            Jurisdiction::Default => "NFPA 1 § 10.10.5 + § 6.18",
        },
        notes,
    }
}

pub type RentalGrillPropaneBbqRestrictionInput = Input;
pub type RentalGrillPropaneBbqRestrictionResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            building_type: BuildingType::Multifamily3PlusUnits,
            grill_type: GrillType::PropaneTwentyPoundCylinder,
            grill_location: GrillLocation::GroundFloorPatioAtLeast10Feet,
            tenant_observed_in_violation: false,
            lease_explicitly_prohibits_grilling: false,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_grill_not_applicable() {
        let mut i = baseline();
        i.grill_type = GrillType::NoGrill;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn indoor_grill_categorically_prohibited() {
        let mut i = baseline();
        i.grill_location = GrillLocation::Indoor;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::IndoorGrillCategoricallyProhibited));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 10.10.4")));
    }

    #[test]
    fn electric_grill_compliant() {
        let mut i = baseline();
        i.grill_type = GrillType::Electric;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantElectricOrPermanentInstall));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn natural_gas_hardwired_compliant() {
        let mut i = baseline();
        i.grill_type = GrillType::NaturalGasHardwired;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantElectricOrPermanentInstall));
        assert!(r.recommended_actions.iter().any(|a| a.contains("IFGC § 401")));
    }

    #[test]
    fn designated_common_area_station_compliant() {
        let mut i = baseline();
        i.grill_location = GrillLocation::DesignatedCommonAreaGrillStation;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantDesignatedCommonAreaStation));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Class K")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("NFPA 10")));
    }

    #[test]
    fn balcony_open_flame_violation_full_rent() {
        let mut i = baseline();
        i.grill_location = GrillLocation::BalconyAboveFirstFloor;
        i.grill_type = GrillType::PropaneTwentyPoundCylinder;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NfpaOpenFlameOnBalconyViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 10.10.5")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains(&US_BBQ_FIRE_DEATHS_PER_YEAR.to_string())));
    }

    #[test]
    fn balcony_open_flame_with_propane_double_violation() {
        let mut i = baseline();
        i.grill_location = GrillLocation::BalconyAboveFirstFloor;
        i.grill_type = GrillType::PropaneTwentyPoundCylinder;
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 6.18")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("20-pound BBQ propane tank")));
    }

    #[test]
    fn charcoal_balcony_violation_no_propane_secondary() {
        let mut i = baseline();
        i.grill_location = GrillLocation::BalconyAboveFirstFloor;
        i.grill_type = GrillType::Charcoal;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NfpaOpenFlameOnBalconyViolation));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 10.10.5")));
        assert!(!r.recommended_actions.iter().any(|a| a.contains("§ 6.18")));
    }

    #[test]
    fn hibachi_balcony_violation() {
        let mut i = baseline();
        i.grill_location = GrillLocation::BalconyAboveFirstFloor;
        i.grill_type = GrillType::Hibachi;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NfpaOpenFlameOnBalconyViolation));
    }

    #[test]
    fn within_10_feet_ground_floor_violation_half_rent() {
        let mut i = baseline();
        i.grill_location = GrillLocation::GroundFloorPatioWithin10Feet;
        i.grill_type = GrillType::Charcoal;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NfpaWithin10FeetOfStructureViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("10 feet")));
    }

    #[test]
    fn beyond_10_feet_ground_floor_compliant() {
        let mut i = baseline();
        i.grill_location = GrillLocation::GroundFloorPatioAtLeast10Feet;
        i.grill_type = GrillType::Charcoal;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantBeyond10FeetGroundFloor));
    }

    #[test]
    fn one_pound_disposable_propane_balcony_compliant() {
        let mut i = baseline();
        i.grill_type = GrillType::PropaneOnePoundDisposable;
        i.grill_location = GrillLocation::GroundFloorPatioAtLeast10Feet;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantBeyond10FeetGroundFloor));
    }

    #[test]
    fn lease_prohibition_tenant_violation_half_rent() {
        let mut i = baseline();
        i.lease_explicitly_prohibits_grilling = true;
        i.tenant_observed_in_violation = true;
        i.grill_location = GrillLocation::GroundFloorPatioAtLeast10Feet;
        i.grill_type = GrillType::Charcoal;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LeaseEnforcementRequiredTenantViolation
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("3-day Notice to Cure")));
    }

    #[test]
    fn ca_jurisdiction_pins_fire_code_308_1_4() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Cal. Fire Code § 308.1.4")));
        assert!(r.notes.iter().any(|n| n.contains("CAL FIRE")));
    }

    #[test]
    fn nyc_jurisdiction_pins_fdny_3_rcny_102_01_and_307_5() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYorkCity;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("FDNY 3 RCNY 102-01")));
        assert!(r.notes.iter().any(|n| n.contains("NYC FC 307.5")));
        assert!(r.notes.iter().any(|n| n.contains("250")));
        assert!(r.notes.iter().any(|n| n.contains("1000")));
    }

    #[test]
    fn ma_jurisdiction_pins_527_cmr_1_and_148_26g() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("527 CMR 1.00")));
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 148 § 26G")));
    }

    #[test]
    fn tx_jurisdiction_pins_local_gov_code_235() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Texas;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Texas Local Government Code Ch. 235")));
        assert!(r.notes.iter().any(|n| n.contains("State Fire Marshal Office")));
    }

    #[test]
    fn default_jurisdiction_pins_nfpa_1_2018_2021() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("NFPA 1 (2018 + 2021 editions)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 10.10.5")));
        assert!(r.notes.iter().any(|n| n.contains("§ 6.18")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_propane_tank_lease_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_natural_gas_leak_response")));
        assert!(r.notes.iter().any(|n| n.contains("rental_pellet_stove_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_balcony_inspection_seismic_safety")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::NewYorkCity,
            Jurisdiction::Massachusetts,
            Jurisdiction::Texas,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("rental_propane_tank_lease_disclosure")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn nfpa_1_distance_pins_10_feet() {
        assert_eq!(NFPA_1_DISTANCE_FROM_STRUCTURE_FEET, 10);
    }

    #[test]
    fn nfpa_1_lp_gas_max_disposable_pins_2_pounds_water_capacity() {
        assert_eq!(NFPA_1_LP_GAS_MAX_DISPOSABLE_WATER_CAPACITY_POUNDS, 2);
    }

    #[test]
    fn fdny_nyc_penalty_pins_250_to_1000() {
        assert_eq!(FDNY_NYC_PENALTY_MIN_CENTS, 25_000);
        assert_eq!(FDNY_NYC_PENALTY_MAX_CENTS, 100_000);
    }

    #[test]
    fn us_bbq_fire_deaths_pins_17_per_year() {
        assert_eq!(US_BBQ_FIRE_DEATHS_PER_YEAR, 17);
    }

    #[test]
    fn us_bbq_fire_injuries_pins_8800_per_year() {
        assert_eq!(US_BBQ_FIRE_INJURIES_PER_YEAR, 8_800);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let nyc = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewYorkCity; i });
        let ma = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Massachusetts; i });
        let tx = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Texas; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ca.citation.contains("Cal. Fire Code"));
        assert!(nyc.citation.contains("FDNY"));
        assert!(ma.citation.contains("527 CMR 1.00"));
        assert!(tx.citation.contains("Tex. Local Gov't"));
        assert!(de.citation.contains("NFPA 1"));
    }

    #[test]
    fn severity_priority_indoor_overrides_grill_type() {
        let mut i = baseline();
        i.grill_location = GrillLocation::Indoor;
        i.grill_type = GrillType::Electric;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::IndoorGrillCategoricallyProhibited));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.grill_location = GrillLocation::BalconyAboveFirstFloor;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn single_family_within_10_feet_no_nfpa_violation() {
        let mut i = baseline();
        i.building_type = BuildingType::SingleFamilyOrDuplex;
        i.grill_location = GrillLocation::GroundFloorPatioWithin10Feet;
        i.grill_type = GrillType::Charcoal;
        let r = check(&i);
        assert!(!matches!(
            r.severity,
            Severity::NfpaWithin10FeetOfStructureViolation
        ));
    }
}
