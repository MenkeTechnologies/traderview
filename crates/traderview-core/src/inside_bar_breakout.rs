//! Inside-Bar Breakout pattern detector.
//!
//! An **inside bar** has `high[i] <= high[i-1]` AND `low[i] >= low[i-1]`
//! — its range fits entirely within the prior bar's range. The "mother
//! bar" is bar `i-1`; the inside bar is `i`.
//!
//! A **breakout** confirms the pattern when a subsequent bar closes
//! beyond the mother bar's range:
//!   - Upside: bar `j` close > `mother.high`
//!   - Downside: bar `j` close < `mother.low`
//!
//! Detector emits one event per inside-bar setup; reports the breakout
//! direction (or None if neither side resolved within `confirm_within`
//! bars).
//!
//! Pure compute. Standard swing/momentum trader pattern.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IbConfig {
    /// Maximum bars to wait for a confirming close past the mother bar.
    pub confirm_within: usize,
    /// Require the inside bar's range to be at MOST this fraction of the mother's.
    pub max_range_ratio: f64,
}

impl Default for IbConfig {
    fn default() -> Self { Self { confirm_within: 5, max_range_ratio: 0.8 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakoutDirection { Up, Down, None }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IbEvent {
    pub mother_bar: usize,
    pub inside_bar: usize,
    pub breakout_bar: Option<usize>,
    pub direction: BreakoutDirection,
    pub range_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IbReport {
    pub events: Vec<IbEvent>,
    pub n_events: usize,
    pub n_resolved_up: usize,
    pub n_resolved_down: usize,
    pub n_unresolved: usize,
}

pub fn detect(bars: &[OhlcBar], cfg: &IbConfig) -> IbReport {
    let n = bars.len();
    if n < 2 { return IbReport::default(); }
    let mut events = Vec::new();
    for i in 1..n {
        let mother = bars[i - 1];
        let cur = bars[i];
        if cur.high > mother.high || cur.low < mother.low { continue; }
        let mother_range = mother.high - mother.low;
        let cur_range = cur.high - cur.low;
        if mother_range <= 0.0 { continue; }
        let ratio = cur_range / mother_range;
        if ratio > cfg.max_range_ratio { continue; }
        // Scan forward for the breakout. `confirm_within` is deserialized
        // from JSON so saturating_add guards against attacker-supplied
        // usize::MAX wrapping i+confirm_within to 0.
        let end = i.saturating_add(cfg.confirm_within).min(n - 1);
        let mut direction = BreakoutDirection::None;
        let mut breakout_at: Option<usize> = None;
        for (j, bar) in bars.iter().enumerate().take(end + 1).skip(i + 1) {
            if bar.close > mother.high {
                direction = BreakoutDirection::Up;
                breakout_at = Some(j);
                break;
            } else if bar.close < mother.low {
                direction = BreakoutDirection::Down;
                breakout_at = Some(j);
                break;
            }
        }
        events.push(IbEvent {
            mother_bar: i - 1, inside_bar: i,
            breakout_bar: breakout_at, direction, range_ratio: ratio,
        });
    }
    let n_events = events.len();
    let n_resolved_up = events.iter().filter(|e| matches!(e.direction, BreakoutDirection::Up)).count();
    let n_resolved_down = events.iter().filter(|e| matches!(e.direction, BreakoutDirection::Down)).count();
    let n_unresolved = events.iter().filter(|e| matches!(e.direction, BreakoutDirection::None)).count();
    IbReport { events, n_events, n_resolved_up, n_resolved_down, n_unresolved }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { open: o, high: h, low: l, close: c } }

    #[test]
    fn empty_or_short_returns_no_events() {
        assert_eq!(detect(&[], &IbConfig::default()).n_events, 0);
        assert_eq!(detect(&[b(100.0, 101.0, 99.0, 100.0)], &IbConfig::default()).n_events, 0);
    }

    #[test]
    fn inside_bar_with_up_breakout_fires() {
        let bars = vec![
            b(100.0, 105.0,  95.0, 102.0),    // 0 mother
            b(102.0, 103.0,  98.0, 100.0),    // 1 inside (range 5 vs mother 10 = 0.5)
            b(100.0, 104.0,  99.0, 103.0),    // 2 close 103 < 105
            b(103.0, 107.0, 102.0, 106.0),    // 3 close 106 > 105 → up breakout
        ];
        let r = detect(&bars, &IbConfig::default());
        assert_eq!(r.events.len(), 1);
        let e = r.events[0];
        assert_eq!(e.mother_bar, 0);
        assert_eq!(e.inside_bar, 1);
        assert_eq!(e.breakout_bar, Some(3));
        assert!(matches!(e.direction, BreakoutDirection::Up));
        assert_eq!(r.n_resolved_up, 1);
    }

    #[test]
    fn inside_bar_with_down_breakout_fires() {
        let bars = vec![
            b(100.0, 105.0,  95.0,  98.0),    // mother
            b(100.0, 103.0,  97.0, 100.0),    // inside
            b(100.0, 104.0,  94.0,  94.5),    // close 94.5 < 95 → down breakout
        ];
        let r = detect(&bars, &IbConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].direction, BreakoutDirection::Down));
        assert_eq!(r.n_resolved_down, 1);
    }

    #[test]
    fn non_inside_bar_doesnt_fire() {
        let bars = vec![
            b(100.0, 105.0,  95.0, 102.0),    // mother
            b(102.0, 106.0,  98.0, 103.0),    // high 106 > mother 105 — NOT inside
        ];
        assert_eq!(detect(&bars, &IbConfig::default()).n_events, 0);
    }

    #[test]
    fn oversized_inside_bar_rejected() {
        // Inside bar range = 8 vs mother range = 10 → ratio 0.8 — at the limit;
        // make the inside bar slightly larger to fail.
        let cfg = IbConfig { confirm_within: 5, max_range_ratio: 0.7 };
        let bars = vec![
            b(100.0, 105.0,  95.0, 102.0),
            b(100.0, 104.0,  96.0, 101.0),    // range 8/10 = 0.8 > 0.7
        ];
        assert_eq!(detect(&bars, &cfg).n_events, 0);
    }

    #[test]
    fn no_breakout_within_window_marks_unresolved() {
        let bars = vec![
            b(100.0, 105.0,  95.0, 102.0),    // mother
            b(100.0, 103.0,  97.0, 101.0),    // inside
            b(101.0, 104.0,  98.0, 102.0),    // ranges within
            b(102.0, 104.5,  98.5, 103.0),
            b(103.0, 104.8,  99.0, 102.5),
        ];
        let r = detect(&bars, &IbConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].direction, BreakoutDirection::None));
        assert!(r.events[0].breakout_bar.is_none());
        assert_eq!(r.n_unresolved, 1);
    }

    #[test]
    fn multiple_setups_tracked() {
        let bars = vec![
            b(100.0, 105.0,  95.0, 102.0),    // 0 mother A
            b(102.0, 103.0,  98.0, 100.0),    // 1 inside A
            b(100.0, 107.0,  94.0, 106.0),    // 2 breakout up (close 106 > 105)
            b(106.0, 110.0, 100.0, 108.0),    // 3 mother B
            b(108.0, 109.0, 103.0, 105.0),    // 4 inside B
            b(105.0, 111.0, 101.0, 110.5),    // 5 close 110.5 > 110 → up
        ];
        let r = detect(&bars, &IbConfig::default());
        assert_eq!(r.events.len(), 2);
        assert_eq!(r.n_resolved_up, 2);
    }

    #[test]
    fn zero_range_mother_skipped() {
        let bars = vec![
            b(100.0, 100.0, 100.0, 100.0),    // mother — zero range
            b(100.0, 100.0, 100.0, 100.0),    // would-be inside
        ];
        assert_eq!(detect(&bars, &IbConfig::default()).n_events, 0);
    }

    #[test]
    fn confirm_within_usize_max_does_not_overflow() {
        // Prior code did `(i + cfg.confirm_within).min(n - 1)`. With
        // confirm_within = usize::MAX the addition wraps to a tiny value
        // and the breakout scan is silently empty. Saturating_add fixes it.
        let cfg = IbConfig { confirm_within: usize::MAX, max_range_ratio: 0.8 };
        let bars = vec![
            b(100.0, 105.0,  95.0, 102.0),    // mother
            b(102.0, 103.0,  98.0, 100.0),    // inside
            b(100.0, 104.0,  99.0, 103.0),
            b(103.0, 107.0, 102.0, 106.0),    // close 106 > 105 → up breakout
        ];
        let r = detect(&bars, &cfg);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].breakout_bar, Some(3));
        assert!(matches!(r.events[0].direction, BreakoutDirection::Up));
    }
}
