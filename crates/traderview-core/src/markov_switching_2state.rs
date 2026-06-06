//! 2-State Markov Regime Switching — Hamilton (1989) Gaussian-mixture
//! HMM fitted via Baum-Welch EM.
//!
//! Model:
//!   state S_t ∈ {0, 1}, transition matrix P = [[p00, p01], [p10, p11]]
//!   emission r_t | S_t = k ~ N(μ_k, σ_k²)
//!
//! E-step: compute filtered probabilities ξ_t(k) = P(S_t = k | r_1..r_t)
//! via forward recursion, then smoothed γ_t(k) via backward recursion.
//! M-step: re-estimate μ_k, σ_k², P from γ and pair-occupancies.
//!
//! Init: state 0 = low-vol, state 1 = high-vol, both means at sample
//! mean, stdevs at 0.5x and 2x sample stdev. Iterate until log-lik
//! converges or hits cap (50 iterations).
//!
//! Returns per-bar smoothed P(S_t = 1) (the high-vol "stress" state),
//! plus final params and total log-likelihood. Useful for regime
//! detection in returns.
//!
//! Pure compute. Companion to `regime_classifier`,
//! `bayesian_change_point_detector`, `garch_1_1`.

#[derive(Debug)]
pub struct Report {
    pub prob_state1: Vec<f64>,
    pub mu0: f64,
    pub mu1: f64,
    pub sigma0: f64,
    pub sigma1: f64,
    pub p00: f64,
    pub p11: f64,
    pub log_likelihood: f64,
    pub iterations: u32,
}

pub fn compute(returns: &[f64]) -> Option<Report> {
    let n = returns.len();
    if n < 30 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mean = returns.iter().sum::<f64>() / n as f64;
    let var = returns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    if var < 1e-18 {
        return None;
    }
    let stdev = var.sqrt();
    // Init.
    let mut mu = [mean, mean];
    let mut sigma = [(stdev * 0.5).max(1e-9), stdev * 2.0];
    let mut p = [[0.95_f64, 0.05], [0.05, 0.95]];
    let mut pi = [0.5_f64, 0.5];
    let mut prev_ll = f64::NEG_INFINITY;
    let mut alpha = vec![[0.0_f64; 2]; n];
    let mut beta = vec![[0.0_f64; 2]; n];
    let mut gamma = vec![[0.0_f64; 2]; n];
    let mut c = vec![0.0_f64; n];
    let mut iter = 0_u32;
    let mut log_lik = f64::NEG_INFINITY;
    for _ in 0..50 {
        iter += 1;
        // E-step: forward (scaled).
        let mut sum0 = 0.0;
        for k in 0..2 {
            let e = gauss_pdf(returns[0], mu[k], sigma[k]);
            alpha[0][k] = pi[k] * e;
            sum0 += alpha[0][k];
        }
        c[0] = if sum0 > 0.0 { 1.0 / sum0 } else { 1.0 };
        alpha[0][0] *= c[0];
        alpha[0][1] *= c[0];
        for t in 1..n {
            let mut sum = 0.0;
            for k in 0..2 {
                let e = gauss_pdf(returns[t], mu[k], sigma[k]);
                alpha[t][k] = (alpha[t - 1][0] * p[0][k] + alpha[t - 1][1] * p[1][k]) * e;
                sum += alpha[t][k];
            }
            c[t] = if sum > 0.0 { 1.0 / sum } else { 1.0 };
            alpha[t][0] *= c[t];
            alpha[t][1] *= c[t];
        }
        log_lik = -c.iter().map(|x| x.ln()).sum::<f64>();
        // Backward.
        beta[n - 1] = [c[n - 1]; 2];
        for t in (0..n - 1).rev() {
            for k in 0..2 {
                let mut s = 0.0;
                for j in 0..2 {
                    let e = gauss_pdf(returns[t + 1], mu[j], sigma[j]);
                    s += p[k][j] * e * beta[t + 1][j];
                }
                beta[t][k] = s * c[t];
            }
        }
        // Smoothed γ and pair-occupancy ξ.
        let mut xi_sum = [[0.0_f64; 2]; 2];
        for t in 0..n {
            let mut z = 0.0;
            for k in 0..2 {
                gamma[t][k] = alpha[t][k] * beta[t][k] / c[t];
                z += gamma[t][k];
            }
            if z > 0.0 {
                gamma[t][0] /= z;
                gamma[t][1] /= z;
            }
        }
        for t in 0..n - 1 {
            let mut z = 0.0;
            let mut xi_t = [[0.0_f64; 2]; 2];
            for k in 0..2 {
                for j in 0..2 {
                    let e = gauss_pdf(returns[t + 1], mu[j], sigma[j]);
                    xi_t[k][j] = alpha[t][k] * p[k][j] * e * beta[t + 1][j];
                    z += xi_t[k][j];
                }
            }
            if z > 0.0 {
                for k in 0..2 {
                    for j in 0..2 {
                        xi_sum[k][j] += xi_t[k][j] / z;
                    }
                }
            }
        }
        // M-step.
        pi = [gamma[0][0], gamma[0][1]];
        for k in 0..2 {
            let denom: f64 = gamma.iter().take(n - 1).map(|g| g[k]).sum();
            if denom > 1e-15 {
                for j in 0..2 {
                    p[k][j] = xi_sum[k][j] / denom;
                }
                let s = p[k][0] + p[k][1];
                if s > 0.0 {
                    p[k][0] /= s;
                    p[k][1] /= s;
                }
            }
        }
        for k in 0..2 {
            let w: f64 = gamma.iter().map(|g| g[k]).sum();
            if w > 1e-15 {
                let m_k: f64 = returns
                    .iter()
                    .zip(gamma.iter())
                    .map(|(r, g)| r * g[k])
                    .sum::<f64>()
                    / w;
                let v_k: f64 = returns
                    .iter()
                    .zip(gamma.iter())
                    .map(|(r, g)| (r - m_k).powi(2) * g[k])
                    .sum::<f64>()
                    / w;
                mu[k] = m_k;
                sigma[k] = v_k.max(1e-18).sqrt();
            }
        }
        if (log_lik - prev_ll).abs() < 1e-7 {
            break;
        }
        prev_ll = log_lik;
    }
    // Convention: state 1 = higher variance.
    let (idx0, idx1) = if sigma[0] <= sigma[1] { (0, 1) } else { (1, 0) };
    let prob_state1: Vec<f64> = gamma.iter().map(|g| g[idx1]).collect();
    Some(Report {
        prob_state1,
        mu0: mu[idx0],
        mu1: mu[idx1],
        sigma0: sigma[idx0],
        sigma1: sigma[idx1],
        p00: p[idx0][idx0],
        p11: p[idx1][idx1],
        log_likelihood: log_lik,
        iterations: iter,
    })
}

fn gauss_pdf(x: f64, mu: f64, sigma: f64) -> f64 {
    if sigma <= 0.0 {
        return 0.0;
    }
    let z = (x - mu) / sigma;
    (1.0 / (sigma * (std::f64::consts::TAU).sqrt())) * (-0.5 * z * z).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let r = vec![0.01_f64; 20];
        assert!(compute(&r).is_none());
        let mut r2 = vec![0.01_f64; 50];
        r2[5] = f64::NAN;
        assert!(compute(&r2).is_none());
    }

    #[test]
    fn flat_returns_yield_zero_variance_rejected() {
        let r = vec![0.005_f64; 100];
        assert!(compute(&r).is_none());
    }

    #[test]
    fn high_vol_segment_inflates_state1_prob() {
        // 100 low-vol + 100 high-vol returns.
        let mut r = Vec::new();
        for i in 0..100 {
            r.push(((i as f64 * 0.3).sin()) * 0.005);
        }
        for i in 0..100 {
            r.push(((i as f64 * 0.5).sin()) * 0.05);
        }
        let rep = compute(&r).unwrap();
        // High-vol bars (idx 100..) should have higher P(state=1) on
        // average than low-vol bars.
        let p_low: f64 = rep.prob_state1[..100].iter().sum::<f64>() / 100.0;
        let p_high: f64 = rep.prob_state1[100..].iter().sum::<f64>() / 100.0;
        assert!(p_high > p_low);
    }

    #[test]
    fn sigma1_at_least_sigma0() {
        let r: Vec<f64> = (0..200).map(|i| ((i as f64 * 0.3).sin()) * 0.02).collect();
        let rep = compute(&r).unwrap();
        assert!(rep.sigma1 >= rep.sigma0 - 1e-12);
    }

    #[test]
    fn transition_probs_in_unit_interval() {
        let r: Vec<f64> = (0..200).map(|i| ((i as f64 * 0.3).sin()) * 0.02).collect();
        let rep = compute(&r).unwrap();
        assert!((0.0..=1.0).contains(&rep.p00));
        assert!((0.0..=1.0).contains(&rep.p11));
    }

    #[test]
    fn prob_lengths_match_input() {
        let r: Vec<f64> = (0..200).map(|i| ((i as f64 * 0.3).sin()) * 0.02).collect();
        let rep = compute(&r).unwrap();
        assert_eq!(rep.prob_state1.len(), 200);
        for &p in &rep.prob_state1 {
            assert!((0.0..=1.0).contains(&p));
        }
    }
}
