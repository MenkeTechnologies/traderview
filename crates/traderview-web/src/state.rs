use sqlx::PgPool;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

use crate::realtime::Hub;

// Registry type lives in traderview-db so the pump module owns it. Web
// re-exports for convenience in route handlers.
pub use traderview_db::alpaca_pump::AlpacaPumpRegistry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMode {
    /// Standalone web — full registration/login required.
    Web,
    /// Tauri desktop — single local user, auth middleware auto-issues tokens.
    Desktop,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub mode: AppMode,
    pub jwt_secret: Arc<Vec<u8>>,
    /// Root directory for on-disk artifacts: receipts/, models/paddleocr/, etc.
    /// Desktop = Tauri app_data_dir + "/traderview". Web = `TRADERVIEW_DATA_DIR`
    /// env var, defaulting to `./data`.
    pub data_dir: Arc<PathBuf>,
    pub hub: Hub,
    /// Bounded permit pool for OCR background jobs. Without this,
    /// uploading 10k photos via the folder scanner would spawn 10k
    /// tesseract child processes racing for CPU + RAM. Permits = min(4,
    /// num_cpus) — small enough to leave the WebView and Postgres room
    /// to breathe on a laptop, large enough to keep the queue draining.
    pub ocr_sem: Arc<Semaphore>,
    /// Shared registry of running Alpaca trade_updates pumps, keyed by
    /// (user_id, paper-vs-live). Populated by both the startup pump
    /// spawn and routes/algo.rs on strategy create/update — so a
    /// freshly-created alpaca-bound strategy gets its pump immediately
    /// instead of waiting for a server restart.
    pub alpaca_pumps: AlpacaPumpRegistry,
}

impl AppState {
    pub fn new(pool: PgPool, mode: AppMode, jwt_secret: Vec<u8>, data_dir: PathBuf) -> Self {
        let permits = std::thread::available_parallelism()
            .map(|n| n.get().clamp(1, 4))
            .unwrap_or(2);
        Self {
            pool,
            mode,
            jwt_secret: Arc::new(jwt_secret),
            data_dir: Arc::new(data_dir),
            hub: Hub::new(),
            ocr_sem: Arc::new(Semaphore::new(permits)),
            alpaca_pumps: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Build an EventSink that maps engine events → realtime::Event via
    /// this state's hub. Used by both the startup pump spawn and the
    /// route layer's ensure_pump_for so freshly-hot-spawned pumps emit
    /// to the same WS stream.
    pub fn build_engine_event_sink(&self) -> traderview_db::algo_engine::EventSink {
        let hub = self.hub.clone();
        Arc::new(move |ev: traderview_db::algo_engine::EngineEvent| {
            use rust_decimal::prelude::ToPrimitive;
            use traderview_db::algo_engine::EngineEvent as E;
            let side_str = |s: traderview_core::algo_strategies::Side| match s {
                traderview_core::algo_strategies::Side::Buy => "buy",
                traderview_core::algo_strategies::Side::Sell => "sell",
            };
            let evt = match ev {
                E::SignalFired {
                    strategy_id,
                    run_id,
                    symbol,
                    side,
                    entry_price,
                    kind,
                } => crate::realtime::Event::AlgoSignalFired {
                    strategy_id: strategy_id.to_string(),
                    run_id: run_id.to_string(),
                    symbol,
                    side: side_str(side),
                    entry_price: entry_price.to_f64().unwrap_or(0.0),
                    kind,
                },
                E::OrderSubmitted {
                    strategy_id,
                    order_id,
                    symbol,
                    side,
                    qty,
                    broker_order_id,
                } => crate::realtime::Event::AlgoOrderSubmitted {
                    strategy_id: strategy_id.to_string(),
                    order_id: order_id.to_string(),
                    symbol,
                    side: side_str(side).into(),
                    qty: qty.to_f64().unwrap_or(0.0),
                    broker_order_id,
                },
                E::FillReceived {
                    strategy_id,
                    order_id,
                    symbol,
                    qty,
                    price,
                } => crate::realtime::Event::AlgoFillReceived {
                    strategy_id: strategy_id.to_string(),
                    order_id: order_id.to_string(),
                    symbol,
                    qty: qty.to_f64().unwrap_or(0.0),
                    price: price.to_f64().unwrap_or(0.0),
                },
            };
            hub.publish(evt);
        })
    }

    pub fn receipts_dir(&self) -> PathBuf {
        self.data_dir.join("receipts")
    }

    pub fn ocr_model_dir(&self) -> PathBuf {
        self.data_dir.join("models").join("paddleocr")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AppMode is small but its identity is load-bearing ─────────────────

    #[test]
    fn app_mode_variants_are_distinct() {
        // The auth middleware switches on this enum — Web demands credentials,
        // Desktop auto-issues. Conflating them is a security regression.
        assert_ne!(AppMode::Web, AppMode::Desktop);
        assert_eq!(AppMode::Web, AppMode::Web);
        assert_eq!(AppMode::Desktop, AppMode::Desktop);
    }

    #[test]
    fn app_mode_is_copy_so_propagation_is_cheap() {
        // Copy is part of the contract — handlers read `state.mode` directly
        // without needing to clone. Removing Copy would silently regress hot
        // request paths to allocate (or fail to compile in many places).
        fn assert_copy<T: Copy>() {}
        assert_copy::<AppMode>();
    }

    // ── data_dir-derived paths ────────────────────────────────────────────

    #[tokio::test]
    async fn receipts_dir_appends_receipts_segment() {
        let st = build_dummy_state(PathBuf::from("/tmp/tv"));
        assert_eq!(st.receipts_dir(), PathBuf::from("/tmp/tv/receipts"));
    }

    #[tokio::test]
    async fn ocr_model_dir_lands_under_models_paddleocr() {
        // receipt_routes.rs + the PaddleOCR loader both reach into the exact
        // path "{data_dir}/models/paddleocr". Renaming either segment moves
        // the on-disk model files and breaks first-run OCR until reinstall.
        let st = build_dummy_state(PathBuf::from("/var/lib/tv"));
        assert_eq!(
            st.ocr_model_dir(),
            PathBuf::from("/var/lib/tv/models/paddleocr")
        );
    }

    #[tokio::test]
    async fn paths_handle_nested_data_dir() {
        let st = build_dummy_state(PathBuf::from("/Users/alice/Library/App/traderview"));
        assert!(st
            .receipts_dir()
            .to_string_lossy()
            .ends_with("/traderview/receipts"));
        assert!(st
            .ocr_model_dir()
            .to_string_lossy()
            .ends_with("/traderview/models/paddleocr"));
    }

    #[tokio::test]
    async fn paths_are_distinct_under_same_data_dir() {
        // Two separate subdirectories — should never collide regardless of
        // data_dir shape.
        let st = build_dummy_state(PathBuf::from("/tmp/x"));
        assert_ne!(st.receipts_dir(), st.ocr_model_dir());
    }

    // ── Build a state without touching a real PgPool (which requires a
    //    running Postgres). connect_lazy needs a tokio context, hence
    //    #[tokio::test] on the call sites.

    fn build_dummy_state(data_dir: PathBuf) -> AppState {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/none")
            .expect("lazy connect cannot fail");
        AppState::new(pool, AppMode::Desktop, vec![0u8; 32], data_dir)
    }
}
