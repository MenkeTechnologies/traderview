//! Merton Jump-Diffusion Path Simulator — GBM augmented with a Poisson
//! jump process for fat-tailed dynamics.
//!
//!   dS/S = (μ - λκ) dt + σ dW + (J - 1) dN
//!
//! where:
//!   N_t ~ Poisson(λt)    (jump arrivals)
//!   ln(J) ~ N(μ_j, σ_j²) (log-jump-size distribution)
//!   κ = E[J - 1] = exp(μ_j + σ_j²/2) - 1   (compensator)
//!
//! Discretized per step dt:
//!   compensated drift: (μ - λκ - σ²/2) dt
//!   diffusion:         σ √dt · Z
//!   jump:              if Poisson(λ·dt) > 0, sum ln(J) samples
//!   S_{i+1} = S_i · exp(drift + diffusion + jump_log_total)
//!
//! Reproducible via seeded LCG. No external dependencies.
//!
//! Pure compute. Companion to `gbm_path_simulator`, `monte_carlo`,
//! `monte_carlo_var`.

#[derive(Debug)]
pub struct Report {
    pub mean_terminal: f64,
    pub stdev_terminal: f64,
    pub mean_log_return: f64,
    pub skew_log_return: f64,
    pub jump_count_total: u64,
    pub paths_run: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    s0: f64, mu: f64, sigma: f64,
    jump_lambda: f64, jump_mean: f64, jump_stdev: f64,
    dt: f64, steps: usize, paths: usize, seed: u64,
) -> Option<Report> {
    let finites = [s0, mu, sigma, jump_lambda, jump_mean, jump_stdev, dt];
    if finites.iter().any(|x| !x.is_finite()) { return None; }
    if s0 <= 0.0 || sigma < 0.0 || dt <= 0.0 { return None; }
    if jump_lambda < 0.0 || jump_stdev < 0.0 { return None; }
    if steps < 1 || paths < 1 { return None; }
    let kappa = (jump_mean + 0.5 * jump_stdev * jump_stdev).exp() - 1.0;
    let drift = (mu - jump_lambda * kappa - 0.5 * sigma * sigma) * dt;
    let diffusion_coef = sigma * dt.sqrt();
    let lambda_dt = jump_lambda * dt;
    let mut state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    let mut terminals = Vec::with_capacity(paths);
    let mut total_jumps: u64 = 0;
    for _ in 0..paths {
        let mut s = s0;
        for _ in 0..steps {
            let z = next_normal(&mut state);
            let k = next_poisson(&mut state, lambda_dt);
            total_jumps += k as u64;
            let mut log_jump = 0.0;
            for _ in 0..k {
                log_jump += jump_mean + jump_stdev * next_normal(&mut state);
            }
            s *= (drift + diffusion_coef * z + log_jump).exp();
        }
        terminals.push(s);
    }
    let n = terminals.len() as f64;
    let mean = terminals.iter().sum::<f64>() / n;
    let var = terminals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let stdev = var.max(0.0).sqrt();
    let log_returns: Vec<f64> = terminals.iter().map(|x| (x / s0).ln()).collect();
    let mean_lr = log_returns.iter().sum::<f64>() / n;
    let var_lr = log_returns.iter().map(|x| (x - mean_lr).powi(2)).sum::<f64>() / n;
    let std_lr = var_lr.max(0.0).sqrt();
    let skew_lr = if std_lr > 1e-12 {
        log_returns.iter().map(|x| ((x - mean_lr) / std_lr).powi(3)).sum::<f64>() / n
    } else { 0.0 };
    Some(Report {
        mean_terminal: mean,
        stdev_terminal: stdev,
        mean_log_return: mean_lr,
        skew_log_return: skew_lr,
        jump_count_total: total_jumps,
        paths_run: paths,
    })
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

/// Knuth's algorithm — exact for small lambda·dt (the trading case).
fn next_poisson(state: &mut u64, lam: f64) -> u32 {
    if lam <= 0.0 { return 0; }
    let limit = (-lam).exp();
    let mut p = 1.0;
    let mut k: u32 = 0;
    loop {
        k += 1;
        p *= next_uniform(state);
        if p <= limit { return k - 1; }
        if k > 1000 { return k; }     // safety cap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(-1.0, 0.05, 0.2, 1.0, 0.0, 0.1, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, -0.2, 1.0, 0.0, 0.1, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, -1.0, 0.0, 0.1, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 1.0, 0.0, -0.1, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 1.0, 0.0, 0.1, 0.0, 100, 50, 1).is_none());
        assert!(compute(f64::NAN, 0.05, 0.2, 1.0, 0.0, 0.1, 0.01, 100, 50, 1).is_none());
    }

    #[test]
    fn zero_lambda_matches_pure_gbm() {
        // No jumps → behaves like GBM. Mean ≈ S_0 · exp(μ·T).
        let r = compute(100.0, 0.05, 0.0, 0.0, 0.0, 0.0, 0.01, 100, 50, 42).unwrap();
        let expected = 100.0 * (0.05 * 1.0_f64).exp();
        assert!((r.mean_terminal - expected).abs() < 1e-6);
        assert_eq!(r.jump_count_total, 0);
    }

    #[test]
    fn negative_jump_mean_yields_negative_skew_in_log_returns() {
        // Downward jumps → fat left tail in log-returns → negative skew.
        // (Skew of terminal prices is dominated by lognormal right-skew
        // and can't be used directly.)
        let r = compute(
            100.0, 0.0, 0.10,
            5.0, -0.05, 0.05,
            0.01, 252, 5000, 7,
        ).unwrap();
        assert!(r.skew_log_return < 0.0);
    }

    #[test]
    fn jump_count_scales_with_lambda() {
        let low = compute(100.0, 0.0, 0.10, 0.5, 0.0, 0.05, 0.01, 100, 500, 1).unwrap();
        let high = compute(100.0, 0.0, 0.10, 5.0, 0.0, 0.05, 0.01, 100, 500, 1).unwrap();
        assert!(high.jump_count_total > low.jump_count_total);
    }

    #[test]
    fn reproducible_with_same_seed() {
        let a = compute(100.0, 0.05, 0.2, 1.0, 0.0, 0.05, 0.01, 100, 50, 99).unwrap();
        let b = compute(100.0, 0.05, 0.2, 1.0, 0.0, 0.05, 0.01, 100, 50, 99).unwrap();
        assert_eq!(a.mean_terminal.to_bits(), b.mean_terminal.to_bits());
        assert_eq!(a.jump_count_total, b.jump_count_total);
    }

    #[test]
    fn terminals_stay_positive() {
        let r = compute(100.0, 0.05, 0.2, 1.0, 0.0, 0.1, 0.01, 252, 200, 11).unwrap();
        // Lognormal jumps preserve positivity.
        assert!(r.mean_terminal > 0.0);
    }
}
