//! Liquidity Void Detector — gaps in price action with no trades
//! (or unusually thin volume). Identifies bars where a strong move
//! occurred on minimal volume, marking levels likely to be revisited
//! ("fair-value gaps" / FVG in ICT terminology, but specifically the
//! ones where institutional algos haven't filled in the imbalance).
//!
//! A void is flagged when:
//!   bar.high - bar.low > avg_range · range_multiplier
//!   AND bar.volume < avg_volume · vol_multiplier
//!
//! For each void, reports the upper and lower price boundaries of the
//! void zone (bar.high and bar.low respectively).
//!
//! Pure compute. Defaults: period = 20, range_multiplier = 2.0,
//! vol_multiplier = 0.5.
//! Companion to `fair_value_gap`, `liquidity_grab`,
//! `liquidity_pool_detector`, `absorption_detector`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LiquidityVoid {
    pub bar_index: usize,
    pub upper: f64,
    pub lower: f64,
    pub range: f64,
    pub volume: f64,
}

pub fn detect(
    bars: &[Bar],
    period: usize,
    range_multiplier: f64,
    vol_multiplier: f64,
) -> Vec<LiquidityVoid> {
    let mut out = Vec::new();
    if period < 2
        || !range_multiplier.is_finite()
        || range_multiplier <= 0.0
        || !vol_multiplier.is_finite()
        || vol_multiplier <= 0.0
        || bars.len() < period + 1
    {
        return out;
    }
    if bars.iter().any(|b| {
        !b.high.is_finite() || !b.low.is_finite() || !b.volume.is_finite() || b.volume < 0.0
    }) {
        return out;
    }
    let p_f = period as f64;
    for i in period..bars.len() {
        let win = &bars[i - period..i];
        let avg_range: f64 = win.iter().map(|b| b.high - b.low).sum::<f64>() / p_f;
        let avg_vol: f64 = win.iter().map(|b| b.volume).sum::<f64>() / p_f;
        let cur = bars[i];
        let range = cur.high - cur.low;
        if range > avg_range * range_multiplier && cur.volume < avg_vol * vol_multiplier {
            out.push(LiquidityVoid {
                bar_index: i,
                upper: cur.high,
                lower: cur.low,
                range,
                volume: cur.volume,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, v: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            volume: v,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 1000.0); 30];
        assert!(detect(&bars, 1, 2.0, 0.5).is_empty());
        assert!(detect(&bars, 20, 0.0, 0.5).is_empty());
        assert!(detect(&bars, 20, 2.0, 0.0).is_empty());
        assert!(detect(&bars[..10], 20, 2.0, 0.5).is_empty());
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 1000.0); 30];
        bars[5] = b(f64::NAN, 99.0, 1000.0);
        assert!(detect(&bars, 20, 2.0, 0.5).is_empty());
        let mut bars2 = vec![b(101.0, 99.0, 1000.0); 30];
        bars2[5] = b(101.0, 99.0, -100.0);
        assert!(detect(&bars2, 20, 2.0, 0.5).is_empty());
    }

    #[test]
    fn flat_market_no_voids() {
        let bars = vec![b(101.0, 99.0, 1000.0); 30];
        assert!(detect(&bars, 20, 2.0, 0.5).is_empty());
    }

    #[test]
    fn high_range_low_volume_bar_detected() {
        // 20 quiet bars (range=2, vol=1000), then big-range low-volume bar.
        let mut bars = vec![b(101.0, 99.0, 1000.0); 20];
        // range = 10 (5×avg=2), volume = 100 (0.1×avg=1000) → void.
        bars.push(b(110.0, 100.0, 100.0));
        let voids = detect(&bars, 20, 2.0, 0.5);
        assert_eq!(voids.len(), 1);
        assert_eq!(voids[0].bar_index, 20);
        assert!((voids[0].upper - 110.0).abs() < 1e-9);
        assert!((voids[0].lower - 100.0).abs() < 1e-9);
    }

    #[test]
    fn high_range_normal_volume_not_void() {
        let mut bars = vec![b(101.0, 99.0, 1000.0); 20];
        // Range matches threshold but volume is normal → not a void.
        bars.push(b(110.0, 100.0, 1000.0));
        let voids = detect(&bars, 20, 2.0, 0.5);
        assert!(voids.is_empty());
    }

    #[test]
    fn low_range_low_volume_not_void() {
        let mut bars = vec![b(101.0, 99.0, 1000.0); 20];
        bars.push(b(101.0, 99.0, 100.0));
        let voids = detect(&bars, 20, 2.0, 0.5);
        assert!(voids.is_empty());
    }
}
