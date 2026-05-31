//! State-by-state bedbug disclosure + inspection-duty compliance.
//!
//! Recent legislative wave (2009-2024) — ~23 states have enacted bedbug-
//! specific legislation following the post-2000 nationwide resurgence
//! that hit dense urban housing hardest. The variation across regimes
//! is significant; most states cluster into four patterns:
//!
//! 1. **Pre-lease history disclosure** — CA Civ. Code § 1954.603 (2017)
//!    requires landlord to disclose any history of infestation in the
//!    unit or building before the tenant signs.
//! 2. **Informational pamphlet only** — AZ A.R.S. § 33-1319 requires
//!    landlord to provide bedbug educational materials but no history
//!    disclosure. Single-family homes exempted.
//! 3. **Post-discovery adjacent-unit notice** — NY RPL § 235-j (2010,
//!    amended 2024) requires landlord to give written notice within 72
//!    hours to tenants of immediately adjacent units (above/below/side)
//!    after learning of an infestation.
//! 4. **Inspection duty on tenant report** — ME 14 M.R.S. § 6021-A
//!    requires landlord to inspect within 5 days of tenant report and
//!    contact licensed pest control within 10 days if confirmed.
//!
//! **No statewide statute** — about half the country still relies on
//! the implied warranty of habitability for bedbug remedies. Local
//! ordinances may apply (NYC Local Law 69 layered on top of NY state
//! law, for example).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BedbugDisclosureRegime {
    PreLeaseHistoryDisclosure,
    InformationalOnly,
    PostDiscoveryAdjacentNotice,
    InspectionDutyOnReport,
    /// State has multiple of the above requirements combined.
    Comprehensive,
    NoStateStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBedbugRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: BedbugDisclosureRegime,
    pub pre_lease_history_disclosure_required: bool,
    pub info_pamphlet_required: bool,
    /// Hours after the landlord learns of infestation in which adjacent-
    /// unit notice must be given. NY = 72.
    pub adjacent_notice_hours: Option<u32>,
    /// Days after tenant report within which landlord must inspect.
    /// ME = 5 days.
    pub inspection_duty_days: Option<u32>,
    /// True if single-family homes are exempted from the regime
    /// (AZ excludes SFR explicitly).
    pub single_family_homes_exempted: bool,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedbugCheckInput {
    pub state_code: String,
    pub is_single_family_home: bool,
    pub pre_lease_history_disclosed: bool,
    pub info_pamphlet_provided: bool,
    pub landlord_learned_of_infestation: bool,
    pub adjacent_units_notified: bool,
    pub hours_since_landlord_learned: u32,
    pub tenant_reported_infestation: bool,
    pub days_since_tenant_report: u32,
    pub inspection_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedbugCheckResult {
    pub disclosure_required: bool,
    pub complies: bool,
    pub violations: Vec<String>,
    pub no_statute_in_state: bool,
    pub single_family_exempt: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateBedbugRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateBedbugRule> {
    let mut v: Vec<&'static StateBedbugRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &BedbugCheckInput) -> BedbugCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return BedbugCheckResult {
                disclosure_required: false,
                complies: false,
                violations: vec!["unknown state code".to_string()],
                no_statute_in_state: true,
                single_family_exempt: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let no_statute = matches!(rule.regime, BedbugDisclosureRegime::NoStateStatute);
    let sfh_exempt = input.is_single_family_home && rule.single_family_homes_exempted;

    if no_statute {
        return BedbugCheckResult {
            disclosure_required: false,
            complies: true,
            violations: vec![],
            no_statute_in_state: true,
            single_family_exempt: false,
            citation: rule.citation,
            note: format!(
                "{}: no statewide bedbug statute — implied habitability covenant + local ordinances apply",
                rule.state_name
            ),
        };
    }

    if sfh_exempt {
        return BedbugCheckResult {
            disclosure_required: false,
            complies: true,
            violations: vec![],
            no_statute_in_state: false,
            single_family_exempt: true,
            citation: rule.citation,
            note: format!(
                "{}: single-family home exempt from {} bedbug regime",
                rule.state_name, rule.state_name
            ),
        };
    }

    let mut violations: Vec<String> = Vec::new();

    // Pre-lease history disclosure check.
    if rule.pre_lease_history_disclosure_required && !input.pre_lease_history_disclosed {
        violations.push(format!(
            "{} requires pre-lease disclosure of any bedbug infestation history; not disclosed",
            rule.state_name
        ));
    }

    // Informational pamphlet check.
    if rule.info_pamphlet_required && !input.info_pamphlet_provided {
        violations.push(format!(
            "{} requires bedbug informational pamphlet/sheet; not provided",
            rule.state_name
        ));
    }

    // Adjacent-unit notice check (only if landlord has actually learned
    // of an infestation).
    if let Some(window_hours) = rule.adjacent_notice_hours {
        if input.landlord_learned_of_infestation {
            let past_deadline = input.hours_since_landlord_learned > window_hours;
            if past_deadline && !input.adjacent_units_notified {
                violations.push(format!(
                    "{} requires adjacent-unit notice within {}h of learning of infestation; {}h have passed without notice",
                    rule.state_name, window_hours, input.hours_since_landlord_learned
                ));
            }
        }
    }

    // Inspection duty on tenant report (only if tenant has reported).
    if let Some(window_days) = rule.inspection_duty_days {
        if input.tenant_reported_infestation {
            let past_deadline = input.days_since_tenant_report > window_days;
            if past_deadline && !input.inspection_completed {
                violations.push(format!(
                    "{} requires landlord inspection within {}d of tenant report; {}d have passed without inspection",
                    rule.state_name, window_days, input.days_since_tenant_report
                ));
            }
        }
    }

    let complies = violations.is_empty();
    let note = if complies {
        format!("{}: bedbug disclosure / inspection requirements satisfied", rule.state_name)
    } else {
        format!(
            "{}: {} bedbug compliance violation(s)",
            rule.state_name,
            violations.len()
        )
    };

    BedbugCheckResult {
        disclosure_required: true,
        complies,
        violations,
        no_statute_in_state: false,
        single_family_exempt: false,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: BedbugDisclosureRegime,
    pre_lease_history_disclosure_required: bool,
    info_pamphlet_required: bool,
    adjacent_notice_hours: Option<u32>,
    inspection_duty_days: Option<u32>,
    single_family_homes_exempted: bool,
    citation: &'static str,
) -> StateBedbugRule {
    StateBedbugRule {
        state_code,
        state_name,
        regime,
        pre_lease_history_disclosure_required,
        info_pamphlet_required,
        adjacent_notice_hours,
        inspection_duty_days,
        single_family_homes_exempted,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateBedbugRule>> = Lazy::new(|| {
    use BedbugDisclosureRegime::*;
    static RULES: &[StateBedbugRule] = &[
        rule("AK", "Alaska", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "AL",
            "Alabama",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Ala. Code § 35-9A-204 (general habitability)",
        ),
        rule("AR", "Arkansas", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "AZ",
            "Arizona",
            InformationalOnly,
            false,
            true,
            None,
            None,
            true,
            "A.R.S. § 33-1319",
        ),
        rule(
            "CA",
            "California",
            PreLeaseHistoryDisclosure,
            true,
            true,
            None,
            None,
            false,
            "Cal. Civ. Code § 1954.603 (2017)",
        ),
        rule("CO", "Colorado", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("CT", "Connecticut", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("DC", "District of Columbia", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("DE", "Delaware", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "FL",
            "Florida",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Fla. Stat. § 83.51 (habitability)",
        ),
        rule(
            "GA",
            "Georgia",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "O.C.G.A. § 44-7-13 (general habitability)",
        ),
        rule("HI", "Hawaii", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "IA",
            "Iowa",
            InspectionDutyOnReport,
            false,
            false,
            None,
            Some(7),
            false,
            "Iowa Code § 562A.15 (general habitability + 7d response)",
        ),
        rule("ID", "Idaho", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "IL",
            "Illinois",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Chicago Municipal Code § 13-12-090",
        ),
        rule("IN", "Indiana", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "KS",
            "Kansas",
            InspectionDutyOnReport,
            false,
            false,
            None,
            Some(5),
            false,
            "K.S.A. § 58-2576a (5d inspect)",
        ),
        rule("KY", "Kentucky", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("LA", "Louisiana", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("MA", "Massachusetts", NoStateStatute, false, false, None, None, false, "no statewide bedbug statute; M.G.L. c. 111 § 127A habitability"),
        rule("MD", "Maryland", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "ME",
            "Maine",
            InspectionDutyOnReport,
            false,
            false,
            None,
            Some(5),
            false,
            "14 M.R.S. § 6021-A (5d inspect / 10d pest control)",
        ),
        rule(
            "MI",
            "Michigan",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "MCL § 554.139 (habitability)",
        ),
        rule(
            "MN",
            "Minnesota",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Minn. Stat. § 504B.181",
        ),
        rule("MO", "Missouri", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("MS", "Mississippi", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("MT", "Montana", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("NC", "North Carolina", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("ND", "North Dakota", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "NE",
            "Nebraska",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Neb. Rev. Stat. § 76-1419 (habitability)",
        ),
        rule(
            "NH",
            "New Hampshire",
            InspectionDutyOnReport,
            false,
            false,
            None,
            Some(7),
            false,
            "RSA § 540-A (habitability + bedbug rules)",
        ),
        rule(
            "NJ",
            "New Jersey",
            PreLeaseHistoryDisclosure,
            true,
            false,
            None,
            None,
            false,
            "N.J.A.C. § 5:10-3 (Bedbug Protocol)",
        ),
        rule("NM", "New Mexico", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "NV",
            "Nevada",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "NRS § 118A.355 (habitability)",
        ),
        rule(
            "NY",
            "New York",
            PostDiscoveryAdjacentNotice,
            false,
            false,
            Some(72),
            None,
            false,
            "RPL § 235-j (2010, amended 2024)",
        ),
        rule(
            "OH",
            "Ohio",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "ORC § 5321.04 (habitability)",
        ),
        rule("OK", "Oklahoma", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "OR",
            "Oregon",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "ORS § 90.320 (habitability)",
        ),
        rule(
            "PA",
            "Pennsylvania",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Pugh v. Holmes (1979) habitability + Philadelphia ordinances",
        ),
        rule(
            "RI",
            "Rhode Island",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "R.I.G.L. § 34-37 + Code",
        ),
        rule("SC", "South Carolina", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "SD",
            "South Dakota",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "SDCL § 43-32 (habitability)",
        ),
        rule("TN", "Tennessee", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "TX",
            "Texas",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Tex. Prop. Code § 92.052 (habitability)",
        ),
        rule("UT", "Utah", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("VA", "Virginia", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("VT", "Vermont", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule("WA", "Washington", NoStateStatute, false, false, None, None, false, "no statewide statute"),
        rule(
            "WI",
            "Wisconsin",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "Wis. Stat. § 704.07 (habitability)",
        ),
        rule(
            "WV",
            "West Virginia",
            InformationalOnly,
            false,
            true,
            None,
            None,
            false,
            "W. Va. Code § 37-6 (habitability)",
        ),
        rule("WY", "Wyoming", NoStateStatute, false, false, None, None, false, "no statewide statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_compliant(state: &str) -> BedbugCheckInput {
        BedbugCheckInput {
            state_code: state.to_string(),
            is_single_family_home: false,
            pre_lease_history_disclosed: true,
            info_pamphlet_provided: true,
            landlord_learned_of_infestation: false,
            adjacent_units_notified: false,
            hours_since_landlord_learned: 0,
            tenant_reported_infestation: false,
            days_since_tenant_report: 0,
            inspection_completed: false,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn ca_pre_lease_history_disclosure_violation() {
        // CA: pre-lease history disclosure required.
        let mut i = fully_compliant("CA");
        i.pre_lease_history_disclosed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("pre-lease disclosure")));
    }

    #[test]
    fn az_info_pamphlet_required() {
        let mut i = fully_compliant("AZ");
        i.info_pamphlet_provided = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("informational pamphlet")));
    }

    #[test]
    fn az_single_family_home_exempted() {
        // AZ § 33-1319 explicitly exempts single-family homes.
        let mut i = fully_compliant("AZ");
        i.is_single_family_home = true;
        i.info_pamphlet_provided = false; // would normally violate
        let r = check(&i);
        assert!(r.complies);
        assert!(r.single_family_exempt);
    }

    #[test]
    fn ca_sfh_not_exempted_from_disclosure() {
        // CA does NOT exempt single-family. Distinguishes from AZ.
        let mut i = fully_compliant("CA");
        i.is_single_family_home = true;
        i.pre_lease_history_disclosed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(!r.single_family_exempt);
    }

    #[test]
    fn ny_72_hour_adjacent_notice_violation_at_73_hours() {
        // NY RPL § 235-j: 72h adjacent-unit notice. At 73h without
        // notice → violation.
        let mut i = fully_compliant("NY");
        i.landlord_learned_of_infestation = true;
        i.hours_since_landlord_learned = 73;
        i.adjacent_units_notified = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("72h") || v.contains("72 ")));
    }

    #[test]
    fn ny_72_hour_window_complies_at_exactly_72_hours() {
        // Boundary: 72h exactly = within window → complies (statute is
        // "within 72 hours", so at exactly 72h is still within).
        let mut i = fully_compliant("NY");
        i.landlord_learned_of_infestation = true;
        i.hours_since_landlord_learned = 72;
        i.adjacent_units_notified = false;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn ny_no_violation_if_no_infestation_learned_of() {
        // If landlord has not yet learned of infestation, the 72h
        // clock hasn't started. No violation even at large hour count.
        let mut i = fully_compliant("NY");
        i.landlord_learned_of_infestation = false;
        i.hours_since_landlord_learned = 1000;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn ny_no_violation_if_adjacent_notified() {
        // 73h elapsed but notice was given → no violation.
        let mut i = fully_compliant("NY");
        i.landlord_learned_of_infestation = true;
        i.hours_since_landlord_learned = 100;
        i.adjacent_units_notified = true;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn me_5_day_inspection_duty_violation_at_6_days() {
        // ME 14 M.R.S. § 6021-A: 5 days inspection.
        let mut i = fully_compliant("ME");
        i.tenant_reported_infestation = true;
        i.days_since_tenant_report = 6;
        i.inspection_completed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("inspection") && v.contains("5d")));
    }

    #[test]
    fn me_5_day_window_complies_at_exactly_5_days() {
        let mut i = fully_compliant("ME");
        i.tenant_reported_infestation = true;
        i.days_since_tenant_report = 5;
        i.inspection_completed = false;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn me_complies_when_inspection_done_even_past_deadline() {
        // Inspection completed → no violation regardless of day count.
        let mut i = fully_compliant("ME");
        i.tenant_reported_infestation = true;
        i.days_since_tenant_report = 30;
        i.inspection_completed = true;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn ks_5_day_window_pinned() {
        let mut i = fully_compliant("KS");
        i.tenant_reported_infestation = true;
        i.days_since_tenant_report = 6;
        i.inspection_completed = false;
        let r = check(&i);
        assert!(!r.complies);
    }

    #[test]
    fn ia_7_day_window_pinned() {
        let mut i = fully_compliant("IA");
        i.tenant_reported_infestation = true;
        i.days_since_tenant_report = 7;
        i.inspection_completed = false;
        let r = check(&i);
        assert!(r.complies); // 7 days = within 7-day window
        let mut i2 = fully_compliant("IA");
        i2.tenant_reported_infestation = true;
        i2.days_since_tenant_report = 8;
        i2.inspection_completed = false;
        let r2 = check(&i2);
        assert!(!r2.complies);
    }

    #[test]
    fn nj_pre_lease_disclosure_required() {
        // NJ N.J.A.C. § 5:10-3 Bedbug Protocol.
        let mut i = fully_compliant("NJ");
        i.pre_lease_history_disclosed = false;
        let r = check(&i);
        assert!(!r.complies);
    }

    #[test]
    fn no_statute_states_always_comply() {
        // States with NoStateStatute regime: AK / AR / CO / CT / DC /
        // HI / etc. Even with all flags missing, compute reports
        // complies=true and no_statute_in_state=true.
        for code in ["AK", "AR", "CO", "CT", "DC", "HI", "ID", "IN", "KY", "LA", "MO", "MS", "MT", "NC", "ND", "NM", "OK", "SC", "TN", "UT", "VA", "VT", "WA", "WY"] {
            let mut i = fully_compliant(code);
            i.pre_lease_history_disclosed = false;
            i.info_pamphlet_provided = false;
            let r = check(&i);
            assert!(r.complies, "{code} should comply (no statute)");
            assert!(r.no_statute_in_state);
        }
    }

    #[test]
    fn unknown_state_handled() {
        let i = fully_compliant("ZZ");
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("NY").is_some());
        assert!(lookup("ny").is_some());
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
    fn pre_lease_history_states_pinned() {
        // CA and NJ require pre-lease history disclosure.
        for code in ["CA", "NJ"] {
            let r = lookup(code).unwrap();
            assert!(r.pre_lease_history_disclosure_required, "{code}");
        }
    }

    #[test]
    fn ny_is_only_state_with_adjacent_notice_hours() {
        // NY is uniquely on the post-discovery adjacent-unit notice
        // regime with the 72h clock.
        let ny = lookup("NY").unwrap();
        assert_eq!(ny.adjacent_notice_hours, Some(72));
        for r in TABLE.values() {
            if r.state_code != "NY" {
                assert!(
                    r.adjacent_notice_hours.is_none(),
                    "{} should not have adjacent_notice_hours",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn inspection_duty_states_pinned() {
        // ME (5d), KS (5d), IA (7d), NH (7d) have inspection duty.
        for (code, days) in [("ME", 5), ("KS", 5), ("IA", 7), ("NH", 7)] {
            let r = lookup(code).unwrap();
            assert_eq!(
                r.inspection_duty_days,
                Some(days),
                "{code} should have inspection_duty_days = {days}"
            );
        }
    }

    #[test]
    fn multiple_simultaneous_violations_stack() {
        // CA: pre-lease + pamphlet both required. Missing both = 2
        // violations.
        let mut i = fully_compliant("CA");
        i.pre_lease_history_disclosed = false;
        i.info_pamphlet_provided = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 2);
    }
}
