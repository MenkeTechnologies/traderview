//! Dark Pool Index (DPI) — rolling % of trading volume printed off
//! exchange (TRF / dark pool prints).
//!
//!   dpi = dark_pool_volume / total_volume          per day
//!   dpi_smoothed = SMA(dpi, period)
//!
//! Convention: DPI > 0.45 (≥ 45% of volume off-exchange) sustained for
//! a week is the canonical "smart money accumulating" signal. DPI < 0.35
//! signals retail-dominated tape (often a topping condition).
//!
//! Caller supplies per-day (dark_volume, total_volume) pairs.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DarkPoolBar {
    pub dark_volume: f64,
    pub total_volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DpiReport {
    pub raw: Vec<Option<f64>>,
    pub smoothed: Vec<Option<f64>>,
    /// Indices where smoothed DPI ≥ `accumulation_threshold` for at
    /// least `min_streak_days` consecutive bars.
    pub accumulation_streaks: Vec<StreakEvent>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StreakEvent {
    pub start_index: usize,
    pub end_index: usize,
    pub mean_dpi: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpiConfig {
    pub smooth_period: usize,
    pub accumulation_threshold: f64,
    pub min_streak_days: usize,
}

impl Default for DpiConfig {
    fn default() -> Self {
        Self { smooth_period: 5, accumulation_threshold: 0.45, min_streak_days: 5 }
    }
}

pub fn compute(bars: &[DarkPoolBar], cfg: &DpiConfig) -> DpiReport {
    let n = bars.len();
    let mut report = DpiReport {
        raw: vec![None; n],
        smoothed: vec![None; n],
        accumulation_streaks: Vec::new(),
    };
    if cfg.smooth_period == 0
        || cfg.min_streak_days == 0
        || !cfg.accumulation_threshold.is_finite()
        || !(0.0..=1.0).contains(&cfg.accumulation_threshold)
    {
        return report;
    }
    for (i, b) in bars.iter().enumerate() {
        if !b.dark_volume.is_finite() || !b.total_volume.is_finite() || b.total_volume <= 0.0 {
            continue;
        }
        let r = b.dark_volume / b.total_volume;
        if (0.0..=1.0).contains(&r) {
            report.raw[i] = Some(r);
        }
    }
    // Smoothed SMA over raw.
    let p = cfg.smooth_period;
    for i in (p - 1)..n {
        let window = &report.raw[i + 1 - p..=i];
        if let Some(sum) = window.iter().try_fold(0.0_f64, |s, x| x.map(|v| s + v)) {
            report.smoothed[i] = Some(sum / p as f64);
        }
    }
    // Streak detector: scan smoothed for consecutive bars ≥ threshold.
    let mut run_start: Option<usize> = None;
    for i in 0..n {
        match report.smoothed[i] {
            Some(v) if v >= cfg.accumulation_threshold => {
                if run_start.is_none() {
                    run_start = Some(i);
                }
            }
            _ => {
                if let Some(start) = run_start {
                    let end = i - 1;
                    let len = end - start + 1;
                    if len >= cfg.min_streak_days {
                        let mean = report.smoothed[start..=end]
                            .iter().filter_map(|x| *x).sum::<f64>() / len as f64;
                        report.accumulation_streaks.push(StreakEvent {
                            start_index: start, end_index: end, mean_dpi: mean,
                        });
                    }
                    run_start = None;
                }
            }
        }
    }
    // Flush trailing run.
    if let Some(start) = run_start {
        let end = n - 1;
        let len = end - start + 1;
        if len >= cfg.min_streak_days {
            let mean = report.smoothed[start..=end]
                .iter().filter_map(|x| *x).sum::<f64>() / len as f64;
            report.accumulation_streaks.push(StreakEvent {
                start_index: start, end_index: end, mean_dpi: mean,
            });
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(d: f64, t: f64) -> DarkPoolBar {
        DarkPoolBar { dark_volume: d, total_volume: t }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], &DpiConfig::default());
        assert!(r.raw.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let bars = vec![b(500.0, 1000.0); 20];
        for cfg in [
            DpiConfig { smooth_period: 0, ..Default::default() },
            DpiConfig { accumulation_threshold: 1.5, ..Default::default() },
            DpiConfig { accumulation_threshold: -1.0, ..Default::default() },
            DpiConfig { min_streak_days: 0, ..Default::default() },
        ] {
            let r = compute(&bars, &cfg);
            assert!(r.raw.iter().all(|x| x.is_none()) && r.accumulation_streaks.is_empty());
        }
    }

    #[test]
    fn zero_total_volume_skipped() {
        let bars = vec![b(500.0, 0.0), b(500.0, 1000.0)];
        let r = compute(&bars, &DpiConfig::default());
        assert!(r.raw[0].is_none());
        assert!(r.raw[1].is_some());
    }

    #[test]
    fn flat_high_dpi_produces_streak_event() {
        // 10 bars all at DPI = 0.55 — sustained accumulation.
        let bars: Vec<DarkPoolBar> = (0..10).map(|_| b(550.0, 1000.0)).collect();
        let r = compute(&bars, &DpiConfig::default());
        assert!(!r.accumulation_streaks.is_empty());
        // Mean DPI in the streak should be ~0.55.
        let mean = r.accumulation_streaks[0].mean_dpi;
        assert!((mean - 0.55).abs() < 0.05);
    }

    #[test]
    fn low_dpi_produces_no_streak() {
        let bars: Vec<DarkPoolBar> = (0..10).map(|_| b(200.0, 1000.0)).collect();
        let r = compute(&bars, &DpiConfig::default());
        assert!(r.accumulation_streaks.is_empty());
    }

    #[test]
    fn raw_value_clamped_to_unit_interval() {
        // dark > total → > 1, should be rejected (data error).
        let bars = vec![b(2000.0, 1000.0)];
        let r = compute(&bars, &DpiConfig::default());
        assert!(r.raw[0].is_none());
    }

    #[test]
    fn short_streak_doesnt_qualify() {
        // 3 bars high, then 7 bars low — streak shorter than min_streak_days=5.
        let mut bars: Vec<DarkPoolBar> = (0..3).map(|_| b(550.0, 1000.0)).collect();
        bars.extend((0..7).map(|_| b(200.0, 1000.0)));
        let r = compute(&bars, &DpiConfig::default());
        assert!(r.accumulation_streaks.is_empty());
    }

    #[test]
    fn nan_volume_safely_skipped() {
        let bars = vec![b(f64::NAN, 1000.0), b(500.0, 1000.0)];
        let r = compute(&bars, &DpiConfig::default());
        assert!(r.raw[0].is_none());
        assert!(r.raw[1].is_some());
    }
}
