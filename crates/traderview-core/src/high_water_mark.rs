//! High-water-mark + incentive-fee tracker.
//!
//! Standard prop / family-and-friends fee structure: the manager earns
//! `incentive_fee_pct` (e.g. 20%) of any equity GROWTH above the
//! historical high-water mark. If equity drops, the HWM stays — the
//! manager must claw back losses before earning new incentive fees.
//!
//! `mgmt_fee_pct` is the flat annual management fee (e.g. 2%),
//! accrued daily and charged at period end.
//!
//! Computes both fees per period + cumulative + new HWM at period end.
//! Pure compute.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeeRates {
    /// Annual management fee as a decimal (0.02 = 2%).
    pub mgmt_fee_pct: f64,
    /// Performance fee as a decimal (0.20 = 20%).
    pub incentive_fee_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodInput {
    /// Equity at the start of the period.
    pub start_equity: Decimal,
    /// Gross equity at the END of the period — before any fee deductions
    /// in this period. Caller does any adjustment for deposits/withdrawals
    /// upstream so this is the pure trading result.
    pub end_equity: Decimal,
    /// Fraction of a year this period covers (e.g. monthly = 1/12).
    pub year_fraction: f64,
    /// HWM at the START of the period — initially equal to the account's
    /// initial funding, then updated each period.
    pub start_hwm: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeriodFee {
    pub mgmt_fee: Decimal,
    pub incentive_fee: Decimal,
    pub total_fee: Decimal,
    /// Net equity after fees deducted.
    pub net_equity: Decimal,
    /// HWM at the END of the period — equals start_hwm OR net_equity if
    /// the latter is higher.
    pub end_hwm: Decimal,
    /// True iff the incentive fee was earned this period (net equity
    /// crossed above the HWM).
    pub incentive_earned: bool,
}

pub fn compute(input: &PeriodInput, rates: &FeeRates) -> PeriodFee {
    let mgmt_pct = Decimal::from_str(&format!("{}", rates.mgmt_fee_pct * input.year_fraction))
        .unwrap_or(Decimal::ZERO);
    // Standard convention: management fee is charged on END-OF-PERIOD
    // equity (most common in the industry, slightly less common is
    // average-equity which we don't model).
    let mgmt_fee = input.end_equity * mgmt_pct;

    // Equity after management fee, BEFORE incentive fee — this is what
    // gets compared against the HWM.
    let after_mgmt = input.end_equity - mgmt_fee;

    let excess = after_mgmt - input.start_hwm;
    let (incentive_fee, earned) = if excess > Decimal::ZERO {
        let pct = Decimal::from_str(&format!("{}", rates.incentive_fee_pct))
            .unwrap_or(Decimal::ZERO);
        (excess * pct, true)
    } else {
        (Decimal::ZERO, false)
    };

    let total = mgmt_fee + incentive_fee;
    let net = input.end_equity - total;
    // New HWM = max(start_hwm, net). HWM moves up only on net basis —
    // never down.
    let end_hwm = if net > input.start_hwm { net } else { input.start_hwm };

    PeriodFee {
        mgmt_fee,
        incentive_fee,
        total_fee: total,
        net_equity: net,
        end_hwm,
        incentive_earned: earned,
    }
}

/// Helper: roll a sequence of PeriodInputs through compute(), threading
/// the HWM from one period to the next. Returns per-period fees + the
/// final HWM. Caller supplies the very first start_hwm.
pub fn roll(periods: &[(PeriodInput, FeeRates)]) -> Vec<PeriodFee> {
    let mut hwm = periods.first().map(|p| p.0.start_hwm).unwrap_or(Decimal::ZERO);
    periods.iter().map(|(p_input, rates)| {
        let local = PeriodInput { start_hwm: hwm, ..p_input.clone() };
        let f = compute(&local, rates);
        hwm = f.end_hwm;
        f
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn rates() -> FeeRates {
        FeeRates { mgmt_fee_pct: 0.02, incentive_fee_pct: 0.20 }
    }

    #[test]
    fn flat_period_charges_only_mgmt_fee() {
        // Started + ended at 100k, no growth.
        let r = compute(&PeriodInput {
            start_equity: d("100000"),
            end_equity:   d("100000"),
            year_fraction: 1.0/12.0,
            start_hwm: d("100000"),
        }, &rates());
        // Mgmt = 100k × 0.02 × 1/12 ≈ $166.67.
        assert!((to_f64(r.mgmt_fee) - 166.666667).abs() < 0.01);
        assert_eq!(r.incentive_fee, Decimal::ZERO);
        assert!(!r.incentive_earned, "no excess over HWM → no incentive");
    }

    #[test]
    fn winning_period_above_hwm_charges_incentive() {
        // 100k → 110k in one month.
        // mgmt = 110000 × 0.02/12 ≈ $183.33 (charged on END equity).
        // after_mgmt = 110000 - 183.33 = $109,816.67.
        // excess over HWM 100k = $9,816.67.
        // incentive = 20% × $9,816.67 ≈ $1,963.33.
        let r = compute(&PeriodInput {
            start_equity: d("100000"),
            end_equity:   d("110000"),
            year_fraction: 1.0/12.0,
            start_hwm: d("100000"),
        }, &rates());
        assert!(r.incentive_earned);
        assert!(r.incentive_fee > Decimal::ZERO);
        assert!((to_f64(r.incentive_fee) - 1963.333333).abs() < 0.5,
            "got incentive_fee = {}", r.incentive_fee);
        // New HWM = net equity (above old HWM).
        assert!(r.end_hwm > d("100000"));
    }

    #[test]
    fn losing_period_below_hwm_keeps_old_hwm_no_incentive() {
        // 100k → 95k loss. No incentive. HWM stays at 100k.
        let r = compute(&PeriodInput {
            start_equity: d("100000"),
            end_equity:   d("95000"),
            year_fraction: 1.0/12.0,
            start_hwm: d("100000"),
        }, &rates());
        assert_eq!(r.incentive_fee, Decimal::ZERO);
        assert!(!r.incentive_earned);
        assert_eq!(r.end_hwm, d("100000"), "HWM never moves down");
    }

    #[test]
    fn recovery_below_old_hwm_charges_no_incentive() {
        // Down month: HWM=100k, account at 95k. Next month recovers to 99k —
        // gain of $4k but STILL below HWM → no incentive yet.
        let r = compute(&PeriodInput {
            start_equity: d("95000"),
            end_equity:   d("99000"),
            year_fraction: 1.0/12.0,
            start_hwm: d("100000"),
        }, &rates());
        assert!(!r.incentive_earned,
            "manager must claw back to HWM before earning new incentive");
        assert_eq!(r.end_hwm, d("100000"));
    }

    #[test]
    fn hwm_advances_between_periods_and_incentive_compounds() {
        // Period 1: 100k → 110k → HWM advances above 100k (to net after fees).
        // Period 2: 110k → 120k → fresh incentive only on excess above NEW HWM.
        let r1 = compute(&PeriodInput {
            start_equity: d("100000"),
            end_equity:   d("110000"),
            year_fraction: 1.0/12.0,
            start_hwm: d("100000"),
        }, &rates());
        // r1.end_hwm = net equity p1 ≈ $107,853.33.
        assert!(r1.end_hwm > d("107000") && r1.end_hwm < d("108000"),
            "HWM after p1 should be net equity ~107,853 but was {}", r1.end_hwm);
        let r2 = compute(&PeriodInput {
            start_equity: d("110000"),
            end_equity:   d("120000"),
            year_fraction: 1.0/12.0,
            start_hwm: r1.end_hwm,
        }, &rates());
        // p2: mgmt = 120000 × 0.0016667 = $200.
        // after_mgmt = $119,800. excess vs $107,853 = $11,946.67.
        // incentive = 20% × $11,946.67 = $2,389.33.
        assert!(r2.incentive_earned);
        assert!((to_f64(r2.incentive_fee) - 2389.333333).abs() < 1.0,
            "got incentive_fee = {}", r2.incentive_fee);
        // HWM keeps climbing.
        assert!(r2.end_hwm > r1.end_hwm,
            "HWM must monotonically increase on winning periods");
    }

    #[test]
    fn roll_threads_hwm_across_periods() {
        let periods = vec![
            (PeriodInput {
                start_equity: d("100000"),
                end_equity:   d("110000"),
                year_fraction: 1.0/12.0,
                start_hwm: d("100000"),
            }, rates()),
            (PeriodInput {
                start_equity: d("110000"),
                end_equity:   d("105000"),    // down
                year_fraction: 1.0/12.0,
                start_hwm: d("0"),            // ignored — roll overrides
            }, rates()),
            (PeriodInput {
                start_equity: d("105000"),
                end_equity:   d("115000"),    // recover but still vs old HWM
                year_fraction: 1.0/12.0,
                start_hwm: d("0"),
            }, rates()),
        ];
        let out = roll(&periods);
        assert_eq!(out.len(), 3);
        assert!(out[0].incentive_earned, "p1: gained above starting HWM");
        assert!(!out[1].incentive_earned, "p2: lost below HWM, no incentive");
        // p3: end at 115k, HWM from p1 ≈ $109,833. After mgmt, still > HWM.
        assert!(out[2].incentive_earned, "p3: recovered above HWM");
    }

    #[test]
    fn zero_year_fraction_charges_no_mgmt_fee() {
        // year_fraction=0 → mgmt fee = 0 regardless of mgmt_fee_pct.
        let r = compute(&PeriodInput {
            start_equity: d("100000"),
            end_equity:   d("100000"),
            year_fraction: 0.0,
            start_hwm: d("100000"),
        }, &rates());
        assert_eq!(r.mgmt_fee, Decimal::ZERO);
    }

    fn to_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
}
