use sqlx::PgPool;
use traderview_core::Account;
use uuid::Uuid;

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Account>> {
    let rows: Vec<(Uuid, Uuid, String, String, String, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT id, user_id, broker, name, base_currency, created_at
               FROM accounts WHERE user_id = $1 ORDER BY created_at ASC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(id, user_id, broker, name, base_currency, created_at)| Account {
            id, user_id, broker, name, base_currency, created_at,
        })
        .collect())
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    broker: &str,
    name: &str,
    base_currency: &str,
) -> anyhow::Result<Account> {
    let (id, created_at): (Uuid, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO accounts (user_id, broker, name, base_currency)
              VALUES ($1, $2, $3, $4) RETURNING id, created_at",
    )
    .bind(user_id)
    .bind(broker)
    .bind(name)
    .bind(base_currency)
    .fetch_one(pool)
    .await?;
    Ok(Account {
        id,
        user_id,
        broker: broker.into(),
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

pub async fn delete(pool: &PgPool, user_id: Uuid, account_id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM accounts WHERE id = $1 AND user_id = $2")
        .bind(account_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}
