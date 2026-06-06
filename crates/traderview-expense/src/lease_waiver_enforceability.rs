//! Lease waiver clause enforceability — when may a residential
//! or commercial lease validly waive tenant rights or landlord
//! liability?
//!
//! Distinct from sibling modules `landlord_harassment`
//! (affirmative-harassment penalties), `habitability_remedies`
//! (post-delivery habitability obligations), and `quiet_
//! enjoyment` (general post-delivery covenant). This module
//! focuses on the lease-drafted WAIVER PROVISION itself — when
//! the contract language attempting to extinguish a right or
//! obligation is void as a matter of statute or public policy.
//!
//! New York — N.Y. Gen. Oblig. Law § 5-321: "Every covenant,
//! agreement or understanding in or in connection with or
//! collateral to any lease of real property exempting the
//! lessor from liability for damages for injuries to person or
//! property caused by or resulting from the negligence of the
//! lessor, his agents, servants or employees, in the operation
//! or maintenance of the demised premises ... shall be deemed
//! to be void as against public policy and wholly
//! unenforceable." NARROW SCOPE — only landlord-negligence
//! exculpatory clauses are void. Applies to residential AND
//! commercial leases. Distinct from indemnification clauses
//! which remain enforceable.
//!
//! California — Cal. Civ. Code § 1953: BROAD SCOPE — voids
//! residential lease provisions waiving any of SIX categories
//! of tenant rights:
//!   (a)(1) § 1950.5 (deposit) or § 1954 (entry) rights;
//!   (a)(2) Right to assert cause of action against lessor;
//!   (a)(3) Right to notice or hearing required by law;
//!   (a)(4) Procedural rights in litigation (including jury
//!          trial waivers in disputes involving tenant rights);
//!   (a)(5) Right to have lessor exercise a duty of care to
//!          prevent personal injury or property damage where
//!          that duty is imposed by law;
//!   (a)(6) Cumulative right to bring claims under the lease
//!          (no waiver of multiple-remedy rights).
//! California is the strictest residential-protection regime;
//! ANY of the six waiver categories is automatically void in a
//! residential lease.
//!
//! Default — common law. Waiver clauses generally enforceable
//! if (a) the waiver is KNOWING and VOLUNTARY, and (b) the
//! waiver does not violate public policy of the jurisdiction.
//! Most states adopt fact-specific analysis with the trend
//! toward voiding broad waivers of statutory tenant rights.
//! Restatement (Second) of Contracts § 178 governs the public
//! policy analysis.
//!
//! Citations: N.Y. Gen. Oblig. Law § 5-321 (negligence
//! exculpatory clauses void); Cal. Civ. Code § 1953 (general
//! lease waiver void provisions); Cal. Civ. Code § 1953(a)(1)
//! (§ 1950.5 + § 1954 rights); Cal. Civ. Code § 1953(a)(2)
//! (future cause of action); Cal. Civ. Code § 1953(a)(3)
//! (notice or hearing); Cal. Civ. Code § 1953(a)(4) (procedural
//! rights — including jury trial); Cal. Civ. Code § 1953(a)(5)
//! (duty of care); Cal. Civ. Code § 1953(a)(6) (cumulative
//! remedies); Cal. Civ. Code § 1950.5 (security deposit);
//! Cal. Civ. Code § 1954 (landlord entry); Restatement (Second)
//! of Contracts § 178 (public policy unenforceability test);
//! Great N. Ins. Co. v. Interior Constr. Corp., 7 N.Y.3d 412
//! (2006) (§ 5-321 indemnity-vs-exculpation distinction).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// N.Y. Gen. Oblig. Law § 5-321 — narrow scope: voids
    /// landlord-negligence exculpatory clauses only.
    NewYork,
    /// Cal. Civ. Code § 1953 — broad scope: voids 6 categories
    /// of residential tenant rights waivers.
    California,
    /// Common-law analysis: knowing + voluntary + public policy.
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WaiverType {
    /// Clause exempting landlord from liability for negligence
    /// causing injury or property damage.
    LandlordNegligenceExculpatory,
    /// Clause waiving tenant's right to jury trial in disputes
    /// involving tenant rights.
    JuryTrialWaiver,
    /// Clause waiving tenant's right to assert future cause of
    /// action against lessor.
    FutureCauseOfActionWaiver,
    /// Clause waiving habitability or duty-of-care rights.
    HabitabilityOrDutyOfCareWaiver,
    /// Clause waiving § 1950.5 (deposit) or § 1954 (entry) rights.
    DepositOrEntryRightsWaiver,
    /// Clause waiving right to notice or hearing required by law.
    NoticeOrHearingRightsWaiver,
    /// Clause waiving cumulative remedy rights.
    CumulativeRemediesWaiver,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub waiver_type: WaiverType,
    pub lease_is_residential: bool,
    pub waiver_in_lease_agreement: bool,
    /// Default-regime factor — was tenant's consent to the
    /// waiver knowing and voluntary (e.g., negotiated lease,
    /// not adhesion contract)?
    pub tenant_knowing_and_voluntary_consent: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub waiver_enforceable: bool,
    pub statutory_void_provision_engaged: bool,
    pub public_policy_unenforceable: bool,
    /// True if waiver covered by a residential-only statute
    /// (e.g., Cal. Civ. Code § 1953).
    pub residential_only_protection_engaged: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    if !input.waiver_in_lease_agreement {
        notes.push(
            "No waiver clause present in lease agreement — no waiver-enforceability \
             analysis required. § 5-321 / § 1953 / common-law public-policy doctrines \
             apply only when the lease attempts to extinguish a right via contractual \
             waiver."
                .to_string(),
        );
        return CheckResult {
            waiver_enforceable: true,
            statutory_void_provision_engaged: false,
            public_policy_unenforceable: false,
            residential_only_protection_engaged: false,
            compliant: true,
            violations,
            citation: citation_text(),
            notes,
        };
    }

    let (
        waiver_enforceable,
        statutory_void_provision_engaged,
        public_policy_unenforceable,
        residential_only_protection_engaged,
    ) = match input.regime {
        Regime::NewYork => {
            // § 5-321 — narrow scope. Only landlord-negligence exculpatory clauses
            // are statutorily void. Applies to residential AND commercial leases.
            let exculpatory_clause =
                matches!(input.waiver_type, WaiverType::LandlordNegligenceExculpatory);
            if exculpatory_clause {
                violations.push(
                    "N.Y. Gen. Oblig. Law § 5-321 — landlord-negligence exculpatory \
                         clause is VOID as against public policy and wholly unenforceable. \
                         Applies to BOTH residential and commercial leases. Distinct from \
                         indemnification clauses (Great N. Ins. Co. v. Interior Constr. \
                         Corp., 7 N.Y.3d 412 (2006) — indemnity remains enforceable)."
                        .to_string(),
                );
                notes.push(
                    "§ 5-321 narrow-scope statute — landlord may still seek \
                         INDEMNIFICATION (separate doctrine) for liability paid to \
                         injured third parties. The void clause is only the one \
                         exempting LANDLORD from liability to tenant for landlord's own \
                         negligence."
                        .to_string(),
                );
                (false, true, true, false)
            } else {
                notes.push(format!(
                    "N.Y. Gen. Oblig. Law § 5-321 narrow scope — {:?} waiver is NOT \
                         within the statute's negligence-exculpatory scope. Common-law \
                         analysis applies; jury-trial waivers + procedural waivers may \
                         be enforceable if knowing and voluntary.",
                    input.waiver_type,
                ));
                let cl_enforceable = input.tenant_knowing_and_voluntary_consent;
                (cl_enforceable, false, !cl_enforceable, false)
            }
        }
        Regime::California => {
            // § 1953 — broad scope, residential leases only.
            if !input.lease_is_residential {
                notes.push(
                    "Cal. Civ. Code § 1953 applies only to RESIDENTIAL leases. \
                         Commercial lease waivers are analyzed under common-law \
                         contract principles + Restatement (Second) of Contracts § 178 \
                         public-policy test."
                        .to_string(),
                );
                let cl_enforceable = input.tenant_knowing_and_voluntary_consent;
                (cl_enforceable, false, !cl_enforceable, false)
            } else {
                // All six § 1953(a) waiver categories are void in residential.
                let void_category = matches!(
                    input.waiver_type,
                    WaiverType::DepositOrEntryRightsWaiver
                        | WaiverType::FutureCauseOfActionWaiver
                        | WaiverType::NoticeOrHearingRightsWaiver
                        | WaiverType::JuryTrialWaiver
                        | WaiverType::HabitabilityOrDutyOfCareWaiver
                        | WaiverType::CumulativeRemediesWaiver
                        | WaiverType::LandlordNegligenceExculpatory
                );
                if void_category {
                    let (subsection, label) = match input.waiver_type {
                        WaiverType::DepositOrEntryRightsWaiver => (
                            "§ 1953(a)(1)",
                            "§ 1950.5 (deposit) or § 1954 (entry) rights",
                        ),
                        WaiverType::FutureCauseOfActionWaiver => {
                            ("§ 1953(a)(2)", "future cause of action against lessor")
                        }
                        WaiverType::NoticeOrHearingRightsWaiver => {
                            ("§ 1953(a)(3)", "right to notice or hearing required by law")
                        }
                        WaiverType::JuryTrialWaiver => (
                            "§ 1953(a)(4)",
                            "procedural rights in litigation including jury trial",
                        ),
                        WaiverType::HabitabilityOrDutyOfCareWaiver
                        | WaiverType::LandlordNegligenceExculpatory => (
                            "§ 1953(a)(5)",
                            "duty of care to prevent personal injury or property damage",
                        ),
                        WaiverType::CumulativeRemediesWaiver => {
                            ("§ 1953(a)(6)", "cumulative remedies under the lease")
                        }
                    };
                    violations.push(format!(
                        "Cal. Civ. Code {} — residential lease waiver of {} is VOID \
                             as contrary to public policy. California § 1953 imposes the \
                             strictest residential-protection regime in the U.S.; ANY of \
                             the six § 1953(a) waiver categories is automatically void.",
                        subsection, label,
                    ));
                    (false, true, true, true)
                } else {
                    notes.push(
                        "Waiver type not within Cal. Civ. Code § 1953(a)(1)-(6) \
                             enumerated categories. Common-law analysis applies."
                            .to_string(),
                    );
                    (true, false, false, false)
                }
            }
        }
        Regime::Default => {
            // Common-law analysis: knowing + voluntary + public policy.
            let public_policy_violation = matches!(
                input.waiver_type,
                WaiverType::HabitabilityOrDutyOfCareWaiver
                    | WaiverType::LandlordNegligenceExculpatory
            );
            if public_policy_violation {
                violations.push(format!(
                    "Common-law public-policy doctrine + Restatement (Second) of \
                         Contracts § 178 — {:?} waiver in lease is unenforceable as \
                         against public policy regardless of knowing/voluntary consent. \
                         Modern majority voids broad waivers of statutory tenant rights \
                         and landlord negligence liability.",
                    input.waiver_type,
                ));
                (false, false, true, false)
            } else if !input.tenant_knowing_and_voluntary_consent {
                violations.push(
                    "Common-law analysis — waiver was not knowing or voluntary \
                         (e.g., adhesion contract without meaningful negotiation). \
                         Procedural unconscionability defense may invalidate the \
                         waiver."
                        .to_string(),
                );
                (false, false, true, false)
            } else {
                notes.push(
                    "Common-law analysis — waiver appears enforceable: not a public-\
                         policy-protected category AND tenant gave knowing + voluntary \
                         consent. Restatement (Second) of Contracts § 178 public-policy \
                         test does not engage."
                        .to_string(),
                );
                (true, false, false, false)
            }
        }
    };

    notes.push(
        "Sibling distinction: this module covers LEASE-DRAFTED WAIVER PROVISIONS — when \
         a contract clause attempting to extinguish a right is void by statute or \
         public policy. Related modules: `landlord_harassment` (affirmative-harassment \
         penalties), `habitability_remedies` (post-delivery habitability obligations), \
         `quiet_enjoyment` (general post-delivery covenant), `plain_language_lease` \
         (statutory clarity requirement). New York § 5-321 has the narrowest scope \
         (only landlord-negligence exculpatory clauses void); California § 1953 has \
         the broadest residential scope (6 categories of tenant rights); Default \
         common-law analyzes knowing/voluntary consent + Restatement § 178 public \
         policy."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        waiver_enforceable,
        statutory_void_provision_engaged,
        public_policy_unenforceable,
        residential_only_protection_engaged,
        compliant,
        violations,
        citation: citation_text(),
        notes,
    }
}

fn citation_text() -> &'static str {
    "N.Y. Gen. Oblig. Law § 5-321 (negligence exculpatory clauses void); Cal. Civ. \
     Code § 1953 (general lease waiver void provisions); Cal. Civ. Code § 1953(a)(1) \
     (§ 1950.5 + § 1954 rights); Cal. Civ. Code § 1953(a)(2) (future cause of action); \
     Cal. Civ. Code § 1953(a)(3) (notice or hearing); Cal. Civ. Code § 1953(a)(4) \
     (procedural rights — jury trial); Cal. Civ. Code § 1953(a)(5) (duty of care); \
     Cal. Civ. Code § 1953(a)(6) (cumulative remedies); Cal. Civ. Code § 1950.5 \
     (security deposit); Cal. Civ. Code § 1954 (landlord entry); Restatement (Second) \
     of Contracts § 178 (public policy unenforceability test); Great N. Ins. Co. v. \
     Interior Constr. Corp., 7 N.Y.3d 412 (2006) (§ 5-321 indemnity-vs-exculpation \
     distinction)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime, waiver_type: WaiverType) -> Input {
        Input {
            regime,
            waiver_type,
            lease_is_residential: true,
            waiver_in_lease_agreement: true,
            tenant_knowing_and_voluntary_consent: true,
        }
    }

    // ── No waiver clause ──────────────────────────────────────

    #[test]
    fn no_waiver_clause_no_analysis() {
        let mut b = input(Regime::NewYork, WaiverType::LandlordNegligenceExculpatory);
        b.waiver_in_lease_agreement = false;
        let r = check(&b);
        assert!(r.waiver_enforceable);
        assert!(r.compliant);
    }

    // ── New York § 5-321 (narrow: negligence exculpatory only) ─

    #[test]
    fn ny_landlord_negligence_exculpatory_void() {
        let r = check(&input(
            Regime::NewYork,
            WaiverType::LandlordNegligenceExculpatory,
        ));
        assert!(!r.waiver_enforceable);
        assert!(r.statutory_void_provision_engaged);
        assert!(r.public_policy_unenforceable);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 5-321")));
    }

    #[test]
    fn ny_negligence_exculpatory_applies_to_commercial() {
        let mut b = input(Regime::NewYork, WaiverType::LandlordNegligenceExculpatory);
        b.lease_is_residential = false;
        let r = check(&b);
        // § 5-321 applies to BOTH residential and commercial leases.
        assert!(!r.waiver_enforceable);
        assert!(r.statutory_void_provision_engaged);
    }

    #[test]
    fn ny_jury_trial_waiver_outside_5_321_scope() {
        let r = check(&input(Regime::NewYork, WaiverType::JuryTrialWaiver));
        // Outside § 5-321 narrow scope. With knowing + voluntary consent → enforceable.
        assert!(r.waiver_enforceable);
        assert!(!r.statutory_void_provision_engaged);
    }

    #[test]
    fn ny_jury_trial_waiver_without_consent_unenforceable() {
        let mut b = input(Regime::NewYork, WaiverType::JuryTrialWaiver);
        b.tenant_knowing_and_voluntary_consent = false;
        let r = check(&b);
        assert!(!r.waiver_enforceable);
        // Statutory void not engaged; common-law analysis blocks.
        assert!(!r.statutory_void_provision_engaged);
        assert!(r.public_policy_unenforceable);
    }

    // ── California § 1953 (broad residential scope) ──────────

    #[test]
    fn ca_residential_jury_trial_waiver_void() {
        let r = check(&input(Regime::California, WaiverType::JuryTrialWaiver));
        assert!(!r.waiver_enforceable);
        assert!(r.statutory_void_provision_engaged);
        assert!(r.residential_only_protection_engaged);
        assert!(r.violations.iter().any(|v| v.contains("§ 1953(a)(4)")));
    }

    #[test]
    fn ca_residential_deposit_rights_waiver_void() {
        let r = check(&input(
            Regime::California,
            WaiverType::DepositOrEntryRightsWaiver,
        ));
        assert!(!r.waiver_enforceable);
        assert!(r.violations.iter().any(|v| v.contains("§ 1953(a)(1)")));
    }

    #[test]
    fn ca_residential_future_cause_of_action_void() {
        let r = check(&input(
            Regime::California,
            WaiverType::FutureCauseOfActionWaiver,
        ));
        assert!(!r.waiver_enforceable);
        assert!(r.violations.iter().any(|v| v.contains("§ 1953(a)(2)")));
    }

    #[test]
    fn ca_residential_notice_hearing_void() {
        let r = check(&input(
            Regime::California,
            WaiverType::NoticeOrHearingRightsWaiver,
        ));
        assert!(!r.waiver_enforceable);
        assert!(r.violations.iter().any(|v| v.contains("§ 1953(a)(3)")));
    }

    #[test]
    fn ca_residential_habitability_void() {
        let r = check(&input(
            Regime::California,
            WaiverType::HabitabilityOrDutyOfCareWaiver,
        ));
        assert!(!r.waiver_enforceable);
        assert!(r.violations.iter().any(|v| v.contains("§ 1953(a)(5)")));
    }

    #[test]
    fn ca_residential_cumulative_remedies_void() {
        let r = check(&input(
            Regime::California,
            WaiverType::CumulativeRemediesWaiver,
        ));
        assert!(!r.waiver_enforceable);
        assert!(r.violations.iter().any(|v| v.contains("§ 1953(a)(6)")));
    }

    #[test]
    fn ca_commercial_lease_outside_1953_scope() {
        let mut b = input(Regime::California, WaiverType::JuryTrialWaiver);
        b.lease_is_residential = false;
        let r = check(&b);
        // § 1953 applies only to residential. Commercial waiver enforceable with consent.
        assert!(r.waiver_enforceable);
        assert!(!r.statutory_void_provision_engaged);
        assert!(!r.residential_only_protection_engaged);
    }

    // ── Default common-law analysis ──────────────────────────

    #[test]
    fn default_knowing_voluntary_enforceable() {
        let r = check(&input(Regime::Default, WaiverType::JuryTrialWaiver));
        // Common-law analysis; not a public-policy-protected category; knowing consent.
        assert!(r.waiver_enforceable);
        assert!(!r.public_policy_unenforceable);
    }

    #[test]
    fn default_not_knowing_voluntary_unenforceable() {
        let mut b = input(Regime::Default, WaiverType::JuryTrialWaiver);
        b.tenant_knowing_and_voluntary_consent = false;
        let r = check(&b);
        assert!(!r.waiver_enforceable);
        assert!(r.public_policy_unenforceable);
    }

    #[test]
    fn default_habitability_waiver_void_regardless_of_consent() {
        let r = check(&input(
            Regime::Default,
            WaiverType::HabitabilityOrDutyOfCareWaiver,
        ));
        // Public-policy violation even with knowing/voluntary consent.
        assert!(!r.waiver_enforceable);
        assert!(r.public_policy_unenforceable);
    }

    #[test]
    fn default_negligence_exculpatory_void_regardless_of_consent() {
        let r = check(&input(
            Regime::Default,
            WaiverType::LandlordNegligenceExculpatory,
        ));
        assert!(!r.waiver_enforceable);
        assert!(r.public_policy_unenforceable);
    }

    // ── Multi-regime invariants ──────────────────────────────

    #[test]
    fn only_california_1953_engages_residential_only_protection() {
        for regime in [Regime::NewYork, Regime::California, Regime::Default] {
            let r = check(&input(regime, WaiverType::JuryTrialWaiver));
            let expected = matches!(regime, Regime::California);
            assert_eq!(
                r.residential_only_protection_engaged, expected,
                "{:?}",
                regime
            );
        }
    }

    #[test]
    fn ny_5_321_applies_to_both_residential_and_commercial_invariant() {
        for residential in [true, false] {
            let mut b = input(Regime::NewYork, WaiverType::LandlordNegligenceExculpatory);
            b.lease_is_residential = residential;
            let r = check(&b);
            // § 5-321 voids exculpatory clauses regardless of residential/commercial.
            assert!(!r.waiver_enforceable, "residential={}", residential);
        }
    }

    #[test]
    fn ca_1953_applies_only_to_residential_invariant() {
        let mut b_res = input(Regime::California, WaiverType::JuryTrialWaiver);
        b_res.lease_is_residential = true;
        let mut b_com = input(Regime::California, WaiverType::JuryTrialWaiver);
        b_com.lease_is_residential = false;

        let r_res = check(&b_res);
        let r_com = check(&b_com);

        assert!(!r_res.waiver_enforceable); // residential — § 1953 voids
        assert!(r_com.waiver_enforceable); // commercial — outside § 1953 scope
    }

    #[test]
    fn ca_residential_all_seven_waiver_types_void() {
        // All 7 waiver enum types are void in CA residential lease.
        let waiver_types = [
            WaiverType::LandlordNegligenceExculpatory,
            WaiverType::JuryTrialWaiver,
            WaiverType::FutureCauseOfActionWaiver,
            WaiverType::HabitabilityOrDutyOfCareWaiver,
            WaiverType::DepositOrEntryRightsWaiver,
            WaiverType::NoticeOrHearingRightsWaiver,
            WaiverType::CumulativeRemediesWaiver,
        ];
        for waiver_type in waiver_types.iter() {
            let r = check(&input(Regime::California, *waiver_type));
            assert!(!r.waiver_enforceable, "{:?}", waiver_type);
        }
    }

    #[test]
    fn default_public_policy_voids_regardless_of_consent_truth_table() {
        // 4-cell truth table: (waiver_type × consent).
        let cells = [
            (WaiverType::LandlordNegligenceExculpatory, true, false),
            (WaiverType::LandlordNegligenceExculpatory, false, false),
            (WaiverType::JuryTrialWaiver, true, true), // not protected; consent OK
            (WaiverType::JuryTrialWaiver, false, false), // not consented
        ];
        for (waiver_type, consent, expected_enforceable) in cells.iter() {
            let mut b = input(Regime::Default, *waiver_type);
            b.tenant_knowing_and_voluntary_consent = *consent;
            let r = check(&b);
            assert_eq!(
                r.waiver_enforceable, *expected_enforceable,
                "waiver={:?} consent={}",
                waiver_type, consent
            );
        }
    }

    // ── Citation + sibling note ──────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&input(Regime::California, WaiverType::JuryTrialWaiver));
        assert!(r.citation.contains("§ 5-321"));
        assert!(r.citation.contains("§ 1953"));
        assert!(r.citation.contains("§ 1953(a)(1)"));
        assert!(r.citation.contains("§ 1953(a)(2)"));
        assert!(r.citation.contains("§ 1953(a)(3)"));
        assert!(r.citation.contains("§ 1953(a)(4)"));
        assert!(r.citation.contains("§ 1953(a)(5)"));
        assert!(r.citation.contains("§ 1953(a)(6)"));
        assert!(r.citation.contains("§ 1950.5"));
        assert!(r.citation.contains("§ 1954"));
        assert!(r
            .citation
            .contains("Restatement (Second) of Contracts § 178"));
        assert!(r.citation.contains("Great N. Ins. Co."));
        assert!(r.citation.contains("7 N.Y.3d 412"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::California, WaiverType::JuryTrialWaiver));
        assert!(
            r.notes.iter().any(|n| n.contains("landlord_harassment")
                && n.contains("habitability_remedies")
                && n.contains("quiet_enjoyment")
                && n.contains("plain_language_lease")
                && n.contains("LEASE-DRAFTED WAIVER PROVISIONS")),
            "sibling distinction note must reference related modules + lease-drafted-waiver focus"
        );
    }

    #[test]
    fn ny_indemnity_distinction_note_when_5_321_engages() {
        let r = check(&input(
            Regime::NewYork,
            WaiverType::LandlordNegligenceExculpatory,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("INDEMNIFICATION")),
            "§ 5-321 indemnity-vs-exculpation distinction note must be present"
        );
    }
}
