//! Aroon — Tushar Chande (1995).
//!
//! Identifies trend strength + direction based on how recently the
//! N-period high (or low) occurred:
//!
//!   Aroon Up = (N - bars_since_highest_high) / N × 100
//!   Aroon Down = (N - bars_since_lowest_low) / N × 100
//!   Aroon Oscillator = Aroon Up - Aroon Down
//!
//! Aroon Up at 100 = highest high was just hit. Up/down crossover
//! → trend change. Oscillator > +50 = strong uptrend, < -50 strong
//! downtrend. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AroonPoint {
    pub up: f64,
    pub down: f64,
    pub oscillator: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<AroonPoint> {
    let n = bars.len();
    let mut out = vec![AroonPoint::default(); n];
    if n < period + 1 || period == 0 {
        return out;
    }
    for i in period..n {
        let window = &bars[(i - period)..=i];
        let mut high_at = 0usize;
        let mut low_at = 0usize;
        let mut max_h = f64::NEG_INFINITY;
        let mut min_l = f64::INFINITY;
        for (j, b) in window.iter().enumerate() {
            // Use >= / <= so ties favor the MOST RECENT occurrence,
            // matching Aroon convention ("days since the N-day high").
            // Using strict > would lock onto the first tie and produce
            // stale Aroon Up readings during choppy congestion.
            if b.high >= max_h {
                max_h = b.high;
                high_at = j;
            }
            if b.low <= min_l {
                min_l = b.low;
                low_at = j;
            }
        }
        let bars_since_high = period - high_at;
        let bars_since_low = period - low_at;
        let aroon_up = (period - bars_since_high) as f64 / period as f64 * 100.0;
        let aroon_down = (period - bars_since_low) as f64 / period as f64 * 100.0;
        out[i] = AroonPoint {
            up: aroon_up,
            down: aroon_down,
            oscillator: aroon_up - aroon_down,
        };
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AroonTrend {
    StrongUp,
    Up,
    Sideways,
    Down,
    StrongDown,
}

pub fn classify(osc: f64) -> AroonTrend {
    if osc > 50.0 {
        AroonTrend::StrongUp
    } else if osc > 0.0 {
        AroonTrend::Up
    } else if osc < -50.0 {
        AroonTrend::StrongDown
    } else if osc < 0.0 {
        AroonTrend::Down
    } else {
        AroonTrend::Sideways
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn under_period_plus_one_returns_zeros() {
        let bars = vec![b(10.0, 9.0); 10];
        let out = compute(&bars, 14);
        for p in &out {
            assert_eq!(p.up, 0.0);
        }
    }

    #[test]
    fn most_recent_high_aroon_up_100() {
        // Last bar has the highest high → Aroon Up = 100.
        // Distinct lows so the most-recent-tied rule doesn't collapse
        // Aroon Down to 100 also (which would zero out oscillator).
        let mut bars: Vec<Bar> = (1..=14)
            .map(|i| b(i as f64, 10.0 - i as f64 * 0.1))
            .collect();
        bars.push(b(100.0, 20.0)); // huge new high, NOT lowest low
        let out = compute(&bars, 14);
        let last = &out[14];
        assert_eq!(last.up, 100.0);
        assert!(last.oscillator > 0.0);
    }

    #[test]
    fn most_recent_low_aroon_down_100() {
        let mut bars: Vec<Bar> = (1..=14).map(|i| b(20.0, 20.0 - i as f64 * 0.1)).collect();
        bars.push(b(20.0, 0.0)); // new low
        let out = compute(&bars, 14);
        let last = &out[14];
        assert_eq!(last.down, 100.0);
    }

    #[test]
    fn strong_uptrend_aroon_up_dominant() {
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let p = 100.0 + i as f64;
                b(p, p - 1.0)
            })
            .collect();
        let out = compute(&bars, 14);
        assert!(out[19].up > out[19].down);
        assert!(out[19].oscillator > 50.0);
    }

    #[test]
    fn strong_downtrend_aroon_down_dominant() {
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let p = 200.0 - i as f64;
                b(p, p - 1.0)
            })
            .collect();
        let out = compute(&bars, 14);
        assert!(out[19].down > out[19].up);
        assert!(out[19].oscillator < -50.0);
    }

    #[test]
    fn tied_highs_use_most_recent_occurrence() {
        // 5 bars all at high=100, period=4. Window has 5 bars all tied.
        // Most-recent-occurrence convention: high_at = last index (4),
        // bars_since_high = 0 → Aroon Up = 100.
        // (With the earlier bug: high_at would lock to 0 and Aroon Up
        // would underestimate to 0.)
        let bars = vec![b(100.0, 90.0); 5];
        let out = compute(&bars, 4);
        assert_eq!(
            out[4].up, 100.0,
            "tied highs must use MOST RECENT occurrence"
        );
    }

    #[test]
    fn tied_lows_use_most_recent_occurrence() {
        let bars = vec![b(110.0, 100.0); 5];
        let out = compute(&bars, 4);
        assert_eq!(out[4].down, 100.0);
    }

    // ─── classify ──────────────────────────────────────────────────────

    #[test]
    fn classify_above_50_strong_up() {
        assert_eq!(classify(60.0), AroonTrend::StrongUp);
    }

    #[test]
    fn classify_under_minus_50_strong_down() {
        assert_eq!(classify(-60.0), AroonTrend::StrongDown);
    }

    #[test]
    fn classify_zero_sideways() {
        assert_eq!(classify(0.0), AroonTrend::Sideways);
    }

    #[test]
    fn classify_positive_under_50_up() {
        assert_eq!(classify(20.0), AroonTrend::Up);
    }

    #[test]
    fn classify_negative_above_minus_50_down() {
        assert_eq!(classify(-20.0), AroonTrend::Down);
    }
}
