//! Donchian Channels — Richard Donchian (1960s).
//!
//! The trend-following indicator at the heart of the Turtle Trading
//! system. Three bands:
//!
//!   Upper_t  = max(high) over last N bars
//!   Lower_t  = min(low)  over last N bars
//!   Middle_t = (Upper + Lower) / 2
//!
//! Trading rules (Turtles):
//!   - Buy when close breaks above Upper (typically N=20)
//!   - Exit long when close breaks below Lower (typically N=10)
//!   - Vice versa for shorts
//!
//! Used in: breakout systems, channel-based trailing stops, volatility
//! envelope strategies.
//!
//! Pure compute. Companion to `keltner_squeeze`, `breakout_detector`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DonchianReport {
    pub upper: Vec<Option<f64>>,
    pub middle: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], period: usize) -> DonchianReport {
    let n = bars.len();
    let mut upper = vec![None; n];
    let mut middle = vec![None; n];
    let mut lower = vec![None; n];
    if period < 2 || n < period {
        return DonchianReport {
            upper,
            middle,
            lower,
        };
    }
    for i in (period - 1)..n {
        let win = &bars[i + 1 - period..=i];
        if win
            .iter()
            .any(|b| !b.high.is_finite() || !b.low.is_finite())
        {
            continue;
        }
        let hi = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let lo = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        if !hi.is_finite() || !lo.is_finite() {
            continue;
        }
        upper[i] = Some(hi);
        lower[i] = Some(lo);
        middle[i] = Some((hi + lo) / 2.0);
    }
    DonchianReport {
        upper,
        middle,
        lower,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty_outputs() {
        let r = compute(&[], 20);
        assert!(r.upper.is_empty());
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let bars: Vec<_> = (0..30)
            .map(|i| b(101.0 + i as f64, 99.0 + i as f64))
            .collect();
        let r = compute(&bars, 1);
        assert!(r.upper.iter().all(|x| x.is_none()));
    }

    #[test]
    fn shorter_than_period_returns_all_none() {
        let bars: Vec<_> = (0..5).map(|_| b(101.0, 99.0)).collect();
        let r = compute(&bars, 20);
        assert!(r.upper.iter().all(|x| x.is_none()));
    }

    #[test]
    fn upper_is_max_high_lower_is_min_low() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                let h = 100.0 + (i as f64).sin() * 5.0;
                let l = 90.0 + (i as f64).cos() * 5.0;
                b(h, l)
            })
            .collect();
        let r = compute(&bars, 10);
        let i = 29;
        let win = &bars[20..=29];
        let expected_hi = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let expected_lo = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        assert!((r.upper[i].unwrap() - expected_hi).abs() < 1e-12);
        assert!((r.lower[i].unwrap() - expected_lo).abs() < 1e-12);
    }

    #[test]
    fn middle_is_midpoint_of_upper_and_lower() {
        let bars: Vec<_> = (0..30)
            .map(|i| b(100.0 + i as f64, 90.0 - i as f64))
            .collect();
        let r = compute(&bars, 10);
        for i in 9..30 {
            let u = r.upper[i].unwrap();
            let l = r.lower[i].unwrap();
            let m = r.middle[i].unwrap();
            assert!((m - (u + l) / 2.0).abs() < 1e-12);
        }
    }

    #[test]
    fn upward_breakout_extends_upper() {
        // Flat bars then one spike up.
        let mut bars = vec![b(101.0, 99.0); 19];
        bars.push(b(120.0, 99.0));
        let r = compute(&bars, 20);
        assert!((r.upper[19].unwrap() - 120.0).abs() < 1e-9);
        assert!((r.lower[19].unwrap() - 99.0).abs() < 1e-9);
    }

    #[test]
    fn flat_bars_yield_constant_channel() {
        let bars = vec![b(101.0, 99.0); 30];
        let r = compute(&bars, 20);
        let last_u = r.upper[29].unwrap();
        let last_l = r.lower[29].unwrap();
        let last_m = r.middle[29].unwrap();
        assert_eq!(last_u, 101.0);
        assert_eq!(last_l, 99.0);
        assert_eq!(last_m, 100.0);
    }

    #[test]
    fn outputs_align_to_input_length() {
        let bars: Vec<_> = (0..50).map(|_| b(101.0, 99.0)).collect();
        let r = compute(&bars, 20);
        assert_eq!(r.upper.len(), 50);
        assert!(r.upper[18].is_none());
        assert!(r.upper[19].is_some());
    }
}
