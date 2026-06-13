//! Preferred stock valuation — a perpetuity. A preferred share pays a fixed
//! dividend forever, so its fair value is just the dividend divided by the
//! required yield:
//!
//! ```text
//! annual dividend = par × dividend rate
//! fair value      = annual dividend / required yield
//! current yield   = annual dividend / market price
//! ```
//!
//! Distinct from `bond-pricing` (finite maturity) and the dividend-discount
//! model (growing common-stock dividends).

use serde::{Deserialize, Serialize};

fn d_par() -> f64 {
    100.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreferredInput {
    #[serde(default = "d_par")]
    pub par_value_usd: f64,
    /// Fixed dividend rate on par, percent.
    pub dividend_rate_pct: f64,
    /// Investor's required yield, percent.
    pub required_yield_pct: f64,
    /// Current market price, for the current yield. Optional.
    #[serde(default)]
    pub market_price_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PreferredResult {
    /// par × dividend rate.
    pub annual_dividend_usd: f64,
    /// Dividend / required yield.
    pub fair_value_usd: Option<f64>,
    /// Dividend / market price, percent; `None` if no price given.
    pub current_yield_pct: Option<f64>,
    /// "undervalued", "overvalued", or "fair" vs the market price.
    pub verdict: Option<String>,
}

pub fn analyze(input: &PreferredInput) -> PreferredResult {
    let dividend = input.par_value_usd * input.dividend_rate_pct / 100.0;

    let fair_value = if input.required_yield_pct > 0.0 {
        Some(dividend / (input.required_yield_pct / 100.0))
    } else {
        None
    };

    let (current_yield, verdict) = if input.market_price_usd > 0.0 {
        let cy = dividend / input.market_price_usd * 100.0;
        let v = fair_value.map(|fv| {
            if (input.market_price_usd - fv).abs() < 1e-6 {
                "fair"
            } else if input.market_price_usd < fv {
                "undervalued"
            } else {
                "overvalued"
            }
            .to_string()
        });
        (Some(cy), v)
    } else {
        (None, None)
    };

    PreferredResult {
        annual_dividend_usd: dividend,
        fair_value_usd: fair_value,
        current_yield_pct: current_yield,
        verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> PreferredInput {
        PreferredInput {
            par_value_usd: 100.0,
            dividend_rate_pct: 6.0,
            required_yield_pct: 5.0,
            market_price_usd: 110.0,
        }
    }

    #[test]
    fn annual_dividend() {
        assert!(close(analyze(&base()).annual_dividend_usd, 6.0));
    }

    #[test]
    fn fair_value() {
        // 6 / 0.05 = 120.
        assert!(close(analyze(&base()).fair_value_usd.unwrap(), 120.0));
    }

    #[test]
    fn current_yield() {
        // 6 / 110 = 5.4545%.
        assert!(close(analyze(&base()).current_yield_pct.unwrap(), 5.454545));
    }

    #[test]
    fn undervalued_below_fair_value() {
        // Market 110 < fair 120.
        assert_eq!(analyze(&base()).verdict.unwrap(), "undervalued");
    }

    #[test]
    fn overvalued_above_fair_value() {
        let r = analyze(&PreferredInput {
            market_price_usd: 130.0,
            ..base()
        });
        assert_eq!(r.verdict.unwrap(), "overvalued");
    }

    #[test]
    fn higher_required_yield_lowers_value() {
        let low = analyze(&base());
        let high = analyze(&PreferredInput {
            required_yield_pct: 8.0,
            ..base()
        });
        assert!(high.fair_value_usd.unwrap() < low.fair_value_usd.unwrap());
    }

    #[test]
    fn at_par_when_yield_equals_rate() {
        let r = analyze(&PreferredInput {
            required_yield_pct: 6.0,
            market_price_usd: 0.0,
            ..base()
        });
        assert!(close(r.fair_value_usd.unwrap(), 100.0));
    }

    #[test]
    fn no_price_no_yield_or_verdict() {
        let r = analyze(&PreferredInput {
            market_price_usd: 0.0,
            ..base()
        });
        assert!(r.current_yield_pct.is_none());
        assert!(r.verdict.is_none());
    }
}
