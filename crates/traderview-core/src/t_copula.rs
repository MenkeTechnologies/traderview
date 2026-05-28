//! Student-t Copula Fit — captures tail dependence missed by the
//! Gaussian copula.
//!
//! Like `gaussian_copula`, transforms each variable to uniform via
//! empirical CDF, then maps to t-distributed marginals via inverse
//! Student-t CDF before computing correlations:
//!
//!   U_i = rank(X_i) / (n + 1)
//!   T_i = t_ν⁻¹(U_i)                          (inverse t-CDF, dof ν)
//!   ρ̂  = sample correlation of T-matrix
//!
//! The degrees-of-freedom ν is supplied by the caller (typical financial
//! values: 3–8). For ν → ∞, the t copula converges to the Gaussian
//! copula; lower ν → stronger tail dependence.
//!
//! Pure compute. Companion to `gaussian_copula`, `tail_dependence`,
//! `realized_correlation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TCopulaReport {
    pub correlation_matrix: Vec<Vec<f64>>,
    pub degrees_of_freedom: f64,
    pub uniform_grades: Vec<Vec<f64>>,
    pub t_scores: Vec<Vec<f64>>,
    pub n_observations: usize,
    pub n_variables: usize,
}

pub fn fit(
    observations: &[Vec<f64>],
    degrees_of_freedom: f64,
) -> Option<TCopulaReport> {
    let n = observations.len();
    if n < 5 || !degrees_of_freedom.is_finite() || degrees_of_freedom <= 2.0 {
        return None;
    }
    let p = observations[0].len();
    if p < 2 { return None; }
    if observations.iter().any(|row| row.len() != p
        || row.iter().any(|x| !x.is_finite())) { return None; }
    let n_f = n as f64;
    let mut u = vec![vec![0.0_f64; p]; n];
    let mut t_scores = vec![vec![0.0_f64; p]; n];
    for j in 0..p {
        let col: Vec<f64> = (0..n).map(|i| observations[i][j]).collect();
        let ranks = rank_with_ties(&col);
        for i in 0..n {
            let grade = ranks[i] / (n_f + 1.0);
            u[i][j] = grade;
            t_scores[i][j] = student_t_inv_cdf(grade, degrees_of_freedom);
        }
    }
    let mut corr = vec![vec![0.0_f64; p]; p];
    let t_means: Vec<f64> = (0..p).map(|j| {
        t_scores.iter().map(|row| row[j]).sum::<f64>() / n_f
    }).collect();
    let t_stds: Vec<f64> = (0..p).map(|j| {
        let var: f64 = t_scores.iter().map(|row| (row[j] - t_means[j]).powi(2)).sum::<f64>()
            / (n_f - 1.0);
        var.max(1e-18).sqrt()
    }).collect();
    for j in 0..p {
        corr[j][j] = 1.0;
        for k in (j + 1)..p {
            let cov: f64 = t_scores.iter()
                .map(|row| (row[j] - t_means[j]) * (row[k] - t_means[k]))
                .sum::<f64>() / (n_f - 1.0);
            let r = (cov / (t_stds[j] * t_stds[k])).clamp(-1.0, 1.0);
            corr[j][k] = r;
            corr[k][j] = r;
        }
    }
    Some(TCopulaReport {
        correlation_matrix: corr,
        degrees_of_freedom,
        uniform_grades: u,
        t_scores,
        n_observations: n,
        n_variables: p,
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

/// Inverse Student-t CDF via Newton-Raphson on the CDF.
fn student_t_inv_cdf(p: f64, dof: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    let mut x = standard_normal_inv_cdf(p) * (dof / (dof - 2.0)).sqrt();
    for _ in 0..50 {
        let f = student_t_cdf(x, dof) - p;
        let pdf = student_t_pdf(x, dof);
        if pdf < 1e-18 { break; }
        let dx = f / pdf;
        x -= dx;
        if dx.abs() < 1e-10 { break; }
    }
    x
}

fn student_t_cdf(x: f64, dof: f64) -> f64 {
    // Series approximation via incomplete-beta: F(x) = 1 − ½ · I_{dof/(dof+x²)}(dof/2, 1/2).
    let z = dof / (dof + x * x);
    let i_z = incomplete_beta(dof / 2.0, 0.5, z);
    if x >= 0.0 { 1.0 - 0.5 * i_z } else { 0.5 * i_z }
}

fn student_t_pdf(x: f64, dof: f64) -> f64 {
    let coef = (lgamma((dof + 1.0) / 2.0) - lgamma(dof / 2.0)
        - 0.5 * (dof * std::f64::consts::PI).ln()).exp();
    coef * (1.0 + x * x / dof).powf(-(dof + 1.0) / 2.0)
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
    let mut c = 1.0_f64;
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

fn standard_normal_inv_cdf(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1, 2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2, 1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1, 2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1, 1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2, 6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3, -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838, -2.549_732_539_343_734,
         4.374_664_141_464_968, 2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
         7.784_695_709_041_462e-3, 3.224_671_290_700_398e-1,
         2.445_134_137_142_996, 3.754_408_661_907_416,
    ];
    let p_low = 0.02425;
    let p_high = 1.0 - p_low;
    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0]*q+C[1])*q+C[2])*q+C[3])*q+C[4])*q+C[5])
            / ((((D[0]*q+D[1])*q+D[2])*q+D[3])*q+1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0]*r+A[1])*r+A[2])*r+A[3])*r+A[4])*r+A[5]) * q
            / (((((B[0]*r+B[1])*r+B[2])*r+B[3])*r+B[4])*r+1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0]*q+C[1])*q+C[2])*q+C[3])*q+C[4])*q+C[5])
            / ((((D[0]*q+D[1])*q+D[2])*q+D[3])*q+1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_or_invalid_dof_returns_none() {
        let obs = vec![vec![1.0, 2.0]; 4];
        assert!(fit(&obs, 5.0).is_none());
        let obs2 = vec![vec![1.0, 2.0]; 10];
        assert!(fit(&obs2, 2.0).is_none());
        assert!(fit(&obs2, f64::NAN).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let obs = vec![
            vec![1.0, 2.0], vec![3.0, f64::NAN], vec![5.0, 6.0],
            vec![7.0, 8.0], vec![9.0, 10.0],
        ];
        assert!(fit(&obs, 5.0).is_none());
    }

    #[test]
    fn correlation_diagonal_one() {
        let obs: Vec<Vec<f64>> = (1..=20).map(|i| vec![i as f64, (i as f64).sin()]).collect();
        let r = fit(&obs, 5.0).unwrap();
        for j in 0..r.n_variables {
            assert!((r.correlation_matrix[j][j] - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn perfectly_monotone_yields_correlation_one() {
        let obs: Vec<Vec<f64>> = (1..=20).map(|i| vec![i as f64, (i as f64).exp()]).collect();
        let r = fit(&obs, 5.0).unwrap();
        // Rank-correlation between monotone variables is +1.
        assert!((r.correlation_matrix[0][1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn anti_monotone_yields_correlation_minus_one() {
        let obs: Vec<Vec<f64>> = (1..=20).map(|i| vec![i as f64, -(i as f64).exp()]).collect();
        let r = fit(&obs, 5.0).unwrap();
        assert!((r.correlation_matrix[0][1] + 1.0).abs() < 1e-9);
    }

    #[test]
    fn dof_recorded_in_report() {
        let obs: Vec<Vec<f64>> = (1..=20).map(|i| vec![i as f64, (i as f64).sin()]).collect();
        let r = fit(&obs, 7.0).unwrap();
        assert_eq!(r.degrees_of_freedom, 7.0);
    }

    #[test]
    fn uniform_grades_in_unit_range() {
        let obs: Vec<Vec<f64>> = (1..=20).map(|i| vec![i as f64, (i as f64 * 0.5).sin()]).collect();
        let r = fit(&obs, 5.0).unwrap();
        for row in &r.uniform_grades {
            for u in row {
                assert!(*u > 0.0 && *u < 1.0);
            }
        }
    }
}
