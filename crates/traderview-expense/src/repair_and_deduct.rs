//! State tenant repair-and-deduct / rent escrow / habitability
//! withholding compliance.
//!
//! Approximately 35 states + DC recognize some form of tenant remedy
//! when the landlord fails the implied warranty of habitability. The
//! mechanics vary sharply across states: from self-help repair-and-
//! deduct (capped at 1-4 months rent) to court-ordered escrow to
//! common-law Marini doctrine withholding.
//!
//! Four regimes:
//!
//! - `SelfHelpRepairAndDeduct` — CA (Civ. Code § 1942: 1 month cap,
//!   max 2 times per 12-month period), TX (Prop. Code § 92.0561:
//!   greater of $500 or 1 month rent), MA (G.L. c. 111 § 127L + c.
//!   239 § 8A: up to 4 months rent — the highest cap in the country),
//!   WA (RCW 59.18.100), and others. Tenant gives written notice +
//!   reasonable time, then repairs and deducts.
//!
//! - `CourtOrderedRentEscrowOnly` — MD (Real Prop. § 8-211): no
//!   self-help permitted; tenant must petition the District Court
//!   to establish a rent escrow account, and the court orders rent
//!   into escrow only when a "substantial and serious threat" to
//!   life, health, or safety exists.
//!
//! - `CommonLawMariniDoctrine` — NJ (Marini v. Ireland, 56 N.J.
//!   130 (1970)): tenant may withhold for habitability defects,
//!   BUT must deposit the FULL withheld amount with the court at
//!   the eviction trial to maintain the habitability defense.
//!   Self-help repair-and-deduct also available under Marini's
//!   companion line of cases.
//!
//! - `NoticeThenWithholdOrTerminate` — FL (Stat. § 83.56(1)):
//!   tenant must give 7-day written notice of the noncompliance,
//!   then may either withhold rent OR terminate the rental
//!   agreement. No formal repair-and-deduct in FL.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemedyRegime {
    SelfHelpRepairAndDeduct,
    CourtOrderedRentEscrowOnly,
    CommonLawMariniDoctrine,
    NoticeThenWithholdOrTerminate,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: RemedyRegime,
    /// Months of rent that the repair-and-deduct amount may not
    /// exceed under SelfHelpRepairAndDeduct regime.
    pub repair_cap_months_rent: Option<u32>,
    /// Optional fixed-dollar floor (TX's "$500 or 1 month, greater"
    /// rule). `None` if pure months-cap.
    pub repair_cap_dollar_floor: Option<i64>,
    /// Statutory limit on number of times the remedy can be used
    /// per 12-month period (CA's 2/year rule).
    pub max_uses_per_12_months: Option<u32>,
    pub required_written_notice_days: u32,
    /// True if state requires the withheld rent to be deposited with
    /// the court at eviction trial (NJ Marini).
    pub must_deposit_with_court_at_trial: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: RemedyRegime,
    repair_cap_months_rent: Option<u32>,
    repair_cap_dollar_floor: Option<i64>,
    max_uses_per_12_months: Option<u32>,
    required_written_notice_days: u32,
    must_deposit_with_court_at_trial: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        repair_cap_months_rent,
        repair_cap_dollar_floor,
        max_uses_per_12_months,
        required_written_notice_days,
        must_deposit_with_court_at_trial,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use RemedyRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // SelfHelpRepairAndDeduct regime.
    m.insert(
        "CA",
        rule(
            SelfHelpRepairAndDeduct,
            Some(1), None, Some(2), 14, false,
            "Cal. Civ. Code § 1942 — repair-and-deduct ≤ 1 month rent + max 2 uses per 12-month period; reasonable notice required",
        ),
    );
    m.insert(
        "TX",
        rule(
            SelfHelpRepairAndDeduct,
            Some(1), Some(500), None, 7, false,
            "Tex. Prop. Code § 92.0561 — repair-and-deduct ≤ greater of $500 or 1 month rent; 7-day notice required",
        ),
    );
    m.insert(
        "MA",
        rule(
            SelfHelpRepairAndDeduct,
            Some(4), None, None, 14, false,
            "Mass. G.L. c. 111 § 127L + c. 239 § 8A — repair-and-deduct up to 4 months rent (highest cap in U.S.); rent withholding also available",
        ),
    );
    m.insert(
        "WA",
        rule(
            SelfHelpRepairAndDeduct,
            Some(1), None, None, 30, false,
            "Wash. RCW 59.18.100 — repair-and-deduct ≤ 1 month rent; statutory notice 30 days for general repairs (shorter for emergencies)",
        ),
    );
    m.insert(
        "IL",
        rule(
            SelfHelpRepairAndDeduct,
            Some(1), Some(500), None, 14, false,
            "765 ILCS 705/0.01 + Chicago RLTO § 5-12-110 — repair-and-deduct ≤ greater of $500 or 1 month rent",
        ),
    );

    // CourtOrderedRentEscrowOnly — MD.
    m.insert(
        "MD",
        rule(
            CourtOrderedRentEscrowOnly,
            None, None, None, 30, true,
            "Md. Real Prop. § 8-211 — no self-help repair-and-deduct; tenant must petition District Court for rent escrow; court orders escrow only on substantial-and-serious-threat finding",
        ),
    );

    // CommonLawMariniDoctrine — NJ.
    m.insert(
        "NJ",
        rule(
            CommonLawMariniDoctrine,
            None, None, None, 0, true,
            "N.J. Marini v. Ireland 56 N.J. 130 (1970) — common-law habitability defense; withheld rent must be deposited with court at eviction trial to maintain defense",
        ),
    );

    // NoticeThenWithholdOrTerminate — FL.
    m.insert(
        "FL",
        rule(
            NoticeThenWithholdOrTerminate,
            None, None, None, 7, false,
            "Fla. Stat. § 83.56(1) — 7-day written notice of noncompliance required; tenant may then withhold rent or terminate; no formal repair-and-deduct",
        ),
    );

    // Default SelfHelpRepairAndDeduct with 1-month cap for remaining
    // states that don't have an explicit framework.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "GA", "HI", "ID", "IN", "IA", "KS", "KY",
        "LA", "ME", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NM", "NY", "NC", "ND", "OH",
        "OK", "OR", "PA", "RI", "SC", "SD", "TN", "UT", "VT", "VA", "WV", "WI", "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                SelfHelpRepairAndDeduct,
                Some(1), None, None, 14, false,
                "Generic repair-and-deduct under common law / state-specific statute; ≤ 1 month rent default cap; written notice required",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairDeductInput {
    pub state_code: String,
    pub monthly_rent_dollars: i64,
    pub repair_costs_paid_dollars: i64,
    pub written_notice_given_days_before_repair: u32,
    pub prior_uses_in_past_12_months: u32,
    /// For NJ Marini regime: did the tenant deposit withheld rent with
    /// the court at the eviction trial?
    pub rent_deposited_with_court_at_trial: bool,
    /// For MD: did the court order rent escrow?
    pub court_ordered_escrow: bool,
    pub habitability_violation_substantial_threat: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairDeductResult {
    pub regime: RemedyRegime,
    pub effective_repair_cap_dollars: i64,
    pub repair_cost_within_cap: bool,
    pub max_uses_per_year_satisfied: bool,
    pub notice_period_satisfied: bool,
    pub court_deposit_requirement_satisfied: bool,
    pub remedy_legally_available: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &RepairDeductInput) -> RepairDeductResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: RemedyRegime::SelfHelpRepairAndDeduct,
        repair_cap_months_rent: Some(1),
        repair_cap_dollar_floor: None,
        max_uses_per_12_months: None,
        required_written_notice_days: 14,
        must_deposit_with_court_at_trial: false,
        citation: "Unknown state code; assuming generic 1-month repair-and-deduct default",
    });

    // Compute effective cap for SelfHelp regime.
    let cap_months_dollars = rule
        .repair_cap_months_rent
        .map(|m| (m as i64).saturating_mul(input.monthly_rent_dollars))
        .unwrap_or(i64::MAX);
    let cap = match rule.repair_cap_dollar_floor {
        Some(floor) => cap_months_dollars.max(floor),
        None => cap_months_dollars,
    };

    let within_cap = match rule.regime {
        RemedyRegime::SelfHelpRepairAndDeduct => input.repair_costs_paid_dollars <= cap,
        _ => true, // Other regimes have no per-use cap
    };

    let uses_ok = match rule.max_uses_per_12_months {
        Some(max_uses) => input.prior_uses_in_past_12_months < max_uses,
        None => true,
    };

    let notice_ok =
        input.written_notice_given_days_before_repair >= rule.required_written_notice_days;

    let deposit_ok =
        !rule.must_deposit_with_court_at_trial || input.rent_deposited_with_court_at_trial;

    let available = match rule.regime {
        RemedyRegime::SelfHelpRepairAndDeduct => within_cap && uses_ok && notice_ok,
        RemedyRegime::CourtOrderedRentEscrowOnly => {
            input.court_ordered_escrow && input.habitability_violation_substantial_threat
        }
        RemedyRegime::CommonLawMariniDoctrine => deposit_ok,
        RemedyRegime::NoticeThenWithholdOrTerminate => notice_ok,
    };

    let note = match rule.regime {
        RemedyRegime::SelfHelpRepairAndDeduct => format!(
            "SelfHelpRepairAndDeduct: cap ${} ({}); ${} spent {}; {} uses prior {}; {} days notice {}.",
            cap,
            if let Some(floor) = rule.repair_cap_dollar_floor {
                format!("greater of ${floor} or {} months rent", rule.repair_cap_months_rent.unwrap_or(0))
            } else {
                format!("{} months rent", rule.repair_cap_months_rent.unwrap_or(0))
            },
            input.repair_costs_paid_dollars,
            if within_cap { "within cap" } else { "EXCEEDS CAP" },
            input.prior_uses_in_past_12_months,
            if uses_ok { "OK" } else { "EXCEEDS LIMIT" },
            input.written_notice_given_days_before_repair,
            if notice_ok { "satisfied" } else { "INSUFFICIENT" },
        ),
        RemedyRegime::CourtOrderedRentEscrowOnly => format!(
            "CourtOrderedRentEscrowOnly (MD §8-211): no self-help; court-ordered escrow {} + substantial-threat finding {}.",
            if input.court_ordered_escrow { "OBTAINED" } else { "MISSING" },
            if input.habitability_violation_substantial_threat { "MET" } else { "NOT MET" },
        ),
        RemedyRegime::CommonLawMariniDoctrine => format!(
            "CommonLawMariniDoctrine (NJ): withheld rent must be deposited with court at trial — deposit {}.",
            if input.rent_deposited_with_court_at_trial { "MADE" } else { "MISSING (defense WAIVED)" },
        ),
        RemedyRegime::NoticeThenWithholdOrTerminate => format!(
            "NoticeThenWithholdOrTerminate (FL §83.56(1)): 7-day written notice {} ({} days given).",
            if notice_ok { "satisfied" } else { "INSUFFICIENT" },
            input.written_notice_given_days_before_repair,
        ),
    };

    RepairDeductResult {
        regime: rule.regime,
        effective_repair_cap_dollars: cap,
        repair_cost_within_cap: within_cap,
        max_uses_per_year_satisfied: uses_ok,
        notice_period_satisfied: notice_ok,
        court_deposit_requirement_satisfied: deposit_ok,
        remedy_legally_available: available,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, rent: i64, cost: i64) -> RepairDeductInput {
        RepairDeductInput {
            state_code: state.to_string(),
            monthly_rent_dollars: rent,
            repair_costs_paid_dollars: cost,
            written_notice_given_days_before_repair: 30,
            prior_uses_in_past_12_months: 0,
            rent_deposited_with_court_at_trial: false,
            court_ordered_escrow: false,
            habitability_violation_substantial_threat: false,
        }
    }

    // CA — 1 month cap + 2/year.

    #[test]
    fn ca_within_1_month_cap_compliant() {
        let r = check(&input("CA", 2_000, 1_500));
        assert!(r.repair_cost_within_cap);
        assert!(r.remedy_legally_available);
    }

    #[test]
    fn ca_exceeds_1_month_cap_violates() {
        let r = check(&input("CA", 2_000, 2_500));
        assert!(!r.repair_cost_within_cap);
        assert!(!r.remedy_legally_available);
    }

    #[test]
    fn ca_third_use_in_12_months_violates() {
        let mut i = input("CA", 2_000, 1_000);
        i.prior_uses_in_past_12_months = 2;
        let r = check(&i);
        assert!(!r.max_uses_per_year_satisfied);
        assert!(!r.remedy_legally_available);
    }

    #[test]
    fn ca_first_use_within_year_ok() {
        let mut i = input("CA", 2_000, 1_000);
        i.prior_uses_in_past_12_months = 1;
        let r = check(&i);
        assert!(r.max_uses_per_year_satisfied);
    }

    // TX — greater of $500 or 1 month.

    #[test]
    fn tx_low_rent_uses_500_dollar_floor() {
        // Rent $300 → 1 month cap $300 < $500 floor → effective $500.
        let r = check(&input("TX", 300, 450));
        assert_eq!(r.effective_repair_cap_dollars, 500);
        assert!(r.repair_cost_within_cap);
    }

    #[test]
    fn tx_high_rent_uses_1_month_cap() {
        // Rent $2k → 1 month cap $2k > $500 → effective $2k.
        let r = check(&input("TX", 2_000, 1_500));
        assert_eq!(r.effective_repair_cap_dollars, 2_000);
        assert!(r.repair_cost_within_cap);
    }

    #[test]
    fn tx_low_rent_exceeds_500_floor_violates() {
        let r = check(&input("TX", 300, 501));
        assert!(!r.repair_cost_within_cap);
    }

    // MA — 4 months cap (highest).

    #[test]
    fn ma_within_4_months_cap_compliant() {
        // 4 × $2k = $8k cap.
        let r = check(&input("MA", 2_000, 7_500));
        assert_eq!(r.effective_repair_cap_dollars, 8_000);
        assert!(r.repair_cost_within_cap);
    }

    #[test]
    fn ma_exceeds_4_months_cap_violates() {
        let r = check(&input("MA", 2_000, 8_500));
        assert!(!r.repair_cost_within_cap);
    }

    // WA — 1 month cap + 30-day notice.

    #[test]
    fn wa_30_day_notice_required() {
        let mut i = input("WA", 2_000, 1_500);
        i.written_notice_given_days_before_repair = 29;
        let r = check(&i);
        assert!(!r.notice_period_satisfied);
        assert!(!r.remedy_legally_available);
    }

    #[test]
    fn wa_30_day_notice_exact_complies() {
        let mut i = input("WA", 2_000, 1_500);
        i.written_notice_given_days_before_repair = 30;
        let r = check(&i);
        assert!(r.notice_period_satisfied);
    }

    // MD — CourtOrderedRentEscrowOnly.

    #[test]
    fn md_no_court_order_no_remedy() {
        let r = check(&input("MD", 2_000, 1_500));
        assert_eq!(r.regime, RemedyRegime::CourtOrderedRentEscrowOnly);
        assert!(!r.remedy_legally_available);
    }

    #[test]
    fn md_court_order_plus_substantial_threat_remedy_available() {
        let mut i = input("MD", 2_000, 1_500);
        i.court_ordered_escrow = true;
        i.habitability_violation_substantial_threat = true;
        let r = check(&i);
        assert!(r.remedy_legally_available);
    }

    #[test]
    fn md_court_order_without_substantial_threat_no_remedy() {
        let mut i = input("MD", 2_000, 1_500);
        i.court_ordered_escrow = true;
        // Missing substantial threat finding.
        let r = check(&i);
        assert!(!r.remedy_legally_available);
    }

    // NJ — Marini doctrine.

    #[test]
    fn nj_no_deposit_at_trial_defense_waived() {
        let r = check(&input("NJ", 2_000, 1_500));
        assert_eq!(r.regime, RemedyRegime::CommonLawMariniDoctrine);
        assert!(!r.remedy_legally_available);
        assert!(r.note.contains("defense WAIVED"));
    }

    #[test]
    fn nj_deposit_made_defense_preserved() {
        let mut i = input("NJ", 2_000, 1_500);
        i.rent_deposited_with_court_at_trial = true;
        let r = check(&i);
        assert!(r.remedy_legally_available);
    }

    // FL — Notice required.

    #[test]
    fn fl_7_day_notice_complies() {
        let mut i = input("FL", 2_000, 1_500);
        i.written_notice_given_days_before_repair = 7;
        let r = check(&i);
        assert_eq!(r.regime, RemedyRegime::NoticeThenWithholdOrTerminate);
        assert!(r.remedy_legally_available);
    }

    #[test]
    fn fl_6_day_notice_insufficient() {
        let mut i = input("FL", 2_000, 1_500);
        i.written_notice_given_days_before_repair = 6;
        let r = check(&i);
        assert!(!r.notice_period_satisfied);
        assert!(!r.remedy_legally_available);
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
    fn only_md_uses_court_ordered_escrow_only() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == RemedyRegime::CourtOrderedRentEscrowOnly {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected MD only with CourtOrderedRentEscrowOnly");
    }

    #[test]
    fn only_nj_uses_marini_doctrine() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == RemedyRegime::CommonLawMariniDoctrine {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected NJ only with CommonLawMariniDoctrine");
    }

    #[test]
    fn only_fl_uses_notice_then_withhold_or_terminate() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == RemedyRegime::NoticeThenWithholdOrTerminate {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "expected FL only with NoticeThenWithholdOrTerminate"
        );
    }

    #[test]
    fn ma_uniquely_has_4_month_cap() {
        let ma_cap = RULES.get("MA").unwrap().repair_cap_months_rent.unwrap();
        assert_eq!(ma_cap, 4);
        // No other state should have 4-month cap.
        for (code, rule) in RULES.iter() {
            if *code != "MA" {
                if let Some(months) = rule.repair_cap_months_rent {
                    assert!(
                        months <= 1,
                        "{code} has {months}-month cap; only MA should have > 1"
                    );
                }
            }
        }
    }

    #[test]
    fn unknown_state_falls_back_to_default() {
        let r = check(&input("XX", 2_000, 1_500));
        assert_eq!(r.regime, RemedyRegime::SelfHelpRepairAndDeduct);
        assert_eq!(r.effective_repair_cap_dollars, 2_000);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ca", 2_000, 1_500));
        assert!(r.remedy_legally_available);
    }

    // Notes.

    #[test]
    fn ca_exceed_cap_note_describes_violation() {
        let r = check(&input("CA", 2_000, 5_000));
        assert!(r.note.contains("EXCEEDS CAP"));
    }

    #[test]
    fn nj_no_deposit_note_describes_waiver() {
        let r = check(&input("NJ", 2_000, 1_000));
        assert!(r.note.contains("defense WAIVED"));
    }
}
