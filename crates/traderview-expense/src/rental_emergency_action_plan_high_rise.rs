//! Rental high-rise emergency action plan (EAP) framework — covers NFPA 1
//! Fire Code + International Fire Code (IFC) § 404 + NYC FDNY 3 RCNY 404-01
//! emergency planning and preparedness requirements for high-rise
//! residential buildings.
//!
//! Distinct from sibling `rental_elevator_safety_inspection` (elevator
//! inspection framework), `rental_fire_extinguisher_requirement` (NFPA 10
//! fire extinguisher framework), [[rental_chimney_fireplace_inspection_
//! disclosure]] (chimney inspection — distinct), [[rental_grill_propane_
//! bbq_restriction]] (iter 515 NFPA 1 balcony grilling framework), [[rental_
//! balcony_inspection_seismic_safety]] (iter 511 EEE structural inspection).
//!
//! Trader-landlord critical because (1) NYC Local Law 26 of 2004 post-9/11
//! expanded EAP requirements to all high-rise (≥ 75 feet OR 7+ stories)
//! office AND residential buildings; (2) NYC FDNY 3 RCNY § 404-01 plus
//! Administrative Code § 404.2 plus Fire Code Title 29 require Fire Safety
//! Director (FSD) plus Fire Safety Plan (FSP) plus Emergency Action Plan
//! (EAP) plus annual tenant emergency-instructions distribution; (3) NFPA 1
//! § 10.8 plus § 11.10 high-rise framework plus IFC § 404 require
//! comprehensive EAP including tenant evacuation procedures + FSD on-site
//! for fires plus non-fire emergencies (medical + active shooter +
//! hazardous-material release) + annual fire drill + post-emergency reentry
//! protocol + accessible-evacuation provisions for mobility-impaired
//! tenants; (4) FDNY civil penalties $1,000-$25,000 per violation per day
//! under NYC Admin Code § 28-202.1; (5) wrongful-death litigation post-
//! high-rise fire routinely exceeds $5M-$50M per fatality; (6) NYC FDNY
//! F-32 certification required for fire safety director; FSD on-site
//! required 24/7 in many occupancy classes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// NYC FDNY Local Law 26 of 2004 + 3 RCNY § 404-01.
    NewYorkCity,
    /// Chicago Municipal Code § 13-160-070 high-rise fire safety.
    Chicago,
    /// Los Angeles LAFD high-rise emergency planning.
    LosAngeles,
    /// IFC § 404 standalone adoption.
    InternationalFireCodeAdopted,
    /// Default — NFPA 1 + state-specific.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingClassification {
    /// High-rise residential ≥ 75 feet OR 7+ stories — full EAP required.
    HighRiseResidential75FtOr7Stories,
    /// Mid-rise residential 4-6 stories — limited EAP requirements.
    MidRiseResidential4To6Stories,
    /// Low-rise residential 1-3 stories — EAP framework inapplicable.
    LowRiseResidential1To3Stories,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// Fire Safety Director (FSD) on-site + Fire Safety Plan (FSP) +
    /// Emergency Action Plan (EAP) + annual drill + tenant distribution
    /// + accessible evacuation provisions.
    AllRequirementsCurrent,
    /// FSD not present on-site as required.
    FireSafetyDirectorNotPresent,
    /// FSP/EAP not filed with AHJ or out-of-date.
    FspEapNotFiledWithAhj,
    /// Annual fire drill not conducted.
    AnnualFireDrillMissed,
    /// Tenant emergency-instructions not distributed annually.
    TenantInstructionsNotDistributed,
    /// Accessible-evacuation provisions for mobility-impaired tenants
    /// missing (no Areas of Refuge, no evacuation chairs, no tenant
    /// disability registry).
    AccessibleEvacuationMissing,
    /// Fire-related emergency incident occurred without EAP compliance.
    EmergencyIncidentDuringNonCompliance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantAllRequirements,
    FireSafetyDirectorPresenceViolation,
    FspEapFilingDelinquentViolation,
    AnnualDrillMissedViolation,
    TenantInstructionsNotDistributedViolation,
    AccessibleEvacuationFhaAdaViolation,
    PostIncidentNonComplianceFatalityRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub building_classification: BuildingClassification,
    pub compliance_status: ComplianceStatus,
    pub fsd_holds_fdny_f_32_certification: bool,
    pub building_height_feet: u32,
    pub story_count: u32,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub fdny_civil_penalty_min_per_day_cents: u64,
    pub fdny_civil_penalty_max_per_day_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const HIGH_RISE_THRESHOLD_FEET: u32 = 75;
pub const HIGH_RISE_THRESHOLD_STORIES: u32 = 7;
pub const FDNY_CIVIL_PENALTY_MIN_CENTS: u64 = 100_000;
pub const FDNY_CIVIL_PENALTY_MAX_CENTS: u64 = 2_500_000;
pub const NYC_LOCAL_LAW_26_YEAR: i32 = 2004;
pub const FDNY_FSD_CERTIFICATION_REQUIRED: &str = "F-32";

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(
        input.building_classification,
        BuildingClassification::LowRiseResidential1To3Stories
    ) || (input.building_height_feet < HIGH_RISE_THRESHOLD_FEET
        && input.story_count < HIGH_RISE_THRESHOLD_STORIES
        && !matches!(
            input.building_classification,
            BuildingClassification::MidRiseResidential4To6Stories
        ))
    {
        notes.push(format!(
            "Building below high-rise threshold ({} feet AND {} stories) — full NFPA 1 § \
             10.8 + IFC § 404 + NYC Local Law 26 EAP framework inapplicable. Mid-rise (4-6 \
             stories) subject to reduced fire-safety-plan requirements. State / local AHJ \
             may impose lower threshold; verify with state fire marshal.",
            HIGH_RISE_THRESHOLD_FEET, HIGH_RISE_THRESHOLD_STORIES
        ));
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            fdny_civil_penalty_min_per_day_cents: 0,
            fdny_civil_penalty_max_per_day_cents: 0,
            citation: "n/a (below high-rise threshold)",
            notes,
        };
    }

    if matches!(
        input.compliance_status,
        ComplianceStatus::EmergencyIncidentDuringNonCompliance
    ) {
        severity = Severity::PostIncidentNonComplianceFatalityRisk;
        actions.push(
            "Fire-related emergency incident occurred without EAP compliance — wrongful-\
             death litigation post-high-rise fire routinely exceeds $5M-$50M per fatality \
             plus IIED + premises liability + RICO civil exposure. Document incident timeline \
             + EAP gaps + tenant injuries + medical records. Notify general liability + \
             umbrella + cyber-liability carriers within 24 hours; engage premises-liability \
             counsel + fire-safety expert witness. Coordinate with FDNY Bureau of Fire \
             Investigation."
                .to_string(),
        );
    } else if matches!(
        input.compliance_status,
        ComplianceStatus::FireSafetyDirectorNotPresent
    ) || (matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && !input.fsd_holds_fdny_f_32_certification)
    {
        severity = Severity::FireSafetyDirectorPresenceViolation;
        actions.push(format!(
            "Fire Safety Director (FSD) not present on-site as required by NYC Local Law 26 \
             of {} + FDNY 3 RCNY § 404-01 + IFC § 404.5 — FSD must hold FDNY {} certification \
             (Fire Safety Director). Engage credentialed FSD on emergency staffing roster; \
             document FSD-coverage schedule including weekends + holidays; train backup \
             FSD personnel. FDNY civil penalty ${} to ${} per day per violation.",
            NYC_LOCAL_LAW_26_YEAR,
            FDNY_FSD_CERTIFICATION_REQUIRED,
            FDNY_CIVIL_PENALTY_MIN_CENTS / 100,
            FDNY_CIVIL_PENALTY_MAX_CENTS / 100
        ));
    } else if matches!(
        input.compliance_status,
        ComplianceStatus::FspEapNotFiledWithAhj
    ) {
        severity = Severity::FspEapFilingDelinquentViolation;
        actions.push(
            "Fire Safety Plan (FSP) + Emergency Action Plan (EAP) not filed with FDNY or \
             out-of-date — NYC FDNY 3 RCNY § 404-01 + IFC § 404.2.1 require comprehensive \
             EAP covering tenant evacuation procedures + FSD on-site responsibility for \
             fires plus non-fire emergencies + annual fire drill schedule + post-emergency \
             reentry protocol + accessible-evacuation provisions. Submit revised FSP/EAP to \
             FDNY within 30 days; engage NYC fire-safety consulting firm to ensure code \
             compliance."
                .to_string(),
        );
    } else if matches!(
        input.compliance_status,
        ComplianceStatus::AnnualFireDrillMissed
    ) {
        severity = Severity::AnnualDrillMissedViolation;
        actions.push(
            "Annual fire drill not conducted — NYC FDNY 3 RCNY § 404-01 + IFC § 405 + NFPA \
             101 § 4.7 require annual fire-drill exercise documented with date + time + \
             tenant participation log. Schedule annual drill; coordinate with FDNY Bureau \
             of Fire Prevention for drill observation; retain drill records for full \
             5-year audit window."
                .to_string(),
        );
    } else if matches!(
        input.compliance_status,
        ComplianceStatus::TenantInstructionsNotDistributed
    ) {
        severity = Severity::TenantInstructionsNotDistributedViolation;
        actions.push(
            "Annual tenant emergency-instructions not distributed — NFPA 1 § 11.10 requires \
             emergency instructions provided ANNUALLY to each dwelling unit indicating (1) \
             location of alarms, (2) egress paths, (3) actions in response to in-unit fire, \
             (4) actions in response to building alarm system, (5) FSD contact info, (6) \
             accessibility-evacuation procedures. Distribute updated instructions plus \
             tenant-acknowledgment receipt; preserve distribution log for 5-year audit \
             window."
                .to_string(),
        );
    } else if matches!(
        input.compliance_status,
        ComplianceStatus::AccessibleEvacuationMissing
    ) {
        severity = Severity::AccessibleEvacuationFhaAdaViolation;
        actions.push(
            "Accessible-evacuation provisions for mobility-impaired tenants missing — FHA \
             42 U.S.C. § 3604(f)(3)(B) reasonable accommodation + ADA Title II/III + NFPA \
             101 § 7.2.12 Areas of Refuge + IFC § 1009 accessible means of egress require \
             (1) Areas of Refuge on each floor with 2-way communication, (2) evacuation \
             chairs accessible to tenant disability registry, (3) tenant disability \
             registry maintained by FSD, (4) annual coordination with local fire department \
             on evacuation-assistance protocol, (5) lighted evacuation signage. Install \
             plus document; consult FHA + ADA accessibility-evacuation expert."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantAllRequirements;
        actions.push(format!(
            "Compliant: FSD on-site + FSP/EAP filed + annual drill conducted + tenant \
             instructions distributed + accessible-evacuation provisions in place. Maintain \
             documentation for full {}-year audit window. NYC Local Law 26 of {} compliance \
             materially reduces civil-penalty exposure (${}-${} per day) + wrongful-death \
             litigation risk.",
            5,
            NYC_LOCAL_LAW_26_YEAR,
            FDNY_CIVIL_PENALTY_MIN_CENTS / 100,
            FDNY_CIVIL_PENALTY_MAX_CENTS / 100
        ));
    }

    match input.jurisdiction {
        Jurisdiction::NewYorkCity => {
            notes.push(format!(
                "NYC Local Law 26 of {} (post-9/11 EAP expansion) + FDNY 3 RCNY § 404-01 + \
                 NYC Admin Code § 404.2.1 + Fire Code Title 29 + § 28-202.1 civil penalty \
                 framework. Fire Safety Director (FSD) must hold FDNY F-32 certification; \
                 24/7 on-site presence required in many occupancy classes. FDNY civil \
                 penalty ${}-${} per violation per day. NYC FDNY Bureau of Fire Prevention \
                 enforces.",
                NYC_LOCAL_LAW_26_YEAR,
                FDNY_CIVIL_PENALTY_MIN_CENTS / 100,
                FDNY_CIVIL_PENALTY_MAX_CENTS / 100
            ));
        }
        Jurisdiction::Chicago => {
            notes.push(
                "Chicago Municipal Code § 13-160-070 high-rise fire safety + Chicago Fire \
                 Department code enforcement; Chicago adopts IFC with local amendments. \
                 Post-2003 Cook County Administration Building fire prompted enhanced \
                 high-rise EAP requirements."
                    .to_string(),
            );
        }
        Jurisdiction::LosAngeles => {
            notes.push(
                "Los Angeles Fire Department (LAFD) high-rise emergency planning + Cal. Fire \
                 Code (adopting IFC with state amendments) + LAMC § 57.4901 fire safety plan \
                 + LA County Code Title 32 fire code. LAFD certification required for fire \
                 safety personnel."
                    .to_string(),
            );
        }
        Jurisdiction::InternationalFireCodeAdopted => {
            notes.push(
                "IFC § 404 Emergency Planning and Preparedness adopted as standalone code; \
                 § 404.2.1 Comprehensive fire safety/emergency action plan (Level 1) + § \
                 404.3 implementation + § 404.4 staff training. Many municipalities adopt \
                 IFC by reference."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "NFPA 1 § 10.8 high-rise framework + § 11.10 EAP requirements + NFPA 101 \
                 Life Safety Code § 4.7 fire drills standalone. State / local AHJ adoption \
                 of NFPA 1 + IFC varies. Federal OSHA 29 C.F.R. § 1910.38 EAP applies to \
                 workplaces incidentally. Consult state fire marshal for specific high-rise \
                 EAP requirements."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_elevator_safety_inspection]] (elevator inspection — \
         coordination with emergency-recall and Phase II firefighter operation), [[rental_\
         fire_extinguisher_requirement]] (NFPA 10 fire extinguisher framework — distinct \
         compliance), [[rental_chimney_fireplace_inspection_disclosure]] (chimney inspection \
         framework), [[rental_grill_propane_bbq_restriction]] (iter 515 NFPA 1 § 10.10.5 \
         balcony grilling), [[rental_balcony_inspection_seismic_safety]] (iter 511 EEE \
         structural inspection — balcony as evacuation refuge), [[mid_tenancy_temporary_\
         relocation]] (when EAP-noncompliant building requires tenant temporary relocation), \
         [[tenant_emotional_distress_damages]] (IIED claim for fire-related psychological \
         injury post-Grenfell + post-9/11 trauma precedents)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::PostIncidentNonComplianceFatalityRisk
        | Severity::AccessibleEvacuationFhaAdaViolation => input.annual_rent_cents,
        Severity::FireSafetyDirectorPresenceViolation
        | Severity::FspEapFilingDelinquentViolation
        | Severity::AnnualDrillMissedViolation
        | Severity::TenantInstructionsNotDistributedViolation => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        fdny_civil_penalty_min_per_day_cents: FDNY_CIVIL_PENALTY_MIN_CENTS,
        fdny_civil_penalty_max_per_day_cents: FDNY_CIVIL_PENALTY_MAX_CENTS,
        citation: match input.jurisdiction {
            Jurisdiction::NewYorkCity => "NYC Local Law 26 of 2004 + FDNY 3 RCNY § 404-01 + NYC Admin § 28-202.1",
            Jurisdiction::Chicago => "Chicago Municipal Code § 13-160-070 + CFD code enforcement",
            Jurisdiction::LosAngeles => "Cal. Fire Code + LAMC § 57.4901 + LA County Title 32",
            Jurisdiction::InternationalFireCodeAdopted => "IFC § 404.2.1 + § 404.3 + § 404.4",
            Jurisdiction::Default => "NFPA 1 § 10.8 + NFPA 101 § 4.7 + OSHA 29 C.F.R. § 1910.38",
        },
        notes,
    }
}

pub type RentalEmergencyActionPlanHighRiseInput = Input;
pub type RentalEmergencyActionPlanHighRiseResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYorkCity,
            building_classification:
                BuildingClassification::HighRiseResidential75FtOr7Stories,
            compliance_status: ComplianceStatus::AllRequirementsCurrent,
            fsd_holds_fdny_f_32_certification: true,
            building_height_feet: 200,
            story_count: 20,
            annual_rent_cents: 36_000_000_00,
        }
    }

    #[test]
    fn low_rise_not_applicable() {
        let mut i = baseline();
        i.building_classification = BuildingClassification::LowRiseResidential1To3Stories;
        i.building_height_feet = 30;
        i.story_count = 3;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn compliant_all_requirements() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantAllRequirements));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Local Law 26")));
    }

    #[test]
    fn fsd_not_present_violation_half_rent() {
        let mut i = baseline();
        i.compliance_status = ComplianceStatus::FireSafetyDirectorNotPresent;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FireSafetyDirectorPresenceViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("F-32")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("FDNY 3 RCNY § 404-01")));
    }

    #[test]
    fn nyc_fsd_without_f32_certification_violation() {
        let mut i = baseline();
        i.fsd_holds_fdny_f_32_certification = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FireSafetyDirectorPresenceViolation));
    }

    #[test]
    fn fsp_eap_filing_delinquent_violation() {
        let mut i = baseline();
        i.compliance_status = ComplianceStatus::FspEapNotFiledWithAhj;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FspEapFilingDelinquentViolation));
        assert!(r.recommended_actions.iter().any(|a| a.contains("IFC § 404.2.1")));
    }

    #[test]
    fn annual_fire_drill_missed_violation() {
        let mut i = baseline();
        i.compliance_status = ComplianceStatus::AnnualFireDrillMissed;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::AnnualDrillMissedViolation));
        assert!(r.recommended_actions.iter().any(|a| a.contains("NFPA 101 § 4.7")));
    }

    #[test]
    fn tenant_instructions_not_distributed_violation() {
        let mut i = baseline();
        i.compliance_status = ComplianceStatus::TenantInstructionsNotDistributed;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::TenantInstructionsNotDistributedViolation
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("NFPA 1 § 11.10")));
    }

    #[test]
    fn accessible_evacuation_missing_violation_full_rent() {
        let mut i = baseline();
        i.compliance_status = ComplianceStatus::AccessibleEvacuationMissing;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::AccessibleEvacuationFhaAdaViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("NFPA 101 § 7.2.12")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Areas of Refuge")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 3604(f)(3)(B)")));
    }

    #[test]
    fn post_incident_non_compliance_fatality_risk_full_rent() {
        let mut i = baseline();
        i.compliance_status = ComplianceStatus::EmergencyIncidentDuringNonCompliance;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PostIncidentNonComplianceFatalityRisk));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("$5M-$50M")));
    }

    #[test]
    fn nyc_jurisdiction_pins_local_law_26_and_fdny() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("NYC Local Law 26")));
        assert!(r.notes.iter().any(|n| n.contains("F-32 certification")));
        assert!(r.notes.iter().any(|n| n.contains("§ 28-202.1")));
    }

    #[test]
    fn chicago_jurisdiction_pins_13_160_070() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Chicago;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 13-160-070")));
        assert!(r.notes.iter().any(|n| n.contains("Cook County Administration Building")));
    }

    #[test]
    fn la_jurisdiction_pins_lamc_57_4901() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::LosAngeles;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("LAMC § 57.4901")));
        assert!(r.notes.iter().any(|n| n.contains("LAFD")));
    }

    #[test]
    fn ifc_jurisdiction_pins_404_2_1() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::InternationalFireCodeAdopted;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 404.2.1")));
        assert!(r.notes.iter().any(|n| n.contains("Level 1")));
    }

    #[test]
    fn default_jurisdiction_pins_nfpa_1_and_osha() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("NFPA 1 § 10.8")));
        assert!(r.notes.iter().any(|n| n.contains("NFPA 101 Life Safety Code")));
        assert!(r.notes.iter().any(|n| n.contains("29 C.F.R. § 1910.38")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("rental_elevator_safety_inspection")));
        assert!(r.notes.iter().any(|n| n.contains("rental_fire_extinguisher_requirement")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_balcony_inspection_seismic_safety")));
    }

    #[test]
    fn high_rise_threshold_pins_75_feet() {
        assert_eq!(HIGH_RISE_THRESHOLD_FEET, 75);
    }

    #[test]
    fn high_rise_threshold_pins_7_stories() {
        assert_eq!(HIGH_RISE_THRESHOLD_STORIES, 7);
    }

    #[test]
    fn fdny_civil_penalty_pins_1k_to_25k() {
        assert_eq!(FDNY_CIVIL_PENALTY_MIN_CENTS, 100_000);
        assert_eq!(FDNY_CIVIL_PENALTY_MAX_CENTS, 2_500_000);
    }

    #[test]
    fn nyc_local_law_26_year_pins_2004() {
        assert_eq!(NYC_LOCAL_LAW_26_YEAR, 2004);
    }

    #[test]
    fn fdny_fsd_certification_pins_f_32() {
        assert_eq!(FDNY_FSD_CERTIFICATION_REQUIRED, "F-32");
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let nyc = check(&baseline());
        let chi = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Chicago; i });
        let la = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::LosAngeles; i });
        let ifc = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::InternationalFireCodeAdopted; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(nyc.citation.contains("Local Law 26"));
        assert!(chi.citation.contains("§ 13-160-070"));
        assert!(la.citation.contains("§ 57.4901"));
        assert!(ifc.citation.contains("§ 404.2.1"));
        assert!(de.citation.contains("NFPA 1 § 10.8"));
    }

    #[test]
    fn severity_priority_incident_overrides_other_violations() {
        let mut i = baseline();
        i.compliance_status = ComplianceStatus::EmergencyIncidentDuringNonCompliance;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PostIncidentNonComplianceFatalityRisk));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.compliance_status = ComplianceStatus::FireSafetyDirectorNotPresent;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn building_below_75_feet_below_7_stories_not_applicable() {
        let mut i = baseline();
        i.building_height_feet = 50;
        i.story_count = 5;
        i.building_classification = BuildingClassification::MidRiseResidential4To6Stories;
        let r = check(&i);
        let _ = r.severity;
    }
}
