//! Put-Call Ratio sentiment indicator.
//!
//! Classic contrarian gauge — when option traders are heavily skewed
//! toward puts, the market is typically near a bottom (excess fear);
//! heavy call buying often marks tops (excess greed).
//!
//! Two variants:
//!   - **Volume P/C** = put_volume / call_volume
//!   - **Open Interest P/C** = put_OI / call_OI
//!
//! Conventional thresholds (broad-market index):
//!   - < 0.7   = bullish extreme (too many calls — contrarian sell signal)
//!   - 0.7-1.0 = normal
//!   - > 1.0   = bearish extreme (too many puts — contrarian buy signal)
//!
//! For single-stock options thresholds shift (high-beta names typically
//! run higher P/C). Caller can override thresholds.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutCallInput {
    pub put_volume: u64,
    pub call_volume: u64,
    pub put_oi: u64,
    pub call_oi: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentimentZone {
    BullishExtreme,    // contrarian SELL (heavy call activity)
    Normal,
    BearishExtreme,    // contrarian BUY (heavy put activity)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PutCallReport {
    /// put_vol / call_vol. None when call_vol = 0.
    pub volume_pc_ratio: Option<f64>,
    pub oi_pc_ratio: Option<f64>,
    pub zone: SentimentZone,
}

impl Default for SentimentZone {
    fn default() -> Self { SentimentZone::Normal }
}

#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub bullish_extreme_below: f64,
    pub bearish_extreme_above: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self { bullish_extreme_below: 0.7, bearish_extreme_above: 1.0 }
    }
}

pub fn compute(input: &PutCallInput, thresh: &Thresholds) -> PutCallReport {
    let vol_pc = if input.call_volume == 0 { None } else {
        Some(input.put_volume as f64 / input.call_volume as f64)
    };
    let oi_pc = if input.call_oi == 0 { None } else {
        Some(input.put_oi as f64 / input.call_oi as f64)
    };
    // Use volume P/C for zone classification — more responsive than OI.
    let zone = match vol_pc {
        Some(v) if v < thresh.bullish_extreme_below => SentimentZone::BullishExtreme,
        Some(v) if v > thresh.bearish_extreme_above => SentimentZone::BearishExtreme,
        _ => SentimentZone::Normal,
    };
    PutCallReport {
        volume_pc_ratio: vol_pc,
        oi_pc_ratio: oi_pc,
        zone,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn i(pv: u64, cv: u64, poi: u64, coi: u64) -> PutCallInput {
        PutCallInput { put_volume: pv, call_volume: cv, put_oi: poi, call_oi: coi }
    }

    #[test]
    fn zero_call_volume_returns_none_ratio() {
        let r = compute(&i(100, 0, 100, 100), &Thresholds::default());
        assert!(r.volume_pc_ratio.is_none());
        assert_eq!(r.zone, SentimentZone::Normal, "no signal when undefined");
    }

    #[test]
    fn low_pc_ratio_bullish_extreme() {
        // 50 puts / 100 calls = 0.5 < 0.7 → bullish extreme (contrarian sell).
        let r = compute(&i(50, 100, 200, 300), &Thresholds::default());
        assert_eq!(r.volume_pc_ratio, Some(0.5));
        assert_eq!(r.zone, SentimentZone::BullishExtreme);
    }

    #[test]
    fn high_pc_ratio_bearish_extreme() {
        // 200 puts / 100 calls = 2.0 > 1.0 → bearish extreme (contrarian buy).
        let r = compute(&i(200, 100, 200, 100), &Thresholds::default());
        assert_eq!(r.volume_pc_ratio, Some(2.0));
        assert_eq!(r.zone, SentimentZone::BearishExtreme);
    }

    #[test]
    fn middling_pc_ratio_normal_zone() {
        // 0.85 between 0.7 and 1.0 → normal.
        let r = compute(&i(85, 100, 0, 0), &Thresholds::default());
        assert_eq!(r.volume_pc_ratio, Some(0.85));
        assert_eq!(r.zone, SentimentZone::Normal);
    }

    #[test]
    fn exactly_at_bullish_threshold_is_normal() {
        // 0.7 == threshold → NOT < (strict).
        let r = compute(&i(70, 100, 0, 0), &Thresholds::default());
        assert_eq!(r.zone, SentimentZone::Normal);
    }

    #[test]
    fn exactly_at_bearish_threshold_is_normal() {
        let r = compute(&i(100, 100, 0, 0), &Thresholds::default());
        assert_eq!(r.zone, SentimentZone::Normal);
    }

    #[test]
    fn oi_ratio_independent_of_zone_classification() {
        // Volume P/C = 0.5 → bullish extreme.
        // OI P/C = 2.0 (heavy long-term put interest) — separately reported.
        let r = compute(&i(50, 100, 200, 100), &Thresholds::default());
        assert_eq!(r.zone, SentimentZone::BullishExtreme);
        assert_eq!(r.oi_pc_ratio, Some(2.0));
    }

    #[test]
    fn custom_thresholds_shift_zones() {
        // For a high-beta single stock, the baseline P/C might be ~1.2.
        // Custom thresholds: bullish < 1.0, bearish > 1.5.
        let custom = Thresholds {
            bullish_extreme_below: 1.0,
            bearish_extreme_above: 1.5,
        };
        // P/C = 0.85 is now bullish extreme by these thresholds.
        let r = compute(&i(85, 100, 0, 0), &custom);
        assert_eq!(r.zone, SentimentZone::BullishExtreme);
    }

    #[test]
    fn zero_call_oi_oi_ratio_none() {
        let r = compute(&i(50, 100, 50, 0), &Thresholds::default());
        assert!(r.oi_pc_ratio.is_none());
    }
}
