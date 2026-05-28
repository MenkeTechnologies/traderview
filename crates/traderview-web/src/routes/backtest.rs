use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::Deserialize;
use traderview_core::backtest::{
    run, walk_forward, BtResult, OptMetric, Preset, PresetKind, WfResult,
};
use traderview_core::BarInterval;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/backtest/run", post(run_handler))
        .route("/backtest/walk-forward", post(walk_forward_handler))
}

#[derive(Deserialize)]
struct Body {
    symbol: String,
    preset: Preset,
    #[serde(default = "default_days")]
    days: i64,
    #[serde(default = "default_capital")]
    initial_capital: f64,
    #[serde(default)]
    fee_per_trade: f64,
}
fn default_days() -> i64 {
    730
}
fn default_capital() -> f64 {
    10_000.0
}

async fn run_handler(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<Body>,
) -> Result<Json<BtResult>, ApiError> {
    let to = Utc::now();
    let from = to - Duration::days(b.days);
    let bars = traderview_db::prices::get_bars(
        &s.pool,
        &b.symbol.to_uppercase(),
        BarInterval::D1,
        from,
        to,
    )
    .await
    .map_err(ApiError::Internal)?;
    if bars.is_empty() {
        return Err(ApiError::BadRequest(format!("no bars for {}", b.symbol)));
    }
    Ok(Json(run(
        &bars,
        b.preset,
        b.initial_capital,
        b.fee_per_trade,
    )))
}

#[derive(Deserialize)]
struct WfBody {
    symbol: String,
    kind: PresetKind,
    #[serde(default = "default_wf_days")]
    days: i64,
    #[serde(default = "default_is_bars")]
    is_bars: usize,
    #[serde(default = "default_oos_bars")]
    oos_bars: usize,
    #[serde(default)]
    step_bars: Option<usize>,
    #[serde(default = "default_capital")]
    initial_capital: f64,
    #[serde(default)]
    fee_per_trade: f64,
    #[serde(default = "default_metric")]
    metric: OptMetric,
}
fn default_wf_days() -> i64 {
    1825
} // 5y
fn default_is_bars() -> usize {
    252
} // ~1y
fn default_oos_bars() -> usize {
    63
} // ~1q
fn default_metric() -> OptMetric {
    OptMetric::Return
}

async fn walk_forward_handler(
    State(s): State<AppState>,
    _u: AuthUser,
    Json(b): Json<WfBody>,
) -> Result<Json<WfResult>, ApiError> {
    let to = Utc::now();
    let from = to - Duration::days(b.days);
    let bars = traderview_db::prices::get_bars(
        &s.pool,
        &b.symbol.to_uppercase(),
        BarInterval::D1,
        from,
        to,
    )
    .await
    .map_err(ApiError::Internal)?;
    if bars.len() < b.is_bars + b.oos_bars {
        return Err(ApiError::BadRequest(format!(
            "need at least {} bars for {} (got {}); pass a larger 'days' or shrink the windows",
            b.is_bars + b.oos_bars,
            b.symbol,
            bars.len()
        )));
    }
    let step = b.step_bars.unwrap_or(b.oos_bars);
    let r = walk_forward(
        &bars,
        traderview_core::backtest::WfConfig {
            kind: b.kind,
            is_bars: b.is_bars,
            oos_bars: b.oos_bars,
            step,
            initial_capital: b.initial_capital,
            fee_per_trade: b.fee_per_trade,
            metric: b.metric,
        },
    );
    Ok(Json(r))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Single-backtest defaults ──────────────────────────────────────────

    #[test]
    fn default_days_matches_2_year_lookback() {
        // 730d gives roughly two years of trading bars — enough warm-up for
        // every Preset's longest indicator (sma200, breakout-channel windows).
        // Shorter defaults silently produce empty equity curves for new tickers.
        assert_eq!(default_days(), 730);
    }

    #[test]
    fn default_capital_matches_10k_starting_equity() {
        // The frontend renders all R-multiples / drawdown % against this base
        // when no capital is sent. Changing the number reshapes every chart.
        assert_eq!(default_capital(), 10_000.0);
    }

    // ── Walk-forward defaults ─────────────────────────────────────────────

    #[test]
    fn default_wf_days_matches_5_year_window() {
        // Walk-forward needs at least is_bars + oos_bars + step bars; 1825d
        // (~5y) is the smallest window that lets the default 252/63 split run
        // multiple oos folds without padding.
        assert_eq!(default_wf_days(), 1825);
    }

    #[test]
    fn default_is_bars_matches_roughly_one_trading_year() {
        // 252 = approximate US trading days per year. Anchoring to 252 keeps
        // is/oos ratios sane across symbols regardless of holiday count.
        assert_eq!(default_is_bars(), 252);
    }

    #[test]
    fn default_oos_bars_matches_roughly_one_quarter() {
        // ~63 trading days per quarter — picked so the WF gets ~4 oos folds
        // per is window.
        assert_eq!(default_oos_bars(), 63);
    }

    #[test]
    fn default_metric_is_return_not_sharpe() {
        // Return is the default optimization metric so first-time users see
        // the most intuitive number. Sharpe/Sortino are explicit opt-ins.
        assert!(matches!(default_metric(), OptMetric::Return));
    }

    #[test]
    fn defaults_combine_to_a_runnable_wf_window() {
        // is + oos must fit inside default_wf_days × ~252/365 trading days.
        // If anyone shrinks days OR grows is/oos, this guard catches it.
        let trading_days = (default_wf_days() as f64 * 252.0 / 365.0) as usize;
        assert!(
            default_is_bars() + default_oos_bars() <= trading_days,
            "is_bars ({}) + oos_bars ({}) won't fit in {} trading days",
            default_is_bars(),
            default_oos_bars(),
            trading_days
        );
    }

    // ── Body deserialization picks up defaults when fields omitted ────────

    #[test]
    fn body_fills_defaults_when_only_required_fields_sent() {
        // Preset is serde-renamed to snake_case; preset shape matches what the
        // frontend sends so the defaults wiring is exercised end-to-end.
        let json = r#"{"symbol":"AAPL","preset":{"sma_cross":{"fast":10,"slow":30}}}"#;
        let b: Body = serde_json::from_str(json).expect("parse");
        assert_eq!(b.days, 730);
        assert_eq!(b.initial_capital, 10_000.0);
        assert_eq!(b.fee_per_trade, 0.0);
    }
}
