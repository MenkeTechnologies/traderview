//! Factor Neutralization — orthogonalize a raw signal against one or
//! more factor exposures to isolate the alpha-bearing residual.
//!
//! For each name in a cross-section, run OLS of `signal_i` on
//! intercept + factor exposures `f_{i,1}, …, f_{i,K}`:
//!
//!   signal_i = α + Σ_k β_k · f_{i,k} + ε_i
//!
//! The neutralized signal is the residual ε_i. Factor regression
//! removes the part of the signal that is mechanically explained by
//! known risk factors (size, value, sector dummies, etc.), leaving the
//! "pure" alpha proxy.
//!
//! Distinct from `composite_factor_scoring` (which AGGREGATES factors
//! into a single score) — this REMOVES factor exposure from a separate
//! signal.
//!
//! Pure compute. Companion to `composite_factor_scoring`,
//! `information_coefficient`, `factor_models`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameInputs {
    pub symbol: String,
    pub raw_signal: f64,
    /// Aligned positionally with `factor_names`.
    pub factor_exposures: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeutralizedName {
    pub symbol: String,
    pub raw_signal: f64,
    pub fitted_signal: f64,
    pub neutralized_signal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactorNeutralizationReport {
    pub names: Vec<NeutralizedName>,
    pub factor_names: Vec<String>,
    pub alpha_intercept: f64,
    pub factor_loadings: Vec<f64>,
    pub r_squared: f64,
    pub n_names: usize,
}

pub fn neutralize(
    factor_names: &[String],
    inputs: &[NameInputs],
) -> Option<FactorNeutralizationReport> {
    let n = inputs.len();
    let k = factor_names.len();
    if n < k + 2 || k == 0 {
        return None;
    }
    if inputs.iter().any(|i| {
        i.factor_exposures.len() != k
            || !i.raw_signal.is_finite()
            || i.factor_exposures.iter().any(|x| !x.is_finite())
    }) {
        return None;
    }
    let p = k + 1;
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    let mut sum_y = 0.0_f64;
    let mut sum_y_sq = 0.0_f64;
    for input in inputs {
        let mut row = vec![1.0_f64];
        row.extend_from_slice(&input.factor_exposures);
        let y = input.raw_signal;
        sum_y += y;
        sum_y_sq += y * y;
        for j in 0..p {
            xty[j] += row[j] * y;
            for kk in 0..p {
                xtx[j][kk] += row[j] * row[kk];
            }
        }
    }
    let coef = solve_linear(&xtx, &xty)?;
    let n_f = n as f64;
    let y_mean = sum_y / n_f;
    let tss = sum_y_sq - n_f * y_mean.powi(2);
    let mut ssr = 0.0_f64;
    let mut names = Vec::with_capacity(n);
    for input in inputs {
        let mut row = vec![1.0_f64];
        row.extend_from_slice(&input.factor_exposures);
        let fitted: f64 = (0..p).map(|j| coef[j] * row[j]).sum();
        let resid = input.raw_signal - fitted;
        ssr += resid * resid;
        names.push(NeutralizedName {
            symbol: input.symbol.clone(),
            raw_signal: input.raw_signal,
            fitted_signal: fitted,
            neutralized_signal: resid,
        });
    }
    let r_sq = if tss > 1e-18 { 1.0 - ssr / tss } else { 0.0 };
    Some(FactorNeutralizationReport {
        names,
        factor_names: factor_names.to_vec(),
        alpha_intercept: coef[0],
        factor_loadings: coef[1..].to_vec(),
        r_squared: r_sq,
        n_names: n,
    })
}

fn solve_linear(m: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = m.len();
    if n == 0 || y.len() != n {
        return None;
    }
    let mut aug = vec![vec![0.0_f64; n + 1]; n];
    for (i, row) in aug.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate().take(n) {
            *slot = m[i][j];
        }
        row[n] = y[i];
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
            if aug[r][i].abs() > aug[pivot][i].abs() {
                pivot = r;
            }
        }
        if aug[pivot][i].abs() < 1e-18 {
            return None;
        }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() {
            *v /= div;
        }
        for r in 0..n {
            if r == i {
                continue;
            }
            let f = aug[r][i];
            if f == 0.0 {
                continue;
            }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() {
                *v -= f * pivot_row[j];
            }
        }
    }
    Some((0..n).map(|i| aug[i][n]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(sym: &str, sig: f64, fac: Vec<f64>) -> NameInputs {
        NameInputs {
            symbol: sym.into(),
            raw_signal: sig,
            factor_exposures: fac,
        }
    }

    #[test]
    fn too_few_names_returns_none() {
        let inputs = vec![n("X", 1.0, vec![1.0])];
        assert!(neutralize(&["beta".into()], &inputs).is_none());
    }

    #[test]
    fn no_factors_returns_none() {
        let inputs: Vec<_> = (0..10)
            .map(|i| n(&format!("S{i}"), i as f64, vec![]))
            .collect();
        assert!(neutralize(&[], &inputs).is_none());
    }

    #[test]
    fn nan_or_dim_mismatch_returns_none() {
        let inputs = vec![
            n("A", 1.0, vec![1.0, 2.0]),
            n("B", f64::NAN, vec![3.0, 4.0]),
            n("C", 2.0, vec![5.0, 6.0]),
            n("D", 3.0, vec![7.0, 8.0]),
        ];
        assert!(neutralize(&["f1".into(), "f2".into()], &inputs).is_none());
        let bad = vec![
            n("A", 1.0, vec![1.0]),
            n("B", 2.0, vec![1.0, 2.0]),
            n("C", 3.0, vec![1.0, 2.0]),
            n("D", 4.0, vec![1.0, 2.0]),
        ];
        assert!(neutralize(&["f1".into(), "f2".into()], &bad).is_none());
    }

    #[test]
    fn signal_fully_explained_by_factor_yields_zero_neutralized() {
        // signal_i = 2 · f_i → factor regression removes everything.
        let inputs: Vec<_> = (1..=20)
            .map(|i| n(&format!("S{i}"), 2.0 * i as f64, vec![i as f64]))
            .collect();
        let r = neutralize(&["beta".into()], &inputs).unwrap();
        for name in &r.names {
            assert!(
                name.neutralized_signal.abs() < 1e-9,
                "{}: neutralized should be 0, got {}",
                name.symbol,
                name.neutralized_signal
            );
        }
        assert!((r.r_squared - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pure_alpha_signal_survives_neutralization() {
        // Signal uncorrelated with factor → fitted ≈ mean, residual ≈ signal - mean.
        let inputs = vec![
            n("A", 1.0, vec![1.0]),
            n("B", 2.0, vec![1.0]),
            n("C", 3.0, vec![1.0]),
            n("D", 4.0, vec![1.0]),
            n("E", 5.0, vec![1.0]),
        ];
        // Constant factor (all 1.0) → singular X'X, returns None.
        assert!(neutralize(&["f1".into()], &inputs).is_none());
    }

    #[test]
    fn factor_loadings_match_known_coefficients() {
        // signal = 0.5 + 1.5·f1 − 0.7·f2
        let inputs: Vec<_> = (0..30)
            .map(|i| {
                let f1 = i as f64 / 10.0;
                let f2 = (i as f64 / 5.0).sin();
                let sig = 0.5 + 1.5 * f1 - 0.7 * f2;
                n(&format!("S{i}"), sig, vec![f1, f2])
            })
            .collect();
        let r = neutralize(&["f1".into(), "f2".into()], &inputs).unwrap();
        assert!((r.alpha_intercept - 0.5).abs() < 1e-6);
        assert!((r.factor_loadings[0] - 1.5).abs() < 1e-6);
        assert!((r.factor_loadings[1] + 0.7).abs() < 1e-6);
    }

    #[test]
    fn neutralized_sum_to_zero() {
        let inputs: Vec<_> = (0..20)
            .map(|i| n(&format!("S{i}"), (i as f64).sin(), vec![i as f64]))
            .collect();
        let r = neutralize(&["beta".into()], &inputs).unwrap();
        let sum: f64 = r.names.iter().map(|n| n.neutralized_signal).sum();
        assert!(
            sum.abs() < 1e-9,
            "with intercept, residuals sum to 0, got {}",
            sum
        );
    }

    #[test]
    fn n_names_reported() {
        let inputs: Vec<_> = (0..15)
            .map(|i| n(&format!("S{i}"), i as f64, vec![(i as f64).sin()]))
            .collect();
        let r = neutralize(&["beta".into()], &inputs).unwrap();
        assert_eq!(r.n_names, 15);
    }
}
