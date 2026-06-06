//! Rolling Z-Score Indicator.
//!
//! Standardized deviation of the close from its rolling mean:
//!
//!   z_t = (close_t - SMA(close, period)) / stdev(close, period)
//!
//! Where the stdev uses the population formula (divide by N).
//!
//! Interpretation: z > +2 = price 2σ above its mean (statistically
//! extreme high); z < -2 = extreme low; mean reversion practitioners
//! fade values at |z| > 2.
//!
//! Pure compute. Default period = 20. Distinct from `rolling_zscore`
//! which operates on arbitrary scalar series; this module wraps the
//! same machinery with the conventional close-price API and adds
//! finite-input guards.
//!
//! Companion to `bollinger_percent_b`, `disparity_index`,
//! `standard_error_bands`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &closes[i + 1 - period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let std = var.max(0.0).sqrt();
        if std > 0.0 {
            *slot = Some((closes[i] - mean) / std);
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
        assert!(compute(&c[..5], 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        assert!(compute(&c, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_z() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn spike_above_mean_yields_positive_z() {
        let mut c = vec![100.0_f64; 19];
        c.push(120.0);
        let r = compute(&c, 20);
        let v = r[19].unwrap();
        assert!(v > 0.0);
    }

    #[test]
    fn spike_below_mean_yields_negative_z() {
        let mut c = vec![100.0_f64; 19];
        c.push(80.0);
        let r = compute(&c, 20);
        assert!(r[19].unwrap() < 0.0);
    }

    #[test]
    fn symmetric_spike_yields_symmetric_z() {
        let mut up = vec![100.0_f64; 19];
        up.push(110.0);
        let mut dn = vec![100.0_f64; 19];
        dn.push(90.0);
        let r_up = compute(&up, 20);
        let r_dn = compute(&dn, 20);
        assert!((r_up[19].unwrap() + r_dn[19].unwrap()).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 20).len(), 50);
    }
}
