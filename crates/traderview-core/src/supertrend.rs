//! SuperTrend indicator — Olivier Seban.
//!
//! Per bar:
//!   median = (high + low) / 2
//!   upper_basic = median + multiplier × ATR
//!   lower_basic = median - multiplier × ATR
//!
//! Then with "trend stickiness":
//!   final_upper_t = min(upper_basic_t, final_upper_{t-1}) if close > final_upper_{t-1}
//!   final_lower_t = max(lower_basic_t, final_lower_{t-1}) if close < final_lower_{t-1}
//!
//! Output: SuperTrend value (acts as dynamic S/R + trailing stop) and
//! trend direction (1 = up, -1 = down).
//!
//! Pure compute. Standard parameters: 10-period ATR, multiplier 3.0.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SupertrendPoint {
    pub upper_band: f64,
    pub lower_band: f64,
    pub super_trend: f64,
    /// 1 = uptrend (close above), -1 = downtrend, 0 = uninitialized.
    pub trend: i8,
}

pub fn compute(bars: &[Bar], atr: &[f64], multiplier: f64) -> Vec<SupertrendPoint> {
    let n = bars.len();
    let mut out = vec![SupertrendPoint::default(); n];
    if n == 0 || atr.len() != n {
        return out;
    }
    let mut prev_upper = 0.0;
    let mut prev_lower = 0.0;
    let mut prev_trend = 0i8;
    let mut prev_close = bars.first().map(|b| b.close).unwrap_or(0.0);
    for i in 0..n {
        let b = bars[i];
        let median = (b.high + b.low) / 2.0;
        let basic_upper = median + multiplier * atr[i];
        let basic_lower = median - multiplier * atr[i];

        // Final upper: tighten if prior trend was up.
        let final_upper = if i == 0 || basic_upper < prev_upper || prev_close > prev_upper {
            basic_upper
        } else {
            prev_upper
        };
        let final_lower = if i == 0 || basic_lower > prev_lower || prev_close < prev_lower {
            basic_lower
        } else {
            prev_lower
        };
        let trend = if i == 0 {
            if b.close >= median {
                1
            } else {
                -1
            }
        } else if prev_trend == 1 && b.close < final_lower {
            -1
        } else if prev_trend == -1 && b.close > final_upper {
            1
        } else if prev_trend == 0 {
            if b.close >= median {
                1
            } else {
                -1
            }
        } else {
            prev_trend
        };
        let super_trend = if trend == 1 { final_lower } else { final_upper };
        out[i] = SupertrendPoint {
            upper_band: final_upper,
            lower_band: final_lower,
            super_trend,
            trend,
        };
        prev_upper = final_upper;
        prev_lower = final_lower;
        prev_trend = trend;
        prev_close = b.close;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 3.0).is_empty());
    }

    #[test]
    fn mismatched_atr_length_returns_zero_points() {
        let bars = vec![b(10.0, 9.0, 9.5)];
        let atr = vec![1.0, 1.0];
        let out = compute(&bars, &atr, 3.0);
        // Empty since len mismatch.
        for p in &out {
            assert_eq!(p.trend, 0);
        }
    }

    #[test]
    fn uptrend_persists_when_price_stays_above_lower_band() {
        let bars: Vec<Bar> = (1..=15)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let atr = vec![1.0; 15];
        let out = compute(&bars, &atr, 3.0);
        // After initial bar, trend should establish and persist up.
        assert_eq!(out[14].trend, 1);
    }

    #[test]
    fn downtrend_persists_when_price_stays_below_upper_band() {
        let bars: Vec<Bar> = (1..=15)
            .map(|i| {
                let c = 200.0 - i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let atr = vec![1.0; 15];
        let out = compute(&bars, &atr, 3.0);
        assert_eq!(out[14].trend, -1);
    }

    #[test]
    fn super_trend_is_lower_band_in_uptrend() {
        let bars: Vec<Bar> = (1..=10)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let atr = vec![1.0; 10];
        let out = compute(&bars, &atr, 3.0);
        let last = &out[9];
        assert_eq!(last.trend, 1);
        assert_eq!(last.super_trend, last.lower_band);
    }

    #[test]
    fn super_trend_is_upper_band_in_downtrend() {
        let bars: Vec<Bar> = (1..=10)
            .map(|i| {
                let c = 200.0 - i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let atr = vec![1.0; 10];
        let out = compute(&bars, &atr, 3.0);
        let last = &out[9];
        assert_eq!(last.trend, -1);
        assert_eq!(last.super_trend, last.upper_band);
    }

    #[test]
    fn trend_flips_when_close_pierces_supertrend() {
        // Uptrend then sudden close below lower band → flip to -1.
        let mut bars: Vec<Bar> = (1..=10)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        bars.push(b(102.0, 70.0, 70.0)); // huge drop
        let atr = vec![1.0; 11];
        let out = compute(&bars, &atr, 3.0);
        assert_eq!(out[10].trend, -1, "huge drop should flip trend");
    }

    #[test]
    fn larger_multiplier_wider_bands() {
        let bars = vec![b(101.0, 99.0, 100.0); 5];
        let atr = vec![1.0; 5];
        let tight = compute(&bars, &atr, 1.0);
        let wide = compute(&bars, &atr, 5.0);
        let tight_range = tight[4].upper_band - tight[4].lower_band;
        let wide_range = wide[4].upper_band - wide[4].lower_band;
        assert!(wide_range > tight_range);
    }
}
