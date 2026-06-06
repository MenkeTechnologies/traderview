//! Anti Setup — Linda Bradford Raschke ("Street Smarts").
//!
//! Momentum-pullback continuation pattern. Uses two fast stochastic
//! values and looks for a brief retracement in the opposite direction
//! of the dominant trend, then re-entry in the trend direction:
//!
//!   k_t      = fast %K(stoch_period) (range 0..100)
//!   d_t      = SMA(k, d_period)
//!
//! Bullish anti (long signal):
//!   - macd_proxy > 0 (we use SMA20 - SMA50 of closes as trend proxy)
//!   - d_t makes a fresh peak above 75 in last `lookback` bars
//!   - then pulls back: d_t drops, k_t crosses BACK above d_t while
//!     d_t > 50  (momentum re-acceleration)
//!
//! Bearish anti (short signal): mirror with d_t making fresh trough
//! below 25 then k_t crosses back below d_t while d_t < 50.
//!
//! Pure compute. Defaults: stoch_period = 14, d_period = 3, trend_short =
//! 20, trend_long = 50, lookback = 5.
//!
//! Companion to `holy_grail`, `pinball_setup`, `turtle_soup`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AntiReport {
    pub long_signal: Vec<bool>,
    pub short_signal: Vec<bool>,
    pub stoch_k: Vec<Option<f64>>,
    pub stoch_d: Vec<Option<f64>>,
}

pub fn compute(
    bars: &[Bar],
    stoch_period: usize,
    d_period: usize,
    trend_short: usize,
    trend_long: usize,
    lookback: usize,
) -> AntiReport {
    let n = bars.len();
    let mut report = AntiReport {
        long_signal: vec![false; n],
        short_signal: vec![false; n],
        stoch_k: vec![None; n],
        stoch_d: vec![None; n],
    };
    if stoch_period < 2
        || d_period < 2
        || trend_short < 2
        || trend_long < 2
        || trend_short >= trend_long
        || lookback < 2
        || n < trend_long.max(stoch_period + d_period + lookback)
    {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return report;
    }
    // Raw %K.
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
    let d_line = sma_opt(&raw_k, d_period);
    report.stoch_k = raw_k.clone();
    report.stoch_d = d_line.clone();
    // Trend proxy via SMA(short) - SMA(long) of closes.
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let sma_s = sma(&closes, trend_short);
    let sma_l = sma(&closes, trend_long);
    for i in 1..n {
        let (Some(kt), Some(dt), Some(dt_prev), Some(kt_prev), Some(ss), Some(sl)) = (
            raw_k[i],
            d_line[i],
            d_line[i - 1],
            raw_k[i - 1],
            sma_s[i],
            sma_l[i],
        ) else {
            continue;
        };
        let trend = ss - sl;
        // Need d_t to have made a peak/trough in the last `lookback` bars.
        let lo_idx = i.saturating_sub(lookback);
        let win_d = &d_line[lo_idx..=i];
        if win_d.iter().any(|x| x.is_none()) {
            continue;
        }
        let vals: Vec<f64> = win_d.iter().filter_map(|x| *x).collect();
        let win_max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let win_min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        // Bullish: trend up, d had peak ≥ 75 in window, current d > 50,
        // and k just crossed back ABOVE d.
        if trend > 0.0 && win_max >= 75.0 && dt > 50.0 && kt_prev <= dt_prev && kt > dt {
            report.long_signal[i] = true;
        }
        if trend < 0.0 && win_min <= 25.0 && dt < 50.0 && kt_prev >= dt_prev && kt < dt {
            report.short_signal[i] = true;
        }
    }
    report
}

fn sma(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    let mut sum: f64 = series[..period].iter().sum();
    out[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += series[i] - series[i - period];
        out[i] = Some(sum / p_f);
    }
    out
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
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 100];
        let r = compute(&bars, 1, 3, 20, 50, 5);
        assert!(!r.long_signal.iter().any(|x| *x));
        let r2 = compute(&bars, 14, 3, 50, 20, 5); // short > long
        assert!(!r2.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 100];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 14, 3, 20, 50, 5);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_yields_no_signals() {
        let bars = vec![b(101.0, 99.0, 100.0); 100];
        let r = compute(&bars, 14, 3, 20, 50, 5);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn stoch_k_in_zero_hundred_range() {
        let bars: Vec<_> = (0..200)
            .map(|i| {
                let m = 100.0 + (i as f64 * 0.2).sin() * 5.0;
                b(m + 1.0, m - 1.0, m)
            })
            .collect();
        let r = compute(&bars, 14, 3, 20, 50, 5);
        for v in r.stoch_k.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 100];
        let r = compute(&bars, 14, 3, 20, 50, 5);
        assert_eq!(r.long_signal.len(), 100);
        assert_eq!(r.short_signal.len(), 100);
    }
}
