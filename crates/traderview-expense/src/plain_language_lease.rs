//! State plain-language lease / consumer-contract requirements.
//!
//! Four states have enacted statutes specifically requiring
//! residential leases to be written in clear, readable language
//! understandable to the average tenant. The 1970s consumer-
//! protection movement produced the original wave: CT (1979,
//! pioneer), NY (1978-effective), NJ (1980), PA (1993). The 46
//! other states + DC rely on general unconscionability /
//! contract-of-adhesion doctrines.
//!
//! Five regimes:
//!
//! `NewYorkClearCoherent50DollarPenalty`: NY only. N.Y. GOL
//! § 5-702 (eff. 1978-11-01). Every written agreement for
//! residential lease (and other consumer transactions) entered
//! into after 1978-11-01 must be written in a "clear and coherent
//! manner" using "words with common and everyday meanings" and
//! appropriately divided and captioned by its sections. Violation
//! remedy: actual damages + **$50 statutory penalty**. Good-faith-
//! compliance defense available; no action after both parties have
//! fully performed.
//!
//! `NewJersey100DollarMinimumPlusAttorneyFees`: NJ only. N.J.S.A.
//! 56:12-1 et seq. (Plain Language Act, eff. 1981). Consumer
//! contracts must be written in a "simple, clear, understandable
//! and easily readable way." Violation remedy: greater of **$100
//! statutory minimum** OR actual damages, plus reasonable attorney
//! fees and court costs. Stronger remedy than NY.
//!
//! `PennsylvaniaPlainLanguageNineTests`: PA only. 73 P.S. § 2201
//! et seq. (Plain Language Consumer Contract Act, eff. 1993).
//! Substantial compliance requires meeting **nine objective tests**:
//! (1) short sentences and paragraphs; (2) everyday words;
//! (3) personal pronouns / actual party names; (4) simple and
//! active verb forms; (5) readable type size; (6) ink contrasting
//! with paper; (7) captioned section headings in boldface; (8)
//! layout / spacing separating paragraphs; (9) clear and coherent
//! organization. Residential leases explicitly covered. Attorney
//! General preapproval procedure available.
//!
//! `ConnecticutDescriptiveReadability`: CT only. Conn. Gen. Stat.
//! § 42-152 (eff. 1979, the oldest U.S. plain-language statute).
//! Hybrid: applies BOTH a descriptive standard ("plain English")
//! and an objective readability standard (Flesch Reading Ease /
//! similar metric).
//!
//! `NoStatewidePlainLanguageRequirement`: 46 other states + DC.
//! Reliance on common-law unconscionability + contract-of-adhesion
//! doctrines; courts may strike confusing terms but no statutory
//! penalty.
//!
//! Sources:
//! [N.Y. GOL § 5-702 — New York Senate](https://www.nysenate.gov/legislation/laws/GOB/5-702),
//! [N.J.S.A. 56:12-1 — NJ Division of Consumer Affairs PDF](https://www.nj.gov/dca/codes/publications/pdf_lti/pln_lang_rev_law.pdf),
//! [73 P.S. § 2201 et seq. — PA General Assembly](https://www.legis.state.pa.us/WU01/LI/LI/US/PDF/2006/0/0176..PDF),
//! [37 Pa. Code Chapter 307 — Plain Language Consumer Contract Preapproval](https://www.pacodeandbulletin.gov/Display/pacode?file=%2Fsecure%2Fpacode%2Fdata%2F037%2Fchapter307%2Fchap307toc.html&d=).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlainLanguageRegime {
    NewYorkClearCoherent50DollarPenalty,
    NewJersey100DollarMinimumPlusAttorneyFees,
    PennsylvaniaPlainLanguageNineTests,
    ConnecticutDescriptiveReadability,
    NoStatewidePlainLanguageRequirement,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: PlainLanguageRegime,
    /// Statutory minimum / fixed penalty in dollars (NY $50, NJ $100).
    pub statutory_penalty_dollars: i64,
    /// True if statute provides reasonable attorney fees as a remedy
    /// in addition to damages (NJ).
    pub attorney_fees_recoverable: bool,
    /// True if the statute carries a good-faith-compliance defense
    /// (NY).
    pub good_faith_defense_available: bool,
    /// True if the statute requires meeting an objective multi-test
    /// readability checklist (PA 9-test; CT readability score).
    pub objective_readability_required: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: PlainLanguageRegime,
    statutory_penalty_dollars: i64,
    attorney_fees_recoverable: bool,
    good_faith_defense_available: bool,
    objective_readability_required: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        statutory_penalty_dollars,
        attorney_fees_recoverable,
        good_faith_defense_available,
        objective_readability_required,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use PlainLanguageRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            NewYorkClearCoherent50DollarPenalty,
            50,
            false,
            true,
            false,
            "N.Y. GOL § 5-702 (eff. 1978-11-01) — every residential lease must be written in a clear and coherent manner using words with common and everyday meanings and appropriately divided and captioned by sections; remedy = actual damages + $50 statutory penalty; good-faith-compliance defense available; no action after both parties have fully performed",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJersey100DollarMinimumPlusAttorneyFees,
            100,
            true,
            false,
            false,
            "N.J.S.A. 56:12-1 et seq. (Plain Language Act, eff. 1981) — consumer contracts must be written in a simple, clear, understandable and easily readable way; remedy = greater of $100 statutory minimum OR actual damages, plus reasonable attorney fees and court costs",
        ),
    );

    m.insert(
        "PA",
        rule(
            PennsylvaniaPlainLanguageNineTests,
            0,
            true,
            false,
            true,
            "73 P.S. § 2201 et seq. (Plain Language Consumer Contract Act, eff. 1993) — residential leases covered; substantial compliance requires meeting 9 objective tests (short sentences + everyday words + personal pronouns/party names + simple active verbs + readable type size + contrasting ink + boldface captions + layout/spacing + clear coherent organization); AG preapproval procedure available; private remedy + attorney fees",
        ),
    );

    m.insert(
        "CT",
        rule(
            ConnecticutDescriptiveReadability,
            0,
            true,
            false,
            true,
            "Conn. Gen. Stat. § 42-152 (eff. 1979, oldest U.S. plain-language statute) — hybrid descriptive standard ('plain English') + objective readability score requirement",
        ),
    );

    // NoStatewidePlainLanguageRequirement default — 46 other states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NM",
        "NC", "ND", "OH", "OK", "OR", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStatewidePlainLanguageRequirement,
                0,
                false,
                false,
                false,
                "No statewide plain-language lease statute; courts may apply common-law unconscionability / contract-of-adhesion doctrines to strike confusing or one-sided terms but no statutory penalty",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainLanguageInput {
    pub state_code: String,
    /// True if the lease was executed on or after the state's
    /// plain-language statute's effective date.
    pub lease_executed_after_statute_effective_date: bool,
    /// True if the lease is written in plain language per the
    /// applicable standard (descriptive judgment).
    pub lease_complies_with_plain_language_standard: bool,
    /// PA-specific: number of the 9 objective tests the lease
    /// satisfies. Substantial compliance generally requires
    /// meeting all 9.
    pub pa_nine_tests_satisfied: u32,
    /// True if the landlord has shown good-faith attempt to
    /// comply (NY good-faith defense).
    pub landlord_good_faith_compliance_attempt: bool,
    /// True if both parties have fully performed their obligations
    /// (NY no-action-after-full-performance limit).
    pub both_parties_fully_performed: bool,
    /// Tenant's actual damages from confusing-lease enforcement
    /// or term unconscionability.
    pub tenant_actual_damages_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainLanguageResult {
    pub regime: PlainLanguageRegime,
    pub statute_applies_on_facts: bool,
    pub landlord_compliant: bool,
    pub good_faith_defense_applies: bool,
    pub tenant_action_barred_by_full_performance: bool,
    pub tenant_remedy_amount_dollars: i64,
    pub attorney_fees_recoverable: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &PlainLanguageInput) -> PlainLanguageResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: PlainLanguageRegime::NoStatewidePlainLanguageRequirement,
        statutory_penalty_dollars: 0,
        attorney_fees_recoverable: false,
        good_faith_defense_available: false,
        objective_readability_required: false,
        citation: "Unknown state code; no statewide plain-language requirement assumed",
    });

    // Statute applies if the regime is not default AND the lease
    // was executed after the statute's effective date.
    let applies = !matches!(
        rule.regime,
        PlainLanguageRegime::NoStatewidePlainLanguageRequirement
    ) && input.lease_executed_after_statute_effective_date;

    // PA-specific 9-test compliance.
    let pa_compliant = match rule.regime {
        PlainLanguageRegime::PennsylvaniaPlainLanguageNineTests => {
            input.pa_nine_tests_satisfied == 9
        }
        _ => true,
    };

    // Compliance: lease meets plain-language standard AND (for PA)
    // all 9 tests pass.
    let lease_compliant = input.lease_complies_with_plain_language_standard && pa_compliant;

    // NY good-faith defense — even non-compliant lease can avoid
    // penalty if landlord shows good-faith attempt.
    let good_faith_defense_applies = rule.good_faith_defense_available
        && input.landlord_good_faith_compliance_attempt
        && applies
        && !lease_compliant;

    // NY action-barred-by-full-performance limit.
    let action_barred = matches!(
        rule.regime,
        PlainLanguageRegime::NewYorkClearCoherent50DollarPenalty
    ) && input.both_parties_fully_performed;

    let landlord_compliant =
        !applies || lease_compliant || good_faith_defense_applies || action_barred;

    let tenant_remedy = if landlord_compliant {
        0
    } else {
        // NJ: max($100 min, actual). NY: actual + $50.
        match rule.regime {
            PlainLanguageRegime::NewJersey100DollarMinimumPlusAttorneyFees => input
                .tenant_actual_damages_dollars
                .max(rule.statutory_penalty_dollars),
            PlainLanguageRegime::NewYorkClearCoherent50DollarPenalty => {
                input.tenant_actual_damages_dollars + rule.statutory_penalty_dollars
            }
            _ => input.tenant_actual_damages_dollars,
        }
    };

    let regime_label = match rule.regime {
        PlainLanguageRegime::NewYorkClearCoherent50DollarPenalty => {
            "New York clear-and-coherent + $50 penalty"
        }
        PlainLanguageRegime::NewJersey100DollarMinimumPlusAttorneyFees => {
            "New Jersey $100-minimum + attorney fees"
        }
        PlainLanguageRegime::PennsylvaniaPlainLanguageNineTests => {
            "Pennsylvania 9-test substantial-compliance"
        }
        PlainLanguageRegime::ConnecticutDescriptiveReadability => {
            "Connecticut descriptive + readability"
        }
        PlainLanguageRegime::NoStatewidePlainLanguageRequirement => {
            "no statewide plain-language requirement"
        }
    };

    let note = if !applies {
        format!(
            "State applies {} regime; statute does not apply on these facts (default state or lease pre-dates effective date).",
            regime_label,
        )
    } else if landlord_compliant {
        if good_faith_defense_applies {
            format!(
                "State applies {} regime; lease non-compliant but landlord shows good-faith attempt → defense applies, no penalty.",
                regime_label,
            )
        } else if action_barred {
            format!(
                "State applies {} regime; both parties fully performed — tenant action statutorily barred.",
                regime_label,
            )
        } else {
            format!(
                "State applies {} regime; landlord compliant on these facts.",
                regime_label,
            )
        }
    } else {
        format!(
            "State applies {} regime; landlord NON-COMPLIANT — tenant remedy ${}{}.",
            regime_label,
            tenant_remedy,
            if rule.attorney_fees_recoverable {
                " plus reasonable attorney fees"
            } else {
                ""
            },
        )
    };

    PlainLanguageResult {
        regime: rule.regime,
        statute_applies_on_facts: applies,
        landlord_compliant,
        good_faith_defense_applies,
        tenant_action_barred_by_full_performance: action_barred,
        tenant_remedy_amount_dollars: tenant_remedy,
        attorney_fees_recoverable: !landlord_compliant && rule.attorney_fees_recoverable,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> PlainLanguageInput {
        PlainLanguageInput {
            state_code: state.to_string(),
            lease_executed_after_statute_effective_date: true,
            lease_complies_with_plain_language_standard: false,
            pa_nine_tests_satisfied: 9,
            landlord_good_faith_compliance_attempt: false,
            both_parties_fully_performed: false,
            tenant_actual_damages_dollars: 500,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ny_clear_coherent_50_dollar_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(
            r.regime,
            PlainLanguageRegime::NewYorkClearCoherent50DollarPenalty
        );
    }

    #[test]
    fn nj_100_dollar_minimum_regime() {
        let r = check(&baseline("NJ"));
        assert_eq!(
            r.regime,
            PlainLanguageRegime::NewJersey100DollarMinimumPlusAttorneyFees
        );
    }

    #[test]
    fn pa_nine_tests_regime() {
        let r = check(&baseline("PA"));
        assert_eq!(
            r.regime,
            PlainLanguageRegime::PennsylvaniaPlainLanguageNineTests
        );
    }

    #[test]
    fn ct_descriptive_readability_regime() {
        let r = check(&baseline("CT"));
        assert_eq!(
            r.regime,
            PlainLanguageRegime::ConnecticutDescriptiveReadability
        );
    }

    #[test]
    fn default_state_no_requirement_regime() {
        for s in ["AL", "CA", "FL", "TX", "WA", "MA", "DC", "WY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                PlainLanguageRegime::NoStatewidePlainLanguageRequirement,
                "expected {s} no-requirement regime"
            );
        }
    }

    // ── NY: actual + $50 penalty + good-faith defense + full-perf bar ─

    #[test]
    fn ny_compliant_lease_no_penalty() {
        let mut i = baseline("NY");
        i.lease_complies_with_plain_language_standard = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
        assert_eq!(r.tenant_remedy_amount_dollars, 0);
    }

    #[test]
    fn ny_non_compliant_lease_actual_plus_50_dollar_penalty() {
        let r = check(&baseline("NY"));
        assert!(!r.landlord_compliant);
        // $500 actual + $50 penalty = $550.
        assert_eq!(r.tenant_remedy_amount_dollars, 550);
    }

    #[test]
    fn ny_good_faith_defense_applies() {
        let mut i = baseline("NY");
        i.landlord_good_faith_compliance_attempt = true;
        let r = check(&i);
        assert!(r.good_faith_defense_applies);
        assert!(r.landlord_compliant);
        assert_eq!(r.tenant_remedy_amount_dollars, 0);
    }

    #[test]
    fn ny_full_performance_bars_action() {
        let mut i = baseline("NY");
        i.both_parties_fully_performed = true;
        let r = check(&i);
        assert!(r.tenant_action_barred_by_full_performance);
        assert!(r.landlord_compliant);
        assert_eq!(r.tenant_remedy_amount_dollars, 0);
    }

    #[test]
    fn ny_pre_1978_lease_statute_does_not_apply() {
        let mut i = baseline("NY");
        i.lease_executed_after_statute_effective_date = false;
        let r = check(&i);
        assert!(!r.statute_applies_on_facts);
        assert!(r.landlord_compliant);
    }

    // ── NJ: greater of $100 OR actual + attorney fees ──────────────

    #[test]
    fn nj_actual_below_100_uses_100_minimum() {
        let mut i = baseline("NJ");
        i.tenant_actual_damages_dollars = 50;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert_eq!(r.tenant_remedy_amount_dollars, 100);
    }

    #[test]
    fn nj_actual_above_100_uses_actual() {
        let mut i = baseline("NJ");
        i.tenant_actual_damages_dollars = 5_000;
        let r = check(&i);
        assert_eq!(r.tenant_remedy_amount_dollars, 5_000);
    }

    #[test]
    fn nj_attorney_fees_recoverable_on_violation() {
        let r = check(&baseline("NJ"));
        assert!(r.attorney_fees_recoverable);
        assert!(r.note.contains("attorney fees"));
    }

    #[test]
    fn nj_no_attorney_fees_when_compliant() {
        let mut i = baseline("NJ");
        i.lease_complies_with_plain_language_standard = true;
        let r = check(&i);
        assert!(!r.attorney_fees_recoverable);
    }

    #[test]
    fn nj_no_good_faith_defense_path() {
        // NJ statute has no good-faith defense — only NY does.
        let mut i = baseline("NJ");
        i.landlord_good_faith_compliance_attempt = true;
        let r = check(&i);
        assert!(!r.good_faith_defense_applies);
        assert!(!r.landlord_compliant);
    }

    // ── PA 9-test substantial compliance ──────────────────────────

    #[test]
    fn pa_all_9_tests_passed_lease_complies_compliant() {
        let mut i = baseline("PA");
        i.pa_nine_tests_satisfied = 9;
        i.lease_complies_with_plain_language_standard = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn pa_8_of_9_tests_satisfied_non_compliant() {
        // PA requires substantial compliance with all 9 tests.
        let mut i = baseline("PA");
        i.pa_nine_tests_satisfied = 8;
        i.lease_complies_with_plain_language_standard = true;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn pa_attorney_fees_recoverable_on_violation() {
        let mut i = baseline("PA");
        i.pa_nine_tests_satisfied = 8;
        i.lease_complies_with_plain_language_standard = true;
        let r = check(&i);
        assert!(r.attorney_fees_recoverable);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_statute_applies() {
        let r = check(&baseline("CA"));
        assert!(!r.statute_applies_on_facts);
        assert!(r.landlord_compliant);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ny_citation_mentions_5_702_and_50_dollar() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("§ 5-702"));
        assert!(r.citation.contains("$50 statutory penalty"));
        assert!(r.citation.contains("good-faith"));
    }

    #[test]
    fn nj_citation_mentions_56_12_and_100_dollar_and_attorney_fees() {
        let r = check(&baseline("NJ"));
        assert!(r.citation.contains("56:12-1"));
        assert!(r.citation.contains("$100 statutory minimum"));
        assert!(r.citation.contains("attorney fees"));
    }

    #[test]
    fn pa_citation_mentions_73_2201_and_9_tests() {
        let r = check(&baseline("PA"));
        assert!(r.citation.contains("73 P.S. § 2201"));
        assert!(r.citation.contains("9 objective tests"));
    }

    #[test]
    fn ct_citation_mentions_42_152_and_oldest() {
        let r = check(&baseline("CT"));
        assert!(r.citation.contains("§ 42-152"));
        assert!(r.citation.contains("oldest"));
    }

    // ── Coverage / single-state-uniqueness ─────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        assert_eq!(RULES.len(), 51);
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn ny_only_50_dollar_penalty_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    PlainLanguageRegime::NewYorkClearCoherent50DollarPenalty
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn nj_only_100_minimum_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    PlainLanguageRegime::NewJersey100DollarMinimumPlusAttorneyFees
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn pa_only_nine_tests_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    PlainLanguageRegime::PennsylvaniaPlainLanguageNineTests
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ct_only_descriptive_readability_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    PlainLanguageRegime::ConnecticutDescriptiveReadability
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ny_only_good_faith_defense_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| r.good_faith_defense_available)
            .count();
        assert_eq!(count, 1, "only NY has good-faith defense");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ny_good_faith_note_mentions_defense() {
        let mut i = baseline("NY");
        i.landlord_good_faith_compliance_attempt = true;
        let r = check(&i);
        assert!(r.note.contains("good-faith attempt"));
        assert!(r.note.contains("defense applies"));
    }

    #[test]
    fn ny_full_performance_note_mentions_barred() {
        let mut i = baseline("NY");
        i.both_parties_fully_performed = true;
        let r = check(&i);
        assert!(r.note.contains("statutorily barred"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ny"));
        assert_eq!(
            r.regime,
            PlainLanguageRegime::NewYorkClearCoherent50DollarPenalty
        );
    }
}
