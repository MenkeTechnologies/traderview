//! Matrix Profile (Mueen, Keogh 2016) — for every subsequence of length
//! `m` in a series, the z-normalized Euclidean distance to its nearest
//! non-trivial neighbor (excluding overlapping windows within ±m/2).
//!
//! Useful for:
//!   - **motif discovery** — pairs of low-distance subsequences are
//!     repeated patterns (e.g. recurring intraday patterns).
//!   - **discord detection** — high-distance points are anomalies that
//!     don't resemble anything else (e.g. flash crashes, fat-finger).
//!
//! Implementation is the naive O(n·n·m) STAMP form: for each query
//! window, scan all candidate windows, exclude trivial matches, keep
//! the min z-normalized Euclidean distance and its index. Suitable for
//! n in the low thousands; for larger n use STOMP/SCRIMP (out of scope).
//!
//! Pure compute. Companion to `dynamic_time_warping`, `motif_discovery`,
//! `pattern_recognition`.

#[derive(Debug)]
pub struct Report {
    /// Distance from each window to its nearest non-trivial neighbor.
    pub profile: Vec<f64>,
    /// Index of that nearest neighbor (or usize::MAX if none).
    pub indices: Vec<usize>,
    /// Indices and distances of the top-k discords (high distance).
    pub top_discords: Vec<(usize, f64)>,
    /// Indices of the top motif pair (lowest distance pair).
    pub top_motif_pair: Option<(usize, usize, f64)>,
}

pub fn compute(series: &[f64], m: usize, top_k_discords: usize) -> Option<Report> {
    let n = series.len();
    if m < 4 || n < 2 * m {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let num_windows = n - m + 1;
    // Pre-compute z-normalized windows.
    let mut z_windows: Vec<Option<Vec<f64>>> = Vec::with_capacity(num_windows);
    for i in 0..num_windows {
        z_windows.push(znorm(&series[i..i + m]));
    }
    let exclusion = m / 2;
    let mut profile = vec![f64::INFINITY; num_windows];
    let mut indices = vec![usize::MAX; num_windows];
    for (i, zi_opt) in z_windows.iter().enumerate() {
        let Some(zi) = zi_opt else { continue };
        for (j, zj_opt) in z_windows.iter().enumerate() {
            if i == j {
                continue;
            }
            if i.abs_diff(j) <= exclusion {
                continue;
            }
            let Some(zj) = zj_opt else { continue };
            let d = euclid(zi, zj);
            if d < profile[i] {
                profile[i] = d;
                indices[i] = j;
            }
        }
    }
    // Top-k discords by descending finite distance.
    let mut ranked: Vec<(usize, f64)> = profile
        .iter()
        .enumerate()
        .filter(|(_, &d)| d.is_finite())
        .map(|(i, &d)| (i, d))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_discords: Vec<(usize, f64)> = ranked.into_iter().take(top_k_discords).collect();
    // Top motif: globally smallest distance.
    let top_motif_pair = profile
        .iter()
        .enumerate()
        .filter(|(_, &d)| d.is_finite())
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, &d)| (i, indices[i], d));
    Some(Report {
        profile,
        indices,
        top_discords,
        top_motif_pair,
    })
}

fn znorm(win: &[f64]) -> Option<Vec<f64>> {
    let n = win.len() as f64;
    let mean = win.iter().sum::<f64>() / n;
    let var = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = var.max(0.0).sqrt();
    if std < 1e-12 {
        return None;
    }
    Some(win.iter().map(|x| (x - mean) / std).collect())
}

fn euclid(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let s = vec![1.0_f64; 50];
        assert!(compute(&s, 3, 5).is_none()); // m too small
        assert!(compute(&s, 30, 5).is_none()); // 2m > n
        let mut s_nan = s.clone();
        s_nan[5] = f64::NAN;
        assert!(compute(&s_nan, 5, 5).is_none());
    }

    #[test]
    fn output_lengths_match_window_count() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();
        let r = compute(&s, 10, 3).unwrap();
        assert_eq!(r.profile.len(), 91);
        assert_eq!(r.indices.len(), 91);
    }

    #[test]
    fn repeated_motif_yields_zero_distance() {
        // Two identical sine bursts → matrix profile should find them.
        let pattern: Vec<f64> = (0..20).map(|i| (i as f64 * 0.5).sin()).collect();
        let mut s = vec![0.0_f64; 100];
        for (i, &v) in pattern.iter().enumerate() {
            s[i] = v;
            s[60 + i] = v;
        }
        // Need some variation in the connecting bars so znorm isn't None.
        for (i, slot) in s.iter_mut().enumerate().take(60).skip(20) {
            *slot = (i as f64 * 0.05).cos() * 0.01;
        }
        let r = compute(&s, 15, 3).unwrap();
        // The min-distance pair should be near 0 (perfect match).
        let (_, _, dist) = r.top_motif_pair.unwrap();
        assert!(dist < 1e-6);
    }

    #[test]
    fn discord_detected_for_outlier_spike() {
        // Smooth sine with one spike → that window should be top discord.
        let mut s: Vec<f64> = (0..150).map(|i| (i as f64 * 0.2).sin()).collect();
        // Inject a giant spike at index 75.
        s[75] = 100.0;
        let r = compute(&s, 15, 3).unwrap();
        let (idx, _) = r.top_discords[0];
        // The discord window should overlap the spike (75 within [idx, idx+15)).
        assert!(idx + 15 > 75 && idx <= 75);
    }

    #[test]
    fn flat_windows_yield_no_neighbor() {
        // All zeros → every window flat → znorm returns None → profile
        // entries are infinity.
        let s = vec![0.0_f64; 100];
        let r = compute(&s, 10, 3).unwrap();
        assert!(r.profile.iter().all(|x| x.is_infinite()));
        assert!(r.top_motif_pair.is_none());
    }

    #[test]
    fn discord_count_capped_by_request() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.3).sin()).collect();
        let r = compute(&s, 10, 5).unwrap();
        assert!(r.top_discords.len() <= 5);
    }
}
