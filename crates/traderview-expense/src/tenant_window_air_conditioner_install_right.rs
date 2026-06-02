//! Multi-jurisdictional TENANT WINDOW AIR CONDITIONER
//! installation right and landlord cooling provision
//! compliance framework. When may a tenant install a
//! window air conditioner unit in a rental property,
//! what bracket safety + falling-AC liability rules apply,
//! when must landlord provide cooling under emerging
//! state and municipal heat-mandate laws, and what
//! failure-mode liabilities expose landlord after a
//! heat-stress fatality or falling-AC injury?
//!
//! Distinct from sibling modules: cooling_requirements
//! (general cooling habitability standards), rental_gas_
//! appliance_ban (electrification), tenant_solar_
//! installation (solar panels), tenant_ev_charging_
//! installation_right (EV charging), rental_window_guard_
//! installation (fall protection from window), tenant_
//! emotional_distress_damages (IIED), rental_natural_gas_
//! leak_response (iter 485).
//!
//! Three-jurisdiction framework:
//!
//! 1. NEW YORK CITY (most recent + most prescriptive) —
//!    NYC Int 0994 of 2024 "Cool Homes for All Act"
//!    enacted 2024-2025, effective fully by 2026: landlord
//!    must provide air conditioning + maintain bedroom
//!    temperature at 78°F or LOWER from JUNE 15 to
//!    SEPTEMBER 15 when outdoor temperature exceeds 82°F.
//!    Applies to BOTH market-rate AND rent-stabilized
//!    units (excludes public housing). Window AC bracket
//!    safety: tenants in buildings TALLER THAN 6 STORIES
//!    must use approved brackets; tenants generally
//!    responsible for bracket installation; landlord
//!    cannot prohibit window AC absent safety basis.
//!    Heat-related fatality data: approximately 500
//!    NYC heat-related deaths annually drove
//!    legislation.
//! 2. CALIFORNIA — Cal. Civ. Code § 1941.1 implied
//!    warranty of habitability requires landlord to
//!    maintain HEATING; cooling not statutorily
//!    required at state level but Cal. Health & Safety
//!    Code § 17920.3 "substandard conditions" includes
//!    inability to maintain reasonable temperature;
//!    select CA cities (Palm Springs, Coachella Valley
//!    municipal codes) require landlord-provided
//!    cooling.
//! 3. DEFAULT — Common-law implied warranty of
//!    habitability per Hilder v. St. Peter, 478 A.2d
//!    202 (Vt. 1984); Green v. Superior Court, 10 Cal.
//!    3d 616 (1974); tort negligence + premises
//!    liability for heat-stress events; Arizona
//!    landlord-cooling regimes (Phoenix City Code
//!    § 39-16, Pima County Code 8.20, ARS § 33-1324(C))
//!    require landlord to provide cooling in summer
//!    months.
//!
//! Tenant rights typology — three install paths:
//! 1. TENANT-INSTALLED WITH LEASE PROVISION — most
//!    common; lease must not unreasonably prohibit
//!    safety-compliant installation
//! 2. LANDLORD-PROVIDED PRE-INSTALLED — NYC Cool Homes
//!    Act applies; landlord owns + maintains
//! 3. REASONABLE ACCOMMODATION REQUEST — disability/
//!    health-condition basis under Fair Housing Act +
//!    state equivalents requires landlord to permit
//!    install (and sometimes pay)
//!
//! Window AC bracket safety requirements (NYC + best
//! practice nationally):
//! 1. UNITS IN BUILDINGS TALLER THAN 6 STORIES require
//!    bracket
//! 2. INSTALLATION MUST PREVENT TILT, SLIP, OR FALL —
//!    units cannot project beyond approved tolerance
//! 3. WINDOW FRAME LOAD CAPACITY: if window cannot
//!    support unit weight, bracket required regardless
//!    of building height
//! 4. BRACKET MUST BE INSTALLED ACCORDING TO MANUFACTURER
//!    SPECIFICATIONS — generally requires fasteners
//!    into structural framing (not just window sash)
//! 5. ANNUAL INSPECTION + RE-TIGHTENING recommended
//!    pre-cooling season
//!
//! Universal five failure-mode liability framework:
//! 1. LANDLORD PROHIBITS WINDOW AC despite no
//!    legitimate safety basis → habitability dispute +
//!    tenant rescission claim under cooling requirements
//! 2. TENANT INSTALLS AC WITHOUT BRACKET on upper floor
//!    in NYC → falling-AC injury liability + landlord
//!    secondary liability under common-law negligence
//!    if knew or should have known
//! 3. NYC COOL HOMES ACT VIOLATION (post-2026) →
//!    Housing Maintenance Code violation + ECB civil
//!    penalty + tenant rent reduction + HPD complaint
//! 4. HEAT-STRESS EVENT DURING HEAT WAVE without AC →
//!    tort negligence + wrongful death + IIED parallel
//!    to tenant_emotional_distress_damages iter 453
//!    (heat-stress fatality settlements $1M+ routine)
//! 5. AC BRACKET NOT MAINTAINED → premises liability +
//!    falling-AC injury liability + insurance denial
//!
//! Trader-landlord critical because (1) NYC Cool Homes
//! Act 2026 effective is the leading state-level AC
//! mandate — non-compliance creates housing-code
//! violation + per-month rent reduction; (2) falling-
//! AC injuries from upper-floor windows are among the
//! highest-stakes premises-liability claims in NYC —
//! settlements routinely exceed $1M; (3) annual NYC
//! heat-related fatalities approximate 500 driving
//! legislative momentum; (4) landlord absolute
//! prohibition of window AC in non-NYC jurisdictions
//! is generally enforceable BUT cannot be unreasonable
//! under habitability doctrine; (5) reasonable
//! accommodation requests under Fair Housing Act for
//! disabled tenants must be granted absent undue
//! hardship.
//!
//! Authority: NYC Int 0994 of 2024 ("Cool Homes for All
//! Act"); NYC Admin. Code § 27-2029 (Heating Code
//! framework); NYC Window AC Bracket requirements;
//! Cal. Civ. Code § 1941.1 (implied warranty);
//! Cal. Health & Safety Code § 17920.3 (substandard
//! conditions); Phoenix City Code § 39-16 (cooling
//! requirement); Pima County Code 8.20; ARS § 33-1324(C)
//! (Arizona landlord cooling); Fair Housing Act
//! reasonable accommodation 42 U.S.C. § 3604(f);
//! Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984);
//! Green v. Superior Court, 10 Cal. 3d 616 (1974).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkCity,
    California,
    Arizona,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPath {
    TenantInstalledWithLease,
    LandlordProvided,
    ReasonableAccommodationDisability,
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub install_path: InstallPath,
    pub building_taller_than_6_stories: bool,
    pub bracket_installed: bool,
    pub bracket_maintained_to_manufacturer_spec: bool,
    pub window_frame_supports_unit_weight: bool,
    pub lease_prohibits_window_ac_no_safety_basis: bool,
    pub landlord_provides_ac_under_nyc_cool_homes_act: bool,
    pub bedroom_temp_above_78f_during_heat_advisory_period: bool,
    pub heat_stress_event_reported: bool,
    pub falling_ac_injury_reported: bool,
    pub disability_accommodation_request: bool,
    pub reasonable_accommodation_granted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    BracketRequiredNotInstalled,
    BracketMaintenanceDeficient,
    UnreasonableProhibition,
    NycCoolHomesActViolation,
    FailureToGrantAccommodation,
    HeatStressEvent,
    FallingAcInjury,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub type TenantWindowAirConditionerInstallRightInput = Input;
pub type TenantWindowAirConditionerInstallRightResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Three-jurisdiction framework: New York City (most recent + most prescriptive — NYC Int 0994 of 2024 'Cool Homes for All Act' fully effective by 2026; landlord must provide AC + maintain bedroom temp ≤78°F from JUNE 15 to SEPTEMBER 15 when outdoor temp exceeds 82°F; applies to market-rate AND rent-stabilized units, excludes public housing); California (Cal. Civ. Code § 1941.1 implied warranty of habitability heating only; Cal. Health & Safety Code § 17920.3 substandard conditions; select CA cities like Palm Springs require cooling); Default (common-law implied warranty per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984); Arizona Phoenix City Code § 39-16 + ARS § 33-1324(C) require landlord-provided cooling in summer months).".to_string(),
        "Window AC bracket safety requirements (NYC + best practice nationally): (1) units in buildings TALLER THAN 6 STORIES require bracket; (2) installation must prevent tilt, slip, or fall; (3) window frame load capacity — if window cannot support unit weight, bracket required regardless of building height; (4) bracket must be installed per manufacturer specifications with fasteners into structural framing; (5) annual inspection + re-tightening recommended pre-cooling season.".to_string(),
        "Tenant rights typology — three install paths: (1) TENANT-INSTALLED WITH LEASE PROVISION (most common; lease must not unreasonably prohibit safety-compliant installation); (2) LANDLORD-PROVIDED PRE-INSTALLED (NYC Cool Homes Act applies; landlord owns + maintains); (3) REASONABLE ACCOMMODATION REQUEST (disability/health-condition basis under Fair Housing Act 42 U.S.C. § 3604(f) + state equivalents — landlord must permit install and sometimes pay).".to_string(),
        "Five universal failure-mode liabilities: (1) landlord prohibits window AC despite no legitimate safety basis → habitability dispute + tenant rescission; (2) tenant installs AC without bracket on upper floor → falling-AC injury + landlord secondary common-law negligence liability; (3) NYC Cool Homes Act violation post-2026 → Housing Maintenance Code violation + ECB civil penalty + rent reduction + HPD complaint; (4) heat-stress event during heat wave → tort negligence + wrongful death + IIED parallel to tenant_emotional_distress_damages iter 453 ($1M+ settlements routine); (5) AC bracket not maintained → premises liability + falling-AC injury + insurance denial.".to_string(),
        "NYC heat-related fatality data: approximately 500 NYC heat-related deaths annually drove Cool Homes for All Act legislation. Falling-AC injuries from upper-floor windows are among the highest-stakes NYC premises-liability claims with settlements routinely exceeding $1M.".to_string(),
        "Companion modules: cooling_requirements (general cooling habitability standards), rental_gas_appliance_ban (electrification), tenant_solar_installation (solar panels), tenant_ev_charging_installation_right (EV charging), rental_window_guard_installation (fall protection from window), tenant_emotional_distress_damages (IIED), rental_natural_gas_leak_response (iter 485).".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if input.falling_ac_injury_reported {
        actions.push("Falling-AC injury reported: engage emergency services + counsel; preserve evidence including bracket + unit + installation records; tort negligence + wrongful death + IIED exposure; $1M+ settlement routine for NYC upper-floor falling-AC injury.".to_string());
    }

    if input.heat_stress_event_reported {
        actions.push("Heat-stress event reported: engage emergency services + counsel; preserve heat-advisory weather records + temperature logs; tort negligence + wrongful death + IIED exposure; approximately 500 NYC heat-related deaths annually drove Cool Homes for All Act legislation.".to_string());
    }

    if matches!(input.install_path, InstallPath::NotApplicable) {
        let mut n = notes;
        n.push("No window AC install path engaged — framework not applicable.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    let nyc_cool_homes_violation = matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && input.bedroom_temp_above_78f_during_heat_advisory_period
        && !input.landlord_provides_ac_under_nyc_cool_homes_act;
    if nyc_cool_homes_violation {
        actions.push("NYC Int 0994 of 2024 'Cool Homes for All Act' VIOLATION: bedroom temperature exceeds 78°F during heat-advisory period (June 15-Sept 15 when outdoor temp > 82°F) AND landlord has not provided AC; Housing Maintenance Code violation + ECB civil penalty + tenant rent reduction + HPD complaint exposure.".to_string());
    }

    let bracket_required_not_installed =
        (input.building_taller_than_6_stories || !input.window_frame_supports_unit_weight)
            && !input.bracket_installed;
    if bracket_required_not_installed
        && !matches!(input.install_path, InstallPath::NotApplicable)
    {
        actions.push("Bracket required but NOT installed: building taller than 6 stories OR window frame cannot support unit weight; falling-AC injury liability + insurance denial. Install bracket to manufacturer specifications with fasteners into structural framing.".to_string());
    }

    let bracket_maintenance_deficient = input.bracket_installed
        && !input.bracket_maintained_to_manufacturer_spec;
    if bracket_maintenance_deficient {
        actions.push("Bracket maintenance deficient — not maintained to manufacturer specification: premises liability exposure; annual inspection + re-tightening required pre-cooling season.".to_string());
    }

    let unreasonable_prohibition = input.lease_prohibits_window_ac_no_safety_basis
        && !matches!(input.install_path, InstallPath::LandlordProvided);
    if unreasonable_prohibition {
        actions.push("Lease prohibits window AC without legitimate safety basis: habitability dispute + tenant rescission claim under common-law habitability + state-specific cooling rights. Landlord prohibition must rest on safety + window-frame + structural rationale.".to_string());
    }

    let failure_to_accommodate = input.disability_accommodation_request
        && !input.reasonable_accommodation_granted;
    if failure_to_accommodate {
        actions.push("Reasonable accommodation request DENIED: Fair Housing Act 42 U.S.C. § 3604(f) requires landlord to permit AC installation for disability/health-condition basis absent undue hardship; HUD enforcement + private action + state fair-housing remedies.".to_string());
    }

    match input.jurisdiction {
        Jurisdiction::NewYorkCity => {
            actions.push("New York City: NYC Int 0994 of 2024 'Cool Homes for All Act' (2026 effective) requires landlord-provided AC + bedroom temp ≤78°F during heat advisory period (June 15-Sept 15 when outdoor temp > 82°F); NYC Window AC Bracket requirements for buildings taller than 6 stories; NYC Admin. Code § 27-2029 heating code framework.".to_string());
        }
        Jurisdiction::California => {
            actions.push("California: Cal. Civ. Code § 1941.1 implied warranty of habitability heating; Cal. Health & Safety Code § 17920.3 substandard conditions includes inability to maintain reasonable temperature; select CA cities (Palm Springs + Coachella Valley) require landlord-provided cooling.".to_string());
        }
        Jurisdiction::Arizona => {
            actions.push("Arizona: Phoenix City Code § 39-16 + Pima County Code 8.20 + ARS § 33-1324(C) require landlord-provided cooling in summer months (Phoenix mandate 78°F max June-September); tenant remedies include rent withholding + repair-and-deduct + HUD complaint.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Green v. Superior Court, 10 Cal. 3d 616 (1974); tort negligence + premises liability for heat-stress events.".to_string());
        }
    }

    let severity = if input.falling_ac_injury_reported {
        Severity::FallingAcInjury
    } else if input.heat_stress_event_reported {
        Severity::HeatStressEvent
    } else if failure_to_accommodate {
        Severity::FailureToGrantAccommodation
    } else if nyc_cool_homes_violation {
        Severity::NycCoolHomesActViolation
    } else if unreasonable_prohibition {
        Severity::UnreasonableProhibition
    } else if bracket_required_not_installed {
        Severity::BracketRequiredNotInstalled
    } else if bracket_maintenance_deficient {
        Severity::BracketMaintenanceDeficient
    } else {
        Severity::Compliant
    };

    Output {
        severity,
        jurisdiction_specific_actions: actions,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYorkCity,
            install_path: InstallPath::TenantInstalledWithLease,
            building_taller_than_6_stories: false,
            bracket_installed: false,
            bracket_maintained_to_manufacturer_spec: false,
            window_frame_supports_unit_weight: true,
            lease_prohibits_window_ac_no_safety_basis: false,
            landlord_provides_ac_under_nyc_cool_homes_act: false,
            bedroom_temp_above_78f_during_heat_advisory_period: false,
            heat_stress_event_reported: false,
            falling_ac_injury_reported: false,
            disability_accommodation_request: false,
            reasonable_accommodation_granted: false,
        }
    }

    #[test]
    fn nyc_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn falling_ac_injury_top_severity() {
        let mut i = baseline();
        i.falling_ac_injury_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FallingAcInjury);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("$1M+ settlement"));
    }

    #[test]
    fn heat_stress_event_severity() {
        let mut i = baseline();
        i.heat_stress_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HeatStressEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("500 NYC heat-related deaths annually"));
    }

    #[test]
    fn not_applicable_install_path() {
        let mut i = baseline();
        i.install_path = InstallPath::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn nyc_cool_homes_act_violation() {
        let mut i = baseline();
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        i.landlord_provides_ac_under_nyc_cool_homes_act = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NycCoolHomesActViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Cool Homes for All Act"));
        assert!(joined.contains("78°F"));
        assert!(joined.contains("82°F"));
    }

    #[test]
    fn nyc_cool_homes_compliant_with_ac() {
        let mut i = baseline();
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        i.landlord_provides_ac_under_nyc_cool_homes_act = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ca_no_cool_homes_act_no_violation_on_temp() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        i.landlord_provides_ac_under_nyc_cool_homes_act = false;
        let out = check(&i);
        // CA doesn't have NYC's cool homes mandate; no violation
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn bracket_required_tall_building_not_installed() {
        let mut i = baseline();
        i.building_taller_than_6_stories = true;
        i.bracket_installed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::BracketRequiredNotInstalled);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("taller than 6 stories"));
    }

    #[test]
    fn bracket_required_weak_window_frame_not_installed() {
        let mut i = baseline();
        i.window_frame_supports_unit_weight = false;
        i.bracket_installed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::BracketRequiredNotInstalled);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("window frame cannot support"));
    }

    #[test]
    fn bracket_installed_tall_building_compliant() {
        let mut i = baseline();
        i.building_taller_than_6_stories = true;
        i.bracket_installed = true;
        i.bracket_maintained_to_manufacturer_spec = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn bracket_maintenance_deficient() {
        let mut i = baseline();
        i.bracket_installed = true;
        i.bracket_maintained_to_manufacturer_spec = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::BracketMaintenanceDeficient);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("not maintained to manufacturer specification"));
    }

    #[test]
    fn unreasonable_prohibition() {
        let mut i = baseline();
        i.lease_prohibits_window_ac_no_safety_basis = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::UnreasonableProhibition);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Lease prohibits window AC without legitimate safety basis"));
    }

    #[test]
    fn landlord_provided_prohibition_no_violation() {
        let mut i = baseline();
        i.install_path = InstallPath::LandlordProvided;
        i.lease_prohibits_window_ac_no_safety_basis = true;
        let out = check(&i);
        // Landlord-provided path: no tenant window AC needed, prohibition irrelevant
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn disability_accommodation_denied() {
        let mut i = baseline();
        i.install_path = InstallPath::ReasonableAccommodationDisability;
        i.disability_accommodation_request = true;
        i.reasonable_accommodation_granted = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FailureToGrantAccommodation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("42 U.S.C. § 3604(f)"));
        assert!(joined.contains("undue hardship"));
    }

    #[test]
    fn disability_accommodation_granted_compliant() {
        let mut i = baseline();
        i.install_path = InstallPath::ReasonableAccommodationDisability;
        i.disability_accommodation_request = true;
        i.reasonable_accommodation_granted = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn nyc_jurisdiction_cites_int_0994() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYorkCity;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("NYC Int 0994 of 2024"));
        assert!(joined.contains("Cool Homes for All Act"));
        assert!(joined.contains("June 15-Sept 15"));
        assert!(joined.contains("§ 27-2029"));
    }

    #[test]
    fn ca_jurisdiction_cites_1941_1_and_17920_3() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("§ 17920.3"));
        assert!(joined.contains("Palm Springs"));
    }

    #[test]
    fn az_jurisdiction_cites_phoenix_code_and_ars() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Arizona;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Phoenix City Code § 39-16"));
        assert!(joined.contains("Pima County Code 8.20"));
        assert!(joined.contains("ARS § 33-1324(C)"));
    }

    #[test]
    fn default_jurisdiction_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
    }

    #[test]
    fn severity_priority_falling_above_heat_above_accommodation_above_cool_homes() {
        let mut i = baseline();
        i.falling_ac_injury_reported = true;
        i.heat_stress_event_reported = true;
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        i.disability_accommodation_request = true;
        i.reasonable_accommodation_granted = false;
        i.install_path = InstallPath::ReasonableAccommodationDisability;
        let out = check(&i);
        // Falling AC wins
        assert_eq!(out.severity, Severity::FallingAcInjury);
    }

    #[test]
    fn severity_heat_stress_above_accommodation_above_cool_homes_violation() {
        let mut i = baseline();
        i.heat_stress_event_reported = true;
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HeatStressEvent);
    }

    #[test]
    fn severity_accommodation_above_cool_homes_violation() {
        let mut i = baseline();
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        i.install_path = InstallPath::ReasonableAccommodationDisability;
        i.disability_accommodation_request = true;
        i.reasonable_accommodation_granted = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FailureToGrantAccommodation);
    }

    #[test]
    fn severity_cool_homes_above_prohibition_above_bracket() {
        let mut i = baseline();
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        i.lease_prohibits_window_ac_no_safety_basis = true;
        i.building_taller_than_6_stories = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NycCoolHomesActViolation);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("NYC Int 0994 of 2024"));
        assert!(joined.contains("Cool Homes for All Act"));
        assert!(joined.contains("78°F"));
        assert!(joined.contains("82°F"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("§ 17920.3"));
        assert!(joined.contains("Phoenix City Code § 39-16"));
        assert!(joined.contains("ARS § 33-1324(C)"));
        assert!(joined.contains("42 U.S.C. § 3604(f)"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("New York City (most recent"));
        assert!(joined.contains("California"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_five_bracket_safety_requirements() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("TALLER THAN 6 STORIES"));
        assert!(joined.contains("tilt, slip, or fall"));
        assert!(joined.contains("window frame load capacity"));
        assert!(joined.contains("manufacturer specifications"));
        assert!(joined.contains("annual inspection"));
    }

    #[test]
    fn note_pins_three_install_paths() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("TENANT-INSTALLED WITH LEASE"));
        assert!(joined.contains("LANDLORD-PROVIDED"));
        assert!(joined.contains("REASONABLE ACCOMMODATION"));
        assert!(joined.contains("42 U.S.C. § 3604(f)"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("prohibits window AC despite no legitimate"));
        assert!(joined.contains("without bracket on upper floor"));
        assert!(joined.contains("Cool Homes Act violation"));
        assert!(joined.contains("heat-stress event"));
        assert!(joined.contains("AC bracket not maintained"));
    }

    #[test]
    fn note_pins_500_heat_deaths_annually() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("500 NYC heat-related deaths annually"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("cooling_requirements"));
        assert!(joined.contains("tenant_solar_installation"));
        assert!(joined.contains("rental_window_guard_installation"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        let ny = check(&Input {
            jurisdiction: Jurisdiction::NewYorkCity,
            ..baseline()
        });
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            ..baseline()
        });
        let az = check(&Input {
            jurisdiction: Jurisdiction::Arizona,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(ny.severity, Severity::Compliant);
        assert_eq!(ca.severity, Severity::Compliant);
        assert_eq!(az.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn nyc_uniquely_cool_homes_act_invariant() {
        let ny = check(&Input {
            jurisdiction: Jurisdiction::NewYorkCity,
            bedroom_temp_above_78f_during_heat_advisory_period: true,
            ..baseline()
        });
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            bedroom_temp_above_78f_during_heat_advisory_period: true,
            ..baseline()
        });
        let az = check(&Input {
            jurisdiction: Jurisdiction::Arizona,
            bedroom_temp_above_78f_during_heat_advisory_period: true,
            ..baseline()
        });
        // Only NYC triggers cool homes act violation
        assert_eq!(ny.severity, Severity::NycCoolHomesActViolation);
        assert_eq!(ca.severity, Severity::Compliant);
        assert_eq!(az.severity, Severity::Compliant);
    }

    #[test]
    fn multiple_violations_stack_in_actions() {
        let mut i = baseline();
        i.building_taller_than_6_stories = true;
        i.bracket_installed = false;
        i.lease_prohibits_window_ac_no_safety_basis = true;
        i.bedroom_temp_above_78f_during_heat_advisory_period = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Cool Homes for All Act"));
        assert!(joined.contains("Bracket required"));
        assert!(joined.contains("legitimate safety basis"));
    }
}
