//! Vasicek Short-Rate Path Simulator — Monte Carlo trajectories of the
//! Ornstein-Uhlenbeck short rate:
//!
//!   dr_t = a · (b - r_t) · dt + σ · dW_t
//!
//! where:
//!   a = mean-reversion speed (a > 0)
//!   b = long-term mean rate
//!   σ = vol of rate changes
//!
//! Exact-step discretization (no Euler bias):
//!   r_{t+dt} = b + (r_t - b) · e^(-a·dt) + σ · sqrt((1-e^(-2a·dt)) / (2a)) · Z
//!
//! Reports trajectory mean / stdev of the terminal rate and the
//! fraction of paths that went negative (Vasicek allows negative
//! rates — a real-world feature post-2014 EUR/CHF).
//!
//! Pure compute. Companion to `vasicek` (ZCB pricer), `cir_pricing`,
//! `hull_white_pricing`, `gbm_path_simulator`.

#[derive(Debug)]
pub struct Report {
    pub mean_terminal_rate: f64,
    pub stdev_terminal_rate: f64,
    pub min_terminal_rate: f64,
    pub max_terminal_rate: f64,
    pub negative_path_fraction: f64,
    pub paths_run: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    r0: f64,
    a: f64,
    b: f64,
    sigma: f64,
    dt: f64,
    steps: usize,
    paths: usize,
    seed: u64,
) -> Option<Report> {
    let finites = [r0, a, b, sigma, dt];
    if finites.iter().any(|x| !x.is_finite()) {
        return None;
    }
    if a <= 0.0 || sigma < 0.0 || dt <= 0.0 {
        return None;
    }
    if steps < 1 || paths < 1 {
        return None;
    }
    let decay = (-a * dt).exp();
    let two_a_dt = 2.0 * a * dt;
    let one_minus_decay2 = 1.0 - (-two_a_dt).exp();
    let noise_scale = sigma * (one_minus_decay2 / (2.0 * a)).max(0.0).sqrt();
    let mut state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    let mut terminals = Vec::with_capacity(paths);
    let mut went_negative = 0_usize;
    for _ in 0..paths {
        let mut r = r0;
        let mut hit_neg = r < 0.0;
        for _ in 0..steps {
            let z = next_normal(&mut state);
            r = b + (r - b) * decay + noise_scale * z;
            if r < 0.0 {
                hit_neg = true;
            }
        }
        terminals.push(r);
        if hit_neg {
            went_negative += 1;
        }
    }
    let n = terminals.len() as f64;
    let mean = terminals.iter().sum::<f64>() / n;
    let var = terminals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let stdev = var.max(0.0).sqrt();
    let min = terminals.iter().copied().fold(f64::INFINITY, f64::min);
    let max = terminals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some(Report {
        mean_terminal_rate: mean,
        stdev_terminal_rate: stdev,
        min_terminal_rate: min,
        max_terminal_rate: max,
        negative_path_fraction: went_negative as f64 / n,
        paths_run: paths,
    })
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
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
    if u1 < 1e-300 {
        u1 = 1e-300;
    }
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(0.03, -1.0, 0.04, 0.01, 0.01, 100, 100, 1).is_none());
        assert!(compute(0.03, 1.0, 0.04, -0.01, 0.01, 100, 100, 1).is_none());
        assert!(compute(0.03, 1.0, 0.04, 0.01, 0.0, 100, 100, 1).is_none());
        assert!(compute(0.03, 1.0, 0.04, 0.01, 0.01, 0, 100, 1).is_none());
        assert!(compute(0.03, 1.0, 0.04, 0.01, 0.01, 100, 0, 1).is_none());
        assert!(compute(f64::NAN, 1.0, 0.04, 0.01, 0.01, 100, 100, 1).is_none());
    }

    #[test]
    fn zero_vol_converges_to_mean_deterministically() {
        // σ=0 + a > 0 → exponential decay to b. After many steps r → b.
        let r = compute(0.01, 5.0, 0.05, 0.0, 0.01, 500, 50, 42).unwrap();
        assert!((r.mean_terminal_rate - 0.05).abs() < 1e-6);
        assert!(r.stdev_terminal_rate < 1e-6);
    }

    #[test]
    fn higher_sigma_increases_terminal_stdev() {
        let low = compute(0.03, 0.5, 0.04, 0.005, 0.01, 100, 2000, 1).unwrap();
        let high = compute(0.03, 0.5, 0.04, 0.05, 0.01, 100, 2000, 1).unwrap();
        assert!(high.stdev_terminal_rate > low.stdev_terminal_rate);
    }

    #[test]
    fn high_sigma_produces_negative_paths() {
        // Low rates + high vol → some paths go negative.
        let r = compute(0.005, 0.5, 0.01, 0.10, 0.01, 100, 1000, 7).unwrap();
        assert!(r.negative_path_fraction > 0.0);
    }

    #[test]
    fn reproducible_with_same_seed() {
        let a = compute(0.03, 0.5, 0.04, 0.01, 0.01, 100, 100, 99).unwrap();
        let b = compute(0.03, 0.5, 0.04, 0.01, 0.01, 100, 100, 99).unwrap();
        assert_eq!(
            a.mean_terminal_rate.to_bits(),
            b.mean_terminal_rate.to_bits()
        );
    }

    #[test]
    fn mean_reverts_toward_b() {
        // Start far from b — mean of terminals should land closer to b.
        let r = compute(0.20, 2.0, 0.04, 0.01, 0.01, 252, 1000, 7).unwrap();
        assert!((r.mean_terminal_rate - 0.04_f64).abs() < (0.20_f64 - 0.04).abs());
    }
}
