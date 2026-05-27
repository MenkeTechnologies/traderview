use chrono::{DateTime, Utc};
use sqlx::PgPool;
use traderview_core::{Mentorship, MentorshipStatus};
use uuid::Uuid;

pub async fn request(
    pool: &PgPool,
    mentor_id: Uuid,
    mentee_id: Uuid,
    scope: &str,
) -> anyhow::Result<Mentorship> {
    let row: Row = sqlx::query_as(
        "INSERT INTO mentorships (mentor_id, mentee_id, scope)
              VALUES ($1, $2, $3)
         ON CONFLICT (mentor_id, mentee_id) DO UPDATE SET status = 'pending'
         RETURNING id, mentor_id, mentee_id, status::text, scope,
                   created_at, accepted_at, revoked_at",
    )
    .bind(mentor_id)
    .bind(mentee_id)
    .bind(scope)
    .fetch_one(pool)
    .await?;
    Ok(row.into())
}

pub async fn accept(pool: &PgPool, mentor_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE mentorships SET status = 'active', accepted_at = now()
          WHERE id = $1 AND mentor_id = $2 AND status = 'pending'",
    )
    .bind(id)
    .bind(mentor_id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn revoke(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE mentorships SET status = 'revoked', revoked_at = now()
          WHERE id = $1 AND (mentor_id = $2 OR mentee_id = $2)",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn mentors_of(pool: &PgPool, mentee_id: Uuid) -> anyhow::Result<Vec<Mentorship>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, mentor_id, mentee_id, status::text, scope,
                created_at, accepted_at, revoked_at
           FROM mentorships WHERE mentee_id = $1 ORDER BY created_at DESC",
    )
    .bind(mentee_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn mentees_of(pool: &PgPool, mentor_id: Uuid) -> anyhow::Result<Vec<Mentorship>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, mentor_id, mentee_id, status::text, scope,
                created_at, accepted_at, revoked_at
           FROM mentorships WHERE mentor_id = $1 ORDER BY created_at DESC",
    )
    .bind(mentor_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// True iff `viewer_id` has an active mentor relationship over `owner_id`'s data.
pub async fn can_view(
    pool: &PgPool,
    viewer_id: Uuid,
    owner_id: Uuid,
) -> anyhow::Result<bool> {
    if viewer_id == owner_id {
        return Ok(true);
    }
    let exists: Option<(bool,)> = sqlx::query_as(
        "SELECT TRUE FROM mentorships
          WHERE mentor_id = $1 AND mentee_id = $2 AND status = 'active' LIMIT 1",
    )
    .bind(viewer_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?;
    Ok(exists.is_some())
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    mentor_id: Uuid,
    mentee_id: Uuid,
    status: String,
    scope: String,
    created_at: DateTime<Utc>,
    accepted_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

impl From<Row> for Mentorship {
    fn from(r: Row) -> Self {
        Mentorship {
            id: r.id,
            mentor_id: r.mentor_id,
            mentee_id: r.mentee_id,
            status: match r.status.as_str() {
                "active" => MentorshipStatus::Active,
                "revoked" => MentorshipStatus::Revoked,
                _ => MentorshipStatus::Pending,
            },
            scope: r.scope,
            created_at: r.created_at,
            accepted_at: r.accepted_at,
            revoked_at: r.revoked_at,
        }
    }
}
