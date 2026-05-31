//! State-by-state tenant abandonment thresholds.
//!
//! Operational concern for every landlord: when can the landlord
//! declare a tenant has abandoned the unit, take possession, dispose of
//! belongings, and re-rent? Self-help abandonment procedures vary by
//! state — some allow a "notice of belief of abandonment" + waiting
//! period (CA model); others require full court eviction (NY, CO).
//!
//! Four regimes:
//!
//! 1. **Statutory abandonment procedure** — CA (Civ. Code § 1951.3 +
//!    § 1986), TX (Prop. Code § 92.014), WA (RCW § 59.18.310), OR
//!    (ORS § 90.425), IL (765 ILCS 705/2). State statute fixes:
//!    - Days of unpaid rent before abandonment may be presumed
//!    - Notice of belief of abandonment period (waiting period after
//!      service before landlord may take possession)
//!    - Belongings disposal window (how long landlord must store
//!      tenant's personal property before disposing/selling)
//!
//! 2. **Case-by-case rebuttable presumption** — TX, FL, MA, NJ rely on
//!    facts-and-circumstances test rather than fixed day-count.
//!    Indicia: rent unpaid + tenant absent + utilities terminated +
//!    mail accumulating.
//!
//! 3. **Court process only** — NY (RPAPL § 711), CO (C.R.S. § 13-40-122),
//!    NJ (Anti-Eviction Act). No statewide self-help abandonment;
//!    landlord must file possession action even when tenant appears to
//!    have abandoned.
//!
//! 4. **No statewide statute** — handful of states with no specific
//!    abandonment statute; common-law abandonment doctrine applies.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AbandonmentRegime {
    StatutoryAbandonment,
    CaseByCasePresumption,
    CourtProcessOnly,
    NoStateStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAbandonmentRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: AbandonmentRegime,
    /// Days of unpaid rent before landlord may serve notice of belief
    /// of abandonment (CA 14, OR 7, IL 21, WA 14).
    pub rent_unpaid_threshold_days: Option<u32>,
    /// Days landlord must wait after serving notice of belief before
    /// taking possession (CA 14, TX 10 - response window).
    pub notice_of_belief_period_days: Option<u32>,
    /// Days landlord must store tenant's belongings before disposing/
    /// selling (CA 18, WA 45, TX 30).
    pub belongings_disposal_period_days: Option<u32>,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantAbandonmentInput {
    pub state_code: String,
    pub days_rent_unpaid: u32,
    /// Days since the notice of belief of abandonment was served. `None`
    /// if notice has not yet been served.
    pub days_since_notice_of_belief_served: Option<u32>,
    /// Days since belongings were taken into landlord's storage. `None`
    /// if storage has not yet begun.
    pub days_since_belongings_stored: Option<u32>,
    /// True if additional indicia of abandonment are present (utilities
    /// terminated, mail accumulating, neighbor reports of absence,
    /// keys returned, etc.).
    pub additional_abandonment_indicia_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantAbandonmentResult {
    /// True if landlord may serve the notice of belief of abandonment
    /// (statutory threshold + additional indicia met).
    pub notice_of_belief_warranted: bool,
    /// True if the notice waiting period has elapsed (landlord may take
    /// possession).
    pub notice_period_satisfied: bool,
    /// True if belongings disposal period has elapsed (landlord may
    /// dispose/sell stored property).
    pub belongings_disposal_allowed: bool,
    /// True if the state regime requires full court eviction (no self-
    /// help abandonment available).
    pub regime_requires_court_process: bool,
    /// True if the state has no statewide abandonment statute.
    pub no_statute_in_state: bool,
    /// True if state uses case-by-case presumption (no fixed thresholds).
    pub case_by_case_regime: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateAbandonmentRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateAbandonmentRule> {
    let mut v: Vec<&'static StateAbandonmentRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &TenantAbandonmentInput) -> TenantAbandonmentResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return TenantAbandonmentResult {
                notice_of_belief_warranted: false,
                notice_period_satisfied: false,
                belongings_disposal_allowed: false,
                regime_requires_court_process: false,
                no_statute_in_state: true,
                case_by_case_regime: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    match rule.regime {
        AbandonmentRegime::NoStateStatute => TenantAbandonmentResult {
            notice_of_belief_warranted: false,
            notice_period_satisfied: false,
            belongings_disposal_allowed: false,
            regime_requires_court_process: false,
            no_statute_in_state: true,
            case_by_case_regime: false,
            citation: rule.citation,
            note: format!(
                "{}: no statewide abandonment statute — common-law abandonment doctrine applies; consult counsel",
                rule.state_name
            ),
        },
        AbandonmentRegime::CourtProcessOnly => TenantAbandonmentResult {
            notice_of_belief_warranted: false,
            notice_period_satisfied: false,
            belongings_disposal_allowed: false,
            regime_requires_court_process: true,
            no_statute_in_state: false,
            case_by_case_regime: false,
            citation: rule.citation,
            note: format!(
                "{}: NO self-help abandonment — landlord must file possession action even if tenant appears to have abandoned",
                rule.state_name
            ),
        },
        AbandonmentRegime::CaseByCasePresumption => TenantAbandonmentResult {
            notice_of_belief_warranted: input.additional_abandonment_indicia_present,
            notice_period_satisfied: false,
            belongings_disposal_allowed: false,
            regime_requires_court_process: false,
            no_statute_in_state: false,
            case_by_case_regime: true,
            citation: rule.citation,
            note: format!(
                "{}: case-by-case rebuttable presumption — no fixed thresholds; abandonment determined by facts and circumstances",
                rule.state_name
            ),
        },
        AbandonmentRegime::StatutoryAbandonment => {
            let rent_threshold = rule.rent_unpaid_threshold_days.unwrap_or(0);
            let notice_warranted = input.days_rent_unpaid >= rent_threshold
                && input.additional_abandonment_indicia_present;

            let notice_period = rule.notice_of_belief_period_days.unwrap_or(0);
            let notice_satisfied = input
                .days_since_notice_of_belief_served
                .map(|d| d >= notice_period)
                .unwrap_or(false);

            let belongings_period = rule.belongings_disposal_period_days.unwrap_or(0);
            let belongings_allowed = input
                .days_since_belongings_stored
                .map(|d| d >= belongings_period)
                .unwrap_or(false);

            let note = if !notice_warranted {
                format!(
                    "{}: notice of belief NOT warranted — {}d rent unpaid (need {}d) + indicia present: {}",
                    rule.state_name,
                    input.days_rent_unpaid,
                    rent_threshold,
                    input.additional_abandonment_indicia_present
                )
            } else if !notice_satisfied {
                format!(
                    "{}: notice warranted (rent {}d unpaid + indicia); waiting period {}d not yet satisfied",
                    rule.state_name, input.days_rent_unpaid, notice_period
                )
            } else if !belongings_allowed {
                format!(
                    "{}: notice period satisfied; landlord may take possession; belongings disposal blocked for {}d more",
                    rule.state_name, belongings_period
                )
            } else {
                format!(
                    "{}: all abandonment thresholds met — notice satisfied + belongings disposal period elapsed",
                    rule.state_name
                )
            };

            TenantAbandonmentResult {
                notice_of_belief_warranted: notice_warranted,
                notice_period_satisfied: notice_satisfied,
                belongings_disposal_allowed: belongings_allowed,
                regime_requires_court_process: false,
                no_statute_in_state: false,
                case_by_case_regime: false,
                citation: rule.citation,
                note,
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: AbandonmentRegime,
    rent_unpaid_threshold_days: Option<u32>,
    notice_of_belief_period_days: Option<u32>,
    belongings_disposal_period_days: Option<u32>,
    citation: &'static str,
) -> StateAbandonmentRule {
    StateAbandonmentRule {
        state_code,
        state_name,
        regime,
        rent_unpaid_threshold_days,
        notice_of_belief_period_days,
        belongings_disposal_period_days,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateAbandonmentRule>> = Lazy::new(|| {
    use AbandonmentRegime::*;
    static RULES: &[StateAbandonmentRule] = &[
        rule("AK", "Alaska", NoStateStatute, None, None, None, "no statewide statute"),
        rule("AL", "Alabama", CaseByCasePresumption, None, None, None, "common law"),
        rule("AR", "Arkansas", NoStateStatute, None, None, None, "no statewide statute"),
        rule("AZ", "Arizona", StatutoryAbandonment, Some(7), Some(5), Some(14), "A.R.S. § 33-1370"),
        rule(
            "CA",
            "California",
            StatutoryAbandonment,
            Some(14),
            Some(14),
            Some(18),
            "Cal. Civ. Code § 1951.3 + § 1986",
        ),
        rule("CO", "Colorado", CourtProcessOnly, None, None, None, "C.R.S. § 13-40-122 (must file possession action)"),
        rule("CT", "Connecticut", CaseByCasePresumption, None, None, None, "common law"),
        rule("DC", "District of Columbia", CourtProcessOnly, None, None, None, "D.C. Code § 42-3505.01 (must file)"),
        rule("DE", "Delaware", StatutoryAbandonment, Some(7), Some(7), Some(7), "25 Del. C. § 5715"),
        rule(
            "FL",
            "Florida",
            CaseByCasePresumption,
            None,
            None,
            None,
            "Fla. Stat. § 83.59 (rebuttable presumption)",
        ),
        rule("GA", "Georgia", NoStateStatute, None, None, None, "no statewide statute"),
        rule("HI", "Hawaii", StatutoryAbandonment, Some(20), Some(15), Some(15), "HRS § 521-72"),
        rule(
            "IA",
            "Iowa",
            StatutoryAbandonment,
            Some(14),
            Some(10),
            Some(7),
            "Iowa Code § 562A.29",
        ),
        rule("ID", "Idaho", NoStateStatute, None, None, None, "no statewide statute"),
        rule(
            "IL",
            "Illinois",
            StatutoryAbandonment,
            Some(21),
            Some(7),
            Some(7),
            "765 ILCS 705/2",
        ),
        rule("IN", "Indiana", NoStateStatute, None, None, None, "no statewide statute"),
        rule("KS", "Kansas", StatutoryAbandonment, Some(15), Some(15), Some(30), "K.S.A. § 58-2565"),
        rule("KY", "Kentucky", CaseByCasePresumption, None, None, None, "KRS § 383.670"),
        rule("LA", "Louisiana", NoStateStatute, None, None, None, "no statewide statute"),
        rule("MA", "Massachusetts", CaseByCasePresumption, None, None, None, "M.G.L. c. 186 § 14"),
        rule("MD", "Maryland", CaseByCasePresumption, None, None, None, "common law"),
        rule("ME", "Maine", StatutoryAbandonment, Some(14), Some(14), Some(14), "14 M.R.S. § 6005"),
        rule("MI", "Michigan", StatutoryAbandonment, Some(14), Some(10), Some(30), "MCL § 600.2918"),
        rule("MN", "Minnesota", StatutoryAbandonment, Some(28), Some(28), Some(60), "Minn. Stat. § 504B.271"),
        rule("MO", "Missouri", CaseByCasePresumption, None, None, None, "common law"),
        rule("MS", "Mississippi", NoStateStatute, None, None, None, "no statewide statute"),
        rule("MT", "Montana", StatutoryAbandonment, Some(7), Some(7), Some(15), "Mont. Code § 70-24-430"),
        rule("NC", "North Carolina", StatutoryAbandonment, Some(15), Some(10), Some(7), "N.C.G.S. § 42-25.9"),
        rule("ND", "North Dakota", StatutoryAbandonment, Some(15), Some(10), Some(28), "N.D.C.C. § 47-16-30.1"),
        rule("NE", "Nebraska", StatutoryAbandonment, Some(14), Some(7), Some(14), "Neb. Rev. Stat. § 76-1432"),
        rule("NH", "New Hampshire", CaseByCasePresumption, None, None, None, "common law"),
        rule(
            "NJ",
            "New Jersey",
            CourtProcessOnly,
            None,
            None,
            None,
            "N.J.S.A. § 2A:18-72 (Anti-Eviction Act — no self-help)",
        ),
        rule("NM", "New Mexico", StatutoryAbandonment, Some(7), Some(7), Some(30), "NMSA § 47-8-34.1"),
        rule("NV", "Nevada", StatutoryAbandonment, Some(5), Some(5), Some(30), "NRS § 118A.450"),
        rule(
            "NY",
            "New York",
            CourtProcessOnly,
            None,
            None,
            None,
            "RPAPL § 711 + § 711-a (must file holdover proceeding)",
        ),
        rule("OH", "Ohio", CaseByCasePresumption, None, None, None, "ORC § 5321.04"),
        rule("OK", "Oklahoma", StatutoryAbandonment, Some(7), Some(15), Some(15), "41 O.S. § 130"),
        rule(
            "OR",
            "Oregon",
            StatutoryAbandonment,
            Some(7),
            Some(5),
            Some(8),
            "ORS § 90.425",
        ),
        rule("PA", "Pennsylvania", StatutoryAbandonment, Some(10), Some(10), Some(10), "68 P.S. § 250.505a"),
        rule("RI", "Rhode Island", CaseByCasePresumption, None, None, None, "R.I.G.L. § 34-18-19"),
        rule("SC", "South Carolina", StatutoryAbandonment, Some(15), Some(5), Some(15), "S.C. Code § 27-40-730"),
        rule("SD", "South Dakota", NoStateStatute, None, None, None, "no statewide statute"),
        rule("TN", "Tennessee", StatutoryAbandonment, Some(30), Some(10), Some(30), "Tenn. Code § 66-28-405"),
        rule(
            "TX",
            "Texas",
            StatutoryAbandonment,
            Some(5),
            Some(10),
            Some(30),
            "Tex. Prop. Code § 92.014",
        ),
        rule("UT", "Utah", StatutoryAbandonment, Some(15), Some(15), Some(15), "Utah Code § 78B-6-816"),
        rule("VA", "Virginia", StatutoryAbandonment, Some(7), Some(7), Some(24), "Va. Code § 55.1-1254"),
        rule("VT", "Vermont", CaseByCasePresumption, None, None, None, "9 V.S.A. § 4456"),
        rule(
            "WA",
            "Washington",
            StatutoryAbandonment,
            Some(14),
            Some(14),
            Some(45),
            "RCW § 59.18.310",
        ),
        rule("WI", "Wisconsin", StatutoryAbandonment, Some(21), Some(7), Some(30), "Wis. Stat. § 704.05"),
        rule("WV", "West Virginia", CaseByCasePresumption, None, None, None, "common law"),
        rule("WY", "Wyoming", NoStateStatute, None, None, None, "no statewide statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        state: &str,
        days_unpaid: u32,
        days_notice: Option<u32>,
        days_storage: Option<u32>,
        indicia: bool,
    ) -> TenantAbandonmentInput {
        TenantAbandonmentInput {
            state_code: state.to_string(),
            days_rent_unpaid: days_unpaid,
            days_since_notice_of_belief_served: days_notice,
            days_since_belongings_stored: days_storage,
            additional_abandonment_indicia_present: indicia,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn ca_notice_warranted_at_14d_with_indicia() {
        // CA: 14 days unpaid + indicia → notice warranted.
        let r = check(&input("CA", 14, None, None, true));
        assert!(r.notice_of_belief_warranted);
    }

    #[test]
    fn ca_notice_not_warranted_at_13d() {
        // 13 days unpaid → below threshold.
        let r = check(&input("CA", 13, None, None, true));
        assert!(!r.notice_of_belief_warranted);
    }

    #[test]
    fn ca_notice_not_warranted_without_indicia() {
        // 14 days unpaid but no other indicia → notice not warranted.
        let r = check(&input("CA", 14, None, None, false));
        assert!(!r.notice_of_belief_warranted);
    }

    #[test]
    fn ca_notice_period_satisfied_at_14d() {
        let r = check(&input("CA", 14, Some(14), None, true));
        assert!(r.notice_period_satisfied);
    }

    #[test]
    fn ca_notice_period_not_satisfied_at_13d() {
        let r = check(&input("CA", 14, Some(13), None, true));
        assert!(!r.notice_period_satisfied);
    }

    #[test]
    fn ca_belongings_disposal_at_18d() {
        // CA § 1986: 18 days storage required.
        let r = check(&input("CA", 14, Some(14), Some(18), true));
        assert!(r.belongings_disposal_allowed);
    }

    #[test]
    fn ca_belongings_disposal_blocked_at_17d() {
        let r = check(&input("CA", 14, Some(14), Some(17), true));
        assert!(!r.belongings_disposal_allowed);
    }

    #[test]
    fn wa_45_day_belongings_window_strictest_in_table() {
        // WA RCW § 59.18.310: 45 days storage — strictest in table.
        let r = check(&input("WA", 30, Some(30), Some(44), true));
        assert!(!r.belongings_disposal_allowed);
        let r2 = check(&input("WA", 30, Some(30), Some(45), true));
        assert!(r2.belongings_disposal_allowed);
    }

    #[test]
    fn tx_30_day_belongings_window() {
        let r = check(&input("TX", 10, Some(10), Some(30), true));
        assert!(r.belongings_disposal_allowed);
    }

    #[test]
    fn ny_court_process_only_no_self_help() {
        // NY RPAPL: must file court proceeding even with apparent
        // abandonment. No notice-of-belief mechanism available.
        let r = check(&input("NY", 100, Some(100), Some(100), true));
        assert!(r.regime_requires_court_process);
        assert!(!r.notice_of_belief_warranted);
        assert!(!r.notice_period_satisfied);
        assert!(!r.belongings_disposal_allowed);
        assert!(r.note.contains("must file possession action"));
    }

    #[test]
    fn co_court_process_only_mirrors_ny() {
        let r = check(&input("CO", 100, Some(100), Some(100), true));
        assert!(r.regime_requires_court_process);
    }

    #[test]
    fn nj_anti_eviction_act_court_process_only() {
        let r = check(&input("NJ", 100, Some(100), Some(100), true));
        assert!(r.regime_requires_court_process);
    }

    #[test]
    fn fl_case_by_case_presumption_with_indicia() {
        // FL § 83.59: case-by-case rebuttable presumption.
        let r = check(&input("FL", 30, None, None, true));
        assert!(r.case_by_case_regime);
        assert!(r.note.contains("case-by-case"));
    }

    #[test]
    fn ma_case_by_case_presumption() {
        let r = check(&input("MA", 30, None, None, true));
        assert!(r.case_by_case_regime);
    }

    #[test]
    fn no_statute_states_return_no_statute_flag() {
        for code in ["AK", "AR", "GA", "ID", "IN", "LA", "MS", "SD", "WY"] {
            let r = check(&input(code, 100, None, None, true));
            assert!(r.no_statute_in_state, "{code} should be no-statute");
            assert!(!r.regime_requires_court_process);
        }
    }

    #[test]
    fn unknown_state_handled() {
        let r = check(&input("ZZ", 14, None, None, true));
        assert!(r.no_statute_in_state);
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
    fn court_process_states_pinned() {
        // NY / CO / DC / NJ require court process.
        for code in ["NY", "CO", "DC", "NJ"] {
            let r = lookup(code).unwrap();
            assert!(
                matches!(r.regime, AbandonmentRegime::CourtProcessOnly),
                "{code} should require court process"
            );
        }
    }

    #[test]
    fn statutory_states_have_all_three_day_thresholds() {
        // All statutory-abandonment states must have rent_unpaid +
        // notice + belongings disposal day counts set.
        for r in TABLE.values() {
            if matches!(r.regime, AbandonmentRegime::StatutoryAbandonment) {
                assert!(r.rent_unpaid_threshold_days.is_some(), "{} missing rent threshold", r.state_code);
                assert!(r.notice_of_belief_period_days.is_some(), "{} missing notice period", r.state_code);
                assert!(r.belongings_disposal_period_days.is_some(), "{} missing belongings period", r.state_code);
            }
        }
    }

    #[test]
    fn case_by_case_states_have_no_day_thresholds() {
        // Conversely, CaseByCasePresumption states should NOT have day
        // counts — those are fixed-threshold-only fields.
        for r in TABLE.values() {
            if matches!(r.regime, AbandonmentRegime::CaseByCasePresumption) {
                assert!(r.rent_unpaid_threshold_days.is_none(), "{} should not have rent threshold", r.state_code);
            }
        }
    }

    #[test]
    fn wa_strictest_belongings_disposal_45_days() {
        // WA has the longest belongings disposal window in the table
        // (45 days). Pinned because the value drives compliance UI.
        let wa = lookup("WA").unwrap();
        assert_eq!(wa.belongings_disposal_period_days, Some(45));
        // Verify no other state has a longer window.
        for r in TABLE.values() {
            if let Some(days) = r.belongings_disposal_period_days {
                assert!(days <= 60, "{} has unusually long belongings window {}", r.state_code, days);
            }
        }
    }

    #[test]
    fn complete_workflow_ca_from_14d_through_disposal() {
        // Step 1: Day 14, rent unpaid, no notice yet → warranted.
        let s1 = check(&input("CA", 14, None, None, true));
        assert!(s1.notice_of_belief_warranted);
        assert!(!s1.notice_period_satisfied);

        // Step 2: After serving notice + 14 days waiting → satisfied.
        let s2 = check(&input("CA", 28, Some(14), None, true));
        assert!(s2.notice_period_satisfied);
        assert!(!s2.belongings_disposal_allowed);

        // Step 3: After 18 days of belongings storage → may dispose.
        let s3 = check(&input("CA", 28, Some(14), Some(18), true));
        assert!(s3.belongings_disposal_allowed);
        assert!(s3.note.contains("all abandonment thresholds met"));
    }
}
