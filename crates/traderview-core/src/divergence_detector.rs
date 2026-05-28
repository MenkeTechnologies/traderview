//! Generic divergence detector — works on any indicator vs price.
//!
//! Detects four divergence types between price pivots and indicator
//! pivots:
//!   - **RegularBullish**:  price LL + indicator HL → expected reversal up
//!   - **RegularBearish**:  price HH + indicator LH → expected reversal down
//!   - **HiddenBullish**:   price HL + indicator LL → trend continuation up
//!   - **HiddenBearish**:   price LH + indicator HH → trend continuation down
//!
//! Caller supplies the indicator series (RSI, MACD histogram, OBV, CMF, ...)
//! and the price series. The detector finds local pivots with `lookback`
//! bars on each side and pairs the most recent two pivots.
//!
//! Pure compute. Distinct from `rsi_divergence` (RSI-specific).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivergenceKind {
    RegularBullish,
    RegularBearish,
    HiddenBullish,
    HiddenBearish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DivergenceEvent {
    pub kind: DivergenceKind,
    pub prior_pivot_index: usize,
    pub recent_pivot_index: usize,
}

pub fn detect(
    prices: &[f64],
    indicator: &[Option<f64>],
    lookback: usize,
) -> Vec<DivergenceEvent> {
    let n = prices.len();
    if n != indicator.len() || lookback == 0 || n < 2 * lookback + 1 {
        return Vec::new();
    }
    // Find pivot highs/lows on prices.
    let highs = find_pivots(prices, lookback, true);
    let lows  = find_pivots(prices, lookback, false);
    let mut events = Vec::new();
    // Regular/Hidden bearish: pair adjacent price highs.
    for w in highs.windows(2) {
        let (prior, recent) = (w[0], w[1]);
        let (Some(p_ind), Some(r_ind)) = (indicator[prior], indicator[recent]) else { continue };
        let price_higher = prices[recent] > prices[prior];
        let ind_lower = r_ind < p_ind;
        if price_higher && ind_lower {
            events.push(DivergenceEvent {
                kind: DivergenceKind::RegularBearish,
                prior_pivot_index: prior, recent_pivot_index: recent,
            });
        } else if !price_higher && r_ind > p_ind {
            events.push(DivergenceEvent {
                kind: DivergenceKind::HiddenBearish,
                prior_pivot_index: prior, recent_pivot_index: recent,
            });
        }
    }
    // Regular/Hidden bullish: pair adjacent price lows.
    for w in lows.windows(2) {
        let (prior, recent) = (w[0], w[1]);
        let (Some(p_ind), Some(r_ind)) = (indicator[prior], indicator[recent]) else { continue };
        let price_lower = prices[recent] < prices[prior];
        let ind_higher = r_ind > p_ind;
        if price_lower && ind_higher {
            events.push(DivergenceEvent {
                kind: DivergenceKind::RegularBullish,
                prior_pivot_index: prior, recent_pivot_index: recent,
            });
        } else if !price_lower && r_ind < p_ind {
            events.push(DivergenceEvent {
                kind: DivergenceKind::HiddenBullish,
                prior_pivot_index: prior, recent_pivot_index: recent,
            });
        }
    }
    // Sort by recent_pivot_index ascending so caller sees chronological order.
    events.sort_by_key(|e| e.recent_pivot_index);
    events
}

fn find_pivots(values: &[f64], lookback: usize, find_high: bool) -> Vec<usize> {
    let n = values.len();
    let mut out = Vec::new();
    if n < 2 * lookback + 1 {
        return out;
    }
    for i in lookback..(n - lookback) {
        if !values[i].is_finite() { continue; }
        let center = values[i];
        let mut is_pivot = true;
        for k in 1..=lookback {
            let left = values[i - k];
            let right = values[i + k];
            if !left.is_finite() || !right.is_finite() {
                is_pivot = false;
                break;
            }
            if find_high {
                if left >= center || right >= center {
                    is_pivot = false; break;
                }
            } else if left <= center || right <= center {
                is_pivot = false; break;
            }
        }
        if is_pivot {
            out.push(i);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &[], 3).is_empty());
    }

    #[test]
    fn lookback_zero_returns_empty() {
        let p = vec![100.0; 30];
        let i: Vec<_> = (0..30).map(|_| Some(50.0)).collect();
        assert!(detect(&p, &i, 0).is_empty());
    }

    #[test]
    fn length_mismatch_returns_empty() {
        let p = vec![100.0; 30];
        let i: Vec<_> = (0..15).map(|_| Some(50.0)).collect();
        assert!(detect(&p, &i, 3).is_empty());
    }

    #[test]
    fn flat_series_yields_no_pivots_and_no_divergences() {
        let p = vec![100.0; 30];
        let i: Vec<_> = (0..30).map(|_| Some(50.0)).collect();
        let ev = detect(&p, &i, 3);
        assert!(ev.is_empty());
    }

    #[test]
    fn regular_bearish_detected_on_higher_high_lower_indicator() {
        // Price peaks at index 8 (105) and 18 (110). Indicator falls.
        let mut p = vec![100.0; 25];
        p[8] = 105.0;
        p[18] = 110.0;
        let mut ind: Vec<Option<f64>> = (0..25).map(|_| Some(50.0)).collect();
        ind[8] = Some(80.0);
        ind[18] = Some(60.0);    // lower indicator high
        let ev = detect(&p, &ind, 3);
        assert!(ev.iter().any(|e| e.kind == DivergenceKind::RegularBearish));
    }

    #[test]
    fn regular_bullish_detected_on_lower_low_higher_indicator() {
        let mut p = vec![100.0; 25];
        p[8] = 95.0;
        p[18] = 90.0;
        let mut ind: Vec<Option<f64>> = (0..25).map(|_| Some(50.0)).collect();
        ind[8] = Some(20.0);
        ind[18] = Some(30.0);    // higher indicator low
        let ev = detect(&p, &ind, 3);
        assert!(ev.iter().any(|e| e.kind == DivergenceKind::RegularBullish));
    }

    #[test]
    fn none_indicator_at_pivot_skipped() {
        let mut p = vec![100.0; 25];
        p[8] = 105.0;
        p[18] = 110.0;
        let mut ind: Vec<Option<f64>> = (0..25).map(|_| Some(50.0)).collect();
        ind[18] = None;
        let ev = detect(&p, &ind, 3);
        assert!(ev.is_empty());
    }
}
