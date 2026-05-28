//! Three classic risk-adjusted-return ratios that use drawdown-based
//! risk denominators rather than stdev — Burke, Sterling, and Ulcer
//! Performance Index (UPI).
//!
//!   Sterling = (annualized_return − risk_free) / |avg_max_drawdown|
//!              (uses average of the K worst drawdowns)
//!
//!   Burke    = (annualized_return − risk_free) / √Σ(DD_i²)
//!              (uses RMS of all drawdowns — penalizes deep losses
//!              quadratically)
//!
//!   UPI      = (annualized_return − risk_free) / ulcer_index
//!              where ulcer_index = √(mean(DD_t²)) over the equity curve
//!              (Peter Martin 1989 — penalizes both depth AND duration
//!              of drawdowns)
//!
//! Pure compute. Caller supplies the equity curve, returns series, and
//! annualization factor. Companion to `ulcer_index` (which only
//! computes the denominator).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct RiskAdjustedReport {
    pub annualized_return: f64,
    pub sterling_ratio: f64,
    pub burke_ratio: f64,
    pub ulcer_performance_index: f64,
    pub avg_max_drawdown: f64,
    pub rms_drawdown: f64,
    pub ulcer_index: f64,
    pub n_observations: usize,
}

pub fn compute(
    equity_curve: &[f64],
    period_returns: &[f64],
    risk_free_annual: f64,
    periods_per_year: f64,
    n_worst_drawdowns: usize,
) -> Option<RiskAdjustedReport> {
    if equity_curve.len() < 2
        || period_returns.is_empty()
        || !risk_free_annual.is_finite()
        || !periods_per_year.is_finite() || periods_per_year <= 0.0
        || n_worst_drawdowns == 0
    {
        return None;
    }
    // Annualized return from the return series.
    let clean_returns: Vec<f64> = period_returns.iter().copied()
        .filter(|x| x.is_finite()).collect();
    if clean_returns.is_empty() { return None; }
    let mean_ret = clean_returns.iter().sum::<f64>() / clean_returns.len() as f64;
    let annual_ret = mean_ret * periods_per_year;
    // Drawdown series along the equity curve.
    let mut dds = Vec::with_capacity(equity_curve.len());
    let mut peak = f64::NEG_INFINITY;
    let mut any_valid = false;
    for v in equity_curve {
        if !v.is_finite() { continue; }
        any_valid = true;
        if *v > peak { peak = *v; }
        let dd = if peak > 0.0 { (peak - v).max(0.0) / peak } else { 0.0 };
        dds.push(dd);
    }
    if !any_valid || dds.is_empty() { return None; }
    let n = dds.len();
    // Ulcer index = √(mean(DD²)) where DD is in percent — output unitless.
    let mean_dd_sq = dds.iter().map(|d| d * d).sum::<f64>() / n as f64;
    let ulcer = mean_dd_sq.sqrt();
    // RMS drawdown = √(sum(DD²)) (Burke's form).
    let rms_dd = dds.iter().map(|d| d * d).sum::<f64>().sqrt();
    // Avg-K-worst drawdown for Sterling.
    let mut sorted_dds = dds.clone();
    sorted_dds.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let k = n_worst_drawdowns.min(n);
    let avg_max_dd = sorted_dds[..k].iter().sum::<f64>() / k as f64;
    let excess = annual_ret - risk_free_annual;
    let sterling = if avg_max_dd > 0.0 { excess / avg_max_dd }
        else if excess.abs() < 1e-12 { 0.0 } else { f64::INFINITY * excess.signum() };
    let burke = if rms_dd > 0.0 { excess / rms_dd }
        else if excess.abs() < 1e-12 { 0.0 } else { f64::INFINITY * excess.signum() };
    let upi = if ulcer > 0.0 { excess / ulcer }
        else if excess.abs() < 1e-12 { 0.0 } else { f64::INFINITY * excess.signum() };
    Some(RiskAdjustedReport {
        annualized_return: annual_ret,
        sterling_ratio: sterling,
        burke_ratio: burke,
        ulcer_performance_index: upi,
        avg_max_drawdown: avg_max_dd,
        rms_drawdown: rms_dd,
        ulcer_index: ulcer,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[], &[0.01], 0.0, 252.0, 5).is_none());
        assert!(compute(&[1.0, 2.0], &[], 0.0, 252.0, 5).is_none());
        assert!(compute(&[1.0, 2.0], &[0.01], 0.0, 0.0, 5).is_none());
        assert!(compute(&[1.0, 2.0], &[0.01], 0.0, 252.0, 0).is_none());
        assert!(compute(&[1.0, 2.0], &[0.01], f64::NAN, 252.0, 5).is_none());
    }

    #[test]
    fn monotonic_increasing_yields_infinite_ratios() {
        let curve: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let rets: Vec<f64> = vec![0.05; 19];
        let r = compute(&curve, &rets, 0.0, 252.0, 5).unwrap();
        assert_eq!(r.avg_max_drawdown, 0.0);
        assert_eq!(r.ulcer_index, 0.0);
        assert!(r.sterling_ratio.is_infinite() && r.sterling_ratio > 0.0);
        assert!(r.burke_ratio.is_infinite() && r.burke_ratio > 0.0);
        assert!(r.ulcer_performance_index.is_infinite() && r.ulcer_performance_index > 0.0);
    }

    #[test]
    fn drawdown_curve_yields_finite_ratios() {
        // 100 → 80 → 100 → 90 → 100.
        let curve = vec![100.0, 80.0, 100.0, 90.0, 100.0];
        let rets = vec![0.0, -0.2, 0.25, -0.10, 0.111];
        let r = compute(&curve, &rets, 0.0, 4.0, 2).unwrap();
        assert!(r.sterling_ratio.is_finite());
        assert!(r.burke_ratio.is_finite());
        assert!(r.ulcer_performance_index.is_finite());
        assert!(r.avg_max_drawdown > 0.0);
        assert!(r.ulcer_index > 0.0);
    }

    #[test]
    fn higher_periods_per_year_scales_return() {
        let curve = vec![100.0; 30];
        let rets = vec![0.001; 30];
        let r_daily = compute(&curve, &rets, 0.0, 252.0, 5).unwrap();
        let r_monthly = compute(&curve, &rets, 0.0, 12.0, 5).unwrap();
        // 252 / 12 = 21 ratio for daily vs monthly.
        let ratio = r_daily.annualized_return / r_monthly.annualized_return;
        assert!((ratio - 21.0).abs() < 1e-9);
    }

    #[test]
    fn nan_inputs_skipped() {
        let curve = vec![100.0, f64::NAN, 80.0, 100.0];
        let rets = vec![0.0, f64::NAN, -0.2, 0.25];
        let r = compute(&curve, &rets, 0.0, 4.0, 2).unwrap();
        assert!(r.n_observations < 4);
    }

    #[test]
    fn risk_free_subtracted_from_excess_return() {
        // With matching r_f = annual_ret, all ratios should be zero.
        let curve = vec![100.0, 80.0, 100.0];
        let rets = vec![0.0, -0.2, 0.25];
        let mean_ret: f64 = rets.iter().sum::<f64>() / 3.0;
        let annual = mean_ret * 252.0;
        let r = compute(&curve, &rets, annual, 252.0, 1).unwrap();
        assert!(r.sterling_ratio.abs() < 1e-9 || r.sterling_ratio == 0.0);
    }

    #[test]
    fn k_worst_drawdowns_caps_at_n() {
        let curve = vec![100.0, 80.0, 90.0];
        let rets = vec![0.0, -0.2, 0.125];
        let r = compute(&curve, &rets, 0.0, 252.0, 1000).unwrap();
        assert!(r.sterling_ratio.is_finite() || r.sterling_ratio == 0.0);
    }

    #[test]
    fn upi_distinguishable_from_sterling_on_skewed_drawdown_distribution() {
        // Sterling denom = mean of K largest drawdowns only.
        // UPI denom = sqrt(mean(DD²)) over the WHOLE curve.
        // For a curve dominated by a single big drawdown surrounded by
        // many zero-DD bars, Sterling's denom is FAR bigger than UPI's
        // → Sterling ratio < UPI ratio. Construct exactly that.
        let mut curve = vec![100.0_f64];
        curve.extend(std::iter::repeat_n(100.0, 50));    // 50 flat (zero DD)
        curve.push(60.0);                                 // 40% drawdown
        curve.extend(std::iter::repeat_n(100.0, 50));    // recover + 50 flat
        let rets: Vec<f64> = curve.windows(2)
            .map(|w| if w[0] > 0.0 { (w[1] - w[0]) / w[0] } else { 0.0 })
            .collect();
        let r = compute(&curve, &rets, 0.0, 252.0, 5).unwrap();
        // Sterling 5-worst should differ measurably from RMS-all.
        assert!((r.sterling_ratio - r.ulcer_performance_index).abs() > 1e-6,
            "expected Sterling ({}) ≠ UPI ({})",
            r.sterling_ratio, r.ulcer_performance_index);
    }
}
