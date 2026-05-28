//! Elder Ray — Bull Power / Bear Power (Alexander Elder).
//!
//! Decomposes the relationship between price and a 13-period EMA into
//! two signals:
//!
//!   bull_power = high − EMA(close, n)
//!   bear_power = low  − EMA(close, n)
//!
//! Both are signed: positive = price stretched above the EMA in that
//! direction; negative = below. Elder's "triple screen": only LONG when
//! bear_power is negative and rising; only SHORT when bull_power is
//! positive and falling. Standard n = 13.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElderRayReport {
    pub bull_power: Vec<Option<f64>>,
    pub bear_power: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], period: usize) -> ElderRayReport {
    let n = bars.len();
    let mut report = ElderRayReport {
        bull_power: vec![None; n],
        bear_power: vec![None; n],
    };
    if period == 0 || n < period {
        return report;
    }
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let ema = ema_closes(&closes, period);
    for i in 0..n {
        if let Some(e) = ema[i] {
            report.bull_power[i] = Some(bars[i].high - e);
            report.bear_power[i] = Some(bars[i].low - e);
        }
    }
    report
}

fn ema_closes(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let seed: f64 = values[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(seed);
    let mut prev = seed;
    for i in period..n {
        prev = alpha * values[i] + (1.0 - alpha) * prev;
        out[i] = Some(prev);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar { high: h, low: l, close: c }
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], 13);
        assert!(r.bull_power.is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 20];
        let r = compute(&bars, 0);
        assert!(r.bull_power.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_bull_and_bear_constant() {
        // EMA equals constant → bull = high - c = 1; bear = low - c = -1.
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 13);
        let bull = r.bull_power[29].expect("populated");
        let bear = r.bear_power[29].expect("populated");
        assert!((bull - 1.0).abs() < 1e-9);
        assert!((bear - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn strong_uptrend_positive_bull_power() {
        let bars: Vec<Bar> = (1..=50)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let r = compute(&bars, 13);
        let bull = r.bull_power[49].expect("populated");
        assert!(bull > 0.0, "uptrend bull power should be positive, got {bull}");
    }

    #[test]
    fn strong_downtrend_negative_bear_power() {
        let bars: Vec<Bar> = (1..=50)
            .map(|i| {
                let c = 200.0 - i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let r = compute(&bars, 13);
        let bear = r.bear_power[49].expect("populated");
        assert!(bear < 0.0);
    }

    #[test]
    fn huge_period_no_panic() {
        let bars = vec![b(101.0, 99.0, 100.0); 5];
        let r = compute(&bars, usize::MAX);
        assert!(r.bull_power.iter().all(|x| x.is_none()));
    }
}
