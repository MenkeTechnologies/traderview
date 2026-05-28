//! Relative Volume Z-Score — standardized RVOL.
//!
//! `relative_volume` already gives the ratio of current volume vs the
//! N-bar past average. This module reports the z-score of that ratio
//! against its own rolling history, providing a statistically-normalized
//! "how unusual is current volume?" reading:
//!
//!   rvol_t = volume_t / SMA(volume, period, ending at t-1)
//!   z_t    = (rvol_t - SMA(rvol, zscore_period)) / stdev(rvol, zscore_period)
//!
//! Useful for cross-symbol comparison — an RVOL of 2.0 may be normal
//! for a slow stock and exceptional for a volatile one, but a z-score
//! of 3 is exceptional regardless of underlying volume profile.
//!
//! Pure compute. Default rvol_period = 20, zscore_period = 60.
//! Companion to `relative_volume`, `relative_volume_scanner`,
//! `volume_burst`.

pub fn compute(
    volumes: &[f64],
    rvol_period: usize,
    zscore_period: usize,
) -> Vec<Option<f64>> {
    let n = volumes.len();
    let mut out = vec![None; n];
    if rvol_period < 2 || zscore_period < 3
        || n < rvol_period + zscore_period { return out; }
    if volumes.iter().any(|v| !v.is_finite() || *v < 0.0) { return out; }
    let p_f = rvol_period as f64;
    let mut rvol = vec![None; n];
    let mut sum: f64 = volumes[..rvol_period].iter().sum();
    for (i, slot) in rvol.iter_mut().enumerate().skip(rvol_period) {
        let avg = sum / p_f;
        if avg > 0.0 {
            *slot = Some(volumes[i] / avg);
        }
        sum += volumes[i] - volumes[i - rvol_period];
    }
    let z_f = zscore_period as f64;
    for (i, slot) in out.iter_mut().enumerate()
        .skip(rvol_period + zscore_period - 1)
    {
        let win = &rvol[i + 1 - zscore_period..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let vals: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        let mean: f64 = vals.iter().sum::<f64>() / z_f;
        let var: f64 = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / z_f;
        let std = var.max(0.0).sqrt();
        if std > 0.0 {
            *slot = Some((rvol[i].unwrap() - mean) / std);
        } else {
            *slot = Some(0.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let v = vec![1000.0_f64; 100];
        assert!(compute(&v, 1, 60).iter().all(|x| x.is_none()));
        assert!(compute(&v, 20, 2).iter().all(|x| x.is_none()));
        assert!(compute(&v[..10], 20, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut v = vec![1000.0_f64; 100];
        v[5] = f64::NAN;
        assert!(compute(&v, 20, 60).iter().all(|x| x.is_none()));
        let mut v2 = vec![1000.0_f64; 100];
        v2[5] = -1.0;
        assert!(compute(&v2, 20, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_volume_yields_zero_zscore() {
        let v = vec![1000.0_f64; 200];
        let r = compute(&v, 20, 60);
        for x in r.iter().flatten() {
            assert!(x.abs() < 1e-9);
        }
    }

    #[test]
    fn volume_spike_yields_high_zscore() {
        let mut v = vec![1000.0_f64; 100];
        v.push(10000.0);
        let r = compute(&v, 20, 60);
        let last = v.len() - 1;
        assert!(r[last].is_some());
        assert!(r[last].unwrap() > 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let v = vec![1000.0_f64; 100];
        assert_eq!(compute(&v, 20, 60).len(), 100);
    }
}
