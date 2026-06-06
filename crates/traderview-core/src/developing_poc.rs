//! Developing Point of Control (POC) — intraday POC tracker.
//!
//! Lightweight companion to `developing_value_area`. For each bar
//! emits the running POC and the bin's accumulated volume. Useful as
//! a single-line chart overlay (the "POC line") that shows where
//! intraday volume is most concentrated as the session develops.
//!
//! Pure compute. Default num_bins = 50.
//! Companion to `volume_at_price`, `developing_value_area`,
//! `session_vwap`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct DevelopingPocPoint {
    pub poc_price: Option<f64>,
    pub poc_volume: f64,
    pub total_volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevelopingPocReport {
    pub per_bar: Vec<DevelopingPocPoint>,
    pub num_bins: usize,
}

pub fn compute(bars: &[Bar], num_bins: usize) -> DevelopingPocReport {
    let n = bars.len();
    let mut report = DevelopingPocReport {
        per_bar: vec![DevelopingPocPoint::default(); n],
        num_bins,
    };
    if n == 0 || num_bins < 2 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.high.is_finite()
            || !b.low.is_finite()
            || !b.volume.is_finite()
            || b.volume < 0.0
            || b.high < b.low
    }) {
        return report;
    }
    let min_price = bars.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
    let max_price = bars.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
    let bin_size = (max_price - min_price) / num_bins as f64;
    if bin_size <= 0.0 {
        return report;
    }
    let mut bin_volumes = vec![0.0_f64; num_bins];
    let mut cumulative = 0.0_f64;
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        for (j, bv) in bin_volumes.iter_mut().enumerate() {
            let bin_low = min_price + bin_size * j as f64;
            let bin_high = bin_low + bin_size;
            let overlap = (bar.high.min(bin_high) - bar.low.max(bin_low)).max(0.0);
            if overlap > 0.0 && range > 0.0 {
                *bv += bar.volume * overlap / range;
            } else if range == 0.0 && bar.high >= bin_low && bar.high < bin_high {
                *bv += bar.volume;
            }
        }
        cumulative += bar.volume;
        if let Some((poc_idx, &poc_vol)) = bin_volumes
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            report.per_bar[i] = DevelopingPocPoint {
                poc_price: Some(min_price + bin_size * (poc_idx as f64 + 0.5)),
                poc_volume: poc_vol,
                total_volume: cumulative,
            };
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, v: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            volume: v,
        }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        let r = compute(&[], 50);
        assert!(r.per_bar.is_empty());
        let bars = vec![b(101.0, 99.0, 1000.0); 5];
        let r2 = compute(&bars, 1);
        assert!(r2.per_bar.iter().all(|p| p.poc_price.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![b(101.0, 99.0, 1000.0), b(f64::NAN, 99.0, 1000.0)];
        let r = compute(&bars, 50);
        assert!(r.per_bar.iter().all(|p| p.poc_price.is_none()));
    }

    #[test]
    fn poc_developed_per_bar() {
        let bars = vec![
            b(101.0, 100.0, 1000.0),
            b(105.0, 100.0, 5000.0),
            b(110.0, 105.0, 1000.0),
        ];
        let r = compute(&bars, 10);
        assert!(r.per_bar.iter().all(|p| p.poc_price.is_some()));
    }

    #[test]
    fn poc_shifts_to_highest_volume_zone() {
        // Final POC should be near the highest-volume strike.
        let bars = vec![
            b(101.0, 100.0, 100.0),
            b(106.0, 105.0, 5000.0),
            b(111.0, 110.0, 100.0),
        ];
        let r = compute(&bars, 12);
        let last = r.per_bar.last().unwrap();
        assert!(last.poc_price.is_some());
        let poc = last.poc_price.unwrap();
        assert!((105.0..=106.0).contains(&poc));
    }

    #[test]
    fn total_volume_grows_monotonically() {
        let bars = vec![b(110.0, 100.0, 1000.0); 5];
        let r = compute(&bars, 10);
        let vols: Vec<f64> = r.per_bar.iter().map(|p| p.total_volume).collect();
        for w in vols.windows(2) {
            assert!(w[1] >= w[0]);
        }
    }
}
