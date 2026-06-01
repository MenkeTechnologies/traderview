//! State landlord former-federal-or-state-ordnance-location
//! disclosure compliance check.
//!
//! Specialized hazard-disclosure topic prompted by the December
//! 10, 1983, Tierra Santa tragedy in San Diego — a live munition
//! exploded in a residential area that was formerly a U.S. military
//! ordnance location, killing two residents. California responded
//! with Cal. Civ. Code § 1940.7, which became the model for the
//! actual-knowledge-based disclosure approach.
//!
//! Rounds out the hazard-disclosure cluster alongside
//! `asbestos_disclosure`, `lead_disclosure`, `mold_disclosure`,
//! `radon_disclosure`, `flood_disclosure`,
//! `meth_contamination_disclosure`, `bedbug_disclosure`,
//! `fire_sprinkler_disclosure`, and `death_in_unit_disclosure`.
//!
//! Three regimes:
//!
//!   - **California** — Cal. Civ. Code § 1940.7. Landlord with
//!     ACTUAL KNOWLEDGE of any former federal or state ordnance
//!     location WITHIN ONE MILE of the residential dwelling shall
//!     give WRITTEN NOTICE to the prospective tenant PRIOR TO the
//!     execution of the rental agreement. For tenancies in
//!     existence on January 1, 1990, written notice shall be given
//!     to tenants AS SOON AS PRACTICABLE thereafter.
//!
//!     Definitions:
//!       - "Former federal or state ordnance location" — area
//!         identified by an agency or instrumentality of the
//!         federal or state government as an area once used for
//!         military training purposes and which may contain
//!         potentially explosive munitions.
//!       - "Neighborhood area" — within one mile of the residential
//!         dwelling.
//!
//!   - **FederalMMRP** — federal Military Munitions Response
//!     Program under the Defense Environmental Restoration Program
//!     (10 U.S.C. § 2710 + § 2701) — DoD / U.S. Army Corps of
//!     Engineers identifies Formerly Used Defense Sites (FUDS) and
//!     maintains a public inventory. No general federal landlord
//!     disclosure mandate; the database is public information that
//!     state actual-knowledge disclosures may reference.
//!
//!   - **Default** — no statutory landlord disclosure mandate.
//!     Federal FUDS database remains public information; common-law
//!     latent-defect disclosure theory may apply where landlord has
//!     actual knowledge.
//!
//! Citations: Cal. Civ. Code § 1940.7(a) (legislative findings —
//! 1983 Tierra Santa tragedy); § 1940.7(b) (actual-knowledge
//! prospective-tenant written-notice requirement); § 1940.7(c)
//! (tenancies in existence on 1990-01-01 as-soon-as-practicable
//! notice); § 1940.7(d) (definitions — "former federal or state
//! ordnance location" and "neighborhood area" within one mile);
//! 10 U.S.C. § 2710 (federal Military Munitions Response Program);
//! 10 U.S.C. § 2701 (Defense Environmental Restoration Program).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    FederalMMRP,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Whether the landlord has ACTUAL KNOWLEDGE of any former
    /// federal or state ordnance location. § 1940.7 trigger.
    pub landlord_has_actual_knowledge: bool,
    /// Whether the former-ordnance location is within ONE MILE of
    /// the residential dwelling (Cal. Civ. Code § 1940.7(d)(2)
    /// "neighborhood area" threshold).
    pub former_ordnance_location_within_1_mile: bool,
    /// Whether the landlord provided written notice to the
    /// PROSPECTIVE tenant PRIOR TO execution of the rental
    /// agreement.
    pub written_notice_provided_before_lease: bool,
    /// Whether the tenancy was in existence on January 1, 1990
    /// (triggers the § 1940.7(c) as-soon-as-practicable notice
    /// requirement for grandfathered tenants).
    pub tenancy_existing_on_1990_01_01: bool,
    /// Whether the landlord provided notice as soon as practicable
    /// to grandfathered tenants (§ 1940.7(c)).
    pub notice_provided_as_soon_as_practicable: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime imposes a statutory disclosure mandate.
    pub disclosure_mandated: bool,
    /// True if the actual-knowledge trigger has fired (landlord
    /// has knowledge AND location within 1 mile).
    pub disclosure_trigger_fired: bool,
    /// True if the regime requires PRE-LEASE written notice to
    /// prospective tenants (§ 1940.7(b)).
    pub pre_lease_notice_required: bool,
    /// True if the regime requires as-soon-as-practicable notice
    /// for grandfathered tenants (§ 1940.7(c)).
    pub grandfathered_tenant_notice_required: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// California Civ. Code § 1940.7(d)(2) — "neighborhood area"
/// threshold of one mile.
pub const CA_NEIGHBORHOOD_AREA_RADIUS_MILES: u32 = 1;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (
        disclosure_mandated,
        pre_lease_notice_required,
        grandfathered_tenant_notice_required,
        citation,
    ): (bool, bool, bool, &'static str) = match input.regime {
        Regime::California => (
            true,
            true,
            true,
            "Cal. Civ. Code § 1940.7(a) (legislative findings — December 10, 1983 Tierra Santa \
             tragedy); § 1940.7(b) (actual-knowledge prospective-tenant written-notice prior to \
             lease execution); § 1940.7(c) (tenancies in existence on 1990-01-01 — as soon as \
             practicable); § 1940.7(d)(1) (former federal or state ordnance location \
             definition); § 1940.7(d)(2) (neighborhood area within one mile of dwelling)",
        ),
        Regime::FederalMMRP => (
            false,
            false,
            false,
            "10 U.S.C. § 2710 (federal Military Munitions Response Program — DoD / U.S. Army \
             Corps of Engineers FUDS public inventory); 10 U.S.C. § 2701 (Defense Environmental \
             Restoration Program); no general federal landlord disclosure mandate",
        ),
        Regime::Default => (
            false,
            false,
            false,
            "No statutory landlord disclosure mandate; federal FUDS database public \
             information; common-law latent-defect disclosure may apply where landlord has \
             actual knowledge",
        ),
    };

    // California compliance — actual-knowledge trigger.
    let disclosure_trigger_fired = matches!(input.regime, Regime::California)
        && input.landlord_has_actual_knowledge
        && input.former_ordnance_location_within_1_mile;

    if disclosure_trigger_fired {
        // Pre-lease notice for new tenancies.
        if !input.tenancy_existing_on_1990_01_01 && !input.written_notice_provided_before_lease {
            violations.push(
                "Cal. Civ. Code § 1940.7(b) — landlord with actual knowledge of former federal \
                 or state ordnance location within one mile of residential dwelling must \
                 provide written notice to prospective tenant PRIOR TO execution of rental \
                 agreement; notice not provided."
                    .to_string(),
            );
        }
        // As-soon-as-practicable notice for grandfathered tenancies.
        if input.tenancy_existing_on_1990_01_01
            && !input.notice_provided_as_soon_as_practicable
        {
            violations.push(
                "Cal. Civ. Code § 1940.7(c) — for tenancies in existence on January 1, 1990, \
                 written notice must be given to tenants as soon as practicable thereafter; \
                 notice not provided to grandfathered tenant."
                    .to_string(),
            );
        }
    }

    // Definitional notes.
    if matches!(input.regime, Regime::California) {
        notes.push(
            "§ 1940.7(d)(1) — 'former federal or state ordnance location' means an area \
             identified by an agency or instrumentality of the federal or state government as \
             an area once used for military training purposes and which may contain \
             potentially explosive munitions."
                .to_string(),
        );
        notes.push(
            "§ 1940.7(d)(2) — 'neighborhood area' means within ONE MILE of the residential \
             dwelling. Locations beyond one mile are outside the disclosure trigger."
                .to_string(),
        );
        if !input.landlord_has_actual_knowledge {
            notes.push(
                "§ 1940.7(b) — disclosure requirement triggered ONLY by landlord's ACTUAL \
                 KNOWLEDGE of the former ordnance location. Constructive knowledge or duty to \
                 investigate is not imposed."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Companion hazard-disclosure modules: asbestos_disclosure (Cal. § 25915 Connelly-\
         Areias-Chacon); lead_disclosure (federal Title X); mold_disclosure; radon_disclosure; \
         flood_disclosure; meth_contamination_disclosure; bedbug_disclosure; \
         fire_sprinkler_disclosure; death_in_unit_disclosure."
            .to_string(),
    );

    CheckResult {
        disclosure_mandated,
        disclosure_trigger_fired,
        pre_lease_notice_required,
        grandfathered_tenant_notice_required,
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
            landlord_has_actual_knowledge: true,
            former_ordnance_location_within_1_mile: true,
            written_notice_provided_before_lease: true,
            tenancy_existing_on_1990_01_01: false,
            notice_provided_as_soon_as_practicable: false,
        }
    }

    // ── California § 1940.7 — actual-knowledge trigger ─────────

    #[test]
    fn california_with_actual_knowledge_within_1_mile_compliant_when_notified() {
        let r = check(&base(Regime::California));
        assert!(r.compliant);
        assert!(r.disclosure_mandated);
        assert!(r.disclosure_trigger_fired);
        assert!(r.pre_lease_notice_required);
        assert!(r.citation.contains("§ 1940.7(a)"));
        assert!(r.citation.contains("Tierra Santa"));
    }

    #[test]
    fn california_with_actual_knowledge_within_1_mile_no_notice_violation() {
        let mut i = base(Regime::California);
        i.written_notice_provided_before_lease = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1940.7(b)") && v.contains("PRIOR TO"))
        );
    }

    #[test]
    fn california_no_actual_knowledge_trigger_not_fired() {
        let mut i = base(Regime::California);
        i.landlord_has_actual_knowledge = false;
        i.written_notice_provided_before_lease = false;
        let r = check(&i);
        assert!(!r.disclosure_trigger_fired);
        // No actual knowledge → no statutory duty → compliant.
        assert!(r.compliant);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("ACTUAL KNOWLEDGE") && n.contains("Constructive"))
        );
    }

    #[test]
    fn california_actual_knowledge_but_location_beyond_1_mile_no_trigger() {
        let mut i = base(Regime::California);
        i.former_ordnance_location_within_1_mile = false;
        i.written_notice_provided_before_lease = false;
        let r = check(&i);
        assert!(!r.disclosure_trigger_fired);
        // Beyond 1 mile = outside neighborhood area = no duty.
        assert!(r.compliant);
    }

    #[test]
    fn california_grandfathered_tenant_practicable_notice_compliant() {
        let mut i = base(Regime::California);
        i.tenancy_existing_on_1990_01_01 = true;
        i.notice_provided_as_soon_as_practicable = true;
        i.written_notice_provided_before_lease = false; // not applicable
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.grandfathered_tenant_notice_required);
    }

    #[test]
    fn california_grandfathered_tenant_no_practicable_notice_violation() {
        let mut i = base(Regime::California);
        i.tenancy_existing_on_1990_01_01 = true;
        i.notice_provided_as_soon_as_practicable = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1940.7(c)") && v.contains("January 1, 1990"))
        );
    }

    // ── Federal MMRP / Default ─────────────────────────────────

    #[test]
    fn federal_mmrp_no_landlord_disclosure_mandate() {
        let r = check(&base(Regime::FederalMMRP));
        assert!(!r.disclosure_mandated);
        assert!(!r.disclosure_trigger_fired);
        assert!(r.citation.contains("10 U.S.C. § 2710"));
        assert!(r.citation.contains("FUDS"));
    }

    #[test]
    fn default_no_statutory_mandate() {
        let r = check(&base(Regime::Default));
        assert!(!r.disclosure_mandated);
        assert!(r.citation.contains("common-law"));
    }

    #[test]
    fn default_even_with_actual_knowledge_no_violation() {
        // Default regime has no statutory mandate; no violation
        // regardless of facts.
        let mut i = base(Regime::Default);
        i.landlord_has_actual_knowledge = true;
        i.former_ordnance_location_within_1_mile = true;
        i.written_notice_provided_before_lease = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    // ── Definitional notes ─────────────────────────────────────

    #[test]
    fn california_definitional_notes_present() {
        let r = check(&base(Regime::California));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1940.7(d)(1)") && n.contains("former federal or state"))
        );
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1940.7(d)(2)") && n.contains("ONE MILE"))
        );
    }

    #[test]
    fn california_constructive_knowledge_not_triggered_note() {
        let mut i = base(Regime::California);
        i.landlord_has_actual_knowledge = false;
        let r = check(&i);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Constructive knowledge"))
        );
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn only_california_mandates_disclosure_invariant() {
        let ca = check(&base(Regime::California));
        assert!(ca.disclosure_mandated);
        for &regime in &[Regime::FederalMMRP, Regime::Default] {
            let r = check(&base(regime));
            assert!(
                !r.disclosure_mandated,
                "{:?}: must NOT impose statutory disclosure mandate",
                regime,
            );
        }
    }

    #[test]
    fn disclosure_trigger_requires_actual_knowledge_AND_within_1_mile_invariant() {
        // 4-cell truth table: trigger fires only when BOTH knowledge
        // AND within-1-mile are true.
        for (knowledge, within_mile, expected_trigger) in [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, true, true),
        ] {
            let mut i = base(Regime::California);
            i.landlord_has_actual_knowledge = knowledge;
            i.former_ordnance_location_within_1_mile = within_mile;
            i.written_notice_provided_before_lease = true;
            let r = check(&i);
            assert_eq!(
                r.disclosure_trigger_fired, expected_trigger,
                "knowledge={} within_mile={} expected_trigger={}",
                knowledge, within_mile, expected_trigger,
            );
        }
    }

    #[test]
    fn non_california_regimes_never_trigger_invariant() {
        for &regime in &[Regime::FederalMMRP, Regime::Default] {
            let mut i = base(regime);
            i.landlord_has_actual_knowledge = true;
            i.former_ordnance_location_within_1_mile = true;
            assert!(
                !check(&i).disclosure_trigger_fired,
                "{:?}: must NOT trigger statutory disclosure",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California)).citation.contains("§ 1940.7"));
        assert!(check(&base(Regime::FederalMMRP)).citation.contains("§ 2710"));
        assert!(
            check(&base(Regime::Default))
                .citation
                .contains("common-law")
        );
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[Regime::California, Regime::FederalMMRP, Regime::Default] {
            let r = check(&base(regime));
            assert!(
                r.notes.iter().any(|n| n.contains("asbestos_disclosure")
                    && n.contains("lead_disclosure")
                    && n.contains("hazard-disclosure")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }

    #[test]
    fn neighborhood_radius_constant_1_mile_invariant() {
        assert_eq!(CA_NEIGHBORHOOD_AREA_RADIUS_MILES, 1);
    }

    #[test]
    fn legislative_findings_cite_tierra_santa_in_california_path() {
        let r = check(&base(Regime::California));
        assert!(r.citation.contains("Tierra Santa"));
        assert!(r.citation.contains("December 10, 1983"));
    }
}
