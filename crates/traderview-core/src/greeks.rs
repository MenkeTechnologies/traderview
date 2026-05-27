//! Black-Scholes-Merton option pricing + Greeks.
//!
//! All inputs in their natural units:
//!   S = spot, K = strike, T = years to expiry, sigma = annual vol (e.g. 0.30),
//!   r = risk-free annual rate, q = continuous dividend yield (0.0 for most).
//! Returns prices in the same currency as S/K.

use serde::Serialize;
use std::f64::consts::{PI, E};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OptKind { Call, Put }

#[derive(Debug, Clone, Serialize)]
pub struct Greeks {
    pub price: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,        // per-day (already divided by 365)
    pub vega: f64,         // per 1 vol pt (already / 100)
    pub rho: f64,          // per 1 rate pt (already / 100)
    pub d1: f64,
    pub d2: f64,
}

pub fn price_and_greeks(kind: OptKind, s: f64, k: f64, t: f64, sigma: f64, r: f64, q: f64) -> Greeks {
    if t <= 0.0 || sigma <= 0.0 || s <= 0.0 || k <= 0.0 {
        // Intrinsic only.
        let intrinsic = match kind {
            OptKind::Call => (s - k).max(0.0),
            OptKind::Put  => (k - s).max(0.0),
        };
        return Greeks { price: intrinsic, delta: 0.0, gamma: 0.0, theta: 0.0, vega: 0.0, rho: 0.0, d1: 0.0, d2: 0.0 };
    }
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let n_d1 = cdf(d1);
    let n_d2 = cdf(d2);
    let pdf_d1 = pdf(d1);

    let disc_r = (-r * t).exp();
    let disc_q = (-q * t).exp();

    let (price, delta, theta_yr, rho) = match kind {
        OptKind::Call => {
            let p = s * disc_q * n_d1 - k * disc_r * n_d2;
            let d = disc_q * n_d1;
            let theta = -(s * disc_q * pdf_d1 * sigma) / (2.0 * sqrt_t)
                        - r * k * disc_r * n_d2
                        + q * s * disc_q * n_d1;
            let rho = k * t * disc_r * n_d2 / 100.0;
            (p, d, theta, rho)
        }
        OptKind::Put => {
            let p = k * disc_r * cdf(-d2) - s * disc_q * cdf(-d1);
            let d = -disc_q * cdf(-d1);
            let theta = -(s * disc_q * pdf_d1 * sigma) / (2.0 * sqrt_t)
                        + r * k * disc_r * cdf(-d2)
                        - q * s * disc_q * cdf(-d1);
            let rho = -k * t * disc_r * cdf(-d2) / 100.0;
            (p, d, theta, rho)
        }
    };
    let gamma = (disc_q * pdf_d1) / (s * sigma * sqrt_t);
    let vega = s * disc_q * pdf_d1 * sqrt_t / 100.0;
    Greeks { price, delta, gamma, theta: theta_yr / 365.0, vega, rho, d1, d2 }
}

/// Standard normal PDF.
fn pdf(x: f64) -> f64 { (-0.5 * x * x).exp() / (2.0 * PI).sqrt() }

/// Standard normal CDF via Abramowitz & Stegun 26.2.17 (max error ≈ 7.5e-8).
fn cdf(x: f64) -> f64 {
    let a1 =  0.319381530;
    let a2 = -0.356563782;
    let a3 =  1.781477937;
    let a4 = -1.821255978;
    let a5 =  1.330274429;
    let l = x.abs();
    let k = 1.0 / (1.0 + 0.2316419 * l);
    let w = 1.0 - 1.0 / (2.0 * PI).sqrt() * (-l * l / 2.0).exp()
        * (a1 * k + a2 * k.powi(2) + a3 * k.powi(3) + a4 * k.powi(4) + a5 * k.powi(5));
    if x >= 0.0 { w } else { 1.0 - w }
    // E import is kept for callers; suppress unused-var lint.
}

/// Newton-Raphson IV solve. Returns annualized vol (e.g. 0.32 = 32%).
pub fn implied_vol(kind: OptKind, market: f64, s: f64, k: f64, t: f64, r: f64, q: f64) -> Option<f64> {
    if market <= 0.0 || t <= 0.0 { return None; }
    let mut sigma = 0.3;
    for _ in 0..50 {
        let g = price_and_greeks(kind, s, k, t, sigma, r, q);
        let diff = g.price - market;
        if diff.abs() < 1e-5 { return Some(sigma); }
        let vega = g.vega * 100.0; // un-scale (price_and_greeks pre-divided)
        if vega.abs() < 1e-10 { break; }
        sigma -= diff / vega;
        sigma = sigma.clamp(0.001, 5.0);
    }
    Some(sigma)
}

const _USE_E: f64 = E;

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }

    #[test]
    fn atm_call_makes_sense() {
        // S=100, K=100, T=1y, sigma=20%, r=5%, q=0 → known BS value ≈ 10.45
        let g = price_and_greeks(OptKind::Call, 100.0, 100.0, 1.0, 0.20, 0.05, 0.0);
        assert!(near(g.price, 10.45, 0.05), "price={}", g.price);
        assert!(g.delta > 0.5 && g.delta < 0.7);
        assert!(g.gamma > 0.0);
        assert!(g.vega > 0.0);
    }

    #[test]
    fn put_call_parity_holds() {
        // C - P = S*e^(-q*T) - K*e^(-r*T)
        let c = price_and_greeks(OptKind::Call, 100.0, 100.0, 0.5, 0.25, 0.04, 0.02).price;
        let p = price_and_greeks(OptKind::Put,  100.0, 100.0, 0.5, 0.25, 0.04, 0.02).price;
        let lhs = c - p;
        let rhs = 100.0 * (-0.02 * 0.5_f64).exp() - 100.0 * (-0.04 * 0.5_f64).exp();
        assert!(near(lhs, rhs, 0.01), "lhs={lhs} rhs={rhs}");
    }

    #[test]
    fn implied_vol_round_trips() {
        let mkt = price_and_greeks(OptKind::Call, 100.0, 105.0, 0.5, 0.35, 0.04, 0.0).price;
        let iv = implied_vol(OptKind::Call, mkt, 100.0, 105.0, 0.5, 0.04, 0.0).unwrap();
        assert!(near(iv, 0.35, 0.001), "iv={}", iv);
    }
}
