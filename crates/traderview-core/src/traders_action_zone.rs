//! Trader's Action Zone (TAZ) — Vince Wagner / Online Trading Academy.
//!
//! Classifies each bar as one of 7 trend/correction zones based on the
//! relationship between the close, a fast EMA, and a slow EMA:
//!
//!   bullish_aligned : close > fast_ema > slow_ema
//!   bullish_pulled  : fast_ema > slow_ema AND close ≤ fast_ema
//!   bullish_extended: close > slow_ema BUT fast_ema ≤ slow_ema
//!     (early-trend recovery / pullback breakout)
//!   transition      : close == fast_ema AND fast_ema == slow_ema
//!   bearish_extended: close < slow_ema BUT fast_ema ≥ slow_ema
//!   bearish_pulled  : fast_ema < slow_ema AND close ≥ fast_ema
//!   bearish_aligned : close < fast_ema < slow_ema
//!
//! Standard periods: fast = 10, slow = 30. Wagner uses these zones as a
//! reading of trend strength: trade only when zones are "aligned",
//! tighten or exit on "pulled" zones.
//!
//! Pure compute. Companion to `guppy_mma`, `alligator`, `triple_screen`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionZone {
    #[default]
    Transition,
    BullishAligned,
    BullishPulled,
    BullishExtended,
    BearishExtended,
    BearishPulled,
    BearishAligned,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TazReport {
    pub fast_ema: Vec<Option<f64>>,
    pub slow_ema: Vec<Option<f64>>,
    pub zone: Vec<Option<ActionZone>>,
    pub fast_period: usize,
    pub slow_period: usize,
}

pub fn compute(closes: &[f64], fast_period: usize, slow_period: usize) -> TazReport {
    let n = closes.len();
    let mut report = TazReport {
        fast_ema: vec![None; n],
        slow_ema: vec![None; n],
        zone: vec![None; n],
        fast_period,
        slow_period,
    };
    if fast_period < 2 || slow_period < 2 || fast_period >= slow_period
        || n < slow_period { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    report.fast_ema = ema(closes, fast_period);
    report.slow_ema = ema(closes, slow_period);
    for (i, close) in closes.iter().enumerate() {
        if let (Some(f), Some(s)) = (report.fast_ema[i], report.slow_ema[i]) {
            report.zone[i] = Some(classify(*close, f, s));
        }
    }
    report
}

fn classify(close: f64, fast: f64, slow: f64) -> ActionZone {
    if close == fast && fast == slow { return ActionZone::Transition; }
    let bullish_emas = fast > slow;
    let bearish_emas = fast < slow;
    if bullish_emas {
        if close > fast { ActionZone::BullishAligned }
        else { ActionZone::BullishPulled }
    } else if bearish_emas {
        if close < fast { ActionZone::BearishAligned }
        else { ActionZone::BearishPulled }
    } else if close > slow {
        ActionZone::BullishExtended
    } else if close < slow {
        ActionZone::BearishExtended
    } else {
        ActionZone::Transition
    }
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 60];
        let r = compute(&c, 1, 30);
        assert!(r.zone.iter().all(|x| x.is_none()));
        let r2 = compute(&c, 30, 10);
        assert!(r2.zone.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 60];
        c[5] = f64::NAN;
        let r = compute(&c, 10, 30);
        assert!(r.zone.iter().all(|x| x.is_none()));
    }

    #[test]
    fn strong_uptrend_marks_bullish_aligned() {
        let c: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 10, 30);
        assert_eq!(r.zone[99].unwrap(), ActionZone::BullishAligned);
    }

    #[test]
    fn strong_downtrend_marks_bearish_aligned() {
        let c: Vec<f64> = (0..100).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 10, 30);
        assert_eq!(r.zone[99].unwrap(), ActionZone::BearishAligned);
    }

    #[test]
    fn classify_branches() {
        // fast > slow, close > fast → BullishAligned
        assert_eq!(classify(110.0, 105.0, 100.0), ActionZone::BullishAligned);
        // fast > slow, close ≤ fast → BullishPulled
        assert_eq!(classify(102.0, 105.0, 100.0), ActionZone::BullishPulled);
        // fast < slow, close < fast → BearishAligned
        assert_eq!(classify(90.0, 95.0, 100.0), ActionZone::BearishAligned);
        // fast < slow, close ≥ fast → BearishPulled
        assert_eq!(classify(98.0, 95.0, 100.0), ActionZone::BearishPulled);
        // fast == slow, close > slow → BullishExtended
        assert_eq!(classify(105.0, 100.0, 100.0), ActionZone::BullishExtended);
        // fast == slow, close < slow → BearishExtended
        assert_eq!(classify(95.0, 100.0, 100.0), ActionZone::BearishExtended);
        // all equal → Transition
        assert_eq!(classify(100.0, 100.0, 100.0), ActionZone::Transition);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 60];
        let r = compute(&c, 10, 30);
        assert_eq!(r.zone.len(), 60);
        assert_eq!(r.fast_ema.len(), 60);
    }
}
