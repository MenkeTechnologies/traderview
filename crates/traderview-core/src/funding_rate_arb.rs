//! Perpetual funding-rate arbitrage (crypto cash-and-carry).
//!
//! Perp funding transfers money between longs and shorts every 8 hours
//! to pin the perp to spot. Positive funding = longs pay shorts, so a
//! delta-neutral LONG-spot + SHORT-perp position collects it; negative
//! funding reverses the legs. The economics:
//!
//!   funding APR  = |rate_8h| × 3 × 365
//!   daily income = |rate_8h| × 3 × notional
//!   fees         = 4 taker legs (enter + exit, both legs)
//!   breakeven    = fees / daily income
//!
//! The basis (perp − spot) is reported as a SEPARATE edge, not summed
//! into net PnL: a perp has no expiry forcing convergence, so the
//! basis is captured only if it actually reverts.
//!
//! Pure compute. Caller supplies live prices and the venue's rates.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub spot_price: f64,
    pub perp_price: f64,
    /// Signed funding rate per 8h interval as a decimal (0.0001 = 1bp).
    pub funding_rate_8h: f64,
    pub notional_usd: f64,
    /// Taker fee per leg, percent (0.05 = 5bp).
    pub taker_fee_pct: f64,
    pub days_held: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// "long_spot_short_perp" (positive funding), the reverse, or
    /// "no_edge" at exactly zero funding.
    pub direction: &'static str,
    pub basis_pct: f64,
    pub funding_apr_pct: f64,
    pub daily_funding_usd: f64,
    pub total_fees_usd: f64,
    /// Days of funding income to cover all four fee legs.
    pub breakeven_days: Option<f64>,
    pub funding_income_usd: f64,
    /// Income − fees over days_held. Basis NOT included.
    pub net_pnl_usd: f64,
    /// |basis| × notional — captured only if the basis converges.
    pub basis_edge_usd: f64,
}

pub fn compute(i: &Input) -> Option<Report> {
    if i.spot_price <= 0.0
        || i.perp_price <= 0.0
        || i.notional_usd <= 0.0
        || i.taker_fee_pct < 0.0
        || i.days_held < 0.0
        || !i.funding_rate_8h.is_finite()
    {
        return None;
    }
    let rate = i.funding_rate_8h.abs();
    let direction = if i.funding_rate_8h > 0.0 {
        "long_spot_short_perp"
    } else if i.funding_rate_8h < 0.0 {
        "short_spot_long_perp"
    } else {
        "no_edge"
    };
    let basis_pct = (i.perp_price / i.spot_price - 1.0) * 100.0;
    let daily_funding_usd = rate * 3.0 * i.notional_usd;
    let total_fees_usd = 4.0 * (i.taker_fee_pct / 100.0) * i.notional_usd;
    let funding_income_usd = daily_funding_usd * i.days_held;
    Some(Report {
        direction,
        basis_pct,
        funding_apr_pct: rate * 3.0 * 365.0 * 100.0,
        daily_funding_usd,
        total_fees_usd,
        breakeven_days: (daily_funding_usd > 0.0).then(|| total_fees_usd / daily_funding_usd),
        funding_income_usd,
        net_pnl_usd: funding_income_usd - total_fees_usd,
        basis_edge_usd: (basis_pct / 100.0).abs() * i.notional_usd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            spot_price: 50_000.0,
            perp_price: 50_100.0,
            funding_rate_8h: 0.0001, // 1bp per 8h
            notional_usd: 100_000.0,
            taker_fee_pct: 0.05,
            days_held: 30.0,
        }
    }

    #[test]
    fn positive_funding_pins_the_whole_ledger() {
        let r = compute(&base()).unwrap();
        assert_eq!(r.direction, "long_spot_short_perp");
        // 1bp × 3 intervals × $100k = $30/day.
        assert!((r.daily_funding_usd - 30.0).abs() < 1e-9);
        // APR = 0.0001 × 3 × 365 = 10.95%.
        assert!((r.funding_apr_pct - 10.95).abs() < 1e-9);
        // 4 taker legs × 5bp × $100k = $200.
        assert!((r.total_fees_usd - 200.0).abs() < 1e-9);
        // $200 / $30 per day = 6.667 days to breakeven.
        assert!((r.breakeven_days.unwrap() - 200.0 / 30.0).abs() < 1e-9);
        // 30 days: $900 income, $700 net after fees.
        assert!((r.funding_income_usd - 900.0).abs() < 1e-9);
        assert!((r.net_pnl_usd - 700.0).abs() < 1e-9);
        // Basis 100/50000 = 0.2% → $200 edge, reported separately.
        assert!((r.basis_pct - 0.2).abs() < 1e-9);
        assert!((r.basis_edge_usd - 200.0).abs() < 1e-9);
    }

    #[test]
    fn negative_funding_flips_legs_same_magnitudes() {
        let r = compute(&Input {
            funding_rate_8h: -0.0001,
            ..base()
        })
        .unwrap();
        assert_eq!(r.direction, "short_spot_long_perp");
        assert!((r.daily_funding_usd - 30.0).abs() < 1e-9);
        assert!((r.net_pnl_usd - 700.0).abs() < 1e-9);
    }

    #[test]
    fn zero_funding_is_no_edge_with_no_breakeven() {
        let r = compute(&Input {
            funding_rate_8h: 0.0,
            ..base()
        })
        .unwrap();
        assert_eq!(r.direction, "no_edge");
        assert_eq!(r.breakeven_days, None);
        // Fees still real: net is NEGATIVE fees, not zero.
        assert!((r.net_pnl_usd + 200.0).abs() < 1e-9);
    }

    #[test]
    fn degenerate_inputs_are_none() {
        assert!(compute(&Input { spot_price: 0.0, ..base() }).is_none());
        assert!(compute(&Input { notional_usd: -1.0, ..base() }).is_none());
        assert!(compute(&Input { taker_fee_pct: -0.01, ..base() }).is_none());
        assert!(compute(&Input { funding_rate_8h: f64::NAN, ..base() }).is_none());
    }
}
