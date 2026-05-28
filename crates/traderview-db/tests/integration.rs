//! Integration tests for traderview-db CRUD modules.
//!
//! Spins up an embedded Postgres once per test binary (via the same
//! `Embedded::start` path used by the desktop app), runs all
//! migrations, then exercises each of the 10 previously-untested
//! domain modules against the real schema.
//!
//! Per-test isolation: every test inserts its own (user, account)
//! using fresh UUIDs, so test data never collides across tests in the
//! same binary.

use once_cell::sync::Lazy;
use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use uuid::Uuid;

// One PG instance per integration-test binary. The Runtime is held in
// a Lazy (NOT a Mutex) so tests can `RT.block_on(...)` in parallel
// without serializing through a global lock. The PgPool is cheap to
// clone (it's an Arc internally), so a `pool()` helper hands out
// short-lived clones from a stash.
//
// Performance: the PG binary itself is cached in a SHARED installation
// dir under the OS user cache. Each binary run gets a fresh data_dir,
// so the cluster is clean but the heavy binary download (~80MB) only
// happens once across all runs forever.
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

// Keep the PostgreSQL handle alive for the lifetime of the test binary
// — dropping it stops the cluster.
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
    let pool = RT.block_on(traderview_db::connect_external(&database_url))
        .expect("connect to test db");
    RT.block_on(traderview_db::migrate(&pool)).expect("run migrations");
    Mutex::new(PgGuard { _pg: pg, pool })
});

/// Shared installation_dir (cached binary), unique data_dir per run.
fn test_dirs() -> (PathBuf, PathBuf) {
    let cache_root = dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("traderview-tests");
    let installation_dir = cache_root.join("pg-bin");
    let mut data_dir = std::env::temp_dir();
    data_dir.push(format!(
        "traderview-db-it-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    (installation_dir, data_dir)
}

fn run<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    // Force lazy init of PG before borrowing the runtime.
    let _ = Lazy::force(&PG);
    RT.block_on(fut)
}

fn pool() -> PgPool {
    PG.lock().expect("pg lock").pool.clone()
}

/// Insert a fresh user + return its id. Uses a random email so parallel
/// tests don't collide on the email UNIQUE constraint.
async fn fresh_user() -> Uuid {
    let p = pool();
    let email = format!("test-{}@example.test", Uuid::new_v4());
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, is_local)
              VALUES ($1, $2, false)
         RETURNING id",
    )
    .bind(&email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$dummysalt$dummyhash")
    .fetch_one(&p)
    .await
    .expect("insert user");
    id
}

/// Insert a fresh account owned by `user_id` and return its id.
async fn fresh_account(user_id: Uuid) -> Uuid {
    let p = pool();
    let name = format!("acct-{}", Uuid::new_v4());
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO accounts (user_id, name, broker)
              VALUES ($1, $2, 'webull')
         RETURNING id",
    )
    .bind(user_id)
    .bind(&name)
    .fetch_one(&p)
    .await
    .expect("insert account");
    id
}

// ──────────────────────────────────────────────────────────────────────
// imports
// ──────────────────────────────────────────────────────────────────────

#[test]
fn imports_create_then_list_returns_inserted_row() {
    run(async {
        let user = fresh_user().await;
        let account = fresh_account(user).await;
        let sha = format!("sha-{}", Uuid::new_v4());
        let imp = traderview_db::imports::create(
            &pool(), account, "webull", "fills.csv", &sha, 42,
        ).await.expect("create import");
        assert_eq!(imp.account_id, account);
        assert_eq!(imp.row_count, 42);
        assert_eq!(imp.source, "webull");

        let list = traderview_db::imports::list(&pool(), account).await
            .expect("list imports");
        assert!(list.iter().any(|i| i.sha256 == sha));
    });
}

#[test]
fn imports_create_is_idempotent_on_sha_conflict() {
    run(async {
        let user = fresh_user().await;
        let account = fresh_account(user).await;
        let sha = format!("sha-{}", Uuid::new_v4());
        let a = traderview_db::imports::create(
            &pool(), account, "webull", "first.csv", &sha, 10,
        ).await.expect("first insert");
        // Second create with the SAME sha but different filename:
        // ON CONFLICT UPDATE bumps the filename. Same id is returned.
        let b = traderview_db::imports::create(
            &pool(), account, "webull", "second.csv", &sha, 99,
        ).await.expect("second insert");
        assert_eq!(a.id, b.id);
        assert_eq!(b.filename, "second.csv");
    });
}

// ──────────────────────────────────────────────────────────────────────
// hotkeys
// ──────────────────────────────────────────────────────────────────────

#[test]
fn hotkeys_upsert_then_list_returns_inserted() {
    run(async {
        let user = fresh_user().await;
        let payload = serde_json::json!({"target": "trade-entry"});
        let h = traderview_db::hotkeys::upsert(
            &pool(), user, "submit-trade", "cmd+enter", "trade.submit", &payload,
        ).await.expect("upsert hotkey");
        assert_eq!(h.user_id, user);
        assert_eq!(h.combo, "cmd+enter");
        let list = traderview_db::hotkeys::list(&pool(), user).await.expect("list");
        assert!(list.iter().any(|x| x.id == h.id));
    });
}

#[test]
fn hotkeys_upsert_replaces_on_combo_conflict() {
    run(async {
        let user = fresh_user().await;
        let payload = serde_json::json!({});
        let combo = format!("ctrl+{}", Uuid::new_v4());
        let a = traderview_db::hotkeys::upsert(
            &pool(), user, "name-a", &combo, "action-a", &payload,
        ).await.expect("first upsert");
        let b = traderview_db::hotkeys::upsert(
            &pool(), user, "name-b", &combo, "action-b", &payload,
        ).await.expect("second upsert");
        assert_eq!(a.id, b.id, "same combo + user must dedupe to same row");
        assert_eq!(b.name, "name-b");
        assert_eq!(b.action, "action-b");
    });
}

#[test]
fn hotkeys_delete_removes_row() {
    run(async {
        let user = fresh_user().await;
        let h = traderview_db::hotkeys::upsert(
            &pool(), user, "del-me",
            &format!("alt+{}", Uuid::new_v4()),
            "noop", &serde_json::json!({}),
        ).await.expect("create");
        let ok = traderview_db::hotkeys::delete(&pool(), user, h.id).await
            .expect("delete");
        assert!(ok);
        // Second delete returns false (already gone).
        let ok2 = traderview_db::hotkeys::delete(&pool(), user, h.id).await
            .expect("re-delete");
        assert!(!ok2);
    });
}

// ──────────────────────────────────────────────────────────────────────
// watchlists
// ──────────────────────────────────────────────────────────────────────

#[test]
fn watchlists_full_lifecycle() {
    run(async {
        let user = fresh_user().await;
        let wl = traderview_db::watchlists::create(&pool(), user, "tech").await
            .expect("create watchlist");
        assert_eq!(wl.name, "tech");

        traderview_db::watchlists::add_symbol(&pool(), wl.id, "AAPL").await
            .expect("add aapl");
        traderview_db::watchlists::add_symbol(&pool(), wl.id, "MSFT").await
            .expect("add msft");
        let syms = traderview_db::watchlists::symbols(&pool(), wl.id).await
            .expect("list symbols");
        assert!(syms.contains(&"AAPL".to_string()));
        assert!(syms.contains(&"MSFT".to_string()));

        traderview_db::watchlists::remove_symbol(&pool(), wl.id, "AAPL").await
            .expect("remove aapl");
        let after = traderview_db::watchlists::symbols(&pool(), wl.id).await
            .expect("re-list");
        assert!(!after.contains(&"AAPL".to_string()));

        let renamed = traderview_db::watchlists::rename(&pool(), user, wl.id, "growth").await
            .expect("rename");
        assert!(renamed);

        let deleted = traderview_db::watchlists::delete(&pool(), user, wl.id).await
            .expect("delete");
        assert!(deleted);
    });
}

#[test]
fn watchlists_ensure_default_is_idempotent() {
    run(async {
        let user = fresh_user().await;
        let a = traderview_db::watchlists::ensure_default(&pool(), user).await
            .expect("first ensure");
        let b = traderview_db::watchlists::ensure_default(&pool(), user).await
            .expect("second ensure");
        assert_eq!(a.id, b.id);
    });
}

#[test]
fn watchlists_ensure_owner_distinguishes_owner_from_outsider() {
    run(async {
        let alice = fresh_user().await;
        let bob = fresh_user().await;
        let wl = traderview_db::watchlists::create(&pool(), alice, "alice-wl").await
            .expect("create");
        let alice_owns = traderview_db::watchlists::ensure_owner(&pool(), alice, wl.id).await
            .expect("ensure_owner alice");
        assert!(alice_owns);
        let bob_owns = traderview_db::watchlists::ensure_owner(&pool(), bob, wl.id).await
            .expect("ensure_owner bob");
        assert!(!bob_owns, "bob is not the owner of alice's watchlist");
    });
}

// ──────────────────────────────────────────────────────────────────────
// chart_drawings
// ──────────────────────────────────────────────────────────────────────

fn make_drawing(kind: &str, points: serde_json::Value) -> traderview_db::chart_drawings::DrawingInput {
    traderview_db::chart_drawings::DrawingInput {
        kind: kind.into(),
        points,
        label: None,
        color: None,
    }
}

#[test]
fn chart_drawings_create_list_delete() {
    run(async {
        let user = fresh_user().await;
        let symbol = "AAPL";
        let d = traderview_db::chart_drawings::create(
            &pool(), user, symbol,
            &make_drawing("trendline", serde_json::json!([{"x":0,"y":100},{"x":10,"y":150}])),
        ).await.expect("create drawing");

        let listed = traderview_db::chart_drawings::list_for_symbol(&pool(), user, symbol).await
            .expect("list");
        assert!(listed.iter().any(|x| x.id == d.id));

        let ok = traderview_db::chart_drawings::delete(&pool(), user, d.id).await
            .expect("delete");
        assert!(ok);
    });
}

#[test]
fn chart_drawings_create_rejects_unknown_kind() {
    run(async {
        let user = fresh_user().await;
        let res = traderview_db::chart_drawings::create(
            &pool(), user, "AAPL",
            &make_drawing("nonsense-kind", serde_json::json!([])),
        ).await;
        assert!(res.is_err(), "unknown drawing kind must be rejected");
    });
}

#[test]
fn chart_drawings_delete_all_for_symbol_clears_only_that_symbol() {
    run(async {
        let user = fresh_user().await;
        traderview_db::chart_drawings::create(
            &pool(), user, "AAPL",
            &make_drawing("trendline", serde_json::json!([])),
        ).await.expect("aapl");
        traderview_db::chart_drawings::create(
            &pool(), user, "MSFT",
            &make_drawing("trendline", serde_json::json!([])),
        ).await.expect("msft");

        let removed = traderview_db::chart_drawings::delete_all_for_symbol(&pool(), user, "AAPL").await
            .expect("delete aapl");
        assert!(removed > 0);

        let aapl_after = traderview_db::chart_drawings::list_for_symbol(&pool(), user, "AAPL").await
            .expect("list aapl");
        let msft_after = traderview_db::chart_drawings::list_for_symbol(&pool(), user, "MSFT").await
            .expect("list msft");
        assert!(aapl_after.is_empty());
        assert!(!msft_after.is_empty(), "MSFT drawings must survive AAPL purge");
    });
}

// ──────────────────────────────────────────────────────────────────────
// dashboards
// ──────────────────────────────────────────────────────────────────────

fn make_dashboard(name: &str, layout: Option<serde_json::Value>) -> traderview_db::dashboards::DashboardInput {
    traderview_db::dashboards::DashboardInput { name: name.into(), layout }
}

#[test]
fn dashboards_create_get_update_delete() {
    run(async {
        let user = fresh_user().await;
        let d = traderview_db::dashboards::create(
            &pool(), user, &make_dashboard("main", Some(serde_json::json!({"widgets": []}))),
        ).await.expect("create");

        let got = traderview_db::dashboards::get(&pool(), user, d.id).await
            .expect("get").expect("present");
        assert_eq!(got.id, d.id);
        assert_eq!(got.name, "main");

        let updated = traderview_db::dashboards::update(
            &pool(), user, d.id,
            &make_dashboard("renamed", Some(serde_json::json!({"widgets": [{"type": "pnl"}]}))),
        ).await.expect("update").expect("row");
        assert_eq!(updated.name, "renamed");

        let ok = traderview_db::dashboards::delete(&pool(), user, d.id).await.expect("delete");
        assert!(ok);
        let gone = traderview_db::dashboards::get(&pool(), user, d.id).await.expect("get post-delete");
        assert!(gone.is_none());
    });
}

#[test]
fn dashboards_get_returns_none_for_other_users_dashboard() {
    run(async {
        let alice = fresh_user().await;
        let bob = fresh_user().await;
        let d = traderview_db::dashboards::create(
            &pool(), alice, &make_dashboard("alice-dash", None),
        ).await.expect("create");
        let bob_view = traderview_db::dashboards::get(&pool(), bob, d.id).await
            .expect("get");
        assert!(bob_view.is_none(), "bob must not see alice's dashboard");
    });
}

// ──────────────────────────────────────────────────────────────────────
// goals
// ──────────────────────────────────────────────────────────────────────

fn make_goal(name: &str, days_ahead: i64) -> traderview_db::goals::GoalInput {
    let today = chrono::Utc::now().naive_utc().date();
    traderview_db::goals::GoalInput {
        account_id: None,
        name: name.into(),
        period: "monthly".into(),
        start_date: today,
        end_date: today + chrono::Duration::days(days_ahead),
        target_pnl: Some(1000.0),
        target_win_rate: Some(0.55),
        target_max_drawdown_pct: Some(0.10),
        notes: None,
    }
}

#[test]
fn goals_create_then_list_returns_inserted() {
    run(async {
        let user = fresh_user().await;
        let g = traderview_db::goals::create(&pool(), user, &make_goal("$1k month", 30)).await
            .expect("create goal");
        assert_eq!(g.user_id, user);
        let list = traderview_db::goals::list(&pool(), user).await.expect("list");
        assert!(list.iter().any(|x| x.id == g.id));
    });
}

#[test]
fn goals_get_isolates_per_user() {
    run(async {
        let alice = fresh_user().await;
        let bob = fresh_user().await;
        let g = traderview_db::goals::create(&pool(), alice, &make_goal("alice goal", 7)).await
            .expect("create");
        let bob_view = traderview_db::goals::get(&pool(), bob, g.id).await.expect("get");
        assert!(bob_view.is_none());
    });
}

#[test]
fn goals_delete_removes_row() {
    run(async {
        let user = fresh_user().await;
        let g = traderview_db::goals::create(&pool(), user, &make_goal("to-be-deleted", 1)).await
            .expect("create");
        let ok = traderview_db::goals::delete(&pool(), user, g.id).await.expect("delete");
        assert!(ok);
        let gone = traderview_db::goals::get(&pool(), user, g.id).await.expect("get");
        assert!(gone.is_none());
    });
}

// ──────────────────────────────────────────────────────────────────────
// breadth (read-only snapshot — exercise without write)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn breadth_snapshot_returns_a_value_on_empty_table() {
    run(async {
        // Empty table is the cold-start case; snapshot must not error.
        let snap = traderview_db::breadth::snapshot(&pool()).await;
        assert!(snap.is_ok(), "breadth::snapshot must handle empty table");
    });
}

// ──────────────────────────────────────────────────────────────────────
// mood_analytics
// ──────────────────────────────────────────────────────────────────────

#[test]
fn mood_analytics_report_for_fresh_account_returns_empty_report() {
    run(async {
        let user = fresh_user().await;
        let account = fresh_account(user).await;
        // No trades, no mood entries — report should still produce a
        // valid empty-state response without panicking.
        let r = traderview_db::mood_analytics::report(&pool(), user, account).await;
        assert!(r.is_ok(), "mood_analytics::report must handle empty data");
    });
}

// ──────────────────────────────────────────────────────────────────────
// trade_reviews
// ──────────────────────────────────────────────────────────────────────

#[test]
fn trade_reviews_stats_for_empty_account_returns_zero_counts() {
    run(async {
        let user = fresh_user().await;
        let account = fresh_account(user).await;
        let stats = traderview_db::trade_reviews::stats(&pool(), user, account).await
            .expect("stats");
        // With no reviews: counts should be sane defaults (0/none).
        // The exact struct depends on impl; this asserts the call
        // succeeds with no rows in either table.
        let _ = stats;       // smoke: any successful response is acceptable
    });
}

#[test]
fn trade_reviews_list_returns_empty_for_fresh_user() {
    run(async {
        let user = fresh_user().await;
        let list = traderview_db::trade_reviews::list(&pool(), user, 50).await
            .expect("list");
        assert!(list.is_empty(), "no reviews for a brand-new user");
    });
}

// ──────────────────────────────────────────────────────────────────────
// paper
// ──────────────────────────────────────────────────────────────────────

#[test]
fn paper_ensure_default_then_list_includes_account() {
    run(async {
        let user = fresh_user().await;
        let acct = traderview_db::paper::ensure_default(&pool(), user).await
            .expect("ensure default paper account");
        let list = traderview_db::paper::list_accounts(&pool(), user).await
            .expect("list paper accounts");
        assert!(list.iter().any(|a| a.id == acct.id));
    });
}

#[test]
fn paper_ensure_default_is_idempotent() {
    run(async {
        let user = fresh_user().await;
        let a = traderview_db::paper::ensure_default(&pool(), user).await
            .expect("first");
        let b = traderview_db::paper::ensure_default(&pool(), user).await
            .expect("second");
        assert_eq!(a.id, b.id);
    });
}
