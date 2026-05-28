//! Stochastic Momentum Index (SMI) — William Blau (1993).
//!
//! Refinement of Stochastic that centers the oscillator around zero by
//! using the distance from midrange rather than from the period low:
//!
//!   midrange  = (high_n + low_n) / 2                  (n-bar HH/LL)
//!   distance  = close − midrange                       (signed)
//!   range     = high_n − low_n                         (full range)
//!   smoothed_d = EMA(EMA(distance, smooth), smooth)
//!   smoothed_r = EMA(EMA(range, smooth), smooth)
//!   SMI       = 100 · smoothed_d / (smoothed_r / 2)
//!
//! Signal line = EMA(SMI, signal). Crosses + zero-line crosses are
//! the standard trade triggers. Smooths Stochastic's noise substantially
//! while keeping its overbought/oversold (typically ±40) interpretation.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmiReport {
    pub smi: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
}

pub fn compute(
    highs: &[f64], lows: &[f64], closes: &[f64],
    period: usize, smooth: usize, signal: usize,
) -> SmiReport {
    let n = closes.len();
    let mut report = SmiReport {
        smi: vec![None; n],
        signal: vec![None; n],
    };
    if highs.len() != n || lows.len() != n || period == 0 || smooth == 0 || signal == 0
        || n < period.saturating_add(2 * smooth)
    {
        return report;
    }
    let mut distance = vec![None::<f64>; n];
    let mut range = vec![None::<f64>; n];
    for i in (period - 1)..n {
        let lo = i + 1 - period;
        let win_high = &highs[lo..=i];
        let win_low = &lows[lo..=i];
        if win_high.iter().any(|x| !x.is_finite()) || win_low.iter().any(|x| !x.is_finite())
            || !closes[i].is_finite()
        {
            continue;
        }
        let hh = win_high.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ll = win_low.iter().copied().fold(f64::INFINITY, f64::min);
        let mid = (hh + ll) / 2.0;
        let r = hh - ll;
        distance[i] = Some(closes[i] - mid);
        range[i] = Some(r);
    }
    let smoothed_d_1 = ema_options(&distance, smooth);
    let smoothed_d = ema_options(&smoothed_d_1, smooth);
    let smoothed_r_1 = ema_options(&range, smooth);
    let smoothed_r = ema_options(&smoothed_r_1, smooth);
    for i in 0..n {
        if let (Some(d), Some(r)) = (smoothed_d[i], smoothed_r[i]) {
            if r > 0.0 {
                let smi_val = 100.0 * d / (r / 2.0);
                if smi_val.is_finite() {
                    report.smi[i] = Some(smi_val);
                }
            } else if r == 0.0 {
                report.smi[i] = Some(0.0);
            }
        }
    }
    let signal_line = ema_options(&report.smi, signal);
    report.signal = signal_line;
    report
}

fn ema_options(values: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n == 0 { return out; }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut prev: Option<f64> = None;
    for (i, v) in values.iter().enumerate() {
        match (v, prev) {
            (Some(val), None) if val.is_finite() => { prev = Some(*val); out[i] = prev; }
            (Some(val), Some(p)) if val.is_finite() => {
                let new = alpha * val + (1.0 - alpha) * p;
                if new.is_finite() {
                    prev = Some(new);
                    out[i] = prev;
                }
            }
            _ => {
                if let Some(p) = prev { out[i] = Some(p); }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], &[], &[], 14, 3, 3);
        assert!(r.smi.is_empty());
    }

    #[test]
    fn dim_mismatch_returns_default() {
        let h = vec![100.0; 30];
        let l = vec![99.0; 30];
        let c = vec![99.5; 15];
        let r = compute(&h, &l, &c, 14, 3, 3);
        assert!(r.smi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_zero_returns_default() {
        let h = vec![100.0; 30];
        let l = vec![99.0; 30];
        let c = vec![99.5; 30];
        let r = compute(&h, &l, &c, 0, 3, 3);
        assert!(r.smi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_zero_smi() {
        // Constant H/L/C → range = 0 → SMI = 0 (or None depending on guard).
        let h = vec![100.0; 50];
        let l = vec![99.0; 50];
        let c = vec![99.5; 50];
        let r = compute(&h, &l, &c, 14, 3, 3);
        for v in r.smi.iter().flatten() {
            // distance = 0 (close exactly at midpoint) → SMI = 0.
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn close_at_period_high_yields_positive_smi() {
        // 14 bars rising. close == high → distance > 0 → SMI > 0.
        let h: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let l: Vec<f64> = h.iter().map(|x| x - 0.5).collect();
        let c: Vec<f64> = h.clone();
        let r = compute(&h, &l, &c, 14, 3, 3);
        let last = r.smi[49].unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn close_at_period_low_yields_negative_smi() {
        let h: Vec<f64> = (0..50).map(|i| 200.0 - i as f64).collect();
        let l: Vec<f64> = h.iter().map(|x| x - 0.5).collect();
        let c: Vec<f64> = l.clone();
        let r = compute(&h, &l, &c, 14, 3, 3);
        let last = r.smi[49].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn smi_bounded_in_minus_100_plus_100() {
        // SMI lies in [−100, +100] when each bar's close lies inside its
        // own [low, high]. With well-formed OHLC the bound holds.
        let h: Vec<f64> = (0..200).map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0).collect();
        let l: Vec<f64> = h.iter().map(|x| x - 1.0).collect();
        let c: Vec<f64> = h.iter().zip(l.iter()).enumerate()
            .map(|(i, (hi, lo))| {
                // Close oscillates inside [low, high] via clamp.
                let raw = hi + (i as f64 * 0.13).cos() * 0.3;
                raw.clamp(*lo, *hi)
            })
            .collect();
        let r = compute(&h, &l, &c, 14, 3, 3);
        for v in r.smi.iter().flatten() {
            assert!((-100.0..=100.0).contains(v),
                "SMI should stay in [-100, 100] for well-formed OHLC, got {v}");
        }
    }

    #[test]
    fn signal_line_lags_smi() {
        let h: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let l: Vec<f64> = h.iter().map(|x| x - 0.5).collect();
        let c: Vec<f64> = h.clone();
        let r = compute(&h, &l, &c, 14, 3, 3);
        assert_eq!(r.smi.len(), r.signal.len());
        // Signal is EMA of SMI → should never lead it.
        let smi_last = r.smi[49].unwrap();
        let sig_last = r.signal[49].unwrap();
        assert!(sig_last <= smi_last);    // both rising in this test → signal lags
    }

    #[test]
    fn nan_inputs_handled() {
        let mut h = vec![100.0; 50];
        let mut l = vec![99.0; 50];
        let c = vec![99.5; 50];
        h[25] = f64::NAN;
        l[25] = f64::NAN;
        let r = compute(&h, &l, &c, 14, 3, 3);
        assert_eq!(r.smi.len(), 50);
    }
}
