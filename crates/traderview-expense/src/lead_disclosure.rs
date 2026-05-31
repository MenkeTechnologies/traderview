//! State-by-state lead-based paint disclosure + abatement compliance.
//!
//! **Federal floor (universal)**: Section 1018 of Title X of the
//! Residential Lead-Based Paint Hazard Reduction Act of 1992 (40 CFR
//! Part 745 / 24 CFR Part 35 Subpart A) — applies to all pre-1978
//! private rentals nationwide. Landlords must:
//!
//!   1. Provide the EPA pamphlet "Protect Your Family From Lead in
//!      Your Home" (or state-approved equivalent)
//!   2. Disclose any known lead-based paint and hazards
//!   3. Provide records of any prior lead inspections / risk
//!      assessments
//!   4. Include the federal Lead Warning Statement language in the
//!      lease
//!   5. Allow a 10-day window for the tenant to conduct a risk
//!      assessment before the lease binds (may be waived/lengthened by
//!      mutual written agreement)
//!
//! **Federal penalty**: up to **$10,000 per violation** (EPA), plus
//! the tenant may sue for treble damages.
//!
//! **State additions** (above the federal floor):
//!
//! - **MA** (1971, strictest in the country) — "deleading" required
//!   when a child under 6 occupies a pre-1978 rental. Landlord must
//!   remove or permanently cover all lead-paint hazards regardless of
//!   blood-lead level.
//! - **NJ** P.L.2021, c.182 (effective July 2022) — periodic
//!   inspection of all pre-1978 rentals at the earlier of July 22,
//!   2024 or first tenant turnover; ongoing 3-year cycle.
//! - **NY** NYC Local Law 1 of 2004 — annual landlord investigation in
//!   pre-1960 multi-family buildings with children under 6.
//! - **MD** Reduction of Lead Risk in Housing Act (1996) — universal
//!   registration of pre-1950 rentals + risk reduction at occupancy
//!   change. Expanded to pre-1978 in 2015.
//! - **CT**, **RI**, **VT**, **IL**, **WI**, **MN** — varying
//!   risk-reduction or inspection regimes layered atop the federal
//!   disclosure floor.
//!
//! Most other states rely on the federal floor alone with no
//! additional state-specific obligations.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LeadStateRegime {
    /// Federal Title X floor only; no state additions.
    FederalFloorOnly,
    /// State requires lead-paint deleading / abatement when a child
    /// under 6 occupies the unit (MA model).
    StateChildBasedDeleading,
    /// State requires periodic inspection on a fixed cycle (NJ 2022,
    /// NY NYC LL1, MD).
    StatePeriodicInspection,
    /// State requires inspection at change of occupancy (RI, VT, IL).
    StateInspectionAtOccupancyChange,
    /// State comprehensive lead-safe regime (typically combines child
    /// trigger + periodic inspection + occupancy-change inspection).
    StateComprehensive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateLeadRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: LeadStateRegime,
    /// True if the state requires landlord action specifically when a
    /// child under 6 occupies the unit (MA, NJ, NY NYC LL1).
    pub child_under_6_triggers_state_action: bool,
    /// True if the state requires periodic inspections of pre-1978
    /// rentals on a recurring cycle.
    pub periodic_inspection_required: bool,
    /// True if the state requires inspection at each change of
    /// occupancy.
    pub inspection_at_change_of_occupancy: bool,
    /// True if the state recognizes treble damages above the federal
    /// floor treble in Title X § 1018.
    pub state_treble_damages_available: bool,
    pub citation: &'static str,
}

/// Federal cutoff year — pre-1978 housing triggers Title X disclosure.
const FEDERAL_PRE_1978_CUTOFF: u32 = 1978;

/// Maximum federal civil penalty per Title X violation as enforced by
/// EPA. Source: 40 CFR § 745.118(c). Periodically adjusted for
/// inflation by EPA but $10,000 remains the statutory baseline.
const FEDERAL_PENALTY_PER_VIOLATION_CENTS: i64 = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadCheckInput {
    pub state_code: String,
    pub year_built: u32,
    pub child_under_6_in_household: bool,
    pub federal_lead_warning_in_lease: bool,
    pub epa_pamphlet_provided: bool,
    pub known_lead_records_disclosed: bool,
    pub ten_day_assessment_window_offered: bool,
    /// State-side: was the required inspection completed (periodic /
    /// occupancy-change, as the state regime requires)?
    pub state_required_inspection_completed: bool,
    /// State-side: when child under 6 triggers state action, did the
    /// landlord complete the required deleading / abatement?
    pub state_required_deleading_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadCheckResult {
    pub federal_disclosure_required: bool,
    pub state_additional_required: bool,
    pub complies: bool,
    pub violations: Vec<String>,
    /// Maximum federal civil penalty per violation under 40 CFR
    /// § 745.118(c).
    pub max_federal_penalty_per_violation_cents: i64,
    pub state_treble_damages_available: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateLeadRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateLeadRule> {
    let mut v: Vec<&'static StateLeadRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &LeadCheckInput) -> LeadCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return LeadCheckResult {
                federal_disclosure_required: input.year_built < FEDERAL_PRE_1978_CUTOFF,
                state_additional_required: false,
                complies: false,
                violations: vec!["unknown state code".to_string()],
                max_federal_penalty_per_violation_cents: FEDERAL_PENALTY_PER_VIOLATION_CENTS,
                state_treble_damages_available: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let federal_required = input.year_built < FEDERAL_PRE_1978_CUTOFF;
    let state_required = !matches!(rule.regime, LeadStateRegime::FederalFloorOnly) && federal_required;

    let mut violations: Vec<String> = Vec::new();

    // Federal floor violations (only if pre-1978).
    if federal_required {
        if !input.federal_lead_warning_in_lease {
            violations.push(
                "Federal Title X Lead Warning Statement language missing from lease".into(),
            );
        }
        if !input.epa_pamphlet_provided {
            violations.push(
                "EPA pamphlet \"Protect Your Family From Lead in Your Home\" not provided"
                    .into(),
            );
        }
        if !input.known_lead_records_disclosed {
            violations.push(
                "Known lead records / prior inspections not disclosed".into(),
            );
        }
        if !input.ten_day_assessment_window_offered {
            violations.push(
                "10-day risk-assessment window not offered to tenant".into(),
            );
        }
    }

    // State-specific violations (only if pre-1978 and state has additions).
    if federal_required {
        if rule.child_under_6_triggers_state_action
            && input.child_under_6_in_household
            && !input.state_required_deleading_completed
        {
            violations.push(format!(
                "{} requires deleading / abatement when child under 6 occupies; not completed",
                rule.state_name
            ));
        }
        if rule.periodic_inspection_required
            && !input.state_required_inspection_completed
        {
            violations.push(format!(
                "{} requires periodic lead inspection; not completed",
                rule.state_name
            ));
        }
        if rule.inspection_at_change_of_occupancy
            && !input.state_required_inspection_completed
        {
            violations.push(format!(
                "{} requires inspection at change of occupancy; not completed",
                rule.state_name
            ));
        }
    }

    let complies = violations.is_empty();
    let note = if !federal_required {
        format!(
            "{}: property built {} (post-1978) — no federal Title X obligation; no state lead requirements apply",
            rule.state_name, input.year_built
        )
    } else if complies {
        format!(
            "{}: pre-1978 property federal Title X + state requirements satisfied",
            rule.state_name
        )
    } else {
        format!(
            "{}: pre-1978 property — {} violation(s); maximum federal civil penalty {}¢ per violation under 40 CFR § 745.118; tenant may also sue for treble damages",
            rule.state_name,
            violations.len(),
            FEDERAL_PENALTY_PER_VIOLATION_CENTS
        )
    };

    LeadCheckResult {
        federal_disclosure_required: federal_required,
        state_additional_required: state_required,
        complies,
        violations,
        max_federal_penalty_per_violation_cents: FEDERAL_PENALTY_PER_VIOLATION_CENTS,
        state_treble_damages_available: rule.state_treble_damages_available,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: LeadStateRegime,
    child_under_6_triggers_state_action: bool,
    periodic_inspection_required: bool,
    inspection_at_change_of_occupancy: bool,
    state_treble_damages_available: bool,
    citation: &'static str,
) -> StateLeadRule {
    StateLeadRule {
        state_code,
        state_name,
        regime,
        child_under_6_triggers_state_action,
        periodic_inspection_required,
        inspection_at_change_of_occupancy,
        state_treble_damages_available,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateLeadRule>> = Lazy::new(|| {
    use LeadStateRegime::*;
    static RULES: &[StateLeadRule] = &[
        rule("AK", "Alaska", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("AL", "Alabama", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("AR", "Arkansas", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("AZ", "Arizona", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("CA", "California", FederalFloorOnly, false, false, false, true, "federal Title X + Cal. Health & Safety Code § 17920.10"),
        rule("CO", "Colorado", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "CT",
            "Connecticut",
            StateChildBasedDeleading,
            true,
            false,
            false,
            true,
            "Conn. Gen. Stat. § 47a-7a + CT Public Health Code § 19a-111",
        ),
        rule(
            "DC",
            "District of Columbia",
            StatePeriodicInspection,
            true,
            true,
            false,
            true,
            "D.C. Code § 8-231 (Lead Hazard Prevention and Elimination Act)",
        ),
        rule("DE", "Delaware", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("FL", "Florida", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("GA", "Georgia", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("HI", "Hawaii", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("IA", "Iowa", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("ID", "Idaho", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "IL",
            "Illinois",
            StateInspectionAtOccupancyChange,
            true,
            false,
            true,
            true,
            "410 ILCS 45 + 410 ILCS 67 (Lead Poisoning Prevention Act)",
        ),
        rule("IN", "Indiana", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("KS", "Kansas", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("KY", "Kentucky", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("LA", "Louisiana", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "MA",
            "Massachusetts",
            StateChildBasedDeleading,
            true,
            false,
            false,
            true,
            "M.G.L. c. 111 § 197 (Massachusetts Lead Law, 1971 — strictest)",
        ),
        rule(
            "MD",
            "Maryland",
            StatePeriodicInspection,
            false,
            true,
            true,
            true,
            "Md. Code Environment §§ 6-801 to 6-852 (Reduction of Lead Risk in Housing Act 1996, expanded 2015)",
        ),
        rule("ME", "Maine", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("MI", "Michigan", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "MN",
            "Minnesota",
            StateChildBasedDeleading,
            true,
            false,
            false,
            false,
            "Minn. Stat. § 144.9501",
        ),
        rule("MO", "Missouri", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("MS", "Mississippi", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("MT", "Montana", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("NC", "North Carolina", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("ND", "North Dakota", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("NE", "Nebraska", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("NH", "New Hampshire", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "NJ",
            "New Jersey",
            StatePeriodicInspection,
            false,
            true,
            true,
            true,
            "N.J. P.L. 2021, c.182 (Lead-Safe Law, July 2022)",
        ),
        rule("NM", "New Mexico", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("NV", "Nevada", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "NY",
            "New York",
            StateComprehensive,
            true,
            true,
            false,
            true,
            "NY Pub. Health Law § 1370 + NYC Local Law 1 of 2004",
        ),
        rule("OH", "Ohio", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("OK", "Oklahoma", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("OR", "Oregon", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("PA", "Pennsylvania", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "RI",
            "Rhode Island",
            StateInspectionAtOccupancyChange,
            true,
            false,
            true,
            true,
            "R.I. Gen. Laws § 42-128.1 (Lead Hazard Mitigation Act 2002)",
        ),
        rule("SC", "South Carolina", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("SD", "South Dakota", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("TN", "Tennessee", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("TX", "Texas", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("UT", "Utah", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("VA", "Virginia", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "VT",
            "Vermont",
            StateInspectionAtOccupancyChange,
            true,
            false,
            true,
            true,
            "18 V.S.A. § 1759 (Essential Maintenance Practices)",
        ),
        rule("WA", "Washington", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule(
            "WI",
            "Wisconsin",
            StateChildBasedDeleading,
            true,
            false,
            false,
            false,
            "Wis. Stat. § 254.18",
        ),
        rule("WV", "West Virginia", FederalFloorOnly, false, false, false, false, "federal Title X only"),
        rule("WY", "Wyoming", FederalFloorOnly, false, false, false, false, "federal Title X only"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_compliant(state: &str, year: u32) -> LeadCheckInput {
        LeadCheckInput {
            state_code: state.to_string(),
            year_built: year,
            child_under_6_in_household: false,
            federal_lead_warning_in_lease: true,
            epa_pamphlet_provided: true,
            known_lead_records_disclosed: true,
            ten_day_assessment_window_offered: true,
            state_required_inspection_completed: true,
            state_required_deleading_completed: true,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn post_1978_property_no_federal_obligation() {
        // 1980-built rental → no Title X obligation regardless of state.
        // Federal disclosure required = false; complies = true.
        let i = fully_compliant("CA", 1980);
        let r = check(&i);
        assert!(!r.federal_disclosure_required);
        assert!(r.complies);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn property_built_1978_exact_is_post_1978_no_obligation() {
        // 1978 exact = NOT pre-1978 (cutoff is < 1978). Pinned because
        // the > / >= boundary is the regulatory bright line.
        let i = fully_compliant("CA", 1978);
        let r = check(&i);
        assert!(!r.federal_disclosure_required);
    }

    #[test]
    fn property_built_1977_triggers_federal_floor() {
        // 1977 = pre-1978 → federal Title X applies.
        let mut i = fully_compliant("CA", 1977);
        // Missing EPA pamphlet → violation.
        i.epa_pamphlet_provided = false;
        let r = check(&i);
        assert!(r.federal_disclosure_required);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("EPA pamphlet")));
    }

    #[test]
    fn federal_floor_all_four_disclosure_elements_required() {
        // Missing each of the four elements individually should violate.
        for missing in 0..4 {
            let mut i = fully_compliant("CA", 1970);
            match missing {
                0 => i.federal_lead_warning_in_lease = false,
                1 => i.epa_pamphlet_provided = false,
                2 => i.known_lead_records_disclosed = false,
                3 => i.ten_day_assessment_window_offered = false,
                _ => unreachable!(),
            }
            let r = check(&i);
            assert!(!r.complies, "Missing element {missing} should violate");
            assert_eq!(r.violations.len(), 1);
        }
    }

    #[test]
    fn ma_strict_law_requires_deleading_with_child_under_6() {
        // MA: child under 6 in pre-1978 → deleading required.
        // Federal compliance alone is not sufficient.
        let mut i = fully_compliant("MA", 1970);
        i.child_under_6_in_household = true;
        i.state_required_deleading_completed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("deleading") && v.contains("child")));
    }

    #[test]
    fn ma_no_child_under_6_no_state_violation() {
        // MA without child under 6 → only federal floor applies.
        let i = fully_compliant("MA", 1970);
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn nj_lead_safe_law_periodic_inspection_required() {
        // NJ Lead-Safe Law 2022: periodic inspection required even
        // without child in household. Missing inspection → violation.
        let mut i = fully_compliant("NJ", 1970);
        i.state_required_inspection_completed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("periodic lead inspection")));
    }

    #[test]
    fn ri_inspection_at_change_of_occupancy_required() {
        // RI Lead Hazard Mitigation Act: inspection at each change of
        // occupancy.
        let mut i = fully_compliant("RI", 1970);
        i.state_required_inspection_completed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("change of occupancy")));
    }

    #[test]
    fn ny_comprehensive_requires_both_periodic_and_child_action() {
        // NY: StateComprehensive — periodic + child-under-6 trigger
        // both apply. Missing both → 2 violations.
        let mut i = fully_compliant("NY", 1970);
        i.child_under_6_in_household = true;
        i.state_required_inspection_completed = false;
        i.state_required_deleading_completed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn federal_floor_only_states_no_state_additions() {
        // TX, FL, CO, AZ etc. — federal floor only. State-side missing
        // should not add violations.
        for code in ["TX", "FL", "CO", "AZ", "AL", "WY", "ID", "OH"] {
            let mut i = fully_compliant(code, 1970);
            i.child_under_6_in_household = true;
            i.state_required_inspection_completed = false;
            i.state_required_deleading_completed = false;
            let r = check(&i);
            assert!(r.complies, "{code} should comply (federal floor only)");
            assert!(!r.state_additional_required);
        }
    }

    #[test]
    fn unknown_state_handled() {
        let i = fully_compliant("ZZ", 1970);
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("MA").is_some());
        assert!(lookup("ma").is_some());
        assert!(lookup("Ma").is_some());
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
    fn child_based_deleading_states_pinned() {
        // CT, MA, MN, WI, NY — states that trigger state action on
        // child-under-6 occupancy.
        for code in ["CT", "MA", "MN", "WI", "NY", "IL", "RI", "VT", "DC"] {
            let r = lookup(code).unwrap();
            assert!(
                r.child_under_6_triggers_state_action,
                "{code} should trigger state action on child under 6"
            );
        }
    }

    #[test]
    fn periodic_inspection_states_pinned() {
        // NJ (2022), MD, NY (NYC LL1), DC — periodic inspection regime.
        for code in ["NJ", "MD", "NY", "DC"] {
            let r = lookup(code).unwrap();
            assert!(
                r.periodic_inspection_required,
                "{code} should require periodic inspection"
            );
        }
    }

    #[test]
    fn occupancy_change_inspection_states_pinned() {
        // IL, RI, VT, MD — inspection at occupancy change.
        for code in ["IL", "RI", "VT", "MD"] {
            let r = lookup(code).unwrap();
            assert!(
                r.inspection_at_change_of_occupancy,
                "{code} should require inspection at occupancy change"
            );
        }
    }

    #[test]
    fn post_1978_property_state_lead_rules_dont_apply() {
        // 1980 rental in MA → no state lead rules apply (MA Lead Law
        // is pre-1978 housing only).
        let mut i = fully_compliant("MA", 1980);
        i.child_under_6_in_household = true;
        i.state_required_deleading_completed = false;
        let r = check(&i);
        assert!(r.complies);
        assert!(!r.federal_disclosure_required);
    }

    #[test]
    fn federal_penalty_constant_matches_40_cfr() {
        // $10,000 per violation per 40 CFR § 745.118(c).
        let i = fully_compliant("CA", 1970);
        let r = check(&i);
        assert_eq!(r.max_federal_penalty_per_violation_cents, 1_000_000);
    }

    #[test]
    fn treble_damages_flag_pinned() {
        // States that recognize treble damages for tenant suit: CA, CT,
        // DC, IL, MA, MD, NJ, NY, RI, VT.
        for code in ["CA", "CT", "DC", "IL", "MA", "MD", "NJ", "NY", "RI", "VT"] {
            let r = lookup(code).unwrap();
            assert!(
                r.state_treble_damages_available,
                "{code} should allow treble damages"
            );
        }
    }

    #[test]
    fn multiple_simultaneous_violations_stack() {
        // NJ has both periodic_inspection AND inspection_at_change_of_
        // occupancy true. Missing pamphlet + missing 10-day window +
        // both NJ inspection violations = 4 distinct violations. Pinned
        // to ensure each inspection rule emits its own line in the
        // violations array rather than collapsing.
        let mut i = fully_compliant("NJ", 1970);
        i.epa_pamphlet_provided = false;
        i.ten_day_assessment_window_offered = false;
        i.state_required_inspection_completed = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 4);
    }

    #[test]
    fn note_describes_post_1978_no_obligation() {
        let i = fully_compliant("MA", 1990);
        let r = check(&i);
        assert!(r.note.contains("post-1978"));
        assert!(r.note.contains("no federal Title X"));
    }
}
