//! Perpetual funding & basis — the cost of carry on a crypto perpetual-futures
//! position and its premium to spot. The funding payment per interval is
//! `notional × funding_rate` (longs pay shorts when the rate is positive); the
//! annualized funding rate scales it by the intervals per day over a year; and
//! the basis is the perp's premium over the index, `(perp − index) ÷ index`.
//! Reports who pays and the position's daily and annualized funding cost. Pure
//! compute. Not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PerpFundingInput {
    pub perp_price_usd: f64,
    pub index_price_usd: f64,
    /// Funding rate per interval, decimal (e.g. 0.0001 = 0.01%).
    pub funding_rate: f64,
    /// Funding intervals per day (e.g. 3 for 8-hour funding).
    #[serde(default = "default_intervals")]
    pub intervals_per_day: f64,
    /// Position notional in USD.
    pub position_notional_usd: f64,
    /// "long" or "short".
    pub side: String,
}

fn default_intervals() -> f64 {
    3.0
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct PerpFundingReport {
    /// (perp − index) ÷ index, percent.
    pub basis_pct: f64,
    /// Funding paid (+) or received (−) per interval for this position's side.
    pub funding_per_interval_usd: f64,
    pub funding_per_day_usd: f64,
    /// funding_rate × intervals_per_day × 365, percent.
    pub annualized_funding_pct: f64,
    /// True when this side pays funding (long & rate>0, or short & rate<0).
    pub pays_funding: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &PerpFundingInput) -> PerpFundingReport {
    let basis = if i.index_price_usd > 0.0 {
        (i.perp_price_usd - i.index_price_usd) / i.index_price_usd * 100.0
    } else {
        0.0
    };
    let is_long = !i.side.trim().eq_ignore_ascii_case("short");
    // Longs pay when the rate is positive; shorts pay when negative.
    let pays = (is_long && i.funding_rate > 0.0) || (!is_long && i.funding_rate < 0.0);
    // Signed magnitude: positive = this side pays out.
    let direction = if is_long { 1.0 } else { -1.0 };
    let per_interval = i.position_notional_usd * i.funding_rate * direction;
    let per_day = per_interval * i.intervals_per_day;
    let annualized = i.funding_rate * i.intervals_per_day * 365.0 * 100.0;

    PerpFundingReport {
        basis_pct: round4(basis),
        funding_per_interval_usd: round2(per_interval),
        funding_per_day_usd: round2(per_day),
        annualized_funding_pct: round2(annualized),
        pays_funding: pays,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> PerpFundingInput {
        PerpFundingInput {
            perp_price_usd: 30_050.0,
            index_price_usd: 30_000.0,
            funding_rate: 0.0001,
            intervals_per_day: 3.0,
            position_notional_usd: 100_000.0,
            side: "long".into(),
        }
    }

    #[test]
    fn long_pays_positive_funding() {
        let d = generate(&base());
        assert!(close(d.basis_pct, 0.1667));
        assert!(close(d.funding_per_interval_usd, 10.0));
        assert!(close(d.funding_per_day_usd, 30.0));
        assert!(close(d.annualized_funding_pct, 10.95));
        assert!(d.pays_funding);
    }

    #[test]
    fn short_receives_positive_funding() {
        let d = generate(&PerpFundingInput { side: "short".into(), ..base() });
        // Short receives → negative payment.
        assert!(close(d.funding_per_interval_usd, -10.0));
        assert!(!d.pays_funding);
    }

    #[test]
    fn negative_funding_long_receives() {
        let d = generate(&PerpFundingInput { funding_rate: -0.0001, ..base() });
        assert!(close(d.funding_per_interval_usd, -10.0));
        assert!(!d.pays_funding);
    }

    #[test]
    fn discount_basis_negative() {
        let d = generate(&PerpFundingInput { perp_price_usd: 29_900.0, ..base() });
        assert!(d.basis_pct < 0.0);
    }
}
