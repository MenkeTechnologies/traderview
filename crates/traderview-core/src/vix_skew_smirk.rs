//! VIX Skew Smirk — implied-volatility skew gauge.
//!
//! Measures the asymmetry between out-of-the-money put IV and OTM call
//! IV at a fixed delta (typically 25-delta). A "smirk" is when the
//! put-side IV substantially exceeds call-side IV — the market is
//! pricing in more tail-risk downside than upside.
//!
//!   skew_t      = iv_25d_put_t - iv_25d_call_t
//!   skew_pct_t  = skew_t / iv_atm_t · 100
//!
//! Range typically 0..30 in normal markets; > 30 → stress regime,
//! < 0 → call-side fear ("upside crash" pricing, rare).
//!
//! Pure compute. Companion to `iv_skew_scanner`, `iv_term_structure`,
//! `vix_basis`, `unusual_options_activity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VixSkewReport {
    pub skew: Vec<Option<f64>>,
    pub skew_pct_of_atm: Vec<Option<f64>>,
    pub is_smirk: Vec<Option<bool>>,
    pub smirk_threshold: f64,
}

pub fn compute(
    iv_25d_put: &[f64],
    iv_25d_call: &[f64],
    iv_atm: &[f64],
    smirk_threshold: f64,
) -> VixSkewReport {
    let n = iv_25d_put.len();
    let mut report = VixSkewReport {
        skew: vec![None; n],
        skew_pct_of_atm: vec![None; n],
        is_smirk: vec![None; n],
        smirk_threshold,
    };
    if n == 0 || iv_25d_call.len() != n || iv_atm.len() != n
        || !smirk_threshold.is_finite() { return report; }
    if iv_25d_put.iter().chain(iv_25d_call.iter()).chain(iv_atm.iter())
        .any(|x| !x.is_finite() || *x <= 0.0) {
        return report;
    }
    for i in 0..n {
        let skew = iv_25d_put[i] - iv_25d_call[i];
        report.skew[i] = Some(skew);
        report.skew_pct_of_atm[i] = Some(skew / iv_atm[i] * 100.0);
        report.is_smirk[i] = Some(skew >= smirk_threshold);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatched_lengths_return_empty() {
        let p = vec![25.0_f64; 5];
        let c = vec![15.0_f64; 4];
        let a = vec![20.0_f64; 5];
        let r = compute(&p, &c, &a, 5.0);
        assert!(r.skew.iter().all(|x| x.is_none()));
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], &[], &[], 5.0);
        assert!(r.skew.is_empty());
    }

    #[test]
    fn nan_or_zero_returns_empty() {
        let p = vec![25.0_f64; 3];
        let mut c = vec![15.0_f64; 3];
        c[1] = f64::NAN;
        let a = vec![20.0_f64; 3];
        let r = compute(&p, &c, &a, 5.0);
        assert!(r.skew.iter().all(|x| x.is_none()));
    }

    #[test]
    fn classic_smirk_flagged() {
        // Put IV 25, call IV 15, ATM IV 20 → skew = 10, pct = 50%.
        let p = vec![25.0; 5];
        let c = vec![15.0; 5];
        let a = vec![20.0; 5];
        let r = compute(&p, &c, &a, 5.0);
        for i in 0..5 {
            assert!((r.skew[i].unwrap() - 10.0).abs() < 1e-9);
            assert!((r.skew_pct_of_atm[i].unwrap() - 50.0).abs() < 1e-9);
            assert!(r.is_smirk[i].unwrap());
        }
    }

    #[test]
    fn no_smirk_when_below_threshold() {
        let p = vec![22.0; 5];
        let c = vec![20.0; 5];
        let a = vec![21.0; 5];
        let r = compute(&p, &c, &a, 5.0);
        for i in 0..5 {
            assert!(!r.is_smirk[i].unwrap());
        }
    }

    #[test]
    fn upside_skew_yields_negative_value() {
        // Call IV > put IV → unusual; market pricing upside crash.
        let p = vec![15.0; 5];
        let c = vec![25.0; 5];
        let a = vec![20.0; 5];
        let r = compute(&p, &c, &a, 5.0);
        for i in 0..5 {
            assert!(r.skew[i].unwrap() < 0.0);
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let p = vec![22.0; 10];
        let c = vec![20.0; 10];
        let a = vec![21.0; 10];
        let r = compute(&p, &c, &a, 5.0);
        assert_eq!(r.skew.len(), 10);
        assert_eq!(r.is_smirk.len(), 10);
    }
}
