//! Morningstar Up/Down Market Capture Ratios.
//!
//!   Up capture (%)   = mean portfolio return in months where benchmark > 0
//!                      ─────────────────────────────────────────────────  · 100
//!                      mean benchmark return in those same months
//!
//!   Down capture (%) = mean portfolio return in months where benchmark < 0
//!                      ─────────────────────────────────────────────────  · 100
//!                      mean benchmark return in those same months
//!
//! Interpretation:
//!   - Up > 100   = portfolio outperforms benchmark in up markets
//!   - Down < 100 = portfolio loses less than benchmark in down markets
//!   - Capture spread = Up − Down > 0 ideal (asymmetric upside participation)
//!
//! Months where benchmark == 0 are excluded (no ratio defined).
//!
//! Pure compute. Companion to `treynor_jensen`, `henriksson_merton`,
//! `treynor_mazuy`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaptureReport {
    pub up_capture_pct: Option<f64>,
    pub down_capture_pct: Option<f64>,
    /// Up capture minus down capture; positive ⇒ asymmetric outperformance.
    pub capture_spread: Option<f64>,
    pub up_months: usize,
    pub down_months: usize,
    pub neutral_months: usize,
    pub mean_portfolio_up: f64,
    pub mean_benchmark_up: f64,
    pub mean_portfolio_down: f64,
    pub mean_benchmark_down: f64,
}

pub fn compute(portfolio: &[f64], benchmark: &[f64]) -> Option<CaptureReport> {
    if portfolio.is_empty() || portfolio.len() != benchmark.len() {
        return None;
    }
    let mut up_p = 0.0;
    let mut up_b = 0.0;
    let mut up_n = 0_usize;
    let mut dn_p = 0.0;
    let mut dn_b = 0.0;
    let mut dn_n = 0_usize;
    let mut neutral = 0_usize;
    for (p, b) in portfolio.iter().zip(benchmark.iter()) {
        if !p.is_finite() || !b.is_finite() {
            continue;
        }
        if *b > 0.0 {
            up_p += p;
            up_b += b;
            up_n += 1;
        } else if *b < 0.0 {
            dn_p += p;
            dn_b += b;
            dn_n += 1;
        } else {
            neutral += 1;
        }
    }
    if up_n == 0 && dn_n == 0 {
        return None;
    }
    let up_p_mean = if up_n > 0 { up_p / up_n as f64 } else { 0.0 };
    let up_b_mean = if up_n > 0 { up_b / up_n as f64 } else { 0.0 };
    let dn_p_mean = if dn_n > 0 { dn_p / dn_n as f64 } else { 0.0 };
    let dn_b_mean = if dn_n > 0 { dn_b / dn_n as f64 } else { 0.0 };
    let up_cap = if up_n > 0 && up_b_mean != 0.0 {
        Some(up_p_mean / up_b_mean * 100.0)
    } else {
        None
    };
    let dn_cap = if dn_n > 0 && dn_b_mean != 0.0 {
        Some(dn_p_mean / dn_b_mean * 100.0)
    } else {
        None
    };
    let spread = match (up_cap, dn_cap) {
        (Some(u), Some(d)) => Some(u - d),
        _ => None,
    };
    Some(CaptureReport {
        up_capture_pct: up_cap,
        down_capture_pct: dn_cap,
        capture_spread: spread,
        up_months: up_n,
        down_months: dn_n,
        neutral_months: neutral,
        mean_portfolio_up: up_p_mean,
        mean_benchmark_up: up_b_mean,
        mean_portfolio_down: dn_p_mean,
        mean_benchmark_down: dn_b_mean,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_mismatched_returns_none() {
        assert!(compute(&[], &[]).is_none());
        assert!(compute(&[0.01], &[0.01, 0.02]).is_none());
    }

    #[test]
    fn all_zero_benchmark_returns_none() {
        let p = vec![0.01, 0.02, 0.03];
        let b = vec![0.0, 0.0, 0.0];
        assert!(compute(&p, &b).is_none());
    }

    #[test]
    fn nan_inputs_skipped() {
        let p = vec![f64::NAN, 0.02, 0.03];
        let b = vec![0.01, 0.02, 0.03];
        let r = compute(&p, &b).unwrap();
        assert_eq!(r.up_months, 2);
    }

    #[test]
    fn portfolio_equal_to_benchmark_yields_100() {
        let p = vec![0.02, -0.01, 0.03, -0.02];
        let b = p.clone();
        let r = compute(&p, &b).unwrap();
        assert!((r.up_capture_pct.unwrap() - 100.0).abs() < 1e-9);
        assert!((r.down_capture_pct.unwrap() - 100.0).abs() < 1e-9);
        assert!(r.capture_spread.unwrap().abs() < 1e-9);
    }

    #[test]
    fn higher_up_capture_for_levered_portfolio() {
        // 1.5x leveraged portfolio: 150% up capture, 150% down capture.
        let b = vec![0.02, -0.01, 0.03, -0.02];
        let p: Vec<f64> = b.iter().map(|x| x * 1.5).collect();
        let r = compute(&p, &b).unwrap();
        assert!((r.up_capture_pct.unwrap() - 150.0).abs() < 1e-9);
        assert!((r.down_capture_pct.unwrap() - 150.0).abs() < 1e-9);
        assert!(r.capture_spread.unwrap().abs() < 1e-9);
    }

    #[test]
    fn asymmetric_strategy_positive_capture_spread() {
        // Captures all upside but only half the downside.
        let b = vec![0.02, -0.02, 0.03, -0.03, 0.01, -0.01];
        let p = vec![0.02, -0.01, 0.03, -0.015, 0.01, -0.005];
        let r = compute(&p, &b).unwrap();
        assert!((r.up_capture_pct.unwrap() - 100.0).abs() < 1e-9);
        assert!((r.down_capture_pct.unwrap() - 50.0).abs() < 1e-9);
        assert!(r.capture_spread.unwrap() > 0.0);
    }

    #[test]
    fn benchmark_only_up_months_yields_no_down_capture() {
        let p = vec![0.01, 0.02, 0.03];
        let b = vec![0.01, 0.02, 0.03];
        let r = compute(&p, &b).unwrap();
        assert!(r.up_capture_pct.is_some());
        assert!(r.down_capture_pct.is_none());
        assert!(r.capture_spread.is_none());
    }

    #[test]
    fn neutral_months_counted_separately() {
        let p = vec![0.01, 0.0, -0.01];
        let b = vec![0.01, 0.0, -0.01];
        let r = compute(&p, &b).unwrap();
        assert_eq!(r.neutral_months, 1);
        assert_eq!(r.up_months, 1);
        assert_eq!(r.down_months, 1);
    }
}
