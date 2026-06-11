//! Odd-lot tender arbitrage — the small-account special.
//!
//! Tenders routinely grant ODD-LOT PRIORITY: holders of fewer than
//! 100 shares are taken in full ahead of proration. Buying 99 shares
//! into a tender above market locks:
//!
//!   profit = (tender − price) × shares − fees
//!
//! Tiny in dollars, large annualized, and it scales across accounts —
//! hence the per-account framing. Proration risk applies only above
//! the odd-lot cap.
//!
//! Pure compute. Companion to `merger_arb` (same expected-value
//! skeleton for the deal-break leg).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct OddLotInput {
    pub market_price: f64,
    pub tender_price: f64,
    /// Shares to buy (odd-lot priority caps at 99).
    pub shares: f64,
    /// Round-trip commissions/fees per account, $.
    #[serde(default)]
    pub fees: f64,
    pub days_to_payment: f64,
    /// Number of accounts running the same odd lot.
    #[serde(default)]
    pub accounts: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OddLotReport {
    pub capital_per_account: f64,
    pub profit_per_account: f64,
    pub return_pct: f64,
    pub annualized_pct: f64,
    /// True when shares ≤ 99 (odd-lot priority — no proration).
    pub odd_lot_priority: bool,
    pub total_profit: f64,
}

pub fn compute(inp: &OddLotInput) -> Option<OddLotReport> {
    if ![inp.market_price, inp.tender_price, inp.shares].iter().all(|v| v.is_finite() && *v > 0.0)
        || !inp.fees.is_finite()
        || inp.fees < 0.0
        || !inp.days_to_payment.is_finite()
        || inp.days_to_payment <= 0.0
        || inp.accounts.map(|a| !a.is_finite() || a < 1.0) == Some(true)
    {
        return None;
    }
    let capital = inp.market_price * inp.shares + inp.fees;
    let profit = (inp.tender_price - inp.market_price) * inp.shares - inp.fees;
    let ret = profit / capital * 100.0;
    let accounts = inp.accounts.unwrap_or(1.0).floor();
    Some(OddLotReport {
        capital_per_account: capital,
        profit_per_account: profit,
        return_pct: ret,
        annualized_pct: ret * 365.0 / inp.days_to_payment,
        odd_lot_priority: inp.shares <= 99.0,
        total_profit: profit * accounts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> OddLotInput {
        OddLotInput {
            market_price: 19.0,
            tender_price: 20.0,
            shares: 99.0,
            fees: 1.0,
            days_to_payment: 36.5,
            accounts: Some(3.0),
        }
    }

    #[test]
    fn odd_lot_hand_walk() {
        // 99 × $1 − $1 fees = $98 on $1882 capital = 5.207%;
        // 36.5 days ⇒ ×10 = 52.07% annualized; 3 accounts = $294.
        let r = compute(&base()).unwrap();
        assert!((r.profit_per_account - 98.0).abs() < 1e-12);
        assert!((r.capital_per_account - 1882.0).abs() < 1e-12);
        assert!((r.return_pct - 98.0 / 1882.0 * 100.0).abs() < 1e-9);
        assert!((r.annualized_pct - r.return_pct * 10.0).abs() < 1e-9);
        assert!(r.odd_lot_priority);
        assert!((r.total_profit - 294.0).abs() < 1e-12);
    }

    #[test]
    fn round_lot_loses_priority() {
        let mut inp = base();
        inp.shares = 100.0;
        let r = compute(&inp).unwrap();
        assert!(!r.odd_lot_priority);
    }

    #[test]
    fn fees_can_eat_the_spread() {
        let mut inp = base();
        inp.fees = 150.0;
        let r = compute(&inp).unwrap();
        assert!(r.profit_per_account < 0.0);
        assert!(r.return_pct < 0.0);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let mut bad = base();
        bad.market_price = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.days_to_payment = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.accounts = Some(0.0);
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.tender_price = f64::NAN;
        assert!(compute(&bad).is_none());
    }
}
