//! State cosigner / lease guarantor enforcement rules.
//!
//! When a landlord-trader accepts a cosigner / guarantor on a
//! residential lease, two key questions arise:
//!
//! 1. Is the cosigner liable for RENEWALS / EXTENSIONS of the
//!    original lease, or only the original term?
//! 2. Does the landlord owe the cosigner statutory notice BEFORE
//!    pursuing collection or reporting adverse credit information?
//!
//! These questions are largely governed by common-law surety
//! principles and the specific text of the guaranty agreement. The
//! key analytical distinction is between a **CONTINUING guaranty**
//! (covers original term + all renewals automatically) and a
//! **SPECIFIC-TERM guaranty** (covers only the original term;
//! cosigner must re-sign for each renewal).
//!
//! Two state-specific regimes layer on top of the common law:
//!
//! - `IllinoisStatutoryNoticeRequired` — Illinois (815 ILCS 505/2S).
//!   Before reporting adverse information to a consumer reporting
//!   agency, providing information to a collection agency, or taking
//!   any collection action against a cosigner, the landlord MUST
//!   notify the cosigner by **first class mail** that (a) the
//!   primary obligor has become delinquent or defaulted, (b) the
//!   cosigner is responsible for payment, AND (c) the cosigner has
//!   **15 days** from the date the notice was sent to pay or make
//!   arrangements. Violation = unlawful practice + civil damages up
//!   to **$250 + reasonable attorney's fees**.
//!
//! - `CommonLawSuretyRules` — all other states + DC. Continuing-vs-
//!   specific-term guaranty doctrine governs renewal liability under
//!   restatement of suretyship + state statutory law (e.g., CA Civ.
//!   Code § 2787-2856 surety provisions). No state-mandated pre-
//!   collection notice to the cosigner.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CosignerRegime {
    IllinoisStatutoryNoticeRequired,
    CommonLawSuretyRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuarantyType {
    ContinuingGuaranty,
    SpecificTermGuaranty,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: CosignerRegime,
    /// Statutory minimum days between landlord notice and collection
    /// action when notice is required.
    pub minimum_notice_days_before_collection: u32,
    /// Statutory damages for failure to provide notice.
    pub statutory_damages_dollars: i64,
    pub attorney_fees_recoverable: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: CosignerRegime,
    minimum_notice_days_before_collection: u32,
    statutory_damages_dollars: i64,
    attorney_fees_recoverable: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        minimum_notice_days_before_collection,
        statutory_damages_dollars,
        attorney_fees_recoverable,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use CosignerRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "IL",
        rule(
            IllinoisStatutoryNoticeRequired,
            15,
            250,
            true,
            "815 ILCS 505/2S — first-class-mail notice required to cosigner before adverse-credit reporting or collection action; 15-day cure period; $250 statutory damages + attorney fees on violation",
        ),
    );

    // CommonLawSuretyRules for all other states + DC.
    let common_law = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DC", "DE", "FL",
        "GA", "HI", "ID", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
        "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
        "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
    ];
    for code in common_law {
        let citation: &'static str = if code == "CA" {
            "Cal. Civ. Code § 2787-2856 — common-law suretyship governs; continuing guaranty covers renewals if written as such; no state-mandated pre-collection notice"
        } else if code == "NY" {
            "N.Y. — common-law suretyship; continuing guaranty doctrine; no state-mandated pre-collection notice (general consumer protection statutes apply)"
        } else {
            "Common-law suretyship: continuing-vs-specific-term guaranty doctrine governs renewal liability; no state-mandated pre-collection notice"
        };
        m.insert(code, rule(CommonLawSuretyRules, 0, 0, false, citation));
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosignerInput {
    pub state_code: String,
    pub guaranty_type: GuarantyType,
    pub lease_renewed: bool,
    pub cosigner_signed_renewal: bool,
    /// True if landlord initiated collection action or adverse-credit
    /// reporting against the cosigner.
    pub collection_action_initiated: bool,
    /// True if landlord sent first-class-mail notice to the cosigner
    /// before initiating collection.
    pub landlord_sent_notice_before_collection: bool,
    pub days_between_notice_and_collection: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosignerResult {
    pub regime: CosignerRegime,
    pub cosigner_liable_for_current_term: bool,
    pub cosigner_liable_for_renewal_term: bool,
    pub statutory_notice_required: bool,
    pub statutory_notice_compliant: bool,
    pub statutory_damages_exposure_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &CosignerInput) -> CosignerResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: CosignerRegime::CommonLawSuretyRules,
        minimum_notice_days_before_collection: 0,
        statutory_damages_dollars: 0,
        attorney_fees_recoverable: false,
        citation: "Unknown state code; common-law suretyship assumed",
    });

    // Cosigner always liable for current/original lease term.
    let liable_current = true;

    // Renewal liability turns on guaranty type.
    let liable_renewal = if !input.lease_renewed {
        false
    } else {
        match input.guaranty_type {
            GuarantyType::ContinuingGuaranty => true,
            GuarantyType::SpecificTermGuaranty => input.cosigner_signed_renewal,
        }
    };

    // Statutory notice analysis under IL regime.
    let (notice_required, notice_compliant, exposure) = match rule.regime {
        CosignerRegime::IllinoisStatutoryNoticeRequired
            if input.collection_action_initiated =>
        {
            let req = true;
            let compliant = input.landlord_sent_notice_before_collection
                && input.days_between_notice_and_collection
                    >= rule.minimum_notice_days_before_collection;
            let exp = if compliant {
                0
            } else {
                rule.statutory_damages_dollars
            };
            (req, compliant, exp)
        }
        _ => (false, true, 0),
    };

    let note = match (rule.regime, input.lease_renewed, input.guaranty_type) {
        (CosignerRegime::IllinoisStatutoryNoticeRequired, _, _) => {
            let mut parts: Vec<String> = Vec::new();
            parts.push(format!(
                "IllinoisStatutoryNoticeRequired (815 ILCS 505/2S): cosigner liable current term; renewal liability = {} ({:?}).",
                liable_renewal, input.guaranty_type,
            ));
            if input.collection_action_initiated {
                if notice_compliant {
                    parts.push(format!(
                        "Pre-collection notice SENT {} days before collection (≥ {} required); compliant.",
                        input.days_between_notice_and_collection,
                        rule.minimum_notice_days_before_collection,
                    ));
                } else {
                    parts.push(format!(
                        "Pre-collection notice VIOLATION: landlord exposure ${} + attorney fees.",
                        exposure,
                    ));
                }
            }
            parts.join(" ")
        }
        (CosignerRegime::CommonLawSuretyRules, true, GuarantyType::ContinuingGuaranty) =>
            "CommonLawSuretyRules: continuing guaranty extends cosigner liability through renewals automatically.".to_string(),
        (CosignerRegime::CommonLawSuretyRules, true, GuarantyType::SpecificTermGuaranty) => format!(
            "CommonLawSuretyRules: specific-term guaranty; cosigner liable for renewal only if cosigner re-signed ({}).",
            if input.cosigner_signed_renewal { "RE-SIGNED" } else { "DID NOT RE-SIGN — no renewal liability" },
        ),
        (CosignerRegime::CommonLawSuretyRules, false, _) =>
            "CommonLawSuretyRules: no renewal occurred; cosigner liable for original term only.".to_string(),
    };

    CosignerResult {
        regime: rule.regime,
        cosigner_liable_for_current_term: liable_current,
        cosigner_liable_for_renewal_term: liable_renewal,
        statutory_notice_required: notice_required,
        statutory_notice_compliant: notice_compliant,
        statutory_damages_exposure_dollars: exposure,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, gtype: GuarantyType) -> CosignerInput {
        CosignerInput {
            state_code: state.to_string(),
            guaranty_type: gtype,
            lease_renewed: false,
            cosigner_signed_renewal: false,
            collection_action_initiated: false,
            landlord_sent_notice_before_collection: false,
            days_between_notice_and_collection: 0,
        }
    }

    // Renewal liability — common law (CA / NY / TX / others).

    #[test]
    fn continuing_guaranty_covers_renewal_automatically() {
        let mut i = input("CA", GuarantyType::ContinuingGuaranty);
        i.lease_renewed = true;
        let r = check(&i);
        assert_eq!(r.regime, CosignerRegime::CommonLawSuretyRules);
        assert!(r.cosigner_liable_for_renewal_term);
    }

    #[test]
    fn specific_term_guaranty_without_resign_no_renewal_liability() {
        let mut i = input("CA", GuarantyType::SpecificTermGuaranty);
        i.lease_renewed = true;
        i.cosigner_signed_renewal = false;
        let r = check(&i);
        assert!(!r.cosigner_liable_for_renewal_term);
        assert!(r.note.contains("DID NOT RE-SIGN"));
    }

    #[test]
    fn specific_term_guaranty_with_resign_covers_renewal() {
        let mut i = input("CA", GuarantyType::SpecificTermGuaranty);
        i.lease_renewed = true;
        i.cosigner_signed_renewal = true;
        let r = check(&i);
        assert!(r.cosigner_liable_for_renewal_term);
    }

    #[test]
    fn no_renewal_cosigner_liable_only_for_original_term() {
        let i = input("CA", GuarantyType::SpecificTermGuaranty);
        let r = check(&i);
        assert!(r.cosigner_liable_for_current_term);
        assert!(!r.cosigner_liable_for_renewal_term);
    }

    // IL statutory notice regime.

    #[test]
    fn il_collection_without_notice_violates() {
        let mut i = input("IL", GuarantyType::ContinuingGuaranty);
        i.collection_action_initiated = true;
        i.landlord_sent_notice_before_collection = false;
        let r = check(&i);
        assert_eq!(r.regime, CosignerRegime::IllinoisStatutoryNoticeRequired);
        assert!(r.statutory_notice_required);
        assert!(!r.statutory_notice_compliant);
        assert_eq!(r.statutory_damages_exposure_dollars, 250);
    }

    #[test]
    fn il_collection_with_notice_15_days_compliant() {
        let mut i = input("IL", GuarantyType::ContinuingGuaranty);
        i.collection_action_initiated = true;
        i.landlord_sent_notice_before_collection = true;
        i.days_between_notice_and_collection = 15;
        let r = check(&i);
        assert!(r.statutory_notice_compliant);
        assert_eq!(r.statutory_damages_exposure_dollars, 0);
    }

    #[test]
    fn il_collection_14_days_before_notice_violates_window() {
        let mut i = input("IL", GuarantyType::ContinuingGuaranty);
        i.collection_action_initiated = true;
        i.landlord_sent_notice_before_collection = true;
        i.days_between_notice_and_collection = 14;
        let r = check(&i);
        assert!(!r.statutory_notice_compliant);
        assert_eq!(r.statutory_damages_exposure_dollars, 250);
    }

    #[test]
    fn il_no_collection_no_notice_required() {
        let i = input("IL", GuarantyType::ContinuingGuaranty);
        let r = check(&i);
        assert!(!r.statutory_notice_required);
        assert!(r.statutory_notice_compliant);
    }

    #[test]
    fn il_violation_note_describes_damages_exposure() {
        let mut i = input("IL", GuarantyType::ContinuingGuaranty);
        i.collection_action_initiated = true;
        let r = check(&i);
        assert!(r.note.contains("VIOLATION"));
        assert!(r.note.contains("$250"));
    }

    // Common-law states have no statutory notice requirement.

    #[test]
    fn ca_collection_without_notice_no_violation() {
        let mut i = input("CA", GuarantyType::ContinuingGuaranty);
        i.collection_action_initiated = true;
        i.landlord_sent_notice_before_collection = false;
        let r = check(&i);
        assert!(!r.statutory_notice_required);
        assert!(r.statutory_notice_compliant);
        assert_eq!(r.statutory_damages_exposure_dollars, 0);
    }

    #[test]
    fn ny_common_law_path() {
        let r = check(&input("NY", GuarantyType::ContinuingGuaranty));
        assert_eq!(r.regime, CosignerRegime::CommonLawSuretyRules);
        assert!(r.citation.contains("N.Y."));
    }

    #[test]
    fn ca_citation_mentions_civ_code() {
        let r = check(&input("CA", GuarantyType::ContinuingGuaranty));
        assert!(r.citation.contains("Cal. Civ. Code § 2787-2856"));
    }

    // Coverage / invariants.

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
    fn only_il_uses_statutory_notice_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == CosignerRegime::IllinoisStatutoryNoticeRequired {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected IL only with IllinoisStatutoryNoticeRequired");
    }

    #[test]
    fn only_il_has_statutory_damages() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.statutory_damages_dollars > 0 {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected IL only with statutory damages");
    }

    #[test]
    fn il_15_day_window_pinned() {
        let il = RULES.get("IL").unwrap();
        assert_eq!(il.minimum_notice_days_before_collection, 15);
    }

    #[test]
    fn il_statutory_damages_250_dollars_pinned() {
        let il = RULES.get("IL").unwrap();
        assert_eq!(il.statutory_damages_dollars, 250);
    }

    #[test]
    fn il_attorney_fees_recoverable() {
        let il = RULES.get("IL").unwrap();
        assert!(il.attorney_fees_recoverable);
    }

    #[test]
    fn unknown_state_falls_back_to_common_law() {
        let r = check(&input("XX", GuarantyType::ContinuingGuaranty));
        assert_eq!(r.regime, CosignerRegime::CommonLawSuretyRules);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = input("il", GuarantyType::ContinuingGuaranty);
        i.collection_action_initiated = true;
        let r = check(&i);
        assert!(!r.statutory_notice_compliant);
    }

    // Notes.

    #[test]
    fn continuing_guaranty_note_describes_automatic_coverage() {
        let mut i = input("CA", GuarantyType::ContinuingGuaranty);
        i.lease_renewed = true;
        let r = check(&i);
        assert!(r.note.contains("continuing guaranty extends"));
    }

    #[test]
    fn no_renewal_note_describes_original_term_only() {
        let r = check(&input("CA", GuarantyType::ContinuingGuaranty));
        assert!(r.note.contains("original term only"));
    }
}
