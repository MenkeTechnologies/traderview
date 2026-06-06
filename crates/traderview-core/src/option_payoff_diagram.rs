//! Option Payoff Diagram — terminal-payoff P/L of a multi-leg option
//! strategy across a grid of underlying prices.
//!
//! Each leg is a long (qty > 0) or short (qty < 0) position in a call,
//! put, or the underlying itself. Each leg has a premium paid (positive)
//! or received (negative) at trade inception.
//!
//! At expiration the intrinsic payoff for one contract is:
//!
//!   call:       max(S - K, 0)
//!   put:        max(K - S, 0)
//!   underlying: S - K     (K = entry price, no kink)
//!
//! Total per-contract P/L for a leg = qty · (intrinsic - premium).
//! Strategy P/L at S = Σ over legs.
//!
//! Pure compute. Companion to `black_scholes`, `greeks`, `option_chain`,
//! `option_strategy_screener`.

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
pub struct PayoffPoint {
    pub spot: f64,
    pub pnl: f64,
}

#[derive(Debug)]
pub struct Report {
    pub points: Vec<PayoffPoint>,
    pub max_profit: f64,
    pub max_loss: f64,
    pub breakevens: Vec<f64>,
}

pub fn compute(legs: &[Leg], spot_min: f64, spot_max: f64, steps: usize) -> Option<Report> {
    if legs.is_empty() || steps < 2 {
        return None;
    }
    if !spot_min.is_finite() || !spot_max.is_finite() {
        return None;
    }
    if spot_min >= spot_max {
        return None;
    }
    for l in legs {
        if !l.strike.is_finite() || !l.premium.is_finite() || !l.qty.is_finite() {
            return None;
        }
    }
    let step = (spot_max - spot_min) / (steps as f64 - 1.0);
    let mut points = Vec::with_capacity(steps);
    let mut max_p = f64::NEG_INFINITY;
    let mut max_l = f64::INFINITY;
    for i in 0..steps {
        let s = spot_min + step * i as f64;
        let pnl = pnl_at(legs, s);
        if pnl > max_p {
            max_p = pnl;
        }
        if pnl < max_l {
            max_l = pnl;
        }
        points.push(PayoffPoint { spot: s, pnl });
    }
    // Breakevens: zero-crossings between adjacent points.
    let mut breakevens = Vec::new();
    for i in 1..points.len() {
        let a = &points[i - 1];
        let b = &points[i];
        if (a.pnl == 0.0) && !breakevens.contains(&a.spot) {
            breakevens.push(a.spot);
        }
        if a.pnl.signum() != b.pnl.signum() && a.pnl != 0.0 && b.pnl != 0.0 {
            let t = -a.pnl / (b.pnl - a.pnl);
            let s = a.spot + t * (b.spot - a.spot);
            breakevens.push(s);
        }
    }
    Some(Report {
        points,
        max_profit: max_p,
        max_loss: max_l,
        breakevens,
    })
}

fn pnl_at(legs: &[Leg], s: f64) -> f64 {
    legs.iter()
        .map(|l| {
            let intrinsic = match l.kind {
                LegKind::Call => (s - l.strike).max(0.0),
                LegKind::Put => (l.strike - s).max(0.0),
                LegKind::Underlying => s - l.strike,
            };
            l.qty * (intrinsic - l.premium)
        })
        .sum()
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
        assert!(compute(&[], 80.0, 120.0, 100).is_none());
        assert!(compute(&legs, 100.0, 100.0, 100).is_none());
        assert!(compute(&legs, 80.0, 120.0, 1).is_none());
        assert!(compute(&legs, f64::NAN, 120.0, 10).is_none());
    }

    #[test]
    fn long_call_breakeven_at_strike_plus_premium() {
        // Long 100C @ $5: breakeven = $105.
        let legs = vec![Leg {
            kind: LegKind::Call,
            strike: 100.0,
            premium: 5.0,
            qty: 1.0,
        }];
        let r = compute(&legs, 80.0, 130.0, 501).unwrap();
        assert!(r.breakevens.iter().any(|b| (b - 105.0).abs() < 0.5));
    }

    #[test]
    fn long_put_breakeven_at_strike_minus_premium() {
        let legs = vec![Leg {
            kind: LegKind::Put,
            strike: 100.0,
            premium: 5.0,
            qty: 1.0,
        }];
        let r = compute(&legs, 70.0, 120.0, 501).unwrap();
        assert!(r.breakevens.iter().any(|b| (b - 95.0).abs() < 0.5));
    }

    #[test]
    fn bull_call_spread_has_capped_profit_and_loss() {
        // Long 100C @ $5, Short 110C @ $2 → max profit $8, max loss $3.
        let legs = vec![
            Leg {
                kind: LegKind::Call,
                strike: 100.0,
                premium: 5.0,
                qty: 1.0,
            },
            Leg {
                kind: LegKind::Call,
                strike: 110.0,
                premium: 2.0,
                qty: -1.0,
            },
        ];
        let r = compute(&legs, 80.0, 130.0, 501).unwrap();
        assert!((r.max_profit - 7.0).abs() < 0.1);
        assert!((r.max_loss + 3.0).abs() < 0.1);
    }

    #[test]
    fn straddle_has_two_breakevens() {
        // Long 100C @ $5 + Long 100P @ $5: breakevens $90 and $110.
        let legs = vec![
            Leg {
                kind: LegKind::Call,
                strike: 100.0,
                premium: 5.0,
                qty: 1.0,
            },
            Leg {
                kind: LegKind::Put,
                strike: 100.0,
                premium: 5.0,
                qty: 1.0,
            },
        ];
        let r = compute(&legs, 70.0, 130.0, 601).unwrap();
        assert_eq!(r.breakevens.len(), 2);
        assert!(r.breakevens.iter().any(|b| (b - 90.0).abs() < 0.5));
        assert!(r.breakevens.iter().any(|b| (b - 110.0).abs() < 0.5));
    }

    #[test]
    fn underlying_leg_acts_linear() {
        // Long 1 share entry $100: P/L = S - 100 (premium term = 0).
        let legs = vec![Leg {
            kind: LegKind::Underlying,
            strike: 100.0,
            premium: 0.0,
            qty: 1.0,
        }];
        let r = compute(&legs, 90.0, 110.0, 21).unwrap();
        assert!((r.points[0].pnl + 10.0).abs() < 1e-9);
        assert!((r.points[20].pnl - 10.0).abs() < 1e-9);
    }

    #[test]
    fn point_count_matches_steps() {
        let legs = vec![Leg {
            kind: LegKind::Call,
            strike: 100.0,
            premium: 5.0,
            qty: 1.0,
        }];
        let r = compute(&legs, 80.0, 120.0, 100).unwrap();
        assert_eq!(r.points.len(), 100);
    }
}
