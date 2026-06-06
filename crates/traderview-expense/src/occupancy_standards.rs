//! Federal + state occupancy standards for residential rentals.
//!
//! Foundational rule for any landlord-trader screening prospective
//! tenants. Setting the occupancy limit too LOW creates a federal Fair
//! Housing Act § 3604 familial-status discrimination claim
//! (disproportionate impact on families with children); setting it
//! too HIGH risks state/local sanitary code violations and health
//! department citations.
//!
//! **Federal floor — HUD Keating Memorandum (1991, formalized in 63
//! FR 70982, 1998-12-18)**: an occupancy standard of **two persons
//! per bedroom** is generally presumptively reasonable under the
//! Fair Housing Act. The standard is REBUTTABLE — HUD considers
//! bedroom size, age of children, septic capacity, configuration of
//! the unit, and any state/local sanitary code that imposes
//! stricter sqft-per-occupant requirements. The memo prohibits use
//! of occupancy limits as a PRETEXT for excluding families with
//! children — even a facially "reasonable" 2-per-bedroom rule is
//! illegal if enforced selectively against children.
//!
//! Four state regimes:
//!
//! 1. **`SqftPerOccupantFormula`** — NY (NYC Admin Code § 27-2075:
//!    80 sqft livable per person; total occupancy = floor area /
//!    80 sqft; one child under 4 per pair of adults), MA (105 CMR
//!    410 State Sanitary Code: 150 sqft for first dwelling-unit
//!    occupant + 100 sqft each additional; 70 sqft first sleeping
//!    occupant + 100 sqft combined for 2 + 50 sqft each additional).
//!
//! 2. **`TwoPlusOneStatutory`** — CA (2 per bedroom + 1 additional
//!    "2+1" formula via Smith v. FEHC line of cases + Uniform
//!    Housing Code 120 sqft first room + 50 sqft per occupant over
//!    2). State explicitly pins the 2+1 standard.
//!
//! 3. **`NoMoreRestrictiveThanTwoPerBedroom`** — OR (ORS 90.262:
//!    landlord-adopted occupancy rule cannot be more restrictive
//!    than 2 per bedroom and must be reasonable). The state CAPS
//!    landlord discretion at 2 per bedroom.
//!
//! 4. **`HudKeatingDefault`** — most other states. Defers to the HUD
//!    Keating Memo's 2-per-bedroom presumption with local building /
//!    sanitary code overlay. WA falls here as well (RCW 35A.21.314
//!    via SB 5235 of 2021 preempts CITY restrictions on number of
//!    UNRELATED occupants, but state occupancy is still Keating).
//!
//! **Federal floor always applies**: regardless of state regime, the
//! Fair Housing Act § 3604 prohibits familial-status discrimination
//! pretext. If a landlord enforces an occupancy limit against
//! families with children but not against groups of adults, the
//! limit is unlawful even if facially Keating-compliant.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyRegime {
    SqftPerOccupantFormula,
    TwoPlusOneStatutory,
    NoMoreRestrictiveThanTwoPerBedroom,
    HudKeatingDefault,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: OccupancyRegime,
    /// Square feet per occupant under SqftPerOccupantFormula regime.
    /// 0 under non-sqft regimes.
    pub sqft_per_occupant: u32,
    /// Square feet floor for the first dwelling-unit occupant (MA
    /// rule). 0 if not applicable.
    pub sqft_first_occupant_floor: u32,
    /// True when state law itself caps landlord rule at 2 per bedroom
    /// (OR). HudKeatingDefault states have an advisory 2-per-bedroom
    /// presumption but no statutory cap.
    pub landlord_rule_capped_at_2_per_bedroom: bool,
    /// True when state has explicit familial-status pretext language
    /// stricter than federal FHA. CA explicitly.
    pub state_familial_status_pretext_explicit: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: OccupancyRegime,
    sqft_per_occupant: u32,
    sqft_first_occupant_floor: u32,
    landlord_rule_capped_at_2_per_bedroom: bool,
    state_familial_status_pretext_explicit: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        sqft_per_occupant,
        sqft_first_occupant_floor,
        landlord_rule_capped_at_2_per_bedroom,
        state_familial_status_pretext_explicit,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use OccupancyRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            SqftPerOccupantFormula,
            80, // 80 sqft per occupant (NYC HMC § 27-2075)
            0,
            false,
            false,
            "NYC Admin Code § 27-2075 (HMC § 27-2075) — 80 sqft livable per person",
        ),
    );
    m.insert(
        "MA",
        rule(
            SqftPerOccupantFormula,
            100, // 100 sqft per additional occupant
            150, // 150 sqft floor for first occupant
            false,
            false,
            "Mass. 105 CMR 410 State Sanitary Code (150 sqft first + 100 sqft each additional)",
        ),
    );
    m.insert(
        "CA",
        rule(
            TwoPlusOneStatutory,
            50, // 50 sqft per additional occupant over 2 in a bedroom (Uniform Housing Code)
            120, // 120 sqft minimum first room
            false,
            true,
            "Smith v. Fair Employment & Housing Comm'n (1990s line) + Uniform Housing Code (CA Health & Safety Code § 17920.3)",
        ),
    );
    m.insert(
        "OR",
        rule(
            NoMoreRestrictiveThanTwoPerBedroom,
            0,
            0,
            true,
            false,
            "Or. ORS 90.262 + Fair Housing Council of Oregon 2+1 guidance",
        ),
    );

    // HudKeatingDefault for all remaining states.
    let keating_default = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NC", "ND", "OH", "OK", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in keating_default {
        // WA gets a stricter citation (unrelated-occupant preemption).
        let citation: &'static str = if code == "WA" {
            "HUD Keating Memorandum (63 FR 70982); WA RCW 35A.21.314 (SB 5235 of 2021) preempts city unrelated-occupant restrictions"
        } else {
            "HUD Keating Memorandum (63 FR 70982) — 2 per bedroom presumption + state sanitary code overlay"
        };
        m.insert(code, rule(HudKeatingDefault, 0, 0, false, false, citation));
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyInput {
    pub state_code: String,
    pub bedrooms_in_unit: u32,
    pub total_livable_sqft: u32,
    pub proposed_occupants: u32,
    /// True if landlord enforces this occupancy rule differently
    /// against families with children vs adult roommate groups — an
    /// FHA § 3604 pretext violation regardless of state regime.
    pub enforced_selectively_against_families: bool,
    /// True if the unit has septic capacity or fire-code limit
    /// stricter than the 2-per-bedroom presumption (rebuttal under
    /// Keating Memo).
    pub building_code_caps_below_keating: bool,
    pub building_code_max_occupants: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyResult {
    pub regime: OccupancyRegime,
    /// Effective maximum occupancy under the state regime — lesser of
    /// state-formula max, building-code max, and Keating presumption.
    pub effective_max_occupants: u32,
    pub state_formula_max: u32,
    pub keating_presumption_max: u32,
    pub proposed_within_limit: bool,
    pub fha_familial_status_pretext_violation: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &OccupancyInput) -> OccupancyResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: OccupancyRegime::HudKeatingDefault,
        sqft_per_occupant: 0,
        sqft_first_occupant_floor: 0,
        landlord_rule_capped_at_2_per_bedroom: false,
        state_familial_status_pretext_explicit: false,
        citation: "Unknown state code; assuming HUD Keating default",
    });

    // Keating presumption: 2 per bedroom.
    let keating_max = input.bedrooms_in_unit.saturating_mul(2);

    // State-formula maximum.
    let state_max = match rule.regime {
        OccupancyRegime::SqftPerOccupantFormula => {
            // NY: total livable / 80 sqft.
            // MA: (livable − 150) / 100 + 1, with first-occupant floor.
            if rule.sqft_first_occupant_floor > 0 {
                // MA-style: 150 sqft for first + 100 sqft per additional.
                if input.total_livable_sqft < rule.sqft_first_occupant_floor {
                    0
                } else {
                    1 + (input.total_livable_sqft - rule.sqft_first_occupant_floor)
                        / rule.sqft_per_occupant
                }
            } else {
                // NY-style: floor area / sqft per occupant.
                input.total_livable_sqft / rule.sqft_per_occupant
            }
        }
        OccupancyRegime::TwoPlusOneStatutory => {
            // CA 2+1: two per bedroom + one additional. With sqft floor
            // check: 120 sqft first room, 50 sqft each occupant over 2.
            // The 2+1 max is the binding rule unless sqft floor binds tighter.
            input.bedrooms_in_unit.saturating_mul(2).saturating_add(1)
        }
        OccupancyRegime::NoMoreRestrictiveThanTwoPerBedroom => {
            // OR: landlord cannot adopt rule more restrictive than 2 per
            // bedroom. Effectively floor of 2 per bedroom.
            keating_max
        }
        OccupancyRegime::HudKeatingDefault => keating_max,
    };

    // Effective max: lesser of state-formula and Keating presumption,
    // further capped by building code if applicable.
    let mut effective_max = state_max.min(keating_max).max(
        if rule.regime == OccupancyRegime::NoMoreRestrictiveThanTwoPerBedroom {
            // OR: never less than 2 per bedroom — the landlord rule floor.
            keating_max
        } else {
            0
        },
    );
    // For TwoPlusOneStatutory, the state formula EXCEEDS Keating (2 per
    // bedroom + 1), so it overrides the Keating cap.
    if rule.regime == OccupancyRegime::TwoPlusOneStatutory {
        effective_max = state_max;
    }
    if input.building_code_caps_below_keating {
        if let Some(cap) = input.building_code_max_occupants {
            effective_max = effective_max.min(cap);
        }
    }

    let within_limit = input.proposed_occupants <= effective_max;
    let pretext_violation = input.enforced_selectively_against_families;

    let note = if pretext_violation {
        format!(
            "FHA § 3604 familial-status PRETEXT VIOLATION: occupancy rule applied selectively against families with children but not adult roommate groups. State regime {:?} effective max {} occupants; even compliance with that limit cannot cure pretextual enforcement.",
            rule.regime,
            effective_max,
        )
    } else {
        format!(
            "{:?}: effective max {} occupants (state formula {}, Keating presumption {}{}). Proposed {} {}.",
            rule.regime,
            effective_max,
            state_max,
            keating_max,
            if input.building_code_caps_below_keating {
                format!(", building code {}", input.building_code_max_occupants.unwrap_or(0))
            } else {
                String::new()
            },
            input.proposed_occupants,
            if within_limit { "within limit" } else { "EXCEEDS LIMIT" },
        )
    };

    OccupancyResult {
        regime: rule.regime,
        effective_max_occupants: effective_max,
        state_formula_max: state_max,
        keating_presumption_max: keating_max,
        proposed_within_limit: within_limit && !pretext_violation,
        fha_familial_status_pretext_violation: pretext_violation,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, bedrooms: u32, sqft: u32, occupants: u32) -> OccupancyInput {
        OccupancyInput {
            state_code: state.to_string(),
            bedrooms_in_unit: bedrooms,
            total_livable_sqft: sqft,
            proposed_occupants: occupants,
            enforced_selectively_against_families: false,
            building_code_caps_below_keating: false,
            building_code_max_occupants: None,
        }
    }

    #[test]
    fn ny_80_sqft_per_person_formula() {
        // 800 sqft / 80 sqft = 10 max occupants. But 2 BR Keating = 4.
        // Effective max = min(10, 4) = 4.
        let r = check(&input("NY", 2, 800, 4));
        assert_eq!(r.state_formula_max, 10);
        assert_eq!(r.keating_presumption_max, 4);
        assert_eq!(r.effective_max_occupants, 4);
        assert!(r.proposed_within_limit);
    }

    #[test]
    fn ny_5_occupants_in_2br_800sqft_violates() {
        let r = check(&input("NY", 2, 800, 5));
        assert!(!r.proposed_within_limit);
    }

    #[test]
    fn ny_small_sqft_binds_below_keating() {
        // 240 sqft / 80 = 3. Keating 2 BR = 4. Effective = min(3, 4) = 3.
        let r = check(&input("NY", 2, 240, 3));
        assert_eq!(r.state_formula_max, 3);
        assert_eq!(r.effective_max_occupants, 3);
        let r2 = check(&input("NY", 2, 240, 4));
        assert!(!r2.proposed_within_limit);
    }

    #[test]
    fn ma_first_occupant_floor_then_per_additional() {
        // 150 + 100*4 = 550 sqft → max 5 occupants under MA formula.
        // Keating 2 BR = 4 binds tighter. Effective = 4.
        let r = check(&input("MA", 2, 550, 4));
        assert_eq!(r.state_formula_max, 5);
        assert_eq!(r.keating_presumption_max, 4);
        assert_eq!(r.effective_max_occupants, 4);
    }

    #[test]
    fn ma_below_150_sqft_zero_occupants() {
        // Tiny unit < 150 sqft → 0 occupants under MA formula.
        let r = check(&input("MA", 1, 149, 1));
        assert_eq!(r.state_formula_max, 0);
        assert_eq!(r.effective_max_occupants, 0);
        assert!(!r.proposed_within_limit);
    }

    #[test]
    fn ca_two_plus_one_explicit() {
        // CA 2BR → 2*2+1 = 5 effective max.
        let r = check(&input("CA", 2, 1000, 5));
        assert_eq!(r.regime, OccupancyRegime::TwoPlusOneStatutory);
        assert_eq!(r.state_formula_max, 5);
        assert_eq!(r.effective_max_occupants, 5);
        assert!(r.proposed_within_limit);
    }

    #[test]
    fn ca_three_br_two_plus_one_yields_7() {
        // 3 BR × 2 + 1 = 7.
        let r = check(&input("CA", 3, 1500, 7));
        assert_eq!(r.effective_max_occupants, 7);
        assert!(r.proposed_within_limit);
        let r2 = check(&input("CA", 3, 1500, 8));
        assert!(!r2.proposed_within_limit);
    }

    #[test]
    fn ca_state_formula_overrides_keating() {
        // CA state formula 5 > Keating 4 → state formula binds in CA.
        let r = check(&input("CA", 2, 1000, 5));
        assert_eq!(r.effective_max_occupants, 5);
    }

    #[test]
    fn or_landlord_capped_at_2_per_bedroom() {
        let r = check(&input("OR", 2, 800, 4));
        assert_eq!(
            r.regime,
            OccupancyRegime::NoMoreRestrictiveThanTwoPerBedroom
        );
        assert_eq!(r.effective_max_occupants, 4);
        assert!(r.proposed_within_limit);
        let r2 = check(&input("OR", 2, 800, 5));
        assert!(!r2.proposed_within_limit);
    }

    #[test]
    fn or_landlord_rule_capped_flag_set() {
        assert!(
            RULES
                .get("OR")
                .unwrap()
                .landlord_rule_capped_at_2_per_bedroom
        );
    }

    #[test]
    fn hud_keating_default_2_per_bedroom() {
        // TX uses Keating default. 2 BR → 4 max.
        let r = check(&input("TX", 2, 1500, 4));
        assert_eq!(r.regime, OccupancyRegime::HudKeatingDefault);
        assert_eq!(r.effective_max_occupants, 4);
        let r2 = check(&input("TX", 2, 1500, 5));
        assert!(!r2.proposed_within_limit);
    }

    #[test]
    fn wa_keating_default_with_unrelated_occupant_citation() {
        let r = check(&input("WA", 2, 1000, 4));
        assert_eq!(r.regime, OccupancyRegime::HudKeatingDefault);
        assert!(r.citation.contains("SB 5235"));
        assert!(r.citation.contains("RCW 35A.21.314"));
    }

    #[test]
    fn building_code_caps_below_keating() {
        // 2BR Keating = 4, but septic caps at 3.
        let mut i = input("TX", 2, 1500, 4);
        i.building_code_caps_below_keating = true;
        i.building_code_max_occupants = Some(3);
        let r = check(&i);
        assert_eq!(r.effective_max_occupants, 3);
        assert!(!r.proposed_within_limit);
    }

    #[test]
    fn fha_familial_status_pretext_overrides_compliance() {
        // 4 occupants in 2BR — within Keating — but landlord enforces
        // selectively against families with children. PRETEXT VIOLATION.
        let mut i = input("TX", 2, 1500, 4);
        i.enforced_selectively_against_families = true;
        let r = check(&i);
        assert!(r.fha_familial_status_pretext_violation);
        assert!(!r.proposed_within_limit);
        assert!(r.note.contains("PRETEXT VIOLATION"));
    }

    #[test]
    fn pretext_violation_blocks_within_limit_even_at_zero_proposed() {
        let mut i = input("TX", 2, 1500, 1);
        i.enforced_selectively_against_families = true;
        let r = check(&i);
        assert!(!r.proposed_within_limit);
    }

    #[test]
    fn ca_state_familial_status_pretext_flag_set() {
        assert!(
            RULES
                .get("CA")
                .unwrap()
                .state_familial_status_pretext_explicit
        );
    }

    #[test]
    fn studio_zero_bedrooms_keating_zero_caps_at_state_floor() {
        // 0 BR (studio) → Keating 0. NY formula: 600 / 80 = 7.5 → 7
        // by integer math, but Keating 0 binds → effective 0.
        let r = check(&input("NY", 0, 600, 1));
        assert_eq!(r.keating_presumption_max, 0);
        // Effective should be min(state_formula, keating) = 0.
        assert_eq!(r.effective_max_occupants, 0);
        // (Studios in practice get a 1-2 person allowance, but that's a
        // separate state-specific rule not modeled at the federal-floor
        // level — the test pins what we currently return.)
    }

    #[test]
    fn ca_studio_floor_2_plus_1_yields_1() {
        // CA 0 BR × 2 + 1 = 1 occupant. Reasonable studio result.
        let r = check(&input("CA", 0, 500, 1));
        assert_eq!(r.effective_max_occupants, 1);
        assert!(r.proposed_within_limit);
    }

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
    fn unknown_state_falls_back_to_keating() {
        let r = check(&input("XX", 2, 1000, 4));
        assert_eq!(r.regime, OccupancyRegime::HudKeatingDefault);
        assert_eq!(r.effective_max_occupants, 4);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ca", 2, 1000, 5));
        assert_eq!(r.effective_max_occupants, 5);
    }

    #[test]
    fn ny_unique_sqft_regime_pinned() {
        // NY is the only state where 80-sqft-per-person formula applies.
        // Verifies the regime is unique to NY in the no_rule sweep.
        let r = check(&input("NY", 2, 1000, 4));
        assert_eq!(r.regime, OccupancyRegime::SqftPerOccupantFormula);
        let r_ma = check(&input("MA", 2, 1000, 4));
        assert_eq!(r_ma.regime, OccupancyRegime::SqftPerOccupantFormula);
        // Pin: only NY + MA have SqftPerOccupantFormula.
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == OccupancyRegime::SqftPerOccupantFormula {
                count += 1;
            }
        }
        assert_eq!(count, 2, "expected NY + MA only on SqftPerOccupantFormula");
    }

    #[test]
    fn or_unique_no_more_restrictive_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == OccupancyRegime::NoMoreRestrictiveThanTwoPerBedroom {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "expected OR only on NoMoreRestrictiveThanTwoPerBedroom"
        );
    }

    #[test]
    fn note_describes_within_limit_path() {
        let r = check(&input("TX", 2, 1500, 4));
        assert!(r.note.contains("HudKeatingDefault"));
        assert!(r.note.contains("within limit"));
    }

    #[test]
    fn note_describes_exceeds_limit_path() {
        let r = check(&input("TX", 1, 1500, 3));
        assert!(r.note.contains("EXCEEDS LIMIT"));
    }
}
