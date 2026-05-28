//! Equivolume Bars — Richard Arms' chart where each bar's WIDTH is
//! proportional to its volume.
//!
//! Pure-compute helper: given an input series of bars and a desired
//! total chart width, returns the per-bar normalized width (sums to
//! `total_width`) and a "volume box" tag classifying each bar:
//!
//!   Narrow:  width ≤ 0.5 × average bar width
//!   Normal:  0.5x..1.5x average
//!   Wide:    > 1.5x average
//!   Power:   > 1.5x average AND range > 1.5x average range
//!     (high volume + high range = strong conviction)
//!
//! Pure compute. Companion to `volume_burst`, `vsa`, `weiss_wave`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub volume: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EquivolumeKind {
    #[default]
    Normal,
    Narrow,
    Wide,
    Power,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquivolumeReport {
    pub widths: Vec<f64>,
    pub kinds: Vec<EquivolumeKind>,
    pub avg_volume: f64,
    pub avg_range: f64,
    pub total_width: f64,
}

pub fn compute(bars: &[Bar], total_width: f64) -> EquivolumeReport {
    let n = bars.len();
    let mut report = EquivolumeReport {
        widths: vec![0.0; n],
        kinds: vec![EquivolumeKind::Normal; n],
        ..Default::default()
    };
    if n == 0 || !total_width.is_finite() || total_width <= 0.0 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()
        || !b.volume.is_finite() || b.volume < 0.0 || b.high < b.low) {
        return report;
    }
    report.total_width = total_width;
    let total_vol: f64 = bars.iter().map(|b| b.volume).sum();
    if total_vol <= 0.0 { return report; }
    for (i, bar) in bars.iter().enumerate() {
        report.widths[i] = bar.volume / total_vol * total_width;
    }
    let avg_vol = total_vol / n as f64;
    let avg_range: f64 = bars.iter().map(|b| b.high - b.low).sum::<f64>() / n as f64;
    report.avg_volume = avg_vol;
    report.avg_range = avg_range;
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        let big_vol = bar.volume > avg_vol * 1.5;
        let big_range = avg_range > 0.0 && range > avg_range * 1.5;
        report.kinds[i] = if big_vol && big_range {
            EquivolumeKind::Power
        } else if big_vol {
            EquivolumeKind::Wide
        } else if bar.volume <= avg_vol * 0.5 {
            EquivolumeKind::Narrow
        } else {
            EquivolumeKind::Normal
        };
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, v: f64) -> Bar { Bar { high: h, low: l, volume: v } }

    #[test]
    fn empty_or_invalid_returns_empty() {
        let r = compute(&[], 100.0);
        assert!(r.widths.is_empty());
        let bars = vec![b(101.0, 99.0, 1000.0); 5];
        let r2 = compute(&bars, 0.0);
        assert!(r2.widths.iter().all(|&w| w == 0.0));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![b(101.0, 99.0, 1000.0), b(f64::NAN, 99.0, 1000.0)];
        let r = compute(&bars, 100.0);
        assert!(r.widths.iter().all(|&w| w == 0.0));
    }

    #[test]
    fn widths_sum_to_total() {
        let bars = vec![b(101.0, 99.0, 1000.0); 10];
        let r = compute(&bars, 100.0);
        let sum: f64 = r.widths.iter().sum();
        assert!((sum - 100.0).abs() < 1e-6);
    }

    #[test]
    fn proportional_widths() {
        let bars = vec![
            b(101.0, 99.0, 1000.0),
            b(101.0, 99.0, 3000.0),
        ];
        let r = compute(&bars, 40.0);
        // Bar 0: 1000/4000 · 40 = 10. Bar 1: 3000/4000 · 40 = 30.
        assert!((r.widths[0] - 10.0).abs() < 1e-6);
        assert!((r.widths[1] - 30.0).abs() < 1e-6);
    }

    #[test]
    fn high_volume_classified_wide_or_power() {
        let mut bars = vec![b(101.0, 99.0, 1000.0); 9];
        bars.push(b(115.0, 95.0, 5000.0));    // 5x avg vol, wide range
        let r = compute(&bars, 100.0);
        assert!(matches!(r.kinds[9],
            EquivolumeKind::Wide | EquivolumeKind::Power));
    }

    #[test]
    fn low_volume_classified_narrow() {
        let mut bars = vec![b(101.0, 99.0, 2000.0); 9];
        bars.push(b(101.0, 99.0, 100.0));
        let r = compute(&bars, 100.0);
        assert_eq!(r.kinds[9], EquivolumeKind::Narrow);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 1000.0); 10];
        let r = compute(&bars, 100.0);
        assert_eq!(r.widths.len(), 10);
        assert_eq!(r.kinds.len(), 10);
    }
}
