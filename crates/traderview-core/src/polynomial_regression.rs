//! Polynomial Regression — OLS fit of y = β_0 + β_1·x + β_2·x² + … +
//! β_d·x^d via the design-matrix approach.
//!
//! For numerical stability:
//!   1. Standardize x (mean-zero, unit-variance) before raising to
//!      powers; otherwise x^d for large d dominates the normal
//!      equations.
//!   2. Solve (XᵀX) β = Xᵀy via Gauss-Jordan elimination.
//!   3. De-standardize coefficients back to original-x space using
//!      the binomial expansion of (x - μ)^k.
//!
//! Returns the coefficients in original-x space (callers can evaluate
//! directly: y = Σ β_k · x^k), the fitted ŷ vector, and R².
//!
//! Pure compute. Companion to `linear_regression_slope`,
//! `ridge_regression`, `belkhayate_timing`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub coefficients: Vec<f64>,        // length = degree + 1
    pub fitted: Vec<f64>,
    pub r_squared: f64,
}

pub fn compute(x: &[f64], y: &[f64], degree: usize) -> Option<Report> {
    let n = x.len();
    if degree == 0 || n < degree + 2 || y.len() != n { return None; }
    if x.iter().chain(y.iter()).any(|v| !v.is_finite()) { return None; }
    // Standardize x for conditioning.
    let mean_x: f64 = x.iter().sum::<f64>() / n as f64;
    let var_x: f64 = x.iter().map(|v| (v - mean_x).powi(2)).sum::<f64>() / n as f64;
    let std_x = var_x.max(1e-18).sqrt();
    let xs: Vec<f64> = x.iter().map(|v| (v - mean_x) / std_x).collect();
    // Build design matrix in standardized x.
    let cols = degree + 1;
    let mut xtx = vec![vec![0.0_f64; cols]; cols];
    let mut xty = vec![0.0_f64; cols];
    for i in 0..n {
        let mut row = vec![1.0_f64; cols];
        for k in 1..cols { row[k] = row[k - 1] * xs[i]; }
        for j in 0..cols {
            for k in j..cols {
                xtx[j][k] += row[j] * row[k];
            }
            xty[j] += row[j] * y[i];
        }
    }
    for j in 0..cols {
        for k in 0..j {
            xtx[j][k] = xtx[k][j];
        }
    }
    let beta_std = solve_linear_system(xtx, xty)?;
    // De-standardize: ŷ(x) = Σ_k β_std[k] · ((x - μ) / σ)^k
    //                       = Σ_k β_std[k] · σ^-k · (x - μ)^k
    //                       = Σ_k β_std[k] · σ^-k · Σ_j C(k,j) · x^j · (-μ)^{k-j}
    // → coefficient on x^j = Σ_{k>=j} β_std[k] · σ^-k · C(k,j) · (-μ)^{k-j}
    let mut coefficients = vec![0.0_f64; cols];
    for j in 0..cols {
        let mut s = 0.0;
        for k in j..cols {
            let c_kj = choose(k, j);
            let sign = if (k - j).is_multiple_of(2) { 1.0 } else { -1.0 };
            s += beta_std[k] * std_x.powi(-(k as i32)) * c_kj as f64
                 * sign * mean_x.powi((k - j) as i32);
        }
        coefficients[j] = s;
    }
    // Evaluate ŷ in original-x space.
    let fitted: Vec<f64> = x.iter().map(|xi| {
        let mut p = 1.0_f64;
        let mut sum = 0.0;
        for &c in &coefficients {
            sum += c * p;
            p *= xi;
        }
        sum
    }).collect();
    let y_mean: f64 = y.iter().sum::<f64>() / n as f64;
    let sst: f64 = y.iter().map(|a| (a - y_mean).powi(2)).sum();
    let sse: f64 = y.iter().zip(fitted.iter()).map(|(a, b)| (a - b).powi(2)).sum();
    let r_squared = if sst > 1e-18 { 1.0 - sse / sst } else { 0.0 };
    Some(Report { coefficients, fitted, r_squared })
}

fn choose(n: usize, k: usize) -> u64 {
    if k > n { return 0; }
    let k = k.min(n - k);
    let mut r = 1_u64;
    for i in 0..k {
        r = r * (n as u64 - i as u64) / (i as u64 + 1);
    }
    r
}

fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for i in 0..n {
        let mut pivot = i;
        let mut pmax = a[i][i].abs();
        for k in i + 1..n {
            if a[k][i].abs() > pmax { pmax = a[k][i].abs(); pivot = k; }
        }
        if pmax < 1e-15 { return None; }
        a.swap(i, pivot);
        b.swap(i, pivot);
        let pv = a[i][i];
        for j in 0..n { a[i][j] /= pv; }
        b[i] /= pv;
        for k in 0..n {
            if k != i && a[k][i].abs() > 0.0 {
                let factor = a[k][i];
                for j in 0..n { a[k][j] -= factor * a[i][j]; }
                b[k] -= factor * b[i];
            }
        }
    }
    Some(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y = vec![1.0_f64; 10];
        assert!(compute(&x, &y, 0).is_none());
        assert!(compute(&x[..3], &y[..3], 5).is_none());
        assert!(compute(&x, &y[..5], 2).is_none());
        let mut bad = y.clone();
        bad[0] = f64::NAN;
        assert!(compute(&x, &bad, 2).is_none());
    }

    #[test]
    fn linear_function_recovered_with_degree_1() {
        // y = 3·x + 5 → coefficients = [5, 3].
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 3.0 * xi + 5.0).collect();
        let r = compute(&x, &y, 1).unwrap();
        assert!((r.coefficients[0] - 5.0).abs() < 1e-6);
        assert!((r.coefficients[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn quadratic_function_recovered_with_degree_2() {
        // y = 2·x² - x + 7.
        let x: Vec<f64> = (-10_i32..=10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi * xi - xi + 7.0).collect();
        let r = compute(&x, &y, 2).unwrap();
        assert!((r.coefficients[0] - 7.0).abs() < 1e-6);
        assert!((r.coefficients[1] + 1.0).abs() < 1e-6);
        assert!((r.coefficients[2] - 2.0).abs() < 1e-6);
        assert!((r.r_squared - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cubic_function_recovered_with_degree_3() {
        let x: Vec<f64> = (-15_i32..=15).map(|i| i as f64 / 2.0).collect();
        let y: Vec<f64> = x.iter().map(|xi| 0.5 * xi * xi * xi - 2.0 * xi + 1.0).collect();
        let r = compute(&x, &y, 3).unwrap();
        assert!((r.coefficients[0] - 1.0).abs() < 1e-4);
        assert!((r.coefficients[1] + 2.0).abs() < 1e-4);
        assert!(r.coefficients[2].abs() < 1e-4);
        assert!((r.coefficients[3] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn fitted_length_matches_input() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| xi * xi).collect();
        let r = compute(&x, &y, 2).unwrap();
        assert_eq!(r.fitted.len(), 20);
    }

    #[test]
    fn r_squared_near_one_for_exact_fit() {
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| xi * xi - 3.0 * xi + 5.0).collect();
        let r = compute(&x, &y, 2).unwrap();
        assert!((r.r_squared - 1.0).abs() < 1e-9);
    }

    #[test]
    fn coefficient_count_equals_degree_plus_one() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y = vec![1.0_f64; 20];
        let r = compute(&x, &y, 4).unwrap();
        assert_eq!(r.coefficients.len(), 5);
    }
}
