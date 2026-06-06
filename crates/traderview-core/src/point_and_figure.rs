//! Point & Figure Chart Constructor — column-based price quantization.
//!
//! Converts a continuous price series into a sequence of fixed-size
//! up-columns (X) and down-columns (O):
//!
//!   - Active column extends in its direction by one box per
//!     `box_size` of price movement.
//!   - Column reverses (switches X↔O) only when price moves
//!     `reversal_boxes · box_size` against the current direction.
//!
//! Price is the only input; time is collapsed (a P&F column may span
//! one bar or many). Used for trend-clarity overlays, breakout
//! detection, and conventional P&F price-objective counting.
//!
//! Pure compute. Companion to `renko`, `tpo_profile`, `darvas_box`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnKind {
    X,
    O,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub kind: ColumnKind,
    pub start_box: i64,
    pub end_box: i64,
    pub bar_start_index: usize,
    pub bar_end_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PfReport {
    pub box_size: f64,
    pub reversal_boxes: usize,
    pub columns: Vec<Column>,
}

pub fn compute(prices: &[f64], box_size: f64, reversal_boxes: usize) -> Option<PfReport> {
    if prices.is_empty() || !box_size.is_finite() || box_size <= 0.0 || reversal_boxes < 2 {
        return None;
    }
    if prices.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut cols: Vec<Column> = Vec::new();
    // Initialize the first column from the first two distinct boxes.
    let first_box = price_to_box(prices[0], box_size);
    // Default to X column starting at first_box.
    let mut current = Column {
        kind: ColumnKind::X,
        start_box: first_box,
        end_box: first_box,
        bar_start_index: 0,
        bar_end_index: 0,
    };
    for (i, p) in prices.iter().enumerate().skip(1) {
        let b = price_to_box(*p, box_size);
        match current.kind {
            ColumnKind::X => {
                if b > current.end_box {
                    current.end_box = b;
                    current.bar_end_index = i;
                } else if (current.end_box - b) >= reversal_boxes as i64 {
                    cols.push(current.clone());
                    current = Column {
                        kind: ColumnKind::O,
                        start_box: current.end_box - 1,
                        end_box: b,
                        bar_start_index: i,
                        bar_end_index: i,
                    };
                }
            }
            ColumnKind::O => {
                if b < current.end_box {
                    current.end_box = b;
                    current.bar_end_index = i;
                } else if (b - current.end_box) >= reversal_boxes as i64 {
                    cols.push(current.clone());
                    current = Column {
                        kind: ColumnKind::X,
                        start_box: current.end_box + 1,
                        end_box: b,
                        bar_start_index: i,
                        bar_end_index: i,
                    };
                }
            }
        }
    }
    cols.push(current);
    Some(PfReport {
        box_size,
        reversal_boxes,
        columns: cols,
    })
}

fn price_to_box(price: f64, box_size: f64) -> i64 {
    (price / box_size).floor() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 1.0, 3).is_none());
    }

    #[test]
    fn invalid_params_return_none() {
        let prices = vec![100.0, 101.0, 102.0];
        assert!(compute(&prices, 0.0, 3).is_none());
        assert!(compute(&prices, -1.0, 3).is_none());
        assert!(compute(&prices, 1.0, 1).is_none()); // reversal < 2
        assert!(compute(&prices, f64::NAN, 3).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let prices = vec![100.0, f64::NAN, 102.0];
        assert!(compute(&prices, 1.0, 3).is_none());
    }

    #[test]
    fn flat_series_yields_single_column() {
        let prices = vec![100.0_f64; 20];
        let r = compute(&prices, 1.0, 3).unwrap();
        assert_eq!(r.columns.len(), 1);
        assert_eq!(r.columns[0].start_box, 100);
        assert_eq!(r.columns[0].end_box, 100);
    }

    #[test]
    fn pure_uptrend_yields_single_x_column() {
        let prices: Vec<f64> = (100..=120).map(|i| i as f64).collect();
        let r = compute(&prices, 1.0, 3).unwrap();
        assert_eq!(r.columns.len(), 1);
        assert_eq!(r.columns[0].kind, ColumnKind::X);
        assert_eq!(r.columns[0].start_box, 100);
        assert_eq!(r.columns[0].end_box, 120);
    }

    #[test]
    fn three_box_reversal_creates_new_column() {
        // Up to 110, then down by 3 boxes triggers O-column.
        let mut prices: Vec<f64> = (100..=110).map(|i| i as f64).collect();
        prices.extend([109.0, 108.0, 107.0]);
        let r = compute(&prices, 1.0, 3).unwrap();
        assert!(r.columns.len() >= 2);
        assert_eq!(r.columns[0].kind, ColumnKind::X);
        assert_eq!(r.columns[1].kind, ColumnKind::O);
    }

    #[test]
    fn small_pullback_does_not_reverse() {
        // 100→110, then 109 (1 box back) → still X column.
        let mut prices: Vec<f64> = (100..=110).map(|i| i as f64).collect();
        prices.push(109.0);
        let r = compute(&prices, 1.0, 3).unwrap();
        assert_eq!(r.columns.len(), 1);
        assert_eq!(r.columns[0].kind, ColumnKind::X);
    }

    #[test]
    fn box_size_scales_quantization() {
        // box_size = 5: only every 5-point move counts as a new box.
        let prices: Vec<f64> = (100..=120).map(|i| i as f64).collect();
        let r1 = compute(&prices, 1.0, 3).unwrap();
        let r5 = compute(&prices, 5.0, 3).unwrap();
        // Both still 1 column, but range of end_box - start_box differs.
        let range_1 = r1.columns[0].end_box - r1.columns[0].start_box;
        let range_5 = r5.columns[0].end_box - r5.columns[0].start_box;
        assert!(
            range_1 > range_5,
            "box=1 produces wider P&F range ({range_1}) than box=5 ({range_5})"
        );
    }

    #[test]
    fn output_carries_input_index_anchors() {
        let prices: Vec<f64> = (100..=120).map(|i| i as f64).collect();
        let r = compute(&prices, 1.0, 3).unwrap();
        assert_eq!(r.columns[0].bar_start_index, 0);
        assert_eq!(r.columns[0].bar_end_index, prices.len() - 1);
    }
}
