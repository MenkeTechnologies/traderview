//! Vanilla Interest-Rate Swap Valuation (fixed-vs-float, plain vanilla).
//!
//! Fixed leg PV:
//!   PV_fix = Σ_i fixed_rate · τ_i · N · D(t_i)
//!
//! Floating leg PV (par-floater identity):
//!   PV_float = N · (D(t_0) − D(t_n))
//!
//! where:
//!   - τ_i is the day-count fraction between fixing dates
//!   - N is the notional
//!   - D(t) is the discount factor at time t (continuous compounding)
//!   - t_0 is the next reset date (typically "now" for an unsettled
//!     swap with no current floating accrual)
//!
//! Swap value to the fixed-rate PAYER:
//!   V_pay_fixed = PV_float − PV_fix
//!
//! Par swap rate (the fixed rate s* that makes V = 0 at inception):
//!   s* = (D(t_0) − D(t_n)) / Σ_i τ_i · D(t_i)
//!
//! Pure compute. Discount curve as piecewise-linear spot curve (cont.
//! compounded). Companion to `yield_curve_bootstrap`, `nelson_siegel`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpotPoint {
    pub time_years: f64,
    pub spot_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwapValuationReport {
    pub fixed_leg_pv: f64,
    pub floating_leg_pv: f64,
    pub net_value_pay_fixed: f64,
    pub par_swap_rate: f64,
    pub annuity_factor: f64,
}

pub fn value(
    notional: f64,
    fixed_rate: f64,
    schedule_times: &[f64],
    next_reset_time: f64,
    curve: &[SpotPoint],
) -> Option<SwapValuationReport> {
    if notional <= 0.0 || !notional.is_finite()
        || !fixed_rate.is_finite() || !next_reset_time.is_finite()
        || next_reset_time < 0.0
        || schedule_times.is_empty() || curve.len() < 2 {
        return None;
    }
    if schedule_times.iter().any(|t| !t.is_finite() || *t < next_reset_time) {
        return None;
    }
    for w in schedule_times.windows(2) {
        if w[1] <= w[0] { return None; }
    }
    if curve.iter().any(|s| !s.time_years.is_finite() || !s.spot_rate.is_finite()
        || s.time_years < 0.0) { return None; }
    for w in curve.windows(2) {
        if w[1].time_years <= w[0].time_years { return None; }
    }
    // Day-count fractions: prefix with next_reset_time.
    let mut prev = next_reset_time;
    let mut taus = Vec::with_capacity(schedule_times.len());
    for t in schedule_times {
        taus.push(t - prev);
        prev = *t;
    }
    // Fixed leg PV.
    let mut fixed_pv = 0.0_f64;
    let mut annuity = 0.0_f64;
    for (t, tau) in schedule_times.iter().zip(taus.iter()) {
        let r = interp_rate(*t, curve)?;
        let d = (-r * t).exp();
        fixed_pv += fixed_rate * tau * notional * d;
        annuity += tau * d;
    }
    // Float leg PV via the par-floater identity.
    let r0 = interp_rate(next_reset_time, curve)?;
    let d0 = (-r0 * next_reset_time).exp();
    let t_end = *schedule_times.last().unwrap();
    let r_end = interp_rate(t_end, curve)?;
    let d_end = (-r_end * t_end).exp();
    let float_pv = notional * (d0 - d_end);
    let net = float_pv - fixed_pv;
    let par_rate = if annuity > 0.0 { (d0 - d_end) / annuity } else { f64::NAN };
    Some(SwapValuationReport {
        fixed_leg_pv: fixed_pv,
        floating_leg_pv: float_pv,
        net_value_pay_fixed: net,
        par_swap_rate: par_rate,
        annuity_factor: annuity,
    })
}

fn interp_rate(t: f64, curve: &[SpotPoint]) -> Option<f64> {
    if curve.is_empty() { return None; }
    if t <= curve[0].time_years { return Some(curve[0].spot_rate); }
    if t >= curve[curve.len() - 1].time_years {
        return Some(curve[curve.len() - 1].spot_rate);
    }
    for w in curve.windows(2) {
        if t >= w[0].time_years && t <= w[1].time_years {
            let span = w[1].time_years - w[0].time_years;
            if span <= 0.0 { return Some(w[0].spot_rate); }
            let frac = (t - w[0].time_years) / span;
            return Some(w[0].spot_rate + frac * (w[1].spot_rate - w[0].spot_rate));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(rate: f64) -> Vec<SpotPoint> {
        vec![
            SpotPoint { time_years: 0.5, spot_rate: rate },
            SpotPoint { time_years: 2.0, spot_rate: rate },
            SpotPoint { time_years: 5.0, spot_rate: rate },
        ]
    }

    fn semi_annual_schedule(years: f64) -> Vec<f64> {
        let n = (years * 2.0).round() as usize;
        (1..=n).map(|i| i as f64 * 0.5).collect()
    }

    #[test]
    fn invalid_inputs_return_none() {
        let curve = flat(0.05);
        let sched = semi_annual_schedule(5.0);
        assert!(value(0.0, 0.05, &sched, 0.0, &curve).is_none());
        assert!(value(1_000_000.0, f64::NAN, &sched, 0.0, &curve).is_none());
        assert!(value(1_000_000.0, 0.05, &[], 0.0, &curve).is_none());
        assert!(value(1_000_000.0, 0.05, &sched, 0.0, &[]).is_none());
    }

    #[test]
    fn non_monotonic_schedule_rejected() {
        let curve = flat(0.05);
        let bad = vec![1.0, 0.5];
        assert!(value(1_000_000.0, 0.05, &bad, 0.0, &curve).is_none());
    }

    #[test]
    fn par_rate_equals_curve_rate_for_flat_curve() {
        // 5y semi-annual swap on flat 5% curve → par swap rate ≈ 5%.
        // (Continuous compounding so exact par = (1 - exp(-r·T)) / annuity.)
        let curve = flat(0.05);
        let sched = semi_annual_schedule(5.0);
        let r = value(1_000_000.0, 0.05, &sched, 0.0, &curve).unwrap();
        // For flat continuously-compounded curve, par_rate ≈ flat rate to ~1%.
        assert!((r.par_swap_rate - 0.05).abs() < 0.005,
            "par rate {} vs flat 5%", r.par_swap_rate);
    }

    #[test]
    fn pay_par_swap_yields_near_zero_value() {
        // Enter as fixed-payer at the par rate → swap value should be ~0.
        let curve = flat(0.05);
        let sched = semi_annual_schedule(5.0);
        let r_temp = value(1_000_000.0, 0.05, &sched, 0.0, &curve).unwrap();
        let par_rate = r_temp.par_swap_rate;
        let r = value(1_000_000.0, par_rate, &sched, 0.0, &curve).unwrap();
        assert!(r.net_value_pay_fixed.abs() < 1.0,
            "at par rate, net value {} should be ~0", r.net_value_pay_fixed);
    }

    #[test]
    fn pay_above_par_yields_negative_value() {
        // Paying above-par fixed → losing money.
        let curve = flat(0.05);
        let sched = semi_annual_schedule(5.0);
        let r = value(1_000_000.0, 0.10, &sched, 0.0, &curve).unwrap();
        assert!(r.net_value_pay_fixed < 0.0,
            "pay 10% on 5% curve should be negative, got {}", r.net_value_pay_fixed);
    }

    #[test]
    fn pay_below_par_yields_positive_value() {
        let curve = flat(0.05);
        let sched = semi_annual_schedule(5.0);
        let r = value(1_000_000.0, 0.01, &sched, 0.0, &curve).unwrap();
        assert!(r.net_value_pay_fixed > 0.0,
            "pay 1% on 5% curve should be positive, got {}", r.net_value_pay_fixed);
    }

    #[test]
    fn fixed_leg_pv_positive() {
        let curve = flat(0.05);
        let sched = semi_annual_schedule(5.0);
        let r = value(1_000_000.0, 0.05, &sched, 0.0, &curve).unwrap();
        assert!(r.fixed_leg_pv > 0.0);
        assert!(r.floating_leg_pv > 0.0);
    }
}
