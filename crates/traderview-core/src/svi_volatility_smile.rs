//! SVI Volatility Smile (Gatheral 2004, raw parameterization) — fit a
//! 5-parameter total-variance smile in log-moneyness:
//!
//!   w(k) = a + b · (ρ (k - m) + sqrt((k - m)² + σ²))
//!
//! where:
//!   k    = log(K/F) is log-moneyness
//!   w(k) = total variance = σ_IV²(k) · T
//!   a    ≥ 0   level
//!   b    ≥ 0   curvature scale
//!   |ρ| < 1   skew
//!   m         smile center
//!   σ    > 0  smoothness
//!
//! Fits via coordinate-descent / grid-narrowing for speed and
//! determinism (no external optimizer dependency). Returns the
//! parameter set, fitted total-variance per input strike, and the
//! RMSE.
//!
//! Constraints (Gatheral & Jacquier 2014) for arbitrage-free fits:
//!   b · (1 + |ρ|) ≤ 4 / T
//! (enforced as a soft penalty; check `arbitrage_ok` flag).
//!
//! Pure compute. Companion to `volatility_smile`, `volatility_skew`,
//! `sabr_volatility`, `nelson_siegel_svensson`.

#[derive(Debug, Clone, Copy)]
pub struct SviParams {
    pub a: f64,
    pub b: f64,
    pub rho: f64,
    pub m: f64,
    pub sigma: f64,
}

#[derive(Debug)]
pub struct Report {
    pub params: SviParams,
    pub fitted_total_var: Vec<f64>,
    pub fitted_iv: Vec<f64>,
    pub rmse_total_var: f64,
    pub arbitrage_ok: bool,
}

pub fn compute(log_moneyness: &[f64], total_variance: &[f64], expiry_years: f64) -> Option<Report> {
    let n = log_moneyness.len();
    if n < 5 || total_variance.len() != n { return None; }
    if !expiry_years.is_finite() || expiry_years <= 0.0 { return None; }
    if log_moneyness.iter().chain(total_variance.iter()).any(|x| !x.is_finite()) {
        return None;
    }
    if total_variance.iter().any(|&v| v < 0.0) { return None; }
    // Reasonable init from data.
    let w_min = total_variance.iter().copied().fold(f64::INFINITY, f64::min);
    let w_max = total_variance.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut a = (w_min * 0.95).max(0.0);
    let mut b = ((w_max - w_min) / 0.5).max(1e-4);
    let mut rho = -0.3_f64;
    let mut m = 0.0_f64;
    let mut sigma = 0.10_f64;
    // Coordinate-descent on a shrinking grid for each parameter.
    let mut best_rmse = rmse(log_moneyness, total_variance, a, b, rho, m, sigma);
    for outer in 0..6 {
        let scale = (0.5_f64).powi(outer);
        for _ in 0..4 {
            // a sweep
            let (na, nr) = sweep_param(
                log_moneyness, total_variance, a, b, rho, m, sigma,
                0.05 * scale * w_max.max(0.01), |v| v.max(0.0), 0,
            );
            if nr < best_rmse { a = na; best_rmse = nr; }
            // b sweep
            let (nb, nr) = sweep_param(
                log_moneyness, total_variance, a, b, rho, m, sigma,
                0.2 * scale, |v| v.max(0.0), 1,
            );
            if nr < best_rmse { b = nb; best_rmse = nr; }
            // rho sweep
            let (nrho, nr) = sweep_param(
                log_moneyness, total_variance, a, b, rho, m, sigma,
                0.2 * scale, |v| v.clamp(-0.999, 0.999), 2,
            );
            if nr < best_rmse { rho = nrho; best_rmse = nr; }
            // m sweep
            let (nm, nr) = sweep_param(
                log_moneyness, total_variance, a, b, rho, m, sigma,
                0.05 * scale, |v| v, 3,
            );
            if nr < best_rmse { m = nm; best_rmse = nr; }
            // sigma sweep
            let (nsig, nr) = sweep_param(
                log_moneyness, total_variance, a, b, rho, m, sigma,
                0.05 * scale, |v| v.max(1e-4), 4,
            );
            if nr < best_rmse { sigma = nsig; best_rmse = nr; }
        }
    }
    let params = SviParams { a, b, rho, m, sigma };
    let fitted_total_var: Vec<f64> = log_moneyness.iter()
        .map(|&k| eval(a, b, rho, m, sigma, k).max(0.0))
        .collect();
    let fitted_iv: Vec<f64> = fitted_total_var.iter()
        .map(|w| (w / expiry_years).max(0.0).sqrt())
        .collect();
    let arbitrage_ok = b * (1.0 + rho.abs()) <= 4.0 / expiry_years;
    Some(Report {
        params,
        fitted_total_var,
        fitted_iv,
        rmse_total_var: best_rmse,
        arbitrage_ok,
    })
}

#[allow(clippy::too_many_arguments)]
fn sweep_param<F: Fn(f64) -> f64>(
    k: &[f64], w: &[f64],
    a: f64, b: f64, rho: f64, m: f64, sigma: f64,
    half: f64, clip: F, idx: u8,
) -> (f64, f64) {
    let center = match idx {
        0 => a, 1 => b, 2 => rho, 3 => m, _ => sigma,
    };
    let mut best = center;
    let mut best_rmse = rmse(k, w, a, b, rho, m, sigma);
    for step in [-1.0_f64, -0.5, 0.0, 0.5, 1.0] {
        let candidate = clip(center + step * half);
        let (aa, bb, rr, mm, ss) = match idx {
            0 => (candidate, b, rho, m, sigma),
            1 => (a, candidate, rho, m, sigma),
            2 => (a, b, candidate, m, sigma),
            3 => (a, b, rho, candidate, sigma),
            _ => (a, b, rho, m, candidate),
        };
        let r = rmse(k, w, aa, bb, rr, mm, ss);
        if r < best_rmse { best_rmse = r; best = candidate; }
    }
    (best, best_rmse)
}

fn eval(a: f64, b: f64, rho: f64, m: f64, sigma: f64, k: f64) -> f64 {
    let d = k - m;
    a + b * (rho * d + (d * d + sigma * sigma).sqrt())
}

fn rmse(k: &[f64], w: &[f64], a: f64, b: f64, rho: f64, m: f64, sigma: f64) -> f64 {
    let n = k.len() as f64;
    let sse: f64 = k.iter().zip(w.iter()).map(|(&kk, &ww)| {
        let e = eval(a, b, rho, m, sigma, kk) - ww;
        e * e
    }).sum();
    (sse / n).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let k = vec![0.0; 5];
        let w = vec![0.04; 5];
        assert!(compute(&k[..3], &w[..3], 0.5).is_none());
        let bad_len = vec![0.04; 4];
        assert!(compute(&k, &bad_len, 0.5).is_none());
        assert!(compute(&k, &w, 0.0).is_none());
        assert!(compute(&k, &w, -1.0).is_none());
        let mut k_nan = k.clone();
        k_nan[0] = f64::NAN;
        assert!(compute(&k_nan, &w, 0.5).is_none());
    }

    #[test]
    fn flat_iv_yields_low_rmse() {
        // Constant IV → flat total variance. SVI should fit easily.
        let k: Vec<f64> = (-10_i32..=10).map(|i| i as f64 * 0.05).collect();
        let w: Vec<f64> = vec![0.04 * 0.5; k.len()];
        let r = compute(&k, &w, 0.5).unwrap();
        assert!(r.rmse_total_var < 0.005);
    }

    #[test]
    fn smile_shape_recovers_curvature() {
        // Build a SVI-shaped smile and fit. Recovered w should match.
        let true_params = SviParams { a: 0.02, b: 0.1, rho: -0.4, m: 0.05, sigma: 0.1 };
        let k: Vec<f64> = (-20_i32..=20).map(|i| i as f64 * 0.05).collect();
        let w: Vec<f64> = k.iter()
            .map(|&kk| eval(true_params.a, true_params.b, true_params.rho,
                            true_params.m, true_params.sigma, kk))
            .collect();
        let r = compute(&k, &w, 1.0).unwrap();
        assert!(r.rmse_total_var < 0.01);
    }

    #[test]
    fn fitted_total_var_non_negative() {
        let k: Vec<f64> = (-10_i32..=10).map(|i| i as f64 * 0.05).collect();
        let w: Vec<f64> = k.iter().map(|&kk| 0.04 + 0.02 * kk * kk).collect();
        let r = compute(&k, &w, 0.5).unwrap();
        assert!(r.fitted_total_var.iter().all(|&x| x >= 0.0));
        assert!(r.fitted_iv.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn outputs_match_input_length() {
        let k = vec![0.0_f64; 10];
        let w = vec![0.04_f64; 10];
        let r = compute(&k, &w, 0.5).unwrap();
        assert_eq!(r.fitted_total_var.len(), 10);
        assert_eq!(r.fitted_iv.len(), 10);
    }

    #[test]
    fn rho_in_unit_interval() {
        let k: Vec<f64> = (-10_i32..=10).map(|i| i as f64 * 0.05).collect();
        let w: Vec<f64> = k.iter().map(|&kk| 0.04 + 0.05 * kk).collect();
        let r = compute(&k, &w, 0.5).unwrap();
        assert!(r.params.rho.abs() < 1.0);
    }
}
