//! Federal Fair Housing Act + state service animal / emotional support
//! animal (ESA) accommodation compliance.
//!
//! Common landlord compliance trap. Federal FHA mandates reasonable
//! accommodation for assistance animals regardless of any "no pets"
//! policy and prohibits pet deposits/fees. State additions impose
//! anti-fraud documentation requirements after several years of
//! fraudulent "emotional support" letters from online mill providers.
//!
//! **Federal floor (universal)**:
//!
//!   - **42 U.S.C. § 3604(f)(3)(B)**: reasonable accommodation required
//!     for tenants with disabilities, including for assistance animals.
//!   - **24 CFR § 100.202**: assistance animals are NOT pets and are
//!     not subject to pet rules, pet deposits, or pet fees.
//!   - **Two-question rule**: landlord may ask (1) does the tenant have
//!     a disability? AND (2) is the animal needed because of the
//!     disability? Landlord may NOT inquire about the specific
//!     disability.
//!   - **Documentation**: landlord may request reliable documentation
//!     ONLY when the disability is not obvious. Letter from doctor /
//!     therapist / mental health professional acceptable.
//!
//! **State anti-fraud additions** (recent legislative wave 2020-2022):
//!
//!   - **FL SB 1084 (2020)** — landlord may require proof of licensing
//!     and vaccination compliance. Documentation must come from
//!     federal/state/local agency or licensed healthcare practitioner
//!     (no online mill letters). Fraudulent ESA documentation is a
//!     misdemeanor.
//!   - **VA Code § 36-96.3:1** — "therapeutic relationship" requirement
//!     for ESA letters; health care providers issuing fraudulent
//!     documentation face Virginia Consumer Protection Act penalties.
//!   - **CA AB 468 (2022)** — 30-day relationship with licensed mental
//!     health professional required before ESA letter is valid.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssistanceAnimalRegime {
    /// Federal FHA floor only — no state additions.
    FederalFloorOnly,
    /// State adds fraud-prevention requirements (licensed practitioner,
    /// therapeutic relationship period, vaccination proof).
    StateAddsFraudPrevention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateServiceAnimalRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: AssistanceAnimalRegime,
    /// Minimum days of therapeutic relationship required before an ESA
    /// letter is valid (CA 30 days under AB 468). `None` if state has
    /// no such requirement.
    pub therapeutic_relationship_days_required: Option<u32>,
    /// True if state requires ESA documentation to come from a licensed
    /// professional (FL, VA, CA).
    pub licensed_practitioner_required: bool,
    /// True if state explicitly authorizes landlord to require proof of
    /// vaccination compliance for assistance animals (FL SB 1084).
    pub vaccination_proof_allowed: bool,
    /// True if state criminalizes or penalizes fraudulent ESA
    /// documentation (FL misdemeanor, VA Consumer Protection Act).
    pub fraud_documentation_penalty: bool,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAnimalCheckInput {
    pub state_code: String,
    pub is_service_animal_or_esa: bool,
    /// True if the tenant's disability is obvious or already known to
    /// the landlord (no documentation request permitted).
    pub disability_obvious_or_known: bool,
    pub esa_documentation_provided: bool,
    pub documentation_from_licensed_practitioner: bool,
    /// Days of therapeutic relationship between tenant and the ESA
    /// letter's issuing professional. `None` if not provided.
    pub days_therapeutic_relationship_before_letter: Option<u32>,
    pub landlord_charging_pet_deposit: bool,
    pub landlord_charging_pet_fee_or_rent: bool,
    pub vaccination_proof_requested_by_landlord: bool,
    pub vaccination_proof_provided: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAnimalCheckResult {
    pub reasonable_accommodation_required: bool,
    pub pet_deposit_prohibited: bool,
    pub pet_fee_prohibited: bool,
    /// True if the documentation provided is sufficient under the
    /// applicable regime (federal + state).
    pub documentation_sufficient: bool,
    /// `Some(true)` if state requires therapeutic relationship and the
    /// provided days satisfy the threshold; `Some(false)` if state
    /// requires but threshold not met; `None` if state has no
    /// requirement.
    pub therapeutic_relationship_satisfied: Option<bool>,
    pub landlord_complies: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateServiceAnimalRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateServiceAnimalRule> {
    let mut v: Vec<&'static StateServiceAnimalRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &ServiceAnimalCheckInput) -> ServiceAnimalCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return ServiceAnimalCheckResult {
                reasonable_accommodation_required: false,
                pet_deposit_prohibited: false,
                pet_fee_prohibited: false,
                documentation_sufficient: false,
                therapeutic_relationship_satisfied: None,
                landlord_complies: false,
                violations: vec!["unknown state code".to_string()],
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    // Federal FHA: reasonable accommodation + no pet fees, universal.
    let accommodation_required = input.is_service_animal_or_esa;
    let deposit_prohibited = accommodation_required;
    let fee_prohibited = accommodation_required;

    // Documentation sufficiency.
    let doc_sufficient = if !accommodation_required || input.disability_obvious_or_known {
        true
    } else if !input.esa_documentation_provided {
        false
    } else {
        !rule.licensed_practitioner_required || input.documentation_from_licensed_practitioner
    };

    // Therapeutic relationship check (state-specific).
    let therapeutic_satisfied = match rule.therapeutic_relationship_days_required {
        Some(threshold) => {
            // Only applies when documentation is being relied on (i.e.,
            // disability not obvious + ESA documentation provided).
            if input.disability_obvious_or_known || !input.esa_documentation_provided {
                None
            } else {
                Some(
                    input
                        .days_therapeutic_relationship_before_letter
                        .map(|d| d >= threshold)
                        .unwrap_or(false),
                )
            }
        }
        None => None,
    };

    // Build violations list (only relevant when assistance animal present).
    let mut violations: Vec<String> = Vec::new();
    if accommodation_required {
        if input.landlord_charging_pet_deposit {
            violations.push(format!(
                "{} + federal FHA: landlord charging pet deposit for assistance animal — prohibited under 24 CFR § 100.202",
                rule.state_name
            ));
        }
        if input.landlord_charging_pet_fee_or_rent {
            violations.push(format!(
                "{} + federal FHA: landlord charging pet fee/rent for assistance animal — prohibited",
                rule.state_name
            ));
        }
        if !input.disability_obvious_or_known && !input.esa_documentation_provided {
            violations.push(format!(
                "{}: disability not obvious AND no ESA documentation provided — tenant must provide reliable documentation",
                rule.state_name
            ));
        }
        if rule.licensed_practitioner_required
            && input.esa_documentation_provided
            && !input.documentation_from_licensed_practitioner
        {
            violations.push(format!(
                "{}: state requires ESA documentation from a licensed practitioner — provided documentation does not meet requirement",
                rule.state_name
            ));
        }
        if let (Some(threshold), Some(false)) = (
            rule.therapeutic_relationship_days_required,
            therapeutic_satisfied,
        ) {
            violations.push(format!(
                "{}: state requires {}-day therapeutic relationship before ESA letter — not satisfied",
                rule.state_name, threshold
            ));
        }
        if rule.vaccination_proof_allowed
            && input.vaccination_proof_requested_by_landlord
            && !input.vaccination_proof_provided
        {
            violations.push(format!(
                "{}: state allows landlord to require vaccination proof; not provided",
                rule.state_name
            ));
        }
    }

    let landlord_complies = violations.is_empty()
        || violations.iter().all(|v| {
            // Tenant-side violations (missing documentation, vaccination
            // proof) don't make the LANDLORD non-compliant.
            v.contains("documentation") || v.contains("vaccination") || v.contains("therapeutic")
        });

    let note = if !accommodation_required {
        format!(
            "{}: input not classified as service animal or ESA — pet rules apply",
            rule.state_name
        )
    } else if violations.is_empty() {
        format!(
            "{}: assistance animal accommodation requirements satisfied",
            rule.state_name
        )
    } else {
        format!(
            "{}: {} accommodation compliance issue(s) — see violations",
            rule.state_name,
            violations.len()
        )
    };

    ServiceAnimalCheckResult {
        reasonable_accommodation_required: accommodation_required,
        pet_deposit_prohibited: deposit_prohibited,
        pet_fee_prohibited: fee_prohibited,
        documentation_sufficient: doc_sufficient,
        therapeutic_relationship_satisfied: therapeutic_satisfied,
        landlord_complies,
        violations,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: AssistanceAnimalRegime,
    therapeutic_relationship_days_required: Option<u32>,
    licensed_practitioner_required: bool,
    vaccination_proof_allowed: bool,
    fraud_documentation_penalty: bool,
    citation: &'static str,
) -> StateServiceAnimalRule {
    StateServiceAnimalRule {
        state_code,
        state_name,
        regime,
        therapeutic_relationship_days_required,
        licensed_practitioner_required,
        vaccination_proof_allowed,
        fraud_documentation_penalty,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateServiceAnimalRule>> = Lazy::new(|| {
    use AssistanceAnimalRegime::*;
    static RULES: &[StateServiceAnimalRule] = &[
        rule("AK", "Alaska", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("AL", "Alabama", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("AR", "Arkansas", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("AZ", "Arizona", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule(
            "CA",
            "California",
            StateAddsFraudPrevention,
            Some(30),
            true,
            false,
            false,
            "Cal. Civ. Code § 54.2 + AB 468 (2022 - 30-day therapeutic relationship)",
        ),
        rule("CO", "Colorado", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("CT", "Connecticut", FederalFloorOnly, None, false, false, false, "Conn. Gen. Stat. § 46a-44"),
        rule("DC", "District of Columbia", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("DE", "Delaware", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule(
            "FL",
            "Florida",
            StateAddsFraudPrevention,
            None,
            true,
            true,
            true,
            "Fla. Stat. § 760.27 (SB 1084 - 2020)",
        ),
        rule("GA", "Georgia", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("HI", "Hawaii", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("IA", "Iowa", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("ID", "Idaho", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("IL", "Illinois", FederalFloorOnly, None, false, false, false, "740 ILCS 13 (Service Animals)"),
        rule("IN", "Indiana", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("KS", "Kansas", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("KY", "Kentucky", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("LA", "Louisiana", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("MA", "Massachusetts", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("MD", "Maryland", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("ME", "Maine", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("MI", "Michigan", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("MN", "Minnesota", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule(
            "MT",
            "Montana",
            StateAddsFraudPrevention,
            None,
            true,
            false,
            true,
            "Mont. Code § 49-2-101 (2021 fraud penalties)",
        ),
        rule("MO", "Missouri", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("MS", "Mississippi", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("NC", "North Carolina", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("ND", "North Dakota", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("NE", "Nebraska", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("NH", "New Hampshire", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("NJ", "New Jersey", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("NM", "New Mexico", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("NV", "Nevada", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("NY", "New York", FederalFloorOnly, None, false, false, false, "NYC Human Rights Law"),
        rule("OH", "Ohio", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("OK", "Oklahoma", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("OR", "Oregon", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("PA", "Pennsylvania", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("RI", "Rhode Island", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("SC", "South Carolina", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("SD", "South Dakota", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("TN", "Tennessee", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("TX", "Texas", FederalFloorOnly, None, false, false, false, "Tex. Hum. Res. Code § 121.003"),
        rule("UT", "Utah", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule(
            "VA",
            "Virginia",
            StateAddsFraudPrevention,
            None,
            true,
            false,
            true,
            "Va. Code § 36-96.3:1 (HB 1242 - 2020)",
        ),
        rule("VT", "Vermont", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("WA", "Washington", FederalFloorOnly, None, false, false, false, "RCW § 49.60.222"),
        rule("WI", "Wisconsin", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("WV", "West Virginia", FederalFloorOnly, None, false, false, false, "federal FHA only"),
        rule("WY", "Wyoming", FederalFloorOnly, None, false, false, false, "federal FHA only"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn base(state: &str) -> ServiceAnimalCheckInput {
        ServiceAnimalCheckInput {
            state_code: state.to_string(),
            is_service_animal_or_esa: true,
            disability_obvious_or_known: false,
            esa_documentation_provided: true,
            documentation_from_licensed_practitioner: true,
            days_therapeutic_relationship_before_letter: Some(60),
            landlord_charging_pet_deposit: false,
            landlord_charging_pet_fee_or_rent: false,
            vaccination_proof_requested_by_landlord: false,
            vaccination_proof_provided: false,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn federal_floor_pet_deposit_prohibited() {
        // FHA universal — pet deposit prohibited for assistance animal.
        let mut i = base("TX");
        i.landlord_charging_pet_deposit = true;
        let r = check(&i);
        assert!(!r.violations.is_empty());
        assert!(r.violations.iter().any(|v| v.contains("pet deposit")));
        assert!(r.pet_deposit_prohibited);
    }

    #[test]
    fn federal_floor_pet_fee_prohibited() {
        let mut i = base("TX");
        i.landlord_charging_pet_fee_or_rent = true;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("pet fee")));
        assert!(r.pet_fee_prohibited);
    }

    #[test]
    fn obvious_disability_no_documentation_required() {
        // FHA: when disability is obvious, no documentation request
        // permitted. Compute reports documentation_sufficient: true even
        // without ESA letter.
        let mut i = base("TX");
        i.disability_obvious_or_known = true;
        i.esa_documentation_provided = false;
        let r = check(&i);
        assert!(r.documentation_sufficient);
    }

    #[test]
    fn ca_ab468_30_day_therapeutic_relationship_satisfied() {
        // CA AB 468: 30-day relationship required. 60 days = satisfied.
        let r = check(&base("CA"));
        assert_eq!(r.therapeutic_relationship_satisfied, Some(true));
    }

    #[test]
    fn ca_ab468_29_days_not_satisfied() {
        let mut i = base("CA");
        i.days_therapeutic_relationship_before_letter = Some(29);
        let r = check(&i);
        assert_eq!(r.therapeutic_relationship_satisfied, Some(false));
        assert!(r.violations.iter().any(|v| v.contains("30-day therapeutic")));
    }

    #[test]
    fn ca_30_days_exact_boundary_satisfied() {
        let mut i = base("CA");
        i.days_therapeutic_relationship_before_letter = Some(30);
        let r = check(&i);
        assert_eq!(r.therapeutic_relationship_satisfied, Some(true));
    }

    #[test]
    fn ca_no_therapeutic_check_when_disability_obvious() {
        // When disability is obvious, no documentation request; the
        // therapeutic relationship requirement does not apply.
        let mut i = base("CA");
        i.disability_obvious_or_known = true;
        i.days_therapeutic_relationship_before_letter = Some(1);
        let r = check(&i);
        assert_eq!(r.therapeutic_relationship_satisfied, None);
    }

    #[test]
    fn fl_sb1084_licensed_practitioner_required() {
        // FL: documentation must come from licensed practitioner. If
        // letter from a non-licensed source → violation.
        let mut i = base("FL");
        i.documentation_from_licensed_practitioner = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("licensed practitioner")));
    }

    #[test]
    fn fl_sb1084_vaccination_proof_required_when_landlord_requests() {
        // FL allows landlord to require vaccination proof. Not provided
        // when requested → violation.
        let mut i = base("FL");
        i.vaccination_proof_requested_by_landlord = true;
        i.vaccination_proof_provided = false;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("vaccination proof")));
    }

    #[test]
    fn fl_no_vaccination_violation_when_landlord_doesnt_request() {
        let mut i = base("FL");
        i.vaccination_proof_requested_by_landlord = false;
        i.vaccination_proof_provided = false;
        let r = check(&i);
        assert!(!r.violations.iter().any(|v| v.contains("vaccination proof")));
    }

    #[test]
    fn tx_no_vaccination_check_even_when_requested() {
        // TX is federal floor only; vaccination proof not explicitly
        // authorized by state. Landlord requesting + tenant not
        // providing should NOT produce a vaccination violation.
        let mut i = base("TX");
        i.vaccination_proof_requested_by_landlord = true;
        i.vaccination_proof_provided = false;
        let r = check(&i);
        assert!(!r.violations.iter().any(|v| v.contains("vaccination proof")));
    }

    #[test]
    fn va_hb1242_licensed_practitioner_required() {
        let mut i = base("VA");
        i.documentation_from_licensed_practitioner = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("licensed practitioner")));
    }

    #[test]
    fn no_animal_no_accommodation_required() {
        // If input is not service/ESA, no accommodation required (pet
        // rules apply normally).
        let mut i = base("CA");
        i.is_service_animal_or_esa = false;
        i.landlord_charging_pet_deposit = true; // would normally violate
        let r = check(&i);
        assert!(!r.reasonable_accommodation_required);
        assert!(!r.pet_deposit_prohibited);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn assistance_animal_full_compliance_passes() {
        let r = check(&base("TX"));
        assert!(r.reasonable_accommodation_required);
        assert!(r.documentation_sufficient);
        assert!(r.landlord_complies);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn unknown_state_handled() {
        let i = base("ZZ");
        let r = check(&i);
        assert!(!r.violations.is_empty());
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
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
    fn state_adds_fraud_prevention_states_pinned() {
        // CA, FL, VA, MT have StateAddsFraudPrevention regime.
        for code in ["CA", "FL", "VA", "MT"] {
            let r = lookup(code).unwrap();
            assert!(
                matches!(r.regime, AssistanceAnimalRegime::StateAddsFraudPrevention),
                "{code} should be StateAddsFraudPrevention"
            );
        }
    }

    #[test]
    fn ca_only_state_with_30_day_therapeutic_relationship() {
        // CA AB 468 is uniquely on the 30-day therapeutic relationship.
        let ca = lookup("CA").unwrap();
        assert_eq!(ca.therapeutic_relationship_days_required, Some(30));
        for r in TABLE.values() {
            if r.state_code != "CA" {
                assert!(
                    r.therapeutic_relationship_days_required.is_none(),
                    "{} should not have therapeutic_relationship_days_required",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn fl_only_state_with_vaccination_proof_explicit() {
        // FL SB 1084 explicitly authorizes vaccination proof.
        let fl = lookup("FL").unwrap();
        assert!(fl.vaccination_proof_allowed);
        for r in TABLE.values() {
            if r.state_code != "FL" {
                assert!(
                    !r.vaccination_proof_allowed,
                    "{} should not have vaccination_proof_allowed",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn fraud_penalty_states_pinned() {
        // FL, VA, MT all penalize fraudulent ESA documentation.
        for code in ["FL", "VA", "MT"] {
            let r = lookup(code).unwrap();
            assert!(r.fraud_documentation_penalty, "{code}");
        }
    }

    #[test]
    fn multiple_violations_stack() {
        let mut i = base("CA");
        i.landlord_charging_pet_deposit = true;
        i.landlord_charging_pet_fee_or_rent = true;
        i.days_therapeutic_relationship_before_letter = Some(10);
        let r = check(&i);
        assert!(r.violations.len() >= 3);
    }

    #[test]
    fn missing_documentation_when_disability_not_obvious() {
        // Tenant has assistance animal but no documentation and
        // disability not obvious → violation noting need for docs.
        let mut i = base("TX");
        i.esa_documentation_provided = false;
        i.disability_obvious_or_known = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("ESA documentation")));
        assert!(!r.documentation_sufficient);
    }
}
