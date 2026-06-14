//! Callable-bond option-adjusted spread (OAS) — values a callable bond on a
//! recombining binomial short-rate lattice and solves for the constant spread
//! (the OAS) that makes the model price equal the market price. The lattice
//! values the bond by backward induction under risk-neutral probability 0.5; at
//! and after the lockout the issuer calls when continuing to hold is worth more
//! than the call price, capping the bond's value. The straight (option-free) value
//! minus the callable value is the embedded call's cost. Pure compute; a faithful
//! single-factor model, not a calibrated production curve. Not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CallableOasInput {
    #[serde(default = "default_face")]
    pub face_value: f64,
    pub coupon_rate_pct: f64,
    pub maturity_years: f64,
    #[serde(default = "default_steps")]
    pub steps: u32,
    /// Short rate at the root, percent.
    pub short_rate_pct: f64,
    /// Short-rate volatility, percent.
    pub rate_vol_pct: f64,
    #[serde(default = "default_face")]
    pub call_price: f64,
    /// Years before the bond becomes callable (call protection).
    #[serde(default)]
    pub lockout_years: f64,
    /// Observed market price (per 100 face) to solve the OAS against.
    pub market_price: f64,
}

fn default_face() -> f64 {
    100.0
}

fn default_steps() -> u32 {
    5
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct CallableOasReport {
    pub straight_price: f64,
    pub callable_price: f64,
    /// Straight − callable: value of the issuer's embedded call.
    pub option_cost: f64,
    /// Option-adjusted spread, in basis points, that matches the market price.
    pub oas_bps: f64,
    pub valid: bool,
}

/// Lattice price of the bond at a parallel spread. `callable` toggles the call.
fn lattice_price(i: &CallableOasInput, spread: f64, callable: bool) -> f64 {
    let n = i.steps.max(1) as usize;
    let dt = i.maturity_years / n as f64;
    let u = (i.rate_vol_pct / 100.0 * dt.sqrt()).exp();
    let r0 = i.short_rate_pct / 100.0;
    let cpn = i.coupon_rate_pct / 100.0 * i.face_value * dt;
    let lockout_steps = (i.lockout_years / dt).round() as usize;
    let q = 0.5;

    // Terminal payoff at step n.
    let mut v: Vec<f64> = vec![i.face_value + cpn; n + 1];
    for step in (0..n).rev() {
        let mut nv = Vec::with_capacity(step + 1);
        for j in 0..=step {
            let r = r0 * u.powi(2 * j as i32 - step as i32);
            let disc = (-(r + spread) * dt).exp();
            let mut cont = disc * (q * v[j + 1] + (1.0 - q) * v[j]) + cpn;
            if callable && step >= lockout_steps {
                cont = cont.min(i.call_price + cpn);
            }
            nv.push(cont);
        }
        v = nv;
    }
    v[0] - cpn
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &CallableOasInput) -> CallableOasReport {
    if i.maturity_years <= 0.0 || i.steps == 0 || i.face_value <= 0.0 {
        return CallableOasReport::default();
    }
    let straight = lattice_price(i, 0.0, false);
    let callable = lattice_price(i, 0.0, true);

    // OAS: bisection on the parallel spread so the callable model price equals the market price.
    let (mut lo, mut hi) = (-0.05_f64, 0.10_f64);
    let mut mid = 0.0;
    for _ in 0..80 {
        mid = (lo + hi) / 2.0;
        if lattice_price(i, mid, true) > i.market_price {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    CallableOasReport {
        straight_price: round4(straight),
        callable_price: round4(callable),
        option_cost: round4(straight - callable),
        oas_bps: round4(mid * 10_000.0),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.05
    }

    fn base() -> CallableOasInput {
        CallableOasInput {
            face_value: 100.0,
            coupon_rate_pct: 5.0,
            maturity_years: 5.0,
            steps: 5,
            short_rate_pct: 4.0,
            rate_vol_pct: 20.0,
            call_price: 100.0,
            lockout_years: 2.0,
            market_price: 99.0,
        }
    }

    #[test]
    fn lattice_prices_and_oas() {
        let d = generate(&base());
        assert!(d.valid);
        // Matches an independent Python lattice.
        assert!(close(d.straight_price, 103.4391));
        assert!(close(d.callable_price, 100.8445));
        assert!(close(d.option_cost, 2.5947));
        assert!((d.oas_bps - 64.69).abs() < 1.0);
    }

    #[test]
    fn callable_below_straight() {
        let d = generate(&base());
        assert!(d.callable_price < d.straight_price);
        assert!(d.option_cost > 0.0);
    }

    #[test]
    fn lower_market_price_higher_oas() {
        let cheap = generate(&CallableOasInput { market_price: 96.0, ..base() });
        let rich = generate(&CallableOasInput { market_price: 101.0, ..base() });
        assert!(cheap.oas_bps > rich.oas_bps);
    }

    #[test]
    fn higher_rate_vol_higher_option_cost() {
        let lo = generate(&base());
        let hi = generate(&CallableOasInput { rate_vol_pct: 35.0, ..base() });
        assert!(hi.option_cost > lo.option_cost);
    }

    #[test]
    fn invalid_inputs() {
        assert!(!generate(&CallableOasInput { maturity_years: 0.0, ..base() }).valid);
    }
}
