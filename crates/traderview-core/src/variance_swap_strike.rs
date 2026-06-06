//! Fair Variance-Swap Strike (Carr-Madan replication, 1998).
//!
//! The model-free fair strike of a variance swap equals the cost of
//! replicating the log-contract via a portfolio of OTM puts and calls:
//!
//!   K_var² = (2 / T) · [r·T − (S₀·exp(r·T) / F − 1) − ln(F / S₀)
//!                       + e^{r·T} · ∫₀^F P(K)/K² dK + e^{r·T} · ∫_F^∞ C(K)/K² dK]
//!
//! Practical discrete approximation:
//!
//!   K_var² ≈ (2·e^{r·T} / T) · Σ_strike Q(K) · ΔK / K²
//!
//! where Q(K) is the OTM option price at strike K (put for K < F,
//! call for K > F).
//!
//! Pure compute. Caller supplies the OTM option chain (strike, type,
//! mid-price), forward F (or assumes S₀·exp(r·T)), and time-to-expiry T.
//!
//! Companion to `variance_swap`, `iv_solver`, `cliquet_option`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OptionQuote {
    pub strike: f64,
    /// "call" or "put"
    pub option_type: OptionType,
    pub mid_price: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VarSwapStrikeReport {
    pub fair_variance_strike: f64,
    pub fair_vol_strike: f64,
    pub n_strikes_used: usize,
    pub forward: f64,
}

pub fn compute(
    spot: f64,
    risk_free_rate: f64,
    time_to_expiry: f64,
    chain: &[OptionQuote],
) -> Option<VarSwapStrikeReport> {
    if !spot.is_finite()
        || spot <= 0.0
        || !risk_free_rate.is_finite()
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || chain.len() < 3
    {
        return None;
    }
    if chain.iter().any(|q| {
        !q.strike.is_finite() || q.strike <= 0.0 || !q.mid_price.is_finite() || q.mid_price < 0.0
    }) {
        return None;
    }
    let forward = spot * (risk_free_rate * time_to_expiry).exp();
    // Keep only OTM quotes; for each strike, take the OTM side only:
    //   - if K < F: keep put (call would be ITM)
    //   - if K > F: keep call
    //   - if K == F: use either (or both averaged; here take avg)
    let mut otm: Vec<(f64, f64)> = Vec::new();
    for q in chain {
        let is_otm = match q.option_type {
            OptionType::Put => q.strike < forward,
            OptionType::Call => q.strike > forward,
        };
        let is_atm = (q.strike - forward).abs() / forward < 1e-6;
        if is_otm || is_atm {
            otm.push((q.strike, q.mid_price));
        }
    }
    if otm.len() < 3 {
        return None;
    }
    otm.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    // Trapezoidal-ish integration: per-strike contribution Q(K) · ΔK / K².
    // ΔK at strike i: average of (K_i − K_{i-1}) and (K_{i+1} − K_i), with
    // half-widths at the endpoints.
    let n = otm.len();
    let mut integral = 0.0_f64;
    for i in 0..n {
        let dk = if n == 1 {
            0.0
        } else if i == 0 {
            otm[1].0 - otm[0].0
        } else if i == n - 1 {
            otm[n - 1].0 - otm[n - 2].0
        } else {
            0.5 * (otm[i + 1].0 - otm[i - 1].0)
        };
        if dk > 0.0 && otm[i].0 > 0.0 {
            integral += otm[i].1 * dk / (otm[i].0 * otm[i].0);
        }
    }
    let discount = (risk_free_rate * time_to_expiry).exp();
    let fair_var = (2.0 * discount / time_to_expiry) * integral;
    let fair_vol = fair_var.max(0.0).sqrt();
    Some(VarSwapStrikeReport {
        fair_variance_strike: fair_var,
        fair_vol_strike: fair_vol,
        n_strikes_used: n,
        forward,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(k: f64, p: f64) -> OptionQuote {
        OptionQuote {
            strike: k,
            option_type: OptionType::Put,
            mid_price: p,
        }
    }
    fn call(k: f64, p: f64) -> OptionQuote {
        OptionQuote {
            strike: k,
            option_type: OptionType::Call,
            mid_price: p,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let chain = vec![put(95.0, 1.0), call(105.0, 1.0)];
        assert!(compute(0.0, 0.05, 0.25, &chain).is_none());
        assert!(compute(100.0, 0.05, 0.0, &chain).is_none());
        assert!(compute(100.0, f64::NAN, 0.25, &chain).is_none());
        assert!(compute(100.0, 0.05, 0.25, &[]).is_none());
    }

    #[test]
    fn fewer_than_three_strikes_returns_none() {
        let chain = vec![put(95.0, 1.0), call(105.0, 1.0)];
        assert!(compute(100.0, 0.05, 0.25, &chain).is_none());
    }

    #[test]
    fn fair_variance_strike_positive_for_normal_chain() {
        let chain = vec![
            put(80.0, 0.20),
            put(90.0, 0.80),
            put(95.0, 1.50),
            put(99.0, 2.50),
            call(101.0, 2.50),
            call(105.0, 1.40),
            call(110.0, 0.70),
            call(120.0, 0.15),
        ];
        let r = compute(100.0, 0.05, 0.25, &chain).unwrap();
        assert!(r.fair_variance_strike > 0.0);
        assert!(r.fair_vol_strike > 0.0);
    }

    #[test]
    fn vol_strike_is_sqrt_of_variance_strike() {
        let chain = vec![
            put(80.0, 0.20),
            put(95.0, 1.50),
            put(99.0, 2.50),
            call(101.0, 2.50),
            call(105.0, 1.40),
            call(120.0, 0.15),
        ];
        let r = compute(100.0, 0.05, 0.25, &chain).unwrap();
        assert!((r.fair_vol_strike - r.fair_variance_strike.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn forward_uses_continuous_compounding() {
        let chain = vec![
            put(80.0, 0.20),
            put(95.0, 1.50),
            put(99.0, 2.50),
            call(101.0, 2.50),
            call(105.0, 1.40),
            call(120.0, 0.15),
        ];
        let r = compute(100.0, 0.05, 1.0, &chain).unwrap();
        let expected_fwd = 100.0_f64 * 0.05_f64.exp();
        assert!((r.forward - expected_fwd).abs() < 1e-9);
    }

    #[test]
    fn richer_otm_chain_yields_higher_var_strike() {
        // More expensive OTM puts = market pricing more downside vol.
        let cheap_otm = vec![
            put(80.0, 0.10),
            put(95.0, 0.50),
            put(99.0, 1.00),
            call(101.0, 1.00),
            call(105.0, 0.50),
            call(120.0, 0.10),
        ];
        let rich_otm = vec![
            put(80.0, 2.00),
            put(95.0, 4.00),
            put(99.0, 5.00),
            call(101.0, 5.00),
            call(105.0, 4.00),
            call(120.0, 2.00),
        ];
        let r_cheap = compute(100.0, 0.05, 0.25, &cheap_otm).unwrap();
        let r_rich = compute(100.0, 0.05, 0.25, &rich_otm).unwrap();
        assert!(r_rich.fair_variance_strike > r_cheap.fair_variance_strike);
    }

    #[test]
    fn nan_or_negative_quotes_rejected() {
        let chain = vec![put(95.0, f64::NAN), call(105.0, 1.0), call(120.0, 0.1)];
        assert!(compute(100.0, 0.05, 0.25, &chain).is_none());
        let neg = vec![put(95.0, -1.0), call(105.0, 1.0), call(120.0, 0.1)];
        assert!(compute(100.0, 0.05, 0.25, &neg).is_none());
    }
}
