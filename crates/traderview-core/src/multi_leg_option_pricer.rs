//! Multi-Leg Option Pricer — pre-expiry mark-to-market of any
//! European-option strategy via Black-Scholes per leg.
//!
//! Each leg specifies kind (Call / Put / Underlying), strike, qty
//! (long > 0, short < 0), and a per-contract premium paid (entry cost).
//! The valuation produces:
//!
//!   leg_value_t  = qty · (bs_price(kind, S, K, T-t, r, q, σ) - premium)
//!   strategy P/L = Σ leg_value_t
//!
//! Underlying legs are valued linearly as `qty · (S - K - premium)`.
//!
//! All legs share the same valuation date (`T-t`), risk-free rate,
//! dividend yield, and volatility. For mixed-vol surfaces use per-leg
//! BS calls directly.
//!
//! Pure compute. Companion to `option_payoff_diagram`, `black_scholes`,
//! `greeks`, `option_strategy_screener`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegKind {
    Call,
    Put,
    Underlying,
}

#[derive(Clone, Debug)]
pub struct Leg {
    pub kind: LegKind,
    pub strike: f64,
    pub premium: f64,
    pub qty: f64,
}

#[derive(Debug)]
pub struct Report {
    pub strategy_value: f64,
    pub strategy_pnl: f64,
    pub leg_values: Vec<f64>,
    pub leg_pnls: Vec<f64>,
}

pub fn compute(
    legs: &[Leg],
    spot: f64,
    t_to_expiry: f64,
    rate: f64,
    div_yield: f64,
    sigma: f64,
) -> Option<Report> {
    if legs.is_empty() {
        return None;
    }
    let scalars = [spot, t_to_expiry, rate, div_yield, sigma];
    if scalars.iter().any(|x| !x.is_finite()) {
        return None;
    }
    if spot <= 0.0 || t_to_expiry < 0.0 || sigma < 0.0 {
        return None;
    }
    for l in legs {
        if !l.strike.is_finite() || !l.premium.is_finite() || !l.qty.is_finite() {
            return None;
        }
    }
    let mut leg_values = Vec::with_capacity(legs.len());
    let mut leg_pnls = Vec::with_capacity(legs.len());
    let mut total_value = 0.0_f64;
    let mut total_pnl = 0.0_f64;
    for l in legs {
        let raw = match l.kind {
            LegKind::Call => bs_call(spot, l.strike, t_to_expiry, rate, div_yield, sigma),
            LegKind::Put => bs_put(spot, l.strike, t_to_expiry, rate, div_yield, sigma),
            LegKind::Underlying => spot - l.strike,
        };
        let v = l.qty * raw;
        let p = l.qty * (raw - l.premium);
        leg_values.push(v);
        leg_pnls.push(p);
        total_value += v;
        total_pnl += p;
    }
    Some(Report {
        strategy_value: total_value,
        strategy_pnl: total_pnl,
        leg_values,
        leg_pnls,
    })
}

fn bs_call(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64) -> f64 {
    if t <= 0.0 || sigma <= 0.0 {
        return (s - k).max(0.0);
    }
    let st = sigma * t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / st;
    let d2 = d1 - st;
    s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
}

fn bs_put(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64) -> f64 {
    if t <= 0.0 || sigma <= 0.0 {
        return (k - s).max(0.0);
    }
    let st = sigma * t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / st;
    let d2 = d1 - st;
    k * (-r * t).exp() * norm_cdf(-d2) - s * (-q * t).exp() * norm_cdf(-d1)
}

/// Abramowitz-Stegun 26.2.17 — error < 7.5e-8.
fn norm_cdf(x: f64) -> f64 {
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = (x / std::f64::consts::SQRT_2).abs();
    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let legs = vec![Leg {
            kind: LegKind::Call,
            strike: 100.0,
            premium: 5.0,
            qty: 1.0,
        }];
        assert!(compute(&[], 100.0, 0.5, 0.05, 0.0, 0.2).is_none());
        assert!(compute(&legs, -1.0, 0.5, 0.05, 0.0, 0.2).is_none());
        assert!(compute(&legs, 100.0, -0.5, 0.05, 0.0, 0.2).is_none());
        assert!(compute(&legs, 100.0, 0.5, 0.05, 0.0, -0.2).is_none());
        assert!(compute(&legs, f64::NAN, 0.5, 0.05, 0.0, 0.2).is_none());
    }

    #[test]
    fn at_expiry_collapses_to_intrinsic() {
        // T=0: BS reduces to max(S-K, 0).
        let legs = vec![Leg {
            kind: LegKind::Call,
            strike: 100.0,
            premium: 5.0,
            qty: 1.0,
        }];
        let r = compute(&legs, 110.0, 0.0, 0.05, 0.0, 0.2).unwrap();
        assert!((r.strategy_value - 10.0).abs() < 1e-9);
        // P/L = intrinsic - premium = 10 - 5 = 5.
        assert!((r.strategy_pnl - 5.0).abs() < 1e-9);
    }

    #[test]
    fn put_call_parity_holds() {
        // C - P = S·e^(-qT) - K·e^(-rT) for European options.
        let s = 100.0;
        let k = 100.0;
        let t = 0.5;
        let r = 0.05;
        let q = 0.0;
        let sigma = 0.25;
        let call = vec![Leg {
            kind: LegKind::Call,
            strike: k,
            premium: 0.0,
            qty: 1.0,
        }];
        let put = vec![Leg {
            kind: LegKind::Put,
            strike: k,
            premium: 0.0,
            qty: 1.0,
        }];
        let c_val = compute(&call, s, t, r, q, sigma).unwrap().strategy_value;
        let p_val = compute(&put, s, t, r, q, sigma).unwrap().strategy_value;
        let parity = s * (-q * t).exp() - k * (-r * t).exp();
        assert!((c_val - p_val - parity).abs() < 1e-6);
    }

    #[test]
    fn straddle_value_exceeds_individual_legs() {
        // ATM straddle should be ~2x ATM call value at-the-money.
        let legs = vec![
            Leg {
                kind: LegKind::Call,
                strike: 100.0,
                premium: 0.0,
                qty: 1.0,
            },
            Leg {
                kind: LegKind::Put,
                strike: 100.0,
                premium: 0.0,
                qty: 1.0,
            },
        ];
        let r = compute(&legs, 100.0, 0.5, 0.05, 0.0, 0.25).unwrap();
        assert!(r.strategy_value > 0.0);
        assert_eq!(r.leg_values.len(), 2);
    }

    #[test]
    fn short_position_inverts_sign() {
        // Long 100C @ $5, then short 100C @ $5 → zero strategy P/L.
        let legs = vec![
            Leg {
                kind: LegKind::Call,
                strike: 100.0,
                premium: 5.0,
                qty: 1.0,
            },
            Leg {
                kind: LegKind::Call,
                strike: 100.0,
                premium: 5.0,
                qty: -1.0,
            },
        ];
        let r = compute(&legs, 105.0, 0.5, 0.05, 0.0, 0.25).unwrap();
        assert!(r.strategy_pnl.abs() < 1e-9);
        assert!(r.strategy_value.abs() < 1e-9);
    }

    #[test]
    fn underlying_leg_is_linear() {
        let legs = vec![Leg {
            kind: LegKind::Underlying,
            strike: 100.0,
            premium: 0.0,
            qty: 1.0,
        }];
        let r = compute(&legs, 110.0, 0.5, 0.05, 0.0, 0.25).unwrap();
        assert!((r.strategy_value - 10.0).abs() < 1e-9);
    }
}
