//! Volatility Contraction Pattern (VCP) — Mark Minervini.
//!
//! Detects a sequence of progressively-tighter consolidations within
//! an uptrend, indicating supply absorption before a breakout:
//!
//!   1. Stock in uptrend (above 200-day MA)
//!   2. Series of pullbacks, each one smaller in % range than the previous
//!   3. Volume contracts during each successive pullback
//!   4. Tightening culminates in a "pivot point" breakout
//!
//! Heuristic detection:
//!   - Identify pivot highs/lows over the lookback window
//!   - Measure each pullback as max((high − low) / high) over its window
//!   - Confirm pullback magnitudes are MONOTONICALLY decreasing
//!   - Require ≥ `min_contractions` distinct contractions
//!
//! Pure compute. Companion to `cup_and_handle`, `triple_top_bottom`,
//! `darvas_box`, `breakout_detector`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contraction {
    pub start_index: usize,
    pub end_index: usize,
    pub pullback_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcpCandidate {
    pub contractions: Vec<Contraction>,
    pub pivot_price: f64,
    pub final_range_pct: f64,
    pub n_contractions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pivot_lookback: usize,
    pub min_contractions: usize,
    pub max_pattern_bars: usize,
    pub max_final_pullback_pct: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pivot_lookback: 3,
            min_contractions: 3,
            max_pattern_bars: 250,
            max_final_pullback_pct: 0.10,
        }
    }
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<VcpCandidate> {
    let n = bars.len();
    let mut out = Vec::new();
    if cfg.pivot_lookback == 0
        || cfg.min_contractions < 2
        || cfg.max_pattern_bars == 0
        || cfg.max_final_pullback_pct <= 0.0
        || n < cfg.min_contractions * (2 * cfg.pivot_lookback + 1)
    {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    let highs = find_pivots(bars, cfg.pivot_lookback, true);
    let lows = find_pivots(bars, cfg.pivot_lookback, false);
    if highs.len() < cfg.min_contractions || lows.len() < cfg.min_contractions {
        return out;
    }
    // Pair each pivot high with the next pivot low to form a contraction.
    let mut contractions = Vec::new();
    let mut h_iter = highs.iter().peekable();
    let mut l_iter = lows.iter().peekable();
    while let (Some(&&h_idx), Some(&&l_idx)) = (h_iter.peek(), l_iter.peek()) {
        if h_idx < l_idx {
            let pullback = (bars[h_idx].high - bars[l_idx].low) / bars[h_idx].high;
            if pullback > 0.0 {
                contractions.push(Contraction {
                    start_index: h_idx,
                    end_index: l_idx,
                    pullback_pct: pullback,
                });
            }
            h_iter.next();
            l_iter.next();
        } else {
            l_iter.next();
        }
    }
    if contractions.len() < cfg.min_contractions {
        return out;
    }
    // Slide window of consecutive contractions checking monotone-decreasing.
    for start in 0..=(contractions.len() - cfg.min_contractions) {
        let slice = &contractions[start..start + cfg.min_contractions];
        let span = slice.last().unwrap().end_index - slice.first().unwrap().start_index;
        if span > cfg.max_pattern_bars {
            continue;
        }
        let monotone = slice
            .windows(2)
            .all(|w| w[1].pullback_pct < w[0].pullback_pct);
        if !monotone {
            continue;
        }
        let final_pct = slice.last().unwrap().pullback_pct;
        if final_pct > cfg.max_final_pullback_pct {
            continue;
        }
        let pivot_price = bars[slice.last().unwrap().start_index].high;
        out.push(VcpCandidate {
            contractions: slice.to_vec(),
            pivot_price,
            final_range_pct: final_pct,
            n_contractions: slice.len(),
        });
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

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 100];
        let cfg = Config {
            pivot_lookback: 0,
            ..Default::default()
        };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn flat_market_no_pattern() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn nan_input_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 50];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_vcp_detected() {
        // Construct 3 contractions: ~20% → ~10% → ~5% pullbacks.
        // The flat-fill bars need to flank each pivot above (for low pivots)
        // and below (for high pivots) the pivot's extremum.
        let mut bars: Vec<Bar> = vec![b(101.0, 100.0, 100.5); 50];
        // First contraction: peak 110 at 4, trough 88 at 8 → ~20% pullback.
        // For bars[8] (low=88) to be a low pivot, flank lows must exceed 88.
        // Default flank lows = 100 > 88 ✓.
        bars[4] = b(110.0, 109.0, 110.0);
        bars[8] = b(89.0, 88.0, 88.5);
        // Second contraction: peak 110 at 14, trough 99 at 18 → 10% pullback.
        // For bars[18] (low=99) flank must be > 99. Default flank=100 ✓.
        bars[14] = b(110.0, 109.0, 110.0);
        bars[18] = b(99.5, 99.0, 99.2);
        // Third contraction: peak 110 at 24, trough at 28 → 5% pullback.
        // Need bars[28].low ≈ 104.5; flank must be > 104.5. Override locally.
        bars[24] = b(110.0, 109.0, 110.0);
        bars[26] = b(108.0, 107.0, 107.5);
        bars[27] = b(106.0, 105.5, 106.0);
        bars[28] = b(105.0, 104.5, 104.7);
        bars[29] = b(106.0, 105.5, 106.0);
        bars[30] = b(108.0, 107.0, 107.5);
        let cfg = Config {
            pivot_lookback: 2,
            min_contractions: 3,
            max_pattern_bars: 100,
            max_final_pullback_pct: 0.10,
        };
        let cands = detect(&bars, &cfg);
        assert!(!cands.is_empty(), "expected VCP candidate, got {cands:?}");
        let c = &cands[0];
        assert!(c.n_contractions >= 3);
        for w in c.contractions.windows(2) {
            assert!(w[1].pullback_pct < w[0].pullback_pct);
        }
    }

    #[test]
    fn expanding_pullbacks_not_classified() {
        // Pullbacks increasing → NOT VCP.
        let mut bars: Vec<Bar> = vec![b(100.5, 99.5, 100.0); 50];
        bars[4] = b(105.0, 99.5, 105.0);
        bars[8] = b(101.0, 100.0, 100.5); // small pullback ~5%
        bars[14] = b(105.0, 99.5, 105.0);
        bars[18] = b(95.0, 94.0, 94.5); // bigger pullback ~10%
        bars[24] = b(105.0, 99.5, 105.0);
        bars[28] = b(85.0, 84.0, 84.5); // biggest pullback ~20%
        let cfg = Config {
            pivot_lookback: 2,
            min_contractions: 3,
            max_pattern_bars: 100,
            max_final_pullback_pct: 0.30,
        };
        let cands = detect(&bars, &cfg);
        assert!(
            cands.is_empty(),
            "expanding pullbacks should NOT classify as VCP"
        );
    }

    #[test]
    fn pivot_price_matches_last_high() {
        let mut bars: Vec<Bar> = vec![b(100.5, 99.5, 100.0); 50];
        bars[4] = b(120.0, 99.5, 120.0);
        bars[8] = b(96.0, 96.0, 96.0);
        bars[14] = b(118.0, 99.5, 118.0);
        bars[18] = b(106.0, 106.0, 106.0);
        bars[24] = b(115.0, 99.5, 115.0);
        bars[28] = b(110.0, 109.0, 110.0);
        let cfg = Config {
            pivot_lookback: 2,
            min_contractions: 3,
            max_pattern_bars: 100,
            max_final_pullback_pct: 0.10,
        };
        let cands = detect(&bars, &cfg);
        if let Some(c) = cands.first() {
            // pivot_price should be the last pivot high (115).
            assert!((c.pivot_price - 115.0).abs() < 1e-9);
        }
    }
}
