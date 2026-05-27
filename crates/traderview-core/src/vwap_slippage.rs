//! VWAP-relative slippage analysis (TCA depth).
//!
//! For each closing trade, compare the realized fill price vs the
//! Volume-Weighted Average Price across the trade's open window. A buy
//! filled BELOW VWAP is a positive entry-quality signal (got a discount);
//! a sell filled ABOVE VWAP is positive on exit. Compute per-trade
//! slippage in basis points + aggregate stats.
//!
//! Pure compute. Caller supplies the bar series for each trade's open
//! window; engine just does the VWAP math + comparison.

use crate::models::TradeSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct BarOhlcv {
    pub typical: Decimal,    // (high+low+close)/3, caller pre-computes
    pub volume: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VwapInput {
    pub side: TradeSide,
    pub fill_price: Decimal,
    /// Bars covering the trade's open window in order.
    /// Caller passes the typical price + volume per bar.
    #[serde(skip)]
    pub bars: Vec<BarOhlcv>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VwapResult {
    pub vwap: Decimal,
    pub fill_price: Decimal,
    /// (fill - vwap) for longs, (vwap - fill) for shorts — positive when
    /// the trader got a better price than VWAP regardless of direction.
    pub slippage_dollars: Decimal,
    /// slippage_dollars / vwap, in basis points (× 10,000).
    pub slippage_bps: f64,
    /// True when the fill beat VWAP (positive slippage in trader-favorable
    /// direction).
    pub beat_vwap: bool,
}

pub fn compute(input: &VwapInput) -> Option<VwapResult> {
    if input.bars.is_empty() { return None; }
    let total_vol: Decimal = input.bars.iter().map(|b| b.volume).sum();
    if total_vol.is_zero() { return None; }
    let numerator: Decimal = input.bars.iter()
        .map(|b| b.typical * b.volume).sum();
    let vwap = numerator / total_vol;

    // Trader-favorable slippage:
    //   long  fill BELOW vwap → positive (got a discount on entry)
    //   short fill ABOVE vwap → positive (got premium on entry)
    // For an EXIT the convention flips, but TCA usually reports entries.
    // Caller passes side reflecting entry direction.
    let slippage = match input.side {
        TradeSide::Long  => vwap - input.fill_price,
        TradeSide::Short => input.fill_price - vwap,
    };
    let slippage_bps = if vwap.is_zero() {
        0.0
    } else {
        to_f64(slippage) / to_f64(vwap) * 10_000.0
    };
    Some(VwapResult {
        vwap,
        fill_price: input.fill_price,
        slippage_dollars: slippage,
        slippage_bps,
        beat_vwap: slippage > Decimal::ZERO,
    })
}

fn to_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

/// Aggregate slippage across many trades.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VwapAggregate {
    pub trades_analyzed: usize,
    pub beat_vwap_count: usize,
    /// Trader-favorable percent (closer to 100 = consistently beats VWAP).
    pub beat_vwap_pct: f64,
    pub avg_slippage_bps: f64,
    pub median_slippage_bps: f64,
}

pub fn aggregate(results: &[VwapResult]) -> VwapAggregate {
    if results.is_empty() { return VwapAggregate::default(); }
    let n = results.len();
    let beat_count = results.iter().filter(|r| r.beat_vwap).count();
    let avg = results.iter().map(|r| r.slippage_bps).sum::<f64>() / n as f64;
    let mut sorted: Vec<f64> = results.iter().map(|r| r.slippage_bps).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = sorted[n / 2];
    VwapAggregate {
        trades_analyzed: n,
        beat_vwap_count: beat_count,
        beat_vwap_pct: beat_count as f64 / n as f64 * 100.0,
        avg_slippage_bps: avg,
        median_slippage_bps: median,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    #[test]
    fn empty_bars_returns_none() {
        let r = compute(&VwapInput {
            side: TradeSide::Long, fill_price: d("100"), bars: vec![],
        });
        assert!(r.is_none());
    }

    #[test]
    fn zero_volume_returns_none() {
        let r = compute(&VwapInput {
            side: TradeSide::Long, fill_price: d("100"),
            bars: vec![BarOhlcv { typical: d("100"), volume: Decimal::ZERO }],
        });
        assert!(r.is_none());
    }

    #[test]
    fn single_bar_vwap_equals_typical() {
        let r = compute(&VwapInput {
            side: TradeSide::Long, fill_price: d("99"),
            bars: vec![BarOhlcv { typical: d("100"), volume: d("1000") }],
        }).unwrap();
        assert_eq!(r.vwap, d("100"));
        // Long filled at 99 vs VWAP 100 → favorable $1.
        assert_eq!(r.slippage_dollars, d("1"));
        assert!(r.beat_vwap);
    }

    #[test]
    fn vwap_volume_weights_correctly() {
        // Two bars: 100 @ 100 vol, 110 @ 900 vol.
        // VWAP = (100×100 + 110×900) / 1000 = (10000 + 99000) / 1000 = 109.
        let r = compute(&VwapInput {
            side: TradeSide::Long, fill_price: d("105"),
            bars: vec![
                BarOhlcv { typical: d("100"), volume: d("100") },
                BarOhlcv { typical: d("110"), volume: d("900") },
            ],
        }).unwrap();
        assert_eq!(r.vwap, d("109"));
        assert_eq!(r.slippage_dollars, d("4"));
        assert!(r.beat_vwap);
    }

    #[test]
    fn long_filled_above_vwap_is_unfavorable() {
        let r = compute(&VwapInput {
            side: TradeSide::Long, fill_price: d("101"),
            bars: vec![BarOhlcv { typical: d("100"), volume: d("1000") }],
        }).unwrap();
        assert_eq!(r.slippage_dollars, d("-1"));
        assert!(!r.beat_vwap);
    }

    #[test]
    fn short_filled_above_vwap_is_favorable() {
        // Short at 101 vs VWAP 100 → got $1 better.
        let r = compute(&VwapInput {
            side: TradeSide::Short, fill_price: d("101"),
            bars: vec![BarOhlcv { typical: d("100"), volume: d("1000") }],
        }).unwrap();
        assert_eq!(r.slippage_dollars, d("1"));
        assert!(r.beat_vwap);
    }

    #[test]
    fn slippage_bps_uses_vwap_as_denominator() {
        let r = compute(&VwapInput {
            side: TradeSide::Long, fill_price: d("99"),
            bars: vec![BarOhlcv { typical: d("100"), volume: d("1000") }],
        }).unwrap();
        // 1 / 100 × 10000 = 100 bps (= 1%).
        assert!((r.slippage_bps - 100.0).abs() < 1e-6);
    }

    #[test]
    fn aggregate_empty_input_zero_report() {
        let a = aggregate(&[]);
        assert_eq!(a.trades_analyzed, 0);
        assert_eq!(a.avg_slippage_bps, 0.0);
    }

    #[test]
    fn aggregate_counts_beat_vwap_correctly() {
        let results = vec![
            VwapResult { beat_vwap: true,  slippage_bps:  5.0, ..Default::default() },
            VwapResult { beat_vwap: true,  slippage_bps: 10.0, ..Default::default() },
            VwapResult { beat_vwap: false, slippage_bps: -3.0, ..Default::default() },
            VwapResult { beat_vwap: false, slippage_bps: -7.0, ..Default::default() },
        ];
        let a = aggregate(&results);
        assert_eq!(a.trades_analyzed, 4);
        assert_eq!(a.beat_vwap_count, 2);
        assert_eq!(a.beat_vwap_pct, 50.0);
        assert!((a.avg_slippage_bps - 1.25).abs() < 1e-9);
    }
}
