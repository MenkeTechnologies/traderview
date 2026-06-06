//! Ledoit-Wolf (2004) covariance matrix shrinkage estimator.
//!
//! Shrinks the noisy sample covariance Σ̂ toward a structured target T:
//!
//!   Σ_LW = δ · T + (1 − δ) · Σ̂
//!
//! Optimal δ ∈ [0, 1] is derived in closed form to minimize the Frobenius-
//! norm MSE between the shrunk estimator and the true covariance. The
//! canonical target is the "constant-correlation" estimator (equal
//! pairwise correlation r̄ and per-asset variances σ²_i):
//!
//!   T_ii = σ²_i
//!   T_ij = r̄ · σ_i · σ_j     (i ≠ j)
//!
//! Pure compute. Numerator π̂ and denominator γ̂ for δ follow Ledoit-Wolf
//! (2003, 2004) eq. (3.8)–(3.12) on the constant-correlation target.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LedoitWolfReport {
    pub shrunk_covariance: Vec<Vec<f64>>,
    pub shrinkage_intensity: f64,
    pub sample_covariance: Vec<Vec<f64>>,
    pub target: Vec<Vec<f64>>,
    pub avg_correlation: f64,
    pub n_observations: usize,
}

pub fn estimate(returns: &[Vec<f64>]) -> Option<LedoitWolfReport> {
    // Input: returns[t][i] — T rows × N cols. Equivalently transpose if
    // caller has assets-first; we adopt time-first.
    let t_obs = returns.len();
    if t_obs < 3 {
        return None;
    }
    let n = returns[0].len();
    if n < 2 {
        return None;
    }
    if returns.iter().any(|row| row.len() != n) {
        return None;
    }
    if returns.iter().any(|row| row.iter().any(|x| !x.is_finite())) {
        return None;
    }
    let t_f = t_obs as f64;
    // Sample mean.
    let mut means = vec![0.0_f64; n];
    for i in 0..n {
        means[i] = returns.iter().map(|r| r[i]).sum::<f64>() / t_f;
    }
    // Demeaned matrix.
    let mut demeaned = vec![vec![0.0_f64; n]; t_obs];
    for (t, row) in returns.iter().enumerate() {
        for i in 0..n {
            demeaned[t][i] = row[i] - means[i];
        }
    }
    // Sample covariance Σ̂ = (1/T) X' X (population estimator, per Ledoit-Wolf).
    let mut sample_cov = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in i..n {
            let s: f64 = demeaned.iter().map(|r| r[i] * r[j]).sum();
            let v = s / t_f;
            sample_cov[i][j] = v;
            sample_cov[j][i] = v;
        }
    }
    // Per-asset stdev.
    let stdev: Vec<f64> = (0..n).map(|i| sample_cov[i][i].max(0.0).sqrt()).collect();
    if stdev.iter().any(|s| !s.is_finite() || *s <= 0.0) {
        return None; // any zero-variance asset makes correlation undefined
    }
    // Average pairwise correlation r̄.
    let mut sum_corr = 0.0_f64;
    let mut pairs = 0_usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let c = sample_cov[i][j] / (stdev[i] * stdev[j]);
            sum_corr += c;
            pairs += 1;
        }
    }
    let r_bar = if pairs > 0 {
        sum_corr / pairs as f64
    } else {
        0.0
    };
    // Target T_ij = r̄ · σ_i · σ_j (diagonal = σ_i²).
    let mut target = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            target[i][j] = if i == j {
                sample_cov[i][i]
            } else {
                r_bar * stdev[i] * stdev[j]
            };
        }
    }
    // π̂ — sum of asymptotic variances of sample-cov entries.
    #[allow(clippy::needless_range_loop)] // i, j indices both needed for matrix access
    let mut pi_hat = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            let mut acc = 0.0_f64;
            for row in demeaned.iter() {
                acc += (row[i] * row[j] - sample_cov[i][j]).powi(2);
            }
            pi_hat += acc / t_f;
        }
    }
    // ρ̂ — sum of asymptotic covariances of sample cov w/ target.
    // For constant-correlation target Ledoit-Wolf approximate by replacing
    // diagonal terms of π̂ with the standard formula and off-diagonals by:
    //   (r̄/2) · (σ_j/σ_i) · θ_ii_ij + (σ_i/σ_j) · θ_jj_ij
    // where θ_kk_ij = (1/T) Σ_t [(x_t,k − x̄_k)² · (x_t,i − x̄_i)(x_t,j − x̄_j)]
    //                  − σ²_k · σ_ij
    let mut rho_hat = 0.0_f64;
    for i in 0..n {
        rho_hat += pi_term_diag(&demeaned, &sample_cov, i, t_obs);
    }
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let theta_ii_ij = theta_kk_ij(&demeaned, &sample_cov, i, i, j, t_obs);
            let theta_jj_ij = theta_kk_ij(&demeaned, &sample_cov, j, i, j, t_obs);
            rho_hat += (r_bar / 2.0)
                * ((stdev[j] / stdev[i]) * theta_ii_ij + (stdev[i] / stdev[j]) * theta_jj_ij);
        }
    }
    // γ̂ — squared Frobenius distance between sample cov and target.
    let mut gamma_hat = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            gamma_hat += (target[i][j] - sample_cov[i][j]).powi(2);
        }
    }
    if gamma_hat <= 0.0 {
        // Sample cov already equals target — no shrinkage needed.
        return Some(LedoitWolfReport {
            shrunk_covariance: sample_cov.clone(),
            shrinkage_intensity: 0.0,
            sample_covariance: sample_cov,
            target,
            avg_correlation: r_bar,
            n_observations: t_obs,
        });
    }
    let kappa = (pi_hat - rho_hat) / gamma_hat;
    let delta = (kappa / t_f).clamp(0.0, 1.0);
    let mut shrunk = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            shrunk[i][j] = delta * target[i][j] + (1.0 - delta) * sample_cov[i][j];
        }
    }
    Some(LedoitWolfReport {
        shrunk_covariance: shrunk,
        shrinkage_intensity: delta,
        sample_covariance: sample_cov,
        target,
        avg_correlation: r_bar,
        n_observations: t_obs,
    })
}

fn pi_term_diag(demeaned: &[Vec<f64>], cov: &[Vec<f64>], i: usize, t: usize) -> f64 {
    let n = cov.len();
    let mut sum = 0.0_f64;
    for j in 0..n {
        let mut acc = 0.0_f64;
        for row in demeaned {
            acc += (row[i] * row[j] - cov[i][j]).powi(2);
        }
        sum += acc / t as f64;
        let _ = n;
    }
    sum
}

fn theta_kk_ij(
    demeaned: &[Vec<f64>],
    cov: &[Vec<f64>],
    k: usize,
    i: usize,
    j: usize,
    t: usize,
) -> f64 {
    let mut acc = 0.0_f64;
    for row in demeaned {
        let dk = row[k];
        acc += (dk * dk - cov[k][k]) * (row[i] * row[j] - cov[i][j]);
    }
    acc / t as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rand_matrix(t: usize, n: usize, seed: u64) -> Vec<Vec<f64>> {
        let mut state = seed;
        (0..t)
            .map(|_| {
                (0..n)
                    .map(|_| {
                        state = state
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407);
                        ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.04
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn too_few_observations_returns_none() {
        assert!(estimate(&[]).is_none());
        assert!(estimate(&[vec![0.01, 0.02]]).is_none());
    }

    #[test]
    fn dim_mismatch_returns_none() {
        let r = vec![vec![0.01, 0.02], vec![0.01], vec![0.01, 0.02]];
        assert!(estimate(&r).is_none());
    }

    #[test]
    fn single_asset_returns_none() {
        let r = vec![vec![0.01], vec![0.02], vec![0.03]];
        assert!(estimate(&r).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut r = rand_matrix(50, 3, 42);
        r[5][1] = f64::NAN;
        assert!(estimate(&r).is_none());
    }

    #[test]
    fn shrinkage_intensity_in_unit_interval() {
        let r = rand_matrix(100, 5, 42);
        let report = estimate(&r).unwrap();
        assert!((0.0..=1.0).contains(&report.shrinkage_intensity));
    }

    #[test]
    fn shrunk_diagonal_equals_sample_diagonal() {
        // For the constant-correlation target, target diagonal = sample
        // diagonal → shrunk diagonal = sample diagonal exactly.
        let r = rand_matrix(100, 4, 7);
        let report = estimate(&r).unwrap();
        for i in 0..4 {
            assert!(
                (report.shrunk_covariance[i][i] - report.sample_covariance[i][i]).abs() < 1e-12
            );
        }
    }

    #[test]
    fn shrunk_matrix_symmetric() {
        let r = rand_matrix(80, 4, 99);
        let report = estimate(&r).unwrap();
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (report.shrunk_covariance[i][j] - report.shrunk_covariance[j][i]).abs() < 1e-12
                );
            }
        }
    }

    #[test]
    fn higher_noise_lower_t_yields_higher_shrinkage() {
        // Fewer observations → more shrinkage toward structured target.
        let small = estimate(&rand_matrix(20, 5, 13)).unwrap();
        let large = estimate(&rand_matrix(500, 5, 13)).unwrap();
        assert!(
            small.shrinkage_intensity >= large.shrinkage_intensity,
            "small T should shrink more than large T: small={} large={}",
            small.shrinkage_intensity,
            large.shrinkage_intensity
        );
    }

    #[test]
    fn avg_correlation_in_minus_one_to_one() {
        let r = rand_matrix(100, 5, 88);
        let report = estimate(&r).unwrap();
        assert!((-1.0..=1.0).contains(&report.avg_correlation));
    }
}
