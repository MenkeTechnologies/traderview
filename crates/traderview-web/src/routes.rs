//! Route module aggregator. Each resource is in its own file under
//! `routes/`; this file just composes them into one `Router`.

use crate::state::AppState;
use axum::Router;

mod accounts;
mod accounts_overview;
mod after_hours;
mod alerts;
mod algo;
mod analysis;
mod annuity_pv_fv;
mod api_tokens;
mod auth;
mod auto_loan;
mod backtest;
mod backtest_presets;
mod barista_fire;
mod bond_tent;
mod breadth;
mod breadth_divergence;
mod calc;
mod cape_indicator;
mod carryover;
mod catalyst_correlations;
mod catalysts;
mod cd_ladder;
mod chart_drawings;
mod chart_indicators;
mod charts;
mod client_errors;
mod car_tco;
mod coast_fire;
mod college_529;
mod comments;
mod community;
mod compare;
mod confluence;
mod confluence_autotrade;
mod corr_matrix;
mod credit_utilization;
mod crypto;
mod csv_wizard;
mod forex;
mod custom_indicators;
mod darkpool;
mod dashboards;
mod data_sources;
mod dca_simulator;
mod debt_avalanche;
mod debt_snowball;
mod discipline;
mod disclosures;
mod dividend_aristocrats;
mod dividend_capture;
mod dividend_tracker;
mod drawdown_cutoff;
mod earnings_cal;
mod earnings_iv;
mod earnings_revisions;
mod economy;
mod emergency_fund;
mod envelope_budget;
mod equity_forecast;
mod ev_vs_ice;
mod executions;
mod export;
mod extras;
mod fafsa_efc;
mod fat_fire;
mod fear_greed;
mod fifty_thirty_twenty;
mod fill_quality;
mod financial_ratios;
mod finnhub_extras;
mod fire_calculator;
mod gamma_squeeze;
mod glide_path;
mod goals;
mod halts;
mod heatmap;
mod heloc;
mod home_maintenance;
mod hotkeys;
mod htb_ranker;
mod hysa_compare;
mod i_bond;
mod imports;
mod insider_clusters;
mod insider_stream;
mod institutional;
mod ipo_lockups;
mod ira_basis;
mod iv_term;
mod journal;
mod journal_ai;
mod lean_fire;
mod lease_vs_buy_car;
mod live_dashboard;
mod live_positions;
mod live_ticks;
mod magic_formula;
mod market_gamma_regime;
mod markets;
mod mentorships;
mod microstructure;
mod mood_analytics;
mod mortgage_amortization;
mod mortgage_refinance;
mod multi_broker;
mod net_worth_tracker;
mod news;
mod note_templates;
mod options;
mod pairs;
mod paper;
mod paper_rebalance;
mod pead;
mod pension_lump_vs_annuity;
mod permanent_portfolio;
mod personal_balance_sheet;
mod personal_cash_flow;
mod plans;
mod portfolio_exposure;
mod position_size;
mod pslf_tracker;
mod premarket;
mod r_distribution;
mod rebalance;
mod rent_vs_buy;
mod rmd_calculator;
mod roth_vs_trad_401k;
mod reports;
mod research;
mod rvol_accel;
mod savings_rate;
mod scanner_backtest;
mod scans;
mod screener;
mod screenshots;
mod search;
mod sector_rotation;
mod sector_rotation_strategy;
mod sector_timing;
mod sectors;
mod sentiment;
mod sentiment_velocity;
mod settings;
mod shares;
mod short_interest;
mod sinking_fund;
mod social_security_age;
mod fundamentals_extra;
mod sp500_predict;
mod squeeze_scanner;
mod stock_recommendation;
// stock_recommendation::router() also serves
// /recommendations/golden-stars + /recommendations/sectors +
// watchers CRUD; no separate module needed.
mod strategy_alerts;
mod strategy_calculators;
mod student_loan_payoff;
mod symbols_catalog;
mod tags;
mod tape_replay;
mod thirteen_f;
mod tax_bracket_optimizer;
mod tax_loss_harvest;
mod tax_lots;
mod three_fund_portfolio;
mod tips_bond;
mod trade_analytics;
mod trade_compare;
mod trade_reviews;
mod trades;
mod uoa_stream;
mod vol;
mod vol_surface;
mod vrp;
mod watchlists;
mod webhooks;
mod webull;
mod ws;
mod zero_based_budget;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/expense", crate::expense_routes::router())
        .nest("/tax", crate::tax_routes::router())
        .nest("/tax-filing", crate::tax_filing_routes::router())
        .nest("/budget", crate::budget_routes::router())
        .nest("/businesses", crate::business_routes::router())
        .nest("/brokers", crate::broker_routes::router())
        .nest("/rental", crate::rental_routes::router())
        .nest("/risk-gate", crate::risk_gate_routes::router())
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
        .merge(chart_indicators::router())
        .merge(calc::router())
        .merge(cape_indicator::router())
        .merge(carryover::router())
        .merge(ira_basis::router())
        .merge(trade_analytics::router())
        .merge(microstructure::router())
        .merge(extras::router())
        .merge(settings::router())
        .merge(plans::router())
        .merge(search::router())
        .merge(symbols_catalog::router())
        .merge(note_templates::router())
        .merge(markets::router())
        .merge(watchlists::router())
        .merge(research::router())
        .merge(screener::router())
        .merge(scans::router())
        .merge(sectors::router())
        .merge(paper::router())
        .merge(paper_rebalance::router())
        .merge(permanent_portfolio::router())
        .merge(alerts::router())
        .merge(hotkeys::router())
        .merge(earnings_iv::router())
        .merge(disclosures::router())
        .merge(institutional::router())
        .merge(sentiment::router())
        .merge(heatmap::router())
        .merge(options::router())
        .merge(crypto::router())
        .merge(forex::router())
        .merge(backtest::router())
        .merge(economy::router())
        .merge(analysis::router())
        .merge(short_interest::router())
        .merge(darkpool::router())
        .merge(vol::router())
        .merge(webhooks::router())
        .merge(breadth::router())
        .merge(fear_greed::router())
        .merge(fire_calculator::router())
        .merge(emergency_fund::router())
        .merge(net_worth_tracker::router())
        .merge(personal_balance_sheet::router())
        .merge(personal_cash_flow::router())
        .merge(financial_ratios::router())
        .merge(savings_rate::router())
        .merge(sinking_fund::router())
        .merge(zero_based_budget::router())
        .merge(fifty_thirty_twenty::router())
        .merge(envelope_budget::router())
        .merge(debt_avalanche::router())
        .merge(debt_snowball::router())
        .merge(credit_utilization::router())
        .merge(auto_loan::router())
        .merge(mortgage_amortization::router())
        .merge(mortgage_refinance::router())
        .merge(rent_vs_buy::router())
        .merge(heloc::router())
        .merge(home_maintenance::router())
        .merge(student_loan_payoff::router())
        .merge(pslf_tracker::router())
        .merge(college_529::router())
        .merge(fafsa_efc::router())
        .merge(car_tco::router())
        .merge(lease_vs_buy_car::router())
        .merge(ev_vs_ice::router())
        .merge(coast_fire::router())
        .merge(barista_fire::router())
        .merge(lean_fire::router())
        .merge(fat_fire::router())
        .merge(rmd_calculator::router())
        .merge(social_security_age::router())
        .merge(roth_vs_trad_401k::router())
        .merge(pension_lump_vs_annuity::router())
        .merge(three_fund_portfolio::router())
        .merge(bond_tent::router())
        .merge(glide_path::router())
        .merge(annuity_pv_fv::router())
        .merge(cd_ladder::router())
        .merge(i_bond::router())
        .merge(tips_bond::router())
        .merge(hysa_compare::router())
        .merge(tax_bracket_optimizer::router())
        .merge(premarket::router())
        .merge(after_hours::router())
        .merge(halts::router())
        .merge(client_errors::router())
        .merge(catalysts::router())
        .merge(catalyst_correlations::router())
        .merge(uoa_stream::router())
        .merge(gamma_squeeze::router())
        .merge(htb_ranker::router())
        .merge(breadth_divergence::router())
        .merge(rvol_accel::router())
        .merge(insider_stream::router())
        .merge(insider_clusters::router())
        .merge(earnings_revisions::router())
        .merge(sector_timing::router())
        .merge(market_gamma_regime::router())
        .merge(scanner_backtest::router())
        .merge(pead::router())
        .merge(portfolio_exposure::router())
        .merge(magic_formula::router())
        .merge(sentiment_velocity::router())
        .merge(confluence::router())
        .merge(confluence_autotrade::router())
        .merge(vrp::router())
        .merge(pairs::router())
        .merge(ipo_lockups::router())
        .merge(iv_term::router())
        .merge(sp500_predict::router())
        .merge(dca_simulator::router())
        .merge(dividend_aristocrats::router())
        .merge(dividend_capture::router())
        .merge(dividend_tracker::router())
        .merge(drawdown_cutoff::router())
        .merge(multi_broker::router())
        .merge(webull::router())
        .merge(vol_surface::router())
        .merge(tax_lots::router())
        .merge(tax_loss_harvest::router())
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
        .merge(live_dashboard::router())
        .merge(live_positions::router())
        .merge(live_ticks::router())
        .merge(corr_matrix::router())
        .merge(strategy_alerts::router())
        .merge(strategy_calculators::router())
        .merge(algo::router())
        .merge(rebalance::router())
        .merge(sector_rotation::router())
        .merge(sector_rotation_strategy::router())
        .merge(tape_replay::router())
        .merge(thirteen_f::router())
        .merge(backtest_presets::router())
        .merge(mood_analytics::router())
        .merge(discipline::router())
        .merge(goals::router())
        .merge(r_distribution::router())
        .merge(trade_reviews::router())
        .merge(equity_forecast::router())
        .merge(fill_quality::router())
        .merge(custom_indicators::router())
        .merge(trade_compare::router())
        .merge(csv_wizard::router())
        .merge(accounts_overview::router())
        .merge(data_sources::router())
        .merge(squeeze_scanner::router())
        .merge(stock_recommendation::router())
        .merge(fundamentals_extra::router())
        .merge(finnhub_extras::router())
}

#[cfg(test)]
mod router_smoke {
    /// Constructs the full api_router so axum's panicking duplicate-route
    /// check fires at test time instead of at server boot. Any future
    /// `.route("/x", ...)` collision will surface here as a test failure.
    #[test]
    fn api_router_builds_without_duplicate_routes() {
        let _ = super::api_router();
    }
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
        let row: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(&s.pool)
            .await?;
        match row {
            Some((owner,)) if owner == user_id => Ok(()),
            Some(_) => Err(ApiError::Forbidden),
            None => Err(ApiError::NotFound),
        }
    }

    pub async fn ensure_broker_owner(
        s: &AppState,
        user_id: Uuid,
        broker_id: Uuid,
    ) -> Result<(), ApiError> {
        let row: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM brokers WHERE id = $1")
            .bind(broker_id)
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
