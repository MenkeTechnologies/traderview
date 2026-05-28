//! Relative Strength vs Market (RS line) — IBD / William O'Neil school.
//!
//! Per-bar normalized ratio of stock close vs benchmark close, indexed
//! to a starting value of 100 so the chart can be compared visually
//! to the price chart:
//!
//!   ratio_t      = stock_t / benchmark_t
//!   rs_line_t    = ratio_t / ratio_0 · 100
//!   rs_change_t  = (rs_line_t - rs_line_{t-period}) / rs_line_{t-period} · 100
//!
//! IBD's interpretation:
//!   RS line at all-time-high while stock breaks out → strongest
//!     possible setup (stock outperforming market into the move)
//!   RS line falling while stock rallies → "fading rally" (stock is
//!     lagging the broader market — weak rally that's likely to fail)
//!
//! `period` is the lookback for the rate-of-change reading.
//!
//! Pure compute. Default period = 63 (one quarter of trading days).
//! Companion to `relative_strength`, `ratio_chart`, `spread_chart`,
//! `momentum_12_1`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RsVsMarketReport {
    pub rs_line: Vec<Option<f64>>,
    pub rs_change_pct: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(
    stock_closes: &[f64],
    benchmark_closes: &[f64],
    period: usize,
) -> RsVsMarketReport {
    let n = stock_closes.len();
    let mut report = RsVsMarketReport {
        rs_line: vec![None; n],
        rs_change_pct: vec![None; n],
        period,
    };
    if n == 0 || benchmark_closes.len() != n || period < 2 { return report; }
    if stock_closes.iter().chain(benchmark_closes.iter()).any(|x| !x.is_finite()) {
        return report;
    }
    let mut first_ratio: Option<f64> = None;
    for i in 0..n {
        if benchmark_closes[i] != 0.0 {
            let r = stock_closes[i] / benchmark_closes[i];
            if first_ratio.is_none() {
                first_ratio = Some(r);
            }
            if let Some(base) = first_ratio {
                if base != 0.0 {
                    report.rs_line[i] = Some(r / base * 100.0);
                }
            }
        }
    }
    for i in period..n {
        if let (Some(cur), Some(prev)) = (report.rs_line[i], report.rs_line[i - period]) {
            if prev != 0.0 {
                report.rs_change_pct[i] = Some((cur - prev) / prev * 100.0);
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
        let s = vec![100.0_f64; 10];
        let b = vec![100.0_f64; 9];
        let r = compute(&s, &b, 5);
        assert!(r.rs_line.iter().all(|x| x.is_none()));
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], &[], 5);
        assert!(r.rs_line.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let s = vec![100.0_f64, f64::NAN, 100.0];
        let b = vec![100.0_f64; 3];
        let r = compute(&s, &b, 2);
        assert!(r.rs_line.iter().all(|x| x.is_none()));
    }

    #[test]
    fn equal_performance_yields_rs_line_one_hundred() {
        let s = vec![100.0_f64; 100];
        let b = vec![100.0_f64; 100];
        let r = compute(&s, &b, 5);
        for v in r.rs_line.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
        for v in r.rs_change_pct.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn outperforming_stock_yields_rising_rs_line() {
        let s: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.5).collect();
        let b = vec![100.0_f64; 100];
        let r = compute(&s, &b, 5);
        assert!(r.rs_line[99].unwrap() > 100.0);
        assert!(r.rs_change_pct[99].unwrap() > 0.0);
    }

    #[test]
    fn underperforming_stock_yields_falling_rs_line() {
        let s = vec![100.0_f64; 100];
        let b: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.5).collect();
        let r = compute(&s, &b, 5);
        assert!(r.rs_line[99].unwrap() < 100.0);
        assert!(r.rs_change_pct[99].unwrap() < 0.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let s = vec![100.0_f64; 10];
        let b = vec![100.0_f64; 10];
        let r = compute(&s, &b, 5);
        assert_eq!(r.rs_line.len(), 10);
        assert_eq!(r.rs_change_pct.len(), 10);
    }
}
