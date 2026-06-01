//! IRC § 7704 — Treatment of publicly traded partnerships as
//! corporations.
//!
//! Trader-critical for any investor holding master limited
//! partnerships (MLPs) or other publicly traded partnerships
//! (PTPs). § 7704(a) treats a PTP as a CORPORATION for federal
//! tax purposes — losing pass-through status and triggering
//! double taxation — UNLESS the partnership satisfies the
//! § 7704(c) qualifying-income exception (90% of gross income
//! must come from passive-type sources).
//!
//! § 7704(b) PTP definition has two prongs:
//!
//!   (1) interests in the partnership are traded on an established
//!       securities market, OR
//!   (2) interests are readily tradable on a secondary market or
//!       the substantial equivalent thereof.
//!
//! Either prong makes the partnership a PTP.
//!
//! § 7704(c) exception applies ONLY where the partnership:
//!
//!   (c)(1) — met the gross-income requirements of (c)(2) for the
//!     current taxable year AND each preceding taxable year
//!     beginning after December 31, 1987 (continuous-compliance
//!     requirement).
//!
//!   (c)(2) — 90% or more of the partnership's gross income for
//!     the taxable year consists of qualifying income as defined
//!     in § 7704(d).
//!
//! § 7704(d) qualifying-income categories (seven):
//!
//!   (A) interest
//!   (B) dividends
//!   (C) real property rents
//!   (D) gain from the sale or disposition of real property
//!   (E) income and gains derived from the exploration, development,
//!       mining, processing, refining, transportation, or marketing
//!       of any mineral or natural resource (oil, gas, minerals,
//!       timber, fertilizer, geothermal, industrial source CO2,
//!       alcohol fuels)
//!   (F) gain from sale or disposition of capital asset held for
//!       qualifying-income production
//!   (G) in the case of certain commodities partnerships, income
//!       and gains from commodities + § 1221(a)(1) inventory
//!
//! § 7704(e) inadvertent-termination relief — Secretary may treat
//! the partnership as continuing to meet the gross-income test if
//! (i) failure was INADVERTENT, (ii) within reasonable time after
//! discovery the partnership takes CORRECTIVE STEPS, and (iii) the
//! partnership AGREES to make adjustments required by the Secretary.
//!
//! Citations: 26 U.S.C. § 7704(a) (general rule — PTP treated as
//! corporation); § 7704(b)(1) (established securities market) +
//! § 7704(b)(2) (readily tradable secondary market) (PTP
//! definition); § 7704(c)(1) (continuous-compliance requirement
//! since 1987-12-31); § 7704(c)(2) (90% qualifying income test);
//! § 7704(d)(1)(A)–(G) (seven qualifying-income categories);
//! § 7704(e) (inadvertent-termination relief).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TradingStatus {
    /// § 7704(b)(1) — interests traded on an established securities
    /// market (NYSE, NASDAQ, etc.).
    EstablishedSecuritiesMarket,
    /// § 7704(b)(2) — interests readily tradable on a secondary
    /// market or the substantial equivalent thereof.
    ReadilyTradableSecondaryMarket,
    /// Outside § 7704 scope — partnership interests not publicly
    /// traded.
    NotPubliclyTraded,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7704Input {
    pub trading_status: TradingStatus,
    /// Total gross income of the partnership for the taxable year
    /// (cents).
    pub gross_income_total_cents: i64,
    /// Sum of all qualifying-income categories under
    /// § 7704(d)(1)(A)–(G) for the taxable year (cents). Caller
    /// computes this from the seven category-specific income
    /// streams.
    pub qualifying_income_cents: i64,
    /// Whether the partnership has met the § 7704(c)(2) 90% test
    /// for EVERY taxable year beginning after 1987-12-31 (continuous-
    /// compliance requirement under § 7704(c)(1)).
    pub met_test_every_year_since_1988: bool,
    /// Whether the failure to meet the gross-income test was
    /// inadvertent (§ 7704(e) prong 1).
    pub inadvertent_failure: bool,
    /// Whether the partnership took CORRECTIVE STEPS within a
    /// reasonable time after discovery (§ 7704(e) prong 2).
    pub corrective_steps_taken: bool,
    /// Whether the partnership agreed to the adjustments required
    /// by the Secretary (§ 7704(e) prong 3).
    pub agreement_to_required_adjustments: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7704Result {
    /// True if the partnership is a PTP under § 7704(b).
    pub is_publicly_traded_partnership: bool,
    /// Qualifying-income percentage in basis points × 100 (e.g.,
    /// 9050 = 90.50%).
    pub qualifying_income_percentage_bp: u32,
    /// True if qualifying income ≥ 90% of gross income.
    pub meets_90_pct_test: bool,
    /// Whether the partnership has met the continuous-compliance
    /// requirement under § 7704(c)(1).
    pub continuous_compliance_satisfied: bool,
    /// Whether § 7704(e) inadvertent-termination relief applies
    /// (all three prongs satisfied).
    pub inadvertent_termination_relief_applies: bool,
    /// Final result: true if § 7704(a) general rule treats the
    /// partnership as a corporation; false if the § 7704(c)
    /// exception (with or without § 7704(e) relief) preserves
    /// pass-through status.
    pub treated_as_corporation: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section7704Input) -> Section7704Result {
    let mut notes: Vec<String> = Vec::new();

    let is_ptp = matches!(
        input.trading_status,
        TradingStatus::EstablishedSecuritiesMarket
            | TradingStatus::ReadilyTradableSecondaryMarket,
    );

    // Compute qualifying-income percentage in basis points × 100.
    let qi_bp = if input.gross_income_total_cents > 0 {
        let qi = input.qualifying_income_cents.max(0);
        let pct = qi.saturating_mul(10_000) / input.gross_income_total_cents;
        pct.clamp(0, 10_000) as u32
    } else {
        0
    };

    let meets_90_pct = qi_bp >= 9_000;
    let continuous_compliance = input.met_test_every_year_since_1988;

    let inadvertent_relief = input.inadvertent_failure
        && input.corrective_steps_taken
        && input.agreement_to_required_adjustments;

    // Not a PTP → § 7704 inapplicable; partnership keeps
    // pass-through status.
    if !is_ptp {
        notes.push(
            "§ 7704(b) — partnership interests are NOT publicly traded; § 7704 does not apply. \
             Partnership retains pass-through status under subchapter K."
                .to_string(),
        );
        return Section7704Result {
            is_publicly_traded_partnership: false,
            qualifying_income_percentage_bp: qi_bp,
            meets_90_pct_test: meets_90_pct,
            continuous_compliance_satisfied: continuous_compliance,
            inadvertent_termination_relief_applies: false,
            treated_as_corporation: false,
            citation: "26 U.S.C. § 7704(b) (publicly traded partnership definition — not \
                       satisfied); subchapter K pass-through treatment applies",
            notes,
        };
    }

    // PTP. Check § 7704(c) exception.
    let exception_satisfied = meets_90_pct && continuous_compliance;

    // § 7704(e) inadvertent-termination relief — applies only when
    // the 90% test failed for the current year (or continuous-
    // compliance broke) but all three prongs are satisfied.
    let relief_engages = !exception_satisfied && inadvertent_relief;

    let treated_as_corporation = !(exception_satisfied || relief_engages);

    // Notes.
    match input.trading_status {
        TradingStatus::EstablishedSecuritiesMarket => {
            notes.push(
                "§ 7704(b)(1) — interests traded on an established securities market (NYSE, \
                 NASDAQ, etc.); partnership is a PTP."
                    .to_string(),
            );
        }
        TradingStatus::ReadilyTradableSecondaryMarket => {
            notes.push(
                "§ 7704(b)(2) — interests readily tradable on a secondary market or the \
                 substantial equivalent thereof; partnership is a PTP."
                    .to_string(),
            );
        }
        TradingStatus::NotPubliclyTraded => unreachable!(),
    }

    if exception_satisfied {
        notes.push(format!(
            "§ 7704(c) exception SATISFIED — qualifying income {} basis points ({}%) ≥ 90%; \
             continuous compliance maintained since 1987-12-31. Partnership retains pass-\
             through status.",
            qi_bp,
            qi_bp as f64 / 100.0,
        ));
    } else if relief_engages {
        notes.push(
            "§ 7704(e) inadvertent-termination relief ENGAGES — failure was inadvertent, \
             corrective steps taken within reasonable time, and partnership agreed to required \
             adjustments. Pass-through status preserved."
                .to_string(),
        );
    } else if !meets_90_pct {
        notes.push(format!(
            "§ 7704(c)(2) 90% test FAILED — qualifying income {} basis points ({}%) < 90% \
             threshold. Without § 7704(e) relief, partnership treated as corporation under \
             § 7704(a).",
            qi_bp,
            qi_bp as f64 / 100.0,
        ));
    } else if !continuous_compliance {
        notes.push(
            "§ 7704(c)(1) continuous-compliance requirement FAILED — partnership did not meet \
             the 90% test in at least one prior taxable year beginning after 1987-12-31. \
             Without § 7704(e) relief, partnership treated as corporation under § 7704(a)."
                .to_string(),
        );
    }

    let citation = if treated_as_corporation {
        "26 U.S.C. § 7704(a) (general rule — PTP treated as corporation); § 7704(b)(1) or \
         § 7704(b)(2) (PTP definition); § 7704(c)(2) (90% qualifying-income test FAILED or \
         § 7704(c)(1) continuous-compliance FAILED); § 7704(e) (inadvertent-termination relief \
         not available)"
    } else if relief_engages {
        "26 U.S.C. § 7704(e) (inadvertent-termination relief — all three prongs satisfied: \
         (i) inadvertent failure, (ii) corrective steps within reasonable time, (iii) \
         agreement to required adjustments); § 7704(b) (PTP definition); subchapter K \
         pass-through preserved"
    } else {
        "26 U.S.C. § 7704(c)(1)–(c)(2) (passive-income exception — 90% qualifying-income test \
         + continuous compliance since 1987-12-31 BOTH satisfied); § 7704(d)(1)(A)–(G) \
         (qualifying-income categories — interest + dividends + real property rents + real \
         property gain + mineral/natural resource income + qualifying capital asset gain + \
         commodities income); § 7704(b) (PTP definition); subchapter K pass-through preserved"
    };

    Section7704Result {
        is_publicly_traded_partnership: true,
        qualifying_income_percentage_bp: qi_bp,
        meets_90_pct_test: meets_90_pct,
        continuous_compliance_satisfied: continuous_compliance,
        inadvertent_termination_relief_applies: relief_engages,
        treated_as_corporation,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        status: TradingStatus,
        gross: i64,
        qi: i64,
        continuous: bool,
        inadvertent: bool,
        corrective: bool,
        agreement: bool,
    ) -> Section7704Input {
        Section7704Input {
            trading_status: status,
            gross_income_total_cents: gross,
            qualifying_income_cents: qi,
            met_test_every_year_since_1988: continuous,
            inadvertent_failure: inadvertent,
            corrective_steps_taken: corrective,
            agreement_to_required_adjustments: agreement,
        }
    }

    // ── § 7704(b) PTP definition ────────────────────────────────

    #[test]
    fn not_publicly_traded_section_7704_inapplicable() {
        let r = compute(&input(
            TradingStatus::NotPubliclyTraded,
            100_000,
            50_000,
            true,
            false,
            false,
            false,
        ));
        assert!(!r.is_publicly_traded_partnership);
        assert!(!r.treated_as_corporation);
        assert!(r.citation.contains("§ 7704(b)"));
        assert!(r.citation.contains("subchapter K"));
    }

    #[test]
    fn established_securities_market_is_ptp() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            95_000,
            true,
            false,
            false,
            false,
        ));
        assert!(r.is_publicly_traded_partnership);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 7704(b)(1)") && n.contains("established"))
        );
    }

    #[test]
    fn readily_tradable_secondary_market_is_ptp() {
        let r = compute(&input(
            TradingStatus::ReadilyTradableSecondaryMarket,
            100_000,
            95_000,
            true,
            false,
            false,
            false,
        ));
        assert!(r.is_publicly_traded_partnership);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 7704(b)(2)") && n.contains("secondary market"))
        );
    }

    // ── § 7704(c)(2) 90% qualifying-income test ─────────────────

    #[test]
    fn ptp_with_95_pct_qualifying_income_passes_test() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            95_000,
            true,
            false,
            false,
            false,
        ));
        assert_eq!(r.qualifying_income_percentage_bp, 9_500);
        assert!(r.meets_90_pct_test);
        assert!(!r.treated_as_corporation);
        assert!(r.citation.contains("§ 7704(c)(1)–(c)(2)"));
        assert!(r.citation.contains("§ 7704(d)(1)(A)–(G)"));
    }

    #[test]
    fn ptp_at_exact_90_pct_boundary_passes_test() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            90_000,
            true,
            false,
            false,
            false,
        ));
        assert_eq!(r.qualifying_income_percentage_bp, 9_000);
        assert!(r.meets_90_pct_test);
        assert!(!r.treated_as_corporation);
    }

    #[test]
    fn ptp_just_below_90_pct_fails_test() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            89_999,
            true,
            false,
            false,
            false,
        ));
        assert_eq!(r.qualifying_income_percentage_bp, 8_999);
        assert!(!r.meets_90_pct_test);
        assert!(r.treated_as_corporation);
        assert!(r.citation.contains("§ 7704(a)"));
        assert!(r.citation.contains("FAILED"));
    }

    #[test]
    fn ptp_with_50_pct_qualifying_income_fails_test() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            50_000,
            true,
            false,
            false,
            false,
        ));
        assert_eq!(r.qualifying_income_percentage_bp, 5_000);
        assert!(!r.meets_90_pct_test);
        assert!(r.treated_as_corporation);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 7704(c)(2)") && n.contains("FAILED"))
        );
    }

    // ── § 7704(c)(1) continuous-compliance requirement ─────────

    #[test]
    fn ptp_passes_current_test_fails_continuous_compliance() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            95_000,
            false, // failed in prior year
            false,
            false,
            false,
        ));
        assert!(r.meets_90_pct_test);
        assert!(!r.continuous_compliance_satisfied);
        assert!(r.treated_as_corporation);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 7704(c)(1)") && n.contains("continuous-compliance"))
        );
    }

    // ── § 7704(e) inadvertent-termination relief ───────────────

    #[test]
    fn ptp_failed_test_but_all_3_relief_prongs_satisfied_pass_through_preserved() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            85_000, // fails 90% test
            true,
            true,  // inadvertent
            true,  // corrective steps
            true,  // agreement to adjustments
        ));
        assert!(!r.meets_90_pct_test);
        assert!(r.inadvertent_termination_relief_applies);
        assert!(!r.treated_as_corporation);
        assert!(r.citation.contains("§ 7704(e)"));
    }

    #[test]
    fn ptp_relief_requires_all_three_prongs_inadvertent() {
        let mut i = input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            85_000,
            true,
            false, // NOT inadvertent
            true,
            true,
        );
        let r = compute(&i);
        assert!(!r.inadvertent_termination_relief_applies);
        assert!(r.treated_as_corporation);

        i.inadvertent_failure = true;
        i.corrective_steps_taken = false; // No corrective steps
        let r2 = compute(&i);
        assert!(!r2.inadvertent_termination_relief_applies);

        i.corrective_steps_taken = true;
        i.agreement_to_required_adjustments = false; // No agreement
        let r3 = compute(&i);
        assert!(!r3.inadvertent_termination_relief_applies);
    }

    #[test]
    fn relief_does_not_engage_when_test_already_satisfied() {
        // When the test passes, § 7704(c) exception applies; § 7704(e)
        // relief doesn't engage (no failure to cure).
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            95_000,
            true,
            true,
            true,
            true,
        ));
        assert!(r.meets_90_pct_test);
        assert!(!r.inadvertent_termination_relief_applies);
        assert!(!r.treated_as_corporation);
    }

    // ── Edge cases ─────────────────────────────────────────────

    #[test]
    fn zero_gross_income_zero_qi_percentage() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            0,
            0,
            true,
            false,
            false,
            false,
        ));
        assert_eq!(r.qualifying_income_percentage_bp, 0);
        assert!(!r.meets_90_pct_test);
        assert!(r.treated_as_corporation);
    }

    #[test]
    fn qi_exceeds_gross_clamps_at_100_pct() {
        // Defensive — caller error: QI > gross.
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            150_000,
            true,
            false,
            false,
            false,
        ));
        assert_eq!(r.qualifying_income_percentage_bp, 10_000);
        assert!(r.meets_90_pct_test);
    }

    #[test]
    fn negative_qi_clamps_at_zero() {
        let r = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            -1_000,
            true,
            false,
            false,
            false,
        ));
        assert_eq!(r.qualifying_income_percentage_bp, 0);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn ptp_status_independent_of_trading_market_invariant() {
        for status in [
            TradingStatus::EstablishedSecuritiesMarket,
            TradingStatus::ReadilyTradableSecondaryMarket,
        ] {
            let r = compute(&input(
                status,
                100_000,
                95_000,
                true,
                false,
                false,
                false,
            ));
            assert!(
                r.is_publicly_traded_partnership,
                "{:?}: must be PTP",
                status,
            );
        }
        let r = compute(&input(
            TradingStatus::NotPubliclyTraded,
            100_000,
            95_000,
            true,
            false,
            false,
            false,
        ));
        assert!(!r.is_publicly_traded_partnership);
    }

    #[test]
    fn exception_requires_both_90_pct_and_continuous_compliance_invariant() {
        for (qi, continuous, expected_corp) in [
            (95_000_i64, true, false), // both pass → exception applies
            (95_000, false, true),     // continuous fails → corporation
            (85_000, true, true),      // 90% fails → corporation
            (85_000, false, true),     // both fail → corporation
        ] {
            let r = compute(&input(
                TradingStatus::EstablishedSecuritiesMarket,
                100_000,
                qi,
                continuous,
                false,
                false,
                false,
            ));
            assert_eq!(
                r.treated_as_corporation, expected_corp,
                "qi={} continuous={} expected_corp={}",
                qi, continuous, expected_corp,
            );
        }
    }

    #[test]
    fn inadvertent_relief_requires_all_3_prongs_8_combo_invariant() {
        // 8 combinations of (inadvertent, corrective, agreement);
        // only (true, true, true) yields relief.
        for inadvertent in [false, true] {
            for corrective in [false, true] {
                for agreement in [false, true] {
                    let r = compute(&input(
                        TradingStatus::EstablishedSecuritiesMarket,
                        100_000,
                        85_000, // fails 90%
                        true,
                        inadvertent,
                        corrective,
                        agreement,
                    ));
                    let expected_relief = inadvertent && corrective && agreement;
                    assert_eq!(
                        r.inadvertent_termination_relief_applies, expected_relief,
                        "inadvertent={} corrective={} agreement={}",
                        inadvertent, corrective, agreement,
                    );
                }
            }
        }
    }

    #[test]
    fn ninety_pct_boundary_strict_at_threshold_invariant() {
        // 8_999 bp → fail; 9_000 bp → pass; 9_001 bp → pass.
        for (qi, expected_pass) in [(8_999_i64, false), (9_000, true), (9_001, true)] {
            let r = compute(&input(
                TradingStatus::EstablishedSecuritiesMarket,
                10_000,
                qi,
                true,
                false,
                false,
                false,
            ));
            assert_eq!(
                r.meets_90_pct_test, expected_pass,
                "qi={} expected_pass={}",
                qi, expected_pass,
            );
        }
    }

    #[test]
    fn not_ptp_never_treated_as_corporation_invariant() {
        // Across all (qi, continuous) combinations, a non-PTP is
        // never treated as a corporation under § 7704.
        for qi in [0_i64, 50_000, 85_000, 90_000, 95_000, 150_000] {
            for continuous in [false, true] {
                let r = compute(&input(
                    TradingStatus::NotPubliclyTraded,
                    100_000,
                    qi,
                    continuous,
                    false,
                    false,
                    false,
                ));
                assert!(
                    !r.treated_as_corporation,
                    "qi={} continuous={}: non-PTP must never be treated as corporation",
                    qi, continuous,
                );
            }
        }
    }

    #[test]
    fn citation_pins_subsections_per_outcome() {
        // Exception applies → cite (c)(1) + (c)(2) + (d)(1).
        let exception = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            95_000,
            true,
            false,
            false,
            false,
        ));
        assert!(exception.citation.contains("§ 7704(c)(1)–(c)(2)"));
        assert!(exception.citation.contains("§ 7704(d)(1)(A)–(G)"));

        // Corporate treatment → cite (a) + (b) + failed-test reason.
        let corp = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            50_000,
            true,
            false,
            false,
            false,
        ));
        assert!(corp.citation.contains("§ 7704(a)"));
        assert!(corp.citation.contains("§ 7704(b)"));

        // Relief applies → cite (e) + (b).
        let relief = compute(&input(
            TradingStatus::EstablishedSecuritiesMarket,
            100_000,
            85_000,
            true,
            true,
            true,
            true,
        ));
        assert!(relief.citation.contains("§ 7704(e)"));
        assert!(relief.citation.contains("§ 7704(b)"));
        assert!(relief.citation.contains("subchapter K"));
    }
}
