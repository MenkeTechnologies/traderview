//! Symbol catalog — every listed ticker, keyed for fast autocomplete.
//!
//! On first read the table is seeded from Finnhub's `/stock/symbol`
//! endpoint (~11k US rows). Subsequent reads are pure-DB. The seed is
//! idempotent — `ON CONFLICT (symbol) DO UPDATE` so a re-seed
//! refreshes descriptions without throwing away existing data.

use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize, sqlx::FromRow)]
pub struct Symbol {
    pub symbol: String,
    // Legacy column from migration 0007 — short, often empty.
    pub name: Option<String>,
    // Finnhub-sourced (migration 0050) — preferred display string.
    pub description: Option<String>,
    pub display_symbol: Option<String>,
    // `type` is a Rust keyword. Rename so the wire still says `type`
    // while the Rust field is a valid ident.
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub kind: Option<String>,
    pub currency: Option<String>,
    pub mic: Option<String>,
    pub exchange: Option<String>,
}

pub async fn count(pool: &PgPool) -> anyhow::Result<i64> {
    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM symbols")
        .fetch_one(pool)
        .await?;
    Ok(n)
}

pub async fn list_all(pool: &PgPool) -> anyhow::Result<Vec<Symbol>> {
    let rows = sqlx::query_as::<_, Symbol>(
        "SELECT symbol, name, description, display_symbol, type,
                currency, mic, exchange
           FROM symbols
          ORDER BY symbol ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Pull every US-exchange listing from Finnhub and upsert into the
/// symbols table. Returns the number of rows touched. If Finnhub has
/// no key configured the call returns an error which callers should
/// surface as a 503-style hint to the user rather than crashing.
pub async fn seed_from_finnhub(pool: &PgPool, exchange: &str) -> anyhow::Result<usize> {
    let raw = crate::finnhub_rest::stock_symbol_list(exchange).await?;
    let arr = raw
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Finnhub stock_symbol_list returned non-array payload"))?;
    let mut tx = pool.begin().await?;
    let mut n = 0;
    for row in arr {
        let symbol = row
            .get("symbol")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let Some(symbol) = symbol else { continue };
        let description = row
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let display = row
            .get("displaySymbol")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let ty = row
            .get("type")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let cur = row
            .get("currency")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let mic = row
            .get("mic")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let figi = row
            .get("figi")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let isin = row
            .get("isin")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let scf = row
            .get("shareClassFIGI")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        // The pre-existing schema (migration 0007) has NOT NULL
        // defaults for `asset_class`, `currency`, `multiplier`. We
        // only set fields we know about — the table-level defaults
        // fill the rest on insert. On conflict we update only the
        // catalog-derived columns so the OHLC consumers' columns
        // (multiplier, tick_size, ...) stay untouched.
        sqlx::query(
            "INSERT INTO symbols
                (symbol, description, display_symbol, type, currency, mic,
                 figi, isin, share_class_figi, exchange, fetched_at)
             VALUES ($1, $2, $3, $4, COALESCE($5, 'USD'), $6, $7, $8, $9, $10, NOW())
             ON CONFLICT (symbol) DO UPDATE SET
                description      = EXCLUDED.description,
                display_symbol   = EXCLUDED.display_symbol,
                type             = EXCLUDED.type,
                currency         = COALESCE(EXCLUDED.currency, symbols.currency),
                mic              = EXCLUDED.mic,
                figi             = EXCLUDED.figi,
                isin             = EXCLUDED.isin,
                share_class_figi = EXCLUDED.share_class_figi,
                exchange         = COALESCE(EXCLUDED.exchange, symbols.exchange),
                fetched_at       = NOW()",
        )
        .bind(&symbol)
        .bind(description)
        .bind(display)
        .bind(ty)
        .bind(cur)
        .bind(mic)
        .bind(figi)
        .bind(isin)
        .bind(scf)
        .bind(exchange)
        .execute(&mut *tx)
        .await?;
        n += 1;
    }
    tx.commit().await?;
    Ok(n)
}
