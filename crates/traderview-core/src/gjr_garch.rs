//! GJR-GARCH(1,1) — Glosten, Jagannathan & Runkle (1993).
//!
//! Asymmetric GARCH that allows negative returns to amplify volatility
//! more than positive returns of the same magnitude (the "leverage
//! effect"):
//!
//!   σ²_t = ω + α · r²_{t−1} + γ · r²_{t−1} · 1{r_{t−1} < 0} + β · σ²_{t−1}
//!
//! where the indicator `1{r < 0}` switches the γ term on for negative
//! shocks only. Stationarity requires:
//!
//!   ω > 0,  α, β, γ ≥ 0,  α + β + γ/2 < 1
//!
//! Unconditional variance:
//!   σ²_∞ = ω / (1 − α − β − γ/2)
//!
//! Estimation: maximum-likelihood under Gaussian innovations via a
//! lightweight Nelder-Mead simplex on a 4-parameter log-space surface
//! (log(ω), logit(α), logit(β), logit(γ)). Approximate but adequate
//! for daily-equity work.
//!
//! Pure compute. Companion to `garch_1_1`, `arch_lm_test`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GjrGarchReport {
    pub omega: f64,
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub unconditional_variance: f64,
    pub log_likelihood: f64,
    pub conditional_variance: Vec<f64>,
}

pub fn estimate(returns: &[f64]) -> Option<GjrGarchReport> {
    if returns.len() < 30 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // Reject truly flat input: float roundoff can yield tiny sample
    // variance even for identical values.
    let (mn, mx) = returns
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| {
            (a.min(*x), b.max(*x))
        });
    if mx - mn <= 0.0 {
        return None;
    }
    let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
    if var <= 0.0 {
        return None;
    }
    // Initial guess: low persistence, mild asymmetry.
    let mut params = [
        (var * 0.05_f64).ln().max(-30.0), // log(omega)
        logit(0.05),                      // logit(alpha)
        logit(0.85),                      // logit(beta)
        logit(0.05),                      // logit(gamma)
    ];
    let centered: Vec<f64> = returns.iter().map(|r| r - mean).collect();
    // Nelder-Mead (shared optimizer — same coefficients the inline
    // version used, so fitted params are unchanged).
    let (best, _) = crate::optimize::nelder_mead(&params, 0.1, 500, |p| {
        neg_log_likelihood_at(&[p[0], p[1], p[2], p[3]], &centered)
    });
    params.copy_from_slice(&best);
    let omega = params[0].exp();
    let alpha = sigmoid(params[1]);
    let beta = sigmoid(params[2]);
    let gamma = sigmoid(params[3]);
    let persistence = alpha + beta + gamma * 0.5;
    if persistence >= 1.0 {
        return None;
    }
    let uncond_var = omega / (1.0 - persistence);
    let cond_var = compute_variance_path(&centered, omega, alpha, beta, gamma, uncond_var);
    let ll = log_likelihood(&centered, &cond_var);
    Some(GjrGarchReport {
        omega,
        alpha,
        beta,
        gamma,
        unconditional_variance: uncond_var,
        log_likelihood: ll,
        conditional_variance: cond_var,
    })
}

fn compute_variance_path(
    r: &[f64],
    omega: f64,
    alpha: f64,
    beta: f64,
    gamma: f64,
    seed_var: f64,
) -> Vec<f64> {
    let n = r.len();
    let mut sv = Vec::with_capacity(n);
    let mut prev = seed_var;
    for i in 0..n {
        let v = if i == 0 {
            seed_var
        } else {
            let leverage = if r[i - 1] < 0.0 { gamma } else { 0.0 };
            omega + alpha * r[i - 1].powi(2) + leverage * r[i - 1].powi(2) + beta * prev
        };
        sv.push(v.max(1e-12));
        prev = v.max(1e-12);
    }
    sv
}

fn log_likelihood(r: &[f64], v: &[f64]) -> f64 {
    let two_pi_ln = (2.0 * std::f64::consts::PI).ln();
    -0.5 * r
        .iter()
        .zip(v.iter())
        .map(|(ri, vi)| two_pi_ln + vi.ln() + ri.powi(2) / vi)
        .sum::<f64>()
}

fn neg_log_likelihood_at(params: &[f64; 4], r: &[f64]) -> f64 {
    let omega = params[0].exp();
    let alpha = sigmoid(params[1]);
    let beta = sigmoid(params[2]);
    let gamma = sigmoid(params[3]);
    let persistence = alpha + beta + gamma * 0.5;
    if persistence >= 1.0 {
        return 1e12;
    }
    let uncond_var = omega / (1.0 - persistence);
    let v = compute_variance_path(r, omega, alpha, beta, gamma, uncond_var);
    -log_likelihood(r, &v)
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}
fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    (p / (1.0 - p)).ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rng(state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let u1 = ((*state >> 32) as f64 / u32::MAX as f64).max(1e-12);
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let u2 = (*state >> 32) as f64 / u32::MAX as f64;
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }

    fn simulate_gjr(
        n: usize,
        omega: f64,
        alpha: f64,
        beta: f64,
        gamma: f64,
        seed: u64,
    ) -> Vec<f64> {
        let mut state = seed;
        let mut sigma2 = omega / (1.0 - alpha - beta - gamma * 0.5);
        let mut out = Vec::with_capacity(n);
        let mut prev_r = 0.0_f64;
        for _ in 0..n {
            let leverage = if prev_r < 0.0 { gamma } else { 0.0 };
            sigma2 = omega + alpha * prev_r.powi(2) + leverage * prev_r.powi(2) + beta * sigma2;
            let r = sigma2.sqrt() * rng(&mut state);
            out.push(r);
            prev_r = r;
        }
        out
    }

    #[test]
    fn too_short_returns_none() {
        let returns = vec![0.01_f64; 10];
        assert!(estimate(&returns).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut returns = vec![0.01_f64; 50];
        returns[10] = f64::NAN;
        assert!(estimate(&returns).is_none());
    }

    #[test]
    fn flat_returns_none() {
        let returns = vec![0.01_f64; 100];
        assert!(estimate(&returns).is_none());
    }

    #[test]
    fn estimated_parameters_satisfy_stationarity() {
        let true_returns = simulate_gjr(500, 0.000005, 0.05, 0.85, 0.10, 42);
        let r = estimate(&true_returns).unwrap();
        let persistence = r.alpha + r.beta + r.gamma * 0.5;
        assert!(
            persistence < 1.0,
            "non-stationary fit: persistence {persistence}"
        );
        assert!(r.omega > 0.0);
    }

    #[test]
    fn unconditional_variance_close_to_sample_variance() {
        let returns = simulate_gjr(500, 0.000005, 0.05, 0.85, 0.10, 7);
        let r = estimate(&returns).unwrap();
        let n = returns.len() as f64;
        let mean: f64 = returns.iter().sum::<f64>() / n;
        let sample_var: f64 = returns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let rel_diff = (r.unconditional_variance - sample_var).abs() / sample_var;
        assert!(
            rel_diff < 0.7,
            "uncond var {} vs sample {}, rel diff = {:.2}",
            r.unconditional_variance,
            sample_var,
            rel_diff
        );
    }

    #[test]
    fn conditional_variance_aligned_to_input() {
        let returns = simulate_gjr(200, 0.000005, 0.05, 0.85, 0.10, 13);
        let r = estimate(&returns).unwrap();
        assert_eq!(r.conditional_variance.len(), returns.len());
        for v in &r.conditional_variance {
            assert!(*v > 0.0);
        }
    }
}
