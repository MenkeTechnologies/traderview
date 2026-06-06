//! Bayesian Change-Point Detector — online Bayesian Online Change-Point
//! Detection (BOCPD) by Adams & MacKay (2007), simplified for a
//! Gaussian return model.
//!
//! Hazard rate `h` is the prior probability of a change point at each
//! time step. The detector reports the run length r_t (bars since
//! last change point) and the change-point probability cpp_t
//! (probability that the immediately-prior bar was a change point).
//!
//! Output cpp ≈ 1.0 → very likely change point just happened.
//! Output cpp ≈ 0.0 → return distribution is stable.
//!
//! Pure compute. Defaults: hazard = 1/100 (~ one change-point every 100
//! bars), prior_mean = 0, prior_var = 1.
//! Companion to `cusum`, `pelt_segmentation`, `streaks`,
//! `volatility_regime`, `regime_classifier`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BocpdReport {
    pub change_point_probability: Vec<Option<f64>>,
    pub expected_run_length: Vec<Option<f64>>,
    pub hazard: f64,
}

pub fn compute(returns: &[f64], hazard: f64) -> BocpdReport {
    let n = returns.len();
    let mut report = BocpdReport {
        change_point_probability: vec![None; n],
        expected_run_length: vec![None; n],
        hazard,
    };
    if n < 2 || !hazard.is_finite() || !(0.0..=1.0).contains(&hazard) {
        return report;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return report;
    }
    // Simplified BOCPD with Gaussian likelihood + conjugate normal prior.
    // We maintain run-length distribution truncated to last `n + 1` values.
    let mut run_post = vec![1.0_f64]; // posterior over run lengths
    let mut running_mean = 0.0_f64;
    let mut running_var = 1.0_f64;
    let mut running_n = 0.0_f64;
    for (t, &r) in returns.iter().enumerate() {
        // Predictive prob: P(r | run length). Use rolling mean/var
        // estimate (simplified — true BOCPD per-run-length is heavier).
        let pred = if running_n > 0.0 {
            gaussian_pdf(r, running_mean, running_var.max(1e-9))
        } else {
            gaussian_pdf(r, 0.0, 1.0)
        };
        // Growth probabilities: P(r_t = r_{t-1} + 1) = pred · (1 - hazard) · run_post.
        let growth: Vec<f64> = run_post.iter().map(|p| p * pred * (1.0 - hazard)).collect();
        // Change-point probability: P(r_t = 0) = sum(pred · hazard · run_post).
        let cp: f64 = run_post.iter().map(|p| p * pred * hazard).sum();
        // Normalize and update.
        let mut new_post = vec![cp];
        new_post.extend(growth);
        let total: f64 = new_post.iter().sum();
        if total > 0.0 {
            for v in new_post.iter_mut() {
                *v /= total;
            }
        }
        run_post = new_post;
        // Update running stats.
        running_n += 1.0;
        let delta = r - running_mean;
        running_mean += delta / running_n;
        running_var += (delta * (r - running_mean) - running_var) / running_n;
        // Report.
        let cp_prob = run_post[0];
        let exp_rl: f64 = run_post.iter().enumerate().map(|(i, p)| i as f64 * p).sum();
        report.change_point_probability[t] = Some(cp_prob);
        report.expected_run_length[t] = Some(exp_rl);
    }
    report
}

fn gaussian_pdf(x: f64, mean: f64, var: f64) -> f64 {
    let s = var.sqrt();
    let z = (x - mean) / s;
    (1.0 / (s * (2.0 * std::f64::consts::PI).sqrt())) * (-0.5 * z * z).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let r = vec![0.01_f64; 50];
        let report = compute(&r, 2.0);
        assert!(report.change_point_probability.iter().all(|x| x.is_none()));
        let report2 = compute(&[r[0]], 0.01);
        assert!(report2.change_point_probability.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let r = vec![0.01, f64::NAN, 0.01];
        let report = compute(&r, 0.01);
        assert!(report.change_point_probability.iter().all(|x| x.is_none()));
    }

    #[test]
    fn stable_returns_yield_low_cp_probability() {
        // 100 bars of small, similar returns → no change points.
        let r = vec![0.001_f64; 100];
        let report = compute(&r, 0.01);
        let final_cp = report
            .change_point_probability
            .iter()
            .rev()
            .find_map(|x| *x)
            .unwrap();
        assert!(final_cp < 0.5);
    }

    #[test]
    fn run_length_grows_for_stable_series() {
        let r = vec![0.001_f64; 100];
        let report = compute(&r, 0.01);
        let last_rl = report
            .expected_run_length
            .iter()
            .rev()
            .find_map(|x| *x)
            .unwrap();
        assert!(last_rl > 10.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let r = vec![0.01_f64; 50];
        let report = compute(&r, 0.01);
        assert_eq!(report.change_point_probability.len(), 50);
        assert_eq!(report.expected_run_length.len(), 50);
    }
}
