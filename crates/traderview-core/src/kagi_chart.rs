//! Kagi Chart — price-only chart with directional lines drawn until
//! price reverses by a configurable amount.
//!
//! Each Kagi line continues in its current direction (up or down) while
//! price extends the move. When price reverses by `reversal_amount`
//! against the current line, a new line in the opposite direction
//! starts. The transition point is called a "shoulder" (down-to-up) or
//! "waist" (up-to-down). Kagi line thickness is traditionally varied —
//! thick (yang) when the line crosses ABOVE a prior peak, thin (yin)
//! when it crosses BELOW a prior trough.
//!
//! Reversal amount can be specified as either an absolute price
//! (`AmountKind::Absolute`) or a percentage of price (`AmountKind::Pct`).
//!
//! Pure compute. Companion to `renko`, `point_and_figure`,
//! `three_line_break`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AmountKind { #[default] Absolute, Pct }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KagiDirection { Up, Down }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KagiLine {
    pub direction: KagiDirection,
    pub anchor_price: f64,
    pub end_price: f64,
    pub source_index: usize,
}

pub fn compute(closes: &[f64], reversal: f64, kind: AmountKind) -> Vec<KagiLine> {
    let mut out = Vec::new();
    if closes.is_empty() || !reversal.is_finite() || reversal <= 0.0 { return out; }
    if closes.iter().any(|x| !x.is_finite() || *x <= 0.0) { return out; }
    let threshold = |anchor: f64| -> f64 {
        match kind {
            AmountKind::Absolute => reversal,
            AmountKind::Pct => anchor * reversal / 100.0,
        }
    };
    let mut direction: Option<KagiDirection> = None;
    let mut anchor = closes[0];
    let mut extreme = closes[0];
    let mut start_idx = 0_usize;
    for (i, &px) in closes.iter().enumerate().skip(1) {
        match direction {
            None => {
                let delta = px - anchor;
                if delta.abs() >= threshold(anchor) {
                    direction = Some(if delta > 0.0 { KagiDirection::Up } else { KagiDirection::Down });
                    extreme = px;
                }
            }
            Some(KagiDirection::Up) => {
                if px > extreme {
                    extreme = px;
                } else if extreme - px >= threshold(extreme) {
                    // Reversal.
                    out.push(KagiLine {
                        direction: KagiDirection::Up,
                        anchor_price: anchor,
                        end_price: extreme,
                        source_index: start_idx,
                    });
                    anchor = extreme;
                    extreme = px;
                    start_idx = i;
                    direction = Some(KagiDirection::Down);
                }
            }
            Some(KagiDirection::Down) => {
                if px < extreme {
                    extreme = px;
                } else if px - extreme >= threshold(extreme) {
                    out.push(KagiLine {
                        direction: KagiDirection::Down,
                        anchor_price: anchor,
                        end_price: extreme,
                        source_index: start_idx,
                    });
                    anchor = extreme;
                    extreme = px;
                    start_idx = i;
                    direction = Some(KagiDirection::Up);
                }
            }
        }
    }
    // Emit the in-progress line at the end.
    if let Some(d) = direction {
        out.push(KagiLine {
            direction: d,
            anchor_price: anchor,
            end_price: extreme,
            source_index: start_idx,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(compute(&[], 1.0, AmountKind::Absolute).is_empty());
        assert!(compute(&[100.0; 5], 0.0, AmountKind::Absolute).is_empty());
        assert!(compute(&[100.0, -1.0], 1.0, AmountKind::Absolute).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        assert!(compute(&[100.0, f64::NAN], 1.0, AmountKind::Absolute).is_empty());
    }

    #[test]
    fn flat_market_no_lines() {
        let r = compute(&[100.0; 20], 1.0, AmountKind::Absolute);
        assert!(r.is_empty());
    }

    #[test]
    fn pure_uptrend_yields_single_up_line() {
        let closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let r = compute(&closes, 1.0, AmountKind::Absolute);
        // After the final loop emit, we get one Up line.
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].direction, KagiDirection::Up);
        assert!((r[0].end_price - 119.0).abs() < 1e-9);
    }

    #[test]
    fn up_then_down_produces_two_lines() {
        let mut closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        closes.extend((0..20).map(|i| 119.0 - i as f64));
        let r = compute(&closes, 2.0, AmountKind::Absolute);
        // Up line then down line.
        assert!(r.len() >= 2);
        assert_eq!(r[0].direction, KagiDirection::Up);
        assert_eq!(r[1].direction, KagiDirection::Down);
    }

    #[test]
    fn pct_threshold_scales_with_price() {
        let closes: Vec<f64> = (0..20).map(|i| 100.0 * (1.0 + i as f64 * 0.005)).collect();
        let r = compute(&closes, 0.5, AmountKind::Pct);
        // Each move ≥ 0.5% triggers progression. Pure uptrend → single line.
        assert!(!r.is_empty());
        assert_eq!(r[0].direction, KagiDirection::Up);
    }
}
