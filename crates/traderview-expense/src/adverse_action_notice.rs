//! Tenant adverse action notice compliance (federal FCRA + state).
//!
//! When a landlord takes "adverse action" against a rental applicant
//! based in any part on a consumer report (credit, criminal, eviction
//! history), federal **FCRA § 615 (15 U.S.C. § 1681m)** requires the
//! landlord to provide the applicant with a written adverse action
//! notice containing specific elements. Several states add their own
//! requirements on top of the federal floor.
//!
//! **What constitutes "adverse action"** for tenant screening:
//! - Outright denial of the rental application
//! - Conditional approval with less favorable terms (higher rent,
//!   higher security deposit than baseline, cosigner requirement,
//!   shorter lease term)
//! - Any other unfavorable decision influenced by the consumer report
//!
//! **FCRA § 615 required notice elements** (federal floor):
//! 1. Statement that adverse action was taken
//! 2. Name, address, telephone number of the consumer reporting
//!    agency (CRA) that supplied the report
//! 3. Statement that the CRA did not make the adverse decision and
//!    cannot explain why
//! 4. Notice of applicant's right to obtain a FREE copy of the report
//!    within 60 days from the CRA
//! 5. Notice of applicant's right to dispute inaccurate information
//!    with the CRA
//!
//! Two regimes:
//!
//! - `StateAddsRequirements` — CA (Cal. Civ. Code § 1785.20.5 + Civ.
//!   Code § 1786 ICRA: stricter content, but-for reason, 12-point-font
//!   credit-check disclosure), WA (RCW 59.18.257 + RCW 19.182.110:
//!   specific adverse action + but-for reason based on application),
//!   NY (GBL § 380-b: written notice with stated reasons).
//!
//! - `FederalFcraOnly` — most other states. FTC enforces FCRA § 615
//!   floor; states add nothing beyond the federal minimum.
//!
//! Note: FCRA § 615 itself does NOT specify a fixed number of days
//! between adverse action and notice — only a "reasonable time"
//! standard. Industry best practice is 5 business days. Module
//! does not pin a specific day count to avoid inventing facts.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdverseActionRegime {
    StateAddsRequirements,
    FederalFcraOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdverseActionType {
    DenialOfApplication,
    HigherDeposit,
    HigherRent,
    CosignerRequired,
    ConditionalShorterTerm,
    None,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: AdverseActionRegime,
    /// True if state explicitly requires the notice to state the
    /// SPECIFIC reason for the adverse action (not just generic
    /// "based on consumer report").
    pub state_requires_specific_reason: bool,
    /// True if state explicitly requires the but-for cause analysis.
    pub state_requires_but_for_reason: bool,
    /// True if state requires 12-point or other formatting standards.
    pub state_requires_formatting_standards: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: AdverseActionRegime,
    state_requires_specific_reason: bool,
    state_requires_but_for_reason: bool,
    state_requires_formatting_standards: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        state_requires_specific_reason,
        state_requires_but_for_reason,
        state_requires_formatting_standards,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use AdverseActionRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // StateAddsRequirements regime.
    m.insert(
        "CA",
        rule(
            StateAddsRequirements,
            true, true, true,
            "Cal. Civ. Code § 1785.20.5 + § 1786 ICRA — written notice with specific reason, 12-point-font credit check disclosure, but-for reason analysis",
        ),
    );
    m.insert(
        "WA",
        rule(
            StateAddsRequirements,
            true, true, false,
            "Wash. RCW 59.18.257 + RCW 19.182.110 — written adverse action notice stating specific reasons + but-for analysis based on application",
        ),
    );
    m.insert(
        "NY",
        rule(
            StateAddsRequirements,
            true, false, false,
            "N.Y. GBL § 380-b — written notice with stated reasons; tenant screening report copy delivery to applicant required",
        ),
    );

    // FederalFcraOnly regime — all remaining states + DC.
    let federal_only = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA",
        "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
        "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD",
        "TN", "TX", "UT", "VT", "VA", "WV", "WI", "WY",
    ];
    for code in federal_only {
        m.insert(
            code,
            rule(
                FederalFcraOnly,
                false, false, false,
                "FCRA § 615 (15 U.S.C. § 1681m) federal floor only; no state-added content requirements",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseActionInput {
    pub state_code: String,
    pub adverse_action_type: AdverseActionType,
    /// True if a consumer report (credit, criminal, eviction)
    /// contributed to the adverse decision. FCRA notice is required
    /// only when this is true.
    pub consumer_report_contributed_to_decision: bool,
    pub written_notice_provided: bool,
    /// Federal FCRA § 615 required elements — caller asserts each.
    pub notice_includes_cra_name_address_phone: bool,
    pub notice_includes_cra_did_not_decide_disclosure: bool,
    pub notice_includes_free_copy_60_day_right: bool,
    pub notice_includes_dispute_right: bool,
    /// State-specific elements where applicable.
    pub notice_states_specific_reason: bool,
    pub notice_includes_but_for_analysis: bool,
    pub notice_meets_formatting_standards: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseActionResult {
    pub regime: AdverseActionRegime,
    pub federal_fcra_notice_required: bool,
    pub federal_fcra_elements_satisfied: bool,
    pub state_elements_satisfied: bool,
    pub overall_compliant: bool,
    pub missing_required_elements: Vec<String>,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &AdverseActionInput) -> AdverseActionResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: AdverseActionRegime::FederalFcraOnly,
        state_requires_specific_reason: false,
        state_requires_but_for_reason: false,
        state_requires_formatting_standards: false,
        citation: "Unknown state code; assuming federal FCRA floor only",
    });

    // FCRA notice required only when adverse action AND consumer
    // report contributed.
    let fcra_required = input.adverse_action_type != AdverseActionType::None
        && input.consumer_report_contributed_to_decision;

    let mut missing: Vec<String> = Vec::new();

    if fcra_required {
        if !input.written_notice_provided {
            missing.push("written notice not provided".to_string());
        }
        if !input.notice_includes_cra_name_address_phone {
            missing.push("CRA name/address/phone".to_string());
        }
        if !input.notice_includes_cra_did_not_decide_disclosure {
            missing.push("CRA-did-not-decide disclosure".to_string());
        }
        if !input.notice_includes_free_copy_60_day_right {
            missing.push("60-day free-copy right".to_string());
        }
        if !input.notice_includes_dispute_right {
            missing.push("dispute right".to_string());
        }
        if rule.state_requires_specific_reason && !input.notice_states_specific_reason {
            missing.push("state-required specific reason".to_string());
        }
        if rule.state_requires_but_for_reason && !input.notice_includes_but_for_analysis {
            missing.push("state-required but-for analysis".to_string());
        }
        if rule.state_requires_formatting_standards
            && !input.notice_meets_formatting_standards
        {
            missing.push("state-required formatting standards".to_string());
        }
    }

    let federal_satisfied = !fcra_required
        || (input.written_notice_provided
            && input.notice_includes_cra_name_address_phone
            && input.notice_includes_cra_did_not_decide_disclosure
            && input.notice_includes_free_copy_60_day_right
            && input.notice_includes_dispute_right);
    let state_satisfied = !fcra_required
        || ((!rule.state_requires_specific_reason || input.notice_states_specific_reason)
            && (!rule.state_requires_but_for_reason
                || input.notice_includes_but_for_analysis)
            && (!rule.state_requires_formatting_standards
                || input.notice_meets_formatting_standards));

    let overall = federal_satisfied && state_satisfied;

    let note = if !fcra_required {
        if input.adverse_action_type == AdverseActionType::None {
            "No adverse action taken; no FCRA § 615 notice required.".to_string()
        } else {
            "Consumer report did not contribute to the adverse decision; FCRA § 615 notice not triggered (state common-law / fair-housing rules may still apply).".to_string()
        }
    } else if overall {
        format!(
            "{:?}: adverse action {:?} based on consumer report; all federal FCRA § 615 + state-specific notice elements satisfied.",
            rule.regime, input.adverse_action_type,
        )
    } else {
        format!(
            "{:?} VIOLATION: adverse action {:?} based on consumer report; missing required elements: {}.",
            rule.regime, input.adverse_action_type, missing.join("; "),
        )
    };

    AdverseActionResult {
        regime: rule.regime,
        federal_fcra_notice_required: fcra_required,
        federal_fcra_elements_satisfied: federal_satisfied,
        state_elements_satisfied: state_satisfied,
        overall_compliant: overall,
        missing_required_elements: missing,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_compliant_input(state: &str) -> AdverseActionInput {
        AdverseActionInput {
            state_code: state.to_string(),
            adverse_action_type: AdverseActionType::DenialOfApplication,
            consumer_report_contributed_to_decision: true,
            written_notice_provided: true,
            notice_includes_cra_name_address_phone: true,
            notice_includes_cra_did_not_decide_disclosure: true,
            notice_includes_free_copy_60_day_right: true,
            notice_includes_dispute_right: true,
            notice_states_specific_reason: true,
            notice_includes_but_for_analysis: true,
            notice_meets_formatting_standards: true,
        }
    }

    fn fcra_minimum_input(state: &str) -> AdverseActionInput {
        AdverseActionInput {
            state_code: state.to_string(),
            adverse_action_type: AdverseActionType::DenialOfApplication,
            consumer_report_contributed_to_decision: true,
            written_notice_provided: true,
            notice_includes_cra_name_address_phone: true,
            notice_includes_cra_did_not_decide_disclosure: true,
            notice_includes_free_copy_60_day_right: true,
            notice_includes_dispute_right: true,
            notice_states_specific_reason: false,
            notice_includes_but_for_analysis: false,
            notice_meets_formatting_standards: false,
        }
    }

    // Federal-only baseline.

    #[test]
    fn tx_fcra_only_minimum_satisfies() {
        let r = check(&fcra_minimum_input("TX"));
        assert_eq!(r.regime, AdverseActionRegime::FederalFcraOnly);
        assert!(r.federal_fcra_notice_required);
        assert!(r.federal_fcra_elements_satisfied);
        assert!(r.state_elements_satisfied);
        assert!(r.overall_compliant);
    }

    #[test]
    fn tx_missing_cra_contact_violates() {
        let mut i = fcra_minimum_input("TX");
        i.notice_includes_cra_name_address_phone = false;
        let r = check(&i);
        assert!(!r.federal_fcra_elements_satisfied);
        assert!(!r.overall_compliant);
        assert!(r.missing_required_elements.iter().any(|s| s.contains("CRA name/address/phone")));
    }

    #[test]
    fn tx_missing_60_day_free_copy_right_violates() {
        let mut i = fcra_minimum_input("TX");
        i.notice_includes_free_copy_60_day_right = false;
        let r = check(&i);
        assert!(!r.overall_compliant);
        assert!(r.missing_required_elements.iter().any(|s| s.contains("60-day")));
    }

    // California state-adds regime.

    #[test]
    fn ca_fcra_only_minimum_violates_state_additions() {
        // FCRA elements present, but CA-specific (specific reason +
        // but-for + formatting) missing.
        let r = check(&fcra_minimum_input("CA"));
        assert_eq!(r.regime, AdverseActionRegime::StateAddsRequirements);
        assert!(r.federal_fcra_elements_satisfied);
        assert!(!r.state_elements_satisfied);
        assert!(!r.overall_compliant);
        // 3 state-specific elements should be missing.
        assert!(r.missing_required_elements.iter().any(|s| s.contains("specific reason")));
        assert!(r.missing_required_elements.iter().any(|s| s.contains("but-for")));
        assert!(r.missing_required_elements.iter().any(|s| s.contains("formatting")));
    }

    #[test]
    fn ca_fully_compliant_passes() {
        let r = check(&fully_compliant_input("CA"));
        assert!(r.overall_compliant);
    }

    #[test]
    fn ca_higher_deposit_triggers_notice_requirement() {
        let mut i = fully_compliant_input("CA");
        i.adverse_action_type = AdverseActionType::HigherDeposit;
        let r = check(&i);
        assert!(r.federal_fcra_notice_required);
        assert!(r.overall_compliant);
    }

    // Washington — specific reason + but-for required; no formatting.

    #[test]
    fn wa_missing_but_for_violates() {
        let mut i = fully_compliant_input("WA");
        i.notice_includes_but_for_analysis = false;
        let r = check(&i);
        assert!(!r.state_elements_satisfied);
        assert!(r.missing_required_elements.iter().any(|s| s.contains("but-for")));
    }

    #[test]
    fn wa_missing_formatting_does_not_violate() {
        // WA doesn't require formatting standards; missing flag should
        // not cause violation.
        let mut i = fully_compliant_input("WA");
        i.notice_meets_formatting_standards = false;
        let r = check(&i);
        assert!(r.state_elements_satisfied);
        assert!(r.overall_compliant);
    }

    // New York — specific reason but no but-for.

    #[test]
    fn ny_specific_reason_required_but_for_not() {
        let mut i = fully_compliant_input("NY");
        i.notice_includes_but_for_analysis = false;
        let r = check(&i);
        assert!(r.state_elements_satisfied); // No but-for requirement in NY
        assert!(r.overall_compliant);

        let mut i2 = fully_compliant_input("NY");
        i2.notice_states_specific_reason = false;
        let r2 = check(&i2);
        assert!(!r2.state_elements_satisfied);
        assert!(r2.missing_required_elements.iter().any(|s| s.contains("specific reason")));
    }

    // No adverse action / no consumer report cases.

    #[test]
    fn no_adverse_action_no_notice_required() {
        let mut i = fully_compliant_input("CA");
        i.adverse_action_type = AdverseActionType::None;
        let r = check(&i);
        assert!(!r.federal_fcra_notice_required);
        assert!(r.overall_compliant);
        assert!(r.note.contains("No adverse action taken"));
    }

    #[test]
    fn adverse_action_without_consumer_report_no_fcra_trigger() {
        let mut i = fully_compliant_input("CA");
        i.consumer_report_contributed_to_decision = false;
        let r = check(&i);
        assert!(!r.federal_fcra_notice_required);
        assert!(r.overall_compliant);
        assert!(r.note.contains("not triggered"));
    }

    // Coverage / structural pins.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(codes.len(), 51, "expected 50 states + DC, got {}", codes.len());
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn state_adds_regime_only_3_states() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == AdverseActionRegime::StateAddsRequirements {
                count += 1;
            }
        }
        assert_eq!(count, 3, "expected CA + WA + NY only on StateAddsRequirements");
    }

    #[test]
    fn only_ca_has_formatting_requirements() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.state_requires_formatting_standards {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected CA only with formatting requirements");
    }

    #[test]
    fn unknown_state_falls_back_to_fcra_only() {
        let r = check(&fcra_minimum_input("XX"));
        assert_eq!(r.regime, AdverseActionRegime::FederalFcraOnly);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&fcra_minimum_input("ca"));
        assert!(!r.overall_compliant); // CA requires state additions
    }

    // Adverse action types.

    #[test]
    fn cosigner_required_triggers_notice() {
        let mut i = fully_compliant_input("TX");
        i.adverse_action_type = AdverseActionType::CosignerRequired;
        let r = check(&i);
        assert!(r.federal_fcra_notice_required);
        assert!(r.overall_compliant);
    }

    #[test]
    fn higher_rent_triggers_notice() {
        let mut i = fully_compliant_input("TX");
        i.adverse_action_type = AdverseActionType::HigherRent;
        let r = check(&i);
        assert!(r.federal_fcra_notice_required);
    }

    // Note text.

    #[test]
    fn note_compliant_includes_regime() {
        let r = check(&fully_compliant_input("CA"));
        assert!(r.note.contains("StateAddsRequirements"));
    }

    #[test]
    fn note_violation_lists_missing_elements() {
        let mut i = fcra_minimum_input("CA");
        i.notice_includes_cra_name_address_phone = false;
        let r = check(&i);
        assert!(r.note.contains("VIOLATION"));
        assert!(r.note.contains("CRA name/address/phone"));
    }
}
