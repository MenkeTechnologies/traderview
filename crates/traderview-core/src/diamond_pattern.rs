//! Diamond Top / Bottom Reversal Pattern Detector.
//!
//! A two-phase consolidation:
//!
//!   1. **Broadening leg** — pivot highs trend UPWARD while pivot lows
//!      trend DOWNWARD (volatility expansion).
//!   2. **Contracting leg** — pivot highs trend DOWNWARD while pivot
//!      lows trend UPWARD (volatility compression — symmetric triangle).
//!
//! When the two legs meet at an apex and price breaks out, the diamond
//! is complete.
//!
//! Diamond-top: forms at market tops after a sustained uptrend, breaks
//! DOWNWARD on confirmation.
//! Diamond-bottom: forms at market bottoms, breaks UPWARD.
//!
//! Detection heuristic:
//!   - Find 5 pivot highs (h1..h5) and 5 pivot lows (l1..l5).
//!   - Broadening leg: h1 < h2 < h3 AND l1 > l2 > l3
//!   - Contracting leg: h3 > h4 > h5 AND l3 < l4 < l5
//!
//! Pure compute. Companion to `triple_top_bottom`, `head_shoulders`,
//! `cup_and_handle`, `bump_and_run`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind { DiamondTop, DiamondBottom }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondCandidate {
    pub kind: PatternKind,
    pub pivot_high_indices: [usize; 5],
    pub pivot_low_indices: [usize; 5],
    pub broadening_apex_index: usize,
    pub contracting_apex_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pivot_lookback: usize,
    pub max_pattern_bars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { pivot_lookback: 3, max_pattern_bars: 250 }
    }
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<DiamondCandidate> {
    let n = bars.len();
    let mut out = Vec::new();
    if cfg.pivot_lookback == 0 || cfg.max_pattern_bars == 0
        || n < 5 + 2 * cfg.pivot_lookback {
        return out;
    }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()) { return out; }
    let highs = find_pivots(bars, cfg.pivot_lookback, true);
    let lows = find_pivots(bars, cfg.pivot_lookback, false);
    if highs.len() < 5 || lows.len() < 5 { return out; }
    // Scan all contiguous 5-pivot windows for both highs and lows.
    for h_window in highs.windows(5) {
        for l_window in lows.windows(5) {
            // Pivots must interleave roughly: pattern span = max - min indices.
            let all_idx = [h_window, l_window].concat();
            let max_idx = *all_idx.iter().max().unwrap();
            let min_idx = *all_idx.iter().min().unwrap();
            if max_idx - min_idx > cfg.max_pattern_bars { continue; }
            // Diamond-top: expanding highs+contracting lows in leg 1,
            // then contracting highs+expanding lows in leg 2.
            if let Some(c) = check_diamond_top(bars, h_window, l_window) {
                out.push(c);
            }
            if let Some(c) = check_diamond_bottom(bars, h_window, l_window) {
                out.push(c);
            }
        }
    }
    out
}

fn check_diamond_top(bars: &[Bar], hi: &[usize], lo: &[usize]) -> Option<DiamondCandidate> {
    // Leg 1: h1 < h2 < h3, l1 > l2 > l3 (broadening)
    if !(bars[hi[0]].high < bars[hi[1]].high && bars[hi[1]].high < bars[hi[2]].high) { return None; }
    if !(bars[lo[0]].low > bars[lo[1]].low && bars[lo[1]].low > bars[lo[2]].low) { return None; }
    // Leg 2: h3 > h4 > h5, l3 < l4 < l5 (contracting)
    if !(bars[hi[2]].high > bars[hi[3]].high && bars[hi[3]].high > bars[hi[4]].high) { return None; }
    if !(bars[lo[2]].low < bars[lo[3]].low && bars[lo[3]].low < bars[lo[4]].low) { return None; }
    Some(DiamondCandidate {
        kind: PatternKind::DiamondTop,
        pivot_high_indices: [hi[0], hi[1], hi[2], hi[3], hi[4]],
        pivot_low_indices: [lo[0], lo[1], lo[2], lo[3], lo[4]],
        broadening_apex_index: hi[2].max(lo[2]),
        contracting_apex_index: hi[4].max(lo[4]),
    })
}

fn check_diamond_bottom(bars: &[Bar], hi: &[usize], lo: &[usize]) -> Option<DiamondCandidate> {
    // Mirror: leg 1 contracts, leg 2 broadens (typical bottom geometry).
    if !(bars[hi[0]].high > bars[hi[1]].high && bars[hi[1]].high > bars[hi[2]].high) { return None; }
    if !(bars[lo[0]].low < bars[lo[1]].low && bars[lo[1]].low < bars[lo[2]].low) { return None; }
    if !(bars[hi[2]].high < bars[hi[3]].high && bars[hi[3]].high < bars[hi[4]].high) { return None; }
    if !(bars[lo[2]].low > bars[lo[3]].low && bars[lo[3]].low > bars[lo[4]].low) { return None; }
    Some(DiamondCandidate {
        kind: PatternKind::DiamondBottom,
        pivot_high_indices: [hi[0], hi[1], hi[2], hi[3], hi[4]],
        pivot_low_indices: [lo[0], lo[1], lo[2], lo[3], lo[4]],
        broadening_apex_index: hi[2].max(lo[2]),
        contracting_apex_index: hi[4].max(lo[4]),
    })
}

fn find_pivots(bars: &[Bar], lookback: usize, find_high: bool) -> Vec<usize> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 2 * lookback + 1 { return out; }
    for i in lookback..(n - lookback) {
        let v = if find_high { bars[i].high } else { bars[i].low };
        let mut is_pivot = true;
        for k in 1..=lookback {
            let l = if find_high { bars[i - k].high } else { bars[i - k].low };
            let r = if find_high { bars[i + k].high } else { bars[i + k].low };
            if find_high {
                if l >= v || r >= v { is_pivot = false; break; }
            } else if l <= v || r <= v { is_pivot = false; break; }
        }
        if is_pivot { out.push(i); }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar { Bar { high: h, low: l } }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(101.0, 99.0); 50];
        let cfg = Config { pivot_lookback: 0, ..Default::default() };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn flat_market_no_pattern() {
        let bars = vec![b(101.0, 99.0); 60];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_diamond_top_detected() {
        // 41 bars; 5 highs at indices 4, 12, 20, 28, 36 with broadening then
        // contracting heights; 5 lows at 8, 16, 24, 32 (4 only, need 5).
        // Use 45 bars with 5 highs at 4, 12, 20, 28, 36 and 5 lows at 0, 8,
        // 16, 24, 32, 40.
        let mut bars = vec![b(100.5, 99.5); 45];
        // Broadening highs: 102, 103, 104 at idx 4, 12, 20.
        bars[4] = b(102.0, 99.5);
        bars[12] = b(103.0, 99.5);
        bars[20] = b(104.0, 99.5);
        // Contracting highs: 103, 102 at idx 28, 36.
        bars[28] = b(103.0, 99.5);
        bars[36] = b(102.0, 99.5);
        // Broadening lows: 99, 98, 97 at idx 8, 16, 24.
        bars[8] = b(100.5, 99.0);
        bars[16] = b(100.5, 98.0);
        bars[24] = b(100.5, 97.0);
        // Contracting lows: 98, 99 at idx 32, 40.
        bars[32] = b(100.5, 98.0);
        bars[40] = b(100.5, 99.0);
        let cands = detect(&bars, &Config { pivot_lookback: 2, max_pattern_bars: 100 });
        assert!(cands.iter().any(|c| c.kind == PatternKind::DiamondTop),
            "expected diamond top, got {cands:?}");
    }

    #[test]
    fn no_broadening_leg_no_pattern() {
        // Symmetric triangle (contracting only) — not a diamond.
        let mut bars = vec![b(100.5, 99.5); 45];
        bars[4] = b(105.0, 99.5);
        bars[12] = b(104.0, 99.5);
        bars[20] = b(103.0, 99.5);
        bars[28] = b(102.0, 99.5);
        bars[36] = b(101.0, 99.5);
        bars[8] = b(100.5, 95.0);
        bars[16] = b(100.5, 96.0);
        bars[24] = b(100.5, 97.0);
        bars[32] = b(100.5, 98.0);
        bars[40] = b(100.5, 99.0);
        let cands: Vec<_> = detect(&bars, &Config { pivot_lookback: 2, max_pattern_bars: 100 })
            .into_iter()
            .filter(|c| c.kind == PatternKind::DiamondTop).collect();
        assert!(cands.is_empty(), "no broadening leg → no diamond, got {cands:?}");
    }

    #[test]
    fn nan_input_returns_empty() {
        let mut bars = vec![b(101.0, 99.0); 50];
        bars[5] = b(f64::NAN, 99.0);
        assert!(detect(&bars, &Config::default()).is_empty());
    }
}
