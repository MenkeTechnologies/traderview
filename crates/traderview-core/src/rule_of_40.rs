//! Rule of 40 — the SaaS/growth-company health check: revenue growth rate plus
//! profit margin should sum to at least 40%. It trades growth against
//! profitability, so a company can pass by growing fast at a loss or growing
//! slowly with fat margins.
//!
//! ```text
//! score = revenue growth % + profit margin %
//! passes when score ≥ 40
//! ```
//!
//! The margin can be whatever the analyst uses — FCF, EBITDA, or net — it's
//! just the percent that pairs with growth.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RuleOf40Input {
    /// Year-over-year revenue growth, percent.
    pub revenue_growth_pct: f64,
    /// Profit margin (FCF / EBITDA / net), percent.
    pub profit_margin_pct: f64,
    /// The bar to clear; defaults to 40.
    #[serde(default = "default_target")]
    pub target_pct: f64,
}

fn default_target() -> f64 {
    40.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RuleOf40Result {
    /// growth + margin.
    pub score_pct: f64,
    /// Whether the score meets or beats the target.
    pub passes: bool,
    /// score − target (positive = surplus, negative = shortfall).
    pub surplus_pct: f64,
    /// Margin you'd need at the current growth to hit the target.
    pub margin_needed_pct: f64,
    /// Growth you'd need at the current margin to hit the target.
    pub growth_needed_pct: f64,
    /// Growth's share of the score, percent; `None` if the score is ≤ 0.
    pub growth_share_pct: Option<f64>,
    /// Margin's share of the score, percent; `None` if the score is ≤ 0.
    pub margin_share_pct: Option<f64>,
}

pub fn analyze(input: &RuleOf40Input) -> RuleOf40Result {
    let score = input.revenue_growth_pct + input.profit_margin_pct;
    let target = input.target_pct;

    let (growth_share, margin_share) = if score > 0.0 {
        (
            Some(input.revenue_growth_pct / score * 100.0),
            Some(input.profit_margin_pct / score * 100.0),
        )
    } else {
        (None, None)
    };

    RuleOf40Result {
        score_pct: score,
        passes: score >= target,
        surplus_pct: score - target,
        margin_needed_pct: target - input.revenue_growth_pct,
        growth_needed_pct: target - input.profit_margin_pct,
        growth_share_pct: growth_share,
        margin_share_pct: margin_share,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(growth: f64, margin: f64) -> RuleOf40Result {
        analyze(&RuleOf40Input {
            revenue_growth_pct: growth,
            profit_margin_pct: margin,
            target_pct: 40.0,
        })
    }

    #[test]
    fn score_is_sum() {
        let r = run(30.0, 15.0);
        assert!(close(r.score_pct, 45.0));
    }

    #[test]
    fn passes_above_target() {
        let r = run(30.0, 15.0);
        assert!(r.passes);
        assert!(close(r.surplus_pct, 5.0));
    }

    #[test]
    fn fails_below_target() {
        let r = run(20.0, 10.0);
        assert!(!r.passes);
        assert!(close(r.surplus_pct, -10.0));
    }

    #[test]
    fn exactly_forty_passes() {
        let r = run(25.0, 15.0);
        assert!(close(r.score_pct, 40.0));
        assert!(r.passes);
    }

    #[test]
    fn margin_needed_at_current_growth() {
        let r = run(25.0, 5.0);
        // Need 40 − 25 = 15% margin to hit 40 at this growth.
        assert!(close(r.margin_needed_pct, 15.0));
    }

    #[test]
    fn growth_needed_at_current_margin() {
        let r = run(10.0, 12.0);
        // Need 40 − 12 = 28% growth at this margin.
        assert!(close(r.growth_needed_pct, 28.0));
    }

    #[test]
    fn hypergrowth_at_a_loss_passes() {
        // 60% growth, −15% margin → 45 ≥ 40.
        let r = run(60.0, -15.0);
        assert!(r.passes);
        assert!(close(r.score_pct, 45.0));
    }

    #[test]
    fn shares_sum_to_100_and_guard_nonpositive() {
        let r = run(30.0, 15.0);
        assert!(close(
            r.growth_share_pct.unwrap() + r.margin_share_pct.unwrap(),
            100.0
        ));
        // Negative total score → shares undefined.
        let neg = run(-30.0, 10.0);
        assert!(neg.growth_share_pct.is_none());
        assert!(neg.margin_share_pct.is_none());
    }
}
