//! TWAP (Time-Weighted Average Price) execution analyzer.
//!
//! Complement to vwap_slippage. TWAP weights each bar equally rather than
//! by volume — useful for execution analysis when the trader was working
//! a passive limit and care about *time-in-market* rather than volume-
//! participation rate.
//!
//! For a long entry filled BELOW the trade-window TWAP, the trader
//! captured a better-than-average price during the time they were
//! actively quoting. For shorts, the inverse.
//!
//! Pure compute. Caller supplies the typical price per bar within the
//! exposure window.

use crate::models::TradeSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwapInput {
    pub side: TradeSide,
    pub fill_price: Decimal,
    pub typical_prices: Vec<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TwapResult {
    pub twap: Decimal,
    pub fill_price: Decimal,
    /// Positive = trader-favorable. (twap - fill) for longs, (fill - twap)
    /// for shorts.
    pub slippage_dollars: Decimal,
    pub slippage_bps: f64,
    pub beat_twap: bool,
}

pub fn compute(input: &TwapInput) -> Option<TwapResult> {
    if input.typical_prices.is_empty() {
        return None;
    }
    let n = Decimal::from(input.typical_prices.len() as u64);
    let sum: Decimal = input.typical_prices.iter().copied().sum();
    let twap = sum / n;
    let slippage = match input.side {
        TradeSide::Long => twap - input.fill_price,
        TradeSide::Short => input.fill_price - twap,
    };
    let slippage_bps = if twap.is_zero() {
        0.0
    } else {
        to_f64(slippage) / to_f64(twap) * 10_000.0
    };
    Some(TwapResult {
        twap,
        fill_price: input.fill_price,
        slippage_dollars: slippage,
        slippage_bps,
        beat_twap: slippage > Decimal::ZERO,
    })
}

fn to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn empty_bars_returns_none() {
        let r = compute(&TwapInput {
            side: TradeSide::Long,
            fill_price: d("100"),
            typical_prices: vec![],
        });
        assert!(r.is_none());
    }

    #[test]
    fn single_bar_twap_equals_price() {
        let r = compute(&TwapInput {
            side: TradeSide::Long,
            fill_price: d("99"),
            typical_prices: vec![d("100")],
        })
        .unwrap();
        assert_eq!(r.twap, d("100"));
        assert_eq!(r.slippage_dollars, d("1"));
        assert!(r.beat_twap);
    }

    #[test]
    fn twap_is_simple_arithmetic_mean() {
        // Three bars at 100, 110, 120 → TWAP = 110.
        let r = compute(&TwapInput {
            side: TradeSide::Long,
            fill_price: d("105"),
            typical_prices: vec![d("100"), d("110"), d("120")],
        })
        .unwrap();
        assert_eq!(r.twap, d("110"));
        assert_eq!(r.slippage_dollars, d("5"));
    }

    #[test]
    fn twap_unlike_vwap_does_not_weight_volume() {
        // Same three bars. TWAP just averages them — no volume consideration.
        let r = compute(&TwapInput {
            side: TradeSide::Long,
            fill_price: d("100"),
            typical_prices: vec![d("100"), d("100"), d("100")],
        })
        .unwrap();
        assert_eq!(r.twap, d("100"));
        assert_eq!(r.slippage_dollars, Decimal::ZERO);
        assert!(!r.beat_twap, "exact match is not a beat");
    }

    #[test]
    fn long_filled_above_twap_is_unfavorable() {
        let r = compute(&TwapInput {
            side: TradeSide::Long,
            fill_price: d("115"),
            typical_prices: vec![d("100"), d("110"), d("120")],
        })
        .unwrap();
        assert_eq!(r.slippage_dollars, d("-5"));
        assert!(!r.beat_twap);
    }

    #[test]
    fn short_filled_above_twap_is_favorable() {
        let r = compute(&TwapInput {
            side: TradeSide::Short,
            fill_price: d("115"),
            typical_prices: vec![d("100"), d("110"), d("120")],
        })
        .unwrap();
        assert_eq!(r.slippage_dollars, d("5"));
        assert!(r.beat_twap);
    }

    #[test]
    fn slippage_bps_uses_twap_as_denominator() {
        let r = compute(&TwapInput {
            side: TradeSide::Long,
            fill_price: d("99"),
            typical_prices: vec![d("100")],
        })
        .unwrap();
        // 1 / 100 × 10000 = 100bps.
        assert!((r.slippage_bps - 100.0).abs() < 1e-6);
    }
}
