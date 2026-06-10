//! Hand-crafted invariant tests for the IRC § 199A QBI deduction.
//!
//! The in-module unit tests in `crates/traderview-tax/src/qbi.rs` cover
//! sub-threshold deduction, full SSTB phase-out, and the TI cap, but
//! they leave three concrete bug classes uncovered:
//!
//!   1. The SSTB **linear phase-in math** for taxable income strictly
//!      *inside* the phase-in window. A miswritten formula
//!      (e.g. `over / window` instead of `(window - over) / window`,
//!      or a missing `round_dp`) would not fail any current test.
//!   2. The **MFJ-specific threshold and window** ($483,900 + $100,000).
//!      Every existing qbi.rs unit test uses Single. A wrong-status
//!      lookup that returned the Single threshold for MFJ would pass
//!      every current qbi.rs unit test.
//!   3. The **threshold-equality boundary** — the code branches on
//!      `taxable_income_before_qbi <= threshold` (sub-threshold path
//!      with no manual-review flag) vs `>` (above-threshold path that
//!      flags). A typo flipping `<=` to `<` would silently flag every
//!      filer whose TI lands exactly on the threshold for manual
//!      review. No existing test pins TI == threshold.
//!
//! Each test below is written to fail loudly if any of those specific
//! bug classes is introduced, with the expected numeric outputs derived
//! by hand from the Rev. Proc. 2024-40 § 3.27 thresholds.

use rust_decimal::Decimal;
use std::str::FromStr;
use traderview_tax::engine::FilingStatus;
use traderview_tax::qbi::{compute, QbiInput};

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

fn dc(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// Single, SSTB, taxable income $25,000 over the threshold — halfway
/// through the $50,000 phase-in window. The linear "allowed" multiplier
/// must equal `(window - over) / window = 0.5`, so the deduction is
/// exactly half of the sub-threshold cap.
///
/// Sub-threshold cap = min(20% × $50k QBI, 20% × $266,950 TI) = min($10,000, $53,390) = $10,000.
/// Phased deduction = $10,000 × 0.5 = $5,000.00.
///
/// A formula inversion (`over/window` instead of `(window-over)/window`)
/// would yield $5,000 here ONLY by coincidence — see the asymmetric
/// $10,000-over test below that disambiguates the two formulas.
#[test]
fn sstb_single_at_window_midpoint_yields_exactly_half_deduction() {
    let r = compute(QbiInput {
        qualified_business_income: d(50_000),
        taxable_income_before_qbi: d(266_950),
        net_capital_gain: Decimal::ZERO,
        is_sstb: true,
        status: FilingStatus::Single,
    });
    assert_eq!(
        r.deduction,
        dc("5000.00"),
        "SSTB at exact phase-in midpoint must yield half the sub-threshold cap"
    );
    assert!(
        r.needs_manual_review,
        "SSTB above threshold must flag manual review per qbi.rs above-threshold contract"
    );
}

/// Asymmetric phase-in disambiguator. Single SSTB, $10,000 over the
/// threshold (20% of window). The CORRECT formula
/// `(window - over) / window = 40,000 / 50,000 = 0.80` yields a
/// deduction of $10,000 × 0.80 = $8,000.00.
///
/// An inverted formula `over / window = 0.20` would yield $2,000 here,
/// which is observably different — this test catches the inversion that
/// the midpoint test cannot.
#[test]
fn sstb_single_at_20_pct_into_window_yields_80_pct_deduction() {
    let r = compute(QbiInput {
        qualified_business_income: d(50_000),
        taxable_income_before_qbi: d(251_950), // = 241_950 + 10_000
        net_capital_gain: Decimal::ZERO,
        is_sstb: true,
        status: FilingStatus::Single,
    });
    assert_eq!(
        r.deduction,
        dc("8000.00"),
        "SSTB 20% into the window must yield 80% of the sub-threshold cap"
    );
}

/// MFJ thresholds are double the Single thresholds: threshold $483,900,
/// window $100,000. The qbi.rs unit tests never exercise MFJ, so a
/// wrong-status lookup that fell through to the Single threshold would
/// pass every current test.
///
/// MFJ SSTB, TI $533,900 (over by $50,000 — halfway through the $100,000
/// MFJ window), QBI $80k:
///   * qbi_20 = $16,000.
///   * ti_cap = 20% × $533,900 = $106,780.
///   * min    = $16,000.
///   * allowed = (100,000 - 50,000) / 100,000 = 0.5.
///   * deduction = $16,000 × 0.5 = $8,000.00.
///
/// If the threshold lookup returned $241,950 (Single) for MFJ, the
/// computed over would be $291,950, which exceeds the $50k Single
/// window, and the function would return $0 — a CHF-failing mismatch
/// with the expected $8,000.
#[test]
fn sstb_mfj_uses_doubled_threshold_and_window() {
    let r = compute(QbiInput {
        qualified_business_income: d(80_000),
        taxable_income_before_qbi: d(533_900),
        net_capital_gain: Decimal::ZERO,
        is_sstb: true,
        status: FilingStatus::Mfj,
    });
    assert_eq!(
        r.deduction,
        dc("8000.00"),
        "MFJ SSTB at MFJ window midpoint must use $483,900 threshold + $100,000 window"
    );
    assert!(r.needs_manual_review);
}

/// Threshold-equality edge: TI exactly equals the MFJ threshold
/// ($483,900). The code uses `<= threshold` for the sub-threshold
/// branch, so the filer must get the FULL deduction with NO manual
/// review flag. A typo to `< threshold` would push them into the
/// above-threshold non-SSTB branch and set `needs_manual_review = true`
/// while leaving the deduction numerically equal (because the non-SSTB
/// above-threshold branch returns the same `qbi_20.min(ti_cap)` value)
/// — silently flagging every on-the-line filer for review.
///
/// The dual assert below catches the flag flip even when the numeric
/// deduction stays the same.
#[test]
fn sub_threshold_path_includes_exact_mfj_threshold_value() {
    let r = compute(QbiInput {
        qualified_business_income: d(100_000),
        taxable_income_before_qbi: d(483_900),
        net_capital_gain: Decimal::ZERO,
        is_sstb: false,
        status: FilingStatus::Mfj,
    });
    assert_eq!(
        r.deduction,
        d(20_000),
        "QBI deduction at exact MFJ threshold should still be 20% × QBI = $20,000"
    );
    assert!(
        !r.needs_manual_review,
        "TI == threshold MUST take the sub-threshold path, NOT flag for manual review"
    );
}

/// `net_capital_gain` larger than `taxable_income_before_qbi` would
/// make the pre-clamp `(TI - cap_gain)` negative. The implementation
/// clamps the TI-side cap at zero via `.max(Decimal::ZERO)`. With
/// `ti_cap = 0`, `min(qbi_20, 0) = 0`, so the deduction must be zero —
/// NOT a panic from a negative Decimal flowing through the 20%
/// multiplication, and NOT a negative deduction.
///
/// A regression that dropped the `.max(Decimal::ZERO)` clamp would
/// produce a NEGATIVE deduction here, which the engine would then
/// subtract from taxable income elsewhere, inflating refunds for a
/// pathological-but-reachable cap-gain-heavy input.
#[test]
fn negative_pre_clamp_ti_cap_yields_zero_not_negative_deduction() {
    let r = compute(QbiInput {
        qualified_business_income: d(50_000),
        taxable_income_before_qbi: d(20_000),
        net_capital_gain: d(60_000), // > taxable_income_before_qbi
        is_sstb: false,
        status: FilingStatus::Single,
    });
    assert_eq!(
        r.deduction,
        Decimal::ZERO,
        "When net_capital_gain > TI, ti_cap clamps to zero and deduction must be zero"
    );
    assert!(
        r.deduction >= Decimal::ZERO,
        "QBI deduction must never go negative regardless of input"
    );
}
