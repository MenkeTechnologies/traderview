//! Gann High-Low Activator — Robert Krausz adaptation of W.D. Gann's
//! swing trading rules (TASC, 1998).
//!
//! Trend-following stop line that flips between a rising "support" mode
//! and a falling "resistance" mode based on close vs the prior bar's
//! SMA of highs/lows:
//!
//!   sma_h_t = SMA(high, period)
//!   sma_l_t = SMA(low,  period)
//!
//! Maintain a direction state. Initial direction set by close vs
//! sma_h/sma_l. Then:
//!   if direction == up:
//!     if close_t < sma_l_{t-1}: flip down, stop = sma_h_{t-1}
//!     else: stop = max(stop_{t-1}, sma_l_{t-1})    (ratchet up)
//!   if direction == down:
//!     if close_t > sma_h_{t-1}: flip up, stop = sma_l_{t-1}
//!     else: stop = min(stop_{t-1}, sma_h_{t-1})    (ratchet down)
//!
//! Default period = 5. Pure compute.
//!
//! Companion to `parabolic_sar`, `chande_kroll_stop`, `elder_safezone_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HlaDirection { #[default] Up, Down }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GannHlaReport {
    pub stop: Vec<Option<f64>>,
    pub direction: Vec<Option<HlaDirection>>,
    pub period: usize,
}

pub fn compute(bars: &[Bar], period: usize) -> GannHlaReport {
    let n = bars.len();
    let mut report = GannHlaReport {
        stop: vec![None; n],
        direction: vec![None; n],
        period,
    };
    if period < 2 || n < period + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let p_f = period as f64;
    let mut sum_h: f64 = bars[..period].iter().map(|b| b.high).sum();
    let mut sum_l: f64 = bars[..period].iter().map(|b| b.low).sum();
    let mut sma_h = vec![None; n];
    let mut sma_l = vec![None; n];
    sma_h[period - 1] = Some(sum_h / p_f);
    sma_l[period - 1] = Some(sum_l / p_f);
    for i in period..n {
        sum_h += bars[i].high - bars[i - period].high;
        sum_l += bars[i].low - bars[i - period].low;
        sma_h[i] = Some(sum_h / p_f);
        sma_l[i] = Some(sum_l / p_f);
    }
    let mut dir: Option<HlaDirection> = None;
    let mut stop: Option<f64> = None;
    for i in period..n {
        let prev_h = sma_h[i - 1].unwrap();
        let prev_l = sma_l[i - 1].unwrap();
        let close = bars[i].close;
        match dir {
            None => {
                if close > prev_h {
                    dir = Some(HlaDirection::Up);
                    stop = Some(prev_l);
                } else if close < prev_l {
                    dir = Some(HlaDirection::Down);
                    stop = Some(prev_h);
                }
            }
            Some(HlaDirection::Up) => {
                if close < prev_l {
                    dir = Some(HlaDirection::Down);
                    stop = Some(prev_h);
                } else {
                    stop = Some(stop.unwrap().max(prev_l));
                }
            }
            Some(HlaDirection::Down) => {
                if close > prev_h {
                    dir = Some(HlaDirection::Up);
                    stop = Some(prev_l);
                } else {
                    stop = Some(stop.unwrap().min(prev_h));
                }
            }
        }
        report.direction[i] = dir;
        report.stop[i] = stop;
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1);
        assert!(r.stop.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..3], 5);
        assert!(r2.stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 5);
        assert!(r.stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn uptrend_flips_to_up_direction() {
        // 5 quiet bars then rising closes: direction → Up, stop = sma_l.
        let mut bars = vec![b(101.0, 99.0, 100.0); 5];
        for i in 0..10 {
            let m = 105.0 + i as f64;
            bars.push(b(m + 1.0, m - 1.0, m));
        }
        let r = compute(&bars, 5);
        let last = bars.len() - 1;
        assert_eq!(r.direction[last].unwrap(), HlaDirection::Up);
    }

    #[test]
    fn downtrend_flips_to_down_direction() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 5];
        for i in 0..10 {
            let m = 95.0 - i as f64;
            bars.push(b(m + 1.0, m - 1.0, m));
        }
        let r = compute(&bars, 5);
        let last = bars.len() - 1;
        assert_eq!(r.direction[last].unwrap(), HlaDirection::Down);
    }

    #[test]
    fn stop_ratchets_up_in_uptrend() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 5];
        for i in 0..15 {
            let m = 105.0 + i as f64;
            bars.push(b(m + 1.0, m - 1.0, m));
        }
        let r = compute(&bars, 5);
        let vals: Vec<f64> = r.stop.iter().flatten().copied().collect();
        // After first up-bar, all subsequent stops should be non-decreasing
        // while direction remains Up.
        for w in vals.windows(2) {
            assert!(w[1] >= w[0] - 1e-9);
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 5);
        assert_eq!(r.stop.len(), 30);
        assert_eq!(r.direction.len(), 30);
    }
}
