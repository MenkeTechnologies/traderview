//! Volume Flow Indicator (VFI) — Markos Katsanos (TASC, 2004).
//!
//! Volume-weighted, noise-filtered cumulative flow that ignores
//! intrabar moves smaller than 0.2σ of recent log-return volatility:
//!
//!   tp_t      = (high + low + close) / 3
//!   log_ret_t = ln(tp_t / tp_{t-1})
//!   cutoff_t  = 0.2 · stdev(log_ret over period bars) · close_t
//!   vol_avg_t = SMA(volume, period)
//!   vol_norm_t = min(volume_t, vol_avg_t · 2.5)
//!
//!   flow_t = if  tp_t - tp_{t-1} >  cutoff_t :  +vol_norm_t
//!            elif tp_t - tp_{t-1} < -cutoff_t : -vol_norm_t
//!            else                              :  0
//!
//!   cumflow_t = Σ flow / vol_avg_t        (normalized cumulative)
//!   VFI_t = EMA(cumflow, smoothing) · 100
//!
//! Pure compute. Defaults: period = 130, smoothing = 3.
//! Companion to `klinger_volume_oscillator`, `volume_zone_oscillator`,
//! `chaikin_money_flow`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64, pub volume: f64 }

pub fn compute(bars: &[Bar], period: usize, smoothing: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || smoothing < 2 || n < period + smoothing { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()
        || !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0) {
        return out;
    }
    let tp: Vec<f64> = bars.iter().map(|b| (b.high + b.low + b.close) / 3.0).collect();
    let mut log_ret = vec![0.0_f64; n];
    for i in 1..n {
        if tp[i - 1] > 0.0 && tp[i] > 0.0 {
            log_ret[i] = (tp[i] / tp[i - 1]).ln();
        }
    }
    let p_f = period as f64;
    let mut cumflow = vec![None; n];
    let mut running = 0.0_f64;
    for i in period..n {
        let lr_win = &log_ret[i + 1 - period..=i];
        let mean: f64 = lr_win.iter().sum::<f64>() / p_f;
        let var: f64 = lr_win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let stdev = var.max(0.0).sqrt();
        let cutoff = 0.2 * stdev * bars[i].close;
        let vol_avg: f64 = bars[i + 1 - period..=i].iter()
            .map(|b| b.volume).sum::<f64>() / p_f;
        let vol_norm = bars[i].volume.min(vol_avg * 2.5);
        let dtp = tp[i] - tp[i - 1];
        let flow = if dtp > cutoff { vol_norm }
            else if dtp < -cutoff { -vol_norm }
            else { 0.0 };
        if vol_avg > 0.0 {
            running += flow / vol_avg;
            cumflow[i] = Some(running);
        }
    }
    let smoothed = ema_opt(&cumflow, smoothing);
    for (i, slot) in out.iter_mut().enumerate() {
        if let Some(s) = smoothed[i] {
            *slot = Some(s * 100.0);
        }
    }
    out
}

fn ema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 { return out; }
    let mut seed_end = None;
    let mut seed_sum = 0.0_f64;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => { seed_sum += x; count += 1; }
            None => { seed_sum = 0.0; count = 0; }
        }
        if count == period { seed_end = Some(i); break; }
    }
    let Some(end) = seed_end else { return out; };
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let mut cur = seed_sum / p_f;
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

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 200];
        assert!(compute(&bars, 1, 3).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..10], 130, 3).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 200];
        bars[5] = b(f64::NAN, 99.0, 100.0, 1000.0);
        assert!(compute(&bars, 130, 3).iter().all(|x| x.is_none()));
        let mut bars2 = vec![b(101.0, 99.0, 100.0, 1000.0); 200];
        bars2[5] = b(101.0, 99.0, 100.0, -100.0);
        assert!(compute(&bars2, 130, 3).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_vfi() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 200];
        let r = compute(&bars, 130, 3);
        // Zero log-returns → cumflow stays at 0 → VFI = 0.
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn strong_uptrend_yields_positive_vfi() {
        let bars: Vec<_> = (0..200).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m, 1000.0)
        }).collect();
        let r = compute(&bars, 130, 3);
        let last = r[199].unwrap();
        assert!(last > 0.0,
            "uptrend should yield positive VFI, got {last}");
    }

    #[test]
    fn strong_downtrend_yields_negative_vfi() {
        let bars: Vec<_> = (0..200).map(|i| {
            let m = 300.0 - i as f64;
            b(m + 0.5, m - 0.5, m, 1000.0)
        }).collect();
        let r = compute(&bars, 130, 3);
        let last = r[199].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 200];
        assert_eq!(compute(&bars, 130, 3).len(), 200);
    }
}
