//! Cumulative Volume Delta Z-Score — normalized CVD divergence indicator.
//!
//! `cumulative_delta` (existing) gives the raw cumulative signed-volume
//! flow. This module standardizes the CVD reading against its rolling
//! N-bar mean and stdev:
//!
//!   z_t = (cvd_t - SMA(cvd, period)) / stdev(cvd, period)
//!
//! Output ≈ 0 in steady accumulation/distribution; |z| > 2 marks
//! statistically significant flow divergences worth attention.
//!
//! Pure compute. Default period = 60 (e.g. 60 1-min bars = 1 hour).
//! Companion to `cumulative_delta`, `order_flow`, `bid_ask_volume_ratio`,
//! `weiss_wave`.

pub fn compute(cvd: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = cvd.len();
    let mut out = vec![None; n];
    if period < 2 || n < period { return out; }
    if cvd.iter().any(|x| !x.is_finite()) { return out; }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &cvd[i + 1 - period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let std = var.max(0.0).sqrt();
        if std > 0.0 {
            *slot = Some((cvd[i] - mean) / std);
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
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 1).iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        assert!(compute(&c, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_cvd_yields_zero_z() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 60);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn spike_above_mean_yields_positive_z() {
        let mut c = vec![100.0_f64; 59];
        c.push(200.0);
        let r = compute(&c, 60);
        let v = r[59].unwrap();
        assert!(v > 0.0);
    }

    #[test]
    fn spike_below_mean_yields_negative_z() {
        let mut c = vec![100.0_f64; 59];
        c.push(20.0);
        let r = compute(&c, 60);
        let v = r[59].unwrap();
        assert!(v < 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 100];
        assert_eq!(compute(&c, 60).len(), 100);
    }
}
