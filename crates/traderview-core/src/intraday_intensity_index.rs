//! Intraday Intensity Index (III) — David Bostian.
//!
//! Volume-weighted measure of where the close lies relative to the
//! bar's high-low range, with downward bias when the close is closer
//! to the low:
//!
//!   III_t = ((2·close - high - low) / (high - low)) · volume
//!
//! Range: scaled by per-bar volume. Output may be reported per bar or
//! cumulatively. A 21-bar SMA "Smart-Money Index" style cumulative
//! version is what most platforms display.
//!
//! Interpretation:
//!   - rising cumulative III → buying pressure intraday
//!   - falling cumulative III → selling pressure
//!   - divergence with price → trend exhaustion
//!
//! Pure compute. Companion to `accumulation_distribution_line`,
//! `chaikin_money_flow`, `mfi`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64, pub volume: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IiiReport {
    pub per_bar: Vec<Option<f64>>,
    pub cumulative: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar]) -> IiiReport {
    let n = bars.len();
    let mut report = IiiReport {
        per_bar: vec![None; n],
        cumulative: vec![None; n],
    };
    if n == 0 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()
        || !b.close.is_finite() || !b.volume.is_finite()) { return report; }
    let mut cum = 0.0_f64;
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        let per = if range > 0.0 {
            ((2.0 * bar.close - bar.high - bar.low) / range) * bar.volume
        } else {
            0.0
        };
        report.per_bar[i] = Some(per);
        cum += per;
        report.cumulative[i] = Some(cum);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[]);
        assert!(r.per_bar.is_empty());
        assert!(r.cumulative.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![b(101.0, 99.0, 100.0, f64::NAN)];
        let r = compute(&bars);
        assert!(r.per_bar.iter().all(|x| x.is_none()));
    }

    #[test]
    fn close_at_high_yields_plus_volume() {
        let bars = vec![b(110.0, 100.0, 110.0, 1000.0)];
        let r = compute(&bars);
        // (220 - 110 - 100) / 10 · 1000 = 1·1000 = 1000.
        assert!((r.per_bar[0].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn close_at_low_yields_minus_volume() {
        let bars = vec![b(110.0, 100.0, 100.0, 1000.0)];
        let r = compute(&bars);
        assert!((r.per_bar[0].unwrap() + 1000.0).abs() < 1e-9);
    }

    #[test]
    fn close_at_mid_yields_zero() {
        let bars = vec![b(110.0, 100.0, 105.0, 1000.0)];
        let r = compute(&bars);
        assert!(r.per_bar[0].unwrap().abs() < 1e-9);
    }

    #[test]
    fn zero_range_bar_yields_zero() {
        let bars = vec![b(100.0, 100.0, 100.0, 1000.0)];
        let r = compute(&bars);
        assert!(r.per_bar[0].unwrap().abs() < 1e-9);
    }

    #[test]
    fn cumulative_correct_three_bar() {
        let bars = vec![
            b(110.0, 100.0, 110.0, 1000.0),    // +1000
            b(110.0, 100.0, 100.0, 1000.0),    // -1000
            b(110.0, 100.0, 110.0, 2000.0),    // +2000
        ];
        let r = compute(&bars);
        assert!((r.cumulative[0].unwrap() - 1000.0).abs() < 1e-9);
        assert!((r.cumulative[1].unwrap()).abs() < 1e-9);
        assert!((r.cumulative[2].unwrap() - 2000.0).abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars);
        assert_eq!(r.per_bar.len(), 30);
        assert_eq!(r.cumulative.len(), 30);
    }
}
