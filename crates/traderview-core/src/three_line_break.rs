//! Three-Line Break (TLB) Chart — Japanese trend-only chart.
//!
//! Builds bars that change direction only when price reverses ENOUGH to
//! break the high (or low) of the most recent N lines (default 3):
//!
//! When in an uptrend: a new up-line is drawn whenever close > prior
//! up-line's close. A new down-line is drawn ONLY when close falls
//! below the low of the prior N up-lines (here, 3).
//!
//! When in a downtrend: mirrored.
//!
//! TLB eliminates noise by enforcing a "three-line break" rule before
//! changing direction.
//!
//! Pure compute. Default num_lines = 3. Companion to `renko`,
//! `kagi_chart`, `point_and_figure`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlbDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TlbLine {
    pub direction: TlbDirection,
    pub open: f64,
    pub close: f64,
    pub source_index: usize,
}

pub fn compute(closes: &[f64], num_lines: usize) -> Vec<TlbLine> {
    let mut out = Vec::new();
    if closes.is_empty() || num_lines < 1 {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let mut direction: Option<TlbDirection> = None;
    let mut last_close = closes[0];
    for (i, &px) in closes.iter().enumerate().skip(1) {
        match direction {
            None => {
                if px > last_close {
                    out.push(TlbLine {
                        direction: TlbDirection::Up,
                        open: last_close,
                        close: px,
                        source_index: i,
                    });
                    last_close = px;
                    direction = Some(TlbDirection::Up);
                } else if px < last_close {
                    out.push(TlbLine {
                        direction: TlbDirection::Down,
                        open: last_close,
                        close: px,
                        source_index: i,
                    });
                    last_close = px;
                    direction = Some(TlbDirection::Down);
                }
            }
            Some(TlbDirection::Up) => {
                if px > last_close {
                    out.push(TlbLine {
                        direction: TlbDirection::Up,
                        open: last_close,
                        close: px,
                        source_index: i,
                    });
                    last_close = px;
                } else {
                    // Need to break low of last `num_lines` up-lines to reverse.
                    let recent_up: Vec<&TlbLine> = out
                        .iter()
                        .rev()
                        .filter(|l| l.direction == TlbDirection::Up)
                        .take(num_lines)
                        .collect();
                    if recent_up.len() >= num_lines {
                        let break_level = recent_up
                            .iter()
                            .map(|l| l.open)
                            .fold(f64::INFINITY, f64::min);
                        if px < break_level {
                            out.push(TlbLine {
                                direction: TlbDirection::Down,
                                open: last_close,
                                close: px,
                                source_index: i,
                            });
                            last_close = px;
                            direction = Some(TlbDirection::Down);
                        }
                    }
                }
            }
            Some(TlbDirection::Down) => {
                if px < last_close {
                    out.push(TlbLine {
                        direction: TlbDirection::Down,
                        open: last_close,
                        close: px,
                        source_index: i,
                    });
                    last_close = px;
                } else {
                    let recent_down: Vec<&TlbLine> = out
                        .iter()
                        .rev()
                        .filter(|l| l.direction == TlbDirection::Down)
                        .take(num_lines)
                        .collect();
                    if recent_down.len() >= num_lines {
                        let break_level = recent_down
                            .iter()
                            .map(|l| l.open)
                            .fold(f64::NEG_INFINITY, f64::max);
                        if px > break_level {
                            out.push(TlbLine {
                                direction: TlbDirection::Up,
                                open: last_close,
                                close: px,
                                source_index: i,
                            });
                            last_close = px;
                            direction = Some(TlbDirection::Up);
                        }
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(compute(&[], 3).is_empty());
        assert!(compute(&[100.0; 5], 0).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        assert!(compute(&[100.0, f64::NAN], 3).is_empty());
    }

    #[test]
    fn flat_market_no_lines() {
        let r = compute(&[100.0; 20], 3);
        assert!(r.is_empty());
    }

    #[test]
    fn pure_uptrend_yields_all_up_lines() {
        let closes: Vec<f64> = (0..10).map(|i| 100.0 + i as f64).collect();
        let r = compute(&closes, 3);
        assert!(r.iter().all(|l| l.direction == TlbDirection::Up));
        assert_eq!(r.len(), 9); // each new close > prior → 9 lines
    }

    #[test]
    fn small_pullback_doesnt_flip() {
        // Up, up, up (3 lines), then a small pullback that doesn't
        // break the low of the prior 3 up-lines.
        let closes = vec![100.0, 102.0, 104.0, 106.0, 105.5];
        let r = compute(&closes, 3);
        // Bars 1..3 produce 3 up-lines. Bar 4 (105.5) doesn't break
        // bar 1's open (100), so no down-line.
        assert!(r.iter().all(|l| l.direction == TlbDirection::Up));
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn deep_pullback_flips_to_down() {
        let closes = vec![100.0, 102.0, 104.0, 106.0, 99.0];
        let r = compute(&closes, 3);
        // Last bar 99 < bar 1 open 100 → flip to down.
        assert!(r.iter().any(|l| l.direction == TlbDirection::Down));
    }
}
