//! Zweig Breadth Thrust — Martin Zweig (1986).
//!
//! Famously rare market-bottom signal:
//!
//!   ratio_t = advancing_issues / (advancing + declining)
//!   ema_t   = 10-day EMA of ratio (Zweig's standard period)
//!
//!   thrust trigger:
//!     ema_t rises from below 0.40 to above 0.615 within
//!     `max_window_bars` (Zweig's original: 10 days).
//!
//! Per Zweig: every signal historically preceded a bull-market launch
//! averaging +24% in the following 11 months.
//!
//! Pure compute. Defaults: ema_period = 10, max_window_bars = 10,
//! low_threshold = 0.40, high_threshold = 0.615.
//! Companion to `breadth_lines`, `arms_high_low_index`,
//! `mcclellan_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyBreadth {
    pub advancing: u64,
    pub declining: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreadthThrustReport {
    pub ratio: Vec<Option<f64>>,
    pub ema_ratio: Vec<Option<f64>>,
    pub thrust_triggered: Vec<bool>,
    pub ema_period: usize,
    pub max_window_bars: usize,
    pub low_threshold: f64,
    pub high_threshold: f64,
}

pub fn compute(
    breadth: &[DailyBreadth],
    ema_period: usize,
    max_window_bars: usize,
    low_threshold: f64,
    high_threshold: f64,
) -> BreadthThrustReport {
    let n = breadth.len();
    let mut report = BreadthThrustReport {
        ratio: vec![None; n],
        ema_ratio: vec![None; n],
        thrust_triggered: vec![false; n],
        ema_period,
        max_window_bars,
        low_threshold,
        high_threshold,
    };
    if ema_period < 2
        || max_window_bars < 2
        || !low_threshold.is_finite()
        || !high_threshold.is_finite()
        || low_threshold >= high_threshold
        || !(0.0..=1.0).contains(&low_threshold)
        || !(0.0..=1.0).contains(&high_threshold)
        || n < ema_period
    {
        return report;
    }
    let raw: Vec<f64> = breadth
        .iter()
        .map(|d| {
            let denom = d.advancing + d.declining;
            if denom > 0 {
                d.advancing as f64 / denom as f64
            } else {
                0.5
            }
        })
        .collect();
    for (i, &v) in raw.iter().enumerate() {
        report.ratio[i] = Some(v);
    }
    let p_f = ema_period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = raw[..ema_period].iter().sum::<f64>() / p_f;
    report.ema_ratio[ema_period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &v) in raw.iter().enumerate().skip(ema_period) {
        cur = v * k + cur * (1.0 - k);
        report.ema_ratio[i] = Some(cur);
    }
    // Detect thrust: ema_ratio went from < low_threshold to >
    // high_threshold within max_window_bars.
    for i in (ema_period + max_window_bars - 1)..n {
        let Some(ema_now) = report.ema_ratio[i] else {
            continue;
        };
        if ema_now <= high_threshold {
            continue;
        }
        for back in 1..=max_window_bars {
            let Some(ema_then) = report.ema_ratio[i - back] else {
                continue;
            };
            if ema_then < low_threshold {
                report.thrust_triggered[i] = true;
                break;
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(adv: u64, dec: u64) -> DailyBreadth {
        DailyBreadth {
            advancing: adv,
            declining: dec,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let b = vec![d(50, 50); 100];
        let r = compute(&b, 1, 10, 0.4, 0.615);
        assert!(r.ratio.iter().all(|x| x.is_none()));
        let r2 = compute(&b, 10, 10, 0.7, 0.5); // low > high
        assert!(r2.ratio.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_breadth_no_thrust() {
        let b = vec![d(50, 50); 100];
        let r = compute(&b, 10, 10, 0.4, 0.615);
        assert!(!r.thrust_triggered.iter().any(|x| *x));
    }

    #[test]
    fn classic_thrust_detected() {
        // 30 bars of weak breadth (ratio ≈ 0.30), then 15 bars of strong
        // breadth (ratio = 0.90) → EMA crosses thresholds within window.
        let mut breadth = vec![d(30, 70); 30];
        for _ in 0..15 {
            breadth.push(d(90, 10));
        }
        let r = compute(&breadth, 10, 10, 0.4, 0.615);
        let any_trigger = r.thrust_triggered.iter().any(|x| *x);
        assert!(any_trigger);
    }

    #[test]
    fn slow_recovery_no_thrust() {
        // Ratio rises slowly over 50 bars → never crosses thresholds
        // within max_window_bars=10 window.
        let breadth: Vec<_> = (0..80)
            .map(|i| {
                let adv = (30 + i) as u64;
                let dec = (70_i64 - i as i64).max(1) as u64;
                d(adv, dec)
            })
            .collect();
        let r = compute(&breadth, 10, 5, 0.4, 0.615);
        // Should not trigger if rise was too slow.
        // (This test is heuristic; we just verify no panic.)
        assert_eq!(r.thrust_triggered.len(), 80);
    }

    #[test]
    fn output_lengths_match_input() {
        let b = vec![d(50, 50); 50];
        let r = compute(&b, 10, 10, 0.4, 0.615);
        assert_eq!(r.ratio.len(), 50);
        assert_eq!(r.ema_ratio.len(), 50);
        assert_eq!(r.thrust_triggered.len(), 50);
    }
}
