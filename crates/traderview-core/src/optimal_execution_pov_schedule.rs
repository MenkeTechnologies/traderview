//! Percentage-of-Volume (POV) Execution Scheduler — generates a
//! per-bar child-order schedule that targets a fixed participation
//! rate of expected market volume.
//!
//! Inputs:
//!   total_order_size   = shares to execute over the session
//!   volume_curve       = expected per-bar volume profile (e.g.
//!                        20-day average per-bar volume in shares)
//!   participation_rate = target fraction of volume to take (0 to 1)
//!
//! Per bar: slice_i = min(participation_rate · volume_i, remaining).
//! Stops when total filled = total_order_size. If the curve can't
//! absorb the order at the requested participation, the residual is
//! left unscheduled (caller decides — extend horizon, lift cap, etc.).
//!
//! Reports per-bar slice sizes, cumulative fill, completion bar
//! (None if not completed), and shortfall (unscheduled remainder).
//!
//! Pure compute. Companion to `implementation_shortfall`,
//! `almgren_chriss`, `vwap_arrival_price`.

#[derive(Debug)]
pub struct Report {
    pub slices: Vec<f64>,
    pub cumulative_fill: Vec<f64>,
    pub completion_bar: Option<usize>,
    pub shortfall: f64,
}

pub fn compute(
    total_order_size: f64,
    volume_curve: &[f64],
    participation_rate: f64,
) -> Option<Report> {
    if !total_order_size.is_finite() || total_order_size <= 0.0 { return None; }
    if !participation_rate.is_finite() || !(0.0..=1.0).contains(&participation_rate) {
        return None;
    }
    if volume_curve.is_empty() { return None; }
    if volume_curve.iter().any(|x| !x.is_finite() || *x < 0.0) { return None; }
    let n = volume_curve.len();
    let mut slices = vec![0.0_f64; n];
    let mut cumulative_fill = vec![0.0_f64; n];
    let mut filled = 0.0_f64;
    let mut completion_bar = None;
    for i in 0..n {
        let want = participation_rate * volume_curve[i];
        let remaining = total_order_size - filled;
        let slice = want.min(remaining).max(0.0);
        slices[i] = slice;
        filled += slice;
        cumulative_fill[i] = filled;
        if completion_bar.is_none() && filled >= total_order_size - 1e-9 {
            completion_bar = Some(i);
        }
    }
    let shortfall = (total_order_size - filled).max(0.0);
    Some(Report { slices, cumulative_fill, completion_bar, shortfall })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let v = vec![1000.0_f64; 10];
        assert!(compute(0.0, &v, 0.1).is_none());
        assert!(compute(-100.0, &v, 0.1).is_none());
        assert!(compute(100.0, &v, -0.1).is_none());
        assert!(compute(100.0, &v, 1.5).is_none());
        assert!(compute(100.0, &[], 0.1).is_none());
        let mut bad = v.clone();
        bad[0] = f64::NAN;
        assert!(compute(100.0, &bad, 0.1).is_none());
        let mut neg = v.clone();
        neg[0] = -1.0;
        assert!(compute(100.0, &neg, 0.1).is_none());
    }

    #[test]
    fn slices_respect_participation_rate() {
        // Order 100, volume 1000/bar, participation 10% → 100/bar.
        let v = vec![1000.0_f64; 10];
        let r = compute(100.0, &v, 0.1).unwrap();
        // First bar should fill 100 then stop.
        assert!((r.slices[0] - 100.0).abs() < 1e-9);
        for v in &r.slices[1..] { assert_eq!(*v, 0.0); }
        assert_eq!(r.completion_bar, Some(0));
        assert_eq!(r.shortfall, 0.0);
    }

    #[test]
    fn small_order_completes_quickly() {
        let v = vec![1000.0_f64; 10];
        let r = compute(50.0, &v, 0.1).unwrap();
        assert!((r.slices[0] - 50.0).abs() < 1e-9);
        assert_eq!(r.completion_bar, Some(0));
    }

    #[test]
    fn large_order_partially_filled() {
        // Order 10_000, volume 1000/bar × 10 bars at 10% = 100/bar
        // = 1000 total executable. Shortfall = 9000.
        let v = vec![1000.0_f64; 10];
        let r = compute(10_000.0, &v, 0.1).unwrap();
        assert!((r.shortfall - 9000.0).abs() < 1e-6);
        assert!(r.completion_bar.is_none());
    }

    #[test]
    fn nonuniform_volume_curve_followed() {
        let v = vec![500.0, 1000.0, 2000.0, 1000.0, 500.0];
        let r = compute(450.0, &v, 0.1).unwrap();
        // Bar 0: 50, bar 1: 100, bar 2: 200, bar 3: 100, bar 4: 0
        // (450 total filled after bar 3).
        assert!((r.slices[0] - 50.0).abs() < 1e-9);
        assert!((r.slices[1] - 100.0).abs() < 1e-9);
        assert!((r.slices[2] - 200.0).abs() < 1e-9);
        assert!((r.slices[3] - 100.0).abs() < 1e-9);
        assert_eq!(r.slices[4], 0.0);
        assert_eq!(r.completion_bar, Some(3));
    }

    #[test]
    fn cumulative_fill_is_monotone() {
        let v = vec![500.0, 1000.0, 2000.0, 1000.0, 500.0];
        let r = compute(450.0, &v, 0.1).unwrap();
        for w in r.cumulative_fill.windows(2) {
            assert!(w[1] >= w[0]);
        }
    }

    #[test]
    fn full_participation_takes_all_volume() {
        let v = vec![100.0, 100.0, 100.0];
        let r = compute(300.0, &v, 1.0).unwrap();
        for s in &r.slices { assert!((s - 100.0).abs() < 1e-9); }
        assert_eq!(r.completion_bar, Some(2));
    }

    #[test]
    fn zero_volume_bars_contribute_no_slice() {
        let v = vec![0.0, 100.0, 0.0, 100.0];
        let r = compute(20.0, &v, 0.1).unwrap();
        assert_eq!(r.slices[0], 0.0);
        assert_eq!(r.slices[2], 0.0);
    }
}
