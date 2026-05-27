use chrono::{DateTime, Utc};
use sqlx::PgPool;
use traderview_core::{slug, ForumCategory, ForumPost, ForumThread};
use uuid::Uuid;

// ---------- categories -----------

pub async fn list_categories(pool: &PgPool) -> anyhow::Result<Vec<ForumCategory>> {
    let rows: Vec<CategoryRow> = sqlx::query_as(
        "SELECT id, slug, name, description, position, is_archived, created_at
           FROM forum_categories
          WHERE is_archived = FALSE ORDER BY position, name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

// ---------- threads -----------

pub async fn create_thread(
    pool: &PgPool,
    category_id: Uuid,
    author_id: Uuid,
    title: &str,
    body_md: &str,
) -> anyhow::Result<(ForumThread, ForumPost)> {
    let base = slug::from_title(title);
    let mut tx = pool.begin().await?;
    let mut thread_slug = base.clone();
    let mut row: Option<ThreadRow> = None;
    for n in 0..6 {
        let candidate = if n == 0 {
            base.clone()
        } else {
            format!("{}-{}", base, slug::random(4))
        };
        let res: Result<ThreadRow, sqlx::Error> = sqlx::query_as(
            "INSERT INTO forum_threads (category_id, author_id, title, slug, last_post_at)
                  VALUES ($1, $2, $3, $4, now())
             RETURNING id, category_id, author_id, title, slug, is_pinned, is_locked,
                       view_count, post_count, last_post_at, created_at",
        )
        .bind(category_id)
        .bind(author_id)
        .bind(title)
        .bind(&candidate)
        .fetch_one(&mut *tx)
        .await;
        match res {
            Ok(r) => {
                thread_slug = candidate;
                row = Some(r);
                break;
            }
            Err(sqlx::Error::Database(db))
                if db.constraint() == Some("forum_threads_category_id_slug_key") =>
            {
                continue
            }
            Err(e) => return Err(e.into()),
        }
    }
    let row = row.ok_or_else(|| anyhow::anyhow!("could not allocate thread slug"))?;

    let post: PostRow = sqlx::query_as(
        "INSERT INTO forum_posts (thread_id, author_id, body_md, is_op)
              VALUES ($1, $2, $3, TRUE)
         RETURNING id, thread_id, author_id, body_md, is_op, edited_at, created_at",
    )
    .bind(row.id)
    .bind(author_id)
    .bind(body_md)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE forum_threads SET post_count = 1 WHERE id = $1")
        .bind(row.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let _ = thread_slug;
    Ok((row.into(), post.into()))
}

pub async fn list_threads_in(
    pool: &PgPool,
    category_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<ForumThread>> {
    let rows: Vec<ThreadRow> = sqlx::query_as(
        "SELECT id, category_id, author_id, title, slug, is_pinned, is_locked,
                view_count, post_count, last_post_at, created_at
           FROM forum_threads WHERE category_id = $1
          ORDER BY is_pinned DESC, last_post_at DESC LIMIT $2",
    )
    .bind(category_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn thread_by_slug(
    pool: &PgPool,
    category_slug: &str,
    thread_slug: &str,
) -> anyhow::Result<Option<ForumThread>> {
    let row: Option<ThreadRow> = sqlx::query_as(
        "SELECT t.id, t.category_id, t.author_id, t.title, t.slug, t.is_pinned, t.is_locked,
                t.view_count, t.post_count, t.last_post_at, t.created_at
           FROM forum_threads t
           JOIN forum_categories c ON c.id = t.category_id
          WHERE c.slug = $1 AND t.slug = $2",
    )
    .bind(category_slug)
    .bind(thread_slug)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn bump_thread_view(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE forum_threads SET view_count = view_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------- posts -----------

pub async fn create_post(
    pool: &PgPool,
    thread_id: Uuid,
    author_id: Uuid,
    body_md: &str,
) -> anyhow::Result<ForumPost> {
    let mut tx = pool.begin().await?;
    let row: PostRow = sqlx::query_as(
        "INSERT INTO forum_posts (thread_id, author_id, body_md)
              VALUES ($1, $2, $3)
         RETURNING id, thread_id, author_id, body_md, is_op, edited_at, created_at",
    )
    .bind(thread_id)
    .bind(author_id)
    .bind(body_md)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE forum_threads SET post_count = post_count + 1, last_post_at = now()
          WHERE id = $1",
    )
    .bind(thread_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row.into())
}

pub async fn list_posts(pool: &PgPool, thread_id: Uuid) -> anyhow::Result<Vec<ForumPost>> {
    let rows: Vec<PostRow> = sqlx::query_as(
        "SELECT id, thread_id, author_id, body_md, is_op, edited_at, created_at
           FROM forum_posts WHERE thread_id = $1 ORDER BY created_at ASC",
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

// ---------- row types -----------

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: Uuid,
    slug: String,
    name: String,
    description: String,
    position: i32,
    is_archived: bool,
    created_at: DateTime<Utc>,
}

impl From<CategoryRow> for ForumCategory {
    fn from(r: CategoryRow) -> Self {
        ForumCategory {
            id: r.id,
            slug: r.slug,
            name: r.name,
            description: r.description,
            position: r.position,
            is_archived: r.is_archived,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ThreadRow {
    id: Uuid,
    category_id: Uuid,
    author_id: Uuid,
    title: String,
    slug: String,
    is_pinned: bool,
    is_locked: bool,
    view_count: i64,
    post_count: i32,
    last_post_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ThreadRow> for ForumThread {
    fn from(r: ThreadRow) -> Self {
        ForumThread {
            id: r.id,
            category_id: r.category_id,
            author_id: r.author_id,
            title: r.title,
            slug: r.slug,
            is_pinned: r.is_pinned,
            is_locked: r.is_locked,
            view_count: r.view_count,
            post_count: r.post_count,
            last_post_at: r.last_post_at,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PostRow {
    id: Uuid,
    thread_id: Uuid,
    author_id: Uuid,
    body_md: String,
    is_op: bool,
    edited_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<PostRow> for ForumPost {
    fn from(r: PostRow) -> Self {
        ForumPost {
            id: r.id,
            thread_id: r.thread_id,
            author_id: r.author_id,
            body_md: r.body_md,
            is_op: r.is_op,
            edited_at: r.edited_at,
            created_at: r.created_at,
        }
    }
}
