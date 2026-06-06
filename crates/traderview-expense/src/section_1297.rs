//! IRC § 1297 — Passive foreign investment company (PFIC) defined.
//!
//! Trader-critical for any investor holding foreign mutual funds,
//! foreign ETFs, or foreign stock. § 1297(a) tests classify a
//! foreign corporation as a PFIC if EITHER:
//!
//!   (1) **75% income test** — 75% or more of gross income for the
//!       taxable year is passive income; OR
//!   (2) **50% asset test** — average percentage of assets which
//!       produce passive income (or are held for the production of
//!       passive income) is at least 50%.
//!
//! PFIC classification triggers the punitive § 1291 excess-
//! distribution + interest-charge regime UNLESS the shareholder
//! elects QEF treatment under § 1295 or mark-to-market under
//! § 1296 (both already in workspace).
//!
//! Companion to:
//!   - `section_1295` (PFIC qualified electing fund election).
//!   - `section_1296` (PFIC mark-to-market election for marketable
//!     PFIC stock).
//!
//! Operative subsections:
//!
//!   § 1297(a)(1) — 75% INCOME TEST: 75% or more of gross income
//!     is "passive income" (§ 1297(b)).
//!
//!   § 1297(a)(2) — 50% ASSET TEST: at least 50% of average assets
//!     produce passive income.
//!
//!   § 1297(b)(1) — PASSIVE INCOME = foreign personal holding
//!     company income under § 954(c) (interest, dividends, rents,
//!     royalties, gain from passive property).
//!
//!   § 1297(b)(2) — EXCEPTIONS:
//!     (A) Active banking income (licensed bank);
//!     (B) Active insurance income (qualifying insurance company);
//!     (C) Related-party income properly allocable to non-passive
//!         income.
//!
//!   § 1297(c) — LOOK-THROUGH RULE: foreign corporation owning at
//!     least 25% (by value) of stock of another corporation is
//!     treated as holding its proportionate share of the
//!     subsidiary's assets and receiving its proportionate share
//!     of the subsidiary's income.
//!
//!   § 1297(d) — ONCE-A-PFIC-ALWAYS-A-PFIC: corporation shall not
//!     be treated as a PFIC during the "qualified portion" of the
//!     shareholder's holding period — the post-acquisition period
//!     during which the corporation ceases to be a PFIC and the
//!     shareholder has made the § 1298(b)(1) purging election.
//!
//! Citations: 26 U.S.C. § 1297(a)(1) (75% income test); § 1297(a)(2)
//! (50% asset test); § 1297(b)(1) (passive income via § 954(c)
//! foreign personal holding company income); § 1297(b)(2)(A)
//! (active banking exception); § 1297(b)(2)(B) (active insurance
//! exception); § 1297(b)(2)(C) (related-party allocation
//! exception); § 1297(c) (25% look-through rule); § 1297(d)
//! (once-a-PFIC-always-a-PFIC with § 1298(b)(1) purging election);
//! § 954(c) (foreign personal holding company income);
//! § 1291 (excess-distribution + interest-charge regime); § 1295
//! (QEF election); § 1296 (mark-to-market election); § 1298(b)(1)
//! (purging election).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PfiCStatus {
    /// Foreign corporation does not satisfy either § 1297(a) test.
    NotPfic,
    /// PFIC via § 1297(a)(1) 75% income test only.
    PfiCByIncomeTest,
    /// PFIC via § 1297(a)(2) 50% asset test only.
    PfiCByAssetTest,
    /// PFIC via BOTH § 1297(a)(1) income test AND § 1297(a)(2)
    /// asset test.
    PfiCByBothTests,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1297Input {
    /// Foreign corporation's gross income for the taxable year
    /// (cents).
    pub gross_income_total_cents: i64,
    /// Gross income that is "passive income" under § 1297(b)(1)
    /// before § 1297(b)(2) exceptions (cents).
    pub passive_income_cents: i64,
    /// § 1297(b)(2)(A) active banking carve-out: income derived in
    /// active conduct of banking business by licensed institution
    /// is treated as non-passive.
    pub active_banking_carve_out_cents: i64,
    /// § 1297(b)(2)(B) active insurance carve-out: income derived
    /// in active conduct of insurance business by qualifying
    /// insurance corporation is treated as non-passive.
    pub active_insurance_carve_out_cents: i64,
    /// § 1297(b)(2)(C) related-party income properly allocable to
    /// non-passive income (cents).
    pub related_party_carve_out_cents: i64,
    /// Average total assets for the taxable year (cents).
    pub avg_total_assets_cents: i64,
    /// Average percentage of assets producing passive income
    /// (basis points × 100; e.g., 5000 = 50.00%).
    pub avg_passive_assets_bp: u32,
    /// § 1297(c) look-through rule — whether the foreign corp owns
    /// at least 25% (by value) of stock of another corporation.
    pub has_25_pct_owned_subsidiary: bool,
    /// Foreign corp's proportionate share of subsidiary's passive
    /// income (cents).
    pub subsidiary_passive_income_share_cents: i64,
    /// Foreign corp's proportionate share of subsidiary's total
    /// income (cents).
    pub subsidiary_total_income_share_cents: i64,
    /// § 1297(d) — whether the once-a-PFIC corporation has had the
    /// shareholder's § 1298(b)(1) purging election applied.
    pub purging_election_applied: bool,
    /// Whether shareholder is in the "qualified portion" of the
    /// holding period (post-purging where the corporation has
    /// ceased to be a PFIC).
    pub in_qualified_portion_of_holding_period: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1297Result {
    pub pfic_status: PfiCStatus,
    /// Net passive income after § 1297(b)(2) exceptions (cents).
    pub net_passive_income_cents: i64,
    /// Net passive income / gross income ratio in basis points
    /// × 100 (e.g., 7500 = 75.00%).
    pub passive_income_percentage_bp: u32,
    /// Effective passive-assets percentage in basis points × 100.
    /// Equal to input `avg_passive_assets_bp` (look-through
    /// applies separately to the underlying subsidiary's assets).
    pub passive_assets_percentage_bp: u32,
    /// True if § 1297(a)(1) 75% income test is satisfied.
    pub income_test_satisfied: bool,
    /// True if § 1297(a)(2) 50% asset test is satisfied.
    pub asset_test_satisfied: bool,
    /// True if the § 1297(c) look-through rule applies (subsidiary
    /// owned 25%+ by value).
    pub look_through_rule_applies: bool,
    /// True if the § 1297(d) once-a-PFIC qualified-portion
    /// exception applies (purging election + qualified portion).
    pub once_a_pfic_qualified_portion_applies: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 1297(a)(1) income test threshold — 75% (7500 bp × 100).
pub const SECTION_1297_INCOME_TEST_BP: u32 = 7_500;
/// § 1297(a)(2) asset test threshold — 50% (5000 bp × 100).
pub const SECTION_1297_ASSET_TEST_BP: u32 = 5_000;
/// § 1297(c) look-through rule threshold — 25% (2500 bp × 100).
pub const SECTION_1297C_LOOK_THROUGH_BP: u32 = 2_500;

pub fn compute(input: &Section1297Input) -> Section1297Result {
    let mut notes: Vec<String> = Vec::new();

    // § 1297(b)(2) exceptions — subtract active banking + active
    // insurance + related-party carve-outs from passive income.
    let carve_outs = input
        .active_banking_carve_out_cents
        .max(0)
        .saturating_add(input.active_insurance_carve_out_cents.max(0))
        .saturating_add(input.related_party_carve_out_cents.max(0));
    let net_passive_income = input.passive_income_cents.saturating_sub(carve_outs).max(0);

    // § 1297(c) look-through rule — add subsidiary's proportionate
    // passive income to numerator AND total income to denominator.
    let mut effective_passive_income = net_passive_income;
    let mut effective_gross_income = input.gross_income_total_cents.max(0);
    if input.has_25_pct_owned_subsidiary {
        effective_passive_income = effective_passive_income
            .saturating_add(input.subsidiary_passive_income_share_cents.max(0));
        effective_gross_income =
            effective_gross_income.saturating_add(input.subsidiary_total_income_share_cents.max(0));
        notes.push(
            "§ 1297(c) look-through rule APPLIES — foreign corp owns at least 25% (by value) \
             of another corporation; subsidiary's proportionate passive income and total \
             income flow up into the income test."
                .to_string(),
        );
    }

    // § 1297(a)(1) 75% income test.
    let passive_income_bp = if effective_gross_income > 0 {
        let pct = effective_passive_income.saturating_mul(10_000) / effective_gross_income;
        pct.clamp(0, 10_000) as u32
    } else {
        0
    };
    let income_test_satisfied = passive_income_bp >= SECTION_1297_INCOME_TEST_BP;

    // § 1297(a)(2) 50% asset test.
    let asset_test_satisfied = input.avg_passive_assets_bp >= SECTION_1297_ASSET_TEST_BP;

    // § 1297(d) once-a-PFIC qualified-portion exception.
    let once_a_pfic_qualified =
        input.purging_election_applied && input.in_qualified_portion_of_holding_period;
    if once_a_pfic_qualified {
        notes.push(
            "§ 1297(d) — once-a-PFIC qualified portion applies: shareholder has made the \
             § 1298(b)(1) purging election AND is in the qualified portion of the holding \
             period. Corporation is NOT treated as a PFIC for this shareholder during this \
             period regardless of tests."
                .to_string(),
        );
    }

    let pfic_status = if once_a_pfic_qualified {
        PfiCStatus::NotPfic
    } else {
        match (income_test_satisfied, asset_test_satisfied) {
            (false, false) => PfiCStatus::NotPfic,
            (true, false) => PfiCStatus::PfiCByIncomeTest,
            (false, true) => PfiCStatus::PfiCByAssetTest,
            (true, true) => PfiCStatus::PfiCByBothTests,
        }
    };

    // Exception notes.
    if input.active_banking_carve_out_cents > 0 {
        notes.push(format!(
            "§ 1297(b)(2)(A) — active banking exception: {} cents of income derived in active \
             conduct of banking business by licensed institution is treated as non-passive.",
            input.active_banking_carve_out_cents,
        ));
    }
    if input.active_insurance_carve_out_cents > 0 {
        notes.push(format!(
            "§ 1297(b)(2)(B) — active insurance exception: {} cents of income derived in \
             active conduct of insurance business by qualifying insurance corporation is \
             treated as non-passive.",
            input.active_insurance_carve_out_cents,
        ));
    }
    if input.related_party_carve_out_cents > 0 {
        notes.push(format!(
            "§ 1297(b)(2)(C) — related-party exception: {} cents of interest/dividends/rent/\
             royalty from related person properly allocable to non-passive income is treated \
             as non-passive.",
            input.related_party_carve_out_cents,
        ));
    }

    notes.push(
        "PFIC classification triggers § 1291 excess-distribution + interest-charge regime \
         UNLESS shareholder elects QEF treatment under § 1295 or mark-to-market under § 1296. \
         See companion modules section_1295 + section_1296."
            .to_string(),
    );

    let citation = if once_a_pfic_qualified {
        "26 U.S.C. § 1297(d) (once-a-PFIC qualified-portion exception); § 1298(b)(1) (purging \
         election); § 1297(a)(1)–(2) (75% income + 50% asset tests not engaged for this \
         shareholder); § 1291 (excess-distribution regime — not triggered)"
    } else {
        "26 U.S.C. § 1297(a)(1) (75% income test); § 1297(a)(2) (50% asset test); § 1297(b)(1) \
         (passive income via § 954(c)); § 1297(b)(2)(A) (active banking exception); \
         § 1297(b)(2)(B) (active insurance exception); § 1297(b)(2)(C) (related-party \
         exception); § 1297(c) (25% look-through rule); § 1291 (excess-distribution regime \
         triggered if PFIC); § 1295 (QEF election); § 1296 (mark-to-market election)"
    };

    Section1297Result {
        pfic_status,
        net_passive_income_cents: net_passive_income,
        passive_income_percentage_bp: passive_income_bp,
        passive_assets_percentage_bp: input.avg_passive_assets_bp,
        income_test_satisfied,
        asset_test_satisfied,
        look_through_rule_applies: input.has_25_pct_owned_subsidiary,
        once_a_pfic_qualified_portion_applies: once_a_pfic_qualified,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        gross: i64,
        passive: i64,
        banking: i64,
        insurance: i64,
        related: i64,
        total_assets: i64,
        passive_assets_bp: u32,
        look_through: bool,
        sub_passive_income: i64,
        sub_total_income: i64,
        purged: bool,
        qualified_portion: bool,
    ) -> Section1297Input {
        Section1297Input {
            gross_income_total_cents: gross,
            passive_income_cents: passive,
            active_banking_carve_out_cents: banking,
            active_insurance_carve_out_cents: insurance,
            related_party_carve_out_cents: related,
            avg_total_assets_cents: total_assets,
            avg_passive_assets_bp: passive_assets_bp,
            has_25_pct_owned_subsidiary: look_through,
            subsidiary_passive_income_share_cents: sub_passive_income,
            subsidiary_total_income_share_cents: sub_total_income,
            purging_election_applied: purged,
            in_qualified_portion_of_holding_period: qualified_portion,
        }
    }

    // ── § 1297(a)(1) 75% income test ────────────────────────────

    #[test]
    fn passive_80_pct_satisfies_income_test() {
        // Passive 80,000 / Gross 100,000 = 80% ≥ 75%.
        let r = compute(&input(
            100_000, 80_000, 0, 0, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.passive_income_percentage_bp, 8_000);
        assert!(r.income_test_satisfied);
        assert_eq!(r.pfic_status, PfiCStatus::PfiCByIncomeTest);
        assert!(r.citation.contains("§ 1297(a)(1)"));
    }

    #[test]
    fn passive_at_75_pct_boundary_satisfies_income_test() {
        let r = compute(&input(
            100_000, 75_000, 0, 0, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.passive_income_percentage_bp, 7_500);
        assert!(r.income_test_satisfied);
    }

    #[test]
    fn passive_just_below_75_pct_fails_income_test() {
        let r = compute(&input(
            100_000, 74_999, 0, 0, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.passive_income_percentage_bp, 7_499);
        assert!(!r.income_test_satisfied);
        assert_eq!(r.pfic_status, PfiCStatus::NotPfic);
    }

    // ── § 1297(a)(2) 50% asset test ─────────────────────────────

    #[test]
    fn passive_assets_60_pct_satisfies_asset_test() {
        let r = compute(&input(
            100_000, 0, 0, 0, 0, 1_000_000, 6_000, false, 0, 0, false, false,
        ));
        assert!(r.asset_test_satisfied);
        assert_eq!(r.pfic_status, PfiCStatus::PfiCByAssetTest);
    }

    #[test]
    fn passive_assets_at_50_pct_boundary_satisfies_asset_test() {
        let r = compute(&input(
            100_000, 0, 0, 0, 0, 1_000_000, 5_000, false, 0, 0, false, false,
        ));
        assert!(r.asset_test_satisfied);
    }

    #[test]
    fn passive_assets_just_below_50_pct_fails_asset_test() {
        let r = compute(&input(
            100_000, 0, 0, 0, 0, 1_000_000, 4_999, false, 0, 0, false, false,
        ));
        assert!(!r.asset_test_satisfied);
        assert_eq!(r.pfic_status, PfiCStatus::NotPfic);
    }

    // ── PFIC by BOTH tests ──────────────────────────────────────

    #[test]
    fn pfic_by_both_tests_simultaneously() {
        let r = compute(&input(
            100_000, 80_000, 0, 0, 0, 1_000_000, 6_000, false, 0, 0, false, false,
        ));
        assert_eq!(r.pfic_status, PfiCStatus::PfiCByBothTests);
        assert!(r.income_test_satisfied);
        assert!(r.asset_test_satisfied);
    }

    // ── § 1297(b)(2) exceptions ─────────────────────────────────

    #[test]
    fn active_banking_exception_reduces_passive_income() {
        // 80,000 passive, 30,000 active banking → net 50,000 / 100,000
        // = 50% < 75%.
        let r = compute(&input(
            100_000, 80_000, 30_000, 0, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.net_passive_income_cents, 50_000);
        assert!(!r.income_test_satisfied);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1297(b)(2)(A)") && n.contains("active banking")));
    }

    #[test]
    fn active_insurance_exception_reduces_passive_income() {
        let r = compute(&input(
            100_000, 80_000, 0, 20_000, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.net_passive_income_cents, 60_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1297(b)(2)(B)") && n.contains("active insurance")));
    }

    #[test]
    fn related_party_exception_reduces_passive_income() {
        let r = compute(&input(
            100_000, 80_000, 0, 0, 15_000, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.net_passive_income_cents, 65_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1297(b)(2)(C)") && n.contains("related-party")));
    }

    #[test]
    fn all_three_exceptions_compound_to_zero_passive() {
        // 80,000 passive minus 30 + 30 + 20 = 80,000 → net 0.
        let r = compute(&input(
            100_000, 80_000, 30_000, 30_000, 20_000, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.net_passive_income_cents, 0);
    }

    #[test]
    fn carve_outs_exceeding_passive_clamp_at_zero() {
        let r = compute(&input(
            100_000, 50_000, 30_000, 30_000, 30_000, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.net_passive_income_cents, 0);
    }

    // ── § 1297(c) look-through rule ─────────────────────────────

    #[test]
    fn look_through_subsidiary_income_flows_into_test() {
        // Foreign corp gross 100,000, passive 30,000.
        // Subsidiary share: 50,000 passive + 60,000 total.
        // Combined: passive (30,000 + 50,000) = 80,000;
        // gross (100,000 + 60,000) = 160,000. 80,000/160,000 = 50%.
        let r = compute(&input(
            100_000, 30_000, 0, 0, 0, 0, 0, true, 50_000, 60_000, false, false,
        ));
        assert!(r.look_through_rule_applies);
        assert_eq!(r.passive_income_percentage_bp, 5_000);
        assert!(!r.income_test_satisfied);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1297(c)") && n.contains("25%")));
    }

    #[test]
    fn look_through_can_make_corp_pfic_by_subsidiary_income() {
        // Without look-through: 0% passive. With: subsidiary
        // contributes 80,000 passive + 100,000 total → combined
        // 80,000/200,000 = 40% (not PFIC).
        // Bigger: 90,000 sub passive + 110,000 sub total →
        // 90,000/210,000 = 42.85% (still not).
        // To trigger: huge sub passive + small sub total →
        // 100,000 sub passive + 100,000 sub total = 100% sub passive.
        // Combined: 100,000/200,000 = 50%. Need 75%+.
        // Try: 500,000 sub passive + 500,000 sub total + 100,000 corp non-passive.
        // Combined: 500,000 passive / 600,000 total = 83.33%.
        let r = compute(&input(
            100_000, 0, 0, 0, 0, 0, 0, true, 500_000, 500_000, false, false,
        ));
        assert_eq!(r.passive_income_percentage_bp, 8_333);
        assert!(r.income_test_satisfied);
        assert_eq!(r.pfic_status, PfiCStatus::PfiCByIncomeTest);
    }

    #[test]
    fn no_look_through_when_subsidiary_below_25_pct() {
        let r = compute(&input(
            100_000, 80_000, 0, 0, 0, 0, 0, false, 50_000, 60_000, false, false,
        ));
        assert!(!r.look_through_rule_applies);
        // Subsidiary contribution ignored.
        assert_eq!(r.passive_income_percentage_bp, 8_000);
    }

    // ── § 1297(d) once-a-PFIC qualified-portion exception ──────

    #[test]
    fn once_a_pfic_qualified_portion_overrides_tests() {
        // Even satisfying both tests, if purging election applied
        // and in qualified portion, NOT treated as PFIC.
        let r = compute(&input(
            100_000, 80_000, 0, 0, 0, 1_000_000, 6_000, false, 0, 0, true, true,
        ));
        assert_eq!(r.pfic_status, PfiCStatus::NotPfic);
        assert!(r.once_a_pfic_qualified_portion_applies);
        assert!(r.citation.contains("§ 1297(d)"));
        assert!(r.citation.contains("§ 1298(b)(1)"));
    }

    #[test]
    fn purging_election_alone_without_qualified_portion_does_not_help() {
        let r = compute(&input(
            100_000, 80_000, 0, 0, 0, 0, 0, false, 0, 0, true, false,
        ));
        assert!(!r.once_a_pfic_qualified_portion_applies);
        assert_eq!(r.pfic_status, PfiCStatus::PfiCByIncomeTest);
    }

    #[test]
    fn qualified_portion_alone_without_purging_does_not_help() {
        let r = compute(&input(
            100_000, 80_000, 0, 0, 0, 0, 0, false, 0, 0, false, true,
        ));
        assert!(!r.once_a_pfic_qualified_portion_applies);
        assert_eq!(r.pfic_status, PfiCStatus::PfiCByIncomeTest);
    }

    // ── Edge cases ─────────────────────────────────────────────

    #[test]
    fn zero_gross_income_zero_passive_percentage() {
        let r = compute(&input(0, 0, 0, 0, 0, 0, 0, false, 0, 0, false, false));
        assert_eq!(r.passive_income_percentage_bp, 0);
        assert!(!r.income_test_satisfied);
    }

    #[test]
    fn passive_exceeds_gross_clamps_at_100_pct() {
        let r = compute(&input(
            100_000, 150_000, 0, 0, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert_eq!(r.passive_income_percentage_bp, 10_000);
        assert!(r.income_test_satisfied);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn either_test_triggers_pfic_status_invariant() {
        // 4-cell truth table: NotPfic, PfiCByIncomeTest,
        // PfiCByAssetTest, PfiCByBothTests.
        for (income_high, assets_high, expected) in [
            (false, false, PfiCStatus::NotPfic),
            (true, false, PfiCStatus::PfiCByIncomeTest),
            (false, true, PfiCStatus::PfiCByAssetTest),
            (true, true, PfiCStatus::PfiCByBothTests),
        ] {
            let passive = if income_high { 80_000 } else { 50_000 };
            let assets_bp = if assets_high { 6_000 } else { 4_000 };
            let r = compute(&input(
                100_000, passive, 0, 0, 0, 0, assets_bp, false, 0, 0, false, false,
            ));
            assert_eq!(
                r.pfic_status, expected,
                "income_high={} assets_high={} expected={:?}",
                income_high, assets_high, expected,
            );
        }
    }

    #[test]
    fn income_test_75_pct_threshold_strict_boundary_invariant() {
        for (bp, expected_satisfied) in [
            (7_499_u32, false),
            (7_500, true), // ≥ 75% — at boundary satisfies
            (7_501, true),
        ] {
            let passive = bp as i64 * 10; // out of 100_000
            let r = compute(&input(
                100_000, passive, 0, 0, 0, 0, 0, false, 0, 0, false, false,
            ));
            assert_eq!(
                r.income_test_satisfied, expected_satisfied,
                "bp={} expected_satisfied={}",
                bp, expected_satisfied,
            );
        }
    }

    #[test]
    fn asset_test_50_pct_threshold_strict_boundary_invariant() {
        for (bp, expected_satisfied) in [
            (4_999_u32, false),
            (5_000, true), // ≥ 50% — at boundary satisfies
            (5_001, true),
        ] {
            let r = compute(&input(
                100_000, 0, 0, 0, 0, 0, bp, false, 0, 0, false, false,
            ));
            assert_eq!(
                r.asset_test_satisfied, expected_satisfied,
                "bp={} expected_satisfied={}",
                bp, expected_satisfied,
            );
        }
    }

    #[test]
    fn once_a_pfic_requires_both_purging_and_qualified_portion_invariant() {
        for (purged, qualified, expected_override) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (true, true, true),
        ] {
            let r = compute(&input(
                100_000, 80_000, 0, 0, 0, 0, 0, false, 0, 0, purged, qualified,
            ));
            assert_eq!(
                r.once_a_pfic_qualified_portion_applies, expected_override,
                "purged={} qualified={} expected_override={}",
                purged, qualified, expected_override,
            );
        }
    }

    #[test]
    fn citation_pins_subsections_per_path() {
        let standard = compute(&input(
            100_000, 80_000, 0, 0, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert!(standard.citation.contains("§ 1297(a)(1)"));
        assert!(standard.citation.contains("§ 1297(a)(2)"));
        assert!(standard.citation.contains("§ 1297(b)(1)"));
        assert!(standard.citation.contains("§ 1297(b)(2)(A)"));
        assert!(standard.citation.contains("§ 1297(b)(2)(B)"));
        assert!(standard.citation.contains("§ 1297(b)(2)(C)"));
        assert!(standard.citation.contains("§ 1297(c)"));
        assert!(standard.citation.contains("§ 1295"));
        assert!(standard.citation.contains("§ 1296"));

        let qualified = compute(&input(
            100_000, 80_000, 0, 0, 0, 0, 0, false, 0, 0, true, true,
        ));
        assert!(qualified.citation.contains("§ 1297(d)"));
        assert!(qualified.citation.contains("§ 1298(b)(1)"));
        assert!(qualified.citation.contains("not triggered"));
    }

    #[test]
    fn pfic_excess_distribution_note_present() {
        let r = compute(&input(
            100_000, 80_000, 0, 0, 0, 0, 0, false, 0, 0, false, false,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("§ 1291")
                && n.contains("excess-distribution")
                && n.contains("§ 1295")
                && n.contains("§ 1296")),
            "PFIC § 1291 + § 1295 + § 1296 cross-reference note must be present"
        );
    }
}
