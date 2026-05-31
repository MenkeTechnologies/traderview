//! State-by-state mold disclosure + remediation compliance.
//!
//! Recent regulatory wave (2001-2025) after California's Toxic Mold
//! Protection Act of 2001 (Civ. Code § 1941.7). Far fewer states have
//! comprehensive mold statutes than bedbug or lead — most rely on the
//! implied warranty of habitability. The states with explicit statutes
//! tend to focus on one of four areas:
//!
//! 1. **Pre-lease known-mold disclosure** — CA Civ. Code § 1941.7
//!    requires written disclosure of any known mold conditions before
//!    the tenant signs the lease.
//! 2. **Move-in inspection report mold notation** — VA Code § 55.1-1215
//!    requires the move-in report to disclose visible mold in areas
//!    readily accessible within the unit (within 5 days of occupancy).
//! 3. **Comprehensive annual inspection + remediation standards** —
//!    NYC Local Law 55 of 2018 (Asthma-Free Housing Act) requires
//!    annual inspections, licensed remediation over 10 sq ft, and
//!    informational materials.
//! 4. **Remediation standards only** — MD 2025+ regulations require
//!    landlord adherence to mold prevention + abatement standards
//!    without a pre-lease disclosure requirement.
//!
//! **No statewide statute** — most of the country. The NJ Mold Safe
//! Housing Act has been introduced multiple times since 2013 but as
//! of the current cutoff remains pending (most recent bill 2026-2027
//! session has not been enacted into law). NJ landlords still owe
//! habitability remediation; just no specific disclosure regime.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MoldRegime {
    PreLeaseKnownMoldDisclosure,
    MoveInReportMoldNotation,
    ComprehensiveAnnualInspection,
    RemediationStandardsOnly,
    HabitabilityCovenantOnly,
    NoStateStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMoldRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: MoldRegime,
    pub pre_lease_disclosure_required: bool,
    pub move_in_report_mold_notation_required: bool,
    pub annual_inspection_required: bool,
    /// Maximum days landlord has to complete remediation after tenant
    /// report (where state law specifies). None = no specific timeline.
    pub remediation_days: Option<u32>,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoldCheckInput {
    pub state_code: String,
    pub pre_lease_disclosure_made: bool,
    pub move_in_report_included_mold_notation: bool,
    pub annual_inspection_completed: bool,
    pub tenant_reported_mold: bool,
    pub days_since_tenant_report: u32,
    pub remediation_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoldCheckResult {
    pub disclosure_required: bool,
    pub complies: bool,
    pub violations: Vec<String>,
    pub no_statute_in_state: bool,
    pub habitability_only: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateMoldRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateMoldRule> {
    let mut v: Vec<&'static StateMoldRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &MoldCheckInput) -> MoldCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return MoldCheckResult {
                disclosure_required: false,
                complies: false,
                violations: vec!["unknown state code".to_string()],
                no_statute_in_state: true,
                habitability_only: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let no_statute = matches!(rule.regime, MoldRegime::NoStateStatute);
    let habitability_only = matches!(rule.regime, MoldRegime::HabitabilityCovenantOnly);

    if no_statute || habitability_only {
        return MoldCheckResult {
            disclosure_required: false,
            complies: true,
            violations: vec![],
            no_statute_in_state: no_statute,
            habitability_only,
            citation: rule.citation,
            note: if no_statute {
                format!(
                    "{}: no statewide mold-disclosure statute — implied habitability + local ordinances apply",
                    rule.state_name
                )
            } else {
                format!(
                    "{}: habitability covenant requires remediation but no specific mold-disclosure statute",
                    rule.state_name
                )
            },
        };
    }

    let mut violations: Vec<String> = Vec::new();

    if rule.pre_lease_disclosure_required && !input.pre_lease_disclosure_made {
        violations.push(format!(
            "{} requires pre-lease disclosure of known mold conditions; not disclosed",
            rule.state_name
        ));
    }

    if rule.move_in_report_mold_notation_required && !input.move_in_report_included_mold_notation
    {
        violations.push(format!(
            "{} requires move-in report to include visible-mold notation; not included",
            rule.state_name
        ));
    }

    if rule.annual_inspection_required && !input.annual_inspection_completed {
        violations.push(format!(
            "{} requires annual mold inspection; not completed",
            rule.state_name
        ));
    }

    if let Some(window_days) = rule.remediation_days {
        if input.tenant_reported_mold
            && !input.remediation_completed
            && input.days_since_tenant_report > window_days
        {
            violations.push(format!(
                "{} requires remediation within {}d of tenant report; {}d have passed without remediation",
                rule.state_name, window_days, input.days_since_tenant_report
            ));
        }
    }

    let complies = violations.is_empty();
    let note = if complies {
        format!("{}: mold-disclosure / remediation requirements satisfied", rule.state_name)
    } else {
        format!(
            "{}: {} mold-compliance violation(s)",
            rule.state_name,
            violations.len()
        )
    };

    MoldCheckResult {
        disclosure_required: true,
        complies,
        violations,
        no_statute_in_state: false,
        habitability_only: false,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: MoldRegime,
    pre_lease_disclosure_required: bool,
    move_in_report_mold_notation_required: bool,
    annual_inspection_required: bool,
    remediation_days: Option<u32>,
    citation: &'static str,
) -> StateMoldRule {
    StateMoldRule {
        state_code,
        state_name,
        regime,
        pre_lease_disclosure_required,
        move_in_report_mold_notation_required,
        annual_inspection_required,
        remediation_days,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateMoldRule>> = Lazy::new(|| {
    use MoldRegime::*;
    static RULES: &[StateMoldRule] = &[
        rule("AK", "Alaska", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("AL", "Alabama", HabitabilityCovenantOnly, false, false, false, None, "Ala. Code § 35-9A-204"),
        rule("AR", "Arkansas", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("AZ", "Arizona", HabitabilityCovenantOnly, false, false, false, None, "A.R.S. § 33-1324"),
        rule(
            "CA",
            "California",
            PreLeaseKnownMoldDisclosure,
            true,
            false,
            false,
            None,
            "Cal. Civ. Code § 1941.7 + Toxic Mold Protection Act (Health & Safety Code § 26100)",
        ),
        rule("CO", "Colorado", HabitabilityCovenantOnly, false, false, false, None, "C.R.S. § 38-12-505"),
        rule("CT", "Connecticut", HabitabilityCovenantOnly, false, false, false, None, "Conn. Gen. Stat. § 47a-7"),
        rule("DC", "District of Columbia", HabitabilityCovenantOnly, false, false, false, None, "14 DCMR § 700"),
        rule("DE", "Delaware", HabitabilityCovenantOnly, false, false, false, None, "25 Del. C. § 5305 (general habitability)"),
        rule("FL", "Florida", HabitabilityCovenantOnly, false, false, false, None, "Fla. Stat. § 83.51"),
        rule("GA", "Georgia", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("HI", "Hawaii", HabitabilityCovenantOnly, false, false, false, None, "HRS § 521-42"),
        rule("IA", "Iowa", HabitabilityCovenantOnly, false, false, false, None, "Iowa Code § 562A.15"),
        rule("ID", "Idaho", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("IL", "Illinois", HabitabilityCovenantOnly, false, false, false, None, "Chicago RLTO § 5-12-130"),
        rule(
            "IN",
            "Indiana",
            HabitabilityCovenantOnly,
            false,
            false,
            false,
            None,
            "Ind. Code § 32-31-8-5 (habitability + investigate reports)",
        ),
        rule("KS", "Kansas", HabitabilityCovenantOnly, false, false, false, None, "K.S.A. § 58-2553"),
        rule("KY", "Kentucky", HabitabilityCovenantOnly, false, false, false, None, "KRS § 383.595"),
        rule("LA", "Louisiana", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("MA", "Massachusetts", HabitabilityCovenantOnly, false, false, false, None, "M.G.L. c. 111 § 127A"),
        rule(
            "MD",
            "Maryland",
            RemediationStandardsOnly,
            false,
            false,
            false,
            Some(30),
            "Md. Code Real Prop. § 8-208.2 (Healthy Homes Act 2008, expanded 2025)",
        ),
        rule("ME", "Maine", HabitabilityCovenantOnly, false, false, false, None, "14 M.R.S. § 6021"),
        rule("MI", "Michigan", HabitabilityCovenantOnly, false, false, false, None, "MCL § 554.139"),
        rule("MN", "Minnesota", HabitabilityCovenantOnly, false, false, false, None, "Minn. Stat. § 504B.181"),
        rule("MO", "Missouri", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("MS", "Mississippi", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("MT", "Montana", HabitabilityCovenantOnly, false, false, false, None, "Mont. Code § 70-24-303"),
        rule("NC", "North Carolina", HabitabilityCovenantOnly, false, false, false, None, "N.C.G.S. § 42-42"),
        rule("ND", "North Dakota", HabitabilityCovenantOnly, false, false, false, None, "N.D.C.C. § 47-16-13.1"),
        rule("NE", "Nebraska", HabitabilityCovenantOnly, false, false, false, None, "Neb. Rev. Stat. § 76-1419"),
        rule("NH", "New Hampshire", HabitabilityCovenantOnly, false, false, false, None, "RSA § 540-A"),
        rule(
            "NJ",
            "New Jersey",
            HabitabilityCovenantOnly,
            false,
            false,
            false,
            None,
            "Mold Safe Housing Act pending since 2013; habitability covenant applies",
        ),
        rule("NM", "New Mexico", HabitabilityCovenantOnly, false, false, false, None, "NMSA § 47-8-20"),
        rule("NV", "Nevada", HabitabilityCovenantOnly, false, false, false, None, "NRS § 118A.290"),
        rule(
            "NY",
            "New York",
            ComprehensiveAnnualInspection,
            false,
            false,
            true,
            None,
            "NYC Local Law 55 of 2018 (Asthma-Free Housing Act) — NYC only; state has no statute",
        ),
        rule("OH", "Ohio", HabitabilityCovenantOnly, false, false, false, None, "ORC § 5321.04"),
        rule("OK", "Oklahoma", HabitabilityCovenantOnly, false, false, false, None, "41 O.S. § 118"),
        rule("OR", "Oregon", HabitabilityCovenantOnly, false, false, false, None, "ORS § 90.320"),
        rule("PA", "Pennsylvania", HabitabilityCovenantOnly, false, false, false, None, "Pugh v. Holmes (1979)"),
        rule("RI", "Rhode Island", HabitabilityCovenantOnly, false, false, false, None, "R.I.G.L. § 34-18-22"),
        rule("SC", "South Carolina", HabitabilityCovenantOnly, false, false, false, None, "S.C. Code § 27-40-440"),
        rule("SD", "South Dakota", NoStateStatute, false, false, false, None, "no statewide statute"),
        rule("TN", "Tennessee", HabitabilityCovenantOnly, false, false, false, None, "Tenn. Code § 66-28-304"),
        rule("TX", "Texas", HabitabilityCovenantOnly, false, false, false, None, "Tex. Prop. Code § 92.052"),
        rule("UT", "Utah", HabitabilityCovenantOnly, false, false, false, None, "Utah Code § 57-22-4"),
        rule(
            "VA",
            "Virginia",
            MoveInReportMoldNotation,
            false,
            true,
            false,
            None,
            "Va. Code § 55.1-1215 (move-in report visible mold)",
        ),
        rule("VT", "Vermont", HabitabilityCovenantOnly, false, false, false, None, "9 V.S.A. § 4457"),
        rule("WA", "Washington", HabitabilityCovenantOnly, false, false, false, None, "RCW § 59.18.060"),
        rule("WI", "Wisconsin", HabitabilityCovenantOnly, false, false, false, None, "Wis. Stat. § 704.07"),
        rule("WV", "West Virginia", HabitabilityCovenantOnly, false, false, false, None, "W. Va. Code § 37-6-30"),
        rule("WY", "Wyoming", NoStateStatute, false, false, false, None, "no statewide statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_compliant(state: &str) -> MoldCheckInput {
        MoldCheckInput {
            state_code: state.to_string(),
            pre_lease_disclosure_made: true,
            move_in_report_included_mold_notation: true,
            annual_inspection_completed: true,
            tenant_reported_mold: false,
            days_since_tenant_report: 0,
            remediation_completed: false,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn ca_pre_lease_disclosure_violation() {
        // CA Civ. Code § 1941.7 — pre-lease known-mold disclosure required.
        let mut i = fully_compliant("CA");
        i.pre_lease_disclosure_made = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("pre-lease disclosure")));
    }

    #[test]
    fn va_move_in_report_mold_notation_required() {
        // VA Code § 55.1-1215 — move-in report visible mold.
        let mut i = fully_compliant("VA");
        i.move_in_report_included_mold_notation = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("move-in report")));
    }

    #[test]
    fn ny_annual_inspection_required_nyc_ll55() {
        // NYC Local Law 55 of 2018 — annual inspection required.
        let mut i = fully_compliant("NY");
        i.annual_inspection_completed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("annual mold inspection")));
    }

    #[test]
    fn md_remediation_30_day_window_violation_at_31_days() {
        // MD § 8-208.2 Healthy Homes Act — 30 days remediation.
        let mut i = fully_compliant("MD");
        i.tenant_reported_mold = true;
        i.days_since_tenant_report = 31;
        i.remediation_completed = false;
        let r = check(&i);
        assert!(!r.complies);
        assert!(r.violations.iter().any(|v| v.contains("remediation within 30d")));
    }

    #[test]
    fn md_30_day_window_complies_at_30_days() {
        // 30 days exact = within the window.
        let mut i = fully_compliant("MD");
        i.tenant_reported_mold = true;
        i.days_since_tenant_report = 30;
        i.remediation_completed = false;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn md_complies_when_remediation_done_past_deadline() {
        // 60 days post-report but remediation complete → complies.
        let mut i = fully_compliant("MD");
        i.tenant_reported_mold = true;
        i.days_since_tenant_report = 60;
        i.remediation_completed = true;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn md_no_violation_without_tenant_report() {
        // If no tenant report, remediation clock isn't running. Even at
        // 1000 days, no violation.
        let mut i = fully_compliant("MD");
        i.tenant_reported_mold = false;
        i.days_since_tenant_report = 1000;
        let r = check(&i);
        assert!(r.complies);
    }

    #[test]
    fn habitability_only_states_always_comply() {
        // States with HabitabilityCovenantOnly regime — implied
        // covenant applies but no specific mold compliance check.
        for code in ["AL", "AZ", "CO", "DE", "FL", "NJ", "TX", "WA", "PA", "OH"] {
            let mut i = fully_compliant(code);
            i.pre_lease_disclosure_made = false;
            let r = check(&i);
            assert!(r.complies, "{code} should be habitability-only");
            assert!(r.habitability_only);
            assert!(!r.no_statute_in_state);
        }
    }

    #[test]
    fn no_statute_states_always_comply() {
        for code in ["AK", "AR", "GA", "ID", "LA", "MO", "MS", "SD", "WY"] {
            let mut i = fully_compliant(code);
            i.pre_lease_disclosure_made = false;
            let r = check(&i);
            assert!(r.complies, "{code} should be no-statute");
            assert!(r.no_statute_in_state);
        }
    }

    #[test]
    fn nj_classified_habitability_only_pending_mold_safe_act() {
        // NJ Mold Safe Housing Act has been pending since 2013 but not
        // enacted. Currently habitability-only.
        let nj = lookup("NJ").unwrap();
        assert!(matches!(nj.regime, MoldRegime::HabitabilityCovenantOnly));
    }

    #[test]
    fn ca_only_pre_lease_state_pinned() {
        // CA is the only state with the PreLeaseKnownMoldDisclosure
        // regime in this table. Other states cluster into other regimes.
        let ca = lookup("CA").unwrap();
        assert!(matches!(ca.regime, MoldRegime::PreLeaseKnownMoldDisclosure));
        for r in TABLE.values() {
            if r.state_code != "CA" {
                assert!(
                    !matches!(r.regime, MoldRegime::PreLeaseKnownMoldDisclosure),
                    "{} should not be pre-lease disclosure regime",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn va_only_move_in_report_state_pinned() {
        let va = lookup("VA").unwrap();
        assert!(matches!(va.regime, MoldRegime::MoveInReportMoldNotation));
        for r in TABLE.values() {
            if r.state_code != "VA" {
                assert!(
                    !matches!(r.regime, MoldRegime::MoveInReportMoldNotation),
                    "{} should not be move-in report regime",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn ny_only_comprehensive_annual_state_pinned() {
        // NY (NYC LL55) is the only state with ComprehensiveAnnualInspection
        // regime.
        let ny = lookup("NY").unwrap();
        assert!(matches!(ny.regime, MoldRegime::ComprehensiveAnnualInspection));
    }

    #[test]
    fn md_only_remediation_standards_state_pinned() {
        let md = lookup("MD").unwrap();
        assert!(matches!(md.regime, MoldRegime::RemediationStandardsOnly));
        assert_eq!(md.remediation_days, Some(30));
    }

    #[test]
    fn ca_with_full_compliance_passes() {
        let r = check(&fully_compliant("CA"));
        assert!(r.complies);
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
    fn nyc_ll55_citation_correct() {
        let ny = lookup("NY").unwrap();
        assert!(ny.citation.contains("Local Law 55"));
        assert!(ny.citation.contains("Asthma-Free"));
    }

    #[test]
    fn ca_citation_includes_toxic_mold_protection_act() {
        let ca = lookup("CA").unwrap();
        assert!(ca.citation.contains("§ 1941.7"));
        assert!(ca.citation.contains("Toxic Mold"));
    }

    #[test]
    fn note_for_habitability_only_states_distinguishes_from_no_statute() {
        // Habitability-only and no-statute should produce DIFFERENT
        // notes so the downstream UI can distinguish them.
        let ha = check(&fully_compliant("AL"));
        let ns = check(&fully_compliant("WY"));
        assert!(ha.note.contains("habitability covenant"));
        assert!(ns.note.contains("no statewide"));
    }

    #[test]
    fn multiple_violations_stack_for_ca() {
        // CA only requires pre-lease disclosure — only one prong can
        // violate. Pinned for stability.
        let mut i = fully_compliant("CA");
        i.pre_lease_disclosure_made = false;
        // Other flags don't matter for CA regime.
        i.move_in_report_included_mold_notation = false;
        i.annual_inspection_completed = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 1);
    }
}
