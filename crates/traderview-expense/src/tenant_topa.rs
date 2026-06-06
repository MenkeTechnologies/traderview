//! State Tenant Opportunity to Purchase Act (TOPA) / right of first
//! refusal on landlord sale.
//!
//! When a landlord sells a residential rental property, several
//! jurisdictions grant the tenant a statutory right of first refusal
//! — the tenant can match a third-party offer and take the property
//! themselves. Recent legislative wave (2020-2024) with significant
//! state-by-state variation.
//!
//! Four regimes:
//!
//! - `AllSalesGeneralTopa` — D.C. (D.C. Code § 42-3404.02 et seq.,
//!   originally 1980): the most aggressive TOPA in the country.
//!   Applies to ALL residential rental sales (any unit count, any
//!   structure). Tenant or tenant organization has 15 days after
//!   receiving a valid third-party sales contract to exercise the
//!   right of first refusal under D.C. Code § 42-3404.08.
//!   Foreclosure, tax sale, and bankruptcy sale are EXEMPT under the
//!   §-3404.02 definition of "sell" / "sale".
//!
//! - `NarrowResidentialTopa` — Maryland (HB 693 of 2024, Renter's
//!   Rights and Stabilization Act): right of first refusal for tenants
//!   of residential properties with **three or fewer** dwelling units.
//!   Strict compliance requirements for landlord notice of intent to
//!   sell. The "Maryland TOPA" model — concept-similar to DC's but
//!   far narrower scope.
//!
//! - `ForeclosureOnlyPriority` — California (SB 1079 of 2020,
//!   effective 2021-01-01 through 2026-01-01): post-foreclosure
//!   tenant buyer priority for 1-4 single-family residences. NOT a
//!   true TOPA — applies only to foreclosure sales, where eligible
//!   tenant buyers, prospective owner-occupants, and community
//!   nonprofits get bidding priority for 15 days after the initial
//!   trustee sale at the same dollar amount as the highest bidder
//!   (or 45 days to submit a bid that EXCEEDS the highest bidder).
//!
//! - `NoStateTopa` — most other states. No statewide right of first
//!   refusal on landlord sale. Some cities (Boston, Somerville,
//!   Minneapolis) have local TOPA efforts; Massachusetts H.1260 /
//!   S.786 would enable statewide opt-in but is not yet enacted.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopaRegime {
    AllSalesGeneralTopa,
    NarrowResidentialTopa,
    ForeclosureOnlyPriority,
    NoStateTopa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleType {
    VoluntaryThirdParty,
    Foreclosure,
    TaxSale,
    BankruptcySale,
    CondominiumConversion,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: TopaRegime,
    /// Days from tenant receipt of valid sale contract / notice for
    /// tenant to exercise the right of first refusal.
    pub exercise_window_days: Option<u32>,
    /// Maximum unit count for the property to be covered. `None` =
    /// no cap (DC covers all sizes); `Some(n)` = unit count must be
    /// ≤ n to be covered (MD covers ≤ 3, CA covers ≤ 4).
    pub max_unit_count_covered: Option<u32>,
    /// True if foreclosure sales are EXEMPT from the framework.
    pub foreclosure_exempt: bool,
    /// True if tax sales are exempt.
    pub tax_sale_exempt: bool,
    /// True if bankruptcy sales are exempt.
    pub bankruptcy_sale_exempt: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: TopaRegime,
    exercise_window_days: Option<u32>,
    max_unit_count_covered: Option<u32>,
    foreclosure_exempt: bool,
    tax_sale_exempt: bool,
    bankruptcy_sale_exempt: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        exercise_window_days,
        max_unit_count_covered,
        foreclosure_exempt,
        tax_sale_exempt,
        bankruptcy_sale_exempt,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use TopaRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // D.C. — the prototype TOPA, all sales covered, 15-day window.
    m.insert(
        "DC",
        rule(
            AllSalesGeneralTopa,
            Some(15),
            None, // No unit-count cap
            true, true, true,
            "D.C. Code § 42-3404.02 et seq. (TOPA, 1980); §42-3404.08 right of first refusal 15 days; §42-3404.02 foreclosure/tax/bankruptcy exempt from 'sale' definition",
        ),
    );

    // Maryland — HB 693 of 2024, 3-or-fewer units.
    m.insert(
        "MD",
        rule(
            NarrowResidentialTopa,
            Some(15), // MD follows DC-like 15-day window; exact varies
            Some(3),
            true, true, true,
            "Md. HB 693 of 2024 — Renter's Rights and Stabilization Act; residential property with 3 or fewer dwelling units",
        ),
    );

    // California — SB 1079, foreclosure-only, 1-4 unit SFR.
    m.insert(
        "CA",
        rule(
            ForeclosureOnlyPriority,
            Some(15),
            Some(4),
            false, // Foreclosure NOT exempt — it's the entire scope
            true,  // Tax sale not covered
            true,  // Bankruptcy not covered
            "Cal. SB 1079 of 2020 (eff. 2021-01-01 through 2026-01-01); post-foreclosure tenant priority for 1-4 unit SFR",
        ),
    );

    // NoStateTopa — all remaining states.
    let no_topa = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA",
        "WV", "WI", "WY",
    ];
    for code in no_topa {
        m.insert(
            code,
            rule(
                NoStateTopa,
                None,
                None,
                false, false, false,
                "No statewide tenant right of first refusal on landlord sale; local ordinances may apply (e.g., Boston/Somerville pending; MA H.1260/S.786 statewide opt-in pending)",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopaInput {
    pub state_code: String,
    pub sale_type: SaleType,
    pub unit_count: u32,
    pub tenant_received_sale_contract_or_notice: bool,
    pub days_since_notice_received: u32,
    pub tenant_exercised_right_to_purchase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopaResult {
    pub regime: TopaRegime,
    pub sale_type_covered: bool,
    pub unit_count_within_scope: bool,
    pub tenant_has_right_of_first_refusal: bool,
    pub exercise_window_days: Option<u32>,
    pub within_exercise_window: bool,
    pub right_still_exercisable: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &TopaInput) -> TopaResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: TopaRegime::NoStateTopa,
        exercise_window_days: None,
        max_unit_count_covered: None,
        foreclosure_exempt: false,
        tax_sale_exempt: false,
        bankruptcy_sale_exempt: false,
        citation: "Unknown state code; assuming no statewide TOPA",
    });

    // Sale type coverage.
    let sale_covered = match (rule.regime, input.sale_type) {
        (TopaRegime::ForeclosureOnlyPriority, SaleType::Foreclosure) => true,
        (TopaRegime::ForeclosureOnlyPriority, _) => false,
        (TopaRegime::NoStateTopa, _) => false,
        (_, SaleType::Foreclosure) => !rule.foreclosure_exempt,
        (_, SaleType::TaxSale) => !rule.tax_sale_exempt,
        (_, SaleType::BankruptcySale) => !rule.bankruptcy_sale_exempt,
        // AllSalesGeneralTopa + NarrowResidentialTopa for remaining
        // VoluntaryThirdParty + CondominiumConversion → all covered.
        _ => true,
    };

    // Unit count check.
    let unit_in_scope = match rule.max_unit_count_covered {
        Some(cap) => input.unit_count <= cap && input.unit_count > 0,
        None => input.unit_count > 0,
    };

    let has_rofr = rule.regime != TopaRegime::NoStateTopa && sale_covered && unit_in_scope;

    let within_window = match rule.exercise_window_days {
        Some(w) => input.days_since_notice_received <= w,
        None => false,
    };
    let still_exercisable = has_rofr
        && input.tenant_received_sale_contract_or_notice
        && within_window
        && !input.tenant_exercised_right_to_purchase;

    let note = match (rule.regime, has_rofr) {
        (TopaRegime::AllSalesGeneralTopa, true) => format!(
            "AllSalesGeneralTopa (DC §42-3404): tenant has ROFR on sale; {}/{} day window {}. Foreclosure/tax/bankruptcy exempt.",
            input.days_since_notice_received,
            rule.exercise_window_days.unwrap_or(0),
            if within_window { "active" } else { "EXPIRED" },
        ),
        (TopaRegime::NarrowResidentialTopa, true) => format!(
            "NarrowResidentialTopa (MD HB 693): tenant has ROFR; property has {} units (≤ {} cap); {}/{} day window.",
            input.unit_count,
            rule.max_unit_count_covered.unwrap_or(0),
            input.days_since_notice_received,
            rule.exercise_window_days.unwrap_or(0),
        ),
        (TopaRegime::ForeclosureOnlyPriority, true) => format!(
            "ForeclosureOnlyPriority (CA SB 1079): tenant has post-foreclosure priority for {}-unit property (≤4 cap); 15-day same-bid window / 45-day exceeding-bid window.",
            input.unit_count,
        ),
        (regime, false) => format!(
            "{:?}: no tenant ROFR on this transaction. Sale type covered: {}; unit count in scope: {}.",
            regime, sale_covered, unit_in_scope,
        ),
        (TopaRegime::NoStateTopa, _) =>
            "NoStateTopa: no statewide TOPA framework; local ordinances may apply".to_string(),
    };

    TopaResult {
        regime: rule.regime,
        sale_type_covered: sale_covered,
        unit_count_within_scope: unit_in_scope,
        tenant_has_right_of_first_refusal: has_rofr,
        exercise_window_days: rule.exercise_window_days,
        within_exercise_window: within_window,
        right_still_exercisable: still_exercisable,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, sale: SaleType, units: u32) -> TopaInput {
        TopaInput {
            state_code: state.to_string(),
            sale_type: sale,
            unit_count: units,
            tenant_received_sale_contract_or_notice: true,
            days_since_notice_received: 5,
            tenant_exercised_right_to_purchase: false,
        }
    }

    // DC — all sales, any unit count.

    #[test]
    fn dc_voluntary_sale_creates_tenant_rofr() {
        let r = check(&input("DC", SaleType::VoluntaryThirdParty, 1));
        assert_eq!(r.regime, TopaRegime::AllSalesGeneralTopa);
        assert!(r.tenant_has_right_of_first_refusal);
        assert!(r.within_exercise_window);
        assert!(r.right_still_exercisable);
    }

    #[test]
    fn dc_large_apartment_complex_still_covered() {
        let r = check(&input("DC", SaleType::VoluntaryThirdParty, 200));
        assert!(r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn dc_foreclosure_exempt() {
        let r = check(&input("DC", SaleType::Foreclosure, 1));
        assert!(!r.sale_type_covered);
        assert!(!r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn dc_tax_sale_exempt() {
        let r = check(&input("DC", SaleType::TaxSale, 1));
        assert!(!r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn dc_bankruptcy_sale_exempt() {
        let r = check(&input("DC", SaleType::BankruptcySale, 1));
        assert!(!r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn dc_15_day_window_day_15_within() {
        let mut i = input("DC", SaleType::VoluntaryThirdParty, 1);
        i.days_since_notice_received = 15;
        let r = check(&i);
        assert!(r.within_exercise_window);
    }

    #[test]
    fn dc_15_day_window_day_16_expired() {
        let mut i = input("DC", SaleType::VoluntaryThirdParty, 1);
        i.days_since_notice_received = 16;
        let r = check(&i);
        assert!(!r.within_exercise_window);
        assert!(!r.right_still_exercisable);
    }

    #[test]
    fn dc_tenant_exercised_no_longer_exercisable() {
        let mut i = input("DC", SaleType::VoluntaryThirdParty, 1);
        i.tenant_exercised_right_to_purchase = true;
        let r = check(&i);
        assert!(r.tenant_has_right_of_first_refusal); // right existed
        assert!(!r.right_still_exercisable); // but already exercised
    }

    // MD — 3-or-fewer-unit cap.

    #[test]
    fn md_3_unit_property_covered() {
        let r = check(&input("MD", SaleType::VoluntaryThirdParty, 3));
        assert_eq!(r.regime, TopaRegime::NarrowResidentialTopa);
        assert!(r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn md_4_unit_property_not_covered() {
        let r = check(&input("MD", SaleType::VoluntaryThirdParty, 4));
        assert!(!r.unit_count_within_scope);
        assert!(!r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn md_single_family_covered() {
        let r = check(&input("MD", SaleType::VoluntaryThirdParty, 1));
        assert!(r.tenant_has_right_of_first_refusal);
    }

    // CA — foreclosure-only, 1-4 unit.

    #[test]
    fn ca_foreclosure_4_unit_covered() {
        let r = check(&input("CA", SaleType::Foreclosure, 4));
        assert_eq!(r.regime, TopaRegime::ForeclosureOnlyPriority);
        assert!(r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn ca_voluntary_sale_not_covered() {
        // CA SB 1079 applies ONLY to foreclosure, not voluntary sales.
        let r = check(&input("CA", SaleType::VoluntaryThirdParty, 2));
        assert!(!r.sale_type_covered);
        assert!(!r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn ca_foreclosure_5_unit_not_covered() {
        let r = check(&input("CA", SaleType::Foreclosure, 5));
        assert!(!r.unit_count_within_scope);
        assert!(!r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn ca_foreclosure_1_unit_covered() {
        let r = check(&input("CA", SaleType::Foreclosure, 1));
        assert!(r.tenant_has_right_of_first_refusal);
    }

    #[test]
    fn ca_tax_sale_not_covered() {
        let r = check(&input("CA", SaleType::TaxSale, 2));
        assert!(!r.tenant_has_right_of_first_refusal);
    }

    // No-TOPA states.

    #[test]
    fn no_topa_states_never_create_rofr() {
        for st in &["TX", "FL", "NY", "WA", "MA", "OR", "GA"] {
            let r = check(&input(st, SaleType::VoluntaryThirdParty, 1));
            assert_eq!(r.regime, TopaRegime::NoStateTopa, "{st}");
            assert!(!r.tenant_has_right_of_first_refusal, "{st}");
        }
    }

    #[test]
    fn no_topa_state_note_mentions_local_ordinances() {
        let r = check(&input("MA", SaleType::VoluntaryThirdParty, 1));
        // Note from rule citation mentions H.1260/S.786 pending statewide.
        assert!(r.citation.contains("H.1260") || r.citation.contains("Boston"));
    }

    // Coverage / invariants.

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
    fn only_dc_uses_all_sales_general_topa() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == TopaRegime::AllSalesGeneralTopa {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected DC only with AllSalesGeneralTopa");
    }

    #[test]
    fn only_md_uses_narrow_residential_topa() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == TopaRegime::NarrowResidentialTopa {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected MD only with NarrowResidentialTopa");
    }

    #[test]
    fn only_ca_uses_foreclosure_only_priority() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == TopaRegime::ForeclosureOnlyPriority {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected CA only with ForeclosureOnlyPriority");
    }

    #[test]
    fn unknown_state_falls_back_to_no_topa() {
        let r = check(&input("XX", SaleType::VoluntaryThirdParty, 1));
        assert_eq!(r.regime, TopaRegime::NoStateTopa);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("dc", SaleType::VoluntaryThirdParty, 1));
        assert!(r.tenant_has_right_of_first_refusal);
    }

    // Notes.

    #[test]
    fn dc_note_describes_general_topa_path() {
        let r = check(&input("DC", SaleType::VoluntaryThirdParty, 5));
        assert!(r.note.contains("AllSalesGeneralTopa"));
        assert!(r.note.contains("Foreclosure/tax/bankruptcy exempt"));
    }

    #[test]
    fn md_note_describes_unit_cap() {
        let r = check(&input("MD", SaleType::VoluntaryThirdParty, 2));
        assert!(r.note.contains("NarrowResidentialTopa"));
        assert!(r.note.contains("≤ 3 cap"));
    }

    #[test]
    fn ca_note_describes_foreclosure_priority_path() {
        let r = check(&input("CA", SaleType::Foreclosure, 2));
        assert!(r.note.contains("ForeclosureOnlyPriority"));
        assert!(r.note.contains("15-day"));
        assert!(r.note.contains("45-day"));
    }

    #[test]
    fn expired_window_described_in_dc_note() {
        let mut i = input("DC", SaleType::VoluntaryThirdParty, 1);
        i.days_since_notice_received = 16;
        let r = check(&i);
        assert!(r.note.contains("EXPIRED"));
    }
}
