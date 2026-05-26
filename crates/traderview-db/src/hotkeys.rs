use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Hotkey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub combo: String,
    pub action: String,
    pub payload: serde_json::Value,
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Hotkey>> {
    Ok(sqlx::query_as::<_, Hotkey>(
        "SELECT id, user_id, name, combo, action, payload
           FROM hotkeys WHERE user_id = $1 ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(pool).await?)
}

pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    combo: &str,
    action: &str,
    payload: &serde_json::Value,
) -> anyhow::Result<Hotkey> {
    Ok(sqlx::query_as::<_, Hotkey>(
        "INSERT INTO hotkeys (user_id, name, combo, action, payload)
              VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, combo) DO UPDATE SET
            name = EXCLUDED.name, action = EXCLUDED.action, payload = EXCLUDED.payload
         RETURNING id, user_id, name, combo, action, payload",
    )
    .bind(user_id).bind(name).bind(combo).bind(action).bind(payload)
    .fetch_one(pool).await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM hotkeys WHERE id = $1 AND user_id = $2")
        .bind(id).bind(user_id).execute(pool).await?;
    Ok(r.rows_affected() > 0)
}
