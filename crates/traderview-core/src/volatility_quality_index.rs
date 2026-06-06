//! Volatility Quality Index (VQI) — Thomas Stridsman ("Trading Systems
//! That Work", 2000).
//!
//! Combines per-bar directional bias with range expansion:
//!
//!   tr_t = max(high_t - low_t, |high_t - close_{t-1}|, |low_t - close_{t-1}|)
//!   range_t = high_t - low_t
//!   raw_t = ((close_t - close_{t-1}) / tr_t + (close_t - open_t) / range_t)
//!           · 0.5 · |close_t - close_{t-1}|
//!   VQI_t = sum(raw_1..raw_t)
//!
//! `raw` is sign-bearing (positive if both up-direction terms align,
//! negative if both down). The cumulative sum exposes regime persistence:
//! steady rising VQI → real trend; flat VQI in trending price = no
//! conviction (likely to reverse).
//!
//! Normalized form:
//!   VQI_norm_t = VQI_t / SMA(close, n) · 100
//!
//! Pure compute. Companion to `elder_thermometer`, `vsa`, `chande_kroll_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VqiReport {
    pub vqi: Vec<Option<f64>>,
    pub vqi_normalized: Vec<Option<f64>>,
    pub normalization_period: usize,
}

pub fn compute(bars: &[Bar], normalization_period: usize) -> VqiReport {
    let n = bars.len();
    let mut report = VqiReport {
        vqi: vec![None; n],
        vqi_normalized: vec![None; n],
        normalization_period,
    };
    if normalization_period < 2 || n < normalization_period + 1 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    let mut cum = 0.0_f64;
    report.vqi[0] = Some(cum);
    for i in 1..n {
        let prev_close = bars[i - 1].close;
        let cur = bars[i];
        let tr = (cur.high - cur.low)
            .max((cur.high - prev_close).abs())
            .max((cur.low - prev_close).abs());
        let range = cur.high - cur.low;
        if tr > 0.0 && range > 0.0 {
            let term1 = (cur.close - prev_close) / tr;
            let term2 = (cur.close - cur.open) / range;
            let raw = ((term1 + term2) * 0.5).abs() * (cur.close - prev_close);
            cum += raw;
        }
        report.vqi[i] = Some(cum);
    }
    // Normalized: VQI / SMA(close, N) · 100.
    let p_f = normalization_period as f64;
    let mut sum: f64 = bars[..normalization_period].iter().map(|b| b.close).sum();
    let mut sma = vec![None; n];
    sma[normalization_period - 1] = Some(sum / p_f);
    for i in normalization_period..n {
        sum += bars[i].close - bars[i - normalization_period].close;
        sma[i] = Some(sum / p_f);
    }
    for (i, v_opt) in report.vqi.iter().enumerate() {
        if let (Some(v), Some(m)) = (v_opt, sma[i]) {
            if m != 0.0 {
                report.vqi_normalized[i] = Some(v / m * 100.0);
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 1);
        assert!(r.vqi.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..5], 14);
        assert!(r2.vqi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        bars[5] = bar(f64::NAN, 101.0, 99.0, 100.5);
        let r = compute(&bars, 14);
        assert!(r.vqi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_vqi() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 14);
        for v in r.vqi.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn rising_closes_yield_positive_vqi() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                let m = 100.0 + i as f64;
                bar(m - 0.4, m + 0.5, m - 0.5, m + 0.3)
            })
            .collect();
        let r = compute(&bars, 14);
        let last = r.vqi[29].unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn falling_closes_yield_negative_vqi() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                let m = 200.0 - i as f64;
                bar(m + 0.4, m + 0.5, m - 0.5, m - 0.3)
            })
            .collect();
        let r = compute(&bars, 14);
        let last = r.vqi[29].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn normalized_in_same_sign_as_vqi() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                let m = 100.0 + i as f64;
                bar(m - 0.4, m + 0.5, m - 0.5, m + 0.3)
            })
            .collect();
        let r = compute(&bars, 14);
        let v = r.vqi[29].unwrap();
        let vn = r.vqi_normalized[29].unwrap();
        assert!(v.signum() == vn.signum());
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 14);
        assert_eq!(r.vqi.len(), 30);
        assert_eq!(r.vqi_normalized.len(), 30);
    }
}
