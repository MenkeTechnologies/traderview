//! Kou (2002) Double-Exponential Jump-Diffusion — GBM with asymmetric
//! Laplace-distributed log-jumps. Captures fat tails and skew that
//! Merton's symmetric normal jumps miss.
//!
//!   dS/S = (μ - λκ) dt + σ dW + (J - 1) dN
//!
//! where:
//!   N_t ~ Poisson(λt)
//!   ln(J) ~ asymmetric Laplace:
//!     up   with prob p,   exponential(η1), mean 1/η1   (positive tail)
//!     down with prob 1-p, -exponential(η2), mean -1/η2 (negative tail)
//!   E`[J]` = p · η1/(η1-1) + (1-p) · η2/(η2+1)
//!   κ = E[J - 1]
//!
//! Constraint: η1 > 1 (so the up-jump mean is finite). Typical defaults
//! match Kou's calibration to S&P 500: p ≈ 0.4, η1 ≈ 10, η2 ≈ 5,
//! λ ≈ 1 per year.
//!
//! Reports terminal-price stats including separately observable
//! up/down jump counts so callers can sanity-check the asymmetry.
//!
//! Pure compute. Companion to `jump_diffusion_simulator` (Merton),
//! `gbm_path_simulator`.

#[derive(Debug)]
pub struct Report {
    pub mean_terminal: f64,
    pub stdev_terminal: f64,
    pub mean_log_return: f64,
    pub skew_log_return: f64,
    pub up_jumps: u64,
    pub down_jumps: u64,
    pub paths_run: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    s0: f64, mu: f64, sigma: f64,
    jump_lambda: f64, up_prob: f64, eta_up: f64, eta_down: f64,
    dt: f64, steps: usize, paths: usize, seed: u64,
) -> Option<Report> {
    let finites = [s0, mu, sigma, jump_lambda, up_prob, eta_up, eta_down, dt];
    if finites.iter().any(|x| !x.is_finite()) { return None; }
    if s0 <= 0.0 || sigma < 0.0 || dt <= 0.0 { return None; }
    if jump_lambda < 0.0 { return None; }
    if !(0.0..=1.0).contains(&up_prob) { return None; }
    if eta_up <= 1.0 || eta_down <= 0.0 { return None; }
    if steps < 1 || paths < 1 { return None; }
    let kappa = up_prob * eta_up / (eta_up - 1.0)
        + (1.0 - up_prob) * eta_down / (eta_down + 1.0) - 1.0;
    let drift = (mu - jump_lambda * kappa - 0.5 * sigma * sigma) * dt;
    let diffusion_coef = sigma * dt.sqrt();
    let lambda_dt = jump_lambda * dt;
    let mut state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    let mut terminals = Vec::with_capacity(paths);
    let mut up_total: u64 = 0;
    let mut down_total: u64 = 0;
    for _ in 0..paths {
        let mut s = s0;
        for _ in 0..steps {
            let z = next_normal(&mut state);
            let k = next_poisson(&mut state, lambda_dt);
            let mut log_jump = 0.0;
            for _ in 0..k {
                let u = next_uniform(&mut state);
                let e = -next_uniform(&mut state).max(1e-300).ln();
                if u < up_prob {
                    log_jump += e / eta_up;
                    up_total += 1;
                } else {
                    log_jump -= e / eta_down;
                    down_total += 1;
                }
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
        up_jumps: up_total,
        down_jumps: down_total,
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

fn next_poisson(state: &mut u64, lam: f64) -> u32 {
    if lam <= 0.0 { return 0; }
    let limit = (-lam).exp();
    let mut p = 1.0;
    let mut k: u32 = 0;
    loop {
        k += 1;
        p *= next_uniform(state);
        if p <= limit { return k - 1; }
        if k > 1000 { return k; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(-1.0, 0.05, 0.2, 1.0, 0.4, 10.0, 5.0, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, -0.2, 1.0, 0.4, 10.0, 5.0, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, -1.0, 0.4, 10.0, 5.0, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 1.0, 1.5, 10.0, 5.0, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 1.0, 0.4, 0.5, 5.0, 0.01, 100, 50, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 1.0, 0.4, 10.0, -1.0, 0.01, 100, 50, 1).is_none());
        assert!(compute(f64::NAN, 0.05, 0.2, 1.0, 0.4, 10.0, 5.0, 0.01, 100, 50, 1).is_none());
    }

    #[test]
    fn zero_lambda_matches_pure_gbm() {
        let r = compute(100.0, 0.05, 0.0, 0.0, 0.4, 10.0, 5.0, 0.01, 100, 50, 42).unwrap();
        let expected = 100.0 * (0.05_f64 * 1.0).exp();
        assert!((r.mean_terminal - expected).abs() < 1e-6);
        assert_eq!(r.up_jumps, 0);
        assert_eq!(r.down_jumps, 0);
    }

    #[test]
    fn asymmetric_p_yields_jump_count_asymmetry() {
        // p_up = 0.1 → up_jumps << down_jumps.
        let r = compute(
            100.0, 0.0, 0.10,
            5.0, 0.1, 10.0, 5.0,
            0.01, 100, 200, 7,
        ).unwrap();
        assert!(r.up_jumps < r.down_jumps);
    }

    #[test]
    fn down_skewed_distribution_yields_negative_skew() {
        // Heavily negative-skewed jumps → negative log-return skew.
        let r = compute(
            100.0, 0.0, 0.10,
            5.0, 0.1, 20.0, 3.0,
            0.01, 252, 5000, 7,
        ).unwrap();
        assert!(r.skew_log_return < 0.0);
    }

    #[test]
    fn reproducible_with_same_seed() {
        let a = compute(100.0, 0.05, 0.2, 1.0, 0.4, 10.0, 5.0, 0.01, 100, 50, 99).unwrap();
        let b = compute(100.0, 0.05, 0.2, 1.0, 0.4, 10.0, 5.0, 0.01, 100, 50, 99).unwrap();
        assert_eq!(a.mean_terminal.to_bits(), b.mean_terminal.to_bits());
        assert_eq!(a.up_jumps, b.up_jumps);
        assert_eq!(a.down_jumps, b.down_jumps);
    }

    #[test]
    fn terminals_stay_positive() {
        let r = compute(100.0, 0.05, 0.2, 1.0, 0.4, 10.0, 5.0, 0.01, 252, 200, 11).unwrap();
        assert!(r.mean_terminal > 0.0);
    }
}
