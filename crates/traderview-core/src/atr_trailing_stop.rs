//! ATR Trailing Stop — simple multi-of-ATR trailing stop that ratchets
//! in the favorable direction.
//!
//! Distinct from `chandelier_exit` (which trails from rolling HH/LL
//! with N×ATR offset) and `elder_safezone_stop` (which uses
//! penetration averages). This module just applies `multiplier · ATR`
//! to the current close, then ratchets:
//!
//!   atr_t       = Wilder ATR(period)
//!   raw_long_t  = close_t - multiplier · atr_t
//!   raw_short_t = close_t + multiplier · atr_t
//!   long_stop_t  = max(raw_long_t, long_stop_{t-1})    (ratchet up)
//!   short_stop_t = min(raw_short_t, short_stop_{t-1})  (ratchet down)
//!
//! Pure compute. Defaults: period = 14, multiplier = 3.0.
//! Companion to `chandelier_exit`, `elder_safezone_stop`,
//! `volatility_stop`, `parabolic_sar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AtrTrailingStopReport {
    pub long_stop: Vec<Option<f64>>,
    pub short_stop: Vec<Option<f64>>,
    pub period: usize,
    pub multiplier: f64,
}

pub fn compute(bars: &[Bar], period: usize, multiplier: f64) -> AtrTrailingStopReport {
    let n = bars.len();
    let mut report = AtrTrailingStopReport {
        long_stop: vec![None; n],
        short_stop: vec![None; n],
        period,
        multiplier,
    };
    if period < 2 || !multiplier.is_finite() || multiplier <= 0.0 || n < period + 1 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return report;
    }
    let mut tr = vec![0.0_f64; n];
    tr[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let pc = bars[i - 1].close;
        tr[i] = (bars[i].high - bars[i].low)
            .max((bars[i].high - pc).abs())
            .max((bars[i].low - pc).abs());
    }
    let p_f = period as f64;
    let seed: f64 = tr[1..=period].iter().sum::<f64>() / p_f;
    let mut atr = vec![None; n];
    atr[period] = Some(seed);
    let mut cur = seed;
    for i in (period + 1)..n {
        cur = (cur * (p_f - 1.0) + tr[i]) / p_f;
        atr[i] = Some(cur);
    }
    let mut last_long: Option<f64> = None;
    let mut last_short: Option<f64> = None;
    for (i, a_opt) in atr.iter().enumerate() {
        if let Some(a) = a_opt {
            let raw_long = bars[i].close - multiplier * a;
            let raw_short = bars[i].close + multiplier * a;
            let new_long = match last_long {
                Some(p) => raw_long.max(p),
                None => raw_long,
            };
            let new_short = match last_short {
                Some(p) => raw_short.min(p),
                None => raw_short,
            };
            report.long_stop[i] = Some(new_long);
            report.short_stop[i] = Some(new_short);
            last_long = Some(new_long);
            last_short = Some(new_short);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1, 3.0);
        assert!(r.long_stop.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 14, 0.0);
        assert!(r2.long_stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 14, 3.0);
        assert!(r.long_stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_stops_around_constant() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 14, 3.0);
        // ATR settles to 2 → long stop ≈ 94, short stop ≈ 106.
        let last = 49;
        assert!((r.long_stop[last].unwrap() - 94.0).abs() < 0.1);
        assert!((r.short_stop[last].unwrap() - 106.0).abs() < 0.1);
    }

    #[test]
    fn long_stop_ratchets_up_in_uptrend() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let m = 100.0 + i as f64;
                b(m + 0.5, m - 0.5, m)
            })
            .collect();
        let r = compute(&bars, 14, 3.0);
        let vals: Vec<f64> = r.long_stop.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] >= w[0] - 1e-9);
        }
    }

    #[test]
    fn short_stop_ratchets_down_in_downtrend() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let m = 200.0 - i as f64;
                b(m + 0.5, m - 0.5, m)
            })
            .collect();
        let r = compute(&bars, 14, 3.0);
        let vals: Vec<f64> = r.short_stop.iter().flatten().copied().collect();
        for w in vals.windows(2) {
            assert!(w[1] <= w[0] + 1e-9);
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 14, 3.0);
        assert_eq!(r.long_stop.len(), 30);
        assert_eq!(r.short_stop.len(), 30);
    }
}
