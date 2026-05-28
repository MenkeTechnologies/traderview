//! Kupiec (1995) Proportion-of-Failures (POF) test — VaR model
//! backtest based on coverage of exceedances.
//!
//! Given a VaR forecast at level α (e.g. α=0.05) and a series of
//! realized returns, count "exceedances" where loss > VaR. Under the
//! null hypothesis (the model is correctly calibrated), the rate of
//! exceedances should equal α.
//!
//!   LR_POF = −2 · ln[ ((1−α)^(n−x) · α^x) / ((1−x/n)^(n−x) · (x/n)^x) ]
//!
//! Under H₀, LR_POF ~ χ²(1). Reject when LR_POF > 3.841 (5% level).
//!
//! Pure compute. Returns the exceedance count, observed rate, expected
//! rate, LR statistic, and a coarse significance bucket comparing
//! against χ²(1) critical values.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Significance { Pct1, Pct5, Pct10, NotRejected }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KupiecReport {
    pub n_observations: usize,
    pub n_exceedances: usize,
    pub observed_rate: f64,
    pub expected_rate: f64,
    pub likelihood_ratio: f64,
    pub significance: Significance,
}

pub fn test(
    realized_returns: &[f64],
    var_forecasts: &[f64],
    alpha: f64,
) -> Option<KupiecReport> {
    let n = realized_returns.len();
    if var_forecasts.len() != n
        || n < 10
        || !alpha.is_finite() || !(0.0..1.0).contains(&alpha) || alpha == 0.0
    {
        return None;
    }
    let mut valid_n = 0_usize;
    let mut exceedances = 0_usize;
    for i in 0..n {
        if !realized_returns[i].is_finite() || !var_forecasts[i].is_finite() { continue; }
        valid_n += 1;
        // Exceedance: loss > VaR (where VaR is a positive number representing
        // a loss magnitude). Loss = −return.
        let loss = -realized_returns[i];
        if loss > var_forecasts[i] {
            exceedances += 1;
        }
    }
    if valid_n < 10 { return None; }
    let n_f = valid_n as f64;
    let x = exceedances as f64;
    let observed_rate = x / n_f;
    // Avoid log(0) — if x == 0 or x == n, use the closed-form limit.
    let log_ratio = if x == 0.0 {
        // L(model) = (1−α)^n, L(unrestricted) = 1 → ratio = (1−α)^n
        -2.0 * ((n_f) * (1.0 - alpha).ln())
    } else if x == n_f {
        -2.0 * ((n_f) * alpha.ln())
    } else {
        let log_model = (n_f - x) * (1.0 - alpha).ln() + x * alpha.ln();
        let log_unrestricted = (n_f - x) * (1.0 - observed_rate).ln()
            + x * observed_rate.ln();
        -2.0 * (log_model - log_unrestricted)
    };
    if !log_ratio.is_finite() || log_ratio < 0.0 { return None; }
    // χ²(1) critical values: 6.635 (1%), 3.841 (5%), 2.706 (10%).
    let sig = if log_ratio > 6.635 { Significance::Pct1 }
        else if log_ratio > 3.841 { Significance::Pct5 }
        else if log_ratio > 2.706 { Significance::Pct10 }
        else { Significance::NotRejected };
    Some(KupiecReport {
        n_observations: valid_n,
        n_exceedances: exceedances,
        observed_rate,
        expected_rate: alpha,
        likelihood_ratio: log_ratio,
        significance: sig,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        assert!(test(&[0.01; 50], &[0.05; 25], 0.05).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[0.01; 5], &[0.05; 5], 0.05).is_none());
    }

    #[test]
    fn invalid_alpha_returns_none() {
        let r = vec![0.01; 50];
        let v = vec![0.05; 50];
        assert!(test(&r, &v, 0.0).is_none());
        assert!(test(&r, &v, 1.0).is_none());
        assert!(test(&r, &v, f64::NAN).is_none());
    }

    #[test]
    fn well_calibrated_model_not_rejected() {
        // VaR at 5%; design returns where 5% are exceedances.
        let n = 200;
        let mut returns = vec![-0.01_f64; n];
        // First 10 are losses bigger than the VaR threshold.
        for slot in returns.iter_mut().take(10) { *slot = -0.10; }
        let vars = vec![0.05_f64; n];
        let r = test(&returns, &vars, 0.05).unwrap();
        // Observed rate = 10/200 = 5% — exact calibration → LR ≈ 0 → not rejected.
        assert!(matches!(r.significance, Significance::NotRejected));
        assert_eq!(r.n_exceedances, 10);
    }

    #[test]
    fn under_estimating_var_rejected_at_5_percent() {
        // VaR set at 1% but actual loss exceedances are 10% (way too many).
        let n = 200;
        let mut returns = vec![-0.001_f64; n];
        for slot in returns.iter_mut().take(20) { *slot = -0.05; }    // 10% exceedances
        let vars = vec![0.01_f64; n];
        let r = test(&returns, &vars, 0.01).unwrap();
        // 20 exceedances vs 2 expected → strong rejection.
        assert!(matches!(r.significance, Significance::Pct1));
        assert_eq!(r.n_exceedances, 20);
    }

    #[test]
    fn zero_exceedances_handled() {
        let returns = vec![-0.001_f64; 100];
        let vars = vec![0.10_f64; 100];
        let r = test(&returns, &vars, 0.05).unwrap();
        assert_eq!(r.n_exceedances, 0);
        assert!(r.likelihood_ratio >= 0.0);
    }

    #[test]
    fn all_exceedances_handled() {
        let returns = vec![-0.20_f64; 100];
        let vars = vec![0.05_f64; 100];
        let r = test(&returns, &vars, 0.05).unwrap();
        assert_eq!(r.n_exceedances, 100);
        assert!(r.likelihood_ratio > 0.0);
    }

    #[test]
    fn nan_pairs_skipped_safely() {
        let mut returns = vec![-0.01_f64; 50];
        let mut vars = vec![0.05_f64; 50];
        returns[10] = f64::NAN;
        vars[20] = f64::NAN;
        let r = test(&returns, &vars, 0.05).unwrap();
        // 48 valid pairs.
        assert_eq!(r.n_observations, 48);
    }
}
