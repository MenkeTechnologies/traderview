//! Donchian Oscillator — close position within the Donchian channel.
//!
//!   hh_t  = max(high over period bars)
//!   ll_t  = min(low  over period bars)
//!   do_t  = (close_t - midline) / (hh_t - ll_t) · 100
//!     where midline = (hh + ll) / 2
//!
//! Range typically [-50, +50] (close at channel high → +50; at channel
//! low → -50; at midline → 0). Crossings of zero confirm Donchian
//! breakouts in either direction.
//!
//! Pure compute. Default period = 20.
//! Companion to `donchian_channels`, `bollinger_percent_b`,
//! `range_filter`, `breakout_detector`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &bars[i + 1 - period..=i];
        let hh = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let ll = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let range = hh - ll;
        if range > 0.0 {
            let mid = (hh + ll) / 2.0;
            *slot = Some((bars[i].close - mid) / range * 100.0);
        } else {
            *slot = Some(0.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 20);
        // hh=101, ll=99, range=2, mid=100, close=100 → DO = 0.
        for v in r.iter().skip(19).flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn close_at_channel_high_yields_fifty() {
        // 19 quiet bars + 1 spike to high.
        let mut bars = vec![b(101.0, 99.0, 100.0); 19];
        bars.push(b(110.0, 99.0, 110.0));
        let r = compute(&bars, 20);
        let v = r[19].unwrap();
        // hh=110, ll=99, mid=104.5, close=110 → (110-104.5)/11 · 100 = 50.
        assert!((v - 50.0).abs() < 1e-9);
    }

    #[test]
    fn close_at_channel_low_yields_minus_fifty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 19];
        bars.push(b(101.0, 90.0, 90.0));
        let r = compute(&bars, 20);
        let v = r[19].unwrap();
        assert!((v + 50.0).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert_eq!(compute(&bars, 20).len(), 30);
    }
}
