//! Bermudan option — early exercise allowed only on a discrete set of
//! dates (subset of [0, T]). Cox-Ross-Rubinstein binomial backward
//! induction with exercise-permission only at requested step indices.
//!
//! Bermudan is a strict generalization:
//!   - Empty exercise schedule = European option
//!   - Every step in schedule = American option (limit case)
//!   - Typical case: quarterly or annual exercise dates
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind { Call, Put }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BermudanReport {
    pub price: f64,
    pub n_steps: usize,
    pub n_exercise_dates: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    spot: f64, strike: f64,
    time_to_expiry: f64,
    risk_free: f64, dividend_yield: f64,
    sigma: f64,
    n_steps: usize,
    exercise_dates_years: &[f64],
    kind: OptionKind,
) -> Option<BermudanReport> {
    if !spot.is_finite() || spot <= 0.0
        || !strike.is_finite() || strike <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free.is_finite() || !dividend_yield.is_finite()
        || !sigma.is_finite() || sigma <= 0.0
        || !(1..=5_000).contains(&n_steps)
    {
        return None;
    }
    if exercise_dates_years.iter().any(|t| !t.is_finite() || *t <= 0.0 || *t > time_to_expiry) {
        return None;
    }
    let dt = time_to_expiry / n_steps as f64;
    let u = (sigma * dt.sqrt()).exp();
    let d = 1.0 / u;
    let disc = (-risk_free * dt).exp();
    let drift = ((risk_free - dividend_yield) * dt).exp();
    let p = (drift - d) / (u - d);
    if !(0.0..=1.0).contains(&p) || !p.is_finite() {
        return None;
    }
    let q = 1.0 - p;
    // Map exercise dates to nearest step indices.
    let mut exercise_steps: Vec<usize> = exercise_dates_years.iter()
        .map(|t| ((t / dt).round() as isize).clamp(1, n_steps as isize) as usize)
        .collect();
    exercise_steps.sort();
    exercise_steps.dedup();
    let exercise_at_step = |s: usize| -> bool {
        exercise_steps.binary_search(&s).is_ok()
    };
    let mut values = vec![0.0_f64; n_steps + 1];
    for (j, slot) in values.iter_mut().enumerate() {
        let s = spot * u.powi((n_steps as i32) - (j as i32) * 2);
        let intrinsic = match kind {
            OptionKind::Call => (s - strike).max(0.0),
            OptionKind::Put  => (strike - s).max(0.0),
        };
        *slot = intrinsic;
    }
    for step in (0..n_steps).rev() {
        let can_exercise = exercise_at_step(step);
        for j in 0..=step {
            let s = spot * u.powi((step as i32) - (j as i32) * 2);
            let continuation = disc * (p * values[j] + q * values[j + 1]);
            if can_exercise {
                let intrinsic = match kind {
                    OptionKind::Call => (s - strike).max(0.0),
                    OptionKind::Put  => (strike - s).max(0.0),
                };
                values[j] = continuation.max(intrinsic);
            } else {
                values[j] = continuation;
            }
        }
    }
    Some(BermudanReport {
        price: values[0],
        n_steps,
        n_exercise_dates: exercise_steps.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 0.5, 0.05, 0.0, 0.2, 200, &[0.25],
                OptionKind::Put).is_none());
            assert!(price(100.0, bad, 0.5, 0.05, 0.0, 0.2, 200, &[0.25],
                OptionKind::Put).is_none());
            assert!(price(100.0, 100.0, bad, 0.05, 0.0, 0.2, 200, &[0.25],
                OptionKind::Put).is_none());
            assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, bad, 200, &[0.25],
                OptionKind::Put).is_none());
        }
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.2, 0, &[0.25],
            OptionKind::Put).is_none());
        // Exercise after expiry.
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.2, 200, &[1.0],
            OptionKind::Put).is_none());
    }

    #[test]
    fn empty_exercise_schedule_collapses_to_european() {
        // Bermudan with no early-exercise dates = European → expect
        // result close to BS European put.
        let r = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.20, 500, &[],
            OptionKind::Put).unwrap();
        assert!(r.price > 0.0);
        assert_eq!(r.n_exercise_dates, 0);
    }

    #[test]
    fn full_grid_exercise_approximates_american() {
        let n = 200;
        let all_steps: Vec<f64> = (1..=n).map(|i| (i as f64) / n as f64).collect();
        let r_bermudan_full = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.20, n, &all_steps,
            OptionKind::Put).unwrap();
        let european = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.20, n, &[],
            OptionKind::Put).unwrap();
        // Full grid → ≥ European; this IS the American limit.
        assert!(r_bermudan_full.price > european.price);
    }

    #[test]
    fn more_exercise_dates_yield_higher_or_equal_price() {
        // Adding exercise dates can never DECREASE the option's value
        // (early exercise is optional; holder picks the larger).
        let sparse = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.20, 200,
            &[0.5], OptionKind::Put).unwrap();
        let dense = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.20, 200,
            &[0.25, 0.5, 0.75], OptionKind::Put).unwrap();
        assert!(dense.price >= sparse.price - 1e-9);
    }

    #[test]
    fn longer_expiry_inflates_bermudan_put() {
        let r_short = price(100.0, 100.0, 0.25, 0.05, 0.0, 0.20, 200,
            &[0.125], OptionKind::Put).unwrap();
        let r_long  = price(100.0, 100.0, 1.00, 0.05, 0.0, 0.20, 200,
            &[0.5], OptionKind::Put).unwrap();
        assert!(r_long.price > r_short.price);
    }

    #[test]
    fn higher_vol_inflates_bermudan_call() {
        let r_low  = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.10, 200,
            &[0.5], OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.40, 200,
            &[0.5], OptionKind::Call).unwrap();
        assert!(r_high.price > r_low.price);
    }
}
