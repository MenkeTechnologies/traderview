//! Sector strength — 11 SPDR sector ETFs ranked by today's % change and
//! relative-strength vs SPY. Mirrors ZenBot's sector ranking widget.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

const SECTORS: &[(&str, &str)] = &[
    ("XLK", "Technology"),
    ("XLF", "Financials"),
    ("XLE", "Energy"),
    ("XLV", "Healthcare"),
    ("XLY", "Consumer Discretionary"),
    ("XLP", "Consumer Staples"),
    ("XLI", "Industrials"),
    ("XLB", "Materials"),
    ("XLU", "Utilities"),
    ("XLRE", "Real Estate"),
    ("XLC", "Communications"),
];

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Sector {
    pub sector: String,
    pub label: String,
    pub price: rust_decimal::Decimal,
    pub change_pct: rust_decimal::Decimal,
    pub rs_vs_spy: Option<rust_decimal::Decimal>,
    pub fetched_at: DateTime<Utc>,
}

pub async fn ranked(pool: &PgPool) -> anyhow::Result<Vec<Sector>> {
    // Refresh cache if older than 5 min.
    let stale: Option<(DateTime<Utc>,)> = sqlx::query_as(
        "SELECT MIN(fetched_at) FROM sector_strength",
    )
    .fetch_optional(pool)
    .await?;
    let needs_refresh = stale
        .map(|(t,)| (Utc::now() - t).num_seconds() > 300)
        .unwrap_or(true);
    if needs_refresh {
        refresh(pool).await?;
    }
    let rows: Vec<Sector> = sqlx::query_as(
        "SELECT sector, label, price, change_pct, rs_vs_spy, fetched_at
           FROM sector_strength ORDER BY change_pct DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn refresh(pool: &PgPool) -> anyhow::Result<()> {
    // Fetch SPY first for relative strength baseline.
    let spy = crate::market_data::quote(pool, "SPY").await.ok();
    let spy_change = spy.and_then(|q| q.change_pct).unwrap_or(0.0);

    for (sym, label) in SECTORS {
        if let Ok(q) = crate::market_data::quote(pool, sym).await {
            let chg = q.change_pct.unwrap_or(0.0);
            let rs = chg - spy_change;
            sqlx::query(
                "INSERT INTO sector_strength (sector, label, price, change_pct, rs_vs_spy, fetched_at)
                      VALUES ($1, $2, $3, $4, $5, now())
                 ON CONFLICT (sector) DO UPDATE SET
                    label = EXCLUDED.label, price = EXCLUDED.price,
                    change_pct = EXCLUDED.change_pct, rs_vs_spy = EXCLUDED.rs_vs_spy,
                    fetched_at = now()",
            )
            .bind(sym)
            .bind(label)
            .bind(rust_decimal::Decimal::try_from(q.price).unwrap_or_default())
            .bind(rust_decimal::Decimal::try_from(chg).unwrap_or_default())
            .bind(rust_decimal::Decimal::try_from(rs).unwrap_or_default())
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}
