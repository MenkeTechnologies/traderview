//! Time-Weighted Average Price (TWAP) Execution Scheduler — slices a
//! total order into equal-size child orders distributed evenly across
//! a fixed number of bars.
//!
//! Per bar:  slice_i = total_order_size / num_slices
//!
//! Counterpart to `optimal_execution_pov_schedule` (which adapts to
//! per-bar expected volume). TWAP is appropriate when:
//!   - No reliable volume curve is available.
//!   - The trader wants to minimize tracking-error vs the simple
//!     time-average price.
//!   - Market-impact estimates favor uniform spreading.
//!
//! Reports per-bar slice sizes, cumulative fill, and the per-bar
//! participation rate if a volume curve is also provided (optional).
//! High participation rates signal a slice could overwhelm the local
//! tape — caller can throttle.
//!
//! Pure compute. Companion to `optimal_execution_pov_schedule`,
//! `implementation_shortfall`, `almgren_chriss`.

#[derive(Debug)]
pub struct Report {
    pub slices: Vec<f64>,
    pub cumulative_fill: Vec<f64>,
    pub max_participation_rate: f64,
}

pub fn compute(
    total_order_size: f64,
    num_slices: usize,
    optional_volume_curve: Option<&[f64]>,
) -> Option<Report> {
    if !total_order_size.is_finite() || total_order_size <= 0.0 { return None; }
    if num_slices == 0 { return None; }
    if let Some(v) = optional_volume_curve {
        if v.len() != num_slices { return None; }
        if v.iter().any(|x| !x.is_finite() || *x < 0.0) { return None; }
    }
    let per_slice = total_order_size / num_slices as f64;
    let slices = vec![per_slice; num_slices];
    let mut cumulative_fill = vec![0.0_f64; num_slices];
    let mut acc = 0.0_f64;
    for (i, s) in slices.iter().enumerate() {
        acc += s;
        cumulative_fill[i] = acc;
    }
    let max_participation_rate = optional_volume_curve.map(|v| {
        slices.iter().zip(v.iter())
            .map(|(s, vol)| if *vol > 0.0 { s / vol } else { 0.0 })
            .fold(0.0_f64, f64::max)
    }).unwrap_or(0.0);
    Some(Report { slices, cumulative_fill, max_participation_rate })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(0.0, 10, None).is_none());
        assert!(compute(-100.0, 10, None).is_none());
        assert!(compute(100.0, 0, None).is_none());
        // Length mismatch when volume curve supplied.
        let v = vec![1000.0_f64; 5];
        assert!(compute(100.0, 10, Some(&v)).is_none());
        let mut bad = vec![1000.0_f64; 10];
        bad[0] = f64::NAN;
        assert!(compute(100.0, 10, Some(&bad)).is_none());
    }

    #[test]
    fn equal_slices_produced() {
        let r = compute(1000.0, 10, None).unwrap();
        for s in &r.slices { assert!((s - 100.0).abs() < 1e-9); }
    }

    #[test]
    fn cumulative_reaches_total() {
        let r = compute(1000.0, 10, None).unwrap();
        assert!((r.cumulative_fill[9] - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn max_participation_calculated_with_volume() {
        // 1000 order / 10 slices = 100 per slice. Volume vec: max 200 → rate 0.5.
        let v = vec![500.0, 500.0, 200.0, 1000.0, 1000.0, 500.0, 500.0, 500.0, 500.0, 500.0];
        let r = compute(1000.0, 10, Some(&v)).unwrap();
        assert!((r.max_participation_rate - 0.5).abs() < 1e-9);
    }

    #[test]
    fn zero_max_participation_without_volume() {
        let r = compute(1000.0, 10, None).unwrap();
        assert_eq!(r.max_participation_rate, 0.0);
    }

    #[test]
    fn single_slice_takes_full_order() {
        let r = compute(1000.0, 1, None).unwrap();
        assert!((r.slices[0] - 1000.0).abs() < 1e-9);
    }
}
