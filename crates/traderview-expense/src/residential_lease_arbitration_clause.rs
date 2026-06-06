//! Residential lease pre-dispute arbitration clause enforceability
//! — when can a landlord include a binding arbitration provision in
//! a residential lease that requires the tenant to arbitrate
//! disputes (habitability, eviction, deposit, fee challenges)
//! rather than file court action?
//!
//! Distinct from `lease_waiver_enforceability` (general lease
//! waiver framework — N.Y. GOL § 5-321 + Cal. Civ. Code § 1953
//! general waiver analysis) and `prevailing_party_attorney_fees`
//! (fee-shifting analysis). This module focuses ONLY on the
//! arbitration / class-action waiver question.
//!
//! Three regimes:
//!
//! **California — Cal. Civ. Code § 1953(a)(4)**. Historically voids
//! residential lease provisions where the tenant waives procedural
//! rights to litigate (jury trial, unlawful detainer adjudication,
//! habitability claims). Recent federal court ruling Brooks v.
//! Greystar Real Estate Partners (S.D. Cal. 2024) has eroded this
//! interpretation under Federal Arbitration Act (FAA) preemption —
//! arbitration clauses now defended on FAA savings clause grounds
//! when fair, voluntary, and meeting federal arbitration standards.
//! Status: contested + evolving.
//!
//! **New Jersey — limits arbitration in residential leases under
//! Atalese v. U.S. Legal Services Group (220 N.J. 220, 2014)**.
//! Arbitration provision must explicitly waive the right to a
//! judicial forum in clear, unambiguous language. Boilerplate
//! arbitration clauses fail Atalese.
//!
//! **Default — Federal Arbitration Act 9 U.S.C. § 1 et seq.**. FAA
//! generally enforces arbitration clauses. § 2 makes them "valid,
//! irrevocable, and enforceable, save upon such grounds as exist at
//! law or in equity for the revocation of any contract." State law
//! defenses (unconscionability, lack of consideration, fraud) apply
//! through FAA § 2 savings clause; states may NOT discriminate
//! against arbitration agreements specifically (AT&T Mobility v.
//! Concepcion, 563 U.S. 333 (2011)). § 4 — court may compel
//! arbitration upon party's motion.
//!
//! **§ 162(q) sexual harassment carve-out** (paired federal
//! statute — Speak Out Act 117 Stat. 2192) makes pre-dispute
//! arbitration unenforceable for sexual harassment / sexual
//! assault claims. Out of scope of this module but flagged in
//! citation.
//!
//! Citations: Cal. Civ. Code § 1953(a)(4) (CA general waiver-of-
//! procedural-rights void rule); Brooks v. Greystar Real Estate
//! Partners (S.D. Cal. 2024 — FAA preemption erosion); N.J.S.A.
//! 2A:23B-1 et seq. (NJ Revised Uniform Arbitration Act); Atalese
//! v. U.S. Legal Services Group, 220 N.J. 220 (2014) (NJ explicit-
//! waiver requirement); 9 U.S.C. §§ 1, 2, 4 (Federal Arbitration
//! Act); AT&T Mobility v. Concepcion, 563 U.S. 333 (2011) (FAA
//! preemption of state anti-arbitration rules); Epic Systems v.
//! Lewis, 584 U.S. 497 (2018) (arbitration clauses enforce
//! class-action waivers); Speak Out Act of 2022 (117 Stat. 2192 —
//! sexual harassment / abuse pre-dispute arbitration ban).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisputeType {
    /// Habitability / repair / warranty of habitability claim.
    Habitability,
    /// Unlawful detainer / eviction action.
    UnlawfulDetainer,
    /// Security deposit refund / damage deduction.
    SecurityDeposit,
    /// Sexual harassment / sexual assault claim. Speak Out Act
    /// federal carve-out makes pre-dispute arbitration
    /// unenforceable.
    SexualHarassmentOrAssault,
    /// Other landlord-tenant dispute (rent dispute, lease term
    /// interpretation, etc.).
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArbitrationClauseInput {
    pub regime: Regime,
    pub dispute_type: DisputeType,
    /// Whether the lease includes an arbitration clause.
    pub arbitration_clause_in_lease: bool,
    /// NJ-only: whether the clause explicitly waives judicial forum
    /// in clear, unambiguous language (Atalese requirement).
    pub nj_explicit_judicial_waiver_language: bool,
    /// Whether the clause includes a class-action waiver. Generally
    /// enforceable under Epic Systems but state laws may differ.
    pub class_action_waiver: bool,
    /// Whether the clause appears unconscionable (procedural or
    /// substantive). FAA § 2 savings clause permits this defense.
    pub unconscionable: bool,
    /// Whether the lease was signed under duress or with material
    /// misrepresentation. Also a FAA § 2 savings clause defense.
    pub duress_or_misrepresentation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ArbitrationClauseResult {
    pub arbitration_enforceable: bool,
    pub class_action_waiver_enforceable: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &ArbitrationClauseInput) -> ArbitrationClauseResult {
    let mut notes: Vec<String> = Vec::new();

    if !input.arbitration_clause_in_lease {
        notes.push(
            "no arbitration clause in lease — § 9 U.S.C. § 2 FAA enforcement does not apply; standard judicial forum available"
                .to_string(),
        );
        return ArbitrationClauseResult {
            arbitration_enforceable: false,
            class_action_waiver_enforceable: false,
            citation: citation_for(input.regime),
            notes,
        };
    }

    if matches!(input.dispute_type, DisputeType::SexualHarassmentOrAssault) {
        notes.push(
            "Speak Out Act of 2022 (117 Stat. 2192) — pre-dispute arbitration of sexual harassment or sexual assault claims is UNENFORCEABLE regardless of state regime; § 162(q) tax restriction also applies to settlements with NDAs"
                .to_string(),
        );
        return ArbitrationClauseResult {
            arbitration_enforceable: false,
            class_action_waiver_enforceable: false,
            citation: citation_for(input.regime),
            notes,
        };
    }

    if input.unconscionable {
        notes.push(
            "9 U.S.C. § 2 savings clause — clause invalidated on unconscionability grounds; AT&T Mobility v. Concepcion (563 U.S. 333, 2011) permits state-law unconscionability defense applied neutrally"
                .to_string(),
        );
        return ArbitrationClauseResult {
            arbitration_enforceable: false,
            class_action_waiver_enforceable: false,
            citation: citation_for(input.regime),
            notes,
        };
    }

    if input.duress_or_misrepresentation {
        notes.push(
            "9 U.S.C. § 2 savings clause — clause invalidated on duress / misrepresentation grounds"
                .to_string(),
        );
        return ArbitrationClauseResult {
            arbitration_enforceable: false,
            class_action_waiver_enforceable: false,
            citation: citation_for(input.regime),
            notes,
        };
    }

    let (enforceable, class_action_enforceable) = match input.regime {
        Regime::California => {
            let traditional_void = matches!(
                input.dispute_type,
                DisputeType::Habitability
                    | DisputeType::UnlawfulDetainer
                    | DisputeType::SecurityDeposit
            );
            if traditional_void {
                notes.push(
                    "Cal. Civ. Code § 1953(a)(4) traditional rule — voids residential lease arbitration provisions for procedural rights (habitability / unlawful detainer / deposit); BUT Brooks v. Greystar Real Estate Partners (S.D. Cal. 2024) eroded this interpretation under FAA preemption"
                        .to_string(),
                );
                notes.push(
                    "CONTESTED status — FAA savings clause analysis required; landlord must show clause is fair, voluntary, and meeting federal arbitration standards"
                        .to_string(),
                );
                (true, input.class_action_waiver)
            } else {
                notes.push(
                    "Cal. Civ. Code § 1953(a)(4) — non-procedural-rights disputes generally arbitrable under FAA absent contestation"
                        .to_string(),
                );
                (true, input.class_action_waiver)
            }
        }
        Regime::NewJersey => {
            if !input.nj_explicit_judicial_waiver_language {
                notes.push(
                    "Atalese v. U.S. Legal Services Group (220 N.J. 220, 2014) — NJ arbitration provision must EXPLICITLY waive judicial forum in clear, unambiguous language; boilerplate clauses fail Atalese"
                        .to_string(),
                );
                (false, false)
            } else {
                notes.push(
                    "Atalese standard satisfied — explicit judicial-forum waiver language present; clause enforceable under N.J.S.A. 2A:23B-1 et seq."
                        .to_string(),
                );
                (true, input.class_action_waiver)
            }
        }
        Regime::Default => {
            notes.push(
                "9 U.S.C. § 2 FAA general rule — arbitration clauses valid, irrevocable, and enforceable; state defenses available through FAA savings clause but states may not discriminate against arbitration agreements specifically (Concepcion)"
                    .to_string(),
            );
            (true, input.class_action_waiver)
        }
    };

    if input.class_action_waiver && class_action_enforceable {
        notes.push(
            "Epic Systems v. Lewis (584 U.S. 497, 2018) — arbitration clauses may enforce class-action waivers under FAA"
                .to_string(),
        );
    }

    ArbitrationClauseResult {
        arbitration_enforceable: enforceable,
        class_action_waiver_enforceable: class_action_enforceable,
        citation: citation_for(input.regime),
        notes,
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Civ. Code § 1953(a)(4); Brooks v. Greystar Real Estate Partners (S.D. Cal. 2024); 9 U.S.C. §§ 1/2/4; AT&T Mobility v. Concepcion (563 U.S. 333, 2011); Epic Systems v. Lewis (584 U.S. 497, 2018); Speak Out Act of 2022 (117 Stat. 2192)",
        Regime::NewJersey => "N.J.S.A. 2A:23B-1 et seq.; Atalese v. U.S. Legal Services Group, 220 N.J. 220 (2014); 9 U.S.C. §§ 1/2/4; Concepcion; Epic Systems; Speak Out Act",
        Regime::Default => "9 U.S.C. §§ 1/2/4 (Federal Arbitration Act); AT&T Mobility v. Concepcion (563 U.S. 333, 2011); Epic Systems v. Lewis (584 U.S. 497, 2018); Speak Out Act of 2022 (117 Stat. 2192)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> ArbitrationClauseInput {
        ArbitrationClauseInput {
            regime: Regime::California,
            dispute_type: DisputeType::Habitability,
            arbitration_clause_in_lease: true,
            nj_explicit_judicial_waiver_language: false,
            class_action_waiver: true,
            unconscionable: false,
            duress_or_misrepresentation: false,
        }
    }

    fn nj_base() -> ArbitrationClauseInput {
        ArbitrationClauseInput {
            regime: Regime::NewJersey,
            dispute_type: DisputeType::Habitability,
            arbitration_clause_in_lease: true,
            nj_explicit_judicial_waiver_language: true,
            class_action_waiver: true,
            unconscionable: false,
            duress_or_misrepresentation: false,
        }
    }

    fn default_base() -> ArbitrationClauseInput {
        ArbitrationClauseInput {
            regime: Regime::Default,
            dispute_type: DisputeType::Habitability,
            arbitration_clause_in_lease: true,
            nj_explicit_judicial_waiver_language: false,
            class_action_waiver: true,
            unconscionable: false,
            duress_or_misrepresentation: false,
        }
    }

    #[test]
    fn no_arbitration_clause_no_enforcement() {
        let mut i = default_base();
        i.arbitration_clause_in_lease = false;
        let r = check(&i);
        assert!(!r.arbitration_enforceable);
        assert!(!r.class_action_waiver_enforceable);
        assert!(r.notes.iter().any(|n| n.contains("no arbitration clause")));
    }

    #[test]
    fn sexual_harassment_pre_dispute_arbitration_unenforceable_universal() {
        for regime in [Regime::California, Regime::NewJersey, Regime::Default] {
            let mut i = default_base();
            i.regime = regime;
            i.dispute_type = DisputeType::SexualHarassmentOrAssault;
            let r = check(&i);
            assert!(
                !r.arbitration_enforceable,
                "Speak Out Act bars arbitration in all regimes"
            );
            assert!(r.notes.iter().any(|n| n.contains("Speak Out Act")));
        }
    }

    #[test]
    fn unconscionability_voids_arbitration_universal() {
        let mut i = default_base();
        i.unconscionable = true;
        let r = check(&i);
        assert!(!r.arbitration_enforceable);
        assert!(r.notes.iter().any(|n| n.contains("Concepcion")));
    }

    #[test]
    fn duress_or_misrepresentation_voids_arbitration() {
        let mut i = default_base();
        i.duress_or_misrepresentation = true;
        let r = check(&i);
        assert!(!r.arbitration_enforceable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("duress / misrepresentation")));
    }

    #[test]
    fn ca_habitability_now_contested_under_brooks() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("Brooks v. Greystar")));
        assert!(r.notes.iter().any(|n| n.contains("CONTESTED status")));
    }

    #[test]
    fn ca_unlawful_detainer_traditional_section_1953_void() {
        let mut i = ca_base();
        i.dispute_type = DisputeType::UnlawfulDetainer;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1953(a)(4) traditional rule")));
    }

    #[test]
    fn ca_security_deposit_traditional_section_1953_void() {
        let mut i = ca_base();
        i.dispute_type = DisputeType::SecurityDeposit;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1953(a)(4) traditional rule")));
    }

    #[test]
    fn ca_other_dispute_arbitrable_under_faa() {
        let mut i = ca_base();
        i.dispute_type = DisputeType::Other;
        let r = check(&i);
        assert!(r.arbitration_enforceable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("non-procedural-rights disputes generally arbitrable")));
    }

    #[test]
    fn nj_explicit_judicial_waiver_satisfies_atalese() {
        let r = check(&nj_base());
        assert!(r.arbitration_enforceable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Atalese standard satisfied")));
    }

    #[test]
    fn nj_missing_explicit_waiver_violates_atalese() {
        let mut i = nj_base();
        i.nj_explicit_judicial_waiver_language = false;
        let r = check(&i);
        assert!(!r.arbitration_enforceable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Atalese") && n.contains("EXPLICITLY waive judicial forum")));
    }

    #[test]
    fn default_faa_general_enforceability() {
        let r = check(&default_base());
        assert!(r.arbitration_enforceable);
        assert!(r.notes.iter().any(
            |n| n.contains("9 U.S.C. § 2") && n.contains("valid, irrevocable, and enforceable")
        ));
    }

    #[test]
    fn class_action_waiver_enforceable_under_epic_systems() {
        let r = check(&default_base());
        assert!(r.class_action_waiver_enforceable);
        assert!(r.notes.iter().any(|n| n.contains("Epic Systems v. Lewis")));
    }

    #[test]
    fn no_class_action_waiver_no_enforcement_note() {
        let mut i = default_base();
        i.class_action_waiver = false;
        let r = check(&i);
        assert!(!r.class_action_waiver_enforceable);
        let epic_notes: Vec<_> = r
            .notes
            .iter()
            .filter(|n| n.contains("Epic Systems"))
            .collect();
        assert!(epic_notes.is_empty());
    }

    #[test]
    fn citation_california_pins_brooks_and_concepcion_and_epic() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§ 1953(a)(4)"));
        assert!(r.citation.contains("Brooks v. Greystar"));
        assert!(r.citation.contains("S.D. Cal. 2024"));
        assert!(r.citation.contains("9 U.S.C. §§ 1/2/4"));
        assert!(r.citation.contains("Concepcion"));
        assert!(r.citation.contains("Epic Systems"));
        assert!(r.citation.contains("Speak Out Act"));
    }

    #[test]
    fn citation_newjersey_pins_atalese_and_state_arbitration_act() {
        let r = check(&nj_base());
        assert!(r.citation.contains("N.J.S.A. 2A:23B-1"));
        assert!(r.citation.contains("Atalese"));
        assert!(r.citation.contains("220 N.J. 220 (2014)"));
    }

    #[test]
    fn citation_default_pins_faa_concepcion_epic_speak_out() {
        let r = check(&default_base());
        assert!(r.citation.contains("9 U.S.C. §§ 1/2/4"));
        assert!(r.citation.contains("Federal Arbitration Act"));
        assert!(r.citation.contains("Concepcion"));
        assert!(r.citation.contains("Epic Systems"));
        assert!(r.citation.contains("Speak Out Act"));
    }

    #[test]
    fn sexual_harassment_disables_class_action_waiver_too() {
        let mut i = default_base();
        i.dispute_type = DisputeType::SexualHarassmentOrAssault;
        let r = check(&i);
        assert!(!r.class_action_waiver_enforceable);
    }

    #[test]
    fn unconscionability_disables_class_action_waiver() {
        let mut i = default_base();
        i.unconscionable = true;
        let r = check(&i);
        assert!(!r.class_action_waiver_enforceable);
    }

    #[test]
    fn no_clause_in_lease_disables_class_action_waiver() {
        let mut i = default_base();
        i.arbitration_clause_in_lease = false;
        let r = check(&i);
        assert!(!r.class_action_waiver_enforceable);
    }

    #[test]
    fn ca_habitability_arbitrable_post_brooks_but_marked_contested() {
        let r = check(&ca_base());
        assert!(r.arbitration_enforceable);
        assert!(r.notes.iter().any(|n| n.contains("CONTESTED")));
    }

    #[test]
    fn nj_unique_atalese_requirement_invariant() {
        let mut i_nj_missing = nj_base();
        i_nj_missing.nj_explicit_judicial_waiver_language = false;
        let r_nj = check(&i_nj_missing);
        assert!(!r_nj.arbitration_enforceable);

        for regime in [Regime::California, Regime::Default] {
            let mut i = nj_base();
            i.regime = regime;
            i.nj_explicit_judicial_waiver_language = false;
            i.dispute_type = DisputeType::Other;
            let r = check(&i);
            assert!(
                r.arbitration_enforceable,
                "regime {:?} does not require Atalese explicit waiver",
                regime
            );
        }
    }

    #[test]
    fn five_dispute_types_routed_correctly() {
        for dt in [
            DisputeType::Habitability,
            DisputeType::UnlawfulDetainer,
            DisputeType::SecurityDeposit,
            DisputeType::Other,
        ] {
            let mut i = default_base();
            i.dispute_type = dt;
            let r = check(&i);
            assert!(
                r.arbitration_enforceable,
                "dispute type {:?} should be arbitrable in Default regime",
                dt
            );
        }

        let mut i_harass = default_base();
        i_harass.dispute_type = DisputeType::SexualHarassmentOrAssault;
        let r_harass = check(&i_harass);
        assert!(!r_harass.arbitration_enforceable);
    }

    #[test]
    fn ca_traditional_void_categories_three_specific_disputes_only() {
        let traditional_void_categories = [
            DisputeType::Habitability,
            DisputeType::UnlawfulDetainer,
            DisputeType::SecurityDeposit,
        ];
        for dt in traditional_void_categories {
            let mut i = ca_base();
            i.dispute_type = dt;
            let r = check(&i);
            assert!(r
                .notes
                .iter()
                .any(|n| n.contains("§ 1953(a)(4) traditional rule")));
        }

        let mut i_other = ca_base();
        i_other.dispute_type = DisputeType::Other;
        let r_other = check(&i_other);
        assert!(r_other
            .notes
            .iter()
            .any(|n| n.contains("non-procedural-rights disputes")));
    }

    #[test]
    fn note_speak_out_act_references_117_stat_2192() {
        let mut i = default_base();
        i.dispute_type = DisputeType::SexualHarassmentOrAssault;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("117 Stat. 2192")));
    }

    #[test]
    fn note_unconscionability_references_concepcion() {
        let mut i = default_base();
        i.unconscionable = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("AT&T Mobility v. Concepcion")));
    }

    #[test]
    fn ca_traditional_void_pathway_includes_brooks_caveat() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("Brooks v. Greystar")));
    }

    #[test]
    fn nj_pre_atalese_boilerplate_clause_unenforceable() {
        let mut i = nj_base();
        i.nj_explicit_judicial_waiver_language = false;
        let r = check(&i);
        assert!(!r.arbitration_enforceable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("boilerplate clauses fail Atalese")));
    }

    #[test]
    fn class_action_waiver_voids_when_underlying_arbitration_voided() {
        let mut i = nj_base();
        i.nj_explicit_judicial_waiver_language = false;
        let r = check(&i);
        assert!(!r.arbitration_enforceable);
        assert!(!r.class_action_waiver_enforceable);
    }
}
