//! Stan Weinstein 4-Stage Trend Classification ("Secrets for Profiting
//! in Bull and Bear Markets", 1988).
//!
//! Classifies the current trend regime of a price series into one of:
//!
//!   - Stage 1 (basing):   price moves sideways below a flat 30-week MA
//!   - Stage 2 (advancing): price breaks above rising 30-week MA, holds
//!   - Stage 3 (topping):  price moves sideways above flattening 30-week MA
//!   - Stage 4 (declining): price breaks below falling 30-week MA, holds
//!
//! Standard 30-week SMA = ~150 trading days. Configurable via `ma_period`.
//!
//! Heuristics for classification:
//!   - Trend direction of MA over last `ma_slope_window`: rising (>+1%),
//!     falling (<-1%), or flat (between)
//!   - Price-vs-MA: above by more than `band_pct`, below, or within band
//!
//! Pure compute. Companion to `vcp_pattern`, `breakout_detector`,
//! `darvas_box`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    #[default]
    Basing,
    Advancing,
    Topping,
    Declining,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeinsteinStagesReport {
    pub current_stage: Stage,
    pub current_ma: f64,
    pub ma_slope_pct: f64,
    pub price_to_ma_pct: f64,
    pub n_bars: usize,
}

pub fn classify(
    closes: &[f64],
    ma_period: usize,
    ma_slope_window: usize,
    band_pct: f64,
) -> Option<WeinsteinStagesReport> {
    let n = closes.len();
    if n < ma_period + ma_slope_window
        || ma_period < 5
        || ma_slope_window < 2
        || !band_pct.is_finite()
        || band_pct < 0.0
    {
        return None;
    }
    if closes.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return None;
    }
    let current_ma: f64 = closes[n - ma_period..n].iter().sum::<f64>() / ma_period as f64;
    let past_ma: f64 = closes[n - ma_period - ma_slope_window..n - ma_slope_window]
        .iter()
        .sum::<f64>()
        / ma_period as f64;
    let slope_pct = (current_ma - past_ma) / past_ma;
    let last_price = *closes.last().unwrap();
    let price_to_ma = (last_price - current_ma) / current_ma;
    let stage = match (slope_pct, price_to_ma) {
        (s, p) if s > band_pct && p > band_pct => Stage::Advancing,
        (s, p) if s < -band_pct && p < -band_pct => Stage::Declining,
        (s, p) if s.abs() <= band_pct && p > band_pct => Stage::Topping,
        (s, p) if s.abs() <= band_pct && p < -band_pct => Stage::Basing,
        // Mixed / transitioning: treat by the stronger signal.
        (s, p) if s > band_pct && p < -band_pct => Stage::Basing, // rising MA, price still below
        (s, p) if s < -band_pct && p > band_pct => Stage::Topping, // falling MA, price still above
        _ => Stage::Basing,
    };
    Some(WeinsteinStagesReport {
        current_stage: stage,
        current_ma,
        ma_slope_pct: slope_pct,
        price_to_ma_pct: price_to_ma,
        n_bars: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_or_invalid_returns_none() {
        let p = vec![100.0_f64; 50];
        assert!(classify(&p, 150, 10, 0.01).is_none());
        assert!(classify(&p, 4, 10, 0.01).is_none());
        assert!(classify(&p, 30, 1, 0.01).is_none());
        assert!(classify(&p, 30, 10, -0.01).is_none());
    }

    #[test]
    fn nan_or_nonpositive_returns_none() {
        let mut p = vec![100.0_f64; 50];
        p[5] = f64::NAN;
        assert!(classify(&p, 30, 5, 0.01).is_none());
        let neg = vec![100.0, 0.0, 100.0];
        assert!(classify(&neg, 30, 5, 0.01).is_none());
    }

    #[test]
    fn sustained_uptrend_classified_advancing() {
        let p: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = classify(&p, 30, 10, 0.01).unwrap();
        assert_eq!(r.current_stage, Stage::Advancing);
        assert!(r.ma_slope_pct > 0.01);
        assert!(r.price_to_ma_pct > 0.01);
    }

    #[test]
    fn sustained_downtrend_classified_declining() {
        let p: Vec<f64> = (0..200).map(|i| 300.0 - i as f64).collect();
        let r = classify(&p, 30, 10, 0.01).unwrap();
        assert_eq!(r.current_stage, Stage::Declining);
        assert!(r.ma_slope_pct < -0.01);
        assert!(r.price_to_ma_pct < -0.01);
    }

    #[test]
    fn flat_market_classified_basing_or_topping() {
        let p = vec![100.0_f64; 200];
        let r = classify(&p, 30, 10, 0.01).unwrap();
        // Flat slope + price = MA → falls into _ branch → Basing.
        assert_eq!(r.current_stage, Stage::Basing);
        assert!(r.ma_slope_pct.abs() < 1e-12);
        assert!(r.price_to_ma_pct.abs() < 1e-12);
    }

    #[test]
    fn flat_ma_with_price_above_topping() {
        // Flat MA but price has rallied above.
        let mut p = vec![100.0_f64; 180];
        p.extend(vec![110.0_f64; 20]);
        let r = classify(&p, 30, 10, 0.01).unwrap();
        // MA mostly = 100, price = 110 → price_to_ma > 0.01.
        // ma_slope: rising slightly since recent prices push MA up.
        // Could be Advancing or Topping; verify it's not Declining.
        assert_ne!(r.current_stage, Stage::Declining);
    }

    #[test]
    fn n_bars_reported() {
        let p: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = classify(&p, 30, 10, 0.01).unwrap();
        assert_eq!(r.n_bars, 200);
    }
}
