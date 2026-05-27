use sqlx::PgPool;
use traderview_core::User;
use uuid::Uuid;

/// First-launch local user for desktop mode. Idempotent.
pub async fn ensure_local(pool: &PgPool) -> anyhow::Result<Uuid> {
    if let Some((id,)) =
        sqlx::query_as::<_, (Uuid,)>("SELECT id FROM users WHERE is_local = true LIMIT 1")
            .fetch_optional(pool)
            .await?
    {
        return Ok(id);
    }
    let (id,): (Uuid,) =
        sqlx::query_as("INSERT INTO users (display_name, is_local) VALUES ($1, true) RETURNING id")
            .bind("local")
            .fetch_one(pool)
            .await?;
    Ok(id)
}

pub async fn create(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    display_name: &str,
) -> anyhow::Result<Uuid> {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, display_name, is_local)
              VALUES ($1, $2, $3, false) RETURNING id",
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

type UserAuthRow = (
    Uuid,
    Option<String>,
    Option<String>,
    String,
    bool,
    chrono::DateTime<chrono::Utc>,
);
type UserRow = (
    Uuid,
    Option<String>,
    String,
    bool,
    chrono::DateTime<chrono::Utc>,
);

pub async fn find_by_email(pool: &PgPool, email: &str) -> anyhow::Result<Option<UserAuth>> {
    let row: Option<UserAuthRow> = sqlx::query_as(
        "SELECT id, email, password_hash, display_name, is_local, created_at
               FROM users WHERE lower(email) = lower($1) LIMIT 1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(id, email, password_hash, display_name, is_local, created_at)| UserAuth {
            user: User {
                id,
                email,
                display_name,
                is_local,
                created_at,
            },
            password_hash,
        },
    ))
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<User>> {
    let row: Option<UserRow> = sqlx::query_as(
        "SELECT id, email, display_name, is_local, created_at
               FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(
        row.map(|(id, email, display_name, is_local, created_at)| User {
            id,
            email,
            display_name,
            is_local,
            created_at,
        }),
    )
}

pub struct UserAuth {
    pub user: User,
    pub password_hash: Option<String>,
}
