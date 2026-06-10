//! § 199A QBI deduction — the alternate TI cap subtracts "net capital
//! gain" from taxable income. Per IRC § 1222 the "net capital gain" is
//! defined as the EXCESS (if any) of net LTCG over net STCL — i.e., it
//! is floored at zero. IRC § 199A(e)(3) inherits the IRC § 1(h)
//! definition which references § 1222.
//!
//! Mechanism the IRS expects (Form 8995-A line 30 worksheet):
//!   ti_cap = 20% × (TI - max(0, net_capital_gain))
//!
//! Bug class targeted: if the implementation skips the `max(0, ...)`
//! clamp on `net_capital_gain` before subtraction, then a taxpayer with
//! a NET LONG-TERM CAPITAL LOSS would see the cap INFLATE
//!   ti_cap = 20% × (TI - (negative loss)) = 20% × (TI + |loss|)
//! which is the opposite direction from the IRS rule (which says: a
//! capital LOSS does NOT change the TI cap, because § 1222 already
//! floored the value at 0).
//!
//! A real taxpayer this catches: a Schedule C consultant with
//! $50k QBI whose stock portfolio has a $10k net long-term loss.
//! Under the bug, their QBI cap would be inflated by $2k (20% × $10k).
//! Under correct math, their QBI deduction stays the same as if the
//! loss were zero.

use rust_decimal::Decimal;
use traderview_tax::{
    compute,
    engine::{ScheduleC, W2},
    FilingStatus, TaxReturn,
};

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

/// Build a small consultant return: $50k Schedule C profit, modest
/// W-2 to drive AGI somewhere sub-threshold for QBI phase-in. Caller
/// supplies the long-term capital gain (positive or negative) to flex
/// the only dimension under test.
fn consultant(net_ltcg: Decimal) -> TaxReturn {
    TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        w2s: vec![W2 {
            box_1_wages: d(20_000),
            ..Default::default()
        }],
        schedule_c: ScheduleC {
            gross_receipts: d(50_000),
            total_expenses: Decimal::ZERO,
            net_profit: d(50_000),
        },
        net_long_term_capital_gain: net_ltcg,
        ..Default::default()
    }
}

/// Baseline pin: zero capital gain → QBI deduction is purely the 20%
/// of net SE income, possibly capped by TI. This anchors the "no loss
/// case" so the assertion below has something to compare against.
///
/// Hand math:
///   W-2 $20k + SE $50k = $70k total income.
///   SE half deduction ≈ $3,532.39 (from se_tax::compute on $50k).
///   AGI ≈ $66,467.61. Std ded $15k → TI before QBI ≈ $51,467.61.
///   QBI raw = 20% × $50k = $10k.
///   TI cap = 20% × ($51,467.61 - 0) = $10,293.52. min = $10k.
#[test]
fn qbi_deduction_zero_capital_gain_baseline() {
    let res = compute(&consultant(Decimal::ZERO));
    // QBI deduction must be exactly 20% × $50k SE = $10k (TI cap > QBI).
    assert_eq!(
        res.qbi_deduction,
        d(10_000),
        "baseline QBI must be 20% × $50k = $10,000 when no cap gain present"
    );
}

/// Bug-catcher: a NET LONG-TERM CAPITAL LOSS must NOT inflate the QBI
/// TI cap. The QBI deduction with -$10k LTCG must equal the QBI
/// deduction with $0 LTCG (since IRC § 1222 floors net capital gain at
/// zero before § 199A(a)(1)(B)(ii) subtracts it from TI).
///
/// Failure mode (current bug): without a `.max(Decimal::ZERO)` on
/// `net_capital_gain` before the subtraction in `qbi::compute`,
/// the cap would compute as
///   ti_cap = 20% × (TI - (-10_000)) = 20% × (TI + 10_000)
/// which is bigger than the no-loss cap — still bigger than the raw
/// 20% × $50k = $10k QBI, so QBI deduction would stay at $10k. To
/// catch the bug, we need the TI cap to be the binding constraint in
/// BOTH cases. That requires a SMALLER QBI / SMALLER TI scenario.
///
/// So we shrink the case: SE = $50k QBI but TI before QBI ≈ $30k →
/// TI cap = 20% × $30k = $6k binds (< $10k raw QBI).
/// With $10k LOSS treated INCORRECTLY:
///   ti_cap = 20% × ($30k - (-$10k)) = $8k → QBI = $8k.
/// With $10k loss CORRECTLY floored at 0:
///   ti_cap = 20% × ($30k - 0) = $6k → QBI = $6k.
/// Δ = $2k. Test asserts QBI deduction is bounded by the correct $6k.
#[test]
fn qbi_net_capital_loss_must_not_inflate_ti_cap() {
    // To make TI cap bind, drop W-2 wages so TI is small enough
    // that 20% × TI < 20% × QBI. Use no W-2 → only SE.
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        schedule_c: ScheduleC {
            gross_receipts: d(50_000),
            total_expenses: Decimal::ZERO,
            net_profit: d(50_000),
        },
        net_long_term_capital_gain: d(-10_000), // ← net long-term LOSS
        ..Default::default()
    };
    // Sanity: pin the AGI / TI before QBI so the test's expected cap
    // arithmetic is grounded, not just asserting an inequality.
    //   SE base = 50k × 0.9235 = 46,175.
    //   SS tax  = 46,175 × 0.124 = 5,725.70.
    //   Medicare = 46,175 × 0.029 = 1,339.08.
    //   Half SE = (5,725.70 + 1,339.08) / 2 = 3,532.39.
    //   AGI cap: max(0, 50,000 - 3,532.39) = 46,467.61.
    //   Std ded = 15,000 → TI before QBI = 31,467.61.
    //   Correct TI cap = 20% × 31,467.61 = $6,293.52.
    //   Bug-form TI cap = 20% × (31,467.61 + 10,000) = $8,293.52.
    let res = compute(&r);

    // The QBI deduction MUST be the correct TI-cap binding ($6,293.52),
    // NOT the bug-inflated cap ($8,293.52).
    //
    // We compare against the no-loss equivalent — that's the strongest
    // pin: WHATEVER the engine computes for $0 cap gain, $- loss must
    // match it, because the loss should be floored to 0 before
    // subtraction.
    let baseline = compute(&TaxReturn {
        net_long_term_capital_gain: Decimal::ZERO,
        ..r.clone()
    });
    assert_eq!(
        res.qbi_deduction, baseline.qbi_deduction,
        "QBI deduction with $10k NET LTCG LOSS must equal QBI deduction with $0 cap gain — \
         IRC § 1222 floors net capital gain at zero before § 199A(a)(1)(B)(ii)"
    );
}
