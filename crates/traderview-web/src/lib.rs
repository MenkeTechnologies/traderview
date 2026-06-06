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
pub mod budget_routes;
pub mod error;
pub mod expense_routes;
pub mod log_mw;
pub mod merchant;
pub mod rate_limit;
pub mod realtime;
pub mod receipt_routes;
pub mod rental_routes;
pub mod risk_gate_routes;
pub mod routes;
pub mod state;
pub mod tax_filing_routes;
pub mod tax_pdf;
pub mod tax_routes;

pub use error::ApiError;
pub use state::{AppMode, AppState};

use axum::Router;
use tower_http::cors::CorsLayer;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api", routes::api_router())
        .with_state(state)
        // Custom middleware logs every request + body-sniffs 4xx/5xx so the
        // log file tells us WHY a widget broke, not just "request failed".
        .layer(axum::middleware::from_fn(log_mw::request_response_logger))
        .layer(CorsLayer::permissive())
}
