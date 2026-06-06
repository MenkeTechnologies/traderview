//! Chandelier Exit — Chuck LeBeau (Active Trader Magazine, 1990s).
//!
//! ATR-based trailing stop "hanging" from the highest high (longs) or
//! lowest low (shorts) over the lookback window:
//!
//!   atr_t  = Wilder ATR over period
//!   long_stop_t  = highest_high(period) − multiplier · atr_t
//!   short_stop_t = lowest_low(period)  + multiplier · atr_t
//!
//! Direction flips when close crosses the opposite stop. While the
//! trade direction holds, the stop ratchets in the favorable direction
//! only:
//!   while long:  stop = max(prior_long_stop, raw_long_stop)
//!   while short: stop = min(prior_short_stop, raw_short_stop)
//!
//! Defaults: period = 22, multiplier = 3.0. Pure compute.
//!
//! Companion to `parabolic_sar`, `volatility_stop`, `chande_kroll_stop`,
//! `elder_safezone_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChandelierDirection {
    #[default]
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChandelierReport {
    pub stop: Vec<Option<f64>>,
    pub direction: Vec<Option<ChandelierDirection>>,
    pub long_stop: Vec<Option<f64>>,
    pub short_stop: Vec<Option<f64>>,
    pub period: usize,
    pub multiplier: f64,
}

pub fn compute(bars: &[Bar], period: usize, multiplier: f64) -> ChandelierReport {
    let n = bars.len();
    let mut report = ChandelierReport {
        stop: vec![None; n],
        direction: vec![None; n],
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
    // Wilder ATR.
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
    // Raw stops over highest/lowest of period.
    for i in (period - 1)..n {
        let win = &bars[i + 1 - period..=i];
        let hh = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let ll = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        if let Some(a) = atr[i] {
            report.long_stop[i] = Some(hh - multiplier * a);
            report.short_stop[i] = Some(ll + multiplier * a);
        }
    }
    // Ratcheted stop + direction.
    let mut dir = ChandelierDirection::Long;
    let mut cur_stop: Option<f64> = None;
    for (i, bar) in bars.iter().enumerate() {
        let raw_long = report.long_stop[i];
        let raw_short = report.short_stop[i];
        if raw_long.is_none() || raw_short.is_none() {
            continue;
        }
        let raw_long = raw_long.unwrap();
        let raw_short = raw_short.unwrap();
        let close = bar.close;
        match dir {
            ChandelierDirection::Long => {
                if close < cur_stop.unwrap_or(raw_long) {
                    dir = ChandelierDirection::Short;
                    cur_stop = Some(raw_short);
                } else {
                    cur_stop = Some(raw_long.max(cur_stop.unwrap_or(raw_long)));
                }
            }
            ChandelierDirection::Short => {
                if close > cur_stop.unwrap_or(raw_short) {
                    dir = ChandelierDirection::Long;
                    cur_stop = Some(raw_long);
                } else {
                    cur_stop = Some(raw_short.min(cur_stop.unwrap_or(raw_short)));
                }
            }
        }
        report.stop[i] = cur_stop;
        report.direction[i] = Some(dir);
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
        assert!(r.stop.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 22, 0.0);
        assert!(r2.stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 22, 3.0);
        assert!(r.stop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn long_stop_below_high_short_stop_above_low() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 22, 3.0);
        let last = 49;
        // HH=101, ATR≈2, mult=3 → long_stop = 101 − 6 = 95.
        assert!((r.long_stop[last].unwrap() - 95.0).abs() < 0.1);
        // LL=99, short_stop = 99 + 6 = 105.
        assert!((r.short_stop[last].unwrap() - 105.0).abs() < 0.1);
    }

    #[test]
    fn uptrend_keeps_long_direction() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let m = 100.0 + i as f64;
                b(m + 0.5, m - 0.5, m)
            })
            .collect();
        let r = compute(&bars, 22, 3.0);
        assert_eq!(r.direction[49].unwrap(), ChandelierDirection::Long);
    }

    #[test]
    fn downtrend_flips_to_short() {
        // Start uptrend, then sharp reversal.
        let mut bars: Vec<_> = (0..30)
            .map(|i| {
                let m = 100.0 + i as f64;
                b(m + 0.5, m - 0.5, m)
            })
            .collect();
        for i in 0..30 {
            let m = 130.0 - 2.0 * i as f64;
            bars.push(b(m + 0.5, m - 0.5, m));
        }
        let r = compute(&bars, 22, 3.0);
        let last = bars.len() - 1;
        assert_eq!(r.direction[last].unwrap(), ChandelierDirection::Short);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 22, 3.0);
        assert_eq!(r.stop.len(), 50);
        assert_eq!(r.direction.len(), 50);
    }
}
