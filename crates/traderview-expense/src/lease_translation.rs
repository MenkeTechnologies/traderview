//! State mandatory lease translation requirement compliance table.
//!
//! When a landlord-trader leases to a non-English-speaking tenant, a
//! small number of states impose statutory translation requirements
//! on the lease itself. California's Civ. Code § 1632 is uniquely
//! strict in the country — failure to comply lets the tenant
//! RESCIND the contract entirely.
//!
//! Three regimes:
//!
//! - `MandatoryTranslationFiveLanguages` — CA (Civ. Code § 1632).
//!   When a residential rental lease > 1 month is negotiated
//!   PRIMARILY in any of **5 specific languages** — Spanish, Chinese,
//!   Tagalog, Vietnamese, or Korean — the landlord MUST provide a
//!   complete, accurate translation of the lease in that language
//!   BEFORE execution. Failure allows the tenant to RESCIND the
//!   contract and seek restitution.
//!
//! - `EnglishRequiredTranslationsNotBinding` — FL. Florida holds
//!   parties to the English contract terms regardless of language
//!   spoken; translations are courtesy copies but not legally
//!   binding. Narrow Fla. Stat. § 636.015 exception applies only
//!   to prepaid limited health service organizations and discount
//!   plan organizations — NOT residential leases.
//!
//! - `NoStateTranslationRequirement` — most other states. No
//!   statewide private-landlord translation mandate. New York has
//!   state-agency language access requirements but does NOT impose
//!   them on private landlords. Federal Fair Housing Act limited
//!   English proficiency (LEP) guidance applies to FEDERALLY-funded
//!   housing only — not private market.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationRegime {
    MandatoryTranslationFiveLanguages,
    EnglishRequiredTranslationsNotBinding,
    NoStateTranslationRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationLanguage {
    English,
    Spanish,
    Chinese,
    Tagalog,
    Vietnamese,
    Korean,
    OtherForeignLanguage,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: TranslationRegime,
    pub citation: &'static str,
}

const fn rule(regime: TranslationRegime, citation: &'static str) -> StateRule {
    StateRule { regime, citation }
}

/// Languages covered by CA Civ. Code § 1632 mandatory translation rule.
const CA_COVERED_LANGUAGES: &[NegotiationLanguage] = &[
    NegotiationLanguage::Spanish,
    NegotiationLanguage::Chinese,
    NegotiationLanguage::Tagalog,
    NegotiationLanguage::Vietnamese,
    NegotiationLanguage::Korean,
];

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use TranslationRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            MandatoryTranslationFiveLanguages,
            "Cal. Civ. Code § 1632 — mandatory translation in Spanish/Chinese/Tagalog/Vietnamese/Korean when residential lease > 1 month negotiated primarily in that language; failure → tenant right to rescind contract",
        ),
    );
    m.insert(
        "FL",
        rule(
            EnglishRequiredTranslationsNotBinding,
            "Florida — English required for legal contracts; translations not binding (Fla. Stat. § 636.015 narrow exception for prepaid health / discount-plan orgs only, NOT residential leases)",
        ),
    );

    // NoStateTranslationRequirement for remaining states + DC.
    let no_rule = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA",
        "WA", "WV", "WI", "WY",
    ];
    for code in no_rule {
        m.insert(
            code,
            rule(
                NoStateTranslationRequirement,
                "No state-level mandatory lease translation requirement; federal FHA LEP guidance applies to federally-funded housing only",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationInput {
    pub state_code: String,
    pub negotiation_language: NegotiationLanguage,
    pub lease_duration_months: u32,
    pub landlord_provided_translation_in_negotiation_language: bool,
    pub translation_provided_before_execution: bool,
    pub is_residential_lease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub regime: TranslationRegime,
    pub translation_required: bool,
    pub landlord_compliant: bool,
    pub tenant_has_rescission_right: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &TranslationInput) -> TranslationResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: TranslationRegime::NoStateTranslationRequirement,
        citation: "Unknown state code; assuming no state-level translation requirement",
    });

    let required = match rule.regime {
        TranslationRegime::MandatoryTranslationFiveLanguages => {
            // CA § 1632: residential lease > 1 month AND negotiated in
            // one of the 5 covered languages.
            input.is_residential_lease
                && input.lease_duration_months > 1
                && CA_COVERED_LANGUAGES.contains(&input.negotiation_language)
        }
        _ => false,
    };

    let compliant = if required {
        input.landlord_provided_translation_in_negotiation_language
            && input.translation_provided_before_execution
    } else {
        true
    };

    let rescission = required && !compliant;

    let note = match (rule.regime, required, compliant) {
        (TranslationRegime::MandatoryTranslationFiveLanguages, true, true) => format!(
            "CA Civ. Code § 1632 SATISFIED: lease negotiated in {:?}, translation provided before execution. Compliant.",
            input.negotiation_language,
        ),
        (TranslationRegime::MandatoryTranslationFiveLanguages, true, false) => format!(
            "CA Civ. Code § 1632 VIOLATION: lease negotiated in {:?} (covered language) but {} {}. TENANT MAY RESCIND CONTRACT and seek restitution.",
            input.negotiation_language,
            if !input.landlord_provided_translation_in_negotiation_language {
                "no translation was provided"
            } else {
                "translation"
            },
            if !input.translation_provided_before_execution
                && input.landlord_provided_translation_in_negotiation_language
            {
                "was not provided BEFORE execution"
            } else {
                ""
            },
        ),
        (TranslationRegime::MandatoryTranslationFiveLanguages, false, _) => {
            if !CA_COVERED_LANGUAGES.contains(&input.negotiation_language) {
                format!(
                    "CA Civ. Code § 1632: negotiation language {:?} is NOT one of the 5 covered languages (Spanish/Chinese/Tagalog/Vietnamese/Korean); no translation required.",
                    input.negotiation_language,
                )
            } else if input.lease_duration_months <= 1 {
                "CA Civ. Code § 1632: lease ≤ 1 month is outside the statute's scope; no translation required.".to_string()
            } else {
                "CA Civ. Code § 1632: non-residential lease outside statute's scope.".to_string()
            }
        }
        (TranslationRegime::EnglishRequiredTranslationsNotBinding, _, _) =>
            "Florida: English-only legal documents — translations are courtesy copies but not binding. No translation mandate for residential leases.".to_string(),
        (TranslationRegime::NoStateTranslationRequirement, _, _) =>
            "NoStateTranslationRequirement: no state-level mandatory lease translation rule applies.".to_string(),
    };

    TranslationResult {
        regime: rule.regime,
        translation_required: required,
        landlord_compliant: compliant,
        tenant_has_rescission_right: rescission,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, lang: NegotiationLanguage) -> TranslationInput {
        TranslationInput {
            state_code: state.to_string(),
            negotiation_language: lang,
            lease_duration_months: 12,
            landlord_provided_translation_in_negotiation_language: true,
            translation_provided_before_execution: true,
            is_residential_lease: true,
        }
    }

    // CA — all 5 covered languages.

    #[test]
    fn ca_spanish_lease_with_translation_complies() {
        let r = check(&input("CA", NegotiationLanguage::Spanish));
        assert_eq!(
            r.regime,
            TranslationRegime::MandatoryTranslationFiveLanguages
        );
        assert!(r.translation_required);
        assert!(r.landlord_compliant);
        assert!(!r.tenant_has_rescission_right);
    }

    #[test]
    fn ca_chinese_lease_required() {
        let r = check(&input("CA", NegotiationLanguage::Chinese));
        assert!(r.translation_required);
    }

    #[test]
    fn ca_tagalog_lease_required() {
        let r = check(&input("CA", NegotiationLanguage::Tagalog));
        assert!(r.translation_required);
    }

    #[test]
    fn ca_vietnamese_lease_required() {
        let r = check(&input("CA", NegotiationLanguage::Vietnamese));
        assert!(r.translation_required);
    }

    #[test]
    fn ca_korean_lease_required() {
        let r = check(&input("CA", NegotiationLanguage::Korean));
        assert!(r.translation_required);
    }

    #[test]
    fn ca_english_lease_no_translation_required() {
        let r = check(&input("CA", NegotiationLanguage::English));
        assert!(!r.translation_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_other_foreign_language_no_requirement() {
        // CA only covers the 5 named languages; other languages (e.g.,
        // Russian, Arabic, Hindi) are NOT covered.
        let r = check(&input("CA", NegotiationLanguage::OtherForeignLanguage));
        assert!(!r.translation_required);
        assert!(r.note.contains("NOT one of the 5 covered languages"));
    }

    // CA failure modes.

    #[test]
    fn ca_spanish_no_translation_provided_rescission_right() {
        let mut i = input("CA", NegotiationLanguage::Spanish);
        i.landlord_provided_translation_in_negotiation_language = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.tenant_has_rescission_right);
        assert!(r.note.contains("TENANT MAY RESCIND"));
    }

    #[test]
    fn ca_translation_after_execution_violates() {
        let mut i = input("CA", NegotiationLanguage::Spanish);
        i.translation_provided_before_execution = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.tenant_has_rescission_right);
    }

    #[test]
    fn ca_short_term_lease_one_month_outside_scope() {
        let mut i = input("CA", NegotiationLanguage::Spanish);
        i.lease_duration_months = 1;
        let r = check(&i);
        assert!(!r.translation_required);
        assert!(r.note.contains("≤ 1 month"));
    }

    #[test]
    fn ca_two_month_lease_within_scope() {
        let mut i = input("CA", NegotiationLanguage::Spanish);
        i.lease_duration_months = 2;
        let r = check(&i);
        assert!(r.translation_required);
    }

    #[test]
    fn ca_non_residential_lease_outside_scope() {
        let mut i = input("CA", NegotiationLanguage::Spanish);
        i.is_residential_lease = false;
        let r = check(&i);
        assert!(!r.translation_required);
    }

    // Florida.

    #[test]
    fn fl_no_translation_mandate() {
        let r = check(&input("FL", NegotiationLanguage::Spanish));
        assert_eq!(
            r.regime,
            TranslationRegime::EnglishRequiredTranslationsNotBinding
        );
        assert!(!r.translation_required);
        assert!(r.note.contains("English-only legal documents"));
    }

    #[test]
    fn fl_translations_not_binding_explicit_in_note() {
        let r = check(&input("FL", NegotiationLanguage::Spanish));
        assert!(r
            .note
            .contains("translations are courtesy copies but not binding"));
    }

    // No-rule states.

    #[test]
    fn no_rule_states_no_translation_required() {
        for st in &["TX", "NY", "IL", "WA", "MA", "OR", "DC"] {
            let r = check(&input(st, NegotiationLanguage::Spanish));
            assert_eq!(
                r.regime,
                TranslationRegime::NoStateTranslationRequirement,
                "{st}"
            );
            assert!(!r.translation_required, "{st}");
            assert!(r.landlord_compliant, "{st}");
        }
    }

    // Coverage / structural.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(
            codes.len(),
            51,
            "expected 50 states + DC, got {}",
            codes.len()
        );
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn ca_unique_mandatory_translation_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == TranslationRegime::MandatoryTranslationFiveLanguages {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "expected CA only with mandatory translation regime"
        );
    }

    #[test]
    fn fl_unique_english_required_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == TranslationRegime::EnglishRequiredTranslationsNotBinding {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected FL only with English-required regime");
    }

    #[test]
    fn unknown_state_falls_back_to_no_rule() {
        let r = check(&input("XX", NegotiationLanguage::Spanish));
        assert_eq!(r.regime, TranslationRegime::NoStateTranslationRequirement);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ca", NegotiationLanguage::Spanish));
        assert!(r.translation_required);
    }

    // 5 languages invariant.

    #[test]
    fn ca_covered_languages_exactly_5() {
        assert_eq!(CA_COVERED_LANGUAGES.len(), 5);
    }

    #[test]
    fn ca_violation_note_describes_rescission() {
        let mut i = input("CA", NegotiationLanguage::Korean);
        i.landlord_provided_translation_in_negotiation_language = false;
        let r = check(&i);
        assert!(r.note.contains("VIOLATION"));
        assert!(r.note.contains("RESCIND"));
    }

    #[test]
    fn ca_compliant_note_describes_satisfaction() {
        let r = check(&input("CA", NegotiationLanguage::Chinese));
        assert!(r.note.contains("§ 1632 SATISFIED"));
    }
}
