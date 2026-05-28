//! Survival-probability curve from a piecewise-constant hazard rate
//! schedule.
//!
//!   Λ(0, T) = Σ_i λ_i · Δt_i      (cumulative hazard up to T)
//!   S(T)   = exp(−Λ(0, T))         (survival probability)
//!   PD(T)  = 1 − S(T)              (probability of default by T)
//!
//! Companion to `cds_pricing` (which uses survival probabilities to
//! discount the protection and premium legs). This module exposes the
//! curve as a first-class API for risk display and capital-at-risk
//! calculations (e.g. EPE/PFE simulations).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HazardSegment {
    /// End time of this segment (years).
    pub end_time_years: f64,
    /// Annualized hazard rate during the segment.
    pub hazard_rate: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SurvivalPoint {
    pub time_years: f64,
    pub cumulative_hazard: f64,
    pub survival_probability: f64,
    pub default_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SurvivalCurveReport {
    pub points: Vec<SurvivalPoint>,
    pub expected_default_time: f64,
}

pub fn build_curve(
    segments: &[HazardSegment],
    query_times: &[f64],
) -> Option<SurvivalCurveReport> {
    if segments.is_empty() || query_times.is_empty() { return None; }
    if segments.iter().any(|s| !s.end_time_years.is_finite() || s.end_time_years <= 0.0
        || !s.hazard_rate.is_finite() || s.hazard_rate < 0.0)
    {
        return None;
    }
    if query_times.iter().any(|t| !t.is_finite() || *t < 0.0) {
        return None;
    }
    // Sort segments by end_time.
    let mut sorted: Vec<HazardSegment> = segments.to_vec();
    sorted.sort_by(|a, b| a.end_time_years.partial_cmp(&b.end_time_years)
        .unwrap_or(std::cmp::Ordering::Equal));
    // Strictly increasing end times.
    for w in sorted.windows(2) {
        if w[1].end_time_years <= w[0].end_time_years { return None; }
    }
    let cumulative_hazard_at = |t: f64| -> f64 {
        if t <= 0.0 { return 0.0; }
        let mut prev_t = 0.0_f64;
        let mut h = 0.0_f64;
        for seg in &sorted {
            let cur_t = seg.end_time_years.min(t);
            if cur_t > prev_t {
                h += seg.hazard_rate * (cur_t - prev_t);
            }
            if t <= seg.end_time_years { return h; }
            prev_t = seg.end_time_years;
        }
        // Past the last segment: extrapolate flat with the last hazard.
        let last = sorted.last().unwrap();
        if t > last.end_time_years {
            h += last.hazard_rate * (t - last.end_time_years);
        }
        h
    };
    let points: Vec<SurvivalPoint> = query_times.iter().map(|&t| {
        let h = cumulative_hazard_at(t);
        let s = (-h).exp();
        SurvivalPoint {
            time_years: t,
            cumulative_hazard: h,
            survival_probability: s,
            default_probability: (1.0 - s).max(0.0),
        }
    }).collect();
    // Expected default time = ∫_0^∞ S(t) dt (mean residual life). Approximate
    // by trapezoidal integration over the query grid (assuming sorted, finite).
    let expected_default_time = {
        let mut sorted_pts = points.clone();
        sorted_pts.sort_by(|a, b| a.time_years.partial_cmp(&b.time_years)
            .unwrap_or(std::cmp::Ordering::Equal));
        let mut area = 0.0_f64;
        for w in sorted_pts.windows(2) {
            let dt = w[1].time_years - w[0].time_years;
            area += dt * (w[0].survival_probability + w[1].survival_probability) / 2.0;
        }
        area
    };
    Some(SurvivalCurveReport { points, expected_default_time })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(t: f64, lam: f64) -> HazardSegment {
        HazardSegment { end_time_years: t, hazard_rate: lam }
    }

    #[test]
    fn empty_returns_none() {
        assert!(build_curve(&[], &[1.0]).is_none());
        assert!(build_curve(&[h(1.0, 0.05)], &[]).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(build_curve(&[h(-1.0, 0.05)], &[1.0]).is_none());
        assert!(build_curve(&[h(1.0, -0.05)], &[1.0]).is_none());
        assert!(build_curve(&[h(1.0, 0.05)], &[-1.0]).is_none());
        assert!(build_curve(&[h(f64::NAN, 0.05)], &[1.0]).is_none());
    }

    #[test]
    fn flat_hazard_yields_exponential_survival() {
        // Constant λ = 0.02 → S(t) = e^{−0.02·t}.
        let segs = vec![h(10.0, 0.02)];
        let queries = vec![1.0, 2.0, 5.0, 10.0];
        let r = build_curve(&segs, &queries).unwrap();
        for pt in &r.points {
            let expected_s = (-0.02_f64 * pt.time_years).exp();
            assert!((pt.survival_probability - expected_s).abs() < 1e-9);
        }
    }

    #[test]
    fn survival_monotonically_decreasing() {
        let segs = vec![h(5.0, 0.01), h(10.0, 0.03), h(20.0, 0.06)];
        let queries: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let r = build_curve(&segs, &queries).unwrap();
        for w in r.points.windows(2) {
            assert!(w[1].survival_probability <= w[0].survival_probability);
        }
    }

    #[test]
    fn default_probability_complements_survival() {
        let segs = vec![h(5.0, 0.05)];
        let r = build_curve(&segs, &[3.0]).unwrap();
        assert!((r.points[0].survival_probability + r.points[0].default_probability - 1.0).abs() < 1e-9);
    }

    #[test]
    fn time_zero_yields_full_survival() {
        let segs = vec![h(5.0, 0.05)];
        let r = build_curve(&segs, &[0.0]).unwrap();
        assert_eq!(r.points[0].survival_probability, 1.0);
        assert_eq!(r.points[0].default_probability, 0.0);
    }

    #[test]
    fn unsorted_segments_handled_via_internal_sort() {
        let segs = vec![h(5.0, 0.05), h(2.0, 0.01)];
        let r = build_curve(&segs, &[1.0, 3.0]).unwrap();
        // At t=1 (inside first sorted segment with λ=0.01) → S = e^{-0.01·1}.
        let expected = (-0.01_f64).exp();
        assert!((r.points[0].survival_probability - expected).abs() < 1e-9);
    }

    #[test]
    fn duplicate_segment_end_times_rejected() {
        let segs = vec![h(2.0, 0.01), h(2.0, 0.05)];
        assert!(build_curve(&segs, &[1.0]).is_none());
    }

    #[test]
    fn higher_hazard_yields_lower_survival() {
        let r_low = build_curve(&[h(5.0, 0.01)], &[5.0]).unwrap();
        let r_high = build_curve(&[h(5.0, 0.10)], &[5.0]).unwrap();
        assert!(r_high.points[0].survival_probability < r_low.points[0].survival_probability);
    }
}
