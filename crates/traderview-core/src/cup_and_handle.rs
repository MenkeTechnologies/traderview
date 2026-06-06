//! Cup-and-Handle pattern detector (William O'Neil / IBD-style).
//!
//! Pattern shape:
//!   - **Cup** — rounded U-shape over the past ~7-65 weeks (we use bars).
//!     Left rim ≈ right rim within `rim_tolerance_pct`. The trough sits
//!     between them; depth (rim → trough) ≤ `max_depth_pct`.
//!   - **Handle** — short consolidation off the right rim, drifting
//!     down 5-15% of the cup depth, over `handle_min_bars`..`handle_max_bars`.
//!   - **Pivot** — the high of the handle (rim − handle dip). A break
//!     above the pivot is the canonical IBD buy point.
//!
//! Pure compute. Returns at most one candidate pattern (the most recent
//! valid setup). Designed as a screener primitive — meant for backtest
//! and watchlist tagging, not for high-frequency execution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub cup_min_bars: usize,
    pub cup_max_bars: usize,
    /// Cup trough must be at least this fraction below the rim — rejects
    /// flat/near-flat series that would otherwise produce degenerate
    /// "cups" with sub-1% depth. IBD convention: ≥ 12% real correction.
    pub min_depth_pct: f64,
    pub max_depth_pct: f64,
    pub rim_tolerance_pct: f64,
    pub handle_min_bars: usize,
    pub handle_max_bars: usize,
    pub max_handle_depth_pct: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cup_min_bars: 30,
            cup_max_bars: 250,
            min_depth_pct: 0.10,
            max_depth_pct: 0.33,
            rim_tolerance_pct: 0.05,
            handle_min_bars: 5,
            handle_max_bars: 25,
            max_handle_depth_pct: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CupHandleCandidate {
    pub left_rim_index: usize,
    pub trough_index: usize,
    pub right_rim_index: usize,
    pub handle_low_index: usize,
    pub last_index: usize,
    pub left_rim_price: f64,
    pub right_rim_price: f64,
    pub trough_price: f64,
    pub handle_low_price: f64,
    pub pivot_price: f64,
    pub depth_pct: f64,
    pub handle_depth_pct: f64,
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Option<CupHandleCandidate> {
    let n = bars.len();
    if cfg.cup_min_bars < 4
        || cfg.cup_max_bars <= cfg.cup_min_bars
        || cfg.handle_min_bars == 0
        || cfg.handle_max_bars < cfg.handle_min_bars
        || cfg.min_depth_pct <= 0.0
        || cfg.max_depth_pct <= cfg.min_depth_pct
        || cfg.rim_tolerance_pct < 0.0
        || cfg.max_handle_depth_pct <= 0.0
        || n < cfg.cup_min_bars + cfg.handle_min_bars
    {
        return None;
    }
    // Walk candidate handle-end indices (most-recent first).
    let last = n - 1;
    for handle_len in cfg.handle_min_bars..=cfg.handle_max_bars.min(n - cfg.cup_min_bars) {
        let right_rim_idx = last - handle_len;
        let handle_slice = &bars[right_rim_idx + 1..=last];
        let right_rim_price = bars[right_rim_idx].high;
        if !right_rim_price.is_finite() || right_rim_price <= 0.0 {
            continue;
        }
        // Handle bars: all closes ≤ right rim AND lowest handle close is
        // within max_handle_depth_pct of rim.
        if handle_slice
            .iter()
            .any(|b| !b.close.is_finite() || b.close > right_rim_price)
        {
            continue;
        }
        let (handle_low_offset, handle_low_bar) =
            handle_slice.iter().enumerate().min_by(|a, b| {
                a.1.low
                    .partial_cmp(&b.1.low)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })?;
        let handle_low_price = handle_low_bar.low;
        let handle_depth_pct = (right_rim_price - handle_low_price) / right_rim_price;
        if handle_depth_pct > cfg.max_handle_depth_pct {
            continue;
        }
        let handle_low_idx = right_rim_idx + 1 + handle_low_offset;
        // Search for matching left rim by walking back.
        for cup_len in cfg.cup_min_bars..=cfg.cup_max_bars.min(right_rim_idx) {
            let left_rim_idx = right_rim_idx - cup_len;
            let left_rim_price = bars[left_rim_idx].high;
            if !left_rim_price.is_finite() || left_rim_price <= 0.0 {
                continue;
            }
            let rim_diff_pct = (left_rim_price - right_rim_price).abs() / left_rim_price;
            if rim_diff_pct > cfg.rim_tolerance_pct {
                continue;
            }
            let cup_slice = &bars[left_rim_idx + 1..right_rim_idx];
            if cup_slice.is_empty() {
                continue;
            }
            let (trough_offset, trough_bar) = cup_slice.iter().enumerate().min_by(|a, b| {
                a.1.low
                    .partial_cmp(&b.1.low)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })?;
            let trough_price = trough_bar.low;
            let depth_pct = (left_rim_price - trough_price) / left_rim_price;
            if depth_pct < cfg.min_depth_pct || depth_pct > cfg.max_depth_pct {
                continue;
            }
            // Right side must roughly recover (already enforced by rim
            // similarity). Pivot = right_rim_price (canonical IBD).
            return Some(CupHandleCandidate {
                left_rim_index: left_rim_idx,
                trough_index: left_rim_idx + 1 + trough_offset,
                right_rim_index: right_rim_idx,
                handle_low_index: handle_low_idx,
                last_index: last,
                left_rim_price,
                right_rim_price,
                trough_price,
                handle_low_price,
                pivot_price: right_rim_price,
                depth_pct,
                handle_depth_pct,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64) -> Bar {
        Bar {
            high: c + 0.5,
            low: c - 0.5,
            close: c,
        }
    }

    #[test]
    fn empty_or_short_returns_none() {
        assert!(detect(&[], &Config::default()).is_none());
        let bars = vec![b(100.0); 10];
        assert!(detect(&bars, &Config::default()).is_none());
    }

    #[test]
    fn invalid_config_returns_none() {
        let bars = vec![b(100.0); 100];
        let cfg = Config {
            cup_max_bars: 0,
            ..Default::default()
        };
        assert!(detect(&bars, &cfg).is_none());
    }

    #[test]
    fn flat_series_no_pattern_detected() {
        let bars = vec![b(100.0); 100];
        assert!(detect(&bars, &Config::default()).is_none());
    }

    #[test]
    fn classic_u_shaped_cup_with_handle_detected() {
        // Construct: 30 bars down 100 → 80, 30 bars up 80 → 100, then a
        // 7-bar handle drifting from 100 → 97.
        let mut bars: Vec<Bar> = Vec::new();
        for i in 0..30 {
            bars.push(b(100.0 - (i as f64) * (20.0 / 29.0)));
        }
        for i in 0..30 {
            bars.push(b(80.0 + (i as f64 + 1.0) * (20.0 / 30.0)));
        }
        // 7-bar handle.
        for i in 0..7 {
            bars.push(b(100.0 - (i as f64 + 1.0) * 0.4));
        }
        let cfg = Config {
            cup_min_bars: 50,
            cup_max_bars: 70,
            handle_min_bars: 5,
            handle_max_bars: 10,
            ..Default::default()
        };
        let cand = detect(&bars, &cfg).expect("should detect cup+handle");
        assert!((cand.left_rim_price - cand.right_rim_price).abs() / cand.left_rim_price < 0.05);
        assert!(cand.depth_pct > 0.15 && cand.depth_pct < 0.25);
        assert!(cand.handle_depth_pct > 0.0 && cand.handle_depth_pct < 0.05);
    }

    #[test]
    fn rims_too_unequal_rejected() {
        // Left rim 100, right rim 80 — clearly not a cup.
        let mut bars: Vec<Bar> = Vec::new();
        for i in 0..30 {
            bars.push(b(100.0 - (i as f64) * 0.7));
        }
        for i in 0..30 {
            bars.push(b(79.0 + (i as f64) * 0.0));
        }
        for _ in 0..7 {
            bars.push(b(79.0));
        }
        assert!(detect(&bars, &Config::default()).is_none());
    }

    #[test]
    fn handle_too_deep_rejected() {
        // Cup 100 → 80 → 100, then handle drops 30% to 70 — way over 15%.
        let mut bars: Vec<Bar> = Vec::new();
        for i in 0..30 {
            bars.push(b(100.0 - (i as f64) * (20.0 / 29.0)));
        }
        for i in 0..30 {
            bars.push(b(80.0 + (i as f64 + 1.0) * (20.0 / 30.0)));
        }
        for i in 0..7 {
            bars.push(b(100.0 - (i as f64 + 1.0) * 5.0));
        } // huge handle drop
        assert!(detect(&bars, &Config::default()).is_none());
    }
}
