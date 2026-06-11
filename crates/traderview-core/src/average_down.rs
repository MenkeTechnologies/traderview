//! Average-down / add-to-position blend — what the add actually buys.
//!
//! Blends the existing lot with the new one and reports the new
//! average cost, the bounce needed to break even BEFORE vs AFTER the
//! add, and the capital-at-risk growth. Averaging down always lowers
//! the breakeven bounce — the question the report answers is by how
//! much, and at what increase in exposure.
//!
//! Pure compute. Companion to `scale_out_planner` (the exit-side
//! mirror), `cost_basis`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AverageDownInput {
    pub current_shares: f64,
    pub current_avg_cost: f64,
    pub add_shares: f64,
    pub add_price: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AverageDownReport {
    pub new_shares: f64,
    pub new_avg_cost: f64,
    /// Bounce from the add price back to breakeven, % — before and
    /// after the add.
    pub bounce_needed_before_pct: f64,
    pub bounce_needed_after_pct: f64,
    pub capital_added: f64,
    pub total_capital: f64,
    /// Exposure growth from the add, %.
    pub exposure_increase_pct: f64,
    /// Unrealized P/L at the add price, before the add.
    pub unrealized_at_add: f64,
}

pub fn compute(inp: &AverageDownInput) -> Option<AverageDownReport> {
    if ![
        inp.current_shares,
        inp.current_avg_cost,
        inp.add_shares,
        inp.add_price,
    ]
    .iter()
    .all(|v| v.is_finite() && *v > 0.0)
    {
        return None;
    }
    let new_shares = inp.current_shares + inp.add_shares;
    let old_capital = inp.current_shares * inp.current_avg_cost;
    let added = inp.add_shares * inp.add_price;
    let new_avg = (old_capital + added) / new_shares;
    Some(AverageDownReport {
        new_shares,
        new_avg_cost: new_avg,
        bounce_needed_before_pct: (inp.current_avg_cost / inp.add_price - 1.0) * 100.0,
        bounce_needed_after_pct: (new_avg / inp.add_price - 1.0) * 100.0,
        capital_added: added,
        total_capital: old_capital + added,
        exposure_increase_pct: added / old_capital * 100.0,
        unrealized_at_add: (inp.add_price - inp.current_avg_cost) * inp.current_shares,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_size_add_hand_walk() {
        // 100 @ $50, add 100 @ $40: avg $45; breakeven bounce falls
        // from +25% to +12.5%; exposure +80% of original capital.
        let r = compute(&AverageDownInput {
            current_shares: 100.0,
            current_avg_cost: 50.0,
            add_shares: 100.0,
            add_price: 40.0,
        })
        .unwrap();
        assert!((r.new_avg_cost - 45.0).abs() < 1e-12);
        assert!((r.bounce_needed_before_pct - 25.0).abs() < 1e-12);
        assert!((r.bounce_needed_after_pct - 12.5).abs() < 1e-12);
        assert!((r.capital_added - 4000.0).abs() < 1e-12);
        assert!((r.total_capital - 9000.0).abs() < 1e-12);
        assert!((r.exposure_increase_pct - 80.0).abs() < 1e-12);
        assert!((r.unrealized_at_add + 1000.0).abs() < 1e-12);
    }

    #[test]
    fn averaging_up_raises_the_basis() {
        // Adds above basis are also legal: avg rises, breakeven bounce
        // goes NEGATIVE (already above water at the add price).
        let r = compute(&AverageDownInput {
            current_shares: 100.0,
            current_avg_cost: 50.0,
            add_shares: 50.0,
            add_price: 60.0,
        })
        .unwrap();
        assert!(r.new_avg_cost > 50.0 && r.new_avg_cost < 60.0);
        assert!(r.bounce_needed_after_pct < 0.0);
        assert!(r.unrealized_at_add > 0.0);
    }

    #[test]
    fn tiny_add_barely_moves_the_average()
    {
        let r = compute(&AverageDownInput {
            current_shares: 1000.0,
            current_avg_cost: 50.0,
            add_shares: 1.0,
            add_price: 40.0,
        })
        .unwrap();
        assert!((r.new_avg_cost - 50.0).abs() < 0.02);
        // The bounce barely improves — small adds don't rescue basis.
        assert!(r.bounce_needed_after_pct > 24.9);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&AverageDownInput {
            current_shares: 0.0,
            current_avg_cost: 50.0,
            add_shares: 100.0,
            add_price: 40.0,
        })
        .is_none());
        assert!(compute(&AverageDownInput {
            current_shares: 100.0,
            current_avg_cost: 50.0,
            add_shares: 100.0,
            add_price: f64::NAN,
        })
        .is_none());
    }
}
