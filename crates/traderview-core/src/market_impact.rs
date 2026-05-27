//! Market-impact / participation-rate analysis.
//!
//! For each trade, compute the trade's qty as a % of average daily volume
//! (ADV). Plot the slippage (in bps) vs participation. Above some
//! threshold the trader is moving the market on themselves — that
//! threshold is what this module identifies.
//:
//! Common rule of thumb: under 1% of ADV the trade is "invisible";
//! 1-5% is "noticeable"; above 5% the trader is the market-maker for
//! that print and gets the worst possible fill.
//!
//! Pure compute. Caller supplies (trade_qty, adv, slippage_bps) tuples;
//! engine emits per-bucket avg/median slippage and identifies the
//! threshold where slippage spikes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct TradeImpact {
    pub qty: f64,
    pub adv: f64,
    pub slippage_bps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStats {
    pub label: String,
    pub trade_count: usize,
    pub avg_slippage_bps: f64,
    pub median_slippage_bps: f64,
    pub max_slippage_bps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketImpactReport {
    pub buckets: Vec<BucketStats>,
    /// First bucket (low-to-high) where avg slippage exceeds the spike
    /// threshold (default 30 bps). None when no spike observed.
    pub impact_threshold_label: Option<String>,
}

pub fn analyze(trades: &[TradeImpact], spike_bps: f64) -> MarketImpactReport {
    let bands = [
        (0.001, "< 0.1% ADV"),
        (0.005, "0.1-0.5% ADV"),
        (0.01,  "0.5-1% ADV"),
        (0.05,  "1-5% ADV"),
        (0.10,  "5-10% ADV"),
        (f64::INFINITY, "> 10% ADV"),
    ];
    let mut by_bucket: Vec<Vec<f64>> = vec![Vec::new(); bands.len()];
    for t in trades {
        if t.adv <= 0.0 { continue; }
        let pct = t.qty / t.adv;
        for (i, (cap, _)) in bands.iter().enumerate() {
            if pct <= *cap {
                by_bucket[i].push(t.slippage_bps);
                break;
            }
        }
    }
    let mut report = MarketImpactReport::default();
    for (i, (_, label)) in bands.iter().enumerate() {
        let vals = &mut by_bucket[i];
        if vals.is_empty() {
            report.buckets.push(BucketStats {
                label: (*label).into(),
                trade_count: 0,
                avg_slippage_bps: 0.0,
                median_slippage_bps: 0.0,
                max_slippage_bps: 0.0,
            });
            continue;
        }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = vals.len();
        let sum: f64 = vals.iter().sum();
        let avg = sum / n as f64;
        let med = vals[n / 2];
        let mx = vals[n - 1];
        report.buckets.push(BucketStats {
            label: (*label).into(),
            trade_count: n,
            avg_slippage_bps: avg,
            median_slippage_bps: med,
            max_slippage_bps: mx,
        });
    }
    report.impact_threshold_label = report.buckets.iter()
        .find(|b| b.trade_count > 0 && b.avg_slippage_bps.abs() > spike_bps)
        .map(|b| b.label.clone());
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(qty: f64, adv: f64, slip: f64) -> TradeImpact {
        TradeImpact { qty, adv, slippage_bps: slip }
    }

    #[test]
    fn empty_returns_no_buckets_active() {
        let r = analyze(&[], 30.0);
        assert!(r.buckets.iter().all(|b| b.trade_count == 0));
        assert!(r.impact_threshold_label.is_none());
    }

    #[test]
    fn tiny_trades_go_in_first_bucket() {
        // qty=100, ADV=1,000,000 → 0.01%, which is ≤ 0.1% bucket.
        let r = analyze(&[t(100.0, 1_000_000.0, -2.0)], 30.0);
        assert_eq!(r.buckets[0].trade_count, 1);
        assert_eq!(r.buckets[0].avg_slippage_bps, -2.0);
    }

    #[test]
    fn medium_trades_go_in_one_to_five_pct_bucket() {
        // 30k qty / 1M ADV = 3% → "1-5% ADV" bucket.
        let r = analyze(&[t(30_000.0, 1_000_000.0, -25.0)], 30.0);
        let bucket = r.buckets.iter().find(|b| b.label == "1-5% ADV").unwrap();
        assert_eq!(bucket.trade_count, 1);
    }

    #[test]
    fn impact_threshold_triggers_on_spike_above_30bps() {
        // 8% ADV slipping -50bps avg → triggers spike threshold.
        let trades = vec![
            t(80_000.0, 1_000_000.0, -50.0),
            t(80_000.0, 1_000_000.0, -60.0),
        ];
        let r = analyze(&trades, 30.0);
        assert_eq!(r.impact_threshold_label.as_deref(), Some("5-10% ADV"));
    }

    #[test]
    fn no_spike_when_all_slippage_low() {
        let trades = vec![
            t(100.0, 1_000_000.0, -2.0),
            t(1000.0, 1_000_000.0, -5.0),
        ];
        let r = analyze(&trades, 30.0);
        assert!(r.impact_threshold_label.is_none());
    }

    #[test]
    fn zero_adv_trades_skipped() {
        // Trade with ADV=0 (data error) skipped — doesn't panic.
        let trades = vec![
            t(100.0, 0.0, -10.0),    // skip
            t(100.0, 1_000_000.0, -2.0),
        ];
        let r = analyze(&trades, 30.0);
        let total_count: usize = r.buckets.iter().map(|b| b.trade_count).sum();
        assert_eq!(total_count, 1, "zero-ADV trade must be excluded");
    }

    #[test]
    fn over_10pct_bucket_catches_extremes() {
        // 200k / 1M = 20% → > 10% bucket.
        let r = analyze(&[t(200_000.0, 1_000_000.0, -200.0)], 30.0);
        let bucket = r.buckets.iter().find(|b| b.label == "> 10% ADV").unwrap();
        assert_eq!(bucket.trade_count, 1);
        assert_eq!(bucket.avg_slippage_bps, -200.0);
    }

    #[test]
    fn bucket_aggregates_avg_median_max() {
        let trades = vec![
            t(30_000.0, 1_000_000.0, -10.0),
            t(40_000.0, 1_000_000.0, -20.0),
            t(50_000.0, 1_000_000.0, -30.0),
        ];
        let r = analyze(&trades, 100.0);    // no spike threshold trip
        let bucket = r.buckets.iter().find(|b| b.label == "1-5% ADV").unwrap();
        assert_eq!(bucket.trade_count, 3);
        assert_eq!(bucket.avg_slippage_bps, -20.0);
        assert_eq!(bucket.median_slippage_bps, -20.0);
        // max in abs sense for negative slippage: max() of sorted-asc is sorted.last() = -10.0.
        assert_eq!(bucket.max_slippage_bps, -10.0);
    }
}
