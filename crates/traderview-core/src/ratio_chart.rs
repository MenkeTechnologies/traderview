//! Ratio Chart — per-bar price ratio of asset A vs reference asset B.
//!
//! Computes ratio_t = close_a_t / close_b_t for each aligned bar, plus
//! a normalized form ratio_norm_t = ratio_t / ratio_0 (base-100 indexed
//! to the first bar) for easy visual comparison of relative
//! performance over the window.
//!
//! Used for:
//!   - Sector rotation (XLF / SPY)
//!   - Risk-on/risk-off (HYG / TLT, XLY / XLP)
//!   - Stock vs benchmark (AAPL / QQQ)
//!   - Gold-silver ratio (XAU / XAG)
//!
//! Pure compute. Companion to `spread_chart`, `relative_strength`,
//! `pair_trade_zscore`, `cointegration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RatioChartReport {
    pub ratio: Vec<Option<f64>>,
    pub ratio_normalized: Vec<Option<f64>>,
    pub n: usize,
}

pub fn compute(closes_a: &[f64], closes_b: &[f64]) -> RatioChartReport {
    let n = closes_a.len();
    let mut report = RatioChartReport {
        ratio: vec![None; n],
        ratio_normalized: vec![None; n],
        n,
    };
    if n == 0 || closes_b.len() != n {
        return report;
    }
    if closes_a
        .iter()
        .chain(closes_b.iter())
        .any(|x| !x.is_finite())
    {
        return report;
    }
    let mut first_ratio: Option<f64> = None;
    for i in 0..n {
        if closes_b[i] != 0.0 {
            let r = closes_a[i] / closes_b[i];
            report.ratio[i] = Some(r);
            if first_ratio.is_none() {
                first_ratio = Some(r);
            }
            if let Some(base) = first_ratio {
                if base != 0.0 {
                    report.ratio_normalized[i] = Some(r / base * 100.0);
                }
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatched_lengths_return_empty() {
        let a = vec![100.0_f64; 10];
        let b = vec![100.0_f64; 9];
        let r = compute(&a, &b);
        assert!(r.ratio.iter().all(|x| x.is_none()));
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], &[]);
        assert!(r.ratio.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let a = vec![100.0_f64, f64::NAN, 100.0];
        let b = vec![100.0_f64; 3];
        let r = compute(&a, &b);
        assert!(r.ratio.iter().all(|x| x.is_none()));
    }

    #[test]
    fn equal_price_yields_ratio_one() {
        let a = vec![100.0_f64; 10];
        let b = vec![100.0_f64; 10];
        let r = compute(&a, &b);
        for v in r.ratio.iter().flatten() {
            assert!((v - 1.0).abs() < 1e-9);
        }
        for v in r.ratio_normalized.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn outperforming_asset_yields_rising_normalized() {
        // A doubles, B flat → ratio doubles → normalized goes 100 → 200.
        let a: Vec<f64> = (0..10).map(|i| 100.0 + i as f64 * 10.0).collect();
        let b = vec![100.0_f64; 10];
        let r = compute(&a, &b);
        assert!((r.ratio_normalized[0].unwrap() - 100.0).abs() < 1e-9);
        assert!(r.ratio_normalized[9].unwrap() > 100.0);
    }

    #[test]
    fn zero_denominator_yields_none_at_bar() {
        let a = vec![100.0_f64; 3];
        let b = vec![100.0, 0.0, 100.0];
        let r = compute(&a, &b);
        assert!(r.ratio[1].is_none());
        assert!(r.ratio[0].is_some());
        assert!(r.ratio[2].is_some());
    }

    #[test]
    fn output_lengths_match_input() {
        let a = vec![100.0_f64; 10];
        let b = vec![100.0_f64; 10];
        let r = compute(&a, &b);
        assert_eq!(r.ratio.len(), 10);
        assert_eq!(r.ratio_normalized.len(), 10);
        assert_eq!(r.n, 10);
    }
}
