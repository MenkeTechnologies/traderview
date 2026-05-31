//! State-by-state landlord entry notice requirements — how many hours of
//! advance notice the landlord owes the tenant before entering for
//! non-emergency purposes (repairs, inspections, showings to prospective
//! buyers or replacement tenants).
//!
//! Most states with a URLTA-derived code default to 24 hours; about a third
//! follow the older URLTA-1972 "two days" wording. A small group (FL, WI)
//! settled on 12 hours via their own rulemaking. Washington is the outlier
//! with **48 hours** for standard entry and a separate **24 hours** carve-out
//! for showings. Roughly sixteen states have no statutory entry-notice
//! requirement at all — common-law "reasonable notice" applies, but there is
//! no fixed hour count a landlord can be measured against. The "Reasonable"
//! states (CT, MN) follow a similar pattern: the statute exists but does not
//! specify hours, so the field is `None` and downstream code treats it the
//! same as "no statute" for compliance purposes.
//!
//! Emergency entry never requires notice in any state. Most statutes also
//! waive notice when the entry is at the tenant's request (e.g. tenant called
//! for a repair). Both cases are modeled as exceptions on the result side
//! rather than as separate state rows — the rules don't vary by jurisdiction.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reason the landlord is entering. Determines which notice column applies
/// and whether an exception waives notice entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPurpose {
    /// Imminent threat to health/safety or property damage. No notice
    /// required in any state.
    Emergency,
    /// Routine repairs, agreed alterations, supply of services.
    Repairs,
    /// Inspection — landlord-initiated walkthrough.
    Inspection,
    /// Showing to prospective tenants or purchasers near lease end.
    /// Washington has a separate 24h column for this purpose.
    Showing,
    /// Tenant requested the entry (called for a repair, etc.). No notice
    /// required in any state per consent-presumed rule.
    TenantRequested,
    /// Tenant has abandoned the unit (extended absence). Most URLTA states
    /// allow entry without notice once the abandonment threshold is met;
    /// this module does not compute the threshold, only marks the exception.
    Abandoned,
}

/// A single state's entry-notice rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntryRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    /// Hours of advance notice required for standard non-emergency entry
    /// (repairs/inspection). `None` = no statutory hour count; common-law
    /// "reasonable notice" applies but is not measurable in hours.
    pub standard_hours: Option<u32>,
    /// Hours required specifically for showings, if the statute carves them
    /// out separately. `None` = same as `standard_hours`.
    pub showing_hours: Option<u32>,
    /// Statute citation (best-effort canonical short form).
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryNoticeInput {
    pub state_code: String,
    pub purpose: EntryPurpose,
    /// Notice the landlord actually gave, in hours before the planned entry.
    pub notice_hours_given: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryNoticeResult {
    /// True if the notice given satisfies (or exceeds) the statutory minimum
    /// for the requested purpose, OR an exception applies.
    pub complies: bool,
    /// Statutory minimum hours of notice for this purpose, or `None` if
    /// the state has no fixed hour count.
    pub required_hours: Option<u32>,
    /// Hours short of the statutory minimum (0 if compliant or no minimum).
    pub shortfall_hours: u32,
    /// Name of the exception that applied (emergency, tenant-requested,
    /// abandoned), if any.
    pub exception: Option<&'static str>,
    /// Statute citation.
    pub citation: &'static str,
    /// Plain-English note explaining the outcome.
    pub note: String,
}

/// Look up a state's rule by USPS code (e.g. "CA", "NY"). Case-insensitive.
pub fn lookup(state_code: &str) -> Option<&'static StateEntryRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

/// All 50 states + DC. Use `.values()` to iterate.
pub fn all_states() -> Vec<&'static StateEntryRule> {
    let mut v: Vec<&'static StateEntryRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

/// Compute whether the notice given satisfies the state's rule for the
/// stated purpose. Emergency, tenant-requested, and abandoned purposes
/// short-circuit to `complies = true` regardless of notice given.
pub fn compute(input: &EntryNoticeInput) -> EntryNoticeResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return EntryNoticeResult {
                complies: false,
                required_hours: None,
                shortfall_hours: 0,
                exception: None,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    // Exceptions that waive notice in every state.
    let exception = match input.purpose {
        EntryPurpose::Emergency => Some("emergency: imminent threat to health/safety or property"),
        EntryPurpose::TenantRequested => Some("tenant-requested entry: consent presumed"),
        EntryPurpose::Abandoned => Some("tenant has abandoned the unit"),
        _ => None,
    };
    if let Some(reason) = exception {
        return EntryNoticeResult {
            complies: true,
            required_hours: None,
            shortfall_hours: 0,
            exception: Some(reason),
            citation: rule.citation,
            note: format!("no notice required — {reason}"),
        };
    }

    // Pick the right column.
    let required = match input.purpose {
        EntryPurpose::Showing => rule.showing_hours.or(rule.standard_hours),
        _ => rule.standard_hours,
    };

    match required {
        None => EntryNoticeResult {
            complies: true,
            required_hours: None,
            shortfall_hours: 0,
            exception: None,
            citation: rule.citation,
            note: format!(
                "{} has no statutory hour minimum; common-law \"reasonable notice\" applies",
                rule.state_name
            ),
        },
        Some(min) if input.notice_hours_given >= min => EntryNoticeResult {
            complies: true,
            required_hours: Some(min),
            shortfall_hours: 0,
            exception: None,
            citation: rule.citation,
            note: format!(
                "{}h given meets {} statutory minimum of {}h",
                input.notice_hours_given, rule.state_name, min
            ),
        },
        Some(min) => EntryNoticeResult {
            complies: false,
            required_hours: Some(min),
            shortfall_hours: min - input.notice_hours_given,
            exception: None,
            citation: rule.citation,
            note: format!(
                "{}h given is {}h short of {} statutory minimum of {}h",
                input.notice_hours_given,
                min - input.notice_hours_given,
                rule.state_name,
                min
            ),
        },
    }
}

// Helper for table construction.
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    standard_hours: Option<u32>,
    showing_hours: Option<u32>,
    citation: &'static str,
) -> StateEntryRule {
    StateEntryRule {
        state_code,
        state_name,
        standard_hours,
        showing_hours,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateEntryRule>> = Lazy::new(|| {
    // Each entry is the canonical state rule. URLTA-1974 default is 2 days;
    // URLTA-2015 (RURLTA) default is 24 hours. The "no statute" rows are
    // states where neither URLTA nor a parallel state-specific code fixes
    // an hour count.
    static RULES: &[StateEntryRule] = &[
        rule("AL", "Alabama", Some(48), None, "Ala. Code § 35-9A-303"),
        rule("AK", "Alaska", Some(24), None, "AS § 34.03.140"),
        rule("AZ", "Arizona", Some(48), None, "ARS § 33-1343"),
        rule("AR", "Arkansas", None, None, "no statute"),
        rule("CA", "California", Some(24), None, "Cal. Civ. Code § 1954"),
        rule("CO", "Colorado", Some(24), None, "C.R.S. § 38-12-510"),
        rule("CT", "Connecticut", None, None, "Conn. Gen. Stat. § 47a-16 (\"reasonable\")"),
        rule("DE", "Delaware", Some(48), None, "25 Del. C. § 5509"),
        rule("DC", "District of Columbia", Some(48), None, "14 DCMR § 304"),
        rule("FL", "Florida", Some(12), None, "Fla. Stat. § 83.53"),
        rule("GA", "Georgia", None, None, "no statute"),
        rule("HI", "Hawaii", Some(48), None, "HRS § 521-53"),
        rule("ID", "Idaho", None, None, "no statute"),
        rule("IL", "Illinois", None, None, "no state statute (Chicago RLTO § 5-12-050: 48h)"),
        rule("IN", "Indiana", None, None, "no statute"),
        rule("IA", "Iowa", Some(24), None, "Iowa Code § 562A.19"),
        rule("KS", "Kansas", None, None, "K.S.A. § 58-2557 (\"reasonable\")"),
        rule("KY", "Kentucky", Some(48), None, "KRS § 383.615"),
        rule("LA", "Louisiana", None, None, "no statute"),
        rule("ME", "Maine", Some(24), None, "14 M.R.S. § 6025"),
        rule("MD", "Maryland", None, None, "no statute (common-law reasonable)"),
        rule("MA", "Massachusetts", None, None, "no statute (lease governs)"),
        rule("MI", "Michigan", None, None, "no statute"),
        rule("MN", "Minnesota", None, None, "Minn. Stat. § 504B.211 (\"reasonable\")"),
        rule("MS", "Mississippi", None, None, "no statute"),
        rule("MO", "Missouri", None, None, "no statute"),
        rule("MT", "Montana", Some(24), None, "Mont. Code § 70-24-312"),
        rule("NE", "Nebraska", Some(24), None, "Neb. Rev. Stat. § 76-1423"),
        rule("NV", "Nevada", Some(24), None, "NRS § 118A.330"),
        rule("NH", "New Hampshire", None, None, "no statute (\"adequate notice\")"),
        rule("NJ", "New Jersey", None, None, "no statute (case law: reasonable)"),
        rule("NM", "New Mexico", Some(24), None, "NMSA § 47-8-24"),
        rule("NY", "New York", None, None, "no statewide statute (NYC: 24h custom)"),
        rule("NC", "North Carolina", None, None, "no statute"),
        rule("ND", "North Dakota", None, None, "no statute (\"reasonable\")"),
        rule("OH", "Ohio", Some(24), None, "ORC § 5321.04"),
        rule("OK", "Oklahoma", Some(24), None, "41 O.S. § 128"),
        rule("OR", "Oregon", Some(24), None, "ORS § 90.322"),
        rule("PA", "Pennsylvania", None, None, "no statute"),
        rule("RI", "Rhode Island", Some(48), None, "R.I.G.L. § 34-18-26"),
        rule("SC", "South Carolina", Some(24), None, "SC Code § 27-40-530"),
        rule("SD", "South Dakota", Some(24), None, "SDCL § 43-32-32"),
        rule("TN", "Tennessee", Some(24), None, "Tenn. Code § 66-28-403"),
        rule("TX", "Texas", None, None, "no statute"),
        rule("UT", "Utah", Some(24), None, "Utah Code § 57-22-4"),
        rule("VT", "Vermont", Some(48), None, "9 V.S.A. § 4460"),
        rule("VA", "Virginia", Some(24), None, "Va. Code § 55.1-1229"),
        rule("WA", "Washington", Some(48), Some(24), "RCW § 59.18.150"),
        rule("WV", "West Virginia", None, None, "no statute"),
        rule("WI", "Wisconsin", Some(12), None, "Wis. Admin. Code ATCP § 134.09"),
        rule("WY", "Wyoming", None, None, "no statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, purpose: EntryPurpose, given: u32) -> EntryNoticeInput {
        EntryNoticeInput {
            state_code: state.to_string(),
            purpose,
            notice_hours_given: given,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        // 50 + DC = 51 rows. If this number changes, somebody added or
        // dropped a row and the change should be deliberate.
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn lookup_is_case_insensitive() {
        // Equally valid spellings for the state code.
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
        assert!(lookup("Ca").is_some());
    }

    #[test]
    fn lookup_returns_none_for_unknown_state() {
        assert!(lookup("ZZ").is_none());
        assert!(lookup("").is_none());
    }

    #[test]
    fn emergency_entry_never_requires_notice() {
        // Even in the strictest state (WA, 48h), emergency entry passes
        // with zero notice.
        for code in ["WA", "CA", "AR", "FL", "AL"] {
            let r = compute(&input(code, EntryPurpose::Emergency, 0));
            assert!(r.complies, "{code} emergency entry must comply with 0h notice");
            assert_eq!(r.shortfall_hours, 0);
            assert!(r.exception.is_some());
        }
    }

    #[test]
    fn tenant_requested_entry_never_requires_notice() {
        // Consent is presumed when the tenant initiated the visit.
        let r = compute(&input("WA", EntryPurpose::TenantRequested, 0));
        assert!(r.complies);
        assert_eq!(r.exception, Some("tenant-requested entry: consent presumed"));
    }

    #[test]
    fn abandoned_unit_never_requires_notice() {
        let r = compute(&input("CA", EntryPurpose::Abandoned, 0));
        assert!(r.complies);
        assert_eq!(r.exception, Some("tenant has abandoned the unit"));
    }

    #[test]
    fn california_24h_repairs_compliant_exact() {
        // CA: 24h is presumed reasonable under Civ. Code § 1954. Exactly
        // meeting the minimum is compliant.
        let r = compute(&input("CA", EntryPurpose::Repairs, 24));
        assert!(r.complies);
        assert_eq!(r.required_hours, Some(24));
        assert_eq!(r.shortfall_hours, 0);
    }

    #[test]
    fn california_23h_repairs_one_hour_short() {
        // One hour under the minimum fails. The shortfall reports the
        // exact gap so a landlord-side caller can know how much earlier
        // to send the notice.
        let r = compute(&input("CA", EntryPurpose::Repairs, 23));
        assert!(!r.complies);
        assert_eq!(r.required_hours, Some(24));
        assert_eq!(r.shortfall_hours, 1);
    }

    #[test]
    fn florida_12h_repairs_minimum() {
        // FL is the lowest of any state with an hour count — 12h per
        // § 83.53. 12h exactly is compliant; 11h is one short.
        let pass = compute(&input("FL", EntryPurpose::Repairs, 12));
        assert!(pass.complies);
        assert_eq!(pass.required_hours, Some(12));

        let fail = compute(&input("FL", EntryPurpose::Repairs, 11));
        assert!(!fail.complies);
        assert_eq!(fail.shortfall_hours, 1);
    }

    #[test]
    fn washington_48h_repairs_but_24h_showings() {
        // WA carves out showings as a separate 24h column even though
        // standard entry is 48h. This is the only state in the table
        // with a per-purpose split, so it's load-bearing for the
        // showing_hours fallback logic.
        let repairs = compute(&input("WA", EntryPurpose::Repairs, 24));
        assert!(!repairs.complies, "WA repairs need 48h, 24h is short");
        assert_eq!(repairs.required_hours, Some(48));
        assert_eq!(repairs.shortfall_hours, 24);

        let showing = compute(&input("WA", EntryPurpose::Showing, 24));
        assert!(showing.complies, "WA showings are OK at 24h");
        assert_eq!(showing.required_hours, Some(24));

        let inspection_short = compute(&input("WA", EntryPurpose::Inspection, 24));
        assert!(!inspection_short.complies, "inspections fall under 48h column");
    }

    #[test]
    fn urlta_states_default_to_48h() {
        // URLTA-1974 default was "two days" (48h). The states that adopted
        // the 1974 wording verbatim — AL, AZ, KY, HI, RI, VT — should all
        // report 48h. Pinning this catches a future incorrect downgrade.
        for code in ["AL", "AZ", "KY", "HI", "RI", "VT"] {
            let r = compute(&input(code, EntryPurpose::Repairs, 24));
            assert!(!r.complies, "{code} should require 48h, 24h is short");
            assert_eq!(r.required_hours, Some(48), "{code} should report 48h");
        }
    }

    #[test]
    fn no_statute_states_report_compliant_at_zero_hours() {
        // For states without an hour count, the compute fn treats every
        // notice level (including 0) as compliant — there's no measurable
        // standard to fail against. The note flags the absence.
        for code in ["AR", "GA", "ID", "IL", "IN", "LA", "MA", "MI", "MO", "MS", "NC", "PA", "TX", "WV", "WY"] {
            let r = compute(&input(code, EntryPurpose::Repairs, 0));
            assert!(r.complies, "{code} has no statute, must report compliant");
            assert!(r.required_hours.is_none());
            assert!(r.note.contains("no statutory hour minimum"));
        }
    }

    #[test]
    fn reasonable_only_states_have_none_required() {
        // CT and MN have entry statutes that require "reasonable" notice
        // without specifying hours. These should behave like no-statute
        // states for the compute fn.
        for code in ["CT", "MN", "KS"] {
            let r = compute(&input(code, EntryPurpose::Repairs, 0));
            assert!(r.complies);
            assert!(r.required_hours.is_none());
        }
    }

    #[test]
    fn unknown_state_code_reports_error() {
        let r = compute(&input("XX", EntryPurpose::Repairs, 24));
        assert!(!r.complies);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn all_states_returns_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        // Verify sort: first state by code should be AK (Alaska), last WY.
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        // Every row, even no-statute ones, must have a citation field
        // populated (either the statute cite or the literal "no statute").
        // This guards against blank-citation regressions when adding rows.
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} has empty citation", r.state_code);
        }
    }

    #[test]
    fn showing_falls_back_to_standard_when_no_carveout() {
        // CA has no separate showing column, so showings use the 24h
        // standard. The fallback path is what makes WA the only special case.
        let r = compute(&input("CA", EntryPurpose::Showing, 24));
        assert!(r.complies);
        assert_eq!(r.required_hours, Some(24));
    }

    #[test]
    fn inspection_uses_standard_column() {
        // Inspections are not carved out separately in any state — they
        // always fall under the standard repairs column.
        let r = compute(&input("OH", EntryPurpose::Inspection, 24));
        assert!(r.complies);
        assert_eq!(r.required_hours, Some(24));
    }

    #[test]
    fn zero_notice_fails_in_hour_count_states() {
        // Any state with an hour count rejects 0h notice for repairs.
        for r in TABLE.values() {
            if r.standard_hours.is_some() {
                let result = compute(&input(r.state_code, EntryPurpose::Repairs, 0));
                assert!(
                    !result.complies,
                    "{} requires {:?}h but 0h was accepted",
                    r.state_code, r.standard_hours
                );
                assert!(result.shortfall_hours > 0);
            }
        }
    }

    #[test]
    fn excess_notice_does_not_change_compliance() {
        // Giving more notice than required is still compliant. The
        // shortfall is 0 (it doesn't go negative). This guards against
        // any future underflow if shortfall ever switches to signed.
        let r = compute(&input("CA", EntryPurpose::Repairs, 96));
        assert!(r.complies);
        assert_eq!(r.shortfall_hours, 0);
    }

    #[test]
    fn florida_12h_is_distinct_from_24h_default() {
        // FL is one of two states with a 12h hour count (FL by statute,
        // WI by admin code). The compute fn must not silently round these
        // to 24 — that would be a false-positive compliance result.
        let r = compute(&input("FL", EntryPurpose::Repairs, 18));
        assert!(r.complies, "FL accepts 18h since min is 12h");
        let r2 = compute(&input("WI", EntryPurpose::Repairs, 18));
        assert!(r2.complies, "WI accepts 18h since min is 12h");
        // But neither would be compliant under the URLTA-default 48h.
        let r3 = compute(&input("AL", EntryPurpose::Repairs, 18));
        assert!(!r3.complies);
    }

    #[test]
    fn emergency_short_circuits_even_for_unknown_state() {
        // An emergency entry should never fail. But the state code is
        // still validated — an unknown state still errors. This is the
        // intentional ordering: lookup first, then exception check.
        let r = compute(&input("ZZ", EntryPurpose::Emergency, 0));
        assert!(!r.complies, "unknown state errors even on emergency");
        assert!(r.note.contains("unknown state"));
    }
}
