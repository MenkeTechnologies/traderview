//! Scanner backtest route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use traderview_db::scanner_backtest;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/scanner-backtest/pead", get(pead))
        .route("/scanner-backtest/insider-clusters", get(insider_clusters))
        .route("/scanner-backtest/all", get(all_scanners))
}

#[derive(Deserialize)]
struct DaysQ {
    #[serde(default = "default_days")]
    days: i64,
    #[serde(default)]
    friction: bool,
}
fn default_days() -> i64 {
    365
}

async fn pead(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<DaysQ>,
) -> Result<Json<scanner_backtest::BacktestResult>, ApiError> {
    Ok(Json(
        scanner_backtest::backtest_pead(&s.pool, q.days).await?,
    ))
}

async fn insider_clusters(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<DaysQ>,
) -> Result<Json<scanner_backtest::BacktestResult>, ApiError> {
    Ok(Json(
        scanner_backtest::backtest_insider_clusters(&s.pool, q.days).await?,
    ))
}

/// Multi-scanner diagnostic: runs every wired adapter at the requested
/// lookback and returns the comparable Sharpe table sorted by 20d
/// annualised Sharpe descending. Adapters that fail (no data, query
/// error) come back with samples_used=0 and an `error` field so the UI
/// can render the gap explicitly.
#[derive(Serialize)]
struct AllScannersResult {
    days: i64,
    scanners: Vec<ScannerRow>,
}

#[derive(Serialize)]
struct ScannerRow {
    scanner: String,
    samples_used: usize,
    horizons: Vec<scanner_backtest::HorizonStats>,
    error: Option<String>,
}

async fn all_scanners(
    State(s): State<AppState>,
    _user: AuthUser,
    Query(q): Query<DaysQ>,
) -> Result<Json<AllScannersResult>, ApiError> {
    let friction_cfg = if q.friction {
        Some(traderview_db::friction::FrictionConfig::baseline_equity())
    } else {
        None
    };
    let apply = |mut r: scanner_backtest::BacktestResult| -> scanner_backtest::BacktestResult {
        if let Some(cfg) = friction_cfg {
            let cost = cfg.round_trip_pct();
            for h in r.horizons.iter_mut() {
                if h.n == 0 {
                    continue;
                }
                h.mean_return_pct -= cost;
                h.median_return_pct -= cost;
                h.total_logret_signed -= cost * h.n as f64 / 100.0;
                if h.stdev_pct > 0.0 && h.horizon_days > 0 {
                    h.annualised_sharpe =
                        h.mean_return_pct / h.stdev_pct * (252.0 / h.horizon_days as f64).sqrt();
                }
            }
        }
        r
    };
    let mut rows: Vec<ScannerRow> = Vec::new();
    for (name, fut) in [
        (
            "pead",
            scanner_backtest::backtest_pead(&s.pool, q.days).await,
        ),
        (
            "insider_clusters",
            scanner_backtest::backtest_insider_clusters(&s.pool, q.days).await,
        ),
    ] {
        match fut.map(apply) {
            Ok(r) => rows.push(ScannerRow {
                scanner: r.scanner,
                samples_used: r.samples_used,
                horizons: r.horizons,
                error: None,
            }),
            Err(e) => rows.push(ScannerRow {
                scanner: name.into(),
                samples_used: 0,
                horizons: Vec::new(),
                error: Some(format!("{e}")),
            }),
        }
    }
    // Sort by 20d Sharpe desc — the horizon Kelly defaults to. Scanners
    // missing the 20d row sink to the bottom.
    rows.sort_by(|a, b| {
        let sa = a
            .horizons
            .iter()
            .find(|h| h.horizon_days == 20)
            .map(|h| h.annualised_sharpe)
            .unwrap_or(f64::NEG_INFINITY);
        let sb = b
            .horizons
            .iter()
            .find(|h| h.horizon_days == 20)
            .map(|h| h.annualised_sharpe)
            .unwrap_or(f64::NEG_INFINITY);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(Json(AllScannersResult {
        days: q.days,
        scanners: rows,
    }))
}
