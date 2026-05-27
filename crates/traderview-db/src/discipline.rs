//! Streaks + discipline scorecard.
//!
//! Streak math: walk closed trades chronologically; track current win/loss
//! run, longest run of each type, and the current run as-of latest trade.
//!
//! Discipline scoring: for each filled trade that links back to a
//! `trade_plans` row, check four rules:
//!   * stop_set:   actual trade has a non-null stop_loss
//!   * stop_honored: trade closed at/above plan.stop_loss (long) or
//!     at/below (short) — never closed worse than plan stop
//!   * qty_within: actual qty <= 1.10 × plan.intended_qty (10% slack)
//!   * direction_match: actual side == plan.side
//!
//! A trade passes when all four rules pass; weekly + monthly discipline %
//! = (passing trades / linked trades) × 100 across windows.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct StreaksReport {
    pub total_closed: usize,
    pub longest_win_streak: usize,
    pub longest_loss_streak: usize,
    pub current_streak_kind: &'static str,    // "win" | "loss" | "none"
    pub current_streak_length: usize,
    pub sparkline: Vec<i8>,                   // +1 win, -1 loss, 0 breakeven (latest 60)
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleEval {
    pub trade_id: Uuid,
    pub plan_id: Uuid,
    pub symbol: String,
    pub date: NaiveDate,
    pub stop_set: bool,
    pub stop_honored: bool,
    pub qty_within: bool,
    pub direction_match: bool,
    pub overall_pass: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowScore {
    pub label: &'static str,
    pub window_days: i64,
    pub linked_trades: usize,
    pub passing: usize,
    pub discipline_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisciplineReport {
    pub streaks: StreaksReport,
    pub rule_evals: Vec<RuleEval>,
    pub weekly: WindowScore,
    pub monthly: WindowScore,
    pub all_time: WindowScore,
    pub rule_breakdown: RuleBreakdown,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleBreakdown {
    pub stop_set_rate: f64,
    pub stop_honored_rate: f64,
    pub qty_within_rate: f64,
    pub direction_match_rate: f64,
}

pub async fn report(pool: &PgPool, _user_id: Uuid, account_id: Uuid)
    -> anyhow::Result<DisciplineReport>
{
    type ClosedTradeRow = (Uuid, String, DateTime<Utc>, Option<Decimal>, String, Decimal, Option<Decimal>, Option<Decimal>);
    let trades: Vec<ClosedTradeRow> = sqlx::query_as(
        "SELECT id, symbol, opened_at, net_pnl, side::text, qty, stop_loss, exit_avg
           FROM trades
          WHERE account_id = $1 AND status = 'closed' AND net_pnl IS NOT NULL
          ORDER BY opened_at ASC",
    ).bind(account_id).fetch_all(pool).await?;

    // ---- Streaks --------------------------------------------------------
    let mut sparkline: Vec<i8> = Vec::with_capacity(trades.len());
    let mut longest_win = 0usize;
    let mut longest_loss = 0usize;
    let mut cur_win = 0usize;
    let mut cur_loss = 0usize;
    let mut last_kind: &'static str = "none";
    let mut last_run = 0usize;
    for (_id, _sym, _opened, pnl, _side, _qty, _stop, _exit) in &trades {
        let v = dec_opt(pnl);
        let bit: i8 = if v > 0.0 { 1 } else if v < 0.0 { -1 } else { 0 };
        sparkline.push(bit);
        if bit == 1 {
            cur_win += 1; cur_loss = 0;
            if cur_win > longest_win { longest_win = cur_win; }
            last_kind = "win"; last_run = cur_win;
        } else if bit == -1 {
            cur_loss += 1; cur_win = 0;
            if cur_loss > longest_loss { longest_loss = cur_loss; }
            last_kind = "loss"; last_run = cur_loss;
        } else {
            cur_win = 0; cur_loss = 0; last_kind = "none"; last_run = 0;
        }
    }
    if sparkline.len() > 60 {
        let n = sparkline.len();
        sparkline = sparkline[n - 60..].to_vec();
    }
    let streaks = StreaksReport {
        total_closed: trades.len(),
        longest_win_streak: longest_win,
        longest_loss_streak: longest_loss,
        current_streak_kind: last_kind,
        current_streak_length: last_run,
        sparkline, computed_at: Utc::now(),
    };

    // ---- Rule evaluations against linked plans --------------------------
    type PlanTradeJoinRow = (Uuid, Uuid, String, Decimal, Option<Decimal>, Uuid, DateTime<Utc>, Option<Decimal>, String, Decimal, Option<Decimal>);
    let plan_rows: Vec<PlanTradeJoinRow> = sqlx::query_as(
        "SELECT p.id, p.linked_trade_id, p.symbol, p.intended_qty, p.stop_loss,
                t.id, t.opened_at, t.exit_avg, t.side::text, t.qty, t.stop_loss
           FROM trade_plans p
           JOIN trades t ON t.id = p.linked_trade_id
          WHERE t.account_id = $1
            AND t.status = 'closed'
            AND p.linked_trade_id IS NOT NULL",
    ).bind(account_id).fetch_all(pool).await?;

    let mut rule_evals: Vec<RuleEval> = Vec::with_capacity(plan_rows.len());
    for (pid, _ltid, psym, p_qty, p_stop, tid, opened, exit_avg, side, t_qty, t_stop) in plan_rows {
        let stop_set = t_stop.is_some();
        let qty_within = {
            let pq = dec(p_qty);
            let tq = dec(t_qty);
            pq <= 0.0 || tq <= pq * 1.10
        };
        // Plan side comes from trade_side_t column; we joined trades on side too.
        // For simplicity we match strings.
        // Compare trade side with plan side. (Plans don't store actual side string
        // here but the implicit assumption is that the linked trade IS the
        // execution of the plan, so we check the plan->trade direction integrity.)
        // We instead look up the plan side separately to keep the SELECT simple:
        let p_side: Option<String> = sqlx::query_scalar(
            "SELECT side::text FROM trade_plans WHERE id = $1",
        ).bind(pid).fetch_optional(pool).await.ok().flatten();
        let direction_match = p_side.as_deref() == Some(side.as_str());
        let stop_honored = match (p_stop, exit_avg, side.as_str()) {
            (Some(ps), Some(ex), "long")  => dec(ex) >= dec(ps),
            (Some(ps), Some(ex), "short") => dec(ex) <= dec(ps),
            (None, _, _) => true,                 // plan had no stop set → can't violate
            _ => true,
        };
        let overall = stop_set && stop_honored && qty_within && direction_match;
        rule_evals.push(RuleEval {
            trade_id: tid, plan_id: pid, symbol: psym,
            date: opened.date_naive(),
            stop_set, stop_honored, qty_within, direction_match,
            overall_pass: overall,
        });
    }

    // ---- Windowed discipline % -----------------------------------------
    let today = Utc::now().date_naive();
    let window = |label: &'static str, days: i64| -> WindowScore {
        let cutoff = today - Duration::days(days);
        let in_w: Vec<&RuleEval> = rule_evals.iter()
            .filter(|r| r.date >= cutoff)
            .collect();
        let linked = in_w.len();
        let pass = in_w.iter().filter(|r| r.overall_pass).count();
        let pct = if linked > 0 { pass as f64 / linked as f64 * 100.0 } else { 0.0 };
        WindowScore {
            label, window_days: days,
            linked_trades: linked, passing: pass, discipline_pct: pct,
        }
    };
    let weekly  = window("week",  7);
    let monthly = window("month", 30);
    let all_time = WindowScore {
        label: "all_time", window_days: 0,
        linked_trades: rule_evals.len(),
        passing: rule_evals.iter().filter(|r| r.overall_pass).count(),
        discipline_pct: if rule_evals.is_empty() { 0.0 } else {
            rule_evals.iter().filter(|r| r.overall_pass).count() as f64
                / rule_evals.len() as f64 * 100.0
        },
    };

    // Per-rule pass rates across all evaluated trades.
    let n = rule_evals.len() as f64;
    let rate = |f: fn(&RuleEval) -> bool| -> f64 {
        if n == 0.0 { return 0.0; }
        rule_evals.iter().filter(|r| f(r)).count() as f64 / n * 100.0
    };
    let rule_breakdown = RuleBreakdown {
        stop_set_rate:        rate(|r| r.stop_set),
        stop_honored_rate:    rate(|r| r.stop_honored),
        qty_within_rate:      rate(|r| r.qty_within),
        direction_match_rate: rate(|r| r.direction_match),
    };

    Ok(DisciplineReport {
        streaks, rule_evals, weekly, monthly, all_time, rule_breakdown,
    })
}

fn dec_opt(d: &Option<Decimal>) -> f64 {
    d.as_ref().map(|x| x.to_string().parse().unwrap_or(0.0)).unwrap_or(0.0)
}
fn dec(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
