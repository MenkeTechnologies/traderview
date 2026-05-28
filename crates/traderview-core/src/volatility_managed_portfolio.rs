//! Volatility-Managed Portfolio — Moreira & Muir (2017).
//!
//! Scales a strategy's exposure inversely with its realized variance:
//!
//!   w_t = target_vol² / σ²_t
//!
//! where σ²_t is the realized variance over the prior `lookback` days.
//! Realized return of the managed portfolio:
//!
//!   r_managed_t = w_t · r_t
//!
//! Moreira-Muir found this transformation enhances the Sharpe ratio
//! of equity-style factor portfolios significantly.
//!
//! Companion outputs:
//!   - per-bar leverage (capped at `max_leverage`)
//!   - per-bar managed return
//!   - cumulative equity curve (compounded)
//!
//! Pure compute. Companion to `vol_targeting_sizer` (single-asset),
//! `risk_parity_weights` (cross-sectional), `rolling_sharpe`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolManagedReport {
    pub managed_returns: Vec<Option<f64>>,
    pub leverages: Vec<Option<f64>>,
    pub cumulative_equity: Vec<Option<f64>>,
    pub mean_leverage: f64,
    pub annualized_managed_vol: f64,
    pub annualized_unmanaged_vol: f64,
    pub n_observations: usize,
}

pub fn compute(
    returns: &[f64],
    lookback: usize,
    target_annualized_vol: f64,
    periods_per_year: f64,
    max_leverage: f64,
) -> Option<VolManagedReport> {
    let n = returns.len();
    if n < lookback + 1 || lookback < 5
        || !target_annualized_vol.is_finite() || target_annualized_vol <= 0.0
        || !periods_per_year.is_finite() || periods_per_year <= 0.0
        || !max_leverage.is_finite() || max_leverage <= 0.0 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) { return None; }
    let target_var_period = target_annualized_vol * target_annualized_vol / periods_per_year;
    let mut managed = vec![None; n];
    let mut leverages = vec![None; n];
    let mut cum = vec![None; n];
    let mut equity = 1.0_f64;
    let mut lev_sum = 0.0_f64;
    let mut lev_count = 0_usize;
    for i in lookback..n {
        let win = &returns[i - lookback..i];
        let n_f = lookback as f64;
        let mean: f64 = win.iter().sum::<f64>() / n_f;
        let var: f64 = win.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_f;
        let lev = if var > 0.0 {
            (target_var_period / var).min(max_leverage)
        } else { max_leverage };
        let managed_ret = lev * returns[i];
        equity *= 1.0 + managed_ret;
        managed[i] = Some(managed_ret);
        leverages[i] = Some(lev);
        cum[i] = Some(equity);
        lev_sum += lev;
        lev_count += 1;
    }
    let mean_lev = if lev_count > 0 { lev_sum / lev_count as f64 } else { 0.0 };
    let realized: Vec<f64> = managed.iter().filter_map(|x| *x).collect();
    let r_n = realized.len();
    let r_mean: f64 = realized.iter().sum::<f64>() / r_n as f64;
    let r_var: f64 = realized.iter().map(|r| (r - r_mean).powi(2)).sum::<f64>()
        / (r_n - 1).max(1) as f64;
    let managed_vol_ann = r_var.max(0.0).sqrt() * periods_per_year.sqrt();
    let unmanaged: Vec<f64> = returns[lookback..].to_vec();
    let u_mean: f64 = unmanaged.iter().sum::<f64>() / unmanaged.len() as f64;
    let u_var: f64 = unmanaged.iter().map(|r| (r - u_mean).powi(2)).sum::<f64>()
        / (unmanaged.len() - 1).max(1) as f64;
    let unmanaged_vol_ann = u_var.max(0.0).sqrt() * periods_per_year.sqrt();
    Some(VolManagedReport {
        managed_returns: managed,
        leverages,
        cumulative_equity: cum,
        mean_leverage: mean_lev,
        annualized_managed_vol: managed_vol_ann,
        annualized_unmanaged_vol: unmanaged_vol_ann,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let r = vec![0.01_f64; 50];
        assert!(compute(&r, 5, 0.0, 252.0, 4.0).is_none());
        assert!(compute(&r, 5, 0.15, 0.0, 4.0).is_none());
        assert!(compute(&r, 5, 0.15, 252.0, 0.0).is_none());
        assert!(compute(&r, 5, f64::NAN, 252.0, 4.0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 50];
        r[5] = f64::NAN;
        assert!(compute(&r, 20, 0.15, 252.0, 4.0).is_none());
    }

    #[test]
    fn flat_returns_yield_capped_leverage() {
        // Zero variance lookback → leverage at cap.
        let r = vec![0.01_f64; 50];
        let result = compute(&r, 20, 0.15, 252.0, 4.0).unwrap();
        // All leverages should be at max_leverage (= 4.0).
        for lev in result.leverages.iter().skip(20).flatten() {
            assert!((lev - 4.0).abs() < 1e-9);
        }
    }

    #[test]
    fn high_volatility_lookback_yields_low_leverage() {
        // High realized vol → leverage scales down.
        let mut state: u64 = 42;
        let r: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.10
        }).collect();
        let result = compute(&r, 60, 0.15, 252.0, 10.0).unwrap();
        // The realized vol is high → mean_leverage should be below cap.
        assert!(result.mean_leverage < 10.0);
    }

    #[test]
    fn managed_vol_close_to_target_when_uncapped() {
        // Stationary iid returns with σ ≈ 1% per period → realized vol
        // matches; managed should achieve target_vol approximately.
        let mut state: u64 = 11;
        let r: Vec<f64> = (0..1000).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            0.01 * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect();
        let target_vol = 0.10;
        let result = compute(&r, 60, target_vol, 252.0, 50.0).unwrap();
        // Should be in the ballpark of target.
        let rel = (result.annualized_managed_vol - target_vol).abs() / target_vol;
        assert!(rel < 0.50,
            "managed vol {} vs target {}, rel diff {}",
            result.annualized_managed_vol, target_vol, rel);
    }

    #[test]
    fn cumulative_equity_starts_around_one() {
        let r = vec![0.001_f64; 100];
        let result = compute(&r, 20, 0.15, 252.0, 4.0).unwrap();
        // First non-None should be near 1 + lev · 0.001.
        let first = result.cumulative_equity.iter().find_map(|x| *x).unwrap();
        assert!(first > 0.95 && first < 1.05);
    }

    #[test]
    fn n_observations_reported() {
        let r = vec![0.001_f64; 50];
        let result = compute(&r, 20, 0.15, 252.0, 4.0).unwrap();
        assert_eq!(result.n_observations, 50);
    }
}
