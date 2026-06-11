//! Screener snapshot store — the background refresher persists each
//! run of the bar screeners + carry screen; routes serve the latest
//! snapshot and the changes vs the prior one without recomputing.
//! Mirrors the golden-stars persistence model.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

/// Screener names stored by the refresher.
pub const SCREENERS: &[&str] = &["seasonality", "risk", "momentum", "mean-reversion", "carry"];

/// Default universe for the bar screeners — index + sector ETFs plus
/// rates/gold, within the 30-symbol screener cap.
pub const SNAPSHOT_UNIVERSE: &[&str] = &[
    "SPY", "QQQ", "DIA", "IWM", "XLK", "XLF", "XLE", "XLV", "XLI", "XLY", "XLP", "XLU", "XLRE",
    "XLB", "XLC", "TLT", "GLD",
];

pub async fn save(pool: &PgPool, screener: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO screener_snapshots (screener, payload) VALUES ($1, $2)")
        .bind(screener)
        .bind(payload)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub created_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

/// Latest two snapshots, newest first.
pub async fn latest_two(pool: &PgPool, screener: &str) -> anyhow::Result<Vec<Snapshot>> {
    let rows: Vec<(DateTime<Utc>, serde_json::Value)> = sqlx::query_as(
        "SELECT created_at, payload FROM screener_snapshots
          WHERE screener = $1 ORDER BY created_at DESC LIMIT 2",
    )
    .bind(screener)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(created_at, payload)| Snapshot { created_at, payload })
        .collect())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ShapeChange {
    pub key: String,
    pub from: String,
    pub to: String,
}

/// Diff two snapshot payloads for categorical flips. Pure — works on
/// the stored JSON so it applies to any screener with a `rows` array:
/// carry rows flip on `shape`, risk rows on `currently_underwater`.
pub fn detect_changes(prior: &serde_json::Value, latest: &serde_json::Value) -> Vec<ShapeChange> {
    let field = |row: &serde_json::Value| -> Option<(String, String)> {
        let key = row
            .get("root")
            .or_else(|| row.get("symbol"))?
            .as_str()?
            .to_string();
        let state = if let Some(s) = row.get("shape").and_then(|x| x.as_str()) {
            s.to_string()
        } else if let Some(b) = row.get("currently_underwater").and_then(|x| x.as_bool()) {
            if b { "underwater" } else { "at_highs" }.to_string()
        } else {
            return None;
        };
        Some((key, state))
    };
    let collect = |v: &serde_json::Value| -> std::collections::HashMap<String, String> {
        v.get("rows")
            .and_then(|r| r.as_array())
            .map(|rows| rows.iter().filter_map(field).collect())
            .unwrap_or_default()
    };
    let p = collect(prior);
    let l = collect(latest);
    let mut out: Vec<ShapeChange> = l
        .iter()
        .filter_map(|(k, to)| {
            let from = p.get(k)?;
            (from != to).then(|| ShapeChange {
                key: k.clone(),
                from: from.clone(),
                to: to.clone(),
            })
        })
        .collect();
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn carry_shape_flip_detected() {
        let prior = json!({"rows": [
            {"root": "CL", "shape": "backwardation"},
            {"root": "GC", "shape": "contango"},
        ]});
        let latest = json!({"rows": [
            {"root": "CL", "shape": "contango"},
            {"root": "GC", "shape": "contango"},
        ]});
        let c = detect_changes(&prior, &latest);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].key, "CL");
        assert_eq!(c[0].from, "backwardation");
        assert_eq!(c[0].to, "contango");
    }

    #[test]
    fn risk_underwater_flip_detected() {
        let prior = json!({"rows": [{"symbol": "SPY", "currently_underwater": false}]});
        let latest = json!({"rows": [{"symbol": "SPY", "currently_underwater": true}]});
        let c = detect_changes(&prior, &latest);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].to, "underwater");
    }

    #[test]
    fn new_symbols_and_missing_fields_are_not_changes() {
        // A symbol only in the latest run has no prior state — no flip.
        let prior = json!({"rows": [{"symbol": "SPY", "currently_underwater": false}]});
        let latest = json!({"rows": [
            {"symbol": "SPY", "currently_underwater": false},
            {"symbol": "QQQ", "currently_underwater": true},
            {"symbol": "IWM"},
        ]});
        assert!(detect_changes(&prior, &latest).is_empty());
        // Payloads without rows arrays are inert, not errors.
        assert!(detect_changes(&json!({}), &json!({"rows": "nope"})).is_empty());
    }
}
