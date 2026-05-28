//! Moving Average Envelope — classical % bands around an SMA/EMA.
//!
//! Envelope bands at a fixed percentage above and below the chosen
//! moving average:
//!
//!   ma_t    = SMA(close, period)   or  EMA(close, period)
//!   upper_t = ma_t · (1 + pct/100)
//!   lower_t = ma_t · (1 - pct/100)
//!
//! Distinct from Bollinger (stdev) and Keltner/STARC (ATR) bands in
//! that the offset is a *fixed* percentage of the midline level —
//! useful for instruments with stable cyclical behavior where
//! volatility-adapting bands wash out the cycle (e.g. forex pairs,
//! commodity futures during steady regimes).
//!
//! Pure compute. Defaults: period = 20, pct = 2.5, use_ema = false.
//! Companion to `bollinger_band_width`, `keltner_squeeze`, `starc_bands`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaEnvelopeReport {
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
    pub period: usize,
    pub pct: f64,
    pub use_ema: bool,
}

pub fn compute(
    closes: &[f64],
    period: usize,
    pct: f64,
    use_ema: bool,
) -> MaEnvelopeReport {
    let n = closes.len();
    let mut report = MaEnvelopeReport {
        middle: vec![None; n],
        upper: vec![None; n],
        lower: vec![None; n],
        period,
        pct,
        use_ema,
    };
    if period < 2 || !pct.is_finite() || pct <= 0.0 || n < period { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    report.middle = if use_ema { ema(closes, period) } else { sma(closes, period) };
    let factor = pct / 100.0;
    for (i, m_opt) in report.middle.iter().enumerate() {
        if let Some(m) = m_opt {
            report.upper[i] = Some(m * (1.0 + factor));
            report.lower[i] = Some(m * (1.0 - factor));
        }
    }
    report
}

fn sma(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let mut sum: f64 = series[..period].iter().sum();
    out[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += series[i] - series[i - period];
        out[i] = Some(sum / p_f);
    }
    out
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 1, 2.5, false);
        assert!(r.middle.iter().all(|x| x.is_none()));
        let r2 = compute(&c, 20, 0.0, false);
        assert!(r2.middle.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        let r = compute(&c, 20, 2.5, false);
        assert!(r.middle.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_bands_symmetric() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20, 2.5, false);
        let last = 49;
        assert!((r.middle[last].unwrap() - 100.0).abs() < 1e-9);
        assert!((r.upper[last].unwrap() - 102.5).abs() < 1e-9);
        assert!((r.lower[last].unwrap() - 97.5).abs() < 1e-9);
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let c: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 20, 2.5, false);
        for i in 19..100 {
            assert!(r.upper[i].unwrap() > r.middle[i].unwrap());
            assert!(r.middle[i].unwrap() > r.lower[i].unwrap());
        }
    }

    #[test]
    fn ema_variant_responds_faster_to_step_change() {
        // 30 bars at 100, then jump to 200. EMA reacts faster than SMA.
        let mut c = vec![100.0_f64; 30];
        c.extend(std::iter::repeat_n(200.0_f64, 5));
        let r_sma = compute(&c, 20, 2.5, false);
        let r_ema = compute(&c, 20, 2.5, true);
        let last = c.len() - 1;
        assert!(r_ema.middle[last].unwrap() > r_sma.middle[last].unwrap(),
            "EMA midline {} should respond faster to step than SMA {}",
            r_ema.middle[last].unwrap(), r_sma.middle[last].unwrap());
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20, 2.5, false);
        assert_eq!(r.middle.len(), 50);
        assert_eq!(r.upper.len(), 50);
        assert_eq!(r.lower.len(), 50);
    }
}
