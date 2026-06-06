//! Dynamic Time Warping — elastic distance between two time series that
//! tolerates non-linear time-axis stretches and compressions.
//!
//! Builds an n×m cost matrix D where:
//!   `D[i][j] = |a_i - b_j| + min(D[i-1][j], D[i][j-1], D[i-1][j-1])`
//!
//! The bottom-right cell `D[n-1][m-1]` is the DTW distance. The optimal
//! warping path is recovered by backtracking through the argmin
//! predecessors.
//!
//! Optional Sakoe-Chiba band radius `r` restricts |i - j| ≤ r to bound
//! warping flexibility and reduce O(nm) cost to O(n·r). `r = 0` means
//! no warping (Euclidean distance on aligned series); larger `r` allows
//! more elastic matching. Set `r = max(n, m)` for unconstrained DTW.
//!
//! Useful for pattern matching: search a query pattern against many
//! historical windows, rank by ascending DTW distance.
//!
//! Pure compute. Companion to `pearson_correlation`,
//! `spearman_correlation`, `distance_correlation`.

#[derive(Debug)]
pub struct Report {
    pub distance: f64,
    pub path: Vec<(usize, usize)>,
}

pub fn compute(a: &[f64], b: &[f64], band_radius: usize) -> Option<Report> {
    let n = a.len();
    let m = b.len();
    if n == 0 || m == 0 {
        return None;
    }
    if a.iter().chain(b.iter()).any(|x| !x.is_finite()) {
        return None;
    }
    let r = band_radius.max(n.abs_diff(m));
    let inf = f64::INFINITY;
    let mut d = vec![vec![inf; m]; n];
    d[0][0] = (a[0] - b[0]).abs();
    for i in 0..n {
        let j_lo = i.saturating_sub(r);
        let j_hi = (i + r + 1).min(m);
        for j in j_lo..j_hi {
            let cost = (a[i] - b[j]).abs();
            let prev = if i == 0 && j == 0 {
                0.0
            } else {
                let mut m1 = inf;
                if i > 0 {
                    m1 = m1.min(d[i - 1][j]);
                }
                if j > 0 {
                    m1 = m1.min(d[i][j - 1]);
                }
                if i > 0 && j > 0 {
                    m1 = m1.min(d[i - 1][j - 1]);
                }
                m1
            };
            d[i][j] = cost + prev;
        }
    }
    if !d[n - 1][m - 1].is_finite() {
        return None;
    }
    let distance = d[n - 1][m - 1];
    // Backtrack.
    let mut path = Vec::new();
    let mut i = n - 1;
    let mut j = m - 1;
    path.push((i, j));
    while i > 0 || j > 0 {
        let up = if i > 0 { d[i - 1][j] } else { inf };
        let left = if j > 0 { d[i][j - 1] } else { inf };
        let diag = if i > 0 && j > 0 { d[i - 1][j - 1] } else { inf };
        if diag <= up && diag <= left {
            i -= 1;
            j -= 1;
        } else if up <= left {
            i -= 1;
        } else {
            j -= 1;
        }
        path.push((i, j));
    }
    path.reverse();
    Some(Report { distance, path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let a = vec![1.0_f64; 5];
        assert!(compute(&[], &a, 0).is_none());
        assert!(compute(&a, &[], 0).is_none());
        let mut a_nan = a.clone();
        a_nan[0] = f64::NAN;
        assert!(compute(&a_nan, &a, 0).is_none());
    }

    #[test]
    fn identical_series_yield_zero_distance() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = compute(&a, &a, 5).unwrap();
        assert!(r.distance.abs() < 1e-12);
        assert_eq!(r.path.len(), 5);
    }

    #[test]
    fn shifted_series_dtw_below_euclidean() {
        // b is a 1-step delayed copy of a; DTW should align ≈ perfectly.
        let a = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![0.0, 0.0, 1.0, 2.0, 3.0, 4.0];
        let r_dtw = compute(&a, &b, 3).unwrap();
        let euclid: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
        assert!(r_dtw.distance < euclid);
    }

    #[test]
    fn band_zero_acts_as_euclidean_for_equal_length() {
        // r = 0 with same length n=m enforces diagonal path → Manhattan.
        let a = vec![1.0, 3.0, 5.0];
        let b = vec![2.0, 2.0, 6.0];
        let r = compute(&a, &b, 0).unwrap();
        // |1-2| + |3-2| + |5-6| = 3.
        assert!((r.distance - 3.0).abs() < 1e-9);
    }

    #[test]
    fn distance_is_symmetric() {
        let a = vec![1.0, 4.0, 2.0, 5.0, 3.0];
        let b = vec![2.0, 1.0, 4.0, 3.0, 5.0];
        let r1 = compute(&a, &b, 5).unwrap();
        let r2 = compute(&b, &a, 5).unwrap();
        assert!((r1.distance - r2.distance).abs() < 1e-12);
    }

    #[test]
    fn warping_path_starts_and_ends_at_corners() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = compute(&a, &b, 5).unwrap();
        assert_eq!(r.path[0], (0, 0));
        assert_eq!(*r.path.last().unwrap(), (3, 4));
    }

    #[test]
    fn distance_non_negative() {
        let a = vec![1.0, -1.0, 2.0, -2.0, 3.0];
        let b = vec![3.0, 1.0, -1.0, 2.0, -2.0];
        let r = compute(&a, &b, 5).unwrap();
        assert!(r.distance >= 0.0);
    }
}
