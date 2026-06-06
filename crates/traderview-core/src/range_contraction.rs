//! Range-contraction pattern detector (NR4, NR7, inside bars).
//!
//! Toby Crabel's "narrowest range of last N bars" pattern is one of the
//! most robust expansion-precursor signals: when today's true range is
//! the SMALLEST of the prior N bars, the next session has an outsized
//! chance of a directional expansion.
//!
//! Two named variants:
//!   - **NR4**: today's range is the narrowest of the prior 4 bars (today + 3 lookbacks).
//!   - **NR7**: same idea over 7 bars — stricter and more predictive.
//!
//! Plus **inside-bar** detection: today's high < yesterday's high AND
//! today's low > yesterday's low (range fully nested). Classic Crabel
//! breakout precursor.
//!
//! Pure compute. Caller supplies OHLC bars.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    Nr4,
    Nr7,
    InsideBar,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PatternHit {
    pub bar_index: usize,
    pub kind: PatternKind,
    /// True range of the qualifying bar (high - low; prior close not used).
    pub range: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternReport {
    pub hits: Vec<PatternHit>,
    pub nr4_count: usize,
    pub nr7_count: usize,
    pub inside_bar_count: usize,
}

pub fn detect(bars: &[OhlcBar]) -> PatternReport {
    let n = bars.len();
    if n == 0 {
        return PatternReport::default();
    }
    let ranges: Vec<f64> = bars.iter().map(|b| b.high - b.low).collect();
    let mut hits = Vec::new();
    let mut nr4 = 0usize;
    let mut nr7 = 0usize;
    let mut inside = 0usize;
    for i in 0..n {
        // NR4: bar i has the smallest range among bars i-3..=i (need i >= 3).
        if i >= 3 {
            let window = &ranges[i - 3..=i];
            if window.iter().all(|&r| ranges[i] <= r) && ranges[i] > 0.0 {
                // Strict: today must be STRICTLY smaller than at least one prior bar.
                // Otherwise a flat range of 1 bar trivially "ties".
                let strictly_smaller = window[..window.len() - 1].iter().any(|&r| ranges[i] < r);
                if strictly_smaller {
                    hits.push(PatternHit {
                        bar_index: i,
                        kind: PatternKind::Nr4,
                        range: ranges[i],
                    });
                    nr4 += 1;
                }
            }
        }
        // NR7: same over 7 bars.
        if i >= 6 {
            let window = &ranges[i - 6..=i];
            if window.iter().all(|&r| ranges[i] <= r) && ranges[i] > 0.0 {
                let strictly_smaller = window[..window.len() - 1].iter().any(|&r| ranges[i] < r);
                if strictly_smaller {
                    hits.push(PatternHit {
                        bar_index: i,
                        kind: PatternKind::Nr7,
                        range: ranges[i],
                    });
                    nr7 += 1;
                }
            }
        }
        // Inside bar: today fully nested inside yesterday.
        if i >= 1 {
            let today = bars[i];
            let yesterday = bars[i - 1];
            if today.high < yesterday.high && today.low > yesterday.low {
                hits.push(PatternHit {
                    bar_index: i,
                    kind: PatternKind::InsideBar,
                    range: ranges[i],
                });
                inside += 1;
            }
        }
    }
    PatternReport {
        hits,
        nr4_count: nr4,
        nr7_count: nr7,
        inside_bar_count: inside,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> OhlcBar {
        OhlcBar {
            high: h,
            low: l,
            close: (h + l) / 2.0,
        }
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let r = detect(&[]);
        assert!(r.hits.is_empty());
        assert_eq!(r.nr4_count, 0);
    }

    #[test]
    fn nr4_fires_when_today_is_narrowest_of_last_four() {
        // Ranges: 5, 4, 3, 2 → bar 3 has the narrowest of the last 4. NR4 hit.
        let bars = vec![
            b(105.0, 100.0),
            b(104.0, 100.0),
            b(103.0, 100.0),
            b(102.0, 100.0),
        ];
        let r = detect(&bars);
        let nr4_hits: Vec<_> = r
            .hits
            .iter()
            .filter(|h| matches!(h.kind, PatternKind::Nr4))
            .collect();
        assert!(!nr4_hits.is_empty(), "expected NR4 at i=3");
        assert_eq!(nr4_hits[0].bar_index, 3);
    }

    #[test]
    fn nr7_requires_seven_bars_of_lookback() {
        // Two bars: nothing fires.
        let bars: Vec<OhlcBar> = (0..2).map(|_| b(101.0, 100.0)).collect();
        let r = detect(&bars);
        assert_eq!(r.nr7_count, 0);
        // Seven bars with the last being narrowest: NR7 hit.
        let mut bars: Vec<OhlcBar> = (0..6).map(|i| b(110.0 - i as f64, 100.0)).collect();
        bars.push(b(101.0, 100.0));
        let r = detect(&bars);
        let nr7_hits: Vec<_> = r
            .hits
            .iter()
            .filter(|h| matches!(h.kind, PatternKind::Nr7))
            .collect();
        assert_eq!(nr7_hits.len(), 1);
        assert_eq!(nr7_hits[0].bar_index, 6);
    }

    #[test]
    fn inside_bar_detected_only_when_fully_nested() {
        // Yesterday: 105/95, Today: 103/97 → fully nested → inside.
        let bars = vec![b(105.0, 95.0), b(103.0, 97.0)];
        let r = detect(&bars);
        assert_eq!(r.inside_bar_count, 1);
        // Today: 103/95 → high < yesterday's high but low == yesterday's low (not strict).
        let bars = vec![b(105.0, 95.0), b(103.0, 95.0)];
        let r = detect(&bars);
        assert_eq!(r.inside_bar_count, 0, "boundary touch is not nested");
        // Today: 106/97 → high > yesterday's high (an outside bar, not inside).
        let bars = vec![b(105.0, 95.0), b(106.0, 97.0)];
        let r = detect(&bars);
        assert_eq!(r.inside_bar_count, 0);
    }

    #[test]
    fn flat_ranges_do_not_emit_nr_pattern() {
        // All bars range = 5. No bar is STRICTLY narrower than the others.
        let bars = vec![b(105.0, 100.0); 10];
        let r = detect(&bars);
        assert_eq!(r.nr4_count, 0, "flat ranges should not emit NR4");
        assert_eq!(r.nr7_count, 0, "flat ranges should not emit NR7");
    }

    #[test]
    fn zero_range_bars_dont_panic() {
        // Open == High == Low (no movement) should be handled gracefully.
        let bars = vec![b(100.0, 100.0); 8];
        let r = detect(&bars);
        assert_eq!(r.nr4_count, 0);
        assert_eq!(r.nr7_count, 0);
    }

    #[test]
    fn nr7_implies_nr4_when_both_satisfied() {
        // 7 bars with widening pattern: 10, 9, 8, 7, 6, 5, 4 — bar 6 (range 4)
        // is narrowest of last 4 AND last 7.
        let bars: Vec<OhlcBar> = (0..7).map(|i| b(110.0 - i as f64, 100.0)).collect();
        let r = detect(&bars);
        let last_bar_hits: Vec<_> = r.hits.iter().filter(|h| h.bar_index == 6).collect();
        let kinds: Vec<PatternKind> = last_bar_hits.iter().map(|h| h.kind).collect();
        assert!(
            kinds.contains(&PatternKind::Nr4) && kinds.contains(&PatternKind::Nr7),
            "narrowest-of-7 should also be narrowest-of-4, got {:?}",
            kinds
        );
    }
}
