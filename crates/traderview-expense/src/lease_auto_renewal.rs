//! State lease automatic renewal / evergreen clause disclosure
//! compliance.
//!
//! When a residential lease contains an auto-renewal ("evergreen")
//! clause that requires either party to give specific notice to
//! prevent renewal, several states statutorily require the LANDLORD
//! to send a written reminder notice before the tenant's
//! non-renewal-notice deadline. Without that reminder, the
//! auto-renewal clause is UNENFORCEABLE — the lease simply expires
//! at the end of the original term.
//!
//! Three regimes:
//!
//! - `PreNonrenewalNotificationFifteenToThirtyDays` — FL (Fla. Stat.
//!   § 83.575), WI (Wis. Stat. § 704.15), NY (GBL § 5-905). Landlord
//!   must send written reminder during a 15-30 day window before
//!   the tenant's non-renewal-notice deadline. The reminder must
//!   "call the tenant's attention" to the auto-renewal clause.
//!   Florida additionally requires the reminder to list "all fees,
//!   penalties, and other charges applicable to the tenant" upon
//!   renewal.
//!
//! - `PreCancellationDeadlineThirtyToSixtyDays` — IL (815 ILCS 601
//!   Automatic Contract Renewal Act, § 10). Applies to 12-month+
//!   contracts with renewal terms exceeding 1 month: written notice
//!   30-60 days before the cancellation deadline. Clause itself
//!   must be "clear and conspicuous" in the original contract.
//!
//! - `NoStateDisclosureRequirement` — most other states. Auto-renewal
//!   clauses generally enforceable as written; landlord has no
//!   statutory reminder duty (though common-law unconscionability
//!   doctrine may apply in extreme cases).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoRenewalRegime {
    PreNonrenewalNotificationFifteenToThirtyDays,
    PreCancellationDeadlineThirtyToSixtyDays,
    NoStateDisclosureRequirement,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: AutoRenewalRegime,
    pub notice_window_min_days: Option<u32>,
    pub notice_window_max_days: Option<u32>,
    /// True if reminder must enumerate fees/penalties/charges (FL).
    pub must_list_fees_and_charges: bool,
    /// True if state requires the auto-renewal clause itself to be
    /// "clear and conspicuous" in the original contract (IL).
    pub clear_conspicuous_clause_required: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: AutoRenewalRegime,
    notice_window_min_days: Option<u32>,
    notice_window_max_days: Option<u32>,
    must_list_fees_and_charges: bool,
    clear_conspicuous_clause_required: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        notice_window_min_days,
        notice_window_max_days,
        must_list_fees_and_charges,
        clear_conspicuous_clause_required,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use AutoRenewalRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // PreNonrenewalNotificationFifteenToThirtyDays.
    m.insert(
        "FL",
        rule(
            PreNonrenewalNotificationFifteenToThirtyDays,
            Some(15), Some(30), true, false,
            "Fla. Stat. § 83.575 — written notice within 15 days before tenant's non-renewal notification period; must list all fees/penalties/charges",
        ),
    );
    m.insert(
        "WI",
        rule(
            PreNonrenewalNotificationFifteenToThirtyDays,
            Some(15), Some(30), false, false,
            "Wis. Stat. § 704.15 — written notice 15-30 days before tenant's non-renewal notice deadline; otherwise auto-renewal unenforceable",
        ),
    );
    m.insert(
        "NY",
        rule(
            PreNonrenewalNotificationFifteenToThirtyDays,
            Some(15), Some(30), false, false,
            "N.Y. GBL § 5-905 — written notice 15-30 days before tenant's non-renewal notice deadline, calling attention to auto-renewal clause",
        ),
    );

    // PreCancellationDeadlineThirtyToSixtyDays — IL.
    m.insert(
        "IL",
        rule(
            PreCancellationDeadlineThirtyToSixtyDays,
            Some(30), Some(60), false, true,
            "815 ILCS 601/10 Automatic Contract Renewal Act — clear-and-conspicuous clause + written notice 30-60 days before cancellation deadline for 12-month+ contracts with renewal > 1 month",
        ),
    );

    // NoStateDisclosureRequirement for all remaining states + DC.
    let no_rule = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DC", "DE", "GA",
        "HI", "ID", "IN", "IA", "KS", "KY", "LA", "ME", "MD", "MA",
        "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN",
        "TX", "UT", "VT", "VA", "WA", "WV", "WY",
    ];
    for code in no_rule {
        m.insert(
            code,
            rule(
                NoStateDisclosureRequirement,
                None, None, false, false,
                "No state-level automatic renewal disclosure requirement; common-law unconscionability may apply",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRenewalInput {
    pub state_code: String,
    pub lease_has_auto_renewal_clause: bool,
    pub clause_is_clear_and_conspicuous: bool,
    pub landlord_sent_reminder_notice: bool,
    pub days_before_deadline_notice_sent: u32,
    pub notice_listed_fees_and_charges: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRenewalResult {
    pub regime: AutoRenewalRegime,
    pub disclosure_required: bool,
    pub notice_window_min_days: Option<u32>,
    pub notice_window_max_days: Option<u32>,
    pub notice_timing_compliant: bool,
    pub clear_conspicuous_clause_compliant: bool,
    pub fees_listing_compliant: bool,
    pub auto_renewal_enforceable: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &AutoRenewalInput) -> AutoRenewalResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: AutoRenewalRegime::NoStateDisclosureRequirement,
        notice_window_min_days: None,
        notice_window_max_days: None,
        must_list_fees_and_charges: false,
        clear_conspicuous_clause_required: false,
        citation: "Unknown state code; assuming no state-level disclosure requirement",
    });

    let required = rule.regime != AutoRenewalRegime::NoStateDisclosureRequirement
        && input.lease_has_auto_renewal_clause;

    let timing_compliant = if required {
        input.landlord_sent_reminder_notice
            && match (rule.notice_window_min_days, rule.notice_window_max_days) {
                (Some(min), Some(max)) => {
                    input.days_before_deadline_notice_sent >= min
                        && input.days_before_deadline_notice_sent <= max
                }
                _ => true,
            }
    } else {
        true
    };
    let clause_compliant = !rule.clear_conspicuous_clause_required
        || input.clause_is_clear_and_conspicuous;
    let fees_compliant = !rule.must_list_fees_and_charges || input.notice_listed_fees_and_charges;

    let enforceable = !required || (timing_compliant && clause_compliant && fees_compliant);

    let note = match (rule.regime, required, enforceable) {
        (_, false, _) => format!(
            "{:?}: no state disclosure requirement applies (either no auto-renewal clause or state has no rule). Auto-renewal enforceable as written.",
            rule.regime
        ),
        (_, true, true) => format!(
            "{:?}: disclosure required and SATISFIED — written notice sent {} days before deadline within {}-{} day window; auto-renewal ENFORCEABLE.",
            rule.regime,
            input.days_before_deadline_notice_sent,
            rule.notice_window_min_days.unwrap_or(0),
            rule.notice_window_max_days.unwrap_or(0),
        ),
        (_, true, false) => {
            let mut issues: Vec<&str> = Vec::new();
            if !timing_compliant {
                issues.push("notice timing outside statutory window or not sent");
            }
            if !clause_compliant {
                issues.push("clause not clear and conspicuous");
            }
            if !fees_compliant {
                issues.push("notice did not list required fees/penalties/charges");
            }
            format!(
                "{:?} VIOLATION: {}. AUTO-RENEWAL UNENFORCEABLE — lease expires at end of original term.",
                rule.regime,
                issues.join("; "),
            )
        }
    };

    AutoRenewalResult {
        regime: rule.regime,
        disclosure_required: required,
        notice_window_min_days: rule.notice_window_min_days,
        notice_window_max_days: rule.notice_window_max_days,
        notice_timing_compliant: timing_compliant,
        clear_conspicuous_clause_compliant: clause_compliant,
        fees_listing_compliant: fees_compliant,
        auto_renewal_enforceable: enforceable,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, has_clause: bool) -> AutoRenewalInput {
        AutoRenewalInput {
            state_code: state.to_string(),
            lease_has_auto_renewal_clause: has_clause,
            clause_is_clear_and_conspicuous: true,
            landlord_sent_reminder_notice: true,
            days_before_deadline_notice_sent: 20,
            notice_listed_fees_and_charges: true,
        }
    }

    // FL — Fla. Stat. § 83.575.

    #[test]
    fn fl_compliant_with_20_day_notice_and_fees_listed() {
        let r = check(&input("FL", true));
        assert_eq!(
            r.regime,
            AutoRenewalRegime::PreNonrenewalNotificationFifteenToThirtyDays
        );
        assert!(r.disclosure_required);
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn fl_notice_14_days_before_violates_window() {
        let mut i = input("FL", true);
        i.days_before_deadline_notice_sent = 14;
        let r = check(&i);
        assert!(!r.notice_timing_compliant);
        assert!(!r.auto_renewal_enforceable);
    }

    #[test]
    fn fl_notice_15_days_exact_boundary_complies() {
        let mut i = input("FL", true);
        i.days_before_deadline_notice_sent = 15;
        let r = check(&i);
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn fl_notice_30_days_exact_boundary_complies() {
        let mut i = input("FL", true);
        i.days_before_deadline_notice_sent = 30;
        let r = check(&i);
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn fl_notice_31_days_outside_window_violates() {
        let mut i = input("FL", true);
        i.days_before_deadline_notice_sent = 31;
        let r = check(&i);
        assert!(!r.auto_renewal_enforceable);
    }

    #[test]
    fn fl_notice_missing_fees_listing_violates() {
        // FL is the only state requiring fees/charges enumeration.
        let mut i = input("FL", true);
        i.notice_listed_fees_and_charges = false;
        let r = check(&i);
        assert!(!r.fees_listing_compliant);
        assert!(!r.auto_renewal_enforceable);
    }

    #[test]
    fn fl_no_notice_at_all_unenforceable() {
        let mut i = input("FL", true);
        i.landlord_sent_reminder_notice = false;
        let r = check(&i);
        assert!(!r.auto_renewal_enforceable);
        assert!(r.note.contains("UNENFORCEABLE"));
    }

    // WI — Wis. Stat. § 704.15.

    #[test]
    fn wi_compliant_with_20_day_notice() {
        let r = check(&input("WI", true));
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn wi_does_not_require_fees_listing() {
        // Pin: WI ≠ FL on the fees-listing requirement.
        let mut i = input("WI", true);
        i.notice_listed_fees_and_charges = false;
        let r = check(&i);
        assert!(r.fees_listing_compliant);
        assert!(r.auto_renewal_enforceable);
    }

    // NY — GBL § 5-905.

    #[test]
    fn ny_15_30_day_window_complies() {
        let r = check(&input("NY", true));
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn ny_notice_outside_window_violates() {
        let mut i = input("NY", true);
        i.days_before_deadline_notice_sent = 45;
        let r = check(&i);
        assert!(!r.auto_renewal_enforceable);
    }

    // IL — 815 ILCS 601/10, distinct 30-60 day window.

    #[test]
    fn il_30_60_day_window_complies_with_45() {
        let mut i = input("IL", true);
        i.days_before_deadline_notice_sent = 45;
        let r = check(&i);
        assert_eq!(
            r.regime,
            AutoRenewalRegime::PreCancellationDeadlineThirtyToSixtyDays
        );
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn il_29_days_below_minimum_violates() {
        let mut i = input("IL", true);
        i.days_before_deadline_notice_sent = 29;
        let r = check(&i);
        assert!(!r.auto_renewal_enforceable);
    }

    #[test]
    fn il_30_days_exact_boundary_complies() {
        let mut i = input("IL", true);
        i.days_before_deadline_notice_sent = 30;
        let r = check(&i);
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn il_60_days_exact_boundary_complies() {
        let mut i = input("IL", true);
        i.days_before_deadline_notice_sent = 60;
        let r = check(&i);
        assert!(r.auto_renewal_enforceable);
    }

    #[test]
    fn il_61_days_above_maximum_violates() {
        let mut i = input("IL", true);
        i.days_before_deadline_notice_sent = 61;
        let r = check(&i);
        assert!(!r.auto_renewal_enforceable);
    }

    #[test]
    fn il_clause_not_clear_and_conspicuous_violates() {
        let mut i = input("IL", true);
        i.days_before_deadline_notice_sent = 45;
        i.clause_is_clear_and_conspicuous = false;
        let r = check(&i);
        assert!(!r.clear_conspicuous_clause_compliant);
        assert!(!r.auto_renewal_enforceable);
    }

    // No-rule states.

    #[test]
    fn no_rule_state_auto_renewal_always_enforceable() {
        for st in &["TX", "CA", "MA", "OR", "DC", "OH"] {
            let r = check(&input(st, true));
            assert_eq!(r.regime, AutoRenewalRegime::NoStateDisclosureRequirement, "{st}");
            assert!(r.auto_renewal_enforceable, "{st}");
        }
    }

    // No auto-renewal clause case.

    #[test]
    fn no_auto_renewal_clause_no_disclosure_required() {
        let r = check(&input("FL", false));
        assert!(!r.disclosure_required);
        assert!(r.auto_renewal_enforceable);
    }

    // Coverage / structural.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(codes.len(), 51, "expected 50 states + DC, got {}", codes.len());
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn fl_wi_ny_share_15_30_day_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == AutoRenewalRegime::PreNonrenewalNotificationFifteenToThirtyDays {
                count += 1;
            }
        }
        assert_eq!(count, 3, "expected FL + WI + NY only");
    }

    #[test]
    fn il_unique_30_60_day_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == AutoRenewalRegime::PreCancellationDeadlineThirtyToSixtyDays {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected IL only");
    }

    #[test]
    fn only_fl_requires_fees_listing() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.must_list_fees_and_charges {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected FL only with fees-listing requirement");
    }

    #[test]
    fn only_il_requires_clear_conspicuous_clause() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.clear_conspicuous_clause_required {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected IL only with clear-conspicuous requirement");
    }

    #[test]
    fn unknown_state_falls_back_to_no_rule() {
        let r = check(&input("XX", true));
        assert_eq!(r.regime, AutoRenewalRegime::NoStateDisclosureRequirement);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("fl", true));
        assert!(r.disclosure_required);
    }

    // Notes.

    #[test]
    fn fl_violation_note_lists_fees_issue() {
        let mut i = input("FL", true);
        i.notice_listed_fees_and_charges = false;
        let r = check(&i);
        assert!(r.note.contains("VIOLATION"));
        assert!(r.note.contains("fees"));
    }

    #[test]
    fn enforceable_note_describes_window_satisfaction() {
        let r = check(&input("FL", true));
        assert!(r.note.contains("ENFORCEABLE"));
        assert!(r.note.contains("15-30 day window"));
    }
}
