//! Holding-period return — the total return on a position, split into the
//! price change and the income (dividends, interest, coupons) collected while
//! held, then annualized over the actual number of days held.
//!
//! ```text
//! total return = (sell − buy + income) / buy
//! annualized   = (1 + total return)^(365 / days) − 1     (geometric)
//! ```
//!
//! The annualized figure compounds the holding-period return to a one-year
//! basis, so a 25% gain over two years annualizes to ~11.8%, and the same gain
//! over six months annualizes to ~56.25%.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct HprInput {
    /// Purchase price per share.
    pub buy_price: f64,
    /// Sale price per share.
    pub sell_price: f64,
    /// Income received per share over the hold (dividends, interest).
    #[serde(default)]
    pub income_per_share: f64,
    /// Days the position was held.
    pub holding_days: f64,
    /// Shares held, for the dollar P&L. Optional.
    #[serde(default)]
    pub shares: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HprResult {
    /// (sell − buy) / buy, percent.
    pub price_return_pct: f64,
    /// income / buy, percent.
    pub income_return_pct: f64,
    /// (sell − buy + income) / buy, percent.
    pub total_return_pct: f64,
    /// holding_days / 365.
    pub holding_years: f64,
    /// Geometric annualized return, percent; `None` if it can't be defined
    /// (no holding period, or a total loss of more than 100%).
    pub annualized_return_pct: Option<f64>,
    /// Total dollar profit/loss (shares × per-share gain incl. income).
    pub total_pl_usd: f64,
}

pub fn analyze(input: &HprInput) -> HprResult {
    // Guard a non-positive buy price — returns are undefined against it.
    if input.buy_price <= 0.0 {
        return HprResult {
            price_return_pct: 0.0,
            income_return_pct: 0.0,
            total_return_pct: 0.0,
            holding_years: input.holding_days / 365.0,
            annualized_return_pct: None,
            total_pl_usd: input.shares * (input.sell_price - input.buy_price + input.income_per_share),
        };
    }

    let price_return = (input.sell_price - input.buy_price) / input.buy_price;
    let income_return = input.income_per_share / input.buy_price;
    let total_return = price_return + income_return;

    let years = input.holding_days / 365.0;
    let growth = 1.0 + total_return;
    let annualized = if years > 0.0 && growth > 0.0 {
        Some((growth.powf(1.0 / years) - 1.0) * 100.0)
    } else {
        None
    };

    HprResult {
        price_return_pct: price_return * 100.0,
        income_return_pct: income_return * 100.0,
        total_return_pct: total_return * 100.0,
        holding_years: years,
        annualized_return_pct: annualized,
        total_pl_usd: input.shares * (input.sell_price - input.buy_price + input.income_per_share),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(buy: f64, sell: f64, income: f64, days: f64) -> HprResult {
        analyze(&HprInput {
            buy_price: buy,
            sell_price: sell,
            income_per_share: income,
            holding_days: days,
            shares: 0.0,
        })
    }

    #[test]
    fn price_and_income_return() {
        let r = run(100.0, 120.0, 5.0, 365.0);
        assert!(close(r.price_return_pct, 20.0));
        assert!(close(r.income_return_pct, 5.0));
        assert!(close(r.total_return_pct, 25.0));
    }

    #[test]
    fn annualized_one_year_equals_total() {
        let r = run(100.0, 120.0, 5.0, 365.0);
        assert!(close(r.annualized_return_pct.unwrap(), 25.0));
        assert!(close(r.holding_years, 1.0));
    }

    #[test]
    fn annualized_two_years_compounds_down() {
        // 25% over 2 years → (1.25)^(1/2) − 1 = 11.8034%.
        let r = run(100.0, 120.0, 5.0, 730.0);
        assert!(close(r.annualized_return_pct.unwrap(), 11.803399));
    }

    #[test]
    fn annualized_half_year_compounds_up() {
        // 25% over 182.5 days → (1.25)^2 − 1 = 56.25%.
        let r = run(100.0, 120.0, 5.0, 182.5);
        assert!(close(r.annualized_return_pct.unwrap(), 56.25));
    }

    #[test]
    fn loss_annualizes() {
        // −20% over 2 years → (0.8)^(1/2) − 1 = −10.5573%.
        let r = run(100.0, 80.0, 0.0, 730.0);
        assert!(close(r.total_return_pct, -20.0));
        assert!(close(r.annualized_return_pct.unwrap(), -10.557281));
    }

    #[test]
    fn dollar_pl_with_shares() {
        let r = analyze(&HprInput {
            buy_price: 100.0,
            sell_price: 120.0,
            income_per_share: 5.0,
            holding_days: 365.0,
            shares: 10.0,
        });
        // 10 × (120 − 100 + 5) = 250.
        assert!(close(r.total_pl_usd, 250.0));
    }

    #[test]
    fn zero_days_has_no_annualized() {
        let r = run(100.0, 120.0, 5.0, 0.0);
        assert!(r.annualized_return_pct.is_none());
        assert!(close(r.total_return_pct, 25.0));
    }

    #[test]
    fn total_loss_has_no_annualized() {
        // Worse than −100% can't be geometrically annualized.
        let r = run(100.0, 0.0, 0.0, 730.0);
        assert!(close(r.total_return_pct, -100.0));
        assert!(r.annualized_return_pct.is_none());
    }
}
