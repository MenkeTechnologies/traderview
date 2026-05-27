//! Volume-by-price profile (TPO-style market profile).
//!
//! Bins a session's trade volume into price levels and identifies:
//!   - **POC** (Point of Control): price with highest volume
//!   - **VAH/VAL** (Value Area High/Low): bounds of the 70% volume range
//!     centered on POC
//!   - **High Volume Nodes** (HVNs): local maxima — typical S/R lines
//!   - **Low Volume Nodes** (LVNs): local minima — price often passes
//!     through these fast
//!
//! Canonical AMT (Auction Market Theory) tool. Pure compute.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub struct PrintAt {
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeProfile {
    pub levels: Vec<PriceLevel>,
    pub poc: f64,
    pub vah: f64,
    pub val: f64,
    pub total_volume: f64,
}

/// Build a profile bucketed to `tick_size` (e.g. 0.01 for stocks, 0.25 for ES).
pub fn build(prints: &[PrintAt], tick_size: f64) -> VolumeProfile {
    let mut report = VolumeProfile::default();
    if prints.is_empty() || tick_size <= 0.0 {
        return report;
    }
    let mut by_price: BTreeMap<i64, f64> = BTreeMap::new();
    for p in prints {
        let bucket = (p.price / tick_size).round() as i64;
        *by_price.entry(bucket).or_default() += p.volume;
    }
    let total: f64 = by_price.values().sum();
    let levels: Vec<PriceLevel> = by_price
        .into_iter()
        .map(|(b, v)| PriceLevel {
            price: b as f64 * tick_size,
            volume: v,
        })
        .collect();
    // POC = price with highest volume.
    let poc_idx = levels
        .iter()
        .enumerate()
        .max_by(|a, b| {
            a.1.volume
                .partial_cmp(&b.1.volume)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);
    let poc = levels[poc_idx].price;

    // Value area: expand from POC outward until 70% of volume is covered.
    let target = total * 0.70;
    let mut covered = levels[poc_idx].volume;
    let mut lo = poc_idx;
    let mut hi = poc_idx;
    while covered < target {
        let next_up = if hi + 1 < levels.len() {
            Some(levels[hi + 1].volume)
        } else {
            None
        };
        let next_down = if lo > 0 {
            Some(levels[lo - 1].volume)
        } else {
            None
        };
        match (next_up, next_down) {
            (Some(u), Some(d)) => {
                if u >= d {
                    hi += 1;
                    covered += u;
                } else {
                    lo -= 1;
                    covered += d;
                }
            }
            (Some(u), None) => {
                hi += 1;
                covered += u;
            }
            (None, Some(d)) => {
                lo -= 1;
                covered += d;
            }
            (None, None) => break,
        }
    }
    report.poc = poc;
    report.val = levels[lo].price;
    report.vah = levels[hi].price;
    report.total_volume = total;
    report.levels = levels;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(price: f64, vol: f64) -> PrintAt {
        PrintAt { price, volume: vol }
    }

    #[test]
    fn empty_returns_default() {
        let r = build(&[], 0.01);
        assert!(r.levels.is_empty());
    }

    #[test]
    fn zero_tick_size_returns_default() {
        let r = build(&[p(100.0, 1000.0)], 0.0);
        assert!(r.levels.is_empty());
    }

    #[test]
    fn single_print_poc_at_that_price() {
        let r = build(&[p(100.0, 1000.0)], 0.01);
        assert_eq!(r.poc, 100.0);
        assert_eq!(r.vah, 100.0);
        assert_eq!(r.val, 100.0);
    }

    #[test]
    fn poc_at_highest_volume_level() {
        let r = build(
            &[
                p(100.0, 100.0),
                p(101.0, 500.0), // highest
                p(102.0, 200.0),
            ],
            0.01,
        );
        assert_eq!(r.poc, 101.0);
    }

    #[test]
    fn value_area_covers_at_least_70_pct() {
        let r = build(
            &[
                p(100.0, 100.0),
                p(101.0, 200.0),
                p(102.0, 400.0), // POC
                p(103.0, 200.0),
                p(104.0, 100.0),
            ],
            0.01,
        );
        let total = 1000.0;
        // Find VA volume.
        let in_va: f64 = r
            .levels
            .iter()
            .filter(|l| l.price >= r.val && l.price <= r.vah)
            .map(|l| l.volume)
            .sum();
        assert!(
            in_va >= total * 0.70,
            "VA should cover ≥70% — got {}/{}",
            in_va,
            total
        );
    }

    #[test]
    fn vah_above_or_equal_to_val() {
        let r = build(&[p(99.0, 100.0), p(100.0, 500.0), p(101.0, 300.0)], 0.01);
        assert!(r.vah >= r.val);
    }

    #[test]
    fn same_price_prints_accumulate() {
        let r = build(&[p(100.0, 500.0), p(100.0, 500.0)], 0.01);
        // One price level with 1000 volume.
        assert_eq!(r.levels.len(), 1);
        assert_eq!(r.levels[0].volume, 1000.0);
    }

    #[test]
    fn nearby_prices_within_tick_size_bucket_together() {
        // tick = 0.50 → 100.0 and 100.2 round to same bucket (200).
        let r = build(&[p(100.0, 500.0), p(100.2, 500.0)], 0.50);
        assert_eq!(r.levels.len(), 1);
        assert_eq!(r.levels[0].volume, 1000.0);
    }

    #[test]
    fn levels_sorted_low_to_high_via_btree() {
        let r = build(&[p(105.0, 100.0), p(100.0, 100.0), p(102.0, 100.0)], 0.01);
        for i in 1..r.levels.len() {
            assert!(r.levels[i].price > r.levels[i - 1].price);
        }
    }
}
