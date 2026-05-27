//! Side-by-side trade comparison.
//!
//! For each requested trade_id we surface:
//!   * Full trade row (symbol, side, qty, entry/exit, P/L, MFE/MAE, R, fees)
//!   * Hold duration in seconds
//!   * Normalized P/L curve: t ∈ [0..1] mapped along the open→close window,
//!     y = (mark − entry_avg) * side_sign / entry_avg * 100 (percent return)
//!     sampled at every cached bar inside the window
//!
//! Auto-selects bar interval per hold duration so each trade gets ~50-200
//! sample points regardless of whether it's a 15-min scalp or a 6-month
//! swing. Frontend overlays the curves on one SVG so the user can spot
//! pattern differences between winners and losers.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct CurvePoint {
    pub t: f64,       // 0..1 along open→close
    pub pnl_pct: f64, // % return vs entry
    pub time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompareRow {
    pub trade_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub status: String,
    pub asset_class: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub qty: f64,
    pub entry_avg: f64,
    pub exit_avg: Option<f64>,
    pub gross_pnl: Option<f64>,
    pub fees: f64,
    pub net_pnl: Option<f64>,
    pub stop_loss: Option<f64>,
    pub initial_target: Option<f64>,
    pub mfe: Option<f64>,
    pub mae: Option<f64>,
    pub risk_amount: Option<f64>,
    pub r_multiple: Option<f64>,
    pub hold_seconds: i64,
    pub bar_interval: &'static str,
    pub curve: Vec<CurvePoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompareReport {
    pub rows: Vec<CompareRow>,
    pub fetched_at: DateTime<Utc>,
}

pub async fn compare(
    pool: &PgPool,
    user_id: Uuid,
    trade_ids: &[Uuid],
) -> anyhow::Result<CompareReport> {
    let mut rows = Vec::with_capacity(trade_ids.len());
    for tid in trade_ids.iter().take(4) {
        let trade = match crate::trades::get(pool, *tid).await? {
            Some(t) => t,
            None => continue,
        };
        // Ownership check via account.
        let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM accounts WHERE id = $1")
            .bind(trade.account_id)
            .fetch_optional(pool)
            .await?;
        if owner.map(|(u,)| u) != Some(user_id) {
            continue;
        }

        let opened = trade.opened_at;
        let closed = trade.closed_at.unwrap_or_else(Utc::now);
        let hold = closed.signed_duration_since(opened);

        let (interval, interval_str): (BarInterval, &'static str) = if hold < Duration::hours(4) {
            (BarInterval::M1, "1m")
        } else if hold < Duration::days(5) {
            (BarInterval::M5, "5m")
        } else if hold < Duration::days(30) {
            (BarInterval::H1, "1h")
        } else {
            (BarInterval::D1, "1d")
        };

        let pad = Duration::seconds(hold.num_seconds() / 20);
        let bars =
            crate::prices::get_bars(pool, &trade.symbol, interval, opened - pad, closed + pad)
                .await
                .unwrap_or_default();

        let entry = dec(trade.entry_avg);
        let side_sign = match format!("{:?}", trade.side).to_lowercase().as_str() {
            "short" => -1.0,
            _ => 1.0,
        };
        let hold_secs = hold.num_seconds().max(1);
        let curve: Vec<CurvePoint> = bars
            .iter()
            .filter(|b| b.bar_time >= opened && b.bar_time <= closed)
            .map(|b| {
                let last = dec(b.close);
                let pnl_pct = if entry > 0.0 {
                    (last - entry) / entry * 100.0 * side_sign
                } else {
                    0.0
                };
                let elapsed = (b.bar_time - opened).num_seconds().max(0);
                CurvePoint {
                    t: elapsed as f64 / hold_secs as f64,
                    pnl_pct,
                    time: b.bar_time,
                }
            })
            .collect();

        let r_multiple = match (trade.net_pnl, trade.risk_amount) {
            (Some(pnl), Some(risk)) => {
                let r = dec(risk);
                if r > 0.0 {
                    Some(dec(pnl) / r)
                } else {
                    None
                }
            }
            _ => None,
        };

        rows.push(CompareRow {
            trade_id: *tid,
            symbol: trade.symbol,
            side: format!("{:?}", trade.side).to_lowercase(),
            status: format!("{:?}", trade.status).to_lowercase(),
            asset_class: format!("{:?}", trade.asset_class).to_lowercase(),
            opened_at: opened,
            closed_at: trade.closed_at,
            qty: dec(trade.qty),
            entry_avg: entry,
            exit_avg: trade.exit_avg.map(dec),
            gross_pnl: trade.gross_pnl.map(dec),
            fees: dec(trade.fees),
            net_pnl: trade.net_pnl.map(dec),
            stop_loss: trade.stop_loss.map(dec),
            initial_target: trade.initial_target.map(dec),
            mfe: trade.mfe.map(dec),
            mae: trade.mae.map(dec),
            risk_amount: trade.risk_amount.map(dec),
            r_multiple,
            hold_seconds: hold_secs,
            bar_interval: interval_str,
            curve,
        });
    }
    Ok(CompareReport {
        rows,
        fetched_at: Utc::now(),
    })
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
