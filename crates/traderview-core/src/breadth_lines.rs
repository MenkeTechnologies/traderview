//! Market-breadth cumulative lines.
//!
//! Three classic breadth measures aggregated as running cumulative sums:
//!
//!   - **Advance-Decline Line (AD line)**: cumulative sum of
//!     `advancing_issues − declining_issues` per day. The single best
//!     non-price market-internals divergence signal — a falling AD line
//!     while the index makes new highs is a famous bear-market precursor.
//!
//!   - **Net New Highs**: cumulative sum of
//!     `new_52w_highs − new_52w_lows`. Tracks expansion vs contraction
//!     of leadership.
//!
//!   - **Up/Down Volume Line**: cumulative sum of
//!     `up_volume − down_volume`. Volume version of the AD line —
//!     adds weight to how much was traded on each side.
//!
//! Pure compute. Caller supplies per-day breadth bars (most recent last).
//! Each output series is aligned to input length and starts at 0.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BreadthBar {
    pub advancing_issues: i64,
    pub declining_issues: i64,
    pub new_52w_highs: i64,
    pub new_52w_lows: i64,
    pub up_volume: f64,
    pub down_volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreadthReport {
    pub ad_line: Vec<f64>,
    pub net_new_highs_line: Vec<f64>,
    pub up_down_volume_line: Vec<f64>,
}

pub fn compute(bars: &[BreadthBar]) -> BreadthReport {
    let n = bars.len();
    let mut report = BreadthReport {
        ad_line: vec![0.0; n],
        net_new_highs_line: vec![0.0; n],
        up_down_volume_line: vec![0.0; n],
    };
    let mut ad = 0.0_f64;
    let mut nh = 0.0_f64;
    let mut uv = 0.0_f64;
    for (i, b) in bars.iter().enumerate() {
        // Use f64 math so i64::MAX/MIN inputs don't overflow.
        ad += b.advancing_issues as f64 - b.declining_issues as f64;
        nh += b.new_52w_highs as f64 - b.new_52w_lows as f64;
        if b.up_volume.is_finite() && b.down_volume.is_finite() {
            uv += b.up_volume - b.down_volume;
        }
        if !ad.is_finite() { ad = 0.0; }
        if !nh.is_finite() { nh = 0.0; }
        if !uv.is_finite() { uv = 0.0; }
        report.ad_line[i] = ad;
        report.net_new_highs_line[i] = nh;
        report.up_down_volume_line[i] = uv;
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(adv: i64, dec: i64, nh: i64, nl: i64, uv: f64, dv: f64) -> BreadthBar {
        BreadthBar {
            advancing_issues: adv, declining_issues: dec,
            new_52w_highs: nh, new_52w_lows: nl,
            up_volume: uv, down_volume: dv,
        }
    }

    #[test]
    fn empty_returns_empty_series() {
        let r = compute(&[]);
        assert!(r.ad_line.is_empty());
    }

    #[test]
    fn single_bar_seeds_with_its_own_delta() {
        let r = compute(&[b(2000, 1000, 100, 50, 500_000.0, 200_000.0)]);
        assert_eq!(r.ad_line[0], 1000.0);
        assert_eq!(r.net_new_highs_line[0], 50.0);
        assert!((r.up_down_volume_line[0] - 300_000.0).abs() < 1e-9);
    }

    #[test]
    fn ad_line_is_cumulative_sum_of_net_advances() {
        let bars = vec![
            b(2000, 1000, 0, 0, 0.0, 0.0),    // +1000 net
            b(1500, 1500, 0, 0, 0.0, 0.0),    // 0 net
            b(1000, 2000, 0, 0, 0.0, 0.0),    // -1000 net
        ];
        let r = compute(&bars);
        assert_eq!(r.ad_line, vec![1000.0, 1000.0, 0.0]);
    }

    #[test]
    fn extreme_i64_inputs_dont_overflow() {
        let bars = vec![
            b(i64::MAX, 0, 0, 0, 0.0, 0.0),
            b(i64::MAX, 0, 0, 0, 0.0, 0.0),
        ];
        let r = compute(&bars);
        // f64 math handles 2·i64::MAX without overflow (≈ 1.8e19).
        assert!(r.ad_line.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn nonfinite_volume_skipped_without_corrupting_line() {
        let bars = vec![
            b(0, 0, 0, 0, 100_000.0, 50_000.0),
            b(0, 0, 0, 0, f64::NAN, 0.0),
            b(0, 0, 0, 0, 200_000.0, 50_000.0),
        ];
        let r = compute(&bars);
        // Bar 1 contributes 50k; bar 2 (NaN) skipped → still 50k; bar 3 adds 150k → 200k.
        assert!((r.up_down_volume_line[0] - 50_000.0).abs() < 1e-9);
        assert!((r.up_down_volume_line[1] - 50_000.0).abs() < 1e-9);
        assert!((r.up_down_volume_line[2] - 200_000.0).abs() < 1e-9);
    }

    #[test]
    fn long_negative_streak_produces_falling_ad_line() {
        let bars: Vec<BreadthBar> = (0..30).map(|_| b(500, 2500, 0, 50, 0.0, 0.0)).collect();
        let r = compute(&bars);
        assert!(r.ad_line[29] < r.ad_line[0]);
        assert!(r.net_new_highs_line[29] < r.net_new_highs_line[0]);
    }
}
