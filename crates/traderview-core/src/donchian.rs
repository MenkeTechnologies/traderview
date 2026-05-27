//! Donchian Channels — Richard Donchian / turtle traders.
//!
//! Per bar:
//!   upper = highest_high_N
//!   lower = lowest_low_N
//!   middle = (upper + lower) / 2
//!
//! Classic 20-period breakout entry (close above upper or below lower) +
//! 10-period exit (opposite breakout) is the canonical "turtle" rule.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DonchianPoint {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    /// True when this bar's close exceeded the prior bar's upper band.
    pub upper_breakout: bool,
    pub lower_breakout: bool,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<DonchianPoint> {
    let n = bars.len();
    let mut out = vec![DonchianPoint::default(); n];
    if n < period || period == 0 {
        return out;
    }
    let mut prev_upper = 0.0;
    let mut prev_lower = 0.0;
    for i in (period - 1)..n {
        // Use the PRIOR window (exclusive of today) so today's break of
        // the band is a real signal.
        let window = if i + 1 > period {
            &bars[(i - period)..i]
        } else {
            &bars[..i]
        };
        let upper = if window.is_empty() {
            bars[i].high
        } else {
            window
                .iter()
                .map(|b| b.high)
                .fold(f64::NEG_INFINITY, f64::max)
        };
        let lower = if window.is_empty() {
            bars[i].low
        } else {
            window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min)
        };
        let middle = (upper + lower) / 2.0;
        let upper_break = i > 0 && bars[i].close > prev_upper;
        let lower_break = i > 0 && bars[i].close < prev_lower;
        out[i] = DonchianPoint {
            upper,
            middle,
            lower,
            upper_breakout: upper_break,
            lower_breakout: lower_break,
        };
        prev_upper = upper;
        prev_lower = lower;
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
        assert!(compute(&[], 20).is_empty());
    }

    #[test]
    fn under_period_returns_zero_points() {
        let bars = vec![b(10.0, 9.0, 9.5); 5];
        let out = compute(&bars, 20);
        for p in &out {
            assert_eq!(p.upper, 0.0);
        }
    }

    #[test]
    fn upper_band_at_highest_high_in_prior_window() {
        // Window of 5 prior bars: highs 10, 11, 12, 13, 14 → upper = 14.
        let bars = vec![
            b(10.0, 9.0, 9.5),
            b(11.0, 10.0, 10.5),
            b(12.0, 11.0, 11.5),
            b(13.0, 12.0, 12.5),
            b(14.0, 13.0, 13.5),
            b(15.0, 14.0, 16.0), // close above upper (14)
        ];
        let out = compute(&bars, 5);
        assert_eq!(out[5].upper, 14.0);
        assert!(out[5].upper_breakout);
    }

    #[test]
    fn lower_band_at_lowest_low_in_prior_window() {
        let bars = vec![
            b(20.0, 15.0, 18.0),
            b(20.0, 14.0, 17.0),
            b(20.0, 13.0, 16.0),
            b(20.0, 12.0, 15.0),
            b(20.0, 11.0, 14.0),
            b(20.0, 10.0, 9.0), // close below lower (11)
        ];
        let out = compute(&bars, 5);
        assert_eq!(out[5].lower, 11.0);
        assert!(out[5].lower_breakout);
    }

    #[test]
    fn middle_is_upper_lower_average() {
        let bars: Vec<Bar> = (1..=10)
            .map(|i| {
                let h = i as f64;
                let l = i as f64 - 0.5;
                b(h, l, (h + l) / 2.0)
            })
            .collect();
        let out = compute(&bars, 5);
        let p = &out[5];
        assert!((p.middle - (p.upper + p.lower) / 2.0).abs() < 1e-9);
    }

    #[test]
    fn no_breakout_when_close_inside_bands() {
        let bars: Vec<Bar> = (1..=10).map(|_| b(100.0, 99.0, 99.5)).collect();
        let out = compute(&bars, 5);
        let p = &out[9];
        assert!(!p.upper_breakout);
        assert!(!p.lower_breakout);
    }

    #[test]
    fn close_equal_to_upper_not_breakout() {
        // 5 stable bars at high 10, then bar 6 with close exactly at 10.
        // prev_upper = 10, close = 10 → strict > false → no breakout.
        let bars = vec![
            b(10.0, 8.0, 9.0),
            b(10.0, 8.0, 9.0),
            b(10.0, 8.0, 9.0),
            b(10.0, 8.0, 9.0),
            b(10.0, 8.0, 9.0),
            b(10.0, 8.0, 10.0), // close exactly at upper
        ];
        let out = compute(&bars, 5);
        assert_eq!(out[5].upper, 10.0);
        assert!(!out[5].upper_breakout, "strict > comparison required");
    }
}
