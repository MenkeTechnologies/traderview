//! Multi-leg option spread P&L at expiration.
//!
//! For a strategy of N legs (each call or put, long or short, at a
//! strike), compute the payoff curve across a price range. Powers the
//! payoff-diagram chart for any of the canonical spreads:
//!   - Long call / put
//!   - Vertical spreads (bull/bear call/put)
//!   - Iron condor / iron butterfly
//!   - Straddle / strangle
//!   - Calendar / diagonal (use intrinsic-only — caller adds extrinsic separately)
//!
//! Each leg contributes:
//!   call: max(price - strike, 0) × qty (positive for long, negative for short)
//!   put:  max(strike - price, 0) × qty
//!
//! Plus the net premium paid/received per share. Multiplier (typically
//! 100) applied at the report level. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leg {
    pub kind: OptionKind,
    pub strike: f64,
    /// Signed contracts: positive = long, negative = short.
    pub contracts: i64,
    /// ABSOLUTE premium per share (always positive). The sign of cash
    /// flow comes from `contracts`: long (positive contracts) = trader
    /// PAYS premium; short (negative contracts) = trader RECEIVES.
    pub premium_per_share: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoffPoint {
    pub price: f64,
    pub pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PayoffReport {
    pub points: Vec<PayoffPoint>,
    pub max_profit: f64,
    pub max_loss: f64,
    pub breakevens: Vec<f64>,
    pub net_debit: f64,
}

pub fn payoff(
    legs: &[Leg],
    price_low: f64,
    price_high: f64,
    steps: usize,
    multiplier: f64,
) -> PayoffReport {
    let mut report = PayoffReport::default();
    if legs.is_empty() || steps == 0 || price_high <= price_low {
        return report;
    }
    let net_debit_per_share: f64 = legs
        .iter()
        .map(|l| l.premium_per_share * l.contracts as f64)
        .sum();
    report.net_debit = net_debit_per_share * multiplier;
    let step = (price_high - price_low) / steps as f64;
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let price = price_low + step * i as f64;
        let mut pnl_per_share = -net_debit_per_share;
        for leg in legs {
            let intrinsic = match leg.kind {
                OptionKind::Call => (price - leg.strike).max(0.0),
                OptionKind::Put => (leg.strike - price).max(0.0),
            };
            pnl_per_share += intrinsic * leg.contracts as f64;
        }
        points.push(PayoffPoint {
            price,
            pnl: pnl_per_share * multiplier,
        });
    }
    report.max_profit = points
        .iter()
        .map(|p| p.pnl)
        .fold(f64::NEG_INFINITY, f64::max);
    report.max_loss = points.iter().map(|p| p.pnl).fold(f64::INFINITY, f64::min);
    // Breakeven detection: sign-change between consecutive points, with
    // de-dup when sampling lands a point exactly on zero (which would
    // make both adjacent windows trigger).
    let mut last_be: Option<f64> = None;
    for w in points.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let product = a.pnl * b.pnl;
        let crosses =
            product < 0.0 || (a.pnl == 0.0 && b.pnl != 0.0) || (b.pnl == 0.0 && a.pnl != 0.0);
        if !crosses {
            continue;
        }
        let span = b.pnl - a.pnl;
        let cross = if span == 0.0 {
            a.price
        } else {
            a.price + (0.0 - a.pnl) / span * (b.price - a.price)
        };
        // Skip if same as previous breakeven (exact zero straddle case).
        let dx = (b.price - a.price).abs();
        if let Some(last) = last_be {
            if (cross - last).abs() < dx * 1.5 {
                continue;
            }
        }
        report.breakevens.push(cross);
        last_be = Some(cross);
    }
    report.points = points;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn long_call(strike: f64, premium: f64) -> Leg {
        Leg {
            kind: OptionKind::Call,
            strike,
            contracts: 1,
            premium_per_share: premium,
        }
    }
    fn short_call(strike: f64, premium: f64) -> Leg {
        Leg {
            kind: OptionKind::Call,
            strike,
            contracts: -1,
            premium_per_share: premium,
        }
    }
    fn long_put(strike: f64, premium: f64) -> Leg {
        Leg {
            kind: OptionKind::Put,
            strike,
            contracts: 1,
            premium_per_share: premium,
        }
    }

    #[test]
    fn empty_legs_returns_default() {
        let r = payoff(&[], 90.0, 110.0, 10, 100.0);
        assert!(r.points.is_empty());
    }

    #[test]
    fn long_call_breakeven_above_strike_by_premium() {
        // Long $100 call at $5 premium → breakeven at $105.
        let legs = vec![long_call(100.0, 5.0)];
        let r = payoff(&legs, 90.0, 120.0, 300, 100.0);
        assert_eq!(r.breakevens.len(), 1);
        assert!((r.breakevens[0] - 105.0).abs() < 0.5);
    }

    #[test]
    fn long_call_max_loss_is_premium_paid() {
        // Below strike: lose entire premium ($5 × 100 = $500).
        let legs = vec![long_call(100.0, 5.0)];
        let r = payoff(&legs, 90.0, 95.0, 10, 100.0);
        assert!((r.max_loss + 500.0).abs() < 1e-6);
    }

    #[test]
    fn long_put_breakeven_below_strike_by_premium() {
        // Long $100 put at $5 → breakeven at $95.
        let legs = vec![long_put(100.0, 5.0)];
        let r = payoff(&legs, 80.0, 110.0, 300, 100.0);
        assert_eq!(r.breakevens.len(), 1);
        assert!((r.breakevens[0] - 95.0).abs() < 0.5);
    }

    #[test]
    fn bull_call_spread_has_two_breakevens_only_when_lower_strike_in_range() {
        // Bull call: long $100 @ $5, short $110 @ $1 (credit). Net debit $4.
        // Breakeven: $104.
        let legs = vec![long_call(100.0, 5.0), short_call(110.0, 1.0)];
        let r = payoff(&legs, 90.0, 120.0, 600, 100.0);
        assert!(!r.breakevens.is_empty());
        assert!(r.breakevens.iter().any(|b| (b - 104.0).abs() < 0.5));
        // Max profit = (10 - 4) × 100 = $600 at $110+.
        assert!((r.max_profit - 600.0).abs() < 1e-6);
        // Max loss = -$4 × 100 = -$400 below $100.
        assert!((r.max_loss + 400.0).abs() < 1e-6);
    }

    #[test]
    fn straddle_two_breakevens_around_strike() {
        // Long $100 call @ $5 + long $100 put @ $5. Net debit $10. Two BEs: $90, $110.
        let legs = vec![long_call(100.0, 5.0), long_put(100.0, 5.0)];
        let r = payoff(&legs, 80.0, 120.0, 800, 100.0);
        assert_eq!(r.breakevens.len(), 2);
        assert!((r.breakevens[0] - 90.0).abs() < 0.5);
        assert!((r.breakevens[1] - 110.0).abs() < 0.5);
    }

    #[test]
    fn iron_condor_has_two_breakevens_max_loss_at_wings() {
        // 5-wide iron condor centered around $100: long $90P / short $95P
        // / short $105C / long $110C. Net credit $2 each wing = $4 total.
        let legs = vec![
            Leg {
                kind: OptionKind::Put,
                strike: 90.0,
                contracts: 1,
                premium_per_share: 1.0,
            },
            Leg {
                kind: OptionKind::Put,
                strike: 95.0,
                contracts: -1,
                premium_per_share: 3.0,
            },
            Leg {
                kind: OptionKind::Call,
                strike: 105.0,
                contracts: -1,
                premium_per_share: 3.0,
            },
            Leg {
                kind: OptionKind::Call,
                strike: 110.0,
                contracts: 1,
                premium_per_share: 1.0,
            },
        ];
        let r = payoff(&legs, 80.0, 120.0, 800, 100.0);
        assert_eq!(r.breakevens.len(), 2);
        // Max profit (collect $4 × 100 = $400) between $95 and $105.
        assert!((r.max_profit - 400.0).abs() < 1e-6);
    }

    #[test]
    fn net_debit_positive_for_debit_strategies() {
        // Long call alone: net debit = premium paid.
        let legs = vec![long_call(100.0, 5.0)];
        let r = payoff(&legs, 90.0, 110.0, 100, 100.0);
        assert_eq!(r.net_debit, 500.0);
    }

    #[test]
    fn net_debit_negative_for_credit_strategies() {
        // Short put alone: $2 credit received per share × 100 = $200 credit
        // → net_debit field becomes -$200 (negative debit = credit).
        let legs = vec![Leg {
            kind: OptionKind::Put,
            strike: 100.0,
            contracts: -1,
            premium_per_share: 2.0,
        }];
        let r = payoff(&legs, 90.0, 110.0, 100, 100.0);
        assert_eq!(r.net_debit, -200.0);
    }

    #[test]
    fn zero_steps_returns_empty() {
        let legs = vec![long_call(100.0, 5.0)];
        let r = payoff(&legs, 90.0, 110.0, 0, 100.0);
        assert!(r.points.is_empty());
    }
}
