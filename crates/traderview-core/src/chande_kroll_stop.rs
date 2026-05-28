//! Chande-Kroll Stop — Tushar Chande & Stanley Kroll (1995).
//!
//! Two-pass volatility trailing stop. First pass computes long/short
//! ATR-based stop candidates over `p` bars; second pass smooths each
//! by taking its `q`-bar extreme:
//!
//!   atr_t      = Wilder ATR over p bars
//!   raw_long_t  = highest_high(p) - x · atr_t
//!   raw_short_t = lowest_low(p)  + x · atr_t
//!   long_stop_t  = highest(raw_long, q)
//!   short_stop_t = lowest (raw_short, q)
//!
//! When long_stop crosses BELOW short_stop, long bias; when long_stop
//! crosses ABOVE short_stop, short bias. Default parameters
//! p = 10, x = 1.0, q = 9.
//!
//! Pure compute. Companion to `elder_safezone_stop`, `parabolic_sar`,
//! `volatility_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChandeKrollReport {
    pub long_stop: Vec<Option<f64>>,
    pub short_stop: Vec<Option<f64>>,
    pub p: usize,
    pub x: f64,
    pub q: usize,
}

pub fn compute(bars: &[Bar], p: usize, x: f64, q: usize) -> ChandeKrollReport {
    let n = bars.len();
    let mut report = ChandeKrollReport {
        long_stop: vec![None; n],
        short_stop: vec![None; n],
        p, x, q,
    };
    if p < 2 || q < 2 || !x.is_finite() || x <= 0.0 || n < p + q { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    // True range and Wilder ATR.
    let mut tr = vec![0.0_f64; n];
    tr[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let pc = bars[i - 1].close;
        tr[i] = (bars[i].high - bars[i].low)
            .max((bars[i].high - pc).abs())
            .max((bars[i].low - pc).abs());
    }
    let p_f = p as f64;
    let mut atr = vec![None; n];
    let seed: f64 = tr[1..=p].iter().sum::<f64>() / p_f;
    atr[p] = Some(seed);
    let mut cur = seed;
    for i in (p + 1)..n {
        cur = (cur * (p_f - 1.0) + tr[i]) / p_f;
        atr[i] = Some(cur);
    }
    // Raw stops over highest_high(p) / lowest_low(p).
    let mut raw_long = vec![None; n];
    let mut raw_short = vec![None; n];
    for i in (p - 1)..n {
        let win = &bars[i + 1 - p..=i];
        let hh = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let ll = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        if let Some(a) = atr[i] {
            raw_long[i] = Some(hh - x * a);
            raw_short[i] = Some(ll + x * a);
        }
    }
    // Smoothed stops over q-bar extremes of the raws.
    for i in (p + q - 1)..n {
        let win_l = &raw_long[i + 1 - q..=i];
        let win_s = &raw_short[i + 1 - q..=i];
        if win_l.iter().all(|x| x.is_some()) {
            let max_l = win_l.iter().filter_map(|x| *x).fold(f64::NEG_INFINITY, f64::max);
            report.long_stop[i] = Some(max_l);
        }
        if win_s.iter().all(|x| x.is_some()) {
            let min_s = win_s.iter().filter_map(|x| *x).fold(f64::INFINITY, f64::min);
            report.short_stop[i] = Some(min_s);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1, 1.0, 9);
        assert!(r.long_stop.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 10, 0.0, 9);
        assert!(r2.long_stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 10, 1.0, 9);
        assert!(r.long_stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn long_stop_always_below_short_stop_on_flat_market() {
        // Flat market: HH=101, LL=99, ATR settles at 2.0.
        // raw_long = 101 - 2 = 99; raw_short = 99 + 2 = 101.
        // long_stop < short_stop.
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 10, 1.0, 9);
        let last = 49;
        let l = r.long_stop[last].unwrap();
        let s = r.short_stop[last].unwrap();
        assert!(l < s, "long stop {l} should be < short stop {s}");
    }

    #[test]
    fn long_stop_rises_in_uptrend() {
        let bars: Vec<_> = (0..50).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 10, 1.0, 9);
        // Confirm monotone non-decreasing where both defined.
        let vals: Vec<f64> = r.long_stop.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] >= w[0] - 1e-9);
        }
    }

    #[test]
    fn short_stop_falls_in_downtrend() {
        let bars: Vec<_> = (0..50).map(|i| {
            let m = 200.0 - i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 10, 1.0, 9);
        let vals: Vec<f64> = r.short_stop.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] <= w[0] + 1e-9);
        }
    }

    #[test]
    fn higher_x_widens_stop_distance() {
        let bars: Vec<_> = (0..50).map(|i| {
            let m = 100.0 + (i as f64 * 0.3).sin() * 5.0;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r1 = compute(&bars, 10, 1.0, 9);
        let r2 = compute(&bars, 10, 3.0, 9);
        // Larger x → stops further from price.
        // long_stop = HH - x·ATR → larger x → smaller long_stop.
        let last = 49;
        assert!(r2.long_stop[last].unwrap() < r1.long_stop[last].unwrap());
        // short_stop = LL + x·ATR → larger x → larger short_stop.
        assert!(r2.short_stop[last].unwrap() > r1.short_stop[last].unwrap());
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 10, 1.0, 9);
        assert_eq!(r.long_stop.len(), 50);
        assert_eq!(r.short_stop.len(), 50);
    }
}
