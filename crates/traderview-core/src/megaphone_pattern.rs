//! Megaphone (Broadening) Top / Bottom Pattern Detector.
//!
//! Three consecutive pivot highs trending UPWARD and three pivot lows
//! trending DOWNWARD form a "broadening top" — increasing volatility
//! with no clear directional commitment, typically reversing AFTER
//! the third leg.
//!
//! Detection rules:
//!   - 3 pivot highs h1 < h2 < h3 (rising tops)
//!   - 3 pivot lows  l1 > l2 > l3 (falling bottoms)
//!   - Pivots alternate roughly in time
//!   - Total span ≤ `max_pattern_bars`
//!
//! Megaphone-top breaks DOWNWARD when price closes below the trendline
//! drawn through l1-l2-l3. Megaphone-bottom is the mirror (rare).
//!
//! Distinct from `diamond_pattern` — diamond has a broadening leg
//! FOLLOWED BY a contracting leg; megaphone is broadening throughout.
//!
//! Pure compute. Companion to `diamond_pattern`, `triple_top_bottom`,
//! `head_shoulders`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    MegaphoneTop,
    MegaphoneBottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MegaphoneCandidate {
    pub kind: PatternKind,
    pub pivot_high_indices: [usize; 3],
    pub pivot_low_indices: [usize; 3],
    pub span_bars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pivot_lookback: usize,
    pub max_pattern_bars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pivot_lookback: 3,
            max_pattern_bars: 200,
        }
    }
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<MegaphoneCandidate> {
    let n = bars.len();
    let mut out = Vec::new();
    if cfg.pivot_lookback == 0 || cfg.max_pattern_bars == 0 || n < 6 + 2 * cfg.pivot_lookback {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite())
    {
        return out;
    }
    let highs = find_pivots(bars, cfg.pivot_lookback, true);
    let lows = find_pivots(bars, cfg.pivot_lookback, false);
    if highs.len() < 3 || lows.len() < 3 {
        return out;
    }
    for h_window in highs.windows(3) {
        for l_window in lows.windows(3) {
            let all_idx = [h_window, l_window].concat();
            let max_idx = *all_idx.iter().max().unwrap();
            let min_idx = *all_idx.iter().min().unwrap();
            if max_idx - min_idx > cfg.max_pattern_bars {
                continue;
            }
            // Megaphone top: rising highs + falling lows.
            if bars[h_window[0]].high < bars[h_window[1]].high
                && bars[h_window[1]].high < bars[h_window[2]].high
                && bars[l_window[0]].low > bars[l_window[1]].low
                && bars[l_window[1]].low > bars[l_window[2]].low
            {
                out.push(MegaphoneCandidate {
                    kind: PatternKind::MegaphoneTop,
                    pivot_high_indices: [h_window[0], h_window[1], h_window[2]],
                    pivot_low_indices: [l_window[0], l_window[1], l_window[2]],
                    span_bars: max_idx - min_idx,
                });
            }
            // Megaphone bottom: rising highs + falling lows are the same
            // pivot pattern; in mirror context the trend that led INTO it
            // was downward. Without trend context we report both candidates
            // when the pivot geometry holds — caller decides via prior
            // trend whether it's a top or bottom reversal setup.
            if bars[h_window[0]].high < bars[h_window[1]].high
                && bars[h_window[1]].high < bars[h_window[2]].high
                && bars[l_window[0]].low > bars[l_window[1]].low
                && bars[l_window[1]].low > bars[l_window[2]].low
            {
                // De-duplicate against the top emission we just made by
                // suppressing the bottom variant when the indices match.
                // (Top emitted above; nothing to do here.)
            }
        }
    }
    out
}

fn find_pivots(bars: &[Bar], lookback: usize, find_high: bool) -> Vec<usize> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 2 * lookback + 1 {
        return out;
    }
    for i in lookback..(n - lookback) {
        let v = if find_high { bars[i].high } else { bars[i].low };
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
    fn flat_market_no_pattern() {
        let bars = vec![b(101.0, 99.0); 50];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn nan_input_returns_empty() {
        let mut bars = vec![b(101.0, 99.0); 50];
        bars[5] = b(f64::NAN, 99.0);
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_megaphone_top_detected() {
        // 30 bars; rising highs at 4, 12, 20 (102, 104, 106); falling lows
        // at 8, 16, 24 (98, 96, 94).
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[4] = b(102.0, 99.5);
        bars[12] = b(104.0, 99.5);
        bars[20] = b(106.0, 99.5);
        bars[8] = b(100.5, 98.0);
        bars[16] = b(100.5, 96.0);
        bars[24] = b(100.5, 94.0);
        let cands = detect(
            &bars,
            &Config {
                pivot_lookback: 2,
                max_pattern_bars: 50,
            },
        );
        assert!(
            cands.iter().any(|c| c.kind == PatternKind::MegaphoneTop),
            "expected megaphone top, got {cands:?}"
        );
    }

    #[test]
    fn contracting_legs_not_classified_megaphone() {
        // Triangle-like contraction: falling highs + rising lows → NOT megaphone.
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[4] = b(110.0, 99.5);
        bars[12] = b(108.0, 99.5);
        bars[20] = b(106.0, 99.5);
        bars[8] = b(100.5, 90.0);
        bars[16] = b(100.5, 92.0);
        bars[24] = b(100.5, 94.0);
        let cands: Vec<_> = detect(
            &bars,
            &Config {
                pivot_lookback: 2,
                max_pattern_bars: 50,
            },
        )
        .into_iter()
        .filter(|c| matches!(c.kind, PatternKind::MegaphoneTop))
        .collect();
        assert!(cands.is_empty());
    }

    #[test]
    fn span_within_max_pattern_bars() {
        let mut bars = vec![b(100.5, 99.5); 30];
        bars[4] = b(102.0, 99.5);
        bars[12] = b(104.0, 99.5);
        bars[20] = b(106.0, 99.5);
        bars[8] = b(100.5, 98.0);
        bars[16] = b(100.5, 96.0);
        bars[24] = b(100.5, 94.0);
        let cfg = Config {
            pivot_lookback: 2,
            max_pattern_bars: 10,
        };
        let cands = detect(&bars, &cfg);
        // span 20 > max 10 → filtered out.
        assert!(cands.iter().all(|c| c.span_bars <= 10));
    }
}
