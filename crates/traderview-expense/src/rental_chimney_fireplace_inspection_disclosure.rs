//! Multi-jurisdictional rental property CHIMNEY +
//! FIREPLACE + SOLID-FUEL-BURNING APPLIANCE inspection
//! and disclosure compliance framework. When a landlord
//! rents a property with a wood-burning fireplace, pellet
//! stove, wood stove, or other solid-fuel-burning
//! appliance, what inspection schedule applies, what
//! disclosure must be given to tenant, and what failure-
//! mode liabilities expose landlord after a chimney fire
//! or carbon monoxide event?
//!
//! Distinct from sibling modules: rental_carbon_monoxide_
//! detector (CO alarm), rental_basement_water_intrusion_
//! disclosure (water damage from chimney crown failure),
//! rental_gas_appliance_ban (electrification policy),
//! rental_hot_water_temperature (different appliance),
//! tenant_emotional_distress_damages (IIED parallel).
//!
//! Three-jurisdiction framework:
//!
//! 1. MAINE (most stringent for rental disclosure) — State
//!    of Maine Professional Financial Regulation Form 2079
//!    chimney/fireplace construction disclosure required
//!    AT TIME OF SALE/RENTAL ACCEPTANCE for residential
//!    properties with chimney or fireplace; Maine adopts
//!    NFPA 211 by reference and requires LEVEL II
//!    INSPECTION at title transfer per state fire code;
//!    common-law habitability under 14 M.R.S. § 6021
//!    implied warranty of fitness for habitation.
//! 2. CONNECTICUT — Connecticut adopts NFPA 211 by
//!    reference in Conn. Gen. Stat. § 29-292 (State Fire
//!    Safety Code); Conn. Public Health Code
//!    § 19-13-B105(o) addresses combustion-air and
//!    venting; annual NFPA 211 inspection recommended;
//!    common-law habitability + Conn. Gen. Stat. § 47a-7
//!    landlord duties.
//! 3. DEFAULT — NFPA 211 Standard for Chimneys,
//!    Fireplaces, Vents, and Solid Fuel-Burning Appliances
//!    is a VOLUNTARY NATIONAL STANDARD recommending
//!    ANNUAL inspection; enforceable as law only when
//!    adopted by local code (many state and municipal
//!    fire codes incorporate by reference); CSIA (Chimney
//!    Safety Institute of America) certification standards
//!    for inspectors; common-law implied warranty of
//!    habitability per Hilder v. St. Peter, 478 A.2d 202
//!    (Vt. 1984) and Green v. Superior Court, 10 Cal. 3d
//!    616 (1974); Cal. Civ. Code § 1941.1 implied warranty
//!    of fit-for-habitation facilities including
//!    heating.
//!
//! NFPA 211 three-level inspection framework:
//! 1. LEVEL I — basic visual inspection when no changes
//!    to chimney system, fuel type, or appliance;
//!    appropriate for routine annual maintenance check
//! 2. LEVEL II — comprehensive accessible inspection
//!    including all accessible portions of chimney
//!    interior and exterior; required on OCCUPANCY
//!    CHANGE (rental transition + sale), fuel change,
//!    damage event, system modification, or weather
//!    event (storm, earthquake, lightning); Maine
//!    mandates at title transfer
//! 3. LEVEL III — invasive inspection when hazards
//!    suggest concealed problem; removal of components
//!    permitted; only when Level I or Level II findings
//!    indicate concealed defect
//!
//! Five universal failure-mode liabilities:
//! 1. Creosote buildup beyond Stage 3 → chimney fire risk
//!    (Class A fire) + structural damage + tenant
//!    emergency-relocation duty (see mid_tenancy_
//!    temporary_relocation)
//! 2. Cracked flue liner → carbon monoxide release into
//!    living space (cross-reference rental_carbon_
//!    monoxide_detector) + IIED + wrongful death exposure
//! 3. Damaged crown / brick deterioration / spalling →
//!    water intrusion into walls and roof (cross-
//!    reference rental_basement_water_intrusion_
//!    disclosure)
//! 4. Animal nesting in flue (birds, raccoons, squirrels)
//!    → CO + fire risk; landlord ordinary maintenance
//!    duty + tenant cooperation required
//! 5. Improper combustion (negative pressure backdraft,
//!    blocked vent) → CO poisoning + IIED parallel to
//!    tenant_emotional_distress_damages iter 453
//!
//! Trader-landlord critical because (1) chimney fires
//! cause approximately 22,000 residential structure fires
//! annually in the US per NFPA data, resulting in $150M+
//! in property damage; (2) carbon monoxide events linked
//! to fireplace/chimney malfunction kill 50-150 Americans
//! annually with thousands of non-fatal poisonings; (3)
//! inherited rural property with neglected chimney is
//! among the highest-stakes habitability risks in rental
//! portfolios; (4) Maine landlord faces statutory
//! disclosure obligation at lease execution; (5) chimney
//! inspection costs $150-$500 per Level II inspection but
//! mitigates wrongful-death litigation exposure that can
//! exceed $5M.
//!
//! Authority: NFPA 211 (Standard for Chimneys, Fireplaces,
//! Vents, and Solid Fuel-Burning Appliances; current
//! edition); State of Maine Professional Financial
//! Regulation Form 2079 chimney/fireplace construction
//! disclosure; 14 M.R.S. § 6021 implied warranty of
//! habitation; Conn. Gen. Stat. § 29-292 State Fire
//! Safety Code adopting NFPA 211 by reference; Conn.
//! Public Health Code § 19-13-B105(o); Conn. Gen. Stat.
//! § 47a-7 landlord duties; Cal. Civ. Code § 1941.1
//! implied warranty; Hilder v. St. Peter, 478 A.2d 202
//! (Vt. 1984); Green v. Superior Court, 10 Cal. 3d 616
//! (1974); CSIA (Chimney Safety Institute of America)
//! certification standards.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Maine,
    Connecticut,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplianceType {
    None,
    WoodBurningFireplace,
    PelletStove,
    WoodStove,
    GasFireplaceVented,
    OilFurnaceChimneyVented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionLevel {
    None,
    LevelI,
    LevelII,
    LevelIII,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub appliance_type: ApplianceType,
    pub last_inspection_months_ago: u32,
    pub last_inspection_level: InspectionLevel,
    pub occupancy_change_since_last_inspection: bool,
    pub damage_event_since_last_inspection: bool,
    pub maine_form_2079_disclosure_provided: bool,
    pub creosote_stage_3_or_higher_observed: bool,
    pub cracked_flue_liner_observed: bool,
    pub damaged_crown_or_spalling_observed: bool,
    pub chimney_fire_event_reported: bool,
    pub co_event_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    DisclosureRequired,
    InspectionOverdue,
    DefectObserved,
    FireOrCoEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub const ANNUAL_INSPECTION_MAX_MONTHS: u32 = 12;

pub type RentalChimneyFireplaceInspectionDisclosureInput = Input;
pub type RentalChimneyFireplaceInspectionDisclosureResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Three-jurisdiction framework: Maine (most stringent — State of Maine Professional Financial Regulation Form 2079 chimney/fireplace construction disclosure required at sale/rental + Level II inspection at title transfer per state fire code adopting NFPA 211 by reference + 14 M.R.S. § 6021 implied warranty of habitation); Connecticut (adopts NFPA 211 by reference in Conn. Gen. Stat. § 29-292 State Fire Safety Code + Conn. Public Health Code § 19-13-B105(o) combustion-air/venting + Conn. Gen. Stat. § 47a-7 landlord duties); Default (NFPA 211 VOLUNTARY annual inspection recommendation enforceable only when adopted by local code + CSIA certification + common-law habitability per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1 implied warranty).".to_string(),
        "NFPA 211 three-level inspection framework: Level I basic visual when no changes to system; Level II comprehensive accessible inspection required on OCCUPANCY CHANGE (rental transition + sale) or fuel change or damage event or system modification or weather event (Maine mandates at title transfer); Level III invasive only when Level I/II indicates concealed defect.".to_string(),
        "Five universal failure-mode liabilities: (1) creosote buildup beyond Stage 3 → chimney fire risk Class A + tenant emergency-relocation duty (mid_tenancy_temporary_relocation); (2) cracked flue liner → CO release (rental_carbon_monoxide_detector) + IIED + wrongful death exposure; (3) damaged crown/brick deterioration/spalling → water intrusion (rental_basement_water_intrusion_disclosure); (4) animal nesting in flue → CO + fire risk + ordinary maintenance duty; (5) improper combustion (negative-pressure backdraft + blocked vent) → CO poisoning + IIED parallel to tenant_emotional_distress_damages iter 453.".to_string(),
        "NFPA chimney fire data: approximately 22,000 residential structure fires annually in US linked to chimney fires per NFPA, resulting in $150M+ property damage. Carbon monoxide events from fireplace/chimney malfunction kill 50-150 Americans annually with thousands of non-fatal poisonings.".to_string(),
        "Companion modules: rental_carbon_monoxide_detector (CO alarm), rental_basement_water_intrusion_disclosure, rental_gas_appliance_ban, rental_hot_water_temperature, tenant_emotional_distress_damages (IIED), mid_tenancy_temporary_relocation.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if matches!(input.appliance_type, ApplianceType::None) {
        let mut n = notes;
        n.push("No chimney/fireplace/solid-fuel appliance present — NFPA 211 inspection requirements not applicable.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    if input.chimney_fire_event_reported || input.co_event_reported {
        actions.push("Fire or CO event reported: engage emergency services + counsel; preserve evidence; tenant emergency relocation duty (mid_tenancy_temporary_relocation); tort negligence + wrongful death + IIED exposure parallel to tenant_emotional_distress_damages iter 453; Level III invasive NFPA 211 inspection required before reuse.".to_string());
    }

    let defect_observed = input.creosote_stage_3_or_higher_observed
        || input.cracked_flue_liner_observed
        || input.damaged_crown_or_spalling_observed;
    if input.creosote_stage_3_or_higher_observed {
        actions.push("Creosote Stage 3 (glazed creosote) or higher observed: Class A chimney fire risk; CSIA-certified sweep + professional removal required before next use; do NOT operate appliance until cleaned.".to_string());
    }
    if input.cracked_flue_liner_observed {
        actions.push("Cracked flue liner observed: carbon monoxide release into living space risk; cross-reference rental_carbon_monoxide_detector; replace or reline before reuse; offer temporary relocation per mid_tenancy_temporary_relocation if appliance is primary heat source.".to_string());
    }
    if input.damaged_crown_or_spalling_observed {
        actions.push("Damaged chimney crown or brick spalling observed: water intrusion path into walls and roof; cross-reference rental_basement_water_intrusion_disclosure; tuckpointing + crown repair required.".to_string());
    }

    let needs_level_ii =
        input.occupancy_change_since_last_inspection || input.damage_event_since_last_inspection;
    if needs_level_ii
        && !matches!(
            input.last_inspection_level,
            InspectionLevel::LevelII | InspectionLevel::LevelIII
        )
    {
        actions.push("NFPA 211 Level II inspection required: occupancy change or damage event since last inspection; engage CSIA-certified chimney sweep inspector.".to_string());
    }

    let inspection_overdue = input.last_inspection_months_ago > ANNUAL_INSPECTION_MAX_MONTHS;
    if inspection_overdue {
        actions.push(format!(
            "Inspection overdue: {} months since last inspection exceeds 12-month NFPA 211 annual recommendation; engage CSIA-certified inspector.",
            input.last_inspection_months_ago
        ));
    }

    match input.jurisdiction {
        Jurisdiction::Maine => {
            if !input.maine_form_2079_disclosure_provided {
                actions.push("Maine: State of Maine Professional Financial Regulation Form 2079 chimney/fireplace construction disclosure NOT provided to tenant at lease execution — Maine statutory disclosure obligation; provide form before acceptance.".to_string());
            }
            actions.push("Maine: Level II NFPA 211 inspection required at title transfer per state fire code; 14 M.R.S. § 6021 implied warranty of habitation applies.".to_string());
        }
        Jurisdiction::Connecticut => {
            actions.push("Connecticut: Conn. Gen. Stat. § 29-292 State Fire Safety Code adopts NFPA 211 by reference; Conn. Public Health Code § 19-13-B105(o) combustion-air and venting; Conn. Gen. Stat. § 47a-7 landlord duties; annual NFPA 211 inspection recommended.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: NFPA 211 voluntary national standard enforceable when adopted by local code; CSIA certification recommended for inspector; common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1.".to_string());
        }
    }

    let maine_disclosure_missing = matches!(input.jurisdiction, Jurisdiction::Maine)
        && !input.maine_form_2079_disclosure_provided;

    let severity = if input.chimney_fire_event_reported || input.co_event_reported {
        Severity::FireOrCoEvent
    } else if defect_observed {
        Severity::DefectObserved
    } else if inspection_overdue
        || (needs_level_ii
            && !matches!(
                input.last_inspection_level,
                InspectionLevel::LevelII | InspectionLevel::LevelIII
            ))
    {
        Severity::InspectionOverdue
    } else if maine_disclosure_missing {
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
            jurisdiction: Jurisdiction::Maine,
            appliance_type: ApplianceType::WoodBurningFireplace,
            last_inspection_months_ago: 6,
            last_inspection_level: InspectionLevel::LevelII,
            occupancy_change_since_last_inspection: false,
            damage_event_since_last_inspection: false,
            maine_form_2079_disclosure_provided: true,
            creosote_stage_3_or_higher_observed: false,
            cracked_flue_liner_observed: false,
            damaged_crown_or_spalling_observed: false,
            chimney_fire_event_reported: false,
            co_event_reported: false,
        }
    }

    #[test]
    fn no_appliance_not_applicable() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::None;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn maine_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn inspection_overdue_over_12_months() {
        let mut i = baseline();
        i.last_inspection_months_ago = 24;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("12-month NFPA 211"));
        assert!(joined.contains("CSIA"));
    }

    #[test]
    fn inspection_exactly_12_months_compliant() {
        let mut i = baseline();
        i.last_inspection_months_ago = 12;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn occupancy_change_requires_level_ii() {
        let mut i = baseline();
        i.occupancy_change_since_last_inspection = true;
        i.last_inspection_level = InspectionLevel::LevelI;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Level II inspection required"));
        assert!(joined.contains("occupancy change"));
    }

    #[test]
    fn occupancy_change_with_level_ii_compliant() {
        let mut i = baseline();
        i.occupancy_change_since_last_inspection = true;
        i.last_inspection_level = InspectionLevel::LevelII;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn damage_event_requires_level_ii() {
        let mut i = baseline();
        i.damage_event_since_last_inspection = true;
        i.last_inspection_level = InspectionLevel::LevelI;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn creosote_stage_3_defect_observed_severity() {
        let mut i = baseline();
        i.creosote_stage_3_or_higher_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Class A chimney fire"));
        assert!(joined.contains("CSIA-certified sweep"));
    }

    #[test]
    fn cracked_flue_cross_references_co_module() {
        let mut i = baseline();
        i.cracked_flue_liner_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("rental_carbon_monoxide_detector"));
        assert!(joined.contains("reline before reuse"));
    }

    #[test]
    fn damaged_crown_cross_references_water_intrusion() {
        let mut i = baseline();
        i.damaged_crown_or_spalling_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("rental_basement_water_intrusion_disclosure"));
        assert!(joined.contains("tuckpointing"));
    }

    #[test]
    fn chimney_fire_event_top_severity() {
        let mut i = baseline();
        i.chimney_fire_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FireOrCoEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Fire or CO event"));
        assert!(joined.contains("Level III invasive"));
    }

    #[test]
    fn co_event_top_severity() {
        let mut i = baseline();
        i.co_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FireOrCoEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("wrongful death"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn maine_form_2079_missing_disclosure_severity() {
        let mut i = baseline();
        i.maine_form_2079_disclosure_provided = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DisclosureRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Form 2079"));
        assert!(joined.contains("Maine statutory disclosure"));
    }

    #[test]
    fn maine_form_2079_missing_in_ct_not_required() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Connecticut;
        i.maine_form_2079_disclosure_provided = false; // irrelevant in CT
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn maine_form_2079_missing_in_default_not_required() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.maine_form_2079_disclosure_provided = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ct_cites_29_292_and_19_13_b105_o() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Connecticut;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 29-292"));
        assert!(joined.contains("§ 19-13-B105(o)"));
        assert!(joined.contains("§ 47a-7"));
    }

    #[test]
    fn default_jurisdiction_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("CSIA"));
    }

    #[test]
    fn severity_priority_event_above_defect_above_inspection_above_disclosure() {
        let mut i = baseline();
        i.chimney_fire_event_reported = true;
        i.creosote_stage_3_or_higher_observed = true;
        i.last_inspection_months_ago = 24;
        i.maine_form_2079_disclosure_provided = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FireOrCoEvent);
    }

    #[test]
    fn severity_defect_above_inspection_above_disclosure() {
        let mut i = baseline();
        i.creosote_stage_3_or_higher_observed = true;
        i.last_inspection_months_ago = 24;
        i.maine_form_2079_disclosure_provided = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
    }

    #[test]
    fn severity_inspection_above_disclosure_when_no_defect() {
        let mut i = baseline();
        i.last_inspection_months_ago = 24;
        i.maine_form_2079_disclosure_provided = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn pellet_stove_treated_as_appliance() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::PelletStove;
        i.last_inspection_months_ago = 24;
        let out = check(&i);
        // Pellet stove still requires NFPA 211 inspection
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn wood_stove_treated_as_appliance() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::WoodStove;
        i.last_inspection_months_ago = 24;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn gas_fireplace_vented_treated_as_appliance() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::GasFireplaceVented;
        i.last_inspection_months_ago = 24;
        let out = check(&i);
        // Vented gas fireplaces still need annual NFPA 211 inspection
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn oil_furnace_chimney_vented_treated_as_appliance() {
        let mut i = baseline();
        i.appliance_type = ApplianceType::OilFurnaceChimneyVented;
        i.last_inspection_months_ago = 24;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("NFPA 211"));
        assert!(joined.contains("Form 2079"));
        assert!(joined.contains("14 M.R.S. § 6021"));
        assert!(joined.contains("§ 29-292"));
        assert!(joined.contains("§ 19-13-B105(o)"));
        assert!(joined.contains("§ 47a-7"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("CSIA"));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Maine (most stringent"));
        assert!(joined.contains("Connecticut"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_nfpa_three_level_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Level I"));
        assert!(joined.contains("Level II"));
        assert!(joined.contains("Level III"));
        assert!(joined.contains("OCCUPANCY CHANGE"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("creosote buildup"));
        assert!(joined.contains("cracked flue liner"));
        assert!(joined.contains("damaged crown"));
        assert!(joined.contains("animal nesting"));
        assert!(joined.contains("improper combustion"));
    }

    #[test]
    fn note_pins_nfpa_22k_fire_data() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("22,000 residential structure fires"));
        assert!(joined.contains("$150M"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_carbon_monoxide_detector"));
        assert!(joined.contains("rental_basement_water_intrusion_disclosure"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
        assert!(joined.contains("mid_tenancy_temporary_relocation"));
    }

    #[test]
    fn maine_uniquely_requires_form_2079_invariant() {
        // Same fact: form not provided
        let me = check(&Input {
            jurisdiction: Jurisdiction::Maine,
            maine_form_2079_disclosure_provided: false,
            ..baseline()
        });
        let ct = check(&Input {
            jurisdiction: Jurisdiction::Connecticut,
            maine_form_2079_disclosure_provided: false,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            maine_form_2079_disclosure_provided: false,
            ..baseline()
        });
        // Only Maine triggers DisclosureRequired
        assert_eq!(me.severity, Severity::DisclosureRequired);
        assert_eq!(ct.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn jurisdiction_truth_table_three_cells() {
        let me = check(&Input {
            jurisdiction: Jurisdiction::Maine,
            ..baseline()
        });
        let ct = check(&Input {
            jurisdiction: Jurisdiction::Connecticut,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(me.severity, Severity::Compliant);
        assert_eq!(ct.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn multiple_defects_stack_in_actions() {
        let mut i = baseline();
        i.creosote_stage_3_or_higher_observed = true;
        i.cracked_flue_liner_observed = true;
        i.damaged_crown_or_spalling_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Class A"));
        assert!(joined.contains("reline before reuse"));
        assert!(joined.contains("tuckpointing"));
    }
}
