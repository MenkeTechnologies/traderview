//! Multi-jurisdictional rental property PRIVATE WELL
//! WATER testing + disclosure compliance framework. When
//! a landlord rents a property served by a private water
//! well rather than a municipal/community water supply,
//! what testing schedule applies, what contaminants must
//! be tested for, what disclosure must be given to tenant,
//! and what failure-mode liabilities expose landlord?
//!
//! Distinct from sibling modules: rental_septic_system_
//! disclosure (iter 465 — OSTDS/septic), rental_lead_pipe_
//! disclosure (lead service lines), rental_basement_water_
//! intrusion_disclosure (water intrusion), rental_water_
//! submetering_disclosure (water billing).
//!
//! Four-jurisdiction framework:
//!
//! 1. NEW JERSEY (most stringent — first-in-nation
//!    landmark) — Private Well Testing Act ("PWTA"),
//!    N.J.S.A. 58:12A-26 et seq., signed into law March
//!    2001, effective September 2002. N.J.S.A. 58:12A-32
//!    "Lessor's water testing responsibilities for private
//!    wells" requires LESSOR to obtain and pay for full
//!    PWTA test once every FIVE YEARS, and to provide a
//!    written copy of test results to each rental unit on
//!    the property WITHIN 30 DAYS of receipt. New lessees
//!    receive the most recent test results before lease
//!    execution. PWTA test panel under N.J.A.C. 7:9E
//!    includes: total coliform bacteria, iron, manganese,
//!    pH, ALL volatile organic compounds (VOCs) with
//!    established Maximum Contaminant Levels, nitrate,
//!    lead, gross alpha radioactivity, plus county-
//!    specific additions (arsenic statewide since 2021;
//!    radon in northern counties; PFAS testing recently
//!    added by NJDEP rule).
//! 2. CONNECTICUT — Conn. Gen. Stat. § 19a-37
//!    "Regulation of water supply wells" + Public Act
//!    16-66 (2016) mandated landlord-paid testing for new
//!    construction wells; Conn. Public Health Code
//!    § 19-13-B51d outlines testing standards. Testing
//!    typically required at well construction and on
//!    transfer of title; landlord-tenant rental disclosure
//!    less stringent than NJ but common-law habitability
//!    + Conn. Gen. Stat. § 47a-7 applies.
//! 3. PENNSYLVANIA — NO state-level rental disclosure
//!    statute for private wells. PA DEP voluntary testing
//!    recommendations under 25 Pa. Code Ch. 109 cover
//!    public water systems only; private wells fall under
//!    common-law landlord duties + Pa. Const. art. I, § 27
//!    Environmental Rights Amendment (limited rental
//!    application).
//! 4. DEFAULT — Most states fall back on (a) federal Safe
//!    Drinking Water Act, 42 U.S.C. § 300f et seq., which
//!    EXPLICITLY excludes private wells serving fewer than
//!    25 individuals or 15 service connections from
//!    regulation (the "private well exemption"); (b)
//!    common-law implied warranty of habitability
//!    (Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984);
//!    Green v. Superior Court, 10 Cal. 3d 616 (1974));
//!    (c) state implied warranty of sanitary facilities
//!    (e.g., Cal. Civ. Code § 1941.1).
//!
//! Universal contaminant panel (PWTA-style baseline):
//! 1. Total coliform bacteria (indicator of pathogen
//!    contamination)
//! 2. Nitrate / nitrite (Methemoglobinemia / blue-baby
//!    syndrome risk; agricultural runoff indicator)
//! 3. Lead (Pb) — corrosion of plumbing
//! 4. Arsenic (As) — geological origin, statewide NJ
//! 5. Volatile organic compounds (VOCs) — TCE, PCE,
//!    benzene, MTBE (industrial / fuel-tank indicator)
//! 6. pH and corrosivity (drives lead leaching)
//! 7. Iron and manganese (aesthetic + secondary MCLs)
//! 8. Gross alpha radioactivity (radium / uranium)
//! 9. PFAS (per- and poly-fluoroalkyl substances) — added
//!    in NJ + several other states recently
//!
//! Universal failure-mode liability framework:
//! 1. Failure to test or disclose → habitability breach +
//!    constructive eviction (Hilder v. St. Peter, 478
//!    A.2d 202 (Vt. 1984)) + rent abatement (see rent_
//!    abatement_construction_nuisance) + tenant rescission
//!    right in NJ
//! 2. Contaminant exceedance of MCL → habitability breach
//!    with landlord duty to remediate via point-of-entry
//!    treatment system, well deepening, or connection to
//!    municipal supply
//! 3. Lead exceedance combined with child residents → lead
//!    paint and water poisoning class of claims (parallel
//!    to rental_lead_pipe_disclosure)
//! 4. PFAS exceedance → emerging litigation theme; CERCLA
//!    designation of PFOA and PFOS as hazardous substances
//!    (EPA April 2024) opens 42 U.S.C. § 9607(a) owner/
//!    operator strict liability
//! 5. Well casing or pressure-tank failure during tenancy
//!    → loss-of-water habitability breach + temporary
//!    relocation (see mid_tenancy_temporary_relocation)
//!
//! Trader-landlord critical because (1) New Jersey PWTA
//! lessor obligation is STRICT — failure to test costs
//! $5,000 statutory penalty per N.J.S.A. 58:12A-31 plus
//! tenant rescission; (2) inherited rural property in NJ
//! must complete PWTA test BEFORE rental; (3) PFAS
//! exceedance from agricultural / military / firefighting-
//! foam source can render a property unrentable until
//! point-of-entry treatment installed ($3K-$8K capex);
//! (4) Connecticut + Massachusetts properties have
//! growing private-well disclosure expectations driven by
//! arsenic + radon + PFAS public-health litigation;
//! (5) Pennsylvania trader-landlords face NO state
//! statutory regime but inherit common-law habitability
//! risk on top of newly-released EPA PFAS National
//! Primary Drinking Water Regulation (April 10, 2024)
//! that sets MCLs for PFOA / PFOS / GenX / PFNA / PFHxS.
//!
//! Authority: N.J.S.A. 58:12A-26; N.J.S.A. 58:12A-27;
//! N.J.S.A. 58:12A-28; N.J.S.A. 58:12A-29; N.J.S.A.
//! 58:12A-30; N.J.S.A. 58:12A-31; N.J.S.A. 58:12A-32
//! (lessor responsibility); N.J.A.C. 7:9E (PWTA Rule);
//! Conn. Gen. Stat. § 19a-37; Conn. Public Act 16-66
//! (2016); Conn. Public Health Code § 19-13-B51d;
//! 25 Pa. Code Ch. 109; Cal. Civ. Code § 1941.1; Hilder
//! v. St. Peter, 478 A.2d 202 (Vt. 1984); Green v.
//! Superior Court, 10 Cal. 3d 616 (1974); Safe Drinking
//! Water Act, 42 U.S.C. § 300f et seq.; EPA PFAS National
//! Primary Drinking Water Regulation, 89 Fed. Reg. 32532
//! (April 26, 2024); CERCLA 42 U.S.C. § 9607(a); EPA
//! April 2024 PFOA / PFOS hazardous-substance designation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewJersey,
    Connecticut,
    Pennsylvania,
    Default,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub property_served_by_private_well: bool,
    pub last_full_panel_test_months_ago: u32,
    pub test_results_provided_to_tenant_within_30_days: bool,
    pub test_results_provided_at_lease_execution: bool,
    pub coliform_exceeds_mcl: bool,
    pub nitrate_exceeds_mcl: bool,
    pub lead_exceeds_action_level: bool,
    pub arsenic_exceeds_mcl: bool,
    pub voc_exceeds_mcl: bool,
    pub pfas_exceeds_2024_epa_mcl: bool,
    pub gross_alpha_exceeds_mcl: bool,
    pub loss_of_water_event_reported: bool,
    pub remediation_system_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    DisclosureRequired,
    TestingOverdue,
    ContaminantExceedance,
    HabitabilityFailure,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub const NJ_PWTA_TEST_CYCLE_MONTHS: u32 = 60;
pub const NJ_DISCLOSURE_WINDOW_DAYS: u32 = 30;

pub type RentalWellWaterDisclosureInput = Input;
pub type RentalWellWaterDisclosureResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = vec![
        "Four-jurisdiction framework: New Jersey (most stringent — Private Well Testing Act N.J.S.A. 58:12A-26 et seq. + N.J.S.A. 58:12A-32 lessor responsibility, 5-year test cycle + 30-day disclosure window + $5,000 statutory penalty under N.J.S.A. 58:12A-31), Connecticut (Conn. Gen. Stat. § 19a-37 + Public Act 16-66 + Public Health Code § 19-13-B51d), Pennsylvania (no state rental disclosure statute; PA DEP voluntary recommendations under 25 Pa. Code Ch. 109; common-law habitability), Default (Safe Drinking Water Act 42 U.S.C. § 300f private-well exemption + Hilder v. St. Peter common-law habitability + Cal. Civ. Code § 1941.1 implied warranty of sanitary facilities).".to_string(),
        "New Jersey PWTA N.J.A.C. 7:9E contaminant panel: total coliform bacteria + iron + manganese + pH + all volatile organic compounds (VOCs) with established Maximum Contaminant Levels + nitrate + lead + gross alpha radioactivity + arsenic statewide (since 2021) + county-specific additions (radon in northern counties; PFAS recently added by NJDEP rule).".to_string(),
        "Universal nine-contaminant baseline panel: total coliform bacteria + nitrate/nitrite + lead + arsenic + VOCs (TCE/PCE/benzene/MTBE) + pH/corrosivity + iron/manganese + gross alpha radioactivity + PFAS (added in NJ + several states after EPA April 10, 2024 National Primary Drinking Water Regulation MCLs for PFOA/PFOS/GenX/PFNA/PFHxS).".to_string(),
        "Five universal failure-mode liabilities: (1) failure to test/disclose → habitability breach + constructive eviction (Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984)) + tenant rescission right in NJ; (2) MCL exceedance → landlord remediation duty; (3) lead + child residents → lead poisoning class-of-claims parallel to rental_lead_pipe_disclosure; (4) PFAS exceedance → emerging litigation + CERCLA 42 U.S.C. § 9607(a) strict liability after EPA April 2024 PFOA/PFOS hazardous-substance designation; (5) well casing/pressure tank failure → loss-of-water habitability breach.".to_string(),
        "Companion modules: rental_septic_system_disclosure (iter 465 — OSTDS/septic), rental_lead_pipe_disclosure (lead service lines), rental_basement_water_intrusion_disclosure, rental_water_submetering_disclosure, rent_abatement_construction_nuisance, mid_tenancy_temporary_relocation.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if !input.property_served_by_private_well {
        notes.push("Property is on municipal/community water supply — private well disclosure not applicable.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes,
        };
    }

    let mut testing_overdue = false;
    let mut disclosure_missing = false;

    match input.jurisdiction {
        Jurisdiction::NewJersey => {
            if input.last_full_panel_test_months_ago > NJ_PWTA_TEST_CYCLE_MONTHS {
                testing_overdue = true;
                actions.push(format!(
                    "New Jersey PWTA: full panel test {} months old exceeds 60-month (5-year) cycle per N.J.S.A. 58:12A-32. Statutory penalty up to $5,000 per N.J.S.A. 58:12A-31. Engage NJDEP-certified laboratory.",
                    input.last_full_panel_test_months_ago
                ));
            }
            if !input.test_results_provided_to_tenant_within_30_days {
                disclosure_missing = true;
                actions.push("New Jersey PWTA: written test results MUST be provided to each rental unit within 30 days of receipt per N.J.S.A. 58:12A-32.".to_string());
            }
            if !input.test_results_provided_at_lease_execution {
                disclosure_missing = true;
                actions.push("New Jersey PWTA: most recent test results MUST be provided to NEW lessee at lease execution per N.J.S.A. 58:12A-32.".to_string());
            }
        }
        Jurisdiction::Connecticut => {
            actions.push("Connecticut: Conn. Gen. Stat. § 19a-37 + Public Act 16-66 + Public Health Code § 19-13-B51d — testing required at well construction and title transfer; rental disclosure obligations grounded in Conn. Gen. Stat. § 47a-7 common-law habitability.".to_string());
        }
        Jurisdiction::Pennsylvania => {
            actions.push("Pennsylvania: NO state-level rental disclosure statute for private wells; PA DEP voluntary testing recommendations under 25 Pa. Code Ch. 109 (public water systems only); landlord common-law habitability duty plus Pa. Const. art. I § 27 Environmental Rights Amendment.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: Safe Drinking Water Act 42 U.S.C. § 300f exempts private wells (< 25 individuals / 15 connections); landlord falls back on common-law habitability + Cal. Civ. Code § 1941.1 implied warranty of sanitary facilities; pump-out / test every 3-5 years industry standard.".to_string());
        }
    }

    let any_contaminant_exceedance = input.coliform_exceeds_mcl
        || input.nitrate_exceeds_mcl
        || input.lead_exceeds_action_level
        || input.arsenic_exceeds_mcl
        || input.voc_exceeds_mcl
        || input.pfas_exceeds_2024_epa_mcl
        || input.gross_alpha_exceeds_mcl;

    if input.coliform_exceeds_mcl {
        actions.push("Total coliform bacteria exceeds MCL — pathogen contamination indicator; remediate via chlorination shock-treatment + retest within 2 weeks; if persistent, install UV disinfection system + advise tenant boil-water advisory.".to_string());
    }
    if input.nitrate_exceeds_mcl {
        actions.push("Nitrate exceeds 10 mg/L MCL — risk to infants under 6 months (methemoglobinemia / blue-baby syndrome); install reverse-osmosis point-of-entry treatment; warn pregnant women + nursing mothers + infants.".to_string());
    }
    if input.lead_exceeds_action_level {
        actions.push("Lead exceeds EPA 15 ppb action level — corrosion-control treatment + pipe replacement; if children under 6 reside, parallel obligations under rental_lead_pipe_disclosure + state lead-paint disclosure regimes.".to_string());
    }
    if input.arsenic_exceeds_mcl {
        actions.push("Arsenic exceeds 10 µg/L MCL — Group 1 IARC carcinogen; install point-of-entry adsorption / reverse-osmosis treatment; NJ statewide testing requirement since 2021.".to_string());
    }
    if input.voc_exceeds_mcl {
        actions.push("Volatile organic compounds (VOCs) exceed MCL — TCE/PCE/benzene/MTBE indicator of industrial / underground storage tank contamination; coordinate with rental_underground_storage_tank_disclosure; install granular activated carbon (GAC) point-of-entry treatment.".to_string());
    }
    if input.pfas_exceeds_2024_epa_mcl {
        actions.push("PFAS exceeds EPA April 10, 2024 National Primary Drinking Water Regulation MCLs (PFOA 4.0 ppt + PFOS 4.0 ppt + GenX/HFPO-DA 10 ppt + PFNA 10 ppt + PFHxS 10 ppt) — install GAC + ion-exchange + reverse-osmosis treatment; CERCLA 42 U.S.C. § 9607(a) owner/operator strict liability exposure after EPA April 2024 PFOA/PFOS hazardous-substance designation.".to_string());
    }
    if input.gross_alpha_exceeds_mcl {
        actions.push("Gross alpha radioactivity exceeds 15 pCi/L MCL — radium / uranium contamination; install ion-exchange or reverse-osmosis treatment; coordinate with state radiation control program.".to_string());
    }

    if input.loss_of_water_event_reported {
        actions.push("Loss-of-water event reported: well casing or pressure-tank failure; habitability breach + temporary relocation duty (see mid_tenancy_temporary_relocation); engage licensed well driller; offer rent abatement.".to_string());
    }

    let severity = if input.loss_of_water_event_reported {
        Severity::HabitabilityFailure
    } else if any_contaminant_exceedance && !input.remediation_system_installed {
        Severity::ContaminantExceedance
    } else if testing_overdue {
        Severity::TestingOverdue
    } else if disclosure_missing {
        Severity::DisclosureRequired
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
            jurisdiction: Jurisdiction::NewJersey,
            property_served_by_private_well: true,
            last_full_panel_test_months_ago: 12,
            test_results_provided_to_tenant_within_30_days: true,
            test_results_provided_at_lease_execution: true,
            coliform_exceeds_mcl: false,
            nitrate_exceeds_mcl: false,
            lead_exceeds_action_level: false,
            arsenic_exceeds_mcl: false,
            voc_exceeds_mcl: false,
            pfas_exceeds_2024_epa_mcl: false,
            gross_alpha_exceeds_mcl: false,
            loss_of_water_event_reported: false,
            remediation_system_installed: false,
        }
    }

    #[test]
    fn property_on_municipal_water_not_applicable() {
        let mut i = baseline();
        i.property_served_by_private_well = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn nj_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn nj_pwta_test_over_60_months_overdue() {
        let mut i = baseline();
        i.last_full_panel_test_months_ago = 72;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TestingOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("60-month (5-year)"));
        assert!(joined.contains("N.J.S.A. 58:12A-32"));
        assert!(joined.contains("$5,000"));
        assert!(joined.contains("N.J.S.A. 58:12A-31"));
    }

    #[test]
    fn nj_pwta_test_exactly_60_months_compliant() {
        let mut i = baseline();
        i.last_full_panel_test_months_ago = 60;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn nj_pwta_missing_30_day_disclosure() {
        let mut i = baseline();
        i.test_results_provided_to_tenant_within_30_days = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DisclosureRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("30 days"));
    }

    #[test]
    fn nj_pwta_missing_lease_execution_disclosure() {
        let mut i = baseline();
        i.test_results_provided_at_lease_execution = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DisclosureRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("NEW lessee at lease execution"));
    }

    #[test]
    fn coliform_exceedance_contaminant_severity() {
        let mut i = baseline();
        i.coliform_exceeds_mcl = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ContaminantExceedance);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("chlorination shock-treatment"));
        assert!(joined.contains("UV disinfection"));
    }

    #[test]
    fn nitrate_exceedance_warns_methemoglobinemia() {
        let mut i = baseline();
        i.nitrate_exceeds_mcl = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ContaminantExceedance);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("methemoglobinemia"));
        assert!(joined.contains("blue-baby syndrome"));
        assert!(joined.contains("10 mg/L"));
    }

    #[test]
    fn lead_exceedance_15_ppb_action_level() {
        let mut i = baseline();
        i.lead_exceeds_action_level = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ContaminantExceedance);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("15 ppb"));
        assert!(joined.contains("rental_lead_pipe_disclosure"));
    }

    #[test]
    fn arsenic_exceedance_iarc_carcinogen() {
        let mut i = baseline();
        i.arsenic_exceeds_mcl = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ContaminantExceedance);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Group 1 IARC carcinogen"));
        assert!(joined.contains("NJ statewide testing requirement since 2021"));
    }

    #[test]
    fn voc_exceedance_cross_references_underground_storage_tank() {
        let mut i = baseline();
        i.voc_exceeds_mcl = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("TCE/PCE/benzene/MTBE"));
        assert!(joined.contains("rental_underground_storage_tank_disclosure"));
        assert!(joined.contains("granular activated carbon"));
    }

    #[test]
    fn pfas_exceedance_2024_epa_rule_with_cercla_liability() {
        let mut i = baseline();
        i.pfas_exceeds_2024_epa_mcl = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("EPA April 10, 2024"));
        assert!(joined.contains("PFOA 4.0 ppt"));
        assert!(joined.contains("PFOS 4.0 ppt"));
        assert!(joined.contains("GenX/HFPO-DA 10 ppt"));
        assert!(joined.contains("CERCLA"));
        assert!(joined.contains("§ 9607(a)"));
    }

    #[test]
    fn gross_alpha_exceedance_radium_uranium() {
        let mut i = baseline();
        i.gross_alpha_exceeds_mcl = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("15 pCi/L"));
        assert!(joined.contains("radium"));
        assert!(joined.contains("uranium"));
    }

    #[test]
    fn loss_of_water_event_habitability_failure_severity() {
        let mut i = baseline();
        i.loss_of_water_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityFailure);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("well casing or pressure-tank failure"));
        assert!(joined.contains("mid_tenancy_temporary_relocation"));
    }

    #[test]
    fn remediation_installed_suppresses_contaminant_severity() {
        let mut i = baseline();
        i.coliform_exceeds_mcl = true;
        i.remediation_system_installed = true;
        let out = check(&i);
        assert_ne!(out.severity, Severity::ContaminantExceedance);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn connecticut_jurisdiction_cites_authorities() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Connecticut;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 19a-37"));
        assert!(joined.contains("Public Act 16-66"));
        assert!(joined.contains("§ 19-13-B51d"));
        assert!(joined.contains("§ 47a-7"));
    }

    #[test]
    fn pennsylvania_jurisdiction_no_state_statute() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Pennsylvania;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("NO state-level rental disclosure"));
        assert!(joined.contains("25 Pa. Code Ch. 109"));
        assert!(joined.contains("Pa. Const. art. I § 27"));
    }

    #[test]
    fn default_jurisdiction_sdwa_private_well_exemption() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("42 U.S.C. § 300f"));
        assert!(joined.contains("25 individuals / 15 connections"));
        assert!(joined.contains("Cal. Civ. Code § 1941.1"));
    }

    #[test]
    fn severity_priority_failure_above_contaminant_above_testing_above_disclosure() {
        let mut i = baseline();
        i.loss_of_water_event_reported = true;
        i.coliform_exceeds_mcl = true;
        i.last_full_panel_test_months_ago = 72;
        i.test_results_provided_to_tenant_within_30_days = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::HabitabilityFailure);
    }

    #[test]
    fn severity_contaminant_above_testing_when_no_failure() {
        let mut i = baseline();
        i.coliform_exceeds_mcl = true;
        i.last_full_panel_test_months_ago = 72;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ContaminantExceedance);
    }

    #[test]
    fn severity_testing_above_disclosure() {
        let mut i = baseline();
        i.last_full_panel_test_months_ago = 72;
        i.test_results_provided_to_tenant_within_30_days = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TestingOverdue);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("N.J.S.A. 58:12A-26"));
        assert!(joined.contains("N.J.S.A. 58:12A-32"));
        assert!(joined.contains("N.J.S.A. 58:12A-31"));
        assert!(joined.contains("N.J.A.C. 7:9E"));
        assert!(joined.contains("§ 19a-37"));
        assert!(joined.contains("Public Act 16-66"));
        assert!(joined.contains("25 Pa. Code Ch. 109"));
        assert!(joined.contains("42 U.S.C. § 300f"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("EPA April 10, 2024"));
        assert!(joined.contains("PFOA"));
        assert!(joined.contains("PFOS"));
        assert!(joined.contains("§ 9607(a)"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("New Jersey (most stringent"));
        assert!(joined.contains("Connecticut"));
        assert!(joined.contains("Pennsylvania"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_nine_contaminant_baseline() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("total coliform"));
        assert!(joined.contains("nitrate"));
        assert!(joined.contains("lead"));
        assert!(joined.contains("arsenic"));
        assert!(joined.contains("VOCs"));
        assert!(joined.contains("pH"));
        assert!(joined.contains("iron"));
        assert!(joined.contains("manganese"));
        assert!(joined.contains("gross alpha"));
        assert!(joined.contains("PFAS"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("failure to test"));
        assert!(joined.contains("MCL exceedance"));
        assert!(joined.contains("lead + child"));
        assert!(joined.contains("PFAS exceedance"));
        assert!(joined.contains("well casing"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_septic_system_disclosure"));
        assert!(joined.contains("rental_lead_pipe_disclosure"));
        assert!(joined.contains("rental_basement_water_intrusion_disclosure"));
        assert!(joined.contains("rental_water_submetering_disclosure"));
        assert!(joined.contains("rent_abatement_construction_nuisance"));
    }

    #[test]
    fn nj_uniquely_strictest_invariant() {
        // Same fact: 72-month-old test
        let nj = check(&Input {
            jurisdiction: Jurisdiction::NewJersey,
            last_full_panel_test_months_ago: 72,
            ..baseline()
        });
        let ct = check(&Input {
            jurisdiction: Jurisdiction::Connecticut,
            last_full_panel_test_months_ago: 72,
            ..baseline()
        });
        let pa = check(&Input {
            jurisdiction: Jurisdiction::Pennsylvania,
            last_full_panel_test_months_ago: 72,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            last_full_panel_test_months_ago: 72,
            ..baseline()
        });
        // Only NJ triggers; CT/PA/Default don't have statutory cycle
        assert_eq!(nj.severity, Severity::TestingOverdue);
        assert_eq!(ct.severity, Severity::Compliant);
        assert_eq!(pa.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        let nj = check(&Input {
            jurisdiction: Jurisdiction::NewJersey,
            last_full_panel_test_months_ago: 72,
            ..baseline()
        });
        let ct = check(&Input {
            jurisdiction: Jurisdiction::Connecticut,
            ..baseline()
        });
        let pa = check(&Input {
            jurisdiction: Jurisdiction::Pennsylvania,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(nj.severity, Severity::TestingOverdue);
        assert_eq!(ct.severity, Severity::Compliant);
        assert_eq!(pa.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn multiple_contaminants_stack() {
        let mut i = baseline();
        i.coliform_exceeds_mcl = true;
        i.nitrate_exceeds_mcl = true;
        i.lead_exceeds_action_level = true;
        i.pfas_exceeds_2024_epa_mcl = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ContaminantExceedance);
        let joined = out.jurisdiction_specific_actions.join(" ");
        // All four contaminant warnings present
        assert!(joined.contains("UV disinfection"));
        assert!(joined.contains("blue-baby"));
        assert!(joined.contains("15 ppb"));
        assert!(joined.contains("EPA April 10, 2024"));
    }
}
