use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::realtime::Hub;

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
}

impl AppState {
    pub fn new(pool: PgPool, mode: AppMode, jwt_secret: Vec<u8>, data_dir: PathBuf) -> Self {
        let permits = std::thread::available_parallelism()
            .map(|n| n.get().min(4).max(1))
            .unwrap_or(2);
        Self {
            pool,
            mode,
            jwt_secret: Arc::new(jwt_secret),
            data_dir: Arc::new(data_dir),
            hub: Hub::new(),
            ocr_sem: Arc::new(Semaphore::new(permits)),
        }
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
