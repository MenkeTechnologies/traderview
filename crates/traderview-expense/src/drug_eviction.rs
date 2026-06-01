//! Federal HUD One-Strike + state drug-related eviction compliance.
//!
//! Drug-related eviction sits at the intersection of federal HUD
//! authority (for public housing and Section 8) and state just-cause
//! eviction statutes (for private market). The federal rule under
//! **42 U.S.C. § 1437d(l)(6)** is the famous "One-Strike" rule —
//! authorizing eviction of any tenant whose household member, guest,
//! or person under their control engages in drug-related criminal
//! activity, regardless of the tenant's personal knowledge or
//! culpability (per *Dept. of Housing & Urban Development v. Rucker*,
//! 535 U.S. 125 (2002)).
//!
//! **Federal floor under 42 U.S.C. § 1437d(l)(6)** (public housing)
//! and 42 U.S.C. § 1437f (Section 8 vouchers and certificates):
//!
//! - Any criminal activity threatening health, safety, or peaceful
//!   enjoyment of the premises by other tenants, OR
//! - Any drug-related criminal activity ON OR OFF the premises
//!   (public housing) / ON OR NEAR the premises (Section 8)
//! - Engaged in by a "covered person": tenant, household member,
//!   guest, or person under tenant's control
//! - Shall be cause for termination of tenancy
//!
//! Originally enacted in the Anti-Drug Abuse Act of 1988. **Rucker
//! (2002)** confirmed strict liability — tenant need not be aware of
//! the household member's or guest's activity.
//!
//! **HUD policy on mitigating circumstances**: while the statute
//! does NOT require mitigation analysis, HUD policy ENCOURAGES PHAs
//! to consider individualized circumstances. The 2024 "Reducing
//! Barriers to HUD-Assisted Housing" proposed rule would further
//! require mitigation analysis.
//!
//! Three regimes for state private-market overlay:
//!
//! `StateJustCauseListsCriminalActivity`: CA (Tenant Protection Act
//! 2019, Civ. Code § 1946.2 — drug activity is enumerated just cause),
//! OR (SB 608, ORS Ch. 90 — drug-related criminal activity is just
//! cause after first year), WA (RCW 59.18.650 — statewide just cause
//! includes criminal activity), NJ (Anti-Eviction Act, N.J.S.A.
//! 2A:18-61.1(n) — drug-related conviction is enumerated cause).
//!
//! `ContractGovernsPrivateMarket`: most states. Lease provisions
//! govern; common-law contract principles apply.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    PublicHousingPha,
    Section8Voucher,
    PrivateMarketRental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateJustCauseRegime {
    StateJustCauseListsCriminalActivity,
    ContractGovernsPrivateMarket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrugActivityType {
    None,
    /// Drug activity by tenant themselves.
    TenantOnPremises,
    TenantOffPremises,
    /// Drug activity by household member (e.g., adult child living in unit).
    HouseholdMemberOnPremises,
    HouseholdMemberOffPremises,
    /// Drug activity by guest of tenant.
    GuestOnPremises,
    GuestOffPremises,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: StateJustCauseRegime,
    pub citation: &'static str,
}

const fn rule(regime: StateJustCauseRegime, citation: &'static str) -> StateRule {
    StateRule { regime, citation }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use StateJustCauseRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            StateJustCauseListsCriminalActivity,
            "Cal. Civ. Code § 1946.2 (Tenant Protection Act of 2019 / AB 1482) — drug-related criminal activity is enumerated just cause for termination",
        ),
    );
    m.insert(
        "OR",
        rule(
            StateJustCauseListsCriminalActivity,
            "Or. ORS Ch. 90 (SB 608 of 2019) — drug-related criminal activity is just cause after first year of occupancy",
        ),
    );
    m.insert(
        "WA",
        rule(
            StateJustCauseListsCriminalActivity,
            "Wash. RCW 59.18.650 (Engrossed SSB 5160 of 2021) — statewide just cause includes criminal activity by tenant or household",
        ),
    );
    m.insert(
        "NJ",
        rule(
            StateJustCauseListsCriminalActivity,
            "N.J.S.A. 2A:18-61.1(n) (Anti-Eviction Act) — drug-related conviction is enumerated cause for eviction",
        ),
    );

    // ContractGovernsPrivateMarket — all other states + DC.
    let contract_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA",
        "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
        "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NM",
        "NY", "NC", "ND", "OH", "OK", "PA", "RI", "SC", "SD", "TN",
        "TX", "UT", "VT", "VA", "WV", "WI", "WY",
    ];
    for code in contract_states {
        m.insert(
            code,
            rule(
                ContractGovernsPrivateMarket,
                "No statewide just-cause statute enumerating criminal activity; lease provisions govern",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEvictionInput {
    pub state_code: String,
    pub property_type: PropertyType,
    pub drug_activity: DrugActivityType,
    /// True if the tenant personally knew of or could have controlled
    /// the household member's or guest's drug activity. Federal
    /// One-Strike rule is strict-liability post-Rucker (2002), so
    /// this only matters for caller-side mitigation analysis.
    pub tenant_aware_of_household_or_guest_activity: bool,
    /// True if the PHA / Section 8 administering entity considered
    /// individualized mitigating circumstances per HUD discretion +
    /// 2024 proposed rule.
    pub pha_considered_mitigating_factors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEvictionResult {
    pub federal_floor_applies: bool,
    pub federal_one_strike_authorizes_eviction: bool,
    pub state_regime: StateJustCauseRegime,
    pub state_just_cause_authorizes_eviction: bool,
    pub eviction_authorized: bool,
    pub tenant_strict_liability_under_rucker: bool,
    pub mitigation_analysis_recommended: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &DrugEvictionInput) -> DrugEvictionResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: StateJustCauseRegime::ContractGovernsPrivateMarket,
        citation: "Unknown state code; assuming contract-governs default",
    });

    let federal_applies = matches!(
        input.property_type,
        PropertyType::PublicHousingPha | PropertyType::Section8Voucher
    );

    // Federal One-Strike: any drug-related activity by covered person
    // authorizes eviction, regardless of tenant knowledge (Rucker
    // strict liability).
    let federal_authorizes = federal_applies
        && !matches!(input.drug_activity, DrugActivityType::None);

    let strict_liability = federal_authorizes
        && matches!(
            input.drug_activity,
            DrugActivityType::HouseholdMemberOnPremises
                | DrugActivityType::HouseholdMemberOffPremises
                | DrugActivityType::GuestOnPremises
                | DrugActivityType::GuestOffPremises
        )
        && !input.tenant_aware_of_household_or_guest_activity;

    let mitigation_recommended = federal_authorizes && !input.pha_considered_mitigating_factors;

    // State just-cause for private market.
    let state_authorizes = matches!(input.property_type, PropertyType::PrivateMarketRental)
        && rule.regime == StateJustCauseRegime::StateJustCauseListsCriminalActivity
        && !matches!(input.drug_activity, DrugActivityType::None);

    // Eviction authorized if either federal floor or state regime authorizes.
    let eviction_authorized = federal_authorizes
        || state_authorizes
        || (matches!(input.property_type, PropertyType::PrivateMarketRental)
            && rule.regime == StateJustCauseRegime::ContractGovernsPrivateMarket
            && !matches!(input.drug_activity, DrugActivityType::None));

    let note = match (input.property_type, rule.regime, input.drug_activity) {
        (_, _, DrugActivityType::None) =>
            "No drug-related activity alleged; eviction not authorized on drug-eviction grounds.".to_string(),
        (PropertyType::PublicHousingPha, _, _) => format!(
            "Federal HUD One-Strike (42 U.S.C. § 1437d(l)(6)): drug activity ({:?}) by covered person authorizes eviction. Rucker (2002) strict liability applies{}.{}",
            input.drug_activity,
            if strict_liability { " — tenant unaware of household/guest activity, but eviction still permitted under federal rule" } else { "" },
            if mitigation_recommended {
                " HUD discretion recommends mitigation analysis (2024 proposed rule)."
            } else { "" },
        ),
        (PropertyType::Section8Voucher, _, _) => format!(
            "Federal Section 8 (42 U.S.C. § 1437f): drug activity ({:?}) authorizes voucher termination + eviction.{}",
            input.drug_activity,
            if mitigation_recommended {
                " HUD discretion recommends mitigation analysis."
            } else { "" },
        ),
        (PropertyType::PrivateMarketRental, StateJustCauseRegime::StateJustCauseListsCriminalActivity, _) => format!(
            "StateJustCauseListsCriminalActivity: state statute lists drug-related criminal activity as enumerated just cause; eviction authorized for {:?}.",
            input.drug_activity,
        ),
        (PropertyType::PrivateMarketRental, StateJustCauseRegime::ContractGovernsPrivateMarket, _) => format!(
            "ContractGovernsPrivateMarket: no state just-cause statute; landlord may evict per lease provisions for {:?}.",
            input.drug_activity,
        ),
    };

    DrugEvictionResult {
        federal_floor_applies: federal_applies,
        federal_one_strike_authorizes_eviction: federal_authorizes,
        state_regime: rule.regime,
        state_just_cause_authorizes_eviction: state_authorizes,
        eviction_authorized,
        tenant_strict_liability_under_rucker: strict_liability,
        mitigation_analysis_recommended: mitigation_recommended,
        citation: format!(
            "Federal 42 U.S.C. § 1437d(l)(6) HUD One-Strike for public housing; 42 U.S.C. § 1437f Section 8 voucher/certificate; Dept. of Housing & Urban Development v. Rucker, 535 U.S. 125 (2002) strict liability; HUD 2024 Reducing Barriers to HUD-Assisted Housing proposed rule on mitigation; state: {}",
            rule.citation,
        ),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, ptype: PropertyType, activity: DrugActivityType) -> DrugEvictionInput {
        DrugEvictionInput {
            state_code: state.to_string(),
            property_type: ptype,
            drug_activity: activity,
            tenant_aware_of_household_or_guest_activity: false,
            pha_considered_mitigating_factors: false,
        }
    }

    // Federal HUD One-Strike.

    #[test]
    fn pha_tenant_drug_activity_authorizes_eviction() {
        let r = check(&input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(r.federal_floor_applies);
        assert!(r.federal_one_strike_authorizes_eviction);
        assert!(r.eviction_authorized);
    }

    #[test]
    fn pha_off_premises_drug_activity_still_triggers_one_strike() {
        // Federal rule covers ON OR OFF premises.
        let r = check(&input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::TenantOffPremises,
        ));
        assert!(r.federal_one_strike_authorizes_eviction);
    }

    #[test]
    fn pha_household_member_activity_strict_liability_applies() {
        let mut i = input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::HouseholdMemberOnPremises,
        );
        i.tenant_aware_of_household_or_guest_activity = false;
        let r = check(&i);
        assert!(r.tenant_strict_liability_under_rucker);
        assert!(r.eviction_authorized);
    }

    #[test]
    fn pha_guest_off_premises_still_triggers_strict_liability() {
        let mut i = input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::GuestOffPremises,
        );
        i.tenant_aware_of_household_or_guest_activity = false;
        let r = check(&i);
        assert!(r.tenant_strict_liability_under_rucker);
    }

    #[test]
    fn pha_tenant_self_activity_not_strict_liability() {
        // Tenant's OWN activity is not strict-liability — they're aware.
        let r = check(&input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(!r.tenant_strict_liability_under_rucker);
    }

    #[test]
    fn pha_mitigation_recommended_when_not_considered() {
        let r = check(&input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(r.mitigation_analysis_recommended);
        assert!(r.note.contains("mitigation analysis"));
    }

    #[test]
    fn pha_mitigation_considered_no_recommendation_flag() {
        let mut i = input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::TenantOnPremises,
        );
        i.pha_considered_mitigating_factors = true;
        let r = check(&i);
        assert!(!r.mitigation_analysis_recommended);
    }

    // Section 8.

    #[test]
    fn section_8_drug_activity_authorizes_eviction() {
        let r = check(&input(
            "TX",
            PropertyType::Section8Voucher,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(r.federal_floor_applies);
        assert!(r.eviction_authorized);
        assert!(r.note.contains("§ 1437f"));
    }

    // Private market — state just-cause regimes.

    #[test]
    fn ca_private_market_lists_criminal_activity_authorizes_eviction() {
        let r = check(&input(
            "CA",
            PropertyType::PrivateMarketRental,
            DrugActivityType::TenantOnPremises,
        ));
        assert_eq!(r.state_regime, StateJustCauseRegime::StateJustCauseListsCriminalActivity);
        assert!(r.state_just_cause_authorizes_eviction);
        assert!(r.eviction_authorized);
        assert!(!r.federal_floor_applies);
    }

    #[test]
    fn or_private_market_authorizes_drug_eviction() {
        let r = check(&input(
            "OR",
            PropertyType::PrivateMarketRental,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(r.state_just_cause_authorizes_eviction);
    }

    #[test]
    fn wa_private_market_authorizes_drug_eviction() {
        let r = check(&input(
            "WA",
            PropertyType::PrivateMarketRental,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(r.state_just_cause_authorizes_eviction);
    }

    #[test]
    fn nj_private_market_authorizes_drug_eviction() {
        let r = check(&input(
            "NJ",
            PropertyType::PrivateMarketRental,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(r.state_just_cause_authorizes_eviction);
    }

    // Private market — contract governs.

    #[test]
    fn tx_private_market_contract_governs() {
        let r = check(&input(
            "TX",
            PropertyType::PrivateMarketRental,
            DrugActivityType::TenantOnPremises,
        ));
        assert_eq!(r.state_regime, StateJustCauseRegime::ContractGovernsPrivateMarket);
        // Eviction still authorized under lease.
        assert!(r.eviction_authorized);
        assert!(!r.state_just_cause_authorizes_eviction);
    }

    #[test]
    fn ny_private_market_contract_governs() {
        let r = check(&input(
            "NY",
            PropertyType::PrivateMarketRental,
            DrugActivityType::TenantOnPremises,
        ));
        assert_eq!(r.state_regime, StateJustCauseRegime::ContractGovernsPrivateMarket);
    }

    // No drug activity.

    #[test]
    fn no_drug_activity_no_eviction_authorized() {
        let r = check(&input(
            "TX",
            PropertyType::PublicHousingPha,
            DrugActivityType::None,
        ));
        assert!(!r.federal_one_strike_authorizes_eviction);
        assert!(!r.eviction_authorized);
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
    fn state_just_cause_regime_only_ca_or_wa_nj() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == StateJustCauseRegime::StateJustCauseListsCriminalActivity {
                count += 1;
            }
        }
        assert_eq!(count, 4, "expected CA + OR + WA + NJ only");
    }

    #[test]
    fn unknown_state_falls_back_to_contract() {
        let r = check(&input("XX", PropertyType::PrivateMarketRental, DrugActivityType::TenantOnPremises));
        assert_eq!(r.state_regime, StateJustCauseRegime::ContractGovernsPrivateMarket);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ca", PropertyType::PrivateMarketRental, DrugActivityType::TenantOnPremises));
        assert!(r.state_just_cause_authorizes_eviction);
    }

    // Notes.

    #[test]
    fn rucker_strict_liability_mentioned_in_pha_note() {
        let mut i = input("TX", PropertyType::PublicHousingPha, DrugActivityType::HouseholdMemberOnPremises);
        i.tenant_aware_of_household_or_guest_activity = false;
        let r = check(&i);
        assert!(r.note.contains("Rucker"));
        assert!(r.note.contains("strict liability"));
    }

    #[test]
    fn citation_mentions_federal_and_state() {
        let r = check(&input(
            "CA",
            PropertyType::PrivateMarketRental,
            DrugActivityType::TenantOnPremises,
        ));
        assert!(r.citation.contains("§ 1437d(l)(6)"));
        assert!(r.citation.contains("Rucker"));
        assert!(r.citation.contains("§ 1946.2"));
    }
}
