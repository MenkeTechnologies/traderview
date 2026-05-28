//! Momentum Crash Protection — Daniel & Moskowitz (2016).
//!
//! Standard momentum strategies crash spectacularly during sharp
//! market rebounds. Daniel-Moskowitz showed that scaling the
//! momentum exposure inversely with realized momentum-vol significantly
//! reduces these tail losses:
//!
//!   w_t = target_vol / forecast_vol_t
//!
//! Optionally combined with a "dead-zone" filter that turns the
//! strategy off when the trailing 1-month return is sharply negative
//! (the so-called Daniel-Moskowitz crash filter).
//!
//! Outputs:
//!   - per-bar leverage
//!   - per-bar managed momentum return
//!   - filter-on-off indicator
//!
//! Distinct from `volatility_managed_portfolio` (which uses any input
//! series and inverse-variance scaling); this uses inverse-vol scaling
//! and adds the crash-state filter.
//!
//! Pure compute. Companion to `volatility_managed_portfolio`,
//! `momentum_12_1`, `vol_targeting_sizer`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrashProtectionReport {
    pub managed_returns: Vec<Option<f64>>,
    pub leverages: Vec<Option<f64>>,
    pub crash_filter_active: Vec<Option<bool>>,
    pub mean_leverage: f64,
    pub n_observations: usize,
}

pub fn manage(
    momentum_returns: &[f64],
    vol_lookback: usize,
    target_annualized_vol: f64,
    periods_per_year: f64,
    max_leverage: f64,
    crash_filter_lookback: usize,
    crash_filter_threshold_pct: f64,
) -> Option<CrashProtectionReport> {
    let n = momentum_returns.len();
    let lookback = vol_lookback.max(crash_filter_lookback);
    if n < lookback + 1 || vol_lookback < 5 || crash_filter_lookback == 0
        || !target_annualized_vol.is_finite() || target_annualized_vol <= 0.0
        || !periods_per_year.is_finite() || periods_per_year <= 0.0
        || !max_leverage.is_finite() || max_leverage <= 0.0
        || !crash_filter_threshold_pct.is_finite() {
        return None;
    }
    if momentum_returns.iter().any(|x| !x.is_finite()) { return None; }
    let target_vol_period = target_annualized_vol / periods_per_year.sqrt();
    let mut managed = vec![None; n];
    let mut leverages = vec![None; n];
    let mut crash_active = vec![None; n];
    let mut lev_sum = 0.0_f64;
    let mut lev_count = 0_usize;
    for i in lookback..n {
        let vol_win = &momentum_returns[i - vol_lookback..i];
        let n_f = vol_lookback as f64;
        let mean: f64 = vol_win.iter().sum::<f64>() / n_f;
        let var: f64 = vol_win.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_f;
        let vol = var.max(0.0).sqrt();
        let raw_lev = if vol > 0.0 { (target_vol_period / vol).min(max_leverage) } else { max_leverage };
        // Crash filter: trailing crash_filter_lookback cumulative return < threshold.
        let crash_win = &momentum_returns[i - crash_filter_lookback..i];
        let cum_ret: f64 = crash_win.iter().fold(1.0_f64, |acc, r| acc * (1.0 + r)) - 1.0;
        let crash_on = cum_ret < crash_filter_threshold_pct;
        let lev = if crash_on { 0.0 } else { raw_lev };
        managed[i] = Some(lev * momentum_returns[i]);
        leverages[i] = Some(lev);
        crash_active[i] = Some(crash_on);
        if lev > 0.0 {
            lev_sum += lev;
            lev_count += 1;
        }
    }
    let mean_lev = if lev_count > 0 { lev_sum / lev_count as f64 } else { 0.0 };
    Some(CrashProtectionReport {
        managed_returns: managed,
        leverages,
        crash_filter_active: crash_active,
        mean_leverage: mean_lev,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let r = vec![0.01_f64; 100];
        assert!(manage(&r, 4, 0.15, 252.0, 4.0, 22, -0.05).is_none());
        assert!(manage(&r, 60, 0.0, 252.0, 4.0, 22, -0.05).is_none());
        assert!(manage(&r, 60, 0.15, 0.0, 4.0, 22, -0.05).is_none());
        assert!(manage(&r, 60, 0.15, 252.0, 0.0, 22, -0.05).is_none());
        assert!(manage(&r, 60, 0.15, 252.0, 4.0, 22, f64::NAN).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 100];
        r[5] = f64::NAN;
        assert!(manage(&r, 60, 0.15, 252.0, 4.0, 22, -0.05).is_none());
    }

    #[test]
    fn crash_filter_zeros_leverage_after_drawdown() {
        // Build returns where the prior 22 days summed cumulative is < -10%.
        let mut r = vec![0.001_f64; 50];
        r.extend(vec![-0.01_f64; 22]);    // bars 50..72: each −1%, cumulative ≈ −20%.
        r.extend(vec![0.001_f64; 30]);
        let result = manage(&r, 60, 0.15, 252.0, 4.0, 22, -0.10).unwrap();
        // At i=72 onwards (just after the crash), crash filter should activate.
        let crash_72 = result.crash_filter_active[72].unwrap();
        assert!(crash_72);
        // Leverage should be 0 when crash filter is active.
        let lev_72 = result.leverages[72].unwrap();
        assert_eq!(lev_72, 0.0);
    }

    #[test]
    fn normal_regime_yields_nonzero_leverage() {
        let mut state: u64 = 42;
        let r: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            0.0005 + ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.01
        }).collect();
        let result = manage(&r, 60, 0.15, 252.0, 4.0, 22, -0.20).unwrap();
        assert!(result.mean_leverage > 0.0);
    }

    #[test]
    fn capped_leverage_in_calm_regime() {
        let r = vec![0.0001_f64; 100];    // very low vol
        let result = manage(&r, 60, 0.15, 252.0, 4.0, 22, -0.20).unwrap();
        // Calm regime → raw leverage would be huge, capped at 4.0.
        let some_lev = result.leverages.iter().filter_map(|x| *x).max_by(|a, b| {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
        }).unwrap();
        assert!(some_lev <= 4.0);
    }

    #[test]
    fn managed_returns_aligned_with_input() {
        let r = vec![0.01_f64; 100];
        let result = manage(&r, 60, 0.15, 252.0, 4.0, 22, -0.20).unwrap();
        assert_eq!(result.managed_returns.len(), 100);
        assert_eq!(result.leverages.len(), 100);
        assert_eq!(result.crash_filter_active.len(), 100);
    }
}
