//! Footprint-style imbalance detector.
//!
//! Per-bar count of price levels that hit a "stacked imbalance" (a
//! diagonal bid×ask ratio that exceeds `min_ratio` for `min_stack`
//! consecutive levels). The classic high-frequency tape pattern for
//! exhaustion / absorption — institutional sweeps tend to stack 3+
//! adjacent prints in one direction at a much higher ratio than the
//! opposite diagonal.
//!
//! Caller supplies per-bar arrays of (bid_vol, ask_vol) at each price
//! level. We compute the diagonal ratio for each level and tag bars
//! that contain a qualifying stack on either side.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

/// One footprint bar = stack of price-levels with bid+ask volume each.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintBar {
    /// Highest-price level first; lowest last. Caller's choice — the
    /// detector only needs them adjacent.
    pub levels: Vec<PriceLevel>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImbalanceConfig {
    /// Diagonal ratio threshold (e.g. 3.0 = ask volume at level N must
    /// be ≥ 3× bid volume at level N+1 to flag a bullish imbalance).
    pub min_ratio: f64,
    /// Minimum number of consecutive levels that must qualify to count
    /// as a "stacked" imbalance.
    pub min_stack: usize,
}

impl Default for ImbalanceConfig {
    fn default() -> Self {
        Self {
            min_ratio: 3.0,
            min_stack: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImbalanceEvent {
    pub bar_index: usize,
    /// Highest price level where the stack began.
    pub top_price: f64,
    /// Lowest price level where the stack ended (inclusive).
    pub bottom_price: f64,
    pub stack_size: usize,
    /// `true` = bullish (ask-dominated stack); `false` = bearish.
    pub bullish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImbalanceReport {
    pub events: Vec<ImbalanceEvent>,
}

pub fn detect(bars: &[FootprintBar], cfg: &ImbalanceConfig) -> ImbalanceReport {
    let mut report = ImbalanceReport::default();
    if !cfg.min_ratio.is_finite() || cfg.min_ratio <= 0.0 || cfg.min_stack < 2 {
        return report;
    }
    for (bi, bar) in bars.iter().enumerate() {
        let n = bar.levels.len();
        if n < cfg.min_stack {
            continue;
        }
        // Bullish stack: ask_vol[i] / bid_vol[i+1] >= min_ratio for
        // `min_stack` consecutive levels going DOWN.
        scan_diagonal(bar, cfg, true, bi, &mut report.events);
        // Bearish: bid_vol[i+1] / ask_vol[i] >= min_ratio.
        scan_diagonal(bar, cfg, false, bi, &mut report.events);
    }
    report
}

fn scan_diagonal(
    bar: &FootprintBar,
    cfg: &ImbalanceConfig,
    bullish: bool,
    bar_index: usize,
    out: &mut Vec<ImbalanceEvent>,
) {
    let levels = &bar.levels;
    let n = levels.len();
    let mut i = 0;
    while i + 1 < n {
        let mut run_end = i;
        while run_end + 1 < n {
            let cur = levels[run_end];
            let nxt = levels[run_end + 1];
            let qualifies = if bullish {
                let denom = nxt.bid_volume;
                denom > 0.0 && cur.ask_volume / denom >= cfg.min_ratio
            } else {
                let denom = cur.ask_volume;
                denom > 0.0 && nxt.bid_volume / denom >= cfg.min_ratio
            };
            if !qualifies {
                break;
            }
            run_end += 1;
        }
        let stack_size = run_end - i + 1;
        if stack_size >= cfg.min_stack {
            out.push(ImbalanceEvent {
                bar_index,
                // Levels are top-down per the doc: `levels[i]` is higher
                // than `levels[i+1]`. Pick the matching extremes.
                top_price: levels[i].price,
                bottom_price: levels[run_end].price,
                stack_size,
                bullish,
            });
            i = run_end + 1;
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lvl(p: f64, b: f64, a: f64) -> PriceLevel {
        PriceLevel {
            price: p,
            bid_volume: b,
            ask_volume: a,
        }
    }

    #[test]
    fn empty_returns_empty() {
        let r = detect(&[], &ImbalanceConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bar = FootprintBar {
            levels: vec![lvl(100.0, 10.0, 30.0); 5],
        };
        for cfg in [
            ImbalanceConfig {
                min_ratio: 0.0,
                min_stack: 3,
            },
            ImbalanceConfig {
                min_ratio: -1.0,
                min_stack: 3,
            },
            ImbalanceConfig {
                min_ratio: f64::NAN,
                min_stack: 3,
            },
            ImbalanceConfig {
                min_ratio: 3.0,
                min_stack: 1,
            },
        ] {
            assert!(detect(std::slice::from_ref(&bar), &cfg).events.is_empty());
        }
    }

    #[test]
    fn bullish_stack_detected_when_ask_dominates_3_consecutive_levels() {
        // 3 levels where ask[i] / bid[i+1] >= 3.0.
        let bar = FootprintBar {
            levels: vec![
                lvl(101.0, 5.0, 30.0), // ask 30 / next bid 5 = 6 ✓
                lvl(100.5, 5.0, 30.0), // ask 30 / next bid 5 = 6 ✓
                lvl(100.0, 5.0, 30.0), // ask 30 / next bid 5 = 6 ✓ (needs level 3)
                lvl(99.5, 5.0, 5.0),
            ],
        };
        let r = detect(&[bar], &ImbalanceConfig::default());
        let bull: Vec<_> = r.events.iter().filter(|e| e.bullish).collect();
        assert_eq!(bull.len(), 1);
        // stack_size counts LEVELS in the qualifying run. 3 transitions
        // chain 4 consecutive levels (101 / 100.5 / 100 / 99.5).
        assert_eq!(bull[0].stack_size, 4);
    }

    #[test]
    fn bearish_stack_detected_when_bid_dominates() {
        let bar = FootprintBar {
            levels: vec![
                lvl(101.0, 30.0, 5.0),
                lvl(100.5, 30.0, 5.0), // bid 30 / prev ask 5 = 6 ✓
                lvl(100.0, 30.0, 5.0), // bid 30 / prev ask 5 = 6 ✓
                lvl(99.5, 30.0, 5.0),  // bid 30 / prev ask 5 = 6 ✓ — 3 transitions = stack of 3
            ],
        };
        let r = detect(&[bar], &ImbalanceConfig::default());
        let bear: Vec<_> = r.events.iter().filter(|e| !e.bullish).collect();
        assert!(!bear.is_empty(), "expected bearish stack");
    }

    #[test]
    fn no_stack_when_volumes_balanced() {
        let bar = FootprintBar {
            levels: (0..5).map(|i| lvl(100.0 + i as f64, 10.0, 10.0)).collect(),
        };
        let r = detect(&[bar], &ImbalanceConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn short_bar_skipped() {
        let bar = FootprintBar {
            levels: vec![lvl(100.0, 5.0, 30.0)],
        };
        let r = detect(&[bar], &ImbalanceConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn zero_denominator_safely_skipped() {
        // ask=30, next bid=0 → can't qualify (division skipped).
        let bar = FootprintBar {
            levels: vec![
                lvl(101.0, 0.0, 30.0),
                lvl(100.5, 0.0, 30.0),
                lvl(100.0, 0.0, 30.0),
            ],
        };
        let r = detect(&[bar], &ImbalanceConfig::default());
        assert!(
            r.events.is_empty(),
            "zero bid denom shouldn't crash or fire"
        );
    }
}
