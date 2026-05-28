//! Bollinger Squeeze — John Bollinger.
//!
//! Detects periods of unusually narrow Bollinger Band width as
//! precursors to volatility expansion. Distinct from `keltner_squeeze`
//! (which checks BB inside KC) and `ttm_squeeze` (Carter variant):
//! this module flags squeezes by comparing current %width against
//! its own rolling low.
//!
//!   sma_t    = SMA(close, bb_period)
//!   stdev_t  = sample stdev of close over bb_period
//!   width_t  = 2 · n_stdev · stdev_t / sma_t      (% of midline)
//!   squeeze_t = width_t <= min(width over lookback bars) · (1 + slack)
//!
//! Default: bb_period = 20, n_stdev = 2.0, lookback = 125, slack = 0.05.
//!
//! Pure compute. Companion to `bollinger_band_width`,
//! `bollinger_percent_b`, `keltner_squeeze`, `ttm_squeeze`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BollingerSqueezeReport {
    pub width_pct: Vec<Option<f64>>,
    pub squeeze_on: Vec<Option<bool>>,
    pub bb_period: usize,
    pub n_stdev: f64,
    pub lookback: usize,
    pub slack: f64,
}

pub fn compute(
    closes: &[f64],
    bb_period: usize,
    n_stdev: f64,
    lookback: usize,
    slack: f64,
) -> BollingerSqueezeReport {
    let n = closes.len();
    let mut report = BollingerSqueezeReport {
        width_pct: vec![None; n],
        squeeze_on: vec![None; n],
        bb_period,
        n_stdev,
        lookback,
        slack,
    };
    if bb_period < 2 || lookback < bb_period
        || !n_stdev.is_finite() || n_stdev <= 0.0
        || !slack.is_finite() || slack < 0.0
        || n < lookback { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    let p_f = bb_period as f64;
    for i in (bb_period - 1)..n {
        let win = &closes[i + 1 - bb_period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let std = var.max(0.0).sqrt();
        if mean.abs() > 0.0 {
            let width = 2.0 * n_stdev * std / mean.abs() * 100.0;
            report.width_pct[i] = Some(width);
        }
    }
    for i in (lookback - 1)..n {
        let win = &report.width_pct[i + 1 - lookback..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let min_w = win.iter().filter_map(|x| *x).fold(f64::INFINITY, f64::min);
        let cur = report.width_pct[i].unwrap();
        report.squeeze_on[i] = Some(cur <= min_w * (1.0 + slack));
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 200];
        let r = compute(&c, 1, 2.0, 125, 0.05);
        assert!(r.width_pct.iter().all(|x| x.is_none()));
        let r2 = compute(&c, 20, 0.0, 125, 0.05);
        assert!(r2.width_pct.iter().all(|x| x.is_none()));
        let r3 = compute(&c, 20, 2.0, 10, 0.05);    // lookback < bb_period
        assert!(r3.width_pct.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 200];
        c[5] = f64::NAN;
        let r = compute(&c, 20, 2.0, 125, 0.05);
        assert!(r.width_pct.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_is_perpetual_squeeze() {
        // Zero stdev → width = 0 → squeeze always on.
        let c = vec![100.0_f64; 200];
        let r = compute(&c, 20, 2.0, 125, 0.05);
        for v in r.squeeze_on.iter().skip(130).flatten() {
            assert!(*v);
        }
    }

    #[test]
    fn expanded_volatility_off() {
        // Quiet 130 bars, then huge volatility surge.
        let mut state: u64 = 42;
        let mut c = vec![100.0_f64; 130];
        for _ in 0..70 {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            c.push(100.0 + (r - 0.5) * 50.0);
        }
        let r = compute(&c, 20, 2.0, 125, 0.05);
        // At least one post-surge bar should be squeeze OFF.
        let any_off = r.squeeze_on.iter().skip(150).flatten().any(|x| !*x);
        assert!(any_off,
            "post-surge bars should report at least one squeeze OFF");
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 200];
        let r = compute(&c, 20, 2.0, 125, 0.05);
        assert_eq!(r.width_pct.len(), 200);
        assert_eq!(r.squeeze_on.len(), 200);
    }
}
