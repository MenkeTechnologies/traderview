//! American option pricing — Cox-Ross-Rubinstein (1979) binomial tree
//! with backward induction + early-exercise check at each node.
//!
//! Tree parameters:
//!   u = e^{σ·√Δt}     d = 1/u
//!   p = (e^{(r−q)·Δt} − d) / (u − d)
//!
//! At each node check max(continuation_value, intrinsic_exercise) —
//! the value is the larger of holding vs exercising right now. This
//! captures the early-exercise premium that pure Black-Scholes misses.
//!
//! Pure compute. n_steps = 200 is a typical accuracy/speed sweet spot
//! (~5-decimal-place vs continuous BS for European-equivalent inputs).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AmericanReport {
    pub price: f64,
    pub n_steps: usize,
    pub early_exercise_premium: f64,
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
) -> Option<AmericanReport> {
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
        || !(1..=5_000).contains(&n_steps)
    {
        return None;
    }
    let dt = time_to_expiry / n_steps as f64;
    let u = (sigma * dt.sqrt()).exp();
    let d = 1.0 / u;
    let disc = (-risk_free * dt).exp();
    let drift = ((risk_free - dividend_yield) * dt).exp();
    let p = (drift - d) / (u - d);
    if !(0.0..=1.0).contains(&p) || !p.is_finite() {
        return None; // unstable tree (rare on sane inputs)
    }
    let q = 1.0 - p;
    // Terminal payoffs at step n.
    let mut values = vec![0.0_f64; n_steps + 1];
    for (j, slot) in values.iter_mut().enumerate() {
        let s = spot * u.powi((n_steps as i32) - (j as i32) * 2);
        let intrinsic = match kind {
            OptionKind::Call => (s - strike).max(0.0),
            OptionKind::Put => (strike - s).max(0.0),
        };
        *slot = intrinsic;
    }
    // Backward induction.
    let mut also_european = values.clone();
    for step in (0..n_steps).rev() {
        for j in 0..=step {
            let s = spot * u.powi((step as i32) - (j as i32) * 2);
            let continuation = disc * (p * values[j] + q * values[j + 1]);
            let intrinsic = match kind {
                OptionKind::Call => (s - strike).max(0.0),
                OptionKind::Put => (strike - s).max(0.0),
            };
            values[j] = continuation.max(intrinsic);
            also_european[j] = disc * (p * also_european[j] + q * also_european[j + 1]);
        }
    }
    let american = values[0];
    let european = also_european[0];
    let premium = (american - european).max(0.0);
    Some(AmericanReport {
        price: american,
        n_steps,
        early_exercise_premium: premium,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 0.5, 0.05, 0.0, 0.2, 100, OptionKind::Call).is_none());
            assert!(price(100.0, bad, 0.5, 0.05, 0.0, 0.2, 100, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, bad, 0.05, 0.0, 0.2, 100, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, bad, 100, OptionKind::Call).is_none());
        }
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.2, 0, OptionKind::Call).is_none());
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.2, 6_000, OptionKind::Call).is_none());
    }

    #[test]
    fn american_call_on_non_dividend_paying_equals_european() {
        // Merton (1973): never optimal to exercise American CALL early on
        // a non-dividend-paying stock → equals European call. Early-
        // exercise premium should be effectively zero.
        let r = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.20, 200, OptionKind::Call).unwrap();
        assert!(
            r.early_exercise_premium < 0.01,
            "non-div American call should ~= European: premium={}",
            r.early_exercise_premium
        );
    }

    #[test]
    fn american_put_has_positive_early_exercise_premium() {
        // American puts ARE worth more than European puts (the canonical
        // counter to Merton's call result).
        let r = price(100.0, 100.0, 1.0, 0.05, 0.0, 0.20, 500, OptionKind::Put).unwrap();
        assert!(
            r.early_exercise_premium > 0.0,
            "American put should be > European put"
        );
    }

    #[test]
    fn deep_itm_american_call_with_dividend_has_premium() {
        // With dividend yield, even calls can be optimal to exercise
        // early; deep-ITM call before ex-div date is the textbook case.
        let r = price(150.0, 100.0, 1.0, 0.05, 0.06, 0.20, 500, OptionKind::Call).unwrap();
        assert!(
            r.early_exercise_premium > 0.0,
            "div-paying ITM call should have early-exercise value, got {}",
            r.early_exercise_premium
        );
    }

    #[test]
    fn deep_itm_american_put_approaches_intrinsic() {
        // At spot = 50 vs strike 100, intrinsic = 50. American put value
        // ≥ 50 (immediate exercise pays exactly that).
        let r = price(50.0, 100.0, 0.5, 0.05, 0.0, 0.20, 500, OptionKind::Put).unwrap();
        assert!(
            r.price >= 49.99,
            "deep ITM put should be ≥ intrinsic, got {}",
            r.price
        );
    }

    #[test]
    fn longer_expiry_inflates_atm_american_call() {
        let r_short = price(100.0, 100.0, 0.10, 0.05, 0.0, 0.20, 200, OptionKind::Call).unwrap();
        let r_long = price(100.0, 100.0, 1.00, 0.05, 0.0, 0.20, 200, OptionKind::Call).unwrap();
        assert!(r_long.price > r_short.price);
    }

    #[test]
    fn higher_vol_inflates_atm_american() {
        let r_low = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.10, 200, OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.40, 200, OptionKind::Call).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn convergence_with_more_steps() {
        // Successive doublings of n_steps should converge.
        let r_50 = price(100.0, 100.0, 1.0, 0.05, 0.02, 0.20, 50, OptionKind::Put).unwrap();
        let r_500 = price(100.0, 100.0, 1.0, 0.05, 0.02, 0.20, 500, OptionKind::Put).unwrap();
        assert!((r_50.price - r_500.price).abs() / r_500.price < 0.05);
    }
}
