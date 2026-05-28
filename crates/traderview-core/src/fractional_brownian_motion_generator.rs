//! Fractional Brownian Motion Generator — long-memory / anti-persistent
//! path simulator via the midpoint-displacement method (Mandelbrot–Voss).
//!
//! Generates `2^levels + 1` evenly-spaced samples of fBm_H(t) on [0, 1]
//! with Hurst exponent H ∈ (0, 1):
//!   H = 0.5   → standard Brownian motion (uncorrelated increments)
//!   H > 0.5   → persistent (trending) — increments positively correlated
//!   H < 0.5   → anti-persistent (mean-reverting)
//!
//! Algorithm (recursive subdivision):
//!   1. Initialize endpoints x(0) = 0, x(1) = N(0, σ²).
//!   2. At level k, for each sub-interval midpoint, set:
//!      `midpoint = average(endpoints) + N(0, σ_k²)` where
//!      `σ_k² = σ² · (1/2)^(2H·k) · (1 - 2^(2H-2))`.
//!   3. Repeat until level == levels.
//!
//! Returns the sample vector. The scaling factor for the variance update
//! produces approximately fBm with the requested Hurst exponent.
//!
//! Pure compute. Companion to `hurst_exponent`, `gbm_path_simulator`,
//! `rescaled_range_analysis`.

pub fn compute(hurst: f64, sigma0: f64, levels: u32, seed: u64) -> Option<Vec<f64>> {
    if !hurst.is_finite() || !sigma0.is_finite() { return None; }
    if !(0.01..=0.99).contains(&hurst) || sigma0 <= 0.0 { return None; }
    if levels == 0 || levels > 18 { return None; }
    let n = (1_usize << levels) + 1;
    let mut state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    let mut x = vec![0.0_f64; n];
    x[n - 1] = sigma0 * next_normal(&mut state);
    let two_h = 2.0 * hurst;
    let factor = 1.0 - (two_h - 2.0).exp2();
    let mut sigma_k = sigma0 * factor.max(0.0).sqrt();
    let mut step = n - 1;
    for _ in 0..levels {
        let half = step / 2;
        if half == 0 { break; }
        let mut i = half;
        while i < n - 1 {
            let avg = 0.5 * (x[i - half] + x[i + half]);
            x[i] = avg + sigma_k * next_normal(&mut state);
            i += step;
        }
        step = half;
        sigma_k *= (-hurst).exp2();        // multiply by 2^(-H)
    }
    Some(x)
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state
}

fn next_uniform(state: &mut u64) -> f64 {
    let u = next_u64(state);
    let mantissa = u >> 11;
    (mantissa as f64) * (1.0_f64 / (1u64 << 53) as f64)
}

fn next_normal(state: &mut u64) -> f64 {
    let mut u1 = next_uniform(state);
    let u2 = next_uniform(state);
    if u1 < 1e-300 { u1 = 1e-300; }
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(0.0, 1.0, 8, 1).is_none());
        assert!(compute(1.5, 1.0, 8, 1).is_none());
        assert!(compute(0.5, 0.0, 8, 1).is_none());
        assert!(compute(0.5, 1.0, 0, 1).is_none());
        assert!(compute(0.5, 1.0, 30, 1).is_none());
        assert!(compute(f64::NAN, 1.0, 8, 1).is_none());
    }

    #[test]
    fn path_length_matches_two_pow_levels_plus_one() {
        let p = compute(0.5, 1.0, 8, 42).unwrap();
        assert_eq!(p.len(), 257);
        let p2 = compute(0.7, 1.0, 5, 1).unwrap();
        assert_eq!(p2.len(), 33);
    }

    #[test]
    fn path_starts_at_zero() {
        let p = compute(0.5, 1.0, 8, 42).unwrap();
        assert_eq!(p[0], 0.0);
    }

    #[test]
    fn reproducible_with_same_seed() {
        let a = compute(0.5, 1.0, 8, 99).unwrap();
        let b = compute(0.5, 1.0, 8, 99).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn higher_hurst_yields_smoother_paths() {
        // Persistent (H=0.9) paths should have lower variance of
        // first differences than anti-persistent (H=0.1) paths of same
        // length and σ₀, on average.
        let p_smooth = compute(0.9, 1.0, 10, 7).unwrap();
        let p_rough = compute(0.1, 1.0, 10, 7).unwrap();
        let var_diff = |p: &[f64]| {
            let d: Vec<f64> = p.windows(2).map(|w| w[1] - w[0]).collect();
            let m = d.iter().sum::<f64>() / d.len() as f64;
            d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / d.len() as f64
        };
        assert!(var_diff(&p_smooth) < var_diff(&p_rough));
    }

    #[test]
    fn all_values_finite() {
        let p = compute(0.7, 1.0, 10, 13).unwrap();
        assert!(p.iter().all(|x| x.is_finite()));
    }
}
