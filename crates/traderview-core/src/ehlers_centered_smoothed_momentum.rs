//! Centered Smoothed Momentum (CSM) — John Ehlers.
//!
//! Momentum (difference between price and the price `momentum_period`
//! bars ago) passed through a SuperSmoother filter to remove jitter:
//!
//!   mom_t = close_t - close_{t-momentum_period}
//!   csm_t = SuperSmoother(mom, smooth_period)
//!
//! Centered at zero. Positive = up-momentum, negative = down-momentum,
//! zero crosses signal trend turns. Less noisy than raw momentum,
//! more responsive than EMA-smoothed momentum.
//!
//! Pure compute. Defaults: momentum_period = 10, smooth_period = 8.
//! Companion to `ehlers_super_smoother`, `roc`, `tsi`, `coppock_curve`.

pub fn compute(closes: &[f64], momentum_period: usize, smooth_period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if momentum_period < 1 || smooth_period < 4 || n < momentum_period + 3 {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let mut mom = vec![0.0_f64; n];
    for i in momentum_period..n {
        mom[i] = closes[i] - closes[i - momentum_period];
    }
    let pi = std::f64::consts::PI;
    let a1 = (-1.414 * pi / smooth_period as f64).exp();
    let b1 = 2.0 * a1 * (1.414 * pi / smooth_period as f64).cos();
    let c2 = b1;
    let c3 = -a1 * a1;
    let c1 = 1.0 - c2 - c3;
    let mut ss = vec![0.0_f64; n];
    for i in (momentum_period + 2)..n {
        ss[i] = c1 * (mom[i] + mom[i - 1]) / 2.0 + c2 * ss[i - 1] + c3 * ss[i - 2];
        out[i] = Some(ss[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 0, 8).iter().all(|x| x.is_none()));
        assert!(compute(&c, 10, 3).iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 10, 8).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        assert!(compute(&c, 10, 8).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_csm() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 10, 8);
        for v in r.iter().skip(30).flatten() {
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn uptrend_yields_positive_csm() {
        let c: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 10, 8);
        // Steady-state mom = period (constant 10) → CSM converges to 10.
        let last = r[99].unwrap();
        assert!(
            last > 8.0 && last < 12.0,
            "uptrend CSM should be near +10, got {last}"
        );
    }

    #[test]
    fn downtrend_yields_negative_csm() {
        let c: Vec<f64> = (0..100).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 10, 8);
        let last = r[99].unwrap();
        assert!(last < -8.0 && last > -12.0);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 10, 8).len(), 50);
    }
}
