//! Roofing Filter — John Ehlers (Cybernetic Analysis for Stocks & Futures).
//!
//! High-pass + low-pass stage cascade designed to isolate the
//! "trading cycle" band (typically 10..48 bars) while removing both
//! short-cycle noise (< hp_period bars) and long-term drift
//! (> hp_period bars).
//!
//! Stage 1: 2-pole high-pass to remove cycles longer than hp_period.
//!   α₁ = (cos(0.707·2π/hp) + sin(0.707·2π/hp) - 1) / cos(0.707·2π/hp)
//!   HP_t = (1 - α₁/2)² · (x_t - 2·x_{t-1} + x_{t-2})
//!          + 2·(1 - α₁)·HP_{t-1}
//!          - (1 - α₁)²·HP_{t-2}
//!
//! Stage 2: Super Smoother (2-pole IIR Butterworth) to remove cycles
//! shorter than ss_period.
//!   a₁ = exp(-1.414·π / ss_period)
//!   b₁ = 2·a₁·cos(1.414·180° / ss_period)
//!   c₂ = b₁
//!   c₃ = -a₁·a₁
//!   c₁ = 1 - c₂ - c₃
//!   SS_t = c₁·(HP_t + HP_{t-1})/2 + c₂·SS_{t-1} + c₃·SS_{t-2}
//!
//! Pure compute. Companion to `ehlers_super_smoother`, `ehlers_decycler`,
//! `hilbert_transform`.

pub fn compute(series: &[f64], hp_period: usize, ss_period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if n < 3 || hp_period < 4 || ss_period < 4 {
        return out;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let pi = std::f64::consts::PI;
    let two_pi = 2.0 * pi;
    // High-pass coefficients.
    let arg = 0.707 * two_pi / hp_period as f64;
    let cos_arg = arg.cos();
    let alpha1 = (cos_arg + arg.sin() - 1.0) / cos_arg;
    let one_minus_a1 = 1.0 - alpha1;
    let coef1 = (1.0 - alpha1 / 2.0).powi(2);
    let coef2 = 2.0 * one_minus_a1;
    let coef3 = one_minus_a1 * one_minus_a1;
    // Super Smoother coefficients.
    let a1 = (-1.414 * pi / ss_period as f64).exp();
    let b1 = 2.0 * a1 * (1.414 * pi / ss_period as f64).cos();
    let c2 = b1;
    let c3 = -a1 * a1;
    let c1 = 1.0 - c2 - c3;
    let mut hp = vec![0.0_f64; n];
    let mut ss = vec![0.0_f64; n];
    for i in 2..n {
        hp[i] = coef1 * (series[i] - 2.0 * series[i - 1] + series[i - 2]) + coef2 * hp[i - 1]
            - coef3 * hp[i - 2];
        ss[i] = c1 * (hp[i] + hp[i - 1]) / 2.0 + c2 * ss[i - 1] + c3 * ss[i - 2];
        out[i] = Some(ss[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_all_none() {
        let s = vec![100.0_f64; 100];
        assert!(compute(&s, 3, 10).iter().all(|x| x.is_none()));
        assert!(compute(&s, 48, 3).iter().all(|x| x.is_none()));
        assert!(compute(&s[..2], 48, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut s = vec![100.0_f64; 100];
        s[5] = f64::NAN;
        assert!(compute(&s, 48, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_signal_settles_to_zero() {
        let s = vec![100.0_f64; 200];
        let r = compute(&s, 48, 10);
        // HP of a constant is zero (after transient).
        for v in r.iter().skip(50).flatten() {
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn output_centered_for_linear_trend() {
        // HP removes the linear drift → output stays near zero.
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 48, 10);
        let vals: Vec<f64> = r.iter().skip(50).flatten().copied().collect();
        let mean: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        // Constant-slope ramp: HP should remove the trend → mean ≈ 0.
        assert!(
            mean.abs() < 1.0,
            "ramp filtered out; mean output should be near 0, got {mean}"
        );
    }

    #[test]
    fn output_length_matches_input() {
        let s = vec![100.0_f64; 100];
        assert_eq!(compute(&s, 48, 10).len(), 100);
    }

    #[test]
    fn first_two_bars_are_none() {
        let s = vec![100.0_f64; 100];
        let r = compute(&s, 48, 10);
        assert!(r[0].is_none());
        assert!(r[1].is_none());
        assert!(r[2].is_some());
    }
}
