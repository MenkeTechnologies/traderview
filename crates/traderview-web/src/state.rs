use sqlx::PgPool;
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
    pub hub: Hub,
}

impl AppState {
    pub fn new(pool: PgPool, mode: AppMode, jwt_secret: Vec<u8>) -> Self {
        Self {
            pool,
            mode,
            jwt_secret: Arc::new(jwt_secret),
            hub: Hub::new(),
        }
    }
}
