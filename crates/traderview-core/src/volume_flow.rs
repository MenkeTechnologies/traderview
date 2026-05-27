//! Volume-flow indicators: On-Balance Volume + Accumulation/Distribution Line.
//!
//! **OBV** (Joseph Granville):
//!   Up day  → OBV += today's volume
//!   Down day → OBV -= today's volume
//!   Flat day → OBV unchanged
//!
//! Cumulative measure of buying/selling pressure. Divergence with price
//! is the canonical signal.
//!
//! **A/D Line** (Marc Chaikin):
//!   MFM = ((close - low) - (high - close)) / (high - low)
//!   MFV = MFM × volume
//!   A/D = cumsum(MFV)
//!
//! Per-bar volume gets a sign based on where the close sits in the bar's
//! range. Close-near-high days are "accumulation"; close-near-low are
//! "distribution".
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Cumulative OBV at the end of each bar in the series.
pub fn obv(bars: &[Bar]) -> Vec<f64> {
    let mut out = Vec::with_capacity(bars.len());
    let mut running = 0.0;
    let mut prev_close: Option<f64> = None;
    for b in bars {
        if let Some(prev) = prev_close {
            if b.close > prev      { running += b.volume; }
            else if b.close < prev { running -= b.volume; }
            // close == prev → unchanged
        }
        // First bar contributes 0 — convention.
        out.push(running);
        prev_close = Some(b.close);
    }
    out
}

/// Cumulative A/D line at the end of each bar in the series.
pub fn accumulation_distribution(bars: &[Bar]) -> Vec<f64> {
    let mut out = Vec::with_capacity(bars.len());
    let mut running = 0.0;
    for b in bars {
        let range = b.high - b.low;
        // Money Flow Multiplier — set to 0 for zero-range bars (limit up/down).
        let mfm = if range > 0.0 {
            ((b.close - b.low) - (b.high - b.close)) / range
        } else { 0.0 };
        let mfv = mfm * b.volume;
        running += mfv;
        out.push(running);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { high: h, low: l, close: c, volume: v }
    }

    // ─── OBV ──────────────────────────────────────────────────────────

    #[test]
    fn obv_first_bar_always_zero() {
        let out = obv(&[b(10.0, 9.0, 9.5, 1000.0)]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn obv_empty_returns_empty() {
        assert!(obv(&[]).is_empty());
    }

    #[test]
    fn obv_up_day_adds_volume() {
        // Bar 1: close 100. Bar 2: close 105 (up) with vol 500.
        let out = obv(&[
            b(101.0, 99.0, 100.0, 1000.0),
            b(106.0, 99.0, 105.0, 500.0),
        ]);
        assert_eq!(out[1], 500.0);
    }

    #[test]
    fn obv_down_day_subtracts_volume() {
        let out = obv(&[
            b(101.0, 99.0, 100.0, 1000.0),
            b(100.0, 94.0, 95.0,  300.0),
        ]);
        assert_eq!(out[1], -300.0);
    }

    #[test]
    fn obv_flat_close_unchanged() {
        let out = obv(&[
            b(101.0, 99.0, 100.0, 1000.0),
            b(102.0, 98.0, 100.0,  500.0),
        ]);
        assert_eq!(out[1], 0.0);
    }

    #[test]
    fn obv_accumulates_across_multiple_bars() {
        let out = obv(&[
            b(101.0, 99.0,  100.0, 1000.0),
            b(105.0, 99.0,  105.0,  500.0),    // +500
            b(106.0, 102.0, 103.0,  200.0),    // -200
            b(108.0, 102.0, 108.0,  100.0),    // +100
        ]);
        assert_eq!(out, vec![0.0, 500.0, 300.0, 400.0]);
    }

    // ─── A/D ──────────────────────────────────────────────────────────

    #[test]
    fn ad_close_at_high_full_accumulation() {
        // close == high → MFM = +1 → MFV = +volume.
        let out = accumulation_distribution(&[b(110.0, 100.0, 110.0, 1000.0)]);
        assert_eq!(out[0], 1000.0);
    }

    #[test]
    fn ad_close_at_low_full_distribution() {
        let out = accumulation_distribution(&[b(110.0, 100.0, 100.0, 1000.0)]);
        assert_eq!(out[0], -1000.0);
    }

    #[test]
    fn ad_close_at_midpoint_zero() {
        let out = accumulation_distribution(&[b(110.0, 100.0, 105.0, 1000.0)]);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn ad_zero_range_bar_contributes_zero() {
        // Limit-up bar: H == L == C → MFM forced to 0.
        let out = accumulation_distribution(&[b(100.0, 100.0, 100.0, 1000.0)]);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn ad_accumulates_across_bars() {
        // Bar 1: close at high → +1000.
        // Bar 2: close at low  → -500.
        let out = accumulation_distribution(&[
            b(110.0, 100.0, 110.0, 1000.0),
            b(110.0, 100.0, 100.0,  500.0),
        ]);
        assert_eq!(out[0], 1000.0);
        assert_eq!(out[1], 500.0);
    }

    #[test]
    fn obv_and_ad_same_length_as_input() {
        let bars: Vec<Bar> = (1..=10).map(|i|
            b(i as f64 + 1.0, i as f64, i as f64 + 0.5, 1000.0)
        ).collect();
        let o = obv(&bars);
        let a = accumulation_distribution(&bars);
        assert_eq!(o.len(), 10);
        assert_eq!(a.len(), 10);
    }
}
