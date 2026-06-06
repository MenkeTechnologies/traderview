//! Triple-Top / Triple-Bottom pattern detector.
//!
//! Pattern:
//!   - Three pivot highs (lows) at approximately equal price levels,
//!     with two intervening pivot lows (highs) defining the "neckline".
//!   - Confirms when price breaks below (above) the neckline.
//!
//! Distinct from head-and-shoulders: triple top has THREE EQUAL peaks
//! (no head higher than shoulders).
//!
//! Detection rules:
//!   - 3 pivot highs within `peak_tolerance_pct` of each other
//!   - 2 pivot lows between them
//!   - Total pattern span ≤ `max_pattern_bars`
//!
//! Pure compute. Williams-fractal-style pivot detection over
//! `pivot_lookback` bars each side.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pivot_lookback: usize,
    pub peak_tolerance_pct: f64,
    pub max_pattern_bars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pivot_lookback: 5,
            peak_tolerance_pct: 0.03,
            max_pattern_bars: 250,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    TripleTop,
    TripleBottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripleCandidate {
    pub kind: PatternKind,
    pub peak_indices: [usize; 3],
    pub trough_indices: [usize; 2],
    pub neckline_price: f64,
    pub average_peak_price: f64,
    pub projection_target: f64,
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<TripleCandidate> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 5 + 2 * cfg.pivot_lookback
        || cfg.pivot_lookback == 0
        || cfg.peak_tolerance_pct < 0.0
        || cfg.max_pattern_bars == 0
    {
        return out;
    }
    let highs = find_pivots(bars, cfg.pivot_lookback, true);
    let lows = find_pivots(bars, cfg.pivot_lookback, false);
    // Triple top: 3 high pivots near-equal + 2 low pivots between.
    for trio in window_3(&highs) {
        let (h1, h2, h3) = trio;
        if h3 - h1 > cfg.max_pattern_bars {
            continue;
        }
        let p1 = bars[h1].high;
        let p2 = bars[h2].high;
        let p3 = bars[h3].high;
        if !p1.is_finite() || !p2.is_finite() || !p3.is_finite() {
            continue;
        }
        let avg = (p1 + p2 + p3) / 3.0;
        let max_dev = ((p1 - avg).abs().max((p2 - avg).abs())).max((p3 - avg).abs());
        if max_dev / avg > cfg.peak_tolerance_pct {
            continue;
        }
        let t1 = lows.iter().copied().find(|i| *i > h1 && *i < h2);
        let t2 = lows.iter().copied().find(|i| *i > h2 && *i < h3);
        let (Some(t1), Some(t2)) = (t1, t2) else {
            continue;
        };
        let neckline = (bars[t1].low + bars[t2].low) / 2.0;
        let projection = neckline - (avg - neckline); // measured-move downside
        out.push(TripleCandidate {
            kind: PatternKind::TripleTop,
            peak_indices: [h1, h2, h3],
            trough_indices: [t1, t2],
            neckline_price: neckline,
            average_peak_price: avg,
            projection_target: projection,
        });
    }
    for trio in window_3(&lows) {
        let (l1, l2, l3) = trio;
        if l3 - l1 > cfg.max_pattern_bars {
            continue;
        }
        let p1 = bars[l1].low;
        let p2 = bars[l2].low;
        let p3 = bars[l3].low;
        if !p1.is_finite() || !p2.is_finite() || !p3.is_finite() {
            continue;
        }
        let avg = (p1 + p2 + p3) / 3.0;
        let max_dev = ((p1 - avg).abs().max((p2 - avg).abs())).max((p3 - avg).abs());
        if max_dev / avg.abs() > cfg.peak_tolerance_pct {
            continue;
        }
        let t1 = highs.iter().copied().find(|i| *i > l1 && *i < l2);
        let t2 = highs.iter().copied().find(|i| *i > l2 && *i < l3);
        let (Some(t1), Some(t2)) = (t1, t2) else {
            continue;
        };
        let neckline = (bars[t1].high + bars[t2].high) / 2.0;
        let projection = neckline + (neckline - avg); // measured-move upside
        out.push(TripleCandidate {
            kind: PatternKind::TripleBottom,
            peak_indices: [l1, l2, l3],
            trough_indices: [t1, t2],
            neckline_price: neckline,
            average_peak_price: avg,
            projection_target: projection,
        });
    }
    out
}

fn window_3(idxs: &[usize]) -> impl Iterator<Item = (usize, usize, usize)> + '_ {
    let mut iter = Vec::new();
    for i in 0..idxs.len().saturating_sub(2) {
        iter.push((idxs[i], idxs[i + 1], idxs[i + 2]));
    }
    iter.into_iter()
}

fn find_pivots(bars: &[Bar], lookback: usize, find_high: bool) -> Vec<usize> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 2 * lookback + 1 {
        return out;
    }
    for i in lookback..(n - lookback) {
        let v = if find_high { bars[i].high } else { bars[i].low };
        if !v.is_finite() {
            continue;
        }
        let mut is_pivot = true;
        for k in 1..=lookback {
            let l = if find_high {
                bars[i - k].high
            } else {
                bars[i - k].low
            };
            let r = if find_high {
                bars[i + k].high
            } else {
                bars[i + k].low
            };
            if !l.is_finite() || !r.is_finite() {
                is_pivot = false;
                break;
            }
            if find_high {
                if l >= v || r >= v {
                    is_pivot = false;
                    break;
                }
            } else if l <= v || r <= v {
                is_pivot = false;
                break;
            }
        }
        if is_pivot {
            out.push(i);
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
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(101.0, 99.0); 30];
        let cfg = Config {
            pivot_lookback: 0,
            ..Default::default()
        };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn flat_series_no_pattern() {
        let bars = vec![b(101.0, 99.0); 50];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_triple_top_detected() {
        // 30 bars; peaks at indices 5, 15, 25; troughs at 10, 20.
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[5] = b(110.0, 109.5);
        bars[15] = b(110.5, 110.0);
        bars[25] = b(109.5, 109.0);
        bars[10] = b(100.0, 95.0);
        bars[20] = b(100.0, 96.0);
        let cfg = Config {
            pivot_lookback: 2,
            peak_tolerance_pct: 0.05,
            ..Default::default()
        };
        let cands = detect(&bars, &cfg);
        assert!(
            cands.iter().any(|c| c.kind == PatternKind::TripleTop),
            "expected triple top, got {cands:?}"
        );
    }

    #[test]
    fn classic_triple_bottom_detected() {
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[5] = b(91.0, 90.0);
        bars[15] = b(90.5, 90.0);
        bars[25] = b(91.5, 90.5);
        bars[10] = b(105.0, 100.0);
        bars[20] = b(104.0, 100.0);
        let cfg = Config {
            pivot_lookback: 2,
            peak_tolerance_pct: 0.05,
            ..Default::default()
        };
        let cands = detect(&bars, &cfg);
        assert!(
            cands.iter().any(|c| c.kind == PatternKind::TripleBottom),
            "expected triple bottom, got {cands:?}"
        );
    }

    #[test]
    fn peaks_too_unequal_rejected() {
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[5] = b(110.0, 109.5);
        bars[15] = b(120.0, 119.5); // 10% higher than first peak
        bars[25] = b(110.0, 109.5);
        bars[10] = b(100.0, 95.0);
        bars[20] = b(100.0, 95.0);
        let cfg = Config {
            pivot_lookback: 2,
            peak_tolerance_pct: 0.03,
            ..Default::default()
        };
        let cands: Vec<_> = detect(&bars, &cfg)
            .into_iter()
            .filter(|c| c.kind == PatternKind::TripleTop)
            .collect();
        assert!(cands.is_empty(), "unequal peaks should reject triple top");
    }

    #[test]
    fn projection_target_below_neckline_for_top() {
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[5] = b(110.0, 109.5);
        bars[15] = b(110.5, 110.0);
        bars[25] = b(109.5, 109.0);
        bars[10] = b(100.0, 95.0);
        bars[20] = b(100.0, 96.0);
        let cfg = Config {
            pivot_lookback: 2,
            peak_tolerance_pct: 0.05,
            ..Default::default()
        };
        let cand = detect(&bars, &cfg)
            .into_iter()
            .find(|c| c.kind == PatternKind::TripleTop)
            .expect("triple top");
        assert!(cand.projection_target < cand.neckline_price);
    }
}
