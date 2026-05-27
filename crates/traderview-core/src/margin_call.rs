//! Margin call distance calculator (Reg-T maintenance requirement).
//!
//! Standard Reg-T maintenance is 25% — if the equity in the account
//! drops below 25% of the long market value, the broker issues a margin
//! call. This calc tells the user "your account triggers at price $X"
//! so they know how much room they have before the call.
//!
//! Formula: maintenance margin call when
//!     equity / market_value < 0.25
//!     where equity = market_value - debt
//! Solve for market_value: market_value > debt / 0.75
//! Per share for a single position: trigger_price = debt / (qty × 0.75).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSnapshot {
    /// Long market value of all positions in dollars.
    pub long_market_value: Decimal,
    /// Margin debt outstanding (what the broker lent you).
    pub margin_debt: Decimal,
    /// Maintenance requirement as a decimal. 0.25 standard, 0.30
    /// for many small-caps, 0.40-0.50 for restricted symbols.
    pub maintenance_pct: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginCallReport {
    /// Current equity (LMV − debt).
    pub current_equity: Decimal,
    /// Current equity / LMV. Watch for this dropping toward
    /// maintenance_pct.
    pub current_equity_pct: f64,
    /// Drop the long market value can take before triggering a call.
    /// Negative means you're already in margin-call territory.
    pub dollar_cushion: Decimal,
    /// Percent the long market value can drop before the call.
    pub pct_cushion: f64,
    /// Already in call?
    pub in_call: bool,
}

pub fn evaluate(snap: &AccountSnapshot) -> MarginCallReport {
    let mut r = MarginCallReport::default();
    r.current_equity = snap.long_market_value - snap.margin_debt;

    if snap.long_market_value.is_zero() {
        // No positions — can't be in margin call.
        return r;
    }
    let one_minus_maint = Decimal::ONE - snap.maintenance_pct;
    if one_minus_maint.is_zero() {
        // 100% maintenance — cash-only account, no margin loan tolerated.
        r.dollar_cushion = -snap.margin_debt;
        r.in_call = snap.margin_debt > Decimal::ZERO;
        return r;
    }
    // LMV at trigger = margin_debt / (1 − maintenance_pct).
    let trigger_lmv = snap.margin_debt / one_minus_maint;
    r.dollar_cushion = snap.long_market_value - trigger_lmv;
    let cushion_f = to_f64(r.dollar_cushion);
    let lmv_f = to_f64(snap.long_market_value);
    r.pct_cushion = if lmv_f > 0.0 { cushion_f / lmv_f } else { 0.0 };
    r.current_equity_pct =
        to_f64(r.current_equity) / lmv_f;
    r.in_call = r.dollar_cushion < Decimal::ZERO;
    r
}

fn to_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    #[test]
    fn fully_cash_account_has_no_margin_call() {
        let r = evaluate(&AccountSnapshot {
            long_market_value: d("50000"),
            margin_debt: Decimal::ZERO,
            maintenance_pct: d("0.25"),
        });
        assert!(!r.in_call);
        assert_eq!(r.dollar_cushion, d("50000"));
    }

    #[test]
    fn standard_25_pct_maintenance_call_threshold() {
        // $100k LMV, $60k margin debt, 25% maintenance.
        // Trigger LMV = 60k / 0.75 = $80,000.
        // Cushion = 100k - 80k = $20,000.
        let r = evaluate(&AccountSnapshot {
            long_market_value: d("100000"),
            margin_debt: d("60000"),
            maintenance_pct: d("0.25"),
        });
        assert!(!r.in_call);
        assert_eq!(r.dollar_cushion, d("20000"));
    }

    #[test]
    fn in_call_when_equity_already_below_maintenance() {
        // $100k LMV, $80k debt → equity $20k = 20%, below 25% maintenance.
        let r = evaluate(&AccountSnapshot {
            long_market_value: d("100000"),
            margin_debt: d("80000"),
            maintenance_pct: d("0.25"),
        });
        assert!(r.in_call);
        assert!(r.dollar_cushion < Decimal::ZERO);
    }

    #[test]
    fn exactly_at_maintenance_is_not_yet_in_call() {
        // LMV $100k, debt $75k → equity $25k = 25%, at exactly the
        // maintenance line. Cushion is zero, not negative.
        let r = evaluate(&AccountSnapshot {
            long_market_value: d("100000"),
            margin_debt: d("75000"),
            maintenance_pct: d("0.25"),
        });
        assert_eq!(r.dollar_cushion, Decimal::ZERO);
        assert!(!r.in_call);
    }

    #[test]
    fn higher_maintenance_pct_shrinks_cushion() {
        // Same numbers, 40% maintenance (small-cap) → trigger LMV =
        // 60k / 0.60 = $100k, cushion = 0.
        let r = evaluate(&AccountSnapshot {
            long_market_value: d("100000"),
            margin_debt: d("60000"),
            maintenance_pct: d("0.40"),
        });
        assert_eq!(r.dollar_cushion, Decimal::ZERO);
    }

    #[test]
    fn cash_only_account_with_debt_is_in_call() {
        // 100% maintenance requirement = no margin allowed. Any debt
        // triggers a call.
        let r = evaluate(&AccountSnapshot {
            long_market_value: d("50000"),
            margin_debt: d("1"),
            maintenance_pct: Decimal::ONE,
        });
        assert!(r.in_call);
    }

    #[test]
    fn zero_lmv_returns_clean_no_call_when_no_debt() {
        let r = evaluate(&AccountSnapshot {
            long_market_value: Decimal::ZERO,
            margin_debt: Decimal::ZERO,
            maintenance_pct: d("0.25"),
        });
        assert!(!r.in_call);
        assert_eq!(r.current_equity, Decimal::ZERO);
    }

    #[test]
    fn pct_cushion_reflects_drop_room() {
        let r = evaluate(&AccountSnapshot {
            long_market_value: d("100000"),
            margin_debt: d("60000"),
            maintenance_pct: d("0.25"),
        });
        // $20k cushion / $100k LMV = 20%.
        assert!((r.pct_cushion - 0.20).abs() < 1e-9);
    }
}
