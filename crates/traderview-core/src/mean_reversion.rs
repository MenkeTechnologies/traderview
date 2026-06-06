//! Mean-reversion signal — z-score + extension threshold.
//!
//! Tradeable mean-reversion needs three things:
//!   1. Price is statistically far from its mean (high |z-score|).
//!   2. The series is mean-reverting (not trending) — measured here by
//!      the share of the lookback window where price crossed the mean.
//!   3. A confirming cross or reversal candle to trigger entry — we don't
//!      detect candle patterns here (use crate::candlestick_patterns) but
//!      we surface the latest z and the most-recent cross direction so
//!      the caller can wire a confirmation rule.
//!
//! Output: a verdict (OverboughtMeanReversion / OversoldMeanReversion /
//! Trending / Neutral) plus the underlying z-score and mean-cross density.
//!
//! Pure compute. Distinct from `rolling_zscore` (which produces a series)
//! — this module emits a single tradeable verdict for the latest bar.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanRevConfig {
    /// Lookback window (bars).
    pub window: usize,
    /// Absolute z-score threshold for an "extension" signal.
    pub extension_z: f64,
    /// Minimum mean-cross density (crosses / window) to qualify the
    /// series as mean-reverting rather than trending.
    pub min_cross_density: f64,
}

impl Default for MeanRevConfig {
    fn default() -> Self {
        Self {
            window: 20,
            extension_z: 2.0,
            min_cross_density: 0.15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    OverboughtMeanReversion,
    OversoldMeanReversion,
    Trending,
    #[default]
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeanRevReport {
    pub verdict: Verdict,
    pub latest_z: f64,
    pub mean: f64,
    pub stdev: f64,
    /// Count of mean-crossings within the window (price moving from above to
    /// below or vice versa).
    pub mean_cross_count: usize,
    pub mean_cross_density: f64,
    pub note: String,
}

pub fn analyze(closes: &[f64], cfg: &MeanRevConfig) -> MeanRevReport {
    let n = closes.len();
    if n == 0 || cfg.window == 0 || n < cfg.window {
        return MeanRevReport {
            note: format!("need at least {} closes, got {}", cfg.window, n),
            ..Default::default()
        };
    }
    let slice = &closes[n - cfg.window..];
    let mean: f64 = slice.iter().sum::<f64>() / cfg.window as f64;
    let var: f64 = slice.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / cfg.window as f64;
    let stdev = var.sqrt();
    let latest = *slice.last().expect("window > 0");
    let z = if stdev > 0.0 {
        (latest - mean) / stdev
    } else {
        0.0
    };
    // Count mean-crossings within the window.
    let mut crosses = 0usize;
    let mut prev_above = slice[0] > mean;
    for &c in &slice[1..] {
        let above = c > mean;
        if above != prev_above {
            crosses += 1;
        }
        prev_above = above;
    }
    let cross_density = crosses as f64 / cfg.window as f64;
    let mean_reverting = cross_density >= cfg.min_cross_density;

    let verdict = if !mean_reverting {
        Verdict::Trending
    } else if z >= cfg.extension_z {
        Verdict::OverboughtMeanReversion
    } else if z <= -cfg.extension_z {
        Verdict::OversoldMeanReversion
    } else {
        Verdict::Neutral
    };
    let note = match verdict {
        Verdict::OverboughtMeanReversion => format!(
            "z={:.2} ≥ {:.2} in a mean-reverting series — fade longs",
            z, cfg.extension_z
        ),
        Verdict::OversoldMeanReversion => {
            format!("z={:.2} ≤ -{:.2} — fade shorts", z, cfg.extension_z)
        }
        Verdict::Trending => format!(
            "cross density {:.2} < {:.2} — series is trending, don't fade",
            cross_density, cfg.min_cross_density
        ),
        Verdict::Neutral => format!(
            "z={:.2} within ±{:.2} — no extension signal",
            z, cfg.extension_z
        ),
    };
    MeanRevReport {
        verdict,
        latest_z: z,
        mean,
        stdev,
        mean_cross_count: crosses,
        mean_cross_density: cross_density,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_closes_returns_default_with_note() {
        let r = analyze(&[1.0, 2.0, 3.0], &MeanRevConfig::default());
        assert!(matches!(r.verdict, Verdict::Neutral));
        assert!(r.note.contains("at least"));
    }

    #[test]
    fn pure_uptrend_is_trending_not_mean_reverting() {
        // Monotonic uptrend → 0 mean crosses in the window → trending.
        let v: Vec<f64> = (1..=25).map(|i| i as f64).collect();
        let r = analyze(&v, &MeanRevConfig::default());
        assert!(
            matches!(r.verdict, Verdict::Trending),
            "monotonic series must classify as Trending, got {:?}",
            r.verdict
        );
        assert_eq!(
            r.mean_cross_count, 1,
            "monotonic = exactly 1 cross at the midpoint of the window"
        );
    }

    #[test]
    fn oscillating_series_with_extreme_latest_is_overbought() {
        // Saw-tooth oscillation gives high cross density. Force the LAST close
        // to be far above the mean for an OverboughtMeanReversion verdict.
        let mut v: Vec<f64> = (0..20)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        // Push the last sample to +5σ above the rolling mean (~0).
        v.push(10.0);
        let cfg = MeanRevConfig {
            window: 21,
            extension_z: 2.0,
            min_cross_density: 0.15,
        };
        let r = analyze(&v, &cfg);
        assert!(
            matches!(r.verdict, Verdict::OverboughtMeanReversion),
            "expected OverboughtMeanReversion, got {:?} (z={}, cross={})",
            r.verdict,
            r.latest_z,
            r.mean_cross_density
        );
    }

    #[test]
    fn oscillating_with_extreme_low_is_oversold() {
        let mut v: Vec<f64> = (0..20)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        v.push(-10.0);
        let cfg = MeanRevConfig {
            window: 21,
            extension_z: 2.0,
            min_cross_density: 0.15,
        };
        let r = analyze(&v, &cfg);
        assert!(matches!(r.verdict, Verdict::OversoldMeanReversion));
    }

    #[test]
    fn within_bounds_is_neutral() {
        // Oscillating ±1 around 0, last value near the mean.
        let v: Vec<f64> = (0..20)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let r = analyze(&v, &MeanRevConfig::default());
        assert!(
            matches!(r.verdict, Verdict::Neutral),
            "small z should be Neutral, got {:?}",
            r.verdict
        );
        assert!(r.latest_z.abs() < 2.0);
    }

    #[test]
    fn zero_variance_doesnt_panic() {
        let v = vec![5.0; 25];
        let r = analyze(&v, &MeanRevConfig::default());
        // Constant series has zero variance → z=0 → Neutral verdict.
        assert_eq!(r.latest_z, 0.0);
        // Cross count = 0 → cross_density = 0 → Trending verdict.
        assert!(
            matches!(r.verdict, Verdict::Trending),
            "constant series should be Trending (0 crosses), got {:?}",
            r.verdict
        );
    }
}
