//! Shared derivative-free optimizer — the Nelder-Mead simplex that
//! gjr_garch originally carried inline, generalized to any dimension
//! and objective closure. Same coefficients and update order
//! (reflection α=1, expansion γ=2, contraction ρ=0.5, shrink σ=0.5;
//! initial simplex = start + `step` on each axis), so callers that
//! migrated from the inline version keep identical numerics.

/// Minimize `f` from `start`; returns (best_point, best_value).
pub fn nelder_mead<F: Fn(&[f64]) -> f64>(
    start: &[f64],
    step: f64,
    max_iter: usize,
    f: F,
) -> (Vec<f64>, f64) {
    let n = start.len();
    if n == 0 {
        return (Vec::new(), f(start));
    }
    let alpha = 1.0;
    let gamma_nm = 2.0;
    let rho = 0.5;
    let sigma = 0.5;
    let mut simplex: Vec<Vec<f64>> = Vec::with_capacity(n + 1);
    simplex.push(start.to_vec());
    for i in 0..n {
        let mut p = start.to_vec();
        p[i] += step;
        simplex.push(p);
    }
    let mut values: Vec<f64> = simplex.iter().map(|p| f(p)).collect();
    for _ in 0..max_iter {
        let mut idx: Vec<usize> = (0..=n).collect();
        idx.sort_by(|a, b| {
            values[*a]
                .partial_cmp(&values[*b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best = idx[0];
        let worst = idx[n];
        let second_worst = idx[n - 1];
        let mut centroid = vec![0.0_f64; n];
        for i in &idx[..n] {
            for j in 0..n {
                centroid[j] += simplex[*i][j];
            }
        }
        for c in centroid.iter_mut() {
            *c /= n as f64;
        }
        let mut reflected = vec![0.0_f64; n];
        for j in 0..n {
            reflected[j] = centroid[j] + alpha * (centroid[j] - simplex[worst][j]);
        }
        let v_refl = f(&reflected);
        if v_refl < values[second_worst] && v_refl >= values[best] {
            simplex[worst] = reflected;
            values[worst] = v_refl;
            continue;
        }
        if v_refl < values[best] {
            let mut expanded = vec![0.0_f64; n];
            for j in 0..n {
                expanded[j] = centroid[j] + gamma_nm * (reflected[j] - centroid[j]);
            }
            let v_exp = f(&expanded);
            if v_exp < v_refl {
                simplex[worst] = expanded;
                values[worst] = v_exp;
            } else {
                simplex[worst] = reflected;
                values[worst] = v_refl;
            }
            continue;
        }
        let mut contracted = vec![0.0_f64; n];
        for j in 0..n {
            contracted[j] = centroid[j] + rho * (simplex[worst][j] - centroid[j]);
        }
        let v_con = f(&contracted);
        if v_con < values[worst] {
            simplex[worst] = contracted;
            values[worst] = v_con;
            continue;
        }
        let best = simplex[idx[0]].clone();
        for i in 1..=n {
            for (x, &b) in simplex[idx[i]].iter_mut().zip(&best) {
                *x = b + sigma * (*x - b);
            }
            values[idx[i]] = f(&simplex[idx[i]]);
        }
    }
    let best_idx = values
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    (simplex[best_idx].clone(), values[best_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimizes_a_shifted_quadratic() {
        let (p, v) = nelder_mead(&[0.0, 0.0], 0.5, 500, |x| {
            (x[0] - 3.0).powi(2) + (x[1] + 1.5).powi(2)
        });
        assert!((p[0] - 3.0).abs() < 1e-4, "{p:?}");
        assert!((p[1] + 1.5).abs() < 1e-4, "{p:?}");
        assert!(v < 1e-8);
    }

    #[test]
    fn minimizes_rosenbrock() {
        let (p, _) = nelder_mead(&[-1.0, 1.0], 0.5, 4000, |x| {
            (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2)
        });
        assert!((p[0] - 1.0).abs() < 1e-2, "{p:?}");
        assert!((p[1] - 1.0).abs() < 1e-2, "{p:?}");
    }

    #[test]
    fn zero_dimension_is_a_no_op() {
        let (p, v) = nelder_mead(&[], 0.1, 10, |_| 42.0);
        assert!(p.is_empty());
        assert_eq!(v, 42.0);
    }
}
