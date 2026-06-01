//! State landlord-lien-on-tenant-property compliance check.
//!
//! Distinct from `abandoned_property_handling` (which addresses
//! disposition of belongings LEFT behind after vacating) — this
//! module addresses the landlord's right to assert a possessory
//! lien on tenant's personal property DURING tenancy as security
//! for unpaid rent.
//!
//! Six regimes with sharply different posture:
//!
//!   - **Texas** — Tex. Prop. Code § 54.041(a) — landlord of a
//!     single or multifamily residence has a STATUTORY LIEN for
//!     unpaid rent that attaches to nonexempt property in the
//!     residence or storage room. Strongest pro-landlord regime
//!     in the matrix. Contractual lien (§ 54.043) is enforceable
//!     ONLY if underlined or printed in conspicuous bold print in
//!     the lease agreement.
//!
//!   - **California** — Cal. Civ. Code § 1861(a) — landlord may
//!     not take physical possession of tenant's property without
//!     first obtaining a court order. Even with a court order,
//!     the lien may NOT be enforced against property necessary
//!     to the tenant's livelihood or any necessary household
//!     items. § 1861.5 through § 1861.27 govern enforcement
//!     procedure.
//!
//!   - **NewYork** — NO STATUTORY landlord lien against tenant
//!     personal property. A lien may exist ONLY as a contractual
//!     term in the original lease OR as a court-rendered judgment
//!     lien. Landlord may not assert self-help possession of
//!     tenant property.
//!
//!   - **Massachusetts** — No precedent for a statutory landlord
//!     lien against personal property in an ordinary tenancy. UCC
//!     Article 9 permits a voluntary lien given by an owner; a
//!     storage lien is available to warehouse operators.
//!
//!   - **Illinois** — Court judgment is required FIRST. Unpaid
//!     rent alone does not automatically create a lien. Landlord
//!     must file a lawsuit, obtain judgment, and then proceed via
//!     judgment-lien process.
//!
//!   - **Default** — varies by state; common-law landlord's lien
//!     generally requires court order. No automatic statutory
//!     lien.
//!
//! Texas, California, and the rest diverge along five axes:
//! (1) statutory lien exists; (2) court order required before
//! enforcement; (3) livelihood / household items carve-out;
//! (4) conspicuous bold-print contractual requirement; (5)
//! self-help possession permitted.
//!
//! Citations: Tex. Prop. Code § 54.041(a) (statutory lien) +
//! § 54.043 (contractual lien conspicuous-bold-print requirement);
//! Cal. Civ. Code § 1861(a) (court order required + livelihood/
//! household carve-outs); Cal. Civ. Code §§ 1861.5–1861.27
//! (enforcement procedure); NY General Obligations Law (no
//! statutory landlord lien); Mass. UCC Article 9 (voluntary lien
//! only); 735 ILCS 5/9 et seq. (Illinois judgment-lien procedure).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Texas,
    California,
    NewYork,
    Massachusetts,
    Illinois,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Whether the landlord obtained a court order before
    /// asserting possession of the tenant's property.
    pub court_order_obtained: bool,
    /// Whether the lease contains a contractual lien clause.
    pub lease_contains_contractual_lien: bool,
    /// Whether the contractual lien clause is in conspicuous bold
    /// print or underlined (required by Tex. Prop. Code § 54.043).
    pub contractual_lien_in_conspicuous_bold: bool,
    /// Whether the targeted property is necessary to the tenant's
    /// livelihood or includes necessary household items (CA
    /// § 1861(a) carve-out).
    pub property_is_livelihood_or_household_necessary: bool,
    /// Whether the landlord took self-help possession of the
    /// tenant's property without judicial process.
    pub self_help_possession_taken: bool,
    /// Whether the landlord obtained a judgment for unpaid rent
    /// before asserting a lien (Illinois requirement).
    pub judgment_for_unpaid_rent_obtained: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime grants a STATUTORY landlord lien on
    /// tenant property for unpaid rent (Texas only).
    pub statutory_lien_exists: bool,
    /// True if the regime requires a COURT ORDER before the
    /// landlord may take possession of tenant property.
    pub court_order_required: bool,
    /// True if the regime carves out property necessary to the
    /// tenant's livelihood or household items (California only).
    pub livelihood_household_carve_out: bool,
    /// True if the regime requires the contractual lien clause to
    /// be in conspicuous bold print / underlined (Texas only).
    pub conspicuous_bold_required_for_contractual_lien: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (
        statutory_lien_exists,
        court_order_required,
        livelihood_household_carve_out,
        conspicuous_bold_required,
        citation,
    ): (bool, bool, bool, bool, &'static str) = match input.regime {
        Regime::Texas => (
            true,
            false,
            false,
            true,
            "Tex. Prop. Code § 54.041(a) (statutory residential landlord lien on nonexempt \
             property in residence for unpaid rent); § 54.043 (contractual lien must be \
             underlined or in conspicuous bold print)",
        ),
        Regime::California => (
            false,
            true,
            true,
            false,
            "Cal. Civ. Code § 1861(a) (court order required; no enforcement against property \
             necessary to tenant's livelihood or household necessary items); §§ 1861.5–1861.27 \
             (enforcement procedure)",
        ),
        Regime::NewYork => (
            false,
            true,
            false,
            false,
            "NY General Obligations Law — no statutory landlord lien; may exist only as \
             contractual term in original lease OR as court-rendered judgment lien; no \
             self-help possession of tenant property permitted",
        ),
        Regime::Massachusetts => (
            false,
            true,
            false,
            false,
            "Massachusetts — no precedent for statutory landlord lien against personal property \
             in ordinary tenancy; UCC Article 9 permits voluntary lien; storage lien available \
             to warehouse operators",
        ),
        Regime::Illinois => (
            false,
            true,
            false,
            false,
            "735 ILCS 5/9 et seq. (Illinois — court judgment required first; unpaid rent alone \
             does not automatically create a lien; landlord must file lawsuit + obtain \
             judgment + proceed via judgment-lien process)",
        ),
        Regime::Default => (
            false,
            true,
            false,
            false,
            "Default — varies by state; common-law landlord's lien generally requires court \
             order; no automatic statutory lien",
        ),
    };

    // Self-help possession check — prohibited where court order is
    // required.
    if court_order_required && input.self_help_possession_taken && !input.court_order_obtained {
        violations.push(format!(
            "{:?} — landlord took self-help possession of tenant property without obtaining a \
             court order; possession is unlawful.",
            input.regime,
        ));
    }

    // California livelihood/household carve-out.
    if matches!(input.regime, Regime::California)
        && input.property_is_livelihood_or_household_necessary
        && (input.self_help_possession_taken || input.court_order_obtained)
    {
        violations.push(
            "Cal. Civ. Code § 1861(a) — lien may NOT be enforced against property necessary to \
             the tenant's livelihood or any necessary household items, even with a court order."
                .to_string(),
        );
    }

    // Texas contractual lien conspicuous-bold requirement.
    if matches!(input.regime, Regime::Texas)
        && input.lease_contains_contractual_lien
        && !input.contractual_lien_in_conspicuous_bold
    {
        violations.push(
            "Tex. Prop. Code § 54.043 — contractual landlord lien is not enforceable unless \
             underlined or printed in conspicuous bold print in the lease agreement; clause \
             present but formatting requirement not satisfied."
                .to_string(),
        );
    }

    // Illinois judgment-required check.
    if matches!(input.regime, Regime::Illinois)
        && input.self_help_possession_taken
        && !input.judgment_for_unpaid_rent_obtained
    {
        violations.push(
            "Illinois — landlord asserted lien on tenant property without first obtaining a \
             judgment for unpaid rent; unlawful self-help."
                .to_string(),
        );
    }

    notes.push(
        "Distinct from abandoned_property_handling (which addresses disposition of belongings \
         LEFT BEHIND after tenant vacates). This module addresses the landlord's right to \
         assert a possessory lien on tenant's personal property DURING tenancy as security \
         for unpaid rent."
            .to_string(),
    );

    if matches!(input.regime, Regime::Texas) {
        notes.push(
            "Tex. Prop. Code § 54.041 — Texas is the STRONGEST pro-landlord regime in this \
             matrix; statutory lien attaches automatically to nonexempt property in the \
             residence for unpaid rent."
                .to_string(),
        );
    }

    if matches!(input.regime, Regime::California) {
        notes.push(
            "Cal. Civ. Code § 1861(a) — California carve-outs for property necessary to \
             tenant's livelihood and necessary household items apply even where a court order \
             has been obtained. Self-help possession is unlawful regardless of order."
                .to_string(),
        );
    }

    CheckResult {
        statutory_lien_exists,
        court_order_required,
        livelihood_household_carve_out,
        conspicuous_bold_required_for_contractual_lien: conspicuous_bold_required,
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
            court_order_obtained: false,
            lease_contains_contractual_lien: false,
            contractual_lien_in_conspicuous_bold: false,
            property_is_livelihood_or_household_necessary: false,
            self_help_possession_taken: false,
            judgment_for_unpaid_rent_obtained: false,
        }
    }

    // ── Texas § 54.041 — strongest pro-landlord regime ─────────

    #[test]
    fn texas_statutory_lien_exists() {
        let r = check(&base(Regime::Texas));
        assert!(r.statutory_lien_exists);
        assert!(!r.court_order_required);
        assert!(r.conspicuous_bold_required_for_contractual_lien);
        assert!(r.citation.contains("§ 54.041(a)"));
    }

    #[test]
    fn texas_contractual_lien_without_bold_print_unenforceable() {
        let mut i = base(Regime::Texas);
        i.lease_contains_contractual_lien = true;
        i.contractual_lien_in_conspicuous_bold = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 54.043") && v.contains("conspicuous bold"))
        );
    }

    #[test]
    fn texas_contractual_lien_in_conspicuous_bold_enforceable() {
        let mut i = base(Regime::Texas);
        i.lease_contains_contractual_lien = true;
        i.contractual_lien_in_conspicuous_bold = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn texas_self_help_permitted_without_court_order() {
        // Texas does not require court order for statutory lien.
        let mut i = base(Regime::Texas);
        i.self_help_possession_taken = true;
        let r = check(&i);
        // No court-order violation in TX.
        assert!(!r.violations.iter().any(|v| v.contains("court order")));
    }

    // ── California § 1861(a) — court order + carve-outs ─────────

    #[test]
    fn california_court_order_required() {
        let r = check(&base(Regime::California));
        assert!(r.court_order_required);
        assert!(r.livelihood_household_carve_out);
        assert!(!r.statutory_lien_exists);
        assert!(r.citation.contains("§ 1861(a)"));
    }

    #[test]
    fn california_self_help_without_court_order_violation() {
        let mut i = base(Regime::California);
        i.self_help_possession_taken = true;
        i.court_order_obtained = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("California") && v.contains("unlawful"))
        );
    }

    #[test]
    fn california_with_court_order_no_self_help_violation() {
        let mut i = base(Regime::California);
        i.court_order_obtained = true;
        i.self_help_possession_taken = true;
        let r = check(&i);
        // With court order, no self-help violation. But still need
        // to clear other gates.
        assert!(r.compliant);
    }

    #[test]
    fn california_livelihood_property_carve_out_violation() {
        let mut i = base(Regime::California);
        i.court_order_obtained = true;
        i.self_help_possession_taken = true;
        i.property_is_livelihood_or_household_necessary = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1861(a)") && v.contains("livelihood"))
        );
    }

    #[test]
    fn california_livelihood_carve_out_even_with_court_order_violation() {
        let mut i = base(Regime::California);
        i.court_order_obtained = true;
        i.self_help_possession_taken = false;
        i.property_is_livelihood_or_household_necessary = true;
        let r = check(&i);
        // Court order obtained but property is livelihood —
        // statute still forbids enforcement.
        assert!(!r.compliant);
    }

    // ── New York — no statutory lien ────────────────────────────

    #[test]
    fn new_york_no_statutory_lien_court_order_required() {
        let r = check(&base(Regime::NewYork));
        assert!(!r.statutory_lien_exists);
        assert!(r.court_order_required);
        assert!(r.citation.contains("no statutory landlord lien"));
    }

    #[test]
    fn new_york_self_help_violation() {
        let mut i = base(Regime::NewYork);
        i.self_help_possession_taken = true;
        let r = check(&i);
        assert!(!r.compliant);
    }

    // ── Massachusetts — UCC voluntary only ──────────────────────

    #[test]
    fn massachusetts_no_statutory_lien() {
        let r = check(&base(Regime::Massachusetts));
        assert!(!r.statutory_lien_exists);
        assert!(r.court_order_required);
        assert!(r.citation.contains("UCC Article 9"));
    }

    // ── Illinois — judgment required first ──────────────────────

    #[test]
    fn illinois_judgment_required_first() {
        let r = check(&base(Regime::Illinois));
        assert!(!r.statutory_lien_exists);
        assert!(r.court_order_required);
        assert!(r.citation.contains("735 ILCS 5/9"));
    }

    #[test]
    fn illinois_self_help_without_judgment_violation() {
        let mut i = base(Regime::Illinois);
        i.self_help_possession_taken = true;
        i.judgment_for_unpaid_rent_obtained = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("Illinois") && v.contains("judgment"))
        );
    }

    #[test]
    fn illinois_self_help_with_judgment_no_violation_specific_to_il() {
        let mut i = base(Regime::Illinois);
        i.self_help_possession_taken = true;
        i.judgment_for_unpaid_rent_obtained = true;
        i.court_order_obtained = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    // ── Default ─────────────────────────────────────────────────

    #[test]
    fn default_court_order_required_no_statutory_lien() {
        let r = check(&base(Regime::Default));
        assert!(!r.statutory_lien_exists);
        assert!(r.court_order_required);
        assert!(r.citation.contains("common-law"));
    }

    // ── Regression-critical multi-regime invariants ────────────

    #[test]
    fn only_texas_has_statutory_lien_invariant() {
        let tx = check(&base(Regime::Texas));
        assert!(tx.statutory_lien_exists);
        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Illinois,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.statutory_lien_exists,
                "{:?}: must NOT have statutory lien",
                regime,
            );
        }
    }

    #[test]
    fn only_california_has_livelihood_carve_out_invariant() {
        let ca = check(&base(Regime::California));
        assert!(ca.livelihood_household_carve_out);
        for &regime in &[
            Regime::Texas,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Illinois,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.livelihood_household_carve_out,
                "{:?}: must NOT have livelihood carve-out",
                regime,
            );
        }
    }

    #[test]
    fn only_texas_requires_conspicuous_bold_for_contractual_lien_invariant() {
        let tx = check(&base(Regime::Texas));
        assert!(tx.conspicuous_bold_required_for_contractual_lien);
        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Illinois,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.conspicuous_bold_required_for_contractual_lien,
                "{:?}: must NOT require conspicuous bold for contractual lien",
                regime,
            );
        }
    }

    #[test]
    fn only_texas_does_not_require_court_order_invariant() {
        let tx = check(&base(Regime::Texas));
        assert!(!tx.court_order_required);
        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Illinois,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.court_order_required,
                "{:?}: must require court order",
                regime,
            );
        }
    }

    #[test]
    fn self_help_without_court_order_violates_5_regimes_invariant() {
        // All non-Texas regimes require court order; self-help
        // without one is a violation.
        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Illinois,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.self_help_possession_taken = true;
            i.court_order_obtained = false;
            let r = check(&i);
            assert!(
                !r.compliant,
                "{:?}: self-help without court order must violate",
                regime,
            );
        }
        // Texas does NOT violate on this fact pattern.
        let mut tx = base(Regime::Texas);
        tx.self_help_possession_taken = true;
        tx.court_order_obtained = false;
        let r = check(&tx);
        assert!(!r.violations.iter().any(|v| v.contains("court order")));
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::Texas)).citation.contains("§ 54.041(a)"));
        assert!(check(&base(Regime::California)).citation.contains("§ 1861(a)"));
        assert!(check(&base(Regime::NewYork)).citation.contains("General Obligations"));
        assert!(check(&base(Regime::Massachusetts)).citation.contains("UCC Article 9"));
        assert!(check(&base(Regime::Illinois)).citation.contains("735 ILCS"));
        assert!(check(&base(Regime::Default)).citation.contains("common-law"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::Texas,
            Regime::California,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Illinois,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("abandoned_property_handling")
                        && n.contains("DURING tenancy")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }
}
