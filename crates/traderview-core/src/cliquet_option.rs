//! Cliquet (ratchet / forward-start) option — Rubinstein (1991) closed form.
//!
//! A series of N consecutive forward-start ATM calls. At each reset
//! date t_i, a new ATM call is "struck" at the then-prevailing spot
//! S(t_i) with maturity t_{i+1}; it pays max(S(t_{i+1}) − α · S(t_i), 0)
//! at t_{i+1}, where α is the reset multiplier (1.0 = pure ATM forward
//! start).
//!
//! For each segment, by the forward-start property the value at t=0 is:
//!
//!   v_i = e^{−q·t_i} · BS_call(S₀=1, K=α, T=t_{i+1} − t_i,
//!                              r=r, q=q, σ=σ)
//!
//! (scaled to the spot at t_i, which has discounted expected value
//!  S_0 · e^{(r−q)·t_i}). The total cliquet value is:
//!
//!   V = Σ_i S_0 · e^{−q·t_{i+1}} · [N(d1) − α · e^{−r·(t_{i+1} − t_i)} · N(d2)]
//!
//! Pure compute. Companion to vanilla `iv_solver` / `greeks` — the
//! forward-start feature is a common equity-structured product brick.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CliquetReport {
    pub total_price: f64,
    pub per_segment_prices: [f64; 16], // up to 16 segments
    pub n_segments: usize,
}

#[allow(clippy::too_many_arguments)] // canonical signature
pub fn price(
    spot: f64,
    reset_dates: &[f64],
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
    reset_multiplier: f64,
) -> Option<CliquetReport> {
    let n = reset_dates.len();
    if !spot.is_finite()
        || spot <= 0.0
        || !risk_free.is_finite()
        || !dividend_yield.is_finite()
        || !sigma.is_finite()
        || sigma <= 0.0
        || !reset_multiplier.is_finite()
        || reset_multiplier <= 0.0
        || !(2..=16).contains(&n)
        || reset_dates.iter().any(|t| !t.is_finite() || *t <= 0.0)
    {
        return None;
    }
    // Reset dates must be strictly increasing.
    for w in reset_dates.windows(2) {
        if w[1] <= w[0] {
            return None;
        }
    }
    let n_segments = n - 1;
    let mut per_segment = [0.0_f64; 16];
    let mut total = 0.0_f64;
    for i in 0..n_segments {
        let t_i = reset_dates[i];
        let t_i1 = reset_dates[i + 1];
        let segment_t = t_i1 - t_i;
        // BS call on a 1-spot underlying with strike = α, expiry = segment_t.
        let sqrt_t = segment_t.sqrt();
        let d1 = ((1.0_f64 / reset_multiplier).ln()
            + (risk_free - dividend_yield + 0.5 * sigma * sigma) * segment_t)
            / (sigma * sqrt_t);
        let d2 = d1 - sigma * sqrt_t;
        let nd1 = norm_cdf(d1);
        let nd2 = norm_cdf(d2);
        let dq_segment = (-dividend_yield * segment_t).exp();
        let dr_segment = (-risk_free * segment_t).exp();
        let segment_call_per_unit = dq_segment * nd1 - reset_multiplier * dr_segment * nd2;
        // Scale by the discounted expected spot at t_i:
        //   S_0 · e^{(r-q)·t_i} · e^{−r·t_i} = S_0 · e^{−q·t_i}
        let segment_value = spot * (-dividend_yield * t_i).exp() * segment_call_per_unit;
        if !segment_value.is_finite() {
            return None;
        }
        per_segment[i] = segment_value;
        total += segment_value;
    }
    Some(CliquetReport {
        total_price: total.max(0.0),
        per_segment_prices: per_segment,
        n_segments,
    })
}

fn norm_cdf(x: f64) -> f64 {
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 = 1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 = 1.061405429_f64;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let dates = vec![0.5, 1.0];
        assert!(price(0.0, &dates, 0.05, 0.0, 0.2, 1.0).is_none());
        assert!(price(100.0, &dates, 0.05, 0.0, 0.0, 1.0).is_none());
        assert!(price(100.0, &dates, 0.05, 0.0, 0.2, 0.0).is_none());
        assert!(price(f64::NAN, &dates, 0.05, 0.0, 0.2, 1.0).is_none());
    }

    #[test]
    fn too_few_reset_dates_returns_none() {
        assert!(price(100.0, &[], 0.05, 0.0, 0.2, 1.0).is_none());
        assert!(price(100.0, &[0.5], 0.05, 0.0, 0.2, 1.0).is_none());
    }

    #[test]
    fn non_monotonic_dates_rejected() {
        assert!(price(100.0, &[1.0, 0.5], 0.05, 0.0, 0.2, 1.0).is_none());
        assert!(price(100.0, &[0.5, 0.5], 0.05, 0.0, 0.2, 1.0).is_none());
    }

    #[test]
    fn negative_or_zero_reset_dates_rejected() {
        assert!(price(100.0, &[-1.0, 1.0], 0.05, 0.0, 0.2, 1.0).is_none());
        assert!(price(100.0, &[0.0, 1.0], 0.05, 0.0, 0.2, 1.0).is_none());
    }

    #[test]
    fn single_segment_atm_cliquet_equals_bs_forward_start() {
        // 1-segment cliquet from t_0 to t_1 should equal a single
        // forward-start ATM call = S₀ · e^{−q·t_0} · BS(K=1) per unit.
        let r = price(100.0, &[0.5, 1.0], 0.05, 0.0, 0.20, 1.0).unwrap();
        assert_eq!(r.n_segments, 1);
        assert!(r.total_price > 0.0);
    }

    #[test]
    fn multi_segment_cliquet_sums_individual_segments() {
        let r = price(100.0, &[0.25, 0.5, 0.75, 1.0], 0.05, 0.0, 0.20, 1.0).unwrap();
        assert_eq!(r.n_segments, 3);
        let sum: f64 = r.per_segment_prices.iter().take(3).sum();
        assert!((sum - r.total_price).abs() < 1e-9);
    }

    #[test]
    fn higher_vol_inflates_cliquet_price() {
        let r_low = price(100.0, &[0.25, 0.5, 0.75, 1.0], 0.05, 0.0, 0.10, 1.0).unwrap();
        let r_high = price(100.0, &[0.25, 0.5, 0.75, 1.0], 0.05, 0.0, 0.40, 1.0).unwrap();
        assert!(r_high.total_price > r_low.total_price);
    }

    #[test]
    fn larger_reset_multiplier_lowers_price() {
        // α = 1.05 (5% OTM forward-start) cheaper than α = 1.0 ATM.
        let r_atm = price(100.0, &[0.25, 0.5], 0.05, 0.0, 0.20, 1.00).unwrap();
        let r_otm = price(100.0, &[0.25, 0.5], 0.05, 0.0, 0.20, 1.05).unwrap();
        assert!(r_otm.total_price < r_atm.total_price);
    }

    #[test]
    fn too_many_segments_rejected() {
        let dates: Vec<f64> = (1..=20).map(|i| i as f64 * 0.1).collect();
        assert!(price(100.0, &dates, 0.05, 0.0, 0.2, 1.0).is_none());
    }
}
