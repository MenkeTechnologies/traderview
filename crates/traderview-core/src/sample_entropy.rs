//! Sample Entropy (Pincus & Goldberger 1994) — measure of regularity
//! in a time series.
//!
//!   SampEn(m, r, N) = −ln(A / B)
//!
//! where:
//!   B = #{pairs of length-m subsequences within tolerance r}
//!   A = #{pairs of length-(m+1) subsequences within tolerance r}
//!
//! Lower entropy = more predictable / regular series. Higher = more
//! random. Distinct from Shannon entropy (which is distribution-based);
//! SampEn captures TEMPORAL regularity.
//!
//! Defaults: m = 2, r = 0.2 · stdev(series) (Pincus convention).
//!
//! Pure compute. O(N²) — caps input length to keep memory bounded.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SampleEntropyReport {
    pub sample_entropy: f64,
    pub m: usize,
    pub tolerance_r: f64,
    pub n_observations: usize,
}

pub fn compute(series: &[f64], m: usize, tolerance: f64) -> Option<SampleEntropyReport> {
    let n = series.len();
    if n < m + 2 || m == 0 || !(2..=2_000).contains(&n)
        || !tolerance.is_finite() || tolerance <= 0.0
    {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) { return None; }
    // Count matching pairs of length m and m+1 within tolerance.
    let (count_m, count_m1) = count_matches(series, m, tolerance);
    if count_m == 0 || count_m1 == 0 {
        return None;
    }
    let entropy = -((count_m1 as f64) / (count_m as f64)).ln();
    if !entropy.is_finite() { return None; }
    Some(SampleEntropyReport {
        sample_entropy: entropy,
        m,
        tolerance_r: tolerance,
        n_observations: n,
    })
}

/// Convenience: compute SampEn with the Pincus-recommended default
/// tolerance r = 0.2 · stdev(series).
pub fn compute_default(series: &[f64]) -> Option<SampleEntropyReport> {
    let n = series.len();
    if n < 4 { return None; }
    let clean: Vec<f64> = series.iter().copied().filter(|x| x.is_finite()).collect();
    if clean.len() < 4 { return None; }
    let mean: f64 = clean.iter().sum::<f64>() / clean.len() as f64;
    let var: f64 = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / clean.len() as f64;
    let sd = var.max(0.0).sqrt();
    if sd <= 0.0 { return None; }
    compute(&clean, 2, 0.2 * sd)
}

fn count_matches(series: &[f64], m: usize, tolerance: f64) -> (usize, usize) {
    let n = series.len();
    let mut count_m = 0_usize;
    let mut count_m1 = 0_usize;
    for i in 0..(n - m) {
        for j in (i + 1)..(n - m) {
            // Chebyshev distance for length-m windows.
            let mut d_m = 0.0_f64;
            let mut matched_m = true;
            for k in 0..m {
                let diff = (series[i + k] - series[j + k]).abs();
                if diff > d_m { d_m = diff; }
                if d_m > tolerance { matched_m = false; break; }
            }
            if matched_m {
                count_m += 1;
                // Extend by one more sample (if both i+m and j+m exist).
                if i + m < n && j + m < n {
                    let diff_extra = (series[i + m] - series[j + m]).abs();
                    let d_m1 = d_m.max(diff_extra);
                    if d_m1 <= tolerance {
                        count_m1 += 1;
                    }
                }
            }
        }
    }
    (count_m, count_m1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 3], 2, 0.1).is_none());
        assert!(compute_default(&[0.01; 3]).is_none());
    }

    #[test]
    fn invalid_params_return_none() {
        let s = vec![0.01; 50];
        assert!(compute(&s, 0, 0.1).is_none());
        assert!(compute(&s, 2, 0.0).is_none());
        assert!(compute(&s, 2, -0.1).is_none());
        assert!(compute(&s, 2, f64::NAN).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut s = vec![0.01; 50];
        s[5] = f64::NAN;
        assert!(compute(&s, 2, 0.1).is_none());
    }

    #[test]
    fn periodic_series_yields_low_entropy() {
        // Simple periodic series → very regular → low SampEn.
        let s: Vec<f64> = (0..200).map(|i| (i as f64 * 0.5).sin()).collect();
        let r = compute_default(&s).unwrap();
        // Periodic should have low entropy (typically < 1).
        assert!(r.sample_entropy < 1.5,
            "periodic series should have low SampEn, got {}", r.sample_entropy);
    }

    #[test]
    fn random_series_yields_higher_entropy_than_constant() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..300).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 32) as f64 / u32::MAX as f64
        }).collect();
        let r = compute_default(&s).unwrap();
        // iid random data should produce SampEn around ln(N) territory (>0).
        assert!(r.sample_entropy > 0.5,
            "random series should have entropy > 0.5, got {}", r.sample_entropy);
    }

    #[test]
    fn compute_default_picks_pincus_tolerance() {
        let mut state: u64 = 7;
        let s: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 32) as f64 / u32::MAX as f64
        }).collect();
        let r = compute_default(&s).unwrap();
        // Tolerance should be ~ 0.2 · sd ≈ 0.06 for uniform[0,1] (sd ≈ 0.289).
        assert!(r.tolerance_r > 0.03 && r.tolerance_r < 0.1);
    }

    #[test]
    fn very_long_input_rejected() {
        let s = vec![0.01_f64; 3_000];
        assert!(compute(&s, 2, 0.1).is_none());
    }
}
