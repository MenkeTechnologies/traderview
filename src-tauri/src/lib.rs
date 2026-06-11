//! Tauri v2 desktop shell.
//!
//! Crash-resilient version:
//!   * Every launch writes a single log file at
//!     `~/Library/Application Support/com.menketechnologies.traderview/traderview.log`
//!     on macOS (Linux/Windows fall back to platform-appropriate paths
//!     using the same bundle identifier).
//!   * A panic hook captures the message + location + backtrace into
//!     `panic.log` in the same directory before the process aborts.
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

#[derive(Clone, serde::Serialize)]
struct ProcessStats {
    pid: u32,
    app_version: &'static str,
    os: String,
    os_version: Option<String>,
    kernel_version: Option<String>,
    arch: &'static str,
    hostname: String,
    cpus: usize,
    cpu_brand: Option<String>,
    cpu_frequency_mhz: u64,
    cpu_percent: f32,
    load_avg_1: f64,
    load_avg_5: f64,
    load_avg_15: f64,
    rss_bytes: u64,
    virtual_bytes: u64,
    threads: usize,
    uptime_secs: u64,
    system_uptime_secs: u64,
    boot_time_secs: u64,
    total_memory_bytes: u64,
    available_memory_bytes: u64,
    used_swap_bytes: u64,
    total_swap_bytes: u64,
    disk_free_bytes: u64,
    disk_total_bytes: u64,
}

/// One-shot snapshot of the running desktop process — PID, RSS / VIRT,
/// CPU%, threads, uptime, host memory totals. Used by the About widget;
/// cheap enough to call on every refresh (sysinfo's `refresh_*` are
/// targeted, not a full re-walk).
#[tauri::command]
fn get_process_stats() -> ProcessStats {
    use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};
    let pid = std::process::id();
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_memory(sysinfo::MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::everything()),
    );
    // Second refresh gives sysinfo a delta to compute CPU%.
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::everything(),
    );
    std::thread::sleep(std::time::Duration::from_millis(120));
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::everything(),
    );

    let proc = sys.process(Pid::from_u32(pid));
    let rss = proc.map(|p| p.memory()).unwrap_or(0);
    let virt = proc.map(|p| p.virtual_memory()).unwrap_or(0);
    let cpu_percent = proc.map(|p| p.cpu_usage()).unwrap_or(0.0);
    let uptime = proc
        .map(|p| p.run_time())
        .unwrap_or_else(|| {
            // Fallback: seconds since System::uptime if we couldn't read our own.
            System::uptime()
        });

    // Thread count — best-effort via /proc on Linux, mach on macOS, NtQuery
    // on Windows. sysinfo doesn't expose thread count cross-platform, so
    // we count via std on platforms that allow it and report 0 otherwise.
    let threads = thread_count_estimate();
    let load = System::load_average();

    // Disk lookup — pick the root volume (Linux: '/', macOS: '/', Windows: 'C:').
    let mut disks = sysinfo::Disks::new_with_refreshed_list();
    disks.refresh();
    let (disk_free, disk_total) = disks
        .iter()
        .find(|d| {
            let mp = d.mount_point().to_string_lossy();
            mp == "/" || mp.starts_with("C:")
        })
        .map(|d| (d.available_space(), d.total_space()))
        .unwrap_or((0, 0));

    let first_cpu = sys.cpus().first();
    let cpu_brand = first_cpu.map(|c| c.brand().to_string());
    let cpu_frequency_mhz = first_cpu.map(|c| c.frequency()).unwrap_or(0);

    ProcessStats {
        pid,
        app_version: env!("CARGO_PKG_VERSION"),
        os: System::name().unwrap_or_else(|| "unknown".into()),
        os_version: System::long_os_version(),
        kernel_version: System::kernel_version(),
        arch: std::env::consts::ARCH,
        hostname: System::host_name().unwrap_or_else(|| "unknown".into()),
        cpus: sys.cpus().len(),
        cpu_brand,
        cpu_frequency_mhz,
        cpu_percent,
        load_avg_1: load.one,
        load_avg_5: load.five,
        load_avg_15: load.fifteen,
        rss_bytes: rss,
        virtual_bytes: virt,
        threads,
        uptime_secs: uptime,
        system_uptime_secs: System::uptime(),
        boot_time_secs: System::boot_time(),
        total_memory_bytes: sys.total_memory(),
        available_memory_bytes: sys.available_memory(),
        used_swap_bytes: sys.used_swap(),
        total_swap_bytes: sys.total_swap(),
        disk_free_bytes: disk_free,
        disk_total_bytes: disk_total,
    }
}

#[cfg(target_os = "linux")]
fn thread_count_estimate() -> usize {
    std::fs::read_dir(format!("/proc/{}/task", std::process::id()))
        .map(|it| it.count())
        .unwrap_or(0)
}
#[cfg(target_os = "macos")]
fn thread_count_estimate() -> usize {
    // mach_task_threads is a syscall; pulling in mach2 + libc just for
    // one number isn't worth it — return 0 (UI hides the row when 0).
    0
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn thread_count_estimate() -> usize { 0 }

/// Read the tail of the log file — up to the last `max_bytes` (default
/// 64 KiB). Returns the raw text so the frontend can render it in a
/// monospaced viewer; line-splitting and filtering happen client-side
/// so this command stays cheap and untyped.
#[tauri::command]
fn read_log_tail(max_bytes: Option<u64>) -> Result<String, String> {
    use std::io::{Read, Seek, SeekFrom};
    let cap = max_bytes.unwrap_or(64 * 1024);
    let path = log_file_path();
    let mut f = std::fs::File::open(&path).map_err(|e| format!("open log: {e}"))?;
    let len = f.metadata().map_err(|e| format!("stat log: {e}"))?.len();
    let start = len.saturating_sub(cap);
    f.seek(SeekFrom::Start(start))
        .map_err(|e| format!("seek log: {e}"))?;
    let mut buf = Vec::with_capacity(cap as usize);
    f.read_to_end(&mut buf)
        .map_err(|e| format!("read log: {e}"))?;
    // If we sliced mid-character, drop the leading partial line so the
    // viewer doesn't render garbage on the first row.
    let s = String::from_utf8_lossy(&buf).into_owned();
    if start > 0 {
        if let Some(nl) = s.find('\n') {
            return Ok(s[nl + 1..].to_string());
        }
    }
    Ok(s)
}

pub fn run() {
    // Install a default rustls crypto provider for the whole process.
    // Without this, every TLS handshake (live-ticks WS to Alpaca /
    // Polygon / Finnhub, Yahoo bars fetcher, Finnhub REST) panics with
    // "Could not automatically determine the process-level
    // CryptoProvider" on rustls 0.23. Idempotent — second call
    // returns Err which we drop.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

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
        // Persists window position, size, and maximize state per label
        // to <app-data>/window-state.json. Auto-restores on launch.
        .plugin(tauri_plugin_window_state::Builder::default().build())
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
        .invoke_handler(tauri::generate_handler![
            get_api_config,
            get_log_path,
            read_log_tail,
            get_process_stats
        ])
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

    // ── Algo runner + broker fill pumps ───────────────────────────
    // Mirrors `crates/traderview-web/src/bin/server.rs::main`. Without
    // this, the desktop binary's algo subsystem was DEAD: strategies
    // could be created and runs started, but no tick ever drove them
    // → no signals, no orders, no fills, no SKIP / EVAL events, no
    // visibility for the user. bars_processed sat at 0 forever.
    {
        let pool = embedded.pool.clone();
        let sink = state.build_engine_event_sink();
        // Forwarder: pipe live_ticks raw trade tape → realtime hub →
        // frontend tape pane. Bounded broadcast (drops on lag).
        let tape_hub = state.hub.clone();
        let mut tape_rx = traderview_db::live_ticks::global().tape_subscribe();
        tokio::spawn(async move {
            while let Ok(t) = tape_rx.recv().await {
                tape_hub.publish(traderview_web::realtime::Event::Tick {
                    symbol: t.symbol,
                    price: t.price,
                    volume: t.volume,
                    ts_ms: t.ts_ms,
                });
            }
        });
        tokio::spawn(traderview_db::algo_runner::run_loop(
            pool.clone(),
            Some(sink.clone()),
        ));

        // Broker fill pumps — one per real broker. Each pump iterates
        // active strategies and connects the appropriate WS / streamer
        // so fills land in algo_fills + executions via record_fill.
        macro_rules! spawn_pump {
            ($mod_name:ident, $label:literal) => {{
                let p = pool.clone();
                let s = sink.clone();
                let r = state.alpaca_pumps.clone();
                tokio::spawn(async move {
                    match traderview_db::$mod_name::spawn_pumps_for_active_strategies(p, Some(s), r).await {
                        Ok(0) => tracing::info!(concat!("no ", $label, "-bound algo strategies; pumps idle")),
                        Ok(n) => tracing::info!(pumps = n, concat!($label, " pumps spawned")),
                        Err(e) => tracing::warn!(error = %e, concat!($label, " spawn_pumps_for_active_strategies failed")),
                    }
                });
            }};
        }
        spawn_pump!(alpaca_pump, "alpaca");
        spawn_pump!(tradier_pump, "tradier");
        spawn_pump!(tastytrade_pump, "tastytrade");
        spawn_pump!(schwab_pump, "schwab");
        spawn_pump!(ibkr_pump, "ibkr");
    }

    // Background refreshers: precomputed dashboard tiles (sectors,
    // breadth, fear/greed, sector rotation, RRG) + the Golden Stars
    // universe — mirrors server.rs so opening those views never
    // triggers a multi-symbol compute. Intervals live in
    // traderview_web::background.
    traderview_web::background::spawn_refreshers(embedded.pool.clone(), state.tiles.clone(), state.hub.clone());

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
    // Warm Polygon's key too — when present, the live tape prefers
    // Polygon's SIP feed (CTA/UTP) over Finnhub's aggregate.
    match traderview_db::data_source_keys::any_polygon_key(&embedded.pool).await {
        Ok(Some(k)) => {
            traderview_db::live_ticks::global().set_polygon_key(k).await;
            tracing::info!("loaded polygon key from DB; live tape will use SIP feed");
        }
        Ok(None) => {
            tracing::info!("no polygon key configured; live tape falls back to finnhub");
        }
        Err(e) => tracing::warn!(error = %e, "failed to load polygon key from DB"),
    }
    // Warm Alpaca — middle provider in the priority chain.
    match traderview_db::data_source_keys::any_alpaca_creds(&embedded.pool).await {
        Ok(Some((id, secret, use_sip))) => {
            let store = traderview_db::live_ticks::global();
            store.set_alpaca_creds(id, secret).await;
            store.set_alpaca_use_sip(use_sip);
            tracing::info!(
                use_sip,
                "loaded alpaca creds; live tape uses {} feed",
                if use_sip { "SIP" } else { "IEX" }
            );
        }
        Ok(None) => tracing::info!("no alpaca creds configured"),
        Err(e) => tracing::warn!(error = %e, "failed to load alpaca creds from DB"),
    }
    // CRITICAL: hand the DB pool to the live-tick store so the 10s
    // tape aggregator can persist closed buckets into `price_bars`
    // (interval='10s'). Without this call, every incoming WS trade
    // gets bucketed in memory + the bucket crosses 10s + persist_bucket
    // is spawned + reads self.pool → None → silently drops the bar
    // with zero log output. price_bars stays empty forever, the algo
    // runner's fetch_recent_bars returns nothing, every strategy SKIPs
    // on no_bars. The standalone server.rs binary already does this;
    // this commit closes the same gap in the Tauri desktop binary.
    traderview_db::live_ticks::global()
        .set_pool(embedded.pool.clone())
        .await;
    // Always-on reference feed — top liquid names per asset class.
    // Forces both the crypto AND equity WS workers to spawn at boot
    // so the user's tape pane shows both streams without having to
    // create a strategy for each. Equity slot stays silent outside
    // RTH (9:30-16:00 ET) on the IEX free tier — that's an Alpaca
    // limitation, not a bug in the tape.
    const TAPE_AUDIT_FEED: &[&str] = &[
        // Crypto (24/7)
        "BTC/USD", "ETH/USD", "SOL/USD", "DOGE/USD", "AVAX/USD",
        // Equity mega caps (RTH only on IEX)
        "SPY", "QQQ", "AAPL", "MSFT", "NVDA", "TSLA",
    ];
    {
        let store = traderview_db::live_ticks::global();
        if store.has_any_provider().await {
            let syms: Vec<String> = TAPE_AUDIT_FEED.iter().map(|s| (*s).to_string()).collect();
            let n = syms.len();
            if let Err(e) = store.ensure_subscribed(syms).await {
                tracing::warn!(error = %e, "boot audit-feed subscribe failed");
            } else {
                tracing::info!(n, "boot subscribed audit feed (crypto + equity)");
            }
        }
    }

    // Boot-time push of the watchlist union into the live-tick
    // subscription set. Without this the WS workers stay idle until
    // the user mutates a watchlist or the candidates/scanner loop
    // fires — neither happens at startup, so existing BTCUSD /
    // AAPL / etc. rows would never stream until manually re-added.
    {
        let store = traderview_db::live_ticks::global();
        if store.has_any_provider().await {
            match traderview_db::watchlists::all_distinct_symbols(&embedded.pool).await {
                Ok(symbols) if !symbols.is_empty() => {
                    let n = symbols.len();
                    if let Err(e) = store.set_symbols(symbols).await {
                        tracing::warn!(error = %e, "boot watchlist push failed");
                    } else {
                        tracing::info!(n, "boot pushed watchlist symbols to live_ticks");
                    }
                }
                Ok(_) => tracing::info!("boot: no watchlist symbols to push"),
                Err(e) => tracing::warn!(error = %e, "boot watchlist scan failed"),
            }
        }
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

/// Bundle identifier — kept in sync with `tauri.conf.json::identifier`.
/// Logs, PG data, JWT secret, and Tauri-managed state all live under
/// the directory keyed by this string so the entire app's on-disk
/// footprint is one tree under the platform's app-data root.
const APP_BUNDLE_ID: &str = "com.menketechnologies.traderview";

fn log_dir() -> PathBuf {
    // Single canonical directory — same root the embedded Postgres,
    // jwt-secret, and Tauri window-state file already use:
    //   macOS   : ~/Library/Application Support/com.menketechnologies.traderview/
    //   Linux   : ~/.local/share/com.menketechnologies.traderview/
    //   Windows : %APPDATA%/com.menketechnologies.traderview/
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home
                .join("Library")
                .join("Application Support")
                .join(APP_BUNDLE_ID);
        }
    }
    if let Some(base) = dirs::data_local_dir() {
        return base.join(APP_BUNDLE_ID);
    }
    std::env::temp_dir().join(APP_BUNDLE_ID)
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
    fn log_dir_ends_in_bundle_id_segment() {
        // Every supported platform branch composes a path ending in
        // the bundle identifier — the panic hook and rolling appender
        // both rely on this for predictable on-disk layout, and it
        // matches where the embedded PG / jwt-secret already live.
        let p = log_dir();
        assert_eq!(
            p.file_name().and_then(|s| s.to_str()),
            Some(APP_BUNDLE_ID),
            "log_dir should end in '{APP_BUNDLE_ID}', got: {}",
            p.display()
        );
    }

    #[test]
    fn log_file_path_appends_log_filename() {
        // The Tauri shell writes
        // ~/Library/Application Support/com.menketechnologies.traderview/traderview.log
        // (macOS) or platform-equivalent. Filename pinned in the module header.
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
        let expected = format!("Library/Application Support/{APP_BUNDLE_ID}");
        assert!(
            s.contains(&expected),
            "expected {expected} in path, got: {s}"
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
