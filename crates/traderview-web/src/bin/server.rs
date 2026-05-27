//! Standalone web server. External Postgres, multi-user.
//!
//! Env vars:
//!   DATABASE_URL              postgres://user:pass@host/db   (required)
//!   TRADERVIEW_JWT_SECRET     hex-encoded, >= 32 bytes       (required)
//!   TRADERVIEW_BIND           default 0.0.0.0:8080
//!   TRADERVIEW_STATIC_DIR     default ../../frontend         (relative to bin)

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use traderview_web::{router, AppMode, AppState};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, env = "TRADERVIEW_JWT_SECRET")]
    jwt_secret: String,

    #[arg(long, env = "TRADERVIEW_BIND", default_value = "0.0.0.0:8080")]
    bind: SocketAddr,

    #[arg(long, env = "TRADERVIEW_STATIC_DIR", default_value = "frontend")]
    static_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "traderview_web=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();
    let jwt_secret = hex::decode(&args.jwt_secret)
        .map_err(|e| anyhow::anyhow!("TRADERVIEW_JWT_SECRET must be hex: {e}"))?;
    if jwt_secret.len() < 32 {
        anyhow::bail!("TRADERVIEW_JWT_SECRET must decode to >= 32 bytes");
    }

    let pool = traderview_db::connect_external(&args.database_url).await?;
    traderview_db::migrate(&pool).await?;

    let state = AppState::new(pool.clone(), AppMode::Web, jwt_secret);

    // Background disclosure poller — every 20s for sub-30s EDGAR/Congress alerts.
    {
        let pool = pool.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            loop {
                let r = traderview_db::disclosures::poll_all(&pool).await;
                let total = r.edgar_inserted + r.senate_inserted + r.house_inserted;
                if total > 0 {
                    tracing::info!(
                        edgar = r.edgar_inserted, senate = r.senate_inserted, house = r.house_inserted,
                        "disclosures polled",
                    );
                    if r.edgar_inserted > 0 {
                        hub.publish(traderview_web::realtime::Event::Disclosure {
                            source: "edgar", inserted: r.edgar_inserted,
                        });
                    }
                    if r.senate_inserted > 0 {
                        hub.publish(traderview_web::realtime::Event::Disclosure {
                            source: "senate", inserted: r.senate_inserted,
                        });
                    }
                    if r.house_inserted > 0 {
                        hub.publish(traderview_web::realtime::Event::Disclosure {
                            source: "house", inserted: r.house_inserted,
                        });
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            }
        });
    }

    // Background earnings calendar poller — every 6h over watchlist symbols.
    {
        let pool = pool.clone();
        tokio::spawn(async move {
            loop {
                match traderview_db::earnings_cal::poll_watchlists(&pool).await {
                    Ok(s) => if s.events_upserted > 0 || s.reactions_computed > 0 {
                        tracing::info!(
                            symbols = s.symbols_polled,
                            events = s.events_upserted,
                            reactions = s.reactions_computed,
                            "earnings polled",
                        );
                    },
                    Err(e) => tracing::warn!(error = %e, "earnings poll failed"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
            }
        });
    }

    // Background news poller — every 5 min over distinct watchlist symbols.
    {
        let pool = pool.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(s) = traderview_db::news::poll_watchlists(&pool).await {
                    if s.inserted > 0 {
                        tracing::info!(symbols = s.symbols_polled, inserted = s.inserted, "news polled");
                        hub.publish(traderview_web::realtime::Event::News {
                            inserted: s.inserted, symbols: s.symbols_polled,
                        });
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            }
        });
    }

    // Background sentiment poller — every 60s, WSB + StockTwits per watchlist symbol.
    {
        let pool = pool.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            loop {
                let (wsb, st) = traderview_db::sentiment::poll_all(&pool).await;
                if wsb + st > 0 {
                    tracing::info!(wsb = wsb, stocktwits = st, "sentiment polled");
                    hub.publish(traderview_web::realtime::Event::Sentiment {
                        wsb, stocktwits: st,
                    });
                }
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }
    let api = router(state);

    let static_service = ServeDir::new(&args.static_dir).append_index_html_on_directories(true);
    let app = api.fallback_service(static_service);

    tracing::info!(%args.bind, static_dir = %args.static_dir.display(), "traderview-web listening");
    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
