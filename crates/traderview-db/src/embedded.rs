//! Embedded Postgres lifecycle for the desktop (Tauri) build.
//!
//! On first launch we download a portable PostgreSQL into the app data dir
//! and start it on a free port. The same data dir is reused across launches.

use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::path::PathBuf;

/// Handle to a running embedded Postgres + sqlx pool.
///
/// Dropping this struct stops the database (best-effort).
pub struct Embedded {
    pub pool: PgPool,
    pub database_url: String,
    pg: PostgreSQL,
}

impl Embedded {
    /// Start (and if necessary initialize) embedded Postgres under `data_dir`,
    /// create the `traderview` database, and return a connected pool.
    pub async fn start(data_dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;

        let settings = Settings {
            installation_dir: data_dir.join("pg-bin"),
            data_dir: data_dir.join("pg-data"),
            password_file: data_dir.join("pg-password"),
            temporary: false,
            ..Default::default()
        };

        tracing::info!(?settings.installation_dir, ?settings.data_dir, "starting embedded postgres");

        let mut pg = PostgreSQL::new(settings);
        pg.setup().await?;
        pg.start().await?;

        let db_name = "traderview";
        if !pg.database_exists(db_name).await? {
            pg.create_database(db_name).await?;
        }
        let database_url = pg.settings().url(db_name);

        let pool = super::connect_external(&database_url).await?;
        super::migrate(&pool).await?;

        Ok(Self {
            pool,
            database_url,
            pg,
        })
    }

    /// Best-effort graceful stop. Failure logged, not propagated.
    pub async fn shutdown(self) {
        let Self { pg, .. } = self;
        if let Err(e) = pg.stop().await {
            tracing::warn!(error = %e, "embedded postgres stop failed");
        }
    }
}
