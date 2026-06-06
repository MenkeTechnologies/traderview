//! Credit Default Swap (CDS) — par spread and PV-of-existing-position
//! under the JPMorgan/ISDA standard model assumptions.
//!
//! Inputs:
//!   - Per-tenor (time, hazard_rate) lambda curve — typically piecewise
//!     constant between coupon dates.
//!   - Per-tenor (time, discount_factor) curve.
//!   - Recovery rate R (typically 0.40).
//!   - Coupon payment schedule (times in years from valuation date).
//!
//! Protection leg PV (per unit notional):
//!   prot = (1 − R) · Σ_i ∫_{t_{i−1}}^{t_i} P(0,t) · λ(t) · e^{−Λ(0,t)} dt
//!
//! Premium leg PV (per unit notional, per 1bp running spread):
//!   prem_per_bp = Σ_i τ_i · P(0,t_i) · e^{−Λ(0,t_i)} · 0.0001
//!
//! Par spread = (prot / prem_per_bp) (in basis points)
//!
//! For analytical tractability we approximate the protection-leg integral
//! with the standard "midpoint" rule: protection over [t_{i−1}, t_i]
//! is approximated by survival_to_midpoint · prob_default_in_period · DF_to_midpoint.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CurvePoint {
    pub time_years: f64,
    /// Discount factor for time_years (typically from OIS curve).
    pub discount_factor: f64,
    /// Annualized hazard rate at this knot.
    pub hazard_rate: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CouponDate {
    pub time_years: f64,
    /// Accrual period (year-fraction) from prior coupon date.
    pub accrual: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CdsReport {
    pub par_spread_bps: f64,
    pub protection_leg_pv: f64,
    pub premium_leg_pv01: f64,
    pub fair_pv_existing: f64,
}

pub fn analyze(
    knots: &[CurvePoint],
    coupons: &[CouponDate],
    recovery_rate: f64,
    notional: f64,
    existing_spread_bps: Option<f64>,
) -> Option<CdsReport> {
    if knots.is_empty()
        || coupons.is_empty()
        || !recovery_rate.is_finite()
        || !(0.0..1.0).contains(&recovery_rate)
        || !notional.is_finite()
        || notional == 0.0
    {
        return None;
    }
    if knots.iter().any(|k| {
        !k.time_years.is_finite()
            || k.time_years <= 0.0
            || !k.discount_factor.is_finite()
            || k.discount_factor <= 0.0
            || k.discount_factor > 1.0 + 1e-9
            || !k.hazard_rate.is_finite()
            || k.hazard_rate < 0.0
    }) {
        return None;
    }
    if coupons.iter().any(|c| {
        !c.time_years.is_finite()
            || c.time_years <= 0.0
            || !c.accrual.is_finite()
            || c.accrual <= 0.0
    }) {
        return None;
    }
    // Interpolate hazard λ(t) and discount P(0,t) piecewise-linearly.
    let mut sorted: Vec<CurvePoint> = knots.to_vec();
    sorted.sort_by(|a, b| {
        a.time_years
            .partial_cmp(&b.time_years)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // Build the survival probability path S(t) = exp(-Λ(0,t)).
    // For each coupon period we accumulate Λ via cumulative integral over
    // sorted hazard knots.
    let mut prot_leg_pv = 0.0_f64;
    let mut prem_leg_pv01 = 0.0_f64;
    let mut prev_t = 0.0_f64;
    let mut prev_survival = 1.0_f64;
    for c in coupons {
        if c.time_years <= prev_t {
            continue;
        }
        let lambda_to_t = integrate_hazard(&sorted, prev_t, c.time_years);
        let survival_to_t = (-cumulative_hazard(&sorted, c.time_years)).exp();
        if !survival_to_t.is_finite() {
            return None;
        }
        let prob_default_in_period = (prev_survival - survival_to_t).max(0.0);
        // Discount to the midpoint of the period for protection leg.
        let mid_t = 0.5 * (prev_t + c.time_years);
        let df_mid = discount_at(&sorted, mid_t);
        let df_end = discount_at(&sorted, c.time_years);
        if !df_mid.is_finite() || !df_end.is_finite() {
            return None;
        }
        prot_leg_pv += (1.0 - recovery_rate) * df_mid * prob_default_in_period;
        prem_leg_pv01 += c.accrual * df_end * survival_to_t * 0.0001;
        prev_t = c.time_years;
        prev_survival = survival_to_t;
        let _ = lambda_to_t;
    }
    if prem_leg_pv01 <= 0.0 {
        return None;
    }
    let par_spread_bps = prot_leg_pv / prem_leg_pv01;
    // PV of an EXISTING swap entered at running_spread S₀ is:
    //   (par − S₀) · prem_pv01 · notional  [for protection BUYER]
    let pv_existing = if let Some(s0) = existing_spread_bps {
        if !s0.is_finite() {
            return None;
        }
        (par_spread_bps - s0) * prem_leg_pv01 * notional
    } else {
        0.0
    };
    Some(CdsReport {
        par_spread_bps,
        protection_leg_pv: prot_leg_pv * notional,
        premium_leg_pv01: prem_leg_pv01 * notional,
        fair_pv_existing: pv_existing,
    })
}

fn integrate_hazard(curve: &[CurvePoint], t0: f64, t1: f64) -> f64 {
    // Trapezoid rule for ∫_{t0}^{t1} λ(s) ds with piecewise-linear λ.
    let lambda_t0 = interp_hazard(curve, t0);
    let lambda_t1 = interp_hazard(curve, t1);
    (lambda_t0 + lambda_t1) / 2.0 * (t1 - t0)
}

fn cumulative_hazard(curve: &[CurvePoint], t: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let mut prev_t = 0.0_f64;
    let mut prev_lambda = curve.first().map(|k| k.hazard_rate).unwrap_or(0.0);
    let mut sum = 0.0_f64;
    for k in curve {
        let cur_t = k.time_years.min(t);
        if cur_t <= prev_t {
            continue;
        }
        sum += 0.5 * (prev_lambda + k.hazard_rate) * (cur_t - prev_t);
        prev_t = cur_t;
        prev_lambda = k.hazard_rate;
        if k.time_years >= t {
            return sum;
        }
    }
    // Extrapolate beyond last knot with flat hazard.
    if t > prev_t {
        sum += prev_lambda * (t - prev_t);
    }
    sum
}

fn interp_hazard(curve: &[CurvePoint], t: f64) -> f64 {
    if curve.is_empty() {
        return 0.0;
    }
    if t <= curve[0].time_years {
        return curve[0].hazard_rate;
    }
    if t >= curve.last().unwrap().time_years {
        return curve.last().unwrap().hazard_rate;
    }
    for w in curve.windows(2) {
        if t >= w[0].time_years && t <= w[1].time_years {
            let frac = (t - w[0].time_years) / (w[1].time_years - w[0].time_years);
            return w[0].hazard_rate + frac * (w[1].hazard_rate - w[0].hazard_rate);
        }
    }
    curve.last().unwrap().hazard_rate
}

fn discount_at(curve: &[CurvePoint], t: f64) -> f64 {
    if curve.is_empty() {
        return 1.0;
    }
    if t <= 0.0 {
        return 1.0;
    }
    // Linear interp on log-DF (equivalently flat forward rate).
    if t <= curve[0].time_years {
        let r = -curve[0].discount_factor.ln() / curve[0].time_years;
        return (-r * t).exp();
    }
    if t >= curve.last().unwrap().time_years {
        let last = curve.last().unwrap();
        let r = -last.discount_factor.ln() / last.time_years;
        return (-r * t).exp();
    }
    for w in curve.windows(2) {
        if t >= w[0].time_years && t <= w[1].time_years {
            let log0 = w[0].discount_factor.ln();
            let log1 = w[1].discount_factor.ln();
            let frac = (t - w[0].time_years) / (w[1].time_years - w[0].time_years);
            return (log0 + frac * (log1 - log0)).exp();
        }
    }
    curve.last().unwrap().discount_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(t: f64, df: f64, lam: f64) -> CurvePoint {
        CurvePoint {
            time_years: t,
            discount_factor: df,
            hazard_rate: lam,
        }
    }
    fn c(t: f64, acc: f64) -> CouponDate {
        CouponDate {
            time_years: t,
            accrual: acc,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let knots = vec![k(1.0, 0.95, 0.01)];
        let coupons = vec![c(0.25, 0.25)];
        // Recovery rate out of range.
        assert!(analyze(&knots, &coupons, 1.5, 1_000_000.0, None).is_none());
        // Zero notional.
        assert!(analyze(&knots, &coupons, 0.40, 0.0, None).is_none());
        // Empty curves.
        assert!(analyze(&[], &coupons, 0.40, 1_000_000.0, None).is_none());
        assert!(analyze(&knots, &[], 0.40, 1_000_000.0, None).is_none());
    }

    #[test]
    fn par_spread_proportional_to_hazard_rate() {
        // Approximate result: par_spread ≈ (1 − R) · λ in bps · 10_000.
        // For λ = 1% and R = 40%, expect ≈ 60 bps.
        let knots = vec![k(5.0, 0.95, 0.01)];
        let coupons: Vec<CouponDate> = (1..=20).map(|i| c(i as f64 * 0.25, 0.25)).collect();
        let r = analyze(&knots, &coupons, 0.40, 1_000_000.0, None).unwrap();
        assert!(
            (r.par_spread_bps - 60.0).abs() < 15.0,
            "expected ~60bps, got {}",
            r.par_spread_bps
        );
    }

    #[test]
    fn higher_hazard_yields_wider_par_spread() {
        let coupons: Vec<CouponDate> = (1..=20).map(|i| c(i as f64 * 0.25, 0.25)).collect();
        let low = analyze(&[k(5.0, 0.95, 0.005)], &coupons, 0.40, 1_000_000.0, None).unwrap();
        let high = analyze(&[k(5.0, 0.95, 0.030)], &coupons, 0.40, 1_000_000.0, None).unwrap();
        assert!(high.par_spread_bps > low.par_spread_bps);
    }

    #[test]
    fn higher_recovery_lowers_par_spread() {
        let knots = vec![k(5.0, 0.95, 0.01)];
        let coupons: Vec<CouponDate> = (1..=20).map(|i| c(i as f64 * 0.25, 0.25)).collect();
        let low_r = analyze(&knots, &coupons, 0.10, 1_000_000.0, None).unwrap();
        let high_r = analyze(&knots, &coupons, 0.70, 1_000_000.0, None).unwrap();
        assert!(high_r.par_spread_bps < low_r.par_spread_bps);
    }

    #[test]
    fn existing_position_pv_zero_at_par() {
        let knots = vec![k(5.0, 0.95, 0.01)];
        let coupons: Vec<CouponDate> = (1..=20).map(|i| c(i as f64 * 0.25, 0.25)).collect();
        let par = analyze(&knots, &coupons, 0.40, 1_000_000.0, None)
            .unwrap()
            .par_spread_bps;
        let r = analyze(&knots, &coupons, 0.40, 1_000_000.0, Some(par)).unwrap();
        assert!(r.fair_pv_existing.abs() < 1.0);
    }

    #[test]
    fn protection_buyer_pv_positive_when_spreads_widen() {
        // Bought protection at 50bps; current par = 60bps → buyer wins.
        let knots = vec![k(5.0, 0.95, 0.01)];
        let coupons: Vec<CouponDate> = (1..=20).map(|i| c(i as f64 * 0.25, 0.25)).collect();
        let r = analyze(&knots, &coupons, 0.40, 1_000_000.0, Some(50.0)).unwrap();
        if r.par_spread_bps > 50.0 {
            assert!(r.fair_pv_existing > 0.0);
        }
    }

    #[test]
    fn premium_leg_increases_with_protection_term() {
        let knots = vec![k(10.0, 0.90, 0.01)];
        let short: Vec<CouponDate> = (1..=4).map(|i| c(i as f64 * 0.25, 0.25)).collect();
        let long: Vec<CouponDate> = (1..=40).map(|i| c(i as f64 * 0.25, 0.25)).collect();
        let r_short = analyze(&knots, &short, 0.40, 1_000_000.0, None).unwrap();
        let r_long = analyze(&knots, &long, 0.40, 1_000_000.0, None).unwrap();
        assert!(r_long.premium_leg_pv01 > r_short.premium_leg_pv01);
    }
}
