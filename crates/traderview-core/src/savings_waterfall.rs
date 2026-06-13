//! Savings waterfall — the financial order of operations.
//!
//! Where should the next dollar of savings go? The widely-taught answer
//! is a priority ladder: secure the employer match (free money) and clear
//! a starter emergency buffer before chasing returns, kill high-interest
//! debt before investing taxable, fill tax-advantaged space (HSA, Roth)
//! before a brokerage. This allocates a month's available savings down
//! that ladder, each rung capped by its remaining gap, with whatever is
//! left landing in a taxable brokerage.
//!
//! Pure compute — every rung's gap is supplied by the caller, so the
//! ordering is the only opinion here.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    /// A small cash buffer (one month / a deductible) before anything else.
    StarterEmergency,
    /// Enough into the 401(k) to capture the full employer match.
    EmployerMatch,
    /// High-interest debt (cards, anything above ~the market's expected return).
    HighInterestDebt,
    /// The full 3–6 month emergency fund.
    FullEmergency,
    /// Tax-advantaged space with an edge: HSA (triple-advantaged), then Roth.
    TaxAdvantaged,
    /// Remaining tax-deferred retirement room (401k/IRA to the limit).
    MaxRetirement,
    /// Everything left over — an ordinary taxable brokerage (uncapped).
    TaxableBrokerage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WaterfallInput {
    /// Dollars available to allocate this month.
    pub monthly_available: Decimal,
    /// Remaining to reach the starter emergency buffer.
    pub starter_emergency_gap: Decimal,
    /// Monthly contribution needed to capture the full employer match.
    pub employer_match_monthly: Decimal,
    /// Balance of high-interest debt to retire.
    pub high_interest_debt: Decimal,
    /// Remaining to reach the full 3–6 month emergency fund.
    pub full_emergency_gap: Decimal,
    /// Remaining tax-advantaged room for the month (HSA + Roth).
    pub tax_advantaged_room_monthly: Decimal,
    /// Remaining tax-deferred retirement room for the month.
    pub retirement_room_monthly: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct Allocation {
    pub step: Step,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct WaterfallPlan {
    pub allocations: Vec<Allocation>,
    pub total_allocated: Decimal,
    /// Always zero — the taxable brokerage absorbs any remainder — but
    /// returned for symmetry and to make a zero-budget case explicit.
    pub unallocated: Decimal,
}

/// Allocate `monthly_available` down the priority ladder, each rung
/// capped by its supplied gap; the remainder lands in a taxable
/// brokerage. A rung that gets nothing is omitted from the plan.
pub fn plan(input: &WaterfallInput) -> WaterfallPlan {
    let mut remaining = input.monthly_available.max(Decimal::ZERO);
    let rungs = [
        (input.starter_emergency_gap, Step::StarterEmergency),
        (input.employer_match_monthly, Step::EmployerMatch),
        (input.high_interest_debt, Step::HighInterestDebt),
        (input.full_emergency_gap, Step::FullEmergency),
        (input.tax_advantaged_room_monthly, Step::TaxAdvantaged),
        (input.retirement_room_monthly, Step::MaxRetirement),
    ];

    let mut allocations = Vec::new();
    for (gap, step) in rungs {
        let amount = remaining.min(gap.max(Decimal::ZERO));
        if amount > Decimal::ZERO {
            allocations.push(Allocation { step, amount });
            remaining -= amount;
        }
    }
    if remaining > Decimal::ZERO {
        allocations.push(Allocation { step: Step::TaxableBrokerage, amount: remaining });
        remaining = Decimal::ZERO;
    }

    let total_allocated = allocations.iter().map(|a| a.amount).sum();
    WaterfallPlan { allocations, total_allocated, unallocated: remaining }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    fn input() -> WaterfallInput {
        WaterfallInput {
            monthly_available: Decimal::ZERO,
            starter_emergency_gap: d(1_000),
            employer_match_monthly: d(500),
            high_interest_debt: d(3_000),
            full_emergency_gap: d(12_000),
            tax_advantaged_room_monthly: d(1_000),
            retirement_room_monthly: d(1_500),
        }
    }

    fn amount_for(plan: &WaterfallPlan, step: Step) -> Decimal {
        plan.allocations
            .iter()
            .find(|a| a.step == step)
            .map(|a| a.amount)
            .unwrap_or(Decimal::ZERO)
    }

    #[test]
    fn fills_rungs_in_priority_order() {
        // $2,000 this month: $1,000 starter, then $500 match, then $500
        // toward the $3k debt. Nothing reaches the later rungs.
        let p = plan(&WaterfallInput { monthly_available: d(2_000), ..input() });
        assert_eq!(amount_for(&p, Step::StarterEmergency), d(1_000));
        assert_eq!(amount_for(&p, Step::EmployerMatch), d(500));
        assert_eq!(amount_for(&p, Step::HighInterestDebt), d(500));
        assert_eq!(amount_for(&p, Step::FullEmergency), Decimal::ZERO);
        assert_eq!(p.total_allocated, d(2_000));
        assert_eq!(p.unallocated, Decimal::ZERO);
    }

    #[test]
    fn match_comes_before_debt() {
        // The whole point of the ordering: even with debt outstanding, a
        // dollar past the starter buffer captures the match first.
        let p = plan(&WaterfallInput { monthly_available: d(1_200), ..input() });
        assert_eq!(amount_for(&p, Step::StarterEmergency), d(1_000));
        assert_eq!(amount_for(&p, Step::EmployerMatch), d(200));
        assert_eq!(amount_for(&p, Step::HighInterestDebt), Decimal::ZERO);
    }

    #[test]
    fn overflow_lands_in_taxable() {
        // More than every gap combined: 1000+500+3000+12000+1000+1500 =
        // 19,000 of capped rungs; $25k leaves $6k for the brokerage.
        let p = plan(&WaterfallInput { monthly_available: d(25_000), ..input() });
        assert_eq!(amount_for(&p, Step::TaxableBrokerage), d(6_000));
        assert_eq!(p.total_allocated, d(25_000));
    }

    #[test]
    fn zero_budget_allocates_nothing() {
        let p = plan(&WaterfallInput { monthly_available: Decimal::ZERO, ..input() });
        assert!(p.allocations.is_empty());
        assert_eq!(p.total_allocated, Decimal::ZERO);
        assert_eq!(p.unallocated, Decimal::ZERO);
    }

    #[test]
    fn negative_budget_clamped_to_zero() {
        let p = plan(&WaterfallInput { monthly_available: d(-500), ..input() });
        assert!(p.allocations.is_empty());
        assert_eq!(p.total_allocated, Decimal::ZERO);
    }

    #[test]
    fn skips_already_satisfied_rungs() {
        // No starter gap and no match needed → the first dollars go
        // straight to debt.
        let p = plan(&WaterfallInput {
            monthly_available: d(1_000),
            starter_emergency_gap: Decimal::ZERO,
            employer_match_monthly: Decimal::ZERO,
            ..input()
        });
        assert_eq!(amount_for(&p, Step::HighInterestDebt), d(1_000));
        assert!(p.allocations.iter().all(|a| a.step != Step::StarterEmergency));
    }

    #[test]
    fn total_allocated_equals_available_when_capacity_exists() {
        for budget in [100i64, 1_500, 9_999, 18_999] {
            let p = plan(&WaterfallInput { monthly_available: d(budget), ..input() });
            assert_eq!(p.total_allocated, d(budget), "budget {budget}");
        }
    }
}
