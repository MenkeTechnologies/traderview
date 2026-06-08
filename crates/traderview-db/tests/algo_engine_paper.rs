//! Algo engine end-to-end with the in-memory broker sink. Verifies:
//!   - momentum signal on a synthetic uptrend reaches a sink call,
//!   - the persisted order mirrors the intent (symbol, side, qty, stops),
//!   - kill switch vetoes engine submission with the right error,
//!   - position cap vetoes when caller reports too many open positions,
//!   - paper-locked strategies refuse alpaca_live submission.
//!
//! Embedded Postgres lifecycle is shared with the main `integration.rs`
//! harness — we spin a separate pool here so the tests are isolated.

use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use once_cell::sync::Lazy;
use postgresql_embedded::{PostgreSQL, Settings};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use traderview_core::models::{BarInterval, PriceBar};
use traderview_db::algo::{self, AlgoStrategy, AlgoStrategyInput};
use traderview_db::algo_engine::{
    process_bar_window, BrokerSink, EngineError, InMemorySink, OrderIntent, SubmittedOrder,
};
use uuid::Uuid;

// ─── runtime + embedded pg ───────────────────────────────────────────────────
//
// Same Lazy pattern as the main integration.rs harness — PG initializes
// synchronously inside Lazy, so subsequent `RT.block_on(test_fut)` calls
// don't nest a block_on inside another block_on.

static RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime")
});

struct PgGuard {
    _pg: PostgreSQL,
    pool: PgPool,
}

static PG: Lazy<Mutex<PgGuard>> = Lazy::new(|| {
    let (installation_dir, data_dir) = test_dirs();
    std::fs::create_dir_all(&installation_dir).expect("mkdir installation_dir");
    std::fs::create_dir_all(&data_dir).expect("mkdir data_dir");
    let pw_path = installation_dir.join("test-password");
    let password = "traderview-test-pw".to_string();
    let _ = std::fs::write(&pw_path, password.as_bytes());
    let settings = Settings {
        installation_dir,
        data_dir,
        password_file: pw_path,
        password,
        temporary: false,
        ..Default::default()
    };
    let mut pg = PostgreSQL::new(settings);
    RT.block_on(async {
        pg.setup().await.expect("setup embedded pg");
        pg.start().await.expect("start embedded pg");
        if !pg.database_exists("traderview").await.expect("db exists") {
            pg.create_database("traderview").await.expect("create db");
        }
    });
    let database_url = pg.settings().url("traderview");
    let pool = RT
        .block_on(traderview_db::connect_external(&database_url))
        .expect("connect to test db");
    RT.block_on(traderview_db::migrate(&pool))
        .expect("run migrations");
    Mutex::new(PgGuard { _pg: pg, pool })
});

fn test_dirs() -> (PathBuf, PathBuf) {
    let cache_root = dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("traderview-tests");
    let installation_dir = cache_root.join("pg-bin");
    let mut data_dir = std::env::temp_dir();
    data_dir.push(format!(
        "traderview-db-algoengine-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    (installation_dir, data_dir)
}

fn pool() -> PgPool {
    PG.lock().expect("pg lock").pool.clone()
}

fn run<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let _ = Lazy::force(&PG);
    RT.block_on(fut)
}

async fn fresh_user_in(pool: &PgPool) -> Uuid {
    let email = format!("engine-{}@example.test", Uuid::new_v4());
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, is_local) VALUES ($1, $2, false) RETURNING id",
    )
    .bind(&email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$dummysalt$dummyhash")
    .fetch_one(pool)
    .await
    .expect("user");
    id
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn bar(t: i64, sym: &str, o: &str, h: &str, l: &str, c: &str, v: u64) -> PriceBar {
    PriceBar {
        symbol: sym.into(),
        interval: BarInterval::M1,
        bar_time: Utc.timestamp_opt(t, 0).unwrap(),
        open: Decimal::from_str(o).unwrap(),
        high: Decimal::from_str(h).unwrap(),
        low: Decimal::from_str(l).unwrap(),
        close: Decimal::from_str(c).unwrap(),
        volume: Decimal::from(v),
        source: "test".into(),
    }
}

/// Build a window that triggers a long entry on the latest bar — the
/// engine walks this verbatim, no expanding-window trick.
fn fresh_long_window(symbol: &str) -> Vec<PriceBar> {
    let mut bars = Vec::new();
    let mut t = 1_700_000_000_i64;
    for _ in 0..35 {
        bars.push(bar(t, symbol, "100.00", "100.10", "99.90", "100.00", 1_000_000));
        t += 60;
    }
    for i in 0..8 {
        let p = 100.0 - (i as f64 + 1.0) * 0.4;
        bars.push(bar(
            t, symbol,
            &format!("{:.2}", p + 0.1),
            &format!("{:.2}", p + 0.2),
            &format!("{:.2}", p - 0.2),
            &format!("{p:.2}"),
            1_000_000,
        ));
        t += 60;
    }
    for i in 0..12 {
        let p = 96.6 + (i as f64 + 1.0) * 0.95;
        let vol = if i == 11 { 4_000_000 } else { 2_000_000 };
        bars.push(bar(
            t, symbol,
            &format!("{:.2}", p - 0.3),
            &format!("{:.2}", p + 0.4),
            &format!("{:.2}", p - 0.4),
            &format!("{p:.2}"),
            vol,
        ));
        t += 60;
    }
    bars
}

/// Walk the window forward, calling the engine on each prefix, returning
/// the first prefix length that produced a submitted order id.
async fn drive_engine_until_signal(
    pool: &PgPool,
    sink: &InMemorySink,
    strategy: &AlgoStrategy,
    run_id: Uuid,
    bars: &[PriceBar],
    equity: f64,
) -> Option<usize> {
    for end in 30..=bars.len() {
        if let Ok(Some(_)) =
            process_bar_window(pool, sink, strategy, run_id, &bars[..end], equity, 0).await
        {
            return Some(end);
        }
    }
    None
}

async fn fresh_account_in(pool: &PgPool, user_id: Uuid) -> Uuid {
    let name = format!("acct-{}", Uuid::new_v4());
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO accounts (user_id, name, broker) VALUES ($1, $2, 'alpaca') RETURNING id",
    )
    .bind(user_id)
    .bind(&name)
    .fetch_one(pool)
    .await
    .expect("insert account");
    id
}

async fn make_strategy(pool: &PgPool, user_id: Uuid, broker_mode: &str) -> AlgoStrategy {
    let account_id = fresh_account_in(pool, user_id).await;
    algo::create_strategy(
        pool,
        user_id,
        AlgoStrategyInput {
            name: format!("engine-test-{}", Uuid::new_v4()),
            enabled: true,
            timeframe: "min1".into(),
            universe_mode: "watchlist".into(),
            watchlist_id: None,
            autoscan_top_n: 25,
            side_mode: "long".into(),
            strategy_type: "momentum".into(),
            account_id: Some(account_id),
            entry_rules: serde_json::json!({}),
            exit_rules: serde_json::json!({}),
            sizing: serde_json::json!({"risk_pct_per_trade": 0.01, "max_pos_pct": 0.20}),
            risk_gates: serde_json::json!({"max_concurrent_positions": 5}),
            broker_mode: broker_mode.into(),
        },
    )
    .await
    .expect("create strategy")
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[test]
fn engine_submits_one_bracket_on_uptrend_signal() {
    run(async {
        let pool = pool();
        let user = fresh_user_in(&pool).await;
        let strategy = make_strategy(&pool, user, "internal_sim").await;
        let run = algo::start_run(&pool, strategy.id).await.expect("run");
        let bars = fresh_long_window("AAPL");

        let sink = InMemorySink::default();
        let signal_at = drive_engine_until_signal(
            &pool, &sink, &strategy, run.id, &bars, 100_000.0,
        )
        .await
        .expect("uptrend produced at least one signal");

        // Exactly one order on the captured side.
        let submitted = sink.submitted.lock().unwrap();
        assert_eq!(submitted.len(), 1, "exactly one bracket per uptrend");
        let intent = &submitted[0];
        assert_eq!(intent.symbol, "AAPL");
        assert_eq!(intent.side, traderview_core::momentum_strategy::Side::Buy);
        assert!(intent.qty > Decimal::ZERO);
        assert!(intent.stop_price < intent.entry_price);
        assert!(intent.take_profit_price > intent.entry_price);

        // Persisted: order in algo_orders, accepted status, broker_order_id set.
        let orders = algo::list_orders(&pool, run.id, 10).await.expect("list");
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].symbol, "AAPL");
        assert_eq!(orders[0].order_class, "bracket");
        // InMemorySink now also returns an immediate_fill payload to
        // exercise the executions-pipeline path, so the order lands as
        // 'filled' rather than the bare 'accepted' it used to.
        assert_eq!(orders[0].status, "filled");
        assert!(orders[0].broker_order_id.is_some());

        // Run counters updated.
        let runs = algo::list_runs(&pool, strategy.id, 10).await.expect("runs");
        assert!(runs[0].signals_emitted >= 1);
        assert!(runs[0].orders_submitted >= 1);

        // Bonus: confirms our fixture's first signal lands at a sensible index.
        assert!(signal_at >= 30 && signal_at <= bars.len());
    });
}

#[test]
fn engine_refuses_when_kill_switch_engaged() {
    run(async {
        let pool = pool();
        let user = fresh_user_in(&pool).await;
        let strategy = make_strategy(&pool, user, "internal_sim").await;
        let run = algo::start_run(&pool, strategy.id).await.expect("run");

        algo::set_kill_switch(&pool, user, strategy.id, true, Some("test halt".into()))
            .await
            .expect("kill");
        let killed = algo::get_strategy(&pool, user, strategy.id)
            .await
            .expect("get")
            .expect("present");

        let bars = fresh_long_window("AAPL");
        let sink = InMemorySink::default();
        let err = process_bar_window(&pool, &sink, &killed, run.id, &bars, 100_000.0, 0)
            .await
            .expect_err("must refuse");
        assert!(matches!(err, EngineError::KillSwitch { .. }), "got {err:?}");
        assert!(sink.submitted.lock().unwrap().is_empty());
    });
}

#[test]
fn engine_refuses_when_position_cap_reached() {
    run(async {
        let pool = pool();
        let user = fresh_user_in(&pool).await;
        let strategy = make_strategy(&pool, user, "internal_sim").await;
        let run = algo::start_run(&pool, strategy.id).await.expect("run");
        let bars = fresh_long_window("AAPL");
        let sink = InMemorySink::default();
        // strategy's max_concurrent_positions = 5; report 5 already open.
        let err = process_bar_window(&pool, &sink, &strategy, run.id, &bars, 100_000.0, 5)
            .await
            .expect_err("must cap");
        assert!(matches!(err, EngineError::PositionCap(5)), "got {err:?}");
        assert!(sink.submitted.lock().unwrap().is_empty());
    });
}

#[test]
fn engine_refuses_alpaca_live_inside_paper_lock_window() {
    run(async {
        let pool = pool();
        let user = fresh_user_in(&pool).await;
        // Strategy is created with broker_mode=alpaca_live BUT paper_locked_until
        // defaults to now() + 30 days at insert.
        let mut s = make_strategy(&pool, user, "alpaca_live").await;
        s.paper_locked_until = Utc::now() + ChronoDuration::days(30);
        let run = algo::start_run(&pool, s.id).await.expect("run");
        let bars = fresh_long_window("AAPL");
        let sink = InMemorySink::default();
        let err = process_bar_window(&pool, &sink, &s, run.id, &bars, 100_000.0, 0)
            .await
            .expect_err("must paper-lock");
        assert!(matches!(err, EngineError::PaperLocked(_)), "got {err:?}");
        assert!(sink.submitted.lock().unwrap().is_empty());
    });
}

/// Stress: a sink that errors out — engine must still persist the order
/// row with status=rejected and a recorded error, then propagate.
#[test]
fn engine_records_rejection_when_sink_errors() {
    struct BrokenSink;
    impl BrokerSink for BrokenSink {
        fn submit_bracket(
            &self,
            _intent: OrderIntent,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>>
        {
            Box::pin(async { Err(EngineError::Broker("simulated".into())) })
        }
    }

    run(async {
        let pool = pool();
        let user = fresh_user_in(&pool).await;
        let strategy = make_strategy(&pool, user, "internal_sim").await;
        let run = algo::start_run(&pool, strategy.id).await.expect("run");
        let bars = fresh_long_window("AAPL");
        let broken = BrokenSink;
        // Drive forward — eventually a signal fires.
        let mut hit_error = false;
        for end in 30..=bars.len() {
            match process_bar_window(&pool, &broken, &strategy, run.id, &bars[..end], 100_000.0, 0)
                .await
            {
                Err(EngineError::Broker(_)) => { hit_error = true; break; }
                _ => continue,
            }
        }
        assert!(hit_error, "broken sink must surface a Broker error");
        let orders = algo::list_orders(&pool, run.id, 10).await.expect("list");
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].status, "rejected");
        assert!(orders[0].error.is_some());
    });
}

/// When a strategy is bound to an account_id, the engine routes every
/// fill through executions::insert_manual + trades::rollup_account so
/// the standard trade pipeline materializes the position. This is the
/// regression test for commit 12 — proves algo activity surfaces in the
/// same tables the dashboards already read.
#[test]
fn engine_fill_lands_in_executions_and_rolls_up_to_trades() {
    run(async {
        let pool = pool();
        let user = fresh_user_in(&pool).await;
        // make_strategy now auto-binds a real account_id (NOT NULL since
        // migration 0056). Capture it for the executions/trades query.
        let strategy = make_strategy(&pool, user, "internal_sim").await;
        let account_id = strategy.account_id;
        let run = algo::start_run(&pool, strategy.id).await.expect("run");
        let bars = fresh_long_window("AAPL");
        let sink = InMemorySink::default();
        let _ = drive_engine_until_signal(&pool, &sink, &strategy, run.id, &bars, 100_000.0)
            .await
            .expect("uptrend produced signal");

        let (exec_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM executions WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_one(&pool)
        .await
        .expect("count executions");
        assert_eq!(exec_count, 1, "exactly one executions row per algo fill");

        let (trade_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM trades WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_one(&pool)
        .await
        .expect("count trades");
        assert!(
            trade_count >= 1,
            "trades::rollup_account must materialize at least one trade row"
        );

        // algo_fills still gets the audit row independently of the pipeline.
        let orders = algo::list_orders(&pool, run.id, 10).await.expect("orders");
        let fills = algo::list_fills(&pool, orders[0].id).await.expect("fills");
        assert_eq!(fills.len(), 1, "algo_fills audit row present");
    });
}
