//! Volume-Weighted Average Price (VWAP) Execution Scheduler — slices a
//! total order proportionally to a historical per-bar volume profile.
//!
//!   weight_i = volume_curve_i / Σ volume_curve_j
//!   slice_i  = total_order_size · weight_i
//!
//! Goal: minimize tracking-error vs the realized session VWAP. Works
//! when the historical volume profile (e.g. 20-day average per 5-min
//! bar) is a good predictor of today's profile — which holds in
//! liquid markets without unusual catalysts.
//!
//! Unlike POV, VWAP commits to executing the full order across the
//! provided window — no shortfall logic. If you need participation-
//! capped scheduling, use `optimal_execution_pov_schedule` instead.
//!
//! Reports per-bar slice sizes, cumulative fill, and the implied
//! peak-bar participation rate (largest slice / its bar volume).
//!
//! Pure compute. Companion to `optimal_execution_twap_schedule`,
//! `optimal_execution_pov_schedule`, `vwap`, `vwap_bands`.

#[derive(Debug)]
pub struct Report {
    pub slices: Vec<f64>,
    pub cumulative_fill: Vec<f64>,
    pub max_participation_rate: f64,
}

pub fn compute(total_order_size: f64, volume_curve: &[f64]) -> Option<Report> {
    if !total_order_size.is_finite() || total_order_size <= 0.0 {
        return None;
    }
    if volume_curve.is_empty() {
        return None;
    }
    if volume_curve.iter().any(|x| !x.is_finite() || *x < 0.0) {
        return None;
    }
    let total_volume: f64 = volume_curve.iter().sum();
    if total_volume <= 0.0 {
        return None;
    }
    let slices: Vec<f64> = volume_curve
        .iter()
        .map(|v| total_order_size * (v / total_volume))
        .collect();
    let mut cumulative_fill = vec![0.0_f64; slices.len()];
    let mut acc = 0.0_f64;
    for (i, s) in slices.iter().enumerate() {
        acc += s;
        cumulative_fill[i] = acc;
    }
    let max_participation_rate = slices
        .iter()
        .zip(volume_curve.iter())
        .map(|(s, v)| if *v > 0.0 { s / v } else { 0.0 })
        .fold(0.0_f64, f64::max);
    Some(Report {
        slices,
        cumulative_fill,
        max_participation_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let v = vec![1000.0_f64; 10];
        assert!(compute(0.0, &v).is_none());
        assert!(compute(-100.0, &v).is_none());
        assert!(compute(100.0, &[]).is_none());
        let zeros = vec![0.0_f64; 10];
        assert!(compute(100.0, &zeros).is_none());
        let mut bad = v.clone();
        bad[0] = f64::NAN;
        assert!(compute(100.0, &bad).is_none());
        let mut neg = v;
        neg[0] = -1.0;
        assert!(compute(100.0, &neg).is_none());
    }

    #[test]
    fn uniform_volume_yields_equal_slices() {
        // Flat volume → VWAP collapses to TWAP.
        let v = vec![1000.0_f64; 10];
        let r = compute(1000.0, &v).unwrap();
        for s in &r.slices {
            assert!((s - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn slices_sum_to_total() {
        let v = vec![500.0, 1000.0, 2000.0, 1500.0, 500.0];
        let r = compute(1000.0, &v).unwrap();
        let sum: f64 = r.slices.iter().sum();
        assert!((sum - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn high_volume_bars_get_proportionally_more() {
        // Bar 2 has 4x bar 0's volume → 4x the slice.
        let v = vec![500.0, 500.0, 2000.0, 500.0];
        let r = compute(1000.0, &v).unwrap();
        assert!((r.slices[2] / r.slices[0] - 4.0).abs() < 1e-9);
    }

    #[test]
    fn cumulative_reaches_total() {
        let v = vec![500.0, 1000.0, 2000.0, 1500.0, 500.0];
        let r = compute(1000.0, &v).unwrap();
        assert!((r.cumulative_fill[4] - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn max_participation_is_consistent() {
        // 10% participation everywhere when total order = 10% of total volume.
        let v = vec![1000.0_f64; 10];
        let r = compute(1000.0, &v).unwrap();
        assert!((r.max_participation_rate - 0.1).abs() < 1e-9);
    }
}
