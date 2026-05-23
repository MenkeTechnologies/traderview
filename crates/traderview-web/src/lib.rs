//! traderview-web — axum router + JWT auth.
//!
//! The same router is mounted by:
//!   * `src/bin/server.rs`  → standalone web binary (multi-user)
//!   * `src-tauri`          → desktop, talking to embedded Postgres
//!
//! The desktop build injects an `AppState` whose `mode == AppMode::Desktop`,
//! which causes the auth middleware to auto-mint a token for the single
//! `is_local = true` user instead of demanding credentials.

pub mod auth;
pub mod error;
pub mod routes;
pub mod state;

pub use error::ApiError;
pub use state::{AppMode, AppState};

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api", routes::api_router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
