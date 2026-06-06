//! California Civil Code § 1632 multilingual lease translation
//! compliance for trader-landlords negotiating residential leases
//! primarily in Spanish, Chinese, Tagalog, Vietnamese, or Korean.
//!
//! Operative rule (CA Civ. Code § 1632(b)): a person engaged in a
//! trade or business who NEGOTIATES PRIMARILY in Spanish, Chinese,
//! Tagalog, Vietnamese, or Korean — orally or in writing — must
//! deliver to the other party, BEFORE the execution of the lease, a
//! TRANSLATION of the contract in the language in which the contract
//! was negotiated that includes a translation of every term and
//! condition.
//!
//! **Residential lease coverage** (CA Civ. Code § 1632(b)(1)(A)):
//! applies to residential leases or rental agreements LONGER THAN
//! ONE MONTH. Month-to-month and shorter tenancies are excluded.
//!
//! **Commercial lease coverage** (CA Civ. Code § 1632(b)(1)(C),
//! added by SB 1103, eff. January 1, 2025): applies to nonresidential-
//! zoned commercial leases entered into with a "qualified commercial
//! tenant" — a microenterprise, a restaurant with fewer than 10
//! employees, or a nonprofit organization with fewer than 20
//! employees, who has provided the landlord with written notice of
//! qualified status and a self-attestation.
//!
//! **Own-interpreter exemption** (CA Civ. Code § 1632(h)): the
//! section does not apply if the party negotiates through their own
//! interpreter. "Own interpreter" means (1) a person, not a minor,
//! (2) able to speak fluently and read with full understanding both
//! English AND the negotiated language, AND (3) NOT employed by or
//! made available through the business.
//!
//! **Non-compliance remedy** (CA Civ. Code § 1632(k)): the aggrieved
//! party may RESCIND the contract. Statutory rescission right is
//! the central enforcement mechanism — no separate damages provision.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const COVERED_LANGUAGES_COUNT: u32 = 5;
#[allow(dead_code)]
pub const RESIDENTIAL_LEASE_MIN_DURATION_MONTHS_EXCLUSIVE: u32 = 1;
#[allow(dead_code)]
pub const INTERPRETER_MIN_AGE_YEARS: u32 = 18;
#[allow(dead_code)]
pub const SB_1103_COMMERCIAL_EFFECTIVE_YEAR: u32 = 2025;
#[allow(dead_code)]
pub const SB_1103_RESTAURANT_EMPLOYEE_THRESHOLD: u32 = 10;
#[allow(dead_code)]
pub const SB_1103_NONPROFIT_EMPLOYEE_THRESHOLD: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationLanguage {
    English,
    Spanish,
    Chinese,
    Tagalog,
    Vietnamese,
    Korean,
    OtherLanguageNotCoveredBy1632,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseType {
    ResidentialLongerThanOneMonth,
    ResidentialMonthToMonthOrShorter,
    CommercialQualifiedTenantPostSb1103,
    CommercialNonQualifiedTenant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicableEnglishNegotiation,
    NotApplicableOtherLanguageNotCovered,
    NotApplicableResidentialMonthToMonthShortTenancy,
    NotApplicableCommercialNonQualifiedTenant,
    CompliantTranslationProvidedBeforeExecution,
    ExemptTenantUsedOwnInterpreter,
    ViolationInterpreterDoesNotMeetStatutoryDefinition,
    ViolationTranslationNotProvidedBeforeExecutionRescindable,
    ViolationTranslationInWrongLanguageRescindable,
    ViolationPartialTranslationMissingTermsRescindable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub negotiation_language: NegotiationLanguage,
    pub lease_type: LeaseType,
    pub lease_term_months: u32,
    pub translation_provided_before_execution: bool,
    pub translation_in_negotiated_language: bool,
    pub translation_includes_all_terms_and_conditions: bool,
    pub tenant_used_own_interpreter: bool,
    pub interpreter_age_years: u32,
    pub interpreter_fluent_in_both_languages: bool,
    pub interpreter_provided_by_landlord: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub translation_required: bool,
    pub rescindable_by_tenant: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type MultilingualLeaseTranslationInput = Input;
pub type MultilingualLeaseTranslationOutput = Output;
pub type MultilingualLeaseTranslationResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "CA Civ. Code § 1632(b) (translation required before execution)".to_string(),
        "CA Civ. Code § 1632(b)(1)(A) (residential lease > 1 month coverage)".to_string(),
        "CA Civ. Code § 1632(b)(1)(C) (commercial coverage added by SB 1103, eff. 2025-01-01)".to_string(),
        "CA Civ. Code § 1632(h) (own-interpreter exemption)".to_string(),
        "CA Civ. Code § 1632(k) (rescission remedy)".to_string(),
        "SB 1103 (Caballero, 2024) — Commercial Tenant Protection Act".to_string(),
        "Reyes v. Superior Court, 118 Cal. App. 3d 159 (1981) — original Spanish-language translation case".to_string(),
    ];

    if matches!(input.negotiation_language, NegotiationLanguage::English) {
        notes.push(
            "Negotiation in English — § 1632 translation requirement not triggered.".to_string(),
        );
        return Output {
            severity: Severity::NotApplicableEnglishNegotiation,
            translation_required: false,
            rescindable_by_tenant: false,
            notes,
            citations,
        };
    }

    if matches!(
        input.negotiation_language,
        NegotiationLanguage::OtherLanguageNotCoveredBy1632
    ) {
        notes.push(format!(
            "Language not among {} covered languages (Spanish, Chinese, Tagalog, Vietnamese, Korean) — § 1632 does not apply.",
            COVERED_LANGUAGES_COUNT
        ));
        return Output {
            severity: Severity::NotApplicableOtherLanguageNotCovered,
            translation_required: false,
            rescindable_by_tenant: false,
            notes,
            citations,
        };
    }

    match input.lease_type {
        LeaseType::ResidentialMonthToMonthOrShorter => {
            notes.push(format!(
                "Residential month-to-month or short tenancy (term {} months ≤ {}) — § 1632(b)(1)(A) requires > 1 month for residential coverage.",
                input.lease_term_months,
                RESIDENTIAL_LEASE_MIN_DURATION_MONTHS_EXCLUSIVE
            ));
            return Output {
                severity: Severity::NotApplicableResidentialMonthToMonthShortTenancy,
                translation_required: false,
                rescindable_by_tenant: false,
                notes,
                citations,
            };
        }
        LeaseType::CommercialNonQualifiedTenant => {
            notes.push("Commercial lease but tenant does not meet SB 1103 qualified-commercial-tenant criteria — § 1632 does not apply.".to_string());
            return Output {
                severity: Severity::NotApplicableCommercialNonQualifiedTenant,
                translation_required: false,
                rescindable_by_tenant: false,
                notes,
                citations,
            };
        }
        _ => {}
    }

    if input.tenant_used_own_interpreter {
        if input.interpreter_age_years < INTERPRETER_MIN_AGE_YEARS
            || !input.interpreter_fluent_in_both_languages
            || input.interpreter_provided_by_landlord
        {
            notes.push(format!(
                "Interpreter fails § 1632(h) statutory definition (age {} < {}, fluent both = {}, landlord-provided = {}). Exemption denied.",
                input.interpreter_age_years,
                INTERPRETER_MIN_AGE_YEARS,
                input.interpreter_fluent_in_both_languages,
                input.interpreter_provided_by_landlord
            ));
            return Output {
                severity: Severity::ViolationInterpreterDoesNotMeetStatutoryDefinition,
                translation_required: true,
                rescindable_by_tenant: true,
                notes,
                citations,
            };
        }
        notes.push("Tenant used their own qualifying interpreter under § 1632(h) — translation requirement exempted.".to_string());
        return Output {
            severity: Severity::ExemptTenantUsedOwnInterpreter,
            translation_required: false,
            rescindable_by_tenant: false,
            notes,
            citations,
        };
    }

    if !input.translation_provided_before_execution {
        notes.push("Translation not provided BEFORE execution — per se § 1632(b) violation; tenant may rescind under § 1632(k).".to_string());
        return Output {
            severity: Severity::ViolationTranslationNotProvidedBeforeExecutionRescindable,
            translation_required: true,
            rescindable_by_tenant: true,
            notes,
            citations,
        };
    }

    if !input.translation_in_negotiated_language {
        notes.push("Translation provided but in wrong language (not the negotiated language) — § 1632(b) violation; tenant may rescind.".to_string());
        return Output {
            severity: Severity::ViolationTranslationInWrongLanguageRescindable,
            translation_required: true,
            rescindable_by_tenant: true,
            notes,
            citations,
        };
    }

    if !input.translation_includes_all_terms_and_conditions {
        notes.push("Translation missing terms or conditions — § 1632(b) requires translation of EVERY term; tenant may rescind.".to_string());
        return Output {
            severity: Severity::ViolationPartialTranslationMissingTermsRescindable,
            translation_required: true,
            rescindable_by_tenant: true,
            notes,
            citations,
        };
    }

    notes.push("§ 1632 compliant: full translation in negotiated language provided before execution covering every term and condition.".to_string());
    Output {
        severity: Severity::CompliantTranslationProvidedBeforeExecution,
        translation_required: true,
        rescindable_by_tenant: false,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_spanish_residential() -> Input {
        Input {
            negotiation_language: NegotiationLanguage::Spanish,
            lease_type: LeaseType::ResidentialLongerThanOneMonth,
            lease_term_months: 12,
            translation_provided_before_execution: true,
            translation_in_negotiated_language: true,
            translation_includes_all_terms_and_conditions: true,
            tenant_used_own_interpreter: false,
            interpreter_age_years: 0,
            interpreter_fluent_in_both_languages: false,
            interpreter_provided_by_landlord: false,
        }
    }

    #[test]
    fn spanish_residential_full_translation_compliant() {
        let out = check(&base_spanish_residential());
        assert_eq!(
            out.severity,
            Severity::CompliantTranslationProvidedBeforeExecution
        );
        assert!(out.translation_required);
        assert!(!out.rescindable_by_tenant);
    }

    #[test]
    fn english_negotiation_not_applicable() {
        let mut i = base_spanish_residential();
        i.negotiation_language = NegotiationLanguage::English;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicableEnglishNegotiation);
    }

    #[test]
    fn chinese_negotiation_covered_same_as_spanish() {
        let mut i = base_spanish_residential();
        i.negotiation_language = NegotiationLanguage::Chinese;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantTranslationProvidedBeforeExecution
        );
    }

    #[test]
    fn tagalog_negotiation_covered() {
        let mut i = base_spanish_residential();
        i.negotiation_language = NegotiationLanguage::Tagalog;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantTranslationProvidedBeforeExecution
        );
    }

    #[test]
    fn vietnamese_negotiation_covered() {
        let mut i = base_spanish_residential();
        i.negotiation_language = NegotiationLanguage::Vietnamese;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantTranslationProvidedBeforeExecution
        );
    }

    #[test]
    fn korean_negotiation_covered() {
        let mut i = base_spanish_residential();
        i.negotiation_language = NegotiationLanguage::Korean;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantTranslationProvidedBeforeExecution
        );
    }

    #[test]
    fn other_uncovered_language_not_applicable() {
        let mut i = base_spanish_residential();
        i.negotiation_language = NegotiationLanguage::OtherLanguageNotCoveredBy1632;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicableOtherLanguageNotCovered);
    }

    #[test]
    fn residential_month_to_month_not_applicable() {
        let mut i = base_spanish_residential();
        i.lease_type = LeaseType::ResidentialMonthToMonthOrShorter;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NotApplicableResidentialMonthToMonthShortTenancy
        );
    }

    #[test]
    fn commercial_non_qualified_tenant_not_applicable() {
        let mut i = base_spanish_residential();
        i.lease_type = LeaseType::CommercialNonQualifiedTenant;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NotApplicableCommercialNonQualifiedTenant
        );
    }

    #[test]
    fn commercial_qualified_tenant_sb_1103_covered() {
        let mut i = base_spanish_residential();
        i.lease_type = LeaseType::CommercialQualifiedTenantPostSb1103;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantTranslationProvidedBeforeExecution
        );
    }

    #[test]
    fn translation_not_provided_before_execution_rescindable() {
        let mut i = base_spanish_residential();
        i.translation_provided_before_execution = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationTranslationNotProvidedBeforeExecutionRescindable
        );
        assert!(out.rescindable_by_tenant);
    }

    #[test]
    fn translation_in_wrong_language_rescindable() {
        let mut i = base_spanish_residential();
        i.translation_in_negotiated_language = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationTranslationInWrongLanguageRescindable
        );
    }

    #[test]
    fn partial_translation_missing_terms_rescindable() {
        let mut i = base_spanish_residential();
        i.translation_includes_all_terms_and_conditions = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationPartialTranslationMissingTermsRescindable
        );
    }

    #[test]
    fn tenant_own_qualifying_interpreter_exempt() {
        let mut i = base_spanish_residential();
        i.tenant_used_own_interpreter = true;
        i.interpreter_age_years = 30;
        i.interpreter_fluent_in_both_languages = true;
        i.interpreter_provided_by_landlord = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptTenantUsedOwnInterpreter);
    }

    #[test]
    fn interpreter_under_18_disqualifies_exemption() {
        let mut i = base_spanish_residential();
        i.tenant_used_own_interpreter = true;
        i.interpreter_age_years = 17;
        i.interpreter_fluent_in_both_languages = true;
        i.interpreter_provided_by_landlord = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationInterpreterDoesNotMeetStatutoryDefinition
        );
        assert!(out.rescindable_by_tenant);
    }

    #[test]
    fn interpreter_provided_by_landlord_disqualifies_exemption() {
        let mut i = base_spanish_residential();
        i.tenant_used_own_interpreter = true;
        i.interpreter_age_years = 30;
        i.interpreter_fluent_in_both_languages = true;
        i.interpreter_provided_by_landlord = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationInterpreterDoesNotMeetStatutoryDefinition
        );
    }

    #[test]
    fn interpreter_not_fluent_disqualifies_exemption() {
        let mut i = base_spanish_residential();
        i.tenant_used_own_interpreter = true;
        i.interpreter_age_years = 30;
        i.interpreter_fluent_in_both_languages = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationInterpreterDoesNotMeetStatutoryDefinition
        );
    }

    #[test]
    fn citations_pin_1632_subsections_b_h_k() {
        let out = check(&base_spanish_residential());
        assert!(out.citations.iter().any(|c| c.contains("§ 1632(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1632(h)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1632(k)")));
    }

    #[test]
    fn citations_pin_sb_1103_and_reyes_case_law() {
        let out = check(&base_spanish_residential());
        assert!(out.citations.iter().any(|c| c.contains("SB 1103")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Reyes v. Superior Court")));
    }

    #[test]
    fn constant_pin_5_covered_languages() {
        assert_eq!(COVERED_LANGUAGES_COUNT, 5);
    }

    #[test]
    fn constant_pin_interpreter_age_18() {
        assert_eq!(INTERPRETER_MIN_AGE_YEARS, 18);
    }

    #[test]
    fn constant_pin_sb_1103_effective_2025() {
        assert_eq!(SB_1103_COMMERCIAL_EFFECTIVE_YEAR, 2025);
    }

    #[test]
    fn constant_pin_sb_1103_restaurant_10_employees() {
        assert_eq!(SB_1103_RESTAURANT_EMPLOYEE_THRESHOLD, 10);
    }

    #[test]
    fn constant_pin_sb_1103_nonprofit_20_employees() {
        assert_eq!(SB_1103_NONPROFIT_EMPLOYEE_THRESHOLD, 20);
    }

    #[test]
    fn interpreter_exactly_18_qualifies() {
        let mut i = base_spanish_residential();
        i.tenant_used_own_interpreter = true;
        i.interpreter_age_years = 18;
        i.interpreter_fluent_in_both_languages = true;
        i.interpreter_provided_by_landlord = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptTenantUsedOwnInterpreter);
    }
}
