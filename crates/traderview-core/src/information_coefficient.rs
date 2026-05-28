//! Information Coefficient (IC) — cross-sectional correlation between
//! predicted and realized values, the canonical quant signal-quality
//! metric.
//!
//! For a cross-section of N assets at a given date:
//!
//!   IC_pearson  = Pearson correlation of (signal_i, realized_return_i)
//!   IC_spearman = Spearman rank correlation of the same
//!
//! When measured across many dates:
//!
//!   IR = mean(IC_t) / sd(IC_t) · √(periods_per_year)
//!
//! This Information Ratio (a.k.a. "fundamental law of active
//! management" ratio per Grinold) is the most common signal-strength
//! summary in factor research.
//!
//! Reports both per-period ICs and aggregate IR. Spearman is more
//! robust to outliers and is the typical "monthly IC" reported.
//!
//! Pure compute. Companion to `spearman_correlation`,
//! `composite_factor_scoring`, `relative_volume_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSection {
    pub signal: Vec<f64>,
    pub realized: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InformationCoefficientReport {
    pub per_period_ic_pearson: Vec<f64>,
    pub per_period_ic_spearman: Vec<f64>,
    pub mean_ic_pearson: f64,
    pub mean_ic_spearman: f64,
    pub stdev_ic_pearson: f64,
    pub stdev_ic_spearman: f64,
    pub information_ratio_pearson: f64,
    pub information_ratio_spearman: f64,
    pub hit_rate: f64,
    pub n_periods: usize,
    pub periods_per_year: f64,
}

pub fn compute(
    cross_sections: &[CrossSection],
    periods_per_year: f64,
) -> Option<InformationCoefficientReport> {
    if cross_sections.is_empty() || !periods_per_year.is_finite() || periods_per_year <= 0.0 {
        return None;
    }
    let mut pearson_ics = Vec::new();
    let mut spearman_ics = Vec::new();
    for cs in cross_sections {
        if cs.signal.len() != cs.realized.len() || cs.signal.len() < 3 { continue; }
        if cs.signal.iter().any(|x| !x.is_finite())
            || cs.realized.iter().any(|x| !x.is_finite()) { continue; }
        if let Some(p) = pearson(&cs.signal, &cs.realized) { pearson_ics.push(p); }
        if let Some(s) = spearman(&cs.signal, &cs.realized) { spearman_ics.push(s); }
    }
    if pearson_ics.is_empty() || spearman_ics.is_empty() { return None; }
    let n_pearson = pearson_ics.len() as f64;
    let n_spearman = spearman_ics.len() as f64;
    let mean_p: f64 = pearson_ics.iter().sum::<f64>() / n_pearson;
    let mean_s: f64 = spearman_ics.iter().sum::<f64>() / n_spearman;
    let sd_p = stdev(&pearson_ics, mean_p);
    let sd_s = stdev(&spearman_ics, mean_s);
    let ir_p = if sd_p > 0.0 { mean_p / sd_p * periods_per_year.sqrt() } else { 0.0 };
    let ir_s = if sd_s > 0.0 { mean_s / sd_s * periods_per_year.sqrt() } else { 0.0 };
    let hit_rate = spearman_ics.iter().filter(|x| **x > 0.0).count() as f64 / n_spearman;
    let n_periods = pearson_ics.len();
    Some(InformationCoefficientReport {
        per_period_ic_pearson: pearson_ics,
        per_period_ic_spearman: spearman_ics,
        mean_ic_pearson: mean_p,
        mean_ic_spearman: mean_s,
        stdev_ic_pearson: sd_p,
        stdev_ic_spearman: sd_s,
        information_ratio_pearson: ir_p,
        information_ratio_spearman: ir_s,
        hit_rate,
        n_periods,
        periods_per_year,
    })
}

fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len() as f64;
    if n < 2.0 { return None; }
    let x_mean: f64 = x.iter().sum::<f64>() / n;
    let y_mean: f64 = y.iter().sum::<f64>() / n;
    let mut sxx = 0.0_f64;
    let mut syy = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..x.len() {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 { return None; }
    Some((sxy / (sxx * syy).sqrt()).clamp(-1.0, 1.0))
}

fn spearman(x: &[f64], y: &[f64]) -> Option<f64> {
    let rx = ranks(x);
    let ry = ranks(y);
    pearson(&rx, &ry)
}

fn ranks(v: &[f64]) -> Vec<f64> {
    let n = v.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|a, b| v[*a].partial_cmp(&v[*b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut r = vec![0.0_f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && v[idx[j + 1]] == v[idx[i]] { j += 1; }
        let mid = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j { r[idx[k]] = mid; }
        i = j + 1;
    }
    r
}

fn stdev(v: &[f64], mean: f64) -> f64 {
    if v.len() < 2 { return 0.0; }
    let var: f64 = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
        / (v.len() - 1) as f64;
    var.max(0.0).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cs(signal: Vec<f64>, realized: Vec<f64>) -> CrossSection {
        CrossSection { signal, realized }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 12.0).is_none());
    }

    #[test]
    fn invalid_periods_per_year_returns_none() {
        let xs = vec![cs(vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0])];
        assert!(compute(&xs, 0.0).is_none());
        assert!(compute(&xs, f64::NAN).is_none());
    }

    #[test]
    fn perfect_signal_yields_ic_one() {
        let xs: Vec<_> = (0..10).map(|seed| {
            let signal: Vec<f64> = (0..20).map(|i| i as f64 + seed as f64).collect();
            let realized: Vec<f64> = signal.iter().map(|s| 2.0 * s).collect();
            cs(signal, realized)
        }).collect();
        let r = compute(&xs, 12.0).unwrap();
        assert!((r.mean_ic_pearson - 1.0).abs() < 1e-9);
        assert!((r.mean_ic_spearman - 1.0).abs() < 1e-9);
        assert!((r.hit_rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn anti_perfect_signal_yields_ic_minus_one() {
        let xs: Vec<_> = (0..10).map(|seed| {
            let signal: Vec<f64> = (0..20).map(|i| i as f64 + seed as f64).collect();
            let realized: Vec<f64> = signal.iter().map(|s| -s).collect();
            cs(signal, realized)
        }).collect();
        let r = compute(&xs, 12.0).unwrap();
        assert!((r.mean_ic_pearson + 1.0).abs() < 1e-9);
        assert!((r.mean_ic_spearman + 1.0).abs() < 1e-9);
        assert!(r.hit_rate < 0.01);
    }

    #[test]
    fn ir_scales_with_periods_per_year() {
        // Same mean/sd of IC, different annualization → IR scales as √f.
        let xs: Vec<_> = (0..10).map(|_| {
            cs(vec![1.0, 2.0, 3.0, 4.0, 5.0], vec![1.1, 2.0, 2.9, 4.1, 5.0])
        }).collect();
        let r12 = compute(&xs, 12.0).unwrap();
        let r252 = compute(&xs, 252.0).unwrap();
        let ratio = r252.information_ratio_pearson / r12.information_ratio_pearson;
        let expected = (252.0_f64 / 12.0).sqrt();
        assert!((ratio - expected).abs() < 1e-6);
    }

    #[test]
    fn mismatched_or_small_cross_sections_skipped() {
        let xs = vec![
            cs(vec![1.0, 2.0], vec![1.0, 2.0]),       // too small
            cs(vec![1.0, 2.0, 3.0], vec![1.0, 2.0]),    // mismatched
            cs(vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0]),
        ];
        let r = compute(&xs, 12.0).unwrap();
        assert_eq!(r.n_periods, 1);
    }

    #[test]
    fn nan_cross_section_skipped() {
        let xs = vec![
            cs(vec![1.0, f64::NAN, 3.0], vec![1.0, 2.0, 3.0]),
            cs(vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0]),
        ];
        let r = compute(&xs, 12.0).unwrap();
        assert_eq!(r.n_periods, 1);
    }

    #[test]
    fn hit_rate_in_unit_range() {
        let xs: Vec<_> = (0..20).map(|seed| {
            let signal: Vec<f64> = (0..10).map(|i| i as f64 + seed as f64).collect();
            let realized: Vec<f64> = (0..10).map(|i| {
                if (seed + i) % 2 == 0 { i as f64 } else { -(i as f64) }
            }).collect();
            cs(signal, realized)
        }).collect();
        let r = compute(&xs, 12.0).unwrap();
        assert!((0.0..=1.0).contains(&r.hit_rate));
    }
}
