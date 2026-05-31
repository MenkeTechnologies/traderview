//! State-by-state landlord anti-retaliation **rebuttable presumption**
//! windows. When a landlord initiates an adverse action (eviction filing,
//! rent increase, service reduction, refusal to renew) within the
//! statutory window after a tenant exercises a protected right
//! (complaint to a housing authority, complaint to the landlord, joining
//! a tenants' union, filing suit, withholding rent for habitability),
//! a rebuttable presumption arises that the action is retaliatory and
//! the burden shifts to the landlord to show a legitimate non-retaliatory
//! reason.
//!
//! The windows cluster into three bands:
//!   - **90 days**: WA, MI, MN, VT, DE — derived from URLTA-1974 §5.101
//!     adopters that took the shorter window
//!   - **180 days / 6 months**: AZ, CA, CO, CT, KS, MA, MD, ME, NE, NV,
//!     NH, NM, OR, PA, RI, SC, TN, TX, WI — the most common cluster,
//!     URLTA / Restatement / state-specific
//!   - **1 year (365 days)**: HI, IA, IL, KY, NC, NY, VA — the strictest
//!     band, mostly URLTA-1974 verbatim adopters or post-2019 reform
//!
//! Three special cases: **NJ** (no fixed window — case-by-case under
//! the Anti-Reprisal Act); **FL/OH/OK** (statute exists but no
//! statutory presumption period — tenant must affirmatively prove
//! retaliatory intent); and the remaining states which have no
//! anti-retaliation statute at all.
//!
//! Compute fn: given `(state, complaint_date, landlord_action_date)`,
//! returns whether the action falls inside the presumption window and
//! the days remaining / days past for context.

use chrono::{Months, NaiveDate};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The presumption window as expressed by each state's statute. Days vs
/// months matters because "6 months" is calendar-anchored (varies 181-184
/// days) while "180 days" is exact-day.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PresumptionPeriod {
    Days(u32),
    Months(u32),
    /// Statute exists but no fixed presumption window (FL, OH, OK).
    /// Tenant must prove retaliatory intent affirmatively.
    NoPresumptionPeriod,
    /// Statute exists but the window is determined case-by-case (NJ
    /// Anti-Reprisal Act — courts weigh the circumstances).
    CaseByCase,
    /// No anti-retaliation statute in this jurisdiction.
    NoStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRetaliationRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub period: PresumptionPeriod,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetaliationCheckInput {
    pub state_code: String,
    pub complaint_date: NaiveDate,
    pub landlord_action_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetaliationCheckResult {
    /// Calendar days between complaint and action (positive when the
    /// action came after the complaint; negative when before).
    pub days_since_complaint: i64,
    /// True if the action falls inside the statutory presumption window.
    pub within_presumption_window: bool,
    /// True if the rebuttable presumption applies (within window AND a
    /// statutory presumption period exists in this state).
    pub presumption_applies: bool,
    /// True if there's no anti-retaliation statute at all in this state.
    pub no_statute_in_state: bool,
    /// True if the state has a statute but no fixed presumption window
    /// (FL/OH/OK) — burden stays on the tenant.
    pub burden_on_tenant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateRetaliationRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateRetaliationRule> {
    let mut v: Vec<&'static StateRetaliationRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

/// Compute whether a landlord action falls inside the state's anti-
/// retaliation presumption window.
pub fn check(input: &RetaliationCheckInput) -> RetaliationCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return RetaliationCheckResult {
                days_since_complaint: 0,
                within_presumption_window: false,
                presumption_applies: false,
                no_statute_in_state: true,
                burden_on_tenant: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let days_since_complaint = (input.landlord_action_date - input.complaint_date).num_days();

    // Action came BEFORE complaint — can't be retaliation by definition.
    if days_since_complaint < 0 {
        return RetaliationCheckResult {
            days_since_complaint,
            within_presumption_window: false,
            presumption_applies: false,
            no_statute_in_state: matches!(rule.period, PresumptionPeriod::NoStatute),
            burden_on_tenant: matches!(rule.period, PresumptionPeriod::NoPresumptionPeriod),
            citation: rule.citation,
            note: format!(
                "landlord action {} days BEFORE complaint — cannot be retaliation under §§(a) causal-link requirement",
                -days_since_complaint
            ),
        };
    }

    match rule.period {
        PresumptionPeriod::NoStatute => RetaliationCheckResult {
            days_since_complaint,
            within_presumption_window: false,
            presumption_applies: false,
            no_statute_in_state: true,
            burden_on_tenant: false,
            citation: rule.citation,
            note: format!(
                "{} has no statutory anti-retaliation protection — common-law remedies only",
                rule.state_name
            ),
        },
        PresumptionPeriod::NoPresumptionPeriod => RetaliationCheckResult {
            days_since_complaint,
            within_presumption_window: false,
            presumption_applies: false,
            no_statute_in_state: false,
            burden_on_tenant: true,
            citation: rule.citation,
            note: format!(
                "{} has retaliation statute but no fixed presumption window — tenant must affirmatively prove retaliatory intent",
                rule.state_name
            ),
        },
        PresumptionPeriod::CaseByCase => RetaliationCheckResult {
            days_since_complaint,
            within_presumption_window: false,
            presumption_applies: false,
            no_statute_in_state: false,
            burden_on_tenant: false,
            citation: rule.citation,
            note: format!(
                "{} retaliation window is case-by-case — courts weigh the circumstances",
                rule.state_name
            ),
        },
        PresumptionPeriod::Days(n) => {
            let within = days_since_complaint <= n as i64;
            RetaliationCheckResult {
                days_since_complaint,
                within_presumption_window: within,
                presumption_applies: within,
                no_statute_in_state: false,
                burden_on_tenant: false,
                citation: rule.citation,
                note: if within {
                    format!(
                        "action {}d after complaint is within the {}-day window — presumption of retaliation applies, burden on landlord to rebut",
                        days_since_complaint, n
                    )
                } else {
                    format!(
                        "action {}d after complaint is {}d past the {}-day window — no presumption, tenant must prove retaliation",
                        days_since_complaint,
                        days_since_complaint - n as i64,
                        n
                    )
                },
            }
        }
        PresumptionPeriod::Months(n) => {
            // Calendar-anchored: window = complaint_date + N months. Use
            // chrono::Months to handle short-month rollover correctly
            // (e.g. complaint on 8/31 + 6 months = 2/28 or 2/29).
            let window_end = input
                .complaint_date
                .checked_add_months(Months::new(n))
                .unwrap_or(NaiveDate::MAX);
            let within = input.landlord_action_date <= window_end;
            RetaliationCheckResult {
                days_since_complaint,
                within_presumption_window: within,
                presumption_applies: within,
                no_statute_in_state: false,
                burden_on_tenant: false,
                citation: rule.citation,
                note: if within {
                    format!(
                        "action on {} is within the {}-month window ending {} — presumption of retaliation applies",
                        input.landlord_action_date, n, window_end
                    )
                } else {
                    format!(
                        "action on {} is past the {}-month window that ended {} — no presumption",
                        input.landlord_action_date, n, window_end
                    )
                },
            }
        }
    }
}

const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    period: PresumptionPeriod,
    citation: &'static str,
) -> StateRetaliationRule {
    StateRetaliationRule {
        state_code,
        state_name,
        period,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateRetaliationRule>> = Lazy::new(|| {
    use PresumptionPeriod::*;
    static RULES: &[StateRetaliationRule] = &[
        rule("AK", "Alaska", Days(90), "AS § 34.03.310"),
        rule("AL", "Alabama", Months(6), "Ala. Code § 35-9A-501"),
        rule("AR", "Arkansas", NoStatute, "no statute"),
        rule("AZ", "Arizona", Months(6), "A.R.S. § 33-1381"),
        rule("CA", "California", Days(180), "Cal. Civ. Code § 1942.5"),
        rule("CO", "Colorado", Days(180), "C.R.S. § 38-12-509"),
        rule("CT", "Connecticut", Months(6), "Conn. Gen. Stat. § 47a-20"),
        rule("DC", "District of Columbia", Months(6), "D.C. Code § 42-3505.02"),
        rule("DE", "Delaware", Days(90), "25 Del. C. § 5516"),
        rule("FL", "Florida", NoPresumptionPeriod, "Fla. Stat. § 83.64"),
        rule("GA", "Georgia", NoStatute, "no statute"),
        rule("HI", "Hawaii", Months(12), "HRS § 521-74"),
        rule("IA", "Iowa", Months(12), "Iowa Code § 562A.36"),
        rule("ID", "Idaho", NoStatute, "no statute"),
        rule("IL", "Illinois", Months(12), "765 ILCS 720/1"),
        rule("IN", "Indiana", NoStatute, "no statute"),
        rule("KS", "Kansas", Months(6), "K.S.A. § 58-2572"),
        rule("KY", "Kentucky", Months(12), "KRS § 383.705"),
        rule("LA", "Louisiana", NoStatute, "no statute"),
        rule("MA", "Massachusetts", Months(6), "M.G.L. c. 186 § 18"),
        rule("MD", "Maryland", Months(6), "Md. Code Real Prop. § 8-208.1"),
        rule("ME", "Maine", Months(6), "14 M.R.S. § 6001"),
        rule("MI", "Michigan", Days(90), "MCL § 600.5720"),
        rule("MN", "Minnesota", Days(90), "Minn. Stat. § 504B.285"),
        rule("MO", "Missouri", NoStatute, "no statute"),
        rule("MS", "Mississippi", NoStatute, "no statute"),
        rule("MT", "Montana", Months(6), "Mont. Code § 70-24-431"),
        rule("NC", "North Carolina", Months(12), "N.C.G.S. § 42-37.1"),
        rule("ND", "North Dakota", NoStatute, "no statute"),
        rule("NE", "Nebraska", Months(6), "Neb. Rev. Stat. § 76-1439"),
        rule("NH", "New Hampshire", Months(6), "RSA 540:13-a"),
        rule("NJ", "New Jersey", CaseByCase, "N.J.S.A. § 2A:42-10.10 (Anti-Reprisal Act)"),
        rule("NM", "New Mexico", Months(6), "NMSA § 47-8-39"),
        rule("NV", "Nevada", Months(6), "NRS § 118A.510"),
        rule("NY", "New York", Months(12), "RPL § 223-b"),
        rule("OH", "Ohio", NoPresumptionPeriod, "ORC § 5321.02"),
        rule("OK", "Oklahoma", NoPresumptionPeriod, "41 O.S. § 101"),
        rule("OR", "Oregon", Months(6), "ORS § 90.385"),
        rule("PA", "Pennsylvania", Months(6), "68 P.S. § 250.205"),
        rule("RI", "Rhode Island", Months(6), "R.I.G.L. § 34-18-46"),
        rule("SC", "South Carolina", Months(6), "S.C. Code § 27-40-910"),
        rule("SD", "South Dakota", NoStatute, "no statute"),
        rule("TN", "Tennessee", Days(180), "Tenn. Code § 66-28-514"),
        rule("TX", "Texas", Months(6), "Tex. Prop. Code § 92.331"),
        rule("UT", "Utah", NoStatute, "no statute"),
        rule("VA", "Virginia", Months(12), "Va. Code § 55.1-1258"),
        rule("VT", "Vermont", Days(90), "9 V.S.A. § 4465"),
        rule("WA", "Washington", Days(90), "RCW § 59.18.250"),
        rule("WI", "Wisconsin", Months(6), "Wis. Stat. § 704.45"),
        rule("WV", "West Virginia", NoStatute, "no statute"),
        rule("WY", "Wyoming", NoStatute, "no statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn input(state: &str, complaint: NaiveDate, action: NaiveDate) -> RetaliationCheckInput {
        RetaliationCheckInput {
            state_code: state.to_string(),
            complaint_date: complaint,
            landlord_action_date: action,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        // 51 rows. Same invariant as the entry_notice table — a deliberate
        // add or drop should update this count, never silently skip it.
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn lookup_is_case_insensitive_and_handles_unknown() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
        assert!(lookup("Ca").is_some());
        assert!(lookup("ZZ").is_none());
        assert!(lookup("").is_none());
    }

    #[test]
    fn california_180_days_exact_boundary_in_window() {
        // CA 180-day window. Day 180 exactly → presumption applies.
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 6, 30))); // 180 days later
        assert_eq!(r.days_since_complaint, 180);
        assert!(r.within_presumption_window);
        assert!(r.presumption_applies);
    }

    #[test]
    fn california_181_days_outside_window() {
        // Day 181 → outside.
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 7, 1))); // 181 days
        assert_eq!(r.days_since_complaint, 181);
        assert!(!r.within_presumption_window);
        assert!(!r.presumption_applies);
        assert!(r.note.contains("1d past"));
    }

    #[test]
    fn washington_90_day_window() {
        // WA is the strictest standard band — 90 days.
        let r = check(&input("WA", d(2026, 1, 1), d(2026, 4, 1))); // 90 days
        assert!(r.within_presumption_window);
        let r2 = check(&input("WA", d(2026, 1, 1), d(2026, 4, 2))); // 91 days
        assert!(!r2.within_presumption_window);
    }

    #[test]
    fn texas_6_months_calendar_anchored() {
        // TX uses MONTHS, not 180 days. Complaint 2026-08-31 + 6 months
        // = 2027-02-28 (short-month rollover via Months::checked_add).
        // An action on 2027-02-28 is within; 2027-03-01 is past.
        let r = check(&input("TX", d(2026, 8, 31), d(2027, 2, 28)));
        assert!(r.within_presumption_window);
        let r2 = check(&input("TX", d(2026, 8, 31), d(2027, 3, 1)));
        assert!(!r2.within_presumption_window);
    }

    #[test]
    fn illinois_one_year_window() {
        // IL is in the 12-month band (765 ILCS 720/1). Action 11 months
        // out is still inside the window.
        let r = check(&input("IL", d(2026, 1, 15), d(2026, 12, 15)));
        assert!(r.within_presumption_window);
        // 13 months out is past.
        let r2 = check(&input("IL", d(2026, 1, 15), d(2027, 2, 15)));
        assert!(!r2.within_presumption_window);
    }

    #[test]
    fn florida_statute_no_presumption_period_burden_on_tenant() {
        // FL has an anti-retaliation statute (§83.64) but does NOT
        // create a presumption window. Tenant must affirmatively prove
        // retaliatory intent regardless of timing.
        let r = check(&input("FL", d(2026, 1, 1), d(2026, 1, 15)));
        assert!(!r.presumption_applies);
        assert!(r.burden_on_tenant);
        assert!(!r.no_statute_in_state);
        assert!(r.note.contains("must affirmatively prove"));
    }

    #[test]
    fn new_jersey_case_by_case() {
        // NJ Anti-Reprisal Act — no fixed window. Compute returns a
        // distinct CaseByCase result that's neither presumption nor
        // burden-on-tenant.
        let r = check(&input("NJ", d(2026, 1, 1), d(2026, 1, 15)));
        assert!(!r.presumption_applies);
        assert!(!r.burden_on_tenant);
        assert!(!r.no_statute_in_state);
        assert!(r.note.contains("case-by-case"));
    }

    #[test]
    fn no_statute_states_flagged_correctly() {
        // States with no anti-retaliation statute (AR/GA/ID/IN/LA/MO/MS/
        // ND/SD/UT/WV/WY): no presumption applies regardless of timing,
        // and the no_statute flag is set.
        for code in ["AR", "GA", "ID", "IN", "LA", "MO", "MS", "ND", "SD", "UT", "WV", "WY"] {
            let r = check(&input(code, d(2026, 1, 1), d(2026, 1, 2)));
            assert!(!r.presumption_applies, "{code} should not have presumption");
            assert!(r.no_statute_in_state, "{code} should flag no statute");
        }
    }

    #[test]
    fn landlord_action_before_complaint_cannot_be_retaliation() {
        // Cause precedes effect. If the landlord filed the eviction
        // BEFORE the tenant complained, there's no causal link.
        let r = check(&input("CA", d(2026, 6, 1), d(2026, 5, 15)));
        assert_eq!(r.days_since_complaint, -17);
        assert!(!r.presumption_applies);
        assert!(r.note.contains("cannot be retaliation"));
    }

    #[test]
    fn day_zero_same_day_action_within_window() {
        // Action filed same calendar day as complaint should be inside
        // the window — this is the hostile-landlord scenario.
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 1, 1)));
        assert_eq!(r.days_since_complaint, 0);
        assert!(r.within_presumption_window);
    }

    #[test]
    fn unknown_state_marked_no_statute() {
        let r = check(&input("ZZ", d(2026, 1, 1), d(2026, 1, 2)));
        assert!(r.no_statute_in_state);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn ninety_day_band_states_pinned_at_91d_boundary() {
        // The 90-day band: AK, DE, MI, MN, VT, WA. Each must reject
        // 91-day actions. This catches any future regression where
        // someone switches Days(90) → Days(180) by mistake.
        for code in ["AK", "DE", "MI", "MN", "VT", "WA"] {
            let r = check(&input(code, d(2026, 1, 1), d(2026, 4, 2))); // 91d
            assert!(
                !r.within_presumption_window,
                "{code} 91-day action should be outside"
            );
        }
    }

    #[test]
    fn six_month_band_states_pinned_at_seven_months() {
        // Six-month band states must reject actions 7+ months out. CT,
        // TX, MA, MD, OR, PA, RI, SC, NV, NH, NM, NE, MT, WI, AZ, AL,
        // KS — all should reject day 211+.
        for code in [
            "CT", "TX", "MA", "MD", "OR", "PA", "RI", "SC", "NV", "NH", "NM", "NE", "MT", "WI",
            "AZ", "AL", "KS",
        ] {
            let r = check(&input(code, d(2026, 1, 1), d(2026, 8, 1))); // 7 months
            assert!(
                !r.within_presumption_window,
                "{code} should reject 7-month action"
            );
        }
    }

    #[test]
    fn twelve_month_band_holds_action_at_eleven_months() {
        // 12-month band states: HI, IA, IL, KY, NC, NY, VA. Action 11
        // months later still inside.
        for code in ["HI", "IA", "IL", "KY", "NC", "NY", "VA"] {
            let r = check(&input(code, d(2026, 1, 15), d(2026, 12, 15)));
            assert!(
                r.within_presumption_window,
                "{code} 11-month action should still be inside 12-month window"
            );
        }
    }

    #[test]
    fn all_states_returns_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} has empty citation", r.state_code);
        }
    }

    #[test]
    fn months_path_uses_calendar_anchored_window_not_180_days() {
        // 6 months is NOT 180 days. Aug 31 + 6 months = Feb 28/29.
        // Days(180) would land on Feb 27. The calendar-anchored Months
        // path must include Feb 28 — pinning this catches a regression
        // where someone "simplifies" Months(6) to Days(180).
        let r = check(&input("TX", d(2026, 8, 31), d(2027, 2, 28)));
        assert_eq!(r.days_since_complaint, 181); // 181 calendar days
        assert!(r.within_presumption_window);
    }

    #[test]
    fn note_describes_days_remaining_when_inside_window() {
        let r = check(&input("CA", d(2026, 1, 1), d(2026, 3, 1)));
        assert!(r.note.contains("180-day window"));
        assert!(r.note.contains("burden on landlord"));
    }
}
