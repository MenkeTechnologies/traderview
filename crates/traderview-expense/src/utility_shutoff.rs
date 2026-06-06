//! State landlord-caused utility shutoff prohibitions.
//!
//! Every state prohibits "self-help eviction" via landlord-initiated
//! utility shutoff. The variation is in the statutory penalties:
//! per-day vs flat vs monthly-rent-multiple vs punitive-damages
//! framework.
//!
//! Five regimes:
//!
//! 1. **`PerDayStatutoryPenalty`** — CA (Civ. Code § 789.3: $100/day,
//!    $250 minimum) and WA (RCW 59.18.300: up to $100/day). Per-day
//!    penalty accrues continuously while the shutoff persists, so
//!    even a short-duration violation can become costly fast.
//!
//! 2. **`FlatPlusOneMonthRentPenalty`** — TX (Prop. Code § 92.008:
//!    $1,000 + one month's rent + actual damages + attorney's fees +
//!    court costs). One-time flat amount per incident regardless of
//!    duration.
//!
//! 3. **`MonthlyRentMultiplePenalty`** — FL (Stat. § 83.67: three
//!    months' rent OR actual damages, whichever is higher, plus
//!    costs and attorney's fees).
//!
//! 4. **`PunitiveDamagesFramework`** — NY (RPL § 235-a + RPAPL 853:
//!    compensatory PLUS punitive damages PLUS treble damages for
//!    unlawful eviction). Combined civil + criminal exposure.
//!
//! 5. **`GeneralProhibitionStandardRemedies`** — most other states.
//!    Self-help eviction prohibited; tenant remedies include
//!    actual damages and injunctive relief; no statute-specific
//!    enhanced penalty.
//!
//! **Common emergency / repair exception**: every state allows
//! interruption for bona-fide repairs, construction, or emergency
//! (water main break, gas leak, etc.). Module models this via
//! `interruption_due_to_bona_fide_repair_or_emergency: bool`.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutoffRegime {
    PerDayStatutoryPenalty,
    FlatPlusOneMonthRentPenalty,
    MonthlyRentMultiplePenalty,
    PunitiveDamagesFramework,
    GeneralProhibitionStandardRemedies,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: ShutoffRegime,
    pub per_day_penalty_dollars: Option<i64>,
    pub minimum_penalty_dollars: Option<i64>,
    pub flat_penalty_dollars: Option<i64>,
    pub monthly_rent_multiplier_months: Option<u32>,
    pub actual_damages_recoverable: bool,
    pub attorney_fees_recoverable: bool,
    pub punitive_damages_available: bool,
    pub treble_damages_available: bool,
    pub criminal_penalties_possible: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: ShutoffRegime,
    per_day_penalty_dollars: Option<i64>,
    minimum_penalty_dollars: Option<i64>,
    flat_penalty_dollars: Option<i64>,
    monthly_rent_multiplier_months: Option<u32>,
    actual_damages_recoverable: bool,
    attorney_fees_recoverable: bool,
    punitive_damages_available: bool,
    treble_damages_available: bool,
    criminal_penalties_possible: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        per_day_penalty_dollars,
        minimum_penalty_dollars,
        flat_penalty_dollars,
        monthly_rent_multiplier_months,
        actual_damages_recoverable,
        attorney_fees_recoverable,
        punitive_damages_available,
        treble_damages_available,
        criminal_penalties_possible,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use ShutoffRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // PerDayStatutoryPenalty regime.
    m.insert(
        "CA",
        rule(
            PerDayStatutoryPenalty,
            Some(100),
            Some(250),
            None,
            None,
            true,
            true,
            false,
            false,
            false,
            "Cal. Civ. Code § 789.3 — $100/day + $250 minimum + actual damages + attorney's fees",
        ),
    );
    m.insert(
        "WA",
        rule(
            PerDayStatutoryPenalty,
            Some(100), None, None, None,
            true, true, false, false, false,
            "Wash. RCW 59.18.300 — up to $100/day + actual damages + prevailing-party attorney's fees and costs",
        ),
    );

    // FlatPlusOneMonthRentPenalty regime.
    m.insert(
        "TX",
        rule(
            FlatPlusOneMonthRentPenalty,
            None, None, Some(1_000), Some(1),
            true, true, false, false, false,
            "Tex. Prop. Code § 92.008 — $1,000 + one month's rent + actual damages + attorney's fees + court costs",
        ),
    );

    // MonthlyRentMultiplePenalty regime.
    m.insert(
        "FL",
        rule(
            MonthlyRentMultiplePenalty,
            None, None, None, Some(3),
            true, true, false, false, false,
            "Fla. Stat. § 83.67 — 3 months' rent OR actual damages (whichever higher) + costs + attorney's fees",
        ),
    );

    // PunitiveDamagesFramework regime.
    m.insert(
        "NY",
        rule(
            PunitiveDamagesFramework,
            None, None, None, None,
            true, true, true, true, true,
            "N.Y. RPL § 235-a + RPAPL 853 — compensatory + punitive damages; treble damages for unlawful eviction; criminal exposure",
        ),
    );

    // GeneralProhibitionStandardRemedies for remaining states + DC.
    let no_specific = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "UT", "VT", "VA", "WV", "WI",
        "WY",
    ];
    for code in no_specific {
        m.insert(
            code,
            rule(
                GeneralProhibitionStandardRemedies,
                None, None, None, None,
                true, false, false, false, false,
                "Self-help eviction prohibited; tenant remedies include actual damages + injunctive relief",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutoffInput {
    pub state_code: String,
    pub landlord_caused_shutoff: bool,
    pub days_without_utility: u32,
    pub monthly_rent_dollars: i64,
    pub tenant_actual_damages_dollars: i64,
    pub interruption_due_to_bona_fide_repair_or_emergency: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutoffResult {
    pub regime: ShutoffRegime,
    pub violation_occurred: bool,
    pub per_day_penalty_total_dollars: i64,
    pub minimum_penalty_floor_dollars: i64,
    pub flat_statutory_penalty_dollars: i64,
    pub monthly_rent_multiplier_penalty_dollars: i64,
    pub total_statutory_damages_dollars: i64,
    pub punitive_damages_available: bool,
    pub treble_damages_available: bool,
    pub attorney_fees_recoverable: bool,
    pub criminal_penalties_possible: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &ShutoffInput) -> ShutoffResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: ShutoffRegime::GeneralProhibitionStandardRemedies,
        per_day_penalty_dollars: None,
        minimum_penalty_dollars: None,
        flat_penalty_dollars: None,
        monthly_rent_multiplier_months: None,
        actual_damages_recoverable: true,
        attorney_fees_recoverable: false,
        punitive_damages_available: false,
        treble_damages_available: false,
        criminal_penalties_possible: false,
        citation: "Unknown state code; assuming general self-help-eviction prohibition",
    });

    // Bona-fide repair/emergency exception bars any violation.
    let violation =
        input.landlord_caused_shutoff && !input.interruption_due_to_bona_fide_repair_or_emergency;

    let per_day_total = if violation {
        rule.per_day_penalty_dollars
            .map(|p| (p as u64).saturating_mul(input.days_without_utility as u64) as i64)
            .unwrap_or(0)
    } else {
        0
    };
    let minimum_floor = if violation {
        rule.minimum_penalty_dollars.unwrap_or(0)
    } else {
        0
    };
    let per_day_with_floor = per_day_total.max(minimum_floor);

    let flat_penalty = if violation {
        rule.flat_penalty_dollars.unwrap_or(0)
    } else {
        0
    };
    let rent_multiple = if violation {
        rule.monthly_rent_multiplier_months
            .map(|m| (m as i64).saturating_mul(input.monthly_rent_dollars))
            .unwrap_or(0)
    } else {
        0
    };

    let total_statutory = match rule.regime {
        ShutoffRegime::PerDayStatutoryPenalty => per_day_with_floor,
        ShutoffRegime::FlatPlusOneMonthRentPenalty => flat_penalty + rent_multiple,
        ShutoffRegime::MonthlyRentMultiplePenalty => {
            // FL: greater of (rent multiple) or actual damages.
            rent_multiple.max(input.tenant_actual_damages_dollars)
        }
        ShutoffRegime::PunitiveDamagesFramework => input.tenant_actual_damages_dollars,
        ShutoffRegime::GeneralProhibitionStandardRemedies => input.tenant_actual_damages_dollars,
    };

    let note = match (rule.regime, violation) {
        (_, false) => {
            if input.interruption_due_to_bona_fide_repair_or_emergency {
                "Bona-fide repair / emergency exception: no violation despite shutoff.".to_string()
            } else {
                "No landlord-caused shutoff; no §-specific penalty exposure.".to_string()
            }
        }
        (ShutoffRegime::PerDayStatutoryPenalty, true) => format!(
            "PerDayStatutoryPenalty VIOLATION: {} day(s) × ${}/day = ${} (minimum ${}) statutory penalty; actual damages + attorney's fees additional.",
            input.days_without_utility,
            rule.per_day_penalty_dollars.unwrap_or(0),
            per_day_total,
            rule.minimum_penalty_dollars.unwrap_or(0),
        ),
        (ShutoffRegime::FlatPlusOneMonthRentPenalty, true) => format!(
            "FlatPlusOneMonthRentPenalty VIOLATION: ${} flat + ${} × {} month(s) rent = ${} statutory; actual damages + attorney's fees additional.",
            flat_penalty,
            input.monthly_rent_dollars,
            rule.monthly_rent_multiplier_months.unwrap_or(0),
            flat_penalty + rent_multiple,
        ),
        (ShutoffRegime::MonthlyRentMultiplePenalty, true) => format!(
            "MonthlyRentMultiplePenalty VIOLATION: greater of {} months' rent (${}) or actual damages (${}) = ${} + costs + attorney's fees.",
            rule.monthly_rent_multiplier_months.unwrap_or(0),
            rent_multiple,
            input.tenant_actual_damages_dollars,
            total_statutory,
        ),
        (ShutoffRegime::PunitiveDamagesFramework, true) =>
            "PunitiveDamagesFramework VIOLATION: compensatory + PUNITIVE damages + TREBLE damages for unlawful eviction; criminal exposure available.".to_string(),
        (ShutoffRegime::GeneralProhibitionStandardRemedies, true) =>
            "GeneralProhibitionStandardRemedies VIOLATION: self-help eviction prohibited; tenant entitled to actual damages + injunctive relief.".to_string(),
    };

    ShutoffResult {
        regime: rule.regime,
        violation_occurred: violation,
        per_day_penalty_total_dollars: per_day_total,
        minimum_penalty_floor_dollars: minimum_floor,
        flat_statutory_penalty_dollars: flat_penalty,
        monthly_rent_multiplier_penalty_dollars: rent_multiple,
        total_statutory_damages_dollars: total_statutory,
        punitive_damages_available: rule.punitive_damages_available && violation,
        treble_damages_available: rule.treble_damages_available && violation,
        attorney_fees_recoverable: rule.attorney_fees_recoverable,
        criminal_penalties_possible: rule.criminal_penalties_possible && violation,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, days: u32, rent: i64) -> ShutoffInput {
        ShutoffInput {
            state_code: state.to_string(),
            landlord_caused_shutoff: true,
            days_without_utility: days,
            monthly_rent_dollars: rent,
            tenant_actual_damages_dollars: 0,
            interruption_due_to_bona_fide_repair_or_emergency: false,
        }
    }

    // CA per-day penalty.

    #[test]
    fn ca_10_days_yields_1000_dollar_penalty() {
        let r = check(&input("CA", 10, 2_000));
        assert_eq!(r.regime, ShutoffRegime::PerDayStatutoryPenalty);
        assert!(r.violation_occurred);
        assert_eq!(r.per_day_penalty_total_dollars, 1_000);
        assert_eq!(r.total_statutory_damages_dollars, 1_000);
    }

    #[test]
    fn ca_1_day_minimum_floor_applies_250() {
        // $100/day × 1 day = $100, but $250 minimum floors it.
        let r = check(&input("CA", 1, 2_000));
        assert_eq!(r.per_day_penalty_total_dollars, 100);
        assert_eq!(r.total_statutory_damages_dollars, 250);
    }

    #[test]
    fn ca_3_days_above_minimum_applies_per_day() {
        // $100 × 3 = $300 > $250 minimum → per-day wins.
        let r = check(&input("CA", 3, 2_000));
        assert_eq!(r.total_statutory_damages_dollars, 300);
    }

    #[test]
    fn ca_attorney_fees_recoverable() {
        let r = check(&input("CA", 10, 2_000));
        assert!(r.attorney_fees_recoverable);
    }

    // WA per-day penalty.

    #[test]
    fn wa_30_days_yields_3000_dollar_penalty() {
        let r = check(&input("WA", 30, 1_500));
        assert_eq!(r.per_day_penalty_total_dollars, 3_000);
    }

    #[test]
    fn wa_no_minimum_floor() {
        // WA has no minimum floor; 1 day × $100 = $100.
        let r = check(&input("WA", 1, 1_500));
        assert_eq!(r.total_statutory_damages_dollars, 100);
    }

    // TX flat + one month rent.

    #[test]
    fn tx_flat_1000_plus_one_month_rent() {
        let r = check(&input("TX", 5, 2_500));
        assert_eq!(r.regime, ShutoffRegime::FlatPlusOneMonthRentPenalty);
        assert_eq!(r.flat_statutory_penalty_dollars, 1_000);
        assert_eq!(r.monthly_rent_multiplier_penalty_dollars, 2_500);
        assert_eq!(r.total_statutory_damages_dollars, 3_500);
    }

    #[test]
    fn tx_duration_does_not_change_penalty() {
        // TX penalty is one-time flat — duration doesn't affect.
        let short = check(&input("TX", 1, 2_500));
        let long = check(&input("TX", 100, 2_500));
        assert_eq!(
            short.total_statutory_damages_dollars,
            long.total_statutory_damages_dollars
        );
    }

    // FL 3 months rent or actual damages.

    #[test]
    fn fl_3_months_rent_when_higher_than_actual() {
        let mut i = input("FL", 5, 2_000);
        i.tenant_actual_damages_dollars = 500;
        let r = check(&i);
        assert_eq!(r.regime, ShutoffRegime::MonthlyRentMultiplePenalty);
        // 3 × $2k = $6k > $500 actual.
        assert_eq!(r.monthly_rent_multiplier_penalty_dollars, 6_000);
        assert_eq!(r.total_statutory_damages_dollars, 6_000);
    }

    #[test]
    fn fl_actual_damages_when_higher_than_rent_multiple() {
        let mut i = input("FL", 5, 1_000);
        i.tenant_actual_damages_dollars = 10_000;
        let r = check(&i);
        // 3 × $1k = $3k < $10k actual → actual wins.
        assert_eq!(r.total_statutory_damages_dollars, 10_000);
    }

    // NY punitive damages framework.

    #[test]
    fn ny_punitive_and_treble_available() {
        let mut i = input("NY", 5, 2_000);
        i.tenant_actual_damages_dollars = 3_000;
        let r = check(&i);
        assert_eq!(r.regime, ShutoffRegime::PunitiveDamagesFramework);
        assert!(r.punitive_damages_available);
        assert!(r.treble_damages_available);
        assert!(r.criminal_penalties_possible);
    }

    #[test]
    fn ny_actual_damages_form_statutory_base() {
        let mut i = input("NY", 5, 2_000);
        i.tenant_actual_damages_dollars = 3_000;
        let r = check(&i);
        assert_eq!(r.total_statutory_damages_dollars, 3_000);
    }

    // General-prohibition states.

    #[test]
    fn or_general_prohibition_no_specific_penalty() {
        let mut i = input("OR", 5, 2_000);
        i.tenant_actual_damages_dollars = 1_500;
        let r = check(&i);
        assert_eq!(r.regime, ShutoffRegime::GeneralProhibitionStandardRemedies);
        assert!(r.violation_occurred);
        assert_eq!(r.total_statutory_damages_dollars, 1_500);
        assert!(!r.attorney_fees_recoverable);
        assert!(!r.punitive_damages_available);
    }

    // Bona-fide exception.

    #[test]
    fn bona_fide_repair_exception_blocks_violation_ca() {
        let mut i = input("CA", 10, 2_000);
        i.interruption_due_to_bona_fide_repair_or_emergency = true;
        let r = check(&i);
        assert!(!r.violation_occurred);
        assert_eq!(r.total_statutory_damages_dollars, 0);
        assert!(r.note.contains("Bona-fide repair"));
    }

    #[test]
    fn no_shutoff_no_violation() {
        let mut i = input("CA", 10, 2_000);
        i.landlord_caused_shutoff = false;
        let r = check(&i);
        assert!(!r.violation_occurred);
        assert_eq!(r.total_statutory_damages_dollars, 0);
    }

    // Coverage / structural pins.

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
    fn per_day_regime_only_ca_and_wa() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == ShutoffRegime::PerDayStatutoryPenalty {
                count += 1;
            }
        }
        assert_eq!(count, 2, "expected CA + WA only on PerDayStatutoryPenalty");
    }

    #[test]
    fn flat_plus_month_regime_only_tx() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == ShutoffRegime::FlatPlusOneMonthRentPenalty {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected TX only on FlatPlusOneMonthRentPenalty");
    }

    #[test]
    fn monthly_rent_multiple_regime_only_fl() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == ShutoffRegime::MonthlyRentMultiplePenalty {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected FL only on MonthlyRentMultiplePenalty");
    }

    #[test]
    fn punitive_regime_only_ny() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == ShutoffRegime::PunitiveDamagesFramework {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected NY only on PunitiveDamagesFramework");
    }

    #[test]
    fn unknown_state_falls_back_to_general() {
        let r = check(&input("XX", 10, 2_000));
        assert_eq!(r.regime, ShutoffRegime::GeneralProhibitionStandardRemedies);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ca", 10, 2_000));
        assert!(r.violation_occurred);
    }

    #[test]
    fn ca_violation_note_describes_per_day_math() {
        let r = check(&input("CA", 10, 2_000));
        assert!(r.note.contains("PerDayStatutoryPenalty VIOLATION"));
        assert!(r.note.contains("10 day"));
        assert!(r.note.contains("$100/day"));
    }

    #[test]
    fn tx_violation_note_describes_flat_plus_rent() {
        let r = check(&input("TX", 5, 2_500));
        assert!(r.note.contains("FlatPlusOneMonthRentPenalty"));
        assert!(r.note.contains("$1000 flat"));
        assert!(r.note.contains("month(s) rent"));
    }

    #[test]
    fn ny_violation_note_describes_punitive_path() {
        let mut i = input("NY", 5, 2_000);
        i.tenant_actual_damages_dollars = 1_000;
        let r = check(&i);
        assert!(r.note.contains("PUNITIVE damages"));
        assert!(r.note.contains("TREBLE damages"));
        assert!(r.note.contains("criminal"));
    }
}
