//! State-by-state radon disclosure + testing compliance.
//!
//! Sibling to `lead_disclosure`, `bedbug_disclosure`, and
//! `mold_disclosure`. EPA-recommended action level is **4.0 pCi/L**
//! (picocuries per liter); levels at/above this are recommended for
//! mitigation though EPA cannot mandate landlord action. State statutes
//! vary in scope:
//!
//! 1. **IL Radon Awareness Act (420 ILCS 46/, eff. Jan 1, 2024)** —
//!    most comprehensive landlord-specific radon law in the country.
//!    Applies to residential units on the **2nd floor or lower**.
//!    Landlord must provide at application (before lease signing) OR
//!    on tenant request:
//!
//!    - Radon Guide for Tenants pamphlet
//!    - Disclosure of Information on Radon Hazards form
//!    - Copies of any radon test records from past 2 years
//!
//!    Tenant has 90 days from lease start to conduct own test, with
//!    10-day window to share results with landlord.
//!
//! 2. **ME 14 M.R.S. § 6030-D** — mandatory landlord TESTING regime
//!    (strongest in country). Landlord must test each building, provide
//!    written notice with results, and notify tenant of right to test.
//!    If radon ≥ 4.0 pCi/L is NOT mitigated, EITHER party may end the
//!    lease with 30 days' notice.
//!
//! 3. **NJ N.J.S.A. § 26:2D-71** — seller/landlord notice required;
//!    radon levels disclosure in real-estate transactions.
//!
//! 4. **FL Fla. Stat. § 404.056** — generic "radon gas warning"
//!    required in all rental lease agreements (statutory language).
//!
//! 5. **IA Iowa Code § 558A, MN Minn. Stat. § 144.496, OR ORS § 105.435**
//!    — radon disclosure in real-estate transfer (sale-focused but
//!    often extended by lease practice to rental).
//!
//! 6. **No statewide statute** — most other states. EPA pamphlet
//!    practice common but not legally required.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RadonRegime {
    /// IL model: comprehensive disclosure on 2nd floor or lower units.
    ComprehensiveLandlordDisclosure,
    /// ME model: landlord mandatory testing + lease-termination right
    /// at 4.0 pCi/L.
    MandatoryTestingAndTerminationRight,
    /// Lease-level disclosure warning required (FL model).
    LeaseLevelWarning,
    /// Real-estate-transfer disclosure (sale-focused; often extended to
    /// rental by lease practice but not strictly required).
    RealEstateTransferDisclosure,
    /// No statewide statute.
    NoStateStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRadonRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: RadonRegime,
    /// True if landlord must provide a radon information pamphlet
    /// (IL, NJ).
    pub landlord_pamphlet_required: bool,
    /// True if landlord must conduct radon testing (ME unique).
    pub landlord_testing_required: bool,
    /// True if statute allows tenant to terminate lease when radon
    /// ≥ 4.0 pCi/L is not mitigated (ME unique).
    pub tenant_termination_right_at_action_level: bool,
    /// True if landlord must provide records of prior radon tests
    /// (IL: 2-year window; ME: all records).
    pub prior_test_records_required: bool,
    /// Tenant's window to conduct own test after lease start. `None` if
    /// state doesn't grant testing right.
    pub tenant_testing_window_days: Option<u32>,
    /// EPA action threshold in picocuries per liter (universal 4.0).
    pub action_level_pcil: Decimal,
    pub citation: &'static str,
}

use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadonDisclosureInput {
    pub state_code: String,
    /// Floor of the rental unit (1 = ground, 2 = second, etc.). IL
    /// limits coverage to 2nd floor or lower (some basements count as 1).
    pub unit_floor: u32,
    pub landlord_provided_pamphlet: bool,
    pub landlord_provided_disclosure_form: bool,
    pub landlord_provided_prior_test_records: bool,
    pub landlord_completed_required_testing: bool,
    pub current_radon_level_pcil: Decimal,
    pub mitigation_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadonDisclosureResult {
    pub disclosure_required: bool,
    pub complies: bool,
    pub violations: Vec<String>,
    /// True if current radon level meets/exceeds EPA action level (4.0).
    pub at_or_above_action_level: bool,
    /// True if tenant has lease-termination right under §6030-D-style
    /// regime due to unmitigated radon at/above action level.
    pub tenant_termination_right_available: bool,
    pub no_statute_in_state: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateRadonRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateRadonRule> {
    let mut v: Vec<&'static StateRadonRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &RadonDisclosureInput) -> RadonDisclosureResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return RadonDisclosureResult {
                disclosure_required: false,
                complies: false,
                violations: vec!["unknown state code".to_string()],
                at_or_above_action_level: false,
                tenant_termination_right_available: false,
                no_statute_in_state: true,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let no_statute = matches!(rule.regime, RadonRegime::NoStateStatute);
    if no_statute {
        return RadonDisclosureResult {
            disclosure_required: false,
            complies: true,
            violations: vec![],
            at_or_above_action_level: input.current_radon_level_pcil >= rule.action_level_pcil,
            tenant_termination_right_available: false,
            no_statute_in_state: true,
            citation: rule.citation,
            note: format!(
                "{}: no statewide radon disclosure statute — EPA pamphlet practice common but not legally required",
                rule.state_name
            ),
        };
    }

    // IL covers only floors 1-2 ("2nd floor or lower"). Skip compliance
    // checks for higher floors.
    let il_floor_excluded =
        matches!(rule.regime, RadonRegime::ComprehensiveLandlordDisclosure) && input.unit_floor > 2;
    if il_floor_excluded {
        return RadonDisclosureResult {
            disclosure_required: false,
            complies: true,
            violations: vec![],
            at_or_above_action_level: input.current_radon_level_pcil >= rule.action_level_pcil,
            tenant_termination_right_available: false,
            no_statute_in_state: false,
            citation: rule.citation,
            note: format!(
                "{}: unit on floor {} above 2nd-floor coverage threshold — radon disclosure not required for upper floors",
                rule.state_name, input.unit_floor
            ),
        };
    }

    let mut violations: Vec<String> = Vec::new();

    if rule.landlord_pamphlet_required && !input.landlord_provided_pamphlet {
        violations.push(format!(
            "{} requires landlord to provide radon pamphlet (Radon Guide for Tenants); not provided",
            rule.state_name
        ));
    }
    if matches!(rule.regime, RadonRegime::ComprehensiveLandlordDisclosure)
        && !input.landlord_provided_disclosure_form
    {
        violations.push(format!(
            "{} requires Disclosure of Information on Radon Hazards form; not provided",
            rule.state_name
        ));
    }
    if rule.prior_test_records_required && !input.landlord_provided_prior_test_records {
        violations.push(format!(
            "{} requires landlord to provide prior radon test records; not provided",
            rule.state_name
        ));
    }
    if rule.landlord_testing_required && !input.landlord_completed_required_testing {
        violations.push(format!(
            "{} requires landlord to conduct radon testing; not completed",
            rule.state_name
        ));
    }

    let at_or_above = input.current_radon_level_pcil >= rule.action_level_pcil;
    let termination_right =
        rule.tenant_termination_right_at_action_level && at_or_above && !input.mitigation_completed;

    let complies = violations.is_empty();
    let note = if complies && termination_right {
        format!(
            "{}: disclosure compliance satisfied; radon at {} pCi/L (≥ {} action level) AND unmitigated — TENANT TERMINATION RIGHT available with 30 days notice",
            rule.state_name,
            input.current_radon_level_pcil,
            rule.action_level_pcil
        )
    } else if complies {
        format!(
            "{}: radon disclosure requirements satisfied",
            rule.state_name
        )
    } else {
        format!(
            "{}: {} radon-disclosure violation(s)",
            rule.state_name,
            violations.len()
        )
    };

    RadonDisclosureResult {
        disclosure_required: true,
        complies,
        violations,
        at_or_above_action_level: at_or_above,
        tenant_termination_right_available: termination_right,
        no_statute_in_state: false,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: RadonRegime,
    landlord_pamphlet_required: bool,
    landlord_testing_required: bool,
    tenant_termination_right_at_action_level: bool,
    prior_test_records_required: bool,
    tenant_testing_window_days: Option<u32>,
    citation: &'static str,
) -> StateRadonRule {
    use rust_decimal::Decimal as D;
    StateRadonRule {
        state_code,
        state_name,
        regime,
        landlord_pamphlet_required,
        landlord_testing_required,
        tenant_termination_right_at_action_level,
        prior_test_records_required,
        tenant_testing_window_days,
        action_level_pcil: "4.0".parse::<D>().unwrap_or(Decimal::ZERO),
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateRadonRule>> = Lazy::new(|| {
    use RadonRegime::*;
    static RULES: Lazy<Vec<StateRadonRule>> = Lazy::new(|| {
        vec![
            rule(
                "AK",
                "Alaska",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "AL",
                "Alabama",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "AR",
                "Arkansas",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "AZ",
                "Arizona",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "CA",
                "California",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "CO",
                "Colorado",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "CT",
                "Connecticut",
                RealEstateTransferDisclosure,
                false,
                false,
                false,
                false,
                None,
                "Conn. Gen. Stat. § 47a-7b (real estate transfer disclosure)",
            ),
            rule(
                "DC",
                "District of Columbia",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "DE",
                "Delaware",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "FL",
                "Florida",
                LeaseLevelWarning,
                false,
                false,
                false,
                false,
                None,
                "Fla. Stat. § 404.056 (radon gas warning required in lease)",
            ),
            rule(
                "GA",
                "Georgia",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "HI",
                "Hawaii",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "IA",
                "Iowa",
                RealEstateTransferDisclosure,
                false,
                false,
                false,
                false,
                None,
                "Iowa Code § 558A (real estate transfer disclosure)",
            ),
            rule(
                "ID",
                "Idaho",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "IL",
                "Illinois",
                ComprehensiveLandlordDisclosure,
                true,
                false,
                false,
                true,
                Some(90),
                "420 ILCS 46/ (Radon Awareness Act eff. 2024-01-01)",
            ),
            rule(
                "IN",
                "Indiana",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "KS",
                "Kansas",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "KY",
                "Kentucky",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "LA",
                "Louisiana",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "MA",
                "Massachusetts",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "MD",
                "Maryland",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "ME",
                "Maine",
                MandatoryTestingAndTerminationRight,
                false,
                true,
                true,
                true,
                Some(30),
                "14 M.R.S. § 6030-D (mandatory testing + termination right ≥ 4.0 pCi/L)",
            ),
            rule(
                "MI",
                "Michigan",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "MN",
                "Minnesota",
                RealEstateTransferDisclosure,
                false,
                false,
                false,
                false,
                None,
                "Minn. Stat. § 144.496",
            ),
            rule(
                "MO",
                "Missouri",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "MS",
                "Mississippi",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "MT",
                "Montana",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "NC",
                "North Carolina",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "ND",
                "North Dakota",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "NE",
                "Nebraska",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "NH",
                "New Hampshire",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "NJ",
                "New Jersey",
                RealEstateTransferDisclosure,
                true,
                false,
                false,
                false,
                None,
                "N.J.S.A. § 26:2D-71 (radon notice in seller/landlord disclosure)",
            ),
            rule(
                "NM",
                "New Mexico",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "NV",
                "Nevada",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "NY",
                "New York",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "OH",
                "Ohio",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "OK",
                "Oklahoma",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "OR",
                "Oregon",
                RealEstateTransferDisclosure,
                false,
                false,
                false,
                false,
                None,
                "ORS § 105.435",
            ),
            rule(
                "PA",
                "Pennsylvania",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "RI",
                "Rhode Island",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "SC",
                "South Carolina",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "SD",
                "South Dakota",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "TN",
                "Tennessee",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "TX",
                "Texas",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "UT",
                "Utah",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "VA",
                "Virginia",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "VT",
                "Vermont",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "WA",
                "Washington",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "WI",
                "Wisconsin",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "WV",
                "West Virginia",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
            rule(
                "WY",
                "Wyoming",
                NoStateStatute,
                false,
                false,
                false,
                false,
                None,
                "no statewide statute",
            ),
        ]
    });
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn input(state: &str, floor: u32, level: Decimal) -> RadonDisclosureInput {
        RadonDisclosureInput {
            state_code: state.to_string(),
            unit_floor: floor,
            landlord_provided_pamphlet: true,
            landlord_provided_disclosure_form: true,
            landlord_provided_prior_test_records: true,
            landlord_completed_required_testing: true,
            current_radon_level_pcil: level,
            mitigation_completed: false,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn il_floor_1_full_disclosure_required_compliant() {
        let r = check(&input("IL", 1, dec!(2.0)));
        assert!(r.disclosure_required);
        assert!(r.complies);
    }

    #[test]
    fn il_missing_pamphlet_violates() {
        let mut i = input("IL", 1, dec!(2.0));
        i.landlord_provided_pamphlet = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("radon pamphlet")));
    }

    #[test]
    fn il_missing_disclosure_form_violates() {
        let mut i = input("IL", 1, dec!(2.0));
        i.landlord_provided_disclosure_form = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Disclosure of Information")));
    }

    #[test]
    fn il_missing_prior_test_records_violates() {
        let mut i = input("IL", 1, dec!(2.0));
        i.landlord_provided_prior_test_records = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("prior radon test records")));
    }

    #[test]
    fn il_floor_3_above_coverage_threshold() {
        // IL covers 2nd floor or lower. 3rd floor unit is exempt.
        let mut i = input("IL", 3, dec!(2.0));
        i.landlord_provided_pamphlet = false; // would normally violate
        let r = check(&i);
        assert!(r.complies);
        assert!(!r.disclosure_required);
        assert!(r.note.contains("above 2nd-floor coverage"));
    }

    #[test]
    fn il_floor_2_at_coverage_boundary() {
        // 2nd floor exactly is covered.
        let r = check(&input("IL", 2, dec!(2.0)));
        assert!(r.disclosure_required);
    }

    #[test]
    fn me_mandatory_testing_required() {
        let mut i = input("ME", 1, dec!(2.0));
        i.landlord_completed_required_testing = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("conduct radon testing")));
    }

    #[test]
    fn me_action_level_unmitigated_triggers_termination_right() {
        // ME: ≥ 4.0 pCi/L AND not mitigated → tenant lease-termination right.
        let i = input("ME", 1, dec!(4.5));
        let r = check(&i);
        assert!(r.complies);
        assert!(r.at_or_above_action_level);
        assert!(r.tenant_termination_right_available);
        assert!(r.note.contains("TENANT TERMINATION RIGHT"));
    }

    #[test]
    fn me_action_level_with_mitigation_no_termination_right() {
        let mut i = input("ME", 1, dec!(4.5));
        i.mitigation_completed = true;
        let r = check(&i);
        assert!(!r.tenant_termination_right_available);
    }

    #[test]
    fn me_below_action_level_no_termination_right() {
        let i = input("ME", 1, dec!(3.9));
        let r = check(&i);
        assert!(!r.at_or_above_action_level);
        assert!(!r.tenant_termination_right_available);
    }

    #[test]
    fn me_exact_action_level_4_0_triggers() {
        let i = input("ME", 1, dec!(4.0));
        let r = check(&i);
        assert!(r.at_or_above_action_level);
        assert!(r.tenant_termination_right_available);
    }

    #[test]
    fn fl_lease_level_warning_regime() {
        // FL has LeaseLevelWarning regime. Default input meets all
        // requirements (no specific flags fail).
        let r = check(&input("FL", 1, dec!(2.0)));
        assert!(r.disclosure_required);
        assert!(r.complies);
    }

    #[test]
    fn nj_pamphlet_required() {
        let mut i = input("NJ", 1, dec!(2.0));
        i.landlord_provided_pamphlet = false;
        let r = check(&i);
        assert!(!r.complies);
    }

    #[test]
    fn no_statute_states_always_comply() {
        for code in ["TX", "CA", "AZ", "GA", "WA", "OR"] {
            // OR is actually RealEstateTransferDisclosure
            if code == "OR" {
                continue;
            }
            let mut i = input(code, 1, dec!(2.0));
            i.landlord_provided_pamphlet = false;
            let r = check(&i);
            assert!(r.complies, "{code} should comply (no statute)");
            assert!(r.no_statute_in_state);
        }
    }

    #[test]
    fn unknown_state_handled() {
        let r = check(&input("ZZ", 1, dec!(2.0)));
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("IL").is_some());
        assert!(lookup("il").is_some());
    }

    #[test]
    fn all_states_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} citation empty", r.state_code);
        }
    }

    #[test]
    fn il_only_state_with_comprehensive_disclosure_regime() {
        let il = lookup("IL").unwrap();
        assert!(matches!(
            il.regime,
            RadonRegime::ComprehensiveLandlordDisclosure
        ));
        for r in TABLE.values() {
            if r.state_code != "IL" {
                assert!(
                    !matches!(r.regime, RadonRegime::ComprehensiveLandlordDisclosure),
                    "{} should not have ComprehensiveLandlordDisclosure regime",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn me_only_state_with_mandatory_testing_and_termination_right() {
        let me = lookup("ME").unwrap();
        assert!(matches!(
            me.regime,
            RadonRegime::MandatoryTestingAndTerminationRight
        ));
        for r in TABLE.values() {
            if r.state_code != "ME" {
                assert!(
                    !matches!(r.regime, RadonRegime::MandatoryTestingAndTerminationRight),
                    "{} should not have ME-mandatory regime",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn epa_action_level_universal_4_pcil() {
        // Every row uses 4.0 pCi/L action level (EPA standard).
        for r in TABLE.values() {
            assert_eq!(r.action_level_pcil, dec!(4.0), "{}", r.state_code);
        }
    }

    #[test]
    fn il_floor_0_basement_covered() {
        // Basement (floor 0) is below 2nd floor → covered.
        let r = check(&input("IL", 0, dec!(2.0)));
        assert!(r.disclosure_required);
    }
}
