//! Opening Range Breakout (ORB) detector.
//!
//! ORB is a fundamental day-trading setup: the first N minutes of the
//! session establish a high/low range; any subsequent break with volume
//! is the directional signal. Toby Crabel popularized 30-min ORB in
//! "Day Trading with Short-Term Price Patterns"; the canonical retail
//! variant is the 5-minute ORB at the NYSE open.
//!
//! Inputs:
//!   - intraday OHLC bars in time-order
//!   - `opening_bars`: the number of bars that constitute the OR window
//!     (e.g. 6 bars × 5-min = 30-min ORB)
//!
//! Outputs:
//!   - the opening range high/low
//!   - the first break above (long signal) and first break below (short
//!     signal) — both can fire in the same session
//!   - per-direction breakout magnitude in points and ATRs (if ATR given)
//!
//! Pure compute. Reuses `OhlcBar` shape from neighboring modules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbConfig {
    pub opening_bars: usize,
    /// Optional ATR for "breach in ATRs" annotation; pass 0 to omit.
    #[serde(default)]
    pub atr: f64,
    /// If true, count only CLOSE-based breakouts (wicks above/below the
    /// range don't trigger). Reduces false signals on liquidity sweeps.
    #[serde(default)]
    pub close_only: bool,
}

impl Default for OrbConfig {
    fn default() -> Self { Self { opening_bars: 6, atr: 0.0, close_only: false } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrbDirection { Up, Down }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrbBreak {
    pub bar_index: usize,
    pub direction: OrbDirection,
    pub breach_price: f64,
    pub breach_distance: f64,
    /// `breach_distance / atr`, only populated if config supplied a positive ATR.
    pub breach_atrs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrbReport {
    pub opening_high: f64,
    pub opening_low: f64,
    pub opening_range: f64,
    pub upper_break: Option<OrbBreak>,
    pub lower_break: Option<OrbBreak>,
}

pub fn detect(bars: &[OhlcBar], cfg: &OrbConfig) -> OrbReport {
    let n = bars.len();
    if n == 0 || cfg.opening_bars == 0 || n <= cfg.opening_bars {
        return OrbReport::default();
    }
    let opening = &bars[..cfg.opening_bars];
    let opening_high = opening.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
    let opening_low  = opening.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    let opening_range = opening_high - opening_low;

    let mut upper_break = None;
    let mut lower_break = None;
    let atr_some = if cfg.atr > 0.0 { Some(cfg.atr) } else { None };
    for (i, bar) in bars.iter().enumerate().take(n).skip(cfg.opening_bars) {
        let probe_up   = if cfg.close_only { bar.close } else { bar.high };
        let probe_down = if cfg.close_only { bar.close } else { bar.low };
        if upper_break.is_none() && probe_up > opening_high {
            let dist = probe_up - opening_high;
            upper_break = Some(OrbBreak {
                bar_index: i,
                direction: OrbDirection::Up,
                breach_price: probe_up,
                breach_distance: dist,
                breach_atrs: atr_some.map(|a| dist / a),
            });
        }
        if lower_break.is_none() && probe_down < opening_low {
            let dist = opening_low - probe_down;
            lower_break = Some(OrbBreak {
                bar_index: i,
                direction: OrbDirection::Down,
                breach_price: probe_down,
                breach_distance: -dist,
                breach_atrs: atr_some.map(|a| -dist / a),
            });
        }
        // Once both directions have fired, no more events possible.
        if upper_break.is_some() && lower_break.is_some() { break; }
    }
    OrbReport {
        opening_high, opening_low, opening_range,
        upper_break, lower_break,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { high: h, low: l, close: c } }

    #[test]
    fn empty_or_short_input_returns_default() {
        assert_eq!(detect(&[], &OrbConfig::default()).opening_range, 0.0);
        let short: Vec<OhlcBar> = (0..3).map(|_| b(100.0, 99.0, 99.5)).collect();
        let r = detect(&short, &OrbConfig::default());
        assert_eq!(r.opening_high, 0.0);    // Default OrbConfig opening_bars=6 > 3.
    }

    #[test]
    fn opening_range_computed_from_first_n_bars() {
        // 6 opening bars with range 99.5 → 101.5, then 3 more bars.
        let mut bars: Vec<OhlcBar> = (0..6).map(|_| b(101.5, 99.5, 100.5)).collect();
        bars.push(b(102.0, 100.0, 101.5));
        bars.push(b(101.0, 100.5, 100.7));
        bars.push(b(101.0, 100.5, 100.7));
        let r = detect(&bars, &OrbConfig::default());
        assert!((r.opening_high - 101.5).abs() < 1e-9);
        assert!((r.opening_low - 99.5).abs() < 1e-9);
        assert!((r.opening_range - 2.0).abs() < 1e-9);
    }

    #[test]
    fn detects_upper_breakout() {
        // Opening high 101.5. Bar 7 wicks to 102.0 → upper break.
        let mut bars: Vec<OhlcBar> = (0..6).map(|_| b(101.5, 99.5, 100.5)).collect();
        bars.push(b(102.0, 100.5, 101.8));
        let r = detect(&bars, &OrbConfig::default());
        assert!(r.upper_break.is_some());
        let ub = r.upper_break.unwrap();
        assert_eq!(ub.bar_index, 6);
        assert!(matches!(ub.direction, OrbDirection::Up));
        assert!((ub.breach_distance - 0.5).abs() < 1e-9);
    }

    #[test]
    fn detects_lower_breakdown() {
        let mut bars: Vec<OhlcBar> = (0..6).map(|_| b(101.5, 99.5, 100.5)).collect();
        bars.push(b(101.0, 99.0, 99.2));
        let r = detect(&bars, &OrbConfig::default());
        assert!(r.lower_break.is_some());
        let lb = r.lower_break.unwrap();
        assert!(matches!(lb.direction, OrbDirection::Down));
    }

    #[test]
    fn close_only_filters_wicks_outside_range() {
        // Opening high 101.5. Bar 7 wicks to 105 but closes at 101.
        // close_only mode shouldn't trigger upper break.
        let mut bars: Vec<OhlcBar> = (0..6).map(|_| b(101.5, 99.5, 100.5)).collect();
        bars.push(b(105.0, 100.0, 101.0));
        let cfg = OrbConfig { opening_bars: 6, atr: 0.0, close_only: true };
        let r = detect(&bars, &cfg);
        assert!(r.upper_break.is_none(), "wick shouldn't trigger close-only ORB");
    }

    #[test]
    fn atr_annotates_breach_in_atrs() {
        let mut bars: Vec<OhlcBar> = (0..6).map(|_| b(101.5, 99.5, 100.5)).collect();
        bars.push(b(103.5, 101.0, 103.0));    // breach 2 above 101.5
        let cfg = OrbConfig { opening_bars: 6, atr: 1.0, close_only: false };
        let r = detect(&bars, &cfg);
        let ub = r.upper_break.unwrap();
        assert!((ub.breach_atrs.unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn both_breaks_fire_in_same_session() {
        // Session shows an up break then a down break — both should fire.
        let mut bars: Vec<OhlcBar> = (0..6).map(|_| b(101.5, 99.5, 100.5)).collect();
        bars.push(b(102.5, 101.0, 102.0));    // upper break
        bars.push(b(102.0, 99.0, 99.5));      // lower break
        let r = detect(&bars, &OrbConfig::default());
        assert!(r.upper_break.is_some());
        assert!(r.lower_break.is_some());
        // Upper was first.
        assert!(r.upper_break.unwrap().bar_index < r.lower_break.unwrap().bar_index);
    }

    #[test]
    fn only_first_break_per_direction_recorded() {
        // Multiple upper breaks — only the first one is captured.
        let mut bars: Vec<OhlcBar> = (0..6).map(|_| b(101.5, 99.5, 100.5)).collect();
        bars.push(b(102.0, 100.5, 101.8));
        bars.push(b(103.0, 101.5, 102.5));
        let r = detect(&bars, &OrbConfig::default());
        assert_eq!(r.upper_break.unwrap().bar_index, 6,
            "first breaker is at bar 6, not bar 7");
    }
}
