//! Market profile (TPO — Time-Price Opportunity).
//!
//! Sierra Chart-class. Distinct from volume profile (`volume_profile.rs`):
//! TPO measures TIME spent at each price level, not volume. Each session is
//! divided into equal-duration brackets (typically 30 minutes); for every
//! bracket, every price level traded during that bracket gets one TPO
//! letter. Stacking those letters by price level gives the canonical TPO
//! histogram and identifies:
//!
//!   - **POC** (Point of Control): price level with the most TPO letters
//!   - **Value Area**: 70% of total TPO letters centered on POC, defining
//!     the high (VAH) and low (VAL) boundaries of fair-value range
//!   - **Single prints**: price levels touched in only one bracket —
//!     classic excess that often gets retested
//!
//! Pure compute. Caller supplies brackets where each is the list of
//! (high, low) prices traded during that period.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BracketRange {
    pub bracket_index: u32,
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TpoLevel {
    pub price: f64,
    pub tpo_count: u32,
    /// True when only one bracket touched this price — "single print".
    pub single_print: bool,
    /// Index of every bracket that touched this price (ascending).
    pub brackets: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TpoReport {
    pub levels: Vec<TpoLevel>,
    pub tick_size: f64,
    pub total_tpos: u32,
    pub poc_price: f64,
    pub value_area_high: f64,
    pub value_area_low: f64,
    /// Single-print prices (excess) in ascending order.
    pub single_prints: Vec<f64>,
}

/// Build a TPO report. `tick_size` quantizes price into TPO rows.
pub fn build(brackets: &[BracketRange], tick_size: f64) -> TpoReport {
    if tick_size <= 0.0 || brackets.is_empty() {
        return TpoReport {
            tick_size,
            ..Default::default()
        };
    }
    // price_bin → set of bracket indices that touched it (BTreeMap gives
    // sorted output; we use a vec to preserve insertion order then dedup).
    let mut by_bin: BTreeMap<i64, Vec<u32>> = BTreeMap::new();
    let quantize = |p: f64| -> i64 { (p / tick_size).round() as i64 };
    for b in brackets {
        if b.high < b.low {
            continue;
        }
        let lo = quantize(b.low);
        let hi = quantize(b.high);
        for q in lo..=hi {
            let entry = by_bin.entry(q).or_default();
            if entry.last() != Some(&b.bracket_index) {
                entry.push(b.bracket_index);
            }
        }
    }
    let levels: Vec<TpoLevel> = by_bin
        .into_iter()
        .map(|(q, mut brackets)| {
            brackets.dedup();
            let price = q as f64 * tick_size;
            let count = brackets.len() as u32;
            TpoLevel {
                price,
                tpo_count: count,
                single_print: count == 1,
                brackets,
            }
        })
        .collect();
    if levels.is_empty() {
        return TpoReport {
            tick_size,
            ..Default::default()
        };
    }

    let total: u32 = levels.iter().map(|l| l.tpo_count).sum();
    // POC = level with max tpo_count.
    let poc_idx = levels
        .iter()
        .enumerate()
        .max_by_key(|(_, l)| l.tpo_count)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let poc_price = levels.get(poc_idx).map(|l| l.price).unwrap_or(0.0);

    // Expand a band around POC until it captures ≥ 70% of total TPO letters.
    // Compare adjacent levels above + below; pick whichever has more TPOs
    // each step (tie → pick lower so VAL gets priority — convention).
    let target = (total as f64 * 0.70).ceil() as u32;
    let mut captured = levels[poc_idx].tpo_count;
    let mut up = poc_idx;
    let mut down = poc_idx;
    while captured < target {
        let above = (up + 1 < levels.len()).then(|| levels[up + 1].tpo_count);
        let below = (down > 0).then(|| levels[down - 1].tpo_count);
        match (above, below) {
            (Some(a), Some(b)) if a > b => {
                up += 1;
                captured += a;
            }
            (Some(_), Some(b)) => {
                down -= 1;
                captured += b;
            }
            (Some(a), None) => {
                up += 1;
                captured += a;
            }
            (None, Some(b)) => {
                down -= 1;
                captured += b;
            }
            (None, None) => break,
        }
    }
    let value_area_high = levels[up].price;
    let value_area_low = levels[down].price;

    let single_prints: Vec<f64> = levels
        .iter()
        .filter(|l| l.single_print)
        .map(|l| l.price)
        .collect();

    TpoReport {
        levels,
        tick_size,
        total_tpos: total,
        poc_price,
        value_area_high,
        value_area_low,
        single_prints,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn br(idx: u32, high: f64, low: f64) -> BracketRange {
        BracketRange {
            bracket_index: idx,
            high,
            low,
        }
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let r = build(&[], 0.25);
        assert!(r.levels.is_empty());
        assert_eq!(r.total_tpos, 0);
    }

    #[test]
    fn single_bracket_marks_every_touched_level_as_single_print() {
        // One bracket trading 100..=102 → 3 levels, all single-print.
        let r = build(&[br(0, 102.0, 100.0)], 1.0);
        assert_eq!(r.levels.len(), 3);
        assert!(r.levels.iter().all(|l| l.single_print));
        assert_eq!(r.single_prints.len(), 3);
        assert_eq!(r.total_tpos, 3);
    }

    #[test]
    fn poc_is_the_most_revisited_level() {
        // Bracket A: 100..=102. Bracket B: 101..=103. Bracket C: 101..=101.
        // Level 101 gets touched by all 3 brackets → POC = 101.
        let r = build(
            &[
                br(0, 102.0, 100.0),
                br(1, 103.0, 101.0),
                br(2, 101.0, 101.0),
            ],
            1.0,
        );
        assert!((r.poc_price - 101.0).abs() < 1e-9);
    }

    #[test]
    fn value_area_captures_at_least_70_percent_of_tpos() {
        // Build a triangle distribution so 70% is well-defined.
        let r = build(
            &[
                br(0, 110.0, 100.0),
                br(1, 108.0, 102.0),
                br(2, 106.0, 104.0),
                br(3, 105.0, 105.0),
            ],
            1.0,
        );
        // Sanity: VAH >= POC >= VAL.
        assert!(r.value_area_high >= r.poc_price);
        assert!(r.poc_price >= r.value_area_low);
        // Capture sum within VA must be ≥ 70% of total.
        let captured: u32 = r
            .levels
            .iter()
            .filter(|l| l.price >= r.value_area_low && l.price <= r.value_area_high)
            .map(|l| l.tpo_count)
            .sum();
        assert!(
            captured * 10 >= r.total_tpos * 7,
            "value area captured {captured} of {} TPOs (need ≥ 70%)",
            r.total_tpos
        );
    }

    #[test]
    fn invalid_bracket_high_below_low_is_skipped() {
        let r = build(&[br(0, 100.0, 105.0)], 1.0);
        assert!(
            r.levels.is_empty(),
            "high<low bracket is malformed and ignored"
        );
    }

    #[test]
    fn invalid_tick_size_returns_empty_report() {
        let r = build(&[br(0, 102.0, 100.0)], 0.0);
        assert!(r.levels.is_empty());
    }
}
