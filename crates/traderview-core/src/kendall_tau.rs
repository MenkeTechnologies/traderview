//! Kendall's Tau-b — rank correlation coefficient that handles ties.
//!
//!   τ_b = (n_c - n_d) / sqrt((n_0 - n_1) · (n_0 - n_2))
//!
//! where:
//!   n_c = concordant pairs (sign(x_i - x_j) == sign(y_i - y_j))
//!   n_d = discordant pairs (signs disagree)
//!   n_0 = n·(n-1)/2 (total pair count)
//!   n_1 = Σ_k t_k·(t_k-1)/2 over tie-groups in x
//!   n_2 = Σ_k u_k·(u_k-1)/2 over tie-groups in y
//!
//! τ_b ∈ [-1, +1]. +1 = perfect concordance, -1 = perfect discordance.
//! Unlike Pearson, makes no distributional assumptions and is robust
//! to monotone transformations.
//!
//! Naive O(n²) implementation — fine for n in the thousands. For
//! n > 10k use Knight's O(n log n) algorithm (out of scope).
//!
//! Pure compute. Companion to `spearman_correlation`,
//! `pearson_correlation`, `distance_correlation`.

#[derive(Debug)]
pub struct Report {
    pub tau_b: f64,
    pub concordant: u64,
    pub discordant: u64,
    pub ties_x: u64,
    pub ties_y: u64,
    pub n: usize,
}

pub fn compute(x: &[f64], y: &[f64]) -> Option<Report> {
    let n = x.len();
    if n < 2 || y.len() != n { return None; }
    if x.iter().chain(y.iter()).any(|v| !v.is_finite()) { return None; }
    let mut n_c: u64 = 0;
    let mut n_d: u64 = 0;
    let mut t_x: u64 = 0;
    let mut t_y: u64 = 0;
    for i in 0..n {
        for j in i + 1..n {
            let dx = x[i] - x[j];
            let dy = y[i] - y[j];
            let tied_x = dx == 0.0;
            let tied_y = dy == 0.0;
            if tied_x && tied_y { /* contributes to neither */ }
            else if tied_x { t_x += 1; }
            else if tied_y { t_y += 1; }
            else if dx.signum() == dy.signum() { n_c += 1; }
            else { n_d += 1; }
        }
    }
    let n_pairs = (n as u64) * (n as u64 - 1) / 2;
    let denom = ((n_pairs - t_x) as f64 * (n_pairs - t_y) as f64).sqrt();
    let tau_b = if denom > 0.0 {
        (n_c as f64 - n_d as f64) / denom
    } else {
        0.0
    };
    Some(Report {
        tau_b,
        concordant: n_c,
        discordant: n_d,
        ties_x: t_x,
        ties_y: t_y,
        n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[], &[]).is_none());
        assert!(compute(&[1.0], &[1.0]).is_none());
        assert!(compute(&[1.0, 2.0], &[1.0]).is_none());
        assert!(compute(&[f64::NAN, 1.0], &[1.0, 2.0]).is_none());
    }

    #[test]
    fn identical_ranks_yield_tau_one() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = compute(&x, &x).unwrap();
        assert!((r.tau_b - 1.0).abs() < 1e-12);
    }

    #[test]
    fn reversed_ranks_yield_tau_minus_one() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let r = compute(&x, &y).unwrap();
        assert!((r.tau_b + 1.0).abs() < 1e-12);
    }

    #[test]
    fn random_uncorrelated_yields_tau_near_zero() {
        // Construct symmetric arrangement with equal concordant/discordant.
        let x: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let y = vec![3.0, 1.0, 4.0, 1.5, 5.0, 9.0, 2.0, 6.0, 5.5, 3.5];
        let r = compute(&x, &y).unwrap();
        assert!(r.tau_b.abs() < 0.6);
    }

    #[test]
    fn tau_in_unit_interval() {
        let x = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        let y = vec![2.0, 4.0, 1.0, 5.0, 3.0];
        let r = compute(&x, &y).unwrap();
        assert!(r.tau_b.abs() <= 1.0);
    }

    #[test]
    fn ties_counted_correctly() {
        // x ties: pairs (0,1) → t_x = 1. y ties: pairs (2,3) → t_y = 1.
        let x = vec![1.0, 1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0, 3.0, 3.0];
        let r = compute(&x, &y).unwrap();
        assert_eq!(r.ties_x, 1);
        assert_eq!(r.ties_y, 1);
    }

    #[test]
    fn known_small_case_concordance() {
        // x=[1,2,3], y=[1,3,2]
        // pairs:
        //   (0,1): dx=-1, dy=-2, sign match → concordant
        //   (0,2): dx=-2, dy=-1, sign match → concordant
        //   (1,2): dx=-1, dy=+1, sign mismatch → discordant
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 3.0, 2.0];
        let r = compute(&x, &y).unwrap();
        assert_eq!(r.concordant, 2);
        assert_eq!(r.discordant, 1);
        // τ_b = (2-1)/3 = 1/3.
        assert!((r.tau_b - 1.0 / 3.0).abs() < 1e-12);
    }
}
