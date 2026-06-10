//! Edge-case + invariant tests for the tax compute modules. Each test
//! pins a specific bug class that the existing per-module unit tests do
//! NOT cover:
//!
//!   * Off-by-one at exact phaseout / band boundaries.
//!   * Partial-cap interaction between W-2 and SE for the SS wage base.
//!   * Capital-gains worksheet ordinary-vs-preferential slicing when the
//!     preferential stack lands EXACTLY on the 0% band ceiling for MFJ.
//!
//! Sources for the hand-math:
//!   * IRS Rev. Proc. 2024-40 § 3.07, § 3.18 (2025 phaseout/breakpoint
//!     constants).
//!   * SSA 2024-10-10 wage-base press release (SS_WAGE_BASE_2025 =
//!     $176,100).

use rust_decimal::Decimal;
use traderview_tax::{
    capital_gains, compute_section_179,
    se_tax::{self, SS_WAGE_BASE_2025},
    section_179::{Section179Input, S179_MAX_2025, S179_PHASEOUT_THRESHOLD_2025},
    FilingStatus,
};

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}
fn dc(s: &str) -> Decimal {
    s.parse::<Decimal>().expect("decimal literal")
}

/// § 179 phaseout: dollar-for-dollar reduction triggers ABOVE the
/// threshold, NOT at the threshold. Per IRC § 179(b)(2) the reduction
/// applies to the "amount by which the cost of section 179 property
/// placed in service exceeds $2,500,000" (2017 baseline) — *exceeds*,
/// not "equals or exceeds". A buy at exactly $3,130,000 must keep the
/// full $1,250,000 ceiling.
///
/// This test catches an off-by-one in `excess_over_phaseout = (cost -
/// threshold).max(0)` that would only surface if the comparison were
/// strict-greater-than vs. greater-or-equal. The current implementation
/// uses subtraction-then-clamp, which yields 0 at equality — correct —
/// but the existing tests only hit $3.5M (above) and $4.38M (full
/// phaseout) and never the boundary itself.
#[test]
fn section_179_at_exact_phaseout_threshold_takes_full_cap() {
    let r = compute_section_179(Section179Input {
        total_qualifying_cost: d(S179_PHASEOUT_THRESHOLD_2025),
        trade_or_business_income: d(10_000_000),
        auto_cap: None,
    });
    assert_eq!(
        r.s179_max_after_phaseout,
        d(S179_MAX_2025),
        "at exactly the $3,130,000 threshold, no phaseout reduction may apply"
    );
    assert_eq!(
        r.s179_deduction,
        d(S179_MAX_2025),
        "income-uncapped, the full $1,250,000 must be deductible"
    );
    // Bonus base = $3,130,000 - $1,250,000 = $1,880,000. 40% = $752,000.
    assert_eq!(r.bonus_depreciation, d(752_000));
    // Invariant: write-off + remaining basis exactly equals cost.
    assert_eq!(
        r.first_year_write_off + r.remaining_macrs_basis,
        d(S179_PHASEOUT_THRESHOLD_2025),
    );
}

/// Self-employment tax SS portion when W-2 SS wages have already
/// consumed PART (but not all) of the 2025 wage base. The cap math is
/// `remaining_cap = max(0, base - w2_ss_wages); ss_taxable = min(se_base,
/// remaining_cap)`. The existing tests only cover the two extreme W-2
/// scenarios (none consumed, cap fully consumed). The realistic case —
/// a moonlighting taxpayer with a regular W-2 — is between these and
/// catches a subtle bug if anyone ever reorders or off-by-ones the
/// `remaining_cap.min(se_base)` direction.
///
/// W-2 SS wages = $100,000 → remaining_cap = $176,100 - $100,000 = $76,100.
/// Net SE = $100,000 → SE base = $92,350.
/// SS-taxable = min($92,350, $76,100) = $76,100.
/// SS tax = $76,100 × 12.4% = $9,436.40.
/// Medicare on full base: $92,350 × 2.9% = $2,678.15.
#[test]
fn se_tax_partial_ss_cap_caps_at_remaining_room_not_full_se_base() {
    let r = se_tax::compute(
        d(100_000),    // net SE earnings
        d(100_000),    // W-2 SS wages already taxed
        d(100_000),    // W-2 Medicare wages
        FilingStatus::Single,
    );
    // SS taxable is capped at the *remaining* cap room, not the full SE base.
    let expected_ss_tax = dc("9436.40");
    assert_eq!(
        r.ss_tax, expected_ss_tax,
        "SS portion must use remaining_cap ($76,100), NOT full SE base ($92,350)"
    );
    // Sanity: had we forgotten the cap, the SS tax would be 92350 × 0.124 = 11,451.40.
    // The bug-form value would be exactly that, so this assertion would catch a
    // regression where someone swapped min/max or dropped the `min(remaining_cap)`.
    assert_ne!(
        r.ss_tax,
        dc("11451.40"),
        "SS tax must NOT equal full-base value — that means the cap dropped"
    );
    // Sanity: SS_WAGE_BASE_2025 constant is what we computed off of.
    assert_eq!(SS_WAGE_BASE_2025, 176_100);
}

/// QDCGTW (Schedule D worksheet): when MFJ preferential income stacks
/// EXACTLY up to (not past) the MFJ 0% top of $96,700, the entire
/// preferential bucket pays 0%. Off-by-one in `zero_band_end =
/// zero_top.min(pref_end)` or in `amount_at_15` (which uses
/// `pref_start.max(zero_top)`) would leak $1 into the 15% band.
///
/// Setup:
///   ORD = $0, PREF = $96,700 → TI = $96,700.
///   PREF stack range = [$0, $96,700]. zero_band: [$0, $96,700].
///   amount_at_0 = $96,700.  amount_at_15 = max(0, $96,700 - $96,700) = $0.
#[test]
fn capital_gains_mfj_pref_exactly_at_zero_pct_top_no_pref_tax() {
    let r = capital_gains::compute(capital_gains::QdcgtwInput {
        taxable_income: d(96_700),
        net_long_term_capital_gain: d(96_700),
        qualified_dividends: Decimal::ZERO,
        status: FilingStatus::Mfj,
    });
    assert_eq!(r.preferential_income, d(96_700));
    assert_eq!(r.ordinary_income, Decimal::ZERO);
    assert_eq!(
        r.amount_at_0_pct,
        d(96_700),
        "preferential stack ending EXACTLY at the 0% top must be 100% in the 0% band"
    );
    assert_eq!(r.amount_at_15_pct, Decimal::ZERO);
    assert_eq!(r.amount_at_20_pct, Decimal::ZERO);
    assert_eq!(r.preferential_tax, Decimal::ZERO);
    // Invariant: the three band amounts must sum to total preferential income.
    assert_eq!(
        r.amount_at_0_pct + r.amount_at_15_pct + r.amount_at_20_pct,
        r.preferential_income,
    );
}

/// QDCGTW band invariant under a stress configuration that crosses BOTH
/// the 0%-to-15% and the 15%-to-20% boundaries inside the same pref
/// bucket. The three slices must (a) sum to total pref, (b) be
/// individually non-negative, (c) preserve `amount_at_15 +
/// amount_at_20 = pref - amount_at_0`. Catches accidental
/// double-counting in `amount_at_20 = pref - amount_at_0 - amount_at_15`
/// when one of the intermediate maxes underflows.
#[test]
fn capital_gains_band_amounts_partition_preferential_exactly() {
    // Single, ORD = $30k, PREF = $600k → TI = $630k.
    // PREF stack [30k, 630k]:
    //   0% band: [30k, 48,350] = 18,350.
    //   15% band: [48,350, 533,400] = 485,050.
    //   20% band: [533,400, 630k] = 96,600.
    //   Sum check: 18,350 + 485,050 + 96,600 = 600,000. ✓
    let r = capital_gains::compute(capital_gains::QdcgtwInput {
        taxable_income: d(630_000),
        net_long_term_capital_gain: d(600_000),
        qualified_dividends: Decimal::ZERO,
        status: FilingStatus::Single,
    });
    assert_eq!(r.preferential_income, d(600_000));
    assert_eq!(r.amount_at_0_pct, d(18_350));
    assert_eq!(r.amount_at_15_pct, d(485_050));
    assert_eq!(r.amount_at_20_pct, d(96_600));
    // Partition invariant — non-negotiable.
    assert_eq!(
        r.amount_at_0_pct + r.amount_at_15_pct + r.amount_at_20_pct,
        r.preferential_income,
    );
    // Each band non-negative.
    assert!(r.amount_at_0_pct >= Decimal::ZERO);
    assert!(r.amount_at_15_pct >= Decimal::ZERO);
    assert!(r.amount_at_20_pct >= Decimal::ZERO);
}
