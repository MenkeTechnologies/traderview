//! Intraday VWAP + N-stdev bands.
//!
//! Canonical mean-reversion reference for intraday trading. As volume
//! accumulates through the session, the VWAP and its variance update.
//! Traders fade extremes ±1σ / ±2σ from VWAP back toward the mean.
//!
//! Streaming computation — caller feeds bars in order, gets running
//! (vwap, upper_1, lower_1, upper_2, lower_2) at each step.
//!
//! Pure compute. Reset between sessions by re-instantiating the
//! accumulator.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct VwapBandsAccumulator {
    /// Sum of (typical_price × volume).
    sum_pv: f64,
    /// Sum of volume.
    sum_v: f64,
    /// Sum of (typical_price^2 × volume) — for variance.
    sum_p2v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VwapSnapshot {
    pub vwap: f64,
    pub upper_band_1sd: f64,
    pub lower_band_1sd: f64,
    pub upper_band_2sd: f64,
    pub lower_band_2sd: f64,
    pub upper_band_3sd: f64,
    pub lower_band_3sd: f64,
}

impl VwapBandsAccumulator {
    pub fn new() -> Self { Self::default() }

    /// Feed a new bar. `typical` is usually (H+L+C)/3.
    pub fn update(&mut self, typical: f64, volume: f64) {
        let v = volume.max(0.0);
        self.sum_pv += typical * v;
        self.sum_v += v;
        self.sum_p2v += typical * typical * v;
    }

    pub fn snapshot(&self) -> VwapSnapshot {
        if self.sum_v == 0.0 { return VwapSnapshot::default(); }
        let vwap = self.sum_pv / self.sum_v;
        let var = (self.sum_p2v / self.sum_v) - vwap * vwap;
        let sd = var.max(0.0).sqrt();
        VwapSnapshot {
            vwap,
            upper_band_1sd: vwap + sd,
            lower_band_1sd: vwap - sd,
            upper_band_2sd: vwap + 2.0 * sd,
            lower_band_2sd: vwap - 2.0 * sd,
            upper_band_3sd: vwap + 3.0 * sd,
            lower_band_3sd: vwap - 3.0 * sd,
        }
    }
}

/// Convenience: rolls a fresh accumulator across `bars` and returns the
/// final snapshot only.
pub fn final_snapshot(bars: &[(f64, f64)]) -> VwapSnapshot {
    let mut acc = VwapBandsAccumulator::new();
    for (t, v) in bars { acc.update(*t, *v); }
    acc.snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_accumulator_returns_zeros() {
        let acc = VwapBandsAccumulator::new();
        let s = acc.snapshot();
        assert_eq!(s.vwap, 0.0);
        assert_eq!(s.upper_band_1sd, 0.0);
    }

    #[test]
    fn single_bar_vwap_equals_typical_zero_stdev() {
        let mut acc = VwapBandsAccumulator::new();
        acc.update(100.0, 1000.0);
        let s = acc.snapshot();
        assert_eq!(s.vwap, 100.0);
        // Zero variance with one bar → bands collapse to vwap.
        assert_eq!(s.upper_band_1sd, 100.0);
        assert_eq!(s.lower_band_1sd, 100.0);
    }

    #[test]
    fn vwap_volume_weights_correctly() {
        let mut acc = VwapBandsAccumulator::new();
        acc.update(100.0, 100.0);
        acc.update(110.0, 900.0);
        // VWAP = (100×100 + 110×900) / 1000 = (10000 + 99000) / 1000 = 109.
        assert_eq!(acc.snapshot().vwap, 109.0);
    }

    #[test]
    fn bands_widen_as_price_dispersion_increases() {
        // Bar 1: 100, Bar 2: 100 → no dispersion.
        let mut acc1 = VwapBandsAccumulator::new();
        acc1.update(100.0, 1.0);
        acc1.update(100.0, 1.0);
        let s1 = acc1.snapshot();
        assert_eq!(s1.upper_band_1sd - s1.lower_band_1sd, 0.0);

        // Bar 1: 100, Bar 2: 110 → spread → bands open.
        let mut acc2 = VwapBandsAccumulator::new();
        acc2.update(100.0, 1.0);
        acc2.update(110.0, 1.0);
        let s2 = acc2.snapshot();
        assert!(s2.upper_band_1sd - s2.lower_band_1sd > 0.0);
    }

    #[test]
    fn bands_symmetric_around_vwap() {
        let mut acc = VwapBandsAccumulator::new();
        for t in [98.0, 100.0, 102.0, 99.0, 101.0] {
            acc.update(t, 1.0);
        }
        let s = acc.snapshot();
        // (upper + lower) / 2 == vwap.
        assert!(((s.upper_band_1sd + s.lower_band_1sd) / 2.0 - s.vwap).abs() < 1e-9);
        assert!(((s.upper_band_2sd + s.lower_band_2sd) / 2.0 - s.vwap).abs() < 1e-9);
    }

    #[test]
    fn two_sd_band_is_twice_one_sd_offset() {
        let mut acc = VwapBandsAccumulator::new();
        for t in [98.0, 102.0] {
            acc.update(t, 1.0);
        }
        let s = acc.snapshot();
        let offset_1 = s.upper_band_1sd - s.vwap;
        let offset_2 = s.upper_band_2sd - s.vwap;
        let offset_3 = s.upper_band_3sd - s.vwap;
        assert!((offset_2 / offset_1 - 2.0).abs() < 1e-9);
        assert!((offset_3 / offset_1 - 3.0).abs() < 1e-9);
    }

    #[test]
    fn negative_volume_clamped_to_zero() {
        let mut acc = VwapBandsAccumulator::new();
        acc.update(100.0, -100.0);    // ignored
        acc.update(100.0, 1000.0);
        assert_eq!(acc.snapshot().vwap, 100.0);
    }

    #[test]
    fn final_snapshot_convenience_matches_manual_roll() {
        let bars = vec![(100.0, 1.0), (105.0, 2.0), (110.0, 3.0)];
        let convenience = final_snapshot(&bars);
        let mut acc = VwapBandsAccumulator::new();
        for (t, v) in &bars { acc.update(*t, *v); }
        let manual = acc.snapshot();
        assert_eq!(convenience.vwap, manual.vwap);
        assert_eq!(convenience.upper_band_2sd, manual.upper_band_2sd);
    }
}
