//! Ichimoku Kinko Hyo Cloud (一目均衡表) — Goichi Hosoda.
//!
//! Five components, computed from bar series:
//!   - **Tenkan-sen** (Conversion): (9-period high + 9-period low) / 2
//!   - **Kijun-sen** (Base): (26-period high + 26-period low) / 2
//!   - **Senkou Span A** (Leading A): (Tenkan + Kijun) / 2, shifted +26
//!   - **Senkou Span B** (Leading B): (52-period high + 52-period low) / 2, shifted +26
//!   - **Chikou Span** (Lagging): close, shifted -26
//!
//! The "cloud" is the area between Senkou A and B, projected forward
//! 26 bars. Bullish when price > cloud AND A > B; bearish opposite.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct IchimokuPoint {
    pub tenkan: f64,
    pub kijun: f64,
    pub senkou_a: f64,
    pub senkou_b: f64,
    pub chikou: f64,
}

pub fn compute(bars: &[Bar]) -> Vec<IchimokuPoint> {
    let n = bars.len();
    let mut out = vec![IchimokuPoint::default(); n];
    if n == 0 {
        return out;
    }
    for i in 0..n {
        let tenkan = midpoint_window(bars, i, 9);
        let kijun = midpoint_window(bars, i, 26);
        out[i].tenkan = tenkan;
        out[i].kijun = kijun;
        // Senkou A is plotted at i+26 (caller renders it shifted forward).
        // We store the value at the bar where it was COMPUTED — caller
        // applies the shift in display.
        out[i].senkou_a = if tenkan != 0.0 && kijun != 0.0 {
            (tenkan + kijun) / 2.0
        } else {
            0.0
        };
        out[i].senkou_b = midpoint_window(bars, i, 52);
        // Chikou is the close, plotted at i-26.
        out[i].chikou = bars[i].close;
    }
    out
}

fn midpoint_window(bars: &[Bar], end: usize, period: usize) -> f64 {
    if period == 0 || end + 1 < period {
        return 0.0;
    }
    let start = end + 1 - period;
    let window = &bars[start..=end];
    let high = window
        .iter()
        .map(|b| b.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
    (high + low) / 2.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudBias {
    Bullish,
    Bearish,
    Neutral,
}

pub fn cloud_bias_at(point: IchimokuPoint, close: f64) -> CloudBias {
    if point.senkou_a == 0.0 || point.senkou_b == 0.0 {
        return CloudBias::Neutral;
    }
    let upper = point.senkou_a.max(point.senkou_b);
    let lower = point.senkou_a.min(point.senkou_b);
    if close > upper && point.senkou_a > point.senkou_b {
        CloudBias::Bullish
    } else if close < lower && point.senkou_a < point.senkou_b {
        CloudBias::Bearish
    } else {
        CloudBias::Neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn warmup_bars_have_zero_values_for_long_windows() {
        // 30 bars — kijun (26) and tenkan (9) computable, senkou_b (52) not.
        let bars: Vec<Bar> = (1..=30)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let out = compute(&bars);
        let last = out.last().unwrap();
        assert!(last.tenkan > 0.0, "tenkan should be computable at bar 30");
        assert!(last.kijun > 0.0, "kijun should be computable at bar 30");
        assert_eq!(last.senkou_b, 0.0, "senkou_b needs 52 bars");
    }

    #[test]
    fn tenkan_is_9_bar_midpoint() {
        // 10 bars with known range. Bar 9 (index 8): 9-bar window covers
        // bars 0..=8, lowest low + highest high over those.
        let bars: Vec<Bar> = (1..=10)
            .map(|i| {
                let h = i as f64 * 10.0;
                let l = i as f64 * 10.0 - 5.0;
                let c = (h + l) / 2.0;
                b(h, l, c)
            })
            .collect();
        let out = compute(&bars);
        // Bar 8 (9th bar): high = 90, low = 5 (from bar 1) → midpoint 47.5.
        assert_eq!(out[8].tenkan, 47.5);
    }

    #[test]
    fn senkou_a_is_average_of_tenkan_kijun() {
        let bars: Vec<Bar> = (1..=60)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let out = compute(&bars);
        let last = out.last().unwrap();
        let expected = (last.tenkan + last.kijun) / 2.0;
        assert!((last.senkou_a - expected).abs() < 1e-9);
    }

    #[test]
    fn chikou_equals_close() {
        let bars = vec![b(101.0, 99.0, 100.0)];
        let out = compute(&bars);
        assert_eq!(out[0].chikou, 100.0);
    }

    #[test]
    fn cloud_bias_bullish_when_close_above_cloud_and_a_above_b() {
        let pt = IchimokuPoint {
            tenkan: 105.0,
            kijun: 100.0,
            senkou_a: 102.5,
            senkou_b: 100.0,
            chikou: 0.0,
        };
        assert_eq!(cloud_bias_at(pt, 110.0), CloudBias::Bullish);
    }

    #[test]
    fn cloud_bias_bearish_when_close_below_cloud_and_a_below_b() {
        let pt = IchimokuPoint {
            tenkan: 95.0,
            kijun: 100.0,
            senkou_a: 97.5,
            senkou_b: 100.0,
            chikou: 0.0,
        };
        assert_eq!(cloud_bias_at(pt, 90.0), CloudBias::Bearish);
    }

    #[test]
    fn cloud_bias_neutral_when_close_inside_cloud() {
        let pt = IchimokuPoint {
            tenkan: 105.0,
            kijun: 100.0,
            senkou_a: 102.5,
            senkou_b: 100.0,
            chikou: 0.0,
        };
        // Close inside the cloud (100..102.5) → neutral.
        assert_eq!(cloud_bias_at(pt, 101.0), CloudBias::Neutral);
    }

    #[test]
    fn cloud_bias_neutral_when_senkou_values_zero() {
        let pt = IchimokuPoint::default();
        assert_eq!(cloud_bias_at(pt, 100.0), CloudBias::Neutral);
    }
}
