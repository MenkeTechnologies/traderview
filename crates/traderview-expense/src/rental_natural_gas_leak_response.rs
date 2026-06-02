//! Multi-jurisdictional rental property NATURAL GAS LEAK
//! detection, response, and landlord-duty compliance
//! framework. When a tenant reports a natural-gas odor
//! (rotten egg / sulfur smell from mercaptan odorant) or
//! a methane-leak event occurs in or around the rental
//! unit, what immediate response duties attach (evacuate,
//! call 911, call utility, do not ignite), what utility-
//! side and landlord-side obligations interact under
//! PHMSA 49 C.F.R. Part 192, and what failure-mode
//! liabilities expose landlord after an explosion, fire,
//! asphyxiation, or carbon-monoxide event?
//!
//! Distinct from sibling modules: rental_propane_tank_
//! lease_disclosure (iter 475 — LP-gas tank disclosure),
//! rental_carbon_monoxide_detector (CO sensor), rental_
//! hardwired_smoke_alarm_responsibility (iter 481 — smoke
//! detection), rental_chimney_fireplace_inspection_
//! disclosure (iter 471), rental_fire_extinguisher_
//! requirement (iter 473), rental_gas_appliance_ban
//! (electrification policy), rental_water_heater_temperature
//! (different appliance), tenant_emotional_distress_
//! damages (IIED parallel).
//!
//! Four-jurisdiction framework:
//!
//! 1. FEDERAL/PHMSA (universal floor) — 49 C.F.R. Part
//!    192 (Transportation of Natural and Other Gas by
//!    Pipeline: Minimum Federal Safety Standards);
//!    § 192.625 odorization requirement (combustible gas
//!    in distribution line must be odorized with
//!    mercaptan such that 1/5 lower explosive limit
//!    concentration is readily detectable by olfaction);
//!    § 192.706 transmission-line leakage surveys at
//!    intervals not exceeding 15 months with at least
//!    once each calendar year; PHMSA Final Rule January
//!    17, 2025 strengthened leakage survey + patrolling
//!    requirements + advanced leak detection performance
//!    standards + leak grading + mandatory repair
//!    timelines.
//! 2. MASSACHUSETTS — 220 C.M.R. + M.G.L. c. 164 § 105A
//!    (gas safety enacted post-Merrimack Valley 2018
//!    explosion that killed 1 + injured 25 + displaced
//!    thousands across Andover/Lawrence/North Andover);
//!    220 C.M.R. 100.00 + 220 C.M.R. 101.00 gas pipeline
//!    safety; landlord duty to engage utility (Eversource,
//!    National Grid, or Columbia Gas) immediately on
//!    tenant gas-odor complaint.
//! 3. CALIFORNIA — California Public Utilities Commission
//!    (CPUC) gas safety with G.O. 112-F gas pipeline
//!    safety and G.O. 58-A natural gas service standards;
//!    Cal. Civ. Code § 1941.1 implied warranty of
//!    habitability includes operable gas service per
//!    Green v. Superior Court, 10 Cal. 3d 616 (1974);
//!    Cal. Pub. Util. Code § 451 mandates safe and
//!    adequate utility service.
//! 4. DEFAULT — common-law implied warranty of
//!    habitability per Hilder v. St. Peter, 478 A.2d 202
//!    (Vt. 1984); tort negligence + premises liability;
//!    state PUC consumer-protection rules.
//!
//! Universal landlord IMMEDIATE-RESPONSE duties on
//! tenant gas-odor complaint (six-step protocol):
//! 1. EVACUATE all occupants from unit and building IMMEDIATELY
//! 2. Call 911 to dispatch emergency services
//! 3. Call utility emergency line (FROM SAFE LOCATION
//!    outside building) — every major utility maintains
//!    24/7 gas-leak hotline
//! 4. DO NOT operate light switches, electronics, vehicle
//!    ignition, or cell phones inside building (any spark
//!    can ignite gas-air mixture)
//! 5. DO NOT enter unit to investigate or shut off appliances
//! 6. Wait for utility + fire department to clear scene
//!    before any re-entry
//!
//! Post-incident landlord obligations:
//! 1. Engage licensed plumber to inspect all gas appliances
//!    + connections before service restoration
//! 2. Document utility's emergency-response report
//! 3. Notify all tenants of incident + cause + remediation
//! 4. Update lease addenda with gas-emergency procedures
//! 5. Consider methane detector installation (emerging
//!    best practice; not yet mandatory in most states)
//!
//! Universal five failure-mode liability framework:
//! 1. FAILED TO RESPOND to tenant gas-odor complaint →
//!    tort negligence + wrongful death + IIED parallel to
//!    tenant_emotional_distress_damages iter 453
//! 2. OPERATED APPLIANCES AFTER ODOR DETECTED (ignition
//!    source) → explosion liability + criminal negligence
//! 3. FAILED TO EVACUATE TENANTS → bodily injury exposure
//!    + wrongful death + premises liability
//! 4. FAILED TO INSPECT GAS APPLIANCES POST-RESTORATION →
//!    CO + leak recurrence + tenant injury
//! 5. INHERITED UNIT WITH DISABLED METHANE DETECTOR OR
//!    NON-FUNCTIONING ALARMS → emerging best-practice
//!    violation (mandatory detector requirements expected
//!    in MA + CA over next 3-5 years post-Merrimack-Valley)
//!
//! Trader-landlord critical because (1) natural-gas
//! explosions are among the highest-stakes premises-
//! liability claims — Merrimack Valley settlements
//! exceeded $1B aggregate; (2) PHMSA Final Rule
//! (effective from January 17, 2025) introduced
//! mandatory ALDS (advanced leak detection) performance
//! standards and graded repair timelines pushing
//! responsibility further onto utility + property owners;
//! (3) wrong response (operating switches, attempting to
//! shut off gas) is the leading cause of post-leak
//! explosion — landlord training of property managers is
//! the single most important compliance task; (4) methane
//! detectors costs $25-$100 per unit vs explosion
//! liability exposure exceeding $10M+; (5) Massachusetts
//! Merrimack Valley framework is the leading model for
//! anticipated state-level reforms.
//!
//! Authority: 49 C.F.R. Part 192 (Transportation of
//! Natural and Other Gas by Pipeline); 49 C.F.R. § 192.625
//! (odorization); 49 C.F.R. § 192.706 (leakage surveys);
//! PHMSA Final Rule, 90 Fed. Reg. (effective January 17,
//! 2025) — Gas Pipeline Leak Detection and Repair;
//! 220 C.M.R. 100.00; 220 C.M.R. 101.00; M.G.L. c. 164
//! § 105A (Massachusetts post-Merrimack Valley gas
//! safety); CPUC General Order 112-F (gas pipeline
//! safety); CPUC General Order 58-A (natural gas service
//! standards); Cal. Pub. Util. Code § 451; Cal. Civ.
//! Code § 1941.1; Green v. Superior Court, 10 Cal. 3d
//! 616 (1974); Hilder v. St. Peter, 478 A.2d 202 (Vt.
//! 1984).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Federal,
    Massachusetts,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub tenant_gas_odor_complaint_received: bool,
    pub evacuated_immediately: bool,
    pub called_911: bool,
    pub called_utility_emergency_line: bool,
    pub avoided_ignition_sources: bool,
    pub avoided_unit_re_entry_before_clearance: bool,
    pub post_clearance_appliance_inspection_completed: bool,
    pub methane_detector_installed: bool,
    pub explosion_or_injury_event_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NoComplaint,
    Compliant,
    BestPracticeRecommended,
    PostClearanceInspectionRequired,
    ResponseProtocolViolation,
    IgnitionSourceViolation,
    EvacuationFailure,
    ExplosionOrInjuryEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub type RentalNaturalGasLeakResponseInput = Input;
pub type RentalNaturalGasLeakResponseResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: Federal/PHMSA (universal floor — 49 C.F.R. Part 192 + § 192.625 odorization + § 192.706 15-month leakage surveys + PHMSA Final Rule January 17, 2025 strengthened ALDS performance standards and mandatory repair timelines); Massachusetts (220 C.M.R. 100.00 + 220 C.M.R. 101.00 + M.G.L. c. 164 § 105A post-Merrimack Valley 2018 gas safety regime); California (CPUC G.O. 112-F gas pipeline safety + G.O. 58-A natural gas service standards + Cal. Pub. Util. Code § 451 safe-and-adequate-service mandate + Cal. Civ. Code § 1941.1 implied warranty of habitability per Green v. Superior Court, 10 Cal. 3d 616 (1974)); Default (common-law implied warranty per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + tort negligence + premises liability + state PUC consumer protection).".to_string(),
        "Universal landlord IMMEDIATE-RESPONSE six-step protocol on tenant gas-odor complaint: (1) EVACUATE all occupants immediately; (2) call 911; (3) call utility emergency line FROM SAFE LOCATION outside building; (4) DO NOT operate light switches/electronics/vehicle ignition/cell phones inside building (spark = explosion); (5) DO NOT enter unit to investigate or shut off appliances; (6) wait for utility + fire department clearance before re-entry.".to_string(),
        "Post-incident obligations: engage licensed plumber to inspect all gas appliances and connections before service restoration; document utility emergency-response report; notify all tenants of incident + cause + remediation; update lease addenda with gas-emergency procedures; consider methane detector installation ($25-$100 per unit; emerging best practice, not yet mandatory in most states).".to_string(),
        "Five universal failure-mode liabilities: (1) FAILED TO RESPOND → tort negligence + wrongful death + IIED parallel to tenant_emotional_distress_damages iter 453; (2) OPERATED APPLIANCES AFTER ODOR DETECTED → explosion liability + criminal negligence; (3) FAILED TO EVACUATE TENANTS → bodily injury + wrongful death + premises liability; (4) FAILED TO INSPECT GAS APPLIANCES POST-RESTORATION → CO + leak recurrence; (5) INHERITED UNIT WITH DISABLED METHANE DETECTOR → emerging best-practice violation (mandatory detectors expected in MA + CA over next 3-5 years).".to_string(),
        "Merrimack Valley 2018 incident: explosion across Andover + Lawrence + North Andover killed 1 + injured 25 + displaced thousands; aggregate settlements exceeded $1B; led to M.G.L. c. 164 § 105A + nationwide PHMSA reform momentum culminating in January 17, 2025 Final Rule on advanced leak detection and repair.".to_string(),
        "Companion modules: rental_propane_tank_lease_disclosure (iter 475 — LP-gas tank disclosure), rental_carbon_monoxide_detector (CO sensor), rental_hardwired_smoke_alarm_responsibility (iter 481 — smoke detection), rental_chimney_fireplace_inspection_disclosure (iter 471), rental_fire_extinguisher_requirement (iter 473), rental_gas_appliance_ban (electrification policy), tenant_emotional_distress_damages.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if input.explosion_or_injury_event_reported {
        actions.push("Explosion or injury event reported: engage emergency services + counsel; preserve evidence including utility records, appliance models, odorant complaint logs; tort negligence + wrongful death + IIED + Hilder v. St. Peter constructive eviction; aggregate settlement exposure exceeding $10M+ per major incident.".to_string());
    }

    if !input.tenant_gas_odor_complaint_received {
        let mut n = notes;
        n.push("No tenant gas-odor complaint received this period — no active response duty triggered. Recommended baseline preventive measures: methane detector installation (emerging best practice) + annual gas appliance inspection by licensed plumber.".to_string());
        let severity = if input.explosion_or_injury_event_reported {
            Severity::ExplosionOrInjuryEvent
        } else {
            Severity::NoComplaint
        };
        return Output {
            severity,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    if !input.evacuated_immediately {
        actions.push("EVACUATION FAILURE: tenant complaint received but evacuation not completed; immediate occupant evacuation is the single most important response step; bodily injury + wrongful death + premises liability exposure.".to_string());
    }

    if !input.avoided_ignition_sources {
        actions.push("IGNITION-SOURCE VIOLATION: light switches / electronics / vehicle ignition / cell phones operated inside building after gas odor reported — any spark can ignite methane-air mixture; explosion + criminal negligence exposure.".to_string());
    }

    if !input.called_911 || !input.called_utility_emergency_line {
        let mut missing: Vec<&str> = Vec::new();
        if !input.called_911 {
            missing.push("911 emergency services");
        }
        if !input.called_utility_emergency_line {
            missing.push("utility emergency line");
        }
        actions.push(format!(
            "Response protocol violation: failed to call {} from safe location outside building.",
            missing.join(" + ")
        ));
    }

    if !input.avoided_unit_re_entry_before_clearance {
        actions.push("Unit re-entry before utility + fire department clearance: explosion + bodily injury exposure; await official scene-clear authorization.".to_string());
    }

    if !input.post_clearance_appliance_inspection_completed {
        actions.push("Post-clearance appliance inspection not completed: engage licensed plumber to inspect all gas appliances and connections before service restoration; CO + leak-recurrence exposure under common-law habitability + § 1941.1 implied warranty.".to_string());
    }

    if !input.methane_detector_installed {
        actions.push("Methane detector NOT installed: emerging best practice ($25-$100 per unit cost vs explosion liability exposure exceeding $10M+); mandatory detector requirements expected in MA + CA over next 3-5 years post-Merrimack-Valley.".to_string());
    }

    match input.jurisdiction {
        Jurisdiction::Federal => {
            actions.push("Federal/PHMSA: 49 C.F.R. Part 192 + § 192.625 odorization + § 192.706 15-month leakage surveys + PHMSA Final Rule January 17, 2025 advanced leak detection + mandatory repair timelines.".to_string());
        }
        Jurisdiction::Massachusetts => {
            actions.push("Massachusetts: 220 C.M.R. 100.00 + 220 C.M.R. 101.00 + M.G.L. c. 164 § 105A post-Merrimack Valley 2018 gas safety regime; engage Eversource / National Grid / Columbia Gas emergency line immediately.".to_string());
        }
        Jurisdiction::California => {
            actions.push("California: CPUC G.O. 112-F + G.O. 58-A + Cal. Pub. Util. Code § 451 safe-and-adequate-service mandate + Cal. Civ. Code § 1941.1 implied warranty; engage PG&E / SoCalGas / SDG&E emergency line immediately.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + tort negligence + premises liability + state PUC consumer-protection rules.".to_string());
        }
    }

    let severity = if input.explosion_or_injury_event_reported {
        Severity::ExplosionOrInjuryEvent
    } else if !input.evacuated_immediately {
        Severity::EvacuationFailure
    } else if !input.avoided_ignition_sources {
        Severity::IgnitionSourceViolation
    } else if !input.called_911
        || !input.called_utility_emergency_line
        || !input.avoided_unit_re_entry_before_clearance
    {
        Severity::ResponseProtocolViolation
    } else if !input.post_clearance_appliance_inspection_completed {
        Severity::PostClearanceInspectionRequired
    } else if !input.methane_detector_installed {
        Severity::BestPracticeRecommended
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
            jurisdiction: Jurisdiction::Massachusetts,
            tenant_gas_odor_complaint_received: true,
            evacuated_immediately: true,
            called_911: true,
            called_utility_emergency_line: true,
            avoided_ignition_sources: true,
            avoided_unit_re_entry_before_clearance: true,
            post_clearance_appliance_inspection_completed: true,
            methane_detector_installed: true,
            explosion_or_injury_event_reported: false,
        }
    }

    #[test]
    fn no_complaint_no_active_duty() {
        let mut i = baseline();
        i.tenant_gas_odor_complaint_received = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoComplaint);
    }

    #[test]
    fn ma_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn explosion_event_top_severity() {
        let mut i = baseline();
        i.explosion_or_injury_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExplosionOrInjuryEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("$10M+"));
        assert!(joined.contains("Hilder v. St. Peter"));
    }

    #[test]
    fn evacuation_failure_severity() {
        let mut i = baseline();
        i.evacuated_immediately = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::EvacuationFailure);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("EVACUATION FAILURE"));
        assert!(joined.contains("single most important"));
    }

    #[test]
    fn ignition_source_violation() {
        let mut i = baseline();
        i.avoided_ignition_sources = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::IgnitionSourceViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("IGNITION-SOURCE VIOLATION"));
        assert!(joined.contains("methane-air mixture"));
    }

    #[test]
    fn failed_to_call_911_protocol_violation() {
        let mut i = baseline();
        i.called_911 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ResponseProtocolViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("911 emergency services"));
    }

    #[test]
    fn failed_to_call_utility_protocol_violation() {
        let mut i = baseline();
        i.called_utility_emergency_line = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ResponseProtocolViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("utility emergency line"));
    }

    #[test]
    fn unit_re_entry_before_clearance_violation() {
        let mut i = baseline();
        i.avoided_unit_re_entry_before_clearance = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ResponseProtocolViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Unit re-entry before utility"));
    }

    #[test]
    fn post_clearance_inspection_required() {
        let mut i = baseline();
        i.post_clearance_appliance_inspection_completed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PostClearanceInspectionRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Post-clearance appliance inspection"));
    }

    #[test]
    fn methane_detector_missing_best_practice_recommendation() {
        let mut i = baseline();
        i.methane_detector_installed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::BestPracticeRecommended);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("$25-$100"));
        assert!(joined.contains("Merrimack-Valley"));
    }

    #[test]
    fn federal_jurisdiction_cites_phmsa() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Federal;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("49 C.F.R. Part 192"));
        assert!(joined.contains("§ 192.625"));
        assert!(joined.contains("§ 192.706"));
        assert!(joined.contains("January 17, 2025"));
    }

    #[test]
    fn ma_jurisdiction_cites_220_cmr_and_164_105a() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("220 C.M.R. 100.00"));
        assert!(joined.contains("220 C.M.R. 101.00"));
        assert!(joined.contains("M.G.L. c. 164 § 105A"));
        assert!(joined.contains("Eversource"));
        assert!(joined.contains("National Grid"));
        assert!(joined.contains("Columbia Gas"));
    }

    #[test]
    fn ca_jurisdiction_cites_cpuc() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("CPUC G.O. 112-F"));
        assert!(joined.contains("G.O. 58-A"));
        assert!(joined.contains("§ 451"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("PG&E"));
        assert!(joined.contains("SoCalGas"));
        assert!(joined.contains("SDG&E"));
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
    fn severity_priority_explosion_above_evacuation_above_ignition() {
        let mut i = baseline();
        i.explosion_or_injury_event_reported = true;
        i.evacuated_immediately = false;
        i.avoided_ignition_sources = false;
        let out = check(&i);
        // Explosion wins
        assert_eq!(out.severity, Severity::ExplosionOrInjuryEvent);
    }

    #[test]
    fn severity_evacuation_above_ignition_above_protocol() {
        let mut i = baseline();
        i.evacuated_immediately = false;
        i.avoided_ignition_sources = false;
        i.called_911 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::EvacuationFailure);
    }

    #[test]
    fn severity_ignition_above_protocol_above_inspection() {
        let mut i = baseline();
        i.avoided_ignition_sources = false;
        i.called_911 = false;
        i.post_clearance_appliance_inspection_completed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::IgnitionSourceViolation);
    }

    #[test]
    fn severity_protocol_above_inspection_above_best_practice() {
        let mut i = baseline();
        i.called_911 = false;
        i.post_clearance_appliance_inspection_completed = false;
        i.methane_detector_installed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ResponseProtocolViolation);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("49 C.F.R. Part 192"));
        assert!(joined.contains("§ 192.625"));
        assert!(joined.contains("§ 192.706"));
        assert!(joined.contains("January 17, 2025"));
        assert!(joined.contains("220 C.M.R. 100.00"));
        assert!(joined.contains("220 C.M.R. 101.00"));
        assert!(joined.contains("M.G.L. c. 164 § 105A"));
        assert!(joined.contains("CPUC G.O. 112-F"));
        assert!(joined.contains("G.O. 58-A"));
        assert!(joined.contains("§ 451"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("Green v. Superior Court"));
        assert!(joined.contains("10 Cal. 3d 616"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Federal/PHMSA (universal floor"));
        assert!(joined.contains("Massachusetts"));
        assert!(joined.contains("California"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_six_step_protocol() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("EVACUATE all occupants"));
        assert!(joined.contains("call 911"));
        assert!(joined.contains("utility emergency line"));
        assert!(joined.contains("DO NOT operate"));
        assert!(joined.contains("DO NOT enter unit"));
        assert!(joined.contains("clearance"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("FAILED TO RESPOND"));
        assert!(joined.contains("OPERATED APPLIANCES"));
        assert!(joined.contains("FAILED TO EVACUATE"));
        assert!(joined.contains("FAILED TO INSPECT"));
        assert!(joined.contains("DISABLED METHANE DETECTOR"));
    }

    #[test]
    fn note_pins_merrimack_valley_context() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Merrimack Valley 2018"));
        assert!(joined.contains("Andover"));
        assert!(joined.contains("Lawrence"));
        assert!(joined.contains("North Andover"));
        assert!(joined.contains("$1B"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_propane_tank_lease_disclosure"));
        assert!(joined.contains("rental_carbon_monoxide_detector"));
        assert!(joined.contains("rental_hardwired_smoke_alarm_responsibility"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        let fed = check(&Input {
            jurisdiction: Jurisdiction::Federal,
            ..baseline()
        });
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            ..baseline()
        });
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(fed.severity, Severity::Compliant);
        assert_eq!(ma.severity, Severity::Compliant);
        assert_eq!(ca.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn ma_uniquely_cites_merrimack_valley_utilities() {
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            ..baseline()
        });
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            ..baseline()
        });
        let joined_ma = ma.jurisdiction_specific_actions.join(" ");
        let joined_ca = ca.jurisdiction_specific_actions.join(" ");
        assert!(joined_ma.contains("Eversource"));
        assert!(joined_ca.contains("PG&E"));
        // CA does NOT mention Eversource (MA utility)
        assert!(!joined_ca.contains("Eversource"));
    }

    #[test]
    fn multiple_violations_stack_in_actions() {
        let mut i = baseline();
        i.evacuated_immediately = false;
        i.avoided_ignition_sources = false;
        i.called_911 = false;
        i.called_utility_emergency_line = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("EVACUATION FAILURE"));
        assert!(joined.contains("IGNITION-SOURCE VIOLATION"));
        assert!(joined.contains("911"));
        assert!(joined.contains("utility emergency line"));
    }
}
