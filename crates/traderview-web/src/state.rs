use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;

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
}

impl AppState {
    pub fn new(pool: PgPool, mode: AppMode, jwt_secret: Vec<u8>, data_dir: PathBuf) -> Self {
        Self {
            pool,
            mode,
            jwt_secret: Arc::new(jwt_secret),
            data_dir: Arc::new(data_dir),
            hub: Hub::new(),
        }
    }

    pub fn receipts_dir(&self) -> PathBuf {
        self.data_dir.join("receipts")
    }

    pub fn ocr_model_dir(&self) -> PathBuf {
        self.data_dir.join("models").join("paddleocr")
    }
}
