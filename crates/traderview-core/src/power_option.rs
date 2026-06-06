//! Power option — European, payoff = (S_T)^a − K for the standard
//! "type 1" variant, or max(S_T − K, 0)^a for "type 2" (capped at K^a
//! for puts). We implement type 1 only here.
//!
//! Closed form (Black-Scholes underlying, a > 0):
//!
//!   adjusted_drift  = a · (r − q) + 0.5 · a · (a − 1) · σ²
//!   adjusted_sigma  = a · σ
//!   d1 = [ln(S/K^{1/a}) + (adj_drift + 0.5·adj_sigma²)·T] / (adj_sigma·√T)
//!   d2 = d1 − adj_sigma·√T
//!   call = S^a · e^{(adj_drift − r)·T} · N(d1) − K · e^{−rT} · N(d2)
//!   put  = K · e^{−rT} · N(−d2) − S^a · e^{(adj_drift − r)·T} · N(−d1)
//!
//! Pure compute. Power = 1 collapses to vanilla Black-Scholes; power > 1
//! amplifies tail risk; 0 < power < 1 dampens it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PowerOptionReport {
    pub price: f64,
    pub adjusted_drift: f64,
    pub adjusted_sigma: f64,
}

#[allow(clippy::too_many_arguments)] // canonical option-pricing signature
pub fn price(
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
    power: f64,
    kind: OptionKind,
) -> Option<PowerOptionReport> {
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
        || !power.is_finite()
        || power <= 0.0
    {
        return None;
    }
    let a = power;
    let mu = a * (risk_free - dividend_yield) + 0.5 * a * (a - 1.0) * sigma * sigma;
    let sig_a = a * sigma;
    let sqrt_t = time_to_expiry.sqrt();
    // Effective strike for the log-ratio is K^{1/a}, but we work with S_T^a
    // exceeding K which is equivalent to S_T > K^{1/a}.
    let k_eff = strike.powf(1.0 / a);
    let d1 = ((spot / k_eff).ln() + (mu + 0.5 * sig_a * sig_a) * time_to_expiry / a)
        / (sig_a * sqrt_t / a);
    // Rewrite more cleanly: d1 is the standard form when we map a·z → z.
    // Equivalent direct form:
    let d1_direct =
        (a * (spot / k_eff).ln() + (mu + 0.5 * sig_a * sig_a) * time_to_expiry) / (sig_a * sqrt_t);
    let _ = d1;
    let d1 = d1_direct;
    let d2 = d1 - sig_a * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let dr = (-risk_free * time_to_expiry).exp();
    let drift_factor = ((mu - risk_free) * time_to_expiry).exp();
    let s_a = spot.powf(a);
    let p = match kind {
        OptionKind::Call => s_a * drift_factor * nd1 - strike * dr * nd2,
        OptionKind::Put => strike * dr * (1.0 - nd2) - s_a * drift_factor * (1.0 - nd1),
    };
    if !p.is_finite() {
        return None;
    }
    Some(PowerOptionReport {
        price: p.max(0.0),
        adjusted_drift: mu,
        adjusted_sigma: sig_a,
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
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 0.5, 0.05, 0.0, 0.2, 2.0, OptionKind::Call).is_none());
            assert!(price(100.0, bad, 0.5, 0.05, 0.0, 0.2, 2.0, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, bad, 0.05, 0.0, 0.2, 2.0, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, bad, 2.0, OptionKind::Call).is_none());
            assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.2, bad, OptionKind::Call).is_none());
        }
    }

    #[test]
    fn power_one_collapses_to_black_scholes() {
        // Standard call: should equal BS call exactly.
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let q = 0.0;
        let v = 0.20;
        let r_power = price(s, k, t, r, q, v, 1.0, OptionKind::Call).unwrap();
        let sqrt_t = t.sqrt();
        let d1 = ((s / k).ln() + (r - q + 0.5 * v * v) * t) / (v * sqrt_t);
        let d2 = d1 - v * sqrt_t;
        let bs_call = s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2);
        assert!(
            (r_power.price - bs_call).abs() < 1e-9,
            "power=1 should match BS: power={} bs={}",
            r_power.price,
            bs_call
        );
    }

    #[test]
    fn higher_power_amplifies_call_price() {
        let r1 = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, 1.0, OptionKind::Call).unwrap();
        let r2 = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, 2.0, OptionKind::Call).unwrap();
        // S² option has bigger payoffs in the right tail → bigger price.
        // But note strike is fixed at 100 — so S²−100 vs S−100 — the
        // payoff blows up for S > 10. The price is much larger.
        assert!(r2.price > r1.price);
    }

    #[test]
    fn put_call_parity_under_power_holds() {
        // Power-option put-call parity:
        //   c − p = S^a · e^{(μ−r)·T} − K · e^{−r·T}
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let q = 0.0;
        let v = 0.20;
        let a = 2.0;
        let c = price(s, k, t, r, q, v, a, OptionKind::Call).unwrap();
        let p = price(s, k, t, r, q, v, a, OptionKind::Put).unwrap();
        let mu = a * (r - q) + 0.5 * a * (a - 1.0) * v * v;
        let parity = s.powf(a) * ((mu - r) * t).exp() - k * (-r * t).exp();
        assert!((c.price - p.price - parity).abs() < 1e-6);
    }

    #[test]
    fn fractional_power_dampens_payoff() {
        // a = 0.5 (sqrt) — much milder payoff than vanilla call.
        let r_sqrt = price(10_000.0, 100.0, 0.5, 0.05, 0.0, 0.20, 0.5, OptionKind::Call).unwrap();
        // S^0.5 = 100, K = 100 → at-the-money in the transformed coords.
        assert!(r_sqrt.price > 0.0);
    }

    #[test]
    fn adjusted_drift_includes_convexity_term() {
        // For power = 2, sigma = 0.2: μ = 2·(r−q) + 0.5·2·1·v² = 2·(r−q) + v².
        let r = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, 2.0, OptionKind::Call).unwrap();
        let expected_mu = 2.0 * (0.05 - 0.0) + 0.20 * 0.20;
        assert!((r.adjusted_drift - expected_mu).abs() < 1e-12);
        assert!((r.adjusted_sigma - 2.0 * 0.20).abs() < 1e-12);
    }
}
