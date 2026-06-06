//! Traders' Dynamic Index (TDI) — Dean Malone.
//!
//! Composite indicator combining smoothed RSI with Bollinger-style
//! volatility bands and a fast/slow signal crossover:
//!
//!   rsi_t     = standard 14-bar RSI(close)
//!   price     = SMA(rsi, 2)            "Green line" / RSI Price
//!   signal    = SMA(rsi, 7)            "Red line" / Signal
//!   midband   = SMA(rsi, 34)           "Yellow line" / Market Base Line
//!   bb_upper  = midband + n_stdev · stdev(rsi, 34)
//!   bb_lower  = midband - n_stdev · stdev(rsi, 34)
//!
//! Signals:
//!   - price > signal → uptrend bias
//!   - price < signal → downtrend bias
//!   - price > bb_upper → overbought
//!   - price < bb_lower → oversold
//!   - Bands narrowing → low volatility, breakout setup
//!
//! Pure compute. Companion to `stochastic_rsi`, `connors_rsi` (if shipped).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TdiReport {
    pub price_line: Vec<Option<f64>>,
    pub signal_line: Vec<Option<f64>>,
    pub market_base: Vec<Option<f64>>,
    pub bb_upper: Vec<Option<f64>>,
    pub bb_lower: Vec<Option<f64>>,
    pub rsi_period: usize,
    pub price_period: usize,
    pub signal_period: usize,
    pub band_period: usize,
    pub n_stdev: f64,
}

pub fn compute(
    closes: &[f64],
    rsi_period: usize,
    price_period: usize,
    signal_period: usize,
    band_period: usize,
    n_stdev: f64,
) -> TdiReport {
    let n = closes.len();
    let mut report = TdiReport {
        price_line: vec![None; n],
        signal_line: vec![None; n],
        market_base: vec![None; n],
        bb_upper: vec![None; n],
        bb_lower: vec![None; n],
        rsi_period,
        price_period,
        signal_period,
        band_period,
        n_stdev,
    };
    if rsi_period < 2
        || price_period < 2
        || signal_period < 2
        || band_period < 2
        || !n_stdev.is_finite()
        || n_stdev <= 0.0
        || n < rsi_period + band_period
    {
        return report;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let rsi = wilder_rsi(closes, rsi_period);
    report.price_line = sma_opt(&rsi, price_period);
    report.signal_line = sma_opt(&rsi, signal_period);
    report.market_base = sma_opt(&rsi, band_period);
    let std_opt = rolling_stdev_opt(&rsi, band_period);
    for (i, s_opt) in std_opt.iter().enumerate() {
        if let (Some(m), Some(s)) = (report.market_base[i], *s_opt) {
            report.bb_upper[i] = Some(m + n_stdev * s);
            report.bb_lower[i] = Some(m - n_stdev * s);
        }
    }
    report
}

fn wilder_rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period + 1 {
        return out;
    }
    let p_f = period as f64;
    let mut sum_gain = 0.0_f64;
    let mut sum_loss = 0.0_f64;
    for i in 1..=period {
        let diff = closes[i] - closes[i - 1];
        if diff > 0.0 {
            sum_gain += diff;
        } else {
            sum_loss -= diff;
        }
    }
    let mut avg_gain = sum_gain / p_f;
    let mut avg_loss = sum_loss / p_f;
    out[period] = Some(rsi_of(avg_gain, avg_loss));
    for i in (period + 1)..n {
        let diff = closes[i] - closes[i - 1];
        let gain = diff.max(0.0);
        let loss = (-diff).max(0.0);
        avg_gain = (avg_gain * (p_f - 1.0) + gain) / p_f;
        avg_loss = (avg_loss * (p_f - 1.0) + loss) / p_f;
        out[i] = Some(rsi_of(avg_gain, avg_loss));
    }
    out
}

fn rsi_of(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss <= 0.0 {
        if avg_gain <= 0.0 {
            50.0
        } else {
            100.0
        }
    } else {
        let rs = avg_gain / avg_loss;
        100.0 - 100.0 / (1.0 + rs)
    }
}

fn sma_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    for i in (period - 1)..n {
        let win = &series[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) {
            continue;
        }
        let s: f64 = win.iter().filter_map(|x| *x).sum();
        out[i] = Some(s / p_f);
    }
    out
}

fn rolling_stdev_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    for i in (period - 1)..n {
        let win = &series[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) {
            continue;
        }
        let mean: f64 = win.iter().filter_map(|x| *x).sum::<f64>() / p_f;
        let var: f64 = win
            .iter()
            .filter_map(|x| *x)
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / p_f;
        out[i] = Some(var.max(0.0).sqrt());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 80];
        let r = compute(&c, 1, 2, 7, 34, 1.6185);
        assert!(r.price_line.iter().all(|x| x.is_none()));
        let r2 = compute(&c, 14, 2, 7, 34, 0.0);
        assert!(r2.price_line.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 80];
        c[5] = f64::NAN;
        let r = compute(&c, 14, 2, 7, 34, 1.6185);
        assert!(r.price_line.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_bands_collapse() {
        let c = vec![100.0_f64; 80];
        let r = compute(&c, 14, 2, 7, 34, 1.6185);
        // Flat market → RSI = 50 → all lines = 50, bands collapse on midband.
        for v in r.price_line.iter().skip(50).flatten() {
            assert!((v - 50.0).abs() < 1e-6);
        }
        for v in r.market_base.iter().skip(50).flatten() {
            assert!((v - 50.0).abs() < 1e-6);
        }
        for v in r.bb_upper.iter().skip(50).flatten() {
            assert!((v - 50.0).abs() < 1e-6);
        }
        for v in r.bb_lower.iter().skip(50).flatten() {
            assert!((v - 50.0).abs() < 1e-6);
        }
    }

    #[test]
    fn uptrend_yields_high_price_line() {
        let c: Vec<f64> = (0..150).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 14, 2, 7, 34, 1.6185);
        assert!(r.price_line[149].unwrap() > 80.0);
    }

    #[test]
    fn downtrend_yields_low_price_line() {
        let c: Vec<f64> = (0..150).map(|i| 200.0 - i as f64 * 0.5).collect();
        let r = compute(&c, 14, 2, 7, 34, 1.6185);
        assert!(r.price_line[149].unwrap() < 20.0);
    }

    #[test]
    fn bands_widen_with_higher_stdev_factor() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..200)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
                100.0 + (r - 0.5) * 10.0
            })
            .collect();
        let r1 = compute(&c, 14, 2, 7, 34, 1.0);
        let r2 = compute(&c, 14, 2, 7, 34, 3.0);
        let last = 199;
        let w1 = r1.bb_upper[last].unwrap() - r1.bb_lower[last].unwrap();
        let w2 = r2.bb_upper[last].unwrap() - r2.bb_lower[last].unwrap();
        assert!(w2 > w1);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 80];
        let r = compute(&c, 14, 2, 7, 34, 1.6185);
        assert_eq!(r.price_line.len(), 80);
        assert_eq!(r.bb_upper.len(), 80);
    }
}
