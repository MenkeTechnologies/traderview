//! State landlord asbestos disclosure compliance check for buildings
//! containing or presumed to contain asbestos-containing material
//! (ACM) or presumed asbestos-containing material (PACM).
//!
//! Rounds out the hazard-disclosure cluster alongside `lead_disclosure`,
//! `mold_disclosure`, `radon_disclosure`, `flood_disclosure`,
//! `meth_contamination_disclosure`, `fire_sprinkler_disclosure`,
//! `bedbug_disclosure`, and `death_in_unit_disclosure`.
//!
//! Federal floor under 29 CFR 1926.1101(k)(2) (OSHA construction
//! standard) requires building owners to notify tenants of PACM /
//! ACM presence when construction work is planned in occupied
//! areas, but the federal standard does NOT impose a lease-signing
//! tenant disclosure mandate. State law varies sharply.
//!
//! Six regimes:
//!
//!   - **California** — Cal. Health & Safety Code §§ 25915 to
//!     25919.7 (Connelly-Areias-Chacon Asbestos Notification Act,
//!     1989). Building owners with KNOWLEDGE of asbestos-containing
//!     construction materials in buildings constructed BEFORE 1979
//!     MUST notify current tenants + employees + contractors +
//!     prospective owners + lenders within 15 days of obtaining
//!     knowledge. **Annual** re-notification required to current
//!     tenants. Notice required regardless of quantity or condition.
//!     Penalty $500/day per § 25917.
//!
//!   - **NewJersey** — N.J.A.C. 5:23-8 implements the construction
//!     regulations alongside OSHA federal floor; no separate state
//!     lease-signing landlord disclosure mandate. Tenants rely on
//!     habitability obligation + federal OSHA + N.J.A.C. 12:120 for
//!     abatement standards.
//!
//!   - **NewYork** — No specific state landlord-tenant asbestos
//!     disclosure mandate at lease signing. Protection comes via
//!     NY MDL (Multiple Dwelling Law) habitability requirements +
//!     OSHA construction-phase + N.Y. Industrial Code Rule 56 for
//!     abatement. NYC has stricter rules under DEP Title 15.
//!
//!   - **FederalOSHA** — 29 CFR 1926.1101(k)(2) construction
//!     standard. Building owners with PACM (presumed ACM —
//!     thermal-system insulation, sprayed-on surfacing, etc. in
//!     buildings built before 1981) must notify tenants /
//!     employers / contractors when construction work is planned.
//!     Coverage limited to commercial / multi-tenant buildings;
//!     does NOT apply to single-family homes.
//!
//!   - **FederalAHERA** — Asbestos Hazard Emergency Response Act,
//!     1986; 40 CFR Part 763. Public schools + private nonprofit
//!     schools only. Triennial inspection + management plan. Does
//!     NOT cover landlord-tenant rental dwellings.
//!
//!   - **Default** — Federal OSHA construction-phase floor +
//!     habitability obligation under state law. No specific lease-
//!     signing landlord disclosure mandate.
//!
//! Citations: Cal. Health & Safety Code §§ 25915, 25916, 25917
//! (Connelly-Areias-Chacon Asbestos Notification Act); 29 CFR
//! § 1926.1101(k)(2) (OSHA construction standard PACM/ACM tenant
//! notification); 40 CFR Part 763 (AHERA for schools); 15 U.S.C.
//! §§ 2641–2671 (Toxic Substances Control Act Title II — AHERA);
//! N.J.A.C. 5:23-8 (NJ construction); N.Y. Multiple Dwelling Law
//! habitability + N.Y. Industrial Code Rule 56 (NY abatement).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    NewYork,
    FederalOSHA,
    FederalAHERA,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    /// Single-family residence — outside OSHA 1926.1101(k)(2) scope.
    SingleFamilyResidence,
    /// Multi-tenant residential building — covered by CA + OSHA
    /// (construction phase).
    MultiTenantResidential,
    /// Commercial / mixed-use building — covered by OSHA full scope.
    Commercial,
    /// Public or nonprofit school — covered by AHERA.
    School,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub building_type: BuildingType,
    /// Year the building was constructed. CA § 25915 covers
    /// buildings constructed BEFORE 1979; OSHA PACM presumption
    /// applies to buildings built before 1981.
    pub construction_year: u32,
    /// Whether the landlord has actual knowledge of
    /// asbestos-containing material (ACM) in the building.
    pub landlord_has_knowledge_of_acm: bool,
    /// Days since the landlord obtained knowledge of ACM. CA
    /// § 25916 requires notification within 15 days.
    pub days_since_knowledge_obtained: u32,
    /// Whether notice was provided to current tenants at the time
    /// of knowledge.
    pub initial_notice_provided: bool,
    /// Whether annual re-notification has been provided to current
    /// tenants (CA § 25916(b) annual requirement).
    pub annual_renotification_provided: bool,
    /// Whether construction work is planned in occupied areas
    /// (triggers OSHA 1926.1101(k)(2) notification duty).
    pub construction_planned_in_occupied_areas: bool,
    /// Whether the OSHA construction-phase notification was provided
    /// to tenants when construction was planned.
    pub osha_construction_notice_provided: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// Whether the regime imposes a lease-signing tenant disclosure
    /// mandate (only CA among the six regimes).
    pub lease_signing_disclosure_mandate: bool,
    /// Statutory deadline in days from knowledge to notice. None
    /// where no specific deadline applies.
    pub notice_deadline_days: Option<u32>,
    /// Whether the regime requires annual re-notification (CA
    /// § 25916(b)).
    pub annual_renotification_required: bool,
    /// Whether the federal OSHA construction-phase notification
    /// duty engages.
    pub osha_construction_notice_required: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// California § 25915 / § 25916 — building must have been
/// constructed before 1979 for the disclosure mandate to engage.
pub const CA_PRE_1979_THRESHOLD_YEAR: u32 = 1979;
/// OSHA PACM presumption — buildings constructed before 1981 are
/// presumed to contain ACM.
pub const OSHA_PACM_PRESUMPTION_YEAR: u32 = 1981;
/// California § 25916(a) — notification deadline.
pub const CA_NOTICE_DEADLINE_DAYS: u32 = 15;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (
        lease_signing_disclosure_mandate,
        notice_deadline_days,
        annual_renotification_required,
        citation,
    ): (bool, Option<u32>, bool, &'static str) = match input.regime {
        Regime::California => (
            true,
            Some(CA_NOTICE_DEADLINE_DAYS),
            true,
            "Cal. Health & Safety Code §§ 25915 (definitions + applicability — buildings \
             constructed before 1979); § 25916(a) (15-day notification deadline from knowledge); \
             § 25916(b) (annual re-notification to current tenants); § 25917 ($500/day penalty); \
             § 25919.7 (Connelly-Areias-Chacon Asbestos Notification Act)",
        ),
        Regime::NewJersey => (
            false,
            None,
            false,
            "N.J.A.C. 5:23-8 (NJ construction regulations — no separate lease-signing landlord \
             disclosure mandate); N.J.A.C. 12:120 (abatement standards); federal OSHA + \
             habitability obligation",
        ),
        Regime::NewYork => (
            false,
            None,
            false,
            "N.Y. Multiple Dwelling Law habitability + N.Y. Industrial Code Rule 56 \
             (abatement standards); NYC DEP Title 15 (NYC-specific stricter rules); no \
             state-level lease-signing landlord disclosure mandate",
        ),
        Regime::FederalOSHA => (
            false,
            None,
            false,
            "29 CFR § 1926.1101(k)(2) (OSHA construction standard — building owner must notify \
             tenants/employers/contractors of PACM/ACM presence when construction work is \
             planned in occupied areas); coverage limited to commercial / multi-tenant; \
             does NOT apply to single-family homes",
        ),
        Regime::FederalAHERA => (
            false,
            None,
            false,
            "40 CFR Part 763 (Asbestos Hazard Emergency Response Act — AHERA); 15 U.S.C. \
             §§ 2641–2671 (Toxic Substances Control Act Title II); covers public + private \
             nonprofit schools only; triennial inspection + management plan; does NOT cover \
             landlord-tenant rental dwellings",
        ),
        Regime::Default => (
            false,
            None,
            false,
            "Federal OSHA construction-phase floor (29 CFR § 1926.1101(k)(2)) + state \
             habitability obligation; no specific lease-signing landlord disclosure mandate",
        ),
    };

    // California § 25915 building-year threshold — only buildings
    // built before 1979 trigger the disclosure mandate.
    if matches!(input.regime, Regime::California)
        && input.construction_year >= CA_PRE_1979_THRESHOLD_YEAR
    {
        notes.push(format!(
            "Cal. Health & Safety Code § 25915 — building was constructed in {}, on or after \
             {} threshold; the asbestos notification act does NOT apply.",
            input.construction_year, CA_PRE_1979_THRESHOLD_YEAR,
        ));
        return CheckResult {
            lease_signing_disclosure_mandate: false,
            notice_deadline_days: None,
            annual_renotification_required: false,
            osha_construction_notice_required: false,
            compliant: true,
            violations,
            citation: "Cal. Health & Safety Code § 25915 (building-year threshold not met)",
            notes,
        };
    }

    // California compliance — knowledge-triggered 15-day deadline +
    // annual re-notification.
    if matches!(input.regime, Regime::California) && input.landlord_has_knowledge_of_acm {
        if !input.initial_notice_provided
            && input.days_since_knowledge_obtained > CA_NOTICE_DEADLINE_DAYS
        {
            violations.push(format!(
                "Cal. § 25916(a) — notification not provided within 15 days of obtaining \
                 knowledge of asbestos-containing materials; {} days have elapsed since \
                 knowledge was obtained.",
                input.days_since_knowledge_obtained,
            ));
        }
        if !input.annual_renotification_provided {
            violations.push(
                "Cal. § 25916(b) — annual re-notification to current tenants is required and \
                 has not been provided."
                    .to_string(),
            );
        }
    }

    // OSHA construction-phase notification.
    let osha_in_scope = matches!(
        input.building_type,
        BuildingType::MultiTenantResidential | BuildingType::Commercial
    );
    let osha_construction_notice_required = osha_in_scope
        && input.construction_planned_in_occupied_areas
        && input.construction_year < OSHA_PACM_PRESUMPTION_YEAR;
    if osha_construction_notice_required && !input.osha_construction_notice_provided {
        violations.push(
            "29 CFR § 1926.1101(k)(2) — OSHA construction-phase tenant notification of PACM/ACM \
             presence required because construction work is planned in occupied areas of a \
             multi-tenant or commercial building constructed before 1981; notification has not \
             been provided."
                .to_string(),
        );
    }

    // School + AHERA path.
    if matches!(input.regime, Regime::FederalAHERA)
        && !matches!(input.building_type, BuildingType::School)
    {
        notes.push(
            "AHERA coverage is limited to public + private nonprofit schools; the building \
             type does not fall within AHERA scope. Tenants must look to OSHA + state \
             habitability protections."
                .to_string(),
        );
    }

    // Single-family residence OSHA exclusion.
    if osha_in_scope
        || !matches!(input.building_type, BuildingType::SingleFamilyResidence)
    {
        // no-op — handled above
    } else if input.construction_planned_in_occupied_areas {
        notes.push(
            "29 CFR § 1926.1101(k)(2) — single-family residence is OUTSIDE OSHA scope; the \
             construction-phase tenant-notification duty does not engage."
                .to_string(),
        );
    }

    notes.push(
        "Companion hazard-disclosure modules: lead_disclosure (federal 42 U.S.C. § 4852d + \
         state additions); mold_disclosure; radon_disclosure; flood_disclosure; \
         meth_contamination_disclosure; bedbug_disclosure; death_in_unit_disclosure; \
         fire_sprinkler_disclosure."
            .to_string(),
    );

    CheckResult {
        lease_signing_disclosure_mandate,
        notice_deadline_days,
        annual_renotification_required,
        osha_construction_notice_required,
        compliant: violations.is_empty(),
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime) -> Input {
        Input {
            regime,
            building_type: BuildingType::MultiTenantResidential,
            construction_year: 1970,
            landlord_has_knowledge_of_acm: true,
            days_since_knowledge_obtained: 5,
            initial_notice_provided: true,
            annual_renotification_provided: true,
            construction_planned_in_occupied_areas: false,
            osha_construction_notice_provided: false,
        }
    }

    // ── California Health & Safety Code §§ 25915–25919.7 ────────

    #[test]
    fn california_pre_1979_with_knowledge_compliant_when_notice_provided() {
        let r = check(&base(Regime::California));
        assert!(r.compliant);
        assert!(r.lease_signing_disclosure_mandate);
        assert_eq!(r.notice_deadline_days, Some(15));
        assert!(r.annual_renotification_required);
        assert!(r.citation.contains("§ 25915"));
        assert!(r.citation.contains("§ 25916(a)"));
        assert!(r.citation.contains("Connelly-Areias-Chacon"));
    }

    #[test]
    fn california_1980_building_outside_pre_1979_threshold() {
        let mut i = base(Regime::California);
        i.construction_year = 1980;
        let r = check(&i);
        // Statute doesn't engage; treated as compliant.
        assert!(r.compliant);
        assert!(!r.lease_signing_disclosure_mandate);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 25915") && n.contains("threshold"))
        );
    }

    #[test]
    fn california_pre_1979_no_initial_notice_past_15_days_violation() {
        let mut i = base(Regime::California);
        i.initial_notice_provided = false;
        i.days_since_knowledge_obtained = 16;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 25916(a)") && v.contains("15 days"))
        );
    }

    #[test]
    fn california_pre_1979_no_initial_notice_within_15_days_still_compliant() {
        // Day 14 of grace period — still within deadline.
        let mut i = base(Regime::California);
        i.initial_notice_provided = false;
        i.days_since_knowledge_obtained = 14;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn california_at_15_day_boundary_still_compliant() {
        // Day 15 = at threshold; statute reads "within 15 days" so
        // day 15 itself is still compliant.
        let mut i = base(Regime::California);
        i.initial_notice_provided = false;
        i.days_since_knowledge_obtained = 15;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn california_missing_annual_renotification_violation() {
        let mut i = base(Regime::California);
        i.annual_renotification_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 25916(b)") && v.contains("annual"))
        );
    }

    #[test]
    fn california_no_knowledge_no_violation() {
        let mut i = base(Regime::California);
        i.landlord_has_knowledge_of_acm = false;
        i.initial_notice_provided = false;
        i.annual_renotification_provided = true;
        let r = check(&i);
        // No knowledge → no 15-day duty triggered. But annual
        // re-notification requirement still standing per § 25916(b);
        // since we provided it, compliant.
        assert!(r.compliant);
    }

    // ── New Jersey ──────────────────────────────────────────────

    #[test]
    fn new_jersey_no_lease_signing_disclosure_mandate() {
        let r = check(&base(Regime::NewJersey));
        assert!(!r.lease_signing_disclosure_mandate);
        assert_eq!(r.notice_deadline_days, None);
        assert!(!r.annual_renotification_required);
        assert!(r.citation.contains("N.J.A.C. 5:23-8"));
    }

    // ── New York ────────────────────────────────────────────────

    #[test]
    fn new_york_no_lease_signing_disclosure_mandate() {
        let r = check(&base(Regime::NewYork));
        assert!(!r.lease_signing_disclosure_mandate);
        assert!(r.citation.contains("Multiple Dwelling Law"));
        assert!(r.citation.contains("Industrial Code Rule 56"));
    }

    // ── Federal OSHA 1926.1101(k)(2) ────────────────────────────

    #[test]
    fn federal_osha_construction_phase_notice_required_for_multi_tenant_pre_1981() {
        let mut i = base(Regime::FederalOSHA);
        i.construction_planned_in_occupied_areas = true;
        i.osha_construction_notice_provided = false;
        let r = check(&i);
        assert!(r.osha_construction_notice_required);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1926.1101(k)(2)") && v.contains("construction-phase"))
        );
    }

    #[test]
    fn federal_osha_construction_phase_compliant_when_notice_provided() {
        let mut i = base(Regime::FederalOSHA);
        i.construction_planned_in_occupied_areas = true;
        i.osha_construction_notice_provided = true;
        let r = check(&i);
        assert!(r.osha_construction_notice_required);
        assert!(r.compliant);
    }

    #[test]
    fn federal_osha_single_family_outside_scope() {
        let mut i = base(Regime::FederalOSHA);
        i.building_type = BuildingType::SingleFamilyResidence;
        i.construction_planned_in_occupied_areas = true;
        i.osha_construction_notice_provided = false;
        let r = check(&i);
        // Single-family is outside OSHA construction scope.
        assert!(!r.osha_construction_notice_required);
        assert!(r.compliant);
    }

    #[test]
    fn federal_osha_post_1981_building_not_pacm_presumed() {
        let mut i = base(Regime::FederalOSHA);
        i.construction_year = 1985;
        i.construction_planned_in_occupied_areas = true;
        i.osha_construction_notice_provided = false;
        let r = check(&i);
        // Post-1981 → no PACM presumption → no automatic notice
        // duty.
        assert!(!r.osha_construction_notice_required);
    }

    #[test]
    fn federal_osha_no_construction_no_notice_required() {
        let r = check(&base(Regime::FederalOSHA));
        assert!(!r.osha_construction_notice_required);
        assert!(r.compliant);
    }

    // ── Federal AHERA ──────────────────────────────────────────

    #[test]
    fn federal_ahera_school_in_scope() {
        let mut i = base(Regime::FederalAHERA);
        i.building_type = BuildingType::School;
        let r = check(&i);
        assert!(r.citation.contains("40 CFR Part 763"));
        assert!(r.citation.contains("AHERA"));
        assert!(!r.lease_signing_disclosure_mandate); // Not a landlord-tenant mandate
    }

    #[test]
    fn federal_ahera_non_school_outside_scope_note() {
        let mut i = base(Regime::FederalAHERA);
        i.building_type = BuildingType::MultiTenantResidential;
        let r = check(&i);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("AHERA") && n.contains("schools"))
        );
    }

    // ── Default — federal floor only ───────────────────────────

    #[test]
    fn default_no_state_specific_mandate() {
        let r = check(&base(Regime::Default));
        assert!(!r.lease_signing_disclosure_mandate);
        assert!(r.citation.contains("Federal OSHA"));
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn only_california_imposes_lease_signing_disclosure_invariant() {
        let ca = check(&base(Regime::California));
        assert!(ca.lease_signing_disclosure_mandate);
        for &regime in &[
            Regime::NewJersey,
            Regime::NewYork,
            Regime::FederalOSHA,
            Regime::FederalAHERA,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.lease_signing_disclosure_mandate,
                "{:?}: must NOT impose lease-signing disclosure mandate",
                regime,
            );
        }
    }

    #[test]
    fn only_california_requires_annual_renotification_invariant() {
        let ca = check(&base(Regime::California));
        assert!(ca.annual_renotification_required);
        for &regime in &[
            Regime::NewJersey,
            Regime::NewYork,
            Regime::FederalOSHA,
            Regime::FederalAHERA,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.annual_renotification_required,
                "{:?}: must NOT require annual re-notification",
                regime,
            );
        }
    }

    #[test]
    fn osha_construction_notice_only_for_multi_tenant_or_commercial_pre_1981_invariant() {
        for bt in [
            BuildingType::SingleFamilyResidence,
            BuildingType::School,
        ] {
            let mut i = base(Regime::FederalOSHA);
            i.building_type = bt;
            i.construction_planned_in_occupied_areas = true;
            assert!(
                !check(&i).osha_construction_notice_required,
                "{:?}: must NOT trigger OSHA construction notice",
                bt,
            );
        }
        for bt in [
            BuildingType::MultiTenantResidential,
            BuildingType::Commercial,
        ] {
            let mut i = base(Regime::FederalOSHA);
            i.building_type = bt;
            i.construction_planned_in_occupied_areas = true;
            assert!(
                check(&i).osha_construction_notice_required,
                "{:?}: must trigger OSHA construction notice (pre-1981 with construction)",
                bt,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California)).citation.contains("§ 25915"));
        assert!(check(&base(Regime::NewJersey)).citation.contains("N.J.A.C."));
        assert!(check(&base(Regime::NewYork)).citation.contains("Multiple Dwelling Law"));
        assert!(
            check(&base(Regime::FederalOSHA))
                .citation
                .contains("§ 1926.1101(k)(2)")
        );
        assert!(check(&base(Regime::FederalAHERA)).citation.contains("AHERA"));
        assert!(check(&base(Regime::Default)).citation.contains("Federal OSHA"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::California,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::FederalOSHA,
            Regime::FederalAHERA,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes.iter().any(|n| n.contains("lead_disclosure")
                    && n.contains("mold_disclosure")
                    && n.contains("radon_disclosure")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }

    #[test]
    fn ca_15_day_boundary_strict_in_violation_check() {
        // Boundary day-15 = compliant; day-16 = violation.
        for (days, expect_violation) in [(15, false), (16, true)] {
            let mut i = base(Regime::California);
            i.initial_notice_provided = false;
            i.days_since_knowledge_obtained = days;
            let has_violation = check(&i)
                .violations
                .iter()
                .any(|v| v.contains("15 days"));
            assert_eq!(
                has_violation, expect_violation,
                "day {}: expected violation={}",
                days, expect_violation,
            );
        }
    }
}
