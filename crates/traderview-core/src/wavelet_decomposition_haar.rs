//! Haar Discrete Wavelet Decomposition — multi-resolution analysis of
//! a price/return series via cascaded Haar filter banks.
//!
//! The Haar wavelet is the simplest orthogonal wavelet:
//!   low-pass  ϕ:  [1/√2,  1/√2]   (averaging)
//!   high-pass ψ:  [1/√2, -1/√2]   (differencing)
//!
//! At each level k:
//!   approximation_k`[i]` = (a_{k-1}`[2i]` + a_{k-1}[2i+1]) / √2
//!   detail_k`[i]`        = (a_{k-1}`[2i]` - a_{k-1}[2i+1]) / √2
//!
//! Returns the final approximation plus the detail coefficients for
//! each level (level 1 = highest frequency, level N = lowest).
//!
//! Input length is truncated to the largest power-of-2 ≤ N. For longer
//! series, increase `levels`; for shorter, decrease.
//!
//! Pure compute. Companion to `kalman_filter_1d`, `savitzky_golay`,
//! `singular_spectrum_analysis`.

#[derive(Debug)]
pub struct Report {
    pub approximation: Vec<f64>,
    pub details: Vec<Vec<f64>>,
    pub levels: u32,
    pub used_length: usize,
}

pub fn compute(series: &[f64], levels: u32) -> Option<Report> {
    let n = series.len();
    if n < 2 || levels == 0 || levels > 20 { return None; }
    if series.iter().any(|x| !x.is_finite()) { return None; }
    let max_levels = (n as f64).log2().floor() as u32;
    if max_levels == 0 { return None; }
    let used_levels = levels.min(max_levels);
    let used_len = 1_usize << used_levels;
    if used_len > n { return None; }
    let mut current: Vec<f64> = series[..used_len].to_vec();
    let mut details: Vec<Vec<f64>> = Vec::with_capacity(used_levels as usize);
    let inv_sqrt2 = 1.0_f64 / std::f64::consts::SQRT_2;
    for _ in 0..used_levels {
        let half = current.len() / 2;
        let mut approx = vec![0.0_f64; half];
        let mut detail = vec![0.0_f64; half];
        for i in 0..half {
            approx[i] = (current[2 * i] + current[2 * i + 1]) * inv_sqrt2;
            detail[i] = (current[2 * i] - current[2 * i + 1]) * inv_sqrt2;
        }
        details.push(detail);
        current = approx;
    }
    Some(Report {
        approximation: current,
        details,
        levels: used_levels,
        used_length: used_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[1.0_f64], 3).is_none());
        let s = vec![1.0_f64; 16];
        assert!(compute(&s, 0).is_none());
        assert!(compute(&s, 25).is_none());
        let mut s_nan = s.clone();
        s_nan[0] = f64::NAN;
        assert!(compute(&s_nan, 2).is_none());
    }

    #[test]
    fn constant_series_yields_zero_detail() {
        // Constant → no high-frequency variation. All detail = 0.
        let s = vec![5.0_f64; 16];
        let r = compute(&s, 4).unwrap();
        for d in &r.details {
            for &v in d {
                assert!(v.abs() < 1e-9);
            }
        }
    }

    #[test]
    fn parseval_energy_conserved() {
        // Σ approx² + Σ details² == Σ input² (orthogonality).
        let s: Vec<f64> = (0..16).map(|i| (i as f64).sin()).collect();
        let r = compute(&s, 4).unwrap();
        let in_energy: f64 = s.iter().map(|x| x * x).sum();
        let approx_e: f64 = r.approximation.iter().map(|x| x * x).sum();
        let detail_e: f64 = r.details.iter()
            .flatten().map(|x| x * x).sum();
        assert!((in_energy - approx_e - detail_e).abs() < 1e-9);
    }

    #[test]
    fn details_size_halves_per_level() {
        let s = vec![1.0_f64; 16];
        let r = compute(&s, 4).unwrap();
        assert_eq!(r.details[0].len(), 8);
        assert_eq!(r.details[1].len(), 4);
        assert_eq!(r.details[2].len(), 2);
        assert_eq!(r.details[3].len(), 1);
        assert_eq!(r.approximation.len(), 1);
    }

    #[test]
    fn used_length_is_power_of_2() {
        let s = vec![1.0_f64; 50];
        let r = compute(&s, 5).unwrap();
        assert_eq!(r.used_length, 32);    // largest 2^k ≤ 50
        assert_eq!(r.levels, 5);
    }

    #[test]
    fn levels_capped_to_log2_floor() {
        let s = vec![1.0_f64; 16];
        let r = compute(&s, 10).unwrap();
        assert_eq!(r.levels, 4);          // log2(16) = 4
    }

    #[test]
    fn impulse_inside_a_pair_localizes_detail() {
        // Impulse at index 9 sits inside pair (8,9), so the level-1
        // detail at index 4 should be the only non-zero coefficient.
        let mut s = vec![0.0_f64; 16];
        s[9] = 1.0;
        let r = compute(&s, 4).unwrap();
        let nonzero: Vec<usize> = r.details[0].iter().enumerate()
            .filter(|(_, &v)| v.abs() > 1e-9).map(|(i, _)| i).collect();
        assert_eq!(nonzero, vec![4]);
    }
}
