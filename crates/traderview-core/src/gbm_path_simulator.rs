//! Geometric Brownian Motion Path Simulator — Monte Carlo paths under
//! the standard log-normal model.
//!
//!   dS_t / S_t = μ dt + σ dW_t
//!   S_t = S_0 · exp((μ - σ²/2) t + σ √t · Z)
//!
//! Per step: S_{i+1} = S_i · exp((μ - σ²/2) dt + σ √dt · Z_{i+1}).
//!
//! Uses a deterministic seeded LCG + Box-Muller for normals — no
//! external rand dependency, reproducible across runs.
//!
//! Returns terminal-price statistics across `paths` simulations of
//! `steps` bars each. Caller picks dt from horizon / steps externally.
//!
//! Pure compute. Companion to `monte_carlo`, `monte_carlo_var`,
//! `monte_carlo_option`, `jump_diffusion_simulator`.

#[derive(Debug)]
pub struct Report {
    pub mean_terminal: f64,
    pub stdev_terminal: f64,
    pub min_terminal: f64,
    pub max_terminal: f64,
    pub paths_run: usize,
}

pub fn compute(
    s0: f64, mu: f64, sigma: f64, dt: f64,
    steps: usize, paths: usize, seed: u64,
) -> Option<Report> {
    if !s0.is_finite() || !mu.is_finite() || !sigma.is_finite() || !dt.is_finite() {
        return None;
    }
    if s0 <= 0.0 || sigma < 0.0 || dt <= 0.0 { return None; }
    if steps < 1 || paths < 1 { return None; }
    let drift = (mu - 0.5 * sigma * sigma) * dt;
    let diffusion = sigma * dt.sqrt();
    let mut state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    let mut terminals = Vec::with_capacity(paths);
    for _ in 0..paths {
        let mut s = s0;
        for _ in 0..steps {
            let z = next_normal(&mut state);
            s *= (drift + diffusion * z).exp();
        }
        terminals.push(s);
    }
    let n = terminals.len() as f64;
    let mean = terminals.iter().sum::<f64>() / n;
    let var = terminals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let stdev = var.max(0.0).sqrt();
    let min = terminals.iter().copied().fold(f64::INFINITY, f64::min);
    let max = terminals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some(Report {
        mean_terminal: mean,
        stdev_terminal: stdev,
        min_terminal: min,
        max_terminal: max,
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
    // Box-Muller.
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
        assert!(compute(-1.0, 0.05, 0.2, 0.01, 100, 100, 1).is_none());
        assert!(compute(100.0, 0.05, -0.2, 0.01, 100, 100, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 0.0, 100, 100, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 0.01, 0, 100, 1).is_none());
        assert!(compute(100.0, 0.05, 0.2, 0.01, 100, 0, 1).is_none());
        assert!(compute(f64::NAN, 0.05, 0.2, 0.01, 100, 100, 1).is_none());
    }

    #[test]
    fn zero_vol_returns_deterministic_drift() {
        // σ=0: S_T = S_0 · exp(μ·T), no noise.
        let r = compute(100.0, 0.05, 0.0, 0.01, 100, 50, 42).unwrap();
        let expected = 100.0 * (0.05 * 1.0_f64).exp();
        assert!((r.mean_terminal - expected).abs() < 1e-6);
        assert!(r.stdev_terminal < 1e-6);
    }

    #[test]
    fn higher_vol_increases_stdev() {
        let low = compute(100.0, 0.05, 0.10, 0.01, 100, 2000, 1).unwrap();
        let high = compute(100.0, 0.05, 0.40, 0.01, 100, 2000, 1).unwrap();
        assert!(high.stdev_terminal > low.stdev_terminal);
    }

    #[test]
    fn terminals_remain_positive() {
        let r = compute(100.0, 0.05, 0.3, 0.01, 252, 100, 7).unwrap();
        assert!(r.min_terminal > 0.0);
    }

    #[test]
    fn reproducible_with_same_seed() {
        let a = compute(100.0, 0.05, 0.2, 0.01, 100, 50, 99).unwrap();
        let b = compute(100.0, 0.05, 0.2, 0.01, 100, 50, 99).unwrap();
        assert_eq!(a.mean_terminal.to_bits(), b.mean_terminal.to_bits());
    }

    #[test]
    fn paths_run_matches_request() {
        let r = compute(100.0, 0.05, 0.2, 0.01, 100, 250, 1).unwrap();
        assert_eq!(r.paths_run, 250);
    }
}
