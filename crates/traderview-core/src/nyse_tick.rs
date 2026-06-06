//! NYSE TICK ($TICK) — instantaneous (advancing uptick - declining
//! downtick) count across all NYSE issues.
//!
//! Caller supplies the per-second/per-minute TICK values; this module
//! provides analysis helpers:
//!
//!   - EMA smoothing over `ema_period`
//!   - extreme flags: |TICK| > ext_threshold (default 1000 → extreme
//!     buying/selling)
//!   - persistence: how many consecutive bars the smoothed reading
//!     has remained on the same side of zero
//!
//! TICK extremes are classic intraday reversal markers (very high
//! TICK → most issues already up → buying exhaustion likely; very low
//! TICK → most issues already down → capitulation).
//!
//! Pure compute. Companion to `arms_index` (TRIN), `breadth_lines`,
//! `arms_high_low_index`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NyseTickReport {
    pub smoothed: Vec<Option<f64>>,
    pub extreme_high: Vec<bool>,
    pub extreme_low: Vec<bool>,
    pub consecutive_sign: Vec<i32>,
    pub ema_period: usize,
    pub ext_threshold: f64,
}

pub fn compute(tick: &[f64], ema_period: usize, ext_threshold: f64) -> NyseTickReport {
    let n = tick.len();
    let mut report = NyseTickReport {
        smoothed: vec![None; n],
        extreme_high: vec![false; n],
        extreme_low: vec![false; n],
        consecutive_sign: vec![0; n],
        ema_period,
        ext_threshold,
    };
    if ema_period < 2 || !ext_threshold.is_finite() || ext_threshold <= 0.0 || n < ema_period {
        return report;
    }
    if tick.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let p_f = ema_period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = tick[..ema_period].iter().sum::<f64>() / p_f;
    report.smoothed[ema_period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &v) in tick.iter().enumerate().skip(ema_period) {
        cur = v * k + cur * (1.0 - k);
        report.smoothed[i] = Some(cur);
    }
    for (i, &v) in tick.iter().enumerate() {
        if v > ext_threshold {
            report.extreme_high[i] = true;
        }
        if v < -ext_threshold {
            report.extreme_low[i] = true;
        }
    }
    let mut run = 0_i32;
    for (i, sm) in report.smoothed.iter().enumerate() {
        if let Some(v) = sm {
            run = if *v > 0.0 {
                if run > 0 {
                    run + 1
                } else {
                    1
                }
            } else if *v < 0.0 {
                if run < 0 {
                    run - 1
                } else {
                    -1
                }
            } else {
                0
            };
            report.consecutive_sign[i] = run;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let t = vec![100.0_f64; 50];
        let r = compute(&t, 1, 1000.0);
        assert!(r.smoothed.iter().all(|x| x.is_none()));
        let r2 = compute(&t, 14, 0.0);
        assert!(r2.smoothed.iter().all(|x| x.is_none()));
        let r3 = compute(&t[..5], 14, 1000.0);
        assert!(r3.smoothed.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut t = vec![100.0_f64; 50];
        t[5] = f64::NAN;
        let r = compute(&t, 14, 1000.0);
        assert!(r.smoothed.iter().all(|x| x.is_none()));
    }

    #[test]
    fn extreme_high_flagged() {
        let mut t = vec![100.0_f64; 30];
        t[15] = 1500.0;
        let r = compute(&t, 14, 1000.0);
        assert!(r.extreme_high[15]);
        assert!(!r.extreme_low[15]);
    }

    #[test]
    fn extreme_low_flagged() {
        let mut t = vec![-100.0_f64; 30];
        t[15] = -1500.0;
        let r = compute(&t, 14, 1000.0);
        assert!(r.extreme_low[15]);
        assert!(!r.extreme_high[15]);
    }

    #[test]
    fn persistent_positive_bars_increment_run() {
        let t = vec![500.0_f64; 30];
        let r = compute(&t, 14, 1000.0);
        // After EMA settles positive, run grows monotonically.
        let last = 29;
        let prev = 28;
        assert!(r.consecutive_sign[last] > r.consecutive_sign[prev]);
        assert!(r.consecutive_sign[last] > 0);
    }

    #[test]
    fn persistent_negative_bars_decrement_run() {
        let t = vec![-500.0_f64; 30];
        let r = compute(&t, 14, 1000.0);
        let last = 29;
        let prev = 28;
        assert!(r.consecutive_sign[last] < r.consecutive_sign[prev]);
        assert!(r.consecutive_sign[last] < 0);
    }

    #[test]
    fn output_lengths_match_input() {
        let t = vec![100.0_f64; 50];
        let r = compute(&t, 14, 1000.0);
        assert_eq!(r.smoothed.len(), 50);
        assert_eq!(r.extreme_high.len(), 50);
        assert_eq!(r.extreme_low.len(), 50);
        assert_eq!(r.consecutive_sign.len(), 50);
    }
}
