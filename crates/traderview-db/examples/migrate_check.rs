//! One-shot: run the crate's embedded migrations against an existing DB.
//!
//! Run:
//!   cargo run -p traderview-db --example migrate_check -- "postgres://postgres:PW@localhost:PORT/traderview"

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: migrate_check <database-url>"))?;
    let pool = sqlx::PgPool::connect(&url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    println!("migrations ok");
    Ok(())
}
