//! Gann Swing Chart — W.D. Gann's directional swing chart.
//!
//! Tracks the prevailing swing direction (up or down). The direction
//! reverses only when an N-bar consecutive close confirms the
//! opposite move. Records:
//!   - swing_direction at each bar (Up | Down | Initial)
//!   - swing_anchor_price: the price at the most recent reversal
//!   - swing_high_so_far / swing_low_so_far during the current leg
//!
//! Reversal rule (Gann's standard 2-bar swing):
//!   In an Up swing: reverse to Down if (close < prior swing low)
//!     after `reversal_bars` consecutive bars closing lower than the
//!     swing high.
//!   In a Down swing: reverse to Up by the symmetric condition.
//!
//! Pure compute. Default reversal_bars = 2.
//! Companion to `swing_points`, `darvas_box`, `zigzag`,
//! `gann_high_low_activator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SwingDirection {
    #[default]
    Initial,
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GannSwingReport {
    pub direction: Vec<SwingDirection>,
    pub swing_anchor: Vec<Option<f64>>,
    pub current_high: Vec<Option<f64>>,
    pub current_low: Vec<Option<f64>>,
    pub reversal_bars: usize,
}

pub fn compute(bars: &[Bar], reversal_bars: usize) -> GannSwingReport {
    let n = bars.len();
    let mut report = GannSwingReport {
        direction: vec![SwingDirection::Initial; n],
        swing_anchor: vec![None; n],
        current_high: vec![None; n],
        current_low: vec![None; n],
        reversal_bars,
    };
    if reversal_bars < 1 || n < reversal_bars + 1 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return report;
    }
    let mut dir = SwingDirection::Initial;
    let mut cur_high = bars[0].high;
    let mut cur_low = bars[0].low;
    let mut anchor = bars[0].close;
    let mut down_count = 0_usize;
    let mut up_count = 0_usize;
    report.direction[0] = dir;
    report.swing_anchor[0] = Some(anchor);
    report.current_high[0] = Some(cur_high);
    report.current_low[0] = Some(cur_low);
    for i in 1..n {
        let bar = bars[i];
        match dir {
            SwingDirection::Initial => {
                if bar.close > anchor {
                    dir = SwingDirection::Up;
                    cur_high = bar.high;
                    cur_low = bar.low;
                } else if bar.close < anchor {
                    dir = SwingDirection::Down;
                    cur_high = bar.high;
                    cur_low = bar.low;
                }
            }
            SwingDirection::Up => {
                if bar.high > cur_high {
                    cur_high = bar.high;
                }
                if bar.close < bars[i - 1].close {
                    down_count += 1;
                } else {
                    down_count = 0;
                }
                if down_count >= reversal_bars && bar.close < cur_low {
                    dir = SwingDirection::Down;
                    anchor = cur_high;
                    cur_low = bar.low;
                    cur_high = bar.high;
                    down_count = 0;
                }
            }
            SwingDirection::Down => {
                if bar.low < cur_low {
                    cur_low = bar.low;
                }
                if bar.close > bars[i - 1].close {
                    up_count += 1;
                } else {
                    up_count = 0;
                }
                if up_count >= reversal_bars && bar.close > cur_high {
                    dir = SwingDirection::Up;
                    anchor = cur_low;
                    cur_high = bar.high;
                    cur_low = bar.low;
                    up_count = 0;
                }
            }
        }
        report.direction[i] = dir;
        report.swing_anchor[i] = Some(anchor);
        report.current_high[i] = Some(cur_high);
        report.current_low[i] = Some(cur_low);
    }
    report
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
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 0);
        assert!(r.direction.iter().all(|d| *d == SwingDirection::Initial));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 2);
        assert!(r.swing_anchor.iter().all(|x| x.is_none()));
    }

    #[test]
    fn rising_close_establishes_up_swing() {
        let mut bars = vec![b(101.0, 99.0, 100.0)];
        bars.push(b(102.0, 100.0, 101.0));
        bars.push(b(103.0, 101.0, 102.0));
        bars.push(b(104.0, 102.0, 103.0));
        let r = compute(&bars, 2);
        assert_eq!(r.direction[3], SwingDirection::Up);
    }

    #[test]
    fn falling_close_establishes_down_swing() {
        let mut bars = vec![b(101.0, 99.0, 100.0)];
        bars.push(b(100.0, 98.0, 99.0));
        bars.push(b(99.0, 97.0, 98.0));
        bars.push(b(98.0, 96.0, 97.0));
        let r = compute(&bars, 2);
        assert_eq!(r.direction[3], SwingDirection::Down);
    }

    #[test]
    fn current_high_tracks_max_during_up_swing() {
        let bars = vec![
            b(101.0, 99.0, 100.0),
            b(105.0, 100.0, 104.0),
            b(108.0, 103.0, 107.0),
            b(110.0, 105.0, 109.0),
        ];
        let r = compute(&bars, 2);
        let last = 3;
        assert!((r.current_high[last].unwrap() - 110.0).abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 2);
        assert_eq!(r.direction.len(), 30);
        assert_eq!(r.swing_anchor.len(), 30);
    }
}
