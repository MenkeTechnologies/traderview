//! Federal FHA + state reasonable accommodation / modification
//! compliance for tenants with disabilities.
//!
//! Federal Fair Housing Act § 3604(f)(3) imposes two distinct
//! disability-rights obligations on landlords:
//!
//!   (A) **Reasonable MODIFICATIONS** — at the tenant's expense, of
//!       existing premises occupied or to be occupied by such person
//!       if the modifications are necessary to afford full enjoyment.
//!       Landlord may require tenant to restore the interior to its
//!       pre-modification condition (with limits).
//!
//!   (B) **Reasonable ACCOMMODATIONS** — in rules, policies,
//!       practices, or services when necessary to afford equal
//!       opportunity to use and enjoy a dwelling. AT LANDLORD'S
//!       EXPENSE unless undue financial/administrative burden or
//!       fundamental alteration of the program.
//!
//! Distinction is load-bearing because each prong has its own cost
//! allocation, denial threshold, and remedy. Waiving a no-pets rule
//! for a service animal = ACCOMMODATION (landlord pays nothing).
//! Installing a wheelchair ramp at the unit's entrance = MODIFICATION
//! (tenant pays).
//!
//! **Three-regime overlay**:
//!
//!   - **California** — Cal. Civ. Code § 54.1(b)(3)(A) mirrors FHA
//!     modification right, AT TENANT'S EXPENSE. § 54.1(b)(3)(B)
//!     restoration agreement permitted but landlord MAY NOT impose
//!     ADDITIONAL SECURITY. Parties MAY agree to ESCROW capped at a
//!     "reasonable estimate of the cost of restoring the premises".
//!     Sensory-disability coverage broader than ADA (includes blind,
//!     visually-impaired, deaf, hearing-impaired) via § 54(b).
//!
//!   - **New York City** — N.Y.C. Admin. Code § 8-107(15)(c) requires
//!     the landlord to engage in a "cooperative dialogue" within a
//!     reasonable time of receiving an accommodation request. The
//!     dialogue must be in writing or orally documented and address
//!     the tenant's needs, potential accommodations, and any
//!     difficulties for the landlord. § 8-102 defines the dialogue.
//!     Failure to engage = DISCRIMINATORY PRACTICE per § 8-107(15).
//!     This is the UNIQUE NYC strict requirement — absent from FHA
//!     and from other state HRLs.
//!
//!   - **Washington** — RCW 49.60.222(2)(b) mirrors the FHA framework
//!     and adopts FHA cost allocation: modifications at tenant
//!     expense, accommodations at landlord expense (subject to undue
//!     burden / fundamental alteration).
//!
//! Citations: 42 U.S.C. § 3604(f)(3)(A) (federal modification);
//! § 3604(f)(3)(B) (federal accommodation); Cal. Civ. Code § 54.1(b)
//! (CA modification + restoration + no-additional-security + escrow
//! cap); Cal. Civ. Code § 54(b) (CA sensory disability); N.Y.C.
//! Admin. Code § 8-107(15)(c) (cooperative-dialogue mandate);
//! § 8-102 (dialogue definition); RCW 49.60.222(2)(b) (Washington).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Federal,
    California,
    NYC,
    Washington,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestType {
    /// Physical change to the unit, common area, or building.
    /// At tenant's expense across all four regimes. Landlord may
    /// require restoration agreement.
    Modification,
    /// Change to a rule, policy, practice, or service. At
    /// landlord's expense (no charge to tenant) unless undue
    /// financial/administrative burden or fundamental alteration.
    Accommodation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub request_type: RequestType,
    pub tenant_has_qualifying_disability: bool,
    /// Whether the requested change is necessary for the tenant's
    /// equal use and enjoyment of the dwelling (FHA nexus).
    pub nexus_to_disability: bool,
    /// Landlord engaged the tenant in a documented dialogue about
    /// the request, accommodations considered, and any difficulties.
    pub landlord_engaged_in_dialogue: bool,
    /// Days elapsed since the landlord received the request.
    pub days_since_request_received: u32,
    /// Modification cost (cents). Used only for modification path.
    pub modification_cost_cents: i64,
    /// Whether the landlord requires the tenant to restore the
    /// interior to its pre-modification condition. Permissible under
    /// federal + state law for modifications that would materially
    /// affect the next occupant.
    pub restoration_required: bool,
    /// Amount the landlord seeks to hold in escrow for the
    /// restoration obligation (cents). California caps this at the
    /// reasonable estimate of restoration cost.
    pub escrow_amount_cents: i64,
    /// Reasonable estimate of restoration cost (cents) — the
    /// California escrow cap benchmark.
    pub restoration_estimate_cents: i64,
    /// Accommodation creates an undue financial or administrative
    /// burden on the landlord. Defeats the § 3604(f)(3)(B) duty.
    pub creates_undue_financial_burden: bool,
    /// Accommodation requires a fundamental alteration of the
    /// landlord's program or service. Defeats § 3604(f)(3)(B).
    pub creates_fundamental_alteration: bool,
    /// Modification would materially impact the next occupant. If
    /// false, restoration agreement is NOT a reasonable condition.
    pub modification_would_materially_impact_next_occupant: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the landlord must grant the request under the
    /// applicable regime.
    pub request_grantable: bool,
    /// Who bears the cost.
    pub cost_borne_by: &'static str,
    /// Whether a restoration agreement is a permissible condition
    /// for this request (only modification path; only when next
    /// occupant materially impacted).
    pub restoration_permissible: bool,
    /// California-specific: whether the escrow amount sits at or
    /// below the reasonable-estimate cap. False outside California
    /// or when no escrow is sought.
    pub escrow_within_cap: bool,
    /// Whether the regime imposes a strict cooperative-dialogue
    /// duty (NYC only).
    pub cooperative_dialogue_required: bool,
    /// Some(true) if dialogue required AND landlord engaged within
    /// reasonable time; Some(false) if required AND not engaged;
    /// None if not required.
    pub cooperative_dialogue_completed_within_reasonable_time: Option<bool>,
    pub violations: Vec<String>,
    pub citation: String,
    pub notes: Vec<String>,
}

/// Reasonable-time benchmark for NYC § 8-107(15) cooperative dialogue.
/// NYC CCHR interpretive guidance treats prompt engagement as the
/// touchstone; 30 days is the conservative ceiling we use here as a
/// bright-line audit value. (Statutory text says "reasonable time"
/// without a fixed deadline; we surface this constant so callers can
/// tune it.)
pub const NYC_DIALOGUE_REASONABLE_TIME_DAYS: u32 = 30;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut grantable = true;
    let cost_borne_by: &'static str;
    let mut restoration_permissible = false;
    let mut escrow_within_cap = false;
    let mut citation_parts: Vec<&'static str> = Vec::new();

    // Universal threshold — must be a qualifying disability AND a
    // nexus between the disability and the requested change.
    if !input.tenant_has_qualifying_disability {
        grantable = false;
        violations.push(
            "Tenant does not have a qualifying disability under § 3604(f)(1)–(2); no FHA \
             reasonable-accommodation or reasonable-modification duty triggered."
                .to_string(),
        );
    }
    if !input.nexus_to_disability {
        grantable = false;
        violations.push(
            "No nexus between the disability and the requested change; § 3604(f)(3) \
             requires the modification or accommodation to be necessary for full enjoyment / \
             equal opportunity to use and enjoy the dwelling."
                .to_string(),
        );
    }

    match input.request_type {
        RequestType::Modification => {
            cost_borne_by = "tenant";
            citation_parts
                .push("42 U.S.C. § 3604(f)(3)(A) (reasonable modification at tenant's expense)");

            // Restoration is permissible only if the modification
            // would materially impact the next occupant.
            restoration_permissible = input.modification_would_materially_impact_next_occupant
                && input.restoration_required;

            if input.restoration_required
                && !input.modification_would_materially_impact_next_occupant
            {
                violations.push(
                    "Restoration agreement is NOT a permissible condition where the modification \
                     would not materially impact the next occupant (per HUD Joint Statement on \
                     Reasonable Modifications 2008 + § 3604(f)(3)(A) reasonableness limit)."
                        .to_string(),
                );
            }

            // California-specific escrow cap and no-additional-security rule.
            if matches!(input.regime, Regime::California) {
                citation_parts.push(
                    "Cal. Civ. Code § 54.1(b)(3)(A) (tenant's expense); § 54.1(b)(3)(B) \
                     (restoration agreement; no additional security; escrow capped at reasonable \
                     estimate of restoration cost)",
                );
                if input.escrow_amount_cents > 0 {
                    if input.escrow_amount_cents <= input.restoration_estimate_cents {
                        escrow_within_cap = true;
                    } else {
                        violations.push(format!(
                            "California § 54.1(b)(3)(B): escrow amount {} cents exceeds the \
                             reasonable-estimate restoration cap {} cents.",
                            input.escrow_amount_cents, input.restoration_estimate_cents,
                        ));
                    }
                }
                notes.push(
                    "California § 54(b) extends modification + accommodation protections to \
                     sensory disability (blind, visually-impaired, deaf, hearing-impaired) more \
                     broadly than the ADA's functional-limitation test."
                        .to_string(),
                );
            } else if input.escrow_amount_cents > 0 {
                // Federal + WA + NYC: no statutory escrow cap. Escrow
                // remains a contract matter, but it's outside the
                // California § 54.1(b)(3)(B) cap regime.
                notes.push(
                    "Outside California, escrow is permissible by agreement but is NOT subject \
                     to a statutory cap analogous to Cal. Civ. Code § 54.1(b)(3)(B)."
                        .to_string(),
                );
            }
        }
        RequestType::Accommodation => {
            cost_borne_by = "landlord";
            citation_parts.push(
                "42 U.S.C. § 3604(f)(3)(B) (reasonable accommodation in rules, policies, \
                 practices, services; at landlord's expense unless undue burden or fundamental \
                 alteration)",
            );
            if input.creates_undue_financial_burden {
                grantable = false;
                violations.push(
                    "Accommodation imposes an undue financial or administrative burden — \
                     § 3604(f)(3)(B) duty defeated."
                        .to_string(),
                );
            }
            if input.creates_fundamental_alteration {
                grantable = false;
                violations.push(
                    "Accommodation would require a fundamental alteration of the landlord's \
                     program or service — § 3604(f)(3)(B) duty defeated."
                        .to_string(),
                );
            }
        }
    }

    // NYC cooperative-dialogue mandate — unique to NYC across our
    // four regimes. Required even where the accommodation is
    // ultimately denied; the failure to engage IS the discriminatory
    // act independent of the substantive outcome.
    let cooperative_dialogue_required = matches!(input.regime, Regime::NYC);
    let cooperative_dialogue_completed_within_reasonable_time = if cooperative_dialogue_required {
        let engaged_in_time = input.landlord_engaged_in_dialogue
            && input.days_since_request_received <= NYC_DIALOGUE_REASONABLE_TIME_DAYS;
        if !input.landlord_engaged_in_dialogue {
            grantable = false;
            violations.push(
                "NYC § 8-107(15)(c): landlord failed to engage in cooperative dialogue. Failure \
                 to engage is itself a discriminatory practice regardless of substantive outcome \
                 (§ 8-102 cooperative dialogue definition)."
                    .to_string(),
            );
        } else if input.days_since_request_received > NYC_DIALOGUE_REASONABLE_TIME_DAYS {
            grantable = false;
            violations.push(format!(
                "NYC § 8-107(15)(c): cooperative dialogue not completed within reasonable time \
                 (request received {} days ago; audit threshold = {} days).",
                input.days_since_request_received, NYC_DIALOGUE_REASONABLE_TIME_DAYS,
            ));
        }
        citation_parts.push(
            "N.Y.C. Admin. Code § 8-107(15)(c) (cooperative-dialogue mandate); § 8-102 \
             (dialogue definition)",
        );
        Some(engaged_in_time)
    } else {
        None
    };

    if matches!(input.regime, Regime::Washington) {
        citation_parts.push("RCW 49.60.222(2)(b) (Washington reasonable modification / accommodation)");
    }

    let citation = citation_parts.join("; ");

    notes.push(
        "Accommodation = change to rule/policy/practice/service (landlord pays unless undue \
         burden / fundamental alteration). Modification = physical change to premises (tenant \
         pays; restoration agreement permissible only where modification materially impacts \
         next occupant)."
            .to_string(),
    );

    CheckResult {
        request_grantable: grantable,
        cost_borne_by,
        restoration_permissible,
        escrow_within_cap,
        cooperative_dialogue_required,
        cooperative_dialogue_completed_within_reasonable_time,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime, request_type: RequestType) -> Input {
        Input {
            regime,
            request_type,
            tenant_has_qualifying_disability: true,
            nexus_to_disability: true,
            landlord_engaged_in_dialogue: true,
            days_since_request_received: 10,
            modification_cost_cents: 0,
            restoration_required: false,
            escrow_amount_cents: 0,
            restoration_estimate_cents: 0,
            creates_undue_financial_burden: false,
            creates_fundamental_alteration: false,
            modification_would_materially_impact_next_occupant: false,
        }
    }

    // ── Federal modification path ────────────────────────────────

    #[test]
    fn federal_modification_grantable_tenant_pays() {
        let r = check(&base(Regime::Federal, RequestType::Modification));
        assert!(r.request_grantable);
        assert_eq!(r.cost_borne_by, "tenant");
        assert!(r.citation.contains("§ 3604(f)(3)(A)"));
        assert!(!r.cooperative_dialogue_required);
        assert_eq!(r.cooperative_dialogue_completed_within_reasonable_time, None);
    }

    #[test]
    fn federal_modification_restoration_permissible_only_when_materially_impacts_next() {
        let mut i = base(Regime::Federal, RequestType::Modification);
        i.restoration_required = true;
        i.modification_would_materially_impact_next_occupant = true;
        let r = check(&i);
        assert!(r.request_grantable);
        assert!(r.restoration_permissible);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn federal_modification_restoration_not_permissible_when_no_material_impact() {
        let mut i = base(Regime::Federal, RequestType::Modification);
        i.restoration_required = true;
        i.modification_would_materially_impact_next_occupant = false;
        let r = check(&i);
        assert!(!r.restoration_permissible);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("Restoration agreement is NOT a permissible condition"))
        );
    }

    // ── Federal accommodation path ───────────────────────────────

    #[test]
    fn federal_accommodation_grantable_landlord_pays() {
        let r = check(&base(Regime::Federal, RequestType::Accommodation));
        assert!(r.request_grantable);
        assert_eq!(r.cost_borne_by, "landlord");
        assert!(r.citation.contains("§ 3604(f)(3)(B)"));
    }

    #[test]
    fn federal_accommodation_undue_burden_defeats_duty() {
        let mut i = base(Regime::Federal, RequestType::Accommodation);
        i.creates_undue_financial_burden = true;
        let r = check(&i);
        assert!(!r.request_grantable);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("undue financial or administrative burden"))
        );
    }

    #[test]
    fn federal_accommodation_fundamental_alteration_defeats_duty() {
        let mut i = base(Regime::Federal, RequestType::Accommodation);
        i.creates_fundamental_alteration = true;
        let r = check(&i);
        assert!(!r.request_grantable);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("fundamental alteration"))
        );
    }

    // ── Nexus / disability threshold ─────────────────────────────

    #[test]
    fn no_disability_no_duty() {
        let mut i = base(Regime::Federal, RequestType::Accommodation);
        i.tenant_has_qualifying_disability = false;
        let r = check(&i);
        assert!(!r.request_grantable);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("does not have a qualifying disability"))
        );
    }

    #[test]
    fn no_nexus_no_duty() {
        let mut i = base(Regime::Federal, RequestType::Modification);
        i.nexus_to_disability = false;
        let r = check(&i);
        assert!(!r.request_grantable);
        assert!(r.violations.iter().any(|v| v.contains("No nexus")));
    }

    // ── California § 54.1 escrow cap ─────────────────────────────

    #[test]
    fn california_modification_escrow_within_cap_compliant() {
        let mut i = base(Regime::California, RequestType::Modification);
        i.restoration_required = true;
        i.modification_would_materially_impact_next_occupant = true;
        i.escrow_amount_cents = 5_000_00;
        i.restoration_estimate_cents = 7_500_00;
        let r = check(&i);
        assert!(r.request_grantable);
        assert!(r.escrow_within_cap);
        assert!(r.citation.contains("§ 54.1(b)(3)(B)"));
        assert!(r.violations.is_empty());
    }

    #[test]
    fn california_modification_escrow_at_cap_boundary_compliant() {
        // At cap = compliant (≤ cap).
        let mut i = base(Regime::California, RequestType::Modification);
        i.restoration_required = true;
        i.modification_would_materially_impact_next_occupant = true;
        i.escrow_amount_cents = 7_500_00;
        i.restoration_estimate_cents = 7_500_00;
        let r = check(&i);
        assert!(r.escrow_within_cap);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn california_modification_escrow_exceeds_cap_violation() {
        let mut i = base(Regime::California, RequestType::Modification);
        i.restoration_required = true;
        i.modification_would_materially_impact_next_occupant = true;
        i.escrow_amount_cents = 10_000_00;
        i.restoration_estimate_cents = 7_500_00;
        let r = check(&i);
        assert!(!r.escrow_within_cap);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("escrow amount") && v.contains("exceeds"))
        );
    }

    #[test]
    fn california_sensory_disability_note() {
        let i = base(Regime::California, RequestType::Modification);
        let r = check(&i);
        assert!(
            r.notes.iter().any(|n| n.contains("§ 54(b)")
                && n.to_lowercase().contains("sensory"))
        );
    }

    // ── NYC cooperative-dialogue mandate ─────────────────────────

    #[test]
    fn nyc_cooperative_dialogue_required_flag_set() {
        let r = check(&base(Regime::NYC, RequestType::Accommodation));
        assert!(r.cooperative_dialogue_required);
        assert_eq!(
            r.cooperative_dialogue_completed_within_reasonable_time,
            Some(true)
        );
        assert!(r.citation.contains("§ 8-107(15)(c)"));
    }

    #[test]
    fn nyc_failure_to_engage_dialogue_is_violation() {
        let mut i = base(Regime::NYC, RequestType::Accommodation);
        i.landlord_engaged_in_dialogue = false;
        let r = check(&i);
        assert!(!r.request_grantable);
        assert_eq!(
            r.cooperative_dialogue_completed_within_reasonable_time,
            Some(false)
        );
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("failed to engage in cooperative dialogue"))
        );
    }

    #[test]
    fn nyc_dialogue_beyond_reasonable_time_is_violation() {
        let mut i = base(Regime::NYC, RequestType::Accommodation);
        i.landlord_engaged_in_dialogue = true;
        i.days_since_request_received = NYC_DIALOGUE_REASONABLE_TIME_DAYS + 1;
        let r = check(&i);
        assert!(!r.request_grantable);
        assert_eq!(
            r.cooperative_dialogue_completed_within_reasonable_time,
            Some(false)
        );
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("not completed within reasonable time"))
        );
    }

    #[test]
    fn nyc_dialogue_at_reasonable_time_boundary_compliant() {
        let mut i = base(Regime::NYC, RequestType::Accommodation);
        i.days_since_request_received = NYC_DIALOGUE_REASONABLE_TIME_DAYS;
        let r = check(&i);
        assert!(r.request_grantable);
        assert_eq!(
            r.cooperative_dialogue_completed_within_reasonable_time,
            Some(true)
        );
    }

    #[test]
    fn nyc_dialogue_failure_independent_of_substantive_grant() {
        // Even if request would otherwise be grantable, failure to
        // engage is its own discriminatory practice.
        let mut i = base(Regime::NYC, RequestType::Accommodation);
        i.landlord_engaged_in_dialogue = false;
        i.creates_undue_financial_burden = false;
        i.creates_fundamental_alteration = false;
        let r = check(&i);
        assert!(!r.request_grantable);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("Failure to engage is itself a discriminatory practice"))
        );
    }

    // ── Washington ───────────────────────────────────────────────

    #[test]
    fn washington_modification_grantable_tenant_pays() {
        let r = check(&base(Regime::Washington, RequestType::Modification));
        assert!(r.request_grantable);
        assert_eq!(r.cost_borne_by, "tenant");
        assert!(r.citation.contains("RCW 49.60.222(2)(b)"));
        assert!(!r.cooperative_dialogue_required);
    }

    #[test]
    fn washington_accommodation_grantable_landlord_pays() {
        let r = check(&base(Regime::Washington, RequestType::Accommodation));
        assert!(r.request_grantable);
        assert_eq!(r.cost_borne_by, "landlord");
        assert!(r.citation.contains("RCW 49.60.222(2)(b)"));
    }

    // ── Regression-critical multi-regime invariants ──────────────

    #[test]
    fn only_nyc_imposes_cooperative_dialogue_mandate_4_regime_invariant() {
        for &regime in &[
            Regime::Federal,
            Regime::California,
            Regime::Washington,
        ] {
            let r = check(&base(regime, RequestType::Accommodation));
            assert!(
                !r.cooperative_dialogue_required,
                "{:?} must NOT require cooperative dialogue",
                regime,
            );
            assert_eq!(
                r.cooperative_dialogue_completed_within_reasonable_time,
                None,
            );
        }
        let nyc = check(&base(Regime::NYC, RequestType::Accommodation));
        assert!(nyc.cooperative_dialogue_required);
    }

    #[test]
    fn only_california_caps_escrow_at_reasonable_estimate_4_regime_invariant() {
        // Out-of-cap escrow should violate ONLY in CA. Federal + WA +
        // NYC are silent on the statutory cap.
        let setup = |regime: Regime| {
            let mut i = base(regime, RequestType::Modification);
            i.restoration_required = true;
            i.modification_would_materially_impact_next_occupant = true;
            i.escrow_amount_cents = 50_000_00;
            i.restoration_estimate_cents = 5_000_00;
            i
        };
        let ca = check(&setup(Regime::California));
        assert!(
            ca.violations
                .iter()
                .any(|v| v.contains("escrow amount") && v.contains("exceeds"))
        );
        for &regime in &[Regime::Federal, Regime::NYC, Regime::Washington] {
            let r = check(&setup(regime));
            assert!(
                !r.violations
                    .iter()
                    .any(|v| v.contains("escrow amount") && v.contains("exceeds")),
                "{:?} must NOT impose the CA escrow cap",
                regime,
            );
        }
    }

    #[test]
    fn modification_always_at_tenant_expense_across_all_regimes_invariant() {
        for &regime in &[
            Regime::Federal,
            Regime::California,
            Regime::NYC,
            Regime::Washington,
        ] {
            let mut i = base(regime, RequestType::Modification);
            // NYC dialogue baseline so it's grantable.
            i.landlord_engaged_in_dialogue = true;
            i.days_since_request_received = 5;
            let r = check(&i);
            assert_eq!(
                r.cost_borne_by, "tenant",
                "{:?}: modification must be at tenant's expense",
                regime,
            );
        }
    }

    #[test]
    fn accommodation_always_at_landlord_expense_across_all_regimes_invariant() {
        for &regime in &[
            Regime::Federal,
            Regime::California,
            Regime::NYC,
            Regime::Washington,
        ] {
            let mut i = base(regime, RequestType::Accommodation);
            i.landlord_engaged_in_dialogue = true;
            i.days_since_request_received = 5;
            let r = check(&i);
            assert_eq!(
                r.cost_borne_by, "landlord",
                "{:?}: accommodation must be at landlord's expense",
                regime,
            );
        }
    }

    #[test]
    fn nexus_requirement_universal_across_regimes_invariant() {
        for &regime in &[
            Regime::Federal,
            Regime::California,
            Regime::NYC,
            Regime::Washington,
        ] {
            let mut i = base(regime, RequestType::Accommodation);
            i.nexus_to_disability = false;
            let r = check(&i);
            assert!(
                r.violations.iter().any(|v| v.contains("No nexus")),
                "{:?}: nexus-to-disability must be required",
                regime,
            );
        }
    }

    #[test]
    fn undue_burden_defeats_accommodation_across_all_regimes_invariant() {
        for &regime in &[
            Regime::Federal,
            Regime::California,
            Regime::NYC,
            Regime::Washington,
        ] {
            let mut i = base(regime, RequestType::Accommodation);
            i.landlord_engaged_in_dialogue = true;
            i.days_since_request_received = 5;
            i.creates_undue_financial_burden = true;
            let r = check(&i);
            assert!(
                !r.request_grantable,
                "{:?}: undue financial burden must defeat accommodation",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_subsection_per_regime_request_type() {
        let fed_mod = check(&base(Regime::Federal, RequestType::Modification));
        let fed_acc = check(&base(Regime::Federal, RequestType::Accommodation));
        let ca_mod = check(&base(Regime::California, RequestType::Modification));
        let nyc_acc = check(&base(Regime::NYC, RequestType::Accommodation));
        let wa_mod = check(&base(Regime::Washington, RequestType::Modification));

        assert!(fed_mod.citation.contains("§ 3604(f)(3)(A)"));
        assert!(fed_acc.citation.contains("§ 3604(f)(3)(B)"));
        assert!(ca_mod.citation.contains("§ 54.1(b)(3)"));
        assert!(nyc_acc.citation.contains("§ 8-107(15)(c)"));
        assert!(nyc_acc.citation.contains("§ 8-102"));
        assert!(wa_mod.citation.contains("RCW 49.60.222(2)(b)"));
    }

    #[test]
    fn notes_always_include_accommodation_vs_modification_distinction() {
        // The distinction note is the central conceptual frame; pin
        // it so future refactors don't drop it.
        for &regime in &[
            Regime::Federal,
            Regime::California,
            Regime::NYC,
            Regime::Washington,
        ] {
            for &rt in &[RequestType::Modification, RequestType::Accommodation] {
                let r = check(&base(regime, rt));
                assert!(
                    r.notes
                        .iter()
                        .any(|n| n.contains("Accommodation") && n.contains("Modification")),
                    "{:?} / {:?}: distinction note must be present",
                    regime,
                    rt,
                );
            }
        }
    }
}
