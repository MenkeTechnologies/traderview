//! Alpha-vs-cost horizon — how long a signal must be held before it
//! pays for its own execution.
//!
//!   breakeven days   = round-trip cost / alpha per day
//!   capacity         = how many round trips per year the alpha funds
//!   net alpha        = gross − cost × turnover
//!
//! A 2bp/day signal with 10bp round-trip costs needs five days of
//! holding; trading it daily hands the whole edge to the market
//! makers. The flip side: cost says nothing until divided by alpha
//! velocity.
//!
//! Pure compute. Companion to `commission_optimizer`,
//! `implementation shortfall` analytics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AlphaHorizonInput {
    /// Expected edge while the position is on, bp/day.
    pub alpha_bp_per_day: f64,
    /// All-in round-trip cost (spread + impact + fees), bp.
    pub round_trip_cost_bp: f64,
    /// Intended holding period, days.
    pub holding_days: f64,
    /// Trading days per year.
    #[serde(default = "default_days_per_year")]
    pub days_per_year: f64,
}

fn default_days_per_year() -> f64 {
    252.0
}

#[derive(Debug, Clone, Serialize)]
pub struct AlphaHorizonReport {
    /// Days of alpha needed to cover one round trip.
    pub breakeven_days: f64,
    /// Holding clears the cost at the intended horizon.
    pub viable: bool,
    /// Net edge per round trip at the intended holding, bp.
    pub net_alpha_per_trade_bp: f64,
    /// Net annualized edge at this cadence, bp/yr.
    pub net_alpha_annual_bp: f64,
    /// Share of gross alpha consumed by costs at this cadence, %.
    pub cost_share_pct: f64,
}

pub fn compute(inp: &AlphaHorizonInput) -> Option<AlphaHorizonReport> {
    if !inp.alpha_bp_per_day.is_finite()
        || inp.alpha_bp_per_day <= 0.0
        || !inp.round_trip_cost_bp.is_finite()
        || inp.round_trip_cost_bp < 0.0
        || !inp.holding_days.is_finite()
        || inp.holding_days <= 0.0
        || !inp.days_per_year.is_finite()
        || inp.days_per_year <= 0.0
    {
        return None;
    }
    let breakeven = inp.round_trip_cost_bp / inp.alpha_bp_per_day;
    let gross_per_trade = inp.alpha_bp_per_day * inp.holding_days;
    let net_per_trade = gross_per_trade - inp.round_trip_cost_bp;
    let trades_per_year = inp.days_per_year / inp.holding_days;
    Some(AlphaHorizonReport {
        breakeven_days: breakeven,
        viable: net_per_trade > 0.0,
        net_alpha_per_trade_bp: net_per_trade,
        net_alpha_annual_bp: net_per_trade * trades_per_year,
        cost_share_pct: if gross_per_trade > 0.0 {
            (inp.round_trip_cost_bp / gross_per_trade * 100.0).min(1000.0)
        } else {
            0.0
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_bp_signal_against_ten_bp_costs() {
        // Breakeven 5 days; held 10 days nets 10bp/trade, 25.2
        // trades/yr ⇒ 252bp/yr net, costs eating 50% of gross.
        let r = compute(&AlphaHorizonInput {
            alpha_bp_per_day: 2.0,
            round_trip_cost_bp: 10.0,
            holding_days: 10.0,
            days_per_year: 252.0,
        })
        .unwrap();
        assert!((r.breakeven_days - 5.0).abs() < 1e-12);
        assert!(r.viable);
        assert!((r.net_alpha_per_trade_bp - 10.0).abs() < 1e-12);
        assert!((r.net_alpha_annual_bp - 252.0).abs() < 1e-9);
        assert!((r.cost_share_pct - 50.0).abs() < 1e-12);
    }

    #[test]
    fn day_trading_the_same_signal_hands_the_edge_to_costs() {
        let r = compute(&AlphaHorizonInput {
            alpha_bp_per_day: 2.0,
            round_trip_cost_bp: 10.0,
            holding_days: 1.0,
            days_per_year: 252.0,
        })
        .unwrap();
        assert!(!r.viable);
        assert!((r.net_alpha_per_trade_bp + 8.0).abs() < 1e-12);
        assert!(r.net_alpha_annual_bp < 0.0); // negative at scale
    }

    #[test]
    fn free_execution_is_always_viable() {
        let r = compute(&AlphaHorizonInput {
            alpha_bp_per_day: 1.0,
            round_trip_cost_bp: 0.0,
            holding_days: 1.0,
            days_per_year: 252.0,
        })
        .unwrap();
        assert_eq!(r.breakeven_days, 0.0);
        assert!(r.viable);
        assert!((r.net_alpha_annual_bp - 252.0).abs() < 1e-9);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&AlphaHorizonInput {
            alpha_bp_per_day: 0.0,
            round_trip_cost_bp: 10.0,
            holding_days: 5.0,
            days_per_year: 252.0,
        })
        .is_none());
        assert!(compute(&AlphaHorizonInput {
            alpha_bp_per_day: 2.0,
            round_trip_cost_bp: -1.0,
            holding_days: 5.0,
            days_per_year: 252.0,
        })
        .is_none());
        assert!(compute(&AlphaHorizonInput {
            alpha_bp_per_day: 2.0,
            round_trip_cost_bp: 10.0,
            holding_days: 0.0,
            days_per_year: 252.0,
        })
        .is_none());
    }
}
