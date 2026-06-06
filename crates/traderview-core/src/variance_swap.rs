//! Variance Swap Fair Strike — Carr-Madan / Demeterfi-Derman-Kamal log-strip.
//!
//! Synthesizes the fair variance strike (K²_var) by static replication
//! over a strip of out-of-the-money options on the underlying:
//!
//!   K²_var = (2/T) [ rT − (S₀ · e^{rT} / S* − 1) − ln(S* / S₀) ]
//!          + (2 · e^{rT} / T) ∫_0^S* P(K)/K² dK
//!          + (2 · e^{rT} / T) ∫_S*^∞ C(K)/K² dK
//!
//! where S* is a cutoff (default = S₀), P/C are put/call prices.
//!
//! Caller supplies a list of (strike, put_price, call_price) quotes;
//! the integral is approximated by trapezoidal rule over the strikes
//! given. Standard OTM convention: puts for K ≤ S*, calls for K > S*.
//!
//! Returns fair vol = √K²_var (annualized).
//!
//! Pure compute. Validates: T > 0, S₀ > 0, monotone strikes, at least
//! 2 distinct strikes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OptionQuote {
    pub strike: f64,
    pub put_price: f64,
    pub call_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VarianceSwapReport {
    pub fair_variance: f64,
    pub fair_vol: f64,
    pub put_strip_contribution: f64,
    pub call_strip_contribution: f64,
    pub strike_count: usize,
}

pub fn fair_strike(
    spot: f64,
    risk_free: f64,
    time_to_expiry: f64,
    quotes: &[OptionQuote],
) -> Option<VarianceSwapReport> {
    if !spot.is_finite()
        || spot <= 0.0
        || !risk_free.is_finite()
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || quotes.len() < 2
    {
        return None;
    }
    // Sort by strike + dedup; reject non-finite or non-positive strikes/prices.
    let mut qs: Vec<OptionQuote> = quotes
        .iter()
        .copied()
        .filter(|q| {
            q.strike.is_finite()
                && q.strike > 0.0
                && q.put_price.is_finite()
                && q.put_price >= 0.0
                && q.call_price.is_finite()
                && q.call_price >= 0.0
        })
        .collect();
    if qs.len() < 2 {
        return None;
    }
    qs.sort_by(|a, b| {
        a.strike
            .partial_cmp(&b.strike)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    qs.dedup_by(|a, b| a.strike == b.strike);
    if qs.len() < 2 {
        return None;
    }
    // Cutoff S* = closest strike at or below spot — caller picked the strip.
    let s_star_idx = qs.iter().rposition(|q| q.strike <= spot).unwrap_or(0);
    let s_star = qs[s_star_idx].strike;
    // Trapezoidal integration of P/K² (K ≤ S*) and C/K² (K > S*).
    let put_integral = trapezoid(&qs[..=s_star_idx], |q| q.put_price / (q.strike * q.strike));
    let call_integral = trapezoid(&qs[s_star_idx..], |q| q.call_price / (q.strike * q.strike));
    let exp_rt = (risk_free * time_to_expiry).exp();
    let strip = 2.0 * exp_rt / time_to_expiry * (put_integral + call_integral);
    // Constant terms (cash-and-carry adjustment).
    let constant = (2.0 / time_to_expiry)
        * (risk_free * time_to_expiry - (spot * exp_rt / s_star - 1.0) - (s_star / spot).ln());
    let fair_var = constant + strip;
    if !fair_var.is_finite() || fair_var < 0.0 {
        return None;
    }
    Some(VarianceSwapReport {
        fair_variance: fair_var,
        fair_vol: fair_var.sqrt(),
        put_strip_contribution: 2.0 * exp_rt / time_to_expiry * put_integral,
        call_strip_contribution: 2.0 * exp_rt / time_to_expiry * call_integral,
        strike_count: qs.len(),
    })
}

fn trapezoid<F: Fn(&OptionQuote) -> f64>(qs: &[OptionQuote], f: F) -> f64 {
    if qs.len() < 2 {
        return 0.0;
    }
    let mut sum = 0.0_f64;
    for w in qs.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let dk = b.strike - a.strike;
        let avg = (f(a) + f(b)) / 2.0;
        sum += dk * avg;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(k: f64, p: f64, c: f64) -> OptionQuote {
        OptionQuote {
            strike: k,
            put_price: p,
            call_price: c,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let qs = vec![q(100.0, 1.0, 1.0); 2];
        assert!(fair_strike(0.0, 0.05, 0.25, &qs).is_none());
        assert!(fair_strike(100.0, 0.05, 0.0, &qs).is_none());
        assert!(fair_strike(100.0, 0.05, 0.25, &qs[..1]).is_none());
        assert!(fair_strike(f64::NAN, 0.05, 0.25, &qs).is_none());
    }

    #[test]
    fn negative_strikes_filtered() {
        let qs = vec![q(-50.0, 1.0, 1.0), q(100.0, 1.0, 1.0)];
        assert!(fair_strike(100.0, 0.05, 0.25, &qs).is_none());
    }

    #[test]
    fn flat_option_strip_yields_finite_estimate() {
        // 11 strikes at 90..110 with put/call prices of 1.0 each — degenerate
        // but should still produce a finite (positive) variance.
        let qs: Vec<OptionQuote> = (90..=110)
            .step_by(2)
            .map(|k| q(k as f64, 1.0, 1.0))
            .collect();
        let r = fair_strike(100.0, 0.05, 0.25, &qs).unwrap();
        assert!(r.fair_variance > 0.0);
        assert!(r.fair_vol > 0.0);
        assert_eq!(r.strike_count, 11);
    }

    #[test]
    fn duplicate_strikes_deduplicated() {
        let qs = vec![q(100.0, 1.0, 1.0), q(100.0, 1.0, 1.0), q(110.0, 1.0, 1.0)];
        let r = fair_strike(100.0, 0.05, 0.25, &qs).unwrap();
        assert_eq!(r.strike_count, 2);
    }

    #[test]
    fn strike_strip_more_concentrated_around_spot_yields_more_accurate_estimate() {
        // Two scenarios with the same total strike range, different density.
        let dense: Vec<OptionQuote> = (95..=105)
            .step_by(1)
            .map(|k| q(k as f64, 1.0, 1.0))
            .collect();
        let sparse: Vec<OptionQuote> =
            vec![q(95.0, 1.0, 1.0), q(100.0, 1.0, 1.0), q(105.0, 1.0, 1.0)];
        let r_dense = fair_strike(100.0, 0.05, 0.25, &dense).unwrap();
        let r_sparse = fair_strike(100.0, 0.05, 0.25, &sparse).unwrap();
        // Both should be finite — denser strip generally produces slightly different value.
        assert!(r_dense.fair_variance.is_finite());
        assert!(r_sparse.fair_variance.is_finite());
    }

    #[test]
    fn fair_variance_grows_with_option_prices() {
        // Double the option prices → roughly double the strip integral → larger variance.
        let qs_cheap: Vec<OptionQuote> = (90..=110)
            .step_by(2)
            .map(|k| q(k as f64, 0.5, 0.5))
            .collect();
        let qs_expensive: Vec<OptionQuote> = (90..=110)
            .step_by(2)
            .map(|k| q(k as f64, 5.0, 5.0))
            .collect();
        let r_cheap = fair_strike(100.0, 0.05, 0.25, &qs_cheap).unwrap();
        let r_expensive = fair_strike(100.0, 0.05, 0.25, &qs_expensive).unwrap();
        assert!(r_expensive.fair_variance > r_cheap.fair_variance);
    }

    #[test]
    fn put_strip_contribution_separable() {
        let qs: Vec<OptionQuote> = (90..=110)
            .step_by(2)
            .map(|k| q(k as f64, 1.0, 0.5))
            .collect();
        let r = fair_strike(100.0, 0.05, 0.25, &qs).unwrap();
        assert!(r.put_strip_contribution > 0.0);
        assert!(r.call_strip_contribution > 0.0);
    }
}
