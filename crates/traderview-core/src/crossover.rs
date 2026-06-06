//! Generic series-crossover detector.
//!
//! "Golden cross" (50d MA crosses above 200d MA), MACD signal cross,
//! RSI crossing 50, stochastic %K vs %D — every one of these is the same
//! pattern: series A crosses series B from below or above. This module
//! detects every such crossing in a parallel-pair input and emits an
//! event log with direction + bar index.
//!
//! Caller pre-computes both series (e.g. via `indicators::sma` and
//! `indicators::ema`). Outputs the LAST crossover plus the full list.
//!
//! Pure compute. Pre-warmup positions where either series is None are
//! treated as "no opinion" and never emit crosses.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossDirection {
    /// Series A crossed UP through series B (bullish for "golden cross"-style).
    Up,
    /// Series A crossed DOWN through series B (bearish for "death cross").
    Down,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Crossover {
    pub bar_index: usize,
    pub direction: CrossDirection,
    pub a_value: f64,
    pub b_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossReport {
    pub crosses: Vec<Crossover>,
    pub last_cross: Option<Crossover>,
    /// Bars since the last cross (None if no cross occurred or input was empty).
    pub bars_since_last: Option<usize>,
}

pub fn detect(a: &[Option<f64>], b: &[Option<f64>]) -> CrossReport {
    let n = a.len().min(b.len());
    if n < 2 {
        return CrossReport::default();
    }
    let mut crosses = Vec::new();
    for i in 1..n {
        let (Some(a_prev), Some(b_prev)) = (a[i - 1], b[i - 1]) else {
            continue;
        };
        let (Some(a_cur), Some(b_cur)) = (a[i], b[i]) else {
            continue;
        };
        // Skip ties on either edge — a cross requires strict prior position.
        if a_prev == b_prev || a_cur == b_cur {
            continue;
        }
        if a_prev < b_prev && a_cur > b_cur {
            crosses.push(Crossover {
                bar_index: i,
                direction: CrossDirection::Up,
                a_value: a_cur,
                b_value: b_cur,
            });
        } else if a_prev > b_prev && a_cur < b_cur {
            crosses.push(Crossover {
                bar_index: i,
                direction: CrossDirection::Down,
                a_value: a_cur,
                b_value: b_cur,
            });
        }
    }
    let last_cross = crosses.last().copied();
    let bars_since_last = last_cross.map(|c| n.saturating_sub(1).saturating_sub(c.bar_index));
    CrossReport {
        crosses,
        last_cross,
        bars_since_last,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[f64]) -> Vec<Option<f64>> {
        v.iter().map(|x| Some(*x)).collect()
    }

    #[test]
    fn empty_or_too_short_input_returns_empty() {
        assert!(detect(&[], &[]).crosses.is_empty());
        assert!(detect(&[Some(1.0)], &[Some(2.0)]).crosses.is_empty());
    }

    #[test]
    fn upward_cross_detected_with_correct_direction() {
        // A = [1, 3], B = [2, 2]. A was below, now above → Up cross at i=1.
        let r = detect(&s(&[1.0, 3.0]), &s(&[2.0, 2.0]));
        assert_eq!(r.crosses.len(), 1);
        assert_eq!(r.crosses[0].bar_index, 1);
        assert!(matches!(r.crosses[0].direction, CrossDirection::Up));
        assert_eq!(r.bars_since_last, Some(0));
    }

    #[test]
    fn downward_cross_detected() {
        let r = detect(&s(&[3.0, 1.0]), &s(&[2.0, 2.0]));
        assert!(matches!(r.crosses[0].direction, CrossDirection::Down));
    }

    #[test]
    fn touch_without_breach_doesnt_emit() {
        // A=[1, 2], B=[2, 2]. A approaches but only touches B — no cross.
        let r = detect(&s(&[1.0, 2.0]), &s(&[2.0, 2.0]));
        assert!(
            r.crosses.is_empty(),
            "tying B doesn't count as a cross, got {:?}",
            r.crosses
        );
    }

    #[test]
    fn pre_warmup_none_values_dont_emit() {
        // First slot is None → skip; A=2,B=3 then A=4,B=3 → would be a cross
        // if the first bar had values, but the None at start blocks it.
        let r = detect(&[None, Some(4.0)], &[Some(3.0), Some(3.0)]);
        assert!(r.crosses.is_empty());
    }

    #[test]
    fn multiple_crosses_tracked_in_order() {
        // Sine vs flat — multiple crosses.
        let a = s(&[1.0, 3.0, 1.0, 3.0, 1.0]);
        let b = s(&[2.0, 2.0, 2.0, 2.0, 2.0]);
        let r = detect(&a, &b);
        assert_eq!(r.crosses.len(), 4, "expected 4 crosses in zig-zag");
        // Last one is Down (3 → 1 below 2).
        assert!(matches!(
            r.last_cross.unwrap().direction,
            CrossDirection::Down
        ));
        assert_eq!(r.bars_since_last, Some(0));
    }

    #[test]
    fn bars_since_last_grows_when_no_recent_cross() {
        // Cross at i=1, then no more crosses for 3 more bars.
        // a: [1, 3, 4, 5, 6]  b: [2, 2, 2, 2, 2]
        // Cross at i=1, then a stays above b → no more crosses.
        let a = s(&[1.0, 3.0, 4.0, 5.0, 6.0]);
        let b = s(&[2.0, 2.0, 2.0, 2.0, 2.0]);
        let r = detect(&a, &b);
        assert_eq!(r.crosses.len(), 1);
        assert_eq!(r.bars_since_last, Some(3)); // last bar - cross bar = 4 - 1 = 3
    }
}
