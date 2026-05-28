//! Momentum Quality Score — composite of momentum z-score × volume
//! z-score.
//!
//! Combines two normalized signals into one trade-quality reading:
//!
//!   ret_t       = ln(close_t / close_{t-period})
//!   mom_z_t     = (ret_t - SMA(ret, z_period)) / stdev(ret, z_period)
//!   vol_z_t     = (volume_t - SMA(volume, z_period)) / stdev(volume, z_period)
//!   score_t     = mom_z_t · vol_z_t
//!
//! Positive readings: momentum AND volume both above norm (or both
//! below — confirmed trend in either direction). Near zero: mixed
//! signal. Strong positive scores filter out low-volume momentum
//! moves that often fail.
//!
//! Pure compute. Defaults: period = 14, z_period = 60.
//! Companion to `velocity_indicator`, `relative_volume_zscore`,
//! `volume_burst`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub close: f64, pub volume: f64 }

pub fn compute(
    bars: &[Bar],
    period: usize,
    z_period: usize,
) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || z_period < 3 || n < period + z_period { return out; }
    if bars.iter().any(|b| !b.close.is_finite() || !b.volume.is_finite()
        || b.close <= 0.0 || b.volume < 0.0) { return out; }
    let mut ret = vec![None; n];
    for i in period..n {
        let prev = bars[i - period].close;
        if prev > 0.0 && bars[i].close > 0.0 {
            ret[i] = Some((bars[i].close / prev).ln());
        }
    }
    let zp = z_period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period + z_period - 1) {
        let ret_win = &ret[i + 1 - z_period..=i];
        if ret_win.iter().any(|x| x.is_none()) { continue; }
        let ret_vals: Vec<f64> = ret_win.iter().filter_map(|x| *x).collect();
        let ret_mean: f64 = ret_vals.iter().sum::<f64>() / zp;
        let ret_var: f64 = ret_vals.iter().map(|x| (x - ret_mean).powi(2)).sum::<f64>() / zp;
        let ret_std = ret_var.max(0.0).sqrt();
        let vol_win = &bars[i + 1 - z_period..=i];
        let vol_mean: f64 = vol_win.iter().map(|b| b.volume).sum::<f64>() / zp;
        let vol_var: f64 = vol_win.iter().map(|b| (b.volume - vol_mean).powi(2)).sum::<f64>() / zp;
        let vol_std = vol_var.max(0.0).sqrt();
        if ret_std > 0.0 && vol_std > 0.0 {
            let mom_z = (ret[i].unwrap() - ret_mean) / ret_std;
            let vol_z = (bars[i].volume - vol_mean) / vol_std;
            *slot = Some(mom_z * vol_z);
        } else {
            *slot = Some(0.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> Bar { Bar { close: c, volume: v } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100.0, 1000.0); 100];
        assert!(compute(&bars, 1, 60).iter().all(|x| x.is_none()));
        assert!(compute(&bars, 14, 2).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..10], 14, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(100.0, 1000.0); 100];
        bars[5] = b(f64::NAN, 1000.0);
        assert!(compute(&bars, 14, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_score() {
        let bars = vec![b(100.0, 1000.0); 100];
        let r = compute(&bars, 14, 60);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn high_momentum_high_volume_yields_high_positive_score() {
        // Flat 70 bars, then explosive close + volume spike.
        let mut bars = vec![b(100.0, 1000.0); 80];
        bars.push(b(120.0, 10000.0));
        let r = compute(&bars, 14, 60);
        let last = bars.len() - 1;
        assert!(r[last].is_some());
        assert!(r[last].unwrap() > 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 1000.0); 100];
        assert_eq!(compute(&bars, 14, 60).len(), 100);
    }
}
