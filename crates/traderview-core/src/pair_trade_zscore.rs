//! Pair-trade z-score signals — generate entry / exit signals on the
//! standardized spread between two cointegrated symbols.
//!
//! Spread: `s_t = y_t − β·x_t − α`
//!
//! z-score: `z_t = (s_t − rolling_mean(s, window)) / rolling_stdev(s, window)`
//!
//! Signals:
//!   - **EnterLong**:  z ≤ −entry_z (spread cheap → buy y, sell x)
//!   - **EnterShort**: z ≥ +entry_z (spread rich → sell y, buy x)
//!   - **ExitFlat**:   |z| ≤ exit_z (spread reverted)
//!   - **StopLoss**:   |z| ≥ stop_z (spread blew through stop → bail)
//!
//! Pure compute. Companion to `cointegration` (which gives the hedge ratio
//! and ADF significance) and `ornstein_uhlenbeck` (which gives the
//! half-life for sizing the entry threshold).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Signal {
    EnterLong,
    EnterShort,
    ExitFlat,
    StopLoss,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub window: usize,
    pub entry_z: f64,
    pub exit_z: f64,
    pub stop_z: f64,
}

impl Default for Config {
    fn default() -> Self { Self { window: 30, entry_z: 2.0, exit_z: 0.5, stop_z: 4.0 } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PairReport {
    pub spread: Vec<f64>,
    pub z_score: Vec<Option<f64>>,
    pub signals: Vec<Option<Signal>>,
}

pub fn compute(
    y: &[f64],
    x: &[f64],
    hedge_ratio: f64,
    intercept: f64,
    cfg: &Config,
) -> Option<PairReport> {
    let n = y.len();
    if x.len() != n
        || n < cfg.window
        || cfg.window < 2
        || !cfg.entry_z.is_finite() || cfg.entry_z <= 0.0
        || !cfg.exit_z.is_finite() || cfg.exit_z < 0.0
        || !cfg.stop_z.is_finite() || cfg.stop_z <= cfg.entry_z
        || !hedge_ratio.is_finite()
        || !intercept.is_finite()
    {
        return None;
    }
    let mut spread = vec![0.0_f64; n];
    let mut have_spread = vec![false; n];
    for i in 0..n {
        if y[i].is_finite() && x[i].is_finite() {
            spread[i] = y[i] - hedge_ratio * x[i] - intercept;
            have_spread[i] = true;
        }
    }
    let mut z = vec![None::<f64>; n];
    let mut signals = vec![None::<Signal>; n];
    for i in (cfg.window - 1)..n {
        if !have_spread[i] { continue; }
        let lo = i + 1 - cfg.window;
        let win = &spread[lo..=i];
        let valid: Vec<f64> = win.iter().zip(have_spread[lo..=i].iter())
            .filter_map(|(s, h)| if *h { Some(*s) } else { None })
            .collect();
        if valid.len() < 2 { continue; }
        let m = valid.iter().sum::<f64>() / valid.len() as f64;
        let var = valid.iter().map(|s| (s - m).powi(2)).sum::<f64>()
            / (valid.len() as f64 - 1.0);
        let sd = var.max(0.0).sqrt();
        if sd <= 0.0 { continue; }
        let z_i = (spread[i] - m) / sd;
        if !z_i.is_finite() { continue; }
        z[i] = Some(z_i);
        let abs_z = z_i.abs();
        signals[i] = Some(if abs_z >= cfg.stop_z {
            Signal::StopLoss
        } else if z_i <= -cfg.entry_z {
            Signal::EnterLong
        } else if z_i >= cfg.entry_z {
            Signal::EnterShort
        } else if abs_z <= cfg.exit_z {
            Signal::ExitFlat
        } else {
            Signal::Hold
        });
    }
    Some(PairReport { spread, z_score: z, signals })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        let y = vec![1.0; 50];
        let x = vec![1.0; 25];
        assert!(compute(&y, &x, 1.0, 0.0, &Config::default()).is_none());
    }

    #[test]
    fn invalid_config_returns_none() {
        let y = vec![1.0; 50];
        let x = vec![1.0; 50];
        assert!(compute(&y, &x, 1.0, 0.0, &Config { window: 0, ..Default::default() }).is_none());
        assert!(compute(&y, &x, 1.0, 0.0, &Config { entry_z: 0.0, ..Default::default() }).is_none());
        assert!(compute(&y, &x, 1.0, 0.0, &Config { stop_z: 1.0, entry_z: 2.0, ..Default::default() }).is_none());
        assert!(compute(&y, &x, f64::NAN, 0.0, &Config::default()).is_none());
    }

    #[test]
    fn flat_spread_yields_no_signals() {
        // y - x = 0 everywhere → spread constant → stdev = 0 → no z score → no signals.
        let n = 100;
        let y = vec![10.0_f64; n];
        let x = vec![10.0_f64; n];
        let r = compute(&y, &x, 1.0, 0.0, &Config::default()).unwrap();
        assert!(r.z_score.iter().all(|z| z.is_none()));
    }

    #[test]
    fn enter_long_when_spread_drops_below_entry() {
        // Build a spread that's stable then drops by 3σ on last bar.
        let n = 100;
        let mut y = vec![10.0_f64; n];
        let x = vec![10.0_f64; n];
        let mut state: u64 = 42;
        for slot in y.iter_mut().take(n - 1) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            *slot = 10.0 + noise;
        }
        y[n - 1] = 10.0 - 5.0;    // big drop → spread = y - x = -5
        let r = compute(&y, &x, 1.0, 0.0, &Config::default()).unwrap();
        assert!(matches!(r.signals[n - 1], Some(Signal::EnterLong) | Some(Signal::StopLoss)));
    }

    #[test]
    fn enter_short_when_spread_jumps_above_entry() {
        let n = 100;
        let mut y = vec![10.0_f64; n];
        let x = vec![10.0_f64; n];
        let mut state: u64 = 99;
        for slot in y.iter_mut().take(n - 1) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            *slot = 10.0 + noise;
        }
        y[n - 1] = 10.0 + 5.0;
        let r = compute(&y, &x, 1.0, 0.0, &Config::default()).unwrap();
        assert!(matches!(r.signals[n - 1], Some(Signal::EnterShort) | Some(Signal::StopLoss)));
    }

    #[test]
    fn signal_at_mean_is_exit_flat() {
        let n = 100;
        let mut y = vec![10.0_f64; n];
        let x = vec![10.0_f64; n];
        let mut state: u64 = 7;
        for slot in y.iter_mut().take(n) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 1.0;
            *slot = 10.0 + noise;
        }
        let r = compute(&y, &x, 1.0, 0.0, &Config::default()).unwrap();
        // Find a bar with |z| ≤ exit_z and assert ExitFlat.
        for (i, z) in r.z_score.iter().enumerate() {
            if let Some(v) = z {
                if v.abs() < 0.3 {
                    assert_eq!(r.signals[i], Some(Signal::ExitFlat));
                    return;
                }
            }
        }
    }

    #[test]
    fn nan_inputs_skipped_safely() {
        let mut y = vec![10.0; 50];
        let x = vec![10.0; 50];
        y[10] = f64::NAN;
        let r = compute(&y, &x, 1.0, 0.0, &Config::default()).unwrap();
        assert!(r.z_score[10].is_none());
    }
}
