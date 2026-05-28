//! Bill Williams Fractals (5-bar pivot).
//!
//! A "fractal" is a 5-bar pattern where the middle bar's high is the
//! highest of the 5 (up fractal) or low is the lowest of the 5 (down
//! fractal). Used by the Alligator system as price-action confirmation
//! and as raw input for trend-line / support-resistance overlays.
//!
//! Distinct from `swing_points` (configurable N-bar pivot detector) —
//! fractals are STRICTLY 5-bar and follow Bill Williams's specific
//! convention (the middle bar must be the strict extreme of the window).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FractalKind { Up, Down }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Fractal {
    pub bar_index: usize,
    pub kind: FractalKind,
    pub price: f64,
}

pub fn detect(bars: &[Bar]) -> Vec<Fractal> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 5 {
        return out;
    }
    for i in 2..n.saturating_sub(2) {
        let center = bars[i];
        let h = center.high;
        let l = center.low;
        let n2 = bars[i - 2];
        let n1 = bars[i - 1];
        let p1 = bars[i + 1];
        let p2 = bars[i + 2];
        // Up fractal: middle strictly highest.
        if h.is_finite()
            && h > n2.high && h > n1.high
            && h > p1.high && h > p2.high
        {
            out.push(Fractal { bar_index: i, kind: FractalKind::Up, price: h });
        }
        // Down fractal: middle strictly lowest.
        if l.is_finite()
            && l < n2.low && l < n1.low
            && l < p1.low && l < p2.low
        {
            out.push(Fractal { bar_index: i, kind: FractalKind::Down, price: l });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_or_too_short_returns_empty() {
        assert!(detect(&[]).is_empty());
        assert!(detect(&[b(101.0, 99.0); 4]).is_empty());
    }

    #[test]
    fn up_fractal_detected_at_middle_high() {
        // Pyramid: highs rise to middle then fall.
        let bars = vec![
            b(100.0, 99.0),
            b(101.0, 100.0),
            b(105.0, 100.0),    // middle — highest
            b(102.0, 100.0),
            b(99.0, 98.0),
        ];
        let f = detect(&bars);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, FractalKind::Up);
        assert_eq!(f[0].bar_index, 2);
        assert_eq!(f[0].price, 105.0);
    }

    #[test]
    fn down_fractal_detected_at_middle_low() {
        let bars = vec![
            b(105.0, 102.0),
            b(104.0, 101.0),
            b(103.0, 95.0),     // middle — lowest
            b(104.0, 101.0),
            b(105.0, 102.0),
        ];
        let f = detect(&bars);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, FractalKind::Down);
        assert_eq!(f[0].price, 95.0);
    }

    #[test]
    fn no_fractal_when_middle_ties_neighbor() {
        // Strict inequality — ties disqualify (Bill Williams's convention).
        let bars = vec![
            b(100.0, 99.0),
            b(101.0, 99.0),
            b(105.0, 99.0),
            b(105.0, 99.0),    // tied with middle — disqualifies
            b(104.0, 99.0),
        ];
        let f = detect(&bars);
        let ups: Vec<_> = f.iter().filter(|x| x.kind == FractalKind::Up).collect();
        assert!(ups.is_empty());
    }

    #[test]
    fn first_and_last_two_bars_never_fractal() {
        let bars = vec![
            b(200.0, 0.0),     // highest high in series but bar 0 — can't be fractal
            b(101.0, 99.0),
            b(102.0, 98.0),
            b(101.0, 99.0),
            b(200.0, 0.0),     // bar n-1 — also can't be fractal
        ];
        let f = detect(&bars);
        for fr in &f {
            assert_eq!(fr.bar_index, 2,
                "fractal index {} outside valid middle range", fr.bar_index);
        }
    }

    #[test]
    fn series_with_clear_local_peaks_emits_up_fractals() {
        // Explicit peaks: a sawtooth on a rising staircase HIDES local
        // peaks because each subsequent "peak" is higher than the prior
        // bar's high (the rising trend dominates). Use distinct peaks
        // separated by deep troughs so each peak is strictly highest in
        // its own 5-bar window.
        let bars = vec![
            b(100.0, 99.0), b(102.0, 100.0),       // approaching peak 1
            b(105.0, 102.0),                        // peak 1 (idx 2)
            b(102.0, 100.0), b(99.0, 98.0),
            b(98.0,  97.0),  b(99.0, 98.0),         // trough + recovery
            b(102.0, 100.0),
            b(108.0, 105.0),                        // peak 2 (idx 8)
            b(102.0, 100.0), b(99.0, 98.0),
        ];
        let f = detect(&bars);
        let ups: Vec<_> = f.iter().filter(|x| x.kind == FractalKind::Up).collect();
        assert!(!ups.is_empty(), "expected at least one up-fractal, got {f:?}");
    }

    #[test]
    fn nan_high_or_low_skipped_safely() {
        let bars = vec![
            b(100.0, 99.0),
            b(101.0, 100.0),
            b(f64::NAN, f64::NAN),    // middle is NaN — no fractal
            b(102.0, 100.0),
            b(99.0, 98.0),
        ];
        let f = detect(&bars);
        assert!(f.is_empty());
    }
}
