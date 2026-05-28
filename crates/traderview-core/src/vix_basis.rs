//! VIX Basis — front-month VX future minus spot VIX.
//!
//!   basis_t = vx_front_t - vix_spot_t
//!   basis_pct_t = basis_t / vix_spot_t · 100
//!
//! Interpretation:
//!   basis > 0 (contango) → futures pricing in more future vol than spot.
//!     Common in normal markets, signals carry cost for vol ETPs.
//!   basis < 0 (backwardation) → futures pricing in LESS future vol.
//!     Signals near-term spike in realized vol; historically marks
//!     stress regimes (financial crisis, COVID, etc.).
//!
//! Magnitude in vol points. Pure compute.
//! Companion to `vix_term_structure`, `vol_risk_premium`,
//! `noise_to_signal_ratio`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VixBasisReport {
    pub basis: Vec<Option<f64>>,
    pub basis_pct: Vec<Option<f64>>,
    pub in_backwardation: Vec<Option<bool>>,
}

pub fn compute(vix_spot: &[f64], vx_front: &[f64]) -> VixBasisReport {
    let n = vix_spot.len();
    let mut report = VixBasisReport {
        basis: vec![None; n],
        basis_pct: vec![None; n],
        in_backwardation: vec![None; n],
    };
    if n == 0 || vx_front.len() != n { return report; }
    if vix_spot.iter().chain(vx_front.iter()).any(|x| !x.is_finite() || *x <= 0.0) {
        return report;
    }
    for i in 0..n {
        let basis = vx_front[i] - vix_spot[i];
        report.basis[i] = Some(basis);
        report.basis_pct[i] = Some(basis / vix_spot[i] * 100.0);
        report.in_backwardation[i] = Some(basis < 0.0);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatched_lengths_return_empty() {
        let s = vec![15.0_f64; 10];
        let f = vec![15.5_f64; 9];
        let r = compute(&s, &f);
        assert!(r.basis.iter().all(|x| x.is_none()));
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], &[]);
        assert!(r.basis.is_empty());
    }

    #[test]
    fn nan_or_zero_returns_empty() {
        let s = vec![15.0_f64; 5];
        let mut f = vec![15.5_f64; 5];
        f[2] = f64::NAN;
        let r = compute(&s, &f);
        assert!(r.basis.iter().all(|x| x.is_none()));
        let mut s2 = vec![15.0_f64; 5];
        s2[2] = -1.0;
        let f2 = vec![15.5_f64; 5];
        let r2 = compute(&s2, &f2);
        assert!(r2.basis.iter().all(|x| x.is_none()));
    }

    #[test]
    fn contango_flagged_correctly() {
        let s = vec![15.0_f64; 5];
        let f = vec![16.5_f64; 5];
        let r = compute(&s, &f);
        for i in 0..5 {
            assert!((r.basis[i].unwrap() - 1.5).abs() < 1e-9);
            assert!((r.basis_pct[i].unwrap() - 10.0).abs() < 1e-9);
            assert!(!r.in_backwardation[i].unwrap());
        }
    }

    #[test]
    fn backwardation_flagged_correctly() {
        let s = vec![25.0_f64; 5];
        let f = vec![22.0_f64; 5];
        let r = compute(&s, &f);
        for i in 0..5 {
            assert!((r.basis[i].unwrap() + 3.0).abs() < 1e-9);
            assert!(r.in_backwardation[i].unwrap());
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let s = vec![15.0_f64; 10];
        let f = vec![15.5_f64; 10];
        let r = compute(&s, &f);
        assert_eq!(r.basis.len(), 10);
        assert_eq!(r.basis_pct.len(), 10);
        assert_eq!(r.in_backwardation.len(), 10);
    }
}
