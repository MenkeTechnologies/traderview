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
                    show_fatal_dialog(app, &format!("app_data_dir: {e}"));
                    return Err(Box::new(std::io::Error::other(
                        format!("app_data_dir: {e}"),
                    )));
                }
            };
            if let Err(e) = std::fs::create_dir_all(&data_dir) {
                tracing::error!(?e, "mkdir app data");
                show_fatal_dialog(app, &format!("mkdir {}: {e}", data_dir.display()));
                return Err(Box::new(e));
            }
            tracing::info!(data_dir = %data_dir.display(), "data dir ready");

            let secret_path = data_dir.join("jwt-secret");
            let jwt_secret = match load_or_create_secret(&secret_path) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(?e, "secret load/create failed");
                    show_fatal_dialog(app, &format!("jwt-secret: {e}"));
                    return Err(Box::new(std::io::Error::other(
                        format!("jwt-secret: {e}"),
                    )));
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
                    show_fatal_dialog(app, &format!("tokio runtime: {e}"));
                    return Err(Box::new(e));
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
                    show_fatal_dialog(
                        app,
                        &format!(
                            "TraderView backend failed to start:\n\n{err_msg}\n\nLog: {}",
                            log_file_path().display()
                        ),
                    );
                    Err(Box::new(std::io::Error::other(
                        format!("backend failed: {err_msg}"),
                    )))
                }
                Err(e) => {
                    tracing::error!(?e, "backend boot timeout/disconnect");
                    show_fatal_dialog(
                        app,
                        &format!(
                            "TraderView backend did not start within 5 minutes.\n\nLog: {}\n\nFirst-run downloads ~80MB of PostgreSQL — check your network.",
                            log_file_path().display()
                        ),
                    );
                    Err(Box::new(std::io::Error::other(
                        format!("backend timeout: {e}"),
                    )))
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

    Ok((ApiConfig { base_url, token }, listener, app_router, embedded))
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
