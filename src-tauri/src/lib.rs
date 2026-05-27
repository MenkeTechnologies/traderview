//! Tauri v2 desktop shell.
//!
//! On launch:
//!   1. Resolve app data dir.
//!   2. Start embedded Postgres (downloads + extracts on first run).
//!   3. Run migrations.
//!   4. Ensure local user exists, mint a long-lived JWT.
//!   5. Start axum on a free localhost port.
//!   6. Expose a Tauri command `get_api_config` returning {base_url, token}
//!      that the frontend reads on startup.

use std::net::SocketAddr;
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

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "traderview_desktop=info,traderview_web=info,traderview_db=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // App data dir lives outside the bundle; embedded postgres + secrets land here.
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("app_data_dir resolvable on all targets")
                .join("traderview");
            std::fs::create_dir_all(&data_dir).expect("mkdir app data");

            // Persist a JWT secret across launches so issued tokens stay valid.
            let secret_path = data_dir.join("jwt-secret");
            let jwt_secret = load_or_create_secret(&secret_path)?;

            // Bring up postgres + axum on a background runtime, then post the
            // resulting ApiConfig back to the main thread before the window loads.
            let rt = Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("traderview-rt")
                    .build()
                    .expect("tokio runtime"),
            );

            let (config_tx, config_rx) = std::sync::mpsc::channel::<ApiConfig>();
            let rt_clone = rt.clone();
            let data_dir_clone = data_dir.clone();
            let jwt_secret_clone = jwt_secret.clone();

            std::thread::spawn(move || {
                rt_clone.block_on(async move {
                    let pg_dir = data_dir_clone.clone();
                    let embedded = Embedded::start(pg_dir)
                        .await
                        .expect("embedded postgres start");

                    let user_id = traderview_db::repo::ensure_local_user(&embedded.pool)
                        .await
                        .expect("ensure local user");

                    let token = traderview_web::auth::issue_token(
                        &jwt_secret_clone,
                        user_id,
                        24 * 365 * 10, // 10y — desktop token, rotated only if user resets data
                    )
                    .expect("issue token");

                    let state = AppState::new(
                        embedded.pool.clone(),
                        AppMode::Desktop,
                        jwt_secret_clone,
                        data_dir_clone,
                    );
                    let app = router(state);

                    let bind: SocketAddr = "127.0.0.1:0".parse().unwrap();
                    let listener = tokio::net::TcpListener::bind(bind).await.expect("bind");
                    let addr = listener.local_addr().expect("local addr");
                    let base_url = format!("http://{addr}");

                    config_tx
                        .send(ApiConfig {
                            base_url: base_url.clone(),
                            token,
                        })
                        .ok();

                    tracing::info!(%base_url, "axum serving");
                    if let Err(e) = axum::serve(listener, app).await {
                        tracing::error!(error = %e, "axum serve failed");
                    }
                });
            });

            // Block briefly for the server to come up. ~First-run downloads PG;
            // we wait up to 5 minutes for that path. Subsequent runs are <1s.
            let config = config_rx
                .recv_timeout(std::time::Duration::from_secs(300))
                .map_err(|e| anyhow::anyhow!("backend boot timeout: {e}"))?;

            app.manage(DesktopState { config });
            // Keep the runtime alive for the program lifetime.
            app.manage(rt);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_api_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
