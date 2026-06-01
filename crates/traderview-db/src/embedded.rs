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

        let pg_data = settings.data_dir.clone();

        // If a prior launch was SIGKILL'd / OOM'd, the postmaster process is
        // gone but the lockfile remains, and pg_ctl will refuse to start
        // ("another server might be running"). Detect a stale lock by reading
        // the PID from line 1 of postmaster.pid and checking liveness; if the
        // PID is dead, remove the lock so the next start() can succeed.
        clean_stale_lock(&pg_data);

        let action = resolve_pg_action(&settings).await?;
        let (settings, start_fresh) = match action {
            PgAction::Reuse(port) => {
                tracing::info!(port, "reusing existing embedded postgres");
                let mut s = settings;
                s.port = port;
                (s, false)
            }
            PgAction::StartFresh => (settings, true),
        };

        let mut pg = PostgreSQL::new(settings);
        pg.setup().await?;

        if start_fresh {
            if let Err(e) = pg.start().await {
                // Race: another start between resolve and pg.start(), or stale
                // lock we couldn't detect — try connecting to whatever is up.
                if let Some(port) = read_postmaster_port(&pg_data) {
                    if can_connect_bootstrap(&pg.settings().clone(), port).await {
                        tracing::warn!(
                            error = %e,
                            port,
                            "pg.start failed but existing instance is reachable; reusing"
                        );
                        let mut reuse = pg.settings().clone();
                        reuse.port = port;
                        pg = PostgreSQL::new(reuse);
                        pg.setup().await?;
                    } else {
                        return Err(e.into());
                    }
                } else {
                    return Err(e.into());
                }
            }
        }

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

enum PgAction {
    Reuse(u16),
    StartFresh,
}

/// Decide whether to attach to an already-running cluster or start a new one.
async fn resolve_pg_action(settings: &Settings) -> anyhow::Result<PgAction> {
    let pg_data = &settings.data_dir;
    let pid_file = pg_data.join("postmaster.pid");
    if !pid_file.exists() {
        return Ok(PgAction::StartFresh);
    }

    let Some((pid, port)) = read_postmaster_info(pg_data) else {
        return Ok(PgAction::StartFresh);
    };

    if !pid_is_alive(pid) {
        return Ok(PgAction::StartFresh);
    }

    if can_connect_bootstrap(settings, port).await {
        return Ok(PgAction::Reuse(port));
    }

    tracing::warn!(
        pid,
        port,
        "postmaster.pid live but connection failed; stopping orphan cluster"
    );
    stop_cluster(settings).await?;
    Ok(PgAction::StartFresh)
}

async fn can_connect_bootstrap(settings: &Settings, port: u16) -> bool {
    let mut s = settings.clone();
    s.port = port;
    let url = s.url("postgres");
    super::connect_external(&url).await.is_ok()
}

/// pg_ctl stop on the data dir, then remove the lockfile if it remains.
async fn stop_cluster(settings: &Settings) -> anyhow::Result<()> {
    let pg = PostgreSQL::new(settings.clone());
    if let Err(e) = pg.stop().await {
        tracing::warn!(error = %e, "pg_ctl stop failed (best-effort)");
    }
    let pid_file = settings.data_dir.join("postmaster.pid");
    if pid_file.exists() {
        let _ = std::fs::remove_file(&pid_file);
    }
    Ok(())
}

/// Parse line 1 (PID) and line 4 (port) from `postmaster.pid`.
fn read_postmaster_info(pg_data: &Path) -> Option<(i32, u16)> {
    let port = read_postmaster_port(pg_data)?;
    let pid_file = pg_data.join("postmaster.pid");
    let contents = std::fs::read_to_string(&pid_file).ok()?;
    let pid: i32 = contents.lines().next()?.trim().parse().ok()?;
    Some((pid, port))
}

fn read_postmaster_port(pg_data: &Path) -> Option<u16> {
    let pid_file = pg_data.join("postmaster.pid");
    let contents = std::fs::read_to_string(&pid_file).ok()?;
    let port_line = contents.lines().nth(3)?;
    port_line.trim().parse().ok()
}

#[cfg(unix)]
fn pid_is_alive(pid: i32) -> bool {
    // kill(pid, 0) probes existence without delivering a signal.
    unsafe { libc::kill(pid, 0) == 0 }
}

#[cfg(not(unix))]
fn pid_is_alive(_pid: i32) -> bool {
    true
}

/// Remove a stale `postmaster.pid` whose recorded PID is not alive.
/// Safe no-op if the file is missing or unreadable.
fn clean_stale_lock(pg_data: &Path) {
    let pid_file = pg_data.join("postmaster.pid");
    let Ok(contents) = std::fs::read_to_string(&pid_file) else {
        return;
    };
    let Some(first) = contents.lines().next() else {
        return;
    };
    let Ok(pid) = first.trim().parse::<i32>() else {
        return;
    };

    if !pid_is_alive(pid) {
        tracing::warn!(
            pid_file = %pid_file.display(),
            stale_pid = pid,
            "removing stale postmaster.pid (recorded PID is dead)"
        );
        let _ = std::fs::remove_file(&pid_file);
    } else {
        tracing::warn!(
            pid_file = %pid_file.display(),
            live_pid = pid,
            "postmaster.pid recorded a live PID; not removing — start may fail"
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Build an isolated temp pg-data dir under the OS temp root, unique per
    /// test invocation. We don't pull in `tempfile` for one helper.
    fn temp_pg_data(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "traderview-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&p).expect("create temp pg-data");
        p
    }

    fn write_pid_file(pg_data: &Path, pid_line: &str) {
        let mut f =
            std::fs::File::create(pg_data.join("postmaster.pid")).expect("create postmaster.pid");
        f.write_all(pid_line.as_bytes()).expect("write pid");
    }

    #[test]
    #[cfg(unix)]
    fn clean_stale_lock_removes_dead_pid() {
        let dir = temp_pg_data("dead-pid");
        // PID 2^30 — pid_max is typically 2^15 / 2^22 on Linux/macOS so this
        // is guaranteed dead. Also can't be the init process (PID 1).
        write_pid_file(&dir, "1073741824\n13434\n1700000000\n5432\n");
        let pid_file = dir.join("postmaster.pid");
        assert!(pid_file.exists());
        clean_stale_lock(&dir);
        assert!(!pid_file.exists(), "stale lock should have been removed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(unix)]
    fn clean_stale_lock_keeps_live_pid() {
        let dir = temp_pg_data("live-pid");
        // Use our own PID — guaranteed alive while this test runs.
        let me = std::process::id();
        write_pid_file(&dir, &format!("{me}\n13434\n1700000000\n5432\n"));
        let pid_file = dir.join("postmaster.pid");
        clean_stale_lock(&dir);
        assert!(pid_file.exists(), "live PID lockfile should NOT be removed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clean_stale_lock_noop_when_file_missing() {
        let dir = temp_pg_data("missing");
        // Don't create the file at all.
        clean_stale_lock(&dir);
        // Test passes if no panic / no error.
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clean_stale_lock_noop_when_garbage() {
        let dir = temp_pg_data("garbage");
        write_pid_file(&dir, "not-a-number\n");
        let pid_file = dir.join("postmaster.pid");
        clean_stale_lock(&dir);
        // Unparseable PID → leave the file alone; the embedded PG layer will
        // give the real error message.
        assert!(pid_file.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_postmaster_port_parses_line_four() {
        let dir = temp_pg_data("port-parse");
        write_pid_file(&dir, "12345\n/data\n1700000000\n5433\n");
        assert_eq!(read_postmaster_port(&dir), Some(5433));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_postmaster_info_parses_pid_and_port() {
        let dir = temp_pg_data("info-parse");
        write_pid_file(&dir, "99999\n/data\n1700000000\n5434\n");
        assert_eq!(read_postmaster_info(&dir), Some((99999, 5434)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_or_create_password_is_deterministic_across_calls() {
        let dir = temp_pg_data("pw");
        let path = dir.join("pg-password");
        let pw1 = load_or_create_password(&path).expect("first call");
        let pw2 = load_or_create_password(&path).expect("second call");
        assert_eq!(
            pw1, pw2,
            "load_or_create_password must return the same password \
             once the file exists — randomizing on relaunch is what \
             caused the auth-failed crashes in pass 1"
        );
        assert!(!pw1.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
