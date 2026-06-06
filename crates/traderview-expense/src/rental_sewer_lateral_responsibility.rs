//! Multi-jurisdictional rental property PRIVATE SEWER
//! LATERAL (PSL) inspection, compliance, and landlord-
//! responsibility framework. When a landlord rents a
//! property connected to municipal sewer service, what
//! ownership/maintenance/inspection obligations apply to
//! the private sewer lateral (the pipe connecting the
//! building to the public sewer main), what point-of-sale
//! compliance certificate requirements apply, and what
//! failure-mode liabilities expose landlord after a
//! sewage backup into the dwelling unit?
//!
//! Distinct from sibling modules: rental_septic_system_
//! disclosure (iter 465 — OSTDS/septic for non-municipal
//! sewer), rental_basement_water_intrusion_disclosure,
//! rental_water_submetering_disclosure, rent_abatement_
//! construction_nuisance, mid_tenancy_temporary_relocation,
//! tenant_emotional_distress_damages (IIED parallel).
//!
//! Four-jurisdiction framework:
//!
//! 1. EAST BAY MUNICIPAL UTILITY DISTRICT (EBMUD)
//!    REGIONAL PSL PROGRAM (most prescriptive) — covers
//!    Alameda, Albany, Emeryville, Oakland, Piedmont,
//!    El Cerrito, Kensington, and Richmond Annex. PSL
//!    Ordinance triggers at: (a) property SALE; (b)
//!    BUILDING/REMODELING in excess of $100,000; OR (c)
//!    CHANGING WATER METER SIZE. Property owner
//!    responsible for ENTIRE lateral from home to public
//!    sewer main (except Alameda + Albany where
//!    responsibility ends at property line or curbside
//!    cleanout). Compliance Certificate required:
//!    20-year validity for complete replacement; 7-year
//!    validity for repair or passed test without repair.
//!    Time Extension Certificate (6 months) available
//!    with $4,500 EBMUD DEPOSIT refunded upon compliance.
//! 2. CITY OF BERKELEY PSL PROGRAM — separate program
//!    effective November 3, 2014 with distinct compliance
//!    requirements; Berkeley Municipal Code Chapter 17.16
//!    Sewer Lateral Inspection and Repair.
//! 3. MASSACHUSETTS — M.G.L. c. 83 § 7 owner-maintenance
//!    duty for sewer pipe from premises to public sewer
//!    line; M.G.L. c. 21 § 26-53 Clean Waters Act
//!    inflow-and-infiltration compliance; 314 C.M.R.
//!    12.00 sewer use regulations.
//! 4. DEFAULT — Common-law implied warranty of
//!    habitability per Hilder v. St. Peter, 478 A.2d 202
//!    (Vt. 1984); Green v. Superior Court, 10 Cal. 3d
//!    616 (1974); Cal. Civ. Code § 1941.1 implied
//!    warranty of sanitary facilities; tort negligence
//!    and premises liability for sewage backup events.
//!
//! Universal failure-mode liability framework:
//! 1. TREE-ROOT INTRUSION (leading cause of PSL failure)
//!    → sewage backup into unit → habitability breach +
//!    Hilder constructive eviction + tenant emergency
//!    relocation duty (mid_tenancy_temporary_relocation)
//! 2. AGED CLAY OR ORANGEBURG PIPE COLLAPSE — pipes
//!    installed 1900-1965 typically vitrified clay or
//!    Orangeburg (tar-paper composite), both subject to
//!    age-related collapse → emergency $5,000-$15,000
//!    trenchless replacement or $15,000-$40,000 open-cut
//! 3. STORMWATER INFLOW AND INFILTRATION (I&I) —
//!    cracked lateral admits groundwater + stormwater
//!    into sanitary sewer, overloading treatment plant
//!    capacity; municipal violation + consent-decree
//!    enforcement under Clean Water Act 33 U.S.C. § 1342
//!    NPDES
//! 4. CROSS-CONNECTION WITH STORM DRAIN — illicit
//!    discharge violation under 33 U.S.C. § 1342;
//!    municipal cleanup cost-recovery
//! 5. FAILURE TO OBTAIN COMPLIANCE CERTIFICATE AT SALE
//!    (EBMUD/Berkeley regions) → escrow delay +
//!    $4,500 EBMUD deposit + potential close-of-escrow
//!    inability
//!
//! Inspection methodology — typical PSL inspection
//! includes video camera scope from cleanout, hydro-jet
//! cleaning, air-pressure test, and dye/smoke test for
//! cross-connection detection. Cost $400-$1,000 per
//! inspection; cheaper than wrongful-death exposure from
//! sewage-backup IIED claim.
//!
//! Trader-landlord critical because (1) property owners
//! in EBMUD region face MANDATORY point-of-sale
//! compliance certificate at $4,500 deposit risk; (2) Bay
//! Area homes built pre-1965 routinely have failing clay
//! or Orangeburg laterals requiring $15K-$40K replacement
//! at sale; (3) sewage backup into tenant unit is among
//! the most actionable habitability claims because raw
//! sewage exposure to children + pregnant tenants
//! triggers heightened tort + IIED liability; (4) tree-
//! root intrusion from city-owned street trees can shift
//! municipal liability but landlord remains primary
//! defendant for tenant claims; (5) inherited PSL
//! deferred maintenance is leading hidden cost in
//! trader-landlord acquisition due diligence.
//!
//! Authority: EBMUD Regional Private Sewer Lateral
//! Program Ordinance; Berkeley Municipal Code Chapter
//! 17.16 (effective November 3, 2014); M.G.L. c. 83 § 7
//! (Massachusetts owner-maintenance duty); M.G.L. c. 21
//! § 26-53 (Massachusetts Clean Waters Act); 314 C.M.R.
//! 12.00 (Massachusetts sewer use regulations); Clean
//! Water Act 33 U.S.C. § 1342 (NPDES); Cal. Civ. Code
//! § 1941.1; Hilder v. St. Peter, 478 A.2d 202 (Vt.
//! 1984); Green v. Superior Court, 10 Cal. 3d 616 (1974).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    EbmudRegional,
    Berkeley,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LateralMaterial {
    PvcOrAbs,
    CastIron,
    VitrifiedClay,
    Orangeburg,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum PointOfSaleTrigger {
    None,
    PropertySale,
    BuildingRemodelOverOneHundredK,
    WaterMeterSizeChange,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub property_on_municipal_sewer: bool,
    pub lateral_material: LateralMaterial,
    pub point_of_sale_trigger: PointOfSaleTrigger,
    pub ebmud_compliance_certificate_valid: bool,
    pub ebmud_compliance_certificate_expires_months_from_now: u32,
    pub ebmud_time_extension_deposit_posted: bool,
    pub last_video_inspection_months_ago: u32,
    pub tree_root_intrusion_observed: bool,
    pub cracked_or_collapsed_observed: bool,
    pub stormwater_i_and_i_observed: bool,
    pub cross_connection_with_storm_drain_observed: bool,
    pub sewage_backup_event_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    InspectionRecommended,
    PosCertificateRequired,
    DefectObserved,
    SewageBackupEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub const RECOMMENDED_INSPECTION_INTERVAL_MONTHS: u32 = 60; // 5-year general best practice

pub type RentalSewerLateralResponsibilityInput = Input;
pub type RentalSewerLateralResponsibilityResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: EBMUD Regional PSL Program (most prescriptive — Alameda + Albany + Emeryville + Oakland + Piedmont + El Cerrito + Kensington + Richmond Annex; point-of-sale + $100K remodel + water meter change triggers; 20-year compliance certificate for complete replacement / 7-year for repair; $4,500 Time Extension Certificate deposit); Berkeley Municipal Code Chapter 17.16 PSL Program (effective November 3, 2014); Massachusetts M.G.L. c. 83 § 7 owner-maintenance duty + M.G.L. c. 21 § 26-53 Clean Waters Act + 314 C.M.R. 12.00; Default common-law habitability per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984) + Green v. Superior Court 10 Cal. 3d 616 (1974) + Cal. Civ. Code § 1941.1 + Clean Water Act 33 U.S.C. § 1342 NPDES.".to_string(),
        "EBMUD point-of-sale triggers: (a) property SALE; (b) BUILDING/REMODELING in excess of $100,000; OR (c) CHANGING WATER METER SIZE. Property owner responsible for ENTIRE lateral from home to public sewer main (except Alameda + Albany where responsibility ends at property line or curbside cleanout).".to_string(),
        "Five universal failure-mode liabilities: (1) TREE-ROOT INTRUSION → sewage backup + Hilder v. St. Peter habitability breach + emergency relocation (mid_tenancy_temporary_relocation); (2) AGED CLAY OR ORANGEBURG PIPE COLLAPSE (pre-1965 installation) → $5K-$15K trenchless or $15K-$40K open-cut replacement; (3) STORMWATER INFLOW AND INFILTRATION (I&I) → municipal violation + consent-decree enforcement under Clean Water Act 33 U.S.C. § 1342 NPDES; (4) CROSS-CONNECTION WITH STORM DRAIN → illicit discharge violation + municipal cleanup cost-recovery; (5) FAILURE TO OBTAIN COMPLIANCE CERTIFICATE AT SALE → escrow delay + $4,500 EBMUD deposit.".to_string(),
        "Inspection methodology: video camera scope from cleanout + hydro-jet cleaning + air-pressure test + dye/smoke test for cross-connection detection ($400-$1,000 per inspection); 5-year general best-practice interval; sewage backup IIED exposure routinely exceeds $100K-$1M per tenant claim.".to_string(),
        "Aged lateral materials: Vitrified Clay (1900-1965 era — brittle, root-prone); Orangeburg (1945-1972 tar-paper composite — collapse-prone); Cast Iron (1900-1980 — corrosion-prone); modern PVC/ABS (1980+ — robust 50+ year lifespan).".to_string(),
        "Companion modules: rental_septic_system_disclosure (iter 465 — OSTDS for non-municipal sewer), rental_basement_water_intrusion_disclosure, rental_water_submetering_disclosure, rent_abatement_construction_nuisance, mid_tenancy_temporary_relocation, tenant_emotional_distress_damages.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if !input.property_on_municipal_sewer {
        let mut n = notes;
        n.push("Property not connected to municipal sewer — PSL framework not applicable. See rental_septic_system_disclosure (iter 465) for OSTDS regimes.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    if input.sewage_backup_event_reported {
        actions.push("Sewage backup event reported: engage emergency services + plumber + counsel; preserve evidence; tenant emergency relocation duty (mid_tenancy_temporary_relocation); Hilder v. St. Peter constructive eviction + tort negligence + IIED parallel to tenant_emotional_distress_damages iter 453; raw-sewage exposure to children + pregnant tenants triggers heightened liability.".to_string());
    }

    let defect_observed = input.tree_root_intrusion_observed
        || input.cracked_or_collapsed_observed
        || input.stormwater_i_and_i_observed
        || input.cross_connection_with_storm_drain_observed;

    if input.tree_root_intrusion_observed {
        actions.push("Tree-root intrusion observed: leading cause of PSL failure; hydro-jet + root-cutter remediation $400-$1,000; if recurrent, full replacement required; consider city-owned street tree liability shift.".to_string());
    }
    if input.cracked_or_collapsed_observed {
        actions.push("Cracked or collapsed lateral observed: emergency replacement required; trenchless cured-in-place pipe (CIPP) lining $5,000-$15,000 or open-cut excavation $15,000-$40,000; aged clay or Orangeburg material likely.".to_string());
    }
    if input.stormwater_i_and_i_observed {
        actions.push("Stormwater inflow and infiltration (I&I) observed: lateral admits groundwater/stormwater into sanitary sewer, overloading treatment plant; municipal violation + consent-decree enforcement under Clean Water Act 33 U.S.C. § 1342 NPDES.".to_string());
    }
    if input.cross_connection_with_storm_drain_observed {
        actions.push("Cross-connection with storm drain observed: illicit discharge violation under 33 U.S.C. § 1342; municipal cleanup cost-recovery exposure; immediate disconnection and dye-test required.".to_string());
    }

    let inspection_overdue =
        input.last_video_inspection_months_ago > RECOMMENDED_INSPECTION_INTERVAL_MONTHS;
    let aged_material_recommends_inspection = matches!(
        input.lateral_material,
        LateralMaterial::VitrifiedClay | LateralMaterial::Orangeburg | LateralMaterial::CastIron
    );

    if (inspection_overdue || aged_material_recommends_inspection) && !defect_observed {
        actions.push(format!(
            "Video inspection recommended: aged lateral material ({:?}) or 5-year inspection interval lapsed ({} months); engage CCTV-equipped plumber for camera scope + hydro-jet cleaning.",
            input.lateral_material, input.last_video_inspection_months_ago
        ));
    }

    let pos_cert_required = matches!(input.jurisdiction, Jurisdiction::EbmudRegional)
        && !matches!(input.point_of_sale_trigger, PointOfSaleTrigger::None)
        && !input.ebmud_compliance_certificate_valid
        && !input.ebmud_time_extension_deposit_posted;
    if pos_cert_required {
        actions.push("EBMUD point-of-sale Compliance Certificate REQUIRED: trigger event occurred; obtain certificate (20-year validity for complete replacement / 7-year for repair or passed test) OR post $4,500 Time Extension Certificate deposit for 6-month grace period.".to_string());
    }

    match input.jurisdiction {
        Jurisdiction::EbmudRegional => {
            actions.push("EBMUD Regional PSL Program: property owner responsible for entire lateral from home to public main (except Alameda + Albany where responsibility ends at property line/curbside cleanout); point-of-sale + $100K remodel + water meter change triggers Compliance Certificate requirement.".to_string());
        }
        Jurisdiction::Berkeley => {
            actions.push("Berkeley: Berkeley Municipal Code Chapter 17.16 Sewer Lateral Inspection and Repair (effective November 3, 2014) — separate from EBMUD; distinct compliance requirements + inspection methodology.".to_string());
        }
        Jurisdiction::Massachusetts => {
            actions.push("Massachusetts: M.G.L. c. 83 § 7 owner-maintenance duty for sewer pipe from premises to public sewer line + M.G.L. c. 21 § 26-53 Clean Waters Act inflow-and-infiltration compliance + 314 C.M.R. 12.00 sewer use regulations.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Green v. Superior Court, 10 Cal. 3d 616 (1974) + Cal. Civ. Code § 1941.1; municipal sewer-use ordinance + Clean Water Act 33 U.S.C. § 1342 NPDES coverage.".to_string());
        }
    }

    let severity = if input.sewage_backup_event_reported {
        Severity::SewageBackupEvent
    } else if defect_observed {
        Severity::DefectObserved
    } else if pos_cert_required {
        Severity::PosCertificateRequired
    } else if inspection_overdue || aged_material_recommends_inspection {
        Severity::InspectionRecommended
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
            jurisdiction: Jurisdiction::EbmudRegional,
            property_on_municipal_sewer: true,
            lateral_material: LateralMaterial::PvcOrAbs,
            point_of_sale_trigger: PointOfSaleTrigger::None,
            ebmud_compliance_certificate_valid: true,
            ebmud_compliance_certificate_expires_months_from_now: 60,
            ebmud_time_extension_deposit_posted: false,
            last_video_inspection_months_ago: 24,
            tree_root_intrusion_observed: false,
            cracked_or_collapsed_observed: false,
            stormwater_i_and_i_observed: false,
            cross_connection_with_storm_drain_observed: false,
            sewage_backup_event_reported: false,
        }
    }

    #[test]
    fn property_on_septic_not_applicable() {
        let mut i = baseline();
        i.property_on_municipal_sewer = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_septic_system_disclosure"));
    }

    #[test]
    fn ebmud_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn sewage_backup_event_top_severity() {
        let mut i = baseline();
        i.sewage_backup_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::SewageBackupEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Sewage backup event"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn tree_root_intrusion_defect_observed() {
        let mut i = baseline();
        i.tree_root_intrusion_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Tree-root intrusion"));
        assert!(joined.contains("city-owned street tree"));
    }

    #[test]
    fn cracked_collapsed_defect_observed() {
        let mut i = baseline();
        i.cracked_or_collapsed_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("trenchless cured-in-place pipe"));
        assert!(joined.contains("$5,000-$15,000"));
    }

    #[test]
    fn stormwater_i_and_i_defect_observed() {
        let mut i = baseline();
        i.stormwater_i_and_i_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("I&I"));
        assert!(joined.contains("33 U.S.C. § 1342 NPDES"));
    }

    #[test]
    fn cross_connection_storm_drain_defect_observed() {
        let mut i = baseline();
        i.cross_connection_with_storm_drain_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("illicit discharge"));
        assert!(joined.contains("dye-test"));
    }

    #[test]
    fn ebmud_pos_sale_no_certificate_pos_required() {
        let mut i = baseline();
        i.point_of_sale_trigger = PointOfSaleTrigger::PropertySale;
        i.ebmud_compliance_certificate_valid = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PosCertificateRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("EBMUD point-of-sale Compliance Certificate"));
        assert!(joined.contains("$4,500"));
        assert!(joined.contains("Time Extension"));
    }

    #[test]
    fn ebmud_pos_remodel_over_100k_pos_required() {
        let mut i = baseline();
        i.point_of_sale_trigger = PointOfSaleTrigger::BuildingRemodelOverOneHundredK;
        i.ebmud_compliance_certificate_valid = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PosCertificateRequired);
    }

    #[test]
    fn ebmud_pos_water_meter_change_pos_required() {
        let mut i = baseline();
        i.point_of_sale_trigger = PointOfSaleTrigger::WaterMeterSizeChange;
        i.ebmud_compliance_certificate_valid = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PosCertificateRequired);
    }

    #[test]
    fn ebmud_pos_with_valid_certificate_compliant() {
        let mut i = baseline();
        i.point_of_sale_trigger = PointOfSaleTrigger::PropertySale;
        i.ebmud_compliance_certificate_valid = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ebmud_pos_with_time_extension_deposit_compliant() {
        let mut i = baseline();
        i.point_of_sale_trigger = PointOfSaleTrigger::PropertySale;
        i.ebmud_compliance_certificate_valid = false;
        i.ebmud_time_extension_deposit_posted = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn berkeley_no_ebmud_pos_certificate_required() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Berkeley;
        i.point_of_sale_trigger = PointOfSaleTrigger::PropertySale;
        i.ebmud_compliance_certificate_valid = false;
        let out = check(&i);
        // EBMUD certificate not relevant in Berkeley; Berkeley has own separate program
        assert_eq!(out.severity, Severity::Compliant);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Berkeley Municipal Code Chapter 17.16"));
    }

    #[test]
    fn vitrified_clay_recommends_inspection() {
        let mut i = baseline();
        i.lateral_material = LateralMaterial::VitrifiedClay;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionRecommended);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Video inspection recommended"));
        assert!(joined.contains("VitrifiedClay"));
    }

    #[test]
    fn orangeburg_recommends_inspection() {
        let mut i = baseline();
        i.lateral_material = LateralMaterial::Orangeburg;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionRecommended);
    }

    #[test]
    fn cast_iron_recommends_inspection() {
        let mut i = baseline();
        i.lateral_material = LateralMaterial::CastIron;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionRecommended);
    }

    #[test]
    fn last_inspection_over_60_months_inspection_recommended() {
        let mut i = baseline();
        i.last_video_inspection_months_ago = 72;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionRecommended);
    }

    #[test]
    fn modern_pvc_within_60_months_compliant() {
        let mut i = baseline();
        i.lateral_material = LateralMaterial::PvcOrAbs;
        i.last_video_inspection_months_ago = 36;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ebmud_jurisdiction_cites_alameda_albany_exception() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::EbmudRegional;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Alameda + Albany"));
        assert!(joined.contains("property line/curbside cleanout"));
    }

    #[test]
    fn berkeley_jurisdiction_cites_chapter_17_16() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Berkeley;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Chapter 17.16"));
        assert!(joined.contains("November 3, 2014"));
    }

    #[test]
    fn ma_jurisdiction_cites_c_83_and_clean_waters_act() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("M.G.L. c. 83 § 7"));
        assert!(joined.contains("M.G.L. c. 21 § 26-53"));
        assert!(joined.contains("314 C.M.R. 12.00"));
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
        assert!(joined.contains("33 U.S.C. § 1342 NPDES"));
    }

    #[test]
    fn severity_priority_backup_above_defect_above_pos_above_inspection() {
        let mut i = baseline();
        i.sewage_backup_event_reported = true;
        i.cracked_or_collapsed_observed = true;
        i.point_of_sale_trigger = PointOfSaleTrigger::PropertySale;
        i.ebmud_compliance_certificate_valid = false;
        i.lateral_material = LateralMaterial::VitrifiedClay;
        let out = check(&i);
        assert_eq!(out.severity, Severity::SewageBackupEvent);
    }

    #[test]
    fn severity_defect_above_pos_above_inspection() {
        let mut i = baseline();
        i.cracked_or_collapsed_observed = true;
        i.point_of_sale_trigger = PointOfSaleTrigger::PropertySale;
        i.ebmud_compliance_certificate_valid = false;
        i.lateral_material = LateralMaterial::VitrifiedClay;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
    }

    #[test]
    fn severity_pos_above_inspection() {
        let mut i = baseline();
        i.point_of_sale_trigger = PointOfSaleTrigger::PropertySale;
        i.ebmud_compliance_certificate_valid = false;
        i.lateral_material = LateralMaterial::VitrifiedClay;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PosCertificateRequired);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("EBMUD Regional"));
        assert!(joined.contains("Berkeley Municipal Code Chapter 17.16"));
        assert!(joined.contains("November 3, 2014"));
        assert!(joined.contains("M.G.L. c. 83 § 7"));
        assert!(joined.contains("M.G.L. c. 21 § 26-53"));
        assert!(joined.contains("314 C.M.R. 12.00"));
        assert!(joined.contains("33 U.S.C. § 1342"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("EBMUD Regional PSL Program"));
        assert!(joined.contains("Berkeley"));
        assert!(joined.contains("Massachusetts"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_ebmud_three_pos_triggers() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("property SALE"));
        assert!(joined.contains("BUILDING/REMODELING"));
        assert!(joined.contains("CHANGING WATER METER"));
        assert!(joined.contains("$100,000"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("TREE-ROOT INTRUSION"));
        assert!(joined.contains("AGED CLAY OR ORANGEBURG"));
        assert!(joined.contains("STORMWATER INFLOW"));
        assert!(joined.contains("CROSS-CONNECTION"));
        assert!(joined.contains("FAILURE TO OBTAIN COMPLIANCE CERTIFICATE"));
    }

    #[test]
    fn note_pins_inspection_methodology() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("video camera scope"));
        assert!(joined.contains("hydro-jet"));
        assert!(joined.contains("dye/smoke test"));
        assert!(joined.contains("$400-$1,000"));
    }

    #[test]
    fn note_pins_aged_lateral_materials() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Vitrified Clay (1900-1965"));
        assert!(joined.contains("Orangeburg"));
        assert!(joined.contains("1945-1972"));
        assert!(joined.contains("Cast Iron"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_septic_system_disclosure"));
        assert!(joined.contains("rental_basement_water_intrusion_disclosure"));
        assert!(joined.contains("rent_abatement_construction_nuisance"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn ebmud_uniquely_strictest_pos_certificate_invariant() {
        // Same fact: property sale with no certificate
        let ebmud = check(&Input {
            jurisdiction: Jurisdiction::EbmudRegional,
            point_of_sale_trigger: PointOfSaleTrigger::PropertySale,
            ebmud_compliance_certificate_valid: false,
            ..baseline()
        });
        let berkeley = check(&Input {
            jurisdiction: Jurisdiction::Berkeley,
            point_of_sale_trigger: PointOfSaleTrigger::PropertySale,
            ebmud_compliance_certificate_valid: false,
            ..baseline()
        });
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            point_of_sale_trigger: PointOfSaleTrigger::PropertySale,
            ebmud_compliance_certificate_valid: false,
            ..baseline()
        });
        // EBMUD triggers PosCertificateRequired; others compliant
        assert_eq!(ebmud.severity, Severity::PosCertificateRequired);
        assert_eq!(berkeley.severity, Severity::Compliant);
        assert_eq!(ma.severity, Severity::Compliant);
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        let ebmud = check(&Input {
            jurisdiction: Jurisdiction::EbmudRegional,
            ..baseline()
        });
        let berkeley = check(&Input {
            jurisdiction: Jurisdiction::Berkeley,
            ..baseline()
        });
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(ebmud.severity, Severity::Compliant);
        assert_eq!(berkeley.severity, Severity::Compliant);
        assert_eq!(ma.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn multiple_defects_stack_in_actions() {
        let mut i = baseline();
        i.tree_root_intrusion_observed = true;
        i.cracked_or_collapsed_observed = true;
        i.stormwater_i_and_i_observed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Tree-root"));
        assert!(joined.contains("cured-in-place"));
        assert!(joined.contains("I&I"));
    }
}
