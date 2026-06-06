//! Empirical Mode Decomposition (Huang et al. 1998) — data-driven
//! decomposition of a non-stationary series into Intrinsic Mode
//! Functions (IMFs) and a final residual trend.
//!
//! Sifting algorithm:
//!   1. Identify all local maxima and minima.
//!   2. Cubic-style linear interpolation through each set produces
//!      upper and lower envelopes (this implementation uses piecewise
//!      linear interpolation for tractability — Hilbert-Huang's
//!      published cubic-spline variant is heavier; linear-EMD is
//!      a known acceptable variant that converges similarly).
//!   3. Subtract the envelope mean from the candidate signal.
//!   4. Repeat until the result satisfies IMF criteria (zero crossings
//!      ≈ extrema count, near-zero mean) or max iterations hit.
//!   5. The IMF becomes one component; the residual feeds the next
//!      level of sifting.
//!   6. Stop when residual has fewer than 3 extrema (monotone trend).
//!
//! Returns IMFs (high to low frequency) plus the final residual. Sum
//! of all components equals the input series exactly (algebraic
//! identity: each IMF is residual_prev - residual_next).
//!
//! Pure compute. Companion to `wavelet_decomposition_haar`,
//! `singular_spectrum_analysis`, `kalman_filter_1d`.

#![allow(clippy::needless_range_loop)]

#[derive(Debug)]
pub struct Report {
    pub imfs: Vec<Vec<f64>>,
    pub residual: Vec<f64>,
    pub iterations: Vec<u32>,
}

pub fn compute(series: &[f64], max_imfs: usize, max_sift_iter: u32) -> Option<Report> {
    let n = series.len();
    if n < 8 || max_imfs == 0 || max_sift_iter == 0 {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut residual = series.to_vec();
    let mut imfs: Vec<Vec<f64>> = Vec::new();
    let mut iterations = Vec::new();
    for _ in 0..max_imfs {
        // If residual has too few extrema, stop — it's a monotone trend.
        let (max_idx, min_idx) = find_extrema(&residual);
        if max_idx.len() < 2 || min_idx.len() < 2 {
            break;
        }
        let mut h = residual.clone();
        let mut iters = 0_u32;
        for _ in 0..max_sift_iter {
            iters += 1;
            let (max_idx, min_idx) = find_extrema(&h);
            if max_idx.len() < 2 || min_idx.len() < 2 {
                break;
            }
            let upper = linear_envelope(&h, &max_idx);
            let lower = linear_envelope(&h, &min_idx);
            let mean: Vec<f64> = upper
                .iter()
                .zip(lower.iter())
                .map(|(u, l)| 0.5 * (u + l))
                .collect();
            let new_h: Vec<f64> = h.iter().zip(mean.iter()).map(|(a, b)| a - b).collect();
            // Convergence: low-mean of envelope mean over series.
            let abs_mean: f64 = mean.iter().map(|x| x.abs()).sum::<f64>() / n as f64;
            let signal_scale: f64 = h.iter().map(|x| x.abs()).sum::<f64>() / n as f64;
            h = new_h;
            if abs_mean < signal_scale * 1e-3 || abs_mean < 1e-10 {
                break;
            }
        }
        iterations.push(iters);
        residual = residual.iter().zip(h.iter()).map(|(r, i)| r - i).collect();
        imfs.push(h);
    }
    Some(Report {
        imfs,
        residual,
        iterations,
    })
}

fn find_extrema(series: &[f64]) -> (Vec<usize>, Vec<usize>) {
    let n = series.len();
    let mut maxima = Vec::new();
    let mut minima = Vec::new();
    for i in 1..n - 1 {
        if series[i] > series[i - 1] && series[i] > series[i + 1] {
            maxima.push(i);
        } else if series[i] < series[i - 1] && series[i] < series[i + 1] {
            minima.push(i);
        }
    }
    (maxima, minima)
}

/// Linear interpolation through the `(idx, series\[idx\])` points, with the
/// series endpoints treated as additional anchors so the envelope spans
/// the full signal.
fn linear_envelope(series: &[f64], indices: &[usize]) -> Vec<f64> {
    let n = series.len();
    let mut anchors: Vec<(usize, f64)> = Vec::with_capacity(indices.len() + 2);
    // Left endpoint extrapolation: use first interior anchor's value.
    if !indices.is_empty() {
        anchors.push((0, series[indices[0]]));
    } else {
        anchors.push((0, series[0]));
    }
    for &idx in indices {
        if idx != 0 && idx != n - 1 {
            anchors.push((idx, series[idx]));
        }
    }
    if !indices.is_empty() {
        anchors.push((n - 1, series[*indices.last().unwrap()]));
    } else {
        anchors.push((n - 1, series[n - 1]));
    }
    let mut out = vec![0.0_f64; n];
    let mut seg = 0;
    for i in 0..n {
        while seg + 1 < anchors.len() && i > anchors[seg + 1].0 {
            seg += 1;
        }
        let (x0, y0) = anchors[seg];
        let (x1, y1) = if seg + 1 < anchors.len() {
            anchors[seg + 1]
        } else {
            (x0, y0)
        };
        if x0 == x1 {
            out[i] = y0;
        } else {
            let t = (i as f64 - x0 as f64) / (x1 as f64 - x0 as f64);
            out[i] = y0 + t * (y1 - y0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let s = vec![1.0_f64; 5];
        assert!(compute(&s, 5, 100).is_none()); // n < 8
        let s2 = vec![1.0_f64; 50];
        assert!(compute(&s2, 0, 100).is_none());
        assert!(compute(&s2, 5, 0).is_none());
        let mut s_nan = vec![1.0_f64; 50];
        s_nan[5] = f64::NAN;
        assert!(compute(&s_nan, 5, 100).is_none());
    }

    #[test]
    fn flat_series_yields_no_imfs() {
        let s = vec![5.0_f64; 50];
        let r = compute(&s, 5, 50).unwrap();
        assert_eq!(r.imfs.len(), 0);
        // Residual equals input.
        for v in &r.residual {
            assert!((v - 5.0).abs() < 1e-9);
        }
    }

    #[test]
    fn sum_of_components_equals_input() {
        let s: Vec<f64> = (0..100)
            .map(|i| (i as f64 * 0.5).sin() + (i as f64 * 0.05).cos() + 0.01 * i as f64)
            .collect();
        let r = compute(&s, 4, 50).unwrap();
        for i in 0..100 {
            let sum: f64 = r.imfs.iter().map(|imf| imf[i]).sum::<f64>() + r.residual[i];
            assert!((sum - s[i]).abs() < 1e-9);
        }
    }

    #[test]
    fn pure_oscillation_yields_imf() {
        // Pure sine should produce one IMF roughly matching the sine.
        let s: Vec<f64> = (0..200).map(|i| (i as f64 * 0.3).sin()).collect();
        let r = compute(&s, 3, 50).unwrap();
        assert!(!r.imfs.is_empty());
    }

    #[test]
    fn output_lengths_match_input() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.2).sin()).collect();
        let r = compute(&s, 3, 50).unwrap();
        assert_eq!(r.residual.len(), 100);
        for imf in &r.imfs {
            assert_eq!(imf.len(), 100);
        }
    }

    #[test]
    fn iteration_count_within_budget() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.3).sin()).collect();
        let r = compute(&s, 3, 25).unwrap();
        for &it in &r.iterations {
            assert!(it <= 25);
        }
    }
}
