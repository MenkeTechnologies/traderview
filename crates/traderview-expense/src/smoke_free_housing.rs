//! Federal HUD + state smoke-free housing compliance.
//!
//! The federal HUD smoke-free rule has been in effect for all public
//! housing since 2018-07-30; private multifamily landlords have no
//! federal mandate but face an expanding patchwork of state and
//! local restrictions. This module covers the federal floor + the
//! handful of states with explicit smoke-free multifamily statutes
//! (CA, MN, OR).
//!
//! **Federal HUD floor (24 CFR § 965.653)** — effective 2018-07-30,
//! all Public Housing Authorities (PHAs) MUST enforce a smoke-free
//! policy covering:
//!
//! - All indoor areas: living units, common areas, electrical
//!   closets, storage units, PHA administrative offices
//! - **25-foot exterior buffer** around housing and admin buildings
//! - All prohibited tobacco products (cigarettes, cigars, pipes,
//!   hookahs); some PHAs include e-cigarettes
//!
//! The federal rule does NOT apply to private market multifamily.
//!
//! Two regimes for state-level additions:
//!
//! `HudFloorPlusStateAdditions`: CA (Berkeley municipal ordinance + Cal.
//! Labor Code § 6404.5 common-area protections; AB 1316 + local bans),
//! MN (Minn. Stat. § 144.414 + 2024 cannabis ban in MUH common-interest
//! communities), OR (landlord conversion to nonsmoking with 90-day
//! written notice to existing tenants under ORS Ch. 90).
//!
//! `HudFloorOnly`: most other states. Federal floor applies to public
//! housing only; private market governed by lease + local ordinances.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmokeFreeRegime {
    HudFloorPlusStateAdditions,
    HudFloorOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    PublicHousingPha,
    PrivateMultifamily,
    PrivateSingleFamily,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: SmokeFreeRegime,
    pub state_requires_common_area_smoke_free: bool,
    /// True if the state requires written notice for landlord
    /// converting an existing tenancy to non-smoking (OR's 90-day
    /// notice under ORS Ch. 90).
    pub state_requires_existing_tenant_conversion_notice: bool,
    pub conversion_notice_required_days: u32,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: SmokeFreeRegime,
    state_requires_common_area_smoke_free: bool,
    state_requires_existing_tenant_conversion_notice: bool,
    conversion_notice_required_days: u32,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        state_requires_common_area_smoke_free,
        state_requires_existing_tenant_conversion_notice,
        conversion_notice_required_days,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use SmokeFreeRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            HudFloorPlusStateAdditions,
            true, false, 0,
            "HUD 24 CFR § 965.653 + Cal. Labor Code § 6404.5 + AB 1316 + local ordinances (Berkeley 2014 smoke-free MUH first in CA) — common area smoke-free required",
        ),
    );
    m.insert(
        "MN",
        rule(
            HudFloorPlusStateAdditions,
            true, false, 0,
            "HUD 24 CFR § 965.653 + Minn. Stat. § 144.414 — common area smoke-free required; 2024 cannabis ban added in MUH common-interest communities (eff. 2024-07-01)",
        ),
    );
    m.insert(
        "OR",
        rule(
            HudFloorPlusStateAdditions,
            false, true, 90,
            "HUD 24 CFR § 965.653 + Or. ORS Ch. 90 — landlord may convert to nonsmoking with 90-day written notice to existing tenants",
        ),
    );

    // HudFloorOnly for all remaining states + DC.
    let hud_only = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NY", "NC", "ND", "OH", "OK", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA",
        "WV", "WI", "WY",
    ];
    for code in hud_only {
        m.insert(
            code,
            rule(
                HudFloorOnly,
                false, false, 0,
                "HUD 24 CFR § 965.653 federal floor for public housing only; private multifamily governed by lease + local ordinances",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeFreeInput {
    pub state_code: String,
    pub property_type: PropertyType,
    pub indoor_smoking_policy_implemented: bool,
    pub common_areas_smoke_free: bool,
    pub outdoor_25_ft_buffer_enforced: bool,
    /// True if the landlord is CONVERTING an existing tenancy to
    /// non-smoking (vs. starting a new tenancy with a no-smoking
    /// lease). Triggers OR's 90-day notice rule.
    pub converting_existing_tenancy_to_nonsmoking: bool,
    pub written_notice_days_given: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeFreeResult {
    pub regime: SmokeFreeRegime,
    pub hud_federal_floor_applies: bool,
    pub state_addition_common_area_required: bool,
    pub state_conversion_notice_required: bool,
    pub hud_indoor_compliant: bool,
    pub hud_25_ft_buffer_compliant: bool,
    pub state_common_area_compliant: bool,
    pub conversion_notice_compliant: bool,
    pub overall_compliant: bool,
    pub violations: Vec<String>,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &SmokeFreeInput) -> SmokeFreeResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: SmokeFreeRegime::HudFloorOnly,
        state_requires_common_area_smoke_free: false,
        state_requires_existing_tenant_conversion_notice: false,
        conversion_notice_required_days: 0,
        citation: "Unknown state code; assuming HUD federal floor only",
    });

    let hud_applies = matches!(input.property_type, PropertyType::PublicHousingPha);

    let mut violations: Vec<String> = Vec::new();

    let hud_indoor_ok = if hud_applies {
        if !input.indoor_smoking_policy_implemented {
            violations
                .push("HUD 24 CFR § 965.653 indoor smoking policy not implemented".to_string());
            false
        } else {
            true
        }
    } else {
        true
    };

    let hud_buffer_ok = if hud_applies {
        if !input.outdoor_25_ft_buffer_enforced {
            violations.push("HUD 25-ft outdoor buffer not enforced".to_string());
            false
        } else {
            true
        }
    } else {
        true
    };

    let state_common_area_required = rule.state_requires_common_area_smoke_free
        && matches!(input.property_type, PropertyType::PrivateMultifamily);
    let state_common_ok = if state_common_area_required {
        if !input.common_areas_smoke_free {
            violations.push("State requires common-area smoke-free policy (CA/MN MUH)".to_string());
            false
        } else {
            true
        }
    } else {
        true
    };

    let conversion_required = rule.state_requires_existing_tenant_conversion_notice
        && input.converting_existing_tenancy_to_nonsmoking;
    let conversion_ok = if conversion_required {
        if input.written_notice_days_given < rule.conversion_notice_required_days {
            violations.push(format!(
                "OR existing-tenant conversion requires {}-day written notice; only {} given",
                rule.conversion_notice_required_days, input.written_notice_days_given,
            ));
            false
        } else {
            true
        }
    } else {
        true
    };

    let overall = hud_indoor_ok && hud_buffer_ok && state_common_ok && conversion_ok;

    let note = match (rule.regime, hud_applies, overall) {
        (_, true, true) =>
            "HUD federal floor: PHA fully compliant with 24 CFR § 965.653 indoor + 25-ft buffer.".to_string(),
        (_, true, false) =>
            format!("HUD federal floor VIOLATION: PHA non-compliance — {}.", violations.join("; ")),
        (SmokeFreeRegime::HudFloorPlusStateAdditions, false, true) =>
            "HudFloorPlusStateAdditions: private property compliant with state common-area / conversion notice rules.".to_string(),
        (SmokeFreeRegime::HudFloorPlusStateAdditions, false, false) =>
            format!("HudFloorPlusStateAdditions VIOLATION: {}.", violations.join("; ")),
        (SmokeFreeRegime::HudFloorOnly, false, _) =>
            "HudFloorOnly: private property — no state mandate; lease + local ordinances govern.".to_string(),
    };

    SmokeFreeResult {
        regime: rule.regime,
        hud_federal_floor_applies: hud_applies,
        state_addition_common_area_required: state_common_area_required,
        state_conversion_notice_required: conversion_required,
        hud_indoor_compliant: hud_indoor_ok,
        hud_25_ft_buffer_compliant: hud_buffer_ok,
        state_common_area_compliant: state_common_ok,
        conversion_notice_compliant: conversion_ok,
        overall_compliant: overall,
        violations,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, ptype: PropertyType) -> SmokeFreeInput {
        SmokeFreeInput {
            state_code: state.to_string(),
            property_type: ptype,
            indoor_smoking_policy_implemented: false,
            common_areas_smoke_free: false,
            outdoor_25_ft_buffer_enforced: false,
            converting_existing_tenancy_to_nonsmoking: false,
            written_notice_days_given: 0,
        }
    }

    // Federal HUD floor — PHA.

    #[test]
    fn pha_with_full_compliance_passes() {
        let mut i = input("TX", PropertyType::PublicHousingPha);
        i.indoor_smoking_policy_implemented = true;
        i.outdoor_25_ft_buffer_enforced = true;
        let r = check(&i);
        assert!(r.hud_federal_floor_applies);
        assert!(r.overall_compliant);
    }

    #[test]
    fn pha_without_indoor_policy_violates() {
        let mut i = input("TX", PropertyType::PublicHousingPha);
        i.outdoor_25_ft_buffer_enforced = true;
        // Missing indoor policy
        let r = check(&i);
        assert!(!r.hud_indoor_compliant);
        assert!(!r.overall_compliant);
        assert!(r.violations.iter().any(|v| v.contains("indoor")));
    }

    #[test]
    fn pha_without_25_ft_buffer_violates() {
        let mut i = input("TX", PropertyType::PublicHousingPha);
        i.indoor_smoking_policy_implemented = true;
        // Missing 25-ft buffer
        let r = check(&i);
        assert!(!r.hud_25_ft_buffer_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn pha_both_violations_listed() {
        let i = input("TX", PropertyType::PublicHousingPha);
        let r = check(&i);
        assert_eq!(r.violations.len(), 2);
    }

    // Private multifamily — state additions.

    #[test]
    fn ca_private_multifamily_requires_common_area_smoke_free() {
        let mut i = input("CA", PropertyType::PrivateMultifamily);
        i.common_areas_smoke_free = false;
        let r = check(&i);
        assert_eq!(r.regime, SmokeFreeRegime::HudFloorPlusStateAdditions);
        assert!(r.state_addition_common_area_required);
        assert!(!r.state_common_area_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn ca_private_multifamily_with_common_area_complies() {
        let mut i = input("CA", PropertyType::PrivateMultifamily);
        i.common_areas_smoke_free = true;
        let r = check(&i);
        assert!(r.overall_compliant);
    }

    #[test]
    fn mn_private_multifamily_common_area_required() {
        let mut i = input("MN", PropertyType::PrivateMultifamily);
        i.common_areas_smoke_free = false;
        let r = check(&i);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn ca_private_single_family_no_common_area_requirement() {
        // §144.414 / Labor Code § 6404.5 common-area rules don't reach
        // single-family rentals.
        let r = check(&input("CA", PropertyType::PrivateSingleFamily));
        assert!(!r.state_addition_common_area_required);
        assert!(r.overall_compliant);
    }

    // OR — conversion notice.

    #[test]
    fn or_conversion_with_90_day_notice_complies() {
        let mut i = input("OR", PropertyType::PrivateMultifamily);
        i.converting_existing_tenancy_to_nonsmoking = true;
        i.written_notice_days_given = 90;
        let r = check(&i);
        assert!(r.state_conversion_notice_required);
        assert!(r.conversion_notice_compliant);
        assert!(r.overall_compliant);
    }

    #[test]
    fn or_conversion_with_89_day_notice_violates() {
        let mut i = input("OR", PropertyType::PrivateMultifamily);
        i.converting_existing_tenancy_to_nonsmoking = true;
        i.written_notice_days_given = 89;
        let r = check(&i);
        assert!(!r.conversion_notice_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn or_new_tenancy_no_conversion_notice_needed() {
        let mut i = input("OR", PropertyType::PrivateMultifamily);
        i.converting_existing_tenancy_to_nonsmoking = false;
        let r = check(&i);
        assert!(!r.state_conversion_notice_required);
        assert!(r.overall_compliant);
    }

    // HudFloorOnly states — private property no state requirement.

    #[test]
    fn tx_private_multifamily_no_state_requirement() {
        let r = check(&input("TX", PropertyType::PrivateMultifamily));
        assert_eq!(r.regime, SmokeFreeRegime::HudFloorOnly);
        assert!(!r.state_addition_common_area_required);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ny_private_multifamily_no_state_requirement() {
        let r = check(&input("NY", PropertyType::PrivateMultifamily));
        assert!(r.overall_compliant);
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
    fn state_additions_only_ca_mn_or() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == SmokeFreeRegime::HudFloorPlusStateAdditions {
                count += 1;
            }
        }
        assert_eq!(count, 3, "expected CA + MN + OR only with state additions");
    }

    #[test]
    fn only_or_has_conversion_notice() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.state_requires_existing_tenant_conversion_notice {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected OR only with conversion notice");
    }

    #[test]
    fn or_unique_90_day_window_pinned() {
        let or = RULES.get("OR").unwrap();
        assert_eq!(or.conversion_notice_required_days, 90);
    }

    #[test]
    fn only_ca_mn_have_common_area_requirement() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.state_requires_common_area_smoke_free {
                count += 1;
            }
        }
        assert_eq!(
            count, 2,
            "expected CA + MN only with common-area requirement"
        );
    }

    #[test]
    fn unknown_state_falls_back_to_hud_only() {
        let r = check(&input("XX", PropertyType::PrivateMultifamily));
        assert_eq!(r.regime, SmokeFreeRegime::HudFloorOnly);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = input("ca", PropertyType::PrivateMultifamily);
        i.common_areas_smoke_free = false;
        let r = check(&i);
        assert!(!r.overall_compliant);
    }

    // Citations.

    #[test]
    fn pha_violation_note_describes_hud() {
        let i = input("TX", PropertyType::PublicHousingPha);
        let r = check(&i);
        assert!(r.note.contains("HUD federal floor VIOLATION"));
    }

    #[test]
    fn mn_citation_mentions_2024_cannabis_amendment() {
        let r = check(&input("MN", PropertyType::PrivateMultifamily));
        assert!(r.citation.contains("2024"));
        assert!(r.citation.contains("cannabis"));
    }
}
