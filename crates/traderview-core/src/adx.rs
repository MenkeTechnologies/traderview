//! ADX — Average Directional Index (Welles Wilder).
//!
//! Canonical trend-strength oscillator. Computes:
//!   - +DI (positive directional indicator)
//!   - -DI (negative directional indicator)
//!   - ADX = smoothed |+DI - -DI| / (+DI + -DI) × 100
//!
//! ADX > 25 = trending. ADX < 20 = chopping / no trend. Direction is
//! given by which DI is higher, not ADX itself.
//!
//! Pure compute. Uses the canonical Wilder smoothing (RMA / 1-period
//! EMA with alpha = 1/period). Input: bar series with high/low/close.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdxPoint {
    pub plus_di: f64,
    pub minus_di: f64,
    pub adx: f64,
}

/// Compute ADX series. Returns one point per input bar AFTER the
/// initial warmup window (length 2 × period). Earlier bars get zero
/// values — caller skips them.
pub fn compute(bars: &[Bar], period: usize) -> Vec<AdxPoint> {
    let n = bars.len();
    let mut out = vec![AdxPoint::default(); n];
    if n < 2 || period == 0 { return out; }
    let p = period as f64;
    let mut atr = 0.0;
    let mut plus_dm_smoothed = 0.0;
    let mut minus_dm_smoothed = 0.0;
    let mut dx_history: Vec<f64> = Vec::with_capacity(period);
    let mut adx_smoothed = 0.0;
    for i in 1..n {
        let prev_close = bars[i-1].close;
        let h = bars[i].high;
        let l = bars[i].low;
        let prev_h = bars[i-1].high;
        let prev_l = bars[i-1].low;
        let tr = (h - l).max((h - prev_close).abs()).max((l - prev_close).abs());
        let up_move = h - prev_h;
        let down_move = prev_l - l;
        let plus_dm = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
        let minus_dm = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };
        if i <= period {
            atr += tr;
            plus_dm_smoothed += plus_dm;
            minus_dm_smoothed += minus_dm;
            if i == period {
                // First smoothed values established.
            }
            continue;
        }
        // Wilder smoothing.
        atr = atr - atr / p + tr;
        plus_dm_smoothed  = plus_dm_smoothed  - plus_dm_smoothed  / p + plus_dm;
        minus_dm_smoothed = minus_dm_smoothed - minus_dm_smoothed / p + minus_dm;
        let plus_di = if atr > 0.0 { 100.0 * plus_dm_smoothed / atr } else { 0.0 };
        let minus_di = if atr > 0.0 { 100.0 * minus_dm_smoothed / atr } else { 0.0 };
        let di_sum = plus_di + minus_di;
        let dx = if di_sum > 0.0 { 100.0 * (plus_di - minus_di).abs() / di_sum } else { 0.0 };
        dx_history.push(dx);
        if dx_history.len() >= period {
            if dx_history.len() == period {
                // First ADX = mean of first `period` DX values.
                adx_smoothed = dx_history.iter().sum::<f64>() / p;
            } else {
                adx_smoothed = (adx_smoothed * (p - 1.0) + dx) / p;
            }
            out[i] = AdxPoint {
                plus_di,
                minus_di,
                adx: adx_smoothed,
            };
        } else {
            out[i] = AdxPoint {
                plus_di,
                minus_di,
                adx: 0.0,
            };
        }
    }
    out
}

/// Classify ADX into trend-strength tiers for the UI badge.
pub fn classify_adx(adx: f64) -> TrendStrength {
    if adx >= 50.0 { TrendStrength::Strong }
    else if adx >= 25.0 { TrendStrength::Trending }
    else if adx >= 20.0 { TrendStrength::Weak }
    else { TrendStrength::None }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendStrength { None, Weak, Trending, Strong }

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn series_under_warmup_emits_zero_adx() {
        let bars = vec![b(10.0, 9.0, 9.5); 5];
        let out = compute(&bars, 14);
        // All ADX values should be zero (insufficient data).
        for p in &out {
            assert_eq!(p.adx, 0.0);
        }
    }

    #[test]
    fn strong_uptrend_yields_high_plus_di_and_rising_adx() {
        // 50 bars of steady uptrend.
        let bars: Vec<Bar> = (1..=50).map(|i| {
            let c = 100.0 + i as f64;
            b(c + 0.5, c - 0.5, c)
        }).collect();
        let out = compute(&bars, 14);
        let last = out.last().unwrap();
        assert!(last.plus_di > last.minus_di,
            "uptrend → +DI > -DI");
        assert!(last.adx > 25.0,
            "strong uptrend should produce ADX > 25, got {}", last.adx);
    }

    #[test]
    fn strong_downtrend_yields_high_minus_di() {
        let bars: Vec<Bar> = (1..=50).map(|i| {
            let c = 200.0 - i as f64;
            b(c + 0.5, c - 0.5, c)
        }).collect();
        let out = compute(&bars, 14);
        let last = out.last().unwrap();
        assert!(last.minus_di > last.plus_di, "downtrend → -DI > +DI");
        assert!(last.adx > 25.0);
    }

    #[test]
    fn choppy_series_yields_low_adx() {
        // Oscillating series — no trend.
        let bars: Vec<Bar> = (1..=50).map(|i| {
            let base = 100.0 + (i % 3) as f64;    // 100, 101, 102, 100, 101, ...
            b(base + 0.5, base - 0.5, base)
        }).collect();
        let out = compute(&bars, 14);
        let last = out.last().unwrap();
        assert!(last.adx < 25.0,
            "choppy series should produce low ADX, got {}", last.adx);
    }

    // ─── classify ────────────────────────────────────────────────────

    #[test]
    fn classify_under_20_is_none() {
        assert_eq!(classify_adx(15.0), TrendStrength::None);
        assert_eq!(classify_adx(19.9), TrendStrength::None);
    }

    #[test]
    fn classify_20_to_25_is_weak() {
        assert_eq!(classify_adx(20.0), TrendStrength::Weak);
        assert_eq!(classify_adx(24.9), TrendStrength::Weak);
    }

    #[test]
    fn classify_25_to_50_is_trending() {
        assert_eq!(classify_adx(25.0), TrendStrength::Trending);
        assert_eq!(classify_adx(49.9), TrendStrength::Trending);
    }

    #[test]
    fn classify_50_plus_is_strong() {
        assert_eq!(classify_adx(50.0), TrendStrength::Strong);
        assert_eq!(classify_adx(80.0), TrendStrength::Strong);
    }
}
