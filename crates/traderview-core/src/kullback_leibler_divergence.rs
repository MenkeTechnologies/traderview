//! Kullback-Leibler Divergence — relative entropy of distribution P
//! with respect to distribution Q.
//!
//!   D_KL(P || Q) = Σ_i p_i · ln(p_i / q_i)        (discrete)
//!
//! Properties:
//!   - D_KL ≥ 0 with equality iff P = Q
//!   - Asymmetric: D_KL(P||Q) ≠ D_KL(Q||P)
//!   - Convex in (P, Q)
//!
//! Two convenience scores:
//!
//!   D_JS(P, Q) = 0.5 · D_KL(P || M) + 0.5 · D_KL(Q || M)
//!     where M = 0.5 · (P + Q) — symmetric Jensen-Shannon divergence
//!
//!   Hellinger(P, Q) = √(0.5 · Σ (√p_i − √q_i)²) ∈ [0, 1] — bounded
//!
//! Both inputs are assumed to be probability mass functions (non-
//! negative, summing to 1 within tolerance). Auto-renormalization is
//! applied if sums are close-but-not-exactly 1 (caller doesn't have
//! to normalize first).
//!
//! Use cases:
//!   - Compare model-implied distribution to empirical
//!   - Detect distribution drift between train and live
//!   - Probabilistic-forecast quality
//!
//! Pure compute. Companion to `gaussian_copula`, `wasserstein_1d`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DivergenceReport {
    pub kl_pq: f64,
    pub kl_qp: f64,
    pub jensen_shannon: f64,
    pub hellinger: f64,
    pub n_categories: usize,
}

pub fn compute(p: &[f64], q: &[f64]) -> Option<DivergenceReport> {
    if p.is_empty() || p.len() != q.len() {
        return None;
    }
    if p.iter().any(|x| !x.is_finite() || *x < 0.0) || q.iter().any(|x| !x.is_finite() || *x < 0.0)
    {
        return None;
    }
    let sum_p: f64 = p.iter().sum();
    let sum_q: f64 = q.iter().sum();
    if sum_p <= 0.0 || sum_q <= 0.0 {
        return None;
    }
    let p_norm: Vec<f64> = p.iter().map(|x| x / sum_p).collect();
    let q_norm: Vec<f64> = q.iter().map(|x| x / sum_q).collect();
    let mut kl_pq = 0.0_f64;
    let mut kl_qp = 0.0_f64;
    let mut js = 0.0_f64;
    let mut hel_sq = 0.0_f64;
    for i in 0..p.len() {
        let pi = p_norm[i];
        let qi = q_norm[i];
        let m = 0.5 * (pi + qi);
        if pi > 0.0 {
            if qi > 0.0 {
                kl_pq += pi * (pi / qi).ln();
            } else {
                kl_pq = f64::INFINITY;
            }
            if m > 0.0 {
                js += 0.5 * pi * (pi / m).ln();
            }
        }
        if qi > 0.0 {
            if pi > 0.0 {
                kl_qp += qi * (qi / pi).ln();
            } else {
                kl_qp = f64::INFINITY;
            }
            if m > 0.0 {
                js += 0.5 * qi * (qi / m).ln();
            }
        }
        hel_sq += (pi.sqrt() - qi.sqrt()).powi(2);
    }
    let hellinger = (0.5 * hel_sq).max(0.0).sqrt();
    Some(DivergenceReport {
        kl_pq,
        kl_qp,
        jensen_shannon: js,
        hellinger,
        n_categories: p.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], &[]).is_none());
    }

    #[test]
    fn mismatched_returns_none() {
        assert!(compute(&[0.5, 0.5], &[1.0]).is_none());
    }

    #[test]
    fn negative_returns_none() {
        assert!(compute(&[0.5, -0.5], &[0.5, 0.5]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[0.5, f64::NAN], &[0.5, 0.5]).is_none());
    }

    #[test]
    fn zero_total_returns_none() {
        assert!(compute(&[0.0, 0.0], &[0.5, 0.5]).is_none());
    }

    #[test]
    fn identical_distributions_yield_zero_divergence() {
        let p = vec![0.25, 0.25, 0.25, 0.25];
        let r = compute(&p, &p).unwrap();
        assert!(r.kl_pq.abs() < 1e-12);
        assert!(r.kl_qp.abs() < 1e-12);
        assert!(r.jensen_shannon.abs() < 1e-12);
        assert!(r.hellinger.abs() < 1e-12);
    }

    #[test]
    fn kl_is_asymmetric() {
        let p = vec![0.7, 0.3];
        let q = vec![0.5, 0.5];
        let r = compute(&p, &q).unwrap();
        assert!(
            (r.kl_pq - r.kl_qp).abs() > 1e-6,
            "KL should be asymmetric: D(P||Q) = {}, D(Q||P) = {}",
            r.kl_pq,
            r.kl_qp
        );
    }

    #[test]
    fn js_is_symmetric() {
        let p = vec![0.7, 0.3];
        let q = vec![0.5, 0.5];
        let r_pq = compute(&p, &q).unwrap();
        let r_qp = compute(&q, &p).unwrap();
        assert!(
            (r_pq.jensen_shannon - r_qp.jensen_shannon).abs() < 1e-12,
            "JS should be symmetric"
        );
    }

    #[test]
    fn auto_normalization_applied() {
        // Unnormalized counts; should give same results as normalized.
        let p = vec![70.0, 30.0];
        let q = vec![50.0, 50.0];
        let r_unnorm = compute(&p, &q).unwrap();
        let p_norm = vec![0.7, 0.3];
        let q_norm = vec![0.5, 0.5];
        let r_norm = compute(&p_norm, &q_norm).unwrap();
        assert!((r_unnorm.kl_pq - r_norm.kl_pq).abs() < 1e-12);
    }

    #[test]
    fn kl_infinite_when_support_mismatch() {
        // P puts mass where Q is zero → D_KL(P||Q) = ∞.
        let p = vec![0.5, 0.5];
        let q = vec![1.0, 0.0];
        let r = compute(&p, &q).unwrap();
        assert!(r.kl_pq.is_infinite());
    }

    #[test]
    fn hellinger_in_unit_range() {
        let p = vec![0.5, 0.5];
        let q = vec![0.1, 0.9];
        let r = compute(&p, &q).unwrap();
        assert!((0.0..=1.0).contains(&r.hellinger));
    }

    #[test]
    fn larger_difference_yields_larger_js() {
        let p = vec![0.5, 0.5];
        let near = vec![0.45, 0.55];
        let far = vec![0.05, 0.95];
        let r_near = compute(&p, &near).unwrap();
        let r_far = compute(&p, &far).unwrap();
        assert!(r_far.jensen_shannon > r_near.jensen_shannon);
    }
}
