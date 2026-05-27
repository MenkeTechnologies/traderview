//! ATR cone — projected price-range bands N days out.
//!
//! For a given entry price, ATR, and forward horizon, project the
//! expected ±1σ / ±2σ price band assuming returns are Brownian.
//! Square-root-of-time scaling: σ_N = ATR × √N.
//!
//! Helps the trader pick rational targets and stops:
//!   - "If I'm aiming for ±2σ in 5 days, I need a target ~$ATR × √5 × 2 away"
//!   - "If my stop is INSIDE 1σ, statistically I'll get stopped out by noise"
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConePoint {
    pub days_forward: usize,
    pub upper_2sd: f64,
    pub upper_1sd: f64,
    pub center: f64,
    pub lower_1sd: f64,
    pub lower_2sd: f64,
}

pub fn project(entry: f64, daily_atr: f64, horizon_days: usize) -> Vec<ConePoint> {
    let mut out = Vec::with_capacity(horizon_days + 1);
    for d in 0..=horizon_days {
        let sigma = daily_atr * (d as f64).sqrt();
        out.push(ConePoint {
            days_forward: d,
            upper_2sd: entry + 2.0 * sigma,
            upper_1sd: entry + sigma,
            center: entry,
            lower_1sd: entry - sigma,
            lower_2sd: entry - 2.0 * sigma,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_horizon_collapses_to_entry() {
        let out = project(100.0, 2.0, 0);
        assert_eq!(out.len(), 1);
        let p = &out[0];
        assert_eq!(p.upper_2sd, 100.0);
        assert_eq!(p.lower_2sd, 100.0);
        assert_eq!(p.center, 100.0);
    }

    #[test]
    fn day_one_sigma_equals_atr() {
        let out = project(100.0, 2.0, 1);
        let day1 = &out[1];
        // σ_1 = ATR × √1 = 2.
        assert_eq!(day1.upper_1sd, 102.0);
        assert_eq!(day1.lower_1sd, 98.0);
        assert_eq!(day1.upper_2sd, 104.0);
        assert_eq!(day1.lower_2sd, 96.0);
    }

    #[test]
    fn day_four_sigma_equals_atr_times_two() {
        // √4 = 2 → σ_4 = ATR × 2 = 4. Upper 1σ = 104.
        let out = project(100.0, 2.0, 4);
        assert_eq!(out[4].upper_1sd, 104.0);
        assert_eq!(out[4].lower_2sd, 92.0);
    }

    #[test]
    fn cone_widens_monotonically() {
        let out = project(100.0, 2.0, 10);
        for i in 1..out.len() {
            assert!(
                out[i].upper_2sd > out[i - 1].upper_2sd,
                "upper band must widen with horizon"
            );
            assert!(
                out[i].lower_2sd < out[i - 1].lower_2sd,
                "lower band must widen with horizon"
            );
        }
    }

    #[test]
    fn cone_symmetric_around_entry() {
        let out = project(100.0, 2.0, 10);
        for p in &out {
            let upper_offset = p.upper_2sd - p.center;
            let lower_offset = p.center - p.lower_2sd;
            assert!((upper_offset - lower_offset).abs() < 1e-12);
        }
    }

    #[test]
    fn zero_atr_yields_flat_cone() {
        // ATR=0 → all bands collapse to entry forever.
        let out = project(100.0, 0.0, 10);
        for p in &out {
            assert_eq!(p.upper_2sd, 100.0);
            assert_eq!(p.lower_2sd, 100.0);
        }
    }

    #[test]
    fn larger_atr_widens_cone_proportionally() {
        // Same horizon, ATR doubled → band offset doubles.
        let small = project(100.0, 1.0, 10);
        let big = project(100.0, 2.0, 10);
        for d in 0..=10 {
            let small_offset = small[d].upper_1sd - 100.0;
            let big_offset = big[d].upper_1sd - 100.0;
            if small_offset > 0.0 {
                assert!((big_offset / small_offset - 2.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn series_length_is_horizon_plus_one() {
        let out = project(100.0, 1.0, 30);
        assert_eq!(out.len(), 31);
        assert_eq!(out[0].days_forward, 0);
        assert_eq!(out[30].days_forward, 30);
    }
}
