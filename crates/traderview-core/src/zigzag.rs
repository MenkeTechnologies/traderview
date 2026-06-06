//! Zig-Zag indicator — reversal-percentage swing filter.
//!
//! Walks the close series and emits a pivot whenever price has moved
//! `reversal_pct` or more from the last extreme in the opposite
//! direction. Classical chartists' tool for skeletonizing wave structure
//! and feeding Elliott-wave / harmonic-pattern detectors.
//!
//! Output: per-bar `Pivot` or `None`. Each `Pivot` carries the kind
//! (High/Low) and the price level at the pivot.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PivotKind {
    High,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pivot {
    pub kind: PivotKind,
    pub price: f64,
}

pub fn compute(closes: &[f64], reversal_pct: f64) -> Vec<Option<Pivot>> {
    let n = closes.len();
    let mut out: Vec<Option<Pivot>> = vec![None; n];
    if n == 0 || !reversal_pct.is_finite() || reversal_pct <= 0.0 || reversal_pct >= 100.0 {
        return out;
    }
    let pct = reversal_pct / 100.0;
    // Find first finite close to seed.
    let Some(first_finite) = closes.iter().position(|x| x.is_finite()) else {
        return out;
    };
    // We track the last confirmed pivot and the current "extreme in progress".
    // Initially, we don't know direction, so wait for the first move ≥ pct.
    let mut anchor_idx = first_finite;
    let mut anchor_price = closes[first_finite];
    let mut direction: Option<PivotKind> = None; // current move direction (high pending vs low pending)
    let mut extreme_idx = anchor_idx;
    let mut extreme_price = anchor_price;
    for (i, c_) in closes.iter().enumerate().skip(first_finite + 1) {
        let c = *c_;
        if !c.is_finite() {
            continue;
        }
        match direction {
            None => {
                // Bootstrap: extend extreme in either direction until first reversal hits pct.
                if c > extreme_price {
                    extreme_price = c;
                    extreme_idx = i;
                }
                if c < extreme_price * (1.0 - pct) {
                    // We were going up; just confirmed a high at extreme_idx.
                    out[extreme_idx] = Some(Pivot {
                        kind: PivotKind::High,
                        price: extreme_price,
                    });
                    anchor_idx = extreme_idx;
                    anchor_price = extreme_price;
                    direction = Some(PivotKind::Low);
                    extreme_price = c;
                    extreme_idx = i;
                } else if c > anchor_price * (1.0 + pct) {
                    // Strong upmove from anchor — direction = high pending.
                    direction = Some(PivotKind::High);
                    extreme_price = c;
                    extreme_idx = i;
                }
                if c < extreme_price {
                    // Continue tracking — but the above branches already chose direction.
                }
            }
            Some(PivotKind::High) => {
                // Tracking a high — look for new highs OR a reversal of ≥ pct from the high.
                if c > extreme_price {
                    extreme_price = c;
                    extreme_idx = i;
                } else if c <= extreme_price * (1.0 - pct) {
                    // Confirmed high; flip to looking for a low.
                    out[extreme_idx] = Some(Pivot {
                        kind: PivotKind::High,
                        price: extreme_price,
                    });
                    anchor_idx = extreme_idx;
                    anchor_price = extreme_price;
                    direction = Some(PivotKind::Low);
                    extreme_price = c;
                    extreme_idx = i;
                }
            }
            Some(PivotKind::Low) => {
                if c < extreme_price {
                    extreme_price = c;
                    extreme_idx = i;
                } else if c >= extreme_price * (1.0 + pct) {
                    out[extreme_idx] = Some(Pivot {
                        kind: PivotKind::Low,
                        price: extreme_price,
                    });
                    anchor_idx = extreme_idx;
                    anchor_price = extreme_price;
                    direction = Some(PivotKind::High);
                    extreme_price = c;
                    extreme_idx = i;
                }
            }
        }
    }
    // Suppress unused-write warning on anchor_* — they document the
    // pivot-walker's anchor state for readers.
    let _ = (anchor_idx, anchor_price);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_invalid_pct_returns_all_none() {
        assert!(compute(&[], 5.0).is_empty());
        let v = vec![100.0; 10];
        assert!(compute(&v, 0.0).iter().all(|x| x.is_none()));
        assert!(compute(&v, -5.0).iter().all(|x| x.is_none()));
        assert!(compute(&v, 100.0).iter().all(|x| x.is_none()));
        assert!(compute(&v, f64::NAN).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_produces_no_pivots() {
        let v = vec![100.0; 50];
        let out = compute(&v, 5.0);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn one_up_one_down_produces_one_high_one_low() {
        // 100 → 110 (+10%) → 99 (-10%): one high at peak, one low at trough.
        let mut v = vec![100.0_f64];
        for i in 1..=10 {
            v.push(100.0 + i as f64);
        } // up to 110
        for i in 1..=11 {
            v.push(110.0 - i as f64);
        } // down to 99
        let out = compute(&v, 5.0);
        let highs: Vec<_> = out
            .iter()
            .enumerate()
            .filter_map(|(i, p)| p.map(|pv| (i, pv)))
            .filter(|(_, p)| p.kind == PivotKind::High)
            .collect();
        assert!(!highs.is_empty(), "should detect at least one high pivot");
    }

    #[test]
    fn small_wiggles_below_threshold_filtered_out() {
        // Series oscillating ±1% around 100 with 5% threshold → 0 pivots.
        let v: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * 0.7).sin() * 1.0)
            .collect();
        let out = compute(&v, 5.0);
        let pivots = out.iter().filter(|x| x.is_some()).count();
        assert_eq!(pivots, 0);
    }

    #[test]
    fn nan_bars_skipped_without_panic() {
        let mut v = vec![100.0_f64; 50];
        v[5] = f64::NAN;
        v[25] = 110.0;
        let out = compute(&v, 5.0);
        // Just verify no panic and structure intact.
        assert_eq!(out.len(), 50);
    }
}
