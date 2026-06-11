//! Short-selling carry — what holding the short actually costs.
//!
//!   net carry rate = borrow fee − cash rate on proceeds
//!                  + dividend yield (the short pays dividends)
//!   breakeven decline = net carry × days/365
//!
//! Easy-to-borrow shorts in a high-rate regime can CARRY POSITIVE
//! (rebate beats fee + dividends); hard-to-borrow names need the
//! stock to fall just to stand still — the breakeven row is the
//! number to beat.
//!
//! Pure compute. Companion to `implied_dividend` (the borrow fee read
//! from option parity), `merger_arb`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ShortCarryInput {
    pub short_price: f64,
    pub shares: f64,
    /// Annual stock-borrow fee, %.
    pub borrow_fee_pct: f64,
    /// Annual rate earned on short proceeds (rebate), %.
    #[serde(default)]
    pub cash_rate_pct: f64,
    /// Annual dividend per share, $ (short pays it).
    #[serde(default)]
    pub annual_dividend: f64,
    pub holding_days: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShortCarryReport {
    /// Net annual carry rate, % of proceeds (positive = costs money).
    pub net_carry_rate_pct: f64,
    /// Carry over the holding period, $ and %.
    pub carry_cost: f64,
    pub carry_cost_pct: f64,
    /// Decline needed just to break even on carry, %.
    pub breakeven_decline_pct: f64,
    pub daily_cost: f64,
    /// True when the rebate beats fee + dividends — paid to be short.
    pub positive_carry: bool,
}

pub fn compute(inp: &ShortCarryInput) -> Option<ShortCarryReport> {
    if ![inp.short_price, inp.shares, inp.holding_days].iter().all(|v| v.is_finite() && *v > 0.0)
        || !inp.borrow_fee_pct.is_finite()
        || inp.borrow_fee_pct < 0.0
        || !inp.cash_rate_pct.is_finite()
        || inp.cash_rate_pct < 0.0
        || !inp.annual_dividend.is_finite()
        || inp.annual_dividend < 0.0
    {
        return None;
    }
    let div_yield = inp.annual_dividend / inp.short_price * 100.0;
    let net_rate = inp.borrow_fee_pct - inp.cash_rate_pct + div_yield;
    let proceeds = inp.short_price * inp.shares;
    let period = inp.holding_days / 365.0;
    let cost = proceeds * net_rate / 100.0 * period;
    Some(ShortCarryReport {
        net_carry_rate_pct: net_rate,
        carry_cost: cost,
        carry_cost_pct: net_rate * period,
        breakeven_decline_pct: net_rate * period,
        daily_cost: proceeds * net_rate / 100.0 / 365.0,
        positive_carry: net_rate < 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn htb_dividend_payer_hand_walk() {
        // 100 sh @ $50, 8% fee, 5% rebate, $2 dividend, 90 days:
        // net = 8 − 5 + 4 = 7%/yr ⇒ 1.726% over the period = $86.30.
        let r = compute(&ShortCarryInput {
            short_price: 50.0,
            shares: 100.0,
            borrow_fee_pct: 8.0,
            cash_rate_pct: 5.0,
            annual_dividend: 2.0,
            holding_days: 90.0,
        })
        .unwrap();
        assert!((r.net_carry_rate_pct - 7.0).abs() < 1e-12);
        assert!((r.breakeven_decline_pct - 7.0 * 90.0 / 365.0).abs() < 1e-12);
        assert!((r.carry_cost - 5000.0 * 0.07 * 90.0 / 365.0).abs() < 1e-9);
        assert!(!r.positive_carry);
    }

    #[test]
    fn easy_borrow_high_rates_carry_positive() {
        // 0.3% GC fee vs 5% rebate, no dividend: paid 4.7%/yr to be
        // short — breakeven decline is NEGATIVE.
        let r = compute(&ShortCarryInput {
            short_price: 50.0,
            shares: 100.0,
            borrow_fee_pct: 0.3,
            cash_rate_pct: 5.0,
            annual_dividend: 0.0,
            holding_days: 365.0,
        })
        .unwrap();
        assert!((r.net_carry_rate_pct + 4.7).abs() < 1e-12);
        assert!(r.positive_carry);
        assert!(r.breakeven_decline_pct < 0.0);
        assert!(r.carry_cost < 0.0);
    }

    #[test]
    fn squeeze_fee_dominates_everything() {
        // 200% borrow on a squeeze name: ~0.55%/DAY of carry.
        let r = compute(&ShortCarryInput {
            short_price: 20.0,
            shares: 500.0,
            borrow_fee_pct: 200.0,
            cash_rate_pct: 5.0,
            annual_dividend: 0.0,
            holding_days: 10.0,
        })
        .unwrap();
        assert!((r.daily_cost - 10_000.0 * 1.95 / 365.0).abs() < 1e-9);
        assert!(r.breakeven_decline_pct > 5.0); // >5% fall needed in 10 days
    }

    #[test]
    fn hostile_inputs_return_none() {
        let base = ShortCarryInput {
            short_price: 50.0,
            shares: 100.0,
            borrow_fee_pct: 8.0,
            cash_rate_pct: 5.0,
            annual_dividend: 2.0,
            holding_days: 90.0,
        };
        assert!(compute(&ShortCarryInput { short_price: 0.0, ..base.clone() }).is_none());
        assert!(compute(&ShortCarryInput { holding_days: 0.0, ..base.clone() }).is_none());
        assert!(compute(&ShortCarryInput { borrow_fee_pct: -1.0, ..base.clone() }).is_none());
        assert!(compute(&ShortCarryInput { annual_dividend: f64::NAN, ..base }).is_none());
    }
}
