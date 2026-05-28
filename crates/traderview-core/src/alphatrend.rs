//! AlphaTrend — Kivanc Ozbilgic (TradingView/TASC, 2021).
//!
//! Hybrid trailing trend line that adapts to volatility using ATR and
//! a momentum filter (RSI or MFI). Choice of momentum filter is
//! ignored here; we use a simplified Wilder RSI form as the gate.
//!
//!   atr_t   = SMA of true range over period
//!   rsi_t   = Wilder RSI of close over period
//!   up_t    = low_t  - multiplier · atr_t
//!   dn_t    = high_t + multiplier · atr_t
//!
//!   alpha_t = if rsi_t >= 50:
//!                 max(alpha_{t-1}, up_t)        (ratchet support up)
//!             else:
//!                 min(alpha_{t-1}, dn_t)        (ratchet resistance down)
//!
//! Direction at bar t = +1 if alpha_t > alpha_{t-1}, -1 if alpha_t <
//! alpha_{t-1}, else 0.
//!
//! Pure compute. Defaults: period = 14, multiplier = 1.0.
//! Companion to `parabolic_sar`, `supertrend`, `chandelier_exit`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlphaTrendReport {
    pub alpha: Vec<Option<f64>>,
    pub direction: Vec<Option<i32>>,
    pub period: usize,
    pub multiplier: f64,
}

pub fn compute(bars: &[Bar], period: usize, multiplier: f64) -> AlphaTrendReport {
    let n = bars.len();
    let mut report = AlphaTrendReport {
        alpha: vec![None; n],
        direction: vec![None; n],
        period,
        multiplier,
    };
    if period < 2 || !multiplier.is_finite() || multiplier <= 0.0
        || n < period + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let p_f = period as f64;
    // ATR via SMA of TR (per AlphaTrend's reference impl, not Wilder).
    let mut tr = vec![0.0_f64; n];
    tr[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let pc = bars[i - 1].close;
        tr[i] = (bars[i].high - bars[i].low)
            .max((bars[i].high - pc).abs())
            .max((bars[i].low - pc).abs());
    }
    let mut atr = vec![None; n];
    let mut sum: f64 = tr[..period].iter().sum();
    atr[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += tr[i] - tr[i - period];
        atr[i] = Some(sum / p_f);
    }
    // Wilder RSI of close.
    let rsi = wilder_rsi(&bars.iter().map(|b| b.close).collect::<Vec<f64>>(), period);
    let mut last: Option<f64> = None;
    for i in 0..n {
        let (Some(a), Some(r)) = (atr[i], rsi[i]) else { continue };
        let up = bars[i].low - multiplier * a;
        let dn = bars[i].high + multiplier * a;
        let next = match last {
            None => if r >= 50.0 { up } else { dn },
            Some(prev) => {
                if r >= 50.0 { up.max(prev) } else { dn.min(prev) }
            }
        };
        let dir = match last {
            Some(prev) => if next > prev { 1 } else if next < prev { -1 } else { 0 },
            None => 0,
        };
        report.alpha[i] = Some(next);
        report.direction[i] = Some(dir);
        last = Some(next);
    }
    report
}

fn wilder_rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period + 1 { return out; }
    let p_f = period as f64;
    let mut sum_g = 0.0_f64;
    let mut sum_l = 0.0_f64;
    for i in 1..=period {
        let d = closes[i] - closes[i - 1];
        if d > 0.0 { sum_g += d; } else { sum_l -= d; }
    }
    let mut avg_g = sum_g / p_f;
    let mut avg_l = sum_l / p_f;
    out[period] = Some(rsi_of(avg_g, avg_l));
    for i in (period + 1)..n {
        let d = closes[i] - closes[i - 1];
        let g = d.max(0.0);
        let l = (-d).max(0.0);
        avg_g = (avg_g * (p_f - 1.0) + g) / p_f;
        avg_l = (avg_l * (p_f - 1.0) + l) / p_f;
        out[i] = Some(rsi_of(avg_g, avg_l));
    }
    out
}

fn rsi_of(g: f64, l: f64) -> f64 {
    if l <= 0.0 { if g <= 0.0 { 50.0 } else { 100.0 } }
    else {
        let rs = g / l;
        100.0 - 100.0 / (1.0 + rs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1, 1.0);
        assert!(r.alpha.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 14, 0.0);
        assert!(r2.alpha.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 14, 1.0);
        assert!(r.alpha.iter().all(|x| x.is_none()));
    }

    #[test]
    fn uptrend_alpha_rises_monotone() {
        let bars: Vec<_> = (0..60).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 14, 1.0);
        let vals: Vec<f64> = r.alpha.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] >= w[0] - 1e-9);
        }
    }

    #[test]
    fn downtrend_alpha_falls_monotone() {
        let bars: Vec<_> = (0..60).map(|i| {
            let m = 200.0 - i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 14, 1.0);
        let vals: Vec<f64> = r.alpha.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] <= w[0] + 1e-9);
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 14, 1.0);
        assert_eq!(r.alpha.len(), 30);
        assert_eq!(r.direction.len(), 30);
    }
}
