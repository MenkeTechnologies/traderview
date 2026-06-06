//! QQE — Quantitative Qualitative Estimation (Igor Livshin).
//!
//! Wilder-smoothed RSI + a dynamic trailing-band (ATR-style) layer:
//!   1. rsi      = standard 14-period RSI(close).
//!   2. rsi_ma   = Wilder-smoothed RSI(rsi, smooth_factor)  (default 5).
//!   3. abs_diff = |rsi_ma_t − rsi_ma_{t−1}|
//!   4. d_ema    = Wilder-smoothed double-EMA(abs_diff, smooth_factor)
//!      (smoothed twice in Livshin's original).
//!   5. trail    = d_ema · qqe_factor      (default 4.236)
//!   6. fast_atr = "follows" rsi_ma at distance ±trail:
//!      going up   → fast_atr_t = max(rsi_ma_t − trail, fast_atr_{t−1})
//!      going down → fast_atr_t = min(rsi_ma_t + trail, fast_atr_{t−1})
//!
//! Signals: rsi_ma cross over fast_atr = buy; under = sell.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QqeReport {
    pub rsi_ma: Vec<Option<f64>>,
    pub fast_atr: Vec<Option<f64>>,
}

pub fn compute(
    closes: &[f64],
    rsi_period: usize,
    smooth_factor: usize,
    qqe_factor: f64,
) -> QqeReport {
    let n = closes.len();
    let mut report = QqeReport {
        rsi_ma: vec![None; n],
        fast_atr: vec![None; n],
    };
    if rsi_period == 0 || smooth_factor == 0 || !qqe_factor.is_finite() || qqe_factor <= 0.0 {
        return report;
    }
    let rsi_series = rsi(closes, rsi_period);
    // Wilder smoothing of RSI itself.
    let rsi_ma = wilder_optional(&rsi_series, smooth_factor);
    // Trailing band layer: take Wilder-smoothed |Δ rsi_ma|, then smooth again.
    let mut abs_diff: Vec<Option<f64>> = vec![None; n];
    for i in 1..n {
        if let (Some(a), Some(b)) = (rsi_ma[i], rsi_ma[i - 1]) {
            abs_diff[i] = Some((a - b).abs());
        }
    }
    let d1 = wilder_optional(&abs_diff, smooth_factor);
    let d2 = wilder_optional(&d1, smooth_factor);
    // Trailing fast_atr follows rsi_ma.
    let mut fast_atr: Vec<Option<f64>> = vec![None; n];
    let mut prev: Option<f64> = None;
    for i in 0..n {
        if let (Some(rm), Some(dev)) = (rsi_ma[i], d2[i]) {
            let trail = dev * qqe_factor;
            let candidate_long = rm - trail;
            let candidate_short = rm + trail;
            let new = match prev {
                None => candidate_long, // seed in the long band
                Some(p) => {
                    if rm > p {
                        // up move — keep trailing band beneath
                        candidate_long.max(p)
                    } else {
                        candidate_short.min(p)
                    }
                }
            };
            if new.is_finite() {
                fast_atr[i] = Some(new);
                prev = Some(new);
            }
        }
    }
    report.rsi_ma = rsi_ma;
    report.fast_atr = fast_atr;
    report
}

fn rsi(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n <= period {
        return out;
    }
    let mut gain = 0.0;
    let mut loss = 0.0;
    for i in 1..=period {
        let d = values[i] - values[i - 1];
        if d >= 0.0 {
            gain += d;
        } else {
            loss -= d;
        }
    }
    gain /= period as f64;
    loss /= period as f64;
    out[period] = Some(rsi_from(gain, loss));
    for i in (period + 1)..n {
        let d = values[i] - values[i - 1];
        let (g, l) = if d >= 0.0 { (d, 0.0) } else { (0.0, -d) };
        gain = (gain * (period as f64 - 1.0) + g) / period as f64;
        loss = (loss * (period as f64 - 1.0) + l) / period as f64;
        out[i] = Some(rsi_from(gain, loss));
    }
    out
}

fn rsi_from(gain: f64, loss: f64) -> f64 {
    if loss == 0.0 {
        return 100.0;
    }
    let rs = gain / loss;
    100.0 - 100.0 / (1.0 + rs)
}

/// Wilder-style smoothed average over an Option-aware series.
fn wilder_optional(values: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    // Find first index with `period` consecutive Somes.
    let mut start: Option<usize> = None;
    let mut run = 0;
    for (i, v) in values.iter().enumerate() {
        if v.is_some() {
            run += 1;
            if run >= period {
                start = Some(i);
                break;
            }
        } else {
            run = 0;
        }
    }
    let Some(s) = start else { return out };
    let seed: f64 = values[s + 1 - period..=s]
        .iter()
        .map(|x| x.unwrap())
        .sum::<f64>()
        / period as f64;
    out[s] = Some(seed);
    let mut prev = seed;
    for i in (s + 1)..n {
        if let Some(v) = values[i] {
            prev = (prev * (period as f64 - 1.0) + v) / period as f64;
            out[i] = Some(prev);
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], 14, 5, 4.236);
        assert!(r.rsi_ma.is_empty());
    }

    #[test]
    fn invalid_factor_returns_all_none() {
        let v = vec![100.0; 100];
        for f in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let r = compute(&v, 14, 5, f);
            assert!(r.rsi_ma.iter().all(|x| x.is_none()), "factor={f}");
        }
    }

    #[test]
    fn zero_period_returns_all_none() {
        let v = vec![100.0; 100];
        for (rp, sf) in [(0, 5), (14, 0)] {
            let r = compute(&v, rp, sf, 4.236);
            assert!(r.rsi_ma.iter().all(|x| x.is_none()), "({rp},{sf})");
        }
    }

    #[test]
    fn rising_series_rsi_ma_high_and_fast_atr_below() {
        let v: Vec<f64> = (1..=120).map(|i| 100.0 + i as f64).collect();
        let r = compute(&v, 14, 5, 4.236);
        let rm = r.rsi_ma.last().copied().flatten().expect("populated");
        let fa = r.fast_atr.last().copied().flatten().expect("populated");
        assert!(rm > 50.0, "rising series RSI-MA should be high, got {rm}");
        // In an uptrend, fast_atr trails BELOW rsi_ma (long band).
        assert!(
            fa <= rm,
            "fast_atr={fa} should trail at-or-below rsi_ma={rm}"
        );
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 5];
        let r = compute(&v, usize::MAX, 5, 4.236);
        assert!(r.rsi_ma.iter().all(|x| x.is_none()));
    }
}
