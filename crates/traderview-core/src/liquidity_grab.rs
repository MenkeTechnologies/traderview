//! Liquidity grab detector — institutional sweep-then-reverse signature.
//!
//! Distinct from `stop_hunt`. A stop-hunt is just *touching* a swing
//! level and reversing intrabar. A liquidity grab confirms a second leg:
//!
//!   1. Price sweeps a prior swing high (or low) by ≥ `min_sweep_atrs`.
//!   2. Within `confirm_within` bars, price reverses past the swing
//!      bar's opposite extreme (close below the sweeping bar's open for
//!      a high sweep; close above for a low sweep).
//!   3. The reversal then *continues* for at least `min_followthrough`
//!      bars without printing a new swing in the swept direction.
//!
//! This is the SMC/ICT setup pattern. Caller pre-computes swings via
//! `crate::swing_points::detect`.
//!
//! Pure compute.

use crate::swing_points::SwingPoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrabConfig {
    pub min_sweep_atrs: f64,
    pub confirm_within: usize,
    pub min_followthrough: usize,
}

impl Default for GrabConfig {
    fn default() -> Self {
        Self {
            min_sweep_atrs: 0.1,
            confirm_within: 3,
            min_followthrough: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrabSide {
    High,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GrabEvent {
    /// Index of the bar that swept the swing.
    pub sweep_bar: usize,
    /// Index of the bar that confirmed the reversal.
    pub confirm_bar: usize,
    /// Index of the swept swing.
    pub swing_index: usize,
    pub side: GrabSide,
    pub sweep_distance: f64,
    pub followthrough_bars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrabReport {
    pub events: Vec<GrabEvent>,
    pub n_events: usize,
}

pub fn detect(
    bars: &[OhlcBar],
    atr: &[f64],
    swings: &[SwingPoint],
    cfg: &GrabConfig,
) -> GrabReport {
    let n = bars.len();
    if n == 0 || atr.len() != n || swings.is_empty() {
        return GrabReport::default();
    }
    let mut events = Vec::new();
    for (sw_idx, sw) in swings.iter().enumerate() {
        let level = sw.price;
        let bar_idx = sw.index;
        let is_high = matches!(sw.kind, crate::swing_points::SwingKind::High);
        // Look forward starting one bar after the swing was made.
        for i in (bar_idx + 1)..n {
            let a = atr[i];
            if !(a.is_finite() && a > 0.0) {
                continue;
            }
            let swept = if is_high {
                bars[i].high > level + cfg.min_sweep_atrs * a
            } else {
                bars[i].low < level - cfg.min_sweep_atrs * a
            };
            if !swept {
                continue;
            }
            // Confirmation: reversal close past the sweeping bar's open.
            let mut confirm_at: Option<usize> = None;
            // saturating_add on the i side mirrors saturating_sub on the n
            // side — both endpoints are JSON-controlled and could overflow
            // usize on a hostile payload.
            let end = i
                .saturating_add(cfg.confirm_within)
                .min(n.saturating_sub(1));
            for j in (i + 1)..=end {
                let reversed = if is_high {
                    bars[j].close < bars[i].open && bars[j].close < level
                } else {
                    bars[j].close > bars[i].open && bars[j].close > level
                };
                if reversed {
                    confirm_at = Some(j);
                    break;
                }
            }
            let Some(confirm) = confirm_at else {
                break;
            };
            // Followthrough — count bars after confirm continuing in the
            // reversal direction without breaching the swept level again.
            let mut ft = 0usize;
            for j in (confirm + 1)..n {
                let still_reversed = if is_high {
                    bars[j].high <= bars[i].high
                } else {
                    bars[j].low >= bars[i].low
                };
                if still_reversed {
                    ft += 1;
                } else {
                    break;
                }
            }
            if ft < cfg.min_followthrough {
                break;
            }
            let sweep_distance = if is_high {
                bars[i].high - level
            } else {
                level - bars[i].low
            };
            events.push(GrabEvent {
                sweep_bar: i,
                confirm_bar: confirm,
                swing_index: sw_idx,
                side: if is_high {
                    GrabSide::High
                } else {
                    GrabSide::Low
                },
                sweep_distance,
                followthrough_bars: ft,
            });
            break; // One grab per swing.
        }
    }
    let n_events = events.len();
    GrabReport { events, n_events }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swing_points::{SwingKind, SwingPoint};

    fn b(o: f64, h: f64, l: f64, c: f64) -> OhlcBar {
        OhlcBar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    fn make_swing(idx: usize, price: f64, kind: SwingKind) -> SwingPoint {
        SwingPoint {
            index: idx,
            price,
            kind,
        }
    }

    #[test]
    fn empty_or_no_swings_returns_no_events() {
        let r = detect(&[], &[], &[], &GrabConfig::default());
        assert!(r.events.is_empty());
        let bars = vec![b(100.0, 101.0, 99.0, 100.5); 10];
        let atr = [1.0; 10];
        let r = detect(&bars, &atr, &[], &GrabConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn high_sweep_with_reversal_and_followthrough_fires() {
        // Swing high at bar 0 (price 110). Bar 1 sweeps to 111.
        // Bar 2 reverses close < bar1.open AND < 110. Bars 3,4 stay below 111.
        let bars = vec![
            b(108.0, 110.0, 107.0, 109.5), // 0  swing high
            b(109.5, 111.0, 109.0, 110.5), // 1  sweep (high 111 > 110+0.1*1=110.1)
            b(110.5, 110.6, 108.0, 108.5), // 2  confirm: close 108.5 < bar1.open=109.5 AND < 110
            b(108.5, 109.0, 107.5, 108.0), // 3  followthrough (high 109 < 111)
            b(108.0, 109.5, 107.0, 107.5), // 4  followthrough (high 109.5 < 111)
            b(107.5, 108.0, 106.5, 106.8), // 5  followthrough
        ];
        let atr = vec![1.0; bars.len()];
        let swings = vec![make_swing(0, 110.0, SwingKind::High)];
        let r = detect(&bars, &atr, &swings, &GrabConfig::default());
        assert_eq!(r.events.len(), 1, "expected 1 grab, got {}", r.events.len());
        let e = r.events[0];
        assert_eq!(e.sweep_bar, 1);
        assert_eq!(e.confirm_bar, 2);
        assert!(matches!(e.side, GrabSide::High));
        assert!(e.followthrough_bars >= 2);
    }

    #[test]
    fn low_sweep_with_reversal_fires() {
        let bars = vec![
            b(101.0, 102.0, 100.0, 101.0), // 0  swing low
            b(100.5, 101.0, 99.0, 99.5),   // 1  sweep (low 99 < 100-0.1)
            b(99.5, 102.0, 99.4, 101.5),   // 2  confirm: close 101.5 > bar1.open=100.5 AND > 100
            b(101.5, 102.5, 101.0, 102.0), // 3  followthrough (low 101 > 99)
            b(102.0, 103.0, 101.5, 102.5), // 4  followthrough
        ];
        let atr = vec![1.0; bars.len()];
        let swings = vec![make_swing(0, 100.0, SwingKind::Low)];
        let r = detect(&bars, &atr, &swings, &GrabConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].side, GrabSide::Low));
    }

    #[test]
    fn sweep_without_reversal_doesnt_fire() {
        let bars = vec![
            b(108.0, 110.0, 107.0, 109.5), // swing high
            b(109.5, 111.0, 109.0, 110.5), // sweep
            b(110.5, 112.0, 110.0, 111.5), // continuation, not reversal
            b(111.5, 113.0, 111.0, 112.5),
            b(112.5, 114.0, 112.0, 113.5),
        ];
        let atr = vec![1.0; bars.len()];
        let swings = vec![make_swing(0, 110.0, SwingKind::High)];
        let r = detect(&bars, &atr, &swings, &GrabConfig::default());
        assert!(r.events.is_empty(), "no reversal → no grab");
    }

    #[test]
    fn sweep_without_followthrough_doesnt_fire() {
        // Sweep + 1-bar reversal then immediate recovery.
        let bars = vec![
            b(108.0, 110.0, 107.0, 109.5), // swing high
            b(109.5, 111.0, 109.0, 110.5), // sweep
            b(110.5, 110.6, 108.0, 108.5), // confirm
            b(108.5, 112.0, 108.0, 111.5), // RECOVERS above sweeping high — followthrough=0
        ];
        let atr = vec![1.0; bars.len()];
        let swings = vec![make_swing(0, 110.0, SwingKind::High)];
        let r = detect(&bars, &atr, &swings, &GrabConfig::default());
        assert!(r.events.is_empty(), "no followthrough → no grab");
    }

    #[test]
    fn mismatched_atr_returns_no_events() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5); 10];
        let atr = vec![1.0; 5];
        let swings = vec![make_swing(0, 100.0, SwingKind::High)];
        let r = detect(&bars, &atr, &swings, &GrabConfig::default());
        assert!(r.events.is_empty());
    }
}
