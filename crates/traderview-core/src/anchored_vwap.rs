//! Anchored VWAP — VWAP from an arbitrary event date.
//!
//! Unlike intraday VWAP (which resets at the open), anchored VWAP starts
//! from a specific event bar (e.g. earnings release, IPO, FOMC) and
//! accumulates from there. The canonical institutional buying-zone /
//! profit-taking reference for swing trades.
//!
//! Returns the (anchored_vwap, ±1σ, ±2σ) per bar AFTER the anchor.
//! Bars BEFORE the anchor return zeros.
//!
//! Pure compute. Anchor identified by bar index in the input series.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub typical: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct AnchoredPoint {
    pub vwap: f64,
    pub upper_1sd: f64,
    pub lower_1sd: f64,
    pub upper_2sd: f64,
    pub lower_2sd: f64,
}

pub fn compute(bars: &[Bar], anchor_index: usize) -> Vec<AnchoredPoint> {
    let mut out = vec![AnchoredPoint::default(); bars.len()];
    if bars.is_empty() || anchor_index >= bars.len() { return out; }
    let mut sum_pv = 0.0;
    let mut sum_v = 0.0;
    let mut sum_p2v = 0.0;
    for i in anchor_index..bars.len() {
        let b = bars[i];
        let v = b.volume.max(0.0);
        sum_pv += b.typical * v;
        sum_v += v;
        sum_p2v += b.typical * b.typical * v;
        if sum_v > 0.0 {
            let vwap = sum_pv / sum_v;
            let var = (sum_p2v / sum_v) - vwap * vwap;
            let sd = var.max(0.0).sqrt();
            out[i] = AnchoredPoint {
                vwap,
                upper_1sd: vwap + sd,
                lower_1sd: vwap - sd,
                upper_2sd: vwap + 2.0 * sd,
                lower_2sd: vwap - 2.0 * sd,
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(t: f64, v: f64) -> Bar { Bar { typical: t, volume: v } }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 0).is_empty());
    }

    #[test]
    fn anchor_at_first_bar_starts_from_zero() {
        let out = compute(&[b(100.0, 1000.0), b(110.0, 1000.0)], 0);
        // Bar 0: VWAP = 100 with zero stdev.
        assert_eq!(out[0].vwap, 100.0);
        assert_eq!(out[0].upper_1sd, 100.0);
        // Bar 1: VWAP = (100+110)/2 = 105.
        assert_eq!(out[1].vwap, 105.0);
        assert!(out[1].upper_1sd > 105.0);
    }

    #[test]
    fn bars_before_anchor_have_zero_points() {
        let out = compute(&[b(100.0, 1000.0), b(105.0, 1000.0), b(110.0, 1000.0)], 1);
        // Bar 0 (before anchor) → default zeros.
        assert_eq!(out[0].vwap, 0.0);
        // Bar 1 (anchor): VWAP = 105.
        assert_eq!(out[1].vwap, 105.0);
    }

    #[test]
    fn anchor_index_past_end_returns_all_zeros() {
        let out = compute(&[b(100.0, 1000.0)], 5);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].vwap, 0.0);
    }

    #[test]
    fn volume_weighted_average_correct() {
        // Anchor at 0: bar0 = 100×1000, bar1 = 110×9000.
        // VWAP = (100,000 + 990,000)/(10,000) = 1,090,000 / 10,000 = 109.
        let out = compute(&[b(100.0, 1000.0), b(110.0, 9000.0)], 0);
        assert_eq!(out[1].vwap, 109.0);
    }

    #[test]
    fn bands_symmetric_around_vwap() {
        let out = compute(&[b(98.0, 1.0), b(100.0, 1.0), b(102.0, 1.0)], 0);
        let p = out[2];
        assert!(((p.upper_1sd + p.lower_1sd) / 2.0 - p.vwap).abs() < 1e-9);
        assert!(((p.upper_2sd + p.lower_2sd) / 2.0 - p.vwap).abs() < 1e-9);
    }

    #[test]
    fn negative_volume_clamped() {
        // Negative volume bar treated as 0 — VWAP unchanged.
        let out = compute(&[b(100.0, 1000.0), b(110.0, -500.0)], 0);
        assert_eq!(out[1].vwap, 100.0);
    }

    #[test]
    fn earnings_anchor_independent_of_pre_anchor_data() {
        // Same post-anchor data → same VWAP regardless of pre-anchor.
        let a = compute(&[b(50.0, 100.0), b(100.0, 1000.0), b(110.0, 1000.0)], 1);
        let b = compute(&[b(200.0, 999.0), b(100.0, 1000.0), b(110.0, 1000.0)], 1);
        assert_eq!(a[2].vwap, b[2].vwap);
    }
}
