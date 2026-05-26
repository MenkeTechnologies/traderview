//! CSV / printable-HTML exports.
//!
//! All endpoints return raw bytes (text/csv or text/html), not JSON, with
//! `Content-Disposition: attachment` so browsers download them. The HTML
//! tax-package endpoint omits attachment so users can open it inline and
//! print-to-PDF from the browser.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use chrono::{Datelike, Utc};
use serde::Deserialize;
use traderview_db::tax_lots::{compute, LotMethod};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/export/executions/:account_id.csv", get(executions_csv))
        .route("/export/trades/:account_id.csv", get(trades_csv))
        .route("/export/tax-lots/:account_id/realized.csv", get(tax_realized_csv))
        .route("/export/tax-lots/:account_id/open.csv", get(tax_open_csv))
        .route("/export/tax-package/:account_id.html", get(tax_package_html))
}

// ---------------------------------------------------------------------------
// CSV writer — tiny inline impl. Quotes any cell containing `,`/`"`/`\n`.
// ---------------------------------------------------------------------------

fn csv_cell(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn csv_row(cells: &[String]) -> String {
    cells.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(",")
}

fn dec_to_string(d: rust_decimal::Decimal) -> String { d.to_string() }
fn dec_opt(d: Option<rust_decimal::Decimal>) -> String { d.map(dec_to_string).unwrap_or_default() }

fn csv_response(filename: &str, body: String) -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, HeaderValue::from_static("text/csv; charset=utf-8")),
            (CONTENT_DISPOSITION,
             HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
                 .unwrap_or(HeaderValue::from_static("attachment"))),
        ],
        body,
    )
}

fn html_response(filename: &str, body: String) -> impl IntoResponse {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
            (CONTENT_DISPOSITION,
             HeaderValue::from_str(&format!("inline; filename=\"{}\"", filename))
                 .unwrap_or(HeaderValue::from_static("inline"))),
        ],
        body,
    )
}

// ---------------------------------------------------------------------------
// /export/executions/:account_id.csv
// ---------------------------------------------------------------------------

async fn executions_csv(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<axum::response::Response, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let execs = traderview_db::executions::list_for_account(&s.pool, account_id)
        .await.map_err(ApiError::Internal)?;
    let mut body = String::new();
    body.push_str(&csv_row(&[
        "executed_at", "symbol", "side", "qty", "price", "fee",
        "asset_class", "option_type", "strike", "expiration",
        "multiplier", "broker_order_id", "id",
    ].iter().map(|s| s.to_string()).collect::<Vec<_>>()));
    body.push('\n');
    for e in execs {
        body.push_str(&csv_row(&[
            e.executed_at.to_rfc3339(),
            e.symbol,
            format!("{:?}", e.side).to_lowercase(),
            dec_to_string(e.qty),
            dec_to_string(e.price),
            dec_to_string(e.fee),
            format!("{:?}", e.asset_class).to_lowercase(),
            e.option_type.map(|o| format!("{:?}", o).to_lowercase()).unwrap_or_default(),
            dec_opt(e.strike),
            e.expiration.map(|d| d.to_string()).unwrap_or_default(),
            dec_to_string(e.multiplier),
            e.broker_order_id.unwrap_or_default(),
            e.id.to_string(),
        ]));
        body.push('\n');
    }
    Ok(csv_response(&format!("executions-{}.csv", account_id), body).into_response())
}

// ---------------------------------------------------------------------------
// /export/trades/:account_id.csv
// ---------------------------------------------------------------------------

async fn trades_csv(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<axum::response::Response, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let trades = traderview_db::trades::list_for_account(
        &s.pool, account_id,
        &traderview_db::trades::TradeFilter::default(),
    ).await.map_err(ApiError::Internal)?;
    let mut body = String::new();
    body.push_str(&csv_row(&[
        "opened_at", "closed_at", "symbol", "side", "status", "qty",
        "entry_avg", "exit_avg", "gross_pnl", "fees", "net_pnl",
        "asset_class", "option_type", "strike", "expiration",
        "stop_loss", "risk_amount", "mfe", "mae", "id",
    ].iter().map(|s| s.to_string()).collect::<Vec<_>>()));
    body.push('\n');
    for t in trades {
        body.push_str(&csv_row(&[
            t.opened_at.to_rfc3339(),
            t.closed_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            t.symbol,
            format!("{:?}", t.side).to_lowercase(),
            format!("{:?}", t.status).to_lowercase(),
            dec_to_string(t.qty),
            dec_to_string(t.entry_avg),
            dec_opt(t.exit_avg),
            dec_opt(t.gross_pnl),
            dec_to_string(t.fees),
            dec_opt(t.net_pnl),
            format!("{:?}", t.asset_class).to_lowercase(),
            t.option_type.map(|o| format!("{:?}", o).to_lowercase()).unwrap_or_default(),
            dec_opt(t.strike),
            t.expiration.map(|d| d.to_string()).unwrap_or_default(),
            dec_opt(t.stop_loss),
            dec_opt(t.risk_amount),
            dec_opt(t.mfe),
            dec_opt(t.mae),
            t.id.to_string(),
        ]));
        body.push('\n');
    }
    Ok(csv_response(&format!("trades-{}.csv", account_id), body).into_response())
}

// ---------------------------------------------------------------------------
// /export/tax-lots/:account_id/realized.csv?year=&method=
// /export/tax-lots/:account_id/open.csv?method=
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TaxParams {
    year: Option<i32>,
    method: Option<LotMethod>,
}

async fn tax_realized_csv(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(p): Query<TaxParams>,
) -> Result<axum::response::Response, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let year = p.year.unwrap_or_else(|| Utc::now().year());
    let method = p.method.unwrap_or(LotMethod::Fifo);
    let r = compute(&s.pool, account_id, year, method).await.map_err(ApiError::Internal)?;
    let mut body = String::new();
    body.push_str(&csv_row(&[
        "symbol", "acquired", "disposed", "holding_days", "term",
        "qty", "cost_basis", "proceeds", "gain_loss", "wash_sale_disallowed",
        "buy_exec_id", "sell_exec_id",
    ].iter().map(|s| s.to_string()).collect::<Vec<_>>()));
    body.push('\n');
    for ev in r.realized {
        body.push_str(&csv_row(&[
            ev.symbol,
            ev.acquired_at.format("%Y-%m-%d").to_string(),
            ev.disposed_at.format("%Y-%m-%d").to_string(),
            ev.holding_days.to_string(),
            if ev.long_term { "LT".into() } else { "ST".into() },
            ev.qty.to_string(),
            format!("{:.2}", ev.cost_basis),
            format!("{:.2}", ev.proceeds),
            format!("{:.2}", ev.gain_loss),
            format!("{:.2}", ev.wash_sale_disallowed),
            ev.buy_exec_id.to_string(),
            ev.sell_exec_id.to_string(),
        ]));
        body.push('\n');
    }
    Ok(csv_response(&format!("tax-realized-{}-{}.csv", year, account_id), body).into_response())
}

async fn tax_open_csv(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(p): Query<TaxParams>,
) -> Result<axum::response::Response, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let year = p.year.unwrap_or_else(|| Utc::now().year());
    let method = p.method.unwrap_or(LotMethod::Fifo);
    let r = compute(&s.pool, account_id, year, method).await.map_err(ApiError::Internal)?;
    let mut body = String::new();
    body.push_str(&csv_row(&[
        "symbol", "acquired", "holding_days", "term",
        "qty_remaining", "cost_per_share", "cost_basis", "exec_id",
    ].iter().map(|s| s.to_string()).collect::<Vec<_>>()));
    body.push('\n');
    for l in r.open_lots {
        body.push_str(&csv_row(&[
            l.symbol,
            l.acquired_at.format("%Y-%m-%d").to_string(),
            l.holding_days.to_string(),
            if l.long_term { "LT".into() } else { "ST".into() },
            l.qty_remaining.to_string(),
            format!("{:.4}", l.cost_per_share),
            format!("{:.2}", l.cost_basis),
            l.exec_id.to_string(),
        ]));
        body.push('\n');
    }
    Ok(csv_response(&format!("tax-open-{}.csv", account_id), body).into_response())
}

// ---------------------------------------------------------------------------
// /export/tax-package/:account_id.html?year=&method=
// Printable single-page HTML tax package. User saves as PDF via browser print.
// ---------------------------------------------------------------------------

async fn tax_package_html(
    State(s): State<AppState>,
    u: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(p): Query<TaxParams>,
) -> Result<axum::response::Response, ApiError> {
    ensure_account_owner(&s, u.id, account_id).await?;
    let year = p.year.unwrap_or_else(|| Utc::now().year());
    let method = p.method.unwrap_or(LotMethod::Fifo);
    let r = compute(&s.pool, account_id, year, method).await.map_err(ApiError::Internal)?;

    let method_str = match method { LotMethod::Fifo => "FIFO", LotMethod::Lifo => "LIFO" };
    let realized_rows: String = r.realized.iter().map(|ev| {
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>\
             <td class=\"r\">{:.4}</td><td class=\"r\">${:.2}</td><td class=\"r\">${:.2}</td>\
             <td class=\"r {sgn}\">${:.2}</td><td class=\"r {wash}\">{wash_amt}</td></tr>",
            ev.symbol,
            ev.acquired_at.format("%Y-%m-%d"),
            ev.disposed_at.format("%Y-%m-%d"),
            ev.holding_days,
            if ev.long_term { "LT" } else { "ST" },
            ev.qty, ev.cost_basis, ev.proceeds, ev.gain_loss,
            sgn = if ev.gain_loss >= 0.0 { "pos" } else { "neg" },
            wash = if ev.wash_sale_disallowed > 0.0 { "warn" } else { "" },
            wash_amt = if ev.wash_sale_disallowed > 0.0 {
                format!("${:.2}", ev.wash_sale_disallowed)
            } else { "—".into() },
        )
    }).collect();

    let open_rows: String = r.open_lots.iter().map(|l| {
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td>\
             <td class=\"r\">{:.4}</td><td class=\"r\">${:.4}</td><td class=\"r\">${:.2}</td></tr>",
            l.symbol,
            l.acquired_at.format("%Y-%m-%d"),
            l.holding_days,
            if l.long_term { "LT" } else { "ST" },
            l.qty_remaining, l.cost_per_share, l.cost_basis,
        )
    }).collect();

    let html = format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8">
<title>Tax package — {year} — {account_id}</title>
<style>
  @media print {{
    @page {{ size: letter; margin: 0.6in; }}
    .nopr {{ display: none; }}
    body {{ -webkit-print-color-adjust: exact; print-color-adjust: exact; }}
  }}
  body {{ font-family: -apple-system, "Helvetica Neue", Arial, sans-serif; color: #111; margin: 24px; }}
  h1 {{ margin: 0 0 4px; font-size: 22px; }}
  h2 {{ margin: 24px 0 8px; font-size: 16px; border-bottom: 2px solid #333; padding-bottom: 2px; }}
  .meta {{ color: #555; font-size: 12px; margin-bottom: 12px; }}
  table {{ width: 100%; border-collapse: collapse; font-size: 11px; margin-bottom: 12px; }}
  th, td {{ border: 1px solid #ccc; padding: 4px 6px; text-align: left; }}
  th {{ background: #eee; }}
  td.r {{ text-align: right; font-variant-numeric: tabular-nums; }}
  .pos {{ color: #00692a; }}
  .neg {{ color: #b21d2a; }}
  .warn {{ color: #b85b00; }}
  .summary {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-bottom: 12px; }}
  .card {{ border: 1px solid #ccc; padding: 8px; }}
  .card .label {{ color: #555; font-size: 10px; text-transform: uppercase; letter-spacing: 0.05em; }}
  .card .value {{ font-size: 16px; font-weight: 600; margin-top: 4px; font-variant-numeric: tabular-nums; }}
  .footer {{ margin-top: 28px; font-size: 10px; color: #777; }}
  button.nopr {{ background: #00aaff; color: white; border: 0; padding: 8px 16px; font-size: 14px; cursor: pointer; }}
</style></head><body>
<button class="nopr" onclick="window.print()">Print / Save as PDF</button>
<h1>Tax package — {year}</h1>
<div class="meta">Account {account_id} · Method {method_str} · Generated {generated} · Long-term = held ≥ 365 days · IRC §1091 wash-sale flagging applied</div>

<div class="summary">
  <div class="card"><div class="label">Net total</div><div class="value {nett_cls}">${nett:.2}</div></div>
  <div class="card"><div class="label">Net short-term</div><div class="value {nst_cls}">${nst:.2}</div></div>
  <div class="card"><div class="label">Net long-term</div><div class="value {nlt_cls}">${nlt:.2}</div></div>
  <div class="card"><div class="label">Wash-sale disallowed</div><div class="value">${wsh:.2}</div></div>
  <div class="card"><div class="label">Total proceeds</div><div class="value">${prc:.2}</div></div>
  <div class="card"><div class="label">Total basis</div><div class="value">${bss:.2}</div></div>
  <div class="card"><div class="label">Realized events</div><div class="value">{rct}</div></div>
  <div class="card"><div class="label">Open lots</div><div class="value">{oct}</div></div>
</div>

<h2>Realized events ({rct})</h2>
<table>
  <thead><tr>
    <th>Symbol</th><th>Acquired</th><th>Disposed</th><th>Days</th><th>Term</th>
    <th class="r">Qty</th><th class="r">Basis</th><th class="r">Proceeds</th>
    <th class="r">Gain/Loss</th><th class="r">Wash</th>
  </tr></thead>
  <tbody>{realized_rows}</tbody>
</table>

<h2>Open lots ({oct})</h2>
<table>
  <thead><tr>
    <th>Symbol</th><th>Acquired</th><th>Days held</th><th>Term</th>
    <th class="r">Qty</th><th class="r">Cost/share</th><th class="r">Basis</th>
  </tr></thead>
  <tbody>{open_rows}</tbody>
</table>

<div class="footer">
  Generated by TraderView. This report is for informational purposes only and does not constitute tax advice.
  Wash-sale disallowed amounts are flagged per IRC §1091 but are NOT added back to replacement-lot basis in this output —
  consult your tax preparer for the full adjustment.
</div>
</body></html>"#,
        year = year,
        account_id = account_id,
        method_str = method_str,
        generated = r.fetched_at.format("%Y-%m-%d %H:%M UTC"),
        nett = r.net_total, nett_cls = if r.net_total >= 0.0 { "pos" } else { "neg" },
        nst = r.net_short_term, nst_cls = if r.net_short_term >= 0.0 { "pos" } else { "neg" },
        nlt = r.net_long_term, nlt_cls = if r.net_long_term >= 0.0 { "pos" } else { "neg" },
        wsh = r.wash_sale_total,
        prc = r.total_proceeds, bss = r.total_basis,
        rct = r.realized_count, oct = r.open_lot_count,
        realized_rows = realized_rows,
        open_rows = open_rows,
    );
    Ok(html_response(&format!("tax-package-{}-{}.html", year, account_id), html).into_response())
}
