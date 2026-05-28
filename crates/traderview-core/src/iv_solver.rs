//! Implied volatility solver — inverts Black-Scholes for σ via Brent's
//! method on the price-residual function.
//!
//!   bs_price(σ) = market_price
//!
//! Brent's method combines bisection's guaranteed convergence with the
//! superlinear convergence of inverse-quadratic interpolation. We bracket
//! σ ∈ [1e-6, 5.0] (0.0001%..500%), which covers every reasonable
//! options market and the deep-vol extremes seen in event-day reprices.
//!
//! Inputs: market_price, spot, strike, time_to_expiry (years), risk_free,
//! dividend_yield, OptionKind. Returns None when:
//!   - inputs are non-finite or non-positive,
//!   - market_price is below intrinsic / above the no-arb upper bound,
//!   - Brent fails to converge in `max_iter` (defaults to 100).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind { Call, Put }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IvReport {
    pub implied_vol: f64,
    pub iterations: usize,
    pub residual: f64,
}

pub fn solve(
    market_price: f64,
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    kind: OptionKind,
) -> Option<IvReport> {
    if !market_price.is_finite() || market_price <= 0.0
        || !spot.is_finite() || spot <= 0.0
        || !strike.is_finite() || strike <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free.is_finite() || !dividend_yield.is_finite()
    {
        return None;
    }
    // No-arbitrage bounds.
    let dq = (-dividend_yield * time_to_expiry).exp();
    let dr = (-risk_free * time_to_expiry).exp();
    let (lower_bound, upper_bound) = match kind {
        OptionKind::Call => ((spot * dq - strike * dr).max(0.0), spot * dq),
        OptionKind::Put  => ((strike * dr - spot * dq).max(0.0), strike * dr),
    };
    // Strict bounds violation → unsolvable; allow tiny rounding slop.
    if market_price + 1e-12 < lower_bound || market_price > upper_bound + 1e-12 {
        return None;
    }
    // Bracket [σ_lo, σ_hi].
    let sigma_lo = 1e-6_f64;
    let sigma_hi = 5.0_f64;
    let f = |sigma: f64| -> f64 {
        bs_price(spot, strike, time_to_expiry, risk_free, dividend_yield, sigma, kind)
            - market_price
    };
    let f_lo = f(sigma_lo);
    let f_hi = f(sigma_hi);
    if f_lo.signum() == f_hi.signum() && f_lo != 0.0 && f_hi != 0.0 {
        return None;    // root not bracketed
    }
    let (a, b, fa, fb, iters, residual) = brent(sigma_lo, sigma_hi, f_lo, f_hi, &f, 100, 1e-9)?;
    let _ = (fa, fb);
    Some(IvReport {
        implied_vol: b,
        iterations: iters,
        residual,
    }).filter(|r| (r.implied_vol - a).is_finite() || (r.implied_vol - sigma_lo).is_finite())
}

fn brent<F: Fn(f64) -> f64>(
    a: f64, b: f64, fa: f64, fb: f64, f: &F, max_iter: usize, tol: f64,
) -> Option<(f64, f64, f64, f64, usize, f64)> {
    let (mut a, mut b, mut fa, mut fb) = (a, b, fa, fb);
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    let mut c = a;
    let mut fc = fa;
    let mut mflag = true;
    let mut d = a;
    for iter in 1..=max_iter {
        if fb.abs() < tol {
            return Some((a, b, fa, fb, iter, fb));
        }
        let s = if fa != fc && fb != fc {
            // Inverse quadratic interpolation.
            a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb))
        } else {
            // Secant.
            b - fb * (b - a) / (fb - fa)
        };
        let cond1 = (s - (3.0 * a + b) / 4.0) * (s - b) >= 0.0;
        let cond2 = mflag && ((s - b).abs() >= (b - c).abs() / 2.0);
        let cond3 = !mflag && ((s - b).abs() >= (c - d).abs() / 2.0);
        let cond4 = mflag && ((b - c).abs() < tol);
        let cond5 = !mflag && ((c - d).abs() < tol);
        let s = if cond1 || cond2 || cond3 || cond4 || cond5 {
            mflag = true;
            (a + b) / 2.0    // bisection fallback
        } else {
            mflag = false;
            s
        };
        let fs = f(s);
        d = c;
        c = b;
        fc = fb;
        if fa * fs < 0.0 {
            b = s; fb = fs;
        } else {
            a = s; fa = fs;
        }
        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
        if (b - a).abs() < tol {
            return Some((a, b, fa, fb, iter, fb));
        }
    }
    None
}

fn bs_price(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64, kind: OptionKind) -> f64 {
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let dq = (-q * t).exp();
    let dr = (-r * t).exp();
    match kind {
        OptionKind::Call => s * dq * nd1 - k * dr * nd2,
        OptionKind::Put  => k * dr * (1.0 - nd2) - s * dq * (1.0 - nd1),
    }
}

fn norm_cdf(x: f64) -> f64 {
    // A&S 26.2.17, max err 7.5e-8.
    let a1 =  0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 =  1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 =  1.061405429_f64;
    let p  =  0.3275911_f64;
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
        assert!(solve(0.0, 100.0, 100.0, 0.25, 0.05, 0.0, OptionKind::Call).is_none());
        assert!(solve(5.0, 0.0, 100.0, 0.25, 0.05, 0.0, OptionKind::Call).is_none());
        assert!(solve(5.0, 100.0, 0.0, 0.25, 0.05, 0.0, OptionKind::Call).is_none());
        assert!(solve(5.0, 100.0, 100.0, 0.0, 0.05, 0.0, OptionKind::Call).is_none());
        assert!(solve(f64::NAN, 100.0, 100.0, 0.25, 0.05, 0.0, OptionKind::Call).is_none());
    }

    #[test]
    fn price_below_intrinsic_returns_none() {
        // Deep ITM call: spot=100, strike=50 → intrinsic ≈ 50 (with dividends discounted).
        // Quoting market_price = 30 < intrinsic should fail.
        assert!(solve(30.0, 100.0, 50.0, 1.0, 0.05, 0.0, OptionKind::Call).is_none());
    }

    #[test]
    fn round_trip_recovers_known_volatility() {
        // Compute a BS price at σ=0.25 then invert: should recover 0.25.
        let s = 100.0; let k = 100.0; let t = 0.5; let r = 0.05; let q = 0.0;
        let true_sigma = 0.25;
        let price = bs_price(s, k, t, r, q, true_sigma, OptionKind::Call);
        let report = solve(price, s, k, t, r, q, OptionKind::Call).unwrap();
        assert!((report.implied_vol - true_sigma).abs() < 1e-6,
            "IV should recover {true_sigma}, got {}", report.implied_vol);
        assert!(report.iterations <= 100);
    }

    #[test]
    fn round_trip_deep_otm_put() {
        let s = 100.0; let k = 80.0; let t = 0.5; let r = 0.05; let q = 0.0;
        let true_sigma = 0.40;
        let price = bs_price(s, k, t, r, q, true_sigma, OptionKind::Put);
        let report = solve(price, s, k, t, r, q, OptionKind::Put).unwrap();
        assert!((report.implied_vol - true_sigma).abs() < 1e-6);
    }

    #[test]
    fn round_trip_high_vol_extreme() {
        // 200% IV (event-day reprice). Should still converge.
        let s = 100.0; let k = 100.0; let t = 0.05; let r = 0.05; let q = 0.0;
        let true_sigma = 2.0;
        let price = bs_price(s, k, t, r, q, true_sigma, OptionKind::Call);
        let report = solve(price, s, k, t, r, q, OptionKind::Call).unwrap();
        assert!((report.implied_vol - true_sigma).abs() < 1e-5);
    }

    #[test]
    fn upper_bound_violation_returns_none() {
        // Call price cannot exceed S·e^{-qT}.
        let s = 100.0_f64; let k = 100.0_f64; let t = 0.5_f64; let r = 0.05_f64; let q = 0.0_f64;
        let upper = s * (-q * t).exp() + 1.0;    // above the no-arb upper bound
        assert!(solve(upper, s, k, t, r, q, OptionKind::Call).is_none());
    }
}
