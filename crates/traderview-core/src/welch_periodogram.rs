//! Welch's Method for Power Spectral Density Estimation.
//!
//! Standard non-parametric PSD estimator:
//!   1. Split series into K overlapping segments of length `segment_len`
//!   2. Window each segment (Hann window by default)
//!   3. Compute DFT magnitude squared
//!   4. Average across segments → smoother PSD with reduced variance
//!
//! For N total samples, segment length L, overlap fraction `overlap`,
//! number of segments K = floor((N − L) / (L · (1 − overlap))) + 1.
//!
//! Frequency grid: 0, 1/L, …, (L/2)/L (in cycles per sample).
//!
//! Pure compute. O(K · L²) direct DFT (no FFT crate dependency).
//! Practical for typical financial windows (L = 64..256, K = 10..50).
//!
//! Companion to `hilbert_transform`, `dfa`, `realized_volatility`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WelchPeriodogramReport {
    /// Frequencies in cycles per sample (0 to 0.5).
    pub frequencies: Vec<f64>,
    /// PSD at each frequency (averaged across windows).
    pub psd: Vec<f64>,
    pub n_segments: usize,
    pub segment_length: usize,
    pub overlap_fraction: f64,
    pub n_observations: usize,
    /// Frequency of peak PSD (excluding DC).
    pub dominant_frequency: f64,
}

pub fn compute(
    series: &[f64],
    segment_length: usize,
    overlap_fraction: f64,
) -> Option<WelchPeriodogramReport> {
    let n = series.len();
    if n < segment_length
        || segment_length < 8
        || !overlap_fraction.is_finite()
        || !(0.0..1.0).contains(&overlap_fraction)
    {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let step = ((segment_length as f64) * (1.0 - overlap_fraction)).max(1.0) as usize;
    let n_half = segment_length / 2 + 1;
    let mut psd_sum = vec![0.0_f64; n_half];
    let mut k_count = 0_usize;
    // Hann window with W_n = 0.5 · (1 − cos(2π·n / (L−1))).
    let window: Vec<f64> = (0..segment_length)
        .map(|n_idx| {
            0.5 * (1.0
                - (2.0 * std::f64::consts::PI * n_idx as f64 / (segment_length - 1) as f64).cos())
        })
        .collect();
    let window_norm: f64 = window.iter().map(|w| w * w).sum::<f64>();
    let mut start = 0_usize;
    while start + segment_length <= n {
        let segment = &series[start..start + segment_length];
        // Windowed segment.
        let windowed: Vec<f64> = segment
            .iter()
            .zip(window.iter())
            .map(|(x, w)| x * w)
            .collect();
        // Direct DFT magnitude squared, normalized by window energy.
        for (k, slot) in psd_sum.iter_mut().enumerate().take(n_half) {
            let mut re = 0.0_f64;
            let mut im = 0.0_f64;
            for (t, x) in windowed.iter().enumerate() {
                let arg = -2.0 * std::f64::consts::PI * (k * t) as f64 / segment_length as f64;
                re += x * arg.cos();
                im += x * arg.sin();
            }
            let mag_sq = (re * re + im * im) / window_norm;
            *slot += mag_sq;
        }
        k_count += 1;
        start += step;
    }
    if k_count == 0 {
        return None;
    }
    let psd: Vec<f64> = psd_sum.iter().map(|p| p / k_count as f64).collect();
    let frequencies: Vec<f64> = (0..n_half)
        .map(|k| k as f64 / segment_length as f64)
        .collect();
    // Dominant freq (skip DC at k=0).
    let dom_idx = psd[1..]
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i + 1)
        .unwrap_or(0);
    let dominant = frequencies[dom_idx];
    Some(WelchPeriodogramReport {
        frequencies,
        psd,
        n_segments: k_count,
        segment_length,
        overlap_fraction,
        n_observations: n,
        dominant_frequency: dominant,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let s = vec![0.0_f64; 100];
        assert!(compute(&s, 4, 0.5).is_none());
        assert!(compute(&s, 200, 0.5).is_none()); // segment > n
        assert!(compute(&s, 32, 1.0).is_none());
        assert!(compute(&s, 32, -0.1).is_none());
        assert!(compute(&s, 32, f64::NAN).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut s = vec![0.0_f64; 100];
        s[5] = f64::NAN;
        assert!(compute(&s, 32, 0.5).is_none());
    }

    #[test]
    fn pure_sinusoid_peaks_at_expected_frequency() {
        // Sine wave with period 16 samples → frequency 1/16 = 0.0625 cycles/sample.
        let s: Vec<f64> = (0..256)
            .map(|n| (2.0 * std::f64::consts::PI * n as f64 / 16.0).sin())
            .collect();
        let r = compute(&s, 64, 0.5).unwrap();
        // Dominant frequency should be near 1/16 = 0.0625.
        assert!((r.dominant_frequency - 0.0625).abs() < 0.02);
    }

    #[test]
    fn flat_signal_concentrates_energy_at_dc() {
        // Constant input × Hann window = Hann window scaled, which has
        // energy spread across low-frequency bins. The DOMINANT bin is
        // still at DC, but other bins are nonzero. Verify DC dominates.
        let s = vec![5.0_f64; 200];
        let r = compute(&s, 64, 0.5).unwrap();
        let max_non_dc = r
            .psd
            .iter()
            .skip(1)
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            r.psd[0] > max_non_dc,
            "DC bin {} should exceed max non-DC {}",
            r.psd[0],
            max_non_dc
        );
    }

    #[test]
    fn zero_signal_yields_zero_psd() {
        let s = vec![0.0_f64; 200];
        let r = compute(&s, 64, 0.5).unwrap();
        for p in &r.psd {
            assert!(p.abs() < 1e-9);
        }
    }

    #[test]
    fn psd_non_negative() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..256)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0
            })
            .collect();
        let r = compute(&s, 64, 0.5).unwrap();
        for p in &r.psd {
            assert!(*p >= 0.0);
        }
    }

    #[test]
    fn frequency_grid_in_unit_interval_half() {
        let s = vec![1.0_f64; 100];
        let r = compute(&s, 32, 0.5).unwrap();
        for f in &r.frequencies {
            assert!((0.0..=0.5).contains(f));
        }
    }

    #[test]
    fn n_segments_matches_step_count() {
        // n=128, segment=64, overlap=0.5 → step=32, segments at 0,32,64 → 3.
        let s = vec![1.0_f64; 128];
        let r = compute(&s, 64, 0.5).unwrap();
        assert_eq!(r.n_segments, 3);
    }

    #[test]
    fn output_lengths_consistent() {
        let s: Vec<f64> = (0..200).map(|i| (i as f64 * 0.1).sin()).collect();
        let r = compute(&s, 64, 0.5).unwrap();
        assert_eq!(r.psd.len(), r.frequencies.len());
        assert_eq!(r.psd.len(), 33); // L/2 + 1 = 33
    }
}
