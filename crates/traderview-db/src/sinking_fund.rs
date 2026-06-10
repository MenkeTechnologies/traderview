//! Sinking-fund planner — multi-goal monthly-contribution allocator.
//!
//! Classic personal-finance pattern: set aside a fixed dollar amount
//! each month into named buckets so the money is there when the
//! lumpy expense arrives (Christmas, car insurance premium, new
//! laptop, vacation, property tax). For each goal:
//!
//!   - target_usd            — what you eventually need
//!   - current_balance_usd   — what's there now
//!   - target_date_months    — how many months until you need it
//!   - monthly_contribution  — what you're setting aside per month
//!
//! Compute returns per goal:
//!   - remaining_usd            — target − current
//!   - required_monthly_usd     — remaining / months (None if months ≤ 0)
//!   - months_to_target_at_rate — remaining / monthly_contribution
//!                                (None if no contribution AND remaining > 0)
//!   - on_track                 — monthly_contribution ≥ required_monthly
//!                                AND months_to_target ≤ target_date
//!   - shortfall_per_month_usd  — required − contribution (≥ 0)
//!
//! Plus aggregates:
//!   - total_target_usd / total_balance_usd / total_remaining_usd
//!   - total_required_monthly_usd / total_monthly_contribution_usd
//!   - aggregate_shortfall_per_month_usd
//!   - status = "on-track" (zero shortfall) | "short" (positive)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GoalInput {
    pub name: String,
    pub target_usd: f64,
    #[serde(default)]
    pub current_balance_usd: f64,
    pub target_date_months: f64,
    #[serde(default)]
    pub monthly_contribution_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SinkingFundInput {
    #[serde(default)]
    pub goals: Vec<GoalInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoalReport {
    pub name: String,
    pub target_usd: f64,
    pub current_balance_usd: f64,
    pub remaining_usd: f64,
    pub target_date_months: f64,
    pub monthly_contribution_usd: f64,
    pub required_monthly_usd: Option<f64>,
    pub months_to_target_at_rate: Option<f64>,
    pub on_track: bool,
    pub shortfall_per_month_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SinkingFundReport {
    pub goals: Vec<GoalReport>,
    pub total_target_usd: f64,
    pub total_balance_usd: f64,
    pub total_remaining_usd: f64,
    pub total_required_monthly_usd: f64,
    pub total_monthly_contribution_usd: f64,
    pub aggregate_shortfall_per_month_usd: f64,
    pub status: String,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn evaluate_goal(g: &GoalInput) -> GoalReport {
    let remaining = (g.target_usd - g.current_balance_usd).max(0.0);
    let required = if g.target_date_months > 0.0 {
        Some(remaining / g.target_date_months)
    } else if remaining <= 0.0 {
        Some(0.0)
    } else {
        None
    };
    let mtt = if remaining <= 0.0 {
        Some(0.0)
    } else if g.monthly_contribution_usd > 0.0 {
        Some(remaining / g.monthly_contribution_usd)
    } else {
        None
    };
    let shortfall = match required {
        Some(r) => (r - g.monthly_contribution_usd).max(0.0),
        None => 0.0,
    };
    let on_track = match (required, mtt) {
        (Some(req), Some(m)) => g.monthly_contribution_usd >= req && m <= g.target_date_months,
        _ => false,
    } || remaining <= 0.0;
    GoalReport {
        name: g.name.clone(),
        target_usd: g.target_usd,
        current_balance_usd: g.current_balance_usd,
        remaining_usd: remaining,
        target_date_months: g.target_date_months,
        monthly_contribution_usd: g.monthly_contribution_usd,
        required_monthly_usd: required,
        months_to_target_at_rate: mtt,
        on_track,
        shortfall_per_month_usd: shortfall,
    }
}

pub fn compute(input: &SinkingFundInput) -> SinkingFundReport {
    let goals: Vec<GoalReport> = input.goals.iter().map(evaluate_goal).collect();
    let total_target: f64 = goals.iter().map(|g| g.target_usd).sum();
    let total_balance: f64 = goals.iter().map(|g| g.current_balance_usd).sum();
    let total_remaining: f64 = goals.iter().map(|g| g.remaining_usd).sum();
    let total_required: f64 = goals.iter().filter_map(|g| g.required_monthly_usd).sum();
    let total_contribution: f64 = goals.iter().map(|g| g.monthly_contribution_usd).sum();
    let aggregate_shortfall: f64 = goals.iter().map(|g| g.shortfall_per_month_usd).sum();
    let status = if aggregate_shortfall <= 1e-9 { "on-track" } else { "short" }.to_string();
    SinkingFundReport {
        goals,
        total_target_usd: total_target,
        total_balance_usd: total_balance,
        total_remaining_usd: total_remaining,
        total_required_monthly_usd: total_required,
        total_monthly_contribution_usd: total_contribution,
        aggregate_shortfall_per_month_usd: aggregate_shortfall,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(name: &str, target: f64, bal: f64, months: f64, contrib: f64) -> GoalInput {
        GoalInput {
            name: name.into(),
            target_usd: target,
            current_balance_usd: bal,
            target_date_months: months,
            monthly_contribution_usd: contrib,
        }
    }

    #[test]
    fn evaluate_goal_already_funded() {
        let r = evaluate_goal(&g("vac", 1000.0, 1200.0, 6.0, 50.0));
        assert_eq!(r.remaining_usd, 0.0);
        assert_eq!(r.required_monthly_usd, Some(0.0));
        assert_eq!(r.months_to_target_at_rate, Some(0.0));
        assert_eq!(r.shortfall_per_month_usd, 0.0);
        assert!(r.on_track);
    }

    #[test]
    fn evaluate_goal_zero_months_unrealistic() {
        let r = evaluate_goal(&g("today", 1000.0, 0.0, 0.0, 100.0));
        assert_eq!(r.remaining_usd, 1000.0);
        assert!(r.required_monthly_usd.is_none());
        assert_eq!(r.shortfall_per_month_usd, 0.0);
        // months ≤ 0 means we can't compute required; not on-track.
        assert!(!r.on_track);
    }

    #[test]
    fn evaluate_goal_no_contribution_with_remaining() {
        let r = evaluate_goal(&g("car", 5000.0, 1000.0, 24.0, 0.0));
        assert_eq!(r.remaining_usd, 4000.0);
        assert_eq!(r.required_monthly_usd.unwrap(), 4000.0 / 24.0);
        assert!(r.months_to_target_at_rate.is_none()); // can't reach with 0/mo
        assert!((r.shortfall_per_month_usd - 4000.0 / 24.0).abs() < 1e-9);
        assert!(!r.on_track);
    }

    #[test]
    fn evaluate_goal_exactly_meeting_required() {
        // remaining $1200 ÷ 12 = $100/mo required, contributing $100 → on-track.
        let r = evaluate_goal(&g("xmas", 1200.0, 0.0, 12.0, 100.0));
        assert_eq!(r.required_monthly_usd, Some(100.0));
        assert_eq!(r.months_to_target_at_rate, Some(12.0));
        assert_eq!(r.shortfall_per_month_usd, 0.0);
        assert!(r.on_track);
    }

    #[test]
    fn evaluate_goal_overcontributing_finishes_early() {
        // $1200 in 12mo at $200/mo → 6mo. Required 100, contributing 200, on track.
        let r = evaluate_goal(&g("xmas", 1200.0, 0.0, 12.0, 200.0));
        assert_eq!(r.months_to_target_at_rate, Some(6.0));
        assert!(r.on_track);
        assert_eq!(r.shortfall_per_month_usd, 0.0);
    }

    #[test]
    fn evaluate_goal_undercontributing_shortfall() {
        // $1200 in 12mo at $50/mo → required 100, short by 50, mtt=24 > 12 → not on-track.
        let r = evaluate_goal(&g("xmas", 1200.0, 0.0, 12.0, 50.0));
        assert_eq!(r.shortfall_per_month_usd, 50.0);
        assert_eq!(r.months_to_target_at_rate, Some(24.0));
        assert!(!r.on_track);
    }

    #[test]
    fn compute_empty_input() {
        let r = compute(&SinkingFundInput { goals: vec![] });
        assert_eq!(r.total_target_usd, 0.0);
        assert_eq!(r.status, "on-track");
        assert_eq!(r.goals.len(), 0);
    }

    #[test]
    fn compute_aggregates_total_correctly() {
        let r = compute(&SinkingFundInput {
            goals: vec![
                g("a", 1200.0,    0.0, 12.0, 100.0),
                g("b", 6000.0, 1000.0, 24.0, 250.0),
                g("c",  500.0,  500.0,  3.0,   0.0),
            ],
        });
        assert_eq!(r.total_target_usd, 7700.0);
        assert_eq!(r.total_balance_usd, 1500.0);
        assert_eq!(r.total_remaining_usd, 6200.0);
        // contributions: 100 + 250 + 0 = 350
        assert_eq!(r.total_monthly_contribution_usd, 350.0);
        // required: 100 (a) + (6000-1000)/24=208.33 (b) + 0 (c) = ~308.33
        assert!((r.total_required_monthly_usd - (100.0 + 5000.0 / 24.0)).abs() < 1e-6);
    }

    #[test]
    fn compute_aggregate_shortfall_status() {
        let r = compute(&SinkingFundInput {
            goals: vec![
                g("a", 1200.0, 0.0, 12.0,  50.0),  // shortfall 50/mo
                g("b", 1200.0, 0.0, 12.0, 100.0),  // on-track
            ],
        });
        assert!((r.aggregate_shortfall_per_month_usd - 50.0).abs() < 1e-9);
        assert_eq!(r.status, "short");
    }

    #[test]
    fn compute_all_on_track_status() {
        let r = compute(&SinkingFundInput {
            goals: vec![
                g("a", 1200.0, 0.0, 12.0, 100.0),
                g("b",  600.0, 0.0,  6.0, 100.0),
            ],
        });
        assert_eq!(r.aggregate_shortfall_per_month_usd, 0.0);
        assert_eq!(r.status, "on-track");
    }

    #[test]
    fn compute_already_funded_zero_remaining_status() {
        let r = compute(&SinkingFundInput {
            goals: vec![g("done", 100.0, 100.0, 6.0, 0.0)],
        });
        assert_eq!(r.total_remaining_usd, 0.0);
        assert_eq!(r.status, "on-track");
    }
}
