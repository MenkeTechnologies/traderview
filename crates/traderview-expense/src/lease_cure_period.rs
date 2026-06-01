//! Tenant cure period for non-rent lease breach — the statutory
//! window during which a tenant may correct a lease violation
//! and avoid forfeiture.
//!
//! Distinct from rent-payment cure periods (covered by
//! `eviction_notices` and `late_payment_grace_period` siblings).
//! Non-rent breaches include unauthorized pets, parking
//! violations, occupancy-limit breaches, cleanliness/sanitation
//! failures, and lease-clause violations.
//!
//! California — Cal. Code Civ. Proc. § 1161(3): 3-day cure
//! period, EXCLUDING Saturdays, Sundays, and judicial holidays.
//! Effectively 3 business days. NOT applicable to non-payment-
//! of-rent or nuisance cases. Tenant who cures within the
//! window saves the lease from forfeiture. Notice must
//! specifically identify the violation, what must be corrected,
//! and by when.
//!
//! Florida — Fla. Stat. § 83.56(2): TWO statutory tracks:
//!   (a) § 83.56(2)(b) CURABLE — 7-day cure period for
//!       violations such as unauthorized pets, guests, vehicles;
//!       parking violations; cleanliness/sanitation failures.
//!   (b) § 83.56(2)(a) NON-CURABLE — destruction, damage, or
//!       misuse of property by intentional act or continued
//!       unreasonable disturbance gets a 7-day NOTICE TO VACATE
//!       with NO cure right.
//!   12-MONTH RECURRENCE RULE: if curable noncompliance recurs
//!       within 12 months after the initial notice, eviction
//!       may proceed WITHOUT delivering a subsequent notice
//!       (subsequent breach is treated as confirmation rather
//!       than fresh violation).
//!
//! New York — N.Y. RPAPL § 753(4): 10-day cure period for
//! breach of lease covenant in residential leases. Tenant who
//! cures within 10 days may stay possession of the lease. New
//! York City Housing Stability and Tenant Protection Act of
//! 2019 (HSTPA) added 30-day cure period for chronic late-rent
//! and nuisance defenses in housing court (though those are
//! distinct from § 753(4) lease-covenant breach).
//!
//! Default — common law: reasonable time to cure, determined
//! case-by-case based on nature of breach + good-faith effort
//! to remedy. Modern Restatement (Second) of Property § 13.1 +
//! Restatement § 16.1 recognize a tenant's right to a
//! reasonable opportunity to cure absent material breach.
//!
//! Citations: Cal. Code Civ. Proc. § 1161(3) (3-day cure
//! excluding weekends/judicial holidays); Fla. Stat.
//! § 83.56(2)(a) (non-curable 7-day vacate); Fla. Stat.
//! § 83.56(2)(b) (curable 7-day cure + 12-month recurrence
//! rule); N.Y. RPAPL § 753(4) (10-day cure for lease-covenant
//! breach); New York Housing Stability and Tenant Protection Act
//! 2019 (HSTPA) (30-day cure for chronic-late-rent / nuisance
//! defenses); Restatement (Second) of Property § 13.1 (default
//! tenant cure rights); Restatement § 16.1 (forfeiture remedy
//! limits).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Cal. Code Civ. Proc. § 1161(3) — 3-day cure excluding
    /// weekends + judicial holidays.
    California,
    /// Fla. Stat. § 83.56(2) — 7-day cure curable / 7-day vacate
    /// non-curable; 12-month recurrence rule.
    Florida,
    /// N.Y. RPAPL § 753(4) — 10-day cure for lease-covenant
    /// breach.
    NewYork,
    /// Common-law reasonable cure period.
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    /// Curable violation — unauthorized pets, parking,
    /// cleanliness failures, occupancy limits.
    Curable,
    /// Non-curable violation — intentional destruction, damage,
    /// or continued unreasonable disturbance. No cure right.
    NonCurable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub violation_type: ViolationType,
    /// Calendar days since notice was served on tenant.
    pub days_since_notice_served: i64,
    /// California-specific — true if the cure window calculation
    /// should exclude weekends and judicial holidays (default
    /// for residential under § 1161(3)).
    pub weekends_holidays_excluded: bool,
    /// Florida-specific — true if same curable violation has
    /// recurred within 12 months of initial notice. Triggers
    /// § 83.56(2)(b) no-subsequent-notice rule.
    pub recurrence_within_12_months: bool,
    /// Florida-specific — number of business days elapsed if
    /// weekend exclusion is relevant.
    pub business_days_since_notice: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// Applicable statutory cure period in days.
    pub cure_period_days: i64,
    /// True if weekends and judicial holidays are excluded from
    /// the cure period calculation (California only).
    pub weekends_excluded: bool,
    /// True if the tenant retains the right to cure under the
    /// applicable statute + violation type.
    pub tenant_may_cure: bool,
    /// True if the cure window has expired and landlord may
    /// proceed with eviction.
    pub cure_window_expired: bool,
    /// Florida-specific — true if § 83.56(2)(b) recurrence rule
    /// bypasses the second notice requirement.
    pub second_notice_required_on_recurrence: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Cal. Code Civ. Proc. § 1161(3) — 3-day cure.
pub const CALIFORNIA_CURE_DAYS: i64 = 3;
/// Fla. Stat. § 83.56(2)(b) — 7-day cure.
pub const FLORIDA_CURE_DAYS: i64 = 7;
/// N.Y. RPAPL § 753(4) — 10-day cure.
pub const NEW_YORK_CURE_DAYS: i64 = 10;
/// Default common-law reasonable cure period (used as
/// safe-harbor baseline).
pub const DEFAULT_REASONABLE_CURE_DAYS: i64 = 14;
/// Fla. Stat. § 83.56(2)(b) — 12-month recurrence window.
pub const FLORIDA_RECURRENCE_WINDOW_MONTHS: i64 = 12;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let (cure_period_days, weekends_excluded) = match input.regime {
        Regime::California => (CALIFORNIA_CURE_DAYS, true),
        Regime::Florida => (FLORIDA_CURE_DAYS, false),
        Regime::NewYork => (NEW_YORK_CURE_DAYS, false),
        Regime::Default => (DEFAULT_REASONABLE_CURE_DAYS, false),
    };

    let tenant_may_cure = matches!(input.violation_type, ViolationType::Curable);

    let days_for_comparison = if input.regime == Regime::California && input.weekends_holidays_excluded {
        input.business_days_since_notice
    } else {
        input.days_since_notice_served
    };
    let cure_window_expired = days_for_comparison >= cure_period_days;

    let second_notice_required_on_recurrence = match input.regime {
        Regime::Florida => !input.recurrence_within_12_months,
        _ => true,
    };

    // Regime-specific notes.
    match input.regime {
        Regime::California => {
            notes.push(format!(
                "Cal. Code Civ. Proc. § 1161(3) — 3-day cure period EXCLUDING Saturdays, \
                 Sundays, and judicial holidays. Effectively 3 BUSINESS DAYS. Days since \
                 notice (calendar): {}; business days for comparison: {}. Cure window {} \
                 expired.",
                input.days_since_notice_served,
                input.business_days_since_notice,
                if cure_window_expired { "HAS" } else { "has NOT" },
            ));
            if !tenant_may_cure {
                notes.push(
                    "§ 1161(3) does not apply to non-payment-of-rent (use § 1161(2) 3-day \
                     pay-or-quit) or nuisance (use § 1161(4) unlawful detainer for waste). \
                     Pure nuisance or material non-curable breach proceeds without cure \
                     right."
                        .to_string(),
                );
            }
        }
        Regime::Florida => {
            match input.violation_type {
                ViolationType::Curable => {
                    notes.push(format!(
                        "Fla. Stat. § 83.56(2)(b) — 7-day cure for curable noncompliance \
                         (unauthorized pets, guests, vehicles; parking; cleanliness). \
                         Days since notice: {}. Cure window {} expired.",
                        input.days_since_notice_served,
                        if cure_window_expired { "HAS" } else { "has NOT" },
                    ));
                    if input.recurrence_within_12_months {
                        notes.push(
                            "§ 83.56(2)(b) 12-MONTH RECURRENCE RULE engaged — same curable \
                             noncompliance has recurred within 12 months of initial notice. \
                             Landlord may commence eviction WITHOUT delivering subsequent \
                             notice. Tenant's cure right does not reset on recurrence."
                                .to_string(),
                        );
                    }
                }
                ViolationType::NonCurable => {
                    notes.push(format!(
                        "Fla. Stat. § 83.56(2)(a) — NON-CURABLE noncompliance (destruction, \
                         damage, or intentional misuse; continued unreasonable disturbance) \
                         gets a 7-day NOTICE TO VACATE with NO cure right. Tenant must \
                         vacate within 7 days of notice. Days since notice: {}.",
                        input.days_since_notice_served,
                    ));
                }
            }
        }
        Regime::NewYork => {
            notes.push(format!(
                "N.Y. RPAPL § 753(4) — 10-day cure for breach of lease covenant in \
                 residential leases. Days since notice: {}. Cure window {} expired. \
                 Distinct from HSTPA 2019 30-day cure for chronic late-rent/nuisance \
                 defenses.",
                input.days_since_notice_served,
                if cure_window_expired { "HAS" } else { "has NOT" },
            ));
        }
        Regime::Default => {
            notes.push(format!(
                "Common-law reasonable cure period — case-by-case determination based on \
                 nature of breach + good-faith effort to remedy. Restatement (Second) of \
                 Property § 13.1 + § 16.1. Safe-harbor baseline {} days; actual reasonable \
                 period may vary. Days since notice: {}.",
                DEFAULT_REASONABLE_CURE_DAYS,
                input.days_since_notice_served,
            ));
        }
    }

    if tenant_may_cure && cure_window_expired {
        violations.push(format!(
            "Cure window of {} {} expired; landlord may proceed with eviction proceeding.",
            cure_period_days,
            if weekends_excluded { "business days" } else { "calendar days" },
        ));
    } else if !tenant_may_cure {
        violations.push(format!(
            "Non-curable violation under {:?} regime — no cure right. Landlord may \
             proceed with eviction proceeding after applicable notice period.",
            input.regime,
        ));
    }

    notes.push(
        "Sibling distinction: this module covers NON-RENT lease breach cure periods. \
         Rent non-payment cure periods are covered by `eviction_notices` (CA § 1161(2) \
         3-day pay-or-quit, FL § 83.56(3) 3-day, NY HSTPA 14-day rent demand) and \
         `late_payment_grace_period` (grace periods before late fees / cure triggers). \
         California uniquely excludes weekends and judicial holidays from the cure-day \
         count; Florida is the only regime with a 12-month recurrence bypass; New York \
         has the longest baseline cure (10 days)."
            .to_string(),
    );

    CheckResult {
        cure_period_days,
        weekends_excluded,
        tenant_may_cure,
        cure_window_expired,
        second_notice_required_on_recurrence,
        compliant: violations.is_empty(),
        violations,
        citation: "Cal. Code Civ. Proc. § 1161(3) (3-day cure excluding Saturdays, \
                   Sundays, and judicial holidays); Cal. Code Civ. Proc. § 1161(2) \
                   (rent-payment 3-day pay-or-quit — sibling regime); Cal. Code Civ. Proc. \
                   § 1161(4) (nuisance unlawful detainer); Fla. Stat. § 83.56(2)(a) \
                   (non-curable 7-day vacate); Fla. Stat. § 83.56(2)(b) (curable 7-day \
                   cure + 12-month recurrence rule); Fla. Stat. § 83.56(3) (rent \
                   non-payment 3-day cure — sibling regime); N.Y. RPAPL § 753(4) \
                   (10-day cure for lease-covenant breach); N.Y. HSTPA 2019 (30-day \
                   chronic-late-rent / nuisance defense); Restatement (Second) of \
                   Property § 13.1 (default tenant cure rights); Restatement § 16.1 \
                   (forfeiture remedy limits)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        violation_type: ViolationType,
        days_calendar: i64,
        days_business: i64,
    ) -> Input {
        Input {
            regime,
            violation_type,
            days_since_notice_served: days_calendar,
            weekends_holidays_excluded: matches!(regime, Regime::California),
            recurrence_within_12_months: false,
            business_days_since_notice: days_business,
        }
    }

    // ── California § 1161(3) ───────────────────────────────────

    #[test]
    fn california_3_business_days_cure_curable() {
        let r = check(&input(Regime::California, ViolationType::Curable, 5, 3));
        assert_eq!(r.cure_period_days, 3);
        assert!(r.weekends_excluded);
        assert!(r.tenant_may_cure);
        // 3 business days = cure window met (boundary).
        assert!(r.cure_window_expired);
    }

    #[test]
    fn california_2_business_days_still_within_window() {
        let r = check(&input(Regime::California, ViolationType::Curable, 4, 2));
        assert!(!r.cure_window_expired);
        assert!(r.tenant_may_cure);
    }

    #[test]
    fn california_uses_business_days_not_calendar() {
        // 5 calendar days but only 2 business days (weekend in middle).
        let r = check(&input(Regime::California, ViolationType::Curable, 5, 2));
        // Should use business days; 2 < 3 → not expired.
        assert!(!r.cure_window_expired);
    }

    #[test]
    fn california_non_curable_no_cure_right() {
        let r = check(&input(Regime::California, ViolationType::NonCurable, 10, 10));
        assert!(!r.tenant_may_cure);
        assert!(!r.compliant);
    }

    // ── Florida § 83.56(2) ─────────────────────────────────────

    #[test]
    fn florida_7_day_cure_curable() {
        let r = check(&input(Regime::Florida, ViolationType::Curable, 5, 0));
        assert_eq!(r.cure_period_days, 7);
        assert!(!r.weekends_excluded);
        assert!(r.tenant_may_cure);
        assert!(!r.cure_window_expired);
    }

    #[test]
    fn florida_at_7_day_boundary_expired() {
        let r = check(&input(Regime::Florida, ViolationType::Curable, 7, 0));
        assert!(r.cure_window_expired);
    }

    #[test]
    fn florida_non_curable_no_cure_right_7_day_vacate() {
        let r = check(&input(Regime::Florida, ViolationType::NonCurable, 5, 0));
        assert_eq!(r.cure_period_days, 7);
        assert!(!r.tenant_may_cure);
    }

    #[test]
    fn florida_recurrence_within_12_months_no_second_notice() {
        let mut b = input(Regime::Florida, ViolationType::Curable, 5, 0);
        b.recurrence_within_12_months = true;
        let r = check(&b);
        assert!(!r.second_notice_required_on_recurrence);
        assert!(r.notes.iter().any(|n| n.contains("12-MONTH RECURRENCE RULE")));
    }

    #[test]
    fn florida_no_recurrence_requires_second_notice() {
        let r = check(&input(Regime::Florida, ViolationType::Curable, 5, 0));
        assert!(r.second_notice_required_on_recurrence);
    }

    // ── New York RPAPL § 753(4) ────────────────────────────────

    #[test]
    fn new_york_10_day_cure_curable() {
        let r = check(&input(Regime::NewYork, ViolationType::Curable, 7, 0));
        assert_eq!(r.cure_period_days, 10);
        assert!(!r.cure_window_expired);
        assert!(r.tenant_may_cure);
    }

    #[test]
    fn new_york_at_10_day_boundary_expired() {
        let r = check(&input(Regime::NewYork, ViolationType::Curable, 10, 0));
        assert!(r.cure_window_expired);
    }

    #[test]
    fn new_york_9_days_within_window() {
        let r = check(&input(Regime::NewYork, ViolationType::Curable, 9, 0));
        assert!(!r.cure_window_expired);
    }

    // ── Default common-law ─────────────────────────────────────

    #[test]
    fn default_14_day_safe_harbor() {
        let r = check(&input(Regime::Default, ViolationType::Curable, 7, 0));
        assert_eq!(r.cure_period_days, 14);
        assert!(!r.cure_window_expired);
    }

    #[test]
    fn default_at_14_day_boundary_expired() {
        let r = check(&input(Regime::Default, ViolationType::Curable, 14, 0));
        assert!(r.cure_window_expired);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn cure_period_strict_ordering_ca_lt_fl_lt_ny_lt_default_invariant() {
        // Cure periods: CA 3 < FL 7 < NY 10 < Default 14.
        assert!(CALIFORNIA_CURE_DAYS < FLORIDA_CURE_DAYS);
        assert!(FLORIDA_CURE_DAYS < NEW_YORK_CURE_DAYS);
        assert!(NEW_YORK_CURE_DAYS < DEFAULT_REASONABLE_CURE_DAYS);
    }

    #[test]
    fn only_california_excludes_weekends_invariant() {
        for regime in [Regime::California, Regime::Florida, Regime::NewYork, Regime::Default] {
            let r = check(&input(regime, ViolationType::Curable, 0, 0));
            let expected = matches!(regime, Regime::California);
            assert_eq!(r.weekends_excluded, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_florida_has_recurrence_bypass_invariant() {
        for regime in [Regime::California, Regime::Florida, Regime::NewYork, Regime::Default] {
            let mut b = input(regime, ViolationType::Curable, 0, 0);
            b.recurrence_within_12_months = true;
            let r = check(&b);
            // Only Florida should report no second notice required.
            let expected_no_second_notice = matches!(regime, Regime::Florida);
            assert_eq!(
                !r.second_notice_required_on_recurrence,
                expected_no_second_notice,
                "{:?}",
                regime
            );
        }
    }

    #[test]
    fn non_curable_no_cure_right_all_regimes_invariant() {
        for regime in [Regime::California, Regime::Florida, Regime::NewYork, Regime::Default] {
            let r = check(&input(regime, ViolationType::NonCurable, 0, 0));
            assert!(!r.tenant_may_cure, "{:?}", regime);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&input(Regime::California, ViolationType::Curable, 0, 0));
        assert!(r.citation.contains("§ 1161(3)"));
        assert!(r.citation.contains("§ 1161(2)"));
        assert!(r.citation.contains("§ 1161(4)"));
        assert!(r.citation.contains("§ 83.56(2)(a)"));
        assert!(r.citation.contains("§ 83.56(2)(b)"));
        assert!(r.citation.contains("§ 83.56(3)"));
        assert!(r.citation.contains("§ 753(4)"));
        assert!(r.citation.contains("HSTPA"));
        assert!(r.citation.contains("Restatement (Second) of Property § 13.1"));
        assert!(r.citation.contains("§ 16.1"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::California, ViolationType::Curable, 0, 0));
        assert!(
            r.notes.iter().any(|n| n.contains("NON-RENT")
                && n.contains("eviction_notices")
                && n.contains("late_payment_grace_period")),
            "sibling-distinction note must reference non-rent vs rent-payment cure regimes"
        );
    }

    #[test]
    fn defensive_negative_days_no_premature_expiration() {
        let r = check(&input(Regime::California, ViolationType::Curable, -5, -5));
        assert!(!r.cure_window_expired);
    }

    #[test]
    fn cure_window_expired_truth_table_curable() {
        // 4 regimes × 2 day-counts (within vs past) = 8-cell sweep.
        let cells = [
            (Regime::California, 2, 2, false),  // 2 business days within 3
            (Regime::California, 3, 3, true),   // 3 business days = boundary
            (Regime::Florida, 6, 0, false),     // 6 < 7
            (Regime::Florida, 7, 0, true),      // boundary
            (Regime::NewYork, 9, 0, false),     // 9 < 10
            (Regime::NewYork, 10, 0, true),     // boundary
            (Regime::Default, 13, 0, false),    // 13 < 14
            (Regime::Default, 14, 0, true),     // boundary
        ];
        for (regime, days_cal, days_biz, expected_expired) in cells.iter() {
            let r = check(&input(*regime, ViolationType::Curable, *days_cal, *days_biz));
            assert_eq!(
                r.cure_window_expired, *expected_expired,
                "regime={:?} cal={} biz={}",
                regime, days_cal, days_biz
            );
        }
    }
}
