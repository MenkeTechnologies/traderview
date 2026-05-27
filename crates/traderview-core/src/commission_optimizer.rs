//! Commission / fee analyzer.
//!
//! For a trader running ~100 trades/month, the difference between
//! $0.005/share and $0.0035/share is real money. This module compares
//! the user's actual commission profile against alternative pricing
//! tiers and shows the cost-per-fill + cost-per-$-traded breakdown.
//!
//! Pure compute. Caller supplies a list of (qty, fee) per execution +
//! the candidate tiers to compare.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Execution {
    pub qty: Decimal,
    pub notional: Decimal,
    pub actual_fee: Decimal,
}

/// One candidate commission tier the user wants to evaluate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier {
    pub name: String,
    /// Fixed dollar fee per trade (e.g. $1 per fill).
    pub per_trade_flat: Decimal,
    /// Per-share rate (e.g. $0.005/sh).
    pub per_share: Decimal,
    /// Per-dollar rate as decimal (e.g. 0.000119 for IBKR Lite).
    pub per_dollar: Decimal,
    /// Floor / cap per trade (cents to dollars).
    pub min_per_trade: Decimal,
    pub max_per_trade: Decimal,
}

impl Tier {
    pub fn fee_for(&self, ex: &Execution) -> Decimal {
        let raw = self.per_trade_flat
            + self.per_share * ex.qty
            + self.per_dollar * ex.notional;
        let bounded = if self.min_per_trade > Decimal::ZERO && raw < self.min_per_trade {
            self.min_per_trade
        } else if self.max_per_trade > Decimal::ZERO && raw > self.max_per_trade {
            self.max_per_trade
        } else {
            raw
        };
        bounded
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierResult {
    pub tier: String,
    pub total_fee: Decimal,
    pub fee_per_trade: Decimal,
    pub fee_per_share: Decimal,
    pub fee_pct_of_notional: f64,
    /// Delta vs the user's actual total. Negative = savings vs actual.
    pub delta_vs_actual: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizerReport {
    pub trade_count: usize,
    pub total_shares: Decimal,
    pub total_notional: Decimal,
    pub actual_total_fee: Decimal,
    /// Each candidate tier scored. Sorted by total_fee ascending —
    /// cheapest first.
    pub tiers: Vec<TierResult>,
    /// Best alternative (sorted[0]) if it beats `actual`. None if user's
    /// actual is already optimal among candidates.
    pub best_alternative: Option<String>,
    /// Annual projected savings if user switches to best alternative
    /// and trade volume stays constant (extrapolated × 12).
    pub projected_annual_savings: Decimal,
}

pub fn evaluate(execs: &[Execution], tiers: &[Tier]) -> OptimizerReport {
    if execs.is_empty() {
        return OptimizerReport::default();
    }
    let trade_count = execs.len();
    let total_shares: Decimal = execs.iter().map(|e| e.qty).sum();
    let total_notional: Decimal = execs.iter().map(|e| e.notional).sum();
    let actual_total: Decimal = execs.iter().map(|e| e.actual_fee).sum();

    let mut results: Vec<TierResult> = tiers.iter().map(|t| {
        let total: Decimal = execs.iter().map(|e| t.fee_for(e)).sum();
        let per_trade = total / Decimal::from(trade_count as u64);
        let per_share = if total_shares.is_zero() {
            Decimal::ZERO
        } else {
            total / total_shares
        };
        let pct = if total_notional.is_zero() {
            0.0
        } else {
            to_f64(total) / to_f64(total_notional) * 100.0
        };
        TierResult {
            tier: t.name.clone(),
            total_fee: total,
            fee_per_trade: per_trade,
            fee_per_share: per_share,
            fee_pct_of_notional: pct,
            delta_vs_actual: total - actual_total,
        }
    }).collect();
    results.sort_by(|a, b| a.total_fee.cmp(&b.total_fee));

    let (best_alternative, annual_savings) = if let Some(best) = results.first() {
        if best.delta_vs_actual < Decimal::ZERO {
            // Negative delta = savings. Project annual.
            (Some(best.tier.clone()), -best.delta_vs_actual * Decimal::from(12))
        } else {
            (None, Decimal::ZERO)
        }
    } else {
        (None, Decimal::ZERO)
    };

    OptimizerReport {
        trade_count,
        total_shares,
        total_notional,
        actual_total_fee: actual_total,
        tiers: results,
        best_alternative,
        projected_annual_savings: annual_savings,
    }
}

fn to_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

/// Common shipped tiers for the calculator UI to pre-populate.
pub fn default_tiers() -> Vec<Tier> {
    let d = |s: &str| Decimal::from_str(s).unwrap();
    vec![
        Tier {
            name: "IBKR Pro tiered".into(),
            per_trade_flat: Decimal::ZERO,
            per_share: d("0.0035"),
            per_dollar: Decimal::ZERO,
            min_per_trade: d("0.35"),
            // IBKR's real cap is 1% of trade value, which is notional-relative
            // and doesn't fit a single per-trade dollar field. The ceiling is
            // almost never the binding constraint for retail-size orders, so
            // we leave it disabled here — Tier consumers wanting a 1%-of-
            // notional cap should model it through `per_dollar` instead.
            max_per_trade: Decimal::ZERO,
        },
        Tier {
            name: "Lightspeed Active".into(),
            per_trade_flat: Decimal::ZERO,
            per_share: d("0.0045"),
            per_dollar: Decimal::ZERO,
            min_per_trade: d("1.00"),
            max_per_trade: Decimal::ZERO,
        },
        Tier {
            name: "Webull (zero-commission)".into(),
            per_trade_flat: Decimal::ZERO,
            per_share: Decimal::ZERO,
            per_dollar: Decimal::ZERO,
            min_per_trade: Decimal::ZERO,
            max_per_trade: Decimal::ZERO,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn ex(qty: &str, notional: &str, fee: &str) -> Execution {
        Execution { qty: d(qty), notional: d(notional), actual_fee: d(fee) }
    }

    #[test]
    fn empty_input_returns_default() {
        let r = evaluate(&[], &default_tiers());
        assert_eq!(r.trade_count, 0);
        assert!(r.best_alternative.is_none());
    }

    #[test]
    fn zero_commission_tier_beats_paid_actual() {
        // 100 trades, 100 sh each at $1 fee. Webull zero would save $100/mo.
        let execs: Vec<_> = (0..100)
            .map(|_| ex("100", "5000", "1.00"))
            .collect();
        let r = evaluate(&execs, &default_tiers());
        assert_eq!(r.actual_total_fee, d("100.00"));
        // Webull tier should win — total fee = 0.
        let webull = r.tiers.iter().find(|t| t.tier.contains("Webull")).unwrap();
        assert_eq!(webull.total_fee, Decimal::ZERO);
        assert_eq!(webull.delta_vs_actual, d("-100.00"));
        assert_eq!(r.best_alternative.as_deref(), Some("Webull (zero-commission)"));
        assert_eq!(r.projected_annual_savings, d("1200.00"));
    }

    #[test]
    fn ibkr_floor_kicks_in_for_tiny_orders() {
        // 10 shares × $0.0035 = $0.035 raw — floor at $0.35.
        let t = &default_tiers()[0];   // IBKR Pro
        let fee = t.fee_for(&ex("10", "500", "0"));
        assert_eq!(fee, d("0.35"));
    }

    #[test]
    fn ibkr_per_share_for_large_orders() {
        // 10,000 shares × $0.0035 = $35 raw — above floor $0.35.
        let t = &default_tiers()[0];
        let fee = t.fee_for(&ex("10000", "500000", "0"));
        assert_eq!(fee, d("35.0000"));
    }

    #[test]
    fn lightspeed_one_dollar_floor() {
        let t = &default_tiers()[1];
        // 100 sh × $0.0045 = $0.45 raw — floor $1.
        let fee = t.fee_for(&ex("100", "5000", "0"));
        assert_eq!(fee, d("1.00"));
    }

    #[test]
    fn user_already_optimal_returns_none_best() {
        // Actual fee is the cheapest (zero). No tier beats it.
        let execs = vec![ex("100", "5000", "0")];
        let r = evaluate(&execs, &default_tiers());
        // Webull tier ties at zero — delta is zero, not negative, so no
        // strict savings → no recommendation.
        assert!(r.best_alternative.is_none());
        assert_eq!(r.projected_annual_savings, Decimal::ZERO);
    }

    #[test]
    fn tiers_sorted_cheapest_first() {
        let execs = vec![ex("100", "5000", "1")];
        let r = evaluate(&execs, &default_tiers());
        // Webull (0) < IBKR ($0.35) < Lightspeed ($1).
        assert_eq!(r.tiers[0].tier, "Webull (zero-commission)");
        assert_eq!(r.tiers.last().unwrap().tier, "Lightspeed Active");
    }

    #[test]
    fn fee_pct_of_notional_computed_correctly() {
        // $1 fee on $1000 notional = 0.1%.
        let execs = vec![ex("10", "1000", "1.00")];
        let r = evaluate(&execs, &[Tier {
            name: "test".into(),
            per_trade_flat: d("1.00"),
            per_share: Decimal::ZERO,
            per_dollar: Decimal::ZERO,
            min_per_trade: Decimal::ZERO,
            max_per_trade: Decimal::ZERO,
        }]);
        assert!((r.tiers[0].fee_pct_of_notional - 0.1).abs() < 1e-9);
    }
}
