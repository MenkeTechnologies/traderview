//! Spearman Rank Correlation (Spearman 1904).
//!
//! Non-parametric correlation that measures monotonic association
//! between two variables. Computed as the Pearson correlation of the
//! ranks of the data:
//!
//!   ρ_S = Pearson(rank(X), rank(Y))
//!
//! Ties are resolved with mid-ranks (average of tied positions).
//!
//! Range [−1, +1]; +1 = perfect monotone increasing, −1 = perfect
//! monotone decreasing, 0 = no monotone association.
//!
//! Advantages over Pearson:
//!   - Robust to outliers
//!   - Captures non-linear monotone relationships (e.g. y = exp(x))
//!   - Distribution-free (no normality assumption)
//!
//! Significance test: under H0 (independence), with n > 10:
//!   t = ρ_S · √((n − 2) / (1 − ρ_S²)) ~ t(n − 2)
//!
//! Pure compute. Companion to `rank_correlation`, `realized_correlation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpearmanReport {
    pub rho: f64,
    pub t_statistic: f64,
    pub p_value: f64,
    pub n_observations: usize,
}

pub fn compute(x: &[f64], y: &[f64]) -> Option<SpearmanReport> {
    let n = x.len();
    if n < 3 || y.len() != n { return None; }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let rx = rank_with_ties(x);
    let ry = rank_with_ties(y);
    let n_f = n as f64;
    let rx_mean: f64 = rx.iter().sum::<f64>() / n_f;
    let ry_mean: f64 = ry.iter().sum::<f64>() / n_f;
    let mut sxy = 0.0_f64;
    let mut sxx = 0.0_f64;
    let mut syy = 0.0_f64;
    for i in 0..n {
        let dx = rx[i] - rx_mean;
        let dy = ry[i] - ry_mean;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 { return None; }
    let rho = sxy / (sxx * syy).sqrt();
    let rho = rho.clamp(-1.0, 1.0);
    // t-test for ρ_S != 0.
    let (t_stat, p_value) = if n > 10 && (1.0 - rho * rho) > 1e-12 {
        let t = rho * ((n - 2) as f64).sqrt() / (1.0 - rho * rho).sqrt();
        let p = student_t_two_sided_pvalue(t, (n - 2) as f64);
        (t, p)
    } else {
        (0.0, 1.0)
    };
    Some(SpearmanReport {
        rho,
        t_statistic: t_stat,
        p_value,
        n_observations: n,
    })
}

fn rank_with_ties(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|a, b| x[*a].partial_cmp(&x[*b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0_f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && x[idx[j + 1]] == x[idx[i]] { j += 1; }
        let mid_rank = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j { ranks[idx[k]] = mid_rank; }
        i = j + 1;
    }
    ranks
}

/// Two-sided p-value from a Student-t statistic. Series approximation
/// of the incomplete-beta function for moderate dof; accurate to ~1e-6
/// for dof > 5.
fn student_t_two_sided_pvalue(t: f64, dof: f64) -> f64 {
    if !t.is_finite() || dof <= 0.0 { return 1.0; }
    let x = dof / (dof + t * t);
    let p = incomplete_beta(dof / 2.0, 0.5, x);
    p.clamp(0.0, 1.0)
}

fn incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    if x >= 1.0 { return 1.0; }
    let bt = (lgamma(a + b) - lgamma(a) - lgamma(b)
        + a * x.ln() + b * (1.0 - x).ln()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        bt * cont_frac(a, b, x) / a
    } else {
        1.0 - bt * cont_frac(b, a, 1.0 - x) / b
    }
}

fn cont_frac(a: f64, b: f64, x: f64) -> f64 {
    let max_iter = 200;
    let eps = 3e-7;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < 1e-30 { d = 1e-30; }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=max_iter {
        let m_f = m as f64;
        let m2 = 2.0 * m_f;
        let aa = m_f * (b - m_f) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 { d = 1e-30; }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 { c = 1e-30; }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m_f) * (qab + m_f) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 { d = 1e-30; }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 { c = 1e-30; }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < eps { return h; }
    }
    h
}

/// Stirling-Lanczos log-Gamma. Free function (not a method on f64) to
/// avoid the upcoming stable-std `f64::ln_gamma` ambiguity.
fn lgamma(z: f64) -> f64 {
    let g = 7.0;
    let p = [
        0.999_999_999_999_81,
        676.520_368_121_885_1,
        -1_259.139_216_722_403,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_72,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_312e-7,
    ];
    if z < 0.5 {
        std::f64::consts::PI.ln()
            - (std::f64::consts::PI * z).sin().abs().ln()
            - lgamma(1.0 - z)
    } else {
        let zm = z - 1.0;
        let mut x = p[0];
        for (i, pi) in p.iter().enumerate().skip(1) {
            x += pi / (zm + i as f64);
        }
        let t = zm + g + 0.5;
        0.5 * (2.0 * std::f64::consts::PI).ln()
            + (zm + 0.5) * t.ln() - t + x.ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[1.0, 2.0], &[3.0, 4.0]).is_none());
    }

    #[test]
    fn mismatched_lengths_return_none() {
        assert!(compute(&[1.0, 2.0, 3.0], &[1.0, 2.0]).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let x = vec![1.0, f64::NAN, 3.0, 4.0];
        let y = vec![1.0, 2.0, 3.0, 4.0];
        assert!(compute(&x, &y).is_none());
    }

    #[test]
    fn flat_input_returns_none() {
        let x = vec![1.0; 10];
        let y: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        assert!(compute(&x, &y).is_none());
    }

    #[test]
    fn monotone_increasing_yields_rho_one() {
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let y: Vec<f64> = (1..=20).map(|i| (i as f64).exp()).collect();
        let r = compute(&x, &y).unwrap();
        assert!((r.rho - 1.0).abs() < 1e-12, "monotone: ρ should be 1, got {}", r.rho);
    }

    #[test]
    fn monotone_decreasing_yields_rho_minus_one() {
        let x: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let y: Vec<f64> = (1..=20).map(|i| -(i as f64)).collect();
        let r = compute(&x, &y).unwrap();
        assert!((r.rho + 1.0).abs() < 1e-12);
    }

    #[test]
    fn ties_handled_via_midranks() {
        // Some ties in y; check that no panic + result is a valid number.
        let x: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let y = vec![1.0, 2.0, 2.0, 3.0, 4.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let r = compute(&x, &y).unwrap();
        assert!(r.rho.is_finite());
        assert!(r.rho > 0.9, "near-monotone with ties: ρ should be > 0.9, got {}", r.rho);
    }

    #[test]
    fn p_value_in_unit_range() {
        let x: Vec<f64> = (0..30).map(|i| (i as f64 * 0.1).sin()).collect();
        let y: Vec<f64> = (0..30).map(|i| (i as f64 * 0.1).cos()).collect();
        let r = compute(&x, &y).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }
}
