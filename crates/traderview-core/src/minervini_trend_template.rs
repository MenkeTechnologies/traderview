//! Mark Minervini Trend Template — 8-criterion screen for stocks in
//! a confirmed up-trend ("Trade Like a Stock Market Wizard", 2013).
//!
//! All 8 criteria must be true for a stock to qualify as in a Stage 2
//! advancing trend:
//!
//!   1. Price above the 150-day and 200-day MAs
//!   2. 150-day MA above 200-day MA
//!   3. 200-day MA trending up for at least 1 month (~22 days)
//!   4. 50-day MA above both 150-day and 200-day MAs
//!   5. Current price above 50-day MA
//!   6. Current price ≥ 30% above 52-week low
//!   7. Current price within 25% of 52-week high
//!   8. Relative-strength rank ≥ 70 (RS line vs S&P 500)
//!
//! Each criterion is checked independently and the result is the
//! conjunction. Per-criterion booleans returned for diagnostics.
//!
//! Pure compute. Companion to `weinstein_stages`, `vcp_pattern`,
//! `darvas_box`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinerviniReport {
    pub all_criteria_met: bool,
    pub criterion_1_price_above_150_and_200_ma: bool,
    pub criterion_2_ma150_above_ma200: bool,
    pub criterion_3_ma200_uptrend_one_month: bool,
    pub criterion_4_ma50_above_ma150_and_ma200: bool,
    pub criterion_5_price_above_ma50: bool,
    pub criterion_6_above_52w_low_by_30pct: bool,
    pub criterion_7_within_25pct_of_52w_high: bool,
    pub criterion_8_rs_rank_70_or_better: bool,
    pub ma_50: f64,
    pub ma_150: f64,
    pub ma_200: f64,
    pub last_price: f64,
}

pub fn classify(closes: &[f64], relative_strength_rank: f64) -> Option<MinerviniReport> {
    let n = closes.len();
    if n < 252 {
        return None;
    }
    if closes.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return None;
    }
    if !relative_strength_rank.is_finite() || !(0.0..=100.0).contains(&relative_strength_rank) {
        return None;
    }
    let last_price = *closes.last().unwrap();
    let sma = |period: usize, offset: usize| -> f64 {
        let end = n - offset;
        let start = end - period;
        closes[start..end].iter().sum::<f64>() / period as f64
    };
    let ma_50 = sma(50, 0);
    let ma_150 = sma(150, 0);
    let ma_200 = sma(200, 0);
    let ma_200_one_month_ago = sma(200, 22);
    let high_52w = closes[n - 252..n]
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let low_52w = closes[n - 252..n]
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let c1 = last_price > ma_150 && last_price > ma_200;
    let c2 = ma_150 > ma_200;
    let c3 = ma_200 > ma_200_one_month_ago;
    let c4 = ma_50 > ma_150 && ma_50 > ma_200;
    let c5 = last_price > ma_50;
    let c6 = last_price >= low_52w * 1.30;
    let c7 = last_price >= high_52w * 0.75;
    let c8 = relative_strength_rank >= 70.0;
    let all = c1 && c2 && c3 && c4 && c5 && c6 && c7 && c8;
    Some(MinerviniReport {
        all_criteria_met: all,
        criterion_1_price_above_150_and_200_ma: c1,
        criterion_2_ma150_above_ma200: c2,
        criterion_3_ma200_uptrend_one_month: c3,
        criterion_4_ma50_above_ma150_and_ma200: c4,
        criterion_5_price_above_ma50: c5,
        criterion_6_above_52w_low_by_30pct: c6,
        criterion_7_within_25pct_of_52w_high: c7,
        criterion_8_rs_rank_70_or_better: c8,
        ma_50,
        ma_150,
        ma_200,
        last_price,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        let closes = vec![100.0_f64; 100];
        assert!(classify(&closes, 80.0).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        let closes = vec![100.0_f64; 252];
        assert!(classify(&closes, -1.0).is_none());
        assert!(classify(&closes, 101.0).is_none());
        let mut bad = vec![100.0_f64; 252];
        bad[5] = -1.0;
        assert!(classify(&bad, 80.0).is_none());
    }

    #[test]
    fn flat_market_fails_uptrend_criteria() {
        let closes = vec![100.0_f64; 300];
        let r = classify(&closes, 80.0).unwrap();
        assert!(!r.all_criteria_met);
        // Flat MA → MA200 not uptrending.
        assert!(!r.criterion_3_ma200_uptrend_one_month);
    }

    #[test]
    fn strong_uptrend_with_rs_passes() {
        // Linear uptrend from 100 → 200 over 300 bars.
        let closes: Vec<f64> = (0..300).map(|i| 100.0 + i as f64 / 3.0).collect();
        let r = classify(&closes, 85.0).unwrap();
        // Various criteria should pass.
        assert!(r.criterion_2_ma150_above_ma200);
        assert!(r.criterion_3_ma200_uptrend_one_month);
        assert!(r.criterion_4_ma50_above_ma150_and_ma200);
        assert!(r.criterion_8_rs_rank_70_or_better);
    }

    #[test]
    fn downtrend_fails_majority_criteria() {
        let closes: Vec<f64> = (0..300).map(|i| 200.0 - i as f64 / 3.0).collect();
        let r = classify(&closes, 80.0).unwrap();
        assert!(!r.all_criteria_met);
        // 50-day MA should be below 200-day in a downtrend.
        assert!(!r.criterion_4_ma50_above_ma150_and_ma200);
        // Price below MAs.
        assert!(!r.criterion_1_price_above_150_and_200_ma);
    }

    #[test]
    fn low_rs_rank_fails_criterion_8() {
        let closes: Vec<f64> = (0..300).map(|i| 100.0 + i as f64 / 3.0).collect();
        let r = classify(&closes, 50.0).unwrap();
        assert!(!r.criterion_8_rs_rank_70_or_better);
    }

    #[test]
    fn mas_reported_in_output() {
        let closes: Vec<f64> = (0..300).map(|i| 100.0 + i as f64).collect();
        let r = classify(&closes, 80.0).unwrap();
        assert!(r.ma_50 > 0.0);
        assert!(r.ma_150 > 0.0);
        assert!(r.ma_200 > 0.0);
        assert_eq!(r.last_price, closes[299]);
    }
}
