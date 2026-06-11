//! Closed-end fund discount/premium and the classic z-score screen.
//!
//!   discount = price / NAV − 1     (negative = trading below NAV)
//!   z        = (discount − mean_discount) / std_discount
//!
//! CEF discounts mean-revert; the screen buys deep-negative z (the
//! fund is cheaper than ITS OWN history, not just cheap) and harvests
//! the reversion plus the yield pickup from buying assets under par.
//!
//! Pure compute. Companion to `merger_arb`, `deep_value`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CefInput {
    pub price: f64,
    pub nav: f64,
    /// Annual distribution per share, $ (0 = skip yield rows).
    #[serde(default)]
    pub annual_distribution: f64,
    /// Historical mean discount, % (e.g. −8.5). Optional z-score leg.
    pub mean_discount_pct: Option<f64>,
    pub std_discount_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CefReport {
    /// Negative = discount, positive = premium, %.
    pub discount_pct: f64,
    pub z_score: Option<f64>,
    /// Distribution yield on price vs on NAV — the discount's income
    /// pickup.
    pub yield_on_price_pct: Option<f64>,
    pub yield_on_nav_pct: Option<f64>,
}

pub fn compute(inp: &CefInput) -> Option<CefReport> {
    if !inp.price.is_finite()
        || inp.price <= 0.0
        || !inp.nav.is_finite()
        || inp.nav <= 0.0
        || !inp.annual_distribution.is_finite()
        || inp.annual_distribution < 0.0
    {
        return None;
    }
    let discount = (inp.price / inp.nav - 1.0) * 100.0;
    let z = match (inp.mean_discount_pct, inp.std_discount_pct) {
        (Some(m), Some(s)) if m.is_finite() && s.is_finite() && s > 0.0 => {
            Some((discount - m) / s)
        }
        (None, None) => None,
        _ => return None, // half-supplied or invalid z inputs
    };
    let has_dist = inp.annual_distribution > 0.0;
    Some(CefReport {
        discount_pct: discount,
        z_score: z,
        yield_on_price_pct: has_dist.then(|| inp.annual_distribution / inp.price * 100.0),
        yield_on_nav_pct: has_dist.then(|| inp.annual_distribution / inp.nav * 100.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discount_z_and_yield_pickup_hand_walk() {
        // $9.20 on $10 NAV = −8% discount; history −5 ± 2 ⇒ z = −1.5.
        let r = compute(&CefInput {
            price: 9.2,
            nav: 10.0,
            annual_distribution: 0.92,
            mean_discount_pct: Some(-5.0),
            std_discount_pct: Some(2.0),
        })
        .unwrap();
        assert!((r.discount_pct + 8.0).abs() < 1e-9);
        assert!((r.z_score.unwrap() + 1.5).abs() < 1e-9);
        // 0.92 on 9.20 = exactly 10% on price vs 9.2% on NAV.
        assert!((r.yield_on_price_pct.unwrap() - 10.0).abs() < 1e-9);
        assert!((r.yield_on_nav_pct.unwrap() - 9.2).abs() < 1e-9);
    }

    #[test]
    fn premium_funds_read_positive() {
        let r = compute(&CefInput {
            price: 11.0,
            nav: 10.0,
            annual_distribution: 0.0,
            mean_discount_pct: None,
            std_discount_pct: None,
        })
        .unwrap();
        assert!((r.discount_pct - 10.0).abs() < 1e-9);
        assert!(r.z_score.is_none());
        assert!(r.yield_on_price_pct.is_none());
    }

    #[test]
    fn hostile_inputs_return_none() {
        let base = CefInput {
            price: 9.0,
            nav: 10.0,
            annual_distribution: 0.0,
            mean_discount_pct: None,
            std_discount_pct: None,
        };
        assert!(compute(&CefInput { nav: 0.0, ..base.clone() }).is_none());
        assert!(compute(&CefInput { price: f64::NAN, ..base.clone() }).is_none());
        // Half-supplied z-score legs reject instead of guessing.
        assert!(compute(&CefInput { mean_discount_pct: Some(-5.0), ..base.clone() }).is_none());
        assert!(compute(&CefInput {
            mean_discount_pct: Some(-5.0),
            std_discount_pct: Some(0.0),
            ..base
        })
        .is_none());
    }
}
