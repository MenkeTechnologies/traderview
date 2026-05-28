//! Jurik Moving Average (JMA) — Mark Jurik.
//!
//! Adaptive low-pass filter that aims to minimize lag while
//! suppressing high-frequency noise. Three parameters:
//!   - length: nominal period (larger = smoother + more lag)
//!   - phase : -100..+100, shapes overshoot vs lag tradeoff
//!   - power : 1.0 = standard, higher = smoother
//!
//! Reference implementation (public form, e.g. ProRealTime/TradingView
//! versions):
//!
//!   beta  = 0.45 · (length - 1) / (0.45 · (length - 1) + 2)
//!   alpha = beta^power
//!   phase_ratio = (phase / 100) + 1.5   if phase ≥ 0
//!                 (1.5)                  otherwise (using offset = 0)
//!     (a common simplification: phaseRatio in [0.5, 2.5])
//!   e0_t = (1 - alpha)·x_t + alpha·e0_{t-1}
//!   e1_t = (x_t - e0_t)·(1 - beta) + beta·e1_{t-1}
//!   e2_t = (e0_t + phaseRatio·e1_t - JMA_{t-1})·(1 - alpha)^2
//!          + alpha^2·e2_{t-1}
//!   JMA_t = JMA_{t-1} + e2_t
//!
//! Pure compute. Companion to `kama`, `vidya`, `tema`, `dema`.

pub fn compute(series: &[f64], length: usize, phase: f64, power: f64) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if n == 0 || length < 2 || !phase.is_finite()
        || !(-100.0..=100.0).contains(&phase)
        || !power.is_finite() || power <= 0.0 {
        return out;
    }
    if series.iter().any(|x| !x.is_finite()) { return out; }
    let l_f = length as f64;
    let beta = 0.45 * (l_f - 1.0) / (0.45 * (l_f - 1.0) + 2.0);
    let alpha = beta.powf(power);
    let phase_ratio = if phase >= 0.0 {
        (phase / 100.0) + 1.5
    } else {
        (phase / 100.0) + 0.5
    };
    let mut e0 = series[0];
    let mut e1 = 0.0_f64;
    let mut e2 = 0.0_f64;
    let mut jma = series[0];
    out[0] = Some(jma);
    let one_minus_alpha = 1.0 - alpha;
    let one_minus_alpha_sq = one_minus_alpha * one_minus_alpha;
    let alpha_sq = alpha * alpha;
    let one_minus_beta = 1.0 - beta;
    for (i, &x) in series.iter().enumerate().skip(1) {
        e0 = one_minus_alpha * x + alpha * e0;
        e1 = (x - e0) * one_minus_beta + beta * e1;
        e2 = (e0 + phase_ratio * e1 - jma) * one_minus_alpha_sq + alpha_sq * e2;
        jma += e2;
        out[i] = Some(jma);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_all_none() {
        let s = vec![100.0_f64; 50];
        assert!(compute(&s, 1, 0.0, 1.0).iter().all(|x| x.is_none()));
        assert!(compute(&s, 14, 150.0, 1.0).iter().all(|x| x.is_none()));
        assert!(compute(&s, 14, 0.0, 0.0).iter().all(|x| x.is_none()));
        assert!(compute(&s, 14, f64::NAN, 1.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        assert!(compute(&s, 14, 0.0, 1.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_constant_jma() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s, 14, 0.0, 1.0);
        for v in r.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn linear_trend_tracks_input() {
        // For a steady linear trend, JMA should converge to the input
        // value (some lag, but no permanent offset).
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 14, 0.0, 1.0);
        let last_jma = r[199].unwrap();
        // JMA should be near the current input (small lag).
        assert!((last_jma - 199.0).abs() < 10.0,
            "JMA {last_jma} should track input near 199");
    }

    #[test]
    fn jma_smoother_than_input_on_noise() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..400).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            100.0 + (r - 0.5) * 10.0
        }).collect();
        let r = compute(&s, 20, 0.0, 1.0);
        let vals: Vec<f64> = r.iter().flatten().copied().collect();
        let mean_in: f64 = s.iter().sum::<f64>() / s.len() as f64;
        let var_in: f64 = s.iter().map(|x| (x - mean_in).powi(2)).sum::<f64>() / s.len() as f64;
        let mean_jma: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        let var_jma: f64 = vals.iter().map(|x| (x - mean_jma).powi(2)).sum::<f64>() / vals.len() as f64;
        assert!(var_jma < var_in,
            "JMA variance {var_jma} should be lower than input {var_in}");
    }

    #[test]
    fn output_length_matches_input() {
        let s = vec![100.0_f64; 50];
        assert_eq!(compute(&s, 14, 0.0, 1.0).len(), 50);
    }
}
