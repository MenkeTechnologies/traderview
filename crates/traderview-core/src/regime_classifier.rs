//! Regime Classifier — combines multiple features to tag each bar as
//! one of 4 market regimes:
//!
//!   TrendUp     : strong directional + low chop (efficiency > 0.6, ROC > 0)
//!   TrendDown   : strong directional + low chop (efficiency > 0.6, ROC < 0)
//!   Range       : low chop, low directional movement (efficiency 0.3..0.6)
//!   Chop        : high chop, weak directional (efficiency < 0.3)
//!
//! Where efficiency_ratio = |close_t - close_{t-period}| / Σ |close_k - close_{k-1}|.
//!
//! Pure compute. Default period = 20.
//! Companion to `efficiency_ratio`, `chande_trend_index`,
//! `choppy_market_index`, `volatility_regime`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MarketRegime {
    #[default]
    Chop,
    Range,
    TrendUp,
    TrendDown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegimeClassifierReport {
    pub regime: Vec<Option<MarketRegime>>,
    pub efficiency: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(closes: &[f64], period: usize) -> RegimeClassifierReport {
    let n = closes.len();
    let mut report = RegimeClassifierReport {
        regime: vec![None; n],
        efficiency: vec![None; n],
        period,
    };
    if period < 2 || n < period + 1 {
        return report;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return report;
    }
    for i in period..n {
        let net = (closes[i] - closes[i - period]).abs();
        let path: f64 = (i - period + 1..=i)
            .map(|k| (closes[k] - closes[k - 1]).abs())
            .sum();
        let eff = if path > 0.0 { net / path } else { 0.0 };
        let roc = closes[i] - closes[i - period];
        let regime = if eff > 0.6 {
            if roc > 0.0 {
                MarketRegime::TrendUp
            } else {
                MarketRegime::TrendDown
            }
        } else if eff > 0.3 {
            MarketRegime::Range
        } else {
            MarketRegime::Chop
        };
        report.efficiency[i] = Some(eff);
        report.regime[i] = Some(regime);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 30];
        let r = compute(&c, 1);
        assert!(r.regime.iter().all(|x| x.is_none()));
        let r2 = compute(&c[..5], 20);
        assert!(r2.regime.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 30];
        c[5] = f64::NAN;
        let r = compute(&c, 20);
        assert!(r.regime.iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfect_uptrend_classified_trend_up() {
        let c: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 20);
        let last = 29;
        assert_eq!(r.regime[last].unwrap(), MarketRegime::TrendUp);
        assert!((r.efficiency[last].unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn perfect_downtrend_classified_trend_down() {
        let c: Vec<f64> = (0..30).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 20);
        assert_eq!(r.regime[29].unwrap(), MarketRegime::TrendDown);
    }

    #[test]
    fn alternating_classified_chop() {
        let c: Vec<f64> = (0_usize..30)
            .map(|i| if i.is_multiple_of(2) { 100.0 } else { 102.0 })
            .collect();
        let r = compute(&c, 20);
        let last = 29;
        // Net ≈ 2 (or 0), path ≈ 38 → eff very low → Chop.
        assert_eq!(r.regime[last].unwrap(), MarketRegime::Chop);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 30];
        let r = compute(&c, 20);
        assert_eq!(r.regime.len(), 30);
        assert_eq!(r.efficiency.len(), 30);
    }
}
