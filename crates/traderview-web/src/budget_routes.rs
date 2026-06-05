//! Personal finance / budgeting.
//!
//! Endpoints:
//!   GET    /api/budget                       — list all budgets + savings goal
//!   PUT    /api/budget/categories/:code      — upsert per-category limit
//!   DELETE /api/budget/categories/:code      — remove a budget
//!   PUT    /api/budget/savings-goal          — set monthly savings target
//!   GET    /api/budget/snapshot?year=&month= — live month snapshot
//!
//! The snapshot endpoint is the load-bearing one — it pulls live
//! transaction totals from `expense_transactions` for the requested
//! month, joins each category's budget limit, and emits per-category
//! progress + the rollup (income / expense / savings rate).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_budgets))
        .route("/categories/:code", put(upsert_category).delete(delete_category))
        .route("/savings-goal", put(set_savings_goal))
        .route("/snapshot", get(snapshot))
}

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
struct BudgetRow {
    category_code: String,
    monthly_limit: Decimal,
    paused: bool,
}

#[derive(Serialize)]
struct BudgetIndex {
    categories: Vec<BudgetRow>,
    monthly_savings_target: Decimal,
}

#[derive(Deserialize)]
struct UpsertBudgetBody {
    monthly_limit: Decimal,
    #[serde(default)]
    paused: Option<bool>,
}

#[derive(Deserialize)]
struct SavingsGoalBody {
    monthly_target: Decimal,
}

#[derive(Deserialize, Default)]
struct SnapshotParams {
    year: Option<i32>,
    month: Option<u32>,
}

#[derive(Serialize)]
struct CategoryProgress {
    category_code: String,
    label: Option<String>,
    monthly_limit: Decimal,
    spent: Decimal,
    pct: f32,
    over: bool,
    paused: bool,
}

#[derive(Serialize)]
struct BudgetSnapshot {
    year: i32,
    month: u32,
    income: Decimal,
    expense: Decimal,
    net: Decimal,
    monthly_savings_target: Decimal,
    savings_rate: f32,
    target_met: bool,
    categories: Vec<CategoryProgress>,
    over_budget_categories: u32,
}

// ── handlers ──────────────────────────────────────────────────────────

async fn list_budgets(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<BudgetIndex>, ApiError> {
    let cats: Vec<BudgetRow> = sqlx::query_as(
        "SELECT category_code, monthly_limit, paused
           FROM budgets WHERE user_id = $1
       ORDER BY category_code",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;

    let target: Option<Decimal> = sqlx::query_scalar(
        "SELECT monthly_target FROM budget_savings_goals WHERE user_id = $1",
    )
    .bind(user.id)
    .fetch_optional(&s.pool)
    .await?;

    Ok(Json(BudgetIndex {
        categories: cats,
        monthly_savings_target: target.unwrap_or(Decimal::ZERO),
    }))
}

async fn upsert_category(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<UpsertBudgetBody>,
) -> Result<Json<BudgetRow>, ApiError> {
    if body.monthly_limit < Decimal::ZERO {
        return Err(ApiError::BadRequest("monthly_limit must be >= 0".into()));
    }
    let paused = body.paused.unwrap_or(false);
    let row: BudgetRow = sqlx::query_as(
        "INSERT INTO budgets (user_id, category_code, monthly_limit, paused)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id, category_code)
             DO UPDATE SET
                 monthly_limit = EXCLUDED.monthly_limit,
                 paused = EXCLUDED.paused,
                 updated_at = NOW()
         RETURNING category_code, monthly_limit, paused",
    )
    .bind(user.id)
    .bind(&code)
    .bind(body.monthly_limit)
    .bind(paused)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn delete_category(
    State(s): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let r = sqlx::query("DELETE FROM budgets WHERE user_id = $1 AND category_code = $2")
        .bind(user.id)
        .bind(&code)
        .execute(&s.pool)
        .await?;
    Ok(Json(serde_json::json!({ "deleted": r.rows_affected() })))
}

async fn set_savings_goal(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<SavingsGoalBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.monthly_target < Decimal::ZERO {
        return Err(ApiError::BadRequest("monthly_target must be >= 0".into()));
    }
    sqlx::query(
        "INSERT INTO budget_savings_goals (user_id, monthly_target)
             VALUES ($1, $2)
             ON CONFLICT (user_id)
             DO UPDATE SET monthly_target = EXCLUDED.monthly_target, updated_at = NOW()",
    )
    .bind(user.id)
    .bind(body.monthly_target)
    .execute(&s.pool)
    .await?;
    Ok(Json(serde_json::json!({ "monthly_target": body.monthly_target })))
}

async fn snapshot(
    State(s): State<AppState>,
    user: AuthUser,
    Query(p): Query<SnapshotParams>,
) -> Result<Json<BudgetSnapshot>, ApiError> {
    let now = Utc::now();
    let year = p.year.unwrap_or_else(|| now.year());
    let month = p.month.unwrap_or_else(|| now.month()).clamp(1, 12);

    // Per-category spend for the month — joins to expense_categories
    // for the human-readable label.
    let rows: Vec<(String, Option<String>, Decimal)> = sqlx::query_as(
        "SELECT c.code,
                c.label,
                COALESCE(SUM(CASE WHEN t.amount < 0 THEN -t.amount ELSE 0 END), 0) AS spent
           FROM expense_categories c
           LEFT JOIN expense_transactions t
                  ON t.category_code = c.code
                 AND t.is_transfer = FALSE
                 AND EXTRACT(YEAR FROM t.posted_at) = $2
                 AND EXTRACT(MONTH FROM t.posted_at) = $3
                 AND EXISTS (
                     SELECT 1 FROM expense_accounts a
                      WHERE a.id = t.account_id AND a.user_id = $1
                 )
       GROUP BY c.code, c.label
       ORDER BY c.code",
    )
    .bind(user.id)
    .bind(year)
    .bind(month as i32)
    .fetch_all(&s.pool)
    .await?;

    // Join the user's budgets (per-category limit + paused).
    let budgets: Vec<(String, Decimal, bool)> = sqlx::query_as(
        "SELECT category_code, monthly_limit, paused
           FROM budgets WHERE user_id = $1",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    use std::collections::HashMap;
    let mut budget_map: HashMap<String, (Decimal, bool)> = HashMap::new();
    for (code, lim, paused) in budgets {
        budget_map.insert(code, (lim, paused));
    }

    let mut categories: Vec<CategoryProgress> = Vec::new();
    let mut total_expense = Decimal::ZERO;
    let mut over_count: u32 = 0;
    for (code, label, spent) in rows {
        let (limit, paused) = budget_map
            .get(&code)
            .cloned()
            .unwrap_or((Decimal::ZERO, false));
        total_expense += spent;
        let pct = if limit > Decimal::ZERO {
            let p = spent / limit;
            let f: f32 = p.try_into().unwrap_or(0.0);
            (f * 100.0).clamp(0.0, 999.0)
        } else {
            0.0
        };
        let over = !paused && limit > Decimal::ZERO && spent > limit;
        if over {
            over_count += 1;
        }
        // Skip categories with no budget AND no spend — they're noise.
        if limit == Decimal::ZERO && spent == Decimal::ZERO {
            continue;
        }
        categories.push(CategoryProgress {
            category_code: code, label,
            monthly_limit: limit, spent,
            pct, over, paused,
        });
    }

    // Income = positive expense_transactions for the month.
    let income_row: Option<Decimal> = sqlx::query_scalar(
        "SELECT COALESCE(SUM(t.amount), 0)
           FROM expense_transactions t
           JOIN expense_accounts a ON a.id = t.account_id
          WHERE a.user_id = $1
            AND t.amount > 0
            AND t.is_transfer = FALSE
            AND EXTRACT(YEAR FROM t.posted_at) = $2
            AND EXTRACT(MONTH FROM t.posted_at) = $3",
    )
    .bind(user.id)
    .bind(year)
    .bind(month as i32)
    .fetch_optional(&s.pool)
    .await?;
    let income = income_row.unwrap_or(Decimal::ZERO);

    let net = income - total_expense;
    let target: Option<Decimal> = sqlx::query_scalar(
        "SELECT monthly_target FROM budget_savings_goals WHERE user_id = $1",
    )
    .bind(user.id)
    .fetch_optional(&s.pool)
    .await?;
    let target = target.unwrap_or(Decimal::ZERO);

    let savings_rate = if income > Decimal::ZERO {
        let r = net / income;
        let f: f32 = r.try_into().unwrap_or(0.0);
        (f * 100.0).max(-999.0).min(999.0)
    } else {
        0.0
    };
    let target_met = net >= target;

    // Sort the categories: over-budget first, then by absolute spend.
    categories.sort_by(|a, b| {
        b.over.cmp(&a.over)
            .then_with(|| b.spent.cmp(&a.spent))
    });

    Ok(Json(BudgetSnapshot {
        year, month, income, expense: total_expense, net,
        monthly_savings_target: target,
        savings_rate, target_met,
        categories,
        over_budget_categories: over_count,
    }))
}

// ── Unit tests: pure-logic helpers ────────────────────────────────────
//
// The endpoints are SQL-driven so they need a Pg pool to exercise; the
// snapshot-shape transformations that are testable in isolation (zero
// income → savings_rate=0, over-budget flag respects paused) live in
// the `_logic` module so we can pin them without a DB.

#[cfg(test)]
mod logic {
    use rust_decimal::Decimal;

    /// Mirror of the in-place snapshot logic — given (income, expense,
    /// target), produce (net, savings_rate%, target_met).
    pub fn rollup(income: Decimal, expense: Decimal, target: Decimal) -> (Decimal, f32, bool) {
        let net = income - expense;
        let savings_rate = if income > Decimal::ZERO {
            let r = net / income;
            let f: f32 = r.try_into().unwrap_or(0.0);
            (f * 100.0).max(-999.0).min(999.0)
        } else {
            0.0
        };
        (net, savings_rate, net >= target)
    }

    #[test]
    fn rollup_zero_income_gives_zero_savings_rate_not_nan() {
        // Edge case the snapshot must survive: a brand-new account with
        // recorded expenses but no income rows yet. Division-by-zero
        // would produce NaN; the engine returns 0 instead, and
        // target_met is correctly false because net (-$500) doesn't
        // exceed target ($0).
        let (net, rate, met) = rollup(Decimal::ZERO, Decimal::from(500), Decimal::ZERO);
        assert_eq!(net, Decimal::from(-500));
        assert_eq!(rate, 0.0, "must be 0.0, never NaN");
        assert!(!met, "negative net cannot meet a target of 0");
    }

    #[test]
    fn rollup_negative_when_overspending() {
        // Income 4000, expense 5000 → net -1000 → rate -25%.
        let (net, rate, met) = rollup(
            Decimal::from(4_000),
            Decimal::from(5_000),
            Decimal::from(500),
        );
        assert_eq!(net, Decimal::from(-1_000));
        assert!((rate - (-25.0)).abs() < 0.01,
            "rate should be -25%, got {rate}");
        assert!(!met, "overspending must not meet positive savings target");
    }

    #[test]
    fn rollup_meets_target_when_savings_exceed_goal() {
        // Income 6000, expense 4000 → net 2000. Target 1500. Met.
        let (net, rate, met) = rollup(
            Decimal::from(6_000),
            Decimal::from(4_000),
            Decimal::from(1_500),
        );
        assert_eq!(net, Decimal::from(2_000));
        assert!((rate - 33.333_3).abs() < 0.01, "33.3% savings rate expected, got {rate}");
        assert!(met);
    }

    #[test]
    fn rollup_savings_rate_clamps_to_finite_range() {
        // Pathological case — defensive clamp against runaway values.
        let (_, rate, _) = rollup(Decimal::from(1), Decimal::from(-10_000), Decimal::ZERO);
        assert!(rate.is_finite());
        assert!(rate <= 999.0 && rate >= -999.0);
    }

    /// Verify the over-budget flag respects the `paused` field — a
    /// paused budget that's over the limit should NOT count as
    /// over-budget.
    #[test]
    fn paused_budget_does_not_count_as_over() {
        // The endpoint computes `over = !paused && spent > limit`.
        // Codify the rule via a tiny helper.
        fn over(spent: Decimal, limit: Decimal, paused: bool) -> bool {
            !paused && limit > Decimal::ZERO && spent > limit
        }
        assert!(over(Decimal::from(120), Decimal::from(100), false));
        assert!(!over(Decimal::from(120), Decimal::from(100), true));  // paused
        assert!(!over(Decimal::from(120), Decimal::ZERO, false));      // no limit set
        assert!(!over(Decimal::from(100), Decimal::from(100), false)); // at the limit (not over)
    }
}
