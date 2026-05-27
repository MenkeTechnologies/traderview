//! Embedded Postgres lifecycle for the desktop (Tauri) build.
//!
//! On first launch we download a portable PostgreSQL into the app data dir
//! and start it on a free port. The same data dir is reused across launches.

use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::path::{Path, PathBuf};

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

        // CRITICAL: `Settings::default()` randomizes the password every launch.
        // The cluster on disk was initialized with a SPECIFIC password the first
        // time, so a new random on the second launch makes auth fail. Persist
        // our own password file alongside the cluster and reuse it forever.
        let pw_path = data_dir.join("pg-password");
        let password = load_or_create_password(&pw_path)?;

        let settings = Settings {
            installation_dir: data_dir.join("pg-bin"),
            data_dir: data_dir.join("pg-data"),
            password_file: pw_path,
            password,
            temporary: false,
            ..Default::default()
        };

        tracing::info!(
            installation_dir = %settings.installation_dir.display(),
            data_dir = %settings.data_dir.display(),
            "starting embedded postgres"
        );

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

/// Read a persisted password from `path`. If it doesn't exist (or is empty),
/// generate a 32-byte url-safe random and write it.
fn load_or_create_password(path: &Path) -> anyhow::Result<String> {
    use rand::RngCore;
    if let Ok(s) = std::fs::read_to_string(path) {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    let pw = hex::encode(buf);
    // Use restrictive perms so other users on the box can't read it.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        use std::io::Write;
        f.write_all(pw.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, &pw)?;
    }
    tracing::info!(path = %path.display(), "generated new embedded postgres password");
    Ok(pw)
}
