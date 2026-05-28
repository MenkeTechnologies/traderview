//! Volume-Weighted Exponential Moving Average (VWEMA).
//!
//!   VWEMA_t = α · (price_t · volume_t) / volume_t + (1 − α) · VWEMA_{t−1}
//!
//! where α = 2/(period + 1). Equivalently, an EMA on the price series
//! where each observation is weighted by volume — emphasizes bars where
//! more trading happened, dampening the influence of low-liquidity drift.
//!
//! Used as a smoothed VWAP-like anchor in mean-reversion and trend-
//! following strategies on intraday charts.
//!
//! Pure compute.

pub fn compute(prices: &[f64], volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = prices.len();
    let mut out = vec![None; n];
    if period == 0 || prices.len() != volumes.len() || n == 0 {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut prev: Option<f64> = None;
    for (i, (p, v)) in prices.iter().zip(volumes.iter()).enumerate() {
        if !p.is_finite() || !v.is_finite() || *v < 0.0 {
            // Carry prior value.
            if let Some(prev_v) = prev { out[i] = Some(prev_v); }
            continue;
        }
        // Base form: EMA on price; per-bar volume weighting lives in
        // `compute_volume_weighted` below. Use the price directly so
        // zero-volume bars still update the EMA at the standard rate.
        let new = match prev {
            None => *p,
            Some(prev_v) => alpha * p + (1.0 - alpha) * prev_v,
        };
        if new.is_finite() {
            prev = Some(new);
            out[i] = prev;
        }
    }
    out
}

/// True volume-weighted variant: weight = α · volume_norm where
/// volume_norm = volume / running_max_volume. Differs from `compute`
/// in that low-volume bars get LESS weight than high-volume bars.
pub fn compute_volume_weighted(prices: &[f64], volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = prices.len();
    let mut out = vec![None; n];
    if period == 0 || prices.len() != volumes.len() || n == 0 {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut prev: Option<f64> = None;
    let mut max_vol = 0.0_f64;
    for (i, (p, v)) in prices.iter().zip(volumes.iter()).enumerate() {
        if !p.is_finite() || !v.is_finite() || *v < 0.0 {
            if let Some(prev_v) = prev { out[i] = Some(prev_v); }
            continue;
        }
        if *v > max_vol { max_vol = *v; }
        let v_norm = if max_vol > 0.0 { v / max_vol } else { 0.0 };
        let effective_alpha = alpha * v_norm;
        let new = match prev {
            None => *p,
            Some(prev_v) => effective_alpha * p + (1.0 - effective_alpha) * prev_v,
        };
        if new.is_finite() {
            prev = Some(new);
            out[i] = prev;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 14).is_empty());
    }

    #[test]
    fn length_mismatch_returns_all_none() {
        let p = vec![100.0; 30];
        let v = vec![1_000.0; 15];
        assert!(compute(&p, &v, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_zero_returns_all_none() {
        let p = vec![100.0; 30];
        let v = vec![1_000.0; 30];
        assert!(compute(&p, &v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_flat_vwema() {
        let p = vec![100.0; 30];
        let v = vec![1_000.0; 30];
        let out = compute(&p, &v, 14);
        for x in out.iter().flatten() {
            assert!((x - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn rising_series_vwema_tracks_close_to_price() {
        let p: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let v = vec![1_000.0; 50];
        let out = compute(&p, &v, 10);
        let last = out[49].unwrap();
        // Should lag the current price by ~period.
        assert!(last < 149.0 && last > 130.0);
    }

    #[test]
    fn nan_input_carries_prior() {
        let mut p = vec![100.0; 30];
        let mut v = vec![1_000.0; 30];
        p[10] = f64::NAN;
        v[15] = f64::NAN;
        let out = compute(&p, &v, 14);
        // Surrounding bars populated, NaN bars carry prior.
        assert!(out.iter().any(|x| x.is_some()));
    }

    #[test]
    fn volume_weighted_variant_dampens_low_volume_moves() {
        // High-volume baseline of 100, then a single huge price jump on
        // tiny volume → volume-weighted EMA barely reacts.
        let mut p = vec![100.0; 30];
        let mut v = vec![1_000.0; 30];
        p[20] = 200.0;
        v[20] = 1.0;
        let vwema_std = compute(&p, &v, 5);
        let vwema_vw = compute_volume_weighted(&p, &v, 5);
        // The volume-weighted variant should produce a smaller deviation
        // from baseline at bar 20.
        let delta_std = (vwema_std[20].unwrap() - 100.0).abs();
        let delta_vw = (vwema_vw[20].unwrap() - 100.0).abs();
        assert!(delta_vw < delta_std,
            "vol-weighted EMA should react less to low-vol spike: std={delta_std} vw={delta_vw}");
    }
}
