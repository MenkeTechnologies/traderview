//! Holdover tenant damages — landlord recovery when a tenant
//! remains in possession after lease expiration.
//!
//! Two structurally distinct regimes exist across U.S. landlord-
//! tenant law: (1) STATUTORY DOUBLE RENT recovery where the
//! statute imposes a multiplier on the damages a landlord may
//! recover for the holdover period (Florida); and (2) RENT-
//! ACCEPTANCE month-to-month conversion where post-expiration
//! rent acceptance automatically renews the tenancy on the same
//! terms (California, New York). Default common-law gives the
//! landlord an ELECTION between treating the holdover as a new
//! tenancy at the same rent OR suing for actual damages
//! (Restatement (Second) of Property § 14.5).
//!
//! Florida — Fla. Stat. § 83.58: "If a tenant holds over and
//! continues in possession of the dwelling unit ... after the
//! expiration of the rental agreement without the permission of
//! the landlord, the landlord may recover possession of the
//! dwelling unit ... and the landlord may recover DOUBLE the
//! amount of rent due on the dwelling unit, or any part thereof,
//! for the period during which the tenant refuses to surrender
//! possession." Strict 2× multiplier; partial months count as
//! full periods per the "any part thereof" clause.
//!
//! California — Cal. Civ. Code § 1945: "If a lessee of real
//! property remains in possession thereof after the expiration
//! of the hiring, and the lessor accepts rent from him, the
//! parties are presumed to have renewed the hiring on the same
//! terms and for the same time, not exceeding one month when the
//! rent is payable monthly, nor in any case one year." Rent
//! acceptance creates rebuttable presumption of month-to-month
//! tenancy. No multiplier. Landlord who accepts rent then
//! attempts to evict on the original notice must serve fresh
//! notice — accepting rent waives the original demand for
//! possession.
//!
//! New York — N.Y. Real Prop. Law § 232-c: "Where a tenant whose
//! term is longer than one month holds over after the expiration
//! of such term, such holding over shall not give to the
//! landlord the option to hold the tenant for a new term solely
//! by virtue of the tenant's holding over. ... If the landlord
//! shall accept rent for any period subsequent to the expiration
//! of such term, then, unless an agreement either express or
//! implied is made providing otherwise, the tenancy created by
//! the acceptance of such rent shall be a tenancy from month to
//! month commencing on the first day after the expiration of
//! such term." Same structural rule as CA: rent acceptance →
//! month-to-month conversion. No multiplier.
//!
//! Default — Restatement (Second) of Property § 14.5: landlord
//! has an election between (a) treating the holdover as a new
//! tenancy at the same rent OR (b) suing the tenant for the
//! actual rental value of the premises during the holdover
//! period as damages for trespass / use and occupancy. No
//! statutory multiplier.
//!
//! Citations: Fla. Stat. § 83.58 (double rent); Cal. Civ. Code
//! § 1945 (rent-acceptance presumption); N.Y. Real Prop. Law
//! § 232-c (month-to-month conversion); Restatement (Second) of
//! Property § 14.5 (landlord election at common law).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Fla. Stat. § 83.58 — statutory 2× rent multiplier.
    Florida,
    /// Cal. Civ. Code § 1945 — rent-acceptance month-to-month
    /// rebuttable presumption.
    California,
    /// N.Y. Real Prop. Law § 232-c — rent-acceptance month-to-
    /// month conversion.
    NewYork,
    /// Restatement (Second) of Property § 14.5 — landlord
    /// election between new tenancy or actual-damages trespass.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub monthly_rent_cents: i64,
    pub days_in_holdover: i64,
    pub rent_accepted_post_expiration: bool,
    /// True if landlord has commenced an eviction action /
    /// served notice to quit / otherwise demanded possession.
    pub landlord_demanded_possession: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// Maximum statutory or common-law damages the landlord may
    /// recover for the holdover period (cents).
    pub maximum_recoverable_damages_cents: i64,
    /// True if a month-to-month tenancy has been created by
    /// post-expiration rent acceptance (CA/NY only).
    pub month_to_month_tenancy_created: bool,
    /// Multiplier applied to the rent in the damages
    /// calculation. 2 = Florida; 1 = everywhere else.
    pub damages_multiplier: i64,
    /// Number of partial-or-full months in the holdover period
    /// for damages calculation purposes.
    pub holdover_months: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Days per month used for partial-month → full-month conversion
/// under Fla. Stat. § 83.58 "or any part thereof" language and
/// for consistency across regimes.
pub const DAYS_PER_MONTH: i64 = 30;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let days = input.days_in_holdover.max(0);
    let rent = input.monthly_rent_cents.max(0);

    // Partial months count as full months ("any part thereof"
    // language under Fla. Stat. § 83.58; same convention applied
    // elsewhere for consistency).
    let holdover_months = if days == 0 {
        0
    } else {
        (days + DAYS_PER_MONTH - 1) / DAYS_PER_MONTH
    };

    let multiplier: i64 = match input.regime {
        Regime::Florida => 2,
        Regime::California | Regime::NewYork | Regime::Default => 1,
    };

    let maximum_recoverable_damages_cents =
        rent.saturating_mul(multiplier).saturating_mul(holdover_months);

    let month_to_month_tenancy_created = matches!(
        input.regime,
        Regime::California | Regime::NewYork
    ) && input.rent_accepted_post_expiration;

    // Regime-specific compliance flags.
    match input.regime {
        Regime::California => {
            if input.rent_accepted_post_expiration
                && input.landlord_demanded_possession
            {
                violations.push(
                    "Cal. Civ. Code § 1945 — landlord accepted rent post-expiration AND \
                     attempted to maintain eviction on original demand; rent acceptance \
                     waives the original notice. Landlord must serve fresh notice to \
                     terminate the month-to-month tenancy created by rent acceptance."
                        .to_string(),
                );
            }
        }
        Regime::NewYork => {
            if input.rent_accepted_post_expiration
                && input.landlord_demanded_possession
            {
                violations.push(
                    "N.Y. Real Prop. Law § 232-c — landlord accepted rent post-expiration \
                     AND attempted to maintain holdover proceeding on original notice; \
                     acceptance of rent creates month-to-month tenancy. Landlord must \
                     serve fresh termination notice and commence new proceeding."
                        .to_string(),
                );
            }
        }
        Regime::Florida | Regime::Default => {}
    }

    match input.regime {
        Regime::Florida => {
            notes.push(format!(
                "Fla. Stat. § 83.58 — landlord may recover DOUBLE the rent ({} cents × 2 \
                 multiplier × {} holdover-months = {} cents). Statute applies to the period \
                 the tenant refuses to surrender possession, including any partial period \
                 (\"any part thereof\").",
                rent, holdover_months, maximum_recoverable_damages_cents,
            ));
            if input.rent_accepted_post_expiration {
                notes.push(
                    "Note: Florida case law on rent acceptance is split — some courts treat \
                     acceptance of rent post-expiration as waiver of the holdover claim, \
                     while others permit recovery of double rent for the period before \
                     acceptance. Document each payment carefully."
                        .to_string(),
                );
            }
        }
        Regime::California => {
            if month_to_month_tenancy_created {
                notes.push(format!(
                    "Cal. Civ. Code § 1945 — REBUTTABLE PRESUMPTION OF MONTH-TO-MONTH \
                     TENANCY engaged by post-expiration rent acceptance. Tenancy presumed \
                     renewed on the same terms for one month (rent payable monthly). \
                     Landlord must serve fresh 30/60/90-day termination notice under \
                     Cal. Civ. Code § 1946 / § 1946.1 to end the new tenancy. Damages \
                     for the holdover-before-acceptance period: {} cents.",
                    maximum_recoverable_damages_cents,
                ));
            } else {
                notes.push(format!(
                    "Cal. Civ. Code § 1945 — no rent accepted post-expiration; no \
                     month-to-month presumption engaged. Landlord may recover actual \
                     rental damages: {} cents for {} months. Pursue unlawful-detainer \
                     under Cal. Code Civ. Proc. § 1161.",
                    maximum_recoverable_damages_cents, holdover_months,
                ));
            }
        }
        Regime::NewYork => {
            if month_to_month_tenancy_created {
                notes.push(format!(
                    "N.Y. Real Prop. Law § 232-c — MONTH-TO-MONTH TENANCY CREATED by \
                     post-expiration rent acceptance. Tenancy commences on the first day \
                     after lease expiration. Landlord must serve fresh termination notice \
                     under N.Y. Real Prop. Law § 232-a (NYC) or § 232-b (outside NYC). \
                     Damages for pre-acceptance period: {} cents.",
                    maximum_recoverable_damages_cents,
                ));
            } else {
                notes.push(format!(
                    "N.Y. Real Prop. Law § 232-c — no rent accepted post-expiration; \
                     month-to-month conversion not triggered. Landlord may pursue \
                     summary holdover proceeding under N.Y. RPAPL § 711(1) and recover \
                     use-and-occupancy damages: {} cents for {} months.",
                    maximum_recoverable_damages_cents, holdover_months,
                ));
            }
        }
        Regime::Default => {
            notes.push(format!(
                "Restatement (Second) of Property § 14.5 — landlord ELECTION between (a) \
                 treating holdover as new tenancy at same terms (rent-acceptance theory) \
                 OR (b) suing for use-and-occupancy / trespass damages for actual rental \
                 value. No statutory multiplier; landlord may recover {} cents (1× rent \
                 × {} holdover-months) plus any provable consequential damages.",
                maximum_recoverable_damages_cents, holdover_months,
            ));
        }
    }

    notes.push(
        "Sibling distinction: STATUTORY-DOUBLE-RENT regimes (FL) impose a damages \
         multiplier on the holdover period; RENT-ACCEPTANCE regimes (CA + NY) convert \
         the tenancy to month-to-month on rent acceptance with no multiplier. Default \
         common-law (Restatement § 14.5) gives the landlord an election between the \
         two outcomes. Crucially, the FL multiplier applies regardless of rent \
         acceptance (subject to split-authority waiver concerns), while CA + NY \
         conversion is mandatory ONLY upon rent acceptance."
            .to_string(),
    );

    CheckResult {
        maximum_recoverable_damages_cents,
        month_to_month_tenancy_created,
        damages_multiplier: multiplier,
        holdover_months,
        compliant: violations.is_empty(),
        violations,
        citation: "Fla. Stat. § 83.58 (double rent — \"any part thereof\" partial-month \
                   rule); Cal. Civ. Code § 1945 (rent-acceptance presumption — same terms, \
                   one-month maximum); Cal. Civ. Code § 1946 + § 1946.1 (termination of \
                   periodic tenancy); N.Y. Real Prop. Law § 232-c (post-expiration \
                   month-to-month conversion); N.Y. Real Prop. Law § 232-a / § 232-b \
                   (termination notice for month-to-month NYC / outside NYC); \
                   N.Y. RPAPL § 711(1) (summary holdover proceeding); Restatement \
                   (Second) of Property § 14.5 (landlord election at common law); \
                   Cal. Code Civ. Proc. § 1161 (California unlawful-detainer)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        rent: i64,
        days: i64,
        accepted: bool,
        demanded: bool,
    ) -> Input {
        Input {
            regime,
            monthly_rent_cents: rent,
            days_in_holdover: days,
            rent_accepted_post_expiration: accepted,
            landlord_demanded_possession: demanded,
        }
    }

    // ── Florida: statutory 2× rent multiplier ──────────────────

    #[test]
    fn florida_30_day_holdover_full_month_double_rent() {
        let r = check(&input(Regime::Florida, 200_000, 30, false, true));
        assert_eq!(r.holdover_months, 1);
        assert_eq!(r.damages_multiplier, 2);
        assert_eq!(r.maximum_recoverable_damages_cents, 400_000);
    }

    #[test]
    fn florida_31_day_holdover_rounds_up_to_2_months_double_rent() {
        let r = check(&input(Regime::Florida, 200_000, 31, false, true));
        assert_eq!(r.holdover_months, 2);
        assert_eq!(r.maximum_recoverable_damages_cents, 800_000);
    }

    #[test]
    fn florida_1_day_partial_month_full_charge_per_any_part_thereof() {
        let r = check(&input(Regime::Florida, 200_000, 1, false, true));
        assert_eq!(r.holdover_months, 1);
        assert_eq!(r.maximum_recoverable_damages_cents, 400_000);
    }

    #[test]
    fn florida_60_day_holdover_2_months_double_rent() {
        let r = check(&input(Regime::Florida, 200_000, 60, false, true));
        assert_eq!(r.holdover_months, 2);
        assert_eq!(r.maximum_recoverable_damages_cents, 800_000);
    }

    #[test]
    fn florida_zero_days_zero_damages() {
        let r = check(&input(Regime::Florida, 200_000, 0, false, true));
        assert_eq!(r.holdover_months, 0);
        assert_eq!(r.maximum_recoverable_damages_cents, 0);
    }

    #[test]
    fn florida_365_day_extreme_holdover_double_rent() {
        let r = check(&input(Regime::Florida, 200_000, 365, false, true));
        // 365 / 30 = 12.166 → ceil = 13 months
        assert_eq!(r.holdover_months, 13);
        assert_eq!(r.maximum_recoverable_damages_cents, 5_200_000);
    }

    #[test]
    fn florida_rent_acceptance_does_not_create_month_to_month() {
        let r = check(&input(Regime::Florida, 200_000, 30, true, true));
        // FL does NOT have a § 1945/§ 232-c style conversion rule.
        assert!(!r.month_to_month_tenancy_created);
    }

    // ── California: § 1945 rent-acceptance presumption ─────────

    #[test]
    fn california_rent_accepted_creates_month_to_month() {
        let r = check(&input(Regime::California, 200_000, 30, true, false));
        assert!(r.month_to_month_tenancy_created);
        assert_eq!(r.damages_multiplier, 1);
    }

    #[test]
    fn california_no_rent_accepted_no_month_to_month() {
        let r = check(&input(Regime::California, 200_000, 30, false, true));
        assert!(!r.month_to_month_tenancy_created);
        assert_eq!(r.maximum_recoverable_damages_cents, 200_000);
        assert!(r.compliant);
    }

    #[test]
    fn california_rent_accepted_plus_eviction_attempt_violation() {
        let r = check(&input(Regime::California, 200_000, 30, true, true));
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("Cal. Civ. Code § 1945")));
        assert!(r.violations.iter().any(|v| v.contains("waives the original notice")));
    }

    // ── New York: § 232-c rent-acceptance conversion ───────────

    #[test]
    fn new_york_rent_accepted_creates_month_to_month() {
        let r = check(&input(Regime::NewYork, 300_000, 30, true, false));
        assert!(r.month_to_month_tenancy_created);
        assert_eq!(r.damages_multiplier, 1);
    }

    #[test]
    fn new_york_no_rent_accepted_no_month_to_month() {
        let r = check(&input(Regime::NewYork, 300_000, 30, false, true));
        assert!(!r.month_to_month_tenancy_created);
        assert_eq!(r.maximum_recoverable_damages_cents, 300_000);
    }

    #[test]
    fn new_york_rent_accepted_plus_holdover_proceeding_violation() {
        let r = check(&input(Regime::NewYork, 300_000, 30, true, true));
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("N.Y. Real Prop. Law § 232-c")));
    }

    // ── Default common-law: § 14.5 landlord election ───────────

    #[test]
    fn default_regime_single_rent_recovery() {
        let r = check(&input(Regime::Default, 200_000, 30, false, true));
        assert_eq!(r.damages_multiplier, 1);
        assert_eq!(r.maximum_recoverable_damages_cents, 200_000);
    }

    #[test]
    fn default_regime_no_month_to_month_auto_creation() {
        let r = check(&input(Regime::Default, 200_000, 30, true, false));
        // Default common-law does NOT auto-convert; § 1945 / § 232-c are statutory.
        assert!(!r.month_to_month_tenancy_created);
    }

    // ── Multi-regime invariants ────────────────────────────────

    #[test]
    fn florida_is_the_only_2x_multiplier_regime_invariant() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let r = check(&input(regime, 200_000, 30, false, true));
            assert_eq!(r.damages_multiplier, 1, "{:?}: must be 1× multiplier", regime);
        }
        let fl = check(&input(Regime::Florida, 200_000, 30, false, true));
        assert_eq!(fl.damages_multiplier, 2);
    }

    #[test]
    fn rent_acceptance_creates_month_to_month_only_for_ca_and_ny_invariant() {
        let mut creates_mtm: Vec<Regime> = Vec::new();
        for regime in [
            Regime::Florida,
            Regime::California,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&input(regime, 200_000, 30, true, false));
            if r.month_to_month_tenancy_created {
                creates_mtm.push(regime);
            }
        }
        assert_eq!(creates_mtm.len(), 2);
        assert!(creates_mtm.contains(&Regime::California));
        assert!(creates_mtm.contains(&Regime::NewYork));
    }

    #[test]
    fn same_input_florida_double_vs_others_single_damages_invariant() {
        let rent = 200_000;
        let days = 30;
        let fl = check(&input(Regime::Florida, rent, days, false, true));
        let ca = check(&input(Regime::California, rent, days, false, true));
        let ny = check(&input(Regime::NewYork, rent, days, false, true));
        let de = check(&input(Regime::Default, rent, days, false, true));

        assert_eq!(fl.maximum_recoverable_damages_cents, 2 * rent);
        assert_eq!(ca.maximum_recoverable_damages_cents, rent);
        assert_eq!(ny.maximum_recoverable_damages_cents, rent);
        assert_eq!(de.maximum_recoverable_damages_cents, rent);
        // FL exactly 2× of the others.
        assert_eq!(
            fl.maximum_recoverable_damages_cents,
            2 * ca.maximum_recoverable_damages_cents
        );
    }

    #[test]
    fn zero_days_zero_damages_all_regimes_invariant() {
        for regime in [
            Regime::Florida,
            Regime::California,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&input(regime, 200_000, 0, false, false));
            assert_eq!(r.holdover_months, 0);
            assert_eq!(r.maximum_recoverable_damages_cents, 0);
        }
    }

    #[test]
    fn citation_pins_all_regime_authorities() {
        let r = check(&input(Regime::Florida, 200_000, 30, false, true));
        assert!(r.citation.contains("Fla. Stat. § 83.58"));
        assert!(r.citation.contains("Cal. Civ. Code § 1945"));
        assert!(r.citation.contains("N.Y. Real Prop. Law § 232-c"));
        assert!(r.citation.contains("Restatement (Second) of Property § 14.5"));
        assert!(r.citation.contains("§ 1946")); // California termination
        assert!(r.citation.contains("RPAPL § 711(1)")); // NY summary proceeding
        assert!(r.citation.contains("§ 1161")); // California unlawful detainer
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::Florida, 200_000, 30, false, true));
        assert!(
            r.notes.iter().any(|n| n.contains("STATUTORY-DOUBLE-RENT regimes")
                && n.contains("RENT-ACCEPTANCE regimes")
                && n.contains("§ 14.5")),
            "sibling-module distinction note must be present"
        );
    }

    #[test]
    fn defensive_negative_days_clamped_to_zero() {
        let r = check(&input(Regime::Florida, 200_000, -5, false, true));
        assert_eq!(r.holdover_months, 0);
        assert_eq!(r.maximum_recoverable_damages_cents, 0);
    }

    #[test]
    fn defensive_negative_rent_clamped_to_zero() {
        let r = check(&input(Regime::Florida, -200_000, 30, false, true));
        assert_eq!(r.maximum_recoverable_damages_cents, 0);
    }

    #[test]
    fn florida_29_days_one_month_boundary() {
        let r = check(&input(Regime::Florida, 100_000, 29, false, true));
        assert_eq!(r.holdover_months, 1);
        assert_eq!(r.maximum_recoverable_damages_cents, 200_000);
    }

    #[test]
    fn single_state_uniqueness_invariant() {
        // Each enum variant produces a distinct citation / damages
        // characteristic combination — 4-cell truth table.
        let mut seen: Vec<(i64, bool)> = Vec::new();
        for regime in [
            Regime::Florida,
            Regime::California,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&input(regime, 200_000, 30, true, false));
            let key = (r.damages_multiplier, r.month_to_month_tenancy_created);
            seen.push(key);
        }
        // FL: (2, false), CA: (1, true), NY: (1, true), Default: (1, false)
        assert_eq!(seen[0], (2, false), "Florida");
        assert_eq!(seen[1], (1, true), "California");
        assert_eq!(seen[2], (1, true), "New York");
        assert_eq!(seen[3], (1, false), "Default");
    }
}
