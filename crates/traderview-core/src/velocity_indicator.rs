//! Velocity Indicator — momentum normalized by volatility.
//!
//! Rate-of-change scaled by Wilder ATR so the reading is comparable
//! across instruments with different price levels:
//!
//!   roc_t      = (close_t - close_{t-period}) / close_{t-period}
//!   atr_pct_t  = Wilder ATR(period) / close_t
//!   velocity_t = roc_t / atr_pct_t            (dimensionless)
//!
//! Interpretation:
//!   |velocity| > 5  → very strong directional move
//!   |velocity| > 2  → strong move
//!   |velocity| < 1  → trend stalling / consolidating
//!
//! Pure compute. Default period = 14. Companion to `efficiency_ratio`,
//! `chande_kroll_stop`, `pretty_good_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    let p_f = period as f64;
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
    for (i, slot) in out.iter_mut().enumerate().skip(period) {
        if i < period { continue; }
        let prev_close = bars[i - period].close;
        if prev_close == 0.0 || bars[i].close == 0.0 { continue; }
        let Some(a) = atr[i] else { continue };
        let atr_pct = a / bars[i].close;
        if atr_pct <= 0.0 { continue; }
        let roc = (bars[i].close - prev_close) / prev_close;
        *slot = Some(roc / atr_pct);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 50];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_velocity() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn rising_close_yields_positive_velocity() {
        let bars: Vec<_> = (0..50).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 14);
        let last = r[49].unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn falling_close_yields_negative_velocity() {
        let bars: Vec<_> = (0..50).map(|i| {
            let m = 200.0 - i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 14);
        let last = r[49].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        assert_eq!(compute(&bars, 14).len(), 50);
    }
}
