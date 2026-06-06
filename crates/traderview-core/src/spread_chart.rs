//! Spread Chart — per-bar price difference of asset A vs reference asset B.
//!
//! Computes spread_t = close_a_t - hedge_ratio · close_b_t. Optionally
//! z-scores the spread series over a `zscore_period` window to make
//! mean-reversion levels visible:
//!
//!   spread_t      = close_a_t - h · close_b_t
//!   zscore_t      = (spread_t - SMA(spread, period)) / stdev(spread, period)
//!
//! Used for:
//!   - Pairs trading (stat-arb)
//!   - Calendar / inter-month spreads in futures
//!   - Credit spreads (corp vs treasury yield)
//!   - Yield curve spreads (10y - 2y)
//!
//! Pure compute. Companion to `ratio_chart`, `pair_trade_zscore`,
//! `cointegration`, `engle_granger_2step`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpreadChartReport {
    pub spread: Vec<Option<f64>>,
    pub zscore: Vec<Option<f64>>,
    pub hedge_ratio: f64,
    pub zscore_period: usize,
}

pub fn compute(
    closes_a: &[f64],
    closes_b: &[f64],
    hedge_ratio: f64,
    zscore_period: usize,
) -> SpreadChartReport {
    let n = closes_a.len();
    let mut report = SpreadChartReport {
        spread: vec![None; n],
        zscore: vec![None; n],
        hedge_ratio,
        zscore_period,
    };
    if n == 0 || closes_b.len() != n || !hedge_ratio.is_finite() || zscore_period < 2 {
        return report;
    }
    if closes_a
        .iter()
        .chain(closes_b.iter())
        .any(|x| !x.is_finite())
    {
        return report;
    }
    for i in 0..n {
        report.spread[i] = Some(closes_a[i] - hedge_ratio * closes_b[i]);
    }
    let p_f = zscore_period as f64;
    for i in (zscore_period - 1)..n {
        let win = &report.spread[i + 1 - zscore_period..=i];
        let vals: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        if vals.len() != zscore_period {
            continue;
        }
        let mean: f64 = vals.iter().sum::<f64>() / p_f;
        let var: f64 = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let std = var.max(0.0).sqrt();
        if std > 0.0 {
            report.zscore[i] = Some((report.spread[i].unwrap() - mean) / std);
        } else {
            report.zscore[i] = Some(0.0);
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
        let r = compute(&a, &b, 1.0, 5);
        assert!(r.spread.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let a = vec![100.0_f64, f64::NAN, 100.0];
        let b = vec![100.0_f64; 3];
        let r = compute(&a, &b, 1.0, 2);
        assert!(r.spread.iter().all(|x| x.is_none()));
    }

    #[test]
    fn equal_price_unit_hedge_yields_zero_spread() {
        let a = vec![100.0_f64; 20];
        let b = vec![100.0_f64; 20];
        let r = compute(&a, &b, 1.0, 5);
        for v in r.spread.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn spread_widens_when_a_outperforms() {
        let a: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let b = vec![100.0_f64; 20];
        let r = compute(&a, &b, 1.0, 5);
        assert!(r.spread[0].unwrap() < r.spread[19].unwrap());
    }

    #[test]
    fn zscore_zero_for_constant_spread() {
        let a = vec![100.0_f64; 20];
        let b = vec![100.0_f64; 20];
        let r = compute(&a, &b, 1.0, 5);
        for v in r.zscore.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn hedge_ratio_adjusts_spread_baseline() {
        let a = vec![200.0_f64; 20];
        let b = vec![100.0_f64; 20];
        let r1 = compute(&a, &b, 1.0, 5);
        let r2 = compute(&a, &b, 2.0, 5);
        // h=1 → spread = 100; h=2 → spread = 0.
        assert!((r1.spread[10].unwrap() - 100.0).abs() < 1e-9);
        assert!(r2.spread[10].unwrap().abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let a = vec![100.0_f64; 10];
        let b = vec![100.0_f64; 10];
        let r = compute(&a, &b, 1.0, 5);
        assert_eq!(r.spread.len(), 10);
        assert_eq!(r.zscore.len(), 10);
    }
}
