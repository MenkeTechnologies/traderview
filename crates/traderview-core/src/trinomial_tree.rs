//! Trinomial-tree American option pricer — Boyle (1986).
//!
//! Each node branches to three children: up, middle, down.
//!   u = e^{σ·√(2·Δt)}     d = 1/u     m = 1
//!   p_u = ((e^{(r−q)·Δt/2} − e^{−σ·√(Δt/2)}) /
//!          (e^{σ·√(Δt/2)} − e^{−σ·√(Δt/2)}))²
//!   p_d = ((e^{σ·√(Δt/2)} − e^{(r−q)·Δt/2}) /
//!          (e^{σ·√(Δt/2)} − e^{−σ·√(Δt/2)}))²
//!   p_m = 1 − p_u − p_d
//!
//! Trinomial converges faster than binomial for smooth payoffs and is
//! the canonical choice for American + barrier options where the
//! middle branch lets a node "stay" at the barrier (improving the
//! discretized boundary).
//!
//! Pure compute. Companion to `american_binomial` — trinomial yields
//! the same answer at typically half the steps.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrinomialReport {
    pub price: f64,
    pub n_steps: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
    n_steps: usize,
    kind: OptionKind,
    american: bool,
) -> Option<TrinomialReport> {
    if !spot.is_finite()
        || spot <= 0.0
        || !strike.is_finite()
        || strike <= 0.0
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || !risk_free.is_finite()
        || !dividend_yield.is_finite()
        || !sigma.is_finite()
        || sigma <= 0.0
        || !(1..=2_000).contains(&n_steps)
    {
        return None;
    }
    let dt = time_to_expiry / n_steps as f64;
    let u = (sigma * (2.0 * dt).sqrt()).exp();
    // d = 1/u (implicit — node prices use u^(step-2j) which handles both
    // directions). Half-step is the canonical Boyle discretization.
    let half_dt = dt / 2.0;
    let exp_drift = ((risk_free - dividend_yield) * half_dt).exp();
    let exp_up = (sigma * half_dt.sqrt()).exp();
    let exp_dn = (-sigma * half_dt.sqrt()).exp();
    let p_u = ((exp_drift - exp_dn) / (exp_up - exp_dn)).powi(2);
    let p_d = ((exp_up - exp_drift) / (exp_up - exp_dn)).powi(2);
    let p_m = 1.0 - p_u - p_d;
    if !(0.0..=1.0).contains(&p_u) || !(0.0..=1.0).contains(&p_d) || !(0.0..=1.0).contains(&p_m) {
        return None;
    }
    let disc = (-risk_free * dt).exp();
    // Index convention: at step k, there are 2k+1 nodes, indexed j = 0..=2k.
    // The spot at (step k, node j) is S0 · u^(k − j).
    let size = 2 * n_steps + 1;
    let mut values = vec![0.0_f64; size];
    // Terminal payoffs.
    for (j, slot) in values.iter_mut().enumerate() {
        let s = spot * u.powi((n_steps as i32) - (j as i32));
        *slot = match kind {
            OptionKind::Call => (s - strike).max(0.0),
            OptionKind::Put => (strike - s).max(0.0),
        };
    }
    // Backward induction.
    for step in (0..n_steps).rev() {
        let node_count = 2 * step + 1;
        for j in 0..node_count {
            let continuation = disc * (p_u * values[j] + p_m * values[j + 1] + p_d * values[j + 2]);
            let s = spot * u.powi((step as i32) - (j as i32));
            values[j] = if american {
                let intrinsic = match kind {
                    OptionKind::Call => (s - strike).max(0.0),
                    OptionKind::Put => (strike - s).max(0.0),
                };
                continuation.max(intrinsic)
            } else {
                continuation
            };
        }
    }
    Some(TrinomialReport {
        price: values[0],
        n_steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 0.5, 0.05, 0.0, 0.2, 100, OptionKind::Call, true).is_none());
            assert!(price(100.0, bad, 0.5, 0.05, 0.0, 0.2, 100, OptionKind::Call, true).is_none());
            assert!(price(
                100.0,
                100.0,
                bad,
                0.05,
                0.0,
                0.2,
                100,
                OptionKind::Call,
                true
            )
            .is_none());
            assert!(price(
                100.0,
                100.0,
                0.5,
                0.05,
                0.0,
                bad,
                100,
                OptionKind::Call,
                true
            )
            .is_none());
        }
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.2, 0, OptionKind::Call, true).is_none());
    }

    #[test]
    fn european_call_close_to_black_scholes() {
        // European trinomial should match Black-Scholes price closely.
        let r = price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.0,
            0.20,
            200,
            OptionKind::Call,
            false,
        )
        .unwrap();
        // BS ATM call ≈ 10.45.
        assert!((r.price - 10.45).abs() < 0.5);
    }

    #[test]
    fn american_put_above_european_put() {
        let r_amer = price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.0,
            0.20,
            200,
            OptionKind::Put,
            true,
        )
        .unwrap();
        let r_euro = price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.0,
            0.20,
            200,
            OptionKind::Put,
            false,
        )
        .unwrap();
        assert!(r_amer.price >= r_euro.price);
    }

    #[test]
    fn american_call_on_non_div_equals_european_to_within_steps() {
        // No early exercise optimal for non-dividend American call.
        let r_amer = price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.0,
            0.20,
            200,
            OptionKind::Call,
            true,
        )
        .unwrap();
        let r_euro = price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.0,
            0.20,
            200,
            OptionKind::Call,
            false,
        )
        .unwrap();
        assert!((r_amer.price - r_euro.price).abs() < 0.01);
    }

    #[test]
    fn deep_itm_american_put_intrinsic_floor() {
        // Spot 50 vs strike 100 → intrinsic 50; American can always exercise.
        let r = price(
            50.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.20,
            200,
            OptionKind::Put,
            true,
        )
        .unwrap();
        assert!(r.price >= 49.99);
    }

    #[test]
    fn convergence_with_more_steps() {
        let r_50 = price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.02,
            0.25,
            50,
            OptionKind::Put,
            true,
        )
        .unwrap();
        let r_500 = price(
            100.0,
            100.0,
            1.0,
            0.05,
            0.02,
            0.25,
            500,
            OptionKind::Put,
            true,
        )
        .unwrap();
        assert!((r_50.price - r_500.price).abs() / r_500.price < 0.03);
    }

    #[test]
    fn higher_vol_inflates_atm_european() {
        let r_low = price(
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.10,
            100,
            OptionKind::Call,
            false,
        )
        .unwrap();
        let r_high = price(
            100.0,
            100.0,
            0.5,
            0.05,
            0.0,
            0.40,
            100,
            OptionKind::Call,
            false,
        )
        .unwrap();
        assert!(r_high.price > r_low.price);
    }
}
