//! Madrid Moving Average Ribbon — TradingView popular indicator.
//!
//! 8 EMAs of varying periods plotted as a ribbon. Color/regime is
//! classified by the alignment of consecutive EMAs:
//!
//!   periods (defaults): 5, 10, 15, 20, 25, 30, 40, 50
//!
//!   BullishStrong : every EMA above the next slower one
//!     (ema_5 > ema_10 > ema_15 > ... > ema_50)
//!   BearishStrong : every EMA below the next slower one
//!   Bullish       : majority of consecutive pairs in ascending order
//!   Bearish       : majority in descending order
//!   Neutral       : otherwise
//!
//! Pure compute. Companion to `guppy_mma`, `alligator`,
//! `traders_action_zone`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MadridRibbonRegime {
    #[default]
    Neutral,
    BullishStrong,
    Bullish,
    Bearish,
    BearishStrong,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MadridRibbonReport {
    /// One inner Vec per EMA period, in the same order as `periods`.
    pub ribbons: Vec<Vec<Option<f64>>>,
    pub periods: Vec<usize>,
    pub regime: Vec<MadridRibbonRegime>,
}

pub fn compute(closes: &[f64]) -> MadridRibbonReport {
    compute_with(closes, vec![5, 10, 15, 20, 25, 30, 40, 50])
}

pub fn compute_with(closes: &[f64], periods: Vec<usize>) -> MadridRibbonReport {
    let n = closes.len();
    let mut report = MadridRibbonReport {
        ribbons: Vec::new(),
        periods: periods.clone(),
        regime: vec![MadridRibbonRegime::Neutral; n],
    };
    if periods.is_empty() || periods.iter().any(|p| *p < 2) { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    for p in &periods {
        report.ribbons.push(ema(closes, *p));
    }
    let k = periods.len();
    for i in 0..n {
        let vals: Option<Vec<f64>> = (0..k).map(|j| report.ribbons[j][i]).collect();
        let Some(vals) = vals else { continue };
        let mut up_pairs = 0_usize;
        let mut dn_pairs = 0_usize;
        for w in vals.windows(2) {
            if w[0] > w[1] { up_pairs += 1; }
            else if w[0] < w[1] { dn_pairs += 1; }
        }
        let total = vals.len() - 1;
        report.regime[i] = if up_pairs == total {
            MadridRibbonRegime::BullishStrong
        } else if dn_pairs == total {
            MadridRibbonRegime::BearishStrong
        } else if up_pairs > dn_pairs {
            MadridRibbonRegime::Bullish
        } else if dn_pairs > up_pairs {
            MadridRibbonRegime::Bearish
        } else {
            MadridRibbonRegime::Neutral
        };
    }
    report
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
    fn empty_or_invalid_returns_empty() {
        let r = compute(&[]);
        // 8 EMAs returned, each an empty inner Vec.
        assert!(r.ribbons.iter().all(|s| s.is_empty()));
        assert!(r.regime.is_empty());
        // Invalid periods → no ribbons constructed at all.
        let c = vec![100.0_f64; 100];
        let r2 = compute_with(&c, vec![1, 5]);
        assert!(r2.ribbons.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        let r = compute(&c);
        assert!(r.ribbons.is_empty());
    }

    #[test]
    fn strong_uptrend_yields_bullish_strong() {
        let c: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c);
        assert_eq!(r.regime[199], MadridRibbonRegime::BullishStrong);
    }

    #[test]
    fn strong_downtrend_yields_bearish_strong() {
        let c: Vec<f64> = (0..200).map(|i| 300.0 - i as f64).collect();
        let r = compute(&c);
        assert_eq!(r.regime[199], MadridRibbonRegime::BearishStrong);
    }

    #[test]
    fn flat_market_yields_neutral() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c);
        // All EMAs converge to 100 → no strict pairwise order → Neutral.
        for v in r.regime.iter().skip(60) {
            assert_eq!(*v, MadridRibbonRegime::Neutral);
        }
    }

    #[test]
    fn ribbons_have_correct_lengths() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c);
        assert_eq!(r.ribbons.len(), 8);
        for s in &r.ribbons {
            assert_eq!(s.len(), 100);
        }
        assert_eq!(r.periods.len(), 8);
    }
}
