//! State-by-state domestic violence (DV) early lease termination rules.
//!
//! Universally available federal floor is **VAWA** (34 U.S.C. § 12491)
//! for federally-assisted housing — HUD-financed, Section 8 vouchers,
//! LIHTC, public housing, etc. — which prohibits eviction or termination
//! based on actual or threatened DV/sexual assault/stalking. State laws
//! extend the protection to the private market and provide affirmative
//! early-termination rights with notice + documentation requirements.
//!
//! Notice periods cluster into four bands:
//!
//! - **3 days**: IL Safe Homes Act (765 ILCS 750/15) — strictest
//!   pro-tenant in the country
//! - **14 days**: CA § 1946.7 / DC § 42-3505.07 / HI § 521-80 /
//!   OR § 90.453 / WA § 59.18.575 (effectively month-end)
//! - **30 days**: TX § 92.0161 / MD § 8-5A-01 / NY § 227-c (most common)
//! - **End of current month**: WA § 59.18.575 — tenant pays through the
//!   month of notice and walks
//!
//! Documentation requirements are nearly uniform across states with
//! statutes: at least ONE of a protective order, police report, or
//! qualified-third-party statement (medical provider, mental health
//! professional, victim service provider, clergy). Most states require
//! the documentation to be **fresh** — within 180 days (CA), 90 days
//! (WA, others), or contemporaneous.
//!
//! **Three special triggers** can override the notice requirement
//! entirely:
//!   - Violence committed by a co-tenant (TX § 92.0161(b))
//!   - Violence committed by the landlord or their agent (WA § 59.18.575)
//!   - Imminent threat scenarios (case-by-case)
//!
//! All three trigger **immediate termination availability** which the
//! `immediate_termination_available` flag on the result surfaces.

use chrono::{Datelike, Days, NaiveDate};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NoticePeriod {
    /// Tenant gives `n` calendar days written notice.
    Days(u32),
    /// Tenant pays through the end of the calendar month of notice, then
    /// quits. (WA § 59.18.575 model.)
    EndOfCurrentMonth,
    /// Statute exists but no fixed notice period — typically because
    /// the state requires "reasonable" notice that depends on context.
    NoFixedPeriod,
    /// No state DV early-termination statute. VAWA floor still applies
    /// to federally-assisted housing.
    NoStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDvRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub notice: NoticePeriod,
    /// Maximum age of accepted documentation (e.g., police report ≤ 180
    /// days old). `None` = no freshness requirement, `Some(0)` = contem-
    /// poraneous only.
    pub documentation_freshness_days: Option<u32>,
    /// True if state allows immediate termination when violence was
    /// committed by a co-tenant in the same unit.
    pub immediate_for_co_tenant_violence: bool,
    /// True if state allows immediate termination when violence was
    /// committed by the landlord or landlord's agent.
    pub immediate_for_landlord_violence: bool,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DvEarlyTerminationInput {
    pub state_code: String,
    pub notice_date: NaiveDate,
    pub planned_termination_date: NaiveDate,
    pub incident_date: NaiveDate,
    pub has_protective_order: bool,
    pub has_police_report: bool,
    pub has_qualified_third_party_attestation: bool,
    pub violence_by_co_tenant: bool,
    pub violence_by_landlord_or_agent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DvEarlyTerminationResult {
    pub complies_with_notice: bool,
    /// Statutory minimum notice in days. `None` for EndOfCurrentMonth
    /// (calendar-anchored) and NoFixedPeriod/NoStatute rows.
    pub required_notice_days: Option<u32>,
    /// Calendar days from notice_date to planned_termination_date.
    pub actual_notice_days: i64,
    pub shortfall_days: i64,
    pub documentation_sufficient: bool,
    /// Documentation freshness check: days from incident_date to
    /// notice_date must be within the state's freshness window.
    pub documentation_within_freshness_window: bool,
    /// True if a state-specific immediate-termination trigger fired
    /// (co-tenant or landlord violence).
    pub immediate_termination_available: bool,
    pub no_statute_in_state: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateDvRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateDvRule> {
    let mut v: Vec<&'static StateDvRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

/// Last day of the month that contains `date`.
fn end_of_month(date: NaiveDate) -> NaiveDate {
    // Move to the first of next month and subtract a day. chrono's
    // Months::checked_add handles month-length variance and year rollover.
    let next_month_first = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
    };
    next_month_first - Days::new(1)
}

pub fn check(input: &DvEarlyTerminationInput) -> DvEarlyTerminationResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return DvEarlyTerminationResult {
                complies_with_notice: false,
                required_notice_days: None,
                actual_notice_days: 0,
                shortfall_days: 0,
                documentation_sufficient: false,
                documentation_within_freshness_window: false,
                immediate_termination_available: false,
                no_statute_in_state: true,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    // Documentation sufficient if ANY of the three forms is present.
    let documentation_sufficient = input.has_protective_order
        || input.has_police_report
        || input.has_qualified_third_party_attestation;

    // Documentation freshness: incident_date must be within the state's
    // freshness window from notice_date.
    let documentation_within_freshness_window = match rule.documentation_freshness_days {
        Some(window) => {
            let days_since = (input.notice_date - input.incident_date).num_days();
            days_since >= 0 && days_since <= window as i64
        }
        None => true, // no freshness requirement
    };

    let actual_notice_days = (input.planned_termination_date - input.notice_date).num_days();

    // Immediate termination triggers — override notice requirement.
    let immediate_available = (rule.immediate_for_co_tenant_violence
        && input.violence_by_co_tenant)
        || (rule.immediate_for_landlord_violence && input.violence_by_landlord_or_agent);

    if matches!(rule.notice, NoticePeriod::NoStatute) {
        return DvEarlyTerminationResult {
            complies_with_notice: false,
            required_notice_days: None,
            actual_notice_days,
            shortfall_days: 0,
            documentation_sufficient,
            documentation_within_freshness_window,
            immediate_termination_available: immediate_available,
            no_statute_in_state: true,
            citation: rule.citation,
            note: format!(
                "{} has no DV early-termination statute; VAWA floor applies only to federally-assisted housing",
                rule.state_name
            ),
        };
    }

    let (required_days, complies) = match rule.notice {
        NoticePeriod::Days(n) => {
            let n_i64 = n as i64;
            (Some(n), immediate_available || actual_notice_days >= n_i64)
        }
        NoticePeriod::EndOfCurrentMonth => {
            // Planned termination must be ≥ end of the calendar month
            // containing the notice date.
            let month_end = end_of_month(input.notice_date);
            let complies = immediate_available || input.planned_termination_date >= month_end;
            (None, complies)
        }
        NoticePeriod::NoFixedPeriod => (None, true),
        NoticePeriod::NoStatute => (None, false),
    };

    let shortfall = match (rule.notice, complies) {
        (NoticePeriod::Days(n), false) => (n as i64 - actual_notice_days).max(0),
        _ => 0,
    };

    let note = if immediate_available {
        let reason = if input.violence_by_co_tenant {
            "co-tenant violence"
        } else {
            "landlord/agent violence"
        };
        format!(
            "{} — {} triggers immediate termination; notice period waived",
            rule.state_name, reason
        )
    } else {
        match rule.notice {
            NoticePeriod::Days(n) => {
                if complies {
                    format!(
                        "{} requires {}-day notice; tenant gave {}d — complies",
                        rule.state_name, n, actual_notice_days
                    )
                } else {
                    format!(
                        "{} requires {}-day notice; tenant gave {}d — {}d short",
                        rule.state_name, n, actual_notice_days, shortfall
                    )
                }
            }
            NoticePeriod::EndOfCurrentMonth => {
                let month_end = end_of_month(input.notice_date);
                if complies {
                    format!(
                        "{} requires termination through end of current month ({}); tenant's planned termination {} — complies",
                        rule.state_name, month_end, input.planned_termination_date
                    )
                } else {
                    format!(
                        "{} requires termination through end of current month ({}); tenant's planned termination {} is before month end — doesn't comply",
                        rule.state_name, month_end, input.planned_termination_date
                    )
                }
            }
            NoticePeriod::NoFixedPeriod => {
                format!(
                    "{}: statute exists with no fixed notice period — \"reasonable\" notice required",
                    rule.state_name
                )
            }
            NoticePeriod::NoStatute => format!("{} no statute", rule.state_name),
        }
    };

    DvEarlyTerminationResult {
        complies_with_notice: complies,
        required_notice_days: required_days,
        actual_notice_days,
        shortfall_days: shortfall,
        documentation_sufficient,
        documentation_within_freshness_window,
        immediate_termination_available: immediate_available,
        no_statute_in_state: false,
        citation: rule.citation,
        note,
    }
}

const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    notice: NoticePeriod,
    documentation_freshness_days: Option<u32>,
    immediate_for_co_tenant_violence: bool,
    immediate_for_landlord_violence: bool,
    citation: &'static str,
) -> StateDvRule {
    StateDvRule {
        state_code,
        state_name,
        notice,
        documentation_freshness_days,
        immediate_for_co_tenant_violence,
        immediate_for_landlord_violence,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateDvRule>> = Lazy::new(|| {
    use NoticePeriod::*;
    static RULES: &[StateDvRule] = &[
        rule("AK", "Alaska", NoStatute, None, false, false, "VAWA only"),
        rule("AL", "Alabama", NoStatute, None, false, false, "VAWA only"),
        rule("AR", "Arkansas", NoStatute, None, false, false, "VAWA only"),
        rule(
            "AZ",
            "Arizona",
            Days(30),
            Some(30),
            true,
            true,
            "A.R.S. § 33-1318",
        ),
        rule(
            "CA",
            "California",
            Days(14),
            Some(180),
            false,
            false,
            "Cal. Civ. Code § 1946.7",
        ),
        rule(
            "CO",
            "Colorado",
            NoFixedPeriod,
            None,
            false,
            false,
            "C.R.S. § 38-12-402",
        ),
        rule(
            "CT",
            "Connecticut",
            Days(30),
            Some(30),
            false,
            false,
            "Conn. Gen. Stat. § 47a-11e",
        ),
        rule(
            "DC",
            "District of Columbia",
            Days(14),
            Some(60),
            false,
            true,
            "D.C. Code § 42-3505.07",
        ),
        rule(
            "DE",
            "Delaware",
            Days(30),
            Some(180),
            false,
            false,
            "25 Del. C. § 5314",
        ),
        rule("FL", "Florida", NoStatute, None, false, false, "VAWA only"),
        rule("GA", "Georgia", NoStatute, None, false, false, "VAWA only"),
        rule(
            "HI",
            "Hawaii",
            Days(14),
            Some(90),
            true,
            true,
            "HRS § 521-80",
        ),
        rule(
            "IA",
            "Iowa",
            Days(30),
            Some(30),
            false,
            false,
            "Iowa Code § 562A.27A",
        ),
        rule("ID", "Idaho", NoStatute, None, false, false, "VAWA only"),
        rule(
            "IL",
            "Illinois",
            Days(3),
            Some(60),
            true,
            true,
            "Safe Homes Act (765 ILCS 750/15)",
        ),
        rule(
            "IN",
            "Indiana",
            Days(30),
            Some(30),
            false,
            false,
            "Ind. Code § 32-31-9-12",
        ),
        rule("KS", "Kansas", NoStatute, None, false, false, "VAWA only"),
        rule("KY", "Kentucky", NoStatute, None, false, false, "VAWA only"),
        rule(
            "LA",
            "Louisiana",
            Days(30),
            None,
            false,
            false,
            "La. R.S. § 9:3261.1",
        ),
        rule(
            "MA",
            "Massachusetts",
            Days(0), // "quickly leave"; effectively immediate under c. 186 § 24
            Some(90),
            true,
            true,
            "M.G.L. c. 186 § 24",
        ),
        rule(
            "MD",
            "Maryland",
            Days(30),
            None,
            false,
            false,
            "Md. Code Real Prop. § 8-5A-02",
        ),
        rule(
            "ME",
            "Maine",
            Days(7),
            None,
            false,
            false,
            "14 M.R.S. § 6002",
        ),
        rule(
            "MI",
            "Michigan",
            Days(30),
            None,
            false,
            false,
            "MCL § 554.601b",
        ),
        rule(
            "MN",
            "Minnesota",
            Days(0),
            None,
            true,
            true,
            "Minn. Stat. § 504B.206 (immediate with documentation)",
        ),
        rule("MO", "Missouri", NoStatute, None, false, false, "VAWA only"),
        rule(
            "MS",
            "Mississippi",
            NoStatute,
            None,
            false,
            false,
            "VAWA only",
        ),
        rule(
            "MT",
            "Montana",
            Days(30),
            None,
            false,
            false,
            "Mont. Code § 70-24-321",
        ),
        rule(
            "NC",
            "North Carolina",
            Days(30),
            Some(90),
            false,
            false,
            "N.C.G.S. § 42-45.1",
        ),
        rule(
            "ND",
            "North Dakota",
            Days(30),
            None,
            false,
            false,
            "N.D.C.C. § 47-16-17.1",
        ),
        rule(
            "NE",
            "Nebraska",
            Days(30),
            Some(30),
            false,
            false,
            "Neb. Rev. Stat. § 76-1431.01",
        ),
        rule(
            "NH",
            "New Hampshire",
            Days(30),
            None,
            false,
            false,
            "RSA § 540:2-a",
        ),
        rule(
            "NJ",
            "New Jersey",
            Days(30),
            Some(60),
            false,
            false,
            "N.J.S.A. § 46:8-9.6",
        ),
        rule(
            "NM",
            "New Mexico",
            Days(7),
            None,
            false,
            false,
            "NMSA § 47-8-33",
        ),
        rule(
            "NV",
            "Nevada",
            Days(30),
            Some(60),
            false,
            false,
            "NRS § 118A.345",
        ),
        rule(
            "NY",
            "New York",
            Days(30),
            None,
            false,
            false,
            "RPL § 227-c",
        ),
        rule("OH", "Ohio", NoStatute, None, false, false, "VAWA only"),
        rule("OK", "Oklahoma", NoStatute, None, false, false, "VAWA only"),
        rule(
            "OR",
            "Oregon",
            Days(14),
            Some(90),
            true,
            true,
            "ORS § 90.453",
        ),
        rule(
            "PA",
            "Pennsylvania",
            Days(30),
            None,
            false,
            false,
            "68 Pa. C.S. § 250.510-A",
        ),
        rule(
            "RI",
            "Rhode Island",
            Days(30),
            Some(180),
            false,
            false,
            "R.I.G.L. § 34-37-1",
        ),
        rule(
            "SC",
            "South Carolina",
            NoStatute,
            None,
            false,
            false,
            "VAWA only",
        ),
        rule(
            "SD",
            "South Dakota",
            Days(30),
            None,
            false,
            false,
            "SDCL § 43-32-19",
        ),
        rule(
            "TN",
            "Tennessee",
            Days(14),
            None,
            false,
            false,
            "Tenn. Code § 66-28-512",
        ),
        rule(
            "TX",
            "Texas",
            Days(30),
            None,
            true,
            false,
            "Tex. Prop. Code § 92.0161",
        ),
        rule(
            "UT",
            "Utah",
            Days(30),
            None,
            false,
            false,
            "Utah Code § 57-22-5.1",
        ),
        rule(
            "VA",
            "Virginia",
            Days(30),
            None,
            false,
            false,
            "Va. Code § 55.1-1236",
        ),
        rule(
            "VT",
            "Vermont",
            Days(14),
            Some(30),
            false,
            false,
            "9 V.S.A. § 4474b",
        ),
        rule(
            "WA",
            "Washington",
            EndOfCurrentMonth,
            Some(90),
            false,
            true,
            "RCW § 59.18.575",
        ),
        rule(
            "WI",
            "Wisconsin",
            Days(28),
            Some(60),
            false,
            false,
            "Wis. Stat. § 704.16",
        ),
        rule(
            "WV",
            "West Virginia",
            NoStatute,
            None,
            false,
            false,
            "VAWA only",
        ),
        rule("WY", "Wyoming", NoStatute, None, false, false, "VAWA only"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn input(
        state: &str,
        notice: NaiveDate,
        planned: NaiveDate,
        incident: NaiveDate,
    ) -> DvEarlyTerminationInput {
        DvEarlyTerminationInput {
            state_code: state.to_string(),
            notice_date: notice,
            planned_termination_date: planned,
            incident_date: incident,
            has_protective_order: false,
            has_police_report: true,
            has_qualified_third_party_attestation: false,
            violence_by_co_tenant: false,
            violence_by_landlord_or_agent: false,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn california_14_day_notice_exact_boundary_complies() {
        // CA § 1946.7: 14 days exactly between notice and termination.
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 1, 15), d(2025, 12, 1)));
        assert_eq!(r.actual_notice_days, 14);
        assert!(r.complies_with_notice);
        assert_eq!(r.required_notice_days, Some(14));
    }

    #[test]
    fn california_13_day_notice_one_short() {
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 1, 14), d(2025, 12, 1)));
        assert_eq!(r.actual_notice_days, 13);
        assert!(!r.complies_with_notice);
        assert_eq!(r.shortfall_days, 1);
    }

    #[test]
    fn texas_30_day_notice_required() {
        let pass = check(&input("TX", d(2026, 1, 1), d(2026, 1, 31), d(2025, 12, 15)));
        assert!(pass.complies_with_notice);
        let fail = check(&input("TX", d(2026, 1, 1), d(2026, 1, 30), d(2025, 12, 15)));
        assert!(!fail.complies_with_notice);
        assert_eq!(fail.shortfall_days, 1);
    }

    #[test]
    fn illinois_three_day_notice_strictest_pro_tenant() {
        // IL Safe Homes Act — 3 days is the strictest pro-tenant rule.
        let r = check(&input("IL", d(2026, 1, 1), d(2026, 1, 4), d(2025, 12, 30)));
        assert_eq!(r.required_notice_days, Some(3));
        assert!(r.complies_with_notice);
    }

    #[test]
    fn washington_end_of_month_calendar_anchored() {
        // WA § 59.18.575: tenant pays through end of month of notice.
        // Notice 2026-01-15 → must terminate ≥ 2026-01-31.
        let pass = check(&input("WA", d(2026, 1, 15), d(2026, 1, 31), d(2026, 1, 10)));
        assert!(pass.complies_with_notice);
        let fail = check(&input("WA", d(2026, 1, 15), d(2026, 1, 30), d(2026, 1, 10)));
        assert!(!fail.complies_with_notice);
        // Required-days is None because it's calendar-anchored.
        assert!(pass.required_notice_days.is_none());
    }

    #[test]
    fn washington_short_month_february_handled() {
        // Notice on Feb 5 in non-leap year → end of month = Feb 28.
        let r = check(&input("WA", d(2026, 2, 5), d(2026, 2, 28), d(2026, 2, 1)));
        assert!(r.complies_with_notice);
        // Feb 27 should fail.
        let r2 = check(&input("WA", d(2026, 2, 5), d(2026, 2, 27), d(2026, 2, 1)));
        assert!(!r2.complies_with_notice);
    }

    #[test]
    fn washington_december_notice_wraps_to_dec_31() {
        // Year-end boundary: Dec 15 notice → end of month = Dec 31.
        // Pinned because the end_of_month helper has year-rollover
        // arithmetic that could regress.
        let r = check(&input(
            "WA",
            d(2026, 12, 15),
            d(2026, 12, 31),
            d(2026, 12, 10),
        ));
        assert!(r.complies_with_notice);
    }

    #[test]
    fn texas_co_tenant_violence_waives_notice() {
        // TX § 92.0161(b) — violence by co-tenant allows immediate
        // termination. Notice gave only 5 days but compliance passes.
        let mut i = input("TX", d(2026, 1, 1), d(2026, 1, 5), d(2025, 12, 30));
        i.violence_by_co_tenant = true;
        let r = check(&i);
        assert!(r.complies_with_notice);
        assert!(r.immediate_termination_available);
        assert!(r.note.contains("co-tenant violence"));
    }

    #[test]
    fn washington_landlord_violence_waives_notice() {
        // WA § 59.18.575 — violence by landlord/agent allows immediate
        // termination. Tenant must deliver documentation within 7 days
        // of leaving (compliance-side detail; compute focuses on notice).
        let mut i = input("WA", d(2026, 1, 15), d(2026, 1, 16), d(2026, 1, 14));
        i.violence_by_landlord_or_agent = true;
        let r = check(&i);
        assert!(r.complies_with_notice);
        assert!(r.immediate_termination_available);
    }

    #[test]
    fn co_tenant_violence_does_not_trigger_immediate_in_states_without_carveout() {
        // CA doesn't have the co-tenant immediate-termination carve-out
        // (Civ. Code § 1946.7 still requires 14-day notice). Setting
        // the flag has no effect — notice/termination span of 7 days
        // is still 7 days short of the 14-day requirement.
        let mut i = input("CA", d(2026, 1, 1), d(2026, 1, 8), d(2025, 12, 1));
        i.violence_by_co_tenant = true;
        let r = check(&i);
        assert!(!r.immediate_termination_available);
        assert!(!r.complies_with_notice);
        assert_eq!(r.actual_notice_days, 7);
        assert_eq!(r.shortfall_days, 7); // 14 - 7
    }

    #[test]
    fn documentation_sufficient_with_only_protective_order() {
        // Any ONE of the three forms is sufficient. Protective order
        // alone qualifies.
        let mut i = input("CA", d(2026, 1, 1), d(2026, 1, 15), d(2025, 12, 1));
        i.has_protective_order = true;
        i.has_police_report = false;
        i.has_qualified_third_party_attestation = false;
        let r = check(&i);
        assert!(r.documentation_sufficient);
    }

    #[test]
    fn documentation_insufficient_when_all_three_false() {
        let mut i = input("CA", d(2026, 1, 1), d(2026, 1, 15), d(2025, 12, 1));
        i.has_protective_order = false;
        i.has_police_report = false;
        i.has_qualified_third_party_attestation = false;
        let r = check(&i);
        assert!(!r.documentation_sufficient);
    }

    #[test]
    fn california_180_day_freshness_window() {
        // CA accepts documentation up to 180 days old. Incident 179
        // days before notice → within window.
        let r = check(&input(
            "CA",
            d(2026, 6, 28),
            d(2026, 7, 12),
            d(2025, 12, 31),
        ));
        let days = (d(2026, 6, 28) - d(2025, 12, 31)).num_days();
        assert_eq!(days, 179);
        assert!(r.documentation_within_freshness_window);

        // Incident 181 days before → outside window.
        let r2 = check(&input(
            "CA",
            d(2026, 6, 30),
            d(2026, 7, 14),
            d(2025, 12, 31),
        ));
        assert!(!r2.documentation_within_freshness_window);
    }

    #[test]
    fn no_freshness_window_means_always_in_window() {
        // States without a freshness requirement (TX, MD, MI, etc.):
        // any-age documentation is within window.
        let r = check(&input("TX", d(2026, 6, 30), d(2026, 7, 30), d(2020, 1, 1)));
        assert!(r.documentation_within_freshness_window);
    }

    #[test]
    fn no_statute_states_flagged_correctly() {
        for code in [
            "AL", "AR", "FL", "GA", "ID", "KS", "KY", "MO", "MS", "OH", "OK", "SC", "WV", "WY",
            "AK",
        ] {
            let r = check(&input(code, d(2026, 1, 1), d(2026, 1, 31), d(2025, 12, 15)));
            assert!(r.no_statute_in_state, "{code} should be flagged no statute");
            assert!(!r.complies_with_notice);
            assert!(r.note.contains("VAWA"));
        }
    }

    #[test]
    fn unknown_state_marked_no_statute() {
        let r = check(&input("ZZ", d(2026, 1, 1), d(2026, 1, 15), d(2025, 12, 1)));
        assert!(r.no_statute_in_state);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
        assert!(lookup("Ca").is_some());
    }

    #[test]
    fn all_states_returns_sorted() {
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
    fn notice_before_incident_documentation_window_negative() {
        // Pathological input: notice date is BEFORE incident date.
        // Documentation can't be fresh because the incident hasn't
        // happened yet. days_since negative → outside window.
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 1, 15), d(2026, 6, 1)));
        assert!(!r.documentation_within_freshness_window);
    }

    #[test]
    fn end_of_month_boundary_january_31() {
        // Notice on Jan 31 → end of month is same day. Termination
        // must be ≥ Jan 31.
        let r = check(&input("WA", d(2026, 1, 31), d(2026, 1, 31), d(2026, 1, 25)));
        assert!(r.complies_with_notice);
    }

    #[test]
    fn shortfall_zero_when_compliant() {
        // Compliant cases have shortfall_days = 0 regardless of regime.
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 2, 1), d(2025, 12, 1)));
        assert!(r.complies_with_notice);
        assert_eq!(r.shortfall_days, 0);
    }

    #[test]
    fn shortfall_only_for_days_regime_not_end_of_month() {
        // EndOfCurrentMonth (WA) non-compliance reports shortfall 0
        // because the regime isn't a day-count comparison. The note
        // text carries the explanation instead.
        let r = check(&input("WA", d(2026, 1, 15), d(2026, 1, 20), d(2026, 1, 10)));
        assert!(!r.complies_with_notice);
        assert_eq!(r.shortfall_days, 0);
    }
}
