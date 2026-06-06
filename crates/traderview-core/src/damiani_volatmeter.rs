//! Damiani Volatmeter — Daniele Damiani.
//!
//! Compares short-window vs long-window ATR (with optional EMA pre-smoothing)
//! to gauge whether the market is in a volatile breakout regime or a
//! quiet, range-bound regime:
//!
//!   atr_fast = Wilder ATR over fast_period   (e.g. 3)
//!   atr_slow = Wilder ATR over slow_period   (e.g. 13)
//!   vol_t    = atr_fast / atr_slow           (ratio, dimensionless)
//!   threshold = 1 + (s_threshold * stdev_factor)    (typ. ≈ 1.4)
//!
//!   regime:
//!     vol > threshold       → Trending   (markets moving, take signals)
//!     vol < 1/threshold     → Quiet      (avoid breakouts, range-fade)
//!     else                  → Transition
//!
//! Pure compute. Companion to `elder_thermometer`, `volatility_quality_index`,
//! `chande_kroll_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DamianiRegime {
    #[default]
    Transition,
    Trending,
    Quiet,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DamianiReport {
    pub vol_ratio: Vec<Option<f64>>,
    pub regime: Vec<Option<DamianiRegime>>,
    pub fast_period: usize,
    pub slow_period: usize,
    pub threshold: f64,
}

pub fn compute(
    bars: &[Bar],
    fast_period: usize,
    slow_period: usize,
    threshold: f64,
) -> DamianiReport {
    let n = bars.len();
    let mut report = DamianiReport {
        vol_ratio: vec![None; n],
        regime: vec![None; n],
        fast_period,
        slow_period,
        threshold,
    };
    if fast_period < 2
        || slow_period < 2
        || fast_period >= slow_period
        || !threshold.is_finite()
        || threshold <= 1.0
        || n < slow_period + 1
    {
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
    let atr_fast = wilder_atr(&tr, fast_period);
    let atr_slow = wilder_atr(&tr, slow_period);
    let inv = 1.0 / threshold;
    for i in 0..n {
        if let (Some(f), Some(s)) = (atr_fast[i], atr_slow[i]) {
            if s > 0.0 {
                let ratio = f / s;
                report.vol_ratio[i] = Some(ratio);
                report.regime[i] = Some(classify(ratio, threshold, inv));
            }
        }
    }
    report
}

fn classify(ratio: f64, hi: f64, lo: f64) -> DamianiRegime {
    if ratio > hi {
        DamianiRegime::Trending
    } else if ratio < lo {
        DamianiRegime::Quiet
    } else {
        DamianiRegime::Transition
    }
}

fn wilder_atr(tr: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = tr.len();
    let mut out = vec![None; n];
    if period == 0 || n < period + 1 {
        return out;
    }
    let p_f = period as f64;
    let seed: f64 = tr[1..=period].iter().sum::<f64>() / p_f;
    out[period] = Some(seed);
    let mut cur = seed;
    for i in (period + 1)..n {
        cur = (cur * (p_f - 1.0) + tr[i]) / p_f;
        out[i] = Some(cur);
    }
    out
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
        let r = compute(&bars, 1, 13, 1.4);
        assert!(r.vol_ratio.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 13, 3, 1.4);
        assert!(r2.vol_ratio.iter().all(|x| x.is_none()));
        let r3 = compute(&bars, 3, 13, 1.0);
        assert!(r3.vol_ratio.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 3, 13, 1.4);
        assert!(r.vol_ratio.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_ratio_one_yields_transition() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 3, 13, 1.4);
        let last = 49;
        assert!((r.vol_ratio[last].unwrap() - 1.0).abs() < 1e-6);
        assert_eq!(r.regime[last].unwrap(), DamianiRegime::Transition);
    }

    #[test]
    fn volatility_burst_marks_trending() {
        // 30 quiet bars, then 10 high-range bars → fast ATR explodes
        // before slow ATR catches up → ratio > 1.4 → Trending.
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        for _ in 0..10 {
            bars.push(b(120.0, 80.0, 100.0));
        }
        let r = compute(&bars, 3, 13, 1.4);
        let last = bars.len() - 1;
        assert!(r.vol_ratio[last].unwrap() > 1.4);
        assert_eq!(r.regime[last].unwrap(), DamianiRegime::Trending);
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(2.0, 1.4, 1.0 / 1.4), DamianiRegime::Trending);
        assert_eq!(classify(0.5, 1.4, 1.0 / 1.4), DamianiRegime::Quiet);
        assert_eq!(classify(1.0, 1.4, 1.0 / 1.4), DamianiRegime::Transition);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 3, 13, 1.4);
        assert_eq!(r.vol_ratio.len(), 30);
        assert_eq!(r.regime.len(), 30);
    }
}
