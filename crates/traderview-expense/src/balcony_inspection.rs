//! California SB 721 (multifamily rental) + SB 326 (condos
//! under Davis-Stirling Act) Exterior Elevated Elements (EEE)
//! inspection compliance — when must a trader-landlord
//! commission a licensed-inspector visual inspection of
//! balconies + decks + exterior stairways + landings + similar
//! wood-framed elements supported by wood framing attached to
//! a multifamily building and elevated more than 6 feet above
//! ground? Trader-landlord critical for multi-unit owners:
//! missed first-inspection deadline of January 1, 2026 (per
//! AB 2579's extension from January 1, 2025) exposes building
//! to safety enforcement + liability.
//!
//! **Three regimes**:
//!
//! **California SB 721 (Cal. Health & Safety Code § 17973;
//! multifamily rental, 3+ units)** — first inspection by
//! **January 1, 2026** (AB 2579 extended from January 1,
//! 2025); recurring inspection every **6 years** thereafter.
//! Qualified inspectors: licensed architects, licensed civil
//! or structural engineers, building contractors with A / B /
//! C-5 license + 5+ years experience, or certified building
//! inspectors. Inspector CANNOT be local government employee
//! AND CANNOT be entity performing repairs. Minimum 15% of
//! each EEE type by direct visual inspection + exploratory
//! openings. Repair deadline: 120 days after identified.
//!
//! **California SB 326 (Cal. Civ. Code § 5551; Davis-Stirling
//! Act condos)** — first inspection by **January 1, 2025**;
//! recurring every **9 years** thereafter. Qualified
//! inspectors limited to licensed architects + civil /
//! structural engineers (no general contractors).
//!
//! **Default — no statutory inspection regime**. Common-law
//! premises-liability duty + jurisdiction-specific local
//! ordinances may apply.
//!
//! Citations: Cal. Health & Safety Code § 17973 (SB 721);
//! Cal. Civ. Code § 5551 (SB 326); AB 2579 (2023 SB 721
//! deadline extension to Jan 1, 2026).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    CaliforniaSb721,
    CaliforniaSb326,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InspectorQualification {
    LicensedArchitect,
    LicensedCivilOrStructuralEngineer,
    ContractorAbc5WithExperience,
    CertifiedBuildingInspector,
    LocalGovernmentEmployee,
    EntityPerformingRepairs,
    Unqualified,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalconyInspectionInput {
    pub regime: Regime,
    pub unit_count: u32,
    pub inspector_qualification: InspectorQualification,
    /// Whether inspection completed by required deadline.
    pub inspection_completed_by_deadline: bool,
    /// Whether the minimum 15% sample of each EEE type
    /// was visually inspected including exploratory openings.
    pub minimum_15_percent_sampled: bool,
    /// Days since identified-repair finding (for 120-day
    /// repair window).
    pub days_since_repair_finding: u32,
    /// Whether identified repairs have been completed.
    pub repairs_completed: bool,
    /// Whether there is at least one EEE element more than
    /// 6 feet above ground.
    pub has_eee_above_6_feet: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BalconyInspectionResult {
    pub compliant: bool,
    pub inspection_required: bool,
    pub first_inspection_deadline: &'static str,
    pub recurring_cycle_years: u32,
    pub qualified_inspector: bool,
    pub repair_deadline_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &BalconyInspectionInput) -> BalconyInspectionResult {
    match input.regime {
        Regime::CaliforniaSb721 => check_sb721(input),
        Regime::CaliforniaSb326 => check_sb326(input),
        Regime::Default => check_default(input),
    }
}

fn check_sb721(input: &BalconyInspectionInput) -> BalconyInspectionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code § 17973 (SB 721) — multifamily rental with 3+ units; first inspection by January 1, 2026 (AB 2579 extended from January 1, 2025); recurring every 6 years"
            .to_string(),
        "SB 721 qualified inspector: licensed architect, licensed civil or structural engineer, contractor with A / B / C-5 license + 5+ years experience, or certified building inspector; NOT local government employee AND NOT entity performing repairs"
            .to_string(),
        "SB 721 sampling — minimum 15% of each EEE type by direct visual inspection + exploratory openings; 120-day repair deadline after finding"
            .to_string(),
    ];

    let inspection_required = input.unit_count >= 3 && input.has_eee_above_6_feet;

    let qualified = matches!(
        input.inspector_qualification,
        InspectorQualification::LicensedArchitect
            | InspectorQualification::LicensedCivilOrStructuralEngineer
            | InspectorQualification::ContractorAbc5WithExperience
            | InspectorQualification::CertifiedBuildingInspector
    );

    if inspection_required {
        if !input.inspection_completed_by_deadline {
            violations.push(
                "Cal. Health & Safety Code § 17973(c) — failed to complete EEE inspection by January 1, 2026 deadline (AB 2579 extension)".to_string(),
            );
        }
        if !qualified {
            violations.push(format!(
                "Cal. Health & Safety Code § 17973(d) — inspector not qualified under SB 721 (got {:?})",
                input.inspector_qualification
            ));
        }
        if !input.minimum_15_percent_sampled {
            violations.push(
                "Cal. Health & Safety Code § 17973(e) — minimum 15% of each EEE type by direct visual inspection + exploratory openings not satisfied".to_string(),
            );
        }
        if input.days_since_repair_finding > 120 && !input.repairs_completed {
            violations.push(format!(
                "Cal. Health & Safety Code § 17973(f) — 120-day repair deadline missed ({} days elapsed since finding)",
                input.days_since_repair_finding
            ));
        }
    }

    BalconyInspectionResult {
        compliant: violations.is_empty(),
        inspection_required,
        first_inspection_deadline: "January 1, 2026",
        recurring_cycle_years: 6,
        qualified_inspector: qualified,
        repair_deadline_engaged: inspection_required && input.days_since_repair_finding > 0,
        violations,
        citation: "Cal. Health & Safety Code § 17973 (SB 721); AB 2579 (2023)",
        notes,
    }
}

fn check_sb326(input: &BalconyInspectionInput) -> BalconyInspectionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 5551 (SB 326) — Davis-Stirling Act condos; first inspection by January 1, 2025; recurring every 9 years"
            .to_string(),
        "SB 326 qualified inspector LIMITED to licensed architect or licensed civil / structural engineer (no general contractors); inspector NOT local government employee AND NOT entity performing repairs"
            .to_string(),
    ];

    let inspection_required = input.has_eee_above_6_feet;

    let qualified = matches!(
        input.inspector_qualification,
        InspectorQualification::LicensedArchitect
            | InspectorQualification::LicensedCivilOrStructuralEngineer
    );

    if inspection_required {
        if !input.inspection_completed_by_deadline {
            violations.push(
                "Cal. Civ. Code § 5551 — failed to complete EEE inspection by January 1, 2025 SB 326 deadline".to_string(),
            );
        }
        if !qualified {
            violations.push(
                "Cal. Civ. Code § 5551(b) — inspector not qualified under SB 326 (licensed architect or licensed civil / structural engineer ONLY)".to_string(),
            );
        }
    }

    BalconyInspectionResult {
        compliant: violations.is_empty(),
        inspection_required,
        first_inspection_deadline: "January 1, 2025",
        recurring_cycle_years: 9,
        qualified_inspector: qualified,
        repair_deadline_engaged: false,
        violations,
        citation: "Cal. Civ. Code § 5551 (SB 326)",
        notes,
    }
}

fn check_default(_input: &BalconyInspectionInput) -> BalconyInspectionResult {
    let notes: Vec<String> = vec![
        "default rule — no statutory EEE inspection regime; common-law premises-liability duty + jurisdiction-specific local ordinances may apply"
            .to_string(),
        "default rule — non-CA jurisdictions may have analogous statutes (e.g., Berkeley local ordinance pre-SB 721) and condo CC&R may impose contractual inspection obligations"
            .to_string(),
    ];

    BalconyInspectionResult {
        compliant: true,
        inspection_required: false,
        first_inspection_deadline: "no statutory deadline",
        recurring_cycle_years: 0,
        qualified_inspector: false,
        repair_deadline_engaged: false,
        violations: Vec::new(),
        citation: "common-law premises-liability + local ordinances",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sb721_compliant() -> BalconyInspectionInput {
        BalconyInspectionInput {
            regime: Regime::CaliforniaSb721,
            unit_count: 6,
            inspector_qualification: InspectorQualification::LicensedArchitect,
            inspection_completed_by_deadline: true,
            minimum_15_percent_sampled: true,
            days_since_repair_finding: 0,
            repairs_completed: true,
            has_eee_above_6_feet: true,
        }
    }

    fn sb326_compliant() -> BalconyInspectionInput {
        let mut i = sb721_compliant();
        i.regime = Regime::CaliforniaSb326;
        i
    }

    fn default_base() -> BalconyInspectionInput {
        let mut i = sb721_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn sb721_compliant_passes() {
        let r = check(&sb721_compliant());
        assert!(r.compliant);
        assert!(r.inspection_required);
        assert!(r.qualified_inspector);
        assert_eq!(r.first_inspection_deadline, "January 1, 2026");
        assert_eq!(r.recurring_cycle_years, 6);
    }

    #[test]
    fn sb721_2_units_no_inspection_required() {
        let mut i = sb721_compliant();
        i.unit_count = 2;
        let r = check(&i);
        assert!(!r.inspection_required);
        assert!(r.compliant);
    }

    #[test]
    fn sb721_3_units_boundary_inspection_required() {
        let mut i = sb721_compliant();
        i.unit_count = 3;
        let r = check(&i);
        assert!(r.inspection_required);
    }

    #[test]
    fn sb721_no_eee_above_6_feet_no_inspection_required() {
        let mut i = sb721_compliant();
        i.has_eee_above_6_feet = false;
        let r = check(&i);
        assert!(!r.inspection_required);
        assert!(r.compliant);
    }

    #[test]
    fn sb721_missed_deadline_violates() {
        let mut i = sb721_compliant();
        i.inspection_completed_by_deadline = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("January 1, 2026") && v.contains("AB 2579")));
    }

    #[test]
    fn sb721_unqualified_inspector_violates() {
        let mut i = sb721_compliant();
        i.inspector_qualification = InspectorQualification::Unqualified;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(!r.qualified_inspector);
    }

    #[test]
    fn sb721_local_gov_employee_disqualified() {
        let mut i = sb721_compliant();
        i.inspector_qualification = InspectorQualification::LocalGovernmentEmployee;
        let r = check(&i);
        assert!(!r.qualified_inspector);
        assert!(!r.compliant);
    }

    #[test]
    fn sb721_entity_performing_repairs_disqualified() {
        let mut i = sb721_compliant();
        i.inspector_qualification = InspectorQualification::EntityPerformingRepairs;
        let r = check(&i);
        assert!(!r.qualified_inspector);
        assert!(!r.compliant);
    }

    #[test]
    fn sb721_missing_15_percent_sampling_violates() {
        let mut i = sb721_compliant();
        i.minimum_15_percent_sampled = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("15%")));
    }

    #[test]
    fn sb721_121_day_repair_window_violation() {
        let mut i = sb721_compliant();
        i.days_since_repair_finding = 121;
        i.repairs_completed = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("120-day") && v.contains("121 days")));
    }

    #[test]
    fn sb721_120_day_repair_window_compliant() {
        let mut i = sb721_compliant();
        i.days_since_repair_finding = 120;
        i.repairs_completed = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn sb721_contractor_abc5_qualifies() {
        let mut i = sb721_compliant();
        i.inspector_qualification = InspectorQualification::ContractorAbc5WithExperience;
        let r = check(&i);
        assert!(r.qualified_inspector);
        assert!(r.compliant);
    }

    #[test]
    fn sb721_certified_inspector_qualifies() {
        let mut i = sb721_compliant();
        i.inspector_qualification = InspectorQualification::CertifiedBuildingInspector;
        let r = check(&i);
        assert!(r.qualified_inspector);
        assert!(r.compliant);
    }

    #[test]
    fn sb721_civil_engineer_qualifies() {
        let mut i = sb721_compliant();
        i.inspector_qualification = InspectorQualification::LicensedCivilOrStructuralEngineer;
        let r = check(&i);
        assert!(r.qualified_inspector);
        assert!(r.compliant);
    }

    #[test]
    fn sb721_citation_pins_authorities() {
        let r = check(&sb721_compliant());
        assert!(r.citation.contains("§ 17973"));
        assert!(r.citation.contains("SB 721"));
        assert!(r.citation.contains("AB 2579"));
    }

    #[test]
    fn sb326_compliant_passes() {
        let r = check(&sb326_compliant());
        assert!(r.compliant);
        assert!(r.inspection_required);
        assert_eq!(r.first_inspection_deadline, "January 1, 2025");
        assert_eq!(r.recurring_cycle_years, 9);
    }

    #[test]
    fn sb326_contractor_disqualified() {
        let mut i = sb326_compliant();
        i.inspector_qualification = InspectorQualification::ContractorAbc5WithExperience;
        let r = check(&i);
        assert!(!r.qualified_inspector);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 5551(b)") && v.contains("ONLY")));
    }

    #[test]
    fn sb326_no_unit_count_requirement_applies_to_all_condos() {
        let mut i = sb326_compliant();
        i.unit_count = 1;
        let r = check(&i);
        assert!(r.inspection_required);
    }

    #[test]
    fn sb326_civil_engineer_qualifies() {
        let mut i = sb326_compliant();
        i.inspector_qualification = InspectorQualification::LicensedCivilOrStructuralEngineer;
        let r = check(&i);
        assert!(r.qualified_inspector);
        assert!(r.compliant);
    }

    #[test]
    fn sb326_certified_inspector_disqualified() {
        let mut i = sb326_compliant();
        i.inspector_qualification = InspectorQualification::CertifiedBuildingInspector;
        let r = check(&i);
        assert!(!r.qualified_inspector);
        assert!(!r.compliant);
    }

    #[test]
    fn sb326_missed_deadline_violates() {
        let mut i = sb326_compliant();
        i.inspection_completed_by_deadline = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("January 1, 2025")));
    }

    #[test]
    fn sb326_citation_pins_civ_code() {
        let r = check(&sb326_compliant());
        assert!(r.citation.contains("§ 5551"));
        assert!(r.citation.contains("SB 326"));
    }

    #[test]
    fn default_no_inspection_required() {
        let r = check(&default_base());
        assert!(!r.inspection_required);
        assert!(r.compliant);
        assert_eq!(r.recurring_cycle_years, 0);
    }

    #[test]
    fn default_citation_pins_premises_liability() {
        let r = check(&default_base());
        assert!(r.citation.contains("premises-liability"));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [
            Regime::CaliforniaSb721,
            Regime::CaliforniaSb326,
            Regime::Default,
        ] {
            let mut i = sb721_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn sb721_uniquely_6_year_sb326_uniquely_9_year_invariant() {
        let r_sb721 = check(&sb721_compliant());
        let r_sb326 = check(&sb326_compliant());
        assert_eq!(r_sb721.recurring_cycle_years, 6);
        assert_eq!(r_sb326.recurring_cycle_years, 9);
    }

    #[test]
    fn sb326_uniquely_excludes_contractors_invariant() {
        let mut i_sb721 = sb721_compliant();
        i_sb721.inspector_qualification = InspectorQualification::ContractorAbc5WithExperience;
        let r_sb721 = check(&i_sb721);
        assert!(r_sb721.qualified_inspector);

        let mut i_sb326 = sb326_compliant();
        i_sb326.inspector_qualification = InspectorQualification::ContractorAbc5WithExperience;
        let r_sb326 = check(&i_sb326);
        assert!(!r_sb326.qualified_inspector);
    }

    #[test]
    fn inspector_qualification_truth_table_sb721() {
        for (q, exp_qualified) in [
            (InspectorQualification::LicensedArchitect, true),
            (
                InspectorQualification::LicensedCivilOrStructuralEngineer,
                true,
            ),
            (InspectorQualification::ContractorAbc5WithExperience, true),
            (InspectorQualification::CertifiedBuildingInspector, true),
            (InspectorQualification::LocalGovernmentEmployee, false),
            (InspectorQualification::EntityPerformingRepairs, false),
            (InspectorQualification::Unqualified, false),
        ] {
            let mut i = sb721_compliant();
            i.inspector_qualification = q;
            let r = check(&i);
            assert_eq!(r.qualified_inspector, exp_qualified);
        }
    }

    #[test]
    fn inspector_qualification_truth_table_sb326() {
        for (q, exp_qualified) in [
            (InspectorQualification::LicensedArchitect, true),
            (
                InspectorQualification::LicensedCivilOrStructuralEngineer,
                true,
            ),
            (InspectorQualification::ContractorAbc5WithExperience, false),
            (InspectorQualification::CertifiedBuildingInspector, false),
            (InspectorQualification::LocalGovernmentEmployee, false),
            (InspectorQualification::EntityPerformingRepairs, false),
            (InspectorQualification::Unqualified, false),
        ] {
            let mut i = sb326_compliant();
            i.inspector_qualification = q;
            let r = check(&i);
            assert_eq!(r.qualified_inspector, exp_qualified);
        }
    }

    #[test]
    fn sb721_note_pins_january_2026_deadline_and_ab_2579() {
        let r = check(&sb721_compliant());
        assert!(r.notes.iter().any(|n| n.contains("January 1, 2026")
            && n.contains("AB 2579")
            && n.contains("January 1, 2025")));
    }

    #[test]
    fn sb721_note_pins_15_percent_and_120_day_repair() {
        let r = check(&sb721_compliant());
        assert!(r.notes.iter().any(|n| n.contains("15%")
            && n.contains("exploratory openings")
            && n.contains("120-day")));
    }

    #[test]
    fn sb326_note_pins_january_2025_deadline_and_9_year_cycle() {
        let r = check(&sb326_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("January 1, 2025") && n.contains("9 years")));
    }

    #[test]
    fn sb721_stacks_4_violations_when_all_4_failures() {
        let mut i = sb721_compliant();
        i.inspection_completed_by_deadline = false;
        i.inspector_qualification = InspectorQualification::Unqualified;
        i.minimum_15_percent_sampled = false;
        i.days_since_repair_finding = 200;
        i.repairs_completed = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 4);
    }

    #[test]
    fn sb721_repairs_completed_short_circuits_120_day_violation() {
        let mut i = sb721_compliant();
        i.days_since_repair_finding = 365;
        i.repairs_completed = true;
        let r = check(&i);
        assert!(r.compliant);
    }
}
