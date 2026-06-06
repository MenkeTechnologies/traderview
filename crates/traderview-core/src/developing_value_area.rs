//! Developing Value Area — intraday VAH/VAL/POC computed incrementally
//! as the session progresses.
//!
//! Per-bar output of (POC, value_area_high, value_area_low) so a
//! charting tool can display how the value area develops across the
//! trading session. Final values match `volume_at_price` POC + VA.
//!
//! Uses same algorithm as `volume_at_price`: bucket by price, accumulate
//! volume proportional to bar's high-low overlap with each bucket,
//! widen out from POC until cumulative volume reaches `value_area_pct`.
//!
//! Pure compute. Defaults: num_bins = 50, value_area_pct = 70.0.
//! Companion to `volume_at_price`, `tpo_profile`, `session_vwap`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct DevelopingPoint {
    pub poc: Option<f64>,
    pub value_area_high: Option<f64>,
    pub value_area_low: Option<f64>,
    pub total_volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevelopingValueAreaReport {
    pub per_bar: Vec<DevelopingPoint>,
    pub num_bins: usize,
    pub value_area_pct: f64,
}

pub fn compute(bars: &[Bar], num_bins: usize, value_area_pct: f64) -> DevelopingValueAreaReport {
    let n = bars.len();
    let mut report = DevelopingValueAreaReport {
        per_bar: vec![DevelopingPoint::default(); n],
        num_bins,
        value_area_pct,
    };
    if n == 0
        || num_bins < 2
        || !value_area_pct.is_finite()
        || !(1.0..=99.9).contains(&value_area_pct)
    {
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
    // Session-wide bin range fixed by overall min/max for stability.
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
        // Find POC.
        let poc_idx = bin_volumes
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);
        if let Some(poc) = poc_idx {
            // Widen out from POC to capture target value-area volume.
            let target = cumulative * value_area_pct / 100.0;
            let mut accum = bin_volumes[poc];
            let mut lo = poc;
            let mut hi = poc;
            while accum < target && (lo > 0 || hi + 1 < bin_volumes.len()) {
                let lo_vol = if lo > 0 { bin_volumes[lo - 1] } else { -1.0 };
                let hi_vol = if hi + 1 < bin_volumes.len() {
                    bin_volumes[hi + 1]
                } else {
                    -1.0
                };
                if hi_vol >= lo_vol {
                    if hi + 1 < bin_volumes.len() {
                        hi += 1;
                        accum += bin_volumes[hi];
                    } else if lo > 0 {
                        lo -= 1;
                        accum += bin_volumes[lo];
                    }
                } else if lo > 0 {
                    lo -= 1;
                    accum += bin_volumes[lo];
                } else if hi + 1 < bin_volumes.len() {
                    hi += 1;
                    accum += bin_volumes[hi];
                }
            }
            report.per_bar[i] = DevelopingPoint {
                poc: Some(min_price + bin_size * (poc as f64 + 0.5)),
                value_area_low: Some(min_price + bin_size * (lo as f64 + 0.5)),
                value_area_high: Some(min_price + bin_size * (hi as f64 + 0.5)),
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
        let r = compute(&[], 50, 70.0);
        assert!(r.per_bar.is_empty());
        let bars = vec![b(101.0, 99.0, 1000.0); 5];
        let r2 = compute(&bars, 1, 70.0);
        assert!(r2.per_bar.iter().all(|p| p.poc.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![b(101.0, 99.0, 1000.0), b(f64::NAN, 99.0, 1000.0)];
        let r = compute(&bars, 50, 70.0);
        assert!(r.per_bar.iter().all(|p| p.poc.is_none()));
    }

    #[test]
    fn poc_developed_per_bar() {
        let bars = vec![
            b(101.0, 100.0, 1000.0),
            b(105.0, 100.0, 5000.0), // higher volume at higher prices
            b(110.0, 105.0, 1000.0),
        ];
        let r = compute(&bars, 10, 70.0);
        // All bars should have POC defined.
        assert!(r.per_bar.iter().all(|p| p.poc.is_some()));
    }

    #[test]
    fn value_area_brackets_poc() {
        let bars = vec![b(110.0, 100.0, 1000.0); 10];
        let r = compute(&bars, 10, 70.0);
        for p in &r.per_bar {
            if let (Some(poc), Some(vah), Some(val)) = (p.poc, p.value_area_high, p.value_area_low)
            {
                assert!(val <= poc + 1e-9 && poc <= vah + 1e-9);
            }
        }
    }

    #[test]
    fn total_volume_grows_monotonically() {
        let bars = vec![b(110.0, 100.0, 1000.0); 5];
        let r = compute(&bars, 10, 70.0);
        let vols: Vec<f64> = r.per_bar.iter().map(|p| p.total_volume).collect();
        for w in vols.windows(2) {
            assert!(w[1] >= w[0]);
        }
        assert!((vols[4] - 5000.0).abs() < 1e-6);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 1000.0); 30];
        let r = compute(&bars, 50, 70.0);
        assert_eq!(r.per_bar.len(), 30);
    }
}
