//! Pretty Good Oscillator (PGO) — Mark Johnson (Stocks & Commodities, 1995).
//!
//! Combines moving-average trend with ATR volatility scaling:
//!
//!   PGO_t = (close_t - SMA(close, N)) / Wilder_ATR(N)
//!
//! Centered at zero with no upper/lower bound. Per Johnson's signal
//! rules:
//!   - PGO > +3 → strong move above mean, consider exit if long
//!   - PGO < -3 → strong move below mean, consider exit if short
//!   - Crossings of ±2.5 used as trade entries (long > 2.5 momentum,
//!     short < -2.5 momentum)
//!
//! Default N = 14. Pure compute.
//!
//! Companion to `disparity_index`, `efficiency_ratio`, `chande_kroll_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    let p_f = period as f64;
    // SMA of closes.
    let mut sma = vec![None; n];
    let mut sum: f64 = bars[..period].iter().map(|b| b.close).sum();
    sma[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += bars[i].close - bars[i - period].close;
        sma[i] = Some(sum / p_f);
    }
    // Wilder ATR.
    let mut tr = vec![0.0_f64; n];
    tr[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let pc = bars[i - 1].close;
        tr[i] = (bars[i].high - bars[i].low)
            .max((bars[i].high - pc).abs())
            .max((bars[i].low - pc).abs());
    }
    let seed: f64 = tr[1..=period].iter().sum::<f64>() / p_f;
    let mut atr = vec![None; n];
    atr[period] = Some(seed);
    let mut cur = seed;
    for i in (period + 1)..n {
        cur = (cur * (p_f - 1.0) + tr[i]) / p_f;
        atr[i] = Some(cur);
    }
    for (i, slot) in out.iter_mut().enumerate() {
        if let (Some(m), Some(a)) = (sma[i], atr[i]) {
            if a > 0.0 {
                *slot = Some((bars[i].close - m) / a);
            }
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
    fn invalid_inputs_return_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_pgo() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn close_above_mean_yields_positive_pgo() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars.push(b(115.0, 99.0, 115.0));
        let r = compute(&bars, 14);
        assert!(r[30].unwrap() > 0.0);
    }

    #[test]
    fn close_below_mean_yields_negative_pgo() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars.push(b(101.0, 85.0, 85.0));
        let r = compute(&bars, 14);
        assert!(r[30].unwrap() < 0.0);
    }

    #[test]
    fn strong_uptrend_pgo_above_one() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let m = 100.0 + i as f64;
                b(m + 0.5, m - 0.5, m)
            })
            .collect();
        let r = compute(&bars, 14);
        // Steady uptrend: close = mean + (n-1)/2 above the 14-bar SMA.
        // ATR ≈ 1 so PGO ≈ (n-1)/2.
        assert!(r[49].unwrap() > 1.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert_eq!(compute(&bars, 14).len(), 30);
    }
}
