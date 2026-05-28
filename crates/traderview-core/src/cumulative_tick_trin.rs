//! Cumulative TICK and TRIN — intraday market-internals integrators.
//!
//! TICK = `up_ticks − down_ticks` across NYSE stocks (live tape).
//! Cumulative TICK = running sum of intraday TICK readings, reset each
//! day. Used to confirm/diverge from index moves: index up + cumulative
//! TICK negative = distribution.
//!
//! TRIN (Arms Index) = (adv / dec) / (up_vol / down_vol). >1 = bearish.
//! Cumulative TRIN's classic reading is the 5/10-day TRIN moving avg:
//! a moving avg ≥ 1.5 sustained marks oversold extremes (contrarian buy).
//!
//! Caller supplies per-bar `(tick, trin)` samples. Both can be NaN at a
//! sample and the integrator skips that point.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sample {
    pub tick: f64,
    pub trin: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Report {
    pub cumulative_tick: Vec<f64>,
    pub trin_sma: Vec<Option<f64>>,
    /// Indices where TRIN SMA crosses above `oversold_threshold` from
    /// below (contrarian buy signal).
    pub oversold_signals: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub trin_period: usize,
    pub oversold_threshold: f64,
}

impl Default for Config {
    fn default() -> Self { Self { trin_period: 5, oversold_threshold: 1.5 } }
}

pub fn compute(samples: &[Sample], cfg: &Config) -> Report {
    let n = samples.len();
    let mut report = Report {
        cumulative_tick: vec![0.0; n],
        trin_sma: vec![None; n],
        oversold_signals: Vec::new(),
    };
    if cfg.trin_period == 0 || !cfg.oversold_threshold.is_finite() {
        return report;
    }
    // Cumulative TICK.
    let mut acc = 0.0_f64;
    for (i, s) in samples.iter().enumerate() {
        if s.tick.is_finite() {
            acc += s.tick;
        }
        if !acc.is_finite() { acc = 0.0; }
        report.cumulative_tick[i] = acc;
    }
    // Rolling SMA of TRIN.
    let p = cfg.trin_period;
    if n >= p {
        for i in (p - 1)..n {
            let window = &samples[i + 1 - p..=i];
            // All trins must be finite + positive to compute the average.
            let mut sum = 0.0;
            let mut ok = true;
            for s in window {
                if !s.trin.is_finite() || s.trin <= 0.0 { ok = false; break; }
                sum += s.trin;
            }
            if ok {
                let v = sum / p as f64;
                if v.is_finite() {
                    report.trin_sma[i] = Some(v);
                }
            }
        }
    }
    // Cross-above signals.
    for i in 1..n {
        if let (Some(prev), Some(now)) = (report.trin_sma[i - 1], report.trin_sma[i]) {
            if prev < cfg.oversold_threshold && now >= cfg.oversold_threshold {
                report.oversold_signals.push(i);
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(tick: f64, trin: f64) -> Sample {
        Sample { tick, trin }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], &Config::default());
        assert!(r.cumulative_tick.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let samples = vec![s(100.0, 1.0); 10];
        let r = compute(&samples, &Config { trin_period: 0, ..Default::default() });
        assert!(r.trin_sma.iter().all(|x| x.is_none()));
        let r = compute(&samples, &Config { oversold_threshold: f64::NAN, ..Default::default() });
        assert!(r.trin_sma.iter().all(|x| x.is_none()));
    }

    #[test]
    fn cumulative_tick_sums_correctly() {
        let samples = vec![s(100.0, 1.0), s(-50.0, 1.0), s(25.0, 1.0)];
        let r = compute(&samples, &Config::default());
        assert_eq!(r.cumulative_tick, vec![100.0, 50.0, 75.0]);
    }

    #[test]
    fn nan_tick_treated_as_zero_increment() {
        let samples = vec![s(100.0, 1.0), s(f64::NAN, 1.0), s(50.0, 1.0)];
        let r = compute(&samples, &Config::default());
        assert_eq!(r.cumulative_tick, vec![100.0, 100.0, 150.0]);
    }

    #[test]
    fn trin_sma_populated_after_warmup() {
        let samples = vec![s(0.0, 1.2); 10];
        let r = compute(&samples, &Config::default());
        // First 4 None (period=5), then populated.
        assert!(r.trin_sma[3].is_none());
        let v = r.trin_sma[4].expect("populated");
        assert!((v - 1.2).abs() < 1e-9);
    }

    #[test]
    fn oversold_cross_above_detected() {
        // TRIN rises from 1.0 → 2.0 — crosses 1.5 threshold.
        let mut samples: Vec<Sample> = Vec::new();
        for _ in 0..5 { samples.push(s(0.0, 1.0)); }    // SMA = 1.0
        for _ in 0..5 { samples.push(s(0.0, 2.0)); }    // SMA pulled toward 2
        let r = compute(&samples, &Config::default());
        assert!(!r.oversold_signals.is_empty(), "should cross 1.5 threshold");
    }

    #[test]
    fn zero_or_negative_trin_skips_window() {
        // Negative TRIN is malformed; SMA window containing one is None.
        let mut samples = vec![s(0.0, 1.0); 10];
        samples[3].trin = -1.0;
        let r = compute(&samples, &Config::default());
        // Windows containing index 3 (positions 3..=7) all None.
        for i in 3..=7 {
            assert!(r.trin_sma[i].is_none(), "i={i}");
        }
    }
}
