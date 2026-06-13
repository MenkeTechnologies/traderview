//! Up / down capture ratio — how much of a benchmark's gains a strategy captures
//! in rising markets versus how much of its losses it suffers in falling markets.
//!
//! Following the Morningstar method, periods are split by the sign of the
//! benchmark return. Within each group the strategy and benchmark returns are
//! geometrically compounded, and the capture is the ratio of the two:
//!
//! ```text
//! up capture   = compound(fund | benchmark > 0) / compound(benchmark | > 0) × 100
//! down capture = compound(fund | benchmark < 0) / compound(benchmark | < 0) × 100
//! capture ratio = up capture / down capture
//! ```
//!
//! An up capture above 100 means the strategy beats the benchmark in up markets;
//! a down capture below 100 means it loses less in down markets. A capture ratio
//! above 1 is the favorable asymmetry every manager wants.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CaptureRatioInput {
    /// Strategy / fund period returns, percent.
    pub fund_returns_pct: Vec<f64>,
    /// Benchmark period returns, percent (paired with the fund returns).
    pub benchmark_returns_pct: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CaptureRatioResult {
    pub up_periods: usize,
    pub down_periods: usize,
    /// Compounded fund return over up-benchmark periods, percent.
    pub fund_up_return_pct: f64,
    pub benchmark_up_return_pct: f64,
    pub fund_down_return_pct: f64,
    pub benchmark_down_return_pct: f64,
    /// None when there are no up periods (or the benchmark was flat).
    pub up_capture_pct: Option<f64>,
    /// None when there are no down periods.
    pub down_capture_pct: Option<f64>,
    /// up capture / down capture. None when either is unavailable or zero.
    pub capture_ratio: Option<f64>,
    /// Up capture > 100 and down capture < 100.
    pub is_favorable: bool,
}

/// Geometric compound of percent returns: ∏(1 + r/100) − 1, as a percent.
fn compound(returns: &[f64]) -> f64 {
    let growth: f64 = returns.iter().fold(1.0, |acc, r| acc * (1.0 + r / 100.0));
    (growth - 1.0) * 100.0
}

pub fn analyze(input: &CaptureRatioInput) -> CaptureRatioResult {
    let n = input
        .fund_returns_pct
        .len()
        .min(input.benchmark_returns_pct.len());

    let mut fund_up = Vec::new();
    let mut bench_up = Vec::new();
    let mut fund_down = Vec::new();
    let mut bench_down = Vec::new();
    for i in 0..n {
        let b = input.benchmark_returns_pct[i];
        let f = input.fund_returns_pct[i];
        if b > 0.0 {
            fund_up.push(f);
            bench_up.push(b);
        } else if b < 0.0 {
            fund_down.push(f);
            bench_down.push(b);
        }
    }

    let fund_up_ret = compound(&fund_up);
    let bench_up_ret = compound(&bench_up);
    let fund_down_ret = compound(&fund_down);
    let bench_down_ret = compound(&bench_down);

    let up_capture = if !bench_up.is_empty() && bench_up_ret != 0.0 {
        Some(fund_up_ret / bench_up_ret * 100.0)
    } else {
        None
    };
    let down_capture = if !bench_down.is_empty() && bench_down_ret != 0.0 {
        Some(fund_down_ret / bench_down_ret * 100.0)
    } else {
        None
    };
    let capture_ratio = match (up_capture, down_capture) {
        (Some(u), Some(d)) if d != 0.0 => Some(u / d),
        _ => None,
    };
    let is_favorable = matches!((up_capture, down_capture), (Some(u), Some(d)) if u > 100.0 && d < 100.0);

    CaptureRatioResult {
        up_periods: bench_up.len(),
        down_periods: bench_down.len(),
        fund_up_return_pct: fund_up_ret,
        benchmark_up_return_pct: bench_up_ret,
        fund_down_return_pct: fund_down_ret,
        benchmark_down_return_pct: bench_down_ret,
        up_capture_pct: up_capture,
        down_capture_pct: down_capture,
        capture_ratio,
        is_favorable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    // Fund amplifies up moves (+20 vs +10) and softens down moves (−5 vs −10).
    fn asymmetric() -> CaptureRatioInput {
        CaptureRatioInput {
            fund_returns_pct: vec![20.0, -5.0, 20.0, -5.0],
            benchmark_returns_pct: vec![10.0, -10.0, 10.0, -10.0],
        }
    }

    #[test]
    fn period_counts() {
        let r = analyze(&asymmetric());
        assert_eq!(r.up_periods, 2);
        assert_eq!(r.down_periods, 2);
    }

    #[test]
    fn compounded_up_returns() {
        let r = analyze(&asymmetric());
        // 1.2 × 1.2 − 1 = 44%; 1.1 × 1.1 − 1 = 21%.
        assert!(close(r.fund_up_return_pct, 44.0));
        assert!(close(r.benchmark_up_return_pct, 21.0));
    }

    #[test]
    fn up_capture_above_100() {
        let r = analyze(&asymmetric());
        // 44 / 21 × 100 = 209.5238.
        assert!(close(r.up_capture_pct.unwrap(), 209.52381));
    }

    #[test]
    fn down_capture_below_100() {
        let r = analyze(&asymmetric());
        // fund −9.75% / bench −19% × 100 = 51.3158.
        assert!(close(r.fund_down_return_pct, -9.75));
        assert!(close(r.benchmark_down_return_pct, -19.0));
        assert!(close(r.down_capture_pct.unwrap(), 51.315789));
    }

    #[test]
    fn capture_ratio_and_favorable() {
        let r = analyze(&asymmetric());
        // 209.5238 / 51.3158 = 4.0831.
        assert!(close(r.capture_ratio.unwrap(), 4.083102));
        assert!(r.is_favorable);
    }

    #[test]
    fn perfect_tracking_is_100() {
        let r = analyze(&CaptureRatioInput {
            fund_returns_pct: vec![10.0, -10.0],
            benchmark_returns_pct: vec![10.0, -10.0],
        });
        assert!(close(r.up_capture_pct.unwrap(), 100.0));
        assert!(close(r.down_capture_pct.unwrap(), 100.0));
        assert!(close(r.capture_ratio.unwrap(), 1.0));
        assert!(!r.is_favorable);
    }

    #[test]
    fn no_down_periods_yields_none() {
        let r = analyze(&CaptureRatioInput {
            fund_returns_pct: vec![12.0, 6.0],
            benchmark_returns_pct: vec![10.0, 5.0],
        });
        assert_eq!(r.down_periods, 0);
        assert!(r.down_capture_pct.is_none());
        assert!(r.capture_ratio.is_none());
        assert!(!r.is_favorable);
    }

    #[test]
    fn flat_benchmark_periods_excluded() {
        let r = analyze(&CaptureRatioInput {
            fund_returns_pct: vec![5.0, 3.0, -2.0],
            benchmark_returns_pct: vec![0.0, 4.0, -3.0],
        });
        // The flat (0%) benchmark period counts as neither up nor down.
        assert_eq!(r.up_periods, 1);
        assert_eq!(r.down_periods, 1);
    }
}
