//! 50/30/20 budget rule (Elizabeth Warren, "All Your Worth", 2005).
//!
//! Split after-tax (net) income into three buckets:
//!   - 50% NEEDS    — rent, groceries, utilities, insurance, minimum
//!                    debt payments, mandatory expenses
//!   - 30% WANTS    — dining out, streaming, hobbies, vacations,
//!                    upgrades to needs (organic groceries, gym)
//!   - 20% SAVINGS  — emergency fund, retirement, brokerage, EXTRA
//!                    debt payment beyond minimum
//!
//! Each input row carries a `bucket` ∈ {needs, wants, savings} and
//! an amount. Compute returns per-bucket actual + ideal + delta + %.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseRow {
    pub name: String,
    pub bucket: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FiftyThirtyTwentyInput {
    pub net_monthly_income_usd: f64,
    #[serde(default)]
    pub rows: Vec<ExpenseRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BucketResult {
    pub bucket: &'static str,
    pub ideal_pct: f64,
    pub ideal_usd: f64,
    pub actual_usd: f64,
    pub actual_pct: f64,
    pub delta_usd: f64,
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct FiftyThirtyTwentyReport {
    pub net_monthly_income_usd: f64,
    pub needs: BucketResult,
    pub wants: BucketResult,
    pub savings: BucketResult,
    pub total_allocated_usd: f64,
    pub unallocated_usd: f64,
    pub overall_status: String,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn sum_for(rows: &[ExpenseRow], bucket: &str) -> f64 {
    rows.iter()
        .filter(|r| r.bucket == bucket)
        .map(|r| r.amount_usd)
        .sum()
}

pub fn evaluate_bucket(
    label: &'static str,
    ideal_pct: f64,
    net_income: f64,
    actual: f64,
) -> BucketResult {
    let ideal_usd = net_income * ideal_pct / 100.0;
    let actual_pct = if net_income > 0.0 {
        actual / net_income * 100.0
    } else {
        0.0
    };
    let delta = actual - ideal_usd;
    let status: &'static str = if label == "savings" {
        // For savings, MORE is better — under is bad.
        if actual >= ideal_usd { "on-track" } else { "under" }
    } else {
        // For needs/wants, under is better.
        if actual <= ideal_usd { "on-track" } else { "over" }
    };
    BucketResult {
        bucket: label,
        ideal_pct,
        ideal_usd,
        actual_usd: actual,
        actual_pct,
        delta_usd: delta,
        status,
    }
}

pub fn compute(input: &FiftyThirtyTwentyInput) -> FiftyThirtyTwentyReport {
    let needs_actual = sum_for(&input.rows, "needs");
    let wants_actual = sum_for(&input.rows, "wants");
    let savings_actual = sum_for(&input.rows, "savings");
    let needs = evaluate_bucket("needs", 50.0, input.net_monthly_income_usd, needs_actual);
    let wants = evaluate_bucket("wants", 30.0, input.net_monthly_income_usd, wants_actual);
    let savings =
        evaluate_bucket("savings", 20.0, input.net_monthly_income_usd, savings_actual);
    let total_alloc = needs_actual + wants_actual + savings_actual;
    let unalloc = input.net_monthly_income_usd - total_alloc;
    let bucket_statuses = [needs.status, wants.status, savings.status];
    let overall = if bucket_statuses.iter().all(|s| *s == "on-track") {
        "on-track"
    } else {
        "off-target"
    }
    .to_string();
    FiftyThirtyTwentyReport {
        net_monthly_income_usd: input.net_monthly_income_usd,
        needs,
        wants,
        savings,
        total_allocated_usd: total_alloc,
        unallocated_usd: unalloc,
        overall_status: overall,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(name: &str, bucket: &str, amt: f64) -> ExpenseRow {
        ExpenseRow { name: name.into(), bucket: bucket.into(), amount_usd: amt }
    }

    #[test]
    fn sum_for_basic() {
        let rs = vec![r("a", "needs", 100.0), r("b", "needs", 50.0), r("c", "wants", 30.0)];
        assert_eq!(sum_for(&rs, "needs"), 150.0);
        assert_eq!(sum_for(&rs, "wants"), 30.0);
        assert_eq!(sum_for(&rs, "savings"), 0.0);
    }

    #[test]
    fn evaluate_bucket_needs_on_track_when_under_ideal() {
        let b = evaluate_bucket("needs", 50.0, 5000.0, 2400.0);
        assert_eq!(b.ideal_usd, 2500.0);
        assert_eq!(b.actual_pct, 48.0);
        assert_eq!(b.delta_usd, -100.0);
        assert_eq!(b.status, "on-track");
    }

    #[test]
    fn evaluate_bucket_needs_over_when_above_ideal() {
        let b = evaluate_bucket("needs", 50.0, 5000.0, 2700.0);
        assert_eq!(b.delta_usd, 200.0);
        assert_eq!(b.status, "over");
    }

    #[test]
    fn evaluate_bucket_savings_under_when_below_ideal() {
        let b = evaluate_bucket("savings", 20.0, 5000.0, 800.0);
        assert_eq!(b.ideal_usd, 1000.0);
        assert_eq!(b.status, "under");
    }

    #[test]
    fn evaluate_bucket_savings_on_track_when_at_or_above() {
        let b = evaluate_bucket("savings", 20.0, 5000.0, 1500.0);
        assert_eq!(b.status, "on-track");
    }

    #[test]
    fn evaluate_bucket_zero_income_zero_pct() {
        let b = evaluate_bucket("needs", 50.0, 0.0, 100.0);
        assert_eq!(b.actual_pct, 0.0);
    }

    #[test]
    fn compute_perfectly_balanced() {
        let r = compute(&FiftyThirtyTwentyInput {
            net_monthly_income_usd: 5000.0,
            rows: vec![
                r("rent",     "needs",   2500.0),
                r("fun",      "wants",   1500.0),
                r("401k",     "savings", 1000.0),
            ],
        });
        assert_eq!(r.needs.delta_usd, 0.0);
        assert_eq!(r.wants.delta_usd, 0.0);
        assert_eq!(r.savings.delta_usd, 0.0);
        assert_eq!(r.overall_status, "on-track");
        assert_eq!(r.unallocated_usd, 0.0);
    }

    #[test]
    fn compute_overspending_wants() {
        let r = compute(&FiftyThirtyTwentyInput {
            net_monthly_income_usd: 5000.0,
            rows: vec![
                r("rent",     "needs",   2400.0),
                r("dining",   "wants",   2000.0),  // 40%, should be 30%
                r("401k",     "savings", 600.0),
            ],
        });
        assert_eq!(r.wants.status, "over");
        assert_eq!(r.savings.status, "under");
        assert_eq!(r.overall_status, "off-target");
    }

    #[test]
    fn compute_unallocated_residual() {
        let r = compute(&FiftyThirtyTwentyInput {
            net_monthly_income_usd: 5000.0,
            rows: vec![r("rent", "needs", 1000.0)],
        });
        assert_eq!(r.unallocated_usd, 4000.0);
    }

    #[test]
    fn compute_all_savings_high_status() {
        let r = compute(&FiftyThirtyTwentyInput {
            net_monthly_income_usd: 5000.0,
            rows: vec![
                r("rent", "needs", 2400.0),
                r("fun",  "wants", 1400.0),
                r("401k", "savings", 1500.0),
            ],
        });
        assert_eq!(r.needs.status, "on-track");
        assert_eq!(r.wants.status, "on-track");
        assert_eq!(r.savings.status, "on-track");
        assert_eq!(r.overall_status, "on-track");
    }

    #[test]
    fn compute_unknown_bucket_ignored() {
        let r = compute(&FiftyThirtyTwentyInput {
            net_monthly_income_usd: 5000.0,
            rows: vec![
                r("rent",     "needs",   2500.0),
                r("fun",      "wants",   1500.0),
                r("401k",     "savings", 1000.0),
                r("bogus",    "bogus",   999.0),  // ignored everywhere
            ],
        });
        // None of the bucket sums should include the bogus row.
        assert_eq!(r.needs.actual_usd, 2500.0);
        assert_eq!(r.wants.actual_usd, 1500.0);
        assert_eq!(r.savings.actual_usd, 1000.0);
        // total allocated only counts the three valid rows, not the bogus.
        assert_eq!(r.total_allocated_usd, 5000.0);
    }
}
