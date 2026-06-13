//! Disability-insurance needs — the monthly benefit that replaces lost income
//! if you can't work, net of any employer long-term-disability (LTD) coverage.
//!
//! Disability policies replace a fraction of income (commonly ~60%, since
//! benefits on premiums you pay are tax-free). The gap is the extra monthly
//! benefit to buy beyond group coverage, checked against actual expenses.
//!
//! ```text
//! target benefit = replacement % × gross monthly income
//! monthly gap    = target − existing LTD coverage   (floored at 0)
//! ```

use serde::{Deserialize, Serialize};

fn d_replacement() -> f64 {
    60.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisabilityInput {
    pub annual_income_usd: f64,
    /// Share of income to replace, percent (≈60% is typical).
    #[serde(default = "d_replacement")]
    pub replacement_pct: f64,
    /// Existing employer/group LTD benefit, monthly.
    #[serde(default)]
    pub existing_coverage_monthly_usd: f64,
    /// Essential monthly expenses, to check coverage adequacy.
    #[serde(default)]
    pub monthly_expenses_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DisabilityResult {
    pub monthly_income_usd: f64,
    /// Target monthly benefit (replacement % of income).
    pub target_monthly_benefit_usd: f64,
    /// Additional monthly benefit to buy beyond existing coverage.
    pub monthly_gap_usd: f64,
    /// Annualized gap.
    pub annual_gap_usd: f64,
    /// Target benefit ÷ monthly expenses; `None` if no expenses given.
    pub expense_coverage_ratio: Option<f64>,
    /// Whether the target benefit covers essential expenses.
    pub covers_expenses: bool,
}

pub fn analyze(input: &DisabilityInput) -> DisabilityResult {
    let monthly_income = input.annual_income_usd / 12.0;
    let target = input.replacement_pct / 100.0 * monthly_income;
    let gap = (target - input.existing_coverage_monthly_usd).max(0.0);

    let (ratio, covers) = if input.monthly_expenses_usd > 0.0 {
        let r = target / input.monthly_expenses_usd;
        (Some(r), target >= input.monthly_expenses_usd)
    } else {
        (None, true)
    };

    DisabilityResult {
        monthly_income_usd: monthly_income,
        target_monthly_benefit_usd: target,
        monthly_gap_usd: gap,
        annual_gap_usd: gap * 12.0,
        expense_coverage_ratio: ratio,
        covers_expenses: covers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> DisabilityInput {
        DisabilityInput {
            annual_income_usd: 90_000.0,
            replacement_pct: 60.0,
            existing_coverage_monthly_usd: 2_000.0,
            monthly_expenses_usd: 4_000.0,
        }
    }

    #[test]
    fn monthly_income_and_target() {
        let r = analyze(&base());
        assert!(close(r.monthly_income_usd, 7_500.0));
        // 60% × 7,500 = 4,500.
        assert!(close(r.target_monthly_benefit_usd, 4_500.0));
    }

    #[test]
    fn monthly_gap() {
        // 4,500 target − 2,000 existing = 2,500.
        let r = analyze(&base());
        assert!(close(r.monthly_gap_usd, 2_500.0));
        assert!(close(r.annual_gap_usd, 30_000.0));
    }

    #[test]
    fn expense_coverage() {
        let r = analyze(&base());
        // 4,500 / 4,000 = 1.125 — covers expenses.
        assert!(close(r.expense_coverage_ratio.unwrap(), 1.125));
        assert!(r.covers_expenses);
    }

    #[test]
    fn does_not_cover_high_expenses() {
        let r = analyze(&DisabilityInput {
            monthly_expenses_usd: 6_000.0,
            ..base()
        });
        assert!(!r.covers_expenses);
        assert!(r.expense_coverage_ratio.unwrap() < 1.0);
    }

    #[test]
    fn ample_existing_coverage_zero_gap() {
        let r = analyze(&DisabilityInput {
            existing_coverage_monthly_usd: 5_000.0,
            ..base()
        });
        assert!(close(r.monthly_gap_usd, 0.0));
    }

    #[test]
    fn higher_replacement_raises_target() {
        let low = analyze(&base());
        let high = analyze(&DisabilityInput {
            replacement_pct: 80.0,
            ..base()
        });
        assert!(high.target_monthly_benefit_usd > low.target_monthly_benefit_usd);
    }

    #[test]
    fn no_expenses_assumes_covered() {
        let r = analyze(&DisabilityInput {
            monthly_expenses_usd: 0.0,
            ..base()
        });
        assert!(r.expense_coverage_ratio.is_none());
        assert!(r.covers_expenses);
    }

    #[test]
    fn existing_coverage_reduces_gap() {
        let less = analyze(&DisabilityInput {
            existing_coverage_monthly_usd: 1_000.0,
            ..base()
        });
        let more = analyze(&DisabilityInput {
            existing_coverage_monthly_usd: 3_000.0,
            ..base()
        });
        assert!(more.monthly_gap_usd < less.monthly_gap_usd);
    }
}
