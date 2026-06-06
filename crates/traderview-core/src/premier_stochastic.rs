//! Premier Stochastic Oscillator — Lee Leibfarth.
//!
//! Smooths the classic Stochastic %K through two cascaded EMAs after
//! mapping into a bounded [-1, +1] range via the Fisher transform:
//!
//!   raw_k = (close − low_n) / (high_n − low_n) · 100
//!   normalized = 0.1 · (raw_k − 50)
//!   smoothed_k = EMA(EMA(normalized, len1), len2)
//!   PSO = (exp(2·smoothed_k) − 1) / (exp(2·smoothed_k) + 1)
//!
//! Output range [−1, +1]. Default lengths: stoch_period = 8,
//! len1 = 5, len2 = 3.
//!
//! Signals:
//!   crossings of ±0.9 = strong overbought/oversold
//!   crossings of zero = trend reversal
//!
//! Pure compute. Companion to `stochastic_rsi`, `stochastic_momentum_index`,
//! `fisher_transform`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(
    bars: &[Bar],
    stoch_period: usize,
    smoothing_1: usize,
    smoothing_2: usize,
) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if stoch_period < 2
        || smoothing_1 < 2
        || smoothing_2 < 2
        || n < stoch_period + smoothing_1 + smoothing_2
    {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    // Raw stochastic %K.
    let mut raw_k = vec![None; n];
    for (i, slot) in raw_k.iter_mut().enumerate().skip(stoch_period - 1) {
        let win = &bars[i + 1 - stoch_period..=i];
        let hh = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let ll = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let range = hh - ll;
        *slot = Some(if range > 0.0 {
            (bars[i].close - ll) / range * 100.0
        } else {
            50.0
        });
    }
    // Normalize.
    let normalized: Vec<Option<f64>> = raw_k.iter().map(|v| v.map(|k| 0.1 * (k - 50.0))).collect();
    // Two-stage EMA.
    let smoothed_1 = ema_opt(&normalized, smoothing_1);
    let smoothed_2 = ema_opt(&smoothed_1, smoothing_2);
    for i in 0..n {
        if let Some(s) = smoothed_2[i] {
            let two_s = (2.0 * s).exp();
            out[i] = Some((two_s - 1.0) / (two_s + 1.0));
        }
    }
    out
}

fn ema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n == 0 {
        return out;
    }
    let mut seed_end = None;
    let mut seed_sum = 0.0;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => {
                seed_sum += x;
                count += 1;
            }
            None => {
                seed_sum = 0.0;
                count = 0;
            }
        }
        if count == period {
            seed_end = Some(i);
            break;
        }
    }
    let Some(end) = seed_end else {
        return out;
    };
    let k = 2.0 / (period as f64 + 1.0);
    let mut cur = seed_sum / period as f64;
    out[end] = Some(cur);
    for i in (end + 1)..n {
        if let Some(v) = series[i] {
            cur = v * k + cur * (1.0 - k);
            out[i] = Some(cur);
        } else {
            out[i] = Some(cur);
        }
    }
    out
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
    fn invalid_params_return_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        assert!(compute(&bars, 1, 5, 3).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 8, 5, 3).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 50];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 8, 5, 3).iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_in_unit_signed_range() {
        let bars: Vec<_> = (0..100)
            .map(|i| {
                let m = 100.0 + (i as f64 * 0.3).sin() * 5.0;
                b(m + 1.0, m - 1.0, m)
            })
            .collect();
        let r = compute(&bars, 8, 5, 3);
        for v in r.iter().flatten() {
            assert!((-1.0..=1.0).contains(v));
        }
    }

    #[test]
    fn uptrend_yields_positive_pso() {
        // Strong uptrend with closes at highs.
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let mid = 100.0 + i as f64;
                b(mid + 0.5, mid - 0.5, mid + 0.5)
            })
            .collect();
        let r = compute(&bars, 8, 5, 3);
        let last = r[49].unwrap();
        assert!(
            last > 0.5,
            "strong uptrend should yield PSO > 0.5, got {last}"
        );
    }

    #[test]
    fn downtrend_yields_negative_pso() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let mid = 200.0 - i as f64;
                b(mid + 0.5, mid - 0.5, mid - 0.5)
            })
            .collect();
        let r = compute(&bars, 8, 5, 3);
        let last = r[49].unwrap();
        assert!(last < -0.5);
    }

    #[test]
    fn flat_market_yields_zero_pso() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 8, 5, 3);
        // Range 0 → raw_k = 50 → normalized = 0 → PSO = 0.
        for v in r.iter().skip(20).flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 8, 5, 3);
        assert_eq!(r.len(), 50);
    }
}
