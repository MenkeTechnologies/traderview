use sqlx::PgPool;
use traderview_core::Account;
use uuid::Uuid;

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Account>> {
    let rows: Vec<(
        Uuid,
        Uuid,
        String,
        Option<Uuid>,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "SELECT id, user_id, broker, broker_id, name, base_currency, created_at
               FROM accounts WHERE user_id = $1 ORDER BY created_at ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, user_id, broker, broker_id, name, base_currency, created_at)| Account {
                id,
                user_id,
                broker,
                broker_id,
                name,
                base_currency,
                created_at,
            },
        )
        .collect())
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    broker: &str,
    name: &str,
    base_currency: &str,
) -> anyhow::Result<Account> {
    // Resolve / auto-create the broker row from the free-text broker label
    // so every account is wired to a broker entity from day one. Slug
    // normalization matches migration 0049's REGEXP_REPLACE so collisions
    // map to the same row (Webull vs webull).
    let slug: String = broker
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let broker_id: Option<Uuid> = if slug.is_empty() {
        None
    } else {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO brokers (user_id, slug, display_name)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, slug) DO UPDATE
                SET updated_at = NOW()
             RETURNING id",
        )
        .bind(user_id)
        .bind(&slug)
        .bind(broker.trim())
        .fetch_one(pool)
        .await?;
        Some(row.0)
    };
    let (id, created_at): (Uuid, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO accounts (user_id, broker, broker_id, name, base_currency)
              VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at",
    )
    .bind(user_id)
    .bind(broker)
    .bind(broker_id)
    .bind(name)
    .bind(base_currency)
    .fetch_one(pool)
    .await?;
    Ok(Account {
        id,
        user_id,
        broker: broker.into(),
        broker_id,
        name: name.into(),
        base_currency: base_currency.into(),
        created_at,
    })
}

pub async fn ensure_default(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Account> {
    if let Some(a) = list(pool, user_id).await?.into_iter().next() {
        return Ok(a);
    }
    create(pool, user_id, "manual", "Main", "USD").await
}

/// Apply optional updates to an account. Any `None` field is left
/// untouched. When `broker` changes the broker row is upserted via the
/// same slug normalization as [`create`], and `broker_id` is rewired
/// to the resolved broker — accounts can never end up orphaned from
/// the brokers table.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    broker: Option<&str>,
    name: Option<&str>,
    base_currency: Option<&str>,
) -> anyhow::Result<Option<Account>> {
    // Resolve a new broker row (and broker_id) when `broker` is set.
    // Stays Option<Option<...>> shaped so we can pass `NULL` to SQL
    // for the COALESCE pattern when no broker change was requested.
    let new_broker_id: Option<Uuid> = if let Some(b) = broker {
        let slug: String = b
            .trim()
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string();
        if slug.is_empty() {
            None
        } else {
            let row: (Uuid,) = sqlx::query_as(
                "INSERT INTO brokers (user_id, slug, display_name)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (user_id, slug) DO UPDATE
                    SET updated_at = NOW()
                 RETURNING id",
            )
            .bind(user_id)
            .bind(&slug)
            .bind(b.trim())
            .fetch_one(pool)
            .await?;
            Some(row.0)
        }
    } else {
        None
    };
    let row: Option<(
        Uuid,
        Uuid,
        String,
        Option<Uuid>,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "UPDATE accounts SET
            broker        = COALESCE($3, broker),
            broker_id     = COALESCE($4, broker_id),
            name          = COALESCE($5, name),
            base_currency = COALESCE($6, base_currency)
         WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, broker, broker_id, name, base_currency, created_at",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(broker)
    .bind(new_broker_id)
    .bind(name)
    .bind(base_currency)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(id, user_id, broker, broker_id, name, base_currency, created_at)| Account {
            id,
            user_id,
            broker,
            broker_id,
            name,
            base_currency,
            created_at,
        },
    ))
}

pub async fn delete(pool: &PgPool, user_id: Uuid, account_id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM accounts WHERE id = $1 AND user_id = $2")
        .bind(account_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}
