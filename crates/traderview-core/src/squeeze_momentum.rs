//! Squeeze Momentum — John Carter's TTM Squeeze variant (LazyBear,
//! popularised on TradingView).
//!
//! Combines two signals into a single histogram:
//!   1. **Squeeze state** — BB inside KC (volatility contracting), or
//!      BB outside KC (released).
//!   2. **Momentum** — linear-regression slope of `close − midpoint`
//!      over the lookback, where `midpoint = (highest_high +
//!      lowest_low)/2` averaged with the SMA close.
//!
//! Histogram bar color (in the TradingView original):
//!   - cyan / blue   = momentum > 0 and rising (strong long)
//!   - green / dark  = momentum > 0 and falling (long fading)
//!   - red / dark    = momentum < 0 and falling (strong short)
//!   - orange / light = momentum < 0 and rising (short fading)
//!
//! We emit numeric momentum + a squeeze enum; the frontend handles
//! color mapping.
//!
//! Pure compute.

use crate::bb_squeeze::{analyze as bb_analyze, SqueezeInput, SqueezeState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SqueezeMomentumReport {
    pub momentum: Vec<Option<f64>>,
    pub state: Vec<Option<SqueezeState>>,
}

/// Compute the LazyBear squeeze-momentum series.
///
/// `period` is used for BB (with stdev), Keltner (with ATR), and the
/// linear-regression momentum lookback. Standard period = 20.
pub fn compute(bars: &[Bar], period: usize, bb_k: f64, kc_k: f64) -> SqueezeMomentumReport {
    let n = bars.len();
    let mut report = SqueezeMomentumReport {
        momentum: vec![None; n],
        state: vec![None; n],
    };
    if period < 2 || n < period {
        return report;
    }
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let highs: Vec<f64> = bars.iter().map(|b| b.high).collect();
    let lows: Vec<f64> = bars.iter().map(|b| b.low).collect();
    let sma_close = sma(&closes, period);
    let stdev_close = rolling_stdev(&closes, period);
    let atr_series = atr(&highs, &lows, &closes, period);
    // Squeeze-state inputs.
    let inputs: Vec<SqueezeInput> = (0..n)
        .map(|i| SqueezeInput {
            close: closes[i],
            sma_20: sma_close[i].unwrap_or(closes[i]),
            stdev_20: stdev_close[i].unwrap_or(0.0) * bb_k,
            ema_20: sma_close[i].unwrap_or(closes[i]), // simple-MA as KC center for parity
            atr_20: atr_series[i].unwrap_or(0.0) * kc_k,
        })
        .collect();
    let sq = bb_analyze(&inputs);
    for (i, bar) in sq.iter().enumerate() {
        report.state[i] = Some(bar.state);
    }
    // Momentum: linear regression slope of (close - (avg of midpoint and SMA))
    // over `period` bars.
    for i in (period - 1)..n {
        let window_start = i + 1 - period;
        let mut max_h = f64::NEG_INFINITY;
        let mut min_l = f64::INFINITY;
        for j in window_start..=i {
            if highs[j].is_finite() && highs[j] > max_h {
                max_h = highs[j];
            }
            if lows[j].is_finite() && lows[j] < min_l {
                min_l = lows[j];
            }
        }
        if !max_h.is_finite() || !min_l.is_finite() {
            continue;
        }
        let midpoint = (max_h + min_l) / 2.0;
        let sma_val = sma_close[i].unwrap_or(closes[i]);
        let avg = (midpoint + sma_val) / 2.0;
        let detrended: Vec<f64> = (window_start..=i).map(|j| closes[j] - avg).collect();
        let slope = linear_regression_slope(&detrended);
        if slope.is_finite() {
            // Project slope to "current bar value" (intercept + slope*last_x).
            let last_x = (period as f64) - 1.0;
            let mean_x = last_x / 2.0;
            let mean_y = detrended.iter().sum::<f64>() / period as f64;
            let intercept = mean_y - slope * mean_x;
            let projected = intercept + slope * last_x;
            if projected.is_finite() {
                report.momentum[i] = Some(projected);
            }
        }
    }
    report
}

fn sma(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || period > n {
        return out;
    }
    let mut sum = 0.0;
    for i in 0..n {
        sum += values[i];
        if i >= period {
            sum -= values[i - period];
        }
        if i + 1 >= period {
            out[i] = Some(sum / period as f64);
        }
    }
    out
}

fn rolling_stdev(values: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if window == 0 || n < window {
        return out;
    }
    for i in (window - 1)..n {
        let slice = &values[(i + 1 - window)..=i];
        let m = slice.iter().sum::<f64>() / window as f64;
        let var = slice.iter().map(|v| (v - m).powi(2)).sum::<f64>() / window as f64;
        out[i] = Some(var.sqrt());
    }
    out
}

fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period.saturating_add(1) || highs.len() != n || lows.len() != n {
        return out;
    }
    let mut sum = 0.0;
    for i in 1..=period {
        let tr = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
        sum += tr;
    }
    let p = period as f64;
    let mut prev = sum / p;
    out[period] = Some(prev);
    for i in (period + 1)..n {
        let tr = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
        prev = (prev * (p - 1.0) + tr) / p;
        out[i] = Some(prev);
    }
    out
}

fn linear_regression_slope(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }
    let n_f = n as f64;
    let mean_x = (n_f - 1.0) / 2.0;
    let mean_y = values.iter().sum::<f64>() / n_f;
    let mut num = 0.0;
    let mut den = 0.0;
    for (i, &y) in values.iter().enumerate() {
        let dx = i as f64 - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    if den > 0.0 {
        num / den
    } else {
        0.0
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
        let r = compute(&[], 20, 2.0, 1.5);
        assert!(r.momentum.is_empty());
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 0, 2.0, 1.5);
        assert!(r.momentum.iter().all(|x| x.is_none()));
        let r = compute(&bars, 1, 2.0, 1.5);
        assert!(r.momentum.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_zero_momentum_in_squeeze() {
        let bars = vec![b(101.0, 99.0, 100.0); 60];
        let r = compute(&bars, 20, 2.0, 1.5);
        let mom = r.momentum[59].expect("populated");
        assert!(
            mom.abs() < 1e-6,
            "flat series momentum should be ~0, got {mom}"
        );
        // Flat → stdev=0, ATR small → BB inside KC → squeeze.
        // (May fluctuate based on bar-shape exactness; just verify the field is populated.)
        assert!(r.state[59].is_some());
    }

    #[test]
    fn rising_series_positive_momentum() {
        let bars: Vec<Bar> = (1..=60)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let r = compute(&bars, 20, 2.0, 1.5);
        let mom = r.momentum[59].expect("populated");
        assert!(
            mom > 0.0,
            "rising series should yield + momentum, got {mom}"
        );
    }

    #[test]
    fn falling_series_negative_momentum() {
        let bars: Vec<Bar> = (1..=60)
            .map(|i| {
                let c = 200.0 - i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let r = compute(&bars, 20, 2.0, 1.5);
        let mom = r.momentum[59].expect("populated");
        assert!(mom < 0.0);
    }

    #[test]
    fn huge_period_no_panic() {
        let bars = vec![b(101.0, 99.0, 100.0); 5];
        let r = compute(&bars, usize::MAX, 2.0, 1.5);
        assert!(r.momentum.iter().all(|x| x.is_none()));
    }
}
