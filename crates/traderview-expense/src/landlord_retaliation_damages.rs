//! Landlord retaliation damages math — how much may a tenant
//! recover when the landlord has retaliated against the tenant's
//! exercise of statutory rights or good-faith complaint?
//!
//! Distinct from sibling module `retaliation_windows` (timing-
//! based rebuttable-presumption windows). This module focuses
//! on the DAMAGES RECOVERY math when retaliation has been found
//! or is alleged with adequate evidence. Trader-critical for
//! landlord-investors operating across multiple regimes — state
//! divergence on damages ranges from common-law actual damages
//! to multi-month rent statutory minimums + punitive damages.
//!
//! California — Cal. Civ. Code § 1942.5(h): Tenant recovers
//! ACTUAL DAMAGES plus PUNITIVE DAMAGES of $100-$2,000 per
//! retaliatory act when fraud, oppression, or malice is shown.
//! Reasonable attorney's fees + costs recoverable. Presumption
//! window: 180 days from protected activity. Punitive amount
//! multiplies by the count of separate retaliatory acts.
//!
//! Massachusetts — G.L. c. 186 § 18: Statutory damages floor +
//! ceiling formula. Damages = MAX(1 month's rent, actual
//! damages); effective ceiling = max(3 months' rent, actual
//! damages). Plus reasonable attorney's fees + costs.
//! Presumption window: 6 months. Rebuttal standard: CLEAR AND
//! CONVINCING evidence (highest civil standard); waiver of
//! § 18 in any lease void and unenforceable.
//!
//! New Jersey — N.J.S.A. 2A:42-10.10: Civil action for damages
//! and injunctive/equitable relief. Statutory presumption of
//! retaliation when reprisal follows tenant's exercise of
//! rights or good-faith complaint. Damages are case-by-case
//! actual damages; landlord may rebut presumption by showing
//! commencement independent of protected activity. Reprisal
//! Law applies to all rental dwellings except owner-occupied
//! premises with ≤ 2 rental units.
//!
//! Default — common-law actual damages. No statutory floor or
//! punitive multiplier; attorney fees recoverable only if lease
//! permits. Most states without explicit retaliation statutes
//! follow this default.
//!
//! Citations: Cal. Civ. Code § 1942.5(h) (actual + $100-$2,000
//! punitive per retaliatory act + attorney fees); Cal. Civ.
//! Code § 1942.5(a) (180-day presumption window); G.L. c. 186
//! § 18 (statutory damages floor 1 month / ceiling 3 months OR
//! actual whichever is greater + attorney fees); G.L. c. 186
//! § 18 (6-month presumption window); G.L. c. 186 § 18 (clear
//! and convincing rebuttal standard); N.J.S.A. 2A:42-10.10
//! (Reprisal Law — civil action for damages + injunctive
//! relief); N.J.S.A. 2A:42-10.11 (retaliation presumption);
//! Restatement (Second) of Property § 14.8 (common-law
//! retaliation rule).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Cal. Civ. Code § 1942.5(h) — actual + $100-$2,000
    /// punitive per act + attorney fees.
    California,
    /// G.L. c. 186 § 18 — max(1-month-rent floor, actual);
    /// ceiling = max(3 months' rent, actual); + attorney fees.
    Massachusetts,
    /// N.J.S.A. 2A:42-10.10 — actual + injunctive relief;
    /// presumption-based.
    NewJersey,
    /// Common-law actual damages.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub monthly_rent_cents: i64,
    pub tenant_actual_damages_cents: i64,
    /// California-specific — true if tenant has shown fraud,
    /// oppression, or malice by landlord (triggers punitive
    /// damages under § 1942.5(h)).
    pub fraud_oppression_or_malice_shown: bool,
    /// California-specific — punitive amount the court awards
    /// per retaliatory act (cents). Must be between $100 and
    /// $2,000; module clamps to range.
    pub punitive_per_act_cents: i64,
    /// California-specific — number of separate retaliatory
    /// acts.
    pub retaliation_acts_count: i64,
    /// True if statutory presumption window engaged (180 days
    /// CA, 6 months MA, NJ case-by-case).
    pub presumption_engaged: bool,
    /// True if landlord successfully rebutted the presumption.
    /// MA requires clear and convincing evidence.
    pub landlord_rebutted_presumption: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// Base statutory damages (cents) — actual + statutory
    /// floor/ceiling math.
    pub statutory_damages_cents: i64,
    /// Punitive damages (cents) — CA-specific.
    pub punitive_damages_cents: i64,
    /// True if attorney's fees recoverable as statutory remedy
    /// in this regime.
    pub attorney_fees_statutory_recoverable: bool,
    /// Total recoverable amount (cents).
    pub total_recoverable_cents: i64,
    /// Presumption window length in days (180 CA, 180 MA = 6
    /// months, varies NJ).
    pub presumption_window_days: i64,
    /// True if MA "clear and convincing" rebuttal standard
    /// applies.
    pub clear_and_convincing_rebuttal_required: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// CA § 1942.5(h) — punitive damages floor (cents).
pub const CA_PUNITIVE_MIN_CENTS: i64 = 10_000;
/// CA § 1942.5(h) — punitive damages ceiling (cents).
pub const CA_PUNITIVE_MAX_CENTS: i64 = 200_000;
/// CA § 1942.5(a) — presumption window (days).
pub const CA_PRESUMPTION_WINDOW_DAYS: i64 = 180;
/// MA c. 186 § 18 — presumption window (days; 6 months).
pub const MA_PRESUMPTION_WINDOW_DAYS: i64 = 180;
/// MA § 18 — statutory damages floor (months of rent).
pub const MA_MIN_MONTHS_RENT: i64 = 1;
/// MA § 18 — statutory damages ceiling (months of rent).
pub const MA_MAX_MONTHS_RENT: i64 = 3;
/// NJ § 2A:42-10.10 — typical presumption window (days; ~90 days).
pub const NJ_PRESUMPTION_WINDOW_DAYS: i64 = 90;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let monthly_rent = input.monthly_rent_cents.max(0);
    let actual_damages = input.tenant_actual_damages_cents.max(0);

    // If landlord successfully rebutted, no damages recoverable.
    if input.landlord_rebutted_presumption {
        notes.push(
            "Landlord has rebutted the retaliation presumption. No retaliation finding; \
             no damages recoverable under this module's regime-specific math. Common-\
             law fallback claims (constructive eviction, intentional infliction) remain \
             available."
                .to_string(),
        );
        return CheckResult {
            statutory_damages_cents: 0,
            punitive_damages_cents: 0,
            attorney_fees_statutory_recoverable: false,
            total_recoverable_cents: 0,
            presumption_window_days: presumption_window_for(input.regime),
            clear_and_convincing_rebuttal_required: matches!(input.regime, Regime::Massachusetts),
            compliant: true,
            violations,
            citation: citation_text(),
            notes,
        };
    }

    let (statutory_damages_cents, punitive_damages_cents, attorney_fees_statutory_recoverable) =
        match input.regime {
            Regime::California => {
                // § 1942.5(h) — actual damages + punitive $100-$2,000 per act.
                let acts = input.retaliation_acts_count.max(0);
                let punitive = if input.fraud_oppression_or_malice_shown && acts > 0 {
                    let per_act = input
                        .punitive_per_act_cents
                        .clamp(CA_PUNITIVE_MIN_CENTS, CA_PUNITIVE_MAX_CENTS);
                    acts.saturating_mul(per_act)
                } else {
                    0
                };
                (actual_damages, punitive, true)
            }
            Regime::Massachusetts => {
                // § 18 — damages = max(1-month-rent floor, actual_damages).
                let one_month = monthly_rent.saturating_mul(MA_MIN_MONTHS_RENT);
                let damages = one_month.max(actual_damages);
                (damages, 0, true)
            }
            Regime::NewJersey => {
                // § 2A:42-10.10 — actual damages + injunctive (case-by-case).
                (actual_damages, 0, true)
            }
            Regime::Default => {
                // Common-law actual damages only. Attorney fees only if lease permits.
                (actual_damages, 0, false)
            }
        };

    let total_recoverable_cents = statutory_damages_cents.saturating_add(punitive_damages_cents);

    if input.presumption_engaged && !input.landlord_rebutted_presumption {
        violations.push(format!(
            "Retaliation finding engaged in {:?} regime — tenant recoverable: {} cents \
             (statutory damages {} + punitive damages {}). Attorney's fees + costs \
             {} recoverable as statutory remedy.",
            input.regime,
            total_recoverable_cents,
            statutory_damages_cents,
            punitive_damages_cents,
            if attorney_fees_statutory_recoverable {
                "ARE"
            } else {
                "ARE NOT"
            },
        ));
    }

    // Regime-specific notes.
    match input.regime {
        Regime::California => {
            notes.push(format!(
                "Cal. Civ. Code § 1942.5(h) — actual damages + PUNITIVE $100-$2,000 per \
                 retaliatory act when fraud/oppression/malice shown. Punitive multiplier \
                 engaged: {} ({} retaliatory acts × {} cents per act = {} cents).",
                input.fraud_oppression_or_malice_shown,
                input.retaliation_acts_count.max(0),
                input
                    .punitive_per_act_cents
                    .clamp(CA_PUNITIVE_MIN_CENTS, CA_PUNITIVE_MAX_CENTS),
                punitive_damages_cents,
            ));
            notes.push(
                "§ 1942.5(a) — 180-day presumption window. Adverse landlord actions \
                 within 180 days of tenant's protected activity create rebuttable \
                 presumption of retaliation; landlord must show legitimate, independent \
                 reason."
                    .to_string(),
            );
        }
        Regime::Massachusetts => {
            let three_months_or_actual = monthly_rent
                .saturating_mul(MA_MAX_MONTHS_RENT)
                .max(actual_damages);
            notes.push(format!(
                "G.L. c. 186 § 18 — damages = MAX(1-month rent floor {} cents, actual \
                 damages {} cents) = {} cents. Effective ceiling = MAX(3 months rent {} \
                 cents, actual damages) = {} cents.",
                monthly_rent.saturating_mul(MA_MIN_MONTHS_RENT),
                actual_damages,
                statutory_damages_cents,
                monthly_rent.saturating_mul(MA_MAX_MONTHS_RENT),
                three_months_or_actual,
            ));
            notes.push(
                "G.L. c. 186 § 18 — 6-month presumption window. Rebuttal standard is \
                 CLEAR AND CONVINCING evidence (highest civil standard); waiver of § 18 \
                 in any lease is void and unenforceable."
                    .to_string(),
            );
        }
        Regime::NewJersey => {
            notes.push(
                "N.J.S.A. 2A:42-10.10 (Reprisal Law) — civil action for damages + \
                 injunctive/equitable relief. Statutory presumption when reprisal \
                 follows protected tenant activity. Landlord may rebut by showing \
                 commencement INDEPENDENT of protected activity. Applies to all rental \
                 dwellings EXCEPT owner-occupied premises with ≤ 2 rental units."
                    .to_string(),
            );
        }
        Regime::Default => {
            notes.push(
                "Common-law actual damages only. No statutory floor or punitive \
                 multiplier in this regime. Attorney's fees recoverable only if lease \
                 expressly permits. Most states without explicit retaliation statutes \
                 follow this default; consider Restatement (Second) of Property § 14.8 \
                 retaliation rule for jurisdiction-specific analysis."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Sibling distinction: this module covers DAMAGES RECOVERY math after retaliation \
         finding. Related modules: `retaliation_windows` (timing-based presumption \
         windows — when adverse action is presumed retaliatory based on proximity to \
         protected activity); `landlord_harassment` (affirmative-harassment civil \
         penalty statutes — CA § 1940.2 + NYC HPA); `lockout_penalties` (lockout-\
         specific damages); `landlord_possession_delivery` (lease-commencement \
         delivery duty + URLTA damages). Massachusetts § 18 has the strongest \
         standard-of-proof (clear and convincing for landlord rebuttal); California \
         § 1942.5(h) has the highest punitive ceiling at $2,000/act with malice \
         showing."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        statutory_damages_cents,
        punitive_damages_cents,
        attorney_fees_statutory_recoverable,
        total_recoverable_cents,
        presumption_window_days: presumption_window_for(input.regime),
        clear_and_convincing_rebuttal_required: matches!(input.regime, Regime::Massachusetts),
        compliant,
        violations,
        citation: citation_text(),
        notes,
    }
}

fn presumption_window_for(regime: Regime) -> i64 {
    match regime {
        Regime::California => CA_PRESUMPTION_WINDOW_DAYS,
        Regime::Massachusetts => MA_PRESUMPTION_WINDOW_DAYS,
        Regime::NewJersey => NJ_PRESUMPTION_WINDOW_DAYS,
        Regime::Default => 0,
    }
}

fn citation_text() -> &'static str {
    "Cal. Civ. Code § 1942.5(h) (actual + $100-$2,000 punitive per retaliatory act + \
     attorney fees); Cal. Civ. Code § 1942.5(a) (180-day presumption window); G.L. \
     c. 186 § 18 (statutory damages 1-month floor / 3-month ceiling OR actual whichever \
     is greater + attorney fees); G.L. c. 186 § 18 (6-month presumption window with \
     clear-and-convincing rebuttal standard); N.J.S.A. 2A:42-10.10 (Reprisal Law — \
     civil action for damages + injunctive relief); N.J.S.A. 2A:42-10.11 (retaliation \
     presumption); Restatement (Second) of Property § 14.8 (common-law retaliation \
     rule)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            monthly_rent_cents: 200_000,          // $2,000/mo
            tenant_actual_damages_cents: 500_000, // $5,000
            fraud_oppression_or_malice_shown: false,
            punitive_per_act_cents: 200_000, // $2,000 per act (max)
            retaliation_acts_count: 1,
            presumption_engaged: true,
            landlord_rebutted_presumption: false,
        }
    }

    // ── California § 1942.5(h) ────────────────────────────────

    #[test]
    fn ca_actual_only_when_no_malice() {
        let r = check(&input(Regime::California));
        // No malice → no punitive. Actual = $5K.
        assert_eq!(r.statutory_damages_cents, 500_000);
        assert_eq!(r.punitive_damages_cents, 0);
        assert_eq!(r.total_recoverable_cents, 500_000);
        assert!(r.attorney_fees_statutory_recoverable);
    }

    #[test]
    fn ca_actual_plus_punitive_when_malice_shown() {
        let mut b = input(Regime::California);
        b.fraud_oppression_or_malice_shown = true;
        let r = check(&b);
        // Actual $5K + 1 act × $2K = $7K.
        assert_eq!(r.statutory_damages_cents, 500_000);
        assert_eq!(r.punitive_damages_cents, 200_000);
        assert_eq!(r.total_recoverable_cents, 700_000);
    }

    #[test]
    fn ca_multiple_retaliatory_acts_multiply_punitive() {
        let mut b = input(Regime::California);
        b.fraud_oppression_or_malice_shown = true;
        b.retaliation_acts_count = 5;
        let r = check(&b);
        // 5 acts × $2K = $10K punitive.
        assert_eq!(r.punitive_damages_cents, 1_000_000);
        assert_eq!(r.total_recoverable_cents, 500_000 + 1_000_000);
    }

    #[test]
    fn ca_punitive_per_act_clamped_to_max() {
        let mut b = input(Regime::California);
        b.fraud_oppression_or_malice_shown = true;
        b.punitive_per_act_cents = 1_000_000; // $10K — way over $2K max
        let r = check(&b);
        // Clamped to $2K.
        assert_eq!(r.punitive_damages_cents, CA_PUNITIVE_MAX_CENTS);
    }

    #[test]
    fn ca_punitive_per_act_clamped_to_min() {
        let mut b = input(Regime::California);
        b.fraud_oppression_or_malice_shown = true;
        b.punitive_per_act_cents = 5_000; // $50 — under $100 min
        let r = check(&b);
        // Clamped to $100.
        assert_eq!(r.punitive_damages_cents, CA_PUNITIVE_MIN_CENTS);
    }

    #[test]
    fn ca_180_day_presumption_window() {
        let r = check(&input(Regime::California));
        assert_eq!(r.presumption_window_days, 180);
    }

    // ── Massachusetts § 18 ───────────────────────────────────

    #[test]
    fn ma_actual_below_one_month_floored_at_one_month() {
        let mut b = input(Regime::Massachusetts);
        b.tenant_actual_damages_cents = 50_000; // $500 — below 1 month rent $2K
        let r = check(&b);
        // Floor = 1 month rent $2K.
        assert_eq!(r.statutory_damages_cents, 200_000);
    }

    #[test]
    fn ma_actual_above_three_months_uses_actual() {
        let mut b = input(Regime::Massachusetts);
        b.tenant_actual_damages_cents = 1_000_000; // $10K — above 3 months $6K
        let r = check(&b);
        // max(1 month $2K, actual $10K) = $10K. Effective ceiling max(3mo, actual) = $10K.
        assert_eq!(r.statutory_damages_cents, 1_000_000);
    }

    #[test]
    fn ma_actual_between_floor_and_three_months_uses_actual() {
        let mut b = input(Regime::Massachusetts);
        b.tenant_actual_damages_cents = 400_000; // $4K — between $2K (1mo) and $6K (3mo)
        let r = check(&b);
        // max(1 month $2K, actual $4K) = $4K.
        assert_eq!(r.statutory_damages_cents, 400_000);
    }

    #[test]
    fn ma_clear_and_convincing_rebuttal_flagged() {
        let r = check(&input(Regime::Massachusetts));
        assert!(r.clear_and_convincing_rebuttal_required);
    }

    #[test]
    fn ma_6_month_presumption_window() {
        let r = check(&input(Regime::Massachusetts));
        assert_eq!(r.presumption_window_days, 180);
    }

    // ── New Jersey § 2A:42-10.10 ─────────────────────────────

    #[test]
    fn nj_actual_damages_only_plus_attorney_fees() {
        let r = check(&input(Regime::NewJersey));
        // Actual $5K + attorney fees statutorily recoverable.
        assert_eq!(r.statutory_damages_cents, 500_000);
        assert_eq!(r.punitive_damages_cents, 0);
        assert!(r.attorney_fees_statutory_recoverable);
    }

    #[test]
    fn nj_90_day_presumption_window() {
        let r = check(&input(Regime::NewJersey));
        assert_eq!(r.presumption_window_days, NJ_PRESUMPTION_WINDOW_DAYS);
    }

    // ── Default common law ───────────────────────────────────

    #[test]
    fn default_actual_only_no_attorney_fees() {
        let r = check(&input(Regime::Default));
        assert_eq!(r.statutory_damages_cents, 500_000);
        // No statutory attorney fees in default regime.
        assert!(!r.attorney_fees_statutory_recoverable);
    }

    #[test]
    fn default_no_presumption_window() {
        let r = check(&input(Regime::Default));
        assert_eq!(r.presumption_window_days, 0);
    }

    // ── Rebuttal — no damages ────────────────────────────────

    #[test]
    fn rebutted_presumption_no_damages_all_regimes() {
        for regime in [
            Regime::California,
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut b = input(regime);
            b.landlord_rebutted_presumption = true;
            let r = check(&b);
            assert_eq!(r.total_recoverable_cents, 0, "{:?}", regime);
            assert!(r.compliant);
        }
    }

    // ── Multi-regime invariants ──────────────────────────────

    #[test]
    fn only_ca_engages_punitive_damages_invariant() {
        for regime in [
            Regime::California,
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut b = input(regime);
            b.fraud_oppression_or_malice_shown = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::California);
            assert_eq!((r.punitive_damages_cents > 0), expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_ma_floors_at_one_month_rent_invariant() {
        for regime in [
            Regime::California,
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut b = input(regime);
            b.tenant_actual_damages_cents = 0;
            let r = check(&b);
            let expected = matches!(regime, Regime::Massachusetts);
            assert_eq!((r.statutory_damages_cents > 0), expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_ma_requires_clear_and_convincing_invariant() {
        for regime in [
            Regime::California,
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let r = check(&input(regime));
            let expected = matches!(regime, Regime::Massachusetts);
            assert_eq!(
                r.clear_and_convincing_rebuttal_required, expected,
                "{:?}",
                regime
            );
        }
    }

    #[test]
    fn statutory_attorney_fees_three_regimes_invariant() {
        // CA, MA, NJ all provide statutory attorney fees; Default does not.
        for regime in [
            Regime::California,
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let r = check(&input(regime));
            let expected = !matches!(regime, Regime::Default);
            assert_eq!(
                r.attorney_fees_statutory_recoverable, expected,
                "{:?}",
                regime
            );
        }
    }

    #[test]
    fn ca_punitive_clamped_truth_table() {
        // 5-cell sweep: input vs clamped output.
        let cells = [
            (5_000, CA_PUNITIVE_MIN_CENTS),     // below min → floor
            (10_000, CA_PUNITIVE_MIN_CENTS),    // exactly min
            (100_000, 100_000),                 // within range
            (200_000, CA_PUNITIVE_MAX_CENTS),   // exactly max
            (1_000_000, CA_PUNITIVE_MAX_CENTS), // above max → ceiling
        ];
        for (input_amount, expected) in cells.iter() {
            let mut b = input(Regime::California);
            b.fraud_oppression_or_malice_shown = true;
            b.punitive_per_act_cents = *input_amount;
            let r = check(&b);
            assert_eq!(r.punitive_damages_cents, *expected);
        }
    }

    #[test]
    fn penalty_constants_invariant() {
        assert_eq!(CA_PUNITIVE_MIN_CENTS, 10_000); // $100
        assert_eq!(CA_PUNITIVE_MAX_CENTS, 200_000); // $2,000
        assert_eq!(CA_PRESUMPTION_WINDOW_DAYS, 180);
        assert_eq!(MA_PRESUMPTION_WINDOW_DAYS, 180); // 6 months
        assert_eq!(MA_MIN_MONTHS_RENT, 1);
        assert_eq!(MA_MAX_MONTHS_RENT, 3);
        assert_eq!(NJ_PRESUMPTION_WINDOW_DAYS, 90);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&input(Regime::California));
        assert!(r.citation.contains("Cal. Civ. Code § 1942.5(h)"));
        assert!(r.citation.contains("Cal. Civ. Code § 1942.5(a)"));
        assert!(r.citation.contains("G.L. c. 186 § 18"));
        assert!(r.citation.contains("N.J.S.A. 2A:42-10.10"));
        assert!(r.citation.contains("N.J.S.A. 2A:42-10.11"));
        assert!(r
            .citation
            .contains("Restatement (Second) of Property § 14.8"));
        assert!(r.citation.contains("clear-and-convincing"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::California));
        assert!(
            r.notes.iter().any(|n| n.contains("retaliation_windows")
                && n.contains("landlord_harassment")
                && n.contains("lockout_penalties")
                && n.contains("landlord_possession_delivery")
                && n.contains("DAMAGES RECOVERY")),
            "sibling distinction note must reference related modules + damages-recovery focus"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_rent_clamped() {
        let mut b = input(Regime::Massachusetts);
        b.monthly_rent_cents = -100_000;
        b.tenant_actual_damages_cents = 0;
        let r = check(&b);
        // Negative rent → 0 floor; actual damages 0 → no recovery.
        assert_eq!(r.statutory_damages_cents, 0);
    }

    #[test]
    fn defensive_negative_actual_damages_clamped() {
        let mut b = input(Regime::California);
        b.tenant_actual_damages_cents = -100_000;
        let r = check(&b);
        // Negative clamped to 0.
        assert_eq!(r.statutory_damages_cents, 0);
    }

    #[test]
    fn defensive_negative_retaliation_acts_clamped() {
        let mut b = input(Regime::California);
        b.fraud_oppression_or_malice_shown = true;
        b.retaliation_acts_count = -5;
        let r = check(&b);
        // Negative clamped to 0 → no punitive.
        assert_eq!(r.punitive_damages_cents, 0);
    }
}
