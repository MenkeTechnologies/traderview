//! Tauri v2 desktop shell.
//!
//! Crash-resilient version:
//!   * Every launch writes a rolling log to `~/Library/Logs/TraderView/desktop.log.YYYY-MM-DD`
//!     on macOS (Linux/Windows fall back to platform-appropriate paths).
//!   * A panic hook captures the message + location + backtrace into
//!     `~/Library/Logs/TraderView/panic.log` before the process aborts.
//!   * Backend bring-up (embedded Postgres → migrations → axum) runs in a
//!     worker thread; the main thread waits on a channel that carries either
//!     an `ApiConfig` (success) or a String error (failure). On error, we
//!     surface a native dialog with the log path and exit cleanly instead
//!     of SIGABRT-ing.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use traderview_db::embedded::Embedded;
use traderview_web::{router, AppMode, AppState};

#[derive(Clone, serde::Serialize)]
struct ApiConfig {
    base_url: String,
    token: String,
}

struct DesktopState {
    config: ApiConfig,
}

#[tauri::command]
fn get_api_config(state: tauri::State<'_, DesktopState>) -> ApiConfig {
    state.config.clone()
}

#[tauri::command]
fn get_log_path() -> String {
    log_file_path().display().to_string()
}

pub fn run() {
    // ---- file logging + panic hook ----------------------------------
    let log_dir_path = log_dir();
    let _ = std::fs::create_dir_all(&log_dir_path);

    // Single fixed file at ~/Library/Application Support/traderview/traderview.log
    // (no rotation — matches the user-requested path).
    let file_appender = tracing_appender::rolling::never(&log_dir_path, "traderview.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Default filter is *deliberately loud* — when widgets break we need to
    // see every failing query, every 4xx/5xx response, every dropped sqlx
    // connection. Override at runtime via RUST_LOG.
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "info,\
         traderview_desktop=debug,\
         traderview_web=debug,\
         traderview_db=debug,\
         traderview_core=info,\
         traderview_import=info,\
         tower_http=debug,\
         axum=debug,\
         sqlx=warn,\
         hyper=warn,\
         hyper_util=warn,\
         reqwest=warn"
            .into()
    });

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true),
        )
        .try_init();

    install_panic_hook(log_dir_path.clone());

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        log_path = %log_file_path().display(),
        "traderview-desktop starting"
    );

    // Leak the appender guard so logs flush for the whole lifetime.
    std::mem::forget(guard);

    // ---- tauri setup -------------------------------------------------
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            tracing::info!("setup: resolving paths");
            let data_dir = match app.path().app_data_dir() {
                Ok(d) => d.join("traderview"),
                Err(e) => {
                    tracing::error!(?e, "app_data_dir failed");
                    fatal_startup(app, &format!("app_data_dir: {e}"));
                }
            };
            if let Err(e) = std::fs::create_dir_all(&data_dir) {
                tracing::error!(?e, "mkdir app data");
                fatal_startup(app, &format!("mkdir {}: {e}", data_dir.display()));
            }
            tracing::info!(data_dir = %data_dir.display(), "data dir ready");

            let secret_path = data_dir.join("jwt-secret");
            let jwt_secret = match load_or_create_secret(&secret_path) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(?e, "secret load/create failed");
                    fatal_startup(app, &format!("jwt-secret: {e}"));
                }
            };
            tracing::info!("jwt secret loaded");

            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("traderview-rt")
                .build()
            {
                Ok(rt) => Arc::new(rt),
                Err(e) => {
                    tracing::error!(?e, "tokio runtime build failed");
                    fatal_startup(app, &format!("tokio runtime: {e}"));
                }
            };

            // Channel: Ok(ApiConfig) on success, Err(String) on failure (no panic).
            let (config_tx, config_rx) =
                std::sync::mpsc::channel::<Result<ApiConfig, String>>();
            let rt_clone = rt.clone();
            let data_dir_clone = data_dir.clone();
            let jwt_secret_clone = jwt_secret.clone();

            let _ = std::thread::Builder::new()
                .name("traderview-backend".into())
                .spawn(move || {
                    rt_clone.block_on(async move {
                        match bring_up_backend(data_dir_clone, jwt_secret_clone).await {
                            Ok((config, listener, app_router, embedded)) => {
                                let _ = config_tx.send(Ok(config));
                                tracing::info!("axum starting");
                                if let Err(e) = axum::serve(listener, app_router).await {
                                    tracing::error!(error = %e, "axum serve failed");
                                }
                                // CRITICAL: keep `embedded` alive until axum::serve
                                // returns. Dropping the Embedded wrapper triggers
                                // postgresql_embedded's pg_ctl stop in its Drop
                                // impl, which kills every pooled connection and
                                // makes every subsequent query time out at 10s.
                                drop(embedded);
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "backend bring-up failed");
                                let _ = config_tx.send(Err(format!("{e:#}")));
                            }
                        }
                    });
                });

            tracing::info!("waiting for backend (timeout 5 min)…");
            match config_rx.recv_timeout(std::time::Duration::from_secs(300)) {
                Ok(Ok(config)) => {
                    tracing::info!(base_url = %config.base_url, "backend ready");
                    app.manage(DesktopState { config });
                    app.manage(rt);
                    Ok(())
                }
                Ok(Err(err_msg)) => {
                    tracing::error!(%err_msg, "backend failed");
                    fatal_startup(
                        app,
                        &format!(
                            "TraderView backend failed to start:\n\n{err_msg}\n\nLog: {}",
                            log_file_path().display()
                        ),
                    );
                }
                Err(e) => {
                    tracing::error!(?e, "backend boot timeout/disconnect");
                    fatal_startup(
                        app,
                        &format!(
                            "TraderView backend did not start within 5 minutes.\n\nLog: {}\n\nFirst-run downloads ~80MB of PostgreSQL — check your network.",
                            log_file_path().display()
                        ),
                    );
                }
            }
        })
        .invoke_handler(tauri::generate_handler![get_api_config, get_log_path])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!(error = %e, "tauri exited with error");
        eprintln!("TraderView exited: {e}\nLog: {}", log_file_path().display());
        std::process::exit(1);
    }
}

async fn bring_up_backend(
    data_dir: PathBuf,
    jwt_secret: Vec<u8>,
) -> anyhow::Result<(ApiConfig, tokio::net::TcpListener, axum::Router, Embedded)> {
    tracing::info!("starting embedded postgres (first run downloads ~80MB)");
    let embedded = Embedded::start(data_dir.clone())
        .await
        .map_err(|e| anyhow::anyhow!("embedded postgres start: {e:#}"))?;
    tracing::info!("embedded postgres up");

    let user_id = traderview_db::users::ensure_local(&embedded.pool)
        .await
        .map_err(|e| anyhow::anyhow!("ensure local user: {e:#}"))?;
    tracing::info!(?user_id, "local user ready");

    let token = traderview_web::auth::issue_token(&jwt_secret, user_id, 24 * 365 * 10)
        .map_err(|e| anyhow::anyhow!("issue token: {e:#}"))?;

    let state = AppState::new(
        embedded.pool.clone(),
        AppMode::Desktop,
        jwt_secret,
        data_dir.clone(),
    );
    let app_router = router(state);

    // Warm Finnhub API key from DB into the LiveTickStore's in-memory
    // slot — mirrors the same boot step in `traderview-web/bin/server.rs`.
    // Without this, REST callers (finnhub_rest, market_data fundamentals/
    // earnings/recommendations, IPO/news/calendar routes) fail with "key
    // not configured" after every restart even when the user has saved
    // the key in Settings → Data Sources.
    match traderview_db::data_source_keys::any_finnhub_key(&embedded.pool).await {
        Ok(Some(k)) => {
            traderview_db::live_ticks::global().set_api_key(k).await;
            tracing::info!("loaded finnhub key from DB into live_ticks store");
        }
        Ok(None) => {
            tracing::info!("no finnhub key configured; set one in Settings → Data Sources");
        }
        Err(e) => tracing::warn!(error = %e, "failed to load finnhub key from DB"),
    }

    // Background pollers — best-effort, return value ignored.
    {
        let pool = embedded.pool.clone();
        tokio::spawn(async move {
            loop {
                let _ = traderview_db::disclosures::poll_all(&pool).await;
                tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            }
        });
    }
    {
        let pool = embedded.pool.clone();
        tokio::spawn(async move {
            loop {
                let _ = traderview_db::sentiment::poll_all(&pool).await;
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    let bind: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(bind).await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{addr}");
    tracing::info!(%base_url, "axum bound");

    Ok((
        ApiConfig { base_url, token },
        listener,
        app_router,
        embedded,
    ))
}

fn load_or_create_secret(path: &std::path::Path) -> anyhow::Result<Vec<u8>> {
    use rand::RngCore;
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(decoded) = hex::decode(&bytes) {
            if decoded.len() >= 32 {
                return Ok(decoded);
            }
        }
    }
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    std::fs::write(path, hex::encode(buf))?;
    Ok(buf.to_vec())
}

// ===========================================================================
// Logging + panic infrastructure
// ===========================================================================

fn log_dir() -> PathBuf {
    // Single canonical directory:
    //   macOS   : ~/Library/Application Support/traderview/
    //   Linux   : ~/.local/share/traderview/
    //   Windows : %APPDATA%/traderview/
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home
                .join("Library")
                .join("Application Support")
                .join("traderview");
        }
    }
    if let Some(base) = dirs::data_local_dir() {
        return base.join("traderview");
    }
    std::env::temp_dir().join("traderview")
}

fn log_file_path() -> PathBuf {
    log_dir().join("traderview.log")
}

fn install_panic_hook(log_dir: PathBuf) {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info.to_string();
        let bt = std::backtrace::Backtrace::force_capture();
        let line = format!(
            "[panic] {} pid={}\n{msg}\n--- backtrace ---\n{bt}\n",
            chrono::Utc::now().to_rfc3339(),
            std::process::id(),
        );
        let panic_file = log_dir.join("panic.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&panic_file)
        {
            use std::io::Write;
            let _ = writeln!(f, "{line}");
        }
        tracing::error!(target: "panic", "{}", line);
        prev(info);
    }));
}

fn show_fatal_dialog(app: &tauri::App, message: &str) {
    use tauri_plugin_dialog::DialogExt;
    let _ = app
        .dialog()
        .message(message)
        .title("TraderView · startup failed")
        .blocking_show();
}

/// Show a native error dialog then exit. Returning `Err` from Tauri's setup
/// hook triggers a panic in tao's `did_finish_launching` (SIGABRT).
fn fatal_startup(app: &tauri::App, message: &str) -> ! {
    show_fatal_dialog(app, message);
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ── log_dir / log_file_path: platform-aware path resolution ───────────

    #[test]
    fn log_dir_ends_in_traderview_segment() {
        // Every supported platform branch composes a path ending in
        // "traderview" — the panic hook and rolling appender both rely on
        // this for predictable on-disk layout.
        let p = log_dir();
        assert_eq!(
            p.file_name().and_then(|s| s.to_str()),
            Some("traderview"),
            "log_dir should end in 'traderview', got: {}",
            p.display()
        );
    }

    #[test]
    fn log_file_path_appends_log_filename() {
        // The Tauri shell writes ~/Library/Application Support/traderview/traderview.log
        // (macOS) or platform-equivalent. The filename is documented in the
        // module header — pin it.
        let p = log_file_path();
        assert_eq!(
            p.file_name().and_then(|s| s.to_str()),
            Some("traderview.log"),
            "log_file_path should end in 'traderview.log', got: {}",
            p.display()
        );
        // And the parent must be log_dir.
        assert_eq!(p.parent(), Some(log_dir().as_path()));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn macos_log_dir_lives_under_application_support() {
        // The doc-string commits to the macOS-specific path; if anyone moves
        // it to ~/Library/Logs without updating both code AND docs, this trips.
        let p = log_dir();
        let s = p.to_string_lossy();
        assert!(
            s.contains("Library/Application Support/traderview"),
            "expected Library/Application Support/traderview in path, got: {s}"
        );
    }

    // ── load_or_create_secret: JWT-secret durability across runs ──────────

    #[test]
    fn load_or_create_secret_creates_32_bytes_on_first_run() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jwt-secret");
        // First call: file doesn't exist → fresh 32-byte secret is written.
        let s = load_or_create_secret(&path).unwrap();
        assert_eq!(s.len(), 32);
        assert!(path.exists());
        // On-disk format is hex (64 chars for 32 bytes).
        let on_disk = std::fs::read(&path).unwrap();
        assert_eq!(on_disk.len(), 64);
        assert!(on_disk.iter().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn load_or_create_secret_returns_same_bytes_on_second_run() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jwt-secret");
        let first = load_or_create_secret(&path).unwrap();
        let second = load_or_create_secret(&path).unwrap();
        // Persistence is the whole point — losing the secret invalidates every
        // issued JWT and forces every user to re-auth.
        assert_eq!(first, second);
    }

    #[test]
    fn load_or_create_secret_regenerates_when_file_is_garbled() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jwt-secret");
        // Write something that isn't hex — load path should fall through to
        // generating a fresh secret rather than panicking or returning empty.
        std::fs::write(&path, b"not-valid-hex-at-all-zzzz").unwrap();
        let s = load_or_create_secret(&path).unwrap();
        assert_eq!(s.len(), 32);
    }

    #[test]
    fn load_or_create_secret_regenerates_when_decoded_too_short() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jwt-secret");
        // Valid hex but only 16 bytes decoded — below the 32-byte safety floor.
        // load_or_create_secret must rewrite a fresh 32-byte secret.
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"00112233445566778899aabbccddeeff").unwrap();
        drop(f);
        let s = load_or_create_secret(&path).unwrap();
        assert_eq!(s.len(), 32);
    }

    #[test]
    fn load_or_create_secret_two_fresh_dirs_yield_different_secrets() {
        // Sanity check the RNG path — two new installations must never
        // accidentally collide. With 2^256 keyspace, equality means the path
        // returned a cached/constant value, which would be catastrophic.
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let sa = load_or_create_secret(&a.path().join("s")).unwrap();
        let sb = load_or_create_secret(&b.path().join("s")).unwrap();
        assert_ne!(sa, sb);
    }
}
