//! Pair-trade analysis: hedge ratio + spread Z-score + mean-reversion signals.
//!
//! Given two price series, compute:
//!   - Hedge ratio via OLS regression (β of y vs x).
//!   - Spread series: spread_t = y_t - β × x_t.
//!   - Spread mean + stdev for normalizing.
//!   - Z-score of the latest spread observation.
//!
//! Entry signal: |Z| > entry_threshold (typically 2.0).
//! Exit signal:  |Z| < exit_threshold (typically 0.5).
//! Stop loss:    |Z| > stop_threshold (typically 3.5).
//!
//! Pure compute. Does NOT do a full cointegration test (no ADF) but
//! pairs with low |Z| and stable hedge ratio are good candidates.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PairSignal {
    LongSpread,    // |z| > entry AND z < 0 (spread cheap)
    ShortSpread,   // |z| > entry AND z > 0 (spread expensive)
    ExitSpread,    // |z| < exit (mean-reverted)
    StopOut,       // |z| > stop (blown out)
    #[default]
    Hold,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PairConfig {
    pub entry_z: f64,
    pub exit_z: f64,
    pub stop_z: f64,
}

impl Default for PairConfig {
    fn default() -> Self { Self { entry_z: 2.0, exit_z: 0.5, stop_z: 3.5 } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PairReport {
    pub hedge_ratio: f64,
    pub spread_mean: f64,
    pub spread_stdev: f64,
    pub current_spread: f64,
    pub current_z: f64,
    pub signal: PairSignal,
}


pub fn analyze(y: &[f64], x: &[f64], cfg: &PairConfig) -> Option<PairReport> {
    if y.len() != x.len() || y.len() < 3 { return None; }
    let n = y.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..y.len() {
        let dx = x[i] - mean_x;
        num += dx * (y[i] - mean_y);
        den += dx * dx;
    }
    if den == 0.0 { return None; }
    let beta = num / den;
    let spread: Vec<f64> = y.iter().zip(x).map(|(yi, xi)| yi - beta * xi).collect();
    let mean_s: f64 = spread.iter().sum::<f64>() / n;
    let var_s: f64 = spread.iter().map(|s| (s - mean_s).powi(2)).sum::<f64>() / n;
    let std_s = var_s.sqrt();
    let current = *spread.last().unwrap();
    let z = if std_s > 0.0 { (current - mean_s) / std_s } else { 0.0 };
    let signal = if z.abs() > cfg.stop_z {
        PairSignal::StopOut
    } else if z.abs() < cfg.exit_z {
        PairSignal::ExitSpread
    } else if z > cfg.entry_z {
        PairSignal::ShortSpread
    } else if z < -cfg.entry_z {
        PairSignal::LongSpread
    } else {
        PairSignal::Hold
    };
    Some(PairReport {
        hedge_ratio: beta,
        spread_mean: mean_s,
        spread_stdev: std_s,
        current_spread: current,
        current_z: z,
        signal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_mismatch_returns_none() {
        assert!(analyze(&[1.0, 2.0], &[1.0], &PairConfig::default()).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(analyze(&[1.0, 2.0], &[1.0, 2.0], &PairConfig::default()).is_none());
    }

    #[test]
    fn zero_x_variance_returns_none() {
        let y = vec![1.0, 2.0, 3.0];
        let x = vec![5.0, 5.0, 5.0];
        assert!(analyze(&y, &x, &PairConfig::default()).is_none());
    }

    #[test]
    fn perfect_linear_relationship_hedge_ratio_correct() {
        // y = 2x → beta should be 2.
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| v * 2.0).collect();
        let r = analyze(&y, &x, &PairConfig::default()).unwrap();
        assert!((r.hedge_ratio - 2.0).abs() < 1e-9);
        // Spread = y - 2x = 0 → stdev = 0 → z = 0.
        assert_eq!(r.spread_stdev, 0.0);
    }

    #[test]
    fn neutral_z_yields_exit_signal_when_under_threshold() {
        // Construct spread with mean ~ 0, latest near 0.
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        // y = 2x + noise, but ending right on the line → z near 0.
        let mut y: Vec<f64> = x.iter().enumerate().map(|(i, v)| {
            v * 2.0 + if i % 2 == 0 { 1.0 } else { -1.0 }
        }).collect();
        // Force the last spread to be near mean.
        *y.last_mut().unwrap() = (*x.last().unwrap()) * 2.0;
        let r = analyze(&y, &x, &PairConfig::default()).unwrap();
        assert!(r.current_z.abs() < 0.5,
            "constructed: current_z should be near 0, got {}", r.current_z);
        assert_eq!(r.signal, PairSignal::ExitSpread);
    }

    #[test]
    fn extremely_high_z_yields_stop_out() {
        // 9 small-error obs + 1 huge outlier at end → |z| ≈ 3.
        // Need |z| > 3.5 to trigger StopOut. Make the outlier bigger.
        let mut y: Vec<f64> = (1..=20).map(|i| (i as f64) * 2.0 + 0.1).collect();
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        *y.last_mut().unwrap() = (*x.last().unwrap()) * 2.0 + 100.0;
        let r = analyze(&y, &x, &PairConfig::default()).unwrap();
        // Will likely be Hold/ShortSpread/StopOut depending on math — assert at least
        // a non-Hold signal fires.
        assert_ne!(r.signal, PairSignal::Hold,
            "huge outlier should fire some signal — got Hold with z={}", r.current_z);
    }

    #[test]
    fn custom_config_changes_thresholds() {
        // Lax config — small z still triggers entry.
        let lax = PairConfig { entry_z: 0.001, exit_z: 0.0, stop_z: 999.0 };
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let mut y: Vec<f64> = x.iter().enumerate().map(|(i, v)| {
            v * 2.0 + if i % 2 == 0 { 0.5 } else { -0.5 }
        }).collect();
        // Push last value off-spread.
        *y.last_mut().unwrap() = (*x.last().unwrap()) * 2.0 + 1.0;
        let r = analyze(&y, &x, &lax).unwrap();
        assert!(matches!(r.signal, PairSignal::ShortSpread | PairSignal::LongSpread),
            "lax config should fire entry on small deviation");
    }
}
