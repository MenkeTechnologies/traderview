//! State landlord-required renters insurance compliance table.
//!
//! Every state currently allows landlords to require tenants to carry
//! renters insurance as a lease condition. The variation is in the
//! statutory ceilings on coverage amounts the landlord may demand,
//! restrictions on naming the landlord as an additional insured, and
//! low-income tenant carveouts.
//!
//! Two regimes:
//!
//! 1. **`StatutoryCapWithLowIncomeExemption`** — Oregon (ORS 90.222)
//!    is currently the strictest framework in the country:
//!    - Coverage cap: $100,000 per occurrence (or the "customary
//!      amount" required by landlords for similar properties in the
//!      same rental market, whichever is greater).
//!    - Landlord may NOT be named as an additional insured (may be
//!      named as having an interest for lapse-notice purposes only).
//!    - Low-income exemption: landlord cannot require insurance from
//!      a tenant whose household income is ≤ 50% of area median
//!      income (AMI), adjusted for family size up to 5 persons.
//!
//! 2. **`GenerallyAllowedNoStateCap`** — most other US states. The
//!    landlord may require renters insurance as a lease condition with
//!    no specific statutory cap on the coverage amount. Common-law
//!    "reasonableness" applies as a backstop.
//!
//! No US state currently PROHIBITS landlords from requiring renters
//! insurance entirely.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsuranceRegime {
    StatutoryCapWithLowIncomeExemption,
    GenerallyAllowedNoStateCap,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: InsuranceRegime,
    /// Maximum liability coverage the landlord may require. `None`
    /// means no statutory cap.
    pub liability_coverage_cap_dollars: Option<i64>,
    /// True if the state explicitly prohibits naming landlord as an
    /// additional insured (but typically allows naming as interest
    /// for lapse-notice purposes).
    pub landlord_additional_insured_prohibited: bool,
    /// Low-income exemption threshold as % of Area Median Income.
    /// `None` if no exemption.
    pub low_income_exemption_pct_ami: Option<u32>,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: InsuranceRegime,
    liability_coverage_cap_dollars: Option<i64>,
    landlord_additional_insured_prohibited: bool,
    low_income_exemption_pct_ami: Option<u32>,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        liability_coverage_cap_dollars,
        landlord_additional_insured_prohibited,
        low_income_exemption_pct_ami,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use InsuranceRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // Oregon — the unique statutory-cap regime.
    m.insert(
        "OR",
        rule(
            StatutoryCapWithLowIncomeExemption,
            Some(100_000),
            true,  // Cannot be named as additional insured
            Some(50), // 50% AMI low-income exemption
            "Or. ORS 90.222 — $100k liability cap; no additional-insured naming; ≤50% AMI exemption",
        ),
    );

    // All other states + DC: GenerallyAllowedNoStateCap.
    let no_cap_states = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN",
        "IA", "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH",
        "NJ", "NM", "NY", "NC", "ND", "OH", "OK", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT",
        "VA", "WA", "WV", "WI", "WY",
    ];
    for code in no_cap_states {
        m.insert(
            code,
            rule(
                GenerallyAllowedNoStateCap,
                None,
                false,
                None,
                "No state-specific cap on landlord renters-insurance requirements; common-law reasonableness applies",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentersInsuranceInput {
    pub state_code: String,
    pub landlord_requires_insurance: bool,
    pub required_liability_coverage_dollars: i64,
    pub landlord_named_as_additional_insured: bool,
    pub landlord_named_as_interest_for_notice_only: bool,
    /// Tenant household income as % of Area Median Income (0-100+).
    /// Used for low-income exemption check.
    pub tenant_household_income_pct_ami: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentersInsuranceResult {
    pub regime: InsuranceRegime,
    pub effective_coverage_cap_dollars: Option<i64>,
    pub coverage_exceeds_state_cap: bool,
    pub additional_insured_violation: bool,
    pub low_income_exemption_applies: bool,
    pub landlord_may_require_for_this_tenant: bool,
    pub overall_compliant: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &RentersInsuranceInput) -> RentersInsuranceResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: InsuranceRegime::GenerallyAllowedNoStateCap,
        liability_coverage_cap_dollars: None,
        landlord_additional_insured_prohibited: false,
        low_income_exemption_pct_ami: None,
        citation: "Unknown state code; assuming generally-allowed-no-cap",
    });

    // Coverage cap check.
    let coverage_exceeds = match rule.liability_coverage_cap_dollars {
        Some(cap) => input.required_liability_coverage_dollars > cap,
        None => false,
    };

    // Additional-insured violation.
    let additional_insured_violation =
        rule.landlord_additional_insured_prohibited && input.landlord_named_as_additional_insured;

    // Low-income exemption.
    let low_income_exemption = rule
        .low_income_exemption_pct_ami
        .is_some_and(|threshold| input.tenant_household_income_pct_ami <= threshold);

    let may_require = if low_income_exemption {
        // Exemption: landlord may NOT require insurance from this tenant.
        false
    } else {
        true
    };

    let overall_compliant = !coverage_exceeds
        && !additional_insured_violation
        && (!input.landlord_requires_insurance || may_require);

    let note = match rule.regime {
        InsuranceRegime::StatutoryCapWithLowIncomeExemption => {
            let mut issues: Vec<String> = Vec::new();
            if coverage_exceeds {
                issues.push(format!(
                    "coverage ${} exceeds state cap ${}",
                    input.required_liability_coverage_dollars,
                    rule.liability_coverage_cap_dollars.unwrap_or(0),
                ));
            }
            if additional_insured_violation {
                issues.push(
                    "landlord named as additional insured (prohibited; may only be named as interest for lapse notice)"
                        .to_string(),
                );
            }
            if low_income_exemption && input.landlord_requires_insurance {
                issues.push(format!(
                    "tenant household income {}% AMI ≤ {}% exemption threshold; landlord MAY NOT require insurance from this tenant",
                    input.tenant_household_income_pct_ami,
                    rule.low_income_exemption_pct_ami.unwrap_or(0),
                ));
            }
            if issues.is_empty() {
                "StatutoryCapWithLowIncomeExemption: compliant — coverage within cap, landlord not named as additional insured, no low-income exemption triggered.".to_string()
            } else {
                format!(
                    "StatutoryCapWithLowIncomeExemption VIOLATION: {}.",
                    issues.join("; ")
                )
            }
        }
        InsuranceRegime::GenerallyAllowedNoStateCap => {
            "GenerallyAllowedNoStateCap: state permits landlord to require renters insurance with no specific statutory cap; common-law reasonableness applies.".to_string()
        }
    };

    RentersInsuranceResult {
        regime: rule.regime,
        effective_coverage_cap_dollars: rule.liability_coverage_cap_dollars,
        coverage_exceeds_state_cap: coverage_exceeds,
        additional_insured_violation,
        low_income_exemption_applies: low_income_exemption,
        landlord_may_require_for_this_tenant: may_require,
        overall_compliant,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> RentersInsuranceInput {
        RentersInsuranceInput {
            state_code: state.to_string(),
            landlord_requires_insurance: true,
            required_liability_coverage_dollars: 50_000,
            landlord_named_as_additional_insured: false,
            landlord_named_as_interest_for_notice_only: true,
            tenant_household_income_pct_ami: 80,
        }
    }

    // Oregon — strictest regime.

    #[test]
    fn or_at_100k_cap_exact_complies() {
        let mut i = input("OR");
        i.required_liability_coverage_dollars = 100_000;
        let r = check(&i);
        assert_eq!(
            r.regime,
            InsuranceRegime::StatutoryCapWithLowIncomeExemption
        );
        assert!(!r.coverage_exceeds_state_cap);
        assert!(r.overall_compliant);
    }

    #[test]
    fn or_101k_coverage_exceeds_cap() {
        let mut i = input("OR");
        i.required_liability_coverage_dollars = 100_001;
        let r = check(&i);
        assert!(r.coverage_exceeds_state_cap);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn or_additional_insured_named_violation() {
        let mut i = input("OR");
        i.landlord_named_as_additional_insured = true;
        let r = check(&i);
        assert!(r.additional_insured_violation);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn or_landlord_as_interest_only_complies() {
        let mut i = input("OR");
        i.landlord_named_as_additional_insured = false;
        i.landlord_named_as_interest_for_notice_only = true;
        let r = check(&i);
        assert!(!r.additional_insured_violation);
        assert!(r.overall_compliant);
    }

    #[test]
    fn or_low_income_50_pct_exemption_blocks_requirement() {
        let mut i = input("OR");
        i.tenant_household_income_pct_ami = 50;
        let r = check(&i);
        assert!(r.low_income_exemption_applies);
        assert!(!r.landlord_may_require_for_this_tenant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn or_low_income_49_pct_exempts() {
        let mut i = input("OR");
        i.tenant_household_income_pct_ami = 49;
        let r = check(&i);
        assert!(r.low_income_exemption_applies);
    }

    #[test]
    fn or_low_income_51_pct_no_exemption() {
        let mut i = input("OR");
        i.tenant_household_income_pct_ami = 51;
        let r = check(&i);
        assert!(!r.low_income_exemption_applies);
        assert!(r.landlord_may_require_for_this_tenant);
    }

    #[test]
    fn or_low_income_exemption_does_not_fire_if_landlord_does_not_require() {
        // If landlord doesn't require insurance, exemption is irrelevant
        // to compliance.
        let mut i = input("OR");
        i.landlord_requires_insurance = false;
        i.tenant_household_income_pct_ami = 30;
        let r = check(&i);
        assert!(r.low_income_exemption_applies); // Still flags the AMI status
        assert!(r.overall_compliant); // But overall complies (nothing required)
    }

    #[test]
    fn or_all_three_violations_stack() {
        let mut i = input("OR");
        i.required_liability_coverage_dollars = 200_000;
        i.landlord_named_as_additional_insured = true;
        i.tenant_household_income_pct_ami = 40;
        let r = check(&i);
        assert!(r.coverage_exceeds_state_cap);
        assert!(r.additional_insured_violation);
        assert!(r.low_income_exemption_applies);
        assert!(!r.overall_compliant);
    }

    // General-allowed states.

    #[test]
    fn ca_no_state_cap_any_amount_complies() {
        let mut i = input("CA");
        i.required_liability_coverage_dollars = 500_000;
        let r = check(&i);
        assert_eq!(r.regime, InsuranceRegime::GenerallyAllowedNoStateCap);
        assert_eq!(r.effective_coverage_cap_dollars, None);
        assert!(!r.coverage_exceeds_state_cap);
        assert!(r.overall_compliant);
    }

    #[test]
    fn tx_landlord_named_as_additional_insured_complies() {
        // TX has no statutory restriction.
        let mut i = input("TX");
        i.landlord_named_as_additional_insured = true;
        let r = check(&i);
        assert!(!r.additional_insured_violation);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ny_low_income_tenant_no_exemption() {
        // NY has no statutory low-income exemption.
        let mut i = input("NY");
        i.tenant_household_income_pct_ami = 30;
        let r = check(&i);
        assert!(!r.low_income_exemption_applies);
        assert!(r.landlord_may_require_for_this_tenant);
    }

    #[test]
    fn fl_complies_with_high_coverage_requirement() {
        let mut i = input("FL");
        i.required_liability_coverage_dollars = 1_000_000;
        let r = check(&i);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ok_no_state_cap_despite_renters_insurance_misconception() {
        // OK does NOT prohibit landlords from requiring renters insurance
        // (Okla. Stat. tit. 41 § 113 — allowed). Pin against the
        // common misconception that Oklahoma forbids the requirement.
        let mut i = input("OK");
        i.landlord_requires_insurance = true;
        let r = check(&i);
        assert_eq!(r.regime, InsuranceRegime::GenerallyAllowedNoStateCap);
        assert!(r.overall_compliant);
    }

    // Coverage / structural pins.

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
    fn or_unique_statutory_cap_regime_invariant() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == InsuranceRegime::StatutoryCapWithLowIncomeExemption {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "expected OR only with StatutoryCapWithLowIncomeExemption regime"
        );
    }

    #[test]
    fn only_or_has_additional_insured_prohibition() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.landlord_additional_insured_prohibited {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "expected OR only with additional_insured_prohibited"
        );
    }

    #[test]
    fn only_or_has_low_income_exemption() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.low_income_exemption_pct_ami.is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected OR only with low_income_exemption");
    }

    #[test]
    fn unknown_state_falls_back_to_no_cap() {
        let r = check(&input("XX"));
        assert_eq!(r.regime, InsuranceRegime::GenerallyAllowedNoStateCap);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = input("or");
        i.required_liability_coverage_dollars = 100_001;
        let r = check(&i);
        assert!(r.coverage_exceeds_state_cap);
    }

    // Note text.

    #[test]
    fn or_compliant_note_describes_state_path() {
        let r = check(&input("OR"));
        assert!(r
            .note
            .contains("StatutoryCapWithLowIncomeExemption: compliant"));
    }

    #[test]
    fn or_violation_note_lists_each_issue() {
        let mut i = input("OR");
        i.required_liability_coverage_dollars = 200_000;
        i.landlord_named_as_additional_insured = true;
        i.tenant_household_income_pct_ami = 40;
        let r = check(&i);
        assert!(r.note.contains("VIOLATION"));
        assert!(r.note.contains("exceeds state cap"));
        assert!(r.note.contains("additional insured"));
        assert!(r.note.contains("AMI"));
    }

    #[test]
    fn general_allowed_note_describes_no_cap() {
        let r = check(&input("CA"));
        assert!(r.note.contains("GenerallyAllowedNoStateCap"));
        assert!(r.note.contains("no specific statutory cap"));
    }
}
