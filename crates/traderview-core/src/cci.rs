//! Commodity Channel Index — Donald Lambert (1980).
//!
//! Per bar:
//!   typical = (high + low + close) / 3
//!   sma_typical = N-period SMA of typical
//!   mean_dev = N-period mean of |typical - sma_typical|
//!   CCI = (typical - sma_typical) / (0.015 × mean_dev)
//!
//! Lambert chose 0.015 to make ~80% of values fall in [-100, +100].
//! Convention:
//!   - CCI > +100 → strong uptrend (or overbought, depending on use)
//!   - CCI < -100 → strong downtrend (or oversold)
//!   - |CCI| > 200 = extreme
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n < period || period == 0 { return out; }
    let typical: Vec<f64> = bars.iter().map(|b| (b.high + b.low + b.close) / 3.0).collect();
    for i in (period - 1)..n {
        let window = &typical[(i + 1 - period)..=i];
        let mean = window.iter().sum::<f64>() / period as f64;
        let mean_dev = window.iter().map(|v| (v - mean).abs()).sum::<f64>() / period as f64;
        out[i] = if mean_dev > 0.0 {
            (typical[i] - mean) / (0.015 * mean_dev)
        } else { 0.0 };
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CciZone {
    ExtremelyOversold,    // < -200
    Oversold,             // -200 to -100
    Neutral,              // -100 to +100
    Overbought,           // +100 to +200
    ExtremelyOverbought,  // > +200
}

pub fn classify(cci: f64) -> CciZone {
    if cci < -200.0 { CciZone::ExtremelyOversold }
    else if cci < -100.0 { CciZone::Oversold }
    else if cci <= 100.0 { CciZone::Neutral }
    else if cci <= 200.0 { CciZone::Overbought }
    else { CciZone::ExtremelyOverbought }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn flat_series_cci_zero() {
        let bars = vec![b(10.0, 9.0, 9.5); 20];
        let out = compute(&bars, 20);
        // Mean dev = 0 → CCI = 0 (div by 0 guard).
        assert_eq!(out[19], 0.0);
    }

    #[test]
    fn strong_uptrend_cci_positive() {
        let bars: Vec<Bar> = (1..=25).map(|i| {
            let c = 100.0 + i as f64;
            b(c + 0.5, c - 0.5, c)
        }).collect();
        let out = compute(&bars, 14);
        assert!(out[24] > 100.0, "strong uptrend should yield CCI > +100");
    }

    #[test]
    fn strong_downtrend_cci_negative() {
        let bars: Vec<Bar> = (1..=25).map(|i| {
            let c = 200.0 - i as f64;
            b(c + 0.5, c - 0.5, c)
        }).collect();
        let out = compute(&bars, 14);
        assert!(out[24] < -100.0);
    }

    #[test]
    fn at_window_average_cci_zero() {
        // Construct bars so latest typical = window SMA → CCI = 0.
        let mut bars = vec![b(10.0, 9.0, 9.5); 14];    // typical = 9.5 each
        bars.push(b(10.0, 9.0, 9.5));    // latest same → SMA = 9.5, dev = 0 → 0.
        let out = compute(&bars, 14);
        assert_eq!(out[14], 0.0);
    }

    // ─── classify ──────────────────────────────────────────────────────

    #[test]
    fn classify_within_100_neutral() {
        assert_eq!(classify(0.0), CciZone::Neutral);
        assert_eq!(classify(100.0), CciZone::Neutral);
        assert_eq!(classify(-100.0), CciZone::Neutral);
    }

    #[test]
    fn classify_over_100_overbought() {
        assert_eq!(classify(150.0), CciZone::Overbought);
    }

    #[test]
    fn classify_over_200_extremely_overbought() {
        assert_eq!(classify(250.0), CciZone::ExtremelyOverbought);
    }

    #[test]
    fn classify_under_minus_100_oversold() {
        assert_eq!(classify(-150.0), CciZone::Oversold);
    }

    #[test]
    fn classify_under_minus_200_extremely_oversold() {
        assert_eq!(classify(-300.0), CciZone::ExtremelyOversold);
    }
}
