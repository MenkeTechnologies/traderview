//! Generic CSV import wizard — accepts any CSV, returns headers + rows,
//! then commits with a user-supplied column → canonical-field mapping.
//!
//! Canonical fields:
//!   symbol      (required)
//!   side        (required) — buy / sell / short / cover (case-insensitive)
//!   qty         (required) — positive decimal
//!   price       (required) — positive decimal
//!   executed_at (required) — RFC3339 or 'YYYY-MM-DD HH:MM:SS' or 'YYYY-MM-DD'
//!   fee         (optional)
//!   broker_order_id (optional)
//!
//! Stage 1 (parse): returns first 1000 rows + detected headers — frontend
//! uses this to render the mapping UI. Stage 2 (commit): re-parses with
//! the mapping, inserts via `executions::insert_parsed` so the dedupe
//! constraint catches duplicates in re-imports.

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use traderview_core::Side;
use traderview_import::{sha256_hex, ParsedExecution};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ParsePreview {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,        // first 1000 rows verbatim
    pub total_rows: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColumnMapping {
    pub symbol: String,                // header name
    pub side: String,
    pub qty: String,
    pub price: String,
    pub executed_at: String,
    pub fee: Option<String>,
    pub broker_order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitResult {
    pub inserted: u64,
    pub skipped_dedupe: u64,
    pub failed_rows: Vec<RowError>,
    pub import_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct RowError {
    pub row_index: usize,
    pub reason: String,
}

pub fn parse_csv(bytes: &[u8]) -> anyhow::Result<ParsePreview> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(bytes);
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.trim().to_string()).collect();
    let mut all_rows: Vec<Vec<String>> = Vec::new();
    for rec in rdr.records() {
        let r = rec?;
        all_rows.push(r.iter().map(|s| s.to_string()).collect());
    }
    let total = all_rows.len();
    let preview = if all_rows.len() > 1000 { all_rows[..1000].to_vec() } else { all_rows.clone() };
    Ok(ParsePreview {
        headers, rows: preview, total_rows: total, sha256: sha256_hex(bytes),
    })
}

pub async fn commit(
    pool: &PgPool,
    account_id: Uuid,
    bytes: &[u8],
    mapping: &ColumnMapping,
) -> anyhow::Result<CommitResult> {
    let preview = parse_csv(bytes)?;
    let mut header_idx: HashMap<&str, usize> = HashMap::new();
    for (i, h) in preview.headers.iter().enumerate() {
        header_idx.insert(h.as_str(), i);
    }
    let need = |name: &str| -> anyhow::Result<usize> {
        header_idx.get(name).copied()
            .ok_or_else(|| anyhow::anyhow!("column not in CSV: {}", name))
    };
    let i_sym  = need(&mapping.symbol)?;
    let i_side = need(&mapping.side)?;
    let i_qty  = need(&mapping.qty)?;
    let i_pri  = need(&mapping.price)?;
    let i_ts   = need(&mapping.executed_at)?;
    let i_fee  = mapping.fee.as_deref().and_then(|n| header_idx.get(n).copied());
    let i_oid  = mapping.broker_order_id.as_deref().and_then(|n| header_idx.get(n).copied());

    // Audit/dedupe at the import level: insert one imports row with the
    // sha256 of the raw bytes. If we've already imported this exact file
    // it errors on the unique constraint; we surface that cleanly.
    let import_id: Uuid = sqlx::query_scalar(
        "INSERT INTO imports (account_id, source, sha256, raw_bytes, status)
              VALUES ($1, 'csv_wizard', $2, $3, 'processed')
          RETURNING id",
    )
    .bind(account_id).bind(&preview.sha256).bind(bytes)
    .fetch_one(pool).await?;

    // Re-walk the full row set (not just the preview slice).
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).flexible(true)
        .from_reader(bytes);
    let _headers = rdr.headers()?.clone();
    let mut inserted = 0u64;
    let mut skipped = 0u64;
    let mut failures = Vec::new();

    for (row_idx, rec) in rdr.records().enumerate() {
        let r = match rec {
            Ok(r) => r,
            Err(e) => { failures.push(RowError { row_index: row_idx, reason: e.to_string() }); continue; }
        };
        match build_exec(&r, i_sym, i_side, i_qty, i_pri, i_ts, i_fee, i_oid) {
            Ok(p) => {
                match traderview_db_executions_insert_parsed(pool, account_id, import_id, &p).await {
                    Ok(true)  => inserted += 1,
                    Ok(false) => skipped  += 1,
                    Err(e)    => failures.push(RowError {
                        row_index: row_idx, reason: e.to_string(),
                    }),
                }
            }
            Err(e) => failures.push(RowError { row_index: row_idx, reason: e.to_string() }),
        }
    }

    Ok(CommitResult { inserted, skipped_dedupe: skipped, failed_rows: failures, import_id })
}

// Thin trampoline so we don't shadow the existing executions module
// name in scope while still using its insert_parsed.
async fn traderview_db_executions_insert_parsed(
    pool: &PgPool, account_id: Uuid, import_id: Uuid, p: &ParsedExecution,
) -> anyhow::Result<bool> {
    crate::executions::insert_parsed(pool, account_id, import_id, p).await
}

fn build_exec(
    r: &csv::StringRecord,
    i_sym: usize, i_side: usize, i_qty: usize, i_pri: usize, i_ts: usize,
    i_fee: Option<usize>, i_oid: Option<usize>,
) -> anyhow::Result<ParsedExecution> {
    let get = |i: usize| r.get(i).map(|s| s.trim()).unwrap_or("");
    let symbol = get(i_sym).to_uppercase();
    if symbol.is_empty() { anyhow::bail!("empty symbol"); }
    let side = parse_side(get(i_side))?;
    let qty = parse_decimal(get(i_qty), "qty")?;
    let price = parse_decimal(get(i_pri), "price")?;
    let fee = i_fee.map(|i| parse_decimal(get(i), "fee").unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);
    let executed_at = parse_dt(get(i_ts))?;
    let broker_order_id = i_oid.and_then(|i| {
        let v = get(i); if v.is_empty() { None } else { Some(v.to_string()) }
    });
    // Stash the entire row as the audit blob so re-parse / debugging is
    // possible without the original file.
    let raw = serde_json::json!({
        "row": r.iter().collect::<Vec<&str>>(),
    });
    let mut p = ParsedExecution::stock(symbol, side, qty, price, fee, executed_at);
    p.broker_order_id = broker_order_id;
    p.raw = raw;
    Ok(p)
}

fn parse_side(s: &str) -> anyhow::Result<Side> {
    match s.to_lowercase().as_str() {
        "buy"   | "b" | "long"  => Ok(Side::Buy),
        "sell"  | "s"           => Ok(Side::Sell),
        "short" | "sh"          => Ok(Side::Short),
        "cover" | "btc" | "cv"  => Ok(Side::Cover),
        other => anyhow::bail!("unknown side: '{}'", other),
    }
}

fn parse_decimal(s: &str, field: &str) -> anyhow::Result<Decimal> {
    let cleaned: String = s.chars()
        .filter(|c| !matches!(c, '$' | ',' | ' '))
        .collect();
    Decimal::from_str(&cleaned)
        .map_err(|e| anyhow::anyhow!("invalid {}: '{}': {}", field, s, e))
}

fn parse_dt(s: &str) -> anyhow::Result<DateTime<Utc>> {
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return Ok(d.with_timezone(&Utc));
    }
    for fmt in &[
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M",
    ] {
        if let Ok(d) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(d, Utc));
        }
    }
    // Bare date — treat as midnight UTC.
    for fmt in &["%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            let dt = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
        }
    }
    anyhow::bail!("could not parse timestamp: '{}'", s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_csv_headers() {
        let csv = b"Ticker,Action,Quantity,Price,Time\nAAPL,buy,100,150.25,2025-01-15 09:30:00\nMSFT,sell,50,400.00,2025-01-15 10:15:00\n";
        let p = parse_csv(csv).unwrap();
        assert_eq!(p.headers, vec!["Ticker", "Action", "Quantity", "Price", "Time"]);
        assert_eq!(p.rows.len(), 2);
        assert_eq!(p.total_rows, 2);
    }

    #[test]
    fn parses_dollar_signs_and_commas_in_numbers() {
        let d = parse_decimal("$1,234.56", "price").unwrap();
        assert_eq!(d, Decimal::from_str("1234.56").unwrap());
    }

    #[test]
    fn parses_multiple_date_formats() {
        let _ = parse_dt("2025-01-15 09:30:00").unwrap();
        let _ = parse_dt("01/15/2025 09:30:00").unwrap();
        let _ = parse_dt("2025-01-15").unwrap();
        let _ = parse_dt("2025-01-15T09:30:00Z").unwrap();
        assert!(parse_dt("definitely not a date").is_err());
    }

    #[test]
    fn side_aliases_resolve() {
        assert!(matches!(parse_side("BUY").unwrap(), Side::Buy));
        assert!(matches!(parse_side("b").unwrap(), Side::Buy));
        assert!(matches!(parse_side("Long").unwrap(), Side::Buy));
        assert!(matches!(parse_side("S").unwrap(), Side::Sell));
        assert!(matches!(parse_side("short").unwrap(), Side::Short));
        assert!(matches!(parse_side("BTC").unwrap(), Side::Cover));
        assert!(parse_side("foo").is_err());
    }
}
