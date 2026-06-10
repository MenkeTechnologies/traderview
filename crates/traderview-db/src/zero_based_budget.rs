//! Zero-based budget calculator (Dave Ramsey / YNAB style).
//!
//! Rule: every dollar of monthly income is assigned a job before the
//! month begins, so `income − Σ planned = 0`. Compute reports:
//!
//!   - total_planned_usd     — sum of every category's planned amount
//!   - total_actual_usd      — sum of actual spend (if filled in)
//!   - leftover_usd          — income − planned (positive = unassigned,
//!                              negative = over-allocated)
//!   - variance_per_category — actual − planned (positive = overspent,
//!                              negative = underspent)
//!   - total_variance_usd    — Σ variance (positive = overspent total)
//!   - is_zero_based         — |leftover| < 1
//!   - status = "zero-based" / "unassigned" (leftover > 0) /
//!              "over-allocated" (leftover < 0)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryRow {
    pub name: String,
    pub planned_usd: f64,
    #[serde(default)]
    pub actual_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZeroBasedBudgetInput {
    pub monthly_income_usd: f64,
    #[serde(default)]
    pub categories: Vec<CategoryRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryResult {
    pub name: String,
    pub planned_usd: f64,
    pub actual_usd: f64,
    pub variance_usd: f64,
    pub variance_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ZeroBasedBudgetReport {
    pub monthly_income_usd: f64,
    pub total_planned_usd: f64,
    pub total_actual_usd: f64,
    pub leftover_usd: f64,
    pub total_variance_usd: f64,
    pub is_zero_based: bool,
    pub status: String,
    pub categories: Vec<CategoryResult>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn category_variance(planned: f64, actual: f64) -> (f64, Option<f64>) {
    let v = actual - planned;
    let pct = if planned.abs() > 1e-9 {
        Some(v / planned.abs() * 100.0)
    } else {
        None
    };
    (v, pct)
}

pub fn status_label(leftover: f64) -> &'static str {
    if leftover.abs() < 1.0 {
        "zero-based"
    } else if leftover > 0.0 {
        "unassigned"
    } else {
        "over-allocated"
    }
}

pub fn compute(input: &ZeroBasedBudgetInput) -> ZeroBasedBudgetReport {
    let categories: Vec<CategoryResult> = input
        .categories
        .iter()
        .map(|c| {
            let (v, pct) = category_variance(c.planned_usd, c.actual_usd);
            CategoryResult {
                name: c.name.clone(),
                planned_usd: c.planned_usd,
                actual_usd: c.actual_usd,
                variance_usd: v,
                variance_pct: pct,
            }
        })
        .collect();
    let total_planned: f64 = categories.iter().map(|c| c.planned_usd).sum();
    let total_actual: f64 = categories.iter().map(|c| c.actual_usd).sum();
    let leftover = input.monthly_income_usd - total_planned;
    let total_variance: f64 = categories.iter().map(|c| c.variance_usd).sum();
    let is_zero = leftover.abs() < 1.0;
    let status = status_label(leftover).to_string();
    ZeroBasedBudgetReport {
        monthly_income_usd: input.monthly_income_usd,
        total_planned_usd: total_planned,
        total_actual_usd: total_actual,
        leftover_usd: leftover,
        total_variance_usd: total_variance,
        is_zero_based: is_zero,
        status,
        categories,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(name: &str, p: f64, a: f64) -> CategoryRow {
        CategoryRow { name: name.into(), planned_usd: p, actual_usd: a }
    }

    #[test]
    fn category_variance_zero_planned_none_pct() {
        let (v, p) = category_variance(0.0, 100.0);
        assert_eq!(v, 100.0);
        assert!(p.is_none());
    }

    #[test]
    fn category_variance_overspent() {
        let (v, p) = category_variance(100.0, 120.0);
        assert_eq!(v, 20.0);
        assert!((p.unwrap() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn category_variance_underspent() {
        let (v, p) = category_variance(100.0, 80.0);
        assert_eq!(v, -20.0);
        assert!((p.unwrap() + 20.0).abs() < 1e-9);
    }

    #[test]
    fn status_zero_based_within_a_dollar() {
        assert_eq!(status_label(0.0), "zero-based");
        assert_eq!(status_label(0.5), "zero-based");
        assert_eq!(status_label(-0.5), "zero-based");
    }

    #[test]
    fn status_unassigned_positive_leftover() {
        assert_eq!(status_label(100.0), "unassigned");
    }

    #[test]
    fn status_over_allocated_negative_leftover() {
        assert_eq!(status_label(-100.0), "over-allocated");
    }

    #[test]
    fn compute_perfect_zero_based() {
        let r = compute(&ZeroBasedBudgetInput {
            monthly_income_usd: 5000.0,
            categories: vec![
                c("rent",    1500.0, 1500.0),
                c("food",     800.0,  750.0),
                c("savings", 1500.0, 1500.0),
                c("misc",    1200.0, 1100.0),
            ],
        });
        assert_eq!(r.total_planned_usd, 5000.0);
        assert_eq!(r.leftover_usd, 0.0);
        assert!(r.is_zero_based);
        assert_eq!(r.status, "zero-based");
        assert_eq!(r.total_variance_usd, -150.0);
    }

    #[test]
    fn compute_unassigned_when_under_allocated() {
        let r = compute(&ZeroBasedBudgetInput {
            monthly_income_usd: 5000.0,
            categories: vec![c("rent", 1500.0, 1500.0)],
        });
        assert_eq!(r.leftover_usd, 3500.0);
        assert!(!r.is_zero_based);
        assert_eq!(r.status, "unassigned");
    }

    #[test]
    fn compute_over_allocated_when_planned_exceeds_income() {
        let r = compute(&ZeroBasedBudgetInput {
            monthly_income_usd: 3000.0,
            categories: vec![
                c("rent",    1500.0, 1500.0),
                c("food",     800.0,  800.0),
                c("savings", 1500.0,    0.0),
            ],
        });
        assert_eq!(r.leftover_usd, -800.0);
        assert_eq!(r.status, "over-allocated");
    }

    #[test]
    fn compute_empty_categories_full_leftover() {
        let r = compute(&ZeroBasedBudgetInput {
            monthly_income_usd: 2000.0,
            categories: vec![],
        });
        assert_eq!(r.total_planned_usd, 0.0);
        assert_eq!(r.leftover_usd, 2000.0);
        assert_eq!(r.status, "unassigned");
    }

    #[test]
    fn compute_category_results_preserve_input_order() {
        let r = compute(&ZeroBasedBudgetInput {
            monthly_income_usd: 1000.0,
            categories: vec![c("a", 100.0, 0.0), c("b", 200.0, 0.0), c("c", 300.0, 0.0)],
        });
        assert_eq!(r.categories[0].name, "a");
        assert_eq!(r.categories[2].name, "c");
    }
}
