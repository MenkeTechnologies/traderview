//! State quiet enjoyment / nuisance statutory compliance.
//!
//! The covenant of quiet enjoyment is implied into every residential
//! lease at common law in every U.S. state — the landlord covenants
//! that the tenant will have undisturbed use and enjoyment of the
//! premises. A breach can take the form of: physical entry without
//! cause, harassment, failure to address ongoing nuisance from other
//! tenants, lockout, utility shutoff, or substantial habitability
//! defects making the premises unsuitable.
//!
//! Most state remedies are the common-law set: constructive eviction
//! (tenant moves out + lease termination + actual damages), abatement
//! of rent, and tort damages. **Massachusetts is the unique outlier**
//! with its codified statutory remedy under G.L. c. 186 § 14 that
//! provides TREBLE damages + criminal exposure for a landlord's
//! intentional breach.
//!
//! Two regimes:
//!
//! `MassachusettsTrebleDamagesAndCriminal`: MA only (G.L. c. 186
//! § 14). Tenant may recover the GREATER of actual damages or
//! 3 × monthly rent. Criminal penalty: $25-$300 fine plus possible
//! imprisonment up to 6 months. Uniquely aggressive remedy that
//! makes intentional landlord harassment in MA a high-stakes risk
//! ([Mass. G.L. c. 186 § 14](https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section14)).
//!
//! `CommonLawImpliedCovenant`: all other states + DC. Common-law
//! implied covenant; remedies include constructive eviction, rent
//! abatement, and actual damages. California codified the covenant
//! at Civ. Code § 1927 but follows common-law remedies; NY and IL
//! handle through related implied warranty of habitability statutes
//! (RPL § 235-b in NY).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuietEnjoymentRegime {
    MassachusettsTrebleDamagesAndCriminal,
    CommonLawImpliedCovenant,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: QuietEnjoymentRegime,
    /// Multiplier on monthly rent for statutory damages floor (3 for MA's
    /// treble rule; 0 for common-law states).
    pub statutory_damages_rent_multiplier: u32,
    pub criminal_exposure_possible: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: QuietEnjoymentRegime,
    statutory_damages_rent_multiplier: u32,
    criminal_exposure_possible: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        statutory_damages_rent_multiplier,
        criminal_exposure_possible,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use QuietEnjoymentRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "MA",
        rule(
            MassachusettsTrebleDamagesAndCriminal,
            3,
            true,
            "Mass. G.L. c. 186 § 14 — interference with quiet enjoyment; damages = greater of actual or 3 × monthly rent; criminal $25-$300 fine + up to 6 months jail",
        ),
    );

    // Common-law states.
    let common_law = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA",
        "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
        "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN",
        "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
    ];
    for code in common_law {
        let citation: &'static str = if code == "CA" {
            "Cal. Civ. Code § 1927 — statutory covenant of quiet enjoyment implied in every lease; common-law remedies (constructive eviction + actual damages)"
        } else if code == "NY" {
            "N.Y. common-law quiet enjoyment + RPL § 235-b implied warranty of habitability; common-law remedies"
        } else if code == "IL" {
            "765 ILCS 705 implied covenant + common-law quiet enjoyment; remedies via constructive eviction and damages"
        } else {
            "Common-law implied covenant of quiet enjoyment; remedies: constructive eviction, rent abatement, actual damages"
        };
        m.insert(code, rule(CommonLawImpliedCovenant, 0, false, citation));
    }
    m.insert("CA", rule(CommonLawImpliedCovenant, 0, false, "Cal. Civ. Code § 1927 — statutory covenant of quiet enjoyment implied in every lease; common-law remedies (constructive eviction + actual damages)"));
    m.insert("NY", rule(CommonLawImpliedCovenant, 0, false, "N.Y. common-law quiet enjoyment + RPL § 235-b implied warranty of habitability; common-law remedies"));
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietEnjoymentInput {
    pub state_code: String,
    pub breach_alleged: bool,
    /// True if the breach is "substantial" — a mere minor inconvenience
    /// does not rise to actionable breach in any state.
    pub breach_substantial: bool,
    /// True if the breach was intentional / wilful (MA criminal
    /// exposure requires intent).
    pub breach_intentional: bool,
    pub monthly_rent_dollars: i64,
    pub actual_damages_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietEnjoymentResult {
    pub regime: QuietEnjoymentRegime,
    pub breach_actionable: bool,
    pub statutory_damages_floor_dollars: i64,
    pub recoverable_damages_dollars: i64,
    pub criminal_exposure_possible: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &QuietEnjoymentInput) -> QuietEnjoymentResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: QuietEnjoymentRegime::CommonLawImpliedCovenant,
        statutory_damages_rent_multiplier: 0,
        criminal_exposure_possible: false,
        citation: "Unknown state code; common-law covenant assumed",
    });

    let actionable = input.breach_alleged && input.breach_substantial;

    let statutory_floor = if actionable && rule.statutory_damages_rent_multiplier > 0 {
        (rule.statutory_damages_rent_multiplier as i64) * input.monthly_rent_dollars
    } else {
        0
    };

    let recoverable = if !actionable {
        0
    } else {
        // MA: greater of actual or 3 × rent.
        // Common-law: actual damages.
        input.actual_damages_dollars.max(statutory_floor)
    };

    let criminal = rule.criminal_exposure_possible
        && actionable
        && input.breach_intentional;

    let note = match (rule.regime, actionable) {
        (_, false) => {
            if !input.breach_alleged {
                "No breach alleged; quiet enjoyment compliance not triggered.".to_string()
            } else {
                "Breach alleged but NOT substantial; common-law and statutory remedies require substantial interference with use and enjoyment.".to_string()
            }
        }
        (QuietEnjoymentRegime::MassachusettsTrebleDamagesAndCriminal, true) => format!(
            "MA G.L. c. 186 § 14: substantial breach; tenant recovers GREATER of actual damages ${} or 3× monthly rent ${} = ${}.{}",
            input.actual_damages_dollars, statutory_floor, recoverable,
            if criminal {
                " Intentional breach triggers $25-$300 fine + possible 6-month jail."
            } else { "" },
        ),
        (QuietEnjoymentRegime::CommonLawImpliedCovenant, true) => format!(
            "CommonLawImpliedCovenant: substantial breach; tenant recovers actual damages ${}. Tenant may also pursue constructive eviction / rent abatement / lease termination.",
            recoverable,
        ),
    };

    QuietEnjoymentResult {
        regime: rule.regime,
        breach_actionable: actionable,
        statutory_damages_floor_dollars: statutory_floor,
        recoverable_damages_dollars: recoverable,
        criminal_exposure_possible: criminal,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> QuietEnjoymentInput {
        QuietEnjoymentInput {
            state_code: state.to_string(),
            breach_alleged: true,
            breach_substantial: true,
            breach_intentional: false,
            monthly_rent_dollars: 2_000,
            actual_damages_dollars: 1_000,
        }
    }

    // MA — treble damages.

    #[test]
    fn ma_substantial_breach_yields_3x_rent_floor() {
        // Actual $1k vs 3 × $2k = $6k → recover $6k.
        let r = check(&input("MA"));
        assert_eq!(r.regime, QuietEnjoymentRegime::MassachusettsTrebleDamagesAndCriminal);
        assert!(r.breach_actionable);
        assert_eq!(r.statutory_damages_floor_dollars, 6_000);
        assert_eq!(r.recoverable_damages_dollars, 6_000);
    }

    #[test]
    fn ma_actual_higher_than_3x_recovers_actual() {
        // Actual $10k > 3 × $2k → recover $10k.
        let mut i = input("MA");
        i.actual_damages_dollars = 10_000;
        let r = check(&i);
        assert_eq!(r.recoverable_damages_dollars, 10_000);
    }

    #[test]
    fn ma_intentional_breach_triggers_criminal() {
        let mut i = input("MA");
        i.breach_intentional = true;
        let r = check(&i);
        assert!(r.criminal_exposure_possible);
        assert!(r.note.contains("possible 6-month jail"));
    }

    #[test]
    fn ma_non_intentional_breach_no_criminal() {
        let r = check(&input("MA"));
        assert!(!r.criminal_exposure_possible);
    }

    #[test]
    fn ma_minor_breach_not_actionable() {
        let mut i = input("MA");
        i.breach_substantial = false;
        let r = check(&i);
        assert!(!r.breach_actionable);
        assert_eq!(r.recoverable_damages_dollars, 0);
    }

    // Common-law states.

    #[test]
    fn ca_substantial_breach_actual_damages_only() {
        let r = check(&input("CA"));
        assert_eq!(r.regime, QuietEnjoymentRegime::CommonLawImpliedCovenant);
        assert!(r.breach_actionable);
        assert_eq!(r.statutory_damages_floor_dollars, 0);
        assert_eq!(r.recoverable_damages_dollars, 1_000);
    }

    #[test]
    fn ca_citation_mentions_1927() {
        let r = check(&input("CA"));
        assert!(r.citation.contains("§ 1927"));
    }

    #[test]
    fn ny_citation_mentions_235_b() {
        let r = check(&input("NY"));
        assert!(r.citation.contains("RPL § 235-b"));
    }

    #[test]
    fn il_citation_mentions_765_ilcs() {
        let r = check(&input("IL"));
        assert!(r.citation.contains("765 ILCS"));
    }

    #[test]
    fn tx_common_law_substantial_breach_actual() {
        let r = check(&input("TX"));
        assert_eq!(r.regime, QuietEnjoymentRegime::CommonLawImpliedCovenant);
        assert!(r.breach_actionable);
        assert_eq!(r.recoverable_damages_dollars, 1_000);
    }

    #[test]
    fn ca_no_criminal_exposure_even_with_intent() {
        let mut i = input("CA");
        i.breach_intentional = true;
        let r = check(&i);
        assert!(!r.criminal_exposure_possible);
    }

    // No breach.

    #[test]
    fn no_breach_alleged_no_actionable() {
        let mut i = input("MA");
        i.breach_alleged = false;
        let r = check(&i);
        assert!(!r.breach_actionable);
        assert_eq!(r.recoverable_damages_dollars, 0);
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
    fn only_ma_uses_treble_damages_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == QuietEnjoymentRegime::MassachusettsTrebleDamagesAndCriminal {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected MA only with treble damages");
    }

    #[test]
    fn only_ma_has_criminal_exposure() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.criminal_exposure_possible {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected MA only with criminal exposure");
    }

    #[test]
    fn only_ma_has_rent_multiplier() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.statutory_damages_rent_multiplier > 0 {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected MA only with statutory rent multiplier");
    }

    #[test]
    fn unknown_state_falls_back_to_common_law() {
        let r = check(&input("XX"));
        assert_eq!(r.regime, QuietEnjoymentRegime::CommonLawImpliedCovenant);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ma"));
        assert_eq!(r.recoverable_damages_dollars, 6_000);
    }

    // Notes.

    #[test]
    fn ma_note_describes_greater_of_actual_or_3x() {
        let r = check(&input("MA"));
        assert!(r.note.contains("GREATER of actual"));
        assert!(r.note.contains("3× monthly rent"));
    }

    #[test]
    fn common_law_note_describes_constructive_eviction() {
        let r = check(&input("CA"));
        assert!(r.note.contains("constructive eviction"));
    }
}
