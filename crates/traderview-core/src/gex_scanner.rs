//! Gamma Exposure (GEX) Scanner — per-strike dealer-net gamma exposure.
//!
//! Models the dealer side of an options market: dealers are typically
//! NET SHORT calls (retail buys calls) and NET LONG puts (retail
//! writes puts), so per the standard convention:
//!
//!   GEX_strike = (call_OI − put_OI) · gamma_per_contract · spot² · 100
//!
//! where gamma_per_contract is Black-Scholes gamma (per share) and the
//! `· 100` converts to per-contract terms (100 shares/contract).
//!
//! Aggregate GEX:
//!
//!   total_GEX = Σ_strike GEX_strike
//!
//! Interpretation:
//!   - Positive total GEX → dealers long gamma → mean-reverting price
//!     action as dealers sell into rallies / buy dips
//!   - Negative total GEX → dealers short gamma → momentum-amplifying
//!     price action (gamma squeeze risk near strikes with large OI)
//!
//! Per-strike: positive GEX at a strike = "magnet" / support;
//! negative GEX = volatility-amplifying / breakout-prone.
//!
//! Pure compute. Caller supplies the option chain + spot + per-contract
//! gamma (which they can get from any BS pricer in this crate).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OptionStrike {
    pub strike: f64,
    pub call_open_interest: f64,
    pub put_open_interest: f64,
    /// Per-contract dollar-gamma multiplier already including the
    /// shares-per-contract factor (typically gamma_per_share · 100 · spot²).
    pub call_gamma_per_contract: f64,
    pub put_gamma_per_contract: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeGex {
    pub strike: f64,
    pub gex_dollars: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GexReport {
    pub total_gex: f64,
    pub per_strike: Vec<StrikeGex>,
    pub zero_gamma_strike: Option<f64>,
    pub largest_positive_strike: Option<f64>,
    pub largest_negative_strike: Option<f64>,
}

pub fn scan(chain: &[OptionStrike]) -> Option<GexReport> {
    if chain.is_empty() { return None; }
    if chain.iter().any(|c| !c.strike.is_finite() || c.strike <= 0.0
        || !c.call_open_interest.is_finite() || c.call_open_interest < 0.0
        || !c.put_open_interest.is_finite() || c.put_open_interest < 0.0
        || !c.call_gamma_per_contract.is_finite()
        || !c.put_gamma_per_contract.is_finite()) {
        return None;
    }
    // Dealers short calls (positive call OI → negative dealer gamma) and
    // long puts (positive put OI → positive dealer gamma).
    let mut per: Vec<StrikeGex> = chain.iter().map(|c| {
        let dealer_gex = c.put_open_interest * c.put_gamma_per_contract
            - c.call_open_interest * c.call_gamma_per_contract;
        StrikeGex { strike: c.strike, gex_dollars: dealer_gex }
    }).collect();
    per.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap_or(std::cmp::Ordering::Equal));
    let total: f64 = per.iter().map(|s| s.gex_dollars).sum();
    // Walk strikes to find the first sign change for zero-gamma estimate.
    let mut zero_gamma = None;
    let mut cumulative = 0.0_f64;
    let mut prev_cum = 0.0_f64;
    let mut prev_strike = 0.0_f64;
    for s in &per {
        cumulative += s.gex_dollars;
        if prev_strike > 0.0 && prev_cum.signum() != cumulative.signum() && cumulative != 0.0 {
            // Linear interp on cumulative GEX between prev_strike and s.strike.
            let span = s.strike - prev_strike;
            let dy = cumulative - prev_cum;
            if dy.abs() > 1e-18 {
                let frac = -prev_cum / dy;
                zero_gamma = Some(prev_strike + frac * span);
            } else {
                zero_gamma = Some(s.strike);
            }
            break;
        }
        prev_cum = cumulative;
        prev_strike = s.strike;
    }
    let largest_pos = per.iter()
        .filter(|s| s.gex_dollars > 0.0)
        .max_by(|a, b| a.gex_dollars.partial_cmp(&b.gex_dollars).unwrap())
        .map(|s| s.strike);
    let largest_neg = per.iter()
        .filter(|s| s.gex_dollars < 0.0)
        .min_by(|a, b| a.gex_dollars.partial_cmp(&b.gex_dollars).unwrap())
        .map(|s| s.strike);
    Some(GexReport {
        total_gex: total,
        per_strike: per,
        zero_gamma_strike: zero_gamma,
        largest_positive_strike: largest_pos,
        largest_negative_strike: largest_neg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(strike: f64, call_oi: f64, put_oi: f64, gamma: f64) -> OptionStrike {
        OptionStrike {
            strike,
            call_open_interest: call_oi,
            put_open_interest: put_oi,
            call_gamma_per_contract: gamma,
            put_gamma_per_contract: gamma,
        }
    }

    #[test]
    fn empty_chain_returns_none() {
        assert!(scan(&[]).is_none());
    }

    #[test]
    fn nan_or_negative_inputs_return_none() {
        assert!(scan(&[s(f64::NAN, 100.0, 100.0, 0.05)]).is_none());
        assert!(scan(&[s(100.0, -1.0, 100.0, 0.05)]).is_none());
        assert!(scan(&[s(0.0, 100.0, 100.0, 0.05)]).is_none());
    }

    #[test]
    fn balanced_chain_yields_zero_total_gex() {
        // Equal call and put OI everywhere → dealer net gamma = 0.
        let chain = vec![
            s(95.0, 1000.0, 1000.0, 0.05),
            s(100.0, 2000.0, 2000.0, 0.07),
            s(105.0, 1500.0, 1500.0, 0.04),
        ];
        let r = scan(&chain).unwrap();
        assert!(r.total_gex.abs() < 1e-9);
    }

    #[test]
    fn call_heavy_chain_yields_negative_gex() {
        // Retail buys calls → dealers short calls → negative dealer gamma.
        let chain = vec![s(100.0, 5000.0, 100.0, 0.05)];
        let r = scan(&chain).unwrap();
        assert!(r.total_gex < 0.0, "call-heavy chain: GEX should be negative, got {}", r.total_gex);
    }

    #[test]
    fn put_heavy_chain_yields_positive_gex() {
        // Retail sells puts → dealers long puts → positive dealer gamma.
        let chain = vec![s(100.0, 100.0, 5000.0, 0.05)];
        let r = scan(&chain).unwrap();
        assert!(r.total_gex > 0.0, "put-heavy chain: GEX should be positive, got {}", r.total_gex);
    }

    #[test]
    fn per_strike_sorted_ascending() {
        let chain = vec![
            s(105.0, 100.0, 200.0, 0.05),
            s(95.0, 200.0, 100.0, 0.04),
            s(100.0, 150.0, 150.0, 0.06),
        ];
        let r = scan(&chain).unwrap();
        for w in r.per_strike.windows(2) {
            assert!(w[1].strike > w[0].strike);
        }
    }

    #[test]
    fn zero_gamma_strike_estimated_between_sign_changes() {
        // Negative GEX at low strikes, large positive at high → cumulative
        // crosses zero between them. Need put-side big enough that cumulative
        // sign flips, not just zeroes out.
        let chain = vec![
            s(95.0, 1000.0, 0.0, 0.05),    // dealer = -50
            s(100.0, 0.0, 2000.0, 0.05),    // dealer = +100; cumulative -50 → +50
        ];
        let r = scan(&chain).unwrap();
        assert!(r.zero_gamma_strike.is_some());
        let zg = r.zero_gamma_strike.unwrap();
        assert!((95.0..=100.0).contains(&zg), "zero-gamma {zg} not in [95, 100]");
    }

    #[test]
    fn largest_positive_and_negative_strikes_identified() {
        let chain = vec![
            s(95.0, 100.0, 5000.0, 0.05),    // strongly positive
            s(100.0, 5000.0, 100.0, 0.05),    // strongly negative
        ];
        let r = scan(&chain).unwrap();
        assert_eq!(r.largest_positive_strike, Some(95.0));
        assert_eq!(r.largest_negative_strike, Some(100.0));
    }
}
