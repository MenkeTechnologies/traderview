//! Route module aggregator. Each resource is in its own file under
//! `routes/`; this file just composes them into one `Router`.

use crate::state::AppState;
use axum::Router;

mod accounts;
mod alerts;
mod analysis;
mod api_tokens;
mod auth;
mod backtest;
mod breadth;
mod chart_drawings;
mod charts;
mod comments;
mod community;
mod compare;
mod crypto;
mod dashboards;
mod darkpool;
mod disclosures;
mod earnings_cal;
mod earnings_iv;
mod economy;
mod executions;
mod export;
mod fear_greed;
mod heatmap;
mod hotkeys;
mod imports;
mod journal;
mod journal_ai;
mod markets;
mod mentorships;
mod news;
mod note_templates;
mod options;
mod paper;
mod plans;
mod position_size;
mod premarket;
mod reports;
mod research;
mod scans;
mod screener;
mod screenshots;
mod search;
mod sectors;
mod sentiment;
mod settings;
mod shares;
mod short_interest;
mod tags;
mod tax_lots;
mod trades;
mod vol;
mod vol_surface;
mod watchlists;
mod webhooks;
mod ws;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(auth::router())
        .merge(accounts::router())
        .merge(trades::router())
        .merge(executions::router())
        .merge(tags::router())
        .merge(journal::router())
        .merge(screenshots::router())
        .merge(imports::router())
        .merge(reports::router())
        .merge(mentorships::router())
        .merge(shares::router())
        .merge(comments::router())
        .merge(community::router())
        .merge(charts::router())
        .merge(settings::router())
        .merge(plans::router())
        .merge(search::router())
        .merge(note_templates::router())
        .merge(markets::router())
        .merge(watchlists::router())
        .merge(research::router())
        .merge(screener::router())
        .merge(scans::router())
        .merge(sectors::router())
        .merge(paper::router())
        .merge(alerts::router())
        .merge(hotkeys::router())
        .merge(earnings_iv::router())
        .merge(disclosures::router())
        .merge(sentiment::router())
        .merge(heatmap::router())
        .merge(options::router())
        .merge(crypto::router())
        .merge(backtest::router())
        .merge(economy::router())
        .merge(analysis::router())
        .merge(short_interest::router())
        .merge(darkpool::router())
        .merge(vol::router())
        .merge(webhooks::router())
        .merge(breadth::router())
        .merge(fear_greed::router())
        .merge(premarket::router())
        .merge(vol_surface::router())
        .merge(tax_lots::router())
        .merge(compare::router())
        .merge(export::router())
        .merge(chart_drawings::router())
        .merge(ws::router())
        .merge(journal_ai::router())
        .merge(api_tokens::router())
        .merge(dashboards::router())
        .merge(news::router())
        .merge(earnings_cal::router())
        .merge(position_size::router())
}

mod helpers {
    use crate::error::ApiError;
    use crate::state::AppState;
    use sqlx::PgPool;
    use uuid::Uuid;

    pub async fn ensure_account_owner(
        s: &AppState,
        user_id: Uuid,
        account_id: Uuid,
    ) -> Result<(), ApiError> {
        let row: Option<(Uuid,)> =
            sqlx::query_as("SELECT user_id FROM accounts WHERE id = $1")
                .bind(account_id)
                .fetch_optional(&s.pool)
                .await?;
        match row {
            Some((owner,)) if owner == user_id => Ok(()),
            Some(_) => Err(ApiError::Forbidden),
            None => Err(ApiError::NotFound),
        }
    }

    pub async fn ensure_trade_owner(
        pool: &PgPool,
        user_id: Uuid,
        trade_id: Uuid,
    ) -> Result<(), ApiError> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT a.user_id FROM trades t JOIN accounts a ON a.id = t.account_id
              WHERE t.id = $1",
        )
        .bind(trade_id)
        .fetch_optional(pool)
        .await?;
        match row {
            Some((owner,)) if owner == user_id => Ok(()),
            Some(_) => Err(ApiError::Forbidden),
            None => Err(ApiError::NotFound),
        }
    }
}
