//! State landlord sex-offender-database notice disclosure compliance
//! check (Megan's Law database notice in residential leases).
//!
//! Federal Megan's Law (originally 42 U.S.C. § 14071, recodified at
//! 34 U.S.C. § 20911 et seq.) requires states to maintain public sex-
//! offender registries. California is the ONLY state in the matrix
//! with a STATUTORY MANDATE that every residential lease include
//! specific verbatim notice language pointing tenants to the state's
//! Megan's Law website. Other states rely on community notification
//! by law enforcement (the registry itself + AG / prosecutor-driven
//! community alerts) rather than a lease-side disclosure.
//!
//! Rounds out the hazard-disclosure / community-information cluster
//! alongside `military_ordnance_disclosure`, `asbestos_disclosure`,
//! `lead_disclosure`, `mold_disclosure`, `radon_disclosure`,
//! `flood_disclosure`, `meth_contamination_disclosure`,
//! `bedbug_disclosure`, `fire_sprinkler_disclosure`, and
//! `death_in_unit_disclosure`.
//!
//! Three regimes:
//!
//!   - **California** — Cal. Civ. Code § 2079.10a + Pen. Code
//!     § 290.46. Every residential lease MUST include the following
//!     VERBATIM statutory notice (no paraphrasing permitted):
//!     "Notice: Pursuant to Section 290.46 of the Penal Code,
//!     information about specified registered sex offenders is made
//!     available to the public via an Internet Web site maintained
//!     by the Department of Justice at www.meganslaw.ca.gov.
//!     Depending on an offender's criminal history, this
//!     information will include either the address at which the
//!     offender resides or the community of residence and ZIP code
//!     in which he or she resides." § 2079.10a(c) — landlord is NOT
//!     required to provide specific names or addresses of registered
//!     offenders; tenant searches the database directly.
//!
//!   - **NewJersey** — N.J.S.A. 2C:7-1 et seq. (Registration and
//!     Community Notification Laws — RCNL). Community-notification
//!     framework administered by county prosecutor and N.J. State
//!     Police. No statutory landlord-tenant lease-disclosure
//!     mandate; the registry is public.
//!
//!   - **Default** — no statutory landlord disclosure mandate.
//!     Federal Megan's Law (34 U.S.C. § 20911 et seq.) and state
//!     registries make information public; common-law fraudulent-
//!     concealment liability may attach where landlord ACTIVELY
//!     misrepresents the absence of registered offenders.
//!
//! California § 2079.10a key load-bearing aspects:
//!   (1) VERBATIM language required — no paraphrasing, no
//!       summarization, no substitution of website URL.
//!   (2) Notice must be PROVIDED at lease execution / when rental
//!       period begins.
//!   (3) Landlord has NO DUTY to investigate the registry or
//!       affirmatively warn about specific offenders.
//!   (4) Landlord has NO DUTY to deny or cancel tenancy based on
//!       registry status (FHA + state anti-discrimination overlay).
//!
//! Citations: Cal. Civ. Code § 2079.10a(a) (verbatim notice
//! language requirement); § 2079.10a(b) (lease-execution timing);
//! § 2079.10a(c) (landlord not required to provide specific
//! offender information); Cal. Pen. Code § 290.46 (Megan's Law
//! database authority); N.J.S.A. 2C:7-1 et seq. (NJ Registration
//! and Community Notification Laws); 34 U.S.C. § 20911 et seq.
//! (federal Megan's Law framework — formerly 42 U.S.C. § 14071).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Whether the lease contains the § 2079.10a Megan's Law database
    /// notice (any version).
    pub lease_contains_megans_law_notice: bool,
    /// Whether the notice in the lease matches the § 2079.10a(a)
    /// VERBATIM statutory language (no paraphrasing).
    pub notice_is_verbatim_statutory_language: bool,
    /// Whether the notice was provided at lease execution / when
    /// rental period began.
    pub notice_provided_at_lease_execution: bool,
    /// Whether the landlord actively misrepresented the absence of
    /// registered offenders (relevant for Default common-law fraud
    /// liability).
    pub landlord_actively_misrepresented_offender_absence: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime imposes a statutory lease-disclosure
    /// mandate (California only).
    pub lease_disclosure_mandate: bool,
    /// True if the regime requires VERBATIM statutory language (no
    /// paraphrasing).
    pub verbatim_language_required: bool,
    /// True if a community-notification framework exists outside
    /// the lease context (every regime — federal + state registries
    /// + state-specific RCNL frameworks).
    pub community_notification_framework_exists: bool,
    /// True if common-law fraudulent-concealment liability is
    /// triggered (Default + any regime where landlord actively
    /// misrepresents).
    pub common_law_fraud_liability_triggered: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (lease_disclosure_mandate, verbatim_language_required, citation): (
        bool,
        bool,
        &'static str,
    ) = match input.regime {
        Regime::California => (
            true,
            true,
            "Cal. Civ. Code § 2079.10a(a) (verbatim Megan's Law database notice required in \
             every residential lease — no paraphrasing); § 2079.10a(b) (notice timing — lease \
             execution / rental period begins); § 2079.10a(c) (landlord not required to \
             provide specific offender names or addresses); Cal. Pen. Code § 290.46 (Megan's \
             Law database authority — www.meganslaw.ca.gov)",
        ),
        Regime::NewJersey => (
            false,
            false,
            "N.J.S.A. 2C:7-1 et seq. (NJ Registration and Community Notification Laws — RCNL); \
             community-notification framework administered by county prosecutor + N.J. State \
             Police; no statutory landlord-tenant lease-disclosure mandate",
        ),
        Regime::Default => (
            false,
            false,
            "34 U.S.C. § 20911 et seq. (federal Megan's Law framework — formerly 42 U.S.C. \
             § 14071); state registries; no statutory landlord lease-disclosure mandate; \
             common-law fraudulent-concealment liability where landlord actively misrepresents",
        ),
    };

    // California compliance — notice presence + verbatim language +
    // timing.
    if matches!(input.regime, Regime::California) {
        if !input.lease_contains_megans_law_notice {
            violations.push(
                "Cal. Civ. Code § 2079.10a(a) — residential lease must contain Megan's Law \
                 database notice; notice not present."
                    .to_string(),
            );
        }
        if input.lease_contains_megans_law_notice && !input.notice_is_verbatim_statutory_language {
            violations.push(
                "Cal. Civ. Code § 2079.10a(a) — notice must be VERBATIM statutory language; \
                 paraphrased or summarized notice does not satisfy the statute."
                    .to_string(),
            );
        }
        if !input.notice_provided_at_lease_execution {
            violations.push(
                "Cal. Civ. Code § 2079.10a(b) — notice must be provided at lease execution / \
                 when rental period begins; timing requirement not satisfied."
                    .to_string(),
            );
        }
    }

    // Common-law fraudulent-concealment liability — triggered where
    // landlord ACTIVELY misrepresents the absence of registered
    // offenders (applies in any regime).
    let common_law_fraud_liability_triggered =
        input.landlord_actively_misrepresented_offender_absence;
    if common_law_fraud_liability_triggered {
        violations.push(
            "Common-law fraudulent-concealment liability — landlord actively misrepresented \
             the absence of registered offenders; potential exposure to fraudulent-\
             misrepresentation claims regardless of state lease-disclosure regime."
                .to_string(),
        );
    }

    // § 2079.10a(c) landlord has no duty to provide specific
    // offender info — clarifying note.
    if matches!(input.regime, Regime::California) {
        notes.push(
            "Cal. Civ. Code § 2079.10a(c) — landlord is NOT required to provide specific \
             registered-offender names or addresses; tenant searches the Megan's Law database \
             directly at www.meganslaw.ca.gov."
                .to_string(),
        );
        notes.push(
            "Landlord has NO statutory duty to investigate the registry or affirmatively warn \
             tenants about specific registered offenders. Affirmative denial or cancellation \
             of tenancy based on registry status alone may trigger FHA + state anti-\
             discrimination scrutiny."
                .to_string(),
        );
    }

    notes.push(
        "Companion modules: military_ordnance_disclosure (Cal. Civ. Code § 1940.7 actual-\
         knowledge ordnance-site disclosure); death_in_unit_disclosure (Cal. Civ. Code \
         § 1710.2 three-year window); hazard-disclosure cluster (asbestos + lead + mold + \
         radon + flood + meth + bedbug + fire-sprinkler). Federal Megan's Law floor at 34 \
         U.S.C. § 20911 et seq."
            .to_string(),
    );

    CheckResult {
        lease_disclosure_mandate,
        verbatim_language_required,
        community_notification_framework_exists: true,
        common_law_fraud_liability_triggered,
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
            lease_contains_megans_law_notice: true,
            notice_is_verbatim_statutory_language: true,
            notice_provided_at_lease_execution: true,
            landlord_actively_misrepresented_offender_absence: false,
        }
    }

    // ── California § 2079.10a ────────────────────────────────────

    #[test]
    fn california_verbatim_notice_at_execution_compliant() {
        let r = check(&base(Regime::California));
        assert!(r.compliant);
        assert!(r.lease_disclosure_mandate);
        assert!(r.verbatim_language_required);
        assert!(r.citation.contains("§ 2079.10a(a)"));
        assert!(r.citation.contains("§ 290.46"));
    }

    #[test]
    fn california_missing_notice_violation() {
        let mut i = base(Regime::California);
        i.lease_contains_megans_law_notice = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2079.10a(a)") && v.contains("not present")));
    }

    #[test]
    fn california_paraphrased_notice_violation() {
        let mut i = base(Regime::California);
        i.notice_is_verbatim_statutory_language = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("VERBATIM") && v.contains("paraphrased")));
    }

    #[test]
    fn california_notice_after_execution_violation() {
        let mut i = base(Regime::California);
        i.notice_provided_at_lease_execution = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2079.10a(b)") && v.contains("lease execution")));
    }

    #[test]
    fn california_landlord_no_specific_offender_info_required_note() {
        let r = check(&base(Regime::California));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 2079.10a(c)")
                    && n.contains("NOT required to provide specific"))
        );
    }

    #[test]
    fn california_no_duty_to_investigate_note() {
        let r = check(&base(Regime::California));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("NO statutory duty to investigate") && n.contains("FHA")));
    }

    // ── New Jersey 2C:7-1 et seq. ───────────────────────────────

    #[test]
    fn new_jersey_no_lease_disclosure_mandate() {
        let r = check(&base(Regime::NewJersey));
        assert!(!r.lease_disclosure_mandate);
        assert!(!r.verbatim_language_required);
        assert!(r.community_notification_framework_exists);
        assert!(r.citation.contains("N.J.S.A. 2C:7-1"));
    }

    #[test]
    fn new_jersey_compliant_without_notice() {
        let mut i = base(Regime::NewJersey);
        i.lease_contains_megans_law_notice = false;
        i.notice_is_verbatim_statutory_language = false;
        i.notice_provided_at_lease_execution = false;
        let r = check(&i);
        // NJ has no lease-disclosure mandate; absence is compliant.
        assert!(r.compliant);
    }

    // ── Default — federal floor only ───────────────────────────

    #[test]
    fn default_no_state_specific_mandate() {
        let r = check(&base(Regime::Default));
        assert!(!r.lease_disclosure_mandate);
        assert!(r.citation.contains("34 U.S.C. § 20911"));
    }

    #[test]
    fn default_compliant_without_lease_notice() {
        let mut i = base(Regime::Default);
        i.lease_contains_megans_law_notice = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    // ── Common-law fraudulent-concealment liability ────────────

    #[test]
    fn active_misrepresentation_triggers_common_law_fraud_default() {
        let mut i = base(Regime::Default);
        i.landlord_actively_misrepresented_offender_absence = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.common_law_fraud_liability_triggered);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("fraudulent-concealment")));
    }

    #[test]
    fn active_misrepresentation_triggers_fraud_even_in_california() {
        let mut i = base(Regime::California);
        i.landlord_actively_misrepresented_offender_absence = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.common_law_fraud_liability_triggered);
    }

    #[test]
    fn active_misrepresentation_triggers_fraud_even_in_new_jersey() {
        let mut i = base(Regime::NewJersey);
        i.landlord_actively_misrepresented_offender_absence = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.common_law_fraud_liability_triggered);
    }

    #[test]
    fn no_misrepresentation_no_common_law_fraud() {
        let r = check(&base(Regime::Default));
        assert!(!r.common_law_fraud_liability_triggered);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn only_california_imposes_lease_disclosure_mandate_invariant() {
        let ca = check(&base(Regime::California));
        assert!(ca.lease_disclosure_mandate);
        for &regime in &[Regime::NewJersey, Regime::Default] {
            let r = check(&base(regime));
            assert!(
                !r.lease_disclosure_mandate,
                "{:?}: must NOT impose statutory lease-disclosure mandate",
                regime,
            );
        }
    }

    #[test]
    fn only_california_requires_verbatim_language_invariant() {
        assert!(check(&base(Regime::California)).verbatim_language_required);
        for &regime in &[Regime::NewJersey, Regime::Default] {
            assert!(
                !check(&base(regime)).verbatim_language_required,
                "{:?}: must NOT require verbatim language",
                regime,
            );
        }
    }

    #[test]
    fn community_notification_framework_exists_across_all_regimes_invariant() {
        // Every regime sits within at least the federal Megan's Law
        // framework (34 U.S.C. § 20911 et seq.).
        for &regime in &[Regime::California, Regime::NewJersey, Regime::Default] {
            let r = check(&base(regime));
            assert!(
                r.community_notification_framework_exists,
                "{:?}: community notification framework must exist",
                regime,
            );
        }
    }

    #[test]
    fn active_misrepresentation_triggers_fraud_universally_invariant() {
        for &regime in &[Regime::California, Regime::NewJersey, Regime::Default] {
            let mut i = base(regime);
            i.landlord_actively_misrepresented_offender_absence = true;
            let r = check(&i);
            assert!(
                r.common_law_fraud_liability_triggered,
                "{:?}: active misrepresentation must trigger fraud liability",
                regime,
            );
        }
    }

    #[test]
    fn ca_three_independent_violations_invariant() {
        // CA fail can be (a) missing notice, (b) paraphrased
        // language, or (c) wrong timing. Each is a standalone
        // violation.
        let configs = [
            (false, true, true, "not present"),
            (true, false, true, "paraphrased"),
            (true, true, false, "lease execution"),
        ];
        for (notice_present, verbatim, at_execution, expected_substring) in configs {
            let mut i = base(Regime::California);
            i.lease_contains_megans_law_notice = notice_present;
            i.notice_is_verbatim_statutory_language = verbatim;
            i.notice_provided_at_lease_execution = at_execution;
            let r = check(&i);
            assert!(!r.compliant);
            assert!(
                r.violations.iter().any(|v| v.contains(expected_substring)),
                "config (present={}, verbatim={}, at_execution={}): expected '{}' violation",
                notice_present,
                verbatim,
                at_execution,
                expected_substring,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California))
            .citation
            .contains("§ 2079.10a"));
        assert!(check(&base(Regime::California))
            .citation
            .contains("§ 290.46"));
        assert!(check(&base(Regime::NewJersey))
            .citation
            .contains("N.J.S.A. 2C:7-1"));
        assert!(check(&base(Regime::Default))
            .citation
            .contains("34 U.S.C. § 20911"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[Regime::California, Regime::NewJersey, Regime::Default] {
            let r = check(&base(regime));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("military_ordnance_disclosure")
                        && n.contains("death_in_unit_disclosure")
                        && n.contains("34 U.S.C. § 20911")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }
}
