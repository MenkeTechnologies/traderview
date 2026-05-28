//! Sterling Ratio (Deane Sterling, 1981).
//!
//! Performance metric relating annualized return to the average
//! magnitude of the largest annual drawdowns:
//!
//!   Sterling = (annualized_return − risk_free) / (mean of top-K abs DD)
//!
//! Original formulation: average of the K largest annual drawdowns,
//! then subtract 10% (sometimes called "Sterling-Cooper").
//!
//! Modern variant: just average of the K largest drawdowns over the
//! full series, no 10% subtraction. We implement BOTH and let the
//! caller choose.
//!
//! Pure compute. Companion to `burke_ratio`, `recovery_factor`,
//! `pain_index`, `ulcer_index`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SterlingReport {
    pub sterling_modern: f64,
    pub sterling_cooper: f64,
    pub mean_top_k_drawdown: f64,
    pub annualized_return: f64,
    pub n_drawdowns: usize,
    pub k_used: usize,
}

pub fn compute(
    equity: &[f64],
    periods_per_year: f64,
    risk_free_annualized: f64,
    top_k: usize,
) -> Option<SterlingReport> {
    let n = equity.len();
    if n < 2 || top_k == 0
        || !periods_per_year.is_finite() || periods_per_year <= 0.0
        || !risk_free_annualized.is_finite() { return None; }
    if equity.iter().any(|x| !x.is_finite() || *x <= 0.0) { return None; }
    let start = equity[0];
    let end = *equity.last().unwrap();
    let cumulative_return = end / start - 1.0;
    let years = (n - 1) as f64 / periods_per_year;
    if years <= 0.0 { return None; }
    let ann_return = (1.0 + cumulative_return).powf(1.0 / years) - 1.0;
    let excess = ann_return - risk_free_annualized;
    // Identify per-trough drawdowns between consecutive new HWMs.
    let mut hwm = start;
    let mut current_trough_dd = 0.0_f64;
    let mut drawdowns = Vec::new();
    for &v in &equity[1..] {
        if v > hwm {
            if current_trough_dd > 0.0 { drawdowns.push(current_trough_dd); }
            hwm = v;
            current_trough_dd = 0.0;
        } else {
            let dd = (hwm - v) / hwm;
            if dd > current_trough_dd { current_trough_dd = dd; }
        }
    }
    if current_trough_dd > 0.0 { drawdowns.push(current_trough_dd); }
    if drawdowns.is_empty() {
        return Some(SterlingReport {
            sterling_modern: 0.0,
            sterling_cooper: 0.0,
            mean_top_k_drawdown: 0.0,
            annualized_return: ann_return,
            n_drawdowns: 0,
            k_used: 0,
        });
    }
    drawdowns.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let k = top_k.min(drawdowns.len());
    let mean_top_k: f64 = drawdowns[..k].iter().sum::<f64>() / k as f64;
    let denom_modern = mean_top_k;
    let denom_cooper = mean_top_k + 0.10;
    let sterling_modern = if denom_modern > 0.0 { excess / denom_modern } else { 0.0 };
    let sterling_cooper = if denom_cooper > 0.0 { excess / denom_cooper } else { 0.0 };
    Some(SterlingReport {
        sterling_modern,
        sterling_cooper,
        mean_top_k_drawdown: mean_top_k,
        annualized_return: ann_return,
        n_drawdowns: drawdowns.len(),
        k_used: k,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[100.0], 252.0, 0.0, 3).is_none());
        assert!(compute(&[100.0, 110.0], 0.0, 0.0, 3).is_none());
        assert!(compute(&[100.0, 110.0], 252.0, 0.0, 0).is_none());
        assert!(compute(&[100.0, f64::NAN], 252.0, 0.0, 3).is_none());
        assert!(compute(&[100.0, 0.0], 252.0, 0.0, 3).is_none());
    }

    #[test]
    fn monotone_uptrend_yields_zero_sterling() {
        let eq: Vec<f64> = (0..253).map(|i| 100.0 + i as f64).collect();
        let r = compute(&eq, 252.0, 0.0, 3).unwrap();
        assert_eq!(r.n_drawdowns, 0);
        assert_eq!(r.sterling_modern, 0.0);
    }

    #[test]
    fn drawdowns_ranked_for_top_k() {
        // Three DDs of 5%, 10%, 20%. Top-2 mean = (20+10)/2 = 15%.
        let eq = vec![
            100.0, 110.0, 88.0,    // DD = 20%
            120.0, 108.0,         // DD = 10%
            130.0, 123.5,         // DD = 5%
            140.0,
        ];
        let r = compute(&eq, 252.0, 0.0, 2).unwrap();
        assert_eq!(r.n_drawdowns, 3);
        assert_eq!(r.k_used, 2);
        assert!((r.mean_top_k_drawdown - 0.15).abs() < 1e-9);
    }

    #[test]
    fn top_k_clamped_to_n_drawdowns() {
        let eq = vec![100.0, 110.0, 95.0, 120.0];
        let r = compute(&eq, 252.0, 0.0, 100).unwrap();
        // Only 1 DD exists; k clamps to 1.
        assert_eq!(r.k_used, 1);
    }

    #[test]
    fn cooper_variant_smaller_than_modern() {
        let eq = vec![100.0, 110.0, 80.0, 120.0];
        let r = compute(&eq, 252.0, 0.0, 1).unwrap();
        // Sterling-Cooper has +10% in denominator → smaller positive ratio
        // when modern is positive.
        if r.sterling_modern > 0.0 {
            assert!(r.sterling_cooper < r.sterling_modern);
        }
    }

    #[test]
    fn annualization_uses_period_count() {
        let eq: Vec<f64> = (0..=252).map(|_| 100.0).collect();
        // Wait, all-100 fails the >0 check? No, all > 0. But equity flat → no return.
        let r = compute(&eq, 252.0, 0.0, 3).unwrap();
        assert!((r.annualized_return).abs() < 1e-12);
    }
}
