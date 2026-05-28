//! TTM Squeeze — John Carter ("Trade the Markets", 2007).
//!
//! Detects low-volatility "coiling" periods that often precede
//! explosive moves. The squeeze is ON when Bollinger Bands lie
//! INSIDE the Keltner Channel; OFF when they expand back outside.
//!
//!   bb_upper = SMA(close, N) + bb_mult · stdev(close, N)
//!   bb_lower = SMA(close, N) - bb_mult · stdev(close, N)
//!   kc_upper = SMA(close, N) + kc_mult · ATR(N)
//!   kc_lower = SMA(close, N) - kc_mult · ATR(N)
//!
//!   squeeze_on_t = bb_upper < kc_upper AND bb_lower > kc_lower
//!
//! Companion momentum histogram (per Carter): linear regression of
//! (close - midpoint), where midpoint is the average of the
//! N-bar HH/LL midpoint and the N-bar SMA(close).
//!
//! Pure compute. Default N = 20, bb_mult = 2.0, kc_mult = 1.5.
//! Companion to `keltner_squeeze`, `bollinger_band_width`, `starc_bands`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TtmSqueezeReport {
    pub squeeze_on: Vec<Option<bool>>,
    pub momentum: Vec<Option<f64>>,
    pub bb_upper: Vec<Option<f64>>,
    pub bb_lower: Vec<Option<f64>>,
    pub kc_upper: Vec<Option<f64>>,
    pub kc_lower: Vec<Option<f64>>,
    pub period: usize,
    pub bb_mult: f64,
    pub kc_mult: f64,
}

pub fn compute(
    bars: &[Bar],
    period: usize,
    bb_mult: f64,
    kc_mult: f64,
) -> TtmSqueezeReport {
    let n = bars.len();
    let mut report = TtmSqueezeReport {
        squeeze_on: vec![None; n],
        momentum: vec![None; n],
        bb_upper: vec![None; n],
        bb_lower: vec![None; n],
        kc_upper: vec![None; n],
        kc_lower: vec![None; n],
        period,
        bb_mult,
        kc_mult,
    };
    if period < 3 || !bb_mult.is_finite() || bb_mult <= 0.0
        || !kc_mult.is_finite() || kc_mult <= 0.0
        || n < period + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let p_f = period as f64;
    // Rolling SMA + stdev of closes; rolling HH/LL.
    for i in (period - 1)..n {
        let win = &bars[i + 1 - period..=i];
        let sma: f64 = win.iter().map(|b| b.close).sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|b| (b.close - sma).powi(2)).sum::<f64>() / p_f;
        let stdev = var.max(0.0).sqrt();
        let hh = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let ll = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        report.bb_upper[i] = Some(sma + bb_mult * stdev);
        report.bb_lower[i] = Some(sma - bb_mult * stdev);
        // KC midline = sma; ATR seeded below.
        let _ = (hh, ll);
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
    let mut atr = vec![None; n];
    let seed: f64 = tr[1..=period].iter().sum::<f64>() / p_f;
    atr[period] = Some(seed);
    let mut cur = seed;
    for i in (period + 1)..n {
        cur = (cur * (p_f - 1.0) + tr[i]) / p_f;
        atr[i] = Some(cur);
    }
    for i in 0..n {
        if let (Some(bb_u), Some(bb_l), Some(a)) = (report.bb_upper[i], report.bb_lower[i], atr[i]) {
            let win = &bars[i + 1 - period..=i];
            let sma: f64 = win.iter().map(|b| b.close).sum::<f64>() / p_f;
            let kc_u = sma + kc_mult * a;
            let kc_l = sma - kc_mult * a;
            report.kc_upper[i] = Some(kc_u);
            report.kc_lower[i] = Some(kc_l);
            report.squeeze_on[i] = Some(bb_u < kc_u && bb_l > kc_l);
            // Momentum: linear regression of (close - midpoint) where
            // midpoint = average of HH/LL mid and SMA.
            let hh = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
            let ll = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
            let mid = ((hh + ll) / 2.0 + sma) / 2.0;
            let delta: Vec<f64> = win.iter().map(|b| b.close - mid).collect();
            report.momentum[i] = Some(lin_reg_endpoint(&delta));
        }
    }
    report
}

fn lin_reg_endpoint(y: &[f64]) -> f64 {
    let n = y.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean: f64 = y.iter().sum::<f64>() / n;
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    for (i, &v) in y.iter().enumerate() {
        let dx = i as f64 - x_mean;
        sxy += dx * (v - y_mean);
        sxx += dx * dx;
    }
    let slope = if sxx > 0.0 { sxy / sxx } else { 0.0 };
    let intercept = y_mean - slope * x_mean;
    intercept + slope * (n - 1.0)    // value at last bar
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 2, 2.0, 1.5);
        assert!(r.squeeze_on.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 20, 0.0, 1.5);
        assert!(r2.squeeze_on.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 50];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 20, 2.0, 1.5);
        assert!(r.squeeze_on.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_squeeze_on() {
        // Flat closes → stdev ~ 0 → BB collapsed; ATR = 2 → KC = ±3
        // (mult 1.5). BB inside KC → squeeze ON.
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 20, 2.0, 1.5);
        for v in r.squeeze_on.iter().skip(25).flatten() {
            assert!(*v, "flat market should report squeeze ON");
        }
    }

    #[test]
    fn volatile_market_yields_squeeze_off() {
        // Strong trend: ATR settles small (HL=1 each bar) but closes
        // span 80 points → BB stdev ≈ 5.77 → BB ±11.5; KC ±1.5.
        // BB outside KC → squeeze OFF.
        let bars: Vec<_> = (0..80).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 20, 2.0, 1.5);
        let any_off = r.squeeze_on.iter().skip(40).flatten().any(|x| !*x);
        assert!(any_off,
            "trending market should have some squeeze-OFF bars");
    }

    #[test]
    fn momentum_positive_in_uptrend() {
        let bars: Vec<_> = (0..80).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 20, 2.0, 1.5);
        let last = r.momentum[79].unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn momentum_negative_in_downtrend() {
        let bars: Vec<_> = (0..80).map(|i| {
            let m = 200.0 - i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 20, 2.0, 1.5);
        let last = r.momentum[79].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 20, 2.0, 1.5);
        assert_eq!(r.squeeze_on.len(), 50);
        assert_eq!(r.momentum.len(), 50);
    }
}
