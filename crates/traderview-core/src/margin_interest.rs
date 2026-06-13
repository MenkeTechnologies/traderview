//! Margin interest — the carry cost of a brokerage margin loan and the
//! break-even return needed to cover it.
//!
//! Brokers accrue margin interest daily on a 360-day basis:
//!
//! ```text
//! daily interest  = borrowed × rate / 360
//! interest cost   = borrowed × rate × days / 360
//! break-even return = interest cost / position value
//! ```
//!
//! Leverage (position / own cash) amplifies both gains and the drag from this
//! carry. Distinct from `margin-call` (the call price) and `margin-runway`
//! (time to a call).

use serde::{Deserialize, Serialize};

fn d_day_count() -> f64 {
    360.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarginInterestInput {
    pub own_cash_usd: f64,
    pub borrowed_amount_usd: f64,
    /// Annual margin rate, percent.
    pub margin_rate_pct: f64,
    pub days_held: f64,
    /// Day-count basis (brokers use 360).
    #[serde(default = "d_day_count")]
    pub day_count: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MarginInterestResult {
    /// own cash + borrowed.
    pub position_value_usd: f64,
    /// position value / own cash.
    pub leverage: Option<f64>,
    pub daily_interest_usd: f64,
    /// Interest over the holding period.
    pub interest_cost_usd: f64,
    pub annual_interest_usd: f64,
    /// Total-position return needed just to cover the interest, percent.
    pub breakeven_return_pct: Option<f64>,
}

pub fn analyze(input: &MarginInterestInput) -> MarginInterestResult {
    let position = input.own_cash_usd + input.borrowed_amount_usd;
    let dc = if input.day_count > 0.0 { input.day_count } else { 360.0 };
    let annual_rate = input.margin_rate_pct / 100.0;

    let daily = input.borrowed_amount_usd * annual_rate / dc;
    let interest_cost = daily * input.days_held;
    let annual = input.borrowed_amount_usd * annual_rate;

    MarginInterestResult {
        position_value_usd: position,
        leverage: if input.own_cash_usd > 0.0 {
            Some(position / input.own_cash_usd)
        } else {
            None
        },
        daily_interest_usd: daily,
        interest_cost_usd: interest_cost,
        annual_interest_usd: annual,
        breakeven_return_pct: if position > 0.0 {
            Some(interest_cost / position * 100.0)
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> MarginInterestInput {
        MarginInterestInput {
            own_cash_usd: 10_000.0,
            borrowed_amount_usd: 10_000.0,
            margin_rate_pct: 8.0,
            days_held: 30.0,
            day_count: 360.0,
        }
    }

    #[test]
    fn position_and_leverage() {
        let r = analyze(&base());
        assert!(close(r.position_value_usd, 20_000.0));
        assert!(close(r.leverage.unwrap(), 2.0));
    }

    #[test]
    fn daily_interest() {
        // 10,000 × 0.08 / 360 = 2.2222.
        assert!(close(analyze(&base()).daily_interest_usd, 2.2222));
    }

    #[test]
    fn interest_cost_over_period() {
        // 2.2222 × 30 = 66.6667.
        assert!(close(analyze(&base()).interest_cost_usd, 66.6667));
    }

    #[test]
    fn annual_interest() {
        assert!(close(analyze(&base()).annual_interest_usd, 800.0));
    }

    #[test]
    fn breakeven_return() {
        // 66.6667 / 20,000 = 0.3333%.
        assert!(close(analyze(&base()).breakeven_return_pct.unwrap(), 0.333333));
    }

    #[test]
    fn more_borrowed_more_interest() {
        let low = analyze(&base());
        let high = analyze(&MarginInterestInput {
            borrowed_amount_usd: 30_000.0,
            ..base()
        });
        assert!(high.interest_cost_usd > low.interest_cost_usd);
        assert!(high.leverage.unwrap() > low.leverage.unwrap());
    }

    #[test]
    fn longer_hold_more_interest() {
        let short = analyze(&base());
        let long = analyze(&MarginInterestInput {
            days_held: 365.0,
            ..base()
        });
        assert!(long.interest_cost_usd > short.interest_cost_usd);
    }

    #[test]
    fn no_borrowing_no_interest() {
        let r = analyze(&MarginInterestInput {
            borrowed_amount_usd: 0.0,
            ..base()
        });
        assert!(close(r.interest_cost_usd, 0.0));
        assert!(close(r.leverage.unwrap(), 1.0));
    }
}
