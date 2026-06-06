//! Liquidity-Adjusted VaR (LVaR) — Bangia, Diebold, Schuermann,
//! Stroughair (1999).
//!
//!   LVaR(α) = VaR(α) + 0.5 · S · (1 + a·z_α)
//!
//! where:
//!   - VaR(α) is the standard parametric VaR (price risk)
//!   - S       = mean bid-ask spread (as a fraction of mid)
//!   - a       = stdev of bid-ask spread / mean spread (sample-derived)
//!   - z_α     = standard-normal α-quantile (one-sided, e.g. 1.645 at 5%)
//!
//! Reading: LVaR accounts for the EXPECTED cost of unwinding the
//! position (half the spread) PLUS a worst-case spread shock at the
//! same confidence level used for price risk. The 0.5 reflects that
//! the round-trip cost is half-spread per unwind side.
//!
//! Pure compute. Caller supplies pre-computed price VaR (e.g. from
//! `conditional_var::compute().var_parametric` or `cornish_fisher`),
//! the position notional, and the bid-ask spread time series.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LvarReport {
    pub price_var: f64,
    pub liquidity_cost: f64,
    pub liquidity_adjusted_var: f64,
    pub mean_spread: f64,
    pub stdev_spread: f64,
    pub spread_scaler: f64,
    pub n_observations: usize,
}

pub fn compute(
    price_var_fraction: f64,
    notional: f64,
    spreads_as_fraction_of_mid: &[f64],
    alpha: f64,
) -> Option<LvarReport> {
    if !price_var_fraction.is_finite()
        || price_var_fraction < 0.0
        || !notional.is_finite()
        || notional <= 0.0
        || !alpha.is_finite()
        || !(0.0..1.0).contains(&alpha)
        || alpha == 0.0
        || spreads_as_fraction_of_mid.is_empty()
    {
        return None;
    }
    let clean: Vec<f64> = spreads_as_fraction_of_mid
        .iter()
        .copied()
        .filter(|x| x.is_finite() && *x >= 0.0)
        .collect();
    let n = clean.len();
    if n < 2 {
        return None;
    }
    let n_f = n as f64;
    let mean = clean.iter().sum::<f64>() / n_f;
    let var = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let stdev = var.max(0.0).sqrt();
    let scaler = if mean > 0.0 { stdev / mean } else { 0.0 };
    let z = inv_norm_cdf(1.0 - alpha); // one-sided quantile of stress
    let liquidity_factor = 0.5 * mean * (1.0 + scaler * z);
    let liquidity_cost = liquidity_factor * notional;
    let price_var_dollar = price_var_fraction * notional;
    let lvar = price_var_dollar + liquidity_cost;
    if !lvar.is_finite() {
        return None;
    }
    Some(LvarReport {
        price_var: price_var_dollar,
        liquidity_cost,
        liquidity_adjusted_var: lvar,
        mean_spread: mean,
        stdev_spread: stdev,
        spread_scaler: scaler,
        n_observations: n,
    })
}

fn inv_norm_cdf(p: f64) -> f64 {
    if !(0.0..=1.0).contains(&p) || !p.is_finite() {
        return f64::NAN;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }
    let plow = 0.02425;
    let phigh = 1.0 - plow;
    let a = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_153_46,
    ];
    let b = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    let c = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    let d = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= phigh {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(-0.01, 1_000_000.0, &[0.001; 10], 0.05).is_none());
        assert!(compute(0.05, 0.0, &[0.001; 10], 0.05).is_none());
        assert!(compute(0.05, 1_000_000.0, &[0.001; 10], 0.0).is_none());
        assert!(compute(0.05, 1_000_000.0, &[0.001; 10], 1.0).is_none());
        assert!(compute(0.05, 1_000_000.0, &[], 0.05).is_none());
    }

    #[test]
    fn nan_spreads_filtered() {
        let r = compute(0.05, 1_000_000.0, &[0.001, f64::NAN, 0.001, 0.002], 0.05).unwrap();
        assert_eq!(r.n_observations, 3);
    }

    #[test]
    fn zero_spread_yields_only_price_var() {
        let r = compute(0.05, 1_000_000.0, &[0.0; 10], 0.05).unwrap();
        assert!((r.liquidity_cost).abs() < 1e-9);
        assert!((r.liquidity_adjusted_var - r.price_var).abs() < 1e-9);
    }

    #[test]
    fn lvar_strictly_greater_than_price_var_for_positive_spread() {
        let r = compute(0.05, 1_000_000.0, &[0.001; 10], 0.05).unwrap();
        assert!(r.liquidity_adjusted_var > r.price_var);
    }

    #[test]
    fn constant_spread_yields_zero_scaler() {
        // 10 identical spreads → stdev/mean = 0 → liquidity factor = 0.5·S.
        let r = compute(0.05, 1_000_000.0, &[0.002; 10], 0.05).unwrap();
        assert!(r.spread_scaler.abs() < 1e-9);
        let expected = 0.5 * 0.002 * 1_000_000.0;
        assert!((r.liquidity_cost - expected).abs() < 1e-6);
    }

    #[test]
    fn higher_alpha_quantile_inflates_lvar() {
        // alpha=0.01 → z = 2.326 (extreme tail) → bigger lvar than alpha=0.05.
        // Test with varying spread so scaler > 0.
        let spreads: Vec<f64> = (0..100).map(|i| 0.001 + (i as f64 * 0.0001)).collect();
        let r_05 = compute(0.05, 1_000_000.0, &spreads, 0.05).unwrap();
        let r_01 = compute(0.05, 1_000_000.0, &spreads, 0.01).unwrap();
        assert!(r_01.liquidity_adjusted_var > r_05.liquidity_adjusted_var);
    }

    #[test]
    fn larger_notional_scales_lvar_proportionally() {
        let r_1m = compute(0.05, 1_000_000.0, &[0.001; 10], 0.05).unwrap();
        let r_10m = compute(0.05, 10_000_000.0, &[0.001; 10], 0.05).unwrap();
        // 10× notional → 10× LVaR.
        assert!((r_10m.liquidity_adjusted_var - 10.0 * r_1m.liquidity_adjusted_var).abs() < 1e-6);
    }
}
