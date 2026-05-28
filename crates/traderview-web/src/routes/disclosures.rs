use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_db::disclosures::{Disclosure, PollResult, Watcher};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/disclosures", get(list))
        .route("/disclosures/poll", post(poll_now))
        .route(
            "/disclosures/watchers",
            get(list_watchers).post(create_watcher),
        )
        .route("/disclosures/watchers/:id", delete(delete_watcher))
}

#[derive(Deserialize)]
struct ListQ {
    kind: Option<String>,
    symbol: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    200
}

async fn list(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ListQ>,
) -> Result<Json<Vec<Disclosure>>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::list(&s.pool, q.kind.as_deref(), q.symbol.as_deref(), q.limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn poll_now(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<PollResult>, ApiError> {
    Ok(Json(traderview_db::disclosures::poll_all(&s.pool).await))
}

async fn list_watchers(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Watcher>>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::list_watchers(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
    #[serde(default = "default_kinds")]
    kinds: Vec<String>,
    symbols: Option<Vec<String>>,
    filers: Option<Vec<String>>,
    min_amount_usd: Option<Decimal>,
    #[serde(default = "default_sound")]
    sound: String,
}
fn default_kinds() -> Vec<String> {
    vec![
        "insider_form4".into(),
        "senate_stock".into(),
        "house_stock".into(),
    ]
}
fn default_sound() -> String {
    "bell".into()
}

async fn create_watcher(
    State(s): State<AppState>,
    user: AuthUser,
    Json(b): Json<CreateBody>,
) -> Result<Json<Watcher>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::create_watcher(
            &s.pool,
            traderview_db::disclosures::NewWatcher {
                user_id: user.id,
                name: &b.name,
                kinds: &b.kinds,
                symbols: b.symbols.as_deref(),
                filers: b.filers.as_deref(),
                min_amount_usd: b.min_amount_usd,
                sound: &b.sound,
            },
        )
        .await
        .map_err(ApiError::Internal)?,
    ))
}

async fn delete_watcher(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::disclosures::delete_watcher(&s.pool, user.id, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Scalar defaults ───────────────────────────────────────────────────

    #[test]
    fn default_limit_matches_200_disclosure_list_cap() {
        // /disclosures defaults to the most-recent 200 items — anything
        // higher hits the frontend's pagination assumption and lower hides
        // SEC bursts that arrive in one minute.
        assert_eq!(default_limit(), 200);
    }

    #[test]
    fn default_sound_matches_bell() {
        // The frontend Web Audio API plays `sounds/{sound}.mp3`; "bell" is
        // the file that ships in the bundle. Renaming silently breaks the
        // notification chime on every watcher.
        assert_eq!(default_sound(), "bell");
    }

    // ── default_kinds: pinned watcher coverage set ────────────────────────

    #[test]
    fn default_kinds_covers_insider_and_political_disclosures() {
        // These three feeds are the entire MVP — insider Form 4, Senate
        // STOCK Act, House STOCK Act. Adding/removing here changes the
        // out-of-the-box detection scope for new users.
        let k = default_kinds();
        assert_eq!(k.len(), 3);
        assert!(k.contains(&"insider_form4".to_string()));
        assert!(k.contains(&"senate_stock".to_string()));
        assert!(k.contains(&"house_stock".to_string()));
    }

    #[test]
    fn default_kinds_uses_snake_case_kind_ids() {
        // The DB enum stores kinds in snake_case; mixing in camelCase
        // produces silent FK mismatches with no rows matched.
        for kind in default_kinds() {
            assert!(
                kind.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
                "kind {kind:?} should be snake_case"
            );
        }
    }

    #[test]
    fn default_kinds_returns_fresh_vec_each_call() {
        // The function rebuilds the Vec each call so callers can mutate the
        // returned value without affecting the next caller's defaults.
        let mut a = default_kinds();
        a.push("extra".into());
        let b = default_kinds();
        assert_eq!(b.len(), 3, "second call leaked first caller's mutation");
    }

    // ── Body serde fills defaults when fields omitted ─────────────────────

    #[test]
    fn create_body_uses_default_kinds_when_field_missing() {
        let json = r#"{"name":"all-insider-buys"}"#;
        let b: CreateBody = serde_json::from_str(json).expect("parse");
        assert_eq!(b.name, "all-insider-buys");
        assert_eq!(b.kinds, default_kinds());
        assert_eq!(b.sound, "bell");
        assert!(b.symbols.is_none());
        assert!(b.filers.is_none());
        assert!(b.min_amount_usd.is_none());
    }

    #[test]
    fn list_query_uses_default_limit_when_field_missing() {
        // The deserializer must hand back default_limit() not 0 — otherwise
        // /disclosures with no params returns an empty page.
        let json = "{}";
        let q: ListQ = serde_json::from_str(json).expect("parse");
        assert_eq!(q.limit, 200);
        assert!(q.kind.is_none());
        assert!(q.symbol.is_none());
    }
}
