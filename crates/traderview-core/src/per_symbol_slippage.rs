//! Per-symbol slippage roll-up.
//!
//! Aggregates VWAP/TWAP slippage results across many trades, grouped by
//! symbol, so the user can spot instruments where execution quality is
//! systematically poor (penny-wide tickers with bad spreads, illiquid
//! names where their order moves the market, etc.).
//!
//! Pure compute. Input is a list of (symbol, slippage_bps) tuples and
//! the aggregator emits per-symbol stats sorted by worst-mean-slippage
//! first so the dashboard can highlight problem instruments.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSlippage {
    pub symbol: String,
    pub trade_count: usize,
    pub mean_bps: f64,
    pub median_bps: f64,
    pub stdev_bps: f64,
    pub worst_bps: f64,
    pub best_bps: f64,
    /// Fraction (0..=1) of fills that BEAT the benchmark — favorable for
    /// the trader. Higher = better execution at this symbol.
    pub beat_rate: f64,
}

pub fn aggregate(records: &[(String, f64)]) -> Vec<SymbolSlippage> {
    if records.is_empty() { return vec![]; }
    let mut groups: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (sym, bps) in records {
        groups.entry(sym.clone()).or_default().push(*bps);
    }
    let mut out: Vec<SymbolSlippage> = groups.into_iter()
        .map(|(sym, vals)| stats_for(sym, vals))
        .collect();
    // Sort by worst (most negative) mean_bps first — these are the
    // problem symbols the dashboard should highlight.
    out.sort_by(|a, b| a.mean_bps.partial_cmp(&b.mean_bps).unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn stats_for(symbol: String, mut vals: Vec<f64>) -> SymbolSlippage {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = vals.len();
    let sum: f64 = vals.iter().sum();
    let mean = sum / n as f64;
    let median = vals[n / 2];
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let stdev = var.sqrt();
    let beat_count = vals.iter().filter(|v| **v > 0.0).count();
    SymbolSlippage {
        symbol,
        trade_count: n,
        mean_bps: mean,
        median_bps: median,
        stdev_bps: stdev,
        worst_bps: vals[0],
        best_bps: vals[n - 1],
        beat_rate: beat_count as f64 / n as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_empty() {
        assert!(aggregate(&[]).is_empty());
    }

    #[test]
    fn single_record_per_symbol_yields_one_entry() {
        let recs = vec![
            ("AAPL".into(), 10.0),
            ("MSFT".into(), -5.0),
        ];
        let out = aggregate(&recs);
        assert_eq!(out.len(), 2);
        // Worst-first sort → MSFT before AAPL.
        assert_eq!(out[0].symbol, "MSFT");
        assert_eq!(out[1].symbol, "AAPL");
    }

    #[test]
    fn beat_rate_reflects_positive_count() {
        // 3 beats out of 4 fills for AAPL.
        let recs = vec![
            ("AAPL".into(),  5.0),
            ("AAPL".into(),  3.0),
            ("AAPL".into(),  1.0),
            ("AAPL".into(), -2.0),
        ];
        let out = aggregate(&recs);
        assert_eq!(out[0].trade_count, 4);
        assert_eq!(out[0].beat_rate, 0.75);
    }

    #[test]
    fn worst_and_best_extracted_from_sorted_distribution() {
        let recs = vec![
            ("AAPL".into(), -10.0),
            ("AAPL".into(),  20.0),
            ("AAPL".into(),   0.0),
        ];
        let out = aggregate(&recs);
        assert_eq!(out[0].worst_bps, -10.0);
        assert_eq!(out[0].best_bps,  20.0);
    }

    #[test]
    fn mean_matches_arithmetic_average() {
        let recs = vec![
            ("X".into(), 10.0),
            ("X".into(), 20.0),
            ("X".into(), 30.0),
        ];
        let out = aggregate(&recs);
        assert_eq!(out[0].mean_bps, 20.0);
        assert_eq!(out[0].median_bps, 20.0);
    }

    #[test]
    fn sort_puts_worst_symbol_first() {
        let recs = vec![
            ("GOOD".into(), 50.0),
            ("BAD".into(), -50.0),
            ("UGLY".into(), -100.0),
        ];
        let out = aggregate(&recs);
        assert_eq!(out[0].symbol, "UGLY",   "worst mean must be first");
        assert_eq!(out[1].symbol, "BAD");
        assert_eq!(out[2].symbol, "GOOD");
    }

    #[test]
    fn stdev_zero_for_constant_series() {
        let recs = vec![
            ("X".into(), 5.0),
            ("X".into(), 5.0),
            ("X".into(), 5.0),
        ];
        let out = aggregate(&recs);
        assert_eq!(out[0].stdev_bps, 0.0);
        assert_eq!(out[0].mean_bps, 5.0);
    }
}
