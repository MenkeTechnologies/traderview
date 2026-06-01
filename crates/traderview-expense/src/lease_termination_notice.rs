//! State landlord-side lease termination / non-renewal notice
//! periods.
//!
//! Direct trader-landlord operational concern: ending a month-to-month
//! tenancy, declining to renew an expiring lease, or raising rent
//! ALL require statutory minimum advance notice that varies sharply
//! across states by tenancy length.
//!
//! Four regimes:
//!
//! 1. **`TieredByTenancyLength`** — NY (HSTPA 2019, RPL § 226-c):
//!    30 days for tenancy < 1 year, 60 days for 1-2 years, 90 days for
//!    2+ years. CA (CCP § 1946.1): 30 days for < 1 year, 60 days for
//!    ≥ 1 year (single 1-year cliff, no 2-year tier).
//!
//! 2. **`JustCauseAfterTwelveMonths`** — OR (SB 608, eff. 2019),
//!    CA (Tenant Protection Act 2019, AB 1482). After 12 months of
//!    occupancy, landlord cannot terminate without one of the
//!    enumerated "just cause" reasons. First 12 months are no-cause
//!    eligible with 30-day notice in OR; CA AB 1482 only applies to
//!    units 15+ years old.
//!
//! 3. **`StatewideJustCauseAlways`** — WA (RCW 59.18.650), NJ
//!    (Anti-Eviction Act of 1974, N.J.S.A. 2A:18-61.1). Just cause
//!    required from day 1 of tenancy; no no-cause termination
//!    allowed by landlord at any tenancy length.
//!
//! 4. **`StandardThirtyDay`** — most US states. 30 days' written
//!    notice for month-to-month termination by either side. No
//!    tenancy-length tiering, no just-cause requirement.
//!
//! **Rent increase notice** is in most states governed by the same
//! statute as termination but often with a different number — CA
//! requires 30 days for increases ≤ 10% but 90 days for increases >
//! 10%; OR requires 90 days for any increase ≤ 10% and 180 days for >
//! 10% — the module pins these separately under
//! `rent_increase_notice_days_under_10_pct` /
//! `rent_increase_notice_days_over_10_pct`.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeRegime {
    TieredByTenancyLength,
    JustCauseAfterTwelveMonths,
    StatewideJustCauseAlways,
    StandardThirtyDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationIntent {
    /// Landlord wants to end month-to-month tenancy or not renew
    /// an expiring lease with no specific cause.
    NoCauseTermination,
    RentIncrease,
    /// Specific cause (tenant breach, owner move-in, demolition, etc.)
    /// — bypasses most no-cause prohibitions but may have its own
    /// statutory notice and reason requirements.
    JustCauseTermination,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: NoticeRegime,
    pub notice_days_under_1_year: u32,
    pub notice_days_1_to_2_years: u32,
    pub notice_days_over_2_years: u32,
    pub rent_increase_notice_days_under_10_pct: u32,
    pub rent_increase_notice_days_over_10_pct: u32,
    /// When `Some(m)`, no-cause termination is unavailable after the
    /// tenant has occupied the unit for `m` months.
    pub no_cause_unavailable_after_months: Option<u32>,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: NoticeRegime,
    notice_days_under_1_year: u32,
    notice_days_1_to_2_years: u32,
    notice_days_over_2_years: u32,
    rent_increase_notice_days_under_10_pct: u32,
    rent_increase_notice_days_over_10_pct: u32,
    no_cause_unavailable_after_months: Option<u32>,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        notice_days_under_1_year,
        notice_days_1_to_2_years,
        notice_days_over_2_years,
        rent_increase_notice_days_under_10_pct,
        rent_increase_notice_days_over_10_pct,
        no_cause_unavailable_after_months,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use NoticeRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // TieredByTenancyLength regime.
    m.insert(
        "NY",
        rule(
            TieredByTenancyLength,
            30,
            60,
            90,
            30, // ≤ 5% increase — no notice required statewide;
                // > 5% triggers same 30/60/90 tiers as termination
            90, // For > 5% increase at 2+ year tenancy
            None,
            "N.Y. RPL § 226-c (HSTPA of 2019)",
        ),
    );
    m.insert(
        "CA",
        rule(
            // CA has both tiered notice AND just-cause for some units;
            // model as JustCauseAfterTwelveMonths (the more restrictive
            // pattern that callers must check).
            JustCauseAfterTwelveMonths,
            30,
            60,
            60, // CA caps at 60-day tier
            30, // ≤ 10% increase
            90, // > 10% increase
            Some(12), // Tenant Protection Act of 2019 (AB 1482)
            "Cal. Civ. Proc. § 1946.1 + Tenant Protection Act 2019 (AB 1482)",
        ),
    );

    // JustCauseAfterTwelveMonths regime.
    m.insert(
        "OR",
        rule(
            JustCauseAfterTwelveMonths,
            30,
            90,
            90,
            90,  // ≤ 10% increase
            180, // > 10% increase
            Some(12),
            "Or. SB 608 of 2019 (ORS 90.427)",
        ),
    );

    // StatewideJustCauseAlways regime.
    m.insert(
        "WA",
        rule(
            StatewideJustCauseAlways,
            20, // Tenant-initiated only; landlord-side requires just cause
            20,
            20,
            60, // Any increase requires 60 days
            120,
            Some(0),
            "Wash. RCW 59.18.650 (Engrossed SSB 5160 of 2021)",
        ),
    );
    m.insert(
        "NJ",
        rule(
            StatewideJustCauseAlways,
            30,
            30,
            30,
            30,
            30,
            Some(0),
            "N.J.S.A. 2A:18-61.1 (Anti-Eviction Act of 1974)",
        ),
    );

    // Other states with notable variants from default 30-day.
    m.insert(
        "DE",
        rule(
            TieredByTenancyLength,
            60,
            60,
            60,
            60,
            60,
            None,
            "Del. Code Ann. tit. 25 § 5106 (60-day landlord notice)",
        ),
    );
    m.insert(
        "GA",
        rule(
            StandardThirtyDay,
            60, // GA requires landlord 60-day vs tenant 30-day
            60,
            60,
            60,
            60,
            None,
            "Ga. Code Ann. § 44-7-7 (60-day landlord notice)",
        ),
    );
    m.insert(
        "FL",
        rule(
            StandardThirtyDay,
            30,
            30,
            30,
            30,
            30,
            None,
            "Fla. Stat. § 83.57(3) (30-day monthly tenancy)",
        ),
    );
    m.insert(
        "MA",
        rule(
            StandardThirtyDay,
            30,
            30,
            30,
            30,
            30,
            None,
            "Mass. Gen. Laws ch. 186 § 12 (30-day or one rental period)",
        ),
    );
    m.insert(
        "TX",
        rule(
            StandardThirtyDay,
            30,
            30,
            30,
            30,
            30,
            None,
            "Tex. Prop. Code § 91.001 (30-day default)",
        ),
    );

    // Bulk default: 30-day standard.
    let default_30 = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "HI", "ID", "IL",
        "IN", "IA", "KS", "KY", "LA", "ME", "MD", "MI", "MN", "MS",
        "MO", "MT", "NE", "NV", "NH", "NM", "NC", "ND", "OH", "OK",
        "PA", "RI", "SC", "SD", "TN", "UT", "VT", "VA", "WV", "WI",
        "WY",
    ];
    for code in default_30 {
        m.insert(
            code,
            rule(
                StandardThirtyDay,
                30,
                30,
                30,
                30,
                30,
                None,
                "Default 30-day month-to-month termination",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeInput {
    pub state_code: String,
    pub tenancy_months: u32,
    pub notice_days_given: i64,
    pub intent: TerminationIntent,
    /// True if landlord-asserted just cause; bypasses no-cause
    /// prohibition under JustCauseAfterTwelveMonths /
    /// StatewideJustCauseAlways regimes.
    pub has_qualifying_just_cause: bool,
    /// For RentIncrease intent: pct increase over current rent in
    /// basis points (e.g., 1000 = 10.00%).
    pub rent_increase_pct_bp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeResult {
    pub regime: NoticeRegime,
    pub required_notice_days: u32,
    pub notice_compliant: bool,
    pub no_cause_termination_unavailable: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &NoticeInput) -> NoticeResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: NoticeRegime::StandardThirtyDay,
        notice_days_under_1_year: 30,
        notice_days_1_to_2_years: 30,
        notice_days_over_2_years: 30,
        rent_increase_notice_days_under_10_pct: 30,
        rent_increase_notice_days_over_10_pct: 30,
        no_cause_unavailable_after_months: None,
        citation: "Unknown state code; assuming 30-day standard",
    });

    // Determine the required notice based on intent + tenancy length.
    let required = match input.intent {
        TerminationIntent::RentIncrease => {
            if input.rent_increase_pct_bp > 1000 {
                rule.rent_increase_notice_days_over_10_pct
            } else {
                rule.rent_increase_notice_days_under_10_pct
            }
        }
        TerminationIntent::NoCauseTermination | TerminationIntent::JustCauseTermination => {
            if input.tenancy_months < 12 {
                rule.notice_days_under_1_year
            } else if input.tenancy_months < 24 {
                rule.notice_days_1_to_2_years
            } else {
                rule.notice_days_over_2_years
            }
        }
    };

    // No-cause termination unavailable for tenancies past the
    // statewide threshold.
    let no_cause_unavail = match rule.no_cause_unavailable_after_months {
        Some(m) => {
            input.intent == TerminationIntent::NoCauseTermination
                && input.tenancy_months >= m
                && !input.has_qualifying_just_cause
        }
        None => false,
    };

    let notice_compliant =
        input.notice_days_given >= required as i64 && !no_cause_unavail;

    let note = if no_cause_unavail {
        format!(
            "No-cause termination unavailable: state requires just cause after {} months of tenancy; current tenancy {} months. Landlord must assert and prove a qualifying cause.",
            rule.no_cause_unavailable_after_months.unwrap_or(0),
            input.tenancy_months,
        )
    } else {
        format!(
            "{:?}: {} days required ({}/{}/{} tiered by tenancy length); {} days given → {}.",
            rule.regime,
            required,
            rule.notice_days_under_1_year,
            rule.notice_days_1_to_2_years,
            rule.notice_days_over_2_years,
            input.notice_days_given,
            if notice_compliant {
                "compliant"
            } else {
                "INSUFFICIENT"
            },
        )
    };

    NoticeResult {
        regime: rule.regime,
        required_notice_days: required,
        notice_compliant,
        no_cause_termination_unavailable: no_cause_unavail,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        state: &str,
        months: u32,
        notice: i64,
        intent: TerminationIntent,
    ) -> NoticeInput {
        NoticeInput {
            state_code: state.to_string(),
            tenancy_months: months,
            notice_days_given: notice,
            intent,
            has_qualifying_just_cause: false,
            rent_increase_pct_bp: 0,
        }
    }

    // NY RPL § 226-c tiered.

    #[test]
    fn ny_under_1yr_30_day_notice_complies() {
        let r = check(&input("NY", 6, 30, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 30);
        assert!(r.notice_compliant);
    }

    #[test]
    fn ny_1_to_2yr_60_day_notice_required() {
        let r = check(&input("NY", 18, 60, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 60);
        assert!(r.notice_compliant);
    }

    #[test]
    fn ny_1_to_2yr_30_day_notice_insufficient() {
        let r = check(&input("NY", 18, 30, TerminationIntent::NoCauseTermination));
        assert!(!r.notice_compliant);
    }

    #[test]
    fn ny_2yr_plus_90_day_notice_required() {
        let r = check(&input("NY", 30, 90, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 90);
        assert!(r.notice_compliant);
    }

    #[test]
    fn ny_24_month_exact_boundary_uses_2yr_tier() {
        // 24 months = 2 years → falls into 2yr+ tier.
        let r = check(&input("NY", 24, 60, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 90);
        assert!(!r.notice_compliant);
    }

    // CA CCP § 1946.1 + Tenant Protection Act.

    #[test]
    fn ca_under_1yr_no_cause_30_day_complies() {
        let r = check(&input("CA", 6, 30, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 30);
        assert!(r.notice_compliant);
        assert!(!r.no_cause_termination_unavailable);
    }

    #[test]
    fn ca_over_12mo_no_cause_unavailable_under_tpa() {
        let r = check(&input("CA", 13, 60, TerminationIntent::NoCauseTermination));
        assert!(r.no_cause_termination_unavailable);
        assert!(!r.notice_compliant);
        assert!(r.note.contains("just cause"));
    }

    #[test]
    fn ca_over_12mo_just_cause_with_60_day_complies() {
        let mut i = input("CA", 18, 60, TerminationIntent::JustCauseTermination);
        i.has_qualifying_just_cause = true;
        let r = check(&i);
        assert!(!r.no_cause_termination_unavailable);
        assert!(r.notice_compliant);
    }

    // OR SB 608.

    #[test]
    fn or_under_12mo_no_cause_30_day_complies() {
        let r = check(&input("OR", 6, 30, TerminationIntent::NoCauseTermination));
        assert!(r.notice_compliant);
    }

    #[test]
    fn or_after_12mo_no_cause_unavailable() {
        let r = check(&input("OR", 13, 90, TerminationIntent::NoCauseTermination));
        assert!(r.no_cause_termination_unavailable);
        assert!(!r.notice_compliant);
    }

    #[test]
    fn or_after_12mo_just_cause_90_day_complies() {
        let mut i = input("OR", 24, 90, TerminationIntent::JustCauseTermination);
        i.has_qualifying_just_cause = true;
        let r = check(&i);
        assert!(r.notice_compliant);
        assert_eq!(r.required_notice_days, 90);
    }

    // WA RCW 59.18.650 — statewide just cause always.

    #[test]
    fn wa_day_one_just_cause_required() {
        // Even at 1 month tenancy, WA requires just cause.
        let r = check(&input("WA", 1, 60, TerminationIntent::NoCauseTermination));
        assert!(r.no_cause_termination_unavailable);
    }

    #[test]
    fn wa_just_cause_termination_complies_with_20_day() {
        let mut i = input("WA", 36, 20, TerminationIntent::JustCauseTermination);
        i.has_qualifying_just_cause = true;
        let r = check(&i);
        assert!(r.notice_compliant);
    }

    // NJ Anti-Eviction Act — also statewide just cause always.

    #[test]
    fn nj_no_cause_unavailable_always() {
        let r = check(&input("NJ", 6, 30, TerminationIntent::NoCauseTermination));
        assert!(r.no_cause_termination_unavailable);
    }

    // Default-30 states.

    #[test]
    fn tx_default_30_day_complies() {
        let r = check(&input("TX", 36, 30, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 30);
        assert!(r.notice_compliant);
    }

    #[test]
    fn tx_29_day_insufficient() {
        let r = check(&input("TX", 6, 29, TerminationIntent::NoCauseTermination));
        assert!(!r.notice_compliant);
    }

    // DE / GA: 60-day landlord notice.

    #[test]
    fn de_60_day_landlord_notice_required() {
        let r = check(&input("DE", 6, 30, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 60);
        assert!(!r.notice_compliant);
    }

    #[test]
    fn ga_60_day_landlord_notice_required() {
        let r = check(&input("GA", 6, 30, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 60);
        assert!(!r.notice_compliant);
    }

    // Rent increase pathway.

    #[test]
    fn ca_rent_increase_under_10pct_30_day_complies() {
        let mut i = input("CA", 12, 30, TerminationIntent::RentIncrease);
        i.rent_increase_pct_bp = 500; // 5%
        let r = check(&i);
        assert_eq!(r.required_notice_days, 30);
        assert!(r.notice_compliant);
    }

    #[test]
    fn ca_rent_increase_over_10pct_90_day_required() {
        let mut i = input("CA", 12, 30, TerminationIntent::RentIncrease);
        i.rent_increase_pct_bp = 1500; // 15%
        let r = check(&i);
        assert_eq!(r.required_notice_days, 90);
        assert!(!r.notice_compliant);
    }

    #[test]
    fn or_rent_increase_over_10pct_180_day_required() {
        let mut i = input("OR", 24, 90, TerminationIntent::RentIncrease);
        i.rent_increase_pct_bp = 1100; // 11%
        let r = check(&i);
        assert_eq!(r.required_notice_days, 180);
        assert!(!r.notice_compliant);
    }

    #[test]
    fn rent_increase_intent_bypasses_no_cause_unavailable() {
        // Rent increase under JustCauseAfterTwelveMonths regime is NOT
        // "termination" — landlord doesn't need just cause to increase
        // rent under the model. (Caller may still be subject to
        // rent-cap statute like CA AB 1482 — that's a separate module.)
        let r = check(
            &NoticeInput {
                state_code: "CA".into(),
                tenancy_months: 18,
                notice_days_given: 30,
                intent: TerminationIntent::RentIncrease,
                has_qualifying_just_cause: false,
                rent_increase_pct_bp: 500,
            },
        );
        assert!(!r.no_cause_termination_unavailable);
        assert!(r.notice_compliant);
    }

    // Coverage / structural pins.

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
    fn unknown_state_falls_back_to_30_day() {
        let r = check(&input("XX", 6, 30, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 30);
        assert!(r.notice_compliant);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ny", 24, 90, TerminationIntent::NoCauseTermination));
        assert_eq!(r.required_notice_days, 90);
    }

    #[test]
    fn no_cause_unavailable_states_have_threshold_set() {
        // Invariant: JustCauseAfterTwelveMonths regime → Some(12);
        // StatewideJustCauseAlways → Some(0). Anything else: None.
        for (code, rule) in RULES.iter() {
            match rule.regime {
                NoticeRegime::JustCauseAfterTwelveMonths => {
                    assert_eq!(
                        rule.no_cause_unavailable_after_months,
                        Some(12),
                        "{code} JustCauseAfterTwelveMonths should set threshold to 12"
                    );
                }
                NoticeRegime::StatewideJustCauseAlways => {
                    assert_eq!(
                        rule.no_cause_unavailable_after_months,
                        Some(0),
                        "{code} StatewideJustCauseAlways should set threshold to 0"
                    );
                }
                _ => {
                    assert_eq!(
                        rule.no_cause_unavailable_after_months, None,
                        "{code} non-just-cause regime should not set threshold"
                    );
                }
            }
        }
    }

    #[test]
    fn note_describes_tiered_path() {
        let r = check(&input("NY", 18, 60, TerminationIntent::NoCauseTermination));
        assert!(r.note.contains("30/60/90"));
        assert!(r.note.contains("compliant"));
    }

    #[test]
    fn note_describes_no_cause_unavailable_path() {
        let r = check(&input("CA", 18, 60, TerminationIntent::NoCauseTermination));
        assert!(r.note.contains("No-cause termination unavailable"));
        assert!(r.note.contains("12 months"));
    }
}
