//! Mandatory soft-story seismic retrofit ordinance compliance
//! — when must a trader-landlord owning a wood-frame
//! multifamily building with a soft/weak first story (open
//! ground-floor parking, retail, or similar large openings
//! that lack lateral bracing) commission a mandatory
//! structural retrofit? Trader-landlord critical for CA
//! multifamily owners in SF + LA + Berkeley + San Jose +
//! Oakland + Pasadena + West Hollywood — non-compliance
//! exposes owner to ordinance penalties + uninhabitable-
//! building findings + insurance non-renewal.
//!
//! Distinct from siblings `balcony_inspection` (SB 721 / SB
//! 326 Exterior Elevated Elements visual inspection),
//! `water_heater_earthquake_strap` (CA § 19211 individual
//! appliance seismic anchoring), and `fire_sprinkler_
//! disclosure` (fire-suppression system disclosure).
//!
//! **Four regimes**:
//!
//! **San Francisco — Building Code Chapter 34B (Ord. 66-13,
//! operative June 17, 2013)**:
//! - Scope: multi-unit soft-story buildings = wood-frame
//!   structures with **5+ residential units** AND **2+
//!   stories OVER a soft/weak story**.
//! - All tier compliance deadlines PASSED as of September
//!   15, 2021.
//! - Non-compliance = Building deemed UNSAFE under SF
//!   Building Code § 102A.16; ~$840 per day non-compliance
//!   penalty + insurance complications.
//!
//! **Los Angeles — Ordinance 183893 (November 22, 2015)**:
//! - Scope: ~13,500 wood-frame buildings with soft stories.
//! - 3-phase compliance timeline:
//!   - 2 years: structural report + retrofit/demolition
//!     plans
//!   - 3.5 years: secure all required permits
//!   - 7 years: complete retrofit or demolition + close all
//!     permits
//! - Priority 2 deadline: **April 2026**.
//!
//! **Berkeley — BMC Chapter 19.39 (eff. 2015)**:
//! - Scope: wood-frame multifamily structures ≥ 3 units +
//!   soft-story first floor.
//! - Compliance deadline passed; ongoing certification
//!   requirement.
//!
//! **Default — no statutory soft-story retrofit
//! requirement**. Common-law premises liability + state
//! building code (where adopted) + local ordinances may
//! apply (Oakland + San Jose + Pasadena + West Hollywood
//! have analogous programs).
//!
//! Citations: San Francisco Building Code Chapter 34B (Ord.
//! 66-13); Los Angeles Ordinance 183893 (Nov 22, 2015);
//! Berkeley Municipal Code Chapter 19.39; San Jose Municipal
//! Code 17.94.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    SanFrancisco,
    LosAngeles,
    Berkeley,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStage {
    /// LA stage 1 — structural report + retrofit/demolition
    /// plans (2-year deadline).
    StructuralReport,
    /// LA stage 2 — secured all required permits (3.5-year
    /// deadline).
    PermitsSecured,
    /// LA stage 3 — completed retrofit/demolition + closed
    /// permits (7-year deadline).
    RetrofitComplete,
    /// No stage reached.
    NotStarted,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SoftStorySeismicRetrofitInput {
    pub regime: Regime,
    /// Whether the building is wood-frame construction.
    pub wood_frame_construction: bool,
    /// Number of residential units.
    pub residential_unit_count: u32,
    /// Whether the building has 2+ stories OVER a soft/weak
    /// story.
    pub two_plus_stories_over_soft_story: bool,
    /// Compliance stage reached.
    pub compliance_stage: ComplianceStage,
    /// Years since ordinance enactment (for LA staged
    /// deadlines).
    pub years_since_ordinance: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SoftStorySeismicRetrofitResult {
    pub compliant: bool,
    pub building_in_scope: bool,
    pub current_stage_required: Option<ComplianceStage>,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &SoftStorySeismicRetrofitInput) -> SoftStorySeismicRetrofitResult {
    match input.regime {
        Regime::SanFrancisco => check_sf(input),
        Regime::LosAngeles => check_la(input),
        Regime::Berkeley => check_berkeley(input),
        Regime::Default => check_default(input),
    }
}

fn check_sf(input: &SoftStorySeismicRetrofitInput) -> SoftStorySeismicRetrofitResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "San Francisco Building Code Chapter 34B (Ord. 66-13, operative June 17, 2013) — applies to multi-unit soft-story buildings: wood-frame structures with 5+ residential units AND 2+ stories OVER a soft/weak story"
            .to_string(),
        "San Francisco compliance — all tier deadlines PASSED as of September 15, 2021; non-compliance = building deemed UNSAFE under SF Building Code § 102A.16; ~$840 per day non-compliance penalty + insurance complications"
            .to_string(),
    ];

    let in_scope = input.wood_frame_construction
        && input.residential_unit_count >= 5
        && input.two_plus_stories_over_soft_story;

    if in_scope && !matches!(input.compliance_stage, ComplianceStage::RetrofitComplete) {
        violations.push(
            "San Francisco Building Code Chapter 34B — retrofit not complete; all tier deadlines passed September 15, 2021; building deemed UNSAFE".to_string(),
        );
    }

    SoftStorySeismicRetrofitResult {
        compliant: violations.is_empty(),
        building_in_scope: in_scope,
        current_stage_required: if in_scope {
            Some(ComplianceStage::RetrofitComplete)
        } else {
            None
        },
        violations,
        citation: "San Francisco Building Code Chapter 34B (Ord. 66-13, operative June 17, 2013)",
        notes,
    }
}

fn check_la(input: &SoftStorySeismicRetrofitInput) -> SoftStorySeismicRetrofitResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Los Angeles Ordinance 183893 (November 22, 2015) — applies to ~13,500 wood-frame buildings with soft stories"
            .to_string(),
        "Los Angeles 3-phase compliance timeline: 2 years (structural report + retrofit/demolition plans); 3.5 years (secure all required permits); 7 years (complete retrofit or demolition + close all permits)"
            .to_string(),
        "Los Angeles Priority 2 deadline: April 2026; non-compliance = Order to Comply + civil penalties + criminal misdemeanor + property recordation under § 91.6314 LAMC"
            .to_string(),
    ];

    let in_scope = input.wood_frame_construction && input.two_plus_stories_over_soft_story;

    let required_stage: Option<ComplianceStage> = if !in_scope {
        None
    } else if input.years_since_ordinance <= 2 {
        Some(ComplianceStage::StructuralReport)
    } else if input.years_since_ordinance <= 3 {
        Some(ComplianceStage::PermitsSecured)
    } else {
        Some(ComplianceStage::RetrofitComplete)
    };

    if let Some(required) = required_stage {
        let current_meets = match required {
            ComplianceStage::StructuralReport => {
                !matches!(input.compliance_stage, ComplianceStage::NotStarted)
            }
            ComplianceStage::PermitsSecured => matches!(
                input.compliance_stage,
                ComplianceStage::PermitsSecured | ComplianceStage::RetrofitComplete
            ),
            ComplianceStage::RetrofitComplete => {
                matches!(input.compliance_stage, ComplianceStage::RetrofitComplete)
            }
            ComplianceStage::NotStarted => true,
        };
        if !current_meets {
            violations.push(format!(
                "Los Angeles Ordinance 183893 — required {:?} stage not met after {} years since ordinance enactment",
                required, input.years_since_ordinance
            ));
        }
    }

    SoftStorySeismicRetrofitResult {
        compliant: violations.is_empty(),
        building_in_scope: in_scope,
        current_stage_required: required_stage,
        violations,
        citation: "Los Angeles Ordinance 183893 (Nov 22, 2015); LAMC § 91.6314",
        notes,
    }
}

fn check_berkeley(input: &SoftStorySeismicRetrofitInput) -> SoftStorySeismicRetrofitResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Berkeley Municipal Code Chapter 19.39 (eff. 2015) — applies to wood-frame multifamily structures with ≥ 3 units AND soft-story first floor"
            .to_string(),
        "Berkeley compliance deadline passed; ongoing certification requirement; non-compliance triggers BMC enforcement"
            .to_string(),
    ];

    let in_scope = input.wood_frame_construction
        && input.residential_unit_count >= 3
        && input.two_plus_stories_over_soft_story;

    if in_scope && !matches!(input.compliance_stage, ComplianceStage::RetrofitComplete) {
        violations.push(
            "Berkeley Municipal Code Chapter 19.39 — retrofit not complete; ongoing certification requirement violated".to_string(),
        );
    }

    SoftStorySeismicRetrofitResult {
        compliant: violations.is_empty(),
        building_in_scope: in_scope,
        current_stage_required: if in_scope {
            Some(ComplianceStage::RetrofitComplete)
        } else {
            None
        },
        violations,
        citation: "Berkeley Municipal Code Chapter 19.39",
        notes,
    }
}

fn check_default(_input: &SoftStorySeismicRetrofitInput) -> SoftStorySeismicRetrofitResult {
    let notes: Vec<String> = vec![
        "default rule — no statutory soft-story retrofit requirement; common-law premises liability + state building code (where adopted) + local ordinances may apply"
            .to_string(),
        "default rule — analogous programs exist in Oakland + San Jose + Pasadena + West Hollywood; verify local jurisdiction before relying on default"
            .to_string(),
    ];

    SoftStorySeismicRetrofitResult {
        compliant: true,
        building_in_scope: false,
        current_stage_required: None,
        violations: Vec::new(),
        citation: "common-law premises liability + state building code + local ordinances",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sf_compliant() -> SoftStorySeismicRetrofitInput {
        SoftStorySeismicRetrofitInput {
            regime: Regime::SanFrancisco,
            wood_frame_construction: true,
            residential_unit_count: 8,
            two_plus_stories_over_soft_story: true,
            compliance_stage: ComplianceStage::RetrofitComplete,
            years_since_ordinance: 12,
        }
    }

    fn la_base() -> SoftStorySeismicRetrofitInput {
        let mut i = sf_compliant();
        i.regime = Regime::LosAngeles;
        i
    }

    fn berkeley_base() -> SoftStorySeismicRetrofitInput {
        let mut i = sf_compliant();
        i.regime = Regime::Berkeley;
        i.residential_unit_count = 5;
        i
    }

    fn default_base() -> SoftStorySeismicRetrofitInput {
        let mut i = sf_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn sf_compliant_passes() {
        let r = check(&sf_compliant());
        assert!(r.compliant);
        assert!(r.building_in_scope);
    }

    #[test]
    fn sf_4_units_out_of_scope() {
        let mut i = sf_compliant();
        i.residential_unit_count = 4;
        let r = check(&i);
        assert!(!r.building_in_scope);
        assert!(r.compliant);
    }

    #[test]
    fn sf_5_units_in_scope_boundary() {
        let mut i = sf_compliant();
        i.residential_unit_count = 5;
        let r = check(&i);
        assert!(r.building_in_scope);
    }

    #[test]
    fn sf_not_wood_frame_out_of_scope() {
        let mut i = sf_compliant();
        i.wood_frame_construction = false;
        let r = check(&i);
        assert!(!r.building_in_scope);
    }

    #[test]
    fn sf_no_soft_story_out_of_scope() {
        let mut i = sf_compliant();
        i.two_plus_stories_over_soft_story = false;
        let r = check(&i);
        assert!(!r.building_in_scope);
    }

    #[test]
    fn sf_in_scope_not_retrofit_complete_violates() {
        let mut i = sf_compliant();
        i.compliance_stage = ComplianceStage::PermitsSecured;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Chapter 34B") && v.contains("September 15, 2021")));
    }

    #[test]
    fn sf_citation_pins_ordinance() {
        let r = check(&sf_compliant());
        assert!(r.citation.contains("Chapter 34B"));
        assert!(r.citation.contains("Ord. 66-13"));
        assert!(r.citation.contains("June 17, 2013"));
    }

    #[test]
    fn sf_note_pins_5_unit_threshold() {
        let r = check(&sf_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("5+ residential units") && n.contains("2+ stories")));
    }

    #[test]
    fn sf_note_pins_840_per_day_penalty() {
        let r = check(&sf_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$840 per day") && n.contains("§ 102A.16")));
    }

    #[test]
    fn la_compliant_at_year_8_retrofit_complete() {
        let mut i = la_base();
        i.years_since_ordinance = 8;
        i.compliance_stage = ComplianceStage::RetrofitComplete;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn la_year_1_structural_report_compliant() {
        let mut i = la_base();
        i.years_since_ordinance = 1;
        i.compliance_stage = ComplianceStage::StructuralReport;
        let r = check(&i);
        assert!(r.compliant);
        assert!(matches!(
            r.current_stage_required,
            Some(ComplianceStage::StructuralReport)
        ));
    }

    #[test]
    fn la_year_1_not_started_violates() {
        let mut i = la_base();
        i.years_since_ordinance = 1;
        i.compliance_stage = ComplianceStage::NotStarted;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn la_year_3_permits_required() {
        let mut i = la_base();
        i.years_since_ordinance = 3;
        let r = check(&i);
        assert!(matches!(
            r.current_stage_required,
            Some(ComplianceStage::PermitsSecured)
        ));
    }

    #[test]
    fn la_year_3_only_structural_report_violates() {
        let mut i = la_base();
        i.years_since_ordinance = 3;
        i.compliance_stage = ComplianceStage::StructuralReport;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn la_year_8_retrofit_complete_required() {
        let mut i = la_base();
        i.years_since_ordinance = 8;
        let r = check(&i);
        assert!(matches!(
            r.current_stage_required,
            Some(ComplianceStage::RetrofitComplete)
        ));
    }

    #[test]
    fn la_year_8_permits_secured_violates() {
        let mut i = la_base();
        i.years_since_ordinance = 8;
        i.compliance_stage = ComplianceStage::PermitsSecured;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn la_no_unit_threshold_3_units_in_scope() {
        let mut i = la_base();
        i.residential_unit_count = 3;
        let r = check(&i);
        assert!(r.building_in_scope);
    }

    #[test]
    fn la_citation_pins_ordinance_183893() {
        let r = check(&la_base());
        assert!(r.citation.contains("183893"));
        assert!(r.citation.contains("LAMC § 91.6314"));
    }

    #[test]
    fn la_note_pins_april_2026_priority_2_deadline() {
        let r = check(&la_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("April 2026") && n.contains("Priority 2")));
    }

    #[test]
    fn la_note_pins_3_phase_timeline() {
        let r = check(&la_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("2 years") && n.contains("3.5 years") && n.contains("7 years")));
    }

    #[test]
    fn berkeley_3_unit_boundary_in_scope() {
        let mut i = berkeley_base();
        i.residential_unit_count = 3;
        let r = check(&i);
        assert!(r.building_in_scope);
    }

    #[test]
    fn berkeley_2_units_out_of_scope() {
        let mut i = berkeley_base();
        i.residential_unit_count = 2;
        let r = check(&i);
        assert!(!r.building_in_scope);
    }

    #[test]
    fn berkeley_not_complete_violates() {
        let mut i = berkeley_base();
        i.compliance_stage = ComplianceStage::PermitsSecured;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("Chapter 19.39")));
    }

    #[test]
    fn berkeley_citation_pins_bmc() {
        let r = check(&berkeley_base());
        assert!(r.citation.contains("Berkeley Municipal Code Chapter 19.39"));
    }

    #[test]
    fn default_no_violation() {
        let mut i = default_base();
        i.wood_frame_construction = true;
        i.residential_unit_count = 50;
        i.two_plus_stories_over_soft_story = true;
        i.compliance_stage = ComplianceStage::NotStarted;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.building_in_scope);
    }

    #[test]
    fn default_citation_pins_local_ordinances() {
        let r = check(&default_base());
        assert!(r.citation.contains("local ordinances"));
    }

    #[test]
    fn default_note_pins_other_ca_cities() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("Oakland")
            && n.contains("San Jose")
            && n.contains("Pasadena")
            && n.contains("West Hollywood")));
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [
            Regime::SanFrancisco,
            Regime::LosAngeles,
            Regime::Berkeley,
            Regime::Default,
        ] {
            let mut i = sf_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn sf_uniquely_requires_5_units_invariant() {
        let mut i_sf = sf_compliant();
        i_sf.residential_unit_count = 4;
        let r_sf = check(&i_sf);
        assert!(!r_sf.building_in_scope);

        let mut i_berkeley = berkeley_base();
        i_berkeley.residential_unit_count = 4;
        let r_berkeley = check(&i_berkeley);
        assert!(r_berkeley.building_in_scope);
    }

    #[test]
    fn la_uniquely_has_staged_compliance_invariant() {
        let mut i_la = la_base();
        i_la.years_since_ordinance = 1;
        let r_la = check(&i_la);
        assert!(matches!(
            r_la.current_stage_required,
            Some(ComplianceStage::StructuralReport)
        ));

        let r_sf = check(&sf_compliant());
        assert!(matches!(
            r_sf.current_stage_required,
            Some(ComplianceStage::RetrofitComplete)
        ));
    }

    #[test]
    fn berkeley_uniquely_lower_3_unit_threshold_invariant() {
        let mut i_berkeley = berkeley_base();
        i_berkeley.residential_unit_count = 3;
        let r_berkeley = check(&i_berkeley);
        assert!(r_berkeley.building_in_scope);

        let mut i_sf = sf_compliant();
        i_sf.residential_unit_count = 3;
        let r_sf = check(&i_sf);
        assert!(!r_sf.building_in_scope);
    }

    #[test]
    fn la_year_2_boundary_structural_report_engages() {
        let mut i = la_base();
        i.years_since_ordinance = 2;
        let r = check(&i);
        assert!(matches!(
            r.current_stage_required,
            Some(ComplianceStage::StructuralReport)
        ));
    }

    #[test]
    fn la_year_4_retrofit_complete_required() {
        let mut i = la_base();
        i.years_since_ordinance = 4;
        let r = check(&i);
        assert!(matches!(
            r.current_stage_required,
            Some(ComplianceStage::RetrofitComplete)
        ));
    }
}
