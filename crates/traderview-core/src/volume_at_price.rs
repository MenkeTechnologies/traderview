//! Volume at Price (Volume Profile) — distribution of traded volume
//! across price levels over a session.
//!
//! Each bar's volume is bucketed into `num_bins` evenly-spaced price
//! buckets between the session min(low) and max(high). Each bar's
//! volume contributes equally to every bucket its high-low range
//! overlaps (proportional to the overlap).
//!
//! Output:
//!   bins`[i]`.center: midpoint of bin i
//!   bins`[i]`.volume: cumulative volume in bin i
//!   poc_index: index of bin with highest volume (Point of Control)
//!   value_area_high / value_area_low: bin centers covering 70% of
//!     total volume centered on POC
//!
//! Pure compute. Default num_bins = 50, value_area_pct = 70.0.
//! Companion to `tpo_profile`, `developing_value_area` if shipped,
//! `cumulative_delta`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub volume: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriceBin {
    pub center: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeAtPriceReport {
    pub bins: Vec<PriceBin>,
    pub poc_index: Option<usize>,
    pub value_area_high: Option<f64>,
    pub value_area_low: Option<f64>,
    pub total_volume: f64,
}

pub fn compute(
    bars: &[Bar],
    num_bins: usize,
    value_area_pct: f64,
) -> VolumeAtPriceReport {
    let mut report = VolumeAtPriceReport::default();
    if bars.is_empty() || num_bins < 2
        || !value_area_pct.is_finite() || !(1.0..=99.9).contains(&value_area_pct) {
        return report;
    }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()
        || !b.volume.is_finite() || b.volume < 0.0 || b.high < b.low) {
        return report;
    }
    let min_price = bars.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
    let max_price = bars.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
    let bin_size = (max_price - min_price) / num_bins as f64;
    if bin_size <= 0.0 { return report; }
    let mut bin_volumes = vec![0.0_f64; num_bins];
    for bar in bars {
        let range = bar.high - bar.low;
        for (i, bv) in bin_volumes.iter_mut().enumerate() {
            let bin_low = min_price + bin_size * i as f64;
            let bin_high = bin_low + bin_size;
            let overlap = (bar.high.min(bin_high) - bar.low.max(bin_low)).max(0.0);
            if overlap > 0.0 && range > 0.0 {
                *bv += bar.volume * overlap / range;
            } else if range == 0.0 && bar.high >= bin_low && bar.high < bin_high {
                *bv += bar.volume;
            }
        }
    }
    let total_volume: f64 = bin_volumes.iter().sum();
    let bins: Vec<PriceBin> = bin_volumes.iter().enumerate().map(|(i, &v)| PriceBin {
        center: min_price + bin_size * (i as f64 + 0.5),
        volume: v,
    }).collect();
    let poc_idx = bins.iter().enumerate()
        .max_by(|a, b| a.1.volume.partial_cmp(&b.1.volume).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i);
    let target_vol = total_volume * value_area_pct / 100.0;
    let (va_low_idx, va_high_idx) = if let Some(poc) = poc_idx {
        let mut accum = bins[poc].volume;
        let mut lo = poc;
        let mut hi = poc;
        while accum < target_vol && (lo > 0 || hi + 1 < bins.len()) {
            let lo_vol = if lo > 0 { bins[lo - 1].volume } else { -1.0 };
            let hi_vol = if hi + 1 < bins.len() { bins[hi + 1].volume } else { -1.0 };
            if hi_vol >= lo_vol {
                if hi + 1 < bins.len() { hi += 1; accum += bins[hi].volume; }
                else if lo > 0 { lo -= 1; accum += bins[lo].volume; }
            } else if lo > 0 { lo -= 1; accum += bins[lo].volume; }
            else if hi + 1 < bins.len() { hi += 1; accum += bins[hi].volume; }
        }
        (Some(lo), Some(hi))
    } else { (None, None) };
    report.bins = bins;
    report.poc_index = poc_idx;
    report.value_area_low = va_low_idx.map(|i| report.bins[i].center);
    report.value_area_high = va_high_idx.map(|i| report.bins[i].center);
    report.total_volume = total_volume;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, v: f64) -> Bar { Bar { high: h, low: l, volume: v } }

    #[test]
    fn empty_or_invalid_returns_empty() {
        let r = compute(&[], 50, 70.0);
        assert!(r.bins.is_empty());
        let r2 = compute(&[b(101.0, 99.0, 1000.0); 10], 1, 70.0);
        assert!(r2.bins.is_empty());
        let r3 = compute(&[b(101.0, 99.0, 1000.0); 10], 50, 0.0);
        assert!(r3.bins.is_empty());
    }

    #[test]
    fn nan_or_invalid_returns_empty() {
        let bars = vec![b(101.0, 99.0, 1000.0), b(f64::NAN, 99.0, 1000.0)];
        let r = compute(&bars, 50, 70.0);
        assert!(r.bins.is_empty());
    }

    #[test]
    fn uniform_volume_distribution() {
        // All bars share same range; volume distributed evenly.
        let bars = vec![b(110.0, 100.0, 1000.0); 20];
        let r = compute(&bars, 10, 70.0);
        // POC could be any bin since all have equal volume.
        assert_eq!(r.bins.len(), 10);
        assert!(r.poc_index.is_some());
        // Total volume conserved (approximately).
        let total = r.bins.iter().map(|x| x.volume).sum::<f64>();
        assert!((total - 20000.0).abs() < 1e-6);
    }

    #[test]
    fn poc_at_high_volume_price() {
        // One bar with huge volume at 105-106, others at the edges.
        let bars = vec![
            b(101.0, 100.0, 100.0),
            b(106.0, 105.0, 50000.0),    // huge volume in middle
            b(111.0, 110.0, 100.0),
        ];
        let r = compute(&bars, 12, 70.0);
        assert!(r.poc_index.is_some());
        let poc_center = r.bins[r.poc_index.unwrap()].center;
        assert!((105.0..=106.0).contains(&poc_center));
    }

    #[test]
    fn value_area_brackets_poc() {
        let bars = vec![b(110.0, 100.0, 1000.0); 10];
        let r = compute(&bars, 10, 70.0);
        assert!(r.value_area_low.unwrap() <= r.value_area_high.unwrap());
    }

    #[test]
    fn total_volume_reported() {
        let bars = vec![b(110.0, 100.0, 1000.0); 5];
        let r = compute(&bars, 10, 70.0);
        assert!((r.total_volume - 5000.0).abs() < 1e-6);
    }
}
